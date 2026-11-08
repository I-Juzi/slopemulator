//! Pure receive plans for build 381.8867, docs 04–07 and 10–11.
//! A queued wrapper is not completed travel or persistence. Unknown continuations
//! are explicit `Unsupported` results. Plans contain no sockets or secret state.
//! Restricted boundaries (comments only): 0x1403710A5 -> 0x14036DE10,
//! 0x140371AC8, 0x1403721D5, 0x1405DF767, 0x140394181,
//! 0x140394427 -> 0x14060E280, 0x1403895C7 -> 0x1403EAB50.
//! Private-object boundary: 0x1406E3100. No implementation.

mod movement;
mod session;
mod token;
mod wire;
use crate::{kick::FailureAction, Position};
pub use movement::*;
pub use session::*;
use std::collections::{BTreeMap, BTreeSet};
pub use token::*;
use wire::Reader;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Handled,
    /// The native function returned false. Its close consequence is NOT known.
    NativeFalse,
    Unsupported(&'static str),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub command: u16,
    /// Body only, without size/command or encryption.
    pub body: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateChange {
    EnterWorldReceived {
        client_version: u32,
    },
    PingReceived([u64; 2]),
    Queued(u16),
    /// Dispatch retains the input for each queued task/action. No body is lost.
    QueuedRequest {
        command: u16,
        body: Vec<u8>,
    },
    ArenaReturned,
    LastChannelChange(u64),
    ContentsTick(u64),
    LobbyTokenAuthQueued(TokenAuthRequest),
    RemovePendingTokenUser(u64),
    TokenAuthFinished(u64),
}

/// Ordered session operations. The adapter must send before it switches processors.
/// No processor object, key, or encryption implementation is stored here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionEffect {
    SendWithCurrentProcessor(Message),
    /// Complete size/command/mark/body fragment. Do not add a second header.
    /// Apply the current session processor to each fragment in vector order.
    SendFragmentWithCurrentProcessor(Vec<u8>),
    ActivateSessionProcessor,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Report {
    AutoLogoutCheck { detail: u16 },
    ClientRun { bytes: [u8; 2] },
}
impl Report {
    pub fn record_id(&self) -> u16 {
        match self {
            Self::AutoLogoutCheck { .. } => 1009,
            Self::ClientRun { .. } => 1042,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DaemonRequest {
    /// Lobby command 5, hqqqi. The world selects the route, not a wire field.
    LobbyConsumeToken {
        world: u16,
        transaction_id: u64,
        client_key: u64,
        lobby_token: u64,
        accepted_address_bits: u32,
    },
    LobbyClean {
        world: u16,
        db_id: u64,
    },
    /// Cached command 271, hQqq; routing is not a wire field.
    ReturnToGame {
        routing: u64,
        db_id: u64,
        object_id: u64,
    },
    /// Cached command 17, hQCqqchb; (one record). Completion is unknown.
    SaveClientOption {
        db_id: u64,
        option_type: u8,
        data: Vec<u8>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub status: Status,
    pub replies: Vec<Message>,
    /// No complete movement broadcast framing is specified. Never invent one.
    pub broadcasts: Vec<Message>,
    pub changes: Vec<StateChange>,
    pub daemon_requests: Vec<DaemonRequest>,
    pub kicks: Vec<FailureAction>,
    pub logs: Vec<&'static str>,
    /// Redacted log records only. These are not game packets or archive codecs.
    pub reports: Vec<Report>,
    pub session_effects: Vec<SessionEffect>,
}
impl Default for Outcome {
    fn default() -> Self {
        Self {
            status: Status::Handled,
            replies: vec![],
            broadcasts: vec![],
            changes: vec![],
            daemon_requests: vec![],
            kicks: vec![],
            logs: vec![],
            reports: vec![],
            session_effects: vec![],
        }
    }
}
impl Outcome {
    fn unsupported(why: &'static str) -> Self {
        Self {
            status: Status::Unsupported(why),
            ..Self::default()
        }
    }
    fn reason(command: u16, reason: u16) -> Self {
        Self {
            replies: vec![Message {
                command,
                body: reason.to_le_bytes().to_vec(),
            }],
            ..Self::default()
        }
    }
    fn reply(command: u16, body: Vec<u8>) -> Self {
        Self {
            replies: vec![Message { command, body }],
            ..Self::default()
        }
    }
    fn log(text: &'static str) -> Self {
        Self {
            logs: vec![text],
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Session {
    pub authenticated: bool,
    pub pc: Option<Pc>,
    pub contents_last_tick: Option<u64>,
    /// Native session +256. None means that the adapter has not supplied it.
    pub hello_mode: Option<u32>,
    /// The adapter must resolve rights/product checks. None is not approval.
    pub dispatch_permission: Option<bool>,
}
#[derive(Clone, Debug)]
pub struct Pc {
    pub object_id: u64,
    pub db_id: u64,
    pub world_id: u16,
    pub routing: u64,
    pub zone: Option<u64>,
    pub native_kind: u8,
    pub position: Position,
    pub yaw: u16,
    pub user_available: bool,
    pub arena_state: u8,
    pub arena_qwords: [u64; 2],
    pub received_enter_world: bool,
    pub client_version: u32,
    pub received_ping: [u64; 2],
    pub achievement_reward: bool,
    pub jumping_ticket: bool,
    pub instant_payment: bool,
    pub combat: bool,
    pub builder: bool,
    pub ignore_channel_cooldown: bool,
    pub last_channel_change: u64,
    pub summon: Option<Summon>,
    pub party_id: u64,
    /// None: action lookup has not run. Some(false): verified missing action.
    pub spawn_action_present: Option<bool>,
    pub life_state: u8,
    pub spawn_state: u8,
    pub spawn_correlation: u32,
}
impl Default for Pc {
    fn default() -> Self {
        Self {
            object_id: 0,
            db_id: 0,
            world_id: 0,
            routing: 0,
            zone: None,
            native_kind: 1,
            position: Position::default(),
            yaw: 0,
            user_available: false,
            arena_state: 0,
            arena_qwords: [0; 2],
            received_enter_world: false,
            client_version: 0,
            received_ping: [0; 2],
            achievement_reward: false,
            jumping_ticket: false,
            instant_payment: false,
            combat: false,
            builder: false,
            ignore_channel_cooldown: false,
            last_channel_change: 0,
            summon: None,
            party_id: 0,
            spawn_action_present: None,
            life_state: 0,
            spawn_state: 0,
            spawn_correlation: 0,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Summon {
    pub control_state: u8,
    pub native_kind: u8,
    pub race: u8,
    pub has_entry: bool,
    pub zone: Option<u64>,
    pub position: Position,
    pub yaw: u16,
}
/// Zone +16224 and its record category, resolved from server-owned data.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum DirectEntryGeo {
    #[default]
    Unknown,
    Missing,
    MissingRecord,
    Category(u8),
}
#[derive(Clone, Debug, Default)]
pub struct Zone {
    pub channel: i16,
    pub direct_entry_geo: DirectEntryGeo,
    pub channel_list: Option<(u16, [u8; 51])>,
    pub forbidden_channel_destination: bool,
    pub zone_type: u16,
    pub portal_subtype: Option<u16>,
    pub environment_index: i16,
    pub life_state: u8,
}
#[derive(Clone, Debug, Default)]
pub struct World {
    pub arena: bool,
    pub game_log_enabled: bool,
    pub team_enabled: bool,
    pub account_inventory_enabled: bool,
    pub channel_type_87_restricted: bool,
    pub tick: u64,
    pub utc_seconds: i64,
    pub config_reload_tick: u64,
    pub worlds: BTreeSet<u16>,
    pub zones: BTreeMap<u64, Zone>,
    pub publisher_code: u8,
    pub publisher: Vec<u16>,
    pub jumping_ticket_ids: BTreeSet<u32>,
    pub additional_service_ids: BTreeSet<u64>,
    pub qq_interface_present: bool,
    pub switches: BTreeMap<(u64, u16), TeleportSwitch>,
    pub teleports: BTreeMap<u64, TeleportRecord>,
    pub air_dash: BTreeMap<(u64, i32), AirDashEnvironment>,
}
#[derive(Clone, Debug)]
pub struct TeleportSwitch {
    pub position: Position,
    pub radius: i16,
    pub teleport_key: u64,
}
#[derive(Clone, Debug)]
pub struct TeleportRecord {
    pub id: i16,
    pub disabled: bool,
}
#[derive(Clone, Debug)]
pub struct AirDashEnvironment {
    pub subtype: u16,
    pub required_environment: i16,
    pub enabled: bool,
    pub helper_65002: Gate,
    pub faction_requirement: i64,
    pub pc_faction: i8,
    pub requirement_92: i32,
    pub pc_value_552: i32,
    pub zone_helper: Option<Gate>,
}

/// Server-owned task/transaction boundary, not a fabricated backend result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    Pass,
    Fail,
    Unknown,
}
/// Results from opaque, server-owned operations. Unknown is never approval.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum HelloOperation {
    #[default]
    Unknown,
    Rejected,
    Reply(Vec<u8>),
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum PackedProperty {
    #[default]
    Unknown,
    /// Exact bytes returned by the native packed-property operation.
    Bytes(Vec<u8>),
    FetchFailed,
}
pub trait Services {
    /// Opaque boundary 0x1406A65D0. Do not implement key setup here.
    fn hello(&mut self, _input: &[u8]) -> HelloOperation {
        HelloOperation::Unknown
    }
    /// Server-owned, complete command-32 input for this PC. None is not approval.
    /// This supplies data only; it does not complete post-32 sync or travel.
    fn initial_world(&mut self, _pc: &Pc) -> Option<Box<packet::world_entry::EnterWorldOk>> {
        None
    }
    fn activated_teleport(&mut self) -> PackedProperty {
        PackedProperty::Unknown
    }
    fn queue(&mut self, command: u16) -> Gate;
    fn logout(&mut self) -> Gate;
    fn available_channel(&mut self, channel: i16) -> Gate;
    /// None means no attraction; Unknown means an unmapped virtual result.
    fn attraction_channel(&mut self, channel: i16) -> Option<Result<(), Option<u16>>>;
}
#[derive(Clone, Debug)]
pub struct MemoryServices {
    pub hello_result: HelloOperation,
    pub initial_world: Option<Box<packet::world_entry::EnterWorldOk>>,
    pub activated_teleport_result: PackedProperty,
    pub queue_result: Gate,
    pub logout_result: Gate,
    pub channel_result: Gate,
    pub attraction_result: Option<Result<(), Option<u16>>>,
    pub queued: Vec<u16>,
    pub logout_calls: usize,
}
impl Default for MemoryServices {
    fn default() -> Self {
        Self {
            hello_result: HelloOperation::Unknown,
            initial_world: None,
            activated_teleport_result: PackedProperty::Unknown,
            queue_result: Gate::Unknown,
            logout_result: Gate::Unknown,
            channel_result: Gate::Unknown,
            attraction_result: None,
            queued: vec![],
            logout_calls: 0,
        }
    }
}
impl Services for MemoryServices {
    fn hello(&mut self, _: &[u8]) -> HelloOperation {
        self.hello_result.clone()
    }
    fn initial_world(&mut self, _: &Pc) -> Option<Box<packet::world_entry::EnterWorldOk>> {
        self.initial_world.take()
    }
    fn activated_teleport(&mut self) -> PackedProperty {
        self.activated_teleport_result.clone()
    }
    fn queue(&mut self, command: u16) -> Gate {
        self.queued.push(command);
        self.queue_result
    }
    fn logout(&mut self) -> Gate {
        self.logout_calls += 1;
        self.logout_result
    }
    fn available_channel(&mut self, _: i16) -> Gate {
        self.channel_result
    }
    fn attraction_channel(&mut self, _: i16) -> Option<Result<(), Option<u16>>> {
        self.attraction_result
    }
}
pub struct Context<'a> {
    pub session: &'a mut Session,
    pub world: &'a World,
    pub services: &'a mut dyn Services,
}
impl Context<'_> {
    fn pc(&self) -> &Pc {
        self.session.pc.as_ref().expect("PC dispatch precondition")
    }
    fn queue(&mut self, command: u16, failure: u16, reason: u16, todo: &'static str) -> Outcome {
        match self.services.queue(command) {
            Gate::Fail => Outcome::reason(failure, reason),
            Gate::Unknown => {
                Outcome::unsupported("TODO ❓ task allocation/initialization/queue adapter")
            }
            Gate::Pass => {
                let mut out = Outcome::unsupported(todo);
                out.changes.push(StateChange::Queued(command));
                out
            }
        }
    }
}

pub const COMMANDS: [u16; 43] = [
    11, 13, 16, 18, 19, 22, 23, 26, 29, 31, 34, 38, 42, 64, 66, 99, 102, 145, 174, 262, 296, 298,
    300, 303, 305, 307, 411, 414, 416, 422, 648, 809, 812, 887, 929, 981, 984, 1149, 1210, 1219,
    1221, 1223, 1239,
];

/// Dispatch body bytes only. Framing is the caller's separate responsibility.
/// Unknown commands log only (04/09, 0x14007BC00). No generic kick.
pub fn dispatch(ctx: &mut Context<'_>, command: u16, req: &[u8]) -> Outcome {
    let mut out = dispatch_inner(ctx, command, req);
    let inputs: Vec<_> = out
        .changes
        .iter()
        .filter_map(|change| match change {
            StateChange::Queued(task) => Some(StateChange::QueuedRequest {
                command: *task,
                body: req.to_vec(),
            }),
            _ => None,
        })
        .collect();
    out.changes.extend(inputs);
    out
}

// Individual handlers require dispatch's PC/rights preconditions. They return
// inert plans; dispatch retains raw request input for queued continuations.
fn dispatch_inner(ctx: &mut Context<'_>, command: u16, req: &[u8]) -> Outcome {
    if !COMMANDS.contains(&command) {
        return Outcome::log("unknown command");
    }
    let any = matches!(command, 16 | 929);
    let unauth = matches!(command, 11 | 13 | 19 | 22 | 23);
    if !any {
        if unauth {
            if ctx.session.authenticated {
                return Outcome::log("unknown command in PC table");
            }
        } else {
            if !ctx.session.authenticated || ctx.session.pc.is_none() {
                return Outcome::log("missing authenticated PC");
            }
            match ctx.session.dispatch_permission {
                Some(true) => {}
                // ❓ Exact rights/product rejection consequences are not specified.
                _ => {
                    return Outcome::unsupported(
                        "TODO ❓ packet rights/product-type dispatch gates, 0x14007BC00",
                    )
                }
            }
        }
    }
    match command {
        11 => handle_hello(ctx, req),
        13 => handle_mobile_hello(ctx, req),
        16 => handle_ping(ctx, req),
        18 => handle_server_ping(ctx, req),
        19 => handle_game_key_auth(ctx, req),
        22 => handle_np_token_auth(ctx, req),
        23 => handle_lobby_token_auth(ctx, req),
        26 => handle_return_to_lobby(ctx, req),
        29 => handle_clean_user_connection_info_in_cligate(ctx, req),
        31 => handle_enter_world(ctx, req),
        34 => handle_query_waiting_number(ctx, req),
        38 => handle_leave_world(ctx, req),
        42 => handle_enter_zone(ctx, req),
        64 => handle_move(ctx, req),
        66 => handle_turn(ctx, req),
        99 => handle_activated_teleport_list(ctx, req),
        102 => handle_detect_teleport_switch(ctx, req),
        145 => handle_return_to_base_camp(ctx, req),
        174 => handle_clear_logout_party_member(ctx, req),
        262 => handle_clear_logout_team_member(ctx, req),
        296 => handle_enter_portal(ctx, req),
        298 => handle_turn_back_from_portal(ctx, req),
        300 => handle_channel_list(ctx, req),
        303 => handle_change_channel(ctx, req),
        305 => handle_save_client_option(ctx, req),
        307 => handle_client_config(ctx, req),
        411 => handle_auto_logout_check(ctx, req),
        414 => handle_check_round_trip_time(ctx, req),
        416 => handle_write_latency_log(ctx, req),
        422 => handle_current_utc_time(ctx, req),
        648 => handle_return_to_game(ctx, req),
        809 => handle_extend_jumping_character_creation_quota_with_warehouse_item(ctx, req),
        812 => handle_upgrade_jumping_character_state_with_creation_quota(ctx, req),
        887 => handle_qq_logout(ctx, req),
        929 => handle_contents_rule_list(ctx, req),
        981 => handle_enter_zone_direct(ctx, req),
        984 => handle_leave_zone_direct(ctx, req),
        1149 => handle_use_jumping_character_ticket(ctx, req),
        1210 => handle_client_run(ctx, req),
        1219 => handle_air_dash(ctx, req),
        1221 => handle_warp_to_pos(ctx, req),
        1223 => handle_use_teleport(ctx, req),
        1239 => handle_use_cave_escape(ctx, req),
        _ => unreachable!("membership checked"),
    }
}
