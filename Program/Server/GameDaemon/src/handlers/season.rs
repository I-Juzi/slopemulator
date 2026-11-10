//! Doc 28: season, ranking, raid and store entries. Build 381.8867.
//! Store entries reuse doc 22. Tasks use the skill queue adapter.
//! Ranking caches and replies stop at explicit boundaries.
use super::skills::queue;
use super::*;

pub const SEASON_COMMANDS: &[u16] = &[
    ids::G_InitializeSeasonContentsInfoReqCmd,
    ids::G_SeasonContentsInfoReqCmd,
    ids::G_RetrieveBoundRaidDungeonInfoReqCmd,
    ids::G_UnbindRaidDungeonReqCmd,
    ids::G_RaidDungeonOnlinePlayerInfoReqCmd,
    ids::G_RaidDungeonExpirationReqCmd,
    ids::G_ReviveByAttractionRuleReqCmd,
    ids::G_TakeAttractionRewardReqCmd,
    ids::G_RetrieveSeasonContentsScheduleReqCmd,
    ids::G_RetrieveMyRankingInfoReqCmd,
    ids::G_RetrieveGuildRankingInfoReqCmd,
    ids::G_RetrieveRankingRewardListReqCmd,
    ids::G_TakeRankingRewardReqCmd,
    ids::G_PublicRaidEventAddScoreReqCmd,
    ids::G_RetrieveChallengePartyRankingInfoReqCmd,
    ids::G_SetPvPModeReqCmd,
    ids::G_PvpResultListReqCmd,
];

/// Each predicate reads server-owned state at its native position.
/// Pass means the predicate in the variant's comment is true.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeasonCheck {
    /// cfg +2744 byte +1028 != 0.
    RaidEnabled,
    /// cfg +2736 byte +476 != 0.
    RankingEnabled,
    /// Raid manager 0x1401A51B0 returns a record.
    RaidExists(u64),
    /// A property from 0x1407E5078..0x1407E509A equals this ID.
    RaidProperty(u64),
    /// Zone +16224 points to a geo record.
    UnbindGeoPresent(u64),
    /// Geo WORD +48 != 177.
    UnbindGeoAllowed(u64),
    /// Global byte +2089 == 0 OR PC WORD +12544 != 77.
    UnbindTrialAllowed,
    /// Guild manager lookup for PC guild ID returns a record.
    GuildExists(u64),
    /// Challenge-party manager lookup for PC qword +800 returns a record.
    ChallengePartyExists,
}

#[derive(Clone, Debug, Default)]
pub struct MemorySeason {
    pub results: BTreeMap<SeasonCheck, Gate>,
    pub checks: Vec<SeasonCheck>,
}
impl MemorySeason {
    pub fn check(&mut self, check: &SeasonCheck) -> Gate {
        self.checks.push(check.clone());
        self.results.get(check).copied().unwrap_or(Gate::Unknown)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeasonTask {
    /// 105: kind 293, vtable 0x140AE9A20. +144=1, +145=count, +146=types.
    InitializeContents { types: Vec<u8> },
    /// 108: kind 245, vtable 0x140B19148.
    ContentsInfo,
    /// 913: kind 163, vtable 0x140C3F6D0. Qword +144.
    BoundRaidInfo { raid: u64 },
    /// 916: kind 161, vtable 0x140B2B080. Raid +144, count +152, pairs +156.
    UnbindRaid { raid: u64, pairs: Vec<(u16, u16)> },
    /// 924: kind 164, vtable 0x140B9BC40. Byte +144=1.
    RaidExpiration { mode: u8 },
    /// 980: kind 251, vtable 0x140B94548.
    ReviveByAttraction,
    /// 985: kind 241, vtable 0x140BC1F28. Decoded qword is not copied.
    TakeAttractionReward,
    /// 1059: kind 289, vtable 0x140BB18F8.
    PublicRaidScore,
    /// 1134: kind 321, vtable 0x140C3FB08. Boolean byte +144.
    SetPvpMode { enabled: bool },
    /// 1213: kind 364, vtable 0x140BADAF0.
    PvpResults,
}
impl From<SeasonTask> for SkillTaskKind {
    fn from(task: SeasonTask) -> Self {
        Self::Season(task)
    }
}

pub(super) fn dispatch_season(ctx: &mut Context<'_>, command: u16, req: &[u8]) -> Outcome {
    match command {
        ids::G_InitializeSeasonContentsInfoReqCmd => {
            handle_initialize_season_contents_info(ctx, req)
        }
        ids::G_SeasonContentsInfoReqCmd => handle_season_contents_info(ctx, req),
        ids::G_RetrieveBoundRaidDungeonInfoReqCmd => {
            handle_retrieve_bound_raid_dungeon_info(ctx, req)
        }
        ids::G_UnbindRaidDungeonReqCmd => handle_unbind_raid_dungeon(ctx, req),
        ids::G_RaidDungeonOnlinePlayerInfoReqCmd => {
            handle_raid_dungeon_online_player_info(ctx, req)
        }
        ids::G_RaidDungeonExpirationReqCmd => handle_raid_dungeon_expiration(ctx, req),
        ids::G_ReviveByAttractionRuleReqCmd => handle_revive_by_attraction_rule(ctx, req),
        ids::G_TakeAttractionRewardReqCmd => handle_take_attraction_reward(ctx, req),
        ids::G_RetrieveSeasonContentsScheduleReqCmd => {
            handle_retrieve_season_contents_schedule(ctx, req)
        }
        ids::G_RetrieveMyRankingInfoReqCmd => handle_retrieve_my_ranking_info(ctx, req),
        ids::G_RetrieveGuildRankingInfoReqCmd => handle_retrieve_guild_ranking_info(ctx, req),
        ids::G_RetrieveRankingRewardListReqCmd => handle_retrieve_ranking_reward_list(ctx, req),
        ids::G_TakeRankingRewardReqCmd => handle_take_ranking_reward(ctx, req),
        ids::G_PublicRaidEventAddScoreReqCmd => handle_public_raid_event_add_score(ctx, req),
        ids::G_RetrieveChallengePartyRankingInfoReqCmd => {
            handle_retrieve_challenge_party_ranking_info(ctx, req)
        }
        ids::G_SetPvPModeReqCmd => handle_set_pvp_mode(ctx, req),
        ids::G_PvpResultListReqCmd => handle_pvp_result_list(ctx, req),
        _ => unreachable!("membership checked"),
    }
}

fn check(ctx: &mut Context<'_>, check: SeasonCheck, fail: Outcome) -> Option<Outcome> {
    match ctx.services.season_check(&check) {
        Gate::Pass => None,
        Gate::Fail => Some(fail),
        Gate::Unknown => Some(Outcome::unsupported("❓ season server-state check")),
    }
}
fn arena_b(ctx: &Context<'_>) -> bool {
    ctx.world.arena || session::arena_blocked(ctx.pc())
}
fn enqueue(
    ctx: &mut Context<'_>,
    command: u16,
    zone: u64,
    kind: SeasonTask,
    fail: Outcome,
) -> Outcome {
    let task = SkillTask {
        command,
        caster: ctx.pc().object_id,
        summon: false,
        zone,
        kind: kind.into(),
    };
    queue(ctx, task, fail)
}
fn simple(
    ctx: &mut Context<'_>,
    command: u16,
    fail: Outcome,
    pc_kind: bool,
    kind: SeasonTask,
) -> Outcome {
    let Some(zone) = ctx.pc().zone else {
        return fail;
    };
    if pc_kind && ctx.pc().native_kind != 1 {
        return fail;
    }
    enqueue(ctx, command, zone, kind, fail)
}
fn pair_count(r: &mut Reader<'_>, min: i16, max: i16) -> Option<i16> {
    r.short()
        .map(|n| n as i16)
        .filter(|n| (min..=max).contains(n))
}
fn read_pairs(r: &mut Reader<'_>, n: i16) -> Option<Vec<(u16, u16)>> {
    (0..n).map(|_| Some((r.short()?, r.short()?))).collect()
}
fn ranking_gate(
    ctx: &mut Context<'_>,
    command: u16,
    disabled: u16,
    form_b: bool,
) -> Option<Outcome> {
    if let Some(out) = check(
        ctx,
        SeasonCheck::RankingEnabled,
        Outcome::reason(command, disabled),
    ) {
        return Some(out);
    }
    if ctx.world.arena || (form_b && session::arena_blocked(ctx.pc())) {
        return Some(Outcome::reason(command, 5));
    }
    None
}

/// parity: 105 28-season-ranking-handlers.md#105--g_initializeseasoncontentsinforeqcmd--partial partial
pub fn handle_initialize_season_contents_info(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let mut r = Reader(req);
    // 0x14069DA00 returns -1 when the count field is short.
    let n = r.short().map(|n| n as i16).unwrap_or(-1);
    if n <= 0 {
        return Outcome::reason(107, 3);
    }
    // The native local buffer has 17 bytes and no cap. Do not copy its overwrite.
    if n > 17 {
        return Outcome::unsupported("❓ initialize contents oversized native buffer");
    }
    let mut types = Vec::new();
    for _ in 0..n {
        let value = r.byte().unwrap_or(0);
        if !(1..=16).contains(&value) || value == 2 {
            return Outcome::reason(107, 3);
        }
        types.push(value);
    }
    if ctx.pc().arena_state == 3 {
        return Outcome::unsupported("❓ arena-lobby initialize contents helper 0x140611EF0");
    }
    simple(
        ctx,
        ids::G_InitializeSeasonContentsInfoReqCmd,
        Outcome::reason(107, 2),
        false,
        SeasonTask::InitializeContents { types },
    )
}

/// parity: 108 28-season-ranking-handlers.md#108--g_seasoncontentsinforeqcmd--partial partial
pub fn handle_season_contents_info(ctx: &mut Context<'_>, _req: &[u8]) -> Outcome {
    simple(
        ctx,
        ids::G_SeasonContentsInfoReqCmd,
        Outcome::reason(110, 2),
        false,
        SeasonTask::ContentsInfo,
    )
}

/// parity: 913 28-season-ranking-handlers.md#913--g_retrieveboundraiddungeoninforeqcmd--partial partial
pub fn handle_retrieve_bound_raid_dungeon_info(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if arena_b(ctx) {
        return Outcome::reason(915, 5);
    }
    if let Some(out) = check(ctx, SeasonCheck::RaidEnabled, Outcome::reason(915, 2)) {
        return out;
    }
    let Some(raid) = Reader(req).quad() else {
        return Outcome::reason(915, 2);
    };
    simple(
        ctx,
        ids::G_RetrieveBoundRaidDungeonInfoReqCmd,
        Outcome::reason(915, 2),
        true,
        SeasonTask::BoundRaidInfo { raid },
    )
}

/// parity: 916 28-season-ranking-handlers.md#916--g_unbindraiddungeonreqcmd--partial partial
pub fn handle_unbind_raid_dungeon(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if arena_b(ctx) {
        return Outcome::reason(918, 5);
    }
    if let Some(out) = check(ctx, SeasonCheck::RaidEnabled, Outcome::reason(918, 2)) {
        return out;
    }
    let mut r = Reader(req);
    let Some(raid) = r.quad() else {
        return Outcome::reason(918, 2);
    };
    if let Some(out) = check(
        ctx,
        SeasonCheck::RaidExists(raid),
        Outcome::reason(918, 2099),
    ) {
        return out;
    }
    let Some(n) = pair_count(&mut r, 0, 15) else {
        return Outcome::reason(918, 2);
    };
    let Some(pairs) = read_pairs(&mut r, n) else {
        return Outcome::reason(918, 2);
    };
    let Some(zone) = ctx.pc().zone else {
        return Outcome::reason(918, 2);
    };
    for (predicate, reason) in [
        (SeasonCheck::UnbindGeoPresent(zone), 2),
        (SeasonCheck::UnbindGeoAllowed(zone), 2103),
        (SeasonCheck::UnbindTrialAllowed, 2104),
    ] {
        if let Some(out) = check(ctx, predicate, Outcome::reason(918, reason)) {
            return out;
        }
    }
    if ctx.pc().native_kind != 1 {
        return Outcome::reason(918, 2);
    }
    enqueue(
        ctx,
        ids::G_UnbindRaidDungeonReqCmd,
        zone,
        SeasonTask::UnbindRaid { raid, pairs },
        Outcome::reason(918, 2),
    )
}

/// parity: 921 28-season-ranking-handlers.md#921--g_raiddungeononlineplayerinforeqcmd--partial partial
pub fn handle_raid_dungeon_online_player_info(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if arena_b(ctx) {
        return Outcome::reason(923, 5);
    }
    if let Some(out) = check(ctx, SeasonCheck::RaidEnabled, Outcome::reason(923, 2)) {
        return out;
    }
    let Some(raid) = Reader(req).quad() else {
        return Outcome::reason(923, 2);
    };
    if let Some(out) = check(
        ctx,
        SeasonCheck::RaidProperty(raid),
        Outcome::reason(923, 2110),
    ) {
        return out;
    }
    if let Some(out) = check(
        ctx,
        SeasonCheck::RaidExists(raid),
        Outcome::reason(923, 2109),
    ) {
        return out;
    }
    Outcome::unsupported("❓ raid online-player record snapshot and reply 922")
}

/// parity: 924 28-season-ranking-handlers.md#924--g_raiddungeonexpirationreqcmd--partial partial
pub fn handle_raid_dungeon_expiration(ctx: &mut Context<'_>, _req: &[u8]) -> Outcome {
    if arena_b(ctx) {
        return Outcome::log("raid expiration: cannot process in arena");
    }
    if let Some(out) = check(ctx, SeasonCheck::RaidEnabled, Outcome::reason(926, 2)) {
        return out;
    }
    simple(
        ctx,
        ids::G_RaidDungeonExpirationReqCmd,
        Outcome::reason(926, 2),
        true,
        SeasonTask::RaidExpiration { mode: 1 },
    )
}

/// parity: 980 28-season-ranking-handlers.md#980--g_revivebyattractionrulereqcmd--partial partial
pub fn handle_revive_by_attraction_rule(ctx: &mut Context<'_>, _req: &[u8]) -> Outcome {
    // Dispatch supplies the h header. There is no body field or failure reply.
    simple(
        ctx,
        ids::G_ReviveByAttractionRuleReqCmd,
        Outcome::log("revive by attraction: task failed"),
        true,
        SeasonTask::ReviveByAttraction,
    )
}

/// parity: 985 28-season-ranking-handlers.md#985--g_takeattractionrewardreqcmd--partial partial
pub fn handle_take_attraction_reward(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if Reader(req).quad().is_none() {
        return Outcome::reason(987, 2);
    }
    // Native code does not copy the decoded qword into this task.
    simple(
        ctx,
        ids::G_TakeAttractionRewardReqCmd,
        Outcome::reason(987, 2),
        true,
        SeasonTask::TakeAttractionReward,
    )
}

/// parity: 989 28-season-ranking-handlers.md#989--g_retrieveseasoncontentsschedulereqcmd--partial partial
pub fn handle_retrieve_season_contents_schedule(ctx: &mut Context<'_>, _req: &[u8]) -> Outcome {
    if let Some(out) = ranking_gate(ctx, 991, 2227, false) {
        return out;
    }
    Outcome::unsupported("❓ season manager and cached schedule reply 990")
}

/// parity: 992 28-season-ranking-handlers.md#992--g_retrievemyrankinginforeqcmd--partial partial
pub fn handle_retrieve_my_ranking_info(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(out) = ranking_gate(ctx, 994, 2229, false) {
        return out;
    }
    let Some(contents) = Reader(req).byte() else {
        return Outcome::reason(994, 2);
    };
    if contents == 0 {
        return Outcome::reason(994, 2230);
    }
    if !(1..=16).contains(&contents) {
        return Outcome::reason(994, 2231);
    }
    Outcome::unsupported("❓ PC ranking timestamp, active season and RankingDaemon command 6")
}

/// parity: 995 28-season-ranking-handlers.md#995--g_retrieveguildrankinginforeqcmd--partial partial
pub fn handle_retrieve_guild_ranking_info(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(out) = ranking_gate(ctx, 997, 2232, false) {
        return out;
    }
    let Some(contents) = Reader(req).byte() else {
        return Outcome::reason(997, 2);
    };
    if contents == 0 {
        return Outcome::reason(997, 2233);
    }
    if let Some(out) = check(
        ctx,
        SeasonCheck::GuildExists(ctx.pc().guild_id),
        Outcome::reason(997, 1554),
    ) {
        return out;
    }
    if contents != 8 {
        // Native sends a zero snapshot for each nonzero type other than 8.
        let mut body = vec![contents];
        body.extend_from_slice(&[0; 34]); // i q q i s(empty) q
        return Outcome::reply(996, body);
    }
    Outcome::unsupported("❓ guild ranking cache, active season and RankingDaemon command 6")
}

/// parity: 998 28-season-ranking-handlers.md#998--g_retrieverankingrewardlistreqcmd--partial partial
pub fn handle_retrieve_ranking_reward_list(ctx: &mut Context<'_>, _req: &[u8]) -> Outcome {
    if let Some(out) = ranking_gate(ctx, 1000, 2229, false) {
        return out;
    }
    Outcome::unsupported("❓ ranking reward cache and RankingDaemon command 9")
}

/// parity: 1001 28-season-ranking-handlers.md#1001--g_takerankingrewardreqcmd--partial partial
pub fn handle_take_ranking_reward(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(out) = ranking_gate(ctx, 1003, 2241, true) {
        return out;
    }
    let mut r = Reader(req);
    let (Some(contents), Some(schedule), Some(result)) = (r.byte(), r.int(), r.byte()) else {
        return Outcome::reason(1003, 2);
    };
    if contents == 0 || schedule == 0 || result == 0 {
        return Outcome::reason(1003, 2242);
    }
    Outcome::unsupported("❓ ranking reward record, eligibility, reward group and action task")
}

/// parity: 1059 28-season-ranking-handlers.md#1059--g_publicraideventaddscorereqcmd--partial partial
pub fn handle_public_raid_event_add_score(ctx: &mut Context<'_>, _req: &[u8]) -> Outcome {
    simple(
        ctx,
        ids::G_PublicRaidEventAddScoreReqCmd,
        Outcome::reason(1061, 2),
        true,
        SeasonTask::PublicRaidScore,
    )
}

/// parity: 1088 28-season-ranking-handlers.md#1088--g_retrievechallengepartyrankinginforeqcmd--partial partial
pub fn handle_retrieve_challenge_party_ranking_info(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if let Some(out) = ranking_gate(ctx, 1090, 2235, false) {
        return out;
    }
    let Some(contents) = Reader(req).byte() else {
        return Outcome::reason(1090, 2);
    };
    if contents != 13 {
        return Outcome::reason(1090, 2236);
    }
    if let Some(out) = check(
        ctx,
        SeasonCheck::ChallengePartyExists,
        Outcome::reason(1090, 2238),
    ) {
        return out;
    }
    Outcome::unsupported(
        "❓ challenge-party ranking cache, active season and RankingDaemon command 6",
    )
}

/// parity: 1134 28-season-ranking-handlers.md#1134--g_setpvpmodereqcmd--partial partial
pub fn handle_set_pvp_mode(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let Some(value) = Reader(req).byte() else {
        return Outcome::reason(1136, 2);
    };
    simple(
        ctx,
        ids::G_SetPvPModeReqCmd,
        Outcome::reason(1136, 2),
        false,
        SeasonTask::SetPvpMode {
            enabled: value != 0,
        },
    )
}

/// parity: 1213 28-season-ranking-handlers.md#1213--g_pvpresultlistreqcmd--partial partial
pub fn handle_pvp_result_list(ctx: &mut Context<'_>, _req: &[u8]) -> Outcome {
    simple(
        ctx,
        ids::G_PvpResultListReqCmd,
        Outcome::reason(1215, 2),
        false,
        SeasonTask::PvpResults,
    )
}
