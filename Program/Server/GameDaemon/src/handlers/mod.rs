//! Pure receive plans for build 381.8867, docs 04–07 and 10–11.
//! A queued wrapper is not completed travel or persistence. Unknown continuations
//! are explicit `Unsupported` results. Plans contain no sockets or secret state.
//! Restricted boundaries (comments only): 0x1403710A5 -> 0x14036DE10,
//! 0x140371AC8, 0x1403721D5, 0x1405DF767, 0x140394181,
//! 0x140394427 -> 0x14060E280, 0x1403895C7 -> 0x1403EAB50.
//! Private-object boundary: 0x1406E3100. No implementation.

mod guild;
mod movement;
mod party;
mod party_model;
mod party_tasks;
mod session;
pub use guild::*;
pub use party::*;
pub use party_model::*;
pub use party_tasks::execute_party_task;
mod challenge;
mod chat;
mod items;
mod items2;
mod items3;
mod items4;
mod items5;
mod market;
mod mentoring;
mod misc;
mod post;
mod quest;
mod quest2;
mod shop;
mod skills;
mod skills2;
mod social;
pub use challenge::*;
pub use chat::*;
pub use items2::*;
pub use items3::*;
pub use items4::*;
pub use items5::*;
pub use market::*;
pub use mentoring::*;
pub use misc::*;
pub use post::*;
pub use quest::*;
pub use quest2::*;
pub use shop::*;
pub use skills::*;
pub use skills2::*;
pub use social::*;
mod token;
mod trade;
pub use items::*;
pub use trade::*;
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
    /// Doc 13 1497: rate timestamp set before decoding.
    TraitTrainTick(u64),
    /// Doc 13 131: global DWORD `0x140FFDCDC` after the request.
    FactionStatisticsCalls(u32),
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
    PartyTaskQueued(PartyTask),
    /// Doc 15: a guild work object was posted to the guild manager.
    GuildWorkPosted(GuildWork),
    GuildTaskQueued(GuildTask),
    /// Doc 12: trade task allocated, initialized and queued.
    TradeTaskQueued(TradeTask),
    /// Doc 12b: item/equipment/craft task queued.
    ItemTaskQueued(ItemTask),
    /// Doc 12b 1521: craft record state set to 6 before backend 367.
    CraftAborting(u64),
    /// Doc 12: account-inventory transaction registered.
    AccountTransactionStarted(AccountTransaction),
    /// `0x1400F9B80` under the guild lock after a failed 506/508 post.
    GuildInviteeRemoved {
        guild_id: u64,
        db_id: u64,
    },
    /// `0x1400F37C0(guild, runtime ID, 2)` after a failed 499/501 post.
    GuildLeaderVoteCleanup {
        guild_id: u64,
        requester: u64,
    },
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
    /// Doc 12b 1521: backend command 367, `hqiiqqq`, via `0x14032AB10`.
    CraftAbort {
        /// PC +20664.
        routing: u64,
        /// PC signed short +96, promoted.
        short_96: i32,
        restriction: u32,
        object_id: u64,
        /// PC +20744.
        db_id: u64,
        craft_id: u64,
    },
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
    /// Command 493, hQqqs. Destination daemon and callback remain unread.
    PartyRegistration {
        world: u16,
        route: u32,
        db_id: u64,
        value: u64,
        name: Vec<u16>,
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
    /// Ordered party sends and state boundaries. Do not also send the summaries.
    pub party_effects: Vec<PartyEffect>,
    /// Ordered trade task sends and state effects.
    pub trade_effects: Vec<TradeEffect>,
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
            party_effects: vec![],
            trade_effects: vec![],
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
    /// PC word +20672. Model default 6 passes lookup mask 6 for existing fixtures.
    pub rights: u16,
    pub db_id: u64,
    pub name: Vec<u16>,
    /// PC +20752 and user +120, separate from user presence.
    pub party_user_blocked: bool,
    pub party_user_state: u32,
    pub team_member: bool,
    pub team_id: u64,
    pub party_registration: Option<PartyRegistration>,
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
    /// PC +336 guild ID; signed > 0 means in a guild.
    pub guild_id: u64,
    /// PC +20760 pending invitation guild ID.
    pub pending_guild_id: u64,
    /// PC +328 faction (signed byte).
    pub faction: i8,
    /// PC byte +13768 = 1: the T2 gate of spec 12 (reason 7).
    pub t2_blocked: bool,
    /// PC +20696 user object present and byte +20752 = 0 (doc 15 CS gate; logs "user is not found").
    pub user_present: bool,
    /// PC byte +344 ≠ 0 (461 → 1600, log "invalid activated faction"). ❓ Meaning not proven.
    pub uniform_faction_blocked: bool,
    /// PC +20880: tick of the last accepted 563/579 request.
    pub join_request_tick: u64,
    /// PC +20888: tick of the last accepted 603 request.
    pub guild_point_tick: u64,
    /// PC +890 bit 1: guild invitation refusal.
    pub guild_invitation_refusal: bool,
    /// PC signed short +96 (doc 05: runtime PC field, ❓ name). Backend 367 field 2.
    pub short_96: i16,
    /// Doc 13 1497: session object qword +8, last train-trait tick.
    pub trait_train_tick: u64,
    /// Doc 13: PC job byte +100 (630/633 require 6).
    pub job: u8,
    /// Doc 13 1065: current skill skin, PC short +776.
    pub current_skin: i16,
    /// Doc 16: PC DWORD +12560 (1 = chat allowed).
    pub chat_state: u32,
    /// Doc 16 1185: PC byte +13740 = 1 (trial jumping character).
    pub trial_jumping: bool,
    /// Doc 17: PC byte +36746 (mentoring type 0, 1 or 2).
    pub mentoring_type: u8,
    /// Doc 19 419: PC qword +20800, tick of the last accepted bot report.
    pub bot_report_tick: u64,
    /// Doc 19 419: level byte of the `0x14060D810` PC snapshot (❓ level).
    pub level: i8,
}
impl Default for Pc {
    fn default() -> Self {
        Self {
            object_id: 0,
            rights: 6,
            db_id: 0,
            name: vec![],
            party_user_blocked: false,
            party_user_state: 0,
            team_member: false,
            team_id: 0,
            party_registration: None,
            world_id: 0,
            routing: 0,
            zone: None,
            native_kind: 1,
            mentoring_type: 0,
            bot_report_tick: 0,
            level: 0,
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
            guild_id: 0,
            pending_guild_id: 0,
            faction: 0,
            t2_blocked: false,
            user_present: false,
            uniform_faction_blocked: false,
            join_request_tick: 0,
            guild_point_tick: 0,
            guild_invitation_refusal: false,
            short_96: 0,
            trait_train_tick: 0,
            job: 0,
            current_skin: 0,
            chat_state: 0,
            trial_jumping: false,
        }
    }
}
#[derive(Clone, Debug, Default)]
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
    pub party_category: u32,
    pub party_subtype: u32,
    pub attraction_kind: Option<u32>,
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
    pub party_config: PartyConfig,
    pub arena: bool,
    pub game_log_enabled: bool,
    pub team_enabled: bool,
    pub account_inventory_enabled: bool,
    /// Daemon +2744 byte +909 ≠ 0: item lock feature (doc 12b 1414–1418).
    pub item_lock_enabled: bool,
    /// Daemon +2744 byte +935 ≠ 0: unseal feature (doc 12b 1383).
    pub unseal_enabled: bool,
    /// Doc 12c: global byte +2088 and cfg byte +404 both ≠ 0 (BoardGacha).
    pub board_gacha_enabled: bool,
    /// Doc 12c: global bytes +2040 and +2008 both ≠ 0 (badge).
    pub badge_enabled: bool,
    /// Doc 12c: signed global DWORD +2092 (slate page count).
    pub slate_pages: i32,
    /// Doc 12c: global byte +1992 ≠ 0 (830 ability passing).
    pub ability_passing_enabled: bool,
    /// Doc 12c: global byte +208 ≠ 0 (947 weapon appearance).
    pub weapon_appearance_enabled: bool,
    /// Doc 12c: global byte +1981 ≠ 0 (1050 custom dress).
    pub custom_dress_enabled: bool,
    /// Doc 12c: cfg byte +720 ≠ 0 (1402, 1405, 1408 growth/awakening/feeding).
    pub item_growth_enabled: bool,
    /// Doc 13: rule config daemon +2744 byte +1045 ≠ 0 (824).
    pub vitality_recharge_enabled: bool,
    /// Doc 13: rule config qword +1264, 1497 rate interval (ticks).
    pub trait_train_interval: u64,
    /// Doc 13: global config byte +2105 ≠ 0 (125/127/129).
    pub faction_enabled: bool,
    /// Doc 13: global config byte +2001 ≠ 0 (1266).
    pub fish_enabled: bool,
    /// Doc 13: global config byte +200 ≠ 0 (633).
    pub summon_rename_enabled: bool,
    /// Doc 13: DataManager byte +82480 ≠ 0 (837/840).
    pub energy_enabled: bool,
    /// Doc 13 837: qword +40 of the DataManager slot-10309 object. None: not loaded.
    pub energy_multiple_mask: Option<u64>,
    /// Doc 13: global config byte +1980 ≠ 0 (1062/1065).
    pub skill_skin_enabled: bool,
    /// Doc 13 131: daemon qwords +6032/+6040/+6048, raw.
    pub faction_statistics: [u64; 3],
    /// Doc 16 51: rule config DWORD +216 ≠ 0 (chat without chat state 1).
    pub chat_rule: bool,
    /// Doc 16 55: rule config byte +1021 ≠ 0.
    pub item_message_enabled: bool,
    /// Doc 16 51/55: text filter service, daemon +2728 qword +48 ≠ 0.
    pub text_filter_present: bool,
    /// Doc 16 896/898/901/1041: cfg (daemon +2736) byte +436 ≠ 0.
    pub achievement_enabled: bool,
    /// Doc 16 1041: achievement service (daemon +5472) usable, `0x140021340`.
    pub achievement_service_usable: bool,
    /// Doc 16 898/901: achievement table by (id, step, job). None: not loaded.
    pub achievements: Option<BTreeMap<(u16, u16, u8), AchievementRecord>>,
    /// Doc 16 1044/1047: global byte +1946 ≠ 0.
    pub time_play_enabled: bool,
    /// Doc 16 1047: daemon +3200 + 160·type: (start +3216, end +3224) when set.
    pub time_play_schedules: [Option<(i64, i64)>; 2],
    /// Doc 17 mentoring fields.
    pub mentoring: MentoringWorld,
    /// Doc 19 misc fields.
    pub misc: MiscWorld,
    /// Doc 18: global byte +1982.
    pub challenge_party_enabled: bool,
    /// Doc 18: shared PC name lookup, daemon +5824.
    pub party: PartyState,
    /// Doc 13 131: global DWORD `0x140FFDCDC`.
    pub faction_statistics_calls: u32,
    /// cfg DWORD +248 when cfg byte +237 ≠ 0, else 0 (backend 367 restriction value).
    pub arena_value: u32,
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
    pub guild: GuildWorld,
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
    /// The adapter owns this state exclusively during each handler/task call.
    fn party_state(&mut self) -> Option<&mut PartyState> {
        None
    }
    /// None is an unread manager result, never an approval.
    fn party_operation(&mut self, _operation: PartyOperation) -> Option<i32> {
        None
    }
    /// Opaque 0x1401E0900 / 0x1401D4130. Do not infer leader selection.
    fn party_invite_leader(&mut self, _target: &Pc, _team: bool) -> PartyLeaderLookup {
        PartyLeaderLookup::Unknown
    }
    /// Allocation, PC-kind initialization, attachment, then queue/dispatch.
    /// The plan retains caster, zone and decoded task fields.
    fn queue_party(&mut self, task: &PartyTask) -> Gate {
        self.queue(task.command)
    }
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
    /// Doc 15 guild boundaries. None / Unknown is never approval.
    fn guild_post(&mut self, _work: &GuildWork) -> Gate {
        Gate::Unknown
    }
    fn queue_guild(&mut self, _task: &GuildTask) -> Gate {
        Gate::Unknown
    }
    /// Doc 12 trade task allocation, PC-kind init and queue.
    fn queue_trade(&mut self, _task: &TradeTask) -> Gate {
        Gate::Unknown
    }
    /// Adapter-owned trade state. Exclusive during one task call.
    fn trade_state(&mut self) -> Option<&mut TradeState> {
        None
    }
    /// Doc 12b item task allocation, PC-kind init and queue/dispatch.
    fn queue_item(&mut self, _task: &ItemTask) -> Gate {
        Gate::Unknown
    }
    /// Doc 13 skill task/action allocation, initialization and queue.
    fn queue_skill(&mut self, _task: &SkillTask) -> Gate {
        Gate::Unknown
    }
    /// Doc 13 manager slot 2379: Some(None) = no record, Some(Some(max level)).
    fn trait_max_level(&mut self, _id: u32) -> Option<Option<i8>> {
        None
    }
    /// Doc 12c BoardGacha manager submit (`0x1403FB2D0`) or user
    /// transaction registration (`0x1400D4FE0`).
    fn board_gacha(&mut self, _request: &BoardGachaRequest) -> Gate {
        Gate::Unknown
    }
    /// Doc 16 chatting-channel manager submit and petition backend requests.
    fn chat(&mut self, _request: &ChatRequest) -> Gate {
        Gate::Unknown
    }
    /// Doc 17 1103: registry checks (daemon +6200) for (mentor, mentee, maximum).
    fn mentoring_registry(&mut self, _mentor: u64, _mentee: u64, _max: i8) -> MentoringRegistry {
        MentoringRegistry::Unknown
    }
    /// Doc 24: market configuration, agent sends and account transactions.
    fn market(&mut self, _check: &MarketCheck) -> Gate {
        Gate::Unknown
    }
    /// Doc 23: cached post requests and account transactions. Unknown is never approval.
    fn post(&mut self, _operation: &PostOperation) -> PostResult {
        PostResult::Unknown
    }
    /// Doc 22: server-owned shop state and tables. Unknown is never approval.
    fn shop_check(&mut self, _check: &ShopCheck) -> Gate {
        Gate::Unknown
    }
    /// Doc 20: server-owned quest configuration. Unknown is never approval.
    fn quest_config(&mut self) -> QuestConfig {
        QuestConfig::Unknown
    }
    /// Doc 21: quest progress configuration, tables and name services.
    fn quest_progress(&mut self, _check: &QuestProgressCheck) -> Gate {
        Gate::Unknown
    }
    /// Doc 18: opaque season and name checks. Unknown is never approval.
    fn challenge_party(&mut self, _check: &ChallengeCheck) -> Gate {
        Gate::Unknown
    }
    /// `tm[603]` virtual +168 recipe lookup.
    fn craft_recipe_exists(&mut self, _recipe: i32) -> Option<bool> {
        None
    }
    /// PC craft record vector (+39760), exclusive under lock +39752.
    fn craft_records(&mut self) -> Option<&mut Vec<Option<CraftRecord>>> {
        None
    }
    /// Doc 12 account-inventory transaction registration (0x1400D4FE0).
    fn account_transaction(&mut self, _txn: &AccountTransaction) -> Gate {
        Gate::Unknown
    }
    /// UIC user lookup for the PC.
    fn guild_user_found(&mut self, _pc: &Pc) -> Option<bool> {
        None
    }
    /// Name filter (PC world +2728, virtual +48).
    fn guild_name_forbidden(&mut self, _name: &[u16]) -> Option<bool> {
        None
    }
    /// Rename type 2 time limit has passed.
    fn guild_rename_time_passed(&mut self, _guild_id: u64) -> Option<bool> {
        None
    }
    /// The PC holds a live guild-join renewal ticket.
    fn guild_join_ticket(&mut self, _db_id: u64) -> Option<bool> {
        None
    }
    /// Opaque `0x1400E6970`. Err carries the helper's reason output.
    fn guild_leave(&mut self, _guild_id: u64, _pc: &Pc) -> Option<Result<(), u16>> {
        None
    }
    /// Opaque `0x1400E5890`. Err carries the helper's reason output.
    fn guild_banish(
        &mut self,
        _guild_id: u64,
        _pc: &Pc,
        _index: u8,
        _name: &[u16],
    ) -> Option<Result<(), u16>> {
        None
    }
    /// Opaque `0x1400FF0D0`: 0 = success, else the reason to reply.
    fn guild_flag_bearer(
        &mut self,
        _guild_id: u64,
        _pc: &Pc,
        _index: u8,
        _name: &[u16],
    ) -> Option<u16> {
        None
    }
    /// 447 contract object + World +5080 lock. Fail = lock not taken.
    fn guild_craft_contract(&mut self, _contract: &GuildCraftContract) -> Gate {
        Gate::Unknown
    }
    /// Opaque `0x1400F6470`: 0 = success, else the reason.
    fn guild_craft_make(&mut self, _guild_id: u64, _db_id: u64, _value: u64) -> Option<u16> {
        None
    }
    /// Opaque `0x140105BA0`: 0 = success, else the reason.
    fn guild_craft_abandon(&mut self, _guild_id: u64, _pc: &Pc) -> Option<u16> {
        None
    }
    /// 461 name rules (`0x14012F9F0`) and name filter. false → 1602.
    fn guild_uniform_name_ok(&mut self, _name: &[u16]) -> Option<bool> {
        None
    }
    /// 461 `0x1400F73E0` then the transaction: 0 = success, else the reason
    /// (transaction register failure = 2).
    fn guild_uniform_contract(&mut self, _contract: &GuildUniformContract) -> Option<u16> {
        None
    }
    /// 465–473 opaque guild helper: 0 = success, else the reason.
    fn guild_uniform_op(
        &mut self,
        _command: u16,
        _guild_id: u64,
        _pc: &Pc,
        _a: u64,
        _b: u64,
    ) -> Option<u16> {
        None
    }
    /// `0x1400FCD00(guild, db_id, bit)`: the member holds the permission.
    fn guild_permission(&mut self, _guild_id: u64, _db_id: u64, _bit: u8) -> Option<bool> {
        None
    }
    fn queue(&mut self, command: u16) -> Gate;
    fn logout(&mut self) -> Gate;
    fn available_channel(&mut self, channel: i16) -> Gate;
    /// None means no attraction; Unknown means an unmapped virtual result.
    fn attraction_channel(&mut self, channel: i16) -> Option<Result<(), Option<u16>>>;
}
#[derive(Clone, Debug)]
pub struct MemoryServices {
    pub party: Option<PartyState>,
    pub party_result: Option<i32>,
    pub party_result_sequence: std::collections::VecDeque<Option<i32>>,
    pub party_leader: PartyLeaderLookup,
    pub team_leader: PartyLeaderLookup,
    pub party_calls: Vec<PartyOperation>,
    pub hello_result: HelloOperation,
    pub initial_world: Option<Box<packet::world_entry::EnterWorldOk>>,
    pub activated_teleport_result: PackedProperty,
    pub queue_result: Gate,
    pub logout_result: Gate,
    pub channel_result: Gate,
    pub attraction_result: Option<Result<(), Option<u16>>>,
    pub queued: Vec<u16>,
    pub logout_calls: usize,
    pub guild: MemoryGuild,
    pub trade: MemoryTrade,
    pub skill_queue_result: Gate,
    pub skill_tasks: Vec<SkillTask>,
    pub trait_levels: Option<BTreeMap<u32, i8>>,
    /// Doc 16 chat/petition boundary result and recorded requests.
    pub chat_result: Gate,
    pub chat_calls: Vec<ChatRequest>,
    pub mentoring_registry_result: MentoringRegistry,
    pub market: MemoryMarket,
    pub post: MemoryPost,
    pub shop: MemoryShop,
    pub quest: MemoryQuest,
    pub quest_progress: MemoryQuestProgress,
    pub challenge: MemoryChallenge,
}
/// Test/default doc 12 boundary results. Unknown is never approval.
#[derive(Clone, Debug)]
pub struct MemoryTrade {
    pub queue_result: Gate,
    pub account_result: Gate,
    pub state: Option<TradeState>,
    pub tasks: Vec<TradeTask>,
    pub transactions: Vec<AccountTransaction>,
    pub item_queue_result: Gate,
    pub item_tasks: Vec<ItemTask>,
    pub recipe_exists: Option<bool>,
    pub crafts: Option<Vec<Option<CraftRecord>>>,
    pub board_gacha_result: Gate,
    pub board_gacha: Vec<BoardGachaRequest>,
}
impl Default for MemoryTrade {
    fn default() -> Self {
        Self {
            queue_result: Gate::Unknown,
            account_result: Gate::Unknown,
            state: None,
            tasks: vec![],
            transactions: vec![],
            item_queue_result: Gate::Unknown,
            item_tasks: vec![],
            recipe_exists: None,
            crafts: None,
            board_gacha_result: Gate::Unknown,
            board_gacha: vec![],
        }
    }
}
/// Test/default guild boundary results.
#[derive(Clone, Debug)]
pub struct MemoryGuild {
    pub post_result: Gate,
    pub queue_result: Gate,
    pub user_found: Option<bool>,
    pub name_forbidden: Option<bool>,
    pub rename_time_passed: Option<bool>,
    pub join_ticket: Option<bool>,
    pub leave_result: Option<Result<(), u16>>,
    pub banish_result: Option<Result<(), u16>>,
    pub flag_bearer_result: Option<u16>,
    pub craft_contract_result: Gate,
    pub craft_make_result: Option<u16>,
    pub craft_abandon_result: Option<u16>,
    pub contracts: Vec<GuildCraftContract>,
    pub uniform_name_ok: Option<bool>,
    pub uniform_contract_result: Option<u16>,
    pub uniform_op_result: Option<u16>,
    pub uniform_contracts: Vec<GuildUniformContract>,
    pub uniform_ops: Vec<(u16, u64, u64)>,
    pub permission: Option<bool>,
    pub posted: Vec<GuildWork>,
    pub tasks: Vec<GuildTask>,
}
impl Default for MemoryGuild {
    fn default() -> Self {
        Self {
            post_result: Gate::Unknown,
            queue_result: Gate::Unknown,
            user_found: None,
            name_forbidden: None,
            rename_time_passed: None,
            join_ticket: None,
            leave_result: None,
            banish_result: None,
            flag_bearer_result: None,
            craft_contract_result: Gate::Unknown,
            craft_make_result: None,
            craft_abandon_result: None,
            uniform_name_ok: None,
            uniform_contract_result: None,
            uniform_op_result: None,
            uniform_contracts: vec![],
            uniform_ops: vec![],
            permission: None,
            contracts: vec![],
            posted: vec![],
            tasks: vec![],
        }
    }
}
impl Default for MemoryServices {
    fn default() -> Self {
        Self {
            party: None,
            party_result: None,
            party_result_sequence: std::collections::VecDeque::new(),
            party_leader: PartyLeaderLookup::Unknown,
            team_leader: PartyLeaderLookup::Unknown,
            party_calls: vec![],
            hello_result: HelloOperation::Unknown,
            initial_world: None,
            activated_teleport_result: PackedProperty::Unknown,
            queue_result: Gate::Unknown,
            logout_result: Gate::Unknown,
            channel_result: Gate::Unknown,
            attraction_result: None,
            queued: vec![],
            logout_calls: 0,
            guild: MemoryGuild::default(),
            trade: MemoryTrade::default(),
            skill_queue_result: Gate::Unknown,
            skill_tasks: vec![],
            trait_levels: None,
            chat_result: Gate::Unknown,
            chat_calls: vec![],
            mentoring_registry_result: MentoringRegistry::Unknown,
            market: MemoryMarket::default(),
            post: MemoryPost::default(),
            shop: MemoryShop::default(),
            quest: MemoryQuest::default(),
            quest_progress: MemoryQuestProgress::default(),
            challenge: MemoryChallenge::default(),
        }
    }
}
impl Services for MemoryServices {
    fn market(&mut self, check: &MarketCheck) -> Gate {
        self.market.check(check)
    }
    fn post(&mut self, operation: &PostOperation) -> PostResult {
        self.post.check(operation)
    }
    fn chat(&mut self, request: &ChatRequest) -> Gate {
        self.chat_calls.push(request.clone());
        self.chat_result
    }
    fn mentoring_registry(&mut self, _: u64, _: u64, _: i8) -> MentoringRegistry {
        self.mentoring_registry_result
    }
    fn shop_check(&mut self, check: &ShopCheck) -> Gate {
        self.shop.check(check)
    }
    fn quest_config(&mut self) -> QuestConfig {
        self.quest.config_reads += 1;
        self.quest
            .configs
            .pop_front()
            .unwrap_or_else(|| self.quest.config.clone())
    }
    fn challenge_party(&mut self, check: &ChallengeCheck) -> Gate {
        self.challenge.calls.push(check.clone());
        match check {
            ChallengeCheck::SeasonOpen => self.challenge.season,
            ChallengeCheck::NameValid { .. } => self.challenge.name_rules,
            ChallengeCheck::NameAllowed { .. } => self.challenge.name_filter,
        }
    }
    fn quest_progress(&mut self, check: &QuestProgressCheck) -> Gate {
        self.quest_progress.check(check)
    }
    fn queue_skill(&mut self, task: &SkillTask) -> Gate {
        if self.skill_queue_result == Gate::Pass {
            self.skill_tasks.push(task.clone());
        }
        self.skill_queue_result
    }
    fn trait_max_level(&mut self, id: u32) -> Option<Option<i8>> {
        Some(self.trait_levels.as_ref()?.get(&id).copied())
    }
    fn queue_trade(&mut self, task: &TradeTask) -> Gate {
        if self.trade.queue_result == Gate::Pass {
            self.trade.tasks.push(task.clone());
        }
        self.trade.queue_result
    }
    fn trade_state(&mut self) -> Option<&mut TradeState> {
        self.trade.state.as_mut()
    }
    fn board_gacha(&mut self, request: &BoardGachaRequest) -> Gate {
        if self.trade.board_gacha_result == Gate::Pass {
            self.trade.board_gacha.push(request.clone());
        }
        self.trade.board_gacha_result
    }
    fn queue_item(&mut self, task: &ItemTask) -> Gate {
        if self.trade.item_queue_result == Gate::Pass {
            self.trade.item_tasks.push(task.clone());
        }
        self.trade.item_queue_result
    }
    fn craft_recipe_exists(&mut self, _recipe: i32) -> Option<bool> {
        self.trade.recipe_exists
    }
    fn craft_records(&mut self) -> Option<&mut Vec<Option<CraftRecord>>> {
        self.trade.crafts.as_mut()
    }
    fn account_transaction(&mut self, txn: &AccountTransaction) -> Gate {
        if self.trade.account_result == Gate::Pass {
            self.trade.transactions.push(txn.clone());
        }
        self.trade.account_result
    }
    fn guild_post(&mut self, work: &GuildWork) -> Gate {
        self.guild.posted.push(work.clone());
        self.guild.post_result
    }
    fn queue_guild(&mut self, task: &GuildTask) -> Gate {
        self.guild.tasks.push(task.clone());
        self.guild.queue_result
    }
    fn guild_user_found(&mut self, _: &Pc) -> Option<bool> {
        self.guild.user_found
    }
    fn guild_name_forbidden(&mut self, _: &[u16]) -> Option<bool> {
        self.guild.name_forbidden
    }
    fn guild_rename_time_passed(&mut self, _: u64) -> Option<bool> {
        self.guild.rename_time_passed
    }
    fn guild_join_ticket(&mut self, _: u64) -> Option<bool> {
        self.guild.join_ticket
    }
    fn guild_leave(&mut self, _: u64, _: &Pc) -> Option<Result<(), u16>> {
        self.guild.leave_result
    }
    fn guild_banish(&mut self, _: u64, _: &Pc, _: u8, _: &[u16]) -> Option<Result<(), u16>> {
        self.guild.banish_result
    }
    fn guild_flag_bearer(&mut self, _: u64, _: &Pc, _: u8, _: &[u16]) -> Option<u16> {
        self.guild.flag_bearer_result
    }
    fn guild_craft_contract(&mut self, contract: &GuildCraftContract) -> Gate {
        self.guild.contracts.push(contract.clone());
        self.guild.craft_contract_result
    }
    fn guild_craft_make(&mut self, _: u64, _: u64, _: u64) -> Option<u16> {
        self.guild.craft_make_result
    }
    fn guild_craft_abandon(&mut self, _: u64, _: &Pc) -> Option<u16> {
        self.guild.craft_abandon_result
    }
    fn guild_uniform_name_ok(&mut self, _: &[u16]) -> Option<bool> {
        self.guild.uniform_name_ok
    }
    fn guild_uniform_contract(&mut self, contract: &GuildUniformContract) -> Option<u16> {
        self.guild.uniform_contracts.push(contract.clone());
        self.guild.uniform_contract_result
    }
    fn guild_permission(&mut self, _: u64, _: u64, _: u8) -> Option<bool> {
        self.guild.permission
    }
    fn guild_uniform_op(&mut self, command: u16, _: u64, _: &Pc, a: u64, b: u64) -> Option<u16> {
        self.guild.uniform_ops.push((command, a, b));
        self.guild.uniform_op_result
    }
    fn party_state(&mut self) -> Option<&mut PartyState> {
        self.party.as_mut()
    }
    fn party_operation(&mut self, operation: PartyOperation) -> Option<i32> {
        self.party_calls.push(operation);
        self.party_result_sequence
            .pop_front()
            .unwrap_or(self.party_result)
    }
    fn party_invite_leader(&mut self, _: &Pc, team: bool) -> PartyLeaderLookup {
        if team {
            self.team_leader
        } else {
            self.party_leader
        }
    }
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
    if !COMMANDS.contains(&command)
        && !PARTY_COMMANDS.contains(&command)
        && !GUILD_COMMANDS.contains(&command)
        && !ITEM_COMMANDS.contains(&command)
        && !ITEM2_COMMANDS.contains(&command)
        && !ITEM3_COMMANDS.contains(&command)
        && !ITEM4_COMMANDS.contains(&command)
        && !ITEM5_COMMANDS.contains(&command)
        && !ITEM6_COMMANDS.contains(&command)
        && !SKILL_COMMANDS.contains(&command)
        && !SKILL2_COMMANDS.contains(&command)
        && !CHAT_COMMANDS.contains(&command)
        && !QQ_COMMANDS.contains(&command)
        && !SOCIAL_COMMANDS.contains(&command)
        && !MENTORING_COMMANDS.contains(&command)
        && !MISC_COMMANDS.contains(&command)
        && !MARKET_COMMANDS.contains(&command)
        && !POST_COMMANDS.contains(&command)
        && !SHOP_COMMANDS.contains(&command)
        && !QUEST_COMMANDS.contains(&command)
        && !QUEST2_COMMANDS.contains(&command)
        && !CHALLENGE_COMMANDS.contains(&command)
    {
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
    if PARTY_COMMANDS.contains(&command) {
        return dispatch_party(ctx, command, req);
    }
    if GUILD_COMMANDS.contains(&command) {
        return guild::dispatch_guild(ctx, command, req);
    }
    if ITEM_COMMANDS.contains(&command) {
        return trade::dispatch_item(ctx, command, req);
    }
    if ITEM2_COMMANDS.contains(&command) {
        return items::dispatch_item2(ctx, command, req);
    }
    if ITEM3_COMMANDS.contains(&command) {
        return items2::dispatch_item3(ctx, command, req);
    }
    if ITEM4_COMMANDS.contains(&command) {
        return items3::dispatch_item4(ctx, command, req);
    }
    if ITEM5_COMMANDS.contains(&command) {
        return items4::dispatch_item5(ctx, command, req);
    }
    if ITEM6_COMMANDS.contains(&command) {
        return items5::dispatch_item6(ctx, command, req);
    }
    if SKILL_COMMANDS.contains(&command) {
        return skills::dispatch_skill(ctx, command, req);
    }
    if SKILL2_COMMANDS.contains(&command) {
        return skills2::dispatch_skill2(ctx, command, req);
    }
    if CHAT_COMMANDS.contains(&command) {
        return chat::dispatch_chat(ctx, command, req);
    }
    if MENTORING_COMMANDS.contains(&command) {
        return mentoring::dispatch_mentoring(ctx, command, req);
    }
    if MISC_COMMANDS.contains(&command) {
        return misc::dispatch_misc(ctx, command, req);
    }
    if MARKET_COMMANDS.contains(&command) {
        return market::dispatch_market(ctx, command, req);
    }
    if POST_COMMANDS.contains(&command) {
        return post::dispatch_post(ctx, command, req);
    }
    if SHOP_COMMANDS.contains(&command) {
        return shop::dispatch_shop(ctx, command, req);
    }
    if QUEST_COMMANDS.contains(&command) {
        return quest::dispatch_quest(ctx, command, req);
    }
    if QUEST2_COMMANDS.contains(&command) {
        return quest2::dispatch_quest2(ctx, command, req);
    }
    if CHALLENGE_COMMANDS.contains(&command) {
        return challenge::dispatch_challenge(ctx, command, req);
    }
    if SOCIAL_COMMANDS.contains(&command) {
        return social::dispatch_social(ctx, command, req);
    }
    if QQ_COMMANDS.contains(&command) {
        // parity: QQ group shares 0x1400DE550 with 887 (doc 16).
        return handle_qq_logout(ctx, req);
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
