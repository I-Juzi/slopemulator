//! Doc 25: arena lobby entry functions. Harness follows gx1_misc.rs.
use gamedaemon::handlers::*;

const PC_ID: u64 = (1 << 48) | 0x11;
const TARGET: u64 = (1 << 48) | 0x22;
const TAIL: [u8; 8] = [9, 8, 7, 6, 5, 4, 3, 2];

struct H {
    session: Session,
    world: World,
    services: MemoryServices,
}
impl H {
    fn new() -> Self {
        let pc = Pc {
            object_id: PC_ID,
            db_id: 77,
            name: vec![0x41],
            short_96: -3,
            world_id: 99,
            zone: Some(9),
            user_present: true,
            arena_state: 3,
            arena_qwords: [11, 22],
            arena_lobby: ArenaLobbyPc {
                route: 33,
                kind: 1,
                attraction: 0,
                waiting_id: 44,
                waiting_tick: 0,
            },
            guild_id: 55,
            ..Pc::default()
        };
        let mut world = World {
            tick: 100_000,
            ..World::default()
        };
        world.party_config.arena_enabled = true;
        let mut services = MemoryServices {
            skill_queue_result: Gate::Pass,
            ..MemoryServices::default()
        };
        use ArenaLobbyStage::*;
        for stage in [
            UpdateSessionTimer,
            PortalExists,
            Enter,
            CrossOptionEnabled,
            SetMatchOption,
            LocalInvitation,
            CrossInvitation,
            GameTimeValid,
            SpecialMatchEnabled,
            Send,
        ] {
            services
                .arena_lobby
                .results
                .insert(stage, ArenaLobbyResult::Pass);
        }
        services
            .arena_lobby
            .results
            .insert(ReservationPending, ArenaLobbyResult::Fail);
        services
            .arena_lobby
            .results
            .insert(LocalWorld, ArenaLobbyResult::Fail);
        services
            .arena_lobby
            .results
            .insert(ReadyFailureTail, ArenaLobbyResult::ReplyTail(TAIL));
        Self {
            session: Session {
                authenticated: true,
                pc: Some(pc),
                dispatch_permission: Some(true),
                ..Session::default()
            },
            world,
            services,
        }
    }
    fn pc(&mut self) -> &mut Pc {
        self.session.pc.as_mut().unwrap()
    }
    fn run(&mut self, command: u16, body: &[u8]) -> Outcome {
        dispatch(
            &mut Context {
                session: &mut self.session,
                world: &self.world,
                services: &mut self.services,
            },
            command,
            body,
        )
    }
    fn set(&mut self, stage: ArenaLobbyStage, result: ArenaLobbyResult) {
        self.services.arena_lobby.results.insert(stage, result);
    }
    fn stages(&self) -> Vec<ArenaLobbyStage> {
        self.services
            .arena_lobby
            .calls
            .iter()
            .map(ArenaLobbyOperation::stage)
            .collect()
    }
    fn last_send(&self) -> &ArenaLobbyBackend {
        self.services
            .arena_lobby
            .calls
            .iter()
            .rev()
            .find_map(|op| match op {
                ArenaLobbyOperation::Send(request) => Some(request),
                _ => None,
            })
            .unwrap()
    }
}
fn replies(out: &Outcome) -> Vec<(u16, Vec<u8>)> {
    out.replies
        .iter()
        .map(|m| (m.command, m.body.clone()))
        .collect()
}
fn r(command: u16, reason: u16) -> Vec<(u16, Vec<u8>)> {
    let mut body = reason.to_le_bytes().to_vec();
    if command == 677 {
        body.extend(TAIL);
    }
    vec![(command, body)]
}
fn unsupported(out: &Outcome) {
    assert!(matches!(out.status, Status::Unsupported(_)), "{out:?}");
}
fn string(body: &mut Vec<u8>, value: &[u16]) {
    body.extend((value.len() as u16).to_le_bytes());
    for unit in value {
        body.extend(unit.to_le_bytes());
    }
}
fn blob(body: &mut Vec<u8>, value: &[u8]) {
    body.extend((value.len() as u16).to_le_bytes());
    body.extend(value);
}
fn target(world: i16, id: u64) -> Vec<u8> {
    let mut body = world.to_le_bytes().to_vec();
    body.extend(id.to_le_bytes());
    body
}
fn options(count: u16, maps: &[u16]) -> Vec<u8> {
    let mut body = vec![4, 2, 7, 1];
    body.extend(count.to_le_bytes());
    for map in maps {
        body.extend(map.to_le_bytes());
    }
    body
}
fn invite(world: i16) -> Vec<u8> {
    let mut body = world.to_le_bytes().to_vec();
    string(&mut body, &[0x42]);
    blob(&mut body, &[4, 5]);
    body
}
fn broadcast(attraction: u8) -> Vec<u8> {
    let mut body = vec![attraction];
    body.extend(0x8877_6655_4433_2211u64.to_le_bytes());
    string(&mut body, &[0x42]);
    blob(&mut body, &[4, 5]);
    body
}
fn join(option: u8) -> Vec<u8> {
    let mut body = 0x8877_6655_4433_2211u64.to_le_bytes().to_vec();
    string(&mut body, &[0x42]);
    body.extend((-12i16).to_le_bytes());
    body.extend([7, option]);
    body
}
fn valid(command: u16) -> Vec<u8> {
    match command {
        639 => 7u64.to_le_bytes().to_vec(),
        642 => vec![1, 0, 0x55],
        654 => options(1, &[3]),
        657 | 672 => target(2, TARGET),
        660 => vec![7, 0],
        663 => invite(2),
        666 => broadcast(1),
        669 => join(1),
        681 | 687 | 696 | 761 | 763 => vec![7],
        693 => vec![1, 0, 0x42, 0],
        _ => vec![],
    }
}
fn prefix() -> Vec<u8> {
    let mut body = 33u64.to_le_bytes().to_vec();
    body.extend(77u64.to_le_bytes());
    body.extend(PC_ID.to_le_bytes());
    body.extend((-3i16).to_le_bytes());
    body
}

#[test]
fn every_command_has_dispatch_and_feature_then_server_mode_gates() {
    let cases = [
        (639, 641, 1691),
        (642, 644, 1723),
        (645, 647, 1725),
        (651, 653, 1731),
        (654, 656, 1736),
        (657, 659, 1748),
        (660, 662, 1755),
        (663, 665, 1759),
        (666, 668, 1759),
        (669, 671, 1765),
        (672, 674, 1781),
        (675, 677, 1788),
        (678, 680, 1805),
        (681, 683, 1810),
        (684, 686, 1834),
        (687, 689, 1849),
        (690, 692, 1854),
        (693, 695, 1859),
        (696, 698, 1863),
        (761, 641, 1691),
        (763, 641, 1691),
    ];
    assert_eq!(ARENA_LOBBY_COMMANDS.len(), cases.len() + 2);
    for (command, failure, disabled) in cases {
        assert!(ARENA_LOBBY_COMMANDS.contains(&command));
        let mut h = H::new();
        h.world.party_config.arena_enabled = false;
        h.world.arena = true;
        assert_eq!(
            replies(&h.run(command, &[])),
            r(failure, disabled),
            "{command}"
        );
        h.world.party_config.arena_enabled = true;
        let mode_reply = if command == 657 { 656 } else { failure };
        assert_eq!(replies(&h.run(command, &[])), r(mode_reply, 5), "{command}");
        assert!(h.services.skill_tasks.is_empty());
        assert!(h
            .stages()
            .iter()
            .all(|s| *s == ArenaLobbyStage::ReadyFailureTail));
    }
    for command in [699, 702] {
        let mut h = H::new();
        h.world.party_config.arena_enabled = false;
        let out = h.run(command, &[]);
        assert!(out.replies.is_empty() && !out.logs.is_empty());
        h.world.party_config.arena_enabled = true;
        h.world.arena = true;
        assert!(h.run(command, &[]).replies.is_empty());
        assert!(h.stages().is_empty());
    }
}

#[test]
fn dispatch_still_requires_an_authenticated_pc_and_permission() {
    for command in ARENA_LOBBY_COMMANDS {
        let mut h = H::new();
        h.session.authenticated = false;
        assert!(h.run(*command, &valid(*command)).replies.is_empty());
        h.session.authenticated = true;
        h.session.dispatch_permission = None;
        unsupported(&h.run(*command, &valid(*command)));
        assert!(h.stages().is_empty());
    }
}

#[test]
fn malformed_bodies_use_native_reply_commands() {
    for (command, failure) in [
        (639, 641),
        (642, 641),
        (654, 656),
        (657, 659),
        (660, 662),
        (663, 665),
        (666, 668),
        (669, 671),
        (672, 674),
        (681, 683),
        (687, 689),
        (693, 695),
        (696, 698),
        (761, 641),
        (763, 641),
    ] {
        let mut h = H::new();
        assert_eq!(replies(&h.run(command, &[])), r(failure, 2), "{command}");
        assert!(h.stages().is_empty());
    }
}

#[test]
fn reserve_zone_and_fatigue_order() {
    let mut h = H::new();
    h.pc().zone = None;
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(639, &[])), r(641, 2));
    assert_eq!(replies(&h.run(639, &valid(639))), r(641, 2));
    h.pc().zone = Some(9);
    assert_eq!(replies(&h.run(639, &valid(639))), r(641, 1712));
    h.pc().user_present = true;
    h.pc().party_user_blocked = true;
    assert_eq!(replies(&h.run(639, &valid(639))), r(641, 1712));
    h.pc().party_user_blocked = false;
    h.pc().party_user_state = 1;
    assert_eq!(replies(&h.run(639, &valid(639))), r(641, 1712));
    assert!(h.stages().is_empty());
}

#[test]
fn reserve_pending_timer_portal_and_task_paths() {
    use ArenaLobbyStage::*;
    let mut h = H::new();
    h.set(ReservationPending, ArenaLobbyResult::Pass);
    let out = h.run(639, &valid(639));
    assert!(out.replies.is_empty() && out.changes.is_empty());
    assert_eq!(h.stages(), vec![ReservationPending]);
    h.services.arena_lobby.calls.clear();
    h.set(ReservationPending, ArenaLobbyResult::Fail);
    h.set(PortalExists, ArenaLobbyResult::Fail);
    assert_eq!(replies(&h.run(639, &valid(639))), r(641, 1687));
    assert_eq!(
        h.stages(),
        vec![ReservationPending, UpdateSessionTimer, PortalExists]
    );
    h.set(PortalExists, ArenaLobbyResult::Pass);
    h.set(UpdateSessionTimer, ArenaLobbyResult::Fail); // Optional send failure is ignored.
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(639, &valid(639))), r(641, 2));
    h.services.skill_queue_result = Gate::Pass;
    h.pc().t2_blocked = true; // No trial-play or PC-kind gate in these entries.
    h.pc().native_kind = 2;
    let out = h.run(639, &valid(639));
    assert_eq!(out.status, Status::Handled);
    assert!(out.changes.contains(&StateChange::QueuedRequest {
        command: 639,
        body: valid(639)
    }));
    let t = h.services.skill_tasks.last().unwrap();
    assert_eq!(
        (t.command, t.caster, t.zone, t.summon),
        (639, PC_ID, 9, false)
    );
    assert_eq!(
        t.kind,
        SkillTaskKind::ArenaLobby(ArenaLobbyTask::CheckPortal { portal_key: 7 })
    );
}

#[test]
fn reservation_unknown_boundaries_never_queue() {
    for stage in [
        ArenaLobbyStage::ReservationPending,
        ArenaLobbyStage::UpdateSessionTimer,
        ArenaLobbyStage::PortalExists,
    ] {
        let mut h = H::new();
        h.set(stage, ArenaLobbyResult::Unknown);
        unsupported(&h.run(639, &valid(639)));
        assert!(h.services.skill_tasks.is_empty());
        assert_eq!(h.stages().last(), Some(&stage));
    }
    let mut h = H::new();
    h.services.skill_queue_result = Gate::Unknown;
    unsupported(&h.run(639, &valid(639)));
}

#[test]
fn enter_state_helper_and_reset_paths() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(642, &valid(642))), r(644, 1724));
    h.pc().arena_state = 2;
    h.pc().arena_qwords[0] = 0;
    assert_eq!(replies(&h.run(642, &valid(642))), r(644, 1724));
    h.pc().arena_qwords[0] = 11;
    h.set(ArenaLobbyStage::Enter, ArenaLobbyResult::Reason(1777));
    assert_eq!(replies(&h.run(642, &valid(642))), r(644, 1777));
    assert_eq!(h.pc().arena_state, 2);
    assert!(
        matches!(h.services.arena_lobby.calls.last(), Some(ArenaLobbyOperation::Enter {
        context_key:22,kind:1,bytes,.. }) if bytes == &[0x55])
    );
    for result in [
        ArenaLobbyResult::Pass,
        ArenaLobbyResult::Reason(0),
        ArenaLobbyResult::Unknown,
    ] {
        h.set(ArenaLobbyStage::Enter, result);
        unsupported(&h.run(642, &valid(642)));
        assert_eq!(h.pc().arena_state, 2);
    }
    h.pc().arena_lobby.attraction = 6;
    h.set(ArenaLobbyStage::Enter, ArenaLobbyResult::Reason(1723));
    let mut expected = r(644, 1723);
    expected.push((646, vec![3]));
    assert_eq!(replies(&h.run(642, &valid(642))), expected);
    assert_eq!(h.pc().arena_state, 4);
    assert_eq!(h.pc().arena_qwords, [0, 0]);
    assert_eq!(
        h.pc().arena_lobby,
        ArenaLobbyPc {
            attraction: 6,
            ..ArenaLobbyPc::default()
        }
    );
    h.pc().arena_state = 2;
    h.pc().arena_qwords[0] = 1;
    h.set(ArenaLobbyStage::Enter, ArenaLobbyResult::Fail);
    assert_eq!(replies(&h.run(642, &valid(642))), expected);
}

#[test]
fn enter_feature_failure_does_not_reset_and_blob_limit_is_150() {
    let mut h = H::new();
    h.pc().arena_state = 2;
    h.world.party_config.arena_enabled = false;
    assert_eq!(replies(&h.run(642, &valid(642))), r(644, 1723));
    assert_eq!(h.pc().arena_state, 2);
    h.world.party_config.arena_enabled = true;
    for body in [vec![0xff, 0xff], vec![151, 0], vec![2, 0, 1]] {
        assert_eq!(replies(&h.run(642, &body)), r(641, 2));
    }
    let mut body = vec![];
    blob(&mut body, &[0; 150]);
    unsupported(&h.run(642, &body));
}

#[test]
fn leave_does_not_require_an_arena_record_or_zone() {
    let mut h = H::new();
    h.pc().arena_state = 4;
    assert_eq!(replies(&h.run(645, &[])), r(647, 1726));
    h.pc().arena_state = 3;
    h.pc().arena_qwords[0] = 0;
    h.pc().zone = None;
    h.set(ArenaLobbyStage::Send, ArenaLobbyResult::Fail);
    assert_eq!(replies(&h.run(645, &[])), r(647, 1725));
    assert_eq!(h.last_send().command, 36);
    let mut body = prefix();
    string(&mut body, &[0x41]);
    body.push(1);
    assert_eq!(h.last_send().body, body);
}

#[test]
fn room_state_and_record_errors_follow_decode() {
    for (command, failure, bad_state, no_record) in [
        (651, 653, 1732, 1733),
        (654, 656, 1737, 1738),
        (657, 659, 1749, 1750),
        (660, 662, 1756, 1757),
        (672, 674, 1782, 1783),
        (675, 677, 1789, 1790),
        (678, 680, 1806, 1807),
        (681, 683, 1811, 1812),
        (684, 686, 1835, 1836),
        (687, 689, 1850, 1851),
        (690, 692, 1855, 1856),
        (693, 695, 1860, 1862),
        (696, 698, 1864, 1866),
        (702, 704, 1880, 1882),
    ] {
        let mut h = H::new();
        h.pc().arena_state = 1;
        h.pc().arena_qwords[0] = 0;
        assert_eq!(
            replies(&h.run(command, &valid(command))),
            r(failure, bad_state),
            "{command}"
        );
        h.pc().arena_state = if command == 702 { 5 } else { 3 };
        assert_eq!(
            replies(&h.run(command, &valid(command))),
            r(failure, no_record),
            "{command}"
        );
        assert!(!h.stages().contains(&ArenaLobbyStage::Send));
    }
}

#[test]
fn send_failure_replies_and_success_boundaries_for_all_backend_entries() {
    for (command, failure, reason, backend) in [
        (645, 647, 1725, 36),
        (651, 653, 1731, 36),
        (657, 659, 1748, 44),
        (660, 662, 1755, 48),
        (666, 668, 1759, 18),
        (669, 671, 1765, 52),
        (672, 674, 1781, 55),
        (675, 677, 1788, 58),
        (678, 680, 1805, 62),
        (681, 683, 1810, 66),
        (684, 686, 1834, 74),
        (687, 689, 1849, 78),
        (690, 692, 1854, 82),
        (693, 695, 1859, 85),
        (696, 698, 1863, 89),
        (702, 704, 1879, 97),
    ] {
        let mut h = H::new();
        if command == 702 {
            h.pc().arena_state = 5;
        }
        h.set(ArenaLobbyStage::Send, ArenaLobbyResult::Fail);
        assert_eq!(
            replies(&h.run(command, &valid(command))),
            r(failure, reason),
            "{command}"
        );
        assert_eq!(h.last_send().command, backend);
        h.set(ArenaLobbyStage::Send, ArenaLobbyResult::Pass);
        let out = h.run(command, &valid(command));
        unsupported(&out);
        assert!(out.replies.is_empty());
        h.set(ArenaLobbyStage::Send, ArenaLobbyResult::Unknown);
        unsupported(&h.run(command, &valid(command)));
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn exact_backend_prefixes_suffixes_and_descriptors() {
    let mut target_suffix = TARGET.to_le_bytes().to_vec();
    target_suffix.extend(2i16.to_le_bytes());
    let cases = [
        (651, 36, "hqqqhsQ", vec![1, 0, 0x41, 0, 2]),
        (657, 44, "hqqqhqh", target_suffix.clone()),
        (660, 48, "hqqqhh", vec![7, 0]),
        (672, 55, "hqqqhqh", target_suffix),
        (675, 58, "hqqqhq", 55u64.to_le_bytes().to_vec()),
        (678, 62, "hqqqh", vec![]),
        (681, 66, "hqqqhcq", {
            let mut v = vec![7];
            v.extend(55u64.to_le_bytes());
            v
        }),
        (684, 74, "hqqqh", vec![]),
        (687, 78, "hqqqhc", vec![7]),
        (690, 82, "hqqqh", vec![]),
        (693, 85, "hqqqhs", vec![1, 0, 0x42, 0]),
        (702, 97, "hqqqhq", 44u64.to_le_bytes().to_vec()),
    ];
    for (command, backend, descriptor, suffix) in cases {
        let mut h = H::new();
        if command == 702 {
            h.pc().arena_state = 5;
        }
        h.run(command, &valid(command));
        let mut expected = prefix();
        expected.extend(suffix);
        assert_eq!(
            h.last_send(),
            &ArenaLobbyBackend {
                world: -3,
                command: backend,
                descriptor,
                body: expected,
            },
            "{command}"
        );
    }
}

#[test]
fn delegate_accepts_zero_fields_but_banish_rejects_them() {
    let mut h = H::new();
    unsupported(&h.run(657, &target(0, 0)));
    for body in [target(0, TARGET), target(-1, TARGET), target(2, 0)] {
        assert_eq!(replies(&h.run(672, &body)), r(674, 2));
    }
    let high_id = 0xfedc_ba98_7654_3210;
    h.run(672, &target(2, high_id));
    assert_eq!(&h.last_send().body[26..34], &high_id.to_le_bytes());
}

#[test]
fn match_options_mask_count_element_and_helper_errors() {
    let mut h = H::new();
    for body in [
        options(4, &[]),
        options(0xffff, &[]),
        vec![4, 2, 7, 1],
        options(2, &[1]),
    ] {
        assert_eq!(replies(&h.run(654, &body)), r(656, 1742));
    }
    h.set(ArenaLobbyStage::CrossOptionEnabled, ArenaLobbyResult::Fail);
    for reason in [2, 1736, 1740, 1742, 1747] {
        h.set(
            ArenaLobbyStage::SetMatchOption,
            ArenaLobbyResult::Reason(reason),
        );
        assert_eq!(
            replies(&h.run(654, &options(3, &[1, 1, 299]))),
            r(656, reason)
        );
        assert!(
            matches!(h.services.arena_lobby.calls.last(), Some(ArenaLobbyOperation::SetMatchOption {
            fields:[4,2,0,1], maps,.. }) if maps==&[1,1,299])
        );
    }
    h.set(ArenaLobbyStage::SetMatchOption, ArenaLobbyResult::Fail);
    assert_eq!(replies(&h.run(654, &options(0, &[]))), r(656, 1736));
    for result in [
        ArenaLobbyResult::Pass,
        ArenaLobbyResult::Reason(0),
        ArenaLobbyResult::Unknown,
    ] {
        h.set(ArenaLobbyStage::SetMatchOption, result);
        unsupported(&h.run(654, &options(0, &[])));
    }
    h.set(
        ArenaLobbyStage::CrossOptionEnabled,
        ArenaLobbyResult::Unknown,
    );
    let out = h.run(654, &[4, 2, 7, 1]);
    unsupported(&out); // Native reads this config before the missing count.
    assert!(out.replies.is_empty());
}

#[test]
fn local_invitation_lookup_and_target_categories() {
    let mut h = H::new();
    h.set(ArenaLobbyStage::LocalWorld, ArenaLobbyResult::Pass);
    assert_eq!(replies(&h.run(663, &invite(2))), r(665, 1760));
    h.world.mentoring.names.insert(vec![0x42], TARGET);
    for (state, category) in [
        (0, 0),
        (1, 1),
        (2, 2),
        (3, 2),
        (4, 2),
        (5, 2),
        (6, 3),
        (7, 0),
    ] {
        h.world.mentoring.players.insert(
            TARGET,
            Pc {
                object_id: TARGET,
                name: vec![0x42],
                arena_state: state,
                zone: None,
                ..Pc::default()
            },
        );
        assert_eq!(
            replies(&h.run(663, &invite(2))),
            vec![(664, vec![category])]
        );
        assert!(
            matches!(h.services.arena_lobby.calls.last(), Some(ArenaLobbyOperation::LocalInvitation {
            target:TARGET,bytes,.. }) if bytes==&[4,5])
        );
    }
    h.world.mentoring.players.get_mut(&TARGET).unwrap().rights = 0;
    assert_eq!(replies(&h.run(663, &invite(2))), r(665, 1760));
}

#[test]
fn cross_invitation_failure_and_local_delivery_unknown() {
    let mut h = H::new();
    h.set(ArenaLobbyStage::CrossInvitation, ArenaLobbyResult::Fail);
    assert_eq!(replies(&h.run(663, &invite(2))), r(665, 1759));
    assert!(
        matches!(h.services.arena_lobby.calls.last(), Some(ArenaLobbyOperation::CrossInvitation {
        world:2,name,bytes,sender }) if name==&[0x42] && bytes==&[4,5] && sender.world==-3)
    );
    h.services.arena_lobby.calls.clear();
    assert_eq!(replies(&h.run(663, &invite(0))), r(665, 1759));
    assert_eq!(h.stages(), vec![ArenaLobbyStage::CrossInvitation]);
    h.set(ArenaLobbyStage::CrossInvitation, ArenaLobbyResult::Pass);
    unsupported(&h.run(663, &invite(-1)));
    h.set(ArenaLobbyStage::LocalWorld, ArenaLobbyResult::Unknown);
    unsupported(&h.run(663, &invite(2)));
    h.set(ArenaLobbyStage::LocalWorld, ArenaLobbyResult::Pass);
    h.world.mentoring.names.insert(vec![0x42], TARGET);
    h.world.mentoring.players.insert(
        TARGET,
        Pc {
            object_id: TARGET,
            ..Pc::default()
        },
    );
    h.set(ArenaLobbyStage::LocalInvitation, ArenaLobbyResult::Unknown);
    unsupported(&h.run(663, &invite(2)));
}

#[test]
fn invitation_string_and_blob_capacity_checks() {
    let mut h = H::new();
    let mut body = 2i16.to_le_bytes().to_vec();
    string(&mut body, &[0x41; 33]);
    blob(&mut body, &[]);
    assert_eq!(replies(&h.run(663, &body)), r(665, 2));
    let mut body = 2i16.to_le_bytes().to_vec();
    string(&mut body, &[0x41; 32]);
    blob(&mut body, &[0; 512]);
    unsupported(&h.run(663, &body));
    body[68..70].copy_from_slice(&513u16.to_le_bytes());
    assert_eq!(replies(&h.run(663, &body)), r(665, 2));
}

#[test]
fn broadcast_attraction_mapping_and_full_body_without_room_gate() {
    let mut h = H::new();
    h.pc().arena_state = 0;
    h.pc().arena_qwords[0] = 0;
    for (attraction, category) in [(0, 1), (1, 2), (2, 2)] {
        unsupported(&h.run(666, &broadcast(attraction)));
        let mut expected = vec![category, 1];
        expected.extend(33u64.to_le_bytes());
        expected.extend(77u64.to_le_bytes());
        expected.extend(PC_ID.to_le_bytes());
        expected.extend(0x8877_6655_4433_2211u64.to_le_bytes());
        string(&mut expected, &[0x42]);
        blob(&mut expected, &[4, 5]);
        assert_eq!(
            h.last_send(),
            &ArenaLobbyBackend {
                world: -3,
                command: 18,
                descriptor: "hQQqqqqsb",
                body: expected,
            }
        );
    }
    for attraction in [3, 127, 255] {
        assert_eq!(replies(&h.run(666, &broadcast(attraction))), r(668, 2));
    }
}

#[test]
fn join_time_check_overrides_both_room_errors() {
    let mut h = H::new();
    for (state, key, reason) in [(0, 0, 1766), (3, 0, 1767)] {
        h.pc().arena_state = state;
        h.pc().arena_qwords[0] = key;
        h.set(ArenaLobbyStage::GameTimeValid, ArenaLobbyResult::Fail);
        assert_eq!(replies(&h.run(669, &join(1))), r(671, 1780));
        h.set(ArenaLobbyStage::GameTimeValid, ArenaLobbyResult::Pass);
        assert_eq!(replies(&h.run(669, &join(1))), r(671, reason));
    }
    h.set(ArenaLobbyStage::GameTimeValid, ArenaLobbyResult::Unknown);
    unsupported(&h.run(669, &join(1)));
    assert!(!h.stages().contains(&ArenaLobbyStage::Send));
}

#[test]
fn join_cross_mask_and_option_mapping_preserve_the_full_room_id() {
    for cross_enabled in [true, false] {
        for option in [0, 1, 2, 3, 255] {
            let mut h = H::new();
            h.set(
                ArenaLobbyStage::CrossOptionEnabled,
                if cross_enabled {
                    ArenaLobbyResult::Pass
                } else {
                    ArenaLobbyResult::Fail
                },
            );
            unsupported(&h.run(669, &join(option)));
            let mut expected = prefix();
            expected.extend(0x8877_6655_4433_2211u64.to_le_bytes());
            string(&mut expected, &[0x42]);
            expected.extend((-12i16).to_le_bytes());
            expected.extend([
                if cross_enabled { 7 } else { 0 },
                if option <= 2 { option } else { 0 },
            ]);
            assert_eq!(h.last_send().body, expected);
            assert_eq!(h.last_send().descriptor, "hqqqhqshcQ");
        }
    }
}

#[test]
fn fatigue_leave_send_precedes_ready_match_and_skip_failure_replies() {
    for (command, failure, left_reason) in [(675, 677, 1793), (681, 683, 1820), (684, 686, 1845)] {
        for busy_source in 0..3 {
            let mut h = H::new();
            match busy_source {
                0 => h.pc().user_present = false,
                1 => h.pc().party_user_blocked = true,
                _ => h.pc().party_user_state = 2,
            }
            assert_eq!(
                replies(&h.run(command, &valid(command))),
                r(failure, left_reason)
            );
            assert_eq!(h.last_send().command, 36);
            assert_eq!(h.last_send().body.last(), Some(&3));
            h.set(ArenaLobbyStage::Send, ArenaLobbyResult::Fail);
            assert_eq!(replies(&h.run(command, &valid(command))), r(failure, 2));
            h.set(ArenaLobbyStage::Send, ArenaLobbyResult::Unknown);
            unsupported(&h.run(command, &valid(command)));
            h.pc().arena_lobby.route = 0;
            assert_eq!(replies(&h.run(command, &valid(command))), r(failure, 2));
            h.pc().arena_lobby.route = 33;
            h.pc().arena_state = 1;
            assert_eq!(replies(&h.run(command, &valid(command))), r(failure, 2));
        }
    }
}

#[test]
fn ready_time_error_and_unknown_reply_tail_never_invent_a_short_body() {
    let mut h = H::new();
    h.set(ArenaLobbyStage::GameTimeValid, ArenaLobbyResult::Fail);
    assert_eq!(replies(&h.run(675, &[])), r(677, 1799));
    assert_eq!(
        h.stages(),
        vec![
            ArenaLobbyStage::GameTimeValid,
            ArenaLobbyStage::ReadyFailureTail
        ]
    );
    h.set(ArenaLobbyStage::ReadyFailureTail, ArenaLobbyResult::Unknown);
    let out = h.run(675, &[]);
    unsupported(&out);
    assert!(out.replies.is_empty());
    h.set(ArenaLobbyStage::GameTimeValid, ArenaLobbyResult::Unknown);
    unsupported(&h.run(675, &[]));
}

#[test]
fn special_match_feature_gate_and_forced_zero_flag() {
    for attraction in 4..=6 {
        let mut h = H::new();
        h.pc().arena_lobby.kind = 2;
        h.pc().arena_lobby.attraction = attraction;
        h.set(ArenaLobbyStage::SpecialMatchEnabled, ArenaLobbyResult::Fail);
        assert_eq!(replies(&h.run(681, &[0xff])), r(683, 1816));
        h.set(
            ArenaLobbyStage::SpecialMatchEnabled,
            ArenaLobbyResult::Unknown,
        );
        unsupported(&h.run(681, &[0xff]));
        h.set(ArenaLobbyStage::SpecialMatchEnabled, ArenaLobbyResult::Pass);
        unsupported(&h.run(681, &[0xff]));
        assert_eq!(h.last_send().body[26], 0);
    }
    for (kind, attraction) in [(1, 4), (2, 3), (2, 7)] {
        let mut h = H::new();
        h.pc().arena_lobby.kind = kind;
        h.pc().arena_lobby.attraction = attraction;
        h.run(681, &[0xff]);
        assert_eq!(h.last_send().body[26], 0xff);
        assert!(!h.stages().contains(&ArenaLobbyStage::SpecialMatchEnabled));
    }
}

#[test]
fn decide_is_one_byte_and_uses_the_reversed_backend_prefix() {
    let mut h = H::new();
    unsupported(&h.run(696, &[0xa5]));
    let mut expected = PC_ID.to_le_bytes().to_vec();
    expected.extend(77u64.to_le_bytes());
    expected.extend(33u64.to_le_bytes());
    expected.extend((-3i16).to_le_bytes());
    expected.push(0xa5);
    assert_eq!(
        h.last_send(),
        &ArenaLobbyBackend {
            world: -3,
            command: 89,
            descriptor: "hqqqhQ",
            body: expected
        }
    );
    unsupported(&h.run(696, &[0xa5, 1, 2, 3, 4, 5, 6, 7]));
    assert_eq!(h.last_send().body.len(), 27);
}

#[test]
fn waiting_number_state_rate_boundary_and_signed_subtraction() {
    let mut h = H::new();
    assert!(h.run(699, &[]).replies.is_empty());
    assert!(h.stages().is_empty());
    h.pc().arena_state = 5;
    for last in [70_001, 100_001] {
        // 29,999 elapsed, then a future stamp.
        h.pc().arena_lobby.waiting_tick = last;
        assert_eq!(h.run(699, &[]).status, Status::Handled);
        assert_eq!(h.pc().arena_lobby.waiting_tick, last);
        assert!(h.stages().is_empty());
    }
    h.pc().arena_lobby.waiting_tick = 70_000;
    unsupported(&h.run(699, &[]));
    assert_eq!(h.pc().arena_lobby.waiting_tick, 100_000);
    assert_eq!(h.last_send().command, 94);
    let mut expected = prefix();
    expected.extend(44u64.to_le_bytes());
    assert_eq!(h.last_send().body, expected);
    let calls = h.stages().len();
    h.run(699, &[]);
    assert_eq!(h.stages().len(), calls);
}

#[test]
fn waiting_number_sets_tick_before_missing_ids_or_failed_unknown_send() {
    for (route, waiting) in [(0, 44), (33, 0)] {
        let mut h = H::new();
        h.pc().arena_state = 5;
        h.pc().arena_lobby.route = route;
        h.pc().arena_lobby.waiting_id = waiting;
        assert_eq!(h.run(699, &[]).status, Status::Handled);
        assert_eq!(h.pc().arena_lobby.waiting_tick, 100_000);
        assert!(h.stages().is_empty());
    }
    for result in [ArenaLobbyResult::Fail, ArenaLobbyResult::Unknown] {
        let mut h = H::new();
        h.pc().arena_state = 5;
        h.set(ArenaLobbyStage::Send, result);
        let out = h.run(699, &[]);
        assert!(out.replies.is_empty());
        assert_eq!(h.pc().arena_lobby.waiting_tick, 100_000);
        if result == ArenaLobbyResult::Unknown {
            unsupported(&out);
        } else {
            assert_eq!(out.status, Status::Handled);
        }
    }
}

#[test]
fn waiting_number_wrap_and_zero_tick_follow_native_comparison() {
    let mut h = H::new();
    h.pc().arena_state = 5;
    h.world.tick = 10_000;
    h.pc().arena_lobby.waiting_tick = u64::MAX - 20_000;
    unsupported(&h.run(699, &[])); // Wrapped elapsed = 30,001.
    h.world.tick = 0;
    h.pc().arena_lobby.waiting_tick = 0;
    unsupported(&h.run(699, &[]));
    assert_eq!(h.pc().arena_lobby.waiting_tick, 0);
}

#[test]
fn party_agreement_and_final_decision_have_distinct_fatigue_gates_and_task_kinds() {
    for command in [761, 763] {
        let mut h = H::new();
        h.pc().zone = None;
        h.pc().user_present = false;
        assert_eq!(replies(&h.run(command, &[1])), r(641, 2));
        h.pc().zone = Some(9);
        if command == 761 {
            assert_eq!(replies(&h.run(command, &[1])), r(641, 1712));
            h.pc().user_present = true;
        }
        h.pc().native_kind = 2;
        h.pc().t2_blocked = true;
        for value in [0, 1, 2, 255] {
            let out = h.run(command, &[value]);
            assert_eq!(out.status, Status::Handled);
            let kind = if command == 761 {
                ArenaLobbyTask::PartyAgreement { agreed: value != 0 }
            } else {
                ArenaLobbyTask::PartyFinalDecision {
                    decided: value != 0,
                }
            };
            assert_eq!(
                h.services.skill_tasks.last().unwrap().kind,
                SkillTaskKind::ArenaLobby(kind)
            );
            assert!(out.changes.contains(&StateChange::QueuedRequest {
                command,
                body: vec![value]
            }));
        }
        h.services.skill_queue_result = Gate::Fail;
        assert_eq!(replies(&h.run(command, &[1])), r(641, 2));
        h.services.skill_queue_result = Gate::Unknown;
        unsupported(&h.run(command, &[1]));
    }
}

#[test]
fn default_arena_service_state_is_unknown() {
    let mut services = MemoryServices::default();
    assert_eq!(
        services.arena_lobby(&ArenaLobbyOperation::CrossOptionEnabled),
        ArenaLobbyResult::Unknown
    );
    assert_eq!(services.arena_lobby.calls.len(), 1);
    let mut h = H::new();
    h.services.arena_lobby = MemoryArenaLobby::default();
    unsupported(&h.run(639, &valid(639)));
    assert!(h.services.skill_tasks.is_empty());
}
