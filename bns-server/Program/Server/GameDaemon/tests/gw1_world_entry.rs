use gamedaemon::handlers::*;
use packet::{fragments::FragmentReader, world_entry::*};

fn snapshot() -> EnterWorldOk {
    let values = std::array::from_fn(|i| match property_code(i + 1).unwrap() {
        'c' => PropertyValue::Byte(0),
        'h' => PropertyValue::Short(0),
        'i' => PropertyValue::Int(0),
        'q' | '?' => PropertyValue::Long(0),
        's' => PropertyValue::String(vec![]),
        'b' => PropertyValue::Bytes(vec![]),
        'B' | 'F' => PropertyValue::Bit(false),
        code => panic!("unknown kind {code}"),
    });
    EnterWorldOk {
        header: EntryHeader {
            world_id: 1,
            object_id: 42,
            jumping_character_state: 0,
            chat_address_bits: 0,
            chat_port: 0,
            chat_world_id: 1,
            manage_port: 0,
            time_zone_bias: 0,
        },
        properties: CreatureProperties::new(values).unwrap(),
        extra_properties: vec![7; 3990],
        opaque_broadcast_block: vec![],
        recycle_remaining: [None; 25],
        acquired_skills: [vec![], vec![]],
        config_bytes: [0; 4],
        pc_cafe_publisher: 0,
        pc_db_id: 43,
        content_bans_enabled: false,
        user: None,
        test_server_enabled: false,
        craft_unions: [CraftUnion::default(); 2],
        slate_pages: std::array::from_fn(|i| SlatePage {
            page: (i + 1) as u8,
            items: [SlateItem {
                item_id: 0,
                level: 0,
            }; 6],
            stones: [0; 9],
            new_stone_id: 0,
            new_stone_position: 0,
        }),
    }
}
fn ready() -> Session {
    Session {
        authenticated: true,
        dispatch_permission: Some(true),
        pc: Some(Pc {
            object_id: 42,
            db_id: 43,
            world_id: 1,
            ..Pc::default()
        }),
        ..Session::default()
    }
}
fn run(
    session: &mut Session,
    world: &World,
    services: &mut MemoryServices,
    body: &[u8],
) -> Outcome {
    dispatch(
        &mut Context {
            session,
            world,
            services,
        },
        31,
        body,
    )
}
fn request() -> Vec<u8> {
    vec![0x12, 0x34, 0x56, 0x78, 1]
}
#[test]
fn complete_32_is_ordered_but_does_not_complete_entry() {
    let mut s = ready();
    let mut svc = MemoryServices {
        initial_world: Some(Box::new(snapshot())),
        ..MemoryServices::default()
    };
    let expected = snapshot().encode_fragments().unwrap();
    let out = run(&mut s, &World::default(), &mut svc, &request());
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert_eq!(
        out.changes,
        vec![StateChange::EnterWorldReceived {
            client_version: 0x78563412
        }]
    );
    assert_eq!(
        out.session_effects,
        expected
            .iter()
            .cloned()
            .map(SessionEffect::SendFragmentWithCurrentProcessor)
            .collect::<Vec<_>>()
    );
    assert!(out.replies.is_empty());
    assert!(out.broadcasts.is_empty());
    assert!(out.daemon_requests.is_empty());
    assert!(out.kicks.is_empty());
    assert!(svc.queued.is_empty());
    assert!(svc.initial_world.is_none());
    let pc = s.pc.as_ref().unwrap();
    assert!(pc.received_enter_world);
    assert_eq!(pc.client_version, 0x78563412);
    assert!(pc.zone.is_none());
    let mut reader = FragmentReader::default();
    let mut completed = false;
    for bytes in &expected {
        if let Some(joined) = reader.add(bytes).unwrap() {
            assert_eq!(joined.cmd, 32);
            completed = true;
        }
    }
    assert!(completed);
    // The native latch remains set. No snapshot request or duplicate 32 occurs.
    svc.initial_world = Some(Box::new(snapshot()));
    let duplicate = run(&mut s, &World::default(), &mut svc, &request());
    assert_eq!(
        duplicate.replies,
        vec![Message {
            command: 33,
            body: vec![2, 0]
        }]
    );
    assert!(duplicate.session_effects.is_empty());
    assert!(svc.initial_world.is_some());
}
#[test]
fn unknown_snapshot_keeps_latch_without_send_or_travel() {
    let mut s = ready();
    let out = run(
        &mut s,
        &World::default(),
        &mut MemoryServices::default(),
        &request(),
    );
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert!(s.pc.unwrap().received_enter_world);
    assert_eq!(out.changes.len(), 1);
    assert!(out.session_effects.is_empty());
    assert!(out.replies.is_empty());
    assert!(out.daemon_requests.is_empty());
}
#[test]
fn adapter_identity_and_size_errors_never_send_native_failure() {
    for kind in 0..3 {
        let mut snapshot = snapshot();
        match kind {
            0 => snapshot.header.object_id += 1,
            1 => snapshot.pc_db_id += 1,
            _ => snapshot.extra_properties.push(0),
        }
        let mut svc = MemoryServices {
            initial_world: Some(Box::new(snapshot)),
            ..MemoryServices::default()
        };
        let mut s = ready();
        let out = run(&mut s, &World::default(), &mut svc, &request());
        assert!(matches!(out.status, Status::Unsupported(_)));
        assert!(out.replies.is_empty());
        assert!(out.session_effects.is_empty());
        assert!(s.pc.unwrap().received_enter_world);
        assert!(svc.queued.is_empty());
    }
}
#[test]
fn wrapper_failures_do_not_fetch_or_send_initial_state() {
    for kind in 0..4 {
        let mut s = ready();
        let mut world = World::default();
        let mut body = request();
        match kind {
            0 => world.arena = true,
            1 => s.pc.as_mut().unwrap().arena_state = 3,
            2 => body.truncate(4),
            _ => s.pc.as_mut().unwrap().received_enter_world = true,
        }
        let mut svc = MemoryServices {
            initial_world: Some(Box::new(snapshot())),
            ..MemoryServices::default()
        };
        let out = run(&mut s, &world, &mut svc, &body);
        assert_eq!(out.status, Status::Handled);
        assert_eq!(
            out.replies,
            vec![Message {
                command: 33,
                body: vec![if kind < 2 { 5 } else { 2 }, 0]
            }]
        );
        assert!(out.changes.is_empty());
        assert!(out.session_effects.is_empty());
        assert!(svc.initial_world.is_some());
    }
}
