//! Doc 25: arena lobby entry functions. Build 381.8867.
//!
//! Zone tasks use `skills::queue`. Opaque state and backend sends use
//! `Services::arena_lobby`. A successful send is not a completed lobby action.
use super::party::string_body;
use super::skills::queue;
use super::*;
use std::collections::VecDeque;

pub const ARENA_LOBBY_COMMANDS: &[u16] = &[
    639, 642, 645, 651, 654, 657, 660, 663, 666, 669, 672, 675, 678, 681, 684, 687, 690, 693, 696,
    699, 702, 761, 763,
];

/// Extends the existing PC arena state. It does not hold a second state/key.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ArenaLobbyPc {
    /// PC qword +37536.
    pub route: u64,
    /// PC byte +37552.
    pub kind: u8,
    /// PC byte +38485; outside the 642 reset ranges.
    pub attraction: u8,
    /// PC qword +37608.
    pub waiting_id: u64,
    /// PC qword +37624, relative server tick.
    pub waiting_tick: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArenaLobbyTask {
    /// 639, kind 112, vtable 0x140B9E890. +144 record pointer, +152 = 0.
    /// The adapter resolves this verified key to a record. It is not a pointer.
    CheckPortal { portal_key: u64 },
    /// 761, kind 41, vtable 0x140B95C00. Normalized bool +144.
    PartyAgreement { agreed: bool },
    /// 763, kind 41, vtable 0x140C3E5C0. Normalized bool +144.
    PartyFinalDecision { decided: bool },
}
impl From<ArenaLobbyTask> for SkillTaskKind {
    fn from(task: ArenaLobbyTask) -> Self {
        SkillTaskKind::ArenaLobby(task)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArenaLobbySender {
    pub object_id: u64,
    pub db_id: u64,
    /// Signed native PC short +96, not the independent model world_id.
    pub world: i16,
    pub name: Vec<u16>,
    pub route: u64,
}
impl ArenaLobbySender {
    fn from_pc(pc: &Pc) -> Self {
        Self {
            object_id: pc.object_id,
            db_id: pc.db_id,
            world: pc.short_96,
            name: pc.name.clone(),
            route: pc.arena_lobby.route,
        }
    }
}

/// Exact body-only backend send plan. No transport header or completion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArenaLobbyBackend {
    pub world: i16,
    pub command: u16,
    pub descriptor: &'static str,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArenaLobbyOperation {
    /// 0x140044850. Pass means a pending return reservation: drop the request.
    ReservationPending {
        db_id: u64,
    },
    /// 0x140044D80 and optional backend 493. Pass also means no record.
    /// Fail means the optional send failed; its result does not reject the request.
    UpdateSessionTimer {
        sender: ArenaLobbySender,
    },
    /// DataManager slot 87 virtual +168. Fail means no portal record.
    PortalExists {
        portal_key: u64,
    },
    /// Remaining 0x140606280 builders, snapshots and side effects.
    Enter {
        sender: ArenaLobbySender,
        context_key: u64,
        kind: u8,
        bytes: Vec<u8>,
    },
    /// Global byte +819.
    CrossOptionEnabled,
    /// Remaining 0x140606CE0 rules and builder. Maps retain wire order and duplicates.
    SetMatchOption {
        sender: ArenaLobbySender,
        fields: [u8; 4],
        maps: Vec<i16>,
    },
    /// Daemon local-world list. Pass = local; Fail = cross-world.
    LocalWorld {
        world: i16,
    },
    /// Deliver target 769 hb before the requester receives 664.
    LocalInvitation {
        sender: ArenaLobbySender,
        target: u64,
        bytes: Vec<u8>,
    },
    /// 0x1400D3320, including its message wrapper.
    CrossInvitation {
        sender: ArenaLobbySender,
        world: i16,
        name: Vec<u16>,
        bytes: Vec<u8>,
    },
    /// 0x1406117A0: user expiry and configured time margin.
    GameTimeValid {
        db_id: u64,
    },
    /// Rule +652 and global +800/+801/+802.
    SpecialMatchEnabled {
        attraction: u8,
    },
    Send(ArenaLobbyBackend),
    /// 675: the native hEq call does not explicitly set its final q argument.
    /// An adapter must supply the exact tail. Unknown must not become zero.
    ReadyFailureTail {
        reason: u16,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ArenaLobbyResult {
    #[default]
    Unknown,
    Pass,
    Fail,
    /// Exact helper error from Enter / SetMatchOption. Zero means success.
    Reason(u16),
    ReplyTail([u8; 8]),
}

/// Test/mock selection keys. These do not represent native state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArenaLobbyStage {
    ReservationPending,
    UpdateSessionTimer,
    PortalExists,
    Enter,
    CrossOptionEnabled,
    SetMatchOption,
    LocalWorld,
    LocalInvitation,
    CrossInvitation,
    GameTimeValid,
    SpecialMatchEnabled,
    Send,
    ReadyFailureTail,
}
impl ArenaLobbyOperation {
    pub fn stage(&self) -> ArenaLobbyStage {
        match self {
            Self::ReservationPending { .. } => ArenaLobbyStage::ReservationPending,
            Self::UpdateSessionTimer { .. } => ArenaLobbyStage::UpdateSessionTimer,
            Self::PortalExists { .. } => ArenaLobbyStage::PortalExists,
            Self::Enter { .. } => ArenaLobbyStage::Enter,
            Self::CrossOptionEnabled => ArenaLobbyStage::CrossOptionEnabled,
            Self::SetMatchOption { .. } => ArenaLobbyStage::SetMatchOption,
            Self::LocalWorld { .. } => ArenaLobbyStage::LocalWorld,
            Self::LocalInvitation { .. } => ArenaLobbyStage::LocalInvitation,
            Self::CrossInvitation { .. } => ArenaLobbyStage::CrossInvitation,
            Self::GameTimeValid { .. } => ArenaLobbyStage::GameTimeValid,
            Self::SpecialMatchEnabled { .. } => ArenaLobbyStage::SpecialMatchEnabled,
            Self::Send(_) => ArenaLobbyStage::Send,
            Self::ReadyFailureTail { .. } => ArenaLobbyStage::ReadyFailureTail,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MemoryArenaLobby {
    pub results: BTreeMap<ArenaLobbyStage, ArenaLobbyResult>,
    /// Optional ordered overrides, consumed before stage defaults.
    pub sequence: VecDeque<ArenaLobbyResult>,
    pub calls: Vec<ArenaLobbyOperation>,
}
impl MemoryArenaLobby {
    pub fn check(&mut self, operation: &ArenaLobbyOperation) -> ArenaLobbyResult {
        self.calls.push(operation.clone());
        self.sequence.pop_front().unwrap_or_else(|| {
            self.results
                .get(&operation.stage())
                .copied()
                .unwrap_or_default()
        })
    }
}

pub(super) fn dispatch_arena_lobby(ctx: &mut Context<'_>, command: u16, req: &[u8]) -> Outcome {
    match command {
        639 => handle_reserve_arena_lobby(ctx, req),
        642 => handle_enter_arena_lobby(ctx, req),
        645 => handle_leave_arena_lobby(ctx, req),
        651 => handle_exit_from_arena_lobby_room(ctx, req),
        654 => handle_set_arena_lobby_room_match_option(ctx, req),
        657 => handle_delegate_arena_lobby_room_team_leader(ctx, req),
        660 => handle_set_arena_lobby_room_password(ctx, req),
        663 => handle_invite_arena_lobby_room(ctx, req),
        666 => handle_broadcast_arena_lobby_room_unrated_invitation(ctx, req),
        669 => handle_join_arena_lobby_room(ctx, req),
        672 => handle_banish_from_arena_lobby(ctx, req),
        675 => handle_ready_to_match_arena_lobby_room(ctx, req),
        678 => handle_unready_to_match_arena_lobby_room(ctx, req),
        681 => handle_match_arena_lobby_room(ctx, req),
        684 => handle_skip_arena_lobby_match(ctx, req),
        687 => handle_cancel_arena_lobby_matching(ctx, req),
        690 => handle_cancel_arena_lobby_match(ctx, req),
        693 => handle_ready_to_go_to_arena(ctx, req),
        696 => handle_decide_to_go_to_arena(ctx, req),
        699 => handle_retrieve_arena_lobby_waiting_number(ctx, req),
        702 => handle_cancel_arena_lobby_waiting(ctx, req),
        761 => handle_reserve_arena_lobby_party_agreement(ctx, req),
        763 => handle_reserve_arena_lobby_party_final_decision(ctx, req),
        _ => unreachable!("membership checked"),
    }
}

fn unknown() -> Outcome {
    Outcome::unsupported("❓ arena lobby server state, builder or send adapter")
}
fn completion() -> Outcome {
    Outcome::unsupported("❓ arena lobby backend completion")
}

/// These entries check cfg mode only, not arena form B. State 3 is valid here.
fn mode_reason(ctx: &Context<'_>, disabled: u16) -> Option<u16> {
    if !ctx.world.party_config.arena_enabled {
        Some(disabled)
    } else if ctx.world.arena {
        Some(5)
    } else {
        None
    }
}
fn busy(pc: &Pc) -> bool {
    !pc.user_present || pc.party_user_blocked || pc.party_user_state != 0
}
fn room_reason(pc: &Pc, state: u8, bad_state: u16, no_record: u16) -> Option<u16> {
    if pc.arena_state != state {
        Some(bad_state)
    } else if pc.arena_qwords[0] == 0 {
        Some(no_record)
    } else {
        None
    }
}
#[allow(clippy::result_large_err)]
fn boolean(ctx: &mut Context<'_>, operation: ArenaLobbyOperation) -> Result<bool, Outcome> {
    match ctx.services.arena_lobby(&operation) {
        ArenaLobbyResult::Pass => Ok(true),
        ArenaLobbyResult::Fail => Ok(false),
        _ => Err(unknown()),
    }
}
fn backend(ctx: &Context<'_>, command: u16, descriptor: &'static str) -> ArenaLobbyBackend {
    let pc = ctx.pc();
    let mut body = pc.arena_lobby.route.to_le_bytes().to_vec();
    body.extend(pc.db_id.to_le_bytes());
    body.extend(pc.object_id.to_le_bytes());
    body.extend(pc.short_96.to_le_bytes());
    ArenaLobbyBackend {
        world: pc.short_96,
        command,
        descriptor,
        body,
    }
}
fn send(ctx: &mut Context<'_>, request: ArenaLobbyBackend, fail: Outcome) -> Outcome {
    match ctx
        .services
        .arena_lobby(&ArenaLobbyOperation::Send(request))
    {
        ArenaLobbyResult::Pass => completion(),
        ArenaLobbyResult::Fail => fail,
        _ => unknown(),
    }
}
fn blob_body(body: &mut Vec<u8>, bytes: &[u8]) {
    body.extend((bytes.len() as u16).to_le_bytes());
    body.extend(bytes);
}
fn leave_request(ctx: &Context<'_>, reason: u8) -> ArenaLobbyBackend {
    let mut request = backend(ctx, 36, "hqqqhsQ");
    string_body(&mut request.body, &ctx.pc().name);
    request.body.push(reason);
    request
}
/// 0x140606B50: fatigue leave request, independent of the arena record key.
#[allow(clippy::result_large_err)]
fn fatigue_leave(ctx: &mut Context<'_>) -> Result<bool, Outcome> {
    if ctx.pc().arena_state != 3 || ctx.pc().arena_lobby.route == 0 {
        return Ok(false);
    }
    let request = leave_request(ctx, 3);
    boolean(ctx, ArenaLobbyOperation::Send(request))
}
fn ready_failure(ctx: &mut Context<'_>, reason: u16) -> Outcome {
    match ctx
        .services
        .arena_lobby(&ArenaLobbyOperation::ReadyFailureTail { reason })
    {
        ArenaLobbyResult::ReplyTail(tail) => {
            let mut body = reason.to_le_bytes().to_vec();
            body.extend(tail);
            Outcome::reply(677, body)
        }
        _ => Outcome::unsupported("❓ 675 failure 677 hEq trailing qword, 0x1403BA521"),
    }
}
fn task(ctx: &Context<'_>, command: u16, zone: u64, kind: ArenaLobbyTask) -> SkillTask {
    SkillTask {
        command,
        caster: ctx.pc().object_id,
        summon: false,
        zone,
        kind: kind.into(),
    }
}

/// parity: 639 25-arena-lobby-handlers.md#639--g_reservearenalobbyreqcmd--partial partial
pub fn handle_reserve_arena_lobby(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1691) {
        return Outcome::reason(641, reason);
    }
    let Some(portal_key) = Reader(req).quad() else {
        return Outcome::reason(641, 2);
    };
    let Some(zone) = ctx.pc().zone else {
        return Outcome::reason(641, 2);
    };
    if busy(ctx.pc()) {
        return Outcome::reason(641, 1712);
    }
    let pending = ArenaLobbyOperation::ReservationPending {
        db_id: ctx.pc().db_id,
    };
    match boolean(ctx, pending) {
        Ok(true) => return Outcome::log("arena: pending return reservation"),
        Ok(false) => {}
        Err(out) => return out,
    }
    let sender = ArenaLobbySender::from_pc(ctx.pc());
    match ctx
        .services
        .arena_lobby(&ArenaLobbyOperation::UpdateSessionTimer { sender })
    {
        ArenaLobbyResult::Pass | ArenaLobbyResult::Fail => {}
        _ => return unknown(),
    }
    match boolean(ctx, ArenaLobbyOperation::PortalExists { portal_key }) {
        Ok(false) => return Outcome::reason(641, 1687),
        Ok(true) => {}
        Err(out) => return out,
    }
    let t = task(ctx, 639, zone, ArenaLobbyTask::CheckPortal { portal_key });
    queue(ctx, t, Outcome::reason(641, 2))
}

/// parity: 642 25-arena-lobby-handlers.md#642--g_enterarenalobbyreqcmd--partial partial
pub fn handle_enter_arena_lobby(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1723) {
        return Outcome::reason(644, reason);
    }
    let Some(bytes) = Reader(req).blob(150) else {
        return Outcome::reason(641, 2);
    };
    if ctx.pc().arena_state != 2 || ctx.pc().arena_qwords[0] == 0 {
        return Outcome::reason(644, 1724);
    }
    let operation = ArenaLobbyOperation::Enter {
        sender: ArenaLobbySender::from_pc(ctx.pc()),
        context_key: ctx.pc().arena_qwords[1],
        kind: ctx.pc().arena_lobby.kind,
        bytes,
    };
    let reason = match ctx.services.arena_lobby(&operation) {
        ArenaLobbyResult::Pass | ArenaLobbyResult::Reason(0) => return completion(),
        ArenaLobbyResult::Fail => 1723,
        ArenaLobbyResult::Reason(reason) => reason,
        _ => return unknown(),
    };
    let mut out = Outcome::reason(644, reason);
    if reason == 1723 {
        let pc = ctx.session.pc.as_mut().expect("PC dispatch");
        pc.arena_state = 4;
        pc.arena_qwords = [0; 2];
        let attraction = pc.arena_lobby.attraction;
        pc.arena_lobby = ArenaLobbyPc {
            attraction,
            ..ArenaLobbyPc::default()
        };
        out.replies.push(Message {
            command: 646,
            body: vec![3],
        });
    }
    out
}

/// parity: 645 25-arena-lobby-handlers.md#645--g_leavearenalobbyreqcmd--partial partial
pub fn handle_leave_arena_lobby(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1725) {
        return Outcome::reason(647, reason);
    }
    if ctx.pc().arena_state != 3 {
        return Outcome::reason(647, 1726);
    }
    send(ctx, leave_request(ctx, 1), Outcome::reason(647, 1725))
}

/// parity: 651 25-arena-lobby-handlers.md#651--g_exitfromarenalobbyroomreqcmd--partial partial
pub fn handle_exit_from_arena_lobby_room(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1731) {
        return Outcome::reason(653, reason);
    }
    if let Some(reason) = room_reason(ctx.pc(), 3, 1732, 1733) {
        return Outcome::reason(653, reason);
    }
    send(ctx, leave_request(ctx, 2), Outcome::reason(653, 1731))
}

/// parity: 654 25-arena-lobby-handlers.md#654--g_setarenalobbyroommatchoptionreqcmd--partial partial
pub fn handle_set_arena_lobby_room_match_option(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1736) {
        return Outcome::reason(656, reason);
    }
    let mut r = Reader(req);
    let (Some(first), Some(second), Some(mut third), Some(fourth)) =
        (r.byte(), r.byte(), r.byte(), r.byte())
    else {
        return Outcome::reason(656, 2);
    };
    match boolean(ctx, ArenaLobbyOperation::CrossOptionEnabled) {
        Ok(false) => third = 0,
        Ok(true) => {}
        Err(out) => return out,
    }
    let Some(count) = r.short().filter(|n| *n <= 3) else {
        return Outcome::reason(656, 1742);
    };
    let Some(maps) = (0..count)
        .map(|_| r.short().map(|n| n as i16))
        .collect::<Option<Vec<_>>>()
    else {
        return Outcome::reason(656, 1742);
    };
    if let Some(reason) = room_reason(ctx.pc(), 3, 1737, 1738) {
        return Outcome::reason(656, reason);
    }
    let operation = ArenaLobbyOperation::SetMatchOption {
        sender: ArenaLobbySender::from_pc(ctx.pc()),
        fields: [first, second, third, fourth],
        maps,
    };
    match ctx.services.arena_lobby(&operation) {
        ArenaLobbyResult::Pass | ArenaLobbyResult::Reason(0) => completion(),
        ArenaLobbyResult::Fail => Outcome::reason(656, 1736),
        ArenaLobbyResult::Reason(reason) => Outcome::reason(656, reason),
        _ => unknown(),
    }
}

fn target_request(ctx: &mut Context<'_>, req: &[u8], banish: bool) -> Outcome {
    let (failure, disabled, bad_state, no_record, command) = if banish {
        (674, 1781, 1782, 1783, 55)
    } else {
        (659, 1748, 1749, 1750, 44)
    };
    if let Some(reason) = mode_reason(ctx, disabled) {
        // Native 657 quirk: only the arena-mode error uses 656.
        let failure = if !banish && reason == 5 { 656 } else { failure };
        return Outcome::reason(failure, reason);
    }
    let mut r = Reader(req);
    let (Some(world), Some(target)) = (r.short(), r.quad()) else {
        return Outcome::reason(failure, 2);
    };
    if banish && ((world as i16) <= 0 || target == 0) {
        return Outcome::reason(failure, 2);
    }
    if let Some(reason) = room_reason(ctx.pc(), 3, bad_state, no_record) {
        return Outcome::reason(failure, reason);
    }
    let mut request = backend(ctx, command, "hqqqhqh");
    request.body.extend(target.to_le_bytes());
    request.body.extend(world.to_le_bytes());
    send(ctx, request, Outcome::reason(failure, disabled))
}

/// parity: 657 25-arena-lobby-handlers.md#657--g_delegatearenalobbyroomteamleaderreqcmd--partial partial
pub fn handle_delegate_arena_lobby_room_team_leader(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    target_request(ctx, req, false)
}

/// parity: 660 25-arena-lobby-handlers.md#660--g_setarenalobbyroompasswordreqcmd--partial partial
pub fn handle_set_arena_lobby_room_password(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1755) {
        return Outcome::reason(662, reason);
    }
    let Some(password) = Reader(req).short() else {
        return Outcome::reason(662, 2);
    };
    if let Some(reason) = room_reason(ctx.pc(), 3, 1756, 1757) {
        return Outcome::reason(662, reason);
    }
    let mut request = backend(ctx, 48, "hqqqhh");
    request.body.extend(password.to_le_bytes());
    send(ctx, request, Outcome::reason(662, 1755))
}

/// parity: 663 25-arena-lobby-handlers.md#663--g_invitearenalobbyroomreqcmd--partial partial
pub fn handle_invite_arena_lobby_room(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1759) {
        return Outcome::reason(665, reason);
    }
    let mut r = Reader(req);
    let (Some(world), Some(name), Some(bytes)) = (r.short(), r.string(33), r.blob(512)) else {
        return Outcome::reason(665, 2);
    };
    let world = world as i16;
    let local = if world > 0 {
        match boolean(ctx, ArenaLobbyOperation::LocalWorld { world }) {
            Ok(local) => local,
            Err(out) => return out,
        }
    } else {
        false
    };
    let sender = ArenaLobbySender::from_pc(ctx.pc());
    if local {
        let Some(target) = ctx.world.mentoring.by_name(&name, 6) else {
            return Outcome::reason(665, 1760);
        };
        let category = match target.arena_state {
            1 => 1,
            2..=5 => 2,
            6 => 3,
            _ => 0,
        };
        let operation = ArenaLobbyOperation::LocalInvitation {
            sender,
            target: target.object_id,
            bytes,
        };
        return match ctx.services.arena_lobby(&operation) {
            ArenaLobbyResult::Pass => Outcome::reply(664, vec![category]),
            _ => unknown(),
        };
    }
    let operation = ArenaLobbyOperation::CrossInvitation {
        sender,
        world,
        name,
        bytes,
    };
    match ctx.services.arena_lobby(&operation) {
        ArenaLobbyResult::Pass => completion(),
        ArenaLobbyResult::Fail => Outcome::reason(665, 1759),
        _ => unknown(),
    }
}

/// parity: 666 25-arena-lobby-handlers.md#666--g_broadcastarenalobbyroomunratedinvitationreqcmd--partial partial
pub fn handle_broadcast_arena_lobby_room_unrated_invitation(
    ctx: &mut Context<'_>,
    req: &[u8],
) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1759) {
        return Outcome::reason(668, reason);
    }
    let mut r = Reader(req);
    let (Some(attraction), Some(room), Some(name), Some(bytes)) =
        (r.byte(), r.quad(), r.string(33), r.blob(512))
    else {
        return Outcome::reason(668, 2);
    };
    let category = match attraction {
        0 => 1,
        1 | 2 => 2,
        _ => return Outcome::reason(668, 2),
    };
    let pc = ctx.pc();
    let mut body = vec![category, 1];
    body.extend(pc.arena_lobby.route.to_le_bytes());
    body.extend(pc.db_id.to_le_bytes());
    body.extend(pc.object_id.to_le_bytes());
    body.extend(room.to_le_bytes());
    string_body(&mut body, &name);
    blob_body(&mut body, &bytes);
    let request = ArenaLobbyBackend {
        world: pc.short_96,
        command: 18,
        descriptor: "hQQqqqqsb",
        body,
    };
    send(ctx, request, Outcome::reason(668, 1759))
}

/// parity: 669 25-arena-lobby-handlers.md#669--g_joinarenalobbyroomreqcmd--partial partial
pub fn handle_join_arena_lobby_room(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1765) {
        return Outcome::reason(671, reason);
    }
    let mut r = Reader(req);
    let (Some(room), Some(name), Some(password), Some(mut cross), Some(option)) =
        (r.quad(), r.string(33), r.short(), r.byte(), r.byte())
    else {
        return Outcome::reason(671, 2);
    };
    match boolean(ctx, ArenaLobbyOperation::CrossOptionEnabled) {
        Ok(false) => cross = 0,
        Ok(true) => {}
        Err(out) => return out,
    }
    let state_error = room_reason(ctx.pc(), 3, 1766, 1767);
    let check = ArenaLobbyOperation::GameTimeValid {
        db_id: ctx.pc().db_id,
    };
    match boolean(ctx, check) {
        Ok(false) => return Outcome::reason(671, 1780),
        Ok(true) => {}
        Err(out) => return out,
    }
    if let Some(reason) = state_error {
        return Outcome::reason(671, reason);
    }
    let mut request = backend(ctx, 52, "hqqqhqshcQ");
    request.body.extend(room.to_le_bytes());
    string_body(&mut request.body, &name);
    request.body.extend(password.to_le_bytes());
    request
        .body
        .extend([cross, if option <= 2 { option } else { 0 }]);
    send(ctx, request, Outcome::reason(671, 1765))
}

/// parity: 672 25-arena-lobby-handlers.md#672--g_banishfromarenalobbyreqcmd--partial partial
pub fn handle_banish_from_arena_lobby(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    target_request(ctx, req, true)
}

/// parity: 675 25-arena-lobby-handlers.md#675--g_readytomatcharenalobbyroomreqcmd--partial partial
pub fn handle_ready_to_match_arena_lobby_room(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1788) {
        return ready_failure(ctx, reason);
    }
    if busy(ctx.pc()) {
        return match fatigue_leave(ctx) {
            Ok(true) => ready_failure(ctx, 1793),
            Ok(false) => ready_failure(ctx, 2),
            Err(out) => out,
        };
    }
    if let Some(reason) = room_reason(ctx.pc(), 3, 1789, 1790) {
        return ready_failure(ctx, reason);
    }
    let check = ArenaLobbyOperation::GameTimeValid {
        db_id: ctx.pc().db_id,
    };
    match boolean(ctx, check) {
        Ok(false) => return ready_failure(ctx, 1799),
        Ok(true) => {}
        Err(out) => return out,
    }
    let mut request = backend(ctx, 58, "hqqqhq");
    request.body.extend(ctx.pc().guild_id.to_le_bytes());
    match ctx
        .services
        .arena_lobby(&ArenaLobbyOperation::Send(request))
    {
        ArenaLobbyResult::Pass => completion(),
        ArenaLobbyResult::Fail => ready_failure(ctx, 1788),
        _ => unknown(),
    }
}

fn empty_room_request(ctx: &mut Context<'_>, command: u16, failure: u16, disabled: u16) -> Outcome {
    if let Some(reason) = mode_reason(ctx, disabled) {
        return Outcome::reason(failure, reason);
    }
    if let Some(reason) = room_reason(ctx.pc(), 3, disabled + 1, disabled + 2) {
        return Outcome::reason(failure, reason);
    }
    let request = backend(ctx, command, "hqqqh");
    send(ctx, request, Outcome::reason(failure, disabled))
}

/// parity: 678 25-arena-lobby-handlers.md#678--g_unreadytomatcharenalobbyroomreqcmd--partial partial
pub fn handle_unready_to_match_arena_lobby_room(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    empty_room_request(ctx, 62, 680, 1805)
}

/// parity: 681 25-arena-lobby-handlers.md#681--g_matcharenalobbyroomreqcmd--partial partial
pub fn handle_match_arena_lobby_room(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1810) {
        return Outcome::reason(683, reason);
    }
    let Some(mut flag) = Reader(req).byte() else {
        return Outcome::reason(683, 2);
    };
    if busy(ctx.pc()) {
        return match fatigue_leave(ctx) {
            Ok(true) => Outcome::reason(683, 1820),
            Ok(false) => Outcome::reason(683, 2),
            Err(out) => out,
        };
    }
    if let Some(reason) = room_reason(ctx.pc(), 3, 1811, 1812) {
        return Outcome::reason(683, reason);
    }
    let pc = ctx.pc();
    if pc.arena_lobby.kind == 2 && (4..=6).contains(&pc.arena_lobby.attraction) {
        let check = ArenaLobbyOperation::SpecialMatchEnabled {
            attraction: pc.arena_lobby.attraction,
        };
        match boolean(ctx, check) {
            Ok(false) => return Outcome::reason(683, 1816),
            Ok(true) => flag = 0,
            Err(out) => return out,
        }
    }
    let mut request = backend(ctx, 66, "hqqqhcq");
    request.body.push(flag);
    request.body.extend(ctx.pc().guild_id.to_le_bytes());
    send(ctx, request, Outcome::reason(683, 1810))
}

/// parity: 684 25-arena-lobby-handlers.md#684--g_skiparenalobbymatchreqcmd--partial partial
pub fn handle_skip_arena_lobby_match(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1834) {
        return Outcome::reason(686, reason);
    }
    // Wire h: the body-only dispatcher cannot express native header decode failure.
    if busy(ctx.pc()) {
        return match fatigue_leave(ctx) {
            Ok(true) => Outcome::reason(686, 1845),
            Ok(false) => Outcome::reason(686, 2),
            Err(out) => out,
        };
    }
    if let Some(reason) = room_reason(ctx.pc(), 3, 1835, 1836) {
        return Outcome::reason(686, reason);
    }
    let request = backend(ctx, 74, "hqqqh");
    send(ctx, request, Outcome::reason(686, 1834))
}

/// parity: 687 25-arena-lobby-handlers.md#687--g_cancelarenalobbymatchingreqcmd--partial partial
pub fn handle_cancel_arena_lobby_matching(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1849) {
        return Outcome::reason(689, reason);
    }
    let Some(flag) = Reader(req).byte() else {
        return Outcome::reason(689, 2);
    };
    if let Some(reason) = room_reason(ctx.pc(), 3, 1850, 1851) {
        return Outcome::reason(689, reason);
    }
    let mut request = backend(ctx, 78, "hqqqhc");
    request.body.push(flag);
    send(ctx, request, Outcome::reason(689, 1849))
}

/// parity: 690 25-arena-lobby-handlers.md#690--g_cancelarenalobbymatchreqcmd--partial partial
pub fn handle_cancel_arena_lobby_match(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    empty_room_request(ctx, 82, 692, 1854)
}

/// parity: 693 25-arena-lobby-handlers.md#693--g_readytogotoarenareqcmd--partial partial
pub fn handle_ready_to_go_to_arena(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1859) {
        return Outcome::reason(695, reason);
    }
    let Some(name) = Reader(req).string(33) else {
        return Outcome::reason(695, 2);
    };
    if let Some(reason) = room_reason(ctx.pc(), 3, 1860, 1862) {
        return Outcome::reason(695, reason);
    }
    let mut request = backend(ctx, 85, "hqqqhs");
    string_body(&mut request.body, &name);
    send(ctx, request, Outcome::reason(695, 1859))
}

/// parity: 696 25-arena-lobby-handlers.md#696--g_decidetogotoarenareqcmd--partial partial
pub fn handle_decide_to_go_to_arena(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1863) {
        return Outcome::reason(698, reason);
    }
    // Descriptor aHq is hQ, not hq. Only the low byte reaches backend 89.
    let Some(decision) = Reader(req).byte() else {
        return Outcome::reason(698, 2);
    };
    if let Some(reason) = room_reason(ctx.pc(), 3, 1864, 1866) {
        return Outcome::reason(698, reason);
    }
    let pc = ctx.pc();
    let mut body = pc.object_id.to_le_bytes().to_vec();
    body.extend(pc.db_id.to_le_bytes());
    body.extend(pc.arena_lobby.route.to_le_bytes());
    body.extend(pc.short_96.to_le_bytes());
    body.push(decision);
    let request = ArenaLobbyBackend {
        world: pc.short_96,
        command: 89,
        descriptor: "hqqqhQ",
        body,
    };
    send(ctx, request, Outcome::reason(698, 1863))
}

/// parity: 699 25-arena-lobby-handlers.md#699--g_retrievearenalobbywaitingnumberreqcmd--partial partial
pub fn handle_retrieve_arena_lobby_waiting_number(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if mode_reason(ctx, 0).is_some() {
        return Outcome::log("arena waiting number: arena disabled or server mode");
    }
    if ctx.pc().arena_state != 5 {
        return Outcome::log("arena waiting number: invalid state");
    }
    let now = ctx.world.tick;
    let last = ctx.pc().arena_lobby.waiting_tick;
    if last != 0 && (now.wrapping_sub(last) as i64) < 30_000 {
        return Outcome::log("arena waiting number: too frequent");
    }
    let pc = ctx.session.pc.as_mut().expect("PC dispatch");
    pc.arena_lobby.waiting_tick = now;
    if pc.arena_lobby.route == 0 || pc.arena_lobby.waiting_id == 0 {
        return Outcome::default();
    }
    let mut request = backend(ctx, 94, "hqqqhq");
    request
        .body
        .extend(ctx.pc().arena_lobby.waiting_id.to_le_bytes());
    // Native ignores send failure. The tick stays set, also on Unknown.
    send(ctx, request, Outcome::default())
}

/// parity: 702 25-arena-lobby-handlers.md#702--g_cancelarenalobbywaitingreqcmd--partial partial
pub fn handle_cancel_arena_lobby_waiting(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if mode_reason(ctx, 0).is_some() {
        return Outcome::log("arena cancel waiting: arena disabled or server mode");
    }
    if let Some(reason) = room_reason(ctx.pc(), 5, 1880, 1882) {
        return Outcome::reason(704, reason);
    }
    let mut request = backend(ctx, 97, "hqqqhq");
    request
        .body
        .extend(ctx.pc().arena_lobby.waiting_id.to_le_bytes());
    send(ctx, request, Outcome::reason(704, 1879))
}

fn party_decision(ctx: &mut Context<'_>, req: &[u8], final_decision: bool) -> Outcome {
    if let Some(reason) = mode_reason(ctx, 1691) {
        return Outcome::reason(641, reason);
    }
    let Some(value) = Reader(req).byte() else {
        return Outcome::reason(641, 2);
    };
    let Some(zone) = ctx.pc().zone else {
        return Outcome::reason(641, 2);
    };
    if !final_decision && busy(ctx.pc()) {
        return Outcome::reason(641, 1712);
    }
    let (command, kind) = if final_decision {
        (
            763,
            ArenaLobbyTask::PartyFinalDecision {
                decided: value != 0,
            },
        )
    } else {
        (761, ArenaLobbyTask::PartyAgreement { agreed: value != 0 })
    };
    let t = task(ctx, command, zone, kind);
    queue(ctx, t, Outcome::reason(641, 2))
}

/// parity: 761 25-arena-lobby-handlers.md#761--g_reservearenalobbypartyagreementreqcmd--partial partial
pub fn handle_reserve_arena_lobby_party_agreement(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    party_decision(ctx, req, false)
}

/// parity: 763 25-arena-lobby-handlers.md#763--g_reservearenalobbypartyfinaldecisionreqcmd--partial partial
pub fn handle_reserve_arena_lobby_party_final_decision(
    ctx: &mut Context<'_>,
    req: &[u8],
) -> Outcome {
    party_decision(ctx, req, true)
}
