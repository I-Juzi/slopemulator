use gamedaemon::handlers::*;
use gamedaemon::Position;

fn invoke(
    command: u16,
    body: &[u8],
    pc: Pc,
    world: World,
    services: &mut MemoryServices,
) -> Outcome {
    let mut session = Session {
        authenticated: true,
        pc: Some(pc),
        dispatch_permission: Some(true),
        ..Session::default()
    };
    dispatch(
        &mut Context {
            session: &mut session,
            world: &world,
            services,
        },
        command,
        body,
    )
}
fn ready() -> Pc {
    Pc {
        zone: Some(1),
        db_id: 7,
        world_id: 9,
        ..Pc::default()
    }
}
fn services() -> MemoryServices {
    MemoryServices {
        queue_result: Gate::Pass,
        ..MemoryServices::default()
    }
}
fn reason(out: &Outcome) -> (u16, u16) {
    let p = &out.replies[0];
    (
        p.command,
        u16::from_le_bytes(p.body[..2].try_into().unwrap()),
    )
}
#[test]
fn channel_list_is_exactly_one_counted_51_byte_block() {
    let mut world = World::default();
    world.zones.insert(
        1,
        Zone {
            channel_list: Some((17, [0xaa; 51])),
            ..Zone::default()
        },
    );
    let out = invoke(300, &[0xff], ready(), world, &mut services());
    assert_eq!(out.replies[0].command, 301);
    assert_eq!(out.replies[0].body.len(), 55);
    assert_eq!(&out.replies[0].body[..4], &[17, 0, 51, 0]);
    assert_eq!(&out.replies[0].body[4..], &[0xaa; 51]);
}
#[test]
fn clean_connection_is_an_inert_lobby_request() {
    let out = invoke(29, &[0xff], ready(), World::default(), &mut services());
    assert_eq!(
        out.daemon_requests,
        vec![DaemonRequest::LobbyClean { world: 9, db_id: 7 }]
    );
    assert!(out.replies.is_empty());
    let world = World {
        arena: true,
        ..World::default()
    };
    assert!(invoke(29, &[], ready(), world, &mut services())
        .daemon_requests
        .is_empty());
}
#[test]
fn unavailable_quota_operation_echoes_only_the_two_quads() {
    let body = [
        1u64.to_le_bytes().to_vec(),
        u64::MAX.to_le_bytes().to_vec(),
        123u32.to_le_bytes().to_vec(),
    ]
    .concat();
    let out = invoke(809, &body, ready(), World::default(), &mut services());
    assert_eq!(reason(&out), (811, 1967));
    assert_eq!(&out.replies[0].body[2..], &body[..16]);
    assert_eq!(
        reason(&invoke(
            812,
            &[0xff],
            ready(),
            World::default(),
            &mut services()
        )),
        (814, 1974)
    );
}
#[test]
fn turn_missing_zone_preserves_request_selector_and_yaw_failure_is_unknown() {
    let mut pc = ready();
    pc.zone = None;
    let out = invoke(66, &[42, 1, 0], pc, World::default(), &mut services());
    assert_eq!(reason(&out), (67, 2));
    assert_eq!(out.replies[0].body[2], 42);
    let out = invoke(
        66,
        &[0, 0x68, 1],
        ready(),
        World::default(),
        &mut services(),
    );
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert!(out.replies.is_empty());
}
#[test]
fn queued_request_keeps_raw_pose_without_authorizing_a_warp() {
    let body = [1, 0, 2, 0, 3, 0, 4, 0];
    let out = invoke(298, &body, ready(), World::default(), &mut services());
    assert!(out.changes.contains(&StateChange::QueuedRequest {
        command: 298,
        body: body.to_vec()
    }));
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert!(out.replies.is_empty());
    assert!(out.broadcasts.is_empty());
}
#[test]
fn unchecked_malformed_requests_never_receive_fabricated_reasons() {
    for command in [66, 102, 298, 303, 809, 1219, 1223, 1239] {
        let out = invoke(command, &[], ready(), World::default(), &mut services());
        assert!(matches!(out.status, Status::Unsupported(_)), "id {command}");
        assert!(out.replies.is_empty());
    }
}
#[test]
fn qq_interface_absence_returns_false_but_no_close_or_kick() {
    let out = invoke(887, &[], ready(), World::default(), &mut services());
    assert_eq!(out.status, Status::NativeFalse);
    assert!(out.kicks.is_empty());
    assert!(out.replies.is_empty());
}
#[test]
fn selector_one_requires_valid_summon_control_even_when_summon_exists() {
    let mut pc = ready();
    pc.summon = Some(Summon {
        control_state: 3,
        native_kind: 2,
        race: 0,
        has_entry: true,
        zone: Some(1),
        position: Position::default(),
        yaw: 0,
    });
    let out = invoke(66, &[1, 0, 0], pc, World::default(), &mut services());
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert!(out.replies.is_empty());
}
