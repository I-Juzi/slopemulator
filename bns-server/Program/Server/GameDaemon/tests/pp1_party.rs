use gamedaemon::handlers::*;
use std::collections::BTreeMap;

const SELF: u64 = 0x10001;
const TARGET: u64 = 0x10002;
const OTHER: u64 = 0x10003;
const TEAM: u64 = 0x400001;
fn name(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}
fn wire_name(text: &str) -> Vec<u8> {
    let units = name(text);
    let mut out = (units.len() as u16).to_le_bytes().to_vec();
    for unit in units {
        out.extend(unit.to_le_bytes());
    }
    out
}
fn q(id: u64) -> Vec<u8> {
    id.to_le_bytes().to_vec()
}
fn target_body(mode: Option<u8>, id: u64, text: &str, options: &[u8]) -> Vec<u8> {
    let mut out = mode.into_iter().collect::<Vec<_>>();
    out.extend(q(id));
    out.extend(wire_name(text));
    out.extend(options);
    out
}
fn request(command: u16) -> Vec<u8> {
    match command {
        147 => target_body(None, TARGET, "wire", &[7, 2, 3, 4, 5, 6]),
        151 => target_body(None, TARGET, "wire", &[7, 2]),
        157 => {
            let mut b = target_body(None, TARGET, "wire", &[]);
            b.extend(5u32.to_le_bytes());
            b.extend([7, 2, 3, 4, 5, 6]);
            b
        }
        161 => [q(999), q(TARGET), 5u32.to_le_bytes().to_vec(), vec![2]].concat(),
        163 => [q(999), q(TARGET), vec![4, 8]].concat(),
        170 | 250 | 253 => q(TARGET),
        172 | 179 | 182 | 256 => vec![1],
        176 => [vec![1, 2], q(TARGET), q(u64::MAX)].concat(),
        202 | 208 => vec![9],
        205 => [q(100), vec![9]].concat(),
        216 => vec![2, 1],
        226 | 234 => target_body(Some(1), TARGET, "wire", &[7, 8, 9, 10, 11]),
        230 => target_body(Some(1), TARGET, "wire", &[7, 8]),
        239 => [vec![7], q(TARGET), q(999)].concat(),
        241 => [q(TARGET), vec![8]].concat(),
        243 => vec![7],
        245 => vec![2, 0, 4],
        247 => [vec![0, 0, 1], q(TARGET), vec![255, 255, 255], q(0)].concat(),
        259 => vec![255, 2],
        424 => 123u32.to_le_bytes().to_vec(),
        429 => [q(TARGET), 55u32.to_le_bytes().to_vec()].concat(),
        _ => vec![],
    }
}
struct Harness {
    session: Session,
    world: World,
    services: MemoryServices,
}
impl Default for Harness {
    fn default() -> Self {
        let pc = Pc {
            object_id: SELF,
            db_id: 10,
            name: name("source"),
            zone: Some(1),
            party_id: 1,
            user_available: true,
            team_member: true,
            team_id: TEAM,
            world_id: 8,
            ..Pc::default()
        };
        let peer = Pc {
            object_id: TARGET,
            db_id: 20,
            name: name("target"),
            ..pc.clone()
        };
        let third = Pc {
            object_id: OTHER,
            db_id: 30,
            name: name("third"),
            ..pc.clone()
        };
        let party = Party {
            difficulty: 1,
            members: vec![
                PartyMember {
                    slot: 5,
                    db_id: 10,
                    runtime_id: SELF,
                    state: 3,
                    ..PartyMember::default()
                },
                PartyMember {
                    slot: 9,
                    db_id: 20,
                    runtime_id: TARGET,
                    state: 3,
                    ..PartyMember::default()
                },
            ],
            ..Party::default()
        };
        let team = Team {
            parties: vec![Some(party)],
            indexed_parties: vec![0],
            membership: BTreeMap::from([(10, 0), (20, 0)]),
            leader_db_id: 10,
            leader_runtime_id: SELF,
            mode: 1,
            ..Team::default()
        };
        let state = PartyState {
            world_mask: 0xffff,
            world_slots: vec![
                None,
                Some(PartyPeer {
                    pc: pc.clone(),
                    rights: 6,
                }),
                Some(PartyPeer {
                    pc: peer,
                    rights: 2,
                }),
                Some(PartyPeer {
                    pc: third,
                    rights: 4,
                }),
            ],
            names: BTreeMap::from([(name("source"), 1), (name("target"), 2), (name("wire"), 2)]),
            pc_teams: vec![0, TEAM, TEAM, 0],
            team_mask: 0xffff,
            teams: vec![None, Some(team)],
            ..PartyState::default()
        };
        let world = World {
            team_enabled: true,
            party_config: PartyConfig {
                default_difficulty: 1,
                vote_enabled: true,
                maximum_grade: 7,
                maximum_bid: 100,
                difficulty_maximum: BTreeMap::from([(1, 6), (2, 6), (3, 6)]),
                ..PartyConfig::default()
            },
            zones: BTreeMap::from([(1, Zone::default())]),
            ..World::default()
        };
        Self {
            session: Session {
                authenticated: true,
                dispatch_permission: Some(true),
                pc: Some(pc),
                ..Session::default()
            },
            world,
            services: MemoryServices {
                party: Some(state),
                queue_result: Gate::Pass,
                ..MemoryServices::default()
            },
        }
    }
}
impl Harness {
    fn pc(&mut self) -> &mut Pc {
        self.session.pc.as_mut().unwrap()
    }
    fn state(&mut self) -> &mut PartyState {
        self.services.party.as_mut().unwrap()
    }
    fn team(&mut self) -> &mut Team {
        self.state().teams[1].as_mut().unwrap()
    }
    fn party(&mut self) -> &mut Party {
        self.team().parties[0].as_mut().unwrap()
    }
    fn peer(&mut self) -> &mut PartyPeer {
        self.state().world_slots[2].as_mut().unwrap()
    }
    fn context(&mut self) -> Context<'_> {
        Context {
            session: &mut self.session,
            world: &self.world,
            services: &mut self.services,
        }
    }
    fn run(&mut self, command: u16, body: &[u8]) -> Outcome {
        dispatch(&mut self.context(), command, body)
    }
    fn task(&mut self, command: u16, body: &[u8]) -> PartyTask {
        let out = self.run(command, body);
        assert!(out.replies.is_empty(), "{command}: {out:?}");
        assert!(out.broadcasts.is_empty());
        assert!(out.kicks.is_empty());
        assert!(out.changes.contains(&StateChange::QueuedRequest {
            command,
            body: body.to_vec()
        }));
        out.changes
            .into_iter()
            .find_map(|c| match c {
                StateChange::PartyTaskQueued(t) => Some(t),
                _ => None,
            })
            .unwrap()
    }
    fn execute(&mut self, task: &PartyTask) -> Outcome {
        execute_party_task(&mut self.context(), task)
    }
    fn full(&mut self, command: u16, body: &[u8]) -> Outcome {
        let task = self.task(command, body);
        self.execute(&task)
    }
}
fn expected(command: u16, reason: u16) -> Message {
    let (response, byte) = match command {
        168 => (169, false),
        170 => (171, false),
        172 => (173, false),
        174 => (175, false),
        176 => (178, false),
        179 => (181, true),
        182 => (184, false),
        202 => (204, false),
        205 => (207, false),
        208 => (210, false),
        211 => (213, false),
        214 => (215, false),
        216 => (219, false),
        239 => (240, false),
        241 => (242, false),
        243 => (244, false),
        245 => (246, false),
        247 => (249, false),
        250 => (252, false),
        253 => (255, false),
        256 => (258, false),
        259 => (261, false),
        262 => (263, false),
        161 => (162, false),
        163 => (164, false),
        _ => panic!("no expected reply for {command}"),
    };
    let mut body = if byte {
        vec![reason as u8]
    } else {
        reason.to_le_bytes().to_vec()
    };
    match command {
        239 | 243 => body.push(7),
        161 => {
            body.extend(5u32.to_le_bytes());
            body.push(2);
        }
        163 => body.push(4),
        _ => {}
    }
    Message {
        command: response,
        body,
    }
}
fn only(out: &Outcome, message: Message) {
    assert_eq!(out.replies, vec![message]);
    assert!(out.broadcasts.is_empty());
    assert!(out.kicks.is_empty());
}
fn destinations(out: &Outcome, command: u16) -> Vec<u64> {
    out.party_effects
        .iter()
        .filter_map(|e| match e {
            PartyEffect::Send { to, message } if message.command == command => Some(*to),
            _ => None,
        })
        .collect()
}

#[test]
fn all_party_commands_have_dispatch_and_parity_tags() {
    let source = [
        include_str!("../src/handlers/party.rs"),
        include_str!("../src/handlers/session.rs"),
    ]
    .concat();
    assert_eq!(PARTY_COMMANDS.len(), 36);
    for command in PARTY_COMMANDS {
        assert!(
            source.contains(&format!("/// parity: {command} 14-party-handlers.md#")),
            "{command}"
        );
        let mut h = Harness::default();
        let out = h.run(*command, &request(*command));
        assert!(!out.logs.contains(&"unknown command"), "{command}");
        assert!(out.kicks.is_empty());
    }
}
#[test]
fn party_dispatch_requires_authenticated_pc_and_permission() {
    for command in PARTY_COMMANDS {
        let mut h = Harness::default();
        h.session.authenticated = false;
        let out = h.run(*command, &request(*command));
        assert!(out.replies.is_empty());
        assert!(out.changes.is_empty());
        h.session.authenticated = true;
        h.session.dispatch_permission = None;
        assert!(matches!(
            h.run(*command, &request(*command)).status,
            Status::Unsupported(_)
        ));
        h.session.dispatch_permission = Some(true);
        h.session.pc = None;
        assert!(h.run(*command, &request(*command)).replies.is_empty());
    }
}
#[test]
fn create_always_replies_156_reason_two_without_mutation() {
    for body in [vec![], vec![1, 2, 3]] {
        let mut h = Harness::default();
        let out = h.run(155, &body);
        only(
            &out,
            Message {
                command: 156,
                body: vec![2, 0],
            },
        );
        assert_eq!(out.status, Status::Handled);
        assert!(out.changes.is_empty());
        assert!(out.daemon_requests.is_empty());
    }
}
#[test]
fn wrapper_missing_zone_replies_keep_width_and_echo() {
    for (command, reason) in [
        (168, 764),
        (170, 2),
        (172, 2),
        (174, 2),
        (176, 2),
        (179, 2),
        (182, 2),
        (202, 2),
        (205, 2),
        (208, 2),
        (211, 2),
        (214, 2),
        (216, 2),
        (239, 852),
        (241, 2),
        (243, 764),
        (245, 2),
        (247, 2),
        (250, 764),
        (253, 2),
        (256, 764),
        (259, 764),
        (262, 2),
        (161, 764),
        (163, 764),
    ] {
        let mut h = Harness::default();
        h.pc().zone = None;
        h.world.arena = command == 216;
        let out = h.run(command, &request(command));
        only(&out, expected(command, reason));
        assert!(h.services.queued.is_empty(), "{command}");
    }
}
#[test]
fn allocation_initialization_and_queue_failures_reply_two() {
    for command in [
        168, 170, 172, 174, 176, 179, 182, 202, 205, 208, 211, 214, 216, 239, 241, 243, 245, 247,
        250, 253, 256, 259, 262, 161, 163,
    ] {
        for kind_failure in [false, true] {
            let mut h = Harness::default();
            h.world.arena = command == 216;
            if kind_failure {
                h.pc().native_kind = 2;
            } else {
                h.services.queue_result = Gate::Fail;
            }
            let out = h.run(command, &request(command));
            only(&out, expected(command, 2));
            assert!(out.changes.is_empty(), "{command}");
        }
    }
}
#[test]
fn typed_tasks_preserve_fields_and_do_not_execute_on_queue() {
    for command in [
        168, 170, 172, 174, 176, 179, 182, 202, 205, 208, 211, 214, 216, 239, 241, 243, 245, 247,
        250, 253, 256, 259, 262, 161, 163,
    ] {
        let mut h = Harness::default();
        h.world.arena = command == 216;
        let mut body = request(command);
        body.extend([0xaa, 0xbb]);
        let task = h.task(command, &body);
        assert_eq!((task.command, task.caster, task.zone), (command, SELF, 1));
        assert_eq!(h.team().invite_authority, 0);
        assert_eq!(h.party().difficulty, 1);
        assert!(h.services.party_calls.is_empty());
        match task.kind {
            PartyTaskKind::Pickup {
                rule,
                grade,
                target,
                money,
            } => assert_eq!((rule, grade, target, money), (1, 2, TARGET, u64::MAX)),
            PartyTaskKind::MoveMember {
                source,
                destination,
            } => {
                assert_eq!(source, (0, 1, TARGET));
                assert_eq!(destination, (0xffff, 0xff, 0));
            }
            PartyTaskKind::JoinParty {
                invitation,
                correlation,
                difficulty,
            } => assert_eq!((invitation, correlation, difficulty), (TARGET, 5, 2)),
            PartyTaskKind::TeamDifficulty {
                selector,
                difficulty,
            } => assert_eq!((selector, difficulty), (255, 2)),
            PartyTaskKind::Reject {
                group,
                invitation,
                correlation,
                kind,
            } => assert_eq!(
                (group, invitation, correlation, kind),
                if command == 163 {
                    (1, TARGET, 4, 8)
                } else {
                    (2, TARGET, 0, 8)
                }
            ),
            _ => {}
        }
    }
}
#[test]
fn checked_decode_and_unchecked_decode_do_not_invent_native_values() {
    for command in [163, 239, 241, 243, 245, 247, 250, 253, 256, 259] {
        let mut h = Harness::default();
        let out = h.run(command, &[]);
        assert_eq!(&out.replies[0].body[..2], &[2, 0], "{command}");
        assert!(out.changes.is_empty());
    }
    for command in [
        147, 151, 157, 161, 170, 172, 176, 179, 182, 202, 205, 208, 226, 230, 424, 429,
    ] {
        let mut h = Harness::default();
        let out = h.run(command, &[]);
        assert!(
            matches!(out.status, Status::Unsupported(_)),
            "{command}: {out:?}"
        );
        assert!(out.replies.is_empty());
        assert!(out.changes.is_empty());
    }
    let mut h = Harness::default();
    let out = h.run(239, &[99]);
    only(
        &out,
        Message {
            command: 240,
            body: vec![2, 0, 99],
        },
    );
}
#[test]
fn config_gates_and_literal_command_quirks() {
    for (command, response) in [
        (245, 246),
        (247, 249),
        (250, 252),
        (253, 252),
        (256, 258),
        (259, 261),
        (262, 263),
    ] {
        let mut h = Harness::default();
        h.world.team_enabled = false;
        only(
            &h.run(command, &request(command)),
            Message {
                command: response,
                body: vec![2, 0],
            },
        );
    }
    let mut h = Harness::default();
    h.world.team_enabled = false;
    assert!(h.task(241, &request(241)).command == 241); // No team-config gate.
    h.world.arena = true;
    only(&h.run(172, &[1]), expected(172, 782));
    for state in [2, 3, 5, 6] {
        h.pc().arena_state = state;
        only(&h.run(179, &[]), expected(179, 5));
    }
    h.world.arena = false;
    only(
        &h.run(216, &[]),
        Message {
            command: 173,
            body: 813u16.to_le_bytes().to_vec(),
        },
    );
    h.world.party_config.vote_enabled = false;
    for command in [216, 220] {
        let out = h.run(command, &[]);
        assert!(out.replies.is_empty());
        assert!(out.changes.is_empty());
    }
}
#[test]
fn wrapper_range_checks_precede_zone_checks() {
    for (command, body, reason) in [
        (176, [vec![4, 1], q(0), q(0)].concat(), 2),
        (179, vec![0], 2),
        (179, vec![4], 2),
        (182, vec![2], 798),
        (256, vec![3], 894),
        (259, vec![0, 0], 2),
        (
            247,
            [vec![255, 255, 1], q(TARGET), vec![0, 0, 0], q(0)].concat(),
            878,
        ),
        (
            247,
            [vec![0, 0, 255], q(TARGET), vec![0, 0, 0], q(0)].concat(),
            878,
        ),
        (
            247,
            [vec![0, 0, 1], q(0), vec![0, 0, 0], q(0)].concat(),
            878,
        ),
    ] {
        let mut h = Harness::default();
        h.pc().zone = None;
        only(&h.run(command, &body), expected(command, reason));
    }
}
#[test]
fn team_authority_has_no_immediate_ack_and_broadcasts_even_unchanged() {
    let mut h = Harness::default();
    let task = h.task(256, &[2, 99]);
    for _ in 0..2 {
        let out = h.execute(&task);
        assert_eq!(out.status, Status::Handled);
        assert!(out.replies.is_empty());
        assert_eq!(h.team().invite_authority, 2);
        assert_eq!(destinations(&out, 274), vec![SELF, TARGET]);
        assert!(out.broadcasts.iter().all(|m| m.body == [2]));
        assert!(out.daemon_requests.is_empty());
    }
}
#[test]
fn team_authority_uses_exact_membership_and_both_leader_identities() {
    for case in 0..8 {
        let mut h = Harness::default();
        let task = h.task(256, &[1]);
        match case {
            0 => h.pc().party_id = 0,
            1 => h.team().leader_db_id = 999,
            2 => h.team().leader_runtime_id = OTHER,
            3 => h.party().members[0].state = 2,
            4 => h.party().members[0].runtime_id = OTHER,
            5 => {
                h.team().membership.remove(&10);
            }
            6 => {
                h.team().membership.insert(10, -1);
            }
            7 => h.state().pc_teams[1] = 0,
            _ => unreachable!(),
        }
        let out = h.execute(&task);
        only(
            &out,
            if case == 0 {
                Message {
                    command: 173,
                    body: 781u16.to_le_bytes().to_vec(),
                }
            } else {
                expected(256, 895)
            },
        );
        assert_eq!(h.team().invite_authority, 0);
    }
}
#[test]
fn team_broadcast_visits_indexed_parties_without_proximity_filter_or_dedup() {
    let mut h = Harness::default();
    let second = Party {
        members: vec![
            PartyMember {
                runtime_id: OTHER,
                state: 3,
                ..PartyMember::default()
            },
            PartyMember {
                runtime_id: TARGET,
                state: 2,
                ..PartyMember::default()
            },
            PartyMember {
                runtime_id: 0,
                state: 3,
                ..PartyMember::default()
            },
        ],
        ..Party::default()
    };
    h.team().parties.extend([None, Some(second)]);
    h.team().indexed_parties = vec![-1, 99, 1, 2, 0, 2];
    h.state().world_slots[3].as_mut().unwrap().pc.zone = None;
    let out = h.full(256, &[1]);
    assert_eq!(destinations(&out, 274), vec![OTHER, SELF, TARGET, OTHER]);
    h.peer().rights = 0;
    let out = h.full(256, &[1]);
    assert_eq!(destinations(&out, 274), vec![OTHER, SELF, OTHER]);
}
#[test]
fn exact_resolver_and_world_lookup_contracts() {
    let mut h = Harness::default();
    let s = h.state();
    assert_eq!(
        s.resolve(10, SELF),
        Some(PartyLocation {
            team: 1,
            party: 0,
            member: 0
        })
    );
    for (db, runtime) in [(0, SELF), (10, 0), (10, 0x20001), (99, SELF)] {
        assert!(s.resolve(db, runtime).is_none());
    }
    s.world_slots[2].as_mut().unwrap().pc.object_id = 0x10022;
    assert_eq!(s.by_id(TARGET).unwrap().pc.object_id, 0x10022); // No full-ID recheck.
    assert!(s.by_id(0x20002).is_none());
    assert!(s.by_id(0).is_none());
    assert_eq!(
        s.by_name(&[name("target"), vec![0], name("ignored")].concat())
            .unwrap()
            .pc
            .object_id,
        0x10022
    );
    assert!(s.by_name(&[]).is_none());
    assert!(s.by_name(&name("TARGET")).is_none());
    s.world_mask = 0xffff_ffff;
    assert!(s.by_id(SELF).is_none()); // Derived index >=20000.
    s.team_mask = 0;
    assert!(s.resolve(10, SELF).is_none());
}
#[test]
fn cancel_vote_sends_old_values_before_clear_then_self_ack() {
    let mut h = Harness::default();
    h.party().members[0].voted_for = 9;
    h.party().members[0].vote = 0xfe;
    let out = h.run(220, &[99]);
    assert_eq!(
        out.replies,
        vec![Message {
            command: 221,
            body: vec![]
        }]
    );
    assert_eq!(
        out.broadcasts,
        vec![Message {
            command: 222,
            body: vec![9, 0xfe]
        }]
    );
    assert!(matches!(
        &out.party_effects[..],
        [
            PartyEffect::Send { to: TARGET, .. },
            PartyEffect::VoteCleared { .. },
            PartyEffect::Send { to: SELF, .. }
        ]
    ));
    assert_eq!(
        (h.party().members[0].voted_for, h.party().members[0].vote),
        (0xff, 0)
    );
    only(
        &h.run(220, &[]),
        Message {
            command: 223,
            body: 821u16.to_le_bytes().to_vec(),
        },
    );
}
#[test]
fn cancel_vote_succeeds_without_target_or_with_offline_target() {
    for case in 0..5 {
        let mut h = Harness::default();
        h.party().members[0].voted_for = 9;
        h.party().members[0].vote = 1;
        match case {
            0 => h.party().members[1].slot = 7,
            1 => h.party().members[1].state = 2,
            2 => h.party().members[1].runtime_id = 0,
            3 => h.state().world_slots[2] = None,
            4 => h.peer().rights = 0,
            _ => unreachable!(),
        }
        let out = h.run(220, &[]);
        only(
            &out,
            Message {
                command: 221,
                body: vec![],
            },
        );
        assert_eq!(
            (h.party().members[0].voted_for, h.party().members[0].vote),
            (0xff, 0)
        );
    }
    let mut h = Harness::default();
    h.party().members[0].state = 2;
    only(
        &h.run(220, &[]),
        Message {
            command: 223,
            body: 820u16.to_le_bytes().to_vec(),
        },
    );
}
#[test]
fn vote_reset_reports_but_never_resets_or_sends() {
    for enabled in [false, true] {
        let mut h = Harness::default();
        h.world.party_config.vote_enabled = enabled;
        h.party().members[0].voted_for = 9;
        h.party().members[0].vote = 1;
        let out = h.run(224, &[77]);
        assert!(out.replies.is_empty());
        assert!(out.broadcasts.is_empty());
        assert!(out.changes.is_empty());
        assert!(out.daemon_requests.is_empty());
        assert!(h.services.queued.is_empty());
        assert_eq!(
            out.party_effects.contains(&PartyEffect::AssertionReport),
            enabled
        );
        assert_eq!(
            (h.party().members[0].voted_for, h.party().members[0].vote),
            (9, 1)
        );
    }
}
#[test]
fn party_authority_reasons_and_broadcast() {
    let mut h = Harness::default();
    let out = h.full(182, &[1]);
    assert!(out.replies.is_empty());
    assert_eq!(destinations(&out, 183), vec![SELF, TARGET]);
    assert_eq!(h.party().invite_authority, 1);
    only(&h.full(182, &[1]), expected(182, 798));
    h.pc().party_id = 0;
    only(&h.full(182, &[0]), expected(182, 796));
    h.pc().party_id = 1;
    h.team().membership.remove(&10);
    only(&h.full(182, &[0]), expected(182, 796));
    let mut h = Harness::default();
    h.party().members.swap(0, 1);
    only(&h.full(182, &[1]), expected(182, 797));
}
#[test]
fn pickup_rules_preserve_negative_money_and_allow_unchanged_values() {
    let mut h = Harness::default();
    let body = [vec![3, 2], q(TARGET), q(u64::MAX)].concat();
    for _ in 0..2 {
        let out = h.full(176, &body);
        assert!(out.replies.is_empty());
        assert_eq!(destinations(&out, 177), vec![SELF, TARGET]);
        assert_eq!(
            out.broadcasts[0].body,
            [vec![0, 2], q(TARGET), q(u64::MAX)].concat()
        );
        assert_eq!(
            (
                h.party().pickup_rule,
                h.party().grade,
                h.party().designated_id,
                h.party().money
            ),
            (0, 2, TARGET, u64::MAX)
        );
    }
    h.world.party_config.rule_three_mode = 2;
    h.full(176, &body);
    assert_eq!(h.party().pickup_rule, 3);
}
#[test]
fn pickup_rejects_wrong_role_signed_grade_or_designated_member() {
    for case in 0..7 {
        let mut h = Harness::default();
        let mut body = request(176);
        match case {
            0 => h.pc().party_id = 0,
            1 => h.party().members.swap(0, 1),
            2 => body[1] = 0,
            3 => body[1] = 128,
            4 => body[1] = 8,
            5 => h.party().members[1].state = 2,
            6 => {
                h.team().membership.remove(&10);
            }
            _ => unreachable!(),
        }
        only(
            &h.full(176, &body),
            expected(176, if case == 0 { 787 } else { 786 }),
        );
        assert_eq!(h.party().pickup_rule, 0);
    }
}
#[test]
fn difficulty_success_self_order_and_team_sync() {
    let mut h = Harness::default();
    let out = h.full(179, &[2]);
    assert_eq!(
        out.replies,
        vec![
            Message {
                command: 193,
                body: vec![2]
            },
            Message {
                command: 180,
                body: vec![2]
            }
        ]
    );
    assert_eq!(destinations(&out, 275), vec![SELF, TARGET]);
    assert!(out.broadcasts.iter().all(|m| m.body == [0, 2]));
    assert_eq!(h.party().difficulty, 2);
    let commands = out
        .party_effects
        .iter()
        .filter_map(|e| match e {
            PartyEffect::Send { message, .. } => Some(message.command),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(commands, vec![193, 180, 275, 275]);
    assert!(matches!(out.status, Status::Unsupported(_))); // No fabricated resize or metadata completion.
}
#[test]
fn difficulty_failure_reasons_truncate_and_still_sync() {
    for (case, reason) in [(0, 790), (1, 791), (2, 793), (3, 794), (4, 793), (5, 794)] {
        let mut h = Harness::default();
        match case {
            0 => h.party().members.swap(0, 1),
            1 => h.party().difficulty = 2,
            2 => {
                h.world.party_config.difficulty_maximum.remove(&2);
            }
            3 => h.party().difficulty_blockers[0] = 1,
            4 => {
                h.world.party_config.difficulty_maximum.insert(2, -1);
            }
            5 => h.party().difficulty_blockers[1] = 1,
            _ => unreachable!(),
        }
        let old = h.party().difficulty;
        let out = h.full(179, &[2]);
        assert_eq!(out.replies, vec![expected(179, reason)]);
        assert_eq!(h.party().difficulty, old);
        assert_eq!(
            destinations(&out, 275),
            if case == 0 {
                vec![TARGET, SELF]
            } else {
                vec![SELF, TARGET]
            }
        );
        assert!(out.broadcasts.iter().all(|m| m.body == [0, old]));
    }
    let mut h = Harness::default();
    h.pc().party_id = 0;
    only(&h.full(179, &[2]), expected(179, 789));
    h.pc().party_id = 1;
    h.team().membership.remove(&10);
    only(&h.full(179, &[2]), expected(179, 2));
    let mut h = Harness::default();
    h.team().mode = 0;
    assert!(h.full(179, &[2]).broadcasts.is_empty());
}
#[test]
fn difficulty_count_bound_and_negative_blockers_pass() {
    let mut h = Harness::default();
    h.world.party_config.difficulty_maximum.insert(2, 2);
    h.party().difficulty_blockers = [-1, 0];
    assert_eq!(h.full(179, &[2]).replies[0].command, 193);
    let mut h = Harness::default();
    h.world.party_config.difficulty_maximum.insert(2, 1);
    assert_eq!(h.full(179, &[2]).replies, vec![expected(179, 793)]);
}
#[test]
fn deferred_missing_membership_uses_spec_replies() {
    for (command, response, reason) in [
        (170, 171, 779u16),
        (245, 246, 781),
        (247, 249, 880),
        (250, 252, 887),
        (253, 255, 891),
        (256, 173, 781),
        (259, 261, 897),
    ] {
        let mut h = Harness::default();
        let task = h.task(command, &request(command));
        h.pc().party_id = 0;
        only(
            &h.execute(&task),
            Message {
                command: response,
                body: reason.to_le_bytes().to_vec(),
            },
        );
        assert!(h.services.party_calls.is_empty());
    }
}
#[test]
fn deferred_leader_target_failures_and_unread_manager_boundary() {
    for (command, response, reason) in [(170, 171, 779u16), (250, 252, 889), (253, 255, 890)] {
        let mut h = Harness::default();
        let task = h.task(command, &request(command));
        h.peer().rights = 0;
        only(
            &h.execute(&task),
            Message {
                command: response,
                body: reason.to_le_bytes().to_vec(),
            },
        );
    }
    let mut h = Harness::default();
    h.party().members[1].state = 2;
    only(
        &h.full(170, &q(TARGET)),
        Message {
            command: 171,
            body: 779u16.to_le_bytes().to_vec(),
        },
    );
    let mut h = Harness::default();
    let out = h.full(170, &q(TARGET));
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert!(out.broadcasts.is_empty());
    h.services.party_result = Some(0);
    only(
        &h.full(170, &q(TARGET)),
        Message {
            command: 171,
            body: 779u16.to_le_bytes().to_vec(),
        },
    );
}
#[test]
fn leave_task_suppression_and_literal_mode_two() {
    let mut h = Harness::default();
    h.world.arena = true;
    h.world.party_config.arena_task_mode = 2;
    h.world.zones.get_mut(&1).unwrap().party_subtype = 164;
    for (command, body) in [(168, vec![]), (243, vec![1]), (243, vec![255])] {
        let out = h.full(command, &body);
        assert_eq!(out.status, Status::Handled);
        assert!(h.services.party_calls.is_empty());
    }
    h.world.arena = false;
    h.world.zones.get_mut(&1).unwrap().attraction_kind = Some(10);
    assert_eq!(h.full(168, &[]).status, Status::Handled);
    h.services.party_result = Some(900);
    only(
        &h.full(243, &[255]),
        Message {
            command: 244,
            body: vec![0x84, 3, 2],
        },
    );
    assert!(matches!(
        h.services.party_calls.last(),
        Some(PartyOperation::LeaveTeam)
    ));
    h.services.party_result = Some(0);
    assert!(h.full(243, &[0]).replies.is_empty());
}
#[test]
fn auction_bid_boundaries_have_no_balance_or_minimum_gate() {
    for money in [0, 100] {
        let mut h = Harness::default();
        h.task(205, &[q(money), vec![255]].concat());
    }
    for money in [101, u64::MAX] {
        let mut h = Harness::default();
        h.pc().zone = None;
        only(
            &h.run(205, &[q(money), vec![1]].concat()),
            expected(205, 33),
        );
    }
}
#[test]
fn direct_party_suggestions_send_target_then_original_echo() {
    for (command, notice, ack, options) in [
        (147, 149, 148, vec![7, 2, 3, 4, 5, 6]),
        (151, 153, 152, vec![7, 2]),
    ] {
        let mut h = Harness::default();
        let out = h.run(command, &target_body(None, 0, "wire", &options));
        assert_eq!(destinations(&out, notice), vec![TARGET]);
        assert_eq!(
            out.broadcasts[0],
            Message {
                command: notice,
                body: [wire_name("source"), options.clone()].concat()
            }
        );
        assert_eq!(
            out.replies[0],
            Message {
                command: ack,
                body: target_body(None, 0, "wire", &options[..2])
            }
        );
        assert!(matches!(
            out.party_effects[0],
            PartyEffect::Send { to: TARGET, .. }
        ));
        assert!(matches!(
            out.party_effects[1],
            PartyEffect::Send { to: SELF, .. }
        ));
        assert!(out.changes.is_empty());
        assert!(h.services.queued.is_empty());
    }
}
#[test]
fn party_suggestion_self_missing_zone_and_arena_failures() {
    for (command, failure, reason) in [(147, 150, 740u16), (151, 154, 742)] {
        for (id, text) in [(SELF, "wire"), (0, "source"), (0x20001, "wire")] {
            let mut h = Harness::default();
            let options = if command == 147 {
                vec![7, 2, 3, 4, 5, 6]
            } else {
                vec![7, 2]
            };
            let out = h.run(command, &target_body(None, id, text, &options));
            only(
                &out,
                Message {
                    command: failure,
                    body: [
                        reason.to_le_bytes().to_vec(),
                        target_body(None, id, text, &options[..2]),
                    ]
                    .concat(),
                },
            );
        }
    }
    let mut h = Harness::default();
    h.peer().pc.party_id = 0;
    h.world.zones.get_mut(&1).unwrap().party_category = 3;
    for subtype in [26, 34, 38, 77, 177] {
        h.world.zones.get_mut(&1).unwrap().party_subtype = subtype;
        assert_eq!(
            &h.run(147, &request(147)).replies[0].body[..2],
            &741u16.to_le_bytes()
        );
    }
    h.peer().pc.party_id = 1;
    h.world.party_config.arena_enabled = true;
    h.peer().pc.arena_state = 5;
    assert_eq!(&h.run(147, &request(147)).replies[0].body[..2], &[6, 0]);
    h.world.party_config.arena_enabled = false;
    h.world.party_config.suggestion_task = true;
    h.peer().pc.zone = None;
    assert_eq!(&h.run(147, &request(147)).replies[0].body[..2], &[2, 0]);
    h.peer().pc.zone = Some(1);
    let out = h.run(147, &request(147));
    let task = out
        .changes
        .iter()
        .find_map(|c| match c {
            StateChange::PartyTaskQueued(t) => Some(t),
            _ => None,
        })
        .unwrap();
    assert_eq!(task.caster, TARGET);
}
#[test]
fn counted_names_consume_all_units_but_lookup_uses_nul_prefix() {
    let mut h = Harness::default();
    let out = h.run(151, &target_body(None, 0, "target\0suffix", &[0xab, 0xcd]));
    assert_eq!(
        out.broadcasts[0].body,
        [wire_name("source"), vec![0xab, 0xcd]].concat()
    );
    assert_eq!(
        out.replies[0].body,
        target_body(None, 0, "target", &[0xab, 0xcd])
    );
    let mut body = q(0);
    body.extend(4000u16.to_le_bytes());
    body.extend(vec![0; 8000]);
    body.extend([1, 2]);
    assert!(matches!(h.run(151, &body).status, Status::Unsupported(_)));
}
#[test]
fn team_reject_suggestion_has_no_resolved_self_check() {
    let mut h = Harness::default();
    let out = h.run(230, &target_body(Some(9), 0, "source", &[7, 8]));
    assert_eq!(destinations(&out, 232), vec![SELF]);
    assert_eq!(
        out.replies[0],
        Message {
            command: 231,
            body: target_body(Some(9), SELF, "source", &[7, 8])
        }
    );
    h.world.team_enabled = false;
    only(
        &h.run(230, &request(230)),
        Message {
            command: 229,
            body: [
                822u16.to_le_bytes().to_vec(),
                target_body(Some(1), TARGET, "wire", &[7]),
            ]
            .concat(),
        },
    );
    h.world.team_enabled = true;
    for id in [SELF, 0x20002] {
        only(
            &h.run(230, &target_body(Some(1), id, "wire", &[7, 8])),
            Message {
                command: 233,
                body: [
                    823u16.to_le_bytes().to_vec(),
                    target_body(Some(1), id, "wire", &[7, 8]),
                ]
                .concat(),
            },
        );
    }
}
#[test]
fn team_suggestion_permission_mapping_and_leader_reroute() {
    for mode in [0, 1, 2, 3, 255] {
        for allowed in [0, 1] {
            let mut h = Harness::default();
            h.services.party_result = Some(allowed);
            let out = h.run(
                226,
                &target_body(Some(mode), TARGET, "wire", &[7, 8, 9, 10, 11]),
            );
            let recipient = if allowed == 0 { SELF } else { TARGET };
            assert_eq!(destinations(&out, 228), vec![recipient]);
            assert_eq!(
                out.replies[0],
                Message {
                    command: 227,
                    body: target_body(
                        Some(mode),
                        recipient,
                        if allowed == 0 { "source" } else { "wire" },
                        &[7]
                    )
                }
            );
            assert_eq!(
                h.services.party_calls,
                vec![PartyOperation::SuggestPermission {
                    team: 1,
                    mode: if (1..=3).contains(&mode) { mode + 1 } else { 0 },
                    party: 0,
                    db: 20,
                    runtime: TARGET
                }]
            );
        }
    }
}
#[test]
fn team_suggestion_failures_and_permission_unknown_do_not_notify() {
    for case in 0..7 {
        let mut h = Harness::default();
        h.services.party_result = Some(0);
        match case {
            0 => h.world.team_enabled = false,
            1 => h.state().world_slots[2] = None,
            2 => h.peer().pc.team_member = false,
            3 => h.peer().pc.team_id = 0,
            4 => {
                h.team().membership.remove(&20);
            }
            5 => h.team().leader_runtime_id = 0,
            6 => h.state().world_slots[1].as_mut().unwrap().rights = 0,
            _ => unreachable!(),
        }
        let out = h.run(226, &request(226));
        assert_eq!(&out.replies[0].body[..2], &822u16.to_le_bytes());
        assert!(out.broadcasts.is_empty());
    }
    let mut h = Harness::default();
    let out = h.run(226, &request(226));
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert!(out.broadcasts.is_empty());
}
#[test]
fn invite_handlers_stop_at_excluded_boundary_without_tasks_or_registration() {
    for command in [157, 234] {
        let mut h = Harness::default();
        h.pc().party_registration = Some(PartyRegistration {
            value: 44,
            route: 55,
        });
        let out = h.run(command, &request(command));
        assert!(matches!(out.status, Status::Unsupported(_)));
        assert!(out.replies.is_empty());
        assert!(out.changes.is_empty());
        assert!(out.daemon_requests.is_empty());
        assert!(h.services.queued.is_empty());
    }
}
#[test]
fn invite_early_failures_echo_original_fields_and_normalized_difficulty() {
    let mut h = Harness::default();
    h.pc().party_user_blocked = true;
    assert_eq!(
        &h.run(157, &request(157)).replies[0].body[..2],
        &754u16.to_le_bytes()
    );
    assert_eq!(
        &h.run(234, &request(234)).replies[0].body[..2],
        &840u16.to_le_bytes()
    );
    h.pc().party_user_blocked = false;
    h.peer().rights = 0;
    let mut b = request(157);
    let difficulty_index = b.len() - 5;
    b[difficulty_index] = 255;
    let out = h.run(157, &b);
    assert_eq!(out.replies[0].command, 159);
    assert_eq!(&out.replies[0].body[..2], &747u16.to_le_bytes());
    assert_eq!(*out.replies[0].body.last().unwrap(), 1);
    assert_eq!(
        &h.run(234, &request(234)).replies[0].body[..2],
        &824u16.to_le_bytes()
    );
    let mut b = request(234);
    b[0] = 0;
    assert_eq!(&h.run(234, &b).replies[0].body[..2], &825u16.to_le_bytes());
    let out = h.run(234, &[3]);
    assert_eq!(
        out.replies[0].body,
        [vec![2, 0, 3], q(0), vec![0, 0, 0]].concat()
    );
}
#[test]
fn ordinary_invite_continuations_attach_to_target_and_use_authoritative_echoes() {
    let mut h = Harness::default();
    let peer = h.peer().pc.clone();
    let input = PartyInviteInput {
        target_id: 0,
        target_name: name("wire"),
        correlation: 77,
        options: [7, 2, 3, 4, 5, 6],
    };
    h.pc().party_registration = Some(PartyRegistration {
        value: 44,
        route: 55,
    });
    let out = continue_party_invite(&mut h.context(), input.clone(), &peer);
    assert_eq!(
        out.daemon_requests,
        vec![DaemonRequest::PartyRegistration {
            world: 8,
            route: 55,
            db_id: 10,
            value: 44,
            name: name("source")
        }]
    );
    let task = out
        .changes
        .iter()
        .find_map(|c| match c {
            StateChange::PartyTaskQueued(t) => Some(t),
            _ => None,
        })
        .unwrap();
    assert_eq!(task.caster, TARGET);
    assert_eq!(
        task.kind,
        PartyTaskKind::InviteParty {
            inviter_db: 10,
            inviter: SELF,
            correlation: 77,
            options: input.options
        }
    );
    let mut no_zone = peer.clone();
    no_zone.zone = None;
    let out = continue_party_invite(&mut h.context(), input, &no_zone);
    assert_eq!(
        out.replies[0].body,
        [
            751u16.to_le_bytes().to_vec(),
            q(TARGET),
            wire_name("target"),
            77u32.to_le_bytes().to_vec(),
            vec![7, 2]
        ]
        .concat()
    );
    let input = TeamInviteInput {
        mode: 1,
        target_id: 0,
        target_name: name("wire"),
        options: [7, 8, 9, 10, 11],
    };
    let out = continue_team_invite(&mut h.context(), input.clone(), &peer);
    assert!(out
        .changes
        .iter()
        .any(|c| matches!(c, StateChange::PartyTaskQueued(t) if t.caster == TARGET)));
    let out = continue_team_invite(&mut h.context(), input, &no_zone);
    assert_eq!(
        out.replies[0].body,
        [
            832u16.to_le_bytes().to_vec(),
            target_body(Some(1), TARGET, "target", &[7])
        ]
        .concat()
    );
}
#[test]
fn join_user_gates_and_registration_precede_failed_dispatch() {
    for command in [161, 239] {
        let mut h = Harness::default();
        h.pc().party_registration = Some(PartyRegistration {
            value: 44,
            route: 55,
        });
        h.services.queue_result = Gate::Fail;
        let out = h.run(command, &request(command));
        assert_eq!(out.daemon_requests.len(), 1);
        only(&out, expected(command, 2));
        h.pc().party_user_state = 1;
        let out = h.run(command, &request(command));
        only(
            &out,
            expected(command, if command == 161 { 768 } else { 863 }),
        );
        assert!(out.daemon_requests.is_empty());
    }
}
#[test]
fn match_registration_result_mapping_and_user_gate() {
    for (result, reason) in [(1, 1318u16), (2, 1317), (4, 1319), (3, 2), (-1, 2)] {
        let mut h = Harness::default();
        h.services.party_result = Some(result);
        only(
            &h.run(424, &request(424)),
            Message {
                command: 426,
                body: reason.to_le_bytes().to_vec(),
            },
        );
        assert_eq!(
            h.services.party_calls,
            vec![PartyOperation::RegisterMatch(123)]
        );
    }
    let mut h = Harness::default();
    h.services.party_result = Some(0);
    only(
        &h.run(424, &request(424)),
        Message {
            command: 425,
            body: vec![],
        },
    );
    h.pc().user_available = false;
    only(
        &h.run(424, &request(424)),
        Message {
            command: 426,
            body: 1320u16.to_le_bytes().to_vec(),
        },
    );
}
#[test]
fn match_reject_sends_authoritative_requester_only_and_missing_target_logs() {
    let mut h = Harness::default();
    let out = h.run(429, &request(429));
    assert!(out.replies.is_empty());
    assert_eq!(destinations(&out, 430), vec![TARGET]);
    assert_eq!(
        out.broadcasts[0].body,
        [q(SELF), 55u32.to_le_bytes().to_vec()].concat()
    );
    h.peer().rights = 0;
    let out = h.run(429, &request(429));
    assert!(out.replies.is_empty());
    assert!(out.broadcasts.is_empty());
    assert!(!out.logs.is_empty());
}
#[test]
fn shared_clear_member_task_keeps_175_for_team_missing_membership() {
    let mut h = Harness::default();
    let task = h.task(262, &[]);
    h.pc().party_id = 0;
    only(
        &h.execute(&task),
        Message {
            command: 175,
            body: vec![2, 0],
        },
    );
    only(
        &h.run(262, &[]),
        Message {
            command: 175,
            body: vec![2, 0],
        },
    ); // Existing doc-10 plan API.
}
