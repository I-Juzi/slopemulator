use gamedaemon::handlers::*;
fn run(
    selector: u8,
    yaw: u16,
    zone: Option<u64>,
    summon: Option<Summon>,
    gate: Gate,
) -> (Outcome, MemoryServices) {
    let mut session = Session {
        authenticated: true,
        dispatch_permission: Some(true),
        pc: Some(Pc {
            zone,
            summon,
            ..Pc::default()
        }),
        ..Session::default()
    };
    let mut services = MemoryServices {
        queue_result: gate,
        ..MemoryServices::default()
    };
    let body = [vec![selector], yaw.to_le_bytes().to_vec()].concat();
    let out = dispatch(
        &mut Context {
            session: &mut session,
            world: &World::default(),
            services: &mut services,
        },
        66,
        &body,
    );
    (out, services)
}
#[test]
fn turn_failure_uses_reason_two_and_original_selector() {
    for selector in [0, 2, 42, 255] {
        for (zone, yaw, gate) in [
            (None, 0, Gate::Pass),
            (Some(1), 360, Gate::Pass),
            (Some(1), u16::MAX, Gate::Pass),
            (Some(1), 359, Gate::Fail),
        ] {
            let (out, services) = run(selector, yaw, zone, None, gate);
            assert_eq!(out.status, Status::Handled);
            assert_eq!(
                out.replies,
                vec![Message {
                    command: 67,
                    body: vec![2, 0, selector]
                }]
            );
            assert!(out.changes.is_empty());
            assert!(out.broadcasts.is_empty());
            assert_eq!(
                services.queued,
                if zone.is_none() || yaw > 359 {
                    vec![]
                } else {
                    vec![66]
                }
            );
        }
    }
}
#[test]
fn turn_summon_resolution_and_selected_zone_are_separate_gates() {
    for state in [0, 1, 2, 3, 255] {
        let summon = Summon {
            control_state: state,
            native_kind: 2,
            race: 0,
            has_entry: true,
            zone: None,
            position: Default::default(),
            yaw: 0,
        };
        let (out, svc) = run(1, 0, Some(1), Some(summon), Gate::Pass);
        assert_eq!(
            out.replies,
            vec![Message {
                command: 67,
                body: vec![2, 0, 1]
            }]
        );
        assert!(svc.queued.is_empty());
    }
    let (out, svc) = run(1, 0, Some(1), None, Gate::Pass);
    assert_eq!(out.replies[0].body, vec![2, 0, 1]);
    assert!(svc.queued.is_empty());
}
#[test]
fn turn_valid_wrapper_and_unknown_queue_do_not_update_yaw_or_broadcast() {
    for yaw in [0, 359] {
        for gate in [Gate::Pass, Gate::Unknown] {
            let (out, svc) = run(0, yaw, Some(1), None, gate);
            assert!(matches!(out.status, Status::Unsupported(_)));
            assert_eq!(svc.queued, vec![66]);
            assert!(out.replies.is_empty());
            assert!(out.broadcasts.is_empty());
            assert_eq!(
                out.changes.contains(&StateChange::Queued(66)),
                gate == Gate::Pass
            );
        }
    }
}
fn activated(property: PackedProperty, gate: Gate) -> Outcome {
    let mut s = Session {
        authenticated: true,
        dispatch_permission: Some(true),
        pc: Some(Pc {
            zone: Some(1),
            ..Pc::default()
        }),
        ..Session::default()
    };
    let mut svc = MemoryServices {
        queue_result: gate,
        activated_teleport_result: property,
        ..MemoryServices::default()
    };
    dispatch(
        &mut Context {
            session: &mut s,
            world: &World::default(),
            services: &mut svc,
        },
        99,
        &[],
    )
}
#[test]
fn activated_teleport_appends_exactly_one_packed_property_block() {
    for bytes in [vec![3, 0, 1, 2, 3], vec![0, 0]] {
        let out = activated(PackedProperty::Bytes(bytes.clone()), Gate::Pass);
        assert_eq!(out.status, Status::Handled);
        assert_eq!(
            out.replies,
            vec![Message {
                command: 100,
                body: bytes
            }]
        );
        assert!(out.changes.contains(&StateChange::Queued(99)));
    }
}
#[test]
fn activated_fetch_failure_differs_from_a_counted_empty_property() {
    let out = activated(PackedProperty::FetchFailed, Gate::Pass);
    assert_eq!(
        out.replies,
        vec![Message {
            command: 100,
            body: vec![]
        }]
    );
    assert_eq!(out.status, Status::Handled);
    let out = activated(PackedProperty::Unknown, Gate::Pass);
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert!(out.replies.is_empty());
}
#[test]
fn activated_queue_gate_and_capacity_fail_closed() {
    for gate in [Gate::Fail, Gate::Unknown] {
        let out = activated(PackedProperty::Bytes(vec![1, 0, 7]), gate);
        if gate == Gate::Fail {
            assert_eq!(
                out.replies,
                vec![Message {
                    command: 101,
                    body: vec![2, 0]
                }]
            );
        } else {
            assert!(out.replies.is_empty());
            assert!(matches!(out.status, Status::Unsupported(_)));
        }
        assert!(out.changes.is_empty());
    }
    let out = activated(PackedProperty::Bytes(vec![0; 3997]), Gate::Pass);
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert!(out.replies.is_empty());
}
