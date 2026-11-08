//! Doc 18: challenge-party entry functions.
use gamedaemon::handlers::*;

const PC_ID: u64 = (1 << 48) | 0x11;
const NAME: &[u16] = &[0x41, 0x42];
const REPLY_COMMANDS: &[(u16, u16)] = &[(1068, 1070), (1071, 1074), (1075, 1077), (1080, 1082)];

struct H {
    session: Session,
    world: World,
    services: MemoryServices,
}
impl H {
    fn new() -> Self {
        let pc = Pc {
            object_id: PC_ID,
            zone: Some(9),
            native_kind: 1,
            job: 4,
            chat_state: 1,
            db_id: 77,
            short_96: 3,
            user_present: true,
            ..Pc::default()
        };
        let world = World {
            challenge_party_enabled: true,
            utc_seconds: 1234,
            tick: 1000,
            trait_train_interval: 100,
            ..World::default()
        };
        let services = MemoryServices {
            skill_queue_result: Gate::Pass,
            challenge: MemoryChallenge {
                season: Gate::Pass,
                name_rules: Gate::Pass,
                name_filter: Gate::Pass,
                ..MemoryChallenge::default()
            },
            ..MemoryServices::default()
        };
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
        let mut ctx = Context {
            session: &mut self.session,
            world: &self.world,
            services: &mut self.services,
        };
        dispatch(&mut ctx, command, body)
    }
    fn last(&self) -> SkillTask {
        self.services.skill_tasks.last().unwrap().clone()
    }
    fn add_target(&mut self, name: &[u16]) {
        // Name lookup permits self. It does not check the target zone or kind.
        let mut pc = self.pc().clone();
        pc.zone = None;
        pc.native_kind = 2;
        self.world
            .party
            .world_slots
            .push(Some(PartyPeer { pc, rights: 6 }));
        self.world.party.names.insert(name.to_vec(), 0);
    }
}
fn replies(out: &Outcome) -> Vec<(u16, Vec<u8>)> {
    out.replies
        .iter()
        .map(|m| (m.command, m.body.clone()))
        .collect()
}
fn r(command: u16, reason: u16) -> Vec<(u16, Vec<u8>)> {
    vec![(command, reason.to_le_bytes().to_vec())]
}
fn unsupported(out: &Outcome) -> bool {
    matches!(out.status, Status::Unsupported(why) if why.starts_with("❓"))
}
fn name_body(name: &[u16]) -> Vec<u8> {
    let mut body = (name.len() as u16).to_le_bytes().to_vec();
    for unit in name {
        body.extend(unit.to_le_bytes());
    }
    body
}
fn body(command: u16) -> Vec<u8> {
    match command {
        1068 | 1071 => name_body(NAME),
        1075 => 123u64.to_le_bytes().to_vec(),
        1078 => [123u64.to_le_bytes().to_vec(), vec![0xFE]].concat(),
        1080 => vec![0xFF],
        _ => unreachable!(),
    }
}
fn expected(command: u16) -> ChallengeTask {
    match command {
        1068 => ChallengeTask::Create {
            name: NAME.to_vec(),
        },
        1071 => ChallengeTask::Invite {
            name: NAME.to_vec(),
        },
        1075 => ChallengeTask::Confirm { invitation: 123 },
        1078 => ChallengeTask::Reject { invitation: 123 },
        1080 => ChallengeTask::Leave { option: 0xFF },
        _ => unreachable!(),
    }
}

#[test]
fn feature_precedes_arena_season_and_decode() {
    for &(command, failure) in REPLY_COMMANDS {
        let mut h = H::new();
        h.world.challenge_party_enabled = false;
        h.world.arena = true;
        h.services.challenge.season = Gate::Unknown;
        assert_eq!(replies(&h.run(command, &[])), r(failure, 2));
        assert!(h.services.challenge.calls.is_empty());
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn arena_form_b_precedes_season_and_decode() {
    for &(command, failure) in REPLY_COMMANDS {
        for state in [2, 3, 5, 6] {
            let mut h = H::new();
            h.pc().arena_state = state;
            h.services.challenge.season = Gate::Unknown;
            assert_eq!(replies(&h.run(command, &[])), r(failure, 5));
            assert!(h.services.challenge.calls.is_empty());
        }
        let mut h = H::new();
        h.world.arena = true;
        assert_eq!(replies(&h.run(command, &[])), r(failure, 5));
        assert!(h.services.challenge.calls.is_empty());
    }
}

#[test]
fn closed_and_unknown_season_precede_decode() {
    for &(command, failure) in REPLY_COMMANDS {
        let mut h = H::new();
        h.services.challenge.season = Gate::Fail;
        assert_eq!(replies(&h.run(command, &[])), r(failure, 2));
        assert_eq!(h.services.challenge.calls, vec![ChallengeCheck::SeasonOpen]);
        h.services.challenge.season = Gate::Unknown;
        let out = h.run(command, &[]);
        assert!(unsupported(&out));
        assert!(out.replies.is_empty());
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn decode_failure_reasons() {
    for (command, failure, reason) in [
        (1068, 1070, 2404),
        (1071, 1074, 2411),
        (1075, 1077, 2421),
        (1080, 1082, 2421),
    ] {
        let valid = body(command);
        for end in 0..valid.len() {
            let mut h = H::new();
            assert_eq!(replies(&h.run(command, &valid[..end])), r(failure, reason));
            assert_eq!(h.services.challenge.calls, vec![ChallengeCheck::SeasonOpen]);
            assert!(h.services.skill_tasks.is_empty());
        }
    }
}

#[test]
fn name_decoder_rejects_negative_and_excess_counts() {
    for (command, failure, reason) in [(1068, 1070, 2404), (1071, 1074, 2411)] {
        for invalid in [vec![0xFF, 0xFF], name_body(&[0x41; 33]), vec![1, 0, 0x41]] {
            let mut h = H::new();
            assert_eq!(replies(&h.run(command, &invalid)), r(failure, reason));
            assert_eq!(h.services.challenge.calls, vec![ChallengeCheck::SeasonOpen]);
        }
        let mut h = H::new();
        h.add_target(&[0x41; 32]);
        assert_eq!(
            h.run(command, &name_body(&[0x41; 32])).status,
            Status::Handled
        );
        assert_eq!(h.services.skill_tasks.len(), 1);
    }
}

#[test]
fn create_name_rules_precede_filter_and_zone() {
    let mut h = H::new();
    h.world.text_filter_present = true;
    h.pc().user_present = false;
    h.pc().zone = None;
    h.services.challenge.name_rules = Gate::Fail;
    assert_eq!(replies(&h.run(1068, &body(1068))), r(1070, 2405));
    assert_eq!(
        h.services.challenge.calls,
        vec![
            ChallengeCheck::SeasonOpen,
            ChallengeCheck::NameValid {
                name: NAME.to_vec()
            }
        ]
    );
    h.services.challenge.name_rules = Gate::Unknown;
    let out = h.run(1068, &body(1068));
    assert!(unsupported(&out));
    assert!(out.replies.is_empty());
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn create_filter_requires_user_before_filter_call() {
    for blocked_flag in [false, true] {
        let mut h = H::new();
        h.world.text_filter_present = true;
        h.pc().user_present = blocked_flag;
        h.pc().party_user_blocked = blocked_flag;
        h.services.challenge.name_filter = Gate::Unknown;
        assert_eq!(replies(&h.run(1068, &body(1068))), r(1070, 2));
        assert_eq!(h.services.challenge.calls.len(), 2);
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn create_filter_reject_unknown_and_pass() {
    let mut h = H::new();
    h.world.text_filter_present = true;
    h.pc().zone = None;
    h.services.challenge.name_filter = Gate::Fail;
    assert_eq!(replies(&h.run(1068, &body(1068))), r(1070, 2405));
    assert_eq!(
        h.services.challenge.calls,
        vec![
            ChallengeCheck::SeasonOpen,
            ChallengeCheck::NameValid {
                name: NAME.to_vec()
            },
            ChallengeCheck::NameAllowed {
                requester: PC_ID,
                name: NAME.to_vec()
            },
        ]
    );
    h.services.challenge.name_filter = Gate::Unknown;
    let out = h.run(1068, &body(1068));
    assert!(unsupported(&out));
    assert!(out.replies.is_empty());
    h.services.challenge.name_filter = Gate::Pass;
    assert_eq!(replies(&h.run(1068, &body(1068))), r(1070, 2));
    h.pc().zone = Some(9);
    assert_eq!(h.run(1068, &body(1068)).status, Status::Handled);
    assert_eq!(h.services.skill_tasks.len(), 1);
}

#[test]
fn create_without_filter_does_not_require_user() {
    let mut h = H::new();
    h.pc().user_present = false;
    h.pc().party_user_blocked = true;
    h.services.challenge.name_filter = Gate::Unknown;
    assert_eq!(h.run(1068, &body(1068)).status, Status::Handled);
    assert_eq!(h.services.skill_tasks.len(), 1);
    assert_eq!(h.services.challenge.calls.len(), 2);
}

#[test]
fn create_empty_name_fails_initializer_after_name_checks() {
    for name in [vec![], vec![0], vec![0, 0x41]] {
        let mut h = H::new();
        assert_eq!(replies(&h.run(1068, &name_body(&name))), r(1070, 2));
        assert_eq!(
            h.services.challenge.calls,
            vec![
                ChallengeCheck::SeasonOpen,
                ChallengeCheck::NameValid { name: vec![] }
            ]
        );
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn names_use_nul_prefix_but_tasks_retain_all_units() {
    let name = [0x41, 0x42, 0, 0x43];
    for command in [1068, 1071] {
        let mut h = H::new();
        h.add_target(NAME);
        h.world.text_filter_present = true;
        assert_eq!(h.run(command, &name_body(&name)).status, Status::Handled);
        let kind = if command == 1068 {
            ChallengeTask::Create {
                name: name.to_vec(),
            }
        } else {
            ChallengeTask::Invite {
                name: name.to_vec(),
            }
        };
        assert_eq!(h.last().kind, SkillTaskKind::ChallengeParty(kind));
        if command == 1068 {
            assert_eq!(
                h.services.challenge.calls[1],
                ChallengeCheck::NameValid {
                    name: NAME.to_vec()
                }
            );
            assert_eq!(
                h.services.challenge.calls[2],
                ChallengeCheck::NameAllowed {
                    requester: PC_ID,
                    name: NAME.to_vec()
                }
            );
        }
    }
}

#[test]
fn invite_lookup_precedes_zone_and_checks_rights() {
    let mut h = H::new();
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1071, &body(1071))), r(1074, 2413));
    h.add_target(NAME);
    h.world.party.world_slots[0].as_mut().unwrap().rights = 1;
    assert_eq!(replies(&h.run(1071, &body(1071))), r(1074, 2413));
    h.world.party.world_slots[0].as_mut().unwrap().rights = 2;
    assert_eq!(replies(&h.run(1071, &body(1071))), r(1074, 2));
    h.pc().zone = Some(9);
    assert_eq!(h.run(1071, &body(1071)).status, Status::Handled);
    assert_eq!(h.last().zone, 9);
    assert_eq!(h.last().kind, SkillTaskKind::ChallengeParty(expected(1071)));
    // Empty name lookup fails before the inline initializer.
    assert_eq!(replies(&h.run(1071, &name_body(&[]))), r(1074, 2413));
}

#[test]
fn confirm_zero_fails_initializer() {
    let mut h = H::new();
    let out = h.run(1075, &0u64.to_le_bytes());
    assert_eq!(replies(&out), r(1077, 2));
    assert!(!out.logs.is_empty());
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn zone_kind_and_queue_failure_paths() {
    for &(command, failure) in REPLY_COMMANDS {
        for gate in 0..3 {
            let mut h = H::new();
            h.add_target(NAME);
            match gate {
                0 => h.pc().zone = None,
                1 => h.pc().native_kind = 2,
                2 => h.services.skill_queue_result = Gate::Fail,
                _ => unreachable!(),
            }
            let out = h.run(command, &body(command));
            assert_eq!(replies(&out), r(failure, 2));
            assert!(!out.logs.is_empty());
            assert!(h.services.skill_tasks.is_empty());
            assert!(out.changes.is_empty());
        }
    }
}

#[test]
fn unknown_queue_never_records_success() {
    for &command in CHALLENGE_COMMANDS {
        let mut h = H::new();
        h.add_target(NAME);
        h.services.skill_queue_result = Gate::Unknown;
        let out = h.run(command, &body(command));
        assert!(unsupported(&out));
        assert!(out.replies.is_empty());
        assert!(out.changes.is_empty());
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn all_five_tasks_keep_fields_zone_and_raw_request() {
    assert_eq!(CHALLENGE_COMMANDS, &[1068, 1071, 1075, 1078, 1080]);
    for &command in CHALLENGE_COMMANDS {
        let mut h = H::new();
        h.add_target(NAME);
        let mut request = body(command);
        request.extend([0xAB, 0xCD]); // Native prefix decode permits trailing bytes.
        let out = h.run(command, &request);
        assert_eq!(out.status, Status::Handled);
        assert!(out.replies.is_empty());
        assert_eq!(h.services.skill_tasks.len(), 1);
        assert_eq!(
            h.last(),
            SkillTask {
                command,
                caster: PC_ID,
                summon: false,
                zone: 9,
                kind: SkillTaskKind::ChallengeParty(expected(command)),
            }
        );
        assert_eq!(
            out.changes,
            vec![
                StateChange::Queued(command),
                StateChange::QueuedRequest {
                    command,
                    body: request
                },
            ]
        );
    }
}

#[test]
fn reject_disabled_is_silent_and_arena_only_logs() {
    let mut h = H::new();
    h.world.challenge_party_enabled = false;
    h.world.arena = true;
    assert_eq!(h.run(1078, &[]), Outcome::default());
    h.world.challenge_party_enabled = true;
    for state in [0, 2, 3, 5, 6] {
        h.pc().arena_state = state;
        h.world.arena = state == 0;
        let out = h.run(1078, &[]);
        assert_eq!(out.status, Status::Handled);
        assert!(out.replies.is_empty());
        assert!(!out.logs.is_empty());
        assert!(h.services.challenge.calls.is_empty());
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn reject_decode_zone_kind_and_queue_failures_only_log() {
    for end in 0..9 {
        let mut h = H::new();
        let out = h.run(1078, &body(1078)[..end]);
        assert!(out.replies.is_empty());
        assert!(!out.logs.is_empty());
        assert!(h.services.skill_tasks.is_empty());
    }
    for gate in 0..3 {
        let mut h = H::new();
        match gate {
            0 => h.pc().zone = None,
            1 => h.pc().native_kind = 2,
            2 => h.services.skill_queue_result = Gate::Fail,
            _ => unreachable!(),
        }
        let out = h.run(1078, &body(1078));
        assert_eq!(out.status, Status::Handled);
        assert!(out.replies.is_empty());
        assert!(!out.logs.is_empty());
        assert!(out.changes.is_empty());
        assert!(h.services.skill_tasks.is_empty());
        assert!(h.services.challenge.calls.is_empty());
    }
}

#[test]
fn reject_skips_season_accepts_zero_and_ignores_byte() {
    for season in [Gate::Fail, Gate::Unknown] {
        for byte in [0, 1, 0xFF] {
            let mut h = H::new();
            h.services.challenge.season = season;
            let request = [0u64.to_le_bytes().to_vec(), vec![byte]].concat();
            let out = h.run(1078, &request);
            assert_eq!(out.status, Status::Handled);
            assert!(out.replies.is_empty());
            assert_eq!(
                h.last().kind,
                SkillTaskKind::ChallengeParty(ChallengeTask::Reject { invitation: 0 })
            );
            assert!(h.services.challenge.calls.is_empty());
        }
    }
}

#[test]
fn leave_does_not_check_option_range() {
    for option in [0, 1, 2, 0x7F, 0xFF] {
        let mut h = H::new();
        assert_eq!(h.run(1080, &[option]).status, Status::Handled);
        assert_eq!(
            h.last().kind,
            SkillTaskKind::ChallengeParty(ChallengeTask::Leave { option })
        );
    }
}

#[test]
fn opaque_mock_defaults_are_unknown() {
    let mut services = MemoryServices::default();
    for check in [
        ChallengeCheck::SeasonOpen,
        ChallengeCheck::NameValid {
            name: NAME.to_vec(),
        },
        ChallengeCheck::NameAllowed {
            requester: PC_ID,
            name: NAME.to_vec(),
        },
    ] {
        assert_eq!(services.challenge_party(&check), Gate::Unknown);
    }
    let mut h = H::new();
    h.services = MemoryServices::default();
    let out = h.run(1068, &body(1068));
    assert!(unsupported(&out));
    assert!(out.replies.is_empty());
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn dispatch_keeps_authentication_and_permission_gates() {
    for &command in CHALLENGE_COMMANDS {
        let mut h = H::new();
        h.session.authenticated = false;
        let out = h.run(command, &body(command));
        assert!(out.replies.is_empty());
        assert_eq!(out.logs, vec!["missing authenticated PC"]);
        h.session.authenticated = true;
        h.session.dispatch_permission = None;
        assert!(matches!(
            h.run(command, &body(command)).status,
            Status::Unsupported(_)
        ));
        assert!(h.services.challenge.calls.is_empty());
        assert!(h.services.skill_tasks.is_empty());
    }
}
