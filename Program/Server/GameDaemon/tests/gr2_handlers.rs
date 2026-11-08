use gamedaemon::{handlers::*, Position};

struct Harness {
    session: Session,
    world: World,
    services: MemoryServices,
}
impl Default for Harness {
    fn default() -> Self {
        let pc = Pc {
            zone: Some(1),
            db_id: 20,
            object_id: 30,
            world_id: 1,
            user_available: true,
            party_id: 1,
            spawn_action_present: Some(true),
            spawn_state: 2,
            spawn_correlation: 0,
            ..Pc::default()
        };
        let mut world = World {
            team_enabled: true,
            account_inventory_enabled: true,
            publisher_code: 1,
            tick: 200000,
            utc_seconds: -123,
            qq_interface_present: true,
            ..World::default()
        };
        world.worlds.insert(1);
        world.zones.insert(
            1,
            Zone {
                channel: 1,
                ..Zone::default()
            },
        );
        world.jumping_ticket_ids.insert(7);
        world.additional_service_ids.insert(0x100000007);
        Self {
            session: Session {
                authenticated: true,
                pc: Some(pc),
                dispatch_permission: Some(true),
                ..Session::default()
            },
            world,
            services: MemoryServices {
                queue_result: Gate::Pass,
                logout_result: Gate::Pass,
                channel_result: Gate::Pass,
                ..MemoryServices::default()
            },
        }
    }
}
impl Harness {
    fn run(&mut self, command: u16, req: &[u8]) -> Outcome {
        self.session.authenticated = !matches!(command, 11 | 13 | 19 | 22 | 23);
        dispatch(
            &mut Context {
                session: &mut self.session,
                world: &self.world,
                services: &mut self.services,
            },
            command,
            req,
        )
    }
    fn pc(&mut self) -> &mut Pc {
        self.session.pc.as_mut().unwrap()
    }
}
fn request(command: u16) -> Vec<u8> {
    match command {
        11 => {
            let mut b = 1278u16.to_le_bytes().to_vec();
            b.extend(0x12D261D120403u64.to_le_bytes());
            b.extend(0u16.to_le_bytes());
            b.extend(0u64.to_le_bytes());
            b.extend([1, 0, 9]);
            b
        }
        13 => vec![0xfe, 4, 0, 0, 0, 0],
        18 | 414 => [1u64.to_le_bytes(), 2u64.to_le_bytes()].concat(),
        23 => [
            1u16.to_le_bytes().to_vec(),
            1u64.to_le_bytes().to_vec(),
            2u64.to_le_bytes().to_vec(),
        ]
        .concat(),
        31 => vec![7, 0, 0, 0, 99],
        42 => vec![0; 12],
        64 => move_request(1, 1),
        66 => vec![0, 0x67, 1],
        102 | 303 | 1239 => vec![2, 0],
        298 => vec![0; 8],
        305 => vec![1, 1, 0, 5],
        416 => vec![0; 21],
        809 => vec![0; 20],
        981 => vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        1149 => [vec![0; 16], 7u64.to_le_bytes().to_vec(), vec![0]].concat(),
        1210 => vec![1, 2],
        1219 => 1u64.to_le_bytes().to_vec(),
        1223 => vec![1, 0, 2, 0],
        _ => vec![],
    }
}
fn move_request(types: u16, cells: u16) -> Vec<u8> {
    let mut b = vec![0; 44]; // selector + six shorts + six ints + three shorts + broadcast
    b[37..39].copy_from_slice(&10u16.to_le_bytes());
    b.extend(types.to_le_bytes());
    for _ in 0..types {
        b.extend([1, 255, 0, 0, 0, 0, 0, 0, 0]);
    }
    b.extend(cells.to_le_bytes());
    for i in 0..cells {
        b.extend(i.to_le_bytes());
        b.extend([0; 4]);
    }
    b
}
fn reason(out: &Outcome) -> Option<(u16, u16)> {
    out.replies.first().and_then(|p| {
        p.body
            .get(..2)
            .map(|b| (p.command, u16::from_le_bytes(b.try_into().unwrap())))
    })
}
macro_rules! handler_test {
    ($name:ident, $id:expr) => {
        #[test]
        fn $name() {
            let mut h = Harness::default();
            let out = h.run($id, &request($id));
            assert!(out.kicks.is_empty());
            assert!(out.broadcasts.is_empty());
            assert!(
                !matches!(out.status, Status::NativeFalse),
                "valid prefix: {:?}",
                out
            );
            // One malformed/unused request per handler. No invented malformed reason.
            let mut malformed = Harness::default();
            let result = malformed.run($id, &[]);
            assert!(result.kicks.is_empty());
            assert!(result.broadcasts.is_empty());
            assert_eq!(malformed.pc().position, Position::default());
            if !matches!($id, 11 | 13 | 16 | 19 | 22 | 23 | 929) {
                let mut missing = Harness::default();
                missing.session.pc = None;
                assert!(!missing.run($id, &request($id)).logs.is_empty());
            }
        }
    };
}
handler_test!(hello, 11);
handler_test!(mobile_hello, 13);
handler_test!(ping, 16);
handler_test!(server_ping, 18);
handler_test!(game_key, 19);
handler_test!(np_token, 22);
handler_test!(lobby_token, 23);
handler_test!(return_lobby, 26);
handler_test!(clean_connection, 29);
handler_test!(enter_world, 31);
handler_test!(waiting_number, 34);
handler_test!(leave_world, 38);
handler_test!(enter_zone, 42);
handler_test!(move_command, 64);
handler_test!(turn, 66);
handler_test!(activated_list, 99);
handler_test!(detect_switch, 102);
handler_test!(base_camp, 145);
handler_test!(clear_party, 174);
handler_test!(clear_team, 262);
handler_test!(portal, 296);
handler_test!(turn_back_portal, 298);
handler_test!(channel_list, 300);
handler_test!(change_channel, 303);
handler_test!(save_option, 305);
handler_test!(client_config, 307);
handler_test!(auto_logout, 411);
handler_test!(round_trip, 414);
handler_test!(latency, 416);
handler_test!(utc, 422);
handler_test!(return_game, 648);
handler_test!(extend_quota, 809);
handler_test!(upgrade_quota, 812);
handler_test!(qq_logout, 887);
handler_test!(contents_rules, 929);
handler_test!(enter_direct, 981);
handler_test!(leave_direct, 984);
handler_test!(jumping_ticket, 1149);
handler_test!(client_run, 1210);
handler_test!(air_dash, 1219);
handler_test!(warp_noop, 1221);
handler_test!(teleport, 1223);
handler_test!(cave_escape, 1239);

#[test]
fn dispatch_default_auth_and_unmapped_rights() {
    let mut h = Harness::default();
    for command in [0, 1, 1599, u16::MAX] {
        let o = h.run(command, &[0xff]);
        assert_eq!(o.status, Status::Handled);
        assert_eq!(o.logs, vec!["unknown command"]);
        assert!(o.replies.is_empty());
    }
    assert_eq!(COMMANDS.len(), 43);
    let mut ctx = Context {
        session: &mut h.session,
        world: &h.world,
        services: &mut h.services,
    };
    ctx.session.authenticated = true;
    assert!(!dispatch(&mut ctx, 19, &[]).logs.is_empty());
    ctx.session.dispatch_permission = None;
    assert!(matches!(
        dispatch(&mut ctx, 38, &[]).status,
        Status::Unsupported(_)
    ));
    ctx.session.authenticated = false;
    assert_eq!(dispatch(&mut ctx, 16, &[]).status, Status::Handled);
}
#[test]
fn unused_bodies_never_create_named_effects() {
    for id in [16, 19, 22, 34, 38, 307, 1221] {
        let mut h = Harness::default();
        let out = h.run(id, &[0xff; 99]);
        assert_eq!(out.status, Status::Handled);
        assert!(out.replies.is_empty());
        assert!(out.changes.is_empty());
        assert!(out.daemon_requests.is_empty());
        assert_eq!(h.pc().zone, Some(1));
        assert_eq!(h.pc().position, Position::default());
    }
}
#[test]
fn mobile_properties_all_versions_duplicates_zero_unknown_and_decode() {
    let pairs = [
        (2, 301),
        (3, 38),
        (4, 29),
        (5, 18),
        (7, 9),
        (8, 9),
        (9, 4),
        (10, 3),
        (11, 20),
        (12, 2),
        (13, 2),
        (14, 4),
        (15, 20),
        (16, 4),
        (17, 4),
        (20, 3),
        (21, 11),
        (22, 11),
        (23, 11),
        (24, 11),
        (25, 11),
        (26, 7),
        (27, 9),
        (34, 1),
        (35, 1),
    ];
    for (id, v) in pairs {
        let mut req = vec![0xfe, 4, 0, 0, 2, 0];
        for _ in 0..2 {
            req.extend((id as u16).to_le_bytes());
            req.extend((v as u16).to_le_bytes());
        }
        assert_eq!(Harness::default().run(13, &req).replies[0].command, 14);
        req[8] ^= 1;
        assert_eq!(reason(&Harness::default().run(13, &req)), Some((15, 63)));
    }
    for req in [
        vec![],
        vec![0xfe, 4, 0, 0],
        vec![0xfe, 4, 0, 0, 0, 0x80],
        vec![0xfe, 4, 0, 0, 1, 0, 2],
    ] {
        assert_eq!(reason(&Harness::default().run(13, &req)), Some((15, 2)));
    }
    for req in [
        vec![0, 0, 0, 0, 0, 0],
        vec![0xfe, 4, 0, 0, 1, 0, 99, 0, 0, 0],
    ] {
        assert_eq!(reason(&Harness::default().run(13, &req)), Some((15, 63)));
    }
}
#[test]
fn hello_ordered_failures_and_publisher_switch() {
    let original = request(11);
    for offset in [0, 2, 10, 20] {
        let mut req = original.clone();
        req[offset] ^= 1;
        assert_eq!(Harness::default().run(11, &req).status, Status::NativeFalse);
    }
    for code in 0..=16 {
        let mut h = Harness::default();
        h.world.publisher_code = code;
        let mut req = original.clone();
        if matches!(code, 2..=8 | 10) {
            req[12..20].copy_from_slice(&0x40000000000u64.to_le_bytes());
        }
        let out = h.run(11, &req);
        if (1..=15).contains(&code) {
            assert!(matches!(out.status, Status::Unsupported(_)));
        } else {
            assert_eq!(out.status, Status::NativeFalse);
        }
    }
}
#[test]
fn ping_echo_rtt_and_signed_utc() {
    let mut h = Harness::default();
    let out = h.run(18, &request(18));
    assert_eq!(h.pc().received_ping, [1, 2]);
    assert_eq!(out.changes, vec![StateChange::PingReceived([1, 2])]);
    assert_eq!(h.run(18, &[]).status, Status::NativeFalse);
    assert_eq!(h.pc().received_ping, [1, 2]);
    assert_eq!(h.run(414, &request(414)).replies[0].body, request(414));
    assert_eq!(h.run(422, &[0xff]).replies[0].body, (-123i64).to_le_bytes());
}
#[test]
fn token_reasons_and_no_authentication_on_queue() {
    let mut h = Harness::default();
    h.world.arena = true;
    assert_eq!(reason(&h.run(23, &[])), Some((25, 5)));
    h.world.arena = false;
    let mut req = request(23);
    req[0] = 2;
    assert_eq!(reason(&h.run(23, &req)), Some((25, 1937)));
    req = request(23);
    req[2..10].fill(0);
    assert_eq!(reason(&h.run(23, &req)), Some((25, 1939)));
    h.services.queue_result = Gate::Fail;
    assert_eq!(reason(&h.run(23, &request(23))), Some((25, 2)));
    h.services.queue_result = Gate::Pass;
    assert!(matches!(
        h.run(23, &request(23)).status,
        Status::Unsupported(_)
    ));
    assert!(!h.session.authenticated);
}
#[test]
fn return_lobby_reason_order() {
    let mut h = Harness::default();
    h.world.arena = true;
    assert_eq!(reason(&h.run(26, &[])), Some((28, 5)));
    assert_eq!(h.services.logout_calls, 0);
    h.world.arena = false;
    h.services.logout_result = Gate::Fail;
    assert_eq!(reason(&h.run(26, &[])), Some((28, 1954)));
    h.services.logout_result = Gate::Pass;
    h.services.queue_result = Gate::Fail;
    assert_eq!(reason(&h.run(26, &[])), Some((28, 2)));
}
#[test]
fn enter_world_latches_before_unknown_composite_and_never_resets() {
    let mut h = Harness::default();
    assert_eq!(reason(&h.run(31, &[])), Some((33, 2)));
    assert!(!h.pc().received_enter_world);
    h.world.arena = true;
    assert_eq!(reason(&h.run(31, &request(31))), Some((33, 5)));
    h.world.arena = false;
    assert!(matches!(
        h.run(31, &request(31)).status,
        Status::Unsupported(_)
    ));
    assert_eq!(h.pc().client_version, 7);
    assert!(h.pc().received_enter_world);
    assert_eq!(reason(&h.run(31, &request(31))), Some((33, 2)));
}
#[test]
fn wrapper_missing_zone_and_queue_failure_reasons() {
    for (id, reply, code) in [
        (42, 44, 2),
        (99, 101, 2),
        (102, 104, 2),
        (145, 146, 2),
        (174, 175, 2),
        (262, 263, 2),
        (296, 297, 2),
        (298, 299, 2),
        (300, 302, 2),
        (303, 304, 2),
        (411, 413, 2),
        (981, 983, 2),
        (1219, 1220, 2),
        (1223, 1225, 2),
        (1239, 1241, 1),
    ] {
        let mut h = Harness::default();
        h.pc().zone = None;
        assert_eq!(
            reason(&h.run(id, &request(id))),
            Some((reply, code)),
            "id {id}"
        );
        let mut h = Harness::default();
        h.services.queue_result = Gate::Fail;
        assert_eq!(
            reason(&h.run(id, &request(id))),
            Some((reply, code)),
            "id {id}"
        );
    }
}
#[test]
fn shared_party_task_keeps_175_for_team_request() {
    let mut h = Harness::default();
    h.world.team_enabled = false;
    assert_eq!(reason(&h.run(262, &[])), Some((263, 2)));
    h.world.team_enabled = true;
    h.pc().party_id = 0;
    assert_eq!(reason(&h.run(262, &[])), Some((175, 2)));
}
#[test]
fn enter_zone_correlation_and_missing_user() {
    let mut h = Harness::default();
    h.pc().user_available = false;
    assert!(h.run(42, &request(42)).replies.is_empty());
    h.pc().user_available = true;
    h.pc().spawn_correlation = 1;
    assert_eq!(reason(&h.run(42, &request(42))), Some((44, 1150)));
}
#[test]
fn contents_throttle_boundaries_do_not_synthesize_cache() {
    for (last, now, reload, allowed) in [
        (0, 100000, 0, true),
        (40000, 69999, 0, false),
        (40000, 70000, 0, true),
        (40000, 50000, 40000, true),
        (1, 30000, 0, true),
    ] {
        let mut h = Harness::default();
        h.session.contents_last_tick = Some(last);
        h.world.tick = now;
        h.world.config_reload_tick = reload;
        let out = h.run(929, &[]);
        if allowed {
            assert!(matches!(out.status, Status::Unsupported(_)));
            assert!(out.replies.is_empty());
        } else {
            assert_eq!(reason(&out), Some((931, 2167)));
        }
        assert_eq!(
            h.session.contents_last_tick,
            Some(if allowed { now } else { last })
        );
        assert_eq!(
            out.changes,
            if allowed {
                vec![StateChange::ContentsTick(now)]
            } else {
                vec![]
            }
        );
    }
}
#[test]
fn options_length_type_zero_id_and_pending_save() {
    for (kind, len) in [(0, 1), (3, 1), (1, 3301), (2, 4001)] {
        let mut req = vec![kind];
        req.extend((len as u16).to_le_bytes());
        req.resize(len + 3, 0);
        assert_eq!(reason(&Harness::default().run(305, &req)), Some((306, 2)));
    }
    let mut h = Harness::default();
    h.pc().db_id = 0;
    assert!(h.run(305, &request(305)).daemon_requests.is_empty());
    h.pc().db_id = 1;
    let out = h.run(305, &request(305));
    assert_eq!(out.daemon_requests.len(), 1);
    assert!(matches!(out.status, Status::Unsupported(_)));
}
#[test]
fn direct_entry_count_type_arena_and_truncated_pairs() {
    let mut h = Harness::default();
    let mut req = request(981);
    h.world.arena = true;
    assert_eq!(reason(&h.run(981, &[])), Some((983, 5)));
    h.world.arena = false;
    req[10] = 17;
    assert_eq!(reason(&h.run(981, &req)), Some((983, 2399)));
    req[10] = 1;
    assert_eq!(reason(&h.run(981, &req)), Some((983, 2)));
    req = request(981);
    req[0] = 0;
    assert_eq!(reason(&h.run(981, &req)), Some((983, 2)));
}
#[test]
fn jumping_ticket_order_and_raw_eight_byte_item_data() {
    let mut h = Harness::default();
    let mut req = request(1149);
    req[20..24].fill(0xff);
    assert!(matches!(h.run(1149, &req).status, Status::Unsupported(_)));
    assert_eq!(reason(&h.run(1149, &[])), Some((1151, 2483)));
    h.world.account_inventory_enabled = false;
    assert_eq!(reason(&h.run(1149, &req)), Some((1151, 2)));
    h.world.account_inventory_enabled = true;
    h.world.arena = true;
    assert_eq!(reason(&h.run(1149, &req)), Some((1151, 5)));
    h.world.arena = false;
    h.world.additional_service_ids.clear();
    assert_eq!(reason(&h.run(1149, &req)), Some((1151, 2499)));
}
#[test]
fn arena_return_mutates_before_backend_and_never_claims_success() {
    let mut h = Harness::default();
    assert_eq!(reason(&h.run(648, &[])), Some((650, 2)));
    h.world.arena = true;
    assert_eq!(reason(&h.run(648, &[])), Some((650, 5)));
    h.world.arena = false;
    h.pc().arena_state = 4;
    h.pc().arena_qwords = [123, 456];
    let out = h.run(648, &[]);
    assert_eq!(h.pc().arena_state, 1);
    assert_eq!(h.pc().arena_qwords, [0, 0]);
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert_eq!(out.daemon_requests.len(), 1);
    assert!(out.replies.is_empty());
}
#[test]
fn movement_count_and_initializer_boundaries() {
    for (types, cells, code) in [(13, 1, 103), (1, 37, 2), (1, 36, 102), (0, 1, 2), (1, 0, 2)] {
        assert_eq!(
            reason(&Harness::default().run(64, &move_request(types, cells))),
            Some((65, code))
        );
    }
    let mut req = move_request(1, 1);
    req[37..39].copy_from_slice(&551u16.to_le_bytes());
    assert_eq!(reason(&Harness::default().run(64, &req)), Some((65, 2)));
    let mut req = move_request(1, 1);
    req[0] = 1;
    assert_eq!(reason(&Harness::default().run(64, &req)), Some((65, 2)));
    assert!(matches!(
        Harness::default().run(64, &move_request(1, 1)).status,
        Status::Unsupported(_)
    ));
}
#[test]
fn channel_task_order_and_signed_channels() {
    for (case, expected) in [
        (0, 1194),
        (1, 2),
        (2, 1195),
        (3, 5),
        (4, 1190),
        (5, 1192),
        (6, 1193),
        (7, 1191),
        (8, 1191),
        (9, 777),
    ] {
        let mut h = Harness::default();
        let mut req = 2i16.to_le_bytes();
        match case {
            0 => h.pc().achievement_reward = true,
            1 => h.pc().combat = true,
            2 => h.pc().instant_payment = true,
            3 => h.pc().arena_state = 2,
            4 => req = 1i16.to_le_bytes(),
            5 => h.services.channel_result = Gate::Fail,
            6 => h.pc().last_channel_change = 199999,
            7 => h.world.zones.get_mut(&1).unwrap().zone_type = 40,
            8 => req = 0i16.to_le_bytes(),
            9 => h.services.attraction_result = Some(Err(Some(777))),
            _ => unreachable!(),
        }
        assert_eq!(
            reason(&h.run(303, &req)),
            Some((304, expected)),
            "case {case}"
        );
    }
    let mut h = Harness::default();
    h.services.channel_result = Gate::Fail;
    let out = h.run(303, &(-1i16).to_le_bytes());
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert_eq!(h.pc().last_channel_change, 200000);
}
#[test]
fn teleport_and_cave_flag_reasons_are_distinct() {
    for (id, reply, a, b) in [(1223, 1225, 1144, 1145), (1239, 1241, 254, 255)] {
        let mut h = Harness::default();
        h.pc().achievement_reward = true;
        h.pc().instant_payment = true;
        assert_eq!(reason(&h.run(id, &request(id))), Some((reply, a)));
        h.pc().achievement_reward = false;
        h.pc().jumping_ticket = true;
        assert_eq!(reason(&h.run(id, &request(id))), Some((reply, a)));
        h.pc().jumping_ticket = false;
        assert_eq!(reason(&h.run(id, &request(id))), Some((reply, b)));
    }
}
#[test]
fn portal_environment_and_life_state() {
    let mut h = Harness::default();
    assert_eq!(reason(&h.run(296, &[])), Some((297, 1152)));
    {
        let z = h.world.zones.get_mut(&1).unwrap();
        z.environment_index = 1;
        z.portal_subtype = Some(7);
    }
    for (state, code) in [(1, 1155), (3, 1155), (9, 2)] {
        h.world.zones.get_mut(&1).unwrap().life_state = state;
        assert_eq!(reason(&h.run(296, &[])), Some((297, code)));
    }
    h.world.zones.get_mut(&1).unwrap().life_state = 2;
    assert!(matches!(h.run(296, &[]).status, Status::Unsupported(_)));
}
#[test]
fn detect_switch_table_and_signed_radius_boundaries() {
    let mut h = Harness::default();
    assert_eq!(reason(&h.run(102, &request(102))), Some((104, 1128)));
    h.world.switches.insert(
        (1, 2),
        TeleportSwitch {
            position: Position { x: 3, y: 4, z: 0 },
            radius: -5,
            teleport_key: 8,
        },
    );
    assert_eq!(reason(&h.run(102, &request(102))), Some((104, 1128))); // Equality passes distance, missing Teleport fails.
    h.world.teleports.insert(
        8,
        TeleportRecord {
            id: 1,
            disabled: false,
        },
    );
    assert!(matches!(
        h.run(102, &request(102)).status,
        Status::Unsupported(_)
    ));
    h.pc().position.z = 1;
    assert_eq!(reason(&h.run(102, &request(102))), Some((104, 1129)));
}
#[test]
fn air_dash_prepare_reason_order() {
    for (case, code) in [(0, 169), (1, 171), (2, 170), (3, 174), (4, 172), (5, 173)] {
        let mut h = Harness::default();
        let mut e = AirDashEnvironment {
            subtype: 11,
            required_environment: 0,
            enabled: true,
            helper_65002: Gate::Pass,
            faction_requirement: 0,
            pc_faction: 1,
            requirement_92: 0,
            pc_value_552: 0,
            zone_helper: None,
        };
        match case {
            0 => e.required_environment = 1,
            1 => e.enabled = false,
            2 => e.helper_65002 = Gate::Fail,
            3 => e.faction_requirement = 2,
            4 => e.requirement_92 = 1,
            5 => e.zone_helper = Some(Gate::Fail),
            _ => unreachable!(),
        }
        h.world.air_dash.insert((1, 1), e);
        assert_eq!(reason(&h.run(1219, &request(1219))), Some((1220, code)));
    }
}
