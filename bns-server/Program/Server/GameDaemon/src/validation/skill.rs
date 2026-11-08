//! Path-specific UseSkillReq subsets, not a universal skill pipeline.
//! TODO ❓ Full modifier derivation, formulas, costs, acquired/filter predicates,
//! action dispatch and the later castability suffix are not mapped (part 2).

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkillKey {
    pub id: i32,
    pub variation: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FailureReply {
    pub command: i16,
    pub reason: i16,
    pub selector: u8,
    pub key: SkillKey,
}

/// Spec: Step 5 part 2, Request parsing; 381 0x140373330. Format hEcic.
/// `None` means scalar decode failed. Other failures retain decoded identity.
pub fn handler_failure(identity: Option<(u8, SkillKey)>) -> FailureReply {
    let (selector, key) = identity.unwrap_or((
        0,
        SkillKey {
            id: 0,
            variation: 0,
        },
    ));
    FailureReply {
        command: 1227,
        reason: 1,
        selector,
        key,
    }
}

/// Bound signed native counts before allocating or reading elements.
/// Returns the first failed group, in native order.
/// Spec: Step 5 part 2, Request parsing; 381 0x140373330, 0x1404988A0.
pub fn modifier_counts(counts: [i32; 4]) -> Result<(), usize> {
    for (index, (count, limit)) in counts.into_iter().zip([8, 38, 4, 26]).enumerate() {
        if !(0..=limit).contains(&count) {
            return Err(index);
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Modifiers {
    pub equip: BTreeSet<u8>,
    pub set_items: BTreeSet<u64>,
    pub item_skill_equip: BTreeSet<u8>,
    pub badge_equip: BTreeSet<u8>,
}

/// Check supplied lengths before deduplication. Group identity stays separate.
/// Spec: Step 5 part 2, Request parsing steps 2/8; 381 0x140373330, 0x1404988A0.
pub fn modifiers(
    equip: &[u8],
    sets: &[u64],
    items: &[u8],
    badges: &[u8],
) -> Result<Modifiers, usize> {
    for (index, (count, limit)) in [equip.len(), sets.len(), items.len(), badges.len()]
        .into_iter()
        .zip([8, 38, 4, 26])
        .enumerate()
    {
        if count > limit {
            return Err(index);
        }
    }
    Ok(Modifiers {
        equip: equip.iter().copied().collect(),
        set_items: sets.iter().copied().collect(),
        item_skill_equip: items.iter().copied().collect(),
        badge_equip: badges.iter().copied().collect(),
    })
}

/// Spec: Step 5 part 2, Request parsing steps 3/4; 381 0x140373330.
pub fn select_caster(
    selector: u8,
    pc_present: bool,
    summon_control: Option<u8>,
    summon_present: bool,
    zone_present: bool,
) -> bool {
    zone_present
        && if selector == 1 {
            summon_present && matches!(summon_control, Some(1 | 2))
        } else {
            pc_present
        }
}

/// Spec: Step 5 part 2, Request parsing step 7; 381 0x1404988A0.
pub fn action_zone(
    zone: Option<u64>,
    caster_zone: Option<u64>,
    caster_present: bool,
    key_present: bool,
) -> bool {
    caster_present && key_present && zone.is_some() && zone == caster_zone
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dependency {
    Skill2355,
    Auxiliary2403,
    Dash2427,
    ResultControl2535,
    GatherRange2451,
}

/// Only the verified manager slots are named. No invented XML field mapping.
/// Spec: Step 5 part 2, Data dependencies; 381 0x1404978D0.
pub trait SkillData {
    fn present(&self, dependency: Dependency, key: SkillKey) -> bool;
    fn target_filter_2571_present(&self, key: SkillKey) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataStatus {
    /// A missing target filter logs here, but does not fail this initializer.
    pub missing_target_filter: bool,
}

/// Check known required dependencies. This is not complete data initialization.
/// Spec: Step 5 part 2, Data dependencies; 381 0x1404978D0.
/// PC-like missing-data reply is 1227/180; other kinds' reply is TODO ❓.
pub fn data_dependencies(data: &impl SkillData, key: SkillKey) -> Result<DataStatus, Dependency> {
    for dependency in [
        Dependency::Skill2355,
        Dependency::Auxiliary2403,
        Dependency::Dash2427,
        Dependency::ResultControl2535,
        Dependency::GatherRange2451,
    ] {
        if !data.present(dependency, key) {
            return Err(dependency);
        }
    }
    Ok(DataStatus {
        missing_target_filter: !data.target_filter_2571_present(key),
    })
}

/// Spec: Step 5 part 2, Data dependencies; 381 0x1404978D0.
pub fn record_575_gate(skill_byte_575: u8, action_byte_628: u8) -> bool {
    skill_byte_575 != 1 || action_byte_628 != 0
}

/// Spec: Step 5 part 2, Request parsing step 6; 381 0x1404BE5A0.
/// The adapter must resolve supported action classes; no class list is invented.
pub fn factory_record(kind_2: u8, resolved_supported_class: bool) -> bool {
    !matches!(kind_2, 1 | 2) && resolved_supported_class
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Pass,
    Reject,
    /// The predicate has not been recovered or implemented. Never means pass.
    Unmapped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReasonDecision {
    Pass,
    Reject(i16),
    Unmapped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    TransformedSoulmask,
    GroceryItem,
    CastFilter,
    ConsumablePoints,
    AcquiredSkill,
    ActionAvailability,
    Castability,
    LocalRecycle,
    ItemSkill,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    Failure { command: i16, reason: i16 },
    Pending(Gate),
}

fn decision(value: Decision, gate: Gate, command: i16, reason: i16) -> Result<(), Stop> {
    match value {
        Decision::Pass => Ok(()),
        Decision::Reject => Err(Stop::Failure { command, reason }),
        Decision::Unmapped => Err(Stop::Pending(gate)),
    }
}
fn reason_decision(value: ReasonDecision, gate: Gate) -> Result<(), Stop> {
    match value {
        ReasonDecision::Pass => Ok(()),
        ReasonDecision::Reject(reason) => Err(Stop::Failure {
            command: 1227,
            reason,
        }),
        ReasonDecision::Unmapped => Err(Stop::Pending(gate)),
    }
}

/// Server predicate adapters. No implementations or default successes are supplied.
/// Spec: Step 5 part 2, Castability prefix; 381 0x1404AB580, 0x1404A6BD0,
/// 0x1404AE9C0, 0x1404ADF60. TODO ❓ Full predicates remain unmapped.
pub trait PrefixChecks {
    fn transformed_soulmask(&mut self) -> Decision;
    fn grocery_item(&mut self) -> Decision;
    fn cast_filter(&mut self) -> ReasonDecision;
    fn consumable_points(&mut self) -> Decision;
    fn acquired_skill(&mut self) -> Decision;
    /// Adapter handles the conditional link-stage branch (184 or 191).
    fn action_availability(&mut self) -> ReasonDecision;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrefixState {
    pub server_dead_578: bool,
    pub action_short_178: i16,
    pub action_byte_628: u8,
    pub rule_byte_31: u8,
    pub server_flags_888: u32,
}

/// Marker for this prefix only. It does not establish complete castability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrefixPassed;

/// Ordered prefix for action type 0, not type 4 or all classes.
/// Spec: Step 5 part 2, Castability orders 1–8; 381 0x1404AB580.
/// TODO ❓ Swim/dash/water/throw/special checks after the prefix are not mapped.
pub fn castability_prefix(
    s: PrefixState,
    checks: &mut impl PrefixChecks,
) -> Result<PrefixPassed, Stop> {
    if s.server_dead_578 {
        return Err(Stop::Failure {
            command: 1227,
            reason: 185,
        });
    }
    decision(
        checks.transformed_soulmask(),
        Gate::TransformedSoulmask,
        1227,
        185,
    )?;
    if s.action_short_178 > 0 {
        decision(checks.grocery_item(), Gate::GroceryItem, 1235, 216)?;
    }
    reason_decision(checks.cast_filter(), Gate::CastFilter)?;
    decision(
        checks.consumable_points(),
        Gate::ConsumablePoints,
        1227,
        190,
    )?;
    if s.action_byte_628 == 0 && s.rule_byte_31 == 0 {
        decision(checks.acquired_skill(), Gate::AcquiredSkill, 1227, 188)?;
    }
    if s.server_flags_888 & 2 != 0 {
        return Err(Stop::Failure {
            command: 1227,
            reason: 184,
        });
    }
    reason_decision(checks.action_availability(), Gate::ActionAvailability)?;
    Ok(PrefixPassed)
}

/// Strict global recycle comparison; out-of-range groups select expiry zero.
/// Spec: Step 5 part 2, Verified check order; 381 0x140618680.
pub fn global_recycle(group: u8, expiries: &[i64; 20], config_700: i32, server_now: i64) -> bool {
    let expiry = if (1..=20).contains(&group) {
        expiries[usize::from(group - 1)]
    } else {
        0
    };
    i128::from(expiry) - i128::from(config_700) < i128::from(server_now)
}

/// An adapter must resolve *complete* selected-path castability, not just prefix.
/// Spec: Step 5 part 2, prepareAction; 381 0x14049A7A0, 0x1406185E0,
/// 0x14049C8B0. TODO ❓ Local recycle expression/lookup is not mapped.
pub trait PrepareChecks {
    fn complete_castability(&mut self) -> ReasonDecision;
    fn local_recycle(&mut self) -> Decision;
    fn item_skill(&mut self) -> Decision;
}

pub struct PrepareState<'a> {
    pub pc_or_summon_owner: bool,
    pub global_group_711: u8,
    pub global_expiries: &'a [i64; 20],
    pub config_700: i32,
    pub server_now: i64,
    pub item_skill_count: usize,
}

/// Ordinary preparation's check phase only. Server position capture, redirection,
/// formula work and special early branches must be resolved before this call.
/// Spec: Step 5 part 2, prepareAction steps 5–9; 381 0x14049A7A0.
/// Success is not complete UseSkill approval or permission to deduct resources.
pub fn ordinary_prepare_checks(
    s: PrepareState<'_>,
    checks: &mut impl PrepareChecks,
) -> Result<(), Stop> {
    reason_decision(checks.complete_castability(), Gate::Castability)?;
    if s.pc_or_summon_owner {
        if !global_recycle(
            s.global_group_711,
            s.global_expiries,
            s.config_700,
            s.server_now,
        ) {
            return Err(Stop::Failure {
                command: 1227,
                reason: 196,
            });
        }
        decision(checks.local_recycle(), Gate::LocalRecycle, 1227, 196)?;
    }
    if s.item_skill_count > 0 {
        decision(checks.item_skill(), Gate::ItemSkill, 1227, 2)?;
    }
    Ok(())
}
