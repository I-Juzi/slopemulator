//! Doc 27: duel, party battle, ranking and season entry functions.
//! Build 381.8867.
//!
//! Part 1 covers the duel and party-battle zone tasks. Task constructors
//! and initialisers (`0x1402B8010`, `0x1402B84F0`) are opaque: the adapter
//! runs them inside `skills::queue`. An initialiser failure is a queue Fail.

use super::*;

pub const DUEL_RANKING_COMMANDS: &[u16] = &[277, 280, 283, 289, 292, 1022];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DuelTask {
    /// 277, kind 227, 0x98 bytes, vtable 0x140C3B318. Order byte +144.
    ChooseDuelOrder { order: u8 },
    /// 280 / 283: DuelTask 0x14036D860, init 0x1402B8010(task, zone, pc, mode).
    /// Mode 1 = ready round, 3 = take tag token (732 uses mode 2).
    Duel { mode: u8 },
    /// 289 / 292: PartyBattleTask 0x14036D8D0, init 0x1402B84F0(task, zone, pc, mode).
    /// Mode 0 = ready, 1 = start.
    PartyBattle { mode: u8 },
    /// 1022, kind 260, 0x98 bytes, vtable 0x140C3A928. Target PC ID qword +144.
    /// ❓ The adapter must also check zone +16536 virtual +8 == 6
    /// (party-battle zone); Fail replies 1024 [2], the same as a queue Fail.
    PartyBattleVote { target: u64 },
}
impl From<DuelTask> for SkillTaskKind {
    fn from(task: DuelTask) -> Self {
        SkillTaskKind::Duel(task)
    }
}

pub(super) fn dispatch_duel_ranking(ctx: &mut Context<'_>, command: u16, req: &[u8]) -> Outcome {
    match command {
        277 => handle_duel_choose_duel_order(ctx, req),
        280 => handle_duel_ready_round(ctx, req),
        283 => handle_duel_take_tag_token(ctx, req),
        289 => handle_party_battle_ready(ctx, req),
        292 => handle_party_battle_start(ctx, req),
        1022 => handle_party_battle_vote_cast(ctx, req),
        _ => unreachable!("membership checked"),
    }
}

fn logged(mut out: Outcome, text: &'static str) -> Outcome {
    out.logs.push(text);
    out
}
fn zone_task(ctx: &mut Context<'_>, command: u16, fail: u16, kind: DuelTask) -> Outcome {
    let Some(zone) = ctx.pc().zone else {
        return logged(Outcome::reason(fail, 2), "duel: zone is null");
    };
    let t = SkillTask {
        command,
        caster: ctx.pc().object_id,
        summon: false,
        zone,
        kind: kind.into(),
    };
    queue(ctx, t, Outcome::reason(fail, 2))
}

/// parity: 277 27-duel-ranking-handlers.md#277--g_duelchooseduelorderreqcmd--partial partial
pub fn handle_duel_choose_duel_order(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let Some(order) = Reader(req).byte() else {
        return logged(Outcome::reason(279, 2), "duel order: decompose failed");
    };
    if order as i8 <= 0 {
        return logged(Outcome::reason(279, 898), "duel order: invalid order");
    }
    zone_task(ctx, 277, 279, DuelTask::ChooseDuelOrder { order })
}

/// parity: 280 27-duel-ranking-handlers.md#280--g_duelreadyroundreqcmd--partial partial
pub fn handle_duel_ready_round(ctx: &mut Context<'_>, _req: &[u8]) -> Outcome {
    // Decode "h" only: a body-only request cannot fail here.
    zone_task(ctx, 280, 282, DuelTask::Duel { mode: 1 })
}

/// parity: 283 27-duel-ranking-handlers.md#283--g_dueltaketagtokenreqcmd--partial partial
pub fn handle_duel_take_tag_token(ctx: &mut Context<'_>, _req: &[u8]) -> Outcome {
    zone_task(ctx, 283, 285, DuelTask::Duel { mode: 3 })
}

/// parity: 289 27-duel-ranking-handlers.md#289--g_partybattlereadyreqcmd--partial partial
pub fn handle_party_battle_ready(ctx: &mut Context<'_>, _req: &[u8]) -> Outcome {
    zone_task(ctx, 289, 291, DuelTask::PartyBattle { mode: 0 })
}

/// parity: 292 27-duel-ranking-handlers.md#292--g_partybattlestartreqcmd--partial partial
pub fn handle_party_battle_start(ctx: &mut Context<'_>, _req: &[u8]) -> Outcome {
    zone_task(ctx, 292, 294, DuelTask::PartyBattle { mode: 1 })
}

/// parity: 1022 27-duel-ranking-handlers.md#1022--g_partybattlevotecastreqcmd--partial partial
pub fn handle_party_battle_vote_cast(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const F: u16 = 1024;
    let Some(target) = Reader(req).quad() else {
        return logged(Outcome::reason(F, 2), "party battle vote: decompose failed");
    };
    if ctx.pc().zone.is_none() {
        return logged(Outcome::reason(F, 2), "party battle vote: zone is null");
    }
    // PC word +46 (caller ID type) and target ID type must both be 1 (PC).
    if ctx.pc().object_id >> 48 != 1 || target >> 48 != 1 {
        return logged(Outcome::reason(F, 2), "party battle vote: invalid vote");
    }
    zone_task(ctx, 1022, F, DuelTask::PartyBattleVote { target })
}
