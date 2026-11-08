//! Generated predicate/table fixtures only. No real game data.
use gamedaemon::validation::skill::*;

fn key() -> SkillKey {
    SkillKey {
        id: 17,
        variation: 3,
    }
}
fn state() -> PrefixState {
    PrefixState {
        server_dead_578: false,
        action_short_178: 1,
        action_byte_628: 0,
        rule_byte_31: 0,
        server_flags_888: 0,
    }
}

#[test]
fn every_modifier_count_has_signed_inclusive_bounds() {
    for (index, max) in [8, 38, 4, 26].into_iter().enumerate() {
        for n in [-1, 0, 1, max - 1, max, max + 1, i32::MAX] {
            let mut counts = [0; 4];
            counts[index] = n;
            assert_eq!(modifier_counts(counts).is_ok(), (0..=max).contains(&n));
        }
    }
    assert_eq!(modifier_counts([-1; 4]), Err(0));
    assert_eq!(modifier_counts([0, -1, -1, -1]), Err(1));
    assert_eq!(modifier_counts([0, 0, -1, -1]), Err(2));
    assert_eq!(modifier_counts([0, 0, 0, -1]), Err(3));
}

#[test]
fn modifier_dedup_does_not_hide_oversized_input() {
    let m = modifiers(&[2, 1, 2], &[9, 9], &[3, 3], &[4, 4]).unwrap();
    assert_eq!(m.equip.into_iter().collect::<Vec<_>>(), [1, 2]);
    assert_eq!(m.set_items.len(), 1);
    assert_eq!(m.item_skill_equip.len(), 1);
    assert_eq!(m.badge_equip.len(), 1);
    assert!(modifiers(&[], &[], &[], &[]).is_ok());
    assert!(modifiers(&[1; 8], &[2; 38], &[3; 4], &[4; 26]).is_ok());
    assert_eq!(modifiers(&[1; 9], &[], &[], &[]), Err(0));
    assert_eq!(modifiers(&[], &[2; 39], &[], &[]), Err(1));
    assert_eq!(modifiers(&[], &[], &[3; 5], &[]), Err(2));
    assert_eq!(modifiers(&[], &[], &[], &[4; 27]), Err(3));
}

#[test]
fn handler_identity_and_scalar_parse_failure_reply() {
    assert_eq!(
        handler_failure(None),
        FailureReply {
            command: 1227,
            reason: 1,
            selector: 0,
            key: SkillKey {
                id: 0,
                variation: 0
            }
        }
    );
    assert_eq!(
        handler_failure(Some((1, key()))),
        FailureReply {
            command: 1227,
            reason: 1,
            selector: 1,
            key: key()
        }
    );
}

#[test]
fn caster_control_and_zone_checks() {
    for control in [None, Some(0), Some(1), Some(2), Some(3)] {
        assert_eq!(
            select_caster(1, true, control, true, true),
            matches!(control, Some(1 | 2))
        );
    }
    assert!(!select_caster(1, true, Some(1), false, true));
    assert!(!select_caster(0, false, Some(1), true, true));
    assert!(select_caster(2, true, None, false, true));
    assert!(!select_caster(0, true, None, false, false));
    assert!(action_zone(Some(7), Some(7), true, true));
    for (zone, caster) in [
        (None, None),
        (None, Some(7)),
        (Some(7), None),
        (Some(7), Some(8)),
    ] {
        assert!(!action_zone(zone, caster, true, true));
    }
    assert!(!action_zone(Some(7), Some(7), false, true));
    assert!(!action_zone(Some(7), Some(7), true, false));
}

#[test]
fn factory_and_record_575_only_apply_mapped_restrictions() {
    for kind in [0, 1, 2, 3, 255] {
        assert_eq!(factory_record(kind, true), !matches!(kind, 1 | 2));
        assert!(!factory_record(kind, false));
    }
    for value in [0, 1, 2] {
        for action in [0, 1, 2] {
            assert_eq!(record_575_gate(value, action), value != 1 || action != 0);
        }
    }
}

struct Data {
    missing: Option<Dependency>,
    filter: bool,
}
impl SkillData for Data {
    fn present(&self, dependency: Dependency, requested: SkillKey) -> bool {
        assert_eq!(requested, key());
        self.missing != Some(dependency)
    }
    fn target_filter_2571_present(&self, _: SkillKey) -> bool {
        self.filter
    }
}
#[test]
fn required_data_failures_and_optional_target_filter() {
    for dependency in [
        Dependency::Skill2355,
        Dependency::Auxiliary2403,
        Dependency::Dash2427,
        Dependency::ResultControl2535,
        Dependency::GatherRange2451,
    ] {
        assert_eq!(
            data_dependencies(
                &Data {
                    missing: Some(dependency),
                    filter: true
                },
                key()
            ),
            Err(dependency)
        );
    }
    for filter in [false, true] {
        assert_eq!(
            data_dependencies(
                &Data {
                    missing: None,
                    filter
                },
                key()
            ),
            Ok(DataStatus {
                missing_target_filter: !filter
            })
        );
    }
}

struct Checks {
    calls: Vec<Gate>,
    reject: Option<Gate>,
    unmapped: Option<Gate>,
}
impl Checks {
    fn new() -> Self {
        Self {
            calls: vec![],
            reject: None,
            unmapped: None,
        }
    }
    fn check(&mut self, gate: Gate) -> Decision {
        self.calls.push(gate);
        if self.unmapped == Some(gate) {
            Decision::Unmapped
        } else if self.reject == Some(gate) {
            Decision::Reject
        } else {
            Decision::Pass
        }
    }
    fn with_reason(&mut self, gate: Gate) -> ReasonDecision {
        match self.check(gate) {
            Decision::Pass => ReasonDecision::Pass,
            Decision::Reject => ReasonDecision::Reject(191),
            Decision::Unmapped => ReasonDecision::Unmapped,
        }
    }
}
impl PrefixChecks for Checks {
    fn transformed_soulmask(&mut self) -> Decision {
        self.check(Gate::TransformedSoulmask)
    }
    fn grocery_item(&mut self) -> Decision {
        self.check(Gate::GroceryItem)
    }
    fn cast_filter(&mut self) -> ReasonDecision {
        self.with_reason(Gate::CastFilter)
    }
    fn consumable_points(&mut self) -> Decision {
        self.check(Gate::ConsumablePoints)
    }
    fn acquired_skill(&mut self) -> Decision {
        self.check(Gate::AcquiredSkill)
    }
    fn action_availability(&mut self) -> ReasonDecision {
        self.with_reason(Gate::ActionAvailability)
    }
}
const ORDER: [Gate; 6] = [
    Gate::TransformedSoulmask,
    Gate::GroceryItem,
    Gate::CastFilter,
    Gate::ConsumablePoints,
    Gate::AcquiredSkill,
    Gate::ActionAvailability,
];

#[test]
fn castability_prefix_has_exact_first_failure_order() {
    let expected = [
        (1227, 185),
        (1235, 216),
        (1227, 191),
        (1227, 190),
        (1227, 188),
        (1227, 191),
    ];
    for (index, gate) in ORDER.into_iter().enumerate() {
        let mut checks = Checks {
            reject: Some(gate),
            ..Checks::new()
        };
        let (command, reason) = expected[index];
        assert_eq!(
            castability_prefix(state(), &mut checks),
            Err(Stop::Failure { command, reason })
        );
        assert_eq!(checks.calls, ORDER[..=index]);
    }
    let mut checks = Checks::new();
    assert_eq!(castability_prefix(state(), &mut checks), Ok(PrefixPassed));
    assert_eq!(checks.calls, ORDER);
}

#[test]
fn unmapped_prefix_predicates_stop_instead_of_passing() {
    for (index, gate) in ORDER.into_iter().enumerate() {
        let mut checks = Checks {
            unmapped: Some(gate),
            ..Checks::new()
        };
        assert_eq!(
            castability_prefix(state(), &mut checks),
            Err(Stop::Pending(gate))
        );
        assert_eq!(checks.calls, ORDER[..=index]);
    }
}

#[test]
fn dead_flag_precedes_predicates_and_disabled_bit_follows_acquired() {
    let mut checks = Checks::new();
    assert_eq!(
        castability_prefix(
            PrefixState {
                server_dead_578: true,
                ..state()
            },
            &mut checks
        ),
        Err(Stop::Failure {
            command: 1227,
            reason: 185
        })
    );
    assert!(checks.calls.is_empty());
    for flags in [0, 1, 2, 3, 4] {
        let mut checks = Checks::new();
        assert_eq!(
            castability_prefix(
                PrefixState {
                    server_flags_888: flags,
                    ..state()
                },
                &mut checks
            )
            .is_ok(),
            flags & 2 == 0
        );
        assert_eq!(
            checks.calls.contains(&Gate::ActionAvailability),
            flags & 2 == 0
        );
        assert!(checks.calls.contains(&Gate::AcquiredSkill));
    }
}

#[test]
fn conditional_grocery_and_acquired_gates() {
    for short in [-1, 0, 1] {
        for action in [0, 1] {
            for rule in [0, 1] {
                let mut checks = Checks::new();
                assert!(castability_prefix(
                    PrefixState {
                        action_short_178: short,
                        action_byte_628: action,
                        rule_byte_31: rule,
                        ..state()
                    },
                    &mut checks
                )
                .is_ok());
                assert_eq!(checks.calls.contains(&Gate::GroceryItem), short > 0);
                assert_eq!(
                    checks.calls.contains(&Gate::AcquiredSkill),
                    action == 0 && rule == 0
                );
            }
        }
    }
}

#[test]
fn global_recycle_strict_boundary_offsets_and_all_groups() {
    let expiries = std::array::from_fn(|i| 100 + i as i64);
    for group in 1..=20 {
        let threshold = expiries[usize::from(group - 1)] - 3;
        assert!(!global_recycle(group, &expiries, 3, threshold - 1));
        assert!(!global_recycle(group, &expiries, 3, threshold));
        assert!(global_recycle(group, &expiries, 3, threshold + 1));
    }
    for group in [0, 21, 255] {
        assert!(!global_recycle(group, &expiries, 0, 0));
        assert!(global_recycle(group, &expiries, 0, 1));
    }
    assert!(!global_recycle(1, &[i64::MAX; 20], -1, i64::MAX));
}
impl PrepareChecks for Checks {
    fn complete_castability(&mut self) -> ReasonDecision {
        self.with_reason(Gate::Castability)
    }
    fn local_recycle(&mut self) -> Decision {
        self.check(Gate::LocalRecycle)
    }
    fn item_skill(&mut self) -> Decision {
        self.check(Gate::ItemSkill)
    }
}
fn prepare<'a>(expiries: &'a [i64; 20]) -> PrepareState<'a> {
    PrepareState {
        pc_or_summon_owner: true,
        global_group_711: 1,
        global_expiries: expiries,
        config_700: 0,
        server_now: 1,
        item_skill_count: 1,
    }
}

#[test]
fn castability_precedes_both_recycles_and_item_skill() {
    let expiries = [1; 20]; // global failure at equality
    let mut checks = Checks {
        reject: Some(Gate::Castability),
        ..Checks::new()
    };
    assert_eq!(
        ordinary_prepare_checks(prepare(&expiries), &mut checks),
        Err(Stop::Failure {
            command: 1227,
            reason: 191
        })
    );
    assert_eq!(checks.calls, [Gate::Castability]);
    let mut checks = Checks::new();
    assert_eq!(
        ordinary_prepare_checks(prepare(&expiries), &mut checks),
        Err(Stop::Failure {
            command: 1227,
            reason: 196
        })
    );
    assert_eq!(checks.calls, [Gate::Castability]);
    for (gate, reason) in [(Gate::LocalRecycle, 196), (Gate::ItemSkill, 2)] {
        let mut checks = Checks {
            reject: Some(gate),
            ..Checks::new()
        };
        assert_eq!(
            ordinary_prepare_checks(prepare(&[0; 20]), &mut checks),
            Err(Stop::Failure {
                command: 1227,
                reason
            })
        );
        let expected = if gate == Gate::LocalRecycle {
            vec![Gate::Castability, Gate::LocalRecycle]
        } else {
            vec![Gate::Castability, Gate::LocalRecycle, Gate::ItemSkill]
        };
        assert_eq!(checks.calls, expected);
    }
}

#[test]
fn preparation_conditionals_and_unmapped_gates() {
    for owner in [false, true] {
        for items in [0, 1] {
            let mut checks = Checks::new();
            assert!(ordinary_prepare_checks(
                PrepareState {
                    pc_or_summon_owner: owner,
                    item_skill_count: items,
                    ..prepare(&[0; 20])
                },
                &mut checks
            )
            .is_ok());
            assert_eq!(checks.calls.contains(&Gate::LocalRecycle), owner);
            assert_eq!(checks.calls.contains(&Gate::ItemSkill), items > 0);
        }
    }
    for gate in [Gate::Castability, Gate::LocalRecycle, Gate::ItemSkill] {
        let mut checks = Checks {
            unmapped: Some(gate),
            ..Checks::new()
        };
        assert_eq!(
            ordinary_prepare_checks(prepare(&[0; 20]), &mut checks),
            Err(Stop::Pending(gate))
        );
    }
}
