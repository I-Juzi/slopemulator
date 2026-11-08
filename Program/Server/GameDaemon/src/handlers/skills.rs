//! Doc 13 part 1: skill use, cancel/trigger, gather/loot, summon recall,
//! exhaustion/respawn, vitality, skill list/pages, traits and mastery entry
//! functions. Build 381.8867.
//!
//! Entry functions decode, gate and queue a `SkillTask` through
//! `Services::queue_skill`. Action creation, initialization and the task
//! actions are adapter boundaries. Coordinates (`u`) decode as shorts, as in
//! doc 11 and the doc 13 notation.
use super::items::reason_u64;
use super::items2::{pairs, u0_passes};
use super::*;

pub const SKILL_COMMANDS: &[u16] = &[
    1226, 1228, 1229, 1231, 1270, 77, 1272, 71, 74, 134, 135, 137, 139, 142, 821, 824, 113, 117,
    1485, 1488, 1491, 1494, 1497, 1389,
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SkillModifiers {
    /// 0..=8 equip positions.
    pub equip_positions: Vec<u8>,
    /// 0..=38 set-item keys.
    pub set_item_keys: Vec<u64>,
    /// 0..=4 item-skill equip positions.
    pub item_skill_positions: Vec<u8>,
    /// 0..=26 badge equip positions.
    pub badge_positions: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkillTaskKind {
    /// 1226: action factory `0x1404BE5A0`, init `0x1404988A0` mode 1.
    UseSkill {
        skill: u32,
        variation: u8,
        target: u64,
        request: u16,
        item_selector: u8,
        modifiers: SkillModifiers,
    },
    /// 1228: init `0x140498D00` mode 2.
    UseSkillToGround {
        skill: u32,
        variation: u8,
        position: [u16; 3],
        request: u16,
        item_selector: u8,
        modifiers: SkillModifiers,
    },
    /// 1229: option DWORD +144.
    CancelAction { option: u8 },
    /// 1231, kind 13. The task is a no-op in this build.
    SkillTrigger { trigger: u8 },
    /// 1270, action class 15.
    Gather { target: u64 },
    /// 77, kind 76.
    GatherCancel,
    /// 1272, action class 14.
    Loot { target: u64 },
    /// 71, action class 50 in the summon zone. Yaw is not copied.
    RecallSummon { position: [u16; 3] },
    /// 134, kind 11.
    SelectTarget { target: u64 },
    /// 135, kind 15.
    DieInExhaustion,
    /// 137, action class 16.
    RestorationInExhaustion,
    /// 139, action class 17.
    HelpRestoration { target: u64 },
    /// 142, init `0x140291520`.
    Respawn { option: u8 },
    /// 824, kind 119.
    RechargeVitality,
    /// 113, kind 82.
    SkillList,
    /// 117, kind 269.
    SetMasteryStatPoint { shorts: [i16; 2], bytes: [i8; 5] },
    /// 1485, kind 153.
    SkillPageTotal,
    /// 1488, kind 150.
    ChangeSkillPage { page: u8 },
    /// 1491, kind 152.
    ExpandSkillPage {
        page: u8,
        name: Vec<u16>,
        pairs: Vec<(u16, u16)>,
    },
    /// 1494, kind 151.
    RenameSkillPage { page: u8, name: Vec<u16> },
    /// 1497, kind 128. `build_up[0]` is forced to 1.
    TrainTrait {
        selectors: [u8; 2],
        build_up: [u8; 6],
        traits: Vec<(u32, i8)>,
    },
    /// 1389, action class 29.
    UnlockMasteryLevel { pairs: Vec<(u16, u16)> },
    /// Doc 13 part 2 (`skills2.rs`).
    Combat(CombatTask),
    /// Doc 16 chat/social zone tasks.
    Chat(ChatTask),
    /// Doc 17 mentoring zone tasks.
    Mentoring(MentoringTask),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillTask {
    pub command: u16,
    /// PC object ID (the owner when `summon` is true).
    pub caster: u64,
    /// The controlled summon is the selected creature.
    pub summon: bool,
    pub zone: u64,
    pub kind: SkillTaskKind,
}

/// parity: 135 13-skill-handlers.md#135--g_dieinexhaustionreqcmd--partial partial
/// parity: 137 13-skill-handlers.md#137--g_restorationinexhaustionreqcmd--partial partial
/// parity: 1485 13-skill-handlers.md#1485--g_skillpagetotalreqcmd--partial partial
/// parity: 821 13-skill-handlers.md#821--g_savevitalityreqcmd--done ported
/// These four go through `pc_task` or log only (821).
pub(super) fn dispatch_skill(ctx: &mut Context<'_>, command: u16, req: &[u8]) -> Outcome {
    match command {
        1226 => handle_use_skill(ctx, req, false),
        1228 => handle_use_skill(ctx, req, true),
        1229 => handle_cancel_action(ctx, req),
        1231 => handle_skill_trigger(ctx, req),
        1270 => handle_gather(ctx, req),
        77 => handle_gather_cancel(ctx, req),
        1272 => handle_loot(ctx, req),
        71 => handle_recall_summon(ctx, req),
        74 => handle_special_skill_gauge_full(ctx, req),
        134 => handle_select_target(ctx, req),
        135 => pc_task(ctx, 135, 136, 1, SkillTaskKind::DieInExhaustion),
        137 => pc_task(ctx, 137, 138, 1, SkillTaskKind::RestorationInExhaustion),
        139 => handle_help_restoration(ctx, req),
        142 => handle_respawn(ctx, req),
        821 => Outcome::log("save vitality: deprecated request"),
        824 => handle_recharge_vitality(ctx, req),
        113 => handle_skill_list(ctx, req),
        117 => handle_set_mastery_stat_point(ctx, req),
        1485 => pc_task(ctx, 1485, 1487, 2, SkillTaskKind::SkillPageTotal),
        1488 => handle_change_skill_page(ctx, req),
        1491 => handle_expand_skill_page(ctx, req),
        1494 => handle_rename_skill_page(ctx, req),
        1497 => handle_train_trait(ctx, req),
        1389 => handle_unlock_mastery_level(ctx, req),
        _ => unreachable!("membership checked"),
    }
}

pub(super) const UNCHECKED_DECODE: &str =
    "❓ unchecked skill decode (short body, field not zero-initialized)";

/// Selected creature for a control selector (doc 13, 1226 rule).
#[derive(Clone, Copy)]
struct Caster {
    summon: bool,
    kind: u8,
    zone: Option<u64>,
}
fn caster(ctx: &Context<'_>, selector: u8) -> Option<Caster> {
    let pc = ctx.pc();
    if selector != 1 {
        return Some(Caster {
            summon: false,
            kind: pc.native_kind,
            zone: pc.zone,
        });
    }
    let s = pc
        .summon
        .as_ref()
        .filter(|s| matches!(s.control_state, 1 | 2))?;
    Some(Caster {
        summon: true,
        kind: s.native_kind,
        zone: s.zone,
    })
}

/// `hEc`: reason, selector.
fn reason_selector(command: u16, reason: u16, selector: u8) -> Outcome {
    let mut body = reason.to_le_bytes().to_vec();
    body.push(selector);
    Outcome::reply(command, body)
}

/// Queue a task; `fail` on adapter failure, unsupported on Unknown.
pub(super) fn queue(ctx: &mut Context<'_>, task: SkillTask, fail: Outcome) -> Outcome {
    match ctx.services.queue_skill(&task) {
        Gate::Pass => Outcome {
            changes: vec![StateChange::Queued(task.command)],
            ..Outcome::default()
        },
        Gate::Fail => fail,
        Gate::Unknown => {
            Outcome::unsupported("❓ skill task/action allocation, init and queue adapter")
        }
    }
}

/// PC task: zone, PC kind 1 (assert then reply), queue. One reason for all.
pub(super) fn pc_task(
    ctx: &mut Context<'_>,
    command: u16,
    failure: u16,
    reason: u16,
    kind: SkillTaskKind,
) -> Outcome {
    let pc = ctx.pc();
    let Some(zone) = pc.zone else {
        return Outcome::reason(failure, reason);
    };
    if pc.native_kind != 1 {
        let mut out = Outcome::reason(failure, reason);
        out.logs.push("skill task: caster is not a PC");
        return out;
    }
    let task = SkillTask {
        command,
        caster: pc.object_id,
        summon: false,
        zone,
        kind,
    };
    queue(ctx, task, Outcome::reason(failure, reason))
}

// ---- Skill use, cancel, trigger ---------------------------------------------------

pub(super) fn counted<T>(
    r: &mut Reader<'_>,
    max: u16,
    item: impl Fn(&mut Reader<'_>) -> Option<T>,
) -> Option<Vec<T>> {
    let n = r.short()? as i16;
    if !(0..=max as i16).contains(&n) {
        return None;
    }
    (0..n).map(|_| item(r)).collect()
}

/// parity: 1226 13-skill-handlers.md#1226--g_useskillreqcmd--partial partial
/// parity: 1228 13-skill-handlers.md#1228--g_useskilltogroundreqcmd--partial partial
pub fn handle_use_skill(ctx: &mut Context<'_>, req: &[u8], ground: bool) -> Outcome {
    const F: u16 = 1227;
    let fail = |reason: u16, selector: u8, skill: u32, variation: u8| {
        let mut body = reason.to_le_bytes().to_vec();
        body.push(selector);
        body.extend(skill.to_le_bytes());
        body.push(variation);
        Outcome::reply(F, body)
    };
    let mut r = Reader(req);
    let scalars = (|| {
        let selector = r.byte()?;
        let skill = r.int()?;
        let variation = r.byte()?;
        let target = if ground {
            Err([r.short()?, r.short()?, r.short()?])
        } else {
            Ok(r.quad()?)
        };
        Some((selector, skill, variation, target, r.short()?, r.byte()?))
    })();
    let Some((selector, skill, variation, target, request, item_selector)) = scalars else {
        return fail(1, 0, 0, 0);
    };
    let failure = fail(1, selector, skill, variation);
    let modifiers = (|| {
        Some(SkillModifiers {
            equip_positions: counted(&mut r, 8, |r| r.byte())?,
            set_item_keys: counted(&mut r, 38, |r| r.quad())?,
            item_skill_positions: counted(&mut r, 4, |r| r.byte())?,
            badge_positions: counted(&mut r, 26, |r| r.byte())?,
        })
    })();
    let Some(modifiers) = modifiers else {
        return failure;
    };
    let Some(Caster {
        summon,
        zone: Some(zone),
        ..
    }) = caster(ctx, selector)
    else {
        return failure;
    };
    let kind = match target {
        Ok(target) => SkillTaskKind::UseSkill {
            skill,
            variation,
            target,
            request,
            item_selector,
            modifiers,
        },
        Err(position) => SkillTaskKind::UseSkillToGround {
            skill,
            variation,
            position,
            request,
            item_selector,
            modifiers,
        },
    };
    let task = SkillTask {
        command: if ground { 1228 } else { 1226 },
        caster: ctx.pc().object_id,
        summon,
        zone,
        kind,
    };
    queue(ctx, task, failure)
}

/// Selector + one byte; failure `hEc` reason 1.
fn selector_task(
    ctx: &mut Context<'_>,
    req: &[u8],
    command: u16,
    build: fn(u8) -> SkillTaskKind,
) -> Outcome {
    let failure = command + 1;
    let mut r = Reader(req);
    let (Some(selector), Some(value)) = (r.byte(), r.byte()) else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    let Some(Caster {
        summon,
        zone: Some(zone),
        ..
    }) = caster(ctx, selector)
    else {
        return reason_selector(failure, 1, selector);
    };
    let task = SkillTask {
        command,
        caster: ctx.pc().object_id,
        summon,
        zone,
        kind: build(value),
    };
    queue(ctx, task, reason_selector(failure, 1, selector))
}

/// parity: 1229 13-skill-handlers.md#1229--g_cancelactionreqcmd--partial partial
pub fn handle_cancel_action(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    selector_task(ctx, req, 1229, |option| SkillTaskKind::CancelAction {
        option,
    })
}

/// parity: 1231 13-skill-handlers.md#1231--g_skilltriggerreqcmd--done ported
pub fn handle_skill_trigger(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    selector_task(ctx, req, 1231, |trigger| SkillTaskKind::SkillTrigger {
        trigger,
    })
}

// ---- Gather, loot, summon recall, gauge, target ----------------------------------

/// parity: 1270 13-skill-handlers.md#1270--g_gatherreqcmd--partial partial
pub fn handle_gather(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if ctx.pc().zone.is_none() {
        return Outcome::reason(1271, 1);
    }
    if !u0_passes(ctx) {
        return Outcome::reason(1271, 683);
    }
    let Some(target) = Reader(req).quad() else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    pc_task(ctx, 1270, 1271, 1, SkillTaskKind::Gather { target })
}

/// parity: 77 13-skill-handlers.md#77--g_gathercancelreqcmd--partial partial
pub fn handle_gather_cancel(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    pc_task(ctx, 77, 78, 1, SkillTaskKind::GatherCancel)
}

/// parity: 1272 13-skill-handlers.md#1272--g_lootreqcmd--partial partial
pub fn handle_loot(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const F: u16 = 1273;
    let Some(target) = Reader(req).quad() else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    let pc = ctx.pc();
    if pc.zone.is_none() {
        return Outcome::reason(F, 1);
    }
    if pc.native_kind != 1 {
        let mut out = Outcome::reason(F, 1);
        out.logs.push("loot: caster is not a PC");
        return out;
    }
    if target >> 48 != 176 {
        // Reason 537, then fall through: reason 1 as well.
        let mut out = Outcome::reason(F, 537);
        out.replies.extend(Outcome::reason(F, 1).replies);
        return out;
    }
    pc_task(ctx, 1272, F, 1, SkillTaskKind::Loot { target })
}

/// parity: 71 13-skill-handlers.md#71--g_recallsummonreqcmd--partial partial
pub fn handle_recall_summon(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const F: u16 = 72;
    let pc = ctx.pc();
    let Some(s) = pc
        .summon
        .as_ref()
        .filter(|s| matches!(s.control_state, 1 | 2))
    else {
        return Outcome::reason(F, 2);
    };
    if s.native_kind != 2 {
        return Outcome::reason(F, 175);
    }
    if s.has_entry && s.race == 11 {
        return Outcome::reason(F, 179);
    }
    let Some(zone) = s.zone else {
        return Outcome::reason(F, 2);
    };
    let mut r = Reader(req);
    let (Some(x), Some(y), Some(z)) = (r.short(), r.short(), r.short()) else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    let task = SkillTask {
        command: 71,
        caster: pc.object_id,
        summon: true,
        zone,
        kind: SkillTaskKind::RecallSummon {
            position: [x, y, z],
        },
    };
    queue(ctx, task, Outcome::reason(F, 2))
}

/// parity: 74 13-skill-handlers.md#74--g_specialskillgaugefullreqcmd--partial partial
pub fn handle_special_skill_gauge_full(_: &mut Context<'_>, _: &[u8]) -> Outcome {
    // Failure 76 `hEh`: the extra short is not recovered. Success uses the
    // `0x140603BD0` scheduler, not the zone queue.
    Outcome::unsupported("❓ 76 extra reply short and 0x140603BD0 scheduler")
}

/// parity: 134 13-skill-handlers.md#134--g_selecttargetreqcmd--partial partial
pub fn handle_select_target(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let Some(target) = Reader(req).quad() else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    let pc = ctx.pc();
    let Some(zone) = pc.zone else {
        return Outcome::log("select target: zone is not found");
    };
    let task = SkillTask {
        command: 134,
        caster: pc.object_id,
        summon: false,
        zone,
        kind: SkillTaskKind::SelectTarget { target },
    };
    queue(ctx, task, Outcome::log("select target: queue failed"))
}

/// parity: 139 13-skill-handlers.md#139--g_helprestorationreqcmd--partial partial
pub fn handle_help_restoration(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const F: u16 = 141;
    let mut r = Reader(req);
    let (Some(selector), Some(target)) = (r.byte(), r.quad()) else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    let failure = reason_selector(F, 1, selector);
    let Some(Caster {
        summon,
        kind,
        zone: Some(zone),
    }) = caster(ctx, selector)
    else {
        return failure;
    };
    if !(1..=3).contains(&kind) {
        let mut out = failure;
        out.logs.push("help restoration: invalid caster kind");
        return out;
    }
    let task = SkillTask {
        command: 139,
        caster: ctx.pc().object_id,
        summon,
        zone,
        kind: SkillTaskKind::HelpRestoration { target },
    };
    queue(ctx, task, failure)
}

/// parity: 142 13-skill-handlers.md#142--g_respawnreqcmd--partial partial
pub fn handle_respawn(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let Some(option) = Reader(req).byte() else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    let pc = ctx.pc();
    let Some(zone) = pc.zone else {
        return Outcome::reason(144, 1);
    };
    let task = SkillTask {
        command: 142,
        caster: pc.object_id,
        summon: false,
        zone,
        kind: SkillTaskKind::Respawn { option },
    };
    queue(ctx, task, Outcome::reason(144, 1))
}

// ---- Vitality, skill list, mastery points, pages ----------------------------------

/// parity: 824 13-skill-handlers.md#824--g_rechargevitalityreqcmd--partial partial
pub fn handle_recharge_vitality(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    // UTC seconds are captured before the gates. The command-only decode
    // always succeeds for a framed message.
    let fail = reason_u64(826, 1, ctx.world.utc_seconds as u64);
    if !ctx.world.vitality_recharge_enabled {
        return fail;
    }
    let pc = ctx.pc();
    let Some(zone) = pc.zone else {
        return fail;
    };
    if pc.native_kind != 1 {
        let mut out = fail;
        out.logs.push("recharge vitality: caster is not a PC");
        return out;
    }
    let task = SkillTask {
        command: 824,
        caster: pc.object_id,
        summon: false,
        zone,
        kind: SkillTaskKind::RechargeVitality,
    };
    queue(ctx, task, fail)
}

/// parity: 113 13-skill-handlers.md#113--g_skilllist2reqcmd--partial partial
pub fn handle_skill_list(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    let pc = ctx.pc();
    let Some(zone) = pc.zone else {
        return Outcome::reason(115, 2);
    };
    let task = SkillTask {
        command: 113,
        caster: pc.object_id,
        summon: false,
        zone,
        kind: SkillTaskKind::SkillList,
    };
    queue(ctx, task, Outcome::reason(115, 2))
}

/// parity: 117 13-skill-handlers.md#117--g_setmasterystatpointreqcmd--partial partial
pub fn handle_set_mastery_stat_point(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let mut r = Reader(req);
    let fields = (|| {
        let shorts = [r.short()? as i16, r.short()? as i16];
        let mut bytes = [0i8; 5];
        for b in &mut bytes {
            *b = r.byte()? as i8;
        }
        Some((shorts, bytes))
    })();
    let Some((shorts, bytes)) = fields else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    if shorts.iter().any(|&s| s < 0) || bytes.iter().any(|&b| b < 0) {
        // Zone and PC kind come first, but use the same reason 2.
        return Outcome::reason(119, 2);
    }
    pc_task(
        ctx,
        117,
        119,
        2,
        SkillTaskKind::SetMasteryStatPoint { shorts, bytes },
    )
}

/// parity: 1488 13-skill-handlers.md#1488--g_changeskillpagereqcmd--partial partial
pub fn handle_change_skill_page(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let Some(page) = Reader(req).byte() else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    pc_task(ctx, 1488, 1490, 2, SkillTaskKind::ChangeSkillPage { page })
}

/// ❓ Decoder capacity 32 UTF-16 units, modelled as length ≤ 32.
const PAGE_NAME_CAPACITY: usize = 33;

/// parity: 1491 13-skill-handlers.md#1491--g_expandskillpagereqcmd--partial partial
pub fn handle_expand_skill_page(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const F: u16 = 1493;
    let mut r = Reader(req);
    let (Some(page), Some(name)) = (r.byte(), r.string(PAGE_NAME_CAPACITY)) else {
        let mut out = Outcome::reason(1330, 1);
        out.logs.push("expand skill page: fixed decode failed");
        return out;
    };
    let Ok(pairs) = pairs(&mut r, 0, 15) else {
        return Outcome::reason(F, 693);
    };
    let pc = ctx.pc();
    let Some(zone) = pc.zone else {
        return Outcome::reason(F, 1);
    };
    if pc.native_kind != 1 {
        // Another kind silently skips initialization and queueing.
        return Outcome::default();
    }
    let task = SkillTask {
        command: 1491,
        caster: pc.object_id,
        summon: false,
        zone,
        kind: SkillTaskKind::ExpandSkillPage { page, name, pairs },
    };
    queue(ctx, task, Outcome::reason(F, 1))
}

/// parity: 1494 13-skill-handlers.md#1494--g_renameskillpagereqcmd--partial partial
pub fn handle_rename_skill_page(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let mut r = Reader(req);
    let (Some(page), Some(name)) = (r.byte(), r.string(PAGE_NAME_CAPACITY)) else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    pc_task(
        ctx,
        1494,
        1496,
        2,
        SkillTaskKind::RenameSkillPage { page, name },
    )
}

// ---- Traits and mastery -----------------------------------------------------------

/// parity: 1497 13-skill-handlers.md#1497--g_traintraitreqcmd--partial partial
pub fn handle_train_trait(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    const F: u16 = 1499;
    let fail = |reason: u16, selectors: [u8; 2]| {
        let mut body = reason.to_le_bytes().to_vec();
        body.extend(selectors);
        Outcome::reply(F, body)
    };
    const NONE: [u8; 2] = [0xFF, 0xFF];
    let Some(zone) = ctx.pc().zone else {
        return fail(2, NONE);
    };
    if !ctx.pc().user_present {
        return fail(2, NONE);
    }
    // Rate gate on the session object. Equality passes. The timestamp is set
    // before decoding and is not rolled back by later failures.
    let now = ctx.world.tick;
    let last = ctx.pc().trait_train_tick;
    if last != 0 && last.saturating_add(ctx.world.trait_train_interval) > now {
        return fail(707, NONE);
    }
    ctx.session.pc.as_mut().expect("PC").trait_train_tick = now;
    let mut out = train_trait_body(ctx, req, zone, &fail);
    out.changes.insert(0, StateChange::TraitTrainTick(now));
    out
}

fn train_trait_body(
    ctx: &mut Context<'_>,
    req: &[u8],
    zone: u64,
    fail: &dyn Fn(u16, [u8; 2]) -> Outcome,
) -> Outcome {
    let mut r = Reader(req);
    let mut fixed = [0u8; 7];
    for b in &mut fixed {
        let Some(v) = r.byte() else {
            return fail(2, [0xFF, 0xFF]);
        };
        *b = v;
    }
    let selectors = [fixed[0], fixed[1]];
    if selectors[1] > 9 {
        return fail(711, selectors);
    }
    if !(1..=5).contains(&selectors[0]) {
        return fail(706, selectors);
    }
    let mut build_up = [1u8; 6];
    build_up[1..].copy_from_slice(&fixed[2..]);
    if build_up.iter().any(|&b| b > 3) {
        return fail(710, selectors);
    }
    let Some(n) = r.short().map(|n| n as i16).filter(|n| (0..=50).contains(n)) else {
        return fail(712, selectors);
    };
    let mut traits = Vec::new();
    for _ in 0..n {
        let (Some(id), Some(level)) = (r.int(), r.byte()) else {
            return fail(702, selectors);
        };
        if id == 0 {
            return fail(713, selectors);
        }
        let max = match ctx.services.trait_max_level(id) {
            None => return Outcome::unsupported("❓ trait table (manager slot 2379)"),
            Some(None) => return fail(713, selectors),
            Some(Some(max)) => max,
        };
        if level as i8 > max {
            return fail(717, selectors);
        }
        traits.push((id, level as i8));
    }
    let task = SkillTask {
        command: 1497,
        caster: ctx.pc().object_id,
        summon: false,
        zone,
        kind: SkillTaskKind::TrainTrait {
            selectors,
            build_up,
            traits,
        },
    };
    queue(ctx, task, fail(2, selectors))
}

/// parity: 1389 13-skill-handlers.md#1389--g_unlockmasterylevelreqcmd--partial partial
pub fn handle_unlock_mastery_level(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let Ok(pairs) = pairs(&mut Reader(req), 0, 15) else {
        return Outcome::reason(1391, 470);
    };
    if ctx.pc().t2_blocked {
        return Outcome::reason(1391, 7);
    }
    let pc = ctx.pc();
    let Some(zone) = pc.zone else {
        return Outcome::reason(1391, 1);
    };
    if pc.native_kind != 1 {
        return Outcome::log("unlock mastery level: caster is not a PC");
    }
    let task = SkillTask {
        command: 1389,
        caster: pc.object_id,
        summon: false,
        zone,
        kind: SkillTaskKind::UnlockMasteryLevel { pairs },
    };
    queue(ctx, task, Outcome::reason(1391, 1))
}
