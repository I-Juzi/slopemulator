//! Generated fixtures only. Values do not come from game records.
use gamedaemon::{validation::movement::*, Position};
use std::cell::RefCell;

fn p(x: i16, y: i16, z: i16) -> Position {
    Position { x, y, z }
}
fn cell(x: i16, y: i16, z: i16) -> Cell {
    Cell { x, y, z }
}
fn mt(code: i8) -> MoveType {
    MoveType {
        code,
        start_cell: 0,
        start_time: 0,
        flag: false,
    }
}
fn layer(floor: i16, ceiling: i16, attribute_11: u8) -> Layer {
    Layer {
        floor,
        ceiling,
        environment: 0,
        attribute_11,
    }
}

#[test]
fn handler_and_initializer_have_distinct_cell_limits() {
    for n in 0..=37 {
        let cells = vec![cell(0, 0, 0); n];
        assert_eq!(handler_counts(1, n, n).is_ok(), n <= 35);
        assert_eq!(
            initializer_limits(Some(7), Some(7), 1, Some(&cells), 0, 1).is_ok(),
            (1..=36).contains(&n)
        );
    }
    assert_eq!(handler_counts(1, 36, 35), Err(Failure::Reason(102)));
    assert_eq!(handler_counts(1, 2, 1), Err(Failure::Reason(102)));
}

#[test]
fn move_type_count_bounds_are_not_merged() {
    let c = [cell(0, 0, 0)];
    for n in 0..=13 {
        assert_eq!(handler_counts(n, 1, 1).is_ok(), n <= 12);
        assert_eq!(
            initializer_limits(Some(7), Some(7), 1, Some(&c), 0, n).is_ok(),
            (1..=12).contains(&n)
        );
    }
    assert_eq!(handler_counts(13, 37, 37), Err(Failure::Reason(103)));
}

#[test]
fn duration_zone_kind_and_null_input_bounds() {
    let c = [cell(0, 0, 0)];
    for duration in [0, 1, 549, 550, 551, u32::MAX] {
        assert_eq!(
            initializer_limits(Some(7), Some(7), 1, Some(&c), duration, 1).is_ok(),
            duration <= 550
        );
    }
    for kind in [0, 1, 2, 3, 4, 255] {
        assert_eq!(
            initializer_limits(Some(7), Some(7), kind, Some(&c), 0, 1).is_ok(),
            (1..=3).contains(&kind)
        );
    }
    for (zone, creature) in [
        (None, None),
        (Some(7), None),
        (None, Some(7)),
        (Some(7), Some(8)),
    ] {
        assert_eq!(
            initializer_limits(zone, creature, 1, Some(&c), 0, 1),
            Err(Failure::Reason(2))
        );
    }
    assert_eq!(
        initializer_limits(Some(7), Some(7), 1, None, 0, 1),
        Err(Failure::Reason(2))
    );
}

#[test]
fn controlled_creature_selection_and_action_rejection() {
    for selector in [0, 1, 2, 255] {
        for control in [None, Some(0), Some(1), Some(2), Some(3)] {
            assert_eq!(
                selects_summon(selector, control).is_ok(),
                selector != 1 || matches!(control, Some(1 | 2))
            );
        }
    }
    for kind in [1, 2, 3] {
        for action in [10, 11, 12] {
            assert_eq!(
                selected_creature(kind, action).is_err(),
                kind == 2 && action == 11
            );
        }
    }
}

#[test]
fn cardinal_cells_reject_duplicates_diagonals_and_gaps() {
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        assert!(records(&[cell(3, 3, 0), cell(3 + dx, 3 + dy, 0)], &[mt(0)], 0).is_ok());
    }
    for (dx, dy) in [(0, 0), (1, 1), (-1, -1), (2, 0), (0, 2)] {
        assert_eq!(
            records(&[cell(3, 3, 0), cell(3 + dx, 3 + dy, 0)], &[mt(0)], 0),
            Err(Failure::Reason(117))
        );
    }
    assert_eq!(
        records(&[cell(i16::MIN, 0, 0), cell(i16::MAX, 0, 0)], &[mt(0)], 0),
        Err(Failure::Reason(117))
    );
}

#[test]
fn negative_cell_indices_fail_but_zero_passes() {
    for (x, y) in [(0, 0), (1, 0), (0, 1), (-1, 0), (0, -1)] {
        assert_eq!(
            records(&[cell(x, y, -1)], &[mt(0)], 0).is_ok(),
            x >= 0 && y >= 0
        );
    }
    assert_eq!(records(&[], &[mt(0)], 0), Err(Failure::Reason(2)));
    assert_eq!(records(&[cell(0, 0, 0)], &[], 0), Err(Failure::Reason(2)));
}

#[test]
fn every_move_type_code_is_checked() {
    for code in i8::MIN..=i8::MAX {
        let allowed = matches!(code, 0..=18 | 22..=36 | 40..=42);
        assert_eq!(valid_move_type(code), allowed);
        assert_eq!(records(&[cell(0, 0, 0)], &[mt(code)], 0).is_ok(), allowed);
    }
}

#[test]
fn marker_cell_bound_is_inclusive_and_time_is_scaled_by_five() {
    let cells = [cell(0, 0, 0), cell(1, 0, 0)];
    for start_cell in [-1, 0, 1, 2, 3] {
        let t = MoveType {
            start_cell,
            ..mt(0)
        };
        assert_eq!(
            records(&cells, &[t], 0).is_ok(),
            (0..=2).contains(&start_cell)
        );
    }
    for time in [-1, 0, 1, 2, 3] {
        for duration in [0, 4, 5, 9, 10, 11] {
            let t = MoveType {
                start_time: time,
                ..mt(0)
            };
            assert_eq!(
                records(&cells, &[t], duration).is_ok(),
                time >= 0 && 5 * time as u32 <= duration
            );
        }
    }
}

#[test]
fn markers_are_nondecreasing_not_strictly_increasing() {
    let cells = [cell(0, 0, 0), cell(1, 0, 0)];
    let a = MoveType {
        start_cell: 1,
        start_time: 1,
        ..mt(0)
    };
    assert!(records(&cells, &[a, a], 10).is_ok());
    assert!(records(
        &cells,
        &[
            a,
            MoveType {
                start_cell: 2,
                start_time: 2,
                ..a
            }
        ],
        10
    )
    .is_ok());
    assert_eq!(
        records(&cells, &[a, MoveType { start_cell: 0, ..a }], 10),
        Err(Failure::Reason(122))
    );
    assert_eq!(
        records(&cells, &[a, MoveType { start_time: 0, ..a }], 10),
        Err(Failure::Reason(123))
    );
}

struct GeometryFixture {
    origin: [i32; 2],
    replies: Vec<Option<Layer>>,
    calls: RefCell<Vec<(i16, i16, i32, i32)>>,
    enabled: bool,
}
impl Geometry for GeometryFixture {
    fn origin(&self) -> [i32; 2] {
        self.origin
    }
    fn layer(&self, x: i16, y: i16, lo: i32, hi: i32) -> Option<Layer> {
        let mut calls = self.calls.borrow_mut();
        let index = calls.len();
        calls.push((x, y, lo, hi));
        self.replies.get(index).copied().flatten()
    }
    fn environment_enabled(&self, _: u64) -> bool {
        self.enabled
    }
}
fn geometry(replies: Vec<Option<Layer>>) -> GeometryFixture {
    GeometryFixture {
        origin: [0, 0],
        replies,
        calls: RefCell::new(vec![]),
        enabled: false,
    }
}

#[test]
fn endpoint_uses_truncation_toward_zero_not_floor_division() {
    for x in [-7, -1, 0, 7] {
        let g = geometry(vec![Some(layer(0, 3, 0))]);
        assert!(endpoint(&g, cell(0, 0, 0), p(x, 0, 0)).is_ok());
    }
    for x in [-8, 8] {
        let g = geometry(vec![]);
        assert_eq!(
            endpoint(&g, cell(0, 0, 0), p(x, 0, 0)),
            Err(Failure::UnmappedReason(UnmappedGate::PositionCell))
        );
        assert!(g.calls.borrow().is_empty());
    }
    let mut g = geometry(vec![Some(layer(0, 3, 0))]);
    g.origin = [11, -9];
    assert!(endpoint(&g, cell(1, 1, 0), p(19, -1, 0)).is_ok());
}

#[test]
fn endpoint_inclusive_floor_ceiling_and_environment() {
    for z in [-1, 0, 3, 4] {
        let g = geometry(vec![Some(layer(0, 3, 0))]);
        assert_eq!(
            endpoint(&g, cell(0, 0, 0), p(0, 0, z)).is_ok(),
            (0..=3).contains(&z)
        );
    }
    let env = Layer {
        environment: 17,
        ..layer(0, 3, 0)
    };
    let g = geometry(vec![Some(env), Some(env)]);
    assert!(endpoint(&g, cell(0, 0, 0), p(0, 0, 0)).is_err());
    let mut g = geometry(vec![Some(env)]);
    g.enabled = true;
    assert!(endpoint(&g, cell(0, 0, 0), p(0, 0, 0)).is_ok());
}

#[test]
fn endpoint_fallback_adjusts_matching_height_and_requeries() {
    let floor = layer(4, 9, 0);
    let g = geometry(vec![None, Some(floor), Some(floor)]);
    let (c, pos, _) = endpoint(&g, cell(0, 0, 2), p(0, 0, 2)).unwrap();
    assert_eq!(c.z, 4);
    assert_eq!(pos.z, 4);
    assert_eq!(
        *g.calls.borrow(),
        [(0, 0, 2, 3), (0, 0, -8, 12), (0, 0, 4, 5)]
    );
    let g = geometry(vec![None, Some(floor), Some(floor)]);
    assert_eq!(endpoint(&g, cell(0, 0, 2), p(0, 0, 6)).unwrap().1.z, 6);
    for replies in [vec![None, None], vec![None, Some(floor), None]] {
        assert!(endpoint(&geometry(replies), cell(0, 0, 2), p(0, 0, 2)).is_err());
    }
}

#[test]
fn layer_queries_widen_short_bounds_and_check_environments() {
    let g = geometry(vec![None, None]);
    let _ = endpoint(&g, cell(0, 0, i16::MAX), p(0, 0, i16::MAX));
    assert_eq!(g.calls.borrow()[0].3, 32768);
    assert_eq!(g.calls.borrow()[1].3, 32777);
    assert!(path_layer(&geometry(vec![Some(layer(0, 9, 0))]), cell(0, 0, 0)).is_ok());
    assert!(path_layer(&geometry(vec![None]), cell(0, 0, 0)).is_err());
    let env = Layer {
        environment: 7,
        ..layer(0, 9, 0)
    };
    assert_eq!(
        path_layer(&geometry(vec![Some(env)]), cell(0, 0, 0)),
        Err(Failure::UnmappedReason(UnmappedGate::Environment))
    );
}

#[test]
fn interval_conjunction_is_not_replaced_with_a_new_rule() {
    assert!(vertical_intervals(layer(0, 10, 0), layer(0, 10, 0)).is_ok());
    assert!(vertical_intervals(layer(0, 10, 0), layer(10, 20, 0)).is_ok());
    assert!(vertical_intervals(layer(0, 0, 0), layer(0, 0, 0)).is_err());
    assert!(vertical_intervals(layer(3, 1, 0), layer(2, 2, 0)).is_err());
}

#[test]
fn start_position_each_axis_boundary_and_signed_tolerance() {
    for axis in 0..3 {
        let limit = if axis == 2 { 10 } else { 8 };
        for delta in [-limit - 1, -limit, 0, limit, limit + 1] {
            let mut xyz = [0; 3];
            xyz[axis] = delta;
            assert_eq!(
                start_position(p(xyz[0], xyz[1], xyz[2]), p(0, 0, 0), 3).is_ok(),
                delta.abs() <= limit
            );
        }
    }
    assert!(start_position(p(0, 0, 0), p(0, 0, 0), -5).is_ok());
    assert!(start_position(p(0, 0, 0), p(0, 0, 0), -6).is_err());
    assert!(start_position(p(i16::MIN, 0, 0), p(i16::MAX, 0, 0), i16::MAX).is_err());
}

#[test]
fn zero_duration_requires_all_coordinates_equal() {
    assert!(zero_duration(0, p(1, 2, 3), p(1, 2, 3)).is_ok());
    for end in [p(2, 2, 3), p(1, 3, 3), p(1, 2, 4)] {
        assert!(zero_duration(0, p(1, 2, 3), end).is_err());
        assert!(zero_duration(1, p(1, 2, 3), end).is_ok());
    }
}

#[test]
fn correction_uses_server_fields_and_preserves_mismatch_counter() {
    assert!(transition_counter(7, 7).is_ok());
    assert_eq!(transition_counter(6, 7), Err(Failure::Reason(104)));
    let c = correction(Failure::Reason(104), 8, p(4, 5, 6), 7).unwrap();
    assert_eq!(
        (
            c.command,
            c.reason,
            c.movement,
            c.position,
            c.transition_counter,
            c.end_action
        ),
        (65, 104, 8, p(4, 5, 6), 7, true)
    );
    assert_eq!(
        correction(Failure::Reason(112), 0, p(4, 5, 6), u16::MAX)
            .unwrap()
            .transition_counter,
        0
    );
    assert!(correction(
        Failure::UnmappedReason(UnmappedGate::Speed),
        0,
        p(0, 0, 0),
        0
    )
    .is_none());
    for reason in [2, 102, 103] {
        assert_eq!(
            handler_failure(reason),
            Correction {
                command: 65,
                reason,
                movement: 0,
                position: Position::default(),
                transition_counter: 0,
                end_action: false
            }
        );
    }
}

fn sources() -> SpeedSources {
    SpeedSources {
        creature_speed: 60,
        config_548: 50,
        selected_record_24: None,
        config_552: 3,
        config_556: 7,
        prior_speed_3904: 0,
        prior_age: 0,
        upward_enabled_1497: false,
        upward_speed_1500: 0,
    }
}

#[test]
fn all_speed_sources_and_type_14_flag() {
    let s = sources();
    for code in 0..=42 {
        if !valid_move_type(code) {
            assert_eq!(selected_speed(mt(code), &s), Err(Failure::Reason(119)));
            continue;
        }
        let expected = match code {
            0 | 1 => 50,
            2..=4 | 14 => 60,
            5..=13 | 34..=36 => 109,
            15 => 30,
            16..=18 => 79,
            22..=24 => 100,
            25..=27 => 90,
            _ => 94,
        };
        assert_eq!(selected_speed(mt(code), &s), Ok(expected));
    }
    assert_eq!(
        selected_speed(
            MoveType {
                flag: true,
                ..mt(14)
            },
            &s
        ),
        Ok(109)
    );
    assert_eq!(
        selected_speed(
            mt(0),
            &SpeedSources {
                selected_record_24: Some(81),
                ..s
            }
        ),
        Ok(81)
    );
    assert_eq!(
        selected_speed(
            mt(15),
            &SpeedSources {
                creature_speed: 61,
                ..s
            }
        ),
        Ok(30)
    );
    assert_eq!(
        selected_speed(
            mt(25),
            &SpeedSources {
                creature_speed: 61,
                ..s
            }
        ),
        Ok(91)
    );
}

#[test]
fn speed_maximum_margins_and_stale_prior_order() {
    let mut s = sources();
    s.prior_speed_3904 = 200;
    for age in [499, 500, 501] {
        s.prior_age = age;
        let r = speed_check(p(0, 0, 0), p(0, 0, 0), 1, &[mt(0), mt(5)], &s).unwrap();
        assert_eq!((r.selected_speed, r.speed_with_margin), (200, 207));
        assert_eq!(r.retained_prior_speed, if age > 500 { 0 } else { 200 });
    }
    s.prior_speed_3904 = 0;
    assert_eq!(
        speed_check(p(0, 0, 0), p(0, 0, 0), 1, &[mt(5), mt(2)], &s)
            .unwrap()
            .speed_with_margin,
        112
    );
    assert!(speed_check(p(0, 0, 0), p(0, 0, 0), 1, &[], &s).is_err());
    assert_eq!(
        speed_check(p(0, 0, 0), p(0, 0, 0), 1, &[mt(19)], &s),
        Err(Failure::Reason(119))
    );
}

#[test]
fn speed_squared_boundary_and_one_unit_xy_tolerance() {
    let s = SpeedSources {
        creature_speed: 10,
        config_552: 0,
        ..sources()
    };
    for distance in [0, 1, 5, 6, 7] {
        assert_eq!(
            speed_check(p(0, 0, 0), p(distance, 1, 0), 500, &[mt(2)], &s).is_ok(),
            distance <= 6
        );
        assert_eq!(
            speed_check(p(0, 0, 0), p(-distance, -1, 0), 500, &[mt(2)], &s).is_ok(),
            distance <= 6
        );
    }
    assert!(speed_check(p(0, 0, 0), p(4, 5, 0), 500, &[mt(2)], &s).is_ok()); // dx=3, dy=4
    assert!(speed_check(p(0, 0, 0), p(5, 5, 0), 500, &[mt(2)], &s).is_err());
    assert!(speed_check(p(0, 0, 0), p(30, 30, 0), 0, &[mt(2)], &s).is_ok()); // separate zero-duration gate
}

#[test]
fn upward_speed_is_conditional_and_equality_passes() {
    let s = SpeedSources {
        upward_enabled_1497: true,
        upward_speed_1500: 10,
        ..sources()
    };
    for z in [-6, 0, 4, 5, 6] {
        assert_eq!(
            speed_check(p(0, 0, 0), p(0, 0, z), 500, &[mt(2)], &s).is_ok(),
            z <= 5
        );
    }
    assert!(speed_check(
        p(0, 0, 0),
        p(0, 0, 6),
        500,
        &[mt(2)],
        &SpeedSources {
            upward_enabled_1497: false,
            ..s
        }
    )
    .is_ok());
}

#[test]
fn duration_history_optional_enforced_and_boundary() {
    assert_eq!(
        duration_history(None, 550),
        DurationResult {
            excess: false,
            accepted: true
        }
    );
    for enforce in [false, true] {
        for request in [9, 10, 11] {
            let h = DurationHistory {
                rolling_total: 20,
                elapsed_server_ticks: 25,
                config_560: 5,
                enforce_564: enforce,
            };
            assert_eq!(
                duration_history(Some(h), request),
                DurationResult {
                    excess: request > 10,
                    accepted: !enforce || request <= 10
                }
            );
        }
    }
    let h = DurationHistory {
        rolling_total: u64::MAX,
        elapsed_server_ticks: u64::MAX,
        config_560: 0,
        enforce_564: true,
    };
    assert!(!duration_history(Some(h), 1).accepted);
}

fn hi(state: u8) -> HeightInput {
    HeightInput {
        native_kind: 1,
        server_state_2300: state,
        origin: layer(0, 30, 0),
        next: layer(0, 30, 0),
        client_origin_z: 0,
        client_next_z: 0,
        move_type: 0,
        short_2256: 0,
        glide_2288: 0,
        boost_2292: 0,
        dive_2296: 0,
        jump_global: Some(0),
    }
}
fn accepted(i: HeightInput) -> bool {
    height_state(i).is_ok()
}

#[test]
fn height_kind_and_unknown_states_match_native_helper() {
    for kind in [0, 1, 2, 3, 255] {
        assert_eq!(
            accepted(HeightInput {
                native_kind: kind,
                ..hi(0)
            }),
            kind == 1
        );
    }
    for state in [11, 12, 13, 255] {
        assert_eq!(height_state(hi(state)), Ok(HeightResult::Accepted));
    }
}

#[test]
fn run_and_sprint_ground_delta_and_attribute_boundaries() {
    for state in [0, 4] {
        for delta in [-25, -24, 0, 20, 21] {
            let input = HeightInput {
                next: layer(delta, delta + 30, 0),
                client_next_z: delta,
                ..hi(state)
            };
            assert_eq!(accepted(input), (-24..=20).contains(&delta));
        }
        assert_eq!(
            height_state(HeightInput {
                next: layer(0, 30, 1),
                ..hi(state)
            }),
            Err(Failure::Reason(144))
        );
    }
}

#[test]
fn run_fall_and_sprint_air_thresholds() {
    assert_eq!(
        height_state(HeightInput {
            client_next_z: 11,
            ..hi(0)
        }),
        Ok(HeightResult::Accepted)
    );
    assert_eq!(
        height_state(HeightInput {
            client_next_z: 12,
            ..hi(0)
        }),
        Ok(HeightResult::Change { state: 2, at_z: 0 })
    );
    assert!(accepted(HeightInput {
        client_next_z: 22,
        ..hi(4)
    }));
    assert!(!accepted(HeightInput {
        client_next_z: 23,
        client_origin_z: 22,
        ..hi(4)
    }));
    assert!(accepted(HeightInput {
        client_next_z: 23,
        client_origin_z: 23,
        ..hi(4)
    }));
    assert!(accepted(HeightInput {
        client_next_z: 23,
        client_origin_z: 24,
        ..hi(4)
    }));
}

#[test]
fn jump_stops_at_unknown_global_or_tracking_and_checks_height() {
    assert_eq!(
        height_state(HeightInput {
            jump_global: None,
            ..hi(1)
        }),
        Ok(HeightResult::Pending(HeightPending::JumpGlobal))
    );
    for z in [-1, 0] {
        assert_eq!(
            height_state(HeightInput {
                client_next_z: z,
                ..hi(1)
            }),
            Ok(HeightResult::Pending(HeightPending::JumpTrackingOrEndJump))
        );
    }
    for attr in [0, 1, 2] {
        assert_eq!(
            height_state(HeightInput {
                client_next_z: 11,
                next: layer(0, 30, attr),
                ..hi(1)
            }),
            Ok(HeightResult::StateOnly {
                state: if attr == 1 { 8 } else { 0 }
            })
        );
    }
    assert_eq!(
        height_state(HeightInput {
            client_next_z: 12,
            ..hi(1)
        }),
        Err(Failure::Reason(113))
    );
}

#[test]
fn fall_drop_bypass_rising_threshold_and_landing_boundaries() {
    assert_eq!(
        height_state(HeightInput {
            client_next_z: -1001,
            ..hi(2)
        }),
        Err(Failure::Reason(115))
    );
    assert!(accepted(HeightInput {
        client_next_z: -1000,
        ..hi(2)
    }));
    assert_eq!(
        height_state(HeightInput {
            client_next_z: -1001,
            move_type: 14,
            ..hi(2)
        }),
        Ok(HeightResult::Accepted)
    );
    assert_eq!(
        height_state(HeightInput {
            client_next_z: 16,
            ..hi(2)
        }),
        Ok(HeightResult::Accepted)
    );
    assert_eq!(
        height_state(HeightInput {
            client_next_z: 17,
            ..hi(2)
        }),
        Ok(HeightResult::Pending(HeightPending::RisingFallHelper))
    );
    assert_eq!(
        height_state(HeightInput {
            client_next_z: 17,
            client_origin_z: 17,
            ..hi(2)
        }),
        Ok(HeightResult::Accepted)
    );
    for attr in [0, 1, 2] {
        assert_eq!(
            height_state(HeightInput {
                client_next_z: 9,
                next: layer(0, 30, attr),
                ..hi(2)
            }),
            Ok(HeightResult::Change {
                state: if attr == 0 { 0 } else { 8 },
                at_z: 0
            })
        );
    }
    assert_eq!(
        height_state(HeightInput {
            client_next_z: 10,
            ..hi(2)
        }),
        Ok(HeightResult::Accepted)
    );
    assert_eq!(rising_fall_result(false), Err(Failure::Reason(114)));
    assert_eq!(
        rising_fall_result(true),
        Ok(HeightResult::StateOnly { state: 3 })
    );
}

#[test]
fn state_three_lands_strictly_below_eleven() {
    assert_eq!(
        height_state(HeightInput {
            client_next_z: 10,
            ..hi(3)
        }),
        Ok(HeightResult::Change { state: 0, at_z: 0 })
    );
    assert_eq!(
        height_state(HeightInput {
            client_next_z: 11,
            ..hi(3)
        }),
        Ok(HeightResult::Accepted)
    );
}

#[test]
fn sprint_jump_glide_boost_dive_caps_include_equality() {
    for (state, cap, reason) in [(5, 135, 113), (7, 25, 146), (9, 5, 153), (10, 0, 154)] {
        for z in [cap - 1, cap, cap + 1] {
            let result = height_state(HeightInput {
                client_next_z: z,
                ..hi(state)
            });
            if z <= cap {
                assert_eq!(result, Ok(HeightResult::Accepted));
            } else {
                assert_eq!(result, Err(Failure::Reason(reason)));
            }
        }
    }
    assert!(accepted(HeightInput {
        glide_2288: i32::MAX,
        ..hi(7)
    }));
}

#[test]
fn swim_and_drift_delta_attribute_and_fall_limits() {
    for state in [6, 8] {
        for delta in [-16, -15, 0, 15, 16] {
            let input = HeightInput {
                next: layer(delta, delta + 30, 1),
                client_next_z: delta,
                ..hi(state)
            };
            assert_eq!(accepted(input), (-15..=15).contains(&delta));
        }
        for attr in [0, 1, 2] {
            assert_eq!(
                accepted(HeightInput {
                    next: layer(0, 30, attr),
                    ..hi(state)
                }),
                attr == 1
            );
        }
        assert_eq!(
            height_state(HeightInput {
                next: layer(0, 30, 1),
                client_next_z: 11,
                ..hi(state)
            }),
            Ok(HeightResult::Accepted)
        );
        assert_eq!(
            height_state(HeightInput {
                client_next_z: 12,
                ..hi(state)
            }),
            Ok(HeightResult::Change { state: 2, at_z: 0 })
        );
    }
}
