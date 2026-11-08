use gamedaemon::handlers::*;

fn ready() -> Session {
    Session {
        authenticated: true,
        dispatch_permission: Some(true),
        pc: Some(Pc {
            zone: Some(1),
            user_available: true,
            spawn_action_present: Some(true),
            spawn_state: 2,
            spawn_correlation: 77,
            ..Pc::default()
        }),
        ..Session::default()
    }
}
fn run(
    session: &mut Session,
    world: &World,
    services: &mut MemoryServices,
    id: u16,
    body: &[u8],
) -> Outcome {
    dispatch(
        &mut Context {
            session,
            world,
            services,
        },
        id,
        body,
    )
}
fn services() -> MemoryServices {
    MemoryServices {
        queue_result: Gate::Pass,
        ..MemoryServices::default()
    }
}
fn hello() -> Vec<u8> {
    [
        1278u16.to_le_bytes().to_vec(),
        0x12D261D120403u64.to_le_bytes().to_vec(),
        vec![0, 0],
        0u64.to_le_bytes().to_vec(),
        vec![1, 0, 99],
    ]
    .concat()
}
fn reason(out: &Outcome, command: u16, reason: u16) {
    assert_eq!(out.status, Status::Handled);
    assert_eq!(
        out.replies,
        vec![Message {
            command,
            body: reason.to_le_bytes().to_vec()
        }]
    );
    assert!(out.kicks.is_empty());
}
#[test]
fn hello_reply_precedes_processor_activation() {
    let mut s = Session {
        hello_mode: Some(3),
        ..Session::default()
    };
    let w = World {
        publisher_code: 1,
        ..World::default()
    };
    let mut svc = services();
    svc.hello_result = HelloOperation::Reply(vec![9, 8, 7]);
    let out = run(&mut s, &w, &mut svc, 11, &hello());
    assert_eq!(out.status, Status::Handled);
    assert_eq!(s.hello_mode, Some(5));
    assert_eq!(
        out.session_effects,
        vec![
            SessionEffect::SendWithCurrentProcessor(Message {
                command: 12,
                body: vec![3, 0, 9, 8, 7]
            }),
            SessionEffect::ActivateSessionProcessor,
        ]
    );
    assert!(out.replies.is_empty()); // The ordered send must not be duplicated.
    assert!(!s.authenticated);
}
#[test]
fn hello_other_modes_do_not_switch_and_unknown_mode_stops_after_send() {
    for mode in [None, Some(0), Some(5), Some(u32::MAX)] {
        let mut s = Session {
            hello_mode: mode,
            ..Session::default()
        };
        let w = World {
            publisher_code: 1,
            ..World::default()
        };
        let mut svc = services();
        svc.hello_result = HelloOperation::Reply(vec![]);
        let out = run(&mut s, &w, &mut svc, 11, &hello());
        assert_eq!(
            out.session_effects,
            vec![SessionEffect::SendWithCurrentProcessor(Message {
                command: 12,
                body: vec![0, 0]
            })]
        );
        assert_eq!(s.hello_mode, mode);
        assert_eq!(matches!(out.status, Status::Unsupported(_)), mode.is_none());
    }
}
#[test]
fn hello_operation_rejection_and_unknown_never_switch_or_send() {
    for operation in [HelloOperation::Unknown, HelloOperation::Rejected] {
        let mut s = Session {
            hello_mode: Some(3),
            ..Session::default()
        };
        let w = World {
            publisher_code: 1,
            ..World::default()
        };
        let mut svc = services();
        let rejected = operation == HelloOperation::Rejected;
        svc.hello_result = operation;
        let out = run(&mut s, &w, &mut svc, 11, &hello());
        assert!(out.session_effects.is_empty());
        assert_eq!(s.hello_mode, Some(3));
        if rejected {
            assert_eq!(out.status, Status::NativeFalse);
        } else {
            assert!(matches!(out.status, Status::Unsupported(_)));
        }
        assert!(out.kicks.is_empty());
    }
}
#[test]
fn hello_capacity_is_a_server_adapter_contract_not_client_approval() {
    for len in [3994, 3995] {
        let mut s = Session {
            hello_mode: Some(3),
            ..Session::default()
        };
        let w = World {
            publisher_code: 1,
            ..World::default()
        };
        let mut svc = services();
        svc.hello_result = HelloOperation::Reply(vec![0; len]);
        let out = run(&mut s, &w, &mut svc, 11, &hello());
        if len == 3994 {
            assert_eq!(out.status, Status::Handled);
            assert_eq!(s.hello_mode, Some(5));
        } else {
            assert!(matches!(out.status, Status::Unsupported(_)));
            assert!(out.session_effects.is_empty());
            assert_eq!(s.hello_mode, Some(3));
        }
    }
}
#[test]
fn hello_publisher_comparison_stops_at_utf16_nul() {
    let mut s = Session {
        hello_mode: Some(3),
        ..Session::default()
    };
    let w = World {
        publisher_code: 1,
        publisher: vec![80],
        ..World::default()
    };
    let mut svc = services();
    svc.hello_result = HelloOperation::Reply(vec![]);
    let mut body = hello()[..10].to_vec();
    body.extend(3u16.to_le_bytes());
    for unit in [80u16, 0, 81] {
        body.extend(unit.to_le_bytes());
    }
    body.extend(&hello()[12..]);
    let out = run(&mut s, &w, &mut svc, 11, &body);
    assert_eq!(out.status, Status::Handled);
    assert_eq!(s.hello_mode, Some(5));
}
#[test]
fn hello_invalid_prefix_never_uses_a_success_result() {
    let mut s = Session {
        hello_mode: Some(3),
        ..Session::default()
    };
    let w = World {
        publisher_code: 1,
        ..World::default()
    };
    let mut svc = services();
    svc.hello_result = HelloOperation::Reply(vec![7]);
    for body in [vec![], {
        let mut b = hello();
        b[0] = 0;
        b
    }] {
        let out = run(&mut s, &w, &mut svc, 11, &body);
        assert_eq!(out.status, Status::NativeFalse);
        assert!(out.session_effects.is_empty());
        assert_eq!(s.hello_mode, Some(3));
    }
}
fn enter_zone_body() -> Vec<u8> {
    [77u32.to_le_bytes().to_vec(), vec![0; 8]].concat()
}
#[test]
fn enter_zone_distinguishes_action_state_and_correlation_in_order() {
    for (present, state, key, expected) in [
        (false, 0, 0, 2),
        (true, 1, 0, 1149),
        (true, 3, 77, 1149),
        (true, 2, 76, 1150),
    ] {
        let mut s = ready();
        let pc = s.pc.as_mut().unwrap();
        pc.spawn_action_present = Some(present);
        pc.spawn_state = state;
        pc.spawn_correlation = key;
        let out = run(
            &mut s,
            &World::default(),
            &mut services(),
            42,
            &enter_zone_body(),
        );
        reason(&out, 44, expected);
        assert!(out.changes.contains(&StateChange::Queued(42)));
        assert!(out.changes.contains(&StateChange::QueuedRequest {
            command: 42,
            body: enter_zone_body()
        }));
    }
}
#[test]
fn enter_zone_unknown_and_matching_actions_never_claim_completed_spawn() {
    for presence in [None, Some(true)] {
        let mut s = ready();
        s.pc.as_mut().unwrap().spawn_action_present = presence;
        let out = run(
            &mut s,
            &World::default(),
            &mut services(),
            42,
            &enter_zone_body(),
        );
        assert!(matches!(out.status, Status::Unsupported(_)));
        assert!(out.replies.is_empty());
        assert!(out.broadcasts.is_empty());
    }
}
#[test]
fn enter_zone_task_checks_never_run_before_queue_success() {
    for gate in [Gate::Fail, Gate::Unknown] {
        let mut s = ready();
        s.pc.as_mut().unwrap().spawn_action_present = Some(false);
        let mut svc = services();
        svc.queue_result = gate;
        let out = run(&mut s, &World::default(), &mut svc, 42, &enter_zone_body());
        assert!(out.changes.is_empty());
        if gate == Gate::Fail {
            reason(&out, 44, 2);
        } else {
            assert!(matches!(out.status, Status::Unsupported(_)));
            assert!(out.replies.is_empty());
        }
    }
}
#[test]
fn enter_zone_non_pc_caster_logs_without_action_reply() {
    let mut s = ready();
    s.pc.as_mut().unwrap().native_kind = 2;
    let out = run(
        &mut s,
        &World::default(),
        &mut services(),
        42,
        &enter_zone_body(),
    );
    assert_eq!(out.status, Status::Handled);
    assert!(out.replies.is_empty());
    assert_eq!(out.logs, vec!["EnterZone task caster is not a PC"]);
}
fn token_request() -> TokenAuthRequest {
    TokenAuthRequest {
        world: 9,
        client_key: 4,
        lobby_token: 5,
    }
}
#[test]
fn token_wrapper_retains_fields_but_does_not_send_or_authenticate() {
    let body = [
        9u16.to_le_bytes().to_vec(),
        4u64.to_le_bytes().to_vec(),
        5u64.to_le_bytes().to_vec(),
    ]
    .concat();
    let mut s = Session::default();
    let mut w = World::default();
    w.worlds.insert(9);
    let out = run(&mut s, &w, &mut services(), 23, &body);
    assert!(out
        .changes
        .contains(&StateChange::LobbyTokenAuthQueued(token_request())));
    assert!(out.daemon_requests.is_empty());
    assert!(out.replies.is_empty());
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert!(!s.authenticated);
}
#[test]
fn token_start_send_and_unknown_callbacks_do_not_emit_command_24() {
    let mut tx = LobbyTokenAuthTransaction::registered(token_request(), 7, 0);
    let out = tx.start();
    assert_eq!(
        out.daemon_requests,
        vec![DaemonRequest::LobbyConsumeToken {
            world: 9,
            transaction_id: 7,
            client_key: 4,
            lobby_token: 5,
            accepted_address_bits: 0,
        }]
    );
    assert!(out.replies.is_empty());
    let out = tx.sent(Gate::Unknown);
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert!(out.changes.is_empty());
    assert!(out.replies.is_empty());
    let out = tx.sent(Gate::Pass);
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert!(out.replies.is_empty());
}
#[test]
fn token_consume_failure_mapping_is_exhaustive_for_short_reasons() {
    for backend in 0..=u16::MAX {
        let mut tx = LobbyTokenAuthTransaction::registered(token_request(), 7, 0);
        tx.start();
        tx.sent(Gate::Pass);
        let out = tx.consume_token_failed(backend);
        let expected = match backend {
            15 | 16 => 1939,
            17 => 1936,
            18..=20 => 2,
            _ => 1,
        };
        reason(&out, 25, expected);
        assert_eq!(
            out.changes,
            vec![
                StateChange::RemovePendingTokenUser(7),
                StateChange::TokenAuthFinished(7)
            ]
        );
        assert!(out.daemon_requests.is_empty());
    }
}
#[test]
fn token_send_failure_removes_pending_user_and_finishes() {
    let mut tx = LobbyTokenAuthTransaction::registered(token_request(), 7, 0);
    tx.start();
    let out = tx.sent(Gate::Fail);
    reason(&out, 25, 2);
    assert_eq!(
        out.changes,
        vec![
            StateChange::RemovePendingTokenUser(7),
            StateChange::TokenAuthFinished(7)
        ]
    );
    let late = tx.consume_token_failed(17);
    assert!(matches!(late.status, Status::Unsupported(_)));
    assert!(late.replies.is_empty());
    assert!(tx.start().daemon_requests.is_empty());
}
#[test]
fn token_out_of_order_callbacks_do_not_use_state_one_mapping() {
    let mut tx = LobbyTokenAuthTransaction::registered(token_request(), 7, 0);
    for out in [tx.consume_token_failed(17), tx.sent(Gate::Fail)] {
        assert!(matches!(out.status, Status::Unsupported(_)));
        assert!(out.replies.is_empty());
        assert!(out.changes.is_empty());
    }
    tx.start();
    assert!(tx.consume_token_failed(17).replies.is_empty());
}
#[test]
fn contents_tick_updates_on_allowed_null_cache_but_never_creates_storage() {
    let w = World {
        tick: 80000,
        ..World::default()
    };
    let mut s = Session {
        contents_last_tick: Some(40000),
        ..Session::default()
    };
    let out = run(&mut s, &w, &mut services(), 929, &[]);
    assert_eq!(s.contents_last_tick, Some(80000));
    assert_eq!(out.changes, vec![StateChange::ContentsTick(80000)]);
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert!(out.replies.is_empty());
    let mut absent = Session::default();
    let out = run(&mut absent, &w, &mut services(), 929, &[]);
    assert_eq!(absent.contents_last_tick, None);
    assert!(out.changes.is_empty());
}
#[test]
fn contents_tick_gate_uses_signed_native_comparisons() {
    for (last, now, reload, allowed) in [
        (u64::MAX, 80000, 0, true),
        (40000, 31000, 0, false),
        (40000, 69999, u64::MAX, false),
        (40000, 1u64 << 63, 0, true),
    ] {
        let mut s = Session {
            contents_last_tick: Some(last),
            ..Session::default()
        };
        let w = World {
            tick: now,
            config_reload_tick: reload,
            ..World::default()
        };
        let out = run(&mut s, &w, &mut services(), 929, &[]);
        if allowed {
            assert!(matches!(out.status, Status::Unsupported(_)));
            assert_eq!(s.contents_last_tick, Some(now));
            assert_eq!(out.changes, vec![StateChange::ContentsTick(now)]);
        } else {
            reason(&out, 931, 2167);
            assert_eq!(s.contents_last_tick, Some(last));
            assert!(out.changes.is_empty());
        }
    }
}
#[test]
fn auto_logout_is_a_redacted_report_not_logout() {
    for (state, detail) in [(0, 1), (1, 2), (2, 3), (3, 0), (255, 0)] {
        let mut s = ready();
        s.pc.as_mut().unwrap().life_state = state;
        let w = World {
            game_log_enabled: true,
            ..World::default()
        };
        let mut svc = services();
        let out = run(&mut s, &w, &mut svc, 411, &[]);
        assert_eq!(out.status, Status::Handled);
        assert_eq!(out.reports, vec![Report::AutoLogoutCheck { detail }]);
        assert_eq!(out.reports[0].record_id(), 1009);
        assert!(out.replies.is_empty());
        assert!(out.daemon_requests.is_empty());
        assert_eq!(svc.logout_calls, 0);
        assert!(s.authenticated);
    }
}
#[test]
fn auto_logout_respects_log_flag_and_queue_gate() {
    let mut s = ready();
    let out = run(&mut s, &World::default(), &mut services(), 411, &[]);
    assert_eq!(out.status, Status::Handled);
    assert!(out.reports.is_empty());
    for gate in [Gate::Fail, Gate::Unknown] {
        let mut svc = services();
        svc.queue_result = gate;
        let w = World {
            game_log_enabled: true,
            ..World::default()
        };
        let out = run(&mut s, &w, &mut svc, 411, &[]);
        assert!(out.reports.is_empty());
        assert!(out.changes.is_empty());
    }
}
#[test]
fn client_run_retains_raw_bytes_without_game_state_changes() {
    for bytes in [[0, 0], [255, 128], [1, 2]] {
        let mut s = ready();
        let out = run(
            &mut s,
            &World::default(),
            &mut services(),
            1210,
            &[bytes[0], bytes[1], 99],
        );
        assert_eq!(out.status, Status::Handled);
        assert_eq!(out.reports, vec![Report::ClientRun { bytes }]);
        assert_eq!(out.reports[0].record_id(), 1042);
        assert!(out.changes.is_empty());
        assert!(out.replies.is_empty());
        assert!(out.broadcasts.is_empty());
        assert_eq!(s.pc.as_ref().unwrap().position, Default::default());
    }
    for body in [vec![], vec![1]] {
        let out = run(
            &mut ready(),
            &World::default(),
            &mut services(),
            1210,
            &body,
        );
        assert!(out.reports.is_empty());
        assert_eq!(out.logs, vec!["ClientRun decode failure"]);
    }
}
#[test]
fn direct_entry_task_prefix_preserves_native_reason_order() {
    for case in 0..9 {
        let mut s = ready();
        let pc = s.pc.as_mut().unwrap();
        let mut geo = DirectEntryGeo::Category(1);
        let mut kind = 1;
        let expected = match case {
            0 => {
                geo = DirectEntryGeo::Missing;
                2
            }
            1 => {
                geo = DirectEntryGeo::MissingRecord;
                2388
            }
            2 => {
                geo = DirectEntryGeo::Category(2);
                2388
            }
            3 => {
                pc.life_state = 1;
                pc.combat = true;
                2390
            }
            4 => {
                pc.combat = true;
                pc.achievement_reward = true;
                2391
            }
            5 => {
                pc.achievement_reward = true;
                pc.instant_payment = true;
                2392
            }
            6 => {
                pc.jumping_ticket = true;
                2392
            }
            7 => {
                pc.instant_payment = true;
                2393
            }
            8 => {
                kind = 2;
                3
            }
            _ => unreachable!(),
        };
        let mut w = World::default();
        w.zones.insert(
            1,
            Zone {
                direct_entry_geo: geo,
                ..Zone::default()
            },
        );
        let mut body = vec![0; 12];
        body[0] = kind;
        let out = run(&mut s, &w, &mut services(), 981, &body);
        reason(&out, 983, expected);
        assert!(out.changes.contains(&StateChange::Queued(981)));
    }
}
#[test]
fn direct_entry_unknown_geo_and_valid_prefix_do_not_authorize_travel() {
    for geo in [DirectEntryGeo::Unknown, DirectEntryGeo::Category(1)] {
        let mut s = ready();
        let mut w = World::default();
        w.zones.insert(
            1,
            Zone {
                direct_entry_geo: geo,
                ..Zone::default()
            },
        );
        let mut body = vec![0; 12];
        body[0] = 1;
        let out = run(&mut s, &w, &mut services(), 981, &body);
        assert!(matches!(out.status, Status::Unsupported(_)));
        assert!(out.replies.is_empty());
        assert!(out.broadcasts.is_empty());
        assert!(out.daemon_requests.is_empty());
    }
}
#[test]
fn direct_leave_wrong_kind_does_not_queue_or_send() {
    let mut s = ready();
    s.pc.as_mut().unwrap().native_kind = 2;
    let mut svc = services();
    let out = run(&mut s, &World::default(), &mut svc, 984, &[]);
    assert_eq!(out.status, Status::Handled);
    assert!(out.replies.is_empty());
    assert!(svc.queued.is_empty());
    assert!(out.changes.is_empty());
    assert!(!out.logs.is_empty());
}
