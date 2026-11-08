//! Generated server objects. No client data or game records.
use gamedaemon::{kick::*, validation::distance::*, Position};
fn p(x: i16, y: i16, z: i16) -> Position {
    Position { x, y, z }
}

#[test]
fn strict_trade_boundary_and_failure_commands() {
    for (path, command, reason) in [
        (Interaction::TradeInvite, 312, 908),
        (Interaction::TradeJoin, 315, 917),
    ] {
        for distance in [0, 62, 63, 64] {
            let result = interaction(path, p(0, 0, 0), p(distance, 0, 0));
            if distance < 63 {
                assert!(result.is_ok());
            } else {
                assert_eq!(
                    result,
                    Err(DistanceFailure {
                        command: Some(command),
                        reason
                    })
                );
            }
        }
        assert!(interaction(path, p(0, 0, 0), p(0, 0, 63)).is_err());
        assert!(interaction(path, p(0, 0, 0), p(0, 63, 0)).is_err());
    }
}

#[test]
fn npc_inclusive_boundaries_and_reply_contracts() {
    use Interaction::*;
    for (path, command, reason) in [
        (ExchangeFactionScore, Some(85), 334),
        (MoveDepot, None, 301),
        (Buy, Some(1342), 306),
        (Sell, Some(1349), 318),
        (Rebuy, None, 326),
        (TradeWithNpc, None, 341),
        (Buy2Npc, Some(1344), 35),
        (Repair, Some(1386), 451),
    ] {
        for distance in [0, 99, 100, 101] {
            let result = interaction(path, p(0, 0, 0), p(distance, 0, 0));
            if distance <= 100 {
                assert!(result.is_ok());
            } else {
                assert_eq!(result, Err(DistanceFailure { command, reason }));
            }
        }
        assert!(interaction(path, p(0, 0, 0), p(60, 0, 80)).is_ok());
        assert!(interaction(path, p(0, 0, 0), p(60, 1, 80)).is_err());
    }
}

#[test]
fn distance_extreme_shorts_do_not_overflow() {
    assert!(interaction(
        Interaction::Buy,
        p(i16::MIN, i16::MIN, i16::MIN),
        p(i16::MAX, i16::MAX, i16::MAX)
    )
    .is_err());
}

#[test]
fn mapped_inventory_bypasses() {
    for helper in [-1, 0, 1] {
        for permission in [false, true] {
            assert_eq!(
                depot_bypasses_npc(helper, permission),
                helper > 0 || permission
            );
        }
    }
    assert!(!buy2_uses_npc(0));
    assert!(buy2_uses_npc(1));
    assert!(buy2_uses_npc(u64::MAX));
}

struct Object {
    kind: u8,
    pos: Position,
    virtual_pos: Option<Position>,
}
impl QuestObject for Object {
    fn native_kind(&self) -> u8 {
        self.kind
    }
    fn position(&self) -> Position {
        self.pos
    }
    fn virtual_152_position(&self) -> Option<Position> {
        self.virtual_pos
    }
}
fn object(kind: u8, x: i16) -> Object {
    Object {
        kind,
        pos: p(x, 0, 0),
        virtual_pos: None,
    }
}

#[test]
fn quest_null_and_kind_checks_precede_mode_two_bypass() {
    assert_eq!(
        quest(None, None, 0, 2),
        Err(QuestFailure::UnmappedPointerOrKind)
    );
    let source = object(1, 0);
    for kind in [0, 1, 2, 4, 32, 240, 241, 255] {
        let target = object(kind, 300);
        assert_eq!(
            quest(Some(&source), Some(&target), 0, 2).is_ok(),
            matches!(kind, 1 | 2 | 4 | 32 | 240 | 241)
        );
        assert_eq!(
            quest(Some(&target), Some(&source), 0, 2).is_ok(),
            matches!(kind, 1 | 2 | 4 | 240 | 241)
        );
    }
    assert!(quest(Some(&source), None, 0, 2).is_err());
    assert!(quest(None, Some(&source), 0, 2).is_err());
}

#[test]
fn quest_inclusive_distance_and_negative_maximum() {
    let source = object(1, 0);
    for max in [0, 7, -7] {
        for x in [0, 6, 7, 8] {
            assert_eq!(
                quest(Some(&source), Some(&object(4, x)), max, 1).is_ok(),
                i32::from(x) <= i32::abs(max)
            );
        }
    }
    assert_eq!(
        quest(Some(&source), Some(&object(4, 8)), 7, 1),
        Err(QuestFailure::Reason(1010))
    );
    assert!(quest(Some(&source), Some(&object(4, i16::MAX)), i32::MIN, 1).is_ok());
}

#[test]
fn quest_kind_32_uses_virtual_server_position() {
    let source = object(1, 0);
    let mut target = object(32, 300);
    assert_eq!(
        quest(Some(&source), Some(&target), 7, 1),
        Err(QuestFailure::Reason(1014))
    );
    target.virtual_pos = Some(p(7, 0, 0));
    assert!(quest(Some(&source), Some(&target), 7, 1).is_ok());
    target.virtual_pos = Some(p(8, 0, 0));
    assert_eq!(
        quest(Some(&source), Some(&target), 7, 1),
        Err(QuestFailure::Reason(1010))
    );
}

#[test]
fn cylinder_radial_vertical_and_target_size_boundaries() {
    for x in [2, 3, 4, 5, 6] {
        assert_eq!(
            skill_cylinder(p(0, 0, 0), p(x, 0, 0), 3, 5, 7, 9).is_ok(),
            (3..=5).contains(&x)
        );
    }
    for z in [-10, -9, -8, 0, 6, 7, 8] {
        assert_eq!(
            skill_cylinder(p(0, 0, 0), p(3, 0, z), 3, 5, 7, 9).is_ok(),
            (-9..=7).contains(&z)
        );
    }
    assert_eq!(sized_skill_ranges(0, 7, 3), (0, 10));
    assert_eq!(sized_skill_ranges(2, 7, 3), (5, 10));
    assert_eq!(
        sized_skill_ranges(i16::MAX, i16::MAX, i16::MAX),
        (65534, 65534)
    );
}

struct Target {
    calls: Vec<u8>,
    fail: Option<u8>,
}
impl Target {
    fn step(&mut self, step: u8) -> bool {
        self.calls.push(step);
        self.fail != Some(step)
    }
}
impl SkillTargetChecks for Target {
    fn range(&mut self) -> bool {
        self.step(0)
    }
    fn direction(&mut self) -> bool {
        self.step(1)
    }
    fn target_filter(&mut self) -> Result<(), i16> {
        if self.step(2) {
            Ok(())
        } else {
            Err(777)
        }
    }
    fn line_of_sight(&mut self) -> bool {
        self.step(3)
    }
}
#[test]
fn ordinary_target_order_and_propagated_filter_reason() {
    for (step, reason) in [(0, 195), (1, 194), (2, 777), (3, 210)] {
        let mut target = Target {
            calls: vec![],
            fail: Some(step),
        };
        assert_eq!(ordinary_skill_target(&mut target), Err(reason));
        assert_eq!(target.calls, (0..=step).collect::<Vec<_>>());
    }
    let mut target = Target {
        calls: vec![],
        fail: None,
    };
    assert!(ordinary_skill_target(&mut target).is_ok());
    assert_eq!(target.calls, [0, 1, 2, 3]);
}

#[test]
fn backend_reason_switch_is_complete_including_default_gaps() {
    for code in -1..=64 {
        let expected = match code {
            0 => 0,
            2..=8 | 21..=23 | 45 => 2,
            15..=19 => 1959 + code - 15,
            25..=35 => 16 + code - 25,
            41 => 27,
            46..=48 => 2515 + code - 46,
            49..=55 => 2523 + code - 49,
            56 | 57 => 2532,
            59..=62 => 2533 + code - 59,
            _ => 1,
        };
        assert_eq!(backend_reason(code), expected as i16, "backend {code}");
    }
    assert_eq!(backend_reason(i32::MIN), 1);
    assert_eq!(backend_reason(i32::MAX), 1);
}

#[test]
fn direct_failure_reasons_and_supplied_signed_shorts() {
    assert_eq!(FailureReason::FinishOrTaskInitialization.client_reason(), 2);
    assert_eq!(FailureReason::TryDrawTicket.client_reason(), 2530);
    assert_eq!(FailureReason::ConditionOrConsume.client_reason(), 2531);
    assert_eq!(FailureReason::Abort(None).client_reason(), 1);
    assert_eq!(FailureReason::Backend(59).client_reason(), 2533);
    for reason in [i16::MIN, -1, 0, 1, 2531, i16::MAX] {
        assert_eq!(FailureReason::Abort(Some(reason)).client_reason(), reason);
        assert_eq!(FailureReason::Expiry(reason).client_reason(), reason);
    }
}

#[test]
fn failure_plan_preserves_session_release_kick_condition_and_state_order() {
    use FailureAction::*;
    for session in [false, true] {
        for kick in [false, true] {
            let actions = board_gacha_failure(session, kick, FailureReason::Expiry(-7));
            let mut expected = if session {
                vec![
                    SendReply {
                        command: 1178,
                        client_reason: -7,
                    },
                    ReleaseSessionReference,
                ]
            } else {
                vec![LogMissingSession]
            };
            if kick {
                expected.extend([LogKick, EnterUserManagerKickBoundary]);
            }
            expected.extend([LogTransactionFailure, SetTransactionState(5)]);
            assert_eq!(actions, expected);
        }
    }
}
