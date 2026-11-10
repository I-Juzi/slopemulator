//! Doc 15 guild receive plans (core group). A posted guild work object or a
//! queued task is not a completed guild operation; the guild manager answers
//! later (❓ unread). All replies are `hh` (command, reason) to self.
use super::*;

pub const GUILD_COMMANDS: &[u16] = &[
    ids::G_CreateGuildReqCmd,
    ids::G_RenameGuildReqCmd,
    ids::G_DestroyGuildReqCmd,
    ids::G_RetrieveGuildInfoReqCmd,
    ids::G_GuildLevelUpReqCmd,
    ids::G_DepositInGuildBankReqCmd,
    ids::G_ContractGuildCraftReqCmd,
    ids::G_AssentGuildCraftReqCmd,
    ids::G_DonateGuildCraftNormalMaterialReqCmd,
    ids::G_DonateGuildCraftBrandMaterialReqCmd,
    ids::G_MakeGuildCraftReqCmd,
    ids::G_TakeGuildCraftReqCmd,
    ids::G_AbandonGuildCraftReqCmd,
    ids::G_ContractGuildUniformReqCmd,
    ids::G_DonateGuildUniformMaterialReqCmd,
    ids::G_AbandonGuildUniformReqCmd,
    ids::G_MakeGuildUniformReqCmd,
    ids::G_RegisterGuildUniformReqCmd,
    ids::G_PutOnGuildUniformReqCmd,
    ids::G_RegisterGuildUniformWithDeletionReqCmd,
    ids::G_IncreaseUniformRegistrationQuotaReqCmd,
    ids::G_RetrieveGuildMembersReqCmd,
    ids::G_EditGuildNotificationMessageReqCmd,
    ids::G_RetrieveGuildMemberPermissionReqCmd,
    ids::G_EditGuildMemberPermissionReqCmd,
    ids::G_ChangeGuildFlagBearerReqCmd,
    ids::G_ChangeGuildMemberGradeReqCmd,
    ids::G_DelegateGuildLeaderReqCmd,
    ids::G_ConfirmGuildLeaderReqCmd,
    ids::G_RejectGuildLeaderReqCmd,
    ids::G_InviteGuildReqCmd,
    ids::G_ConfirmInvitationGuildReqCmd,
    ids::G_RejectInvitationGuildReqCmd,
    ids::G_JoinGuildReqCmd,
    ids::G_LeaveGuildReqCmd,
    ids::G_BanishGuildMemberReqCmd,
    ids::G_QueryGuildInfoReqCmd,
    ids::G_RetrieveGuildJoinTicketListPerGuildReqCmd,
    ids::G_RetrieveGuildJoinTicketListPerPcReqCmd,
    ids::G_SendGuildApplicationReqCmd,
    ids::G_CancelGuildApplicationReqCmd,
    ids::G_RejectGuildApplicationReqCmd,
    ids::G_AcceptGuildApplicationReqCmd,
    ids::G_SendGuildInvitationReqCmd,
    ids::G_SendGuildInvitation2ReqCmd,
    ids::G_CancelGuildInvitationReqCmd,
    ids::G_RejectGuildInvitationReqCmd,
    ids::G_AcceptGuildInvitationReqCmd,
    ids::G_UpdateGuildApplicationRefusalReqCmd,
    ids::G_UpdateGuildInvitationRefusalReqCmd,
    ids::G_QueryGuildInfoToGamedReqCmd,
    ids::G_AccumulateGuildPointReqCmd,
    ids::G_GuildLevelUpWithGuildPointReqCmd,
    ids::G_PurchaseGuildBenefitReqCmd,
    ids::G_CreateGuildAllianceReqCmd,
    ids::G_AcceptGuildAllianceReqCmd,
    ids::G_RejectGuildAllianceReqCmd,
    ids::G_DeleteGuildAllianceReqCmd,
    ids::G_WarpToGuildMemberReqCmd,
    ids::G_CallGuildMemberReqCmd,
    ids::G_WarpToGuildMeetingPlaceReqCmd,
    ids::G_WarpToGuildMember2ReqCmd,
    ids::G_CallGuildMember2ReqCmd,
    ids::G_WarpToGuildMeetingPlace2ReqCmd,
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
    /// Config +2107 byte = 1: guild point system on. 443 replies reason 2;
    /// 603 and 606 reply reason 2 when it is off.
    pub level_up_disabled: bool,
    /// Config +2116 dword: seconds between accepted 603 requests (0 = none).
    pub guild_point_cooldown_secs: u32,
    /// Config +2136 byte: guild benefit purchase (610) enabled.
    pub benefit_enabled: bool,
    /// Config +2144 byte: guild alliance (615–623) enabled.
    pub alliance_enabled: bool,
    /// Config +2138 byte = 1: guild warp version 2. 1274/1277/1280 reply 2
    /// when it is on; 1283/1286/1289 reply 2 when it is off.
    pub warp_v2: bool,
    /// Config +104 signed dword, copied into the 1274 task +192 (❓ meaning).
    pub warp_member_value: i32,
    /// Config +2106 byte: guild bank deposit enabled.
    pub bank_enabled: bool,
    /// Config +64 signed dword: largest money deposit.
    pub bank_money_limit: i32,
    /// Permission mask limits: [normal (World +6264..), open (World +6304..)].
    pub permission_limits: [[u32; 4]; 2],
    /// `qword_140EE58A0 + 152`: guild uniform is in use (0 → reply 1568).
    pub uniform_enabled: bool,
    /// Config +2044 dword: seconds between 563/579/583 requests.
    pub join_request_cooldown_secs: u32,
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
    /// Guild +32417 faction byte (563 compares it with PC +328).
    pub faction: i8,
    /// Guild +32784 byte: guild refuses applications.
    pub application_refusal: bool,
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
    /// Guild-manager PC lookup `0x1403F7D20(World +5824, q, 1)`.
    /// ❓ The key may be the PC DB ID or the object ID.
    pub players: BTreeMap<u64, GuildPlayer>,
    /// `0x1400EF0B0(guild, index)`: (guild ID, member index) → PC object ID.
    pub member_slots: BTreeMap<(u64, u32), u64>,
    /// `0x1400F2900(guild, &out)`: 32-byte meeting place. Missing = false.
    pub meeting_places: BTreeMap<u64, [u8; 32]>,
}
/// Fields 583 reads from the looked-up PC.
#[derive(Clone, Debug, Default)]
pub struct GuildPlayer {
    /// PC +194 name.
    pub name: Vec<u16>,
    /// PC +328 faction.
    pub faction: i8,
    /// PC +20744 DB ID.
    pub db_id: u64,
    /// PC +40 object ID.
    pub object_id: u64,
    /// PC +37520 arena state (2, 3, 5, 6 = in arena).
    pub arena_state: u32,
    /// PC +2224 zone.
    pub zone: Option<u64>,
    /// PC +46 native kind.
    pub native_kind: u16,
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
    /// Separate 0x120-byte GuildAllianceWSO (`0x14067C5B0`), posted with
    /// `0x1403FB2D0`. +120 op (0 create, 1 accept, 2 reject, 3 delete),
    /// +128/+136 first guild, +208/+216 second guild, +282 reason.
    Alliance {
        op: u32,
        first_id: u64,
        first_name: Vec<u16>,
        second_id: u64,
        second_name: Vec<u16>,
        reason: u16,
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
            GuildWorkKind::Join { .. } | GuildWorkKind::Alliance { .. } => return None,
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
    /// 463 task (`0x14053EE50`): +152 uniform ID, records.
    UniformDonate { value: u64, items: Vec<(u16, u16)> },
    /// 475 task (`0x14053FE40`): +148 flag, records.
    UniformQuota { flag: u8, items: Vec<(u16, u16)> },
    /// Join-ticket task (`0x14036D550`, size 0xB0) set up by `0x140297680`
    /// with an op code and three values. ❓ Init body and its results unread.
    JoinTicket { op: u8, a: u64, b: u64, c: u64 },
    /// 596 task (+68 = 326, size 0xE8): +152 DB ID, +160 name, +226 raw value.
    ApplicationRefusal {
        db_id: u64,
        name: Vec<u16>,
        value: u64,
    },
    /// 600 task (+68 = 327, size 0xA0): +144 DB ID, +152 raw value.
    InvitationRefusal { db_id: u64, value: u64 },
    /// 603 task (+68 = 350, size 0xA8): +144 DB ID, +152 guild, +160 points.
    AccumulatePoint { db_id: u64, points: u32 },
    /// 606 task (+68 = 353, size 0xA0): +144 DB ID, +152 guild.
    LevelUpWithPoint { db_id: u64 },
    /// 610 task (+68 = 356, size 0xA8): +144 DB ID, +152 guild,
    /// +160 benefit, +162 type, +164 duration.
    PurchaseBenefit {
        db_id: u64,
        benefit: i16,
        kind: i16,
        duration: i32,
    },
    /// 1274 action (factory kind 53): +152 member PC, +160 value,
    /// +192 config +104, +200 records, +264 count.
    WarpToMember {
        member: u64,
        value: u16,
        config: i32,
        records: Vec<(u16, u16)>,
    },
    /// 1280 action (factory kind 55): no fields.
    WarpToMeetingPlace,
    /// 1283 task (+68 = 358, size 0xF8) on the target zone: +128 target,
    /// +144 PC, +152 DB ID, +160 guild, +168 member, +176 records, +240 count.
    WarpToMember2 {
        target: u64,
        db_id: u64,
        member: u64,
        records: Vec<(u16, u16)>,
    },
    /// 1286 action (factory kind 57): +152 PC, +160 DB ID, +168 guild,
    /// +176 records, +240 count.
    CallMember2 {
        db_id: u64,
        records: Vec<(u16, u16)>,
    },
    /// 1289 action (factory kind 58): +152 PC, +160 DB ID, +168 guild,
    /// +176 meeting place (32 bytes).
    WarpToMeetingPlace2 { db_id: u64, place: [u8; 32] },
}
/// 461 decoded request. ❓ The helper outputs (Destination +148…) are not read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuildUniformContract {
    pub guild_id: u64,
    pub requester: u64,
    pub value: u64,
    pub name: Vec<u16>,
    pub params: Vec<u8>,
    pub recipes: Vec<u32>,
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
        ids::G_CreateGuildReqCmd => handle_create_guild(ctx, req),
        ids::G_RenameGuildReqCmd => handle_rename_guild(ctx, req),
        ids::G_DestroyGuildReqCmd => handle_destroy_guild(ctx, req),
        ids::G_RetrieveGuildInfoReqCmd => handle_retrieve_guild_info(ctx, req),
        ids::G_GuildLevelUpReqCmd => handle_guild_level_up(ctx, req),
        ids::G_DepositInGuildBankReqCmd => handle_deposit_in_guild_bank(ctx, req),
        ids::G_RetrieveGuildMemberPermissionReqCmd => {
            handle_retrieve_guild_member_permission(ctx, req)
        }
        ids::G_EditGuildMemberPermissionReqCmd => handle_edit_guild_member_permission(ctx, req),
        ids::G_ChangeGuildFlagBearerReqCmd => handle_change_guild_flag_bearer(ctx, req),
        ids::G_ContractGuildCraftReqCmd => handle_contract_guild_craft(ctx, req),
        ids::G_AssentGuildCraftReqCmd => handle_assent_guild_craft(ctx, req),
        ids::G_DonateGuildCraftNormalMaterialReqCmd => {
            handle_donate_guild_craft_normal_material(ctx, req)
        }
        ids::G_DonateGuildCraftBrandMaterialReqCmd => {
            handle_donate_guild_craft_brand_material(ctx, req)
        }
        ids::G_MakeGuildCraftReqCmd => handle_make_guild_craft(ctx, req),
        ids::G_TakeGuildCraftReqCmd => handle_take_guild_craft(ctx, req),
        ids::G_AbandonGuildCraftReqCmd => handle_abandon_guild_craft(ctx, req),
        ids::G_ContractGuildUniformReqCmd => handle_contract_guild_uniform(ctx, req),
        ids::G_DonateGuildUniformMaterialReqCmd => handle_donate_guild_uniform_material(ctx, req),
        ids::G_AbandonGuildUniformReqCmd => handle_abandon_guild_uniform(ctx, req),
        ids::G_MakeGuildUniformReqCmd => handle_make_guild_uniform(ctx, req),
        ids::G_RegisterGuildUniformReqCmd => handle_register_guild_uniform(ctx, req),
        ids::G_PutOnGuildUniformReqCmd => handle_put_on_guild_uniform(ctx, req),
        ids::G_RegisterGuildUniformWithDeletionReqCmd => {
            handle_register_guild_uniform_with_deletion(ctx, req)
        }
        ids::G_IncreaseUniformRegistrationQuotaReqCmd => {
            handle_increase_uniform_registration_quota(ctx, req)
        }
        ids::G_RetrieveGuildMembersReqCmd => handle_retrieve_guild_members(ctx, req),
        ids::G_EditGuildNotificationMessageReqCmd => {
            handle_edit_guild_notification_message(ctx, req)
        }
        ids::G_ChangeGuildMemberGradeReqCmd => handle_change_guild_member_grade(ctx, req),
        ids::G_DelegateGuildLeaderReqCmd => handle_delegate_guild_leader(ctx, req),
        ids::G_ConfirmGuildLeaderReqCmd => handle_confirm_guild_leader(ctx, req),
        ids::G_RejectGuildLeaderReqCmd => handle_reject_guild_leader(ctx, req),
        ids::G_InviteGuildReqCmd => handle_invite_guild(ctx, req),
        ids::G_ConfirmInvitationGuildReqCmd => handle_confirm_invitation_guild(ctx, req),
        ids::G_RejectInvitationGuildReqCmd => handle_reject_invitation_guild(ctx, req),
        ids::G_JoinGuildReqCmd => handle_join_guild(ctx, req),
        ids::G_LeaveGuildReqCmd => handle_leave_guild(ctx, req),
        ids::G_BanishGuildMemberReqCmd => handle_banish_guild_member(ctx, req),
        ids::G_QueryGuildInfoReqCmd => handle_query_guild_info(ctx, req),
        ids::G_RetrieveGuildJoinTicketListPerGuildReqCmd => {
            handle_retrieve_guild_join_ticket_list_per_guild(ctx, req)
        }
        ids::G_RetrieveGuildJoinTicketListPerPcReqCmd => {
            handle_retrieve_guild_join_ticket_list_per_pc(ctx, req)
        }
        ids::G_SendGuildApplicationReqCmd => handle_send_guild_application(ctx, req),
        ids::G_CancelGuildApplicationReqCmd => handle_cancel_guild_application(ctx, req),
        ids::G_RejectGuildApplicationReqCmd => handle_reject_guild_application(ctx, req),
        ids::G_AcceptGuildApplicationReqCmd => handle_accept_guild_application(ctx, req),
        ids::G_SendGuildInvitationReqCmd => handle_send_guild_invitation(ctx, req),
        ids::G_SendGuildInvitation2ReqCmd => handle_send_guild_invitation2(ctx, req),
        ids::G_CancelGuildInvitationReqCmd => handle_cancel_guild_invitation(ctx, req),
        ids::G_RejectGuildInvitationReqCmd => handle_reject_guild_invitation(ctx, req),
        ids::G_AcceptGuildInvitationReqCmd => handle_accept_guild_invitation(ctx, req),
        ids::G_UpdateGuildApplicationRefusalReqCmd => {
            handle_update_guild_application_refusal(ctx, req)
        }
        ids::G_UpdateGuildInvitationRefusalReqCmd => {
            handle_update_guild_invitation_refusal(ctx, req)
        }
        ids::G_QueryGuildInfoToGamedReqCmd => handle_query_guild_info_to_gamed(ctx, req),
        ids::G_AccumulateGuildPointReqCmd => handle_accumulate_guild_point(ctx, req),
        ids::G_GuildLevelUpWithGuildPointReqCmd => handle_guild_level_up_with_guild_point(ctx, req),
        ids::G_PurchaseGuildBenefitReqCmd => handle_purchase_guild_benefit(ctx, req),
        ids::G_CreateGuildAllianceReqCmd => handle_create_guild_alliance(ctx, req),
        ids::G_AcceptGuildAllianceReqCmd => handle_accept_guild_alliance(ctx, req),
        ids::G_RejectGuildAllianceReqCmd => handle_reject_guild_alliance(ctx, req),
        ids::G_DeleteGuildAllianceReqCmd => handle_delete_guild_alliance(ctx, req),
        ids::G_WarpToGuildMemberReqCmd => handle_warp_to_guild_member(ctx, req),
        ids::G_CallGuildMemberReqCmd => handle_call_guild_member(ctx, req),
        ids::G_WarpToGuildMeetingPlaceReqCmd => handle_warp_to_guild_meeting_place(ctx, req),
        ids::G_WarpToGuildMember2ReqCmd => handle_warp_to_guild_member2(ctx, req),
        ids::G_CallGuildMember2ReqCmd => handle_call_guild_member2(ctx, req),
        ids::G_WarpToGuildMeetingPlace2ReqCmd => handle_warp_to_guild_meeting_place2(ctx, req),
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
    // `0x14069DA00` reads a signed short count (doc 12, 797 builder).
    let Some(count) = r.short().map(|c| c as i16) else {
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
/// `C` (u16) count (`count > 5` rejects) then (slot, amount) records.
fn craft_records(r: &mut Reader<'_>) -> Option<Vec<(u16, u16)>> {
    let count = r.short()?;
    if count > 5 {
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
    if !ctx.pc().user_present {
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
    let (Some(amount), Some(value), Some(name)) = (r.int(), r.byte(), r.string(NAME_UNITS)) else {
        return Outcome::reason(R, 1553);
    };
    let Some(list_a) = craft_records(&mut r) else {
        return Outcome::reason(R, 1553);
    };
    let Some(list_b) = craft_records(&mut r) else {
        return Outcome::reason(R, 1553);
    };
    // Build 381 excludes the last A record from the general cross-list loop.
    let crossed = list_a
        .iter()
        .take(list_a.len().saturating_sub(1))
        .any(|(s, _)| list_b.iter().any(|(o, _)| o == s))
        || (list_a.len() == 1 && list_b.len() == 1 && list_a[0].0 == list_b[0].0);
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
        value: u64::from(value),
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
    if !ctx.pc().user_present {
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
    if !ctx.pc().user_present {
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
    if !ctx.pc().user_present {
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

/// Uniform gate (U → 1568), then arena form B → 5, then T2 → 7.
fn uniform_gates(ctx: &Context<'_>, reply: u16) -> Option<Outcome> {
    if !ctx.world.guild.config.uniform_enabled {
        return Some(Outcome::reason(reply, 1568));
    }
    craft_gates(ctx, reply)
}

/// parity: 461 15-guild-handlers.md#461--g_contractguilduniformreqcmd--partial partial
pub fn handle_contract_guild_uniform(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ 0x1400F73E0 body and the 0x24210A transaction work.
    const R: u16 = 462;
    if let Some(out) = uniform_gates(ctx, R) {
        return out;
    }
    if !ctx.pc().user_present {
        return Outcome::reason(R, 1);
    }
    let mut r = Reader(req);
    let (Some(value), Some(name)) = (r.quad(), r.string(NAME_UNITS)) else {
        return Outcome::reason(R, 1553);
    };
    let params: Option<Vec<u8>> = r
        .short()
        .filter(|n| *n <= 28)
        .and_then(|n| (0..n).map(|_| r.byte().filter(|b| *b != 0)).collect());
    let Some(params) = params else {
        return Outcome::reason(R, 1553);
    };
    let recipes: Option<Vec<u32>> = r
        .short()
        .filter(|n| *n <= 28)
        .and_then(|n| (0..n).map(|_| r.int()).collect());
    let Some(recipes) = recipes else {
        return Outcome::reason(R, 1553);
    };
    match ctx.services.guild_uniform_name_ok(&name) {
        None => return boundary("❓ guild uniform name rules 0x14012F9F0 / name filter"),
        Some(false) => return Outcome::reason(R, 1602),
        Some(true) => {}
    }
    if ctx.pc().uniform_faction_blocked {
        return Outcome::reason(R, 1600);
    }
    if guild_id(ctx.pc()) == 0 {
        return Outcome::reason(R, 1554);
    }
    let Some(id) = guild_with_action(ctx, 8) else {
        return Outcome::reason(R, 2);
    };
    let contract = GuildUniformContract {
        guild_id: id,
        requester: ctx.pc().object_id,
        value,
        name,
        params,
        recipes,
    };
    match ctx.services.guild_uniform_contract(&contract) {
        None => boundary("❓ guild uniform contract helper 0x1400F73E0 / transaction"),
        Some(0) => Outcome::default(),
        Some(reason) => Outcome::reason(R, reason),
    }
}

/// parity: 463 15-guild-handlers.md#463--g_donateguilduniformmaterialreqcmd--done ported
pub fn handle_donate_guild_uniform_material(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 464;
    if let Some(out) = uniform_gates(ctx, R) {
        return out;
    }
    let mut r = Reader(req);
    let Some(value) = r.quad() else {
        return Outcome::reason(R, 1553);
    };
    let Some(items) = craft_records(&mut r) else {
        return Outcome::reason(R, 1553);
    };
    if items.iter().any(|(s, a)| *s == 0 || (*a as i16) <= 0) {
        return Outcome::reason(R, 1553);
    }
    if has_duplicate_slot(&items) {
        return Outcome::reason(R, 1592);
    }
    let pc = ctx.pc().clone();
    let Some(zone) = pc.zone else {
        return Outcome::reason(R, 2);
    };
    if pc.native_kind != 1 || value == 0 || items.is_empty() {
        return Outcome::reason(R, 2);
    }
    let task = GuildTask {
        command: 463,
        caster: pc.object_id,
        zone,
        guild_id: pc.guild_id,
        kind: GuildTaskKind::UniformDonate { value, items },
    };
    queue_shared(ctx, task, R)
}

/// 465–473: U, B, T2, decode, guild, then one opaque helper.
fn uniform_op(ctx: &mut Context<'_>, command: u16, req: &[u8], two: bool) -> Outcome {
    let reply = command + 1;
    if let Some(out) = uniform_gates(ctx, reply) {
        return out;
    }
    let mut r = Reader(req);
    let Some(a) = r.quad() else {
        return Outcome::reason(reply, 1553);
    };
    let b = if two {
        let Some(b) = r.quad() else {
            return Outcome::reason(reply, 1553);
        };
        b
    } else {
        0
    };
    let id = guild_id(ctx.pc());
    if id == 0 {
        return Outcome::reason(reply, 1554);
    }
    if !ctx.world.guild.guilds.contains_key(&id) {
        return Outcome::reason(reply, 2);
    }
    let pc = ctx.pc().clone();
    match ctx.services.guild_uniform_op(command, id, &pc, a, b) {
        None => boundary("❓ guild uniform helper (0x1400F7B00 / F7D40 / F8420 / F92D0 / F8920)"),
        Some(0) => Outcome::default(),
        Some(reason) => Outcome::reason(reply, reason),
    }
}

/// parity: 465 15-guild-handlers.md#465--g_abandonguilduniformreqcmd--partial partial
pub fn handle_abandon_guild_uniform(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ 0x1400F7B00 body.
    uniform_op(ctx, 465, req, false)
}
/// parity: 467 15-guild-handlers.md#467--g_makeguilduniformreqcmd--partial partial
pub fn handle_make_guild_uniform(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ 0x1400F7D40 body.
    uniform_op(ctx, 467, req, false)
}
/// parity: 469 15-guild-handlers.md#469--g_registerguilduniformreqcmd--partial partial
pub fn handle_register_guild_uniform(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ 0x1400F8420 body.
    uniform_op(ctx, 469, req, false)
}
/// parity: 471 15-guild-handlers.md#471--g_putonguilduniformreqcmd--partial partial
pub fn handle_put_on_guild_uniform(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ 0x1400F92D0 body.
    uniform_op(ctx, 471, req, false)
}
/// parity: 473 15-guild-handlers.md#473--g_registerguilduniformwithdeletionreqcmd--partial partial
pub fn handle_register_guild_uniform_with_deletion(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ 0x1400F8920 body.
    uniform_op(ctx, 473, req, true)
}

/// parity: 475 15-guild-handlers.md#475--g_increaseuniformregistrationquotareqcmd--done ported
pub fn handle_increase_uniform_registration_quota(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 476;
    if let Some(out) = uniform_gates(ctx, R) {
        return out;
    }
    if !ctx.pc().user_present {
        return Outcome::reason(R, 1);
    }
    let mut r = Reader(req);
    let Some(flag) = r.byte() else {
        return Outcome::reason(R, 1553);
    };
    let Some(items) = craft_records(&mut r) else {
        return Outcome::reason(R, 1553);
    };
    if has_duplicate_slot(&items) {
        return Outcome::reason(R, 1553);
    }
    let pc = ctx.pc().clone();
    let Some(zone) = pc.zone else {
        return Outcome::reason(R, 2);
    };
    if pc.native_kind != 1 || items.is_empty() || items.iter().any(|(s, a)| *s == 0 || *a == 0) {
        return Outcome::reason(R, 2);
    }
    let task = GuildTask {
        command: 475,
        caster: pc.object_id,
        zone,
        guild_id: pc.guild_id,
        kind: GuildTaskKind::UniformQuota { flag, items },
    };
    queue_shared(ctx, task, R)
}

/// Join-renewal row 18 on the looked-up guild.
const JOIN_ROW: u32 = 18;
fn trimmed(name: &[u16]) -> &[u16] {
    let end = name.iter().position(|u| *u == 0).unwrap_or(name.len());
    &name[..end]
}
/// Config +2044 seconds × 1000 against PC +20880. Some(now) when allowed.
fn join_cooldown(ctx: &Context<'_>) -> Option<u64> {
    let now = ctx.world.tick;
    let wait = u64::from(ctx.world.guild.config.join_request_cooldown_secs) * 1000;
    (ctx.pc().join_request_tick + wait <= now).then_some(now)
}
fn row18(ctx: &Context<'_>, id: u64) -> bool {
    ctx.world
        .guild
        .guilds
        .get(&id)
        .is_some_and(|g| ctx.world.guild.action_enabled(g, JOIN_ROW))
}
/// Queue a join-ticket task. `stamp` = new PC +20880 on success (563, 579, 583).
fn ticket_task(
    ctx: &mut Context<'_>,
    command: u16,
    reply: u16,
    op: u8,
    abc: (u64, u64, u64),
    stamp: Option<u64>,
) -> Outcome {
    let pc = ctx.pc().clone();
    let Some(zone) = pc.zone else {
        return Outcome::reason(reply, 2);
    };
    let task = GuildTask {
        command,
        caster: pc.object_id,
        zone,
        guild_id: pc.guild_id,
        kind: GuildTaskKind::JoinTicket {
            op,
            a: abc.0,
            b: abc.1,
            c: abc.2,
        },
    };
    let out = queue_shared(ctx, task, reply);
    if let (Some(now), Some(StateChange::Queued(_))) = (stamp, out.changes.first()) {
        if let Some(pc) = ctx.session.pc.as_mut() {
            pc.join_request_tick = now;
        }
    }
    out
}

/// parity: 557 15-guild-handlers.md#557--g_retrieveguildjointicketlistperguildreqcmd--partial partial
pub fn handle_retrieve_guild_join_ticket_list_per_guild(
    ctx: &mut Context<'_>,
    req: &[u8],
) -> Outcome {
    // ❓ 558 `hiqCqqhhq;` leading `i` field is not read; the list is not composed.
    const R: u16 = 559;
    if !ctx.world.guild.config.join_renewal {
        return Outcome::reason(R, 2);
    }
    if arena_a(ctx) {
        return Outcome::reason(R, 5);
    }
    let id = guild_id(ctx.pc());
    if id == 0 || Reader(req).quad().is_none() || !row18(ctx, id) {
        return Outcome::reason(R, 2);
    }
    let db_id = ctx.pc().db_id;
    for bit in [5, 17] {
        match ctx.services.guild_permission(id, db_id, bit) {
            None => return boundary("❓ guild permission helper 0x1400FCD00"),
            Some(true) => return boundary("❓ 558 join ticket list compose"),
            Some(false) => {}
        }
    }
    Outcome::reason(R, 2)
}

/// parity: 560 15-guild-handlers.md#560--g_retrieveguildjointicketlistperpcreqcmd--partial partial
pub fn handle_retrieve_guild_join_ticket_list_per_pc(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    // ❓ 561 `hiCqqhhq;` leading `i` field is not read; the list is not composed.
    const R: u16 = 562;
    if !ctx.world.guild.config.join_renewal {
        return Outcome::reason(R, 2);
    }
    if arena_a(ctx) {
        return Outcome::reason(R, 5);
    }
    if guild_id(ctx.pc()) != 0 {
        return Outcome::reason(R, 2);
    }
    boundary("❓ 561 join ticket list compose")
}

/// parity: 563 15-guild-handlers.md#563--g_sendguildapplicationreqcmd--partial partial
pub fn handle_send_guild_application(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 565;
    if !ctx.world.guild.config.join_renewal {
        return Outcome::reason(R, 2);
    }
    if arena_b(ctx) {
        return Outcome::reason(R, 5);
    }
    let Some(now) = join_cooldown(ctx) else {
        return Outcome::reason(R, 1624);
    };
    if guild_id(ctx.pc()) != 0 {
        return Outcome::reason(R, 1626);
    }
    let Some(name) = Reader(req).string(NAME_UNITS) else {
        return Outcome::reason(R, 2);
    };
    if !nonempty(&name) {
        return Outcome::reason(R, 2);
    }
    let found = ctx
        .world
        .guild
        .guilds
        .iter()
        .find(|(_, g)| trimmed(&g.name) == trimmed(&name));
    let Some((&id, guild)) = found else {
        return Outcome::reason(R, 2);
    };
    if guild.faction != ctx.pc().faction {
        return Outcome::reason(R, 1625);
    }
    if guild.guild_type == 2 {
        return Outcome::reason(R, 1623);
    }
    if !row18(ctx, id) {
        return Outcome::reason(R, 2);
    }
    let db_id = ctx.pc().db_id;
    ticket_task(ctx, 563, R, 1, (db_id, db_id, id), Some(now))
}

/// Shared shape of 567, 588 and 592: (q) names a guild; values (dbid, dbid, q).
fn pc_side_ticket(
    ctx: &mut Context<'_>,
    command: u16,
    reply: u16,
    op: u8,
    guildless: bool,
    req: &[u8],
) -> Outcome {
    if !ctx.world.guild.config.join_renewal {
        return Outcome::reason(reply, 2);
    }
    if arena_b(ctx) {
        return Outcome::reason(reply, 5);
    }
    if guildless && guild_id(ctx.pc()) != 0 {
        return Outcome::reason(reply, 2);
    }
    let target = Reader(req).quad().unwrap_or(0);
    if target == 0 || !row18(ctx, target) {
        return Outcome::reason(reply, 2);
    }
    let db_id = ctx.pc().db_id;
    ticket_task(ctx, command, reply, op, (db_id, db_id, target), None)
}

/// Shared shape of 571, 575, 579 and 584: (q) names a PC DB ID; values
/// (dbid, q, own guild).
fn guild_side_ticket(
    ctx: &mut Context<'_>,
    command: u16,
    reply: u16,
    op: u8,
    req: &[u8],
    cooldown: Option<u16>,
) -> Outcome {
    if !ctx.world.guild.config.join_renewal {
        return Outcome::reason(reply, 2);
    }
    if arena_b(ctx) {
        return Outcome::reason(reply, 5);
    }
    let mut stamp = None;
    if let Some(reason) = cooldown {
        let Some(now) = join_cooldown(ctx) else {
            return Outcome::reason(reply, reason);
        };
        stamp = Some(now);
    }
    let id = guild_id(ctx.pc());
    if id == 0 {
        return Outcome::reason(reply, 2);
    }
    let target = Reader(req).quad().unwrap_or(0);
    if target == 0 || !row18(ctx, id) {
        return Outcome::reason(reply, 2);
    }
    let db_id = ctx.pc().db_id;
    ticket_task(ctx, command, reply, op, (db_id, target, id), stamp)
}

/// parity: 567 15-guild-handlers.md#567--g_cancelguildapplicationreqcmd--partial partial
pub fn handle_cancel_guild_application(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    pc_side_ticket(ctx, 567, 569, 2, true, req)
}
/// parity: 571 15-guild-handlers.md#571--g_rejectguildapplicationreqcmd--partial partial
pub fn handle_reject_guild_application(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    guild_side_ticket(ctx, 571, 573, 3, req, None)
}
/// parity: 575 15-guild-handlers.md#575--g_acceptguildapplicationreqcmd--partial partial
pub fn handle_accept_guild_application(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    guild_side_ticket(ctx, 575, 577, 4, req, None)
}
/// parity: 579 15-guild-handlers.md#579--g_sendguildinvitationreqcmd--partial partial
pub fn handle_send_guild_invitation(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    guild_side_ticket(ctx, 579, 581, 5, req, Some(1644))
}

/// parity: 583 15-guild-handlers.md#583--g_sendguildinvitation2reqcmd--partial partial
pub fn handle_send_guild_invitation2(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // Replies go to 581 (not 585). A faction mismatch sends 1645 and the
    // handler continues (no return in build 381).
    const R: u16 = 581;
    if !ctx.world.guild.config.join_renewal {
        return Outcome::reason(R, 2);
    }
    if arena_b(ctx) {
        return Outcome::reason(R, 5);
    }
    let Some(now) = join_cooldown(ctx) else {
        return Outcome::reason(R, 1644);
    };
    let id = guild_id(ctx.pc());
    if id == 0 {
        return Outcome::reason(R, 2);
    }
    let mut r = Reader(req);
    let (Some(name), Some(key)) = (r.string(NAME_UNITS), r.quad()) else {
        return Outcome::reason(R, 2);
    };
    if !nonempty(&name) || key == 0 {
        return Outcome::reason(R, 2);
    }
    let Some(target) = ctx.world.guild.players.get(&key).cloned() else {
        return Outcome::reason(R, 2);
    };
    if trimmed(&target.name) != trimmed(&name) {
        return Outcome::reason(R, 2);
    }
    let Some(guild) = ctx.world.guild.guilds.get(&id) else {
        return Outcome::reason(R, 2);
    };
    let mismatch = guild.faction != target.faction;
    let mut out = if row18(ctx, id) {
        let db_id = ctx.pc().db_id;
        ticket_task(ctx, 583, R, 5, (db_id, target.db_id, id), Some(now))
    } else {
        Outcome::reason(R, 2)
    };
    if mismatch {
        let first = Outcome::reason(R, 1645);
        out.replies.splice(0..0, first.replies);
    }
    out
}

/// parity: 584 15-guild-handlers.md#584--g_cancelguildinvitationreqcmd--partial partial
pub fn handle_cancel_guild_invitation(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    guild_side_ticket(ctx, 584, 586, 6, req, None)
}
/// parity: 588 15-guild-handlers.md#588--g_rejectguildinvitationreqcmd--partial partial
pub fn handle_reject_guild_invitation(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    pc_side_ticket(ctx, 588, 590, 7, false, req)
}
/// parity: 592 15-guild-handlers.md#592--g_acceptguildinvitationreqcmd--partial partial
pub fn handle_accept_guild_invitation(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    pc_side_ticket(ctx, 592, 594, 8, true, req)
}

/// parity: 596 15-guild-handlers.md#596--g_updateguildapplicationrefusalreqcmd--done ported
pub fn handle_update_guild_application_refusal(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 598;
    if !ctx.world.guild.config.join_renewal {
        return Outcome::reason(R, 2);
    }
    if arena_b(ctx) {
        return Outcome::reason(R, 5);
    }
    let Some(value) = Reader(req).byte().map(u64::from) else {
        return Outcome::reason(R, 2);
    };
    let id = guild_id(ctx.pc());
    if id == 0 || !row18(ctx, id) {
        return Outcome::reason(R, 2);
    }
    if (value == 1) == ctx.world.guild.guilds[&id].application_refusal {
        return Outcome::reason(R, 2);
    }
    let pc = ctx.pc().clone();
    let Some(zone) = pc.zone else {
        return Outcome::reason(R, 2);
    };
    if pc.native_kind != 1 {
        return Outcome::reason(R, 2);
    }
    let mut name = pc.name.clone();
    name.truncate(NAME_UNITS);
    let task = GuildTask {
        command: 596,
        caster: pc.object_id,
        zone,
        guild_id: id,
        kind: GuildTaskKind::ApplicationRefusal {
            db_id: pc.db_id,
            name,
            value,
        },
    };
    queue_shared(ctx, task, R)
}

/// parity: 600 15-guild-handlers.md#600--g_updateguildinvitationrefusalreqcmd--done ported
pub fn handle_update_guild_invitation_refusal(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 602;
    if !ctx.world.guild.config.join_renewal {
        return Outcome::reason(R, 2);
    }
    if arena_b(ctx) {
        return Outcome::reason(R, 5);
    }
    let Some(value) = Reader(req).byte().map(u64::from) else {
        return Outcome::reason(R, 2);
    };
    let pc = ctx.pc().clone();
    if pc.guild_invitation_refusal == (value == 1) {
        return Outcome::reason(R, 2);
    }
    let Some(zone) = pc.zone else {
        return Outcome::reason(R, 2);
    };
    if pc.native_kind != 1 {
        return Outcome::reason(R, 2);
    }
    let task = GuildTask {
        command: 600,
        caster: pc.object_id,
        zone,
        guild_id: pc.guild_id,
        kind: GuildTaskKind::InvitationRefusal {
            db_id: pc.db_id,
            value,
        },
    };
    queue_shared(ctx, task, R)
}

/// parity: 518 15-guild-handlers.md#518--g_queryguildinfotogamedreqcmd--partial partial
pub fn handle_query_guild_info_to_gamed(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // ❓ 519 `hiCqQsCc;;` leading `i` and record tail are not read.
    const R: u16 = 520;
    if arena_a(ctx) {
        return Outcome::reason(R, 5);
    }
    let mut r = Reader(req);
    let Some(count) = r.short() else {
        return Outcome::reason(R, 2);
    };
    if count > 50 {
        return Outcome::reason(R, 2);
    }
    for _ in 0..count {
        if r.quad().unwrap_or(0) == 0 {
            return Outcome::reason(R, 2);
        }
    }
    boundary("❓ 519 guild info list compose")
}

/// Guild point and benefit tasks: zone, native kind 1, then queue.
fn point_task(ctx: &mut Context<'_>, reply: u16, kind: GuildTaskKind) -> Outcome {
    let pc = ctx.pc().clone();
    let Some(zone) = pc.zone else {
        return Outcome::reason(reply, 2);
    };
    if pc.native_kind != 1 || pc.db_id == 0 || pc.guild_id == 0 {
        return Outcome::reason(reply, 2);
    }
    let task = GuildTask {
        command: reply - 2,
        caster: pc.object_id,
        zone,
        guild_id: pc.guild_id,
        kind,
    };
    queue_task(ctx, task, reply)
}

/// parity: 603 15-guild-handlers.md#603--g_accumulateguildpointreqcmd--done ported
pub fn handle_accumulate_guild_point(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 605;
    let cfg = &ctx.world.guild.config;
    if !cfg.level_up_disabled {
        return point_reply(R, 2);
    }
    let wait = u64::from(cfg.guild_point_cooldown_secs) * 1000;
    if arena_b(ctx) {
        return point_reply(R, 5);
    }
    let now = ctx.world.tick;
    if wait != 0 && ctx.pc().guild_point_tick + wait > now {
        return point_reply(R, 1658);
    }
    let Some(points) = Reader(req).int() else {
        return point_reply(R, 2);
    };
    let id = guild_id(ctx.pc());
    if id == 0 {
        return point_reply(R, 2);
    }
    if !ctx.world.guild.guilds.contains_key(&id) {
        return point_reply(R, 1659);
    }
    if !ctx
        .world
        .guild
        .action_enabled(&ctx.world.guild.guilds[&id], 19)
    {
        return point_reply(R, 2);
    }
    if points == 0 {
        return point_reply(R, 2);
    }
    let db_id = ctx.pc().db_id;
    let mut out = point_task(ctx, R, GuildTaskKind::AccumulatePoint { db_id, points });
    for reply in &mut out.replies {
        reply.body.extend(0u64.to_le_bytes());
    }
    if matches!(out.changes.first(), Some(StateChange::Queued(_))) {
        if let Some(pc) = ctx.session.pc.as_mut() {
            pc.guild_point_tick = now;
        }
    }
    out
}
/// 605 is `hEq`; build 381 never sets the q argument (❓ stale stack value).
/// This port sends 0.
fn point_reply(reply: u16, reason: u16) -> Outcome {
    let mut out = Outcome::reason(reply, reason);
    for r in &mut out.replies {
        r.body.extend(0u64.to_le_bytes());
    }
    out
}

/// parity: 606 15-guild-handlers.md#606--g_guildlevelupwithguildpointreqcmd--done ported
pub fn handle_guild_level_up_with_guild_point(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    const R: u16 = 608;
    if !ctx.world.guild.config.level_up_disabled {
        return Outcome::reason(R, 2);
    }
    let db_id = ctx.pc().db_id;
    point_task(ctx, R, GuildTaskKind::LevelUpWithPoint { db_id })
}

/// parity: 610 15-guild-handlers.md#610--g_purchaseguildbenefitreqcmd--done ported
pub fn handle_purchase_guild_benefit(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 612;
    if !ctx.world.guild.config.benefit_enabled {
        return Outcome::reason(R, 2);
    }
    if arena_b(ctx) {
        return Outcome::reason(R, 5);
    }
    let mut r = Reader(req);
    let (Some(benefit), Some(kind), Some(duration)) = (r.short(), r.short(), r.int()) else {
        return Outcome::reason(R, 2);
    };
    let (benefit, kind, duration) = (benefit as i16, kind as i16, duration as i32);
    if benefit <= 0 || !(1..=2).contains(&kind) || duration < 0 {
        return Outcome::reason(R, 2);
    }
    let db_id = ctx.pc().db_id;
    let task = GuildTaskKind::PurchaseBenefit {
        db_id,
        benefit,
        kind,
        duration,
    };
    point_task(ctx, R, task)
}

/// Gates shared by 615–623: enabled → 2, arena form B → 5, own guild → 2.
/// Err = the failure reason.
fn alliance_gates(ctx: &Context<'_>) -> Result<(u64, Vec<u16>), u16> {
    if !ctx.world.guild.config.alliance_enabled {
        return Err(2);
    }
    if arena_b(ctx) {
        return Err(5);
    }
    let id = guild_id(ctx.pc());
    guild_named(ctx, id).ok_or(2)
}
fn alliance_post(
    ctx: &mut Context<'_>,
    reply: u16,
    op: u32,
    first: (u64, Vec<u16>),
    second: (u64, Vec<u16>),
    reason: u16,
) -> Outcome {
    let pc = ctx.pc().clone();
    if pc.db_id == 0 || first.0 == 0 || second.0 == 0 || !nonempty(&first.1) || !nonempty(&second.1)
    {
        return Outcome::reason(reply, 2);
    }
    let work = GuildWork {
        guild_id: pc.guild_id,
        requester: pc.db_id,
        kind: GuildWorkKind::Alliance {
            op,
            first_id: first.0,
            first_name: first.1,
            second_id: second.0,
            second_name: second.1,
            reason,
        },
    };
    post(ctx, work, Outcome::reason(reply, 2))
}
fn guild_named(ctx: &Context<'_>, id: u64) -> Option<(u64, Vec<u16>)> {
    let g = ctx.world.guild.guilds.get(&id)?;
    Some((id, trimmed(&g.name).to_vec()))
}

/// parity: 615 15-guild-handlers.md#615--g_createguildalliancereqcmd--done ported
pub fn handle_create_guild_alliance(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 617;
    let own = match alliance_gates(ctx) {
        Ok(own) => own,
        Err(reason) => return Outcome::reason(R, reason),
    };
    let Some(name) = Reader(req).string(NAME_UNITS) else {
        return Outcome::reason(R, 2);
    };
    if !nonempty(&name) {
        return Outcome::reason(R, 2);
    }
    let found = ctx
        .world
        .guild
        .guilds
        .iter()
        .find(|(_, g)| trimmed(&g.name) == trimmed(&name))
        .map(|(id, _)| *id);
    let Some(ally) = found else {
        return Outcome::reason(R, 1668);
    };
    if ally == own.0 {
        return Outcome::reason(R, 1669);
    }
    let name = trimmed(&name).to_vec();
    alliance_post(ctx, R, 0, own, (ally, name), 0)
}

/// 619/623 shape: (q) names the other guild.
fn alliance_by_id(ctx: &mut Context<'_>, reply: u16, op: u32, req: &[u8]) -> Outcome {
    let own = match alliance_gates(ctx) {
        Ok(own) => own,
        Err(reason) => return Outcome::reason(reply, reason),
    };
    let Some(key) = Reader(req).quad() else {
        return Outcome::reason(reply, 2);
    };
    let Some(other) = guild_named(ctx, key) else {
        return Outcome::reason(reply, 2);
    };
    if op == 3 {
        alliance_post(ctx, reply, op, own, other, 0)
    } else {
        alliance_post(ctx, reply, op, other, own, 0)
    }
}

/// parity: 619 15-guild-handlers.md#619--g_acceptguildalliancereqcmd--done ported
pub fn handle_accept_guild_alliance(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    alliance_by_id(ctx, 620, 1, req)
}

/// parity: 621 15-guild-handlers.md#621--g_rejectguildalliancereqcmd--done ported
pub fn handle_reject_guild_alliance(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 622;
    let own = match alliance_gates(ctx) {
        Ok(own) => own,
        Err(reason) => return Outcome::reason(R, reason),
    };
    // `Q` is one byte. The decoder writes it into a zeroed word; movzx copies
    // that word to +282 (0x1403B62AC, 0x1403B6334).
    let mut r = Reader(req);
    let (Some(key), Some(reason)) = (r.quad(), r.byte()) else {
        return Outcome::reason(R, 2);
    };
    let Some(other) = guild_named(ctx, key) else {
        return Outcome::reason(R, 2);
    };
    alliance_post(ctx, R, 2, other, own, u16::from(reason))
}

/// parity: 623 15-guild-handlers.md#623--g_deleteguildalliancereqcmd--done ported
pub fn handle_delete_guild_alliance(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    alliance_by_id(ctx, 625, 3, req)
}

/// Warp gates: version flag → 2, arena form B → 5, no guild → `no_guild`.
fn warp_gates(ctx: &Context<'_>, v2: bool, no_guild: u16) -> Result<u64, u16> {
    if ctx.world.guild.config.warp_v2 != v2 {
        return Err(2);
    }
    if arena_b(ctx) {
        return Err(5);
    }
    let id = guild_id(ctx.pc());
    if id == 0 {
        return Err(no_guild);
    }
    Ok(id)
}
/// Achievement reward (+21096), jumping ticket (+13752) or instant payment
/// (+21080), each read under its lock.
fn warp_busy(pc: &Pc) -> bool {
    pc.achievement_reward || pc.jumping_ticket || pc.instant_payment
}
/// `C` group: two-byte count (> 16 rejects, read unsigned), then (h, h)
/// records. Err(true) = count too big.
fn warp_records(r: &mut Reader<'_>) -> Result<Vec<(u16, u16)>, bool> {
    let count = r.short().ok_or(false)?;
    if count > 16 {
        return Err(true);
    }
    (0..count)
        .map(|_| Some((r.short()?, r.short()?)))
        .collect::<Option<Vec<_>>>()
        .ok_or(false)
}
/// Factory action on the PC zone: zone → 2, kind 1 → 2, queue fail → 2.
fn warp_action(ctx: &mut Context<'_>, command: u16, reply: u16, kind: GuildTaskKind) -> Outcome {
    let pc = ctx.pc().clone();
    let Some(zone) = pc.zone else {
        return Outcome::reason(reply, 2);
    };
    if pc.native_kind != 1 {
        return Outcome::reason(reply, 2);
    }
    let task = GuildTask {
        command,
        caster: pc.object_id,
        zone,
        guild_id: pc.guild_id,
        kind,
    };
    queue_task(ctx, task, reply)
}

/// parity: 1274 15-guild-handlers.md#1274--g_warptoguildmemberreqcmd--done ported
pub fn handle_warp_to_guild_member(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 1276;
    let id = match warp_gates(ctx, false, 1470) {
        Ok(id) => id,
        Err(reason) => return Outcome::reason(R, reason),
    };
    let Some(guild) = ctx.world.guild.guilds.get(&id) else {
        return Outcome::reason(R, 1470);
    };
    if !ctx.world.guild.action_enabled(guild, 14) {
        return Outcome::reason(R, 2);
    }
    if warp_busy(ctx.pc()) {
        return Outcome::reason(R, 1464);
    }
    let mut r = Reader(req);
    let (Some(index), Some(value)) = (r.byte(), r.short()) else {
        return Outcome::reason(R, 1553);
    };
    let records = match warp_records(&mut r) {
        Ok(records) => records,
        Err(true) => return Outcome::reason(R, 1468),
        Err(false) => return Outcome::reason(R, 1553),
    };
    let member = ctx
        .world
        .guild
        .member_slots
        .get(&(id, u32::from(index)))
        .copied()
        .unwrap_or(0);
    if member == 0 || member == ctx.pc().object_id {
        return Outcome::reason(R, 1473);
    }
    let config = ctx.world.guild.config.warp_member_value;
    let kind = GuildTaskKind::WarpToMember {
        member,
        value,
        config,
        records,
    };
    warp_action(ctx, 1274, R, kind)
}

/// parity: 1277 15-guild-handlers.md#1277--g_callguildmemberreqcmd--partial partial
pub fn handle_call_guild_member(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    const R: u16 = 1279;
    if ctx.world.guild.config.warp_v2 {
        return Outcome::reason(R, 2);
    }
    if arena_b(ctx) {
        return Outcome::reason(R, 5);
    }
    // STOP excluded restriction query 0x1403A8344 -> 0x14060E280 (1488).
    boundary("❓ STOP excluded restriction call in 1277; call task unread")
}

/// parity: 1280 15-guild-handlers.md#1280--g_warptoguildmeetingplacereqcmd--done ported
pub fn handle_warp_to_guild_meeting_place(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    const R: u16 = 1282;
    let id = match warp_gates(ctx, false, 1494) {
        Ok(id) => id,
        Err(reason) => return Outcome::reason(R, reason),
    };
    let Some(guild) = ctx.world.guild.guilds.get(&id) else {
        return Outcome::reason(R, 1494);
    };
    if !ctx.world.guild.action_enabled(guild, 16) {
        return Outcome::reason(R, 2);
    }
    if warp_busy(ctx.pc()) {
        return Outcome::reason(R, 1491);
    }
    warp_action(ctx, 1280, R, GuildTaskKind::WarpToMeetingPlace)
}

/// parity: 1283 15-guild-handlers.md#1283--g_warptoguildmember2reqcmd--done ported
pub fn handle_warp_to_guild_member2(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 1285;
    let id = match warp_gates(ctx, true, 1509) {
        Ok(id) => id,
        Err(reason) => return Outcome::reason(R, reason),
    };
    let mut r = Reader(req);
    let Some(index) = r.byte() else {
        return Outcome::reason(R, 2);
    };
    let Ok(records) = warp_records(&mut r) else {
        return Outcome::reason(R, 2);
    };
    if records.iter().any(|(_, b)| (*b as i16) <= 0) {
        return Outcome::reason(R, 2);
    }
    let Some(guild) = ctx.world.guild.guilds.get(&id) else {
        return Outcome::reason(R, 2);
    };
    let Some(&member) = ctx
        .world
        .guild
        .member_slots
        .get(&(id, u32::from(index)))
        .filter(|&&member| member != 0)
    else {
        return Outcome::reason(R, 1500);
    };
    if !ctx.world.guild.action_enabled(guild, 14) || member == ctx.pc().object_id {
        return Outcome::reason(R, 2);
    }
    let Some(target) = ctx.world.guild.players.get(&member).cloned() else {
        return Outcome::reason(R, 2);
    };
    if matches!(target.arena_state, 2 | 3 | 5 | 6) {
        return Outcome::reason(R, 1499);
    }
    let Some(zone) = target.zone else {
        return Outcome::reason(R, 2);
    };
    let pc = ctx.pc().clone();
    if target.native_kind != 1 || pc.object_id == 0 || pc.db_id == 0 {
        return Outcome::reason(R, 2);
    }
    let task = GuildTask {
        command: 1283,
        caster: pc.object_id,
        zone,
        guild_id: id,
        kind: GuildTaskKind::WarpToMember2 {
            target: target.object_id,
            db_id: pc.db_id,
            member,
            records,
        },
    };
    queue_shared(ctx, task, R)
}

/// parity: 1286 15-guild-handlers.md#1286--g_callguildmember2reqcmd--done ported
pub fn handle_call_guild_member2(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const R: u16 = 1288;
    let id = match warp_gates(ctx, true, 1523) {
        Ok(id) => id,
        Err(reason) => return Outcome::reason(R, reason),
    };
    if !row_on(ctx, id, 15) {
        return Outcome::reason(R, 2);
    }
    let Ok(records) = warp_records(&mut Reader(req)) else {
        return Outcome::reason(R, 2);
    };
    let pc = ctx.pc().clone();
    if pc.object_id == 0 || pc.db_id == 0 {
        return Outcome::reason(R, 2);
    }
    let kind = GuildTaskKind::CallMember2 {
        db_id: pc.db_id,
        records,
    };
    warp_action(ctx, 1286, R, kind)
}

/// parity: 1289 15-guild-handlers.md#1289--g_warptoguildmeetingplace2reqcmd--done ported
pub fn handle_warp_to_guild_meeting_place2(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    const R: u16 = 1291;
    let id = match warp_gates(ctx, true, 1535) {
        Ok(id) => id,
        Err(reason) => return Outcome::reason(R, reason),
    };
    if !row_on(ctx, id, 16) {
        return Outcome::reason(R, 2);
    }
    let Some(place) = ctx.world.guild.meeting_places.get(&id).copied() else {
        return Outcome::reason(R, 2);
    };
    let field = |a: usize, n: usize| {
        place[a..a + n]
            .iter()
            .rev()
            .fold(0u64, |v, b| v << 8 | u64::from(*b))
    };
    let expiry = field(24, 8) as i64;
    let now = ctx.world.utc_seconds;
    if field(0, 8) == 0 || field(8, 4) == 0 || field(12, 2) == 0 || now > expiry {
        return Outcome::reason(R, 2);
    }
    let pc = ctx.pc().clone();
    if pc.object_id == 0 || pc.db_id == 0 || expiry == 0 {
        return Outcome::reason(R, 2);
    }
    let kind = GuildTaskKind::WarpToMeetingPlace2 {
        db_id: pc.db_id,
        place,
    };
    warp_action(ctx, 1289, R, kind)
}
/// Guild lookup and action row. Missing guild = off.
fn row_on(ctx: &Context<'_>, id: u64, row: u32) -> bool {
    ctx.world
        .guild
        .guilds
        .get(&id)
        .is_some_and(|g| ctx.world.guild.action_enabled(g, row))
}
