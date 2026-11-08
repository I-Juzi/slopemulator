//! Doc 14 receive plans. Task execution is explicit and never implied by queueing.
use super::*;

pub const PARTY_COMMANDS: &[u16] = &[
    147, 151, 155, 157, 161, 163, 168, 170, 172, 174, 176, 179, 182, 202, 205, 208, 211, 214, 216,
    220, 224, 226, 230, 234, 239, 241, 243, 245, 247, 250, 253, 256, 259, 262, 424, 429,
];

pub(super) fn dispatch_party(ctx: &mut Context<'_>, command: u16, req: &[u8]) -> Outcome {
    match command {
        147 => handle_suggest_party(ctx, req),
        151 => handle_reject_suggest_party(ctx, req),
        155 => handle_create_party(ctx, req),
        157 => handle_invite_party(ctx, req),
        161 => handle_join_party(ctx, req),
        163 => handle_reject_party(ctx, req),
        168 => handle_leave_party(ctx, req),
        170 => handle_delegate_party_leader(ctx, req),
        172 => handle_banish_party(ctx, req),
        174 => handle_clear_logout_party_member(ctx, req),
        176 => handle_change_party_pickup_rule(ctx, req),
        179 => handle_change_party_difficulty(ctx, req),
        182 => handle_change_party_invite_auth(ctx, req),
        202 => handle_finish_party_auction(ctx, req),
        205 => handle_bid_current_auction_item(ctx, req),
        208 => handle_give_up_current_auction_item(ctx, req),
        211 => handle_distribution_current_auction_item(ctx, req),
        214 => handle_pass_party_auction(ctx, req),
        216 => handle_party_vote_cast(ctx, req),
        220 => handle_party_vote_cancel_cast(ctx, req),
        224 => handle_party_vote_reset(ctx, req),
        226 => handle_suggest_team(ctx, req),
        230 => handle_reject_suggest_team(ctx, req),
        234 => handle_invite_team(ctx, req),
        239 => handle_join_team(ctx, req),
        241 => handle_reject_team(ctx, req),
        243 => handle_leave_team(ctx, req),
        245 => handle_banish_team(ctx, req),
        247 => handle_team_member_party_change(ctx, req),
        250 => handle_delegate_team_leader(ctx, req),
        253 => handle_nominate_party_leader(ctx, req),
        256 => handle_change_team_invite_auth(ctx, req),
        259 => handle_change_team_party_difficulty(ctx, req),
        262 => handle_clear_logout_team_member(ctx, req),
        424 => handle_register_party_match(ctx, req),
        429 => handle_reject_party_match(ctx, req),
        _ => unreachable!("party command table"),
    }
}

/// Zero-initialized fields retain a decoded prefix on checked decode failure.
/// Unchecked native locals are never replaced with these defaults on a send.
struct Decode<'a> {
    reader: Reader<'a>,
    ok: bool,
}
impl<'a> Decode<'a> {
    fn new(req: &'a [u8]) -> Self {
        Self {
            reader: Reader(req),
            ok: true,
        }
    }
    fn read<T: Default>(&mut self, f: impl FnOnce(&mut Reader<'a>) -> Option<T>) -> T {
        if !self.ok {
            return T::default();
        }
        match f(&mut self.reader) {
            Some(v) => v,
            None => {
                self.ok = false;
                T::default()
            }
        }
    }
    fn byte(&mut self) -> u8 {
        self.read(|r| r.byte())
    }
    fn short(&mut self) -> u16 {
        self.read(|r| r.short())
    }
    fn int(&mut self) -> u32 {
        self.read(|r| r.int())
    }
    fn quad(&mut self) -> u64 {
        self.read(|r| r.quad())
    }
    fn name(&mut self) -> Vec<u16> {
        self.read(|r| r.string(4000))
    }
    fn bytes<const N: usize>(&mut self) -> [u8; N] {
        std::array::from_fn(|_| self.byte())
    }
}
pub(super) fn unchecked() -> Outcome {
    Outcome::unsupported("❓ unchecked native PARTY decode values")
}
pub(super) fn boundary(text: &'static str) -> Outcome {
    let mut out = Outcome::unsupported(text);
    out.party_effects.push(PartyEffect::Boundary(text));
    out
}
pub(super) fn string_body(body: &mut Vec<u8>, name: &[u16]) {
    // Native sends use the NUL-terminated prefix, not the consumed wire count.
    let name = name_prefix(name);
    body.extend((name.len() as u16).to_le_bytes());
    for unit in name {
        body.extend(unit.to_le_bytes());
    }
}
pub(super) fn send(out: &mut Outcome, to: u64, message: Message) {
    out.party_effects.push(PartyEffect::Send {
        to,
        message: message.clone(),
    });
    out.broadcasts.push(message);
}
pub(super) fn self_send(out: &mut Outcome, pc: &Pc, message: Message) {
    out.party_effects.push(PartyEffect::Send {
        to: pc.object_id,
        message: message.clone(),
    });
    out.replies.push(message);
}
pub(super) fn failure(command: u16, reason: u16) -> Message {
    Message {
        command,
        body: reason.to_le_bytes().to_vec(),
    }
}
pub(super) fn byte_failure(command: u16, reason: u16) -> Message {
    Message {
        command,
        body: vec![reason as u8],
    }
}
fn reply(message: Message) -> Outcome {
    Outcome::reply(message.command, message.body)
}
pub(super) fn queue_party(
    ctx: &mut Context<'_>,
    caster: &Pc,
    command: u16,
    kind: PartyTaskKind,
    error: Message,
) -> Outcome {
    let Some(zone) = caster.zone else {
        return reply(error);
    };
    if caster.native_kind != 1 {
        return reply(error);
    }
    let task = PartyTask {
        command,
        caster: caster.object_id,
        zone,
        kind,
    };
    match ctx.services.queue_party(&task) {
        Gate::Fail => reply(error),
        Gate::Unknown => boundary("❓ PARTY task allocation/attachment/queue adapter"),
        Gate::Pass => Outcome {
            changes: vec![
                StateChange::Queued(command),
                StateChange::PartyTaskQueued(task),
            ],
            ..Outcome::default()
        },
    }
}
fn self_task(ctx: &mut Context<'_>, command: u16, kind: PartyTaskKind, error: Message) -> Outcome {
    let pc = ctx.pc().clone();
    queue_party(ctx, &pc, command, kind, error)
}
pub(super) fn user_ok(pc: &Pc) -> bool {
    pc.user_available && !pc.party_user_blocked && pc.party_user_state == 0
}
pub(super) fn registration(ctx: &Context<'_>, out: &mut Outcome) {
    if !ctx.world.arena {
        if let Some(reg) = &ctx.pc().party_registration {
            out.daemon_requests.push(DaemonRequest::PartyRegistration {
                world: ctx.pc().world_id,
                route: reg.route,
                db_id: ctx.pc().db_id,
                value: reg.value,
                name: ctx.pc().name.clone(),
            });
        }
    }
}
fn target(ctx: &mut Context<'_>, id: u64, name: &[u16]) -> Result<Option<Pc>, &'static str> {
    let Some(state) = ctx.services.party_state() else {
        return Err("❓ world PARTY lookup adapter");
    };
    Ok(state.target(id, name).map(|peer| peer.pc.clone()))
}
fn special_zone(ctx: &Context<'_>, pc: &Pc) -> Result<bool, &'static str> {
    let Some(zone) = pc.zone else {
        return Ok(false);
    };
    let Some(zone) = ctx.world.zones.get(&zone) else {
        return Err("❓ missing server-owned PARTY zone metadata");
    };
    Ok(zone.party_category == 3 && matches!(zone.party_subtype, 26 | 34 | 38 | 77 | 177))
}
fn target_arena_blocked(ctx: &Context<'_>, pc: &Pc) -> bool {
    ctx.world.party_config.arena_enabled && !ctx.world.arena && session::arena_blocked(pc)
}

/// parity: 155 14-party-handlers.md#155--g_createpartyreqcmd--done ported
pub fn handle_create_party(_: &mut Context<'_>, _: &[u8]) -> Outcome {
    let mut out = Outcome::reason(156, 2);
    out.logs.push("party creation is unavailable");
    out
}
/// parity: 168 14-party-handlers.md#168--g_leavepartyreqcmd--partial partial
pub fn handle_leave_party(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    // ❓ Removal, replacement leader, notifications and backend work.
    if ctx.pc().zone.is_none() {
        return Outcome::reason(169, 764);
    }
    self_task(ctx, 168, PartyTaskKind::LeaveParty, failure(169, 2))
}
/// parity: 170 14-party-handlers.md#170--g_delegatepartyleaderreqcmd--partial partial
pub fn handle_delegate_party_leader(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Leader authorization/mutation/rollback and notifications.
    let Some(target) = Reader(req).quad() else {
        return unchecked();
    };
    self_task(
        ctx,
        170,
        PartyTaskKind::DelegateParty { target },
        failure(171, 2),
    )
}
/// parity: 172 14-party-handlers.md#172--g_banishpartyreqcmd--partial partial
pub fn handle_banish_party(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Deferred role/slot/removal/leader/backend paths.
    let Some(slot) = Reader(req).byte() else {
        return unchecked();
    };
    if ctx.world.arena {
        return Outcome::reason(173, 782);
    }
    self_task(
        ctx,
        172,
        PartyTaskKind::BanishParty { slot },
        failure(173, 2),
    )
}
/// parity: 176 14-party-handlers.md#176--g_changepartypickuprulereqcmd--partial partial
pub fn handle_change_party_pickup_rule(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Semantic field/config names, optional report sink and downstream effects.
    let mut r = Decode::new(req);
    let (rule, grade, target, money) = (r.byte(), r.byte(), r.quad(), r.quad());
    if !r.ok {
        return unchecked();
    }
    if rule > 3 {
        return Outcome::reason(178, 2);
    }
    self_task(
        ctx,
        176,
        PartyTaskKind::Pickup {
            rule,
            grade,
            target,
            money,
        },
        failure(178, 2),
    )
}
/// parity: 179 14-party-handlers.md#179--g_changepartydifficultytypereqcmd--partial partial
pub fn handle_change_party_difficulty(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Resizing, PC metadata sync, report sink and downstream work.
    if ctx.world.arena || session::arena_blocked(ctx.pc()) {
        return reply(byte_failure(181, 5));
    }
    let Some(difficulty) = Reader(req).byte() else {
        return unchecked();
    };
    if !(1..=3).contains(&difficulty) {
        return reply(byte_failure(181, 2));
    }
    self_task(
        ctx,
        179,
        PartyTaskKind::Difficulty { difficulty },
        byte_failure(181, 2),
    )
}
/// parity: 182 14-party-handlers.md#182--g_changepartyinviteauthreqcmd--partial partial
pub fn handle_change_party_invite_auth(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Spec retains helper gaps; use the explicit shared contracts where read.
    let Some(authority) = Reader(req).byte() else {
        return unchecked();
    };
    if authority > 1 {
        return Outcome::reason(184, 798);
    }
    self_task(
        ctx,
        182,
        PartyTaskKind::PartyAuthority { authority },
        failure(184, 2),
    )
}

#[derive(Clone, Debug)]
pub struct PartyInviteInput {
    pub target_id: u64,
    pub target_name: Vec<u16>,
    pub correlation: u32,
    pub options: [u8; 6],
}
fn party_invite_failure(input: &PartyInviteInput, reason: u16) -> Outcome {
    let mut body = reason.to_le_bytes().to_vec();
    body.extend(input.target_id.to_le_bytes());
    string_body(&mut body, &input.target_name);
    body.extend(input.correlation.to_le_bytes());
    body.extend(&input.options[..2]);
    Outcome::reply(159, body)
}
/// parity: 157 14-party-handlers.md#157--g_invitepartyreqcmd--partial partial
pub fn handle_invite_party(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Decode-failure echo can contain unspecified native locals.
    let mut r = Decode::new(req);
    let mut input = PartyInviteInput {
        target_id: r.quad(),
        target_name: r.name(),
        correlation: r.int(),
        options: r.bytes(),
    };
    if !r.ok {
        return unchecked();
    }
    if !user_ok(ctx.pc()) {
        return party_invite_failure(&input, 754);
    }
    if !(1..=3).contains(&input.options[1]) {
        input.options[1] = ctx.world.party_config.default_difficulty;
    }
    match target(ctx, input.target_id, &input.target_name) {
        Err(why) => boundary(why),
        Ok(None) => party_invite_failure(&input, 747),
        Ok(Some(_)) => {
            boundary("❓ STOP excluded restriction call in 157; cleanup and invitation task unread")
        }
    }
}
/// Ordinary continuation only. The caller must reach this after the excluded
/// boundary. This does not implement or approve that boundary's policy.
pub fn continue_party_invite(
    ctx: &mut Context<'_>,
    mut input: PartyInviteInput,
    target: &Pc,
) -> Outcome {
    input.target_id = target.object_id;
    input.target_name = target.name.clone();
    if !(1..=3).contains(&input.options[1]) {
        input.options[1] = ctx.world.party_config.default_difficulty;
    }
    let reason = if target.object_id == ctx.pc().object_id || target.db_id == ctx.pc().db_id {
        Some(747)
    } else if ctx.pc().party_id == 0
        && input.options[1] != ctx.world.party_config.default_difficulty
    {
        match special_zone(ctx, ctx.pc()) {
            Ok(true) => Some(758),
            Ok(false) => None,
            Err(why) => return boundary(why),
        }
    } else {
        None
    };
    if let Some(reason) = reason {
        return party_invite_failure(&input, reason);
    }
    if target_arena_blocked(ctx, target) {
        return party_invite_failure(&input, 6);
    }
    let mut prefix = Outcome::default();
    registration(ctx, &mut prefix);
    let error = party_invite_failure(&input, if target.zone.is_none() { 751 } else { 2 });
    let mut out = queue_party(
        ctx,
        target,
        157,
        PartyTaskKind::InviteParty {
            inviter_db: ctx.pc().db_id,
            inviter: ctx.pc().object_id,
            correlation: input.correlation,
            options: input.options,
        },
        error.replies[0].clone(),
    );
    out.daemon_requests = prefix.daemon_requests;
    // ❓ Post-resolution failure cleanup 0x14060B260 remains unread.
    if !out.replies.is_empty() {
        out.status = Status::Unsupported("❓ invite failure cleanup");
    }
    out
}
/// parity: 161 14-party-handlers.md#161--g_joinpartyreqcmd--partial partial
pub fn handle_join_party(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Invitation authorization, insertion, inheritance and callbacks.
    let mut r = Decode::new(req);
    let (_, invitation, correlation, difficulty) = (r.quad(), r.quad(), r.int(), r.byte());
    if !r.ok {
        return unchecked();
    }
    let error = |reason: u16| {
        let mut body = reason.to_le_bytes().to_vec();
        body.extend(correlation.to_le_bytes());
        body.push(difficulty);
        Message { command: 162, body }
    };
    if !user_ok(ctx.pc()) {
        return reply(error(768));
    }
    if !(1..=3).contains(&difficulty) {
        return reply(error(2));
    }
    if ctx.pc().zone.is_none() {
        return reply(error(764));
    }
    let mut prefix = Outcome::default();
    registration(ctx, &mut prefix);
    let mut out = self_task(
        ctx,
        161,
        PartyTaskKind::JoinParty {
            invitation,
            correlation,
            difficulty,
        },
        error(2),
    );
    out.daemon_requests = prefix.daemon_requests;
    out
}
/// parity: 163 14-party-handlers.md#163--g_rejectpartyreqcmd--partial partial
pub fn handle_reject_party(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Rejection match, removal, notification and timeout interaction.
    let mut r = Decode::new(req);
    let (_, invitation, correlation, kind) = (r.quad(), r.quad(), r.byte(), r.byte());
    let error = |reason: u16| Message {
        command: 164,
        body: [reason.to_le_bytes().as_slice(), &[correlation]].concat(),
    };
    if !r.ok {
        return reply(error(2));
    }
    if ctx.pc().zone.is_none() {
        return reply(error(764));
    }
    self_task(
        ctx,
        163,
        PartyTaskKind::Reject {
            group: 1,
            invitation,
            correlation,
            kind,
        },
        error(2),
    )
}
/// parity: 243 14-party-handlers.md#243--g_leaveteamreqcmd--partial partial
pub fn handle_leave_team(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Manager removal/disband/leader/backend paths; mode 1 shares 168.
    let mut r = Decode::new(req);
    let mode = r.byte();
    let error = |reason: u16| Message {
        command: 244,
        body: [reason.to_le_bytes().as_slice(), &[mode]].concat(),
    };
    if !r.ok || !ctx.world.team_enabled {
        return reply(error(2));
    }
    if ctx.pc().zone.is_none() {
        return reply(error(764));
    }
    let kind = if mode == 1 {
        PartyTaskKind::LeaveParty
    } else {
        PartyTaskKind::LeaveTeam { mode }
    };
    self_task(ctx, 243, kind, error(2))
}
/// parity: 245 14-party-handlers.md#245--g_banishteamreqcmd--partial partial
pub fn handle_banish_team(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Manager authority, indexes, removal, notifications and backend work.
    if !ctx.world.team_enabled {
        return Outcome::reason(246, 2);
    }
    let mut r = Decode::new(req);
    let (party, slot) = (r.short(), r.byte());
    if !r.ok {
        return Outcome::reason(246, 2);
    }
    self_task(
        ctx,
        245,
        PartyTaskKind::BanishTeam { party, slot },
        failure(246, 2),
    )
}
/// parity: 247 14-party-handlers.md#247--g_teammemberpartychangereqcmd--partial partial
pub fn handle_team_member_party_change(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Move/swap, leader/rules, sync and backend effects.
    if !ctx.world.team_enabled {
        return Outcome::reason(249, 2);
    }
    let mut r = Decode::new(req);
    let source = (r.short(), r.byte(), r.quad());
    let destination = (r.short(), r.byte(), r.quad());
    if !r.ok {
        return Outcome::reason(249, 2);
    }
    if source.0 == 0xffff || source.1 == 0xff || source.2 == 0 {
        return Outcome::reason(249, 878);
    }
    self_task(
        ctx,
        247,
        PartyTaskKind::MoveMember {
            source,
            destination,
        },
        failure(249, 2),
    )
}
/// parity: 250 14-party-handlers.md#250--g_delegateteamleaderreqcmd--partial partial
pub fn handle_delegate_team_leader(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Manager current-leader/target checks, mutation and notifications.
    if !ctx.world.team_enabled {
        return Outcome::reason(252, 2);
    }
    let Some(target) = Reader(req).quad() else {
        return Outcome::reason(252, 2);
    };
    if ctx.pc().zone.is_none() {
        return Outcome::reason(252, 764);
    }
    self_task(
        ctx,
        250,
        PartyTaskKind::DelegateTeam { target },
        failure(252, 2),
    )
}
/// parity: 253 14-party-handlers.md#253--g_nominatepartyleaderreqcmd--partial partial
pub fn handle_nominate_party_leader(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Manager team/party role, target checks and leader update.
    if !ctx.world.team_enabled {
        return Outcome::reason(252, 2);
    }
    let Some(target) = Reader(req).quad() else {
        return Outcome::reason(255, 2);
    };
    self_task(
        ctx,
        253,
        PartyTaskKind::NominateLeader { target },
        failure(255, 2),
    )
}
/// parity: 256 14-party-handlers.md#256--g_changeteaminviteauthreqcmd--done ported
pub fn handle_change_team_invite_auth(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if !ctx.world.team_enabled {
        return Outcome::reason(258, 2);
    }
    let Some(authority) = Reader(req).byte() else {
        return Outcome::reason(258, 2);
    };
    if authority > 2 {
        return Outcome::reason(258, 894);
    }
    if ctx.pc().zone.is_none() {
        return Outcome::reason(258, 764);
    }
    self_task(
        ctx,
        256,
        PartyTaskKind::TeamAuthority { authority },
        failure(258, 2),
    )
}
/// parity: 259 14-party-handlers.md#259--g_changeteampartydifficultytypereqcmd--partial partial
pub fn handle_change_team_party_difficulty(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Manager selector/role/content checks, mutation and synchronization.
    if !ctx.world.team_enabled {
        return Outcome::reason(261, 2);
    }
    let mut r = Decode::new(req);
    let (selector, difficulty) = (r.byte(), r.byte());
    if !r.ok || !(1..=3).contains(&difficulty) {
        return Outcome::reason(261, 2);
    }
    if ctx.pc().zone.is_none() {
        return Outcome::reason(261, 764);
    }
    self_task(
        ctx,
        259,
        PartyTaskKind::TeamDifficulty {
            selector,
            difficulty,
        },
        failure(261, 2),
    )
}

fn suggestion_failure(
    command: u16,
    reason: u16,
    mode: Option<u8>,
    id: u64,
    name: &[u16],
    options: &[u8],
) -> Outcome {
    let mut body = reason.to_le_bytes().to_vec();
    if let Some(mode) = mode {
        body.push(mode);
    }
    body.extend(id.to_le_bytes());
    string_body(&mut body, name);
    body.extend(options);
    Outcome::reply(command, body)
}
fn suggestion_notify(command: u16, mode: Option<u8>, name: &[u16], options: &[u8]) -> Message {
    let mut body = vec![];
    if let Some(mode) = mode {
        body.push(mode);
    }
    string_body(&mut body, name);
    body.extend(options);
    Message { command, body }
}
fn suggestion_ack(
    command: u16,
    mode: Option<u8>,
    id: u64,
    name: &[u16],
    options: &[u8],
) -> Message {
    let mut body = vec![];
    if let Some(mode) = mode {
        body.push(mode);
    }
    body.extend(id.to_le_bytes());
    string_body(&mut body, name);
    body.extend(options);
    Message { command, body }
}
/// parity: 147 14-party-handlers.md#147--g_suggestpartyreqcmd--partial partial
pub fn handle_suggest_party(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Task execution, option/config names, malformed locals and downstream work.
    let mut r = Decode::new(req);
    let (id, name, options) = (r.quad(), r.name(), r.bytes::<6>());
    if !r.ok {
        return unchecked();
    }
    let error = |reason| suggestion_failure(150, reason, None, id, &name, &options[..2]);
    if id == ctx.pc().object_id {
        return error(740);
    }
    let peer = match target(ctx, id, &name) {
        Err(why) => return boundary(why),
        Ok(None) => return error(740),
        Ok(Some(pc)) => pc,
    };
    if peer.object_id == ctx.pc().object_id {
        return error(740);
    }
    if peer.party_id == 0 && options[1] != ctx.world.party_config.default_difficulty {
        match special_zone(ctx, &peer) {
            Ok(true) => return error(741),
            Err(why) => return boundary(why),
            _ => {}
        }
    }
    if target_arena_blocked(ctx, &peer) {
        return error(6);
    }
    if ctx.world.party_config.suggestion_task {
        return queue_party(
            ctx,
            &peer,
            147,
            PartyTaskKind::SuggestParty {
                requester: ctx.pc().object_id,
                options,
            },
            error(2).replies[0].clone(),
        );
    }
    let mut out = Outcome::default();
    send(
        &mut out,
        peer.object_id,
        suggestion_notify(149, None, &ctx.pc().name, &options),
    );
    self_send(
        &mut out,
        ctx.pc(),
        suggestion_ack(148, None, id, &name, &options[..2]),
    );
    out
}
/// parity: 151 14-party-handlers.md#151--g_rejectsuggestpartyreqcmd--partial partial
pub fn handle_reject_suggest_party(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Native malformed values and name-map comparator remain unspecified.
    let mut r = Decode::new(req);
    let (id, name, options) = (r.quad(), r.name(), r.bytes::<2>());
    if !r.ok {
        return unchecked();
    }
    let error = || suggestion_failure(154, 742, None, id, &name, &options);
    if id == ctx.pc().object_id {
        return error();
    }
    let peer = match target(ctx, id, &name) {
        Err(why) => return boundary(why),
        Ok(None) => return error(),
        Ok(Some(pc)) => pc,
    };
    if peer.object_id == ctx.pc().object_id {
        return error();
    }
    let mut out = Outcome::default();
    send(
        &mut out,
        peer.object_id,
        suggestion_notify(153, None, &ctx.pc().name, &options),
    );
    self_send(
        &mut out,
        ctx.pc(),
        suggestion_ack(152, None, id, &name, &options),
    );
    out
}
/// parity: 226 14-party-handlers.md#226--g_suggestteamreqcmd--partial partial
pub fn handle_suggest_team(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Permission predicate, exact mode/options and malformed locals.
    let mut r = Decode::new(req);
    let (mode, mut id, mut name, options) = (r.byte(), r.quad(), r.name(), r.bytes::<5>());
    if !r.ok {
        return unchecked();
    }
    if !ctx.world.team_enabled || id == ctx.pc().object_id {
        return suggestion_failure(229, 822, Some(mode), id, &name, &options[..1]);
    }
    let peer = match target(ctx, id, &name) {
        Err(why) => return boundary(why),
        Ok(None) => return suggestion_failure(229, 822, Some(mode), id, &name, &options[..1]),
        Ok(Some(pc)) => pc,
    };
    id = peer.object_id;
    let error = |reason| suggestion_failure(229, reason, Some(mode), id, &name, &options[..1]);
    if target_arena_blocked(ctx, &peer) {
        return error(6);
    }
    if !peer.team_member || peer.team_id == 0 {
        return error(822);
    }
    let location = match ctx
        .services
        .party_state()
        .and_then(|s| s.resolve(peer.db_id, peer.object_id))
    {
        Some(v) => v,
        None => return error(822),
    };
    let permission = if (1..=3).contains(&mode) { mode + 1 } else { 0 };
    let Some(allowed) = ctx
        .services
        .party_operation(PartyOperation::SuggestPermission {
            team: location.team,
            mode: permission,
            party: location.party as u8,
            db: peer.db_id,
            runtime: peer.object_id,
        })
    else {
        return boundary("❓ team suggestion permission predicate");
    };
    let recipient = if allowed != 0 {
        peer
    } else {
        let leader = ctx.services.party_state().and_then(|s| {
            s.teams
                .get(location.team)?
                .as_ref()
                .map(|t| t.leader_runtime_id)
        });
        match leader {
            Some(id) => match target(ctx, id, &[]) {
                Ok(Some(pc)) => pc,
                Err(why) => return boundary(why),
                _ => return error(822),
            },
            None => return error(822),
        }
    };
    if allowed == 0 {
        name = recipient.name.clone();
    }
    let mut out = Outcome::default();
    send(
        &mut out,
        recipient.object_id,
        suggestion_notify(228, Some(mode), &ctx.pc().name, &options),
    );
    self_send(
        &mut out,
        ctx.pc(),
        suggestion_ack(227, Some(mode), recipient.object_id, &name, &options[..1]),
    );
    out
}
/// parity: 230 14-party-handlers.md#230--g_rejectsuggestteamreqcmd--partial partial
pub fn handle_reject_suggest_team(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Exact mode/options, name-map comparator and malformed native values.
    let mut r = Decode::new(req);
    let (mode, id, name, options) = (r.byte(), r.quad(), r.name(), r.bytes::<2>());
    if !r.ok {
        return unchecked();
    }
    if !ctx.world.team_enabled {
        return suggestion_failure(229, 822, Some(mode), id, &name, &options[..1]);
    }
    if id == ctx.pc().object_id {
        return suggestion_failure(233, 823, Some(mode), id, &name, &options);
    }
    let peer = match target(ctx, id, &name) {
        Err(why) => return boundary(why),
        Ok(None) => return suggestion_failure(233, 823, Some(mode), id, &name, &options),
        Ok(Some(pc)) => pc,
    };
    // Deliberately no resolved-self check for name lookup.
    let mut out = Outcome::default();
    send(
        &mut out,
        peer.object_id,
        suggestion_notify(232, Some(mode), &ctx.pc().name, &options),
    );
    self_send(
        &mut out,
        ctx.pc(),
        suggestion_ack(231, Some(mode), peer.object_id, &name, &options),
    );
    out
}

#[derive(Clone, Debug)]
pub struct TeamInviteInput {
    pub mode: u8,
    pub target_id: u64,
    pub target_name: Vec<u16>,
    pub options: [u8; 5],
}
fn team_invite_failure(input: &TeamInviteInput, reason: u16) -> Outcome {
    suggestion_failure(
        236,
        reason,
        Some(input.mode),
        input.target_id,
        &input.target_name,
        &input.options[..1],
    )
}
/// parity: 234 14-party-handlers.md#234--g_inviteteamreqcmd--partial partial
pub fn handle_invite_team(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Excluded boundary, leader resolvers, task policy, callbacks and DB work.
    let mut r = Decode::new(req);
    let input = TeamInviteInput {
        mode: r.byte(),
        target_id: r.quad(),
        target_name: r.name(),
        options: r.bytes(),
    };
    if !r.ok || !ctx.world.team_enabled {
        return team_invite_failure(&input, 2);
    }
    if !user_ok(ctx.pc()) {
        return team_invite_failure(&input, 840);
    }
    if !(1..=3).contains(&input.mode) {
        return team_invite_failure(&input, 825);
    }
    match target(ctx, input.target_id, &input.target_name) {
        Err(why) => boundary(why),
        Ok(None) => team_invite_failure(&input, 824),
        Ok(Some(_)) => boundary("❓ STOP excluded restriction call in 234; invitation task unread"),
    }
}
fn invite_leader(ctx: &mut Context<'_>, peer: &Pc, team: bool) -> Result<Option<Pc>, &'static str> {
    match ctx.services.party_invite_leader(peer, team) {
        PartyLeaderLookup::Unknown => Err("❓ party/team invite leader resolver"),
        PartyLeaderLookup::Missing | PartyLeaderLookup::Runtime(0) => Ok(None),
        PartyLeaderLookup::Runtime(id) => target(ctx, id, &[]),
    }
}
/// Ordinary continuation only; never approves the excluded restriction boundary.
pub fn continue_team_invite(
    ctx: &mut Context<'_>,
    mut input: TeamInviteInput,
    peer: &Pc,
) -> Outcome {
    input.target_id = peer.object_id;
    input.target_name = peer.name.clone();
    if peer.object_id == ctx.pc().object_id || peer.db_id == ctx.pc().db_id {
        return team_invite_failure(&input, 824);
    }
    if target_arena_blocked(ctx, peer) {
        return team_invite_failure(&input, 6);
    }
    let mut prefix = Outcome::default();
    registration(ctx, &mut prefix);
    let result = (|| {
        let caster = match input.mode {
            1 => peer.clone(),
            2 => {
                if ctx.pc().party_id == 0 {
                    return team_invite_failure(&input, 829);
                }
                if peer.party_id == 0 {
                    return team_invite_failure(&input, 837);
                }
                match invite_leader(ctx, peer, false) {
                    Ok(Some(pc)) => pc,
                    Ok(None) => return team_invite_failure(&input, 835),
                    Err(why) => return boundary(why),
                }
            }
            3 => {
                if ctx.pc().party_id == 0 || !ctx.pc().team_member || ctx.pc().team_id == 0 {
                    return team_invite_failure(&input, 829);
                }
                if !peer.team_member || peer.team_id == 0 {
                    return team_invite_failure(&input, 836);
                }
                match invite_leader(ctx, peer, true) {
                    Ok(Some(pc)) => pc,
                    Ok(None) => return team_invite_failure(&input, 834),
                    Err(why) => return boundary(why),
                }
            }
            _ => return boundary("❓ invalid ordinary team invite continuation"),
        };
        let error = team_invite_failure(&input, if caster.zone.is_none() { 832 } else { 2 });
        queue_party(
            ctx,
            &caster,
            234,
            PartyTaskKind::InviteTeam {
                mode: input.mode,
                inviter_db: ctx.pc().db_id,
                inviter: ctx.pc().object_id,
                options: input.options,
                target: peer.object_id,
            },
            error.replies[0].clone(),
        )
    })();
    Outcome {
        daemon_requests: prefix.daemon_requests,
        ..result
    }
}
/// parity: 239 14-party-handlers.md#239--g_jointeamreqcmd--partial partial
pub fn handle_join_team(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Deferred identity/mode/role/capacity, conversion, merge and callbacks.
    let mut r = Decode::new(req);
    let (mode, invitation, other) = (r.byte(), r.quad(), r.quad());
    let error = |reason: u16| Message {
        command: 240,
        body: [reason.to_le_bytes().as_slice(), &[mode]].concat(),
    };
    if !r.ok || !ctx.world.team_enabled {
        return reply(error(2));
    }
    if !user_ok(ctx.pc()) {
        return reply(error(863));
    }
    if invitation == ctx.pc().object_id {
        return reply(error(2));
    }
    if ctx.pc().zone.is_none() {
        return reply(error(852));
    }
    let mut prefix = Outcome::default();
    registration(ctx, &mut prefix);
    let mut out = self_task(
        ctx,
        239,
        PartyTaskKind::JoinTeam {
            mode,
            invitation,
            other,
        },
        error(2),
    );
    out.daemon_requests = prefix.daemon_requests;
    out
}
/// parity: 241 14-party-handlers.md#241--g_rejectteamreqcmd--partial partial
pub fn handle_reject_team(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Shared task invitation identity/kind/notification and downstream work.
    let mut r = Decode::new(req);
    let (invitation, kind) = (r.quad(), r.byte());
    if !r.ok {
        return Outcome::reason(242, 2);
    }
    self_task(
        ctx,
        241,
        PartyTaskKind::Reject {
            group: 2,
            invitation,
            correlation: 0,
            kind,
        },
        failure(242, 2),
    )
}
/// parity: 216 14-party-handlers.md#216--g_partyvotecastreqcmd--partial partial
pub fn handle_party_vote_cast(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Task vote state, role/target checks, tally/timing and notifications.
    if !ctx.world.party_config.vote_enabled {
        return Outcome::log("party vote disabled");
    }
    if !ctx.world.arena {
        return Outcome::reason(173, 813);
    }
    let mut r = Decode::new(req);
    let (target, vote) = (r.byte(), r.byte());
    if !r.ok {
        return unchecked();
    }
    self_task(
        ctx,
        216,
        PartyTaskKind::Vote { target, vote },
        failure(219, 2),
    )
}
/// parity: 220 14-party-handlers.md#220--g_partyvotecancelcastreqcmd--done ported
pub fn handle_party_vote_cancel_cast(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if !ctx.world.party_config.vote_enabled {
        return Outcome::log("party vote disabled");
    }
    let pc = ctx.pc().clone();
    let Some(state) = ctx.services.party_state() else {
        return boundary("❓ PARTY state adapter");
    };
    let Some(location) = state.resolve(pc.db_id, pc.object_id) else {
        return Outcome::reason(223, 820);
    };
    let party = state.teams[location.team].as_ref().unwrap().parties[location.party]
        .as_ref()
        .unwrap();
    let member = &party.members[location.member];
    if member.voted_for == 0xff {
        return Outcome::reason(223, 821);
    }
    let (slot, vote) = (member.voted_for, member.vote);
    let recipient = party
        .members
        .iter()
        .find(|m| m.slot == slot)
        .filter(|m| m.state == 3 && m.runtime_id != 0)
        .and_then(|m| state.by_id(m.runtime_id))
        .map(|p| p.pc.object_id);
    let mut out = Outcome::default();
    if let Some(to) = recipient {
        send(
            &mut out,
            to,
            Message {
                command: 222,
                body: vec![slot, vote],
            },
        );
    }
    let member = &mut state.teams[location.team].as_mut().unwrap().parties[location.party]
        .as_mut()
        .unwrap()
        .members[location.member];
    member.vote = 0;
    member.voted_for = 0xff;
    out.party_effects
        .push(PartyEffect::VoteCleared { location });
    self_send(
        &mut out,
        &pc,
        Message {
            command: 221,
            body: vec![],
        },
    );
    out.logs.push("party vote cancel");
    out
}
/// parity: 224 14-party-handlers.md#224--g_partyvoteresetreqcmd--done ported
pub fn handle_party_vote_reset(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if ctx.world.party_config.vote_enabled {
        // This is the existing assertion/report call, not a vote reset.
        let mut out = Outcome::log("party vote reset assertion/report");
        out.party_effects.push(PartyEffect::AssertionReport);
        out
    } else {
        Outcome::log("party vote disabled")
    }
}

fn auction(
    ctx: &mut Context<'_>,
    req: &[u8],
    command: u16,
    failure_id: u16,
    operation: AuctionOperation,
    has_byte: bool,
) -> Outcome {
    let mut r = Decode::new(req);
    let value = if operation == AuctionOperation::Bid {
        r.quad()
    } else {
        0
    };
    let byte = if has_byte { r.byte() } else { 0 };
    if !r.ok {
        return unchecked();
    }
    if operation == AuctionOperation::Bid
        && ((value as i64) < 0 || value as i64 > ctx.world.party_config.maximum_bid)
    {
        return Outcome::reason(failure_id, 33);
    }
    self_task(
        ctx,
        command,
        PartyTaskKind::Auction {
            operation,
            value,
            byte,
        },
        failure(failure_id, 2),
    )
}
/// parity: 214 14-party-handlers.md#214--g_passpartyauctionreqcmd--partial partial
pub fn handle_pass_party_auction(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Auction identity, pass mutation, item/money handling and backend work.
    auction(ctx, req, 214, 215, AuctionOperation::Pass, false)
}
/// parity: 205 14-party-handlers.md#205--g_bidcurrentauctionitemreqcmd--partial partial
pub fn handle_bid_current_auction_item(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Maximum/byte names, balance/minimum/auction checks, debit and persistence.
    auction(ctx, req, 205, 207, AuctionOperation::Bid, true)
}
/// parity: 202 14-party-handlers.md#202--g_finishpartyauctionreqcmd--partial partial
pub fn handle_finish_party_auction(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Auction/leader/item checks, awards/money, notifications and persistence.
    auction(ctx, req, 202, 204, AuctionOperation::Finish, true)
}
/// parity: 211 14-party-handlers.md#211--g_distributioncurrentauctionitemreqcmd--partial partial
pub fn handle_distribution_current_auction_item(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Leader/auction checks, distribution, ownership and persistence.
    auction(ctx, req, 211, 213, AuctionOperation::Distribute, false)
}
/// parity: 208 14-party-handlers.md#208--g_giveupcurrentauctionitemreqcmd--partial partial
pub fn handle_give_up_current_auction_item(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Byte/auction identity, give-up transition and item/money/backend effects.
    auction(ctx, req, 208, 210, AuctionOperation::GiveUp, true)
}
/// parity: 424 14-party-handlers.md#424--g_registerpartymatchreqcmd--partial partial
pub fn handle_register_party_match(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Manager selector/role/cooldown, registration and downstream effects.
    let Some(selector) = Reader(req).int() else {
        return unchecked();
    };
    if !user_ok(ctx.pc()) {
        return Outcome::reason(426, 1320);
    }
    match ctx
        .services
        .party_operation(PartyOperation::RegisterMatch(selector))
    {
        None => boundary("❓ party match registration manager"),
        Some(0) => Outcome::reply(425, vec![]),
        Some(result) => Outcome::reason(
            426,
            match result {
                1 => 1318,
                2 => 1317,
                4 => 1319,
                _ => 2,
            },
        ),
    }
}
/// parity: 429 14-party-handlers.md#429--g_rejectpartymatchreqcmd--partial partial
pub fn handle_reject_party_match(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Correlation meaning and native unchecked malformed values.
    let mut r = Decode::new(req);
    let (id, correlation) = (r.quad(), r.int());
    if !r.ok {
        return unchecked();
    }
    let peer = match target(ctx, id, &[]) {
        Err(why) => return boundary(why),
        Ok(None) => return Outcome::log("party match target missing"),
        Ok(Some(pc)) => pc,
    };
    let mut out = Outcome::default();
    send(
        &mut out,
        peer.object_id,
        Message {
            command: 430,
            body: [
                ctx.pc().object_id.to_le_bytes().as_slice(),
                &correlation.to_le_bytes(),
            ]
            .concat(),
        },
    );
    out
}
