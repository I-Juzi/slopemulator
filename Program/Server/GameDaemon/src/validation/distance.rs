//! Server-position checks only. This module does not resolve client object IDs.
use crate::Position;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interaction {
    TradeInvite,
    TradeJoin,
    ExchangeFactionScore,
    MoveDepot,
    Buy,
    Sell,
    Rebuy,
    TradeWithNpc,
    Buy2Npc,
    Repair,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DistanceFailure {
    /// None means the outer reply command is not mapped. Do not invent one.
    pub command: Option<i16>,
    pub reason: i16,
}

/// Only the named resolved trade/NPC branch. Not every inventory operation.
/// NPC kind must be 4 at the caller; service/store checks are separate.
/// Spec: Step 5 part 3, Common rule table; 381 0x1403DE170, 0x1403DEEA0,
/// 0x140517ED0, 0x14051E910, 0x14051F7A0, 0x140520250, 0x140520E90,
/// 0x140522710, 0x140522A00, 0x140526F10, 0x140528D80.
pub fn interaction(
    path: Interaction,
    source: Position,
    target: Position,
) -> Result<(), DistanceFailure> {
    use Interaction::*;
    let (accepted, command, reason) = match path {
        TradeInvite => (source.squared_distance(target) < 3969, Some(312), 908),
        TradeJoin => (source.squared_distance(target) < 3969, Some(315), 917),
        path => {
            let (command, reason) = match path {
                ExchangeFactionScore => (Some(85), 334),
                MoveDepot => (None, 301),
                Buy => (Some(1342), 306),
                Sell => (Some(1349), 318),
                Rebuy => (None, 326),
                TradeWithNpc => (None, 341),
                Buy2Npc => (Some(1344), 35),
                Repair => (Some(1386), 451),
                TradeInvite | TradeJoin => unreachable!(),
            };
            (source.squared_distance(target) <= 10000, command, reason)
        }
    };
    if accepted {
        Ok(())
    } else {
        Err(DistanceFailure { command, reason })
    }
}

/// Bypass applies before NPC lookup. Inputs are resolved server helper results.
/// Spec: Step 5 part 3, Conditional bypasses; 381 0x1405EC080(pc,25),
/// 0x1405ED890, callers 0x14051E910, 0x14051F7A0.
/// TODO ❓ Property/permission meaning remains unmapped.
pub fn depot_bypasses_npc(helper_5ec080: i32, helper_5ed890: bool) -> bool {
    helper_5ec080 > 0 || helper_5ed890
}

/// Spec: Step 5 part 3, Conditional bypasses; 381 0x140526F10.
pub fn buy2_uses_npc(object_id: u64) -> bool {
    object_id != 0
}

/// Resolved live server objects. Kind 32 must use virtual +152 for its position.
/// Spec: Step 5 part 3, Quest distance; 381 0x140573EA0.
pub trait QuestObject {
    fn native_kind(&self) -> u8;
    fn position(&self) -> Position;
    fn virtual_152_position(&self) -> Option<Position>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestFailure {
    /// TODO ❓ Native null/kind return reason is not mapped.
    UnmappedPointerOrKind,
    Reason(i16),
}

/// Mode 2 skips distance only after pointer and kind checks. Negative maximum
/// is squared as a signed integer; no new nonnegative constraint is added.
/// Spec: Step 5 part 3, Quest pseudo-steps 1–9; 381 0x140573EA0.
pub fn quest(
    source: Option<&dyn QuestObject>,
    target: Option<&dyn QuestObject>,
    maximum: i32,
    mode: i32,
) -> Result<(), QuestFailure> {
    let (Some(source), Some(target)) = (source, target) else {
        return Err(QuestFailure::UnmappedPointerOrKind);
    };
    if !matches!(source.native_kind(), 1 | 2 | 4 | 240 | 241)
        || !matches!(target.native_kind(), 1 | 2 | 4 | 32 | 240 | 241)
    {
        return Err(QuestFailure::UnmappedPointerOrKind);
    }
    if mode == 2 {
        return Ok(());
    }
    let target_position = if target.native_kind() == 32 {
        target
            .virtual_152_position()
            .ok_or(QuestFailure::Reason(1014))?
    } else {
        target.position()
    };
    if i64::from(maximum) * i64::from(maximum) < source.position().squared_distance(target_position)
    {
        Err(QuestFailure::Reason(1010))
    } else {
        Ok(())
    }
}

/// Cylinder helper only; origin/range adapters must use resolved server data.
/// Spec: Step 5 part 2, Target/range; 381 0x1404AE5F0, 0x1404A63D0.
/// Equality passes on radial and vertical boundaries. No universal skill route.
pub fn skill_cylinder(
    source: Position,
    target: Position,
    minimum: i32,
    maximum: i32,
    height: i32,
    depth: i32,
) -> Result<(), i16> {
    let dx = i128::from(target.x) - i128::from(source.x);
    let dy = i128::from(target.y) - i128::from(source.y);
    let dz = i128::from(target.z) - i128::from(source.z);
    let radial = dx * dx + dy * dy;
    let vertical = i128::from(if dz >= 0 { height } else { depth });
    if i128::from(minimum).pow(2) <= radial
        && radial <= i128::from(maximum).pow(2)
        && dz * dz <= vertical * vertical
    {
        Ok(())
    } else {
        Err(195)
    }
}

/// Target size affects nonzero minimum and maximum. Widen before addition.
/// Spec: Step 5 part 2, Target/range; 381 0x1404A63D0.
pub fn sized_skill_ranges(minimum: i16, maximum: i16, target_size: i16) -> (i32, i32) {
    (
        if minimum != 0 {
            i32::from(minimum) + i32::from(target_size)
        } else {
            0
        },
        i32::from(maximum) + i32::from(target_size),
    )
}

/// Range -> direction -> target filter -> line of sight on the ordinary target
/// path only. Geometry uses server positions with z+20 and target short +2304.
/// Spec: Step 5 part 2, Target/range; 381 0x1404A63D0, 0x1404A7730,
/// 0x1400D6CD0. TODO ❓ Dispatch routes, direction/filter/LOS internals.
pub trait SkillTargetChecks {
    fn range(&mut self) -> bool;
    fn direction(&mut self) -> bool;
    fn target_filter(&mut self) -> Result<(), i16>;
    fn line_of_sight(&mut self) -> bool;
}

/// Spec: Step 5 part 2, Target/range; 381 0x1404A63D0.
/// The caller must resolve the server target before this path-specific helper.
pub fn ordinary_skill_target(checks: &mut impl SkillTargetChecks) -> Result<(), i16> {
    if !checks.range() {
        return Err(195);
    }
    if !checks.direction() {
        return Err(194);
    }
    checks.target_filter()?;
    if !checks.line_of_sight() {
        return Err(210);
    }
    Ok(())
}
