//! Doc 15 guild receive plans (core group). A posted guild work object or a
//! queued task is not a completed guild operation; the guild manager answers
//! later (❓ unread). All replies are `hh` (command, reason) to self.
use super::*;

pub const GUILD_COMMANDS: &[u16] = &[
    431, 434, 437, 440, 443, 445, 447, 449, 451, 453, 455, 457, 459, 477, 480, 483, 486, 489, 491,
    494, 499, 501, 503, 506, 508, 509, 511, 513, 515,
];

/// Server config read by the guild handlers (`qword_140EE58A0`, World +6208).
#[derive(Clone, Debug, Default)]
pub struct GuildConfig {
    /// Config +1616 byte.
    pub open_limit_enabled: bool,
    /// Config +1620 dword.
    pub open_limit: u32,
    /// World +6208 dword.
    pub open_guild_count: u32,
    /// Config +2105 byte: PC faction must be > 0 to create a guild.
    pub faction_required: bool,
    /// Config +244 byte.
    pub rename_enabled: bool,
    /// Config +246 UTF-16 prefix for rename type 1. Empty = type 1 disabled.
    pub rename_prefix: Vec<u16>,
    /// Config +768 dword nonzero: rename type 2 (by item) enabled.
    pub rename_by_item: bool,
    /// Config +2041 byte = 1: guild-join renewal tickets are checked.
    pub join_renewal: bool,
    /// Config +776 byte = 1: delegate leader posts op 29, else 25.
    pub delegate_alt_op: bool,
    /// Config +2107 byte = 1: 443 replies reason 2 (❓ level-up disabled).
    pub level_up_disabled: bool,
    /// Config +2106 byte: guild bank deposit enabled.
    pub bank_enabled: bool,
    /// Config +64 signed dword: largest money deposit.
    pub bank_money_limit: i32,
    /// Permission mask limits: [normal (World +6264..), open (World +6304..)].
    pub permission_limits: [[u32; 4]; 2],
}
/// Guild +32416 type, +32418 name, +32771 rename-in-progress.
#[derive(Clone, Debug, Default)]
pub struct Guild {
    pub guild_type: u8,
    pub name: Vec<u16>,
    pub rename_in_progress: bool,
    /// Guild +32488: leader PC DB ID.
    pub leader_db_id: u64,
    /// A guild craft object exists (`0x140105E30` under lock +34456).
    pub craft_active: bool,
}
/// Guild action table row: byte +16 (normal guild) and +17 (open guild).
#[derive(Clone, Copy, Debug, Default)]
pub struct GuildActionRow {
    pub normal: bool,
    pub open: bool,
}
#[derive(Clone, Debug, Default)]
pub struct GuildWorld {
    pub config: GuildConfig,
    pub guilds: BTreeMap<u64, Guild>,
    /// Missing row = native null row = not enabled.
    pub actions: BTreeMap<u32, GuildActionRow>,
}
impl GuildWorld {
    fn action_enabled(&self, guild: &Guild, row: u32) -> bool {
        let Some(r) = self.actions.get(&row) else {
            return false;
        };
        match guild.guild_type {
            1 => r.normal,
            2 => r.open,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuildWorkKind {
    RetrieveInfo,
    RetrieveMembers,
    LevelUp,
    RetrievePermission,
    /// +412..+424.
    EditPermission {
        masks: [u32; 4],
    },
    /// +170, at most 121 units copied.
    EditNotice {
        text: Vec<u16>,
    },
    /// +160 PC DB ID.
    ConfirmInvitation {
        db_id: u64,
    },
    /// Requester field holds the PC DB ID for this op.
    RejectInvitation,
    ChangeGrade {
        index: u8,
        grade: u8,
        name: Vec<u16>,
    },
    DelegateLeader {
        alt: bool,
        index: u8,
        name: Vec<u16>,
    },
    RejectLeader,
    ConfirmLeader,
    /// Separate 0x90-byte join object (`0x140678250(obj, 42)`).
    Join {
        db_id: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuildWork {
    pub guild_id: u64,
    pub requester: u64,
    pub kind: GuildWorkKind,
}
impl GuildWork {
    /// WSO +140 operation. None for the separate join object.
    pub fn op(&self) -> Option<u32> {
        Some(match &self.kind {
            GuildWorkKind::RetrieveInfo => 2,
            GuildWorkKind::RetrieveMembers => 3,
            GuildWorkKind::LevelUp => 6,
            GuildWorkKind::RetrievePermission => 13,
            GuildWorkKind::EditPermission { .. } => 14,
            GuildWorkKind::EditNotice { .. } => 11,
            GuildWorkKind::ConfirmInvitation { .. } => 17,
            GuildWorkKind::RejectInvitation => 18,
            GuildWorkKind::ChangeGrade { .. } => 22,
            GuildWorkKind::DelegateLeader { alt: true, .. } => 29,
            GuildWorkKind::DelegateLeader { alt: false, .. } => 25,
            GuildWorkKind::RejectLeader => 26,
            GuildWorkKind::ConfirmLeader => 27,
            GuildWorkKind::Join { .. } => return None,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuildTaskKind {
    /// Register-guild task, vtable `off_140CF12B8`, task +68 = 101.
    Create {
        guild_type: u8,
        name: Vec<u16>,
        value: u64,
    },
    /// Rename task (`0x140543E50`). Items are (slot, count) records.
    Rename {
        rename_type: u8,
        name: Vec<u16>,
        items: Vec<(u16, u16)>,
    },
    /// Deposit task (`0x140540BF0`): +144 = 1, +148 money, (slot, amount) records.
    Deposit { money: i32, items: Vec<(u16, u16)> },
    /// 449 task (+68 = 167): +152 value.
    CraftAssent { value: u64 },
    /// 451 task (`0x14053C7D0`).
    CraftDonateNormal { value: u64, items: Vec<(u16, u16)> },
    /// 453 task (`0x14053D830`): +160 dword, +164 byte.
    CraftDonateBrand {
        value: u64,
        extra: u32,
        flag: u8,
        items: Vec<(u16, u16)>,
    },
    /// 457 task (`0x14027F720`): +152 value.
    CraftTake { value: u64 },
}
/// 447 decoded request. ❓ The native copy into the 0x158 object is not seen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuildCraftContract {
    pub guild_id: u64,
    pub requester: u64,
    pub amount: u32,
    pub value: u64,
    pub name: Vec<u16>,
    pub list_a: Vec<(u16, u16)>,
    pub list_b: Vec<(u16, u16)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuildTask {
    pub command: u16,
    pub caster: u64,
    pub zone: u64,
    pub guild_id: u64,
    pub kind: GuildTaskKind,
}

/// Name buffers in these handlers hold 33 UTF-16 units.
const NAME_UNITS: usize = 33;
/// ❓ Notice decode buffer size is not read; the copy keeps 121 units.
const NOTICE_UNITS: usize = 4000;

pub(super) fn dispatch_guild(ctx: &mut Context<'_>, command: u16, req: &[u8]) -> Outcome {
    match command {
        431 => handle_create_guild(ctx, req),
        434 => handle_rename_guild(ctx, req),
        437 => handle_destroy_guild(ctx, req),
        440 => handle_retrieve_guild_info(ctx, req),
        443 => handle_guild_level_up(ctx, req),
        445 => handle_deposit_in_guild_bank(ctx, req),
        483 => handle_retrieve_guild_member_permission(ctx, req),
        486 => handle_edit_guild_member_permission(ctx, req),
        489 => handle_change_guild_flag_bearer(ctx, req),
        447 => handle_contract_guild_craft(ctx, req),
        449 => handle_assent_guild_craft(ctx, req),
        451 => handle_donate_guild_craft_normal_material(ctx, req),
        453 => handle_donate_guild_craft_brand_material(ctx, req),
        455 => handle_make_guild_craft(ctx, req),
        457 => handle_take_guild_craft(ctx, req),
        459 => handle_abandon_guild_craft(ctx, req),
        477 => handle_retrieve_guild_members(ctx, req),
        480 => handle_edit_guild_notification_message(ctx, req),
        491 => handle_change_guild_member_grade(ctx, req),
        494 => handle_delegate_guild_leader(ctx, req),
        499 => handle_confirm_guild_leader(ctx, req),
        501 => handle_reject_guild_leader(ctx, req),
        503 => handle_invite_guild(ctx, req),
        506 => handle_confirm_invitation_guild(ctx, req),
        508 => handle_reject_invitation_guild(ctx, req),
        509 => handle_join_guild(ctx, req),
        511 => handle_leave_guild(ctx, req),
        513 => handle_banish_guild_member(ctx, req),
        515 => handle_query_guild_info(ctx, req),
        _ => unreachable!("guild command table"),
    }
}

/// Form A: World +2736 config (+237 byte and +248 dword).
fn arena_a(ctx: &Context<'_>) -> bool {
    ctx.world.arena
}
/// Form B: form A, or PC +37520 state 2, 3, 5 or 6.
fn arena_b(ctx: &Context<'_>) -> bool {
    ctx.world.arena || session::arena_blocked(ctx.pc())
}
fn guild_id(pc: &Pc) -> u64 {
    pc.guild_id
}
fn in_guild(pc: &Pc) -> bool {
    (pc.guild_id as i64) > 0
}
/// `Source[0] != 0`: the first unit is not NUL.
fn nonempty(name: &[u16]) -> bool {
    name.first().is_some_and(|u| *u != 0)
}
fn boundary(text: &'static str) -> Outcome {
    Outcome::unsupported(text)
}
/// Post a work object. Pass: no reply. Fail: the caller's failure.
fn post(ctx: &mut Context<'_>, work: GuildWork, failure: Outcome) -> Outcome {
    match ctx.services.guild_post(&work) {
        Gate::Pass => Outcome {
            changes: vec![StateChange::GuildWorkPosted(work)],
            ..Outcome::default()
        },
        Gate::Fail => failure,
        Gate::Unknown => boundary("❓ guild WSO allocation/post adapter"),
    }
}
/// Guild lookup + action row gate. None = reply reason 2.
fn guild_with_action(ctx: &Context<'_>, row: u32) -> Option<u64> {
    let id = guild_id(ctx.pc());
    let guild = ctx.world.guild.guilds.get(&id)?;
    ctx.world.guild.action_enabled(guild, row).then_some(id)
}
fn queue_task(ctx: &mut Context<'_>, task: GuildTask, reply: u16) -> Outcome {
    match ctx.services.queue_guild(&task) {
        Gate::Pass => Outcome {
            changes: vec![
                StateChange::Queued(task.command),
                StateChange::GuildTaskQueued(task),
            ],
            ..Outcome::default()
        },
        Gate::Fail => Outcome::reason(reply, 2),
        Gate::Unknown => boundary("❓ guild task allocation/attachment/queue adapter"),
    }
}

/// parity: 431 15-guild-handlers.md#431--g_createguildreqcmd--partial partial
pub fn handle_create_guild(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Register-guild task body, 432/433 from the task, DB procedure.
    if arena_b(ctx) {
        return Outcome::reason(433, 5);
    }
    let mut r = Reader(req);
    let (Some(guild_type), Some(name), Some(value)) = (r.byte(), r.string(NAME_UNITS), r.quad())
    else {
        return Outcome::reason(433, 1352);
    };
    if guild_type == 0 {
        return Outcome::reason(433, 1352);
    }
    let cfg = &ctx.world.guild.config;
    if guild_type == 2 && cfg.open_limit_enabled && cfg.open_guild_count >= cfg.open_limit {
        return Outcome::reason(433, 1352);
    }
    let pc = ctx.pc().clone();
    match ctx.services.guild_user_found(&pc) {
        None => return boundary("❓ guild UIC user lookup adapter"),
        Some(false) => return Outcome::reason(433, 2),
        Some(true) => {}
    }
    match ctx.services.guild_name_forbidden(&name) {
        None => return boundary("❓ guild name filter adapter"),
        Some(true) => return Outcome::reason(433, 1356),
        Some(false) => {}
    }
    if in_guild(&pc) {
        return Outcome::reason(433, 1358);
    }
    if ctx.world.guild.config.faction_required && pc.faction <= 0 {
        return Outcome::reason(433, 2);
    }
    let Some(zone) = pc.zone else {
        return Outcome::reason(433, 2);
    };
    if guild_type == 0 || !nonempty(&name) || value == 0 || pc.native_kind != 1 {
        return Outcome::reason(433, 2);
    }
    let task = GuildTask {
        command: 431,
        caster: pc.object_id,
        zone,
        guild_id: 0,
        kind: GuildTaskKind::Create {
            guild_type,
            name,
            value,
        },
    };
    queue_task(ctx, task, 433)
}

/// parity: 434 15-guild-handlers.md#434--g_renameguildreqcmd--partial partial
pub fn handle_rename_guild(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Rename task body, 435 notify, DB procedure.
    const R: u16 = 436;
    if arena_b(ctx) {
        return Outcome::reason(R, 5);
    }
    let cfg = ctx.world.guild.config.clone();
    if !cfg.rename_enabled {
        return Outcome::reason(R, 1363);
    }
    let id = guild_id(ctx.pc());
    if id == 0 {
        return Outcome::reason(R, 2);
    }
    let Some(guild) = ctx.world.guild.guilds.get(&id).cloned() else {
        return Outcome::reason(R, 2);
    };
    if !ctx.world.guild.action_enabled(&guild, 17) {
        return Outcome::reason(R, 2);
    }
    if guild.rename_in_progress {
        return Outcome::reason(R, 2);
    }
    let mut r = Reader(req);
    let (Some(rename_type), Some(name)) = (r.byte(), r.string(NAME_UNITS)) else {
        return Outcome::reason(R, 2);
    };
    // Missing count reads as 0xFFFF = -1.
    let count = r.short().unwrap_or(0xFFFF) as i16;
    if !(0..=16).contains(&count) {
        return Outcome::reason(R, 2);
    }
    let mut items = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (Some(slot), Some(n)) = (r.short(), r.short()) else {
            return Outcome::reason(R, 2);
        };
        if n == 0 {
            return Outcome::reason(R, 2);
        }
        items.push((slot, n));
    }
    match rename_type {
        1 => {
            if cfg.rename_prefix.is_empty() {
                return Outcome::reason(R, 1363);
            }
            if count != 0 {
                return Outcome::reason(R, 2);
            }
            // ❓ Exact compare loop (0x1403A2310): prefix match on the old name.
            if !guild.name.starts_with(&cfg.rename_prefix) {
                return Outcome::reason(R, 1368);
            }
        }
        2 => {
            if !cfg.rename_by_item {
                return Outcome::reason(R, 1363);
            }
            if count != 1 {
                return Outcome::reason(R, 2);
            }
            match ctx.services.guild_rename_time_passed(id) {
                None => return boundary("❓ guild rename time limit adapter"),
                Some(false) => return Outcome::reason(R, 1367),
                Some(true) => {}
            }
        }
        _ => return Outcome::reason(R, 2),
    }
    let pc = ctx.pc().clone();
    match ctx.services.guild_user_found(&pc) {
        None => return boundary("❓ guild UIC user lookup adapter"),
        Some(false) => return Outcome::reason(R, 2),
        Some(true) => {}
    }
    match ctx.services.guild_name_forbidden(&name) {
        None => return boundary("❓ guild name filter adapter"),
        Some(true) => return Outcome::reason(R, 1371),
        Some(false) => {}
    }
    let Some(zone) = pc.zone else {
        return Outcome::reason(R, 2);
    };
    if pc.native_kind != 1 {
        return Outcome::reason(R, 2);
    }
    let task = GuildTask {
        command: 434,
        caster: pc.object_id,
        zone,
        guild_id: id,
        kind: GuildTaskKind::Rename {
            rename_type,
            name,
            items,
        },
    };
    queue_task(ctx, task, R)
}

/// parity: 437 15-guild-handlers.md#437--g_destroyguildreqcmd--done ported
pub fn handle_destroy_guild(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if arena_b(ctx) {
        return Outcome::reason(439, 5);
    }
    let mut out = Outcome::reason(439, 1374);
    out.logs.push("destroy guild is disabled");
    out
}

/// parity: 440 15-guild-handlers.md#440--g_retrieveguildinforeqcmd--done ported
pub fn handle_retrieve_guild_info(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    simple_post(ctx, 442, 1384, GuildWorkKind::RetrieveInfo)
}
/// parity: 477 15-guild-handlers.md#477--g_retrieveguildmembersreqcmd--done ported
pub fn handle_retrieve_guild_members(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    simple_post(ctx, 479, 1385, GuildWorkKind::RetrieveMembers)
}
fn simple_post(ctx: &mut Context<'_>, reply: u16, no_guild: u16, kind: GuildWorkKind) -> Outcome {
    if arena_a(ctx) {
        return Outcome::reason(reply, 5);
    }
    let id = guild_id(ctx.pc());
    if id == 0 {
        return Outcome::reason(reply, no_guild);
    }
    let work = GuildWork {
        guild_id: id,
        requester: ctx.pc().object_id,
        kind,
    };
    post(ctx, work, Outcome::reason(reply, 2))
}

/// parity: 480 15-guild-handlers.md#480--g_editguildnotificationmessagereqcmd--done ported
pub fn handle_edit_guild_notification_message(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 481;
    if arena_a(ctx) {
        return Outcome::reason(R, 5);
    }
    let Some(mut text) = Reader(req).string(NOTICE_UNITS) else {
        return Outcome::reason(R, 1386);
    };
    if !nonempty(&text) {
        return Outcome::reason(R, 1386);
    }
    if guild_id(ctx.pc()) == 0 {
        return Outcome::reason(R, 1389);
    }
    let Some(id) = guild_with_action(ctx, 13) else {
        return Outcome::reason(R, 2);
    };
    text.truncate(121);
    let work = GuildWork {
        guild_id: id,
        requester: ctx.pc().object_id,
        kind: GuildWorkKind::EditNotice { text },
    };
    post(ctx, work, Outcome::reason(R, 2))
}

/// parity: 491 15-guild-handlers.md#491--g_changeguildmembergradereqcmd--done ported
pub fn handle_change_guild_member_grade(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 492;
    if arena_a(ctx) {
        return Outcome::reason(R, 5);
    }
    let mut r = Reader(req);
    let (Some(index), Some(name), Some(grade)) = (r.byte(), r.string(NAME_UNITS), r.byte()) else {
        return Outcome::reason(R, 1403);
    };
    if guild_id(ctx.pc()) == 0 {
        return Outcome::reason(R, 1405);
    }
    let Some(id) = guild_with_action(ctx, 3) else {
        return Outcome::reason(R, 2);
    };
    if !nonempty(&name) {
        return Outcome::reason(R, 2);
    }
    let work = GuildWork {
        guild_id: id,
        requester: ctx.pc().object_id,
        kind: GuildWorkKind::ChangeGrade { index, grade, name },
    };
    post(ctx, work, Outcome::reason(R, 2))
}

/// parity: 494 15-guild-handlers.md#494--g_delegateguildleaderreqcmd--done ported
pub fn handle_delegate_guild_leader(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 495;
    if arena_a(ctx) {
        return Outcome::reason(R, 5);
    }
    let mut r = Reader(req);
    let (Some(index), Some(name)) = (r.byte(), r.string(NAME_UNITS)) else {
        return Outcome::reason(R, 1409);
    };
    if guild_id(ctx.pc()) == 0 {
        return Outcome::reason(R, 1410);
    }
    let Some(id) = guild_with_action(ctx, 9) else {
        return Outcome::reason(R, 2);
    };
    if !nonempty(&name) {
        return Outcome::reason(R, 2);
    }
    let work = GuildWork {
        guild_id: id,
        requester: ctx.pc().object_id,
        kind: GuildWorkKind::DelegateLeader {
            alt: ctx.world.guild.config.delegate_alt_op,
            index,
            name,
        },
    };
    post(ctx, work, Outcome::reason(R, 2))
}

/// parity: 499 15-guild-handlers.md#499--g_confirmguildleaderreqcmd--done ported
pub fn handle_confirm_guild_leader(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    leader_vote(ctx, 500, GuildWorkKind::ConfirmLeader)
}
/// parity: 501 15-guild-handlers.md#501--g_rejectguildleaderreqcmd--done ported
pub fn handle_reject_guild_leader(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    leader_vote(ctx, 502, GuildWorkKind::RejectLeader)
}
fn leader_vote(ctx: &mut Context<'_>, reply: u16, kind: GuildWorkKind) -> Outcome {
    if arena_a(ctx) {
        return Outcome::reason(reply, 5);
    }
    let id = guild_id(ctx.pc());
    if id == 0 {
        return Outcome::reason(reply, 1410);
    }
    let requester = ctx.pc().object_id;
    let mut failure = Outcome::default();
    // 0x1400F37C0(guild, runtime ID, 2) runs only when the guild is found.
    if ctx.world.guild.guilds.contains_key(&id) {
        failure.changes.push(StateChange::GuildLeaderVoteCleanup {
            guild_id: id,
            requester,
        });
    }
    failure.replies.push(Message {
        command: reply,
        body: 2u16.to_le_bytes().to_vec(),
    });
    let work = GuildWork {
        guild_id: id,
        requester,
        kind,
    };
    post(ctx, work, failure)
}

/// parity: 503 15-guild-handlers.md#503--g_inviteguildreqcmd--done ported
pub fn handle_invite_guild(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if arena_b(ctx) {
        return Outcome::reason(504, 5);
    }
    let mut out = Outcome::reason(504, 1427);
    out.logs.push("invite guild is disabled");
    out
}

/// parity: 506 15-guild-handlers.md#506--g_confirminvitationguildreqcmd--done ported
pub fn handle_confirm_invitation_guild(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    const R: u16 = 507;
    if arena_a(ctx) {
        return Outcome::reason(R, 5);
    }
    let pending = ctx.pc().pending_guild_id;
    if pending == 0 {
        return Outcome::reason(R, 1440);
    }
    if !ctx.world.guild.guilds.contains_key(&pending) {
        return Outcome::reason(R, 1439);
    }
    let pc = ctx.pc().clone();
    let work = GuildWork {
        guild_id: pending,
        requester: pc.object_id,
        kind: GuildWorkKind::ConfirmInvitation { db_id: pc.db_id },
    };
    let failure = Outcome {
        changes: vec![StateChange::GuildInviteeRemoved {
            guild_id: pending,
            db_id: pc.db_id,
        }],
        ..Outcome::default()
    };
    let out = post(ctx, work, failure);
    if out
        .changes
        .iter()
        .any(|c| matches!(c, StateChange::GuildInviteeRemoved { .. }))
    {
        set_pending(ctx, 0);
    }
    out
}

/// parity: 508 15-guild-handlers.md#508--g_rejectinvitationguildreqcmd--done ported
pub fn handle_reject_invitation_guild(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if arena_a(ctx) {
        return Outcome::log("reject invitation guild: cannot process in arena");
    }
    let pending = ctx.pc().pending_guild_id;
    let db_id = ctx.pc().db_id;
    if pending == 0 || !ctx.world.guild.guilds.contains_key(&pending) {
        set_pending(ctx, 0);
        return Outcome::log("reject invitation guild fail, reset pending guild id");
    }
    let work = GuildWork {
        guild_id: pending,
        requester: db_id,
        kind: GuildWorkKind::RejectInvitation,
    };
    let failure = Outcome {
        changes: vec![StateChange::GuildInviteeRemoved {
            guild_id: pending,
            db_id,
        }],
        logs: vec!["reject invitation guild fail, reset pending guild id"],
        ..Outcome::default()
    };
    let out = post(ctx, work, failure);
    if !matches!(out.status, Status::Unsupported(_)) {
        set_pending(ctx, 0);
    }
    out
}
fn set_pending(ctx: &mut Context<'_>, value: u64) {
    if let Some(pc) = ctx.session.pc.as_mut() {
        pc.pending_guild_id = value;
    }
}

/// parity: 509 15-guild-handlers.md#509--g_joinguildreqcmd--done ported
pub fn handle_join_guild(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 510;
    if arena_b(ctx) {
        return Outcome::reason(R, 5);
    }
    let Some(target) = Reader(req).quad() else {
        return Outcome::reason(R, 1446);
    };
    if target == 0 {
        return Outcome::reason(R, 1447);
    }
    if in_guild(ctx.pc()) {
        return Outcome::reason(R, 1451);
    }
    let Some(guild) = ctx.world.guild.guilds.get(&target) else {
        return Outcome::reason(R, 1448);
    };
    if guild.guild_type != 2 {
        return Outcome::reason(R, 1449);
    }
    if !ctx.world.guild.action_enabled(guild, 1) {
        return Outcome::reason(R, 2);
    }
    let pc = ctx.pc().clone();
    if ctx.world.guild.config.join_renewal && pc.db_id != 0 {
        match ctx.services.guild_join_ticket(pc.db_id) {
            None => return boundary("❓ guild join ticket manager adapter"),
            Some(true) => return Outcome::reason(R, 1457),
            Some(false) => {}
        }
    }
    let work = GuildWork {
        guild_id: target,
        requester: pc.object_id,
        kind: GuildWorkKind::Join { db_id: pc.db_id },
    };
    post(ctx, work, Outcome::reason(R, 2))
}

/// parity: 511 15-guild-handlers.md#511--g_leaveguildreqcmd--partial partial
pub fn handle_leave_guild(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    // ❓ 0x1400E6970 removal, broadcasts, DB work.
    const R: u16 = 512;
    if arena_a(ctx) {
        return Outcome::reason(R, 5);
    }
    if guild_id(ctx.pc()) == 0 {
        return Outcome::reason(R, 1459);
    }
    let Some(id) = guild_with_action(ctx, 11) else {
        return Outcome::reason(R, 2);
    };
    let pc = ctx.pc().clone();
    match ctx.services.guild_leave(id, &pc) {
        None => boundary("❓ guild leave helper 0x1400E6970"),
        Some(Err(_reason)) => Outcome::reason(R, 2),
        Some(Ok(())) => Outcome::default(),
    }
}

/// parity: 513 15-guild-handlers.md#513--g_banishguildmemberreqcmd--partial partial
pub fn handle_banish_guild_member(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ 0x1400E5890 removal, broadcasts, DB work.
    const R: u16 = 514;
    if arena_a(ctx) {
        return Outcome::reason(R, 5);
    }
    let mut r = Reader(req);
    let (Some(index), Some(name)) = (r.byte(), r.string(NAME_UNITS)) else {
        return Outcome::reason(R, 1420);
    };
    if (index as i8) < 0 {
        return Outcome::reason(R, 1420);
    }
    if guild_id(ctx.pc()) == 0 {
        return Outcome::reason(R, 1422);
    }
    let Some(id) = guild_with_action(ctx, 12) else {
        return Outcome::reason(R, 2);
    };
    let pc = ctx.pc().clone();
    match ctx.services.guild_banish(id, &pc, index, &name) {
        None => boundary("❓ guild banish helper 0x1400E5890"),
        Some(Err(reason)) => Outcome::reason(R, reason),
        Some(Ok(())) => Outcome::default(),
    }
}

/// parity: 515 15-guild-handlers.md#515--g_queryguildinforeqcmd--partial partial
pub fn handle_query_guild_info(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Reply 516 body (cache branch, uniform records) is not specified.
    const R: u16 = 517;
    let mut r = Reader(req);
    let count = r.short().unwrap_or(0xFFFF);
    if count > 50 {
        return Outcome::reason(R, 2);
    }
    let mut ids = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let Some(id) = r.quad() else {
            return Outcome::reason(R, 2);
        };
        if id == 0 {
            return Outcome::reason(R, 2);
        }
        ids.push(id);
    }
    let _ = ctx;
    boundary("❓ reply 516 hiCqQsCc;; composition")
}

/// parity: 443 15-guild-handlers.md#443--g_guildlevelupreqcmd--done ported
pub fn handle_guild_level_up(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    const R: u16 = 444;
    if arena_b(ctx) {
        return Outcome::reason(R, 5);
    }
    if ctx.world.guild.config.level_up_disabled {
        return Outcome::reason(R, 2);
    }
    if guild_id(ctx.pc()) == 0 {
        return Outcome::reason(R, 1554);
    }
    let Some(id) = guild_with_action(ctx, 5) else {
        return Outcome::reason(R, 2);
    };
    let work = GuildWork {
        guild_id: id,
        requester: ctx.pc().object_id,
        kind: GuildWorkKind::LevelUp,
    };
    post(ctx, work, Outcome::reason(R, 2))
}

/// parity: 445 15-guild-handlers.md#445--g_depositinguildbankreqcmd--done ported
pub fn handle_deposit_in_guild_bank(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 446;
    let cfg = &ctx.world.guild.config;
    if !cfg.bank_enabled {
        return Outcome::reason(R, 2);
    }
    if arena_b(ctx) {
        return Outcome::reason(R, 5);
    }
    if ctx.pc().t2_blocked {
        return Outcome::reason(R, 7);
    }
    let mut r = Reader(req);
    let Some(money) = r.int().map(|v| v as i32) else {
        return Outcome::reason(R, 1553);
    };
    if money < 0 || money > cfg.bank_money_limit {
        return Outcome::reason(R, 33);
    }
    if guild_id(ctx.pc()) == 0 {
        return Outcome::reason(R, 2);
    }
    if guild_with_action(ctx, 6).is_none() {
        return Outcome::reason(R, 2);
    }
    // ❓ A missing count byte: the reader result is treated as a decode failure.
    let Some(count) = r.byte().map(|c| c as i8) else {
        return Outcome::reason(R, 1553);
    };
    if !(0..=5).contains(&count) || (money == 0 && count == 0) {
        return Outcome::reason(R, 1553);
    }
    if money > 0 && count > 0 {
        return Outcome::reason(R, 1579);
    }
    let mut items: Vec<(u16, u16)> = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (Some(slot), Some(amount)) = (r.short(), r.short()) else {
            return Outcome::reason(R, 1553);
        };
        if items.iter().any(|(s, _)| *s == slot) {
            return Outcome::reason(R, 1553);
        }
        items.push((slot, amount));
    }
    let pc = ctx.pc().clone();
    let Some(zone) = pc.zone else {
        return Outcome::reason(R, 2);
    };
    if pc.native_kind != 1 || items.iter().any(|(s, a)| *s == 0 || *a == 0) {
        return Outcome::reason(R, 2);
    }
    let task = GuildTask {
        command: 445,
        caster: pc.object_id,
        zone,
        guild_id: pc.guild_id,
        kind: GuildTaskKind::Deposit { money, items },
    };
    queue_shared(ctx, task, R)
}

/// parity: 483 15-guild-handlers.md#483--g_retrieveguildmemberpermissionreqcmd--done ported
pub fn handle_retrieve_guild_member_permission(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    const R: u16 = 485;
    if arena_a(ctx) {
        return Outcome::reason(R, 5);
    }
    let id = guild_id(ctx.pc());
    if id == 0 {
        return Outcome::reason(R, 1384);
    }
    if !ctx.world.guild.guilds.contains_key(&id) {
        return Outcome::reason(R, 2);
    }
    let work = GuildWork {
        guild_id: id,
        requester: ctx.pc().object_id,
        kind: GuildWorkKind::RetrievePermission,
    };
    post(ctx, work, Outcome::reason(R, 2))
}

/// parity: 486 15-guild-handlers.md#486--g_editguildmemberpermissionreqcmd--done ported
pub fn handle_edit_guild_member_permission(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 488;
    if arena_a(ctx) {
        return Outcome::reason(R, 5);
    }
    let mut r = Reader(req);
    let (Some(a), Some(b), Some(c), Some(d)) = (r.int(), r.int(), r.int(), r.int()) else {
        return Outcome::reason(R, 1395);
    };
    let masks = [a, b, c, d];
    if guild_id(ctx.pc()) == 0 {
        return Outcome::reason(R, 1398);
    }
    let Some(id) = guild_with_action(ctx, 2) else {
        return Outcome::reason(R, 2);
    };
    let open = ctx.world.guild.guilds[&id].guild_type == 2;
    let limits = ctx.world.guild.config.permission_limits[usize::from(open)];
    // ❓ Native compare is `limit < (limit | mask)`, possibly signed.
    if masks.iter().zip(limits).any(|(m, l)| m & !l != 0) {
        return Outcome::reason(R, 1396);
    }
    let work = GuildWork {
        guild_id: id,
        requester: ctx.pc().object_id,
        kind: GuildWorkKind::EditPermission { masks },
    };
    post(ctx, work, Outcome::reason(R, 2))
}

/// parity: 489 15-guild-handlers.md#489--g_changeguildflagbearerreqcmd--partial partial
pub fn handle_change_guild_flag_bearer(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ 0x1400FF0D0 member lookup, broadcasts, DB work.
    const R: u16 = 490;
    if arena_b(ctx) {
        return Outcome::reason(R, 5);
    }
    if ctx.pc().t2_blocked {
        return Outcome::reason(R, 7);
    }
    let mut r = Reader(req);
    let (Some(index), Some(name)) = (r.byte(), r.string(NAME_UNITS)) else {
        return Outcome::reason(R, 1553);
    };
    if guild_id(ctx.pc()) == 0 {
        return Outcome::reason(R, 1554);
    }
    let Some(id) = guild_with_action(ctx, 4) else {
        return Outcome::reason(R, 2);
    };
    let pc = ctx.pc().clone();
    match ctx.services.guild_flag_bearer(id, &pc, index, &name) {
        None => boundary("❓ guild flag bearer helper 0x1400FF0D0"),
        Some(0) => Outcome::default(),
        Some(reason) => Outcome::reason(R, reason),
    }
}

/// Tasks queued with `0x140603BD0` (no failure branch). Gate::Fail stands for
/// the allocation failure, which replies reason 2.
fn queue_shared(ctx: &mut Context<'_>, task: GuildTask, reply: u16) -> Outcome {
    match ctx.services.queue_guild(&task) {
        Gate::Unknown => boundary("❓ guild task allocation adapter"),
        Gate::Fail => Outcome::reason(reply, 2),
        Gate::Pass => Outcome {
            changes: vec![
                StateChange::Queued(task.command),
                StateChange::GuildTaskQueued(task),
            ],
            ..Outcome::default()
        },
    }
}
/// `c` count (`(u16)count > 5` rejects) then (slot, amount) records.
fn craft_records(r: &mut Reader<'_>) -> Option<Vec<(u16, u16)>> {
    let count = r.byte()? as i8;
    if !(0..=5).contains(&count) {
        return None;
    }
    (0..count).map(|_| Some((r.short()?, r.short()?))).collect()
}
fn has_duplicate_slot(items: &[(u16, u16)]) -> bool {
    items
        .iter()
        .enumerate()
        .any(|(i, (s, _))| items[i + 1..].iter().any(|(o, _)| o == s))
}
/// Arena form B → 5, then T2 → 7.
fn craft_gates(ctx: &Context<'_>, reply: u16) -> Option<Outcome> {
    if arena_b(ctx) {
        return Some(Outcome::reason(reply, 5));
    }
    if ctx.pc().t2_blocked {
        return Some(Outcome::reason(reply, 7));
    }
    None
}

/// parity: 447 15-guild-handlers.md#447--g_contractguildcraftreqcmd--partial partial
pub fn handle_contract_guild_craft(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ Contract object contents and its later work are not read.
    const R: u16 = 448;
    if let Some(out) = craft_gates(ctx, R) {
        return out;
    }
    if !ctx.pc().craft_session {
        return Outcome::reason(R, 1);
    }
    if guild_id(ctx.pc()) == 0 {
        return Outcome::reason(R, 2);
    }
    let Some(id) = guild_with_action(ctx, 7) else {
        return Outcome::reason(R, 2);
    };
    let mut r = Reader(req);
    // ❓ Name decode capacity is not read; 33 units as the other guild names.
    let (Some(amount), Some(value), Some(name)) = (r.int(), r.quad(), r.string(NAME_UNITS)) else {
        return Outcome::reason(R, 1553);
    };
    let Some(list_a) = craft_records(&mut r) else {
        return Outcome::reason(R, 1553);
    };
    let Some(list_b) = craft_records(&mut r) else {
        return Outcome::reason(R, 1553);
    };
    let crossed = list_a
        .iter()
        .any(|(s, _)| list_b.iter().any(|(o, _)| o == s));
    if has_duplicate_slot(&list_a) || crossed || has_duplicate_slot(&list_b) {
        return Outcome::reason(R, 1553);
    }
    if ctx.pc().zone.is_none() {
        return Outcome::reason(R, 1553);
    }
    let contract = GuildCraftContract {
        guild_id: id,
        requester: ctx.pc().object_id,
        amount,
        value,
        name,
        list_a,
        list_b,
    };
    match ctx.services.guild_craft_contract(&contract) {
        Gate::Pass => Outcome::default(),
        Gate::Fail => Outcome::reason(R, 1553),
        Gate::Unknown => boundary("❓ guild craft contract object / World +5080 lock"),
    }
}

/// parity: 449 15-guild-handlers.md#449--g_assentguildcraftreqcmd--done ported
pub fn handle_assent_guild_craft(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 450;
    if let Some(out) = craft_gates(ctx, R) {
        return out;
    }
    if !ctx.pc().craft_session {
        return Outcome::reason(R, 1);
    }
    let mut r = Reader(req);
    let (Some(value), Some(_)) = (r.quad(), r.byte()) else {
        return Outcome::reason(R, 1553);
    };
    let pc = ctx.pc().clone();
    let Some(zone) = pc.zone else {
        return Outcome::reason(R, 1553);
    };
    let task = GuildTask {
        command: 449,
        caster: pc.object_id,
        zone,
        guild_id: pc.guild_id,
        kind: GuildTaskKind::CraftAssent { value },
    };
    queue_shared(ctx, task, R)
}

type DonateHeader = fn(&mut Reader<'_>) -> Option<(u64, u32, u8)>;

fn donate(
    ctx: &mut Context<'_>,
    reply: u16,
    req: &[u8],
    header: DonateHeader,
    brand: bool,
) -> Outcome {
    if let Some(out) = craft_gates(ctx, reply) {
        return out;
    }
    if !ctx.pc().craft_session {
        return Outcome::reason(reply, 1);
    }
    let mut r = Reader(req);
    let Some((value, extra, flag)) = header(&mut r) else {
        return Outcome::reason(reply, 1553);
    };
    let Some(items) = craft_records(&mut r) else {
        return Outcome::reason(reply, 1553);
    };
    if has_duplicate_slot(&items) {
        return Outcome::reason(reply, 1592);
    }
    let pc = ctx.pc().clone();
    let Some(zone) = pc.zone else {
        return Outcome::reason(reply, 2);
    };
    if pc.native_kind != 1
        || value == 0
        || items.is_empty()
        || items.iter().any(|(s, a)| *s == 0 || *a == 0)
    {
        return Outcome::reason(reply, 2);
    }
    let kind = if brand {
        GuildTaskKind::CraftDonateBrand {
            value,
            extra,
            flag,
            items,
        }
    } else {
        GuildTaskKind::CraftDonateNormal { value, items }
    };
    let task = GuildTask {
        command: reply - 1,
        caster: pc.object_id,
        zone,
        guild_id: pc.guild_id,
        kind,
    };
    queue_shared(ctx, task, reply)
}

/// parity: 451 15-guild-handlers.md#451--g_donateguildcraftnormalmaterialreqcmd--done ported
pub fn handle_donate_guild_craft_normal_material(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    donate(ctx, 452, req, |r| Some((r.quad()?, 0, 0)), false)
}
/// parity: 453 15-guild-handlers.md#453--g_donateguildcraftbrandmaterialreqcmd--done ported
pub fn handle_donate_guild_craft_brand_material(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    donate(
        ctx,
        454,
        req,
        |r| Some((r.quad()?, r.int()?, r.byte()?)),
        true,
    )
}

/// parity: 455 15-guild-handlers.md#455--g_makeguildcraftreqcmd--partial partial
pub fn handle_make_guild_craft(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ 0x1400F6470 body.
    const R: u16 = 456;
    if let Some(out) = craft_gates(ctx, R) {
        return out;
    }
    let Some(value) = Reader(req).quad() else {
        return Outcome::reason(R, 1553);
    };
    if !ctx.pc().craft_session {
        return Outcome::reason(R, 2);
    }
    let id = guild_id(ctx.pc());
    if id == 0 {
        return Outcome::reason(R, 1554);
    }
    if !ctx.world.guild.guilds.contains_key(&id) {
        return Outcome::reason(R, 2);
    }
    let db_id = ctx.pc().db_id;
    match ctx.services.guild_craft_make(id, db_id, value) {
        None => boundary("❓ guild craft make helper 0x1400F6470"),
        Some(0) => Outcome::default(),
        Some(reason) => Outcome::reason(R, reason),
    }
}

/// parity: 457 15-guild-handlers.md#457--g_takeguildcraftreqcmd--done ported
pub fn handle_take_guild_craft(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 458;
    if let Some(out) = craft_gates(ctx, R) {
        return out;
    }
    let mut r = Reader(req);
    let (Some(value), Some(_)) = (r.quad(), r.short()) else {
        return Outcome::reason(R, 1553);
    };
    let pc = ctx.pc().clone();
    let Some(zone) = pc.zone else {
        return Outcome::reason(R, 2);
    };
    if pc.native_kind != 1 {
        return Outcome::reason(R, 2);
    }
    let task = GuildTask {
        command: 457,
        caster: pc.object_id,
        zone,
        guild_id: pc.guild_id,
        kind: GuildTaskKind::CraftTake { value },
    };
    queue_shared(ctx, task, R)
}

/// parity: 459 15-guild-handlers.md#459--g_abandonguildcraftreqcmd--partial partial
pub fn handle_abandon_guild_craft(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ 0x140105BA0 body.
    const R: u16 = 460;
    if let Some(out) = craft_gates(ctx, R) {
        return out;
    }
    if Reader(req).quad().is_none() {
        return Outcome::reason(R, 1553);
    }
    let id = guild_id(ctx.pc());
    if id == 0 {
        return Outcome::reason(R, 1554);
    }
    let Some(guild) = ctx.world.guild.guilds.get(&id) else {
        return Outcome::reason(R, 2);
    };
    if guild.leader_db_id != ctx.pc().db_id {
        return Outcome::reason(R, 1597);
    }
    if !guild.craft_active {
        return Outcome::reason(R, 1555);
    }
    let pc = ctx.pc().clone();
    match ctx.services.guild_craft_abandon(id, &pc) {
        None => boundary("❓ guild craft abandon helper 0x140105BA0"),
        Some(0) => Outcome::default(),
        Some(reason) => Outcome::reason(R, reason),
    }
}
