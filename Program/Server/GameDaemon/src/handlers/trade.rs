//! Doc 12 part 1: profession stubs (121/123), PC trade (311–339) and account
//! inventory (793–805). Build 381.8867.
//!
//! Trade request handlers only gate and queue a task. `execute_trade_task`
//! runs the task body after the adapter dequeues it, on adapter-owned
//! `TradeState`. Escrow item moves, backend command 274 and the inventory
//! batches are boundaries: they stop before any state change.
use super::*;
use crate::validation::distance::{interaction, Interaction};

pub const ITEM_COMMANDS: &[u16] = &[
    ids::G_AcquireProfessionReqCmd,
    ids::G_DisposeProfessionReqCmd,
    ids::G_InviteTradeReqCmd,
    ids::G_JoinTradeReqCmd,
    ids::G_DeclineTradeReqCmd,
    ids::G_CancelTradeReqCmd,
    ids::G_AppendTradeSuggestionReqCmd,
    ids::G_RemoveTradeSuggestionReqCmd,
    ids::G_SuggestTradeMoneyReqCmd,
    ids::G_SubmitTradeReqCmd,
    ids::G_RetractTradeReqCmd,
    ids::G_ConfirmTradeReqCmd,
    ids::G_GetGoodsCountFromAccountInventoryReqCmd,
    ids::G_ListGoodsFromAccountInventoryReqCmd,
    ids::G_TakeGoodsFromAccountInventoryReqCmd,
    ids::G_RetrieveWarehouseLobbyServiceItemListReqCmd,
    ids::G_RetrieveWarehouseItemListReqCmd,
];

/// Decoded trade task fields. Native task kinds 46–53.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TradeTaskKind {
    /// Kind 46, +144 guest ID.
    Invite { guest: u64 },
    /// Kind 47, +152 inviter ID.
    Join { inviter: u64 },
    /// Kind 48, +152 inviter ID, +160 decline byte.
    Decline { inviter: u64, decline: u8 },
    /// Kind 49.
    Cancel,
    /// Kind 50, op 3: +192 slot, +194 count.
    Append { slot: u16, count: i16 },
    /// Kind 50, op 4: +196 escrow slot byte.
    Remove { slot: u8 },
    /// Kind 50, op 2: +184 offered money.
    Money { amount: i64 },
    /// Kind 51.
    Submit,
    /// Kind 52.
    Retract,
    /// Kind 53.
    Confirm,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TradeTask {
    pub command: u16,
    /// Task +128 caller object ID.
    pub caster: u64,
    pub zone: u64,
    pub kind: TradeTaskKind,
}

/// One escrow entry (inventory qword indices 166..173, slots 1..8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EscrowItem {
    /// Item native +16 ≠ 0.
    pub native_ok: bool,
}
/// PC fields the trade helpers read. The adapter keeps them current.
#[derive(Clone, Debug, Default)]
pub struct TradePlayer {
    /// PC +40.
    pub object_id: u64,
    /// PC +2224.
    pub zone: Option<u64>,
    /// Server signed-short x/y/z at +268/+270/+272.
    pub position: Position,
    /// PC byte +13768 = 1.
    pub t2_blocked: bool,
    /// PC byte +888 mask 1 set or byte +578 ≠ 0.
    pub busy: bool,
    /// User qword[984], qword[992]. Present only when PC +20696 holds a user
    /// and PC byte +20752 = 0.
    pub timed_window: Option<(i64, i64)>,
    /// PC +19648 trade object (adapter key).
    pub trade: Option<u64>,
    /// PC +312 signed money.
    pub money: i64,
    /// PC signed byte +276.
    pub level: i8,
    /// PC +12688 count, entries at +12696 (DWORD) / +12704 (active byte).
    pub restriction_entries: Vec<(u32, bool)>,
    /// `0x1405114E0(inventory, 2)` usable limit.
    pub usable_limit: i16,
    /// Inventory signed byte at PC +15096.
    pub reserved_15096: i8,
    /// Inventory signed short at PC +15082.
    pub used_15082: i16,
    /// Inventory short [80] (container-2 slot count, exclusive bound).
    pub bag_slots: u16,
    /// Escrow slots 1..8 (index 0 = slot 1).
    pub escrow: [Option<EscrowItem>; 8],
}
impl TradePlayer {
    /// `s16(usable_limit - s8[15096] - s16[15082])`, 16-bit arithmetic.
    pub fn free_slot_estimate(&self) -> i16 {
        self.usable_limit
            .wrapping_sub(i16::from(self.reserved_15096))
            .wrapping_sub(self.used_15082)
    }
    /// `0x140515DB0`: non-null escrow slots 1..8.
    pub fn escrow_count(&self) -> i16 {
        self.escrow.iter().flatten().count() as i16
    }
}
/// Trade object (0x918 bytes). Index 0 = inviter block (+32), 1 = guest (+144).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trade {
    pub inviter: u64,
    pub guest: u64,
    /// Participant states +40 / +152.
    pub states: [u8; 2],
    /// Offered money +48 / +160.
    pub offers: [i64; 2],
}
impl Trade {
    fn ids(&self) -> [u64; 2] {
        [self.inviter, self.guest]
    }
}
/// Table-manager slot 555 record for operation 4 (`0x140601580`).
#[derive(Clone, Copy, Debug, Default)]
pub struct RestrictionRecord {
    /// Byte +36 ≠ 0.
    pub active: bool,
    /// Signed short +16.
    pub level: i16,
    /// Integer +20.
    pub required: i32,
}
/// Adapter-owned trade state. Exclusive during one task call.
#[derive(Clone, Debug, Default)]
pub struct TradeState {
    pub players: BTreeMap<u64, TradePlayer>,
    pub trades: BTreeMap<u64, Trade>,
    pub next_trade: u64,
    /// `qword_140EE58A0.qword[24]`: largest money offer.
    pub offer_max: i64,
    /// `qword_140EE5418.qword[20224]`: money maximum.
    pub money_max: i64,
    /// Config byte +721 ≠ 0 (timed-user gate on).
    pub timed_gate: bool,
    /// Record selected by daemon byte +86857 (1..15, else 0). None passes.
    pub restriction: Option<RestrictionRecord>,
}
/// Ordered trade task effects. Apply in vector order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TradeEffect {
    Send {
        to: u64,
        message: Message,
    },
    /// `0x1403E3E80`: property 109 change. Reason 551 (debit) / 151 (credit).
    MoneyChanged {
        pc: u64,
        delta: i64,
        reason: u16,
    },
    /// `0x1403E3B40`: both links cleared, trade released.
    Unlinked {
        trade: u64,
    },
}

/// Account-inventory transaction (doc 12 common rules).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccountTransactionKind {
    /// 793: event 57, backend command 4.
    GoodsCount,
    /// 796: event 58, backend command 7.
    ListGoods { selector: i8 },
    /// 799: event 59, backend command 10.
    TakeGoods { goods: u64, selector: u8 },
    /// 802: event 60, backend command 13.
    LobbyServiceList { selector: u8, token: u64 },
    /// 805: event 61, backend command 16.
    WarehouseList { selector: u8, token: u64, list: i8 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountTransaction {
    pub command: u16,
    pub db_id: u64,
    pub kind: AccountTransactionKind,
}

pub(super) fn dispatch_item(ctx: &mut Context<'_>, command: u16, req: &[u8]) -> Outcome {
    match command {
        ids::G_AcquireProfessionReqCmd | ids::G_DisposeProfessionReqCmd => {
            handle_profession_stub(ctx, req)
        }
        ids::G_InviteTradeReqCmd => handle_invite_trade(ctx, req),
        ids::G_JoinTradeReqCmd => handle_join_trade(ctx, req),
        ids::G_DeclineTradeReqCmd => handle_decline_trade(ctx, req),
        ids::G_CancelTradeReqCmd => handle_cancel_trade(ctx, req),
        ids::G_AppendTradeSuggestionReqCmd => handle_append_trade_suggestion(ctx, req),
        ids::G_RemoveTradeSuggestionReqCmd => handle_remove_trade_suggestion(ctx, req),
        ids::G_SuggestTradeMoneyReqCmd => handle_suggest_trade_money(ctx, req),
        ids::G_SubmitTradeReqCmd => handle_submit_trade(ctx, req),
        ids::G_RetractTradeReqCmd => handle_retract_trade(ctx, req),
        ids::G_ConfirmTradeReqCmd => handle_confirm_trade(ctx, req),
        ids::G_GetGoodsCountFromAccountInventoryReqCmd => handle_get_goods_count(ctx, req),
        ids::G_ListGoodsFromAccountInventoryReqCmd => handle_list_goods(ctx, req),
        ids::G_TakeGoodsFromAccountInventoryReqCmd => handle_take_goods(ctx, req),
        ids::G_RetrieveWarehouseLobbyServiceItemListReqCmd => {
            handle_retrieve_lobby_service_list(ctx, req)
        }
        ids::G_RetrieveWarehouseItemListReqCmd => handle_retrieve_warehouse_list(ctx, req),
        _ => unreachable!("membership checked"),
    }
}

/// parity: 121 12-item-handlers.md#command-121--g_acquireprofessionreqcmd--done ported
/// parity: 123 12-item-handlers.md#command-123--g_disposeprofessionreqcmd--done ported
/// `0x14001AFE0` returns true. It reads nothing and does nothing.
pub fn handle_profession_stub(_: &mut Context<'_>, _: &[u8]) -> Outcome {
    Outcome::default()
}

/// T1: arena config or PC +37520 state 2/3/5/6 → 5. T2: PC +13768 → 7.
fn trade_gates(ctx: &Context<'_>, reply: u16, t2: bool) -> Option<Outcome> {
    if ctx.world.arena || session::arena_blocked(ctx.pc()) {
        return Some(Outcome::reason(reply, 5));
    }
    if t2 && ctx.pc().t2_blocked {
        return Some(Outcome::reason(reply, 7));
    }
    None
}
/// Zone null → 2 (with the assert log). Queue fail → 2.
fn trade_queue(ctx: &mut Context<'_>, reply: u16, command: u16, kind: TradeTaskKind) -> Outcome {
    let pc = ctx.pc().clone();
    let Some(zone) = pc.zone else {
        let mut out = Outcome::reason(reply, 2);
        out.logs.push("trade: PC zone is null");
        return out;
    };
    let task = TradeTask {
        command,
        caster: pc.object_id,
        zone,
        kind,
    };
    match ctx.services.queue_trade(&task) {
        Gate::Pass => Outcome {
            changes: vec![
                StateChange::Queued(command),
                StateChange::TradeTaskQueued(task),
            ],
            ..Outcome::default()
        },
        Gate::Fail => Outcome::reason(reply, 2),
        Gate::Unknown => Outcome::unsupported("❓ trade task allocation/queue adapter"),
    }
}
/// The trade handlers do not test the decoder result. The native value of a
/// field the decoder did not write is not proven, so a short body stops here.
const UNCHECKED_DECODE: &str = "❓ unchecked trade decode (short body)";

/// parity: 311 12-item-handlers.md#command-311--g_invitetradereqcmd partial
pub fn handle_invite_trade(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(out) = trade_gates(ctx, 312, true) {
        return out;
    }
    let Some(guest) = Reader(req).quad() else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    trade_queue(ctx, 312, 311, TradeTaskKind::Invite { guest })
}
/// parity: 314 12-item-handlers.md#command-314--g_jointradereqcmd partial
pub fn handle_join_trade(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(out) = trade_gates(ctx, 315, true) {
        return out;
    }
    let Some(inviter) = Reader(req).quad() else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    trade_queue(ctx, 315, 314, TradeTaskKind::Join { inviter })
}
/// parity: 317 12-item-handlers.md#command-317--g_declinetradereqcmd partial
pub fn handle_decline_trade(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(out) = trade_gates(ctx, 318, true) {
        return out;
    }
    let mut r = Reader(req);
    let (Some(inviter), Some(decline)) = (r.quad(), r.byte()) else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    trade_queue(ctx, 318, 317, TradeTaskKind::Decline { inviter, decline })
}
/// parity: 320 12-item-handlers.md#command-320--g_canceltradereqcmd partial
pub fn handle_cancel_trade(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if let Some(out) = trade_gates(ctx, 321, true) {
        return out;
    }
    trade_queue(ctx, 321, 320, TradeTaskKind::Cancel)
}
/// parity: 324 12-item-handlers.md#command-324--g_appendtradesuggestionreqcmd partial
pub fn handle_append_trade_suggestion(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(out) = trade_gates(ctx, 323, true) {
        return out;
    }
    let mut r = Reader(req);
    let (Some(slot), Some(count)) = (r.short(), r.short()) else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    let count = count as i16;
    trade_queue(ctx, 323, 324, TradeTaskKind::Append { slot, count })
}
/// parity: 327 12-item-handlers.md#command-327--g_removetradesuggestionreqcmd partial
pub fn handle_remove_trade_suggestion(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(out) = trade_gates(ctx, 323, true) {
        return out;
    }
    let Some(slot) = Reader(req).byte() else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    trade_queue(ctx, 323, 327, TradeTaskKind::Remove { slot })
}
/// parity: 330 12-item-handlers.md#command-330--g_suggesttrademoneyreqcmd ported
pub fn handle_suggest_trade_money(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(out) = trade_gates(ctx, 323, true) {
        return out;
    }
    let Some(amount) = Reader(req).quad() else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    let amount = amount as i64;
    trade_queue(ctx, 323, 330, TradeTaskKind::Money { amount })
}
/// parity: 333 12-item-handlers.md#command-333--g_submittradereqcmd ported
pub fn handle_submit_trade(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if let Some(out) = trade_gates(ctx, 334, false) {
        return out;
    }
    trade_queue(ctx, 334, 333, TradeTaskKind::Submit)
}
/// parity: 336 12-item-handlers.md#command-336--g_retracttradereqcmd ported
pub fn handle_retract_trade(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if let Some(out) = trade_gates(ctx, 337, false) {
        return out;
    }
    trade_queue(ctx, 337, 336, TradeTaskKind::Retract)
}
/// parity: 339 12-item-handlers.md#command-339--g_confirmtradereqcmd partial
pub fn handle_confirm_trade(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if let Some(out) = trade_gates(ctx, 340, false) {
        return out;
    }
    trade_queue(ctx, 340, 339, TradeTaskKind::Confirm)
}

// ---- Task bodies -------------------------------------------------------

fn send(out: &mut Outcome, to: u64, command: u16, body: Vec<u8>) {
    out.trade_effects.push(TradeEffect::Send {
        to,
        message: Message { command, body },
    });
}
fn send_both(out: &mut Outcome, trade: &Trade, command: u16, body: Vec<u8>) {
    for to in trade.ids() {
        send(out, to, command, body.clone());
    }
}
fn ids_body(a: u64, b: u64) -> Vec<u8> {
    [a.to_le_bytes(), b.to_le_bytes()].concat()
}
/// `hqQ`: object ID, one-byte `Q`.
fn id_q_body(id: u64, q: u8) -> Vec<u8> {
    let mut body = id.to_le_bytes().to_vec();
    body.push(q);
    body
}
/// Object ID bits 48..63 = 1 (PC kind).
fn pc_kind(id: u64) -> bool {
    id >> 48 == 1
}
fn timed_blocked(state: &TradeState, id: u64, now: i64) -> bool {
    state.timed_gate
        && state
            .players
            .get(&id)
            .and_then(|p| p.timed_window)
            .is_some_and(|(from, to)| from <= now && now <= to)
}
/// `0x140601580(pc, 4)`.
fn restriction_passes(record: Option<RestrictionRecord>, pc: &TradePlayer) -> bool {
    let Some(rec) = record else {
        return true;
    };
    if !rec.active || (rec.level == 0 && rec.required == 0) {
        return true;
    }
    if rec.level > 0 && i16::from(pc.level) >= rec.level {
        return true;
    }
    rec.required > 0
        && pc
            .restriction_entries
            .iter()
            .any(|(value, active)| *active && *value as i32 == rec.required)
}
/// Link invariant: caller.trade == trade, inviter and inviter.trade == trade,
/// guest present. Failure logs/asserts with no reply.
fn linked(state: &TradeState, caller: u64, trade_id: u64) -> Option<Trade> {
    let trade = state.trades.get(&trade_id)?.clone();
    let has = |id: u64| state.players.get(&id).map(|p| p.trade);
    if has(caller)? != Some(trade_id) || has(trade.inviter)? != Some(trade_id) {
        return None;
    }
    has(trade.guest)?;
    Some(trade)
}
/// `0x1403E3CA0`: caller.trade == trade, inviter.trade non-null, guest present.
/// Unlike automatic retract, suggestions do not require inviter.trade == trade.
fn suggestion_linked(state: &TradeState, caller: u64, trade_id: u64) -> Option<Trade> {
    let trade = state.trades.get(&trade_id)?.clone();
    if state.players.get(&caller)?.trade != Some(trade_id)
        || state.players.get(&trade.inviter)?.trade.is_none()
    {
        return None;
    }
    state.players.get(&trade.guest)?;
    Some(trade)
}
fn member(trade: &Trade, caller: u64) -> Option<usize> {
    trade.ids().iter().position(|id| *id == caller)
}
fn zone_of(state: &TradeState, id: u64) -> Option<u64> {
    state.players.get(&id).and_then(|p| p.zone)
}
/// Equal non-null participant zones.
fn same_zone(state: &TradeState, trade: &Trade) -> bool {
    let z = zone_of(state, trade.inviter);
    z.is_some() && z == zone_of(state, trade.guest)
}
fn invariant_log() -> Outcome {
    Outcome::log("trade: link invariant failed")
}
fn player(state: &mut TradeState, id: u64) -> &mut TradePlayer {
    state.players.get_mut(&id).expect("linked participant")
}
/// `0x1403E3B40`: clear both links, release the trade.
fn unlink(state: &mut TradeState, trade_id: u64, out: &mut Outcome) {
    if let Some(trade) = state.trades.remove(&trade_id) {
        for id in trade.ids() {
            if let Some(p) = state.players.get_mut(&id) {
                p.trade = None;
            }
        }
    }
    out.trade_effects
        .push(TradeEffect::Unlinked { trade: trade_id });
}

/// Run a dequeued trade task with its caster as the PC.
pub fn execute_trade_task(ctx: &mut Context<'_>, task: &TradeTask) -> Outcome {
    let now = ctx.world.utc_seconds;
    let Some(state) = ctx.services.trade_state() else {
        return Outcome::unsupported("❓ trade state adapter");
    };
    let caller = task.caster;
    if !state.players.contains_key(&caller) {
        return Outcome::unsupported("❓ task caster PC unavailable");
    }
    let caller_trade = state.players[&caller].trade;
    match task.kind {
        TradeTaskKind::Invite { guest } => invite(state, caller, guest, now),
        TradeTaskKind::Join { inviter } => join(state, caller, inviter, now),
        TradeTaskKind::Decline { inviter, decline } => {
            decline_trade(state, caller, inviter, decline)
        }
        TradeTaskKind::Cancel => match caller_trade {
            None => Outcome::reason(321, 920),
            Some(t) => cancel(state, caller, t),
        },
        TradeTaskKind::Money { .. }
        | TradeTaskKind::Append { .. }
        | TradeTaskKind::Remove { .. } => {
            let Some(t) = caller_trade else {
                return Outcome::reason(323, 924);
            };
            suggestion(state, caller, t, &task.kind)
        }
        TradeTaskKind::Submit => match caller_trade {
            None => Outcome::reason(321, 933),
            Some(t) => submit(state, caller, t, now),
        },
        TradeTaskKind::Retract => match caller_trade {
            None => Outcome::reason(321, 937),
            Some(t) => retract(state, caller, t),
        },
        TradeTaskKind::Confirm => match caller_trade {
            None => Outcome::reason(321, 940),
            Some(t) => confirm(state, caller, t, now),
        },
    }
}

/// `0x1403E4670` → `0x1403DE170`. Reply 312.
fn invite(state: &mut TradeState, caller: u64, guest: u64, now: i64) -> Outcome {
    const R: u16 = 312;
    let me = state.players[&caller].clone();
    let Some(zone) = me.zone else {
        return Outcome::log("trade: invite caller zone is null");
    };
    if !pc_kind(guest) || guest == caller {
        return Outcome::reason(R, 907);
    }
    // Resolution through caller.zone +16232: the adapter's player map.
    let Some(other) = state.players.get(&guest).cloned() else {
        return Outcome::reason(R, 907);
    };
    if other.zone != Some(zone) {
        return Outcome::reason(R, 904);
    }
    if other.t2_blocked {
        return Outcome::reason(R, 8);
    }
    if timed_blocked(state, caller, now) {
        return Outcome::reason(R, 2);
    }
    if timed_blocked(state, guest, now) {
        return Outcome::reason(R, 911);
    }
    if me.trade.is_some() {
        return Outcome::reason(R, 905);
    }
    if me.busy {
        return Outcome::reason(R, 906);
    }
    if other.trade.is_some() {
        return Outcome::reason(R, 909);
    }
    if other.busy {
        return Outcome::reason(R, 910);
    }
    if let Err(fail) = interaction(Interaction::TradeInvite, me.position, other.position) {
        return Outcome::reason(R, fail.reason as u16);
    }
    if !restriction_passes(state.restriction, &me) || !restriction_passes(state.restriction, &other)
    {
        return Outcome::reason(R, 2);
    }
    state.next_trade += 1;
    let id = state.next_trade;
    let trade = Trade {
        inviter: caller,
        guest,
        states: [0, 0],
        offers: [0, 0],
    };
    player(state, caller).trade = Some(id);
    player(state, guest).trade = Some(id);
    let mut out = Outcome::default();
    send_both(&mut out, &trade, 313, ids_body(caller, guest));
    state.trades.insert(id, trade);
    // ❓ Config +537 audit helper 0x140369F70 is not ported (no client effect).
    out
}

/// `0x1403E47A0` → `0x1403DEEA0`. Reply 315.
fn join(state: &mut TradeState, caller: u64, inviter: u64, now: i64) -> Outcome {
    const R: u16 = 315;
    if state.players[&caller].zone.is_none() {
        return Outcome::reason(R, 2);
    }
    if !pc_kind(inviter) {
        return Outcome::reason(R, 915);
    }
    // `0x140259CA0`: resolve the inviter in the caller zone.
    let zone = state.players[&caller].zone;
    let Some(host) = state.players.get(&inviter).filter(|p| p.zone == zone) else {
        return Outcome::reason(R, 915);
    };
    let Some(t) = host.trade else {
        return Outcome::reason(R, 916);
    };
    // The helper checks the caller link against the inviter's trade object.
    let Some(trade) = state.trades.get(&t).cloned() else {
        return invariant_log();
    };
    let inviter_linked = state.players.get(&trade.inviter).map(|p| p.trade) == Some(Some(t));
    if !inviter_linked || !state.players.contains_key(&trade.guest) {
        return invariant_log();
    }
    if trade.guest != caller {
        return Outcome::reason(R, 916);
    }
    if !same_zone(state, &trade) {
        return Outcome::reason(R, 912);
    }
    if timed_blocked(state, trade.inviter, now) || timed_blocked(state, trade.guest, now) {
        return Outcome::reason(R, 2);
    }
    if trade.states[0] != 0 {
        return Outcome::reason(R, 916);
    }
    if state.players[&trade.guest].busy {
        return Outcome::reason(R, 914);
    }
    let (a, b) = (
        state.players[&trade.inviter].position,
        state.players[&trade.guest].position,
    );
    if let Err(fail) = interaction(Interaction::TradeJoin, a, b) {
        return Outcome::reason(R, fail.reason as u16);
    }
    let entry = state.trades.get_mut(&t).expect("trade present");
    entry.states = [1, 1];
    let mut out = Outcome::default();
    send_both(&mut out, &trade, 316, ids_body(trade.inviter, trade.guest));
    out
}

/// `0x1403E4980` → `0x1403DFA20`. Reply 318.
fn decline_trade(state: &mut TradeState, caller: u64, inviter: u64, decline: u8) -> Outcome {
    const R: u16 = 318;
    if state.players[&caller].zone.is_none() {
        return Outcome::reason(R, 2);
    }
    if !pc_kind(inviter) {
        return Outcome::reason(R, 918);
    }
    // `0x140259CA0`: resolve the inviter in the caller zone.
    let zone = state.players[&caller].zone;
    let Some(host) = state.players.get(&inviter).filter(|p| p.zone == zone) else {
        return Outcome::reason(R, 918);
    };
    let Some(t) = host.trade else {
        return Outcome::reason(R, 919);
    };
    let Some(trade) = state.trades.get(&t).cloned() else {
        return invariant_log();
    };
    let inviter_linked = state.players.get(&trade.inviter).map(|p| p.trade) == Some(Some(t));
    if !inviter_linked || !state.players.contains_key(&trade.guest) {
        return invariant_log();
    }
    if trade.guest != caller || trade.states[0] != 0 {
        return Outcome::reason(R, 919);
    }
    let mut out = Outcome::default();
    let mut body = ids_body(trade.inviter, trade.guest);
    body.push(decline);
    send_both(&mut out, &trade, 319, body);
    unlink(state, t, &mut out);
    out
}

/// `0x1403E0230(trade, actor, byte)`: return escrow, send 322, unlink.
/// Escrow return is a boundary when either escrow holds an item.
fn cancel_with(
    state: &mut TradeState,
    trade_id: u64,
    actor: u64,
    byte: u8,
    out: &mut Outcome,
) -> bool {
    let trade = state.trades[&trade_id].clone();
    if trade
        .ids()
        .iter()
        .any(|id| state.players[id].escrow_count() > 0)
    {
        return false;
    }
    send_both(out, &trade, 322, id_q_body(actor, byte));
    unlink(state, trade_id, out);
    true
}
const ESCROW_RETURN: &str = "❓ escrow return 0x140515A80 / fallback 0x1405110B0";

/// `0x1403E4BB0` → `0x1403E0230(trade, caller, 1)`. Reply 321.
fn cancel(state: &mut TradeState, caller: u64, t: u64) -> Outcome {
    let Some(trade) = linked(state, caller, t) else {
        return invariant_log();
    };
    if member(&trade, caller).is_none() {
        return Outcome::reason(321, 920);
    }
    let mut out = Outcome::default();
    if !cancel_with(state, t, caller, 1, &mut out) {
        return Outcome::unsupported(ESCROW_RETURN);
    }
    out
}

/// `0x1403E2240`: automatic retract. Silent on invariant failure.
fn auto_retract(state: &mut TradeState, caller: u64, t: u64, out: &mut Outcome) {
    let Some(trade) = linked(state, caller, t) else {
        return;
    };
    let Some(me) = member(&trade, caller) else {
        return;
    };
    if !same_zone(state, &trade) {
        return;
    }
    if trade.states[me] == 1 && trade.states[1 - me] == 2 {
        state.trades.get_mut(&t).expect("trade").states = [1, 1];
        send_both(out, &trade, 338, id_q_body(caller, 1));
    }
}

/// `0x1403E4D20`: 324/327/330 runner. Reply 323.
fn suggestion(state: &mut TradeState, caller: u64, t: u64, kind: &TradeTaskKind) -> Outcome {
    const R: u16 = 323;
    // The money range check runs before the automatic retract.
    if let TradeTaskKind::Money { amount } = *kind {
        if !(0..=state.offer_max).contains(&amount) {
            return Outcome::reason(R, 930);
        }
    }
    let mut out = Outcome::default();
    auto_retract(state, caller, t, &mut out);
    // `0x1403E3CA0` then membership 924, zone 923 (pointer compare), state 925.
    let Some(trade) = suggestion_linked(state, caller, t) else {
        out.logs.push("trade: link invariant failed");
        return out;
    };
    let fail = |mut out: Outcome, reason: u16| {
        out.replies.push(Message {
            command: R,
            body: reason.to_le_bytes().to_vec(),
        });
        out
    };
    let Some(me) = member(&trade, caller) else {
        return fail(out, 924);
    };
    if zone_of(state, trade.inviter) != zone_of(state, trade.guest) {
        return fail(out, 923);
    }
    if trade.states[me] != 1 {
        return fail(out, 925);
    }
    let peer = trade.ids()[1 - me];
    match *kind {
        TradeTaskKind::Money { amount } => {
            let money = state.players[&caller].money;
            let headroom = state.money_max.wrapping_sub(state.players[&peer].money);
            if money < amount {
                return fail(out, 926);
            }
            if amount > headroom {
                return fail(out, 928);
            }
            state.trades.get_mut(&t).expect("trade").offers[me] = amount;
            send(&mut out, caller, 332, amount.to_le_bytes().to_vec());
            send(&mut out, peer, 331, ids_body(caller, amount as u64));
            out
        }
        TradeTaskKind::Append { slot, count } => {
            let receiver = &state.players[&peer];
            let mine = &state.players[&caller];
            if receiver.free_slot_estimate() < mine.escrow_count() + 1 {
                return fail(out, 927);
            }
            // `0x140515360`: count > 0, 0 < slot < bag slots, free escrow slot.
            if count <= 0 || slot == 0 || slot >= mine.bag_slots || mine.escrow_count() >= 8 {
                return fail(out, 926);
            }
            out.status =
                Status::Unsupported("❓ escrow insert 0x140515360 / eligibility 0x1405593A0");
            out
        }
        TradeTaskKind::Remove { slot } => {
            // `0x140515770`: slots 1..8, item present, native +16 ≠ 0.
            let ok = (1..=8).contains(&slot)
                && state.players[&caller].escrow[usize::from(slot) - 1]
                    .is_some_and(|i| i.native_ok);
            if !ok {
                return fail(out, 931);
            }
            out.status = Status::Unsupported(ESCROW_RETURN);
            out
        }
        _ => unreachable!("suggestion kinds only"),
    }
}

/// `0x1403E4EB0` → `0x1403E17A0`. Reply 334.
fn submit(state: &mut TradeState, caller: u64, t: u64, now: i64) -> Outcome {
    const R: u16 = 334;
    let Some(trade) = linked(state, caller, t) else {
        return invariant_log();
    };
    let Some(me) = member(&trade, caller) else {
        return Outcome::reason(R, 933);
    };
    if !same_zone(state, &trade) {
        return Outcome::reason(R, 932);
    }
    if timed_blocked(state, trade.inviter, now) || timed_blocked(state, trade.guest, now) {
        return Outcome::reason(R, 2);
    }
    if trade.states[me] != 1 {
        return Outcome::reason(R, 934);
    }
    if state.players[&caller].money < trade.offers[me] {
        return Outcome::reason(R, 935);
    }
    state.trades.get_mut(&t).expect("trade").states[me] = 2;
    let mut out = Outcome::default();
    send_both(&mut out, &trade, 335, caller.to_le_bytes().to_vec());
    out
}

/// `0x1403E4F30` → `0x1403E1DD0`. Reply 337.
fn retract(state: &mut TradeState, caller: u64, t: u64) -> Outcome {
    const R: u16 = 337;
    let Some(trade) = linked(state, caller, t) else {
        return invariant_log();
    };
    let Some(me) = member(&trade, caller) else {
        return Outcome::reason(R, 937);
    };
    if !same_zone(state, &trade) {
        return Outcome::reason(R, 936);
    }
    if trade.states[me] != 2 || trade.states[1 - me] == 3 {
        return Outcome::reason(R, 938);
    }
    state.trades.get_mut(&t).expect("trade").states = [1, 1];
    let mut out = Outcome::default();
    send_both(&mut out, &trade, 338, id_q_body(caller, 0));
    out
}

/// `0x1403E4FB0` → `0x1403E2550`. Reply 340.
fn confirm(state: &mut TradeState, caller: u64, t: u64, now: i64) -> Outcome {
    const R: u16 = 340;
    let Some(trade) = linked(state, caller, t) else {
        return invariant_log();
    };
    let Some(me) = member(&trade, caller) else {
        return Outcome::reason(R, 940);
    };
    if !same_zone(state, &trade) {
        return Outcome::reason(R, 939);
    }
    if timed_blocked(state, trade.inviter, now) || timed_blocked(state, trade.guest, now) {
        return Outcome::reason(R, 2);
    }
    let peer_state = trade.states[1 - me];
    if trade.states[me] != 2 || !matches!(peer_state, 2 | 3) {
        return Outcome::reason(R, 941);
    }
    let mut out = Outcome::default();
    if peer_state == 2 {
        state.trades.get_mut(&t).expect("trade").states[me] = 3;
        send_both(&mut out, &trade, 341, caller.to_le_bytes().to_vec());
        return out;
    }
    // Peer state 3: unlink flag set. Final checks in order.
    let (inv, gst) = (&state.players[&trade.inviter], &state.players[&trade.guest]);
    let all_ok = |p: &TradePlayer| p.escrow.iter().flatten().all(|i| i.native_ok);
    let failure = if inv.money < trade.offers[0] {
        Some((trade.inviter, 5))
    } else if gst.free_slot_estimate() < inv.escrow_count() {
        Some((trade.guest, 9))
    } else if trade.offers[0] > state.money_max.wrapping_sub(gst.money) {
        Some((trade.guest, 10))
    } else if !all_ok(inv) {
        Some((trade.inviter, 5))
    } else if gst.money < trade.offers[1] {
        Some((trade.guest, 5))
    } else if inv.free_slot_estimate() < gst.escrow_count() {
        Some((trade.inviter, 6))
    } else if trade.offers[1] > state.money_max.wrapping_sub(inv.money) {
        Some((trade.inviter, 7))
    } else if !all_ok(gst) {
        Some((trade.guest, 5))
    } else {
        None
    };
    if let Some((actor, byte)) = failure {
        // `cancel_with` unlinks on success; the task unlink is then a no-op.
        if !cancel_with(state, t, actor, byte, &mut out) {
            return Outcome::unsupported(ESCROW_RETURN);
        }
        return out;
    }
    if inv.escrow_count() > 0 || gst.escrow_count() > 0 {
        return Outcome::unsupported("❓ escrow transfer 0x140515960 / backend command 274");
    }
    // `0x1403E3E80` inviter → guest, then guest → inviter. Zero delta: no change.
    for (from, to, amount) in [
        (trade.inviter, trade.guest, trade.offers[0]),
        (trade.guest, trade.inviter, trade.offers[1]),
    ] {
        if amount == 0 {
            continue;
        }
        player(state, from).money -= amount;
        out.trade_effects.push(TradeEffect::MoneyChanged {
            pc: from,
            delta: amount.wrapping_neg(),
            reason: 551,
        });
        player(state, to).money += amount;
        out.trade_effects.push(TradeEffect::MoneyChanged {
            pc: to,
            delta: amount,
            reason: 151,
        });
    }
    send_both(&mut out, &trade, 341, caller.to_le_bytes().to_vec());
    unlink(state, t, &mut out);
    out
}

// ---- Account inventory -------------------------------------------------

/// Config +404 off → 2, then arena config (+237/+248) → 5.
fn account_gates(ctx: &Context<'_>) -> Option<u16> {
    if !ctx.world.account_inventory_enabled {
        return Some(2);
    }
    if ctx.world.arena {
        return Some(5);
    }
    None
}
fn no_user() -> Outcome {
    Outcome::log("account inventory: user is not found")
}
fn start_transaction(
    ctx: &mut Context<'_>,
    kind: AccountTransactionKind,
    command: u16,
    fail: Outcome,
) -> Outcome {
    let txn = AccountTransaction {
        command,
        db_id: ctx.pc().db_id,
        kind,
    };
    match ctx.services.account_transaction(&txn) {
        Gate::Pass => Outcome {
            changes: vec![StateChange::AccountTransactionStarted(txn)],
            ..Outcome::default()
        },
        Gate::Fail => fail,
        Gate::Unknown => Outcome::unsupported("❓ account transaction register 0x1400D4FE0"),
    }
}

/// parity: 793 12-item-handlers.md#command-793--g_getgoodscountfromaccountinventoryreqcmd partial
pub fn handle_get_goods_count(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if let Some(reason) = account_gates(ctx) {
        return Outcome::reason(795, reason);
    }
    if !ctx.pc().user_present {
        return no_user();
    }
    start_transaction(
        ctx,
        AccountTransactionKind::GoodsCount,
        793,
        Outcome::reason(795, 2),
    )
}
/// parity: 796 12-item-handlers.md#command-796--g_listgoodsfromaccountinventoryreqcmd partial
pub fn handle_list_goods(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(reason) = account_gates(ctx) {
        return Outcome::reason(798, reason);
    }
    let Some(selector) = Reader(req).byte().map(|b| b as i8).filter(|s| *s >= 0) else {
        return Outcome::reason(798, 2);
    };
    if !ctx.pc().user_present {
        return no_user();
    }
    let kind = AccountTransactionKind::ListGoods { selector };
    start_transaction(ctx, kind, 796, Outcome::reason(798, 2))
}
/// parity: 799 12-item-handlers.md#command-799--g_takegoodsfromaccountinventoryreqcmd partial
pub fn handle_take_goods(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(reason) = account_gates(ctx) {
        return Outcome::reason(801, reason);
    }
    if !ctx.pc().user_present {
        return no_user();
    }
    let mut r = Reader(req);
    let (Some(goods), Some(selector)) = (r.quad(), r.byte()) else {
        return Outcome::unsupported("❓ unchecked 799 decode (short body)");
    };
    let kind = AccountTransactionKind::TakeGoods { goods, selector };
    start_transaction(ctx, kind, 799, Outcome::reason(801, 2))
}
/// `hEQq` failure body: reason, selector byte, token.
fn echo(command: u16, reason: u16, selector: u8, token: u64) -> Outcome {
    let mut body = reason.to_le_bytes().to_vec();
    body.push(selector);
    body.extend(token.to_le_bytes());
    Outcome::reply(command, body)
}
/// parity: 802 12-item-handlers.md#command-802--g_retrievewarehouselobbyserviceitemlistreqcmd partial
pub fn handle_retrieve_lobby_service_list(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // Decode precedes the gates. `Q` is one byte (BloodLib codec).
    let mut r = Reader(req);
    let (Some(selector), Some(token)) = (r.byte(), r.quad()) else {
        return Outcome::unsupported("❓ unchecked 802 decode (short body)");
    };
    if let Some(reason) = account_gates(ctx) {
        return echo(804, reason, selector, token);
    }
    if !ctx.pc().user_present {
        return no_user();
    }
    let kind = AccountTransactionKind::LobbyServiceList { selector, token };
    start_transaction(ctx, kind, 802, echo(804, 2, selector, token))
}
/// parity: 805 12-item-handlers.md#command-805--g_retrievewarehouseitemlistreqcmd partial
pub fn handle_retrieve_warehouse_list(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let mut r = Reader(req);
    let (Some(selector), Some(token), Some(list)) = (r.byte(), r.quad(), r.byte()) else {
        return Outcome::unsupported("❓ unchecked 805 decode (short body)");
    };
    if let Some(reason) = account_gates(ctx) {
        return echo(807, reason, selector, token);
    }
    let list = list as i8;
    if list < 0 {
        return echo(807, 2, selector, token);
    }
    if !ctx.pc().user_present {
        return no_user();
    }
    let kind = AccountTransactionKind::WarehouseList {
        selector,
        token,
        list,
    };
    start_transaction(ctx, kind, 805, echo(807, 2, selector, token))
}
