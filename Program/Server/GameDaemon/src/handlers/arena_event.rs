//! Doc 26: arena-event (GM) and arena dungeon recruit entry functions.
//! Build 381.8867.
//!
//! Zone tasks use `skills::queue`. Arena portal records, geo zones and
//! backend sends are opaque and go through `Services::arena_event`.
use super::party::string_body;
use super::skills::queue;
use super::*;

pub const ARENA_EVENT_COMMANDS: &[u16] = &[
    723, 726, 729, 732, 733, 736, 739, 742, 745, 748, 751, 754, 757, 959, 963,
];

/// Doc 26 shared fields.
#[derive(Clone, Debug, Default)]
pub struct ArenaEventWorld {
    /// `qword_140EE58A0` byte +792: arena dungeon recruit is enabled (959/963).
    pub recruit_enabled: bool,
    /// Daemon byte +4152: arena attraction of this daemon (1 duel, 2 dungeon ❓ names).
    pub attraction: i8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArenaEventTask {
    /// 723, kind 158, vtable 0x140BAC8D8. Target PC +144.
    TeleportTo { target: u64 },
    /// 726, 0xA8 bytes, base 0x14028DF50 type 15, vtable 0x140AE66D8.
    /// Init 0x140279630(task, 0, zone, pc, target).
    Recall { target: u64 },
    /// 729, 0xA8 bytes, vtable 0x140A91A48, init 0x14027FD10.
    /// DWORD +152 = geo zone ID (the adapter resolves it), words +156/+158/+160.
    WarpToPos { x: u16, y: u16, z: u16 },
    /// 732, DuelTask 0x14036D860, init 0x1402B8010 mode 2.
    DuelGameStart,
    /// 739, kind 159, vtable 0x140BC2450. Portal record +144.
    ReturnToArenaLobby { portal_key: u64 },
    /// 742, kind 328, vtable 0x140B99D08. Portal record +144.
    ReturnToArenaLobbyAllPc { portal_key: u64 },
    /// 745, kind 181, vtable 0x140BB94B8. Caster = target PC, requester +144.
    RetrievePcAsset { requester: u64 },
    /// 748, kind 182, vtable 0x140BC1A38. Byte +144.
    WarpToDuelRoundStartPcSpawn { flag: u8 },
    /// 751, 0xA0 bytes, vtable 0x140BC1A90, init 0x14027FB90. Byte +152 (1 standby, 4 finish).
    SetBattleRoyalState { state: u8 },
    /// 754, kind 331, vtable 0x140B96E28.
    InZonePcLoc,
    /// 757, kind 338, vtable 0x140BC1E08.
    BattleRoyalScore,
    /// 959 / 963: CheckArenaPortalUsableTask 0x14036D670, kind 112, vtable 0x140B9E890.
    /// Portal record +144, mode DWORD +152 (2 publish, 3 cancel).
    CheckPortal { portal_key: u64, mode: u32 },
}
impl From<ArenaEventTask> for SkillTaskKind {
    fn from(task: ArenaEventTask) -> Self {
        SkillTaskKind::ArenaEvent(task)
    }
}

/// Opaque server operations. Unknown is never approval.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArenaEventOperation {
    /// DataManager slot 87 virtual +168. Fail means no portal record.
    PortalExists { portal_key: u64 },
    /// Zone qword +16224 (geo zone). Fail means null.
    GeoZone { zone: u64 },
    /// Body-only send through daemon +3840 (0x1400D2EE0) to world `world`.
    Send {
        world: i16,
        command: u16,
        descriptor: &'static str,
        body: Vec<u8>,
    },
}

#[derive(Clone, Debug)]
pub struct MemoryArenaEvent {
    pub portal: Gate,
    pub geo_zone: Gate,
    pub send: Gate,
    pub calls: Vec<ArenaEventOperation>,
}
impl Default for MemoryArenaEvent {
    fn default() -> Self {
        Self {
            portal: Gate::Unknown,
            geo_zone: Gate::Unknown,
            send: Gate::Unknown,
            calls: Vec::new(),
        }
    }
}
impl MemoryArenaEvent {
    pub fn check(&mut self, operation: &ArenaEventOperation) -> Gate {
        self.calls.push(operation.clone());
        match operation {
            ArenaEventOperation::PortalExists { .. } => self.portal,
            ArenaEventOperation::GeoZone { .. } => self.geo_zone,
            ArenaEventOperation::Send { .. } => self.send,
        }
    }
}

pub(super) fn dispatch_arena_event(ctx: &mut Context<'_>, command: u16, req: &[u8]) -> Outcome {
    match command {
        723 => handle_arena_event_teleport_to(ctx, req),
        726 => handle_arena_event_recall(ctx, req),
        729 => handle_arena_event_warp_to_pos(ctx, req),
        732 => handle_arena_event_duel_game_start(ctx, req),
        733 => handle_arena_event_find_arena_session(ctx, req),
        736 => handle_arena_event_move_to_arena(ctx, req),
        739 => handle_arena_event_return_to_arena_lobby(ctx, req),
        742 => handle_arena_event_return_to_arena_lobby_all_pc(ctx, req),
        745 => handle_arena_event_retrieve_pc_asset(ctx, req),
        748 => handle_arena_event_warp_to_duel_round_start(ctx, req),
        751 => handle_arena_event_set_battle_royal_state(ctx, req),
        754 => handle_arena_event_in_zone_pc_loc(ctx, req),
        757 => handle_arena_event_battle_royal_score(ctx, req),
        959 => handle_publish_recruit_arena_dungeon_member(ctx, req),
        963 => handle_cancel_recruit_arena_dungeon_member(ctx, req),
        _ => unreachable!("membership checked"),
    }
}

fn unknown() -> Outcome {
    Outcome::unsupported("❓ arena event portal, geo zone or send adapter")
}
fn gate(ctx: &mut Context<'_>, operation: ArenaEventOperation) -> Option<bool> {
    match ctx.services.arena_event(&operation) {
        Gate::Pass => Some(true),
        Gate::Fail => Some(false),
        Gate::Unknown => None,
    }
}
/// PC byte +532 in 1..=7 (`CheckArenaEventPacketUsable`).
fn authorized(pc: &Pc) -> bool {
    (1..=7).contains(&pc.gm_grade)
}
/// cfg +237 ≠ 0 and cfg +248 ≠ 0: this daemon is an arena daemon.
fn in_arena(ctx: &Context<'_>) -> bool {
    ctx.world.party_config.arena_enabled && ctx.world.arena
}
fn task_for(command: u16, caster: u64, zone: u64, kind: ArenaEventTask) -> SkillTask {
    SkillTask {
        command,
        caster,
        summon: false,
        zone,
        kind: kind.into(),
    }
}
fn task(ctx: &Context<'_>, command: u16, zone: u64, kind: ArenaEventTask) -> SkillTask {
    task_for(command, ctx.pc().object_id, zone, kind)
}
/// Queue, then send an empty success reply (descriptor `h`, `asc_1408CFBE8`).
fn queue_reply(ctx: &mut Context<'_>, t: SkillTask, ok: u16, fail: Outcome) -> Outcome {
    let mut out = queue(ctx, t, fail);
    if out
        .changes
        .iter()
        .any(|c| matches!(c, StateChange::Queued(_)))
    {
        out.replies.push(Message {
            command: ok,
            body: Vec::new(),
        });
    }
    out
}
fn logged(mut out: Outcome, text: &'static str) -> Outcome {
    out.logs.push(text);
    out
}

/// parity: 723 26-arena-event-handlers.md#723--g_arenaeventteleporttoreqcmd--partial partial
pub fn handle_arena_event_teleport_to(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const F: u16 = 725;
    if !authorized(ctx.pc()) {
        return logged(
            Outcome::reason(F, 2),
            "arena event teleport: not authorized",
        );
    }
    let Some(name) = Reader(req).string(33) else {
        return logged(
            Outcome::reason(F, 2),
            "arena event teleport: decompose failed",
        );
    };
    let Some(target) = ctx.world.mentoring.by_name(&name, 6).map(|pc| pc.object_id) else {
        return logged(
            Outcome::reason(F, 1916),
            "arena event teleport: target is null",
        );
    };
    let Some(zone) = ctx.pc().zone else {
        return logged(
            Outcome::reason(F, 2),
            "arena event teleport: cannot find zone",
        );
    };
    let t = task(ctx, 723, zone, ArenaEventTask::TeleportTo { target });
    queue(ctx, t, Outcome::reason(F, 2))
}

/// parity: 726 26-arena-event-handlers.md#726--g_arenaeventrecallreqcmd--partial partial
pub fn handle_arena_event_recall(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const F: u16 = 728;
    if !in_arena(ctx) {
        return logged(Outcome::reason(F, 2), "arena event recall: not is arena");
    }
    if !authorized(ctx.pc()) {
        return logged(Outcome::reason(F, 2), "arena event recall: not authorized");
    }
    let Some(name) = Reader(req).string(33) else {
        return logged(
            Outcome::reason(F, 2),
            "arena event recall: decompose failed",
        );
    };
    let Some((target, target_zone)) = ctx
        .world
        .mentoring
        .by_name(&name, 6)
        .map(|pc| (pc.object_id, pc.zone))
    else {
        return logged(
            Outcome::reason(F, 1917),
            "arena event recall: failed to find pc",
        );
    };
    let Some(zone) = ctx.pc().zone else {
        return logged(Outcome::reason(F, 2), "arena event recall: zone is null");
    };
    if target_zone != Some(zone) {
        return logged(
            Outcome::reason(F, 1918),
            "arena event recall: different zone",
        );
    }
    let t = task(ctx, 726, zone, ArenaEventTask::Recall { target });
    queue_reply(ctx, t, 727, Outcome::reason(F, 2))
}

/// parity: 729 26-arena-event-handlers.md#729--g_arenaeventwarptoposreqcmd--partial partial
pub fn handle_arena_event_warp_to_pos(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const F: u16 = 731;
    if !in_arena(ctx) {
        return logged(Outcome::reason(F, 2), "arena event warp: not is arena");
    }
    if !authorized(ctx.pc()) {
        return logged(Outcome::reason(F, 2), "arena event warp: not authorized");
    }
    let mut r = Reader(req);
    let (Some(x), Some(y), Some(z)) = (r.short(), r.short(), r.short()) else {
        return logged(Outcome::reason(F, 2), "arena event warp: decompose failed");
    };
    let Some(zone) = ctx.pc().zone else {
        return logged(Outcome::reason(F, 2), "arena event warp: zone is null");
    };
    match gate(ctx, ArenaEventOperation::GeoZone { zone }) {
        Some(true) => {}
        Some(false) => return logged(Outcome::reason(F, 2), "arena event warp: geo zone is null"),
        None => return unknown(),
    }
    let t = task(ctx, 729, zone, ArenaEventTask::WarpToPos { x, y, z });
    queue_reply(ctx, t, 730, Outcome::reason(F, 2))
}

/// parity: 732 26-arena-event-handlers.md#732--g_arenaeventduelgamestartreqcmd--partial partial
pub fn handle_arena_event_duel_game_start(ctx: &mut Context<'_>, _req: &[u8]) -> Outcome {
    // Native quirk: every failure replies 282 (G_DuelReadyRoundAck), not 734.
    const F: u16 = 282;
    if !ctx.world.party_config.arena_enabled {
        return logged(
            Outcome::reason(F, 2),
            "arena event duel start: arena not enabled",
        );
    }
    if !ctx.world.arena {
        return logged(
            Outcome::reason(F, 2),
            "arena event duel start: not arena daemon",
        );
    }
    if ctx.world.arena_event.attraction != 1 {
        return logged(
            Outcome::reason(F, 2),
            "arena event duel start: invalid attraction",
        );
    }
    if !authorized(ctx.pc()) {
        return logged(
            Outcome::reason(F, 2),
            "arena event duel start: not authorized",
        );
    }
    let Some(zone) = ctx.pc().zone else {
        return logged(
            Outcome::reason(F, 2),
            "arena event duel start: cannot find zone",
        );
    };
    let t = task(ctx, 732, zone, ArenaEventTask::DuelGameStart);
    queue(ctx, t, Outcome::reason(F, 2))
}

fn send(
    ctx: &mut Context<'_>,
    command: u16,
    descriptor: &'static str,
    body: Vec<u8>,
) -> Option<bool> {
    let world = ctx.pc().short_96;
    gate(
        ctx,
        ArenaEventOperation::Send {
            world,
            command,
            descriptor,
            body,
        },
    )
}

/// parity: 733 26-arena-event-handlers.md#733--g_arenaeventfindarenasessionreqcmd--partial partial
pub fn handle_arena_event_find_arena_session(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const F: u16 = 735;
    if !ctx.world.party_config.arena_enabled {
        return logged(
            Outcome::reason(F, 1919),
            "arena event find session: arena not enabled",
        );
    }
    if ctx.world.arena {
        return logged(
            Outcome::reason(F, 5),
            "arena event find session: already in arena",
        );
    }
    if !authorized(ctx.pc()) {
        return logged(
            Outcome::reason(F, 2),
            "arena event find session: not authorized",
        );
    }
    let mut r = Reader(req);
    let (Some(kind), Some(name)) = (r.byte(), r.string(128)) else {
        return logged(
            Outcome::reason(F, 2),
            "arena event find session: decompose failed",
        );
    };
    // 0 → 1; 1 or 2 → 2; other → invalid.
    let attraction = match kind as i8 {
        0 => 1u8,
        1 | 2 => 2,
        _ => {
            return logged(
                Outcome::reason(F, 2),
                "arena event find session: invalid attraction",
            );
        }
    };
    let pc = ctx.pc();
    let mut body = pc.db_id.to_le_bytes().to_vec();
    body.extend(pc.object_id.to_le_bytes());
    body.push(attraction);
    string_body(&mut body, &name);
    match send(ctx, 140, "hqqQs", body) {
        Some(true) => Outcome::default(),
        Some(false) => logged(
            Outcome::reason(F, 1919),
            "arena event find session: arena disabled",
        ),
        None => unknown(),
    }
}

/// parity: 736 26-arena-event-handlers.md#736--g_arenaeventmovetoarenareqcmd--partial partial
pub fn handle_arena_event_move_to_arena(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const F: u16 = 738;
    if !ctx.world.party_config.arena_enabled {
        return logged(
            Outcome::reason(F, 1921),
            "arena event move: arena not enabled",
        );
    }
    if ctx.world.arena {
        return logged(Outcome::reason(F, 5), "arena event move: already in arena");
    }
    if !authorized(ctx.pc()) {
        return logged(Outcome::reason(F, 2), "arena event move: not authorized");
    }
    let Some(arena_id) = Reader(req).int().map(|v| v as i32) else {
        return logged(Outcome::reason(F, 2), "arena event move: decompose failed");
    };
    if arena_id <= 0 {
        return logged(Outcome::reason(F, 2), "arena event move: invalid arena id");
    }
    let pc = ctx.pc();
    let reason = if pc.arena_state != 3 {
        1922
    } else if pc.arena_qwords[0] == 0 {
        1923
    } else {
        0
    };
    if reason != 0 {
        return logged(Outcome::reason(F, reason), "arena event move: fail to send");
    }
    let mut body = pc.arena_lobby.route.to_le_bytes().to_vec();
    body.extend(pc.db_id.to_le_bytes());
    body.extend(pc.object_id.to_le_bytes());
    body.extend(pc.short_96.to_le_bytes());
    body.extend(arena_id.to_le_bytes());
    // ❓ The trailing `Q` argument of "hqqqhiQ" is not visible in the decompile.
    match send(ctx, 137, "hqqqhiQ", body) {
        Some(true) => Outcome::default(),
        Some(false) => logged(Outcome::reason(F, 1921), "arena event move: fail to send"),
        None => unknown(),
    }
}

fn portal(ctx: &mut Context<'_>, portal_key: u64) -> Option<bool> {
    gate(ctx, ArenaEventOperation::PortalExists { portal_key })
}

/// parity: 739 26-arena-event-handlers.md#739--g_arenaeventreturntoarenalobbyreqcmd--partial partial
pub fn handle_arena_event_return_to_arena_lobby(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const F: u16 = 741;
    if !ctx.world.party_config.arena_enabled {
        return logged(
            Outcome::reason(F, 1928),
            "arena event return: arena not enabled",
        );
    }
    if !ctx.world.arena {
        return logged(Outcome::reason(F, 1929), "arena event return: not in arena");
    }
    if !authorized(ctx.pc()) {
        return logged(Outcome::reason(F, 2), "arena event return: not authorized");
    }
    let Some(portal_key) = Reader(req).quad() else {
        return logged(
            Outcome::reason(F, 2),
            "arena event return: decompose failed",
        );
    };
    match portal(ctx, portal_key) {
        Some(true) => {}
        Some(false) => {
            return logged(
                Outcome::reason(F, 1927),
                "arena event return: invalid portal key",
            )
        }
        None => return unknown(),
    }
    let Some(zone) = ctx.pc().zone else {
        return logged(Outcome::reason(F, 2), "arena event return: zone is null");
    };
    let t = task(
        ctx,
        739,
        zone,
        ArenaEventTask::ReturnToArenaLobby { portal_key },
    );
    queue(ctx, t, Outcome::reason(F, 2))
}

/// parity: 742 26-arena-event-handlers.md#742--g_arenaeventreturntoarenalobbyallpcreqcmd--partial partial
pub fn handle_arena_event_return_to_arena_lobby_all_pc(
    ctx: &mut Context<'_>,
    req: &[u8],
) -> Outcome {
    const F: u16 = 744;
    if !in_arena(ctx) {
        return logged(
            Outcome::reason(F, 2),
            "arena event return all: not is arena",
        );
    }
    if !authorized(ctx.pc()) {
        return logged(
            Outcome::reason(F, 2),
            "arena event return all: not authorized",
        );
    }
    let Some(portal_key) = Reader(req).quad() else {
        return logged(
            Outcome::reason(F, 2),
            "arena event return all: decompose failed",
        );
    };
    match portal(ctx, portal_key) {
        Some(true) => {}
        Some(false) => {
            return logged(
                Outcome::reason(F, 2),
                "arena event return all: invalid portal key",
            )
        }
        None => return unknown(),
    }
    let Some(zone) = ctx.pc().zone else {
        return logged(
            Outcome::reason(F, 2),
            "arena event return all: zone is null",
        );
    };
    let t = task(
        ctx,
        742,
        zone,
        ArenaEventTask::ReturnToArenaLobbyAllPc { portal_key },
    );
    // Native quirk: the queue failure replies 741, not 744.
    queue_reply(ctx, t, 743, Outcome::reason(741, 2))
}

/// parity: 745 26-arena-event-handlers.md#745--g_arenaeventretreivepcassetreqcmd--partial partial
pub fn handle_arena_event_retrieve_pc_asset(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const F: u16 = 747;
    if !authorized(ctx.pc()) {
        return logged(
            Outcome::reason(F, 2),
            "arena event pc asset: not authorized",
        );
    }
    let Some(name) = Reader(req).string(33) else {
        return logged(
            Outcome::reason(F, 2),
            "arena event pc asset: decompose failed",
        );
    };
    let Some((target, target_zone)) = ctx
        .world
        .mentoring
        .by_name(&name, 6)
        .map(|pc| (pc.object_id, pc.zone))
    else {
        return logged(
            Outcome::reason(F, 1932),
            "arena event pc asset: cannot find target",
        );
    };
    let Some(zone) = target_zone else {
        return logged(Outcome::reason(F, 2), "arena event pc asset: zone is null");
    };
    let requester = ctx.pc().object_id;
    let t = task_for(
        745,
        target,
        zone,
        ArenaEventTask::RetrievePcAsset { requester },
    );
    queue(ctx, t, Outcome::reason(F, 2))
}

/// parity: 748 26-arena-event-handlers.md#748--g_arenaeventwarptoduelroundstartpcspawnreqcmd--partial partial
pub fn handle_arena_event_warp_to_duel_round_start(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const F: u16 = 750;
    if !ctx.world.party_config.arena_enabled {
        return logged(
            Outcome::reason(F, 1933),
            "arena event duel spawn: arena not enabled",
        );
    }
    if !ctx.world.arena {
        return logged(
            Outcome::reason(F, 1934),
            "arena event duel spawn: not in arena",
        );
    }
    if !authorized(ctx.pc()) {
        return logged(
            Outcome::reason(F, 2),
            "arena event duel spawn: not authorized",
        );
    }
    let Some(flag) = Reader(req).byte() else {
        return logged(
            Outcome::reason(F, 2),
            "arena event duel spawn: decompose failed",
        );
    };
    if flag > 1 {
        return logged(
            Outcome::reason(F, 2),
            "arena event duel spawn: invalid param",
        );
    }
    let Some(zone) = ctx.pc().zone else {
        return logged(
            Outcome::reason(F, 2),
            "arena event duel spawn: zone is null",
        );
    };
    let t = task(
        ctx,
        748,
        zone,
        ArenaEventTask::WarpToDuelRoundStartPcSpawn { flag },
    );
    queue(ctx, t, Outcome::reason(F, 2))
}

/// Case-insensitive ASCII compare, like `wcsicmp` for these literals.
fn eq_ignore_case(text: &[u16], literal: &str) -> bool {
    text.len() == literal.len()
        && text
            .iter()
            .zip(literal.bytes())
            .all(|(&a, b)| a < 0x80 && (a as u8).eq_ignore_ascii_case(&b))
}

/// parity: 751 26-arena-event-handlers.md#751--g_arenaeventsetbattleroyalstatereqcmd--partial partial
pub fn handle_arena_event_set_battle_royal_state(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const F: u16 = 753;
    if !in_arena(ctx) {
        return logged(
            Outcome::reason(F, 2),
            "arena event battle royal: not is arena",
        );
    }
    if !authorized(ctx.pc()) {
        return logged(
            Outcome::reason(F, 2),
            "arena event battle royal: not authorized",
        );
    }
    let Some(text) = Reader(req).string(128) else {
        return logged(
            Outcome::reason(F, 2),
            "arena event battle royal: decompose failed",
        );
    };
    let Some(zone) = ctx.pc().zone else {
        return logged(
            Outcome::reason(F, 2),
            "arena event battle royal: zone is null",
        );
    };
    let text: Vec<u16> = text.into_iter().take_while(|&c| c != 0).collect();
    let state = if eq_ignore_case(&text, "standby") {
        1
    } else if eq_ignore_case(&text, "finish") {
        4
    } else {
        return logged(
            Outcome::reason(F, 2),
            "arena event battle royal: failed to initialize",
        );
    };
    let t = task(
        ctx,
        751,
        zone,
        ArenaEventTask::SetBattleRoyalState { state },
    );
    queue_reply(ctx, t, 752, Outcome::reason(F, 2))
}

fn zone_event(ctx: &mut Context<'_>, command: u16, fail: u16, kind: ArenaEventTask) -> Outcome {
    if !in_arena(ctx) {
        return logged(Outcome::reason(fail, 2), "arena event: not is arena");
    }
    if !authorized(ctx.pc()) {
        return logged(Outcome::reason(fail, 2), "arena event: not authorized");
    }
    // Decode "h" only: an empty body cannot fail here.
    let Some(zone) = ctx.pc().zone else {
        return logged(Outcome::reason(fail, 2), "arena event: zone is null");
    };
    let t = task(ctx, command, zone, kind);
    queue(ctx, t, Outcome::reason(fail, 2))
}

/// parity: 754 26-arena-event-handlers.md#754--g_arenaeventinzonepclocreqcmd--partial partial
pub fn handle_arena_event_in_zone_pc_loc(ctx: &mut Context<'_>, _req: &[u8]) -> Outcome {
    zone_event(ctx, 754, 756, ArenaEventTask::InZonePcLoc)
}

/// parity: 757 26-arena-event-handlers.md#757--g_arenaeventbattleroyalscorereqcmd--partial partial
pub fn handle_arena_event_battle_royal_score(ctx: &mut Context<'_>, _req: &[u8]) -> Outcome {
    zone_event(ctx, 757, 759, ArenaEventTask::BattleRoyalScore)
}

fn recruit(ctx: &mut Context<'_>, req: &[u8], command: u16, fail: u16, mode: u32) -> Outcome {
    if !ctx.world.party_config.arena_enabled {
        return logged(
            Outcome::reason(fail, 2214),
            "arena recruit: arena not enabled",
        );
    }
    if !ctx.world.arena {
        return logged(Outcome::reason(fail, 2215), "arena recruit: not in arena");
    }
    if !ctx.world.arena_event.recruit_enabled {
        return logged(Outcome::reason(fail, 2213), "arena recruit: not in arena");
    }
    if ctx.world.arena_event.attraction != 2 {
        return logged(
            Outcome::reason(fail, 2214),
            "arena recruit: not dungeon attraction",
        );
    }
    let Some(portal_key) = Reader(req).quad() else {
        return logged(Outcome::reason(fail, 2), "arena recruit: decompose failed");
    };
    match portal(ctx, portal_key) {
        Some(true) => {}
        Some(false) => {
            return logged(
                Outcome::reason(fail, 2216),
                "arena recruit: invalid portal key",
            )
        }
        None => return unknown(),
    }
    let Some(zone) = ctx.pc().zone else {
        return logged(Outcome::reason(fail, 2), "arena recruit: zone is null");
    };
    let t = task(
        ctx,
        command,
        zone,
        ArenaEventTask::CheckPortal { portal_key, mode },
    );
    queue(ctx, t, Outcome::reason(fail, 2))
}

/// parity: 959 26-arena-event-handlers.md#959--g_publishrecruitarenadungeonmemberreqcmd--partial partial
pub fn handle_publish_recruit_arena_dungeon_member(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    recruit(ctx, req, 959, 962, 2)
}

/// parity: 963 26-arena-event-handlers.md#963--g_cancelrecruitarenadungeonmemberreqcmd--partial partial
pub fn handle_cancel_recruit_arena_dungeon_member(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    recruit(ctx, req, 963, 966, 3)
}
