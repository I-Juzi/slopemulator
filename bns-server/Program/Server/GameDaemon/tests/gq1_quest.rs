//! Doc 20: quest, discovery and survey entry functions.
use gamedaemon::handlers::*;

const PC_ID: u64 = (1 << 48) | 0x11;

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
            db_id: 77,
            user_present: true,
            ..Pc::default()
        };
        let services = MemoryServices {
            skill_queue_result: Gate::Pass,
            quest: MemoryQuest {
                config: QuestConfig::Known(rules()),
                ..MemoryQuest::default()
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
            world: World::default(),
            services,
        }
    }
    fn pc(&mut self) -> &mut Pc {
        self.session.pc.as_mut().unwrap()
    }
    fn rules(&mut self) -> &mut QuestRules {
        match &mut self.services.quest.config {
            QuestConfig::Known(rules) => rules,
            QuestConfig::Unknown => panic!("test requires known rules"),
        }
    }
    fn run(&mut self, command: u16, body: &[u8]) -> Outcome {
        let before = self.services.skill_tasks.len();
        let mut ctx = Context {
            session: &mut self.session,
            world: &self.world,
            services: &mut self.services,
        };
        let out = dispatch(&mut ctx, command, body);
        if out.changes.contains(&StateChange::Queued(command)) {
            assert_eq!(self.services.skill_tasks.len(), before + 1);
            assert!(out.changes.contains(&StateChange::QueuedRequest {
                command,
                body: body.to_vec(),
            }));
        } else {
            assert_eq!(self.services.skill_tasks.len(), before);
        }
        out
    }
    fn last(&self) -> SkillTask {
        self.services.skill_tasks.last().unwrap().clone()
    }
    fn queued(&mut self, command: u16, body: &[u8], kind: QuestTask) {
        let out = self.run(command, body);
        assert_eq!(out.status, Status::Handled);
        assert!(out.replies.is_empty());
        assert_eq!(
            self.last(),
            SkillTask {
                command,
                caster: PC_ID,
                summon: false,
                zone: 9,
                kind: SkillTaskKind::Quest(kind),
            }
        );
    }
}
fn rules() -> QuestRules {
    QuestRules {
        region: 2,
        refresh_enabled: true,
        skip_max_level: 70,
        skip_mastery_enabled: true,
        skip_max_mastery: 50,
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
fn register_failure() -> Vec<(u16, Vec<u8>)> {
    vec![(400, vec![2, 0, 0xFF])]
}
fn string(prefix: &[u8], units: &[u16]) -> Vec<u8> {
    let mut body = prefix.to_vec();
    body.extend((units.len() as u16).to_le_bytes());
    for c in units {
        body.extend(c.to_le_bytes());
    }
    body
}
fn orders(values: &[u8]) -> Vec<u8> {
    let mut body = (values.len() as u16).to_le_bytes().to_vec();
    body.extend(values);
    body
}
fn unsupported(out: Outcome) {
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert!(out.replies.is_empty());
    assert!(out.changes.is_empty());
}

#[test]
fn list_tasks_and_failure_paths() {
    for (command, fail, kind) in [
        (86, 88, QuestTask::List),
        (89, 91, QuestTask::CompleteList),
        (93, 95, QuestTask::DiscoveryAreaList),
    ] {
        let mut h = H::new();
        h.queued(command, &[], kind.clone());
        // Native does not decode the body or check kind, arena or trial state.
        h.pc().native_kind = 2;
        h.pc().arena_state = 2;
        h.pc().t2_blocked = true;
        h.queued(command, &[0xFF], kind);
        h.services.skill_queue_result = Gate::Fail;
        assert_eq!(replies(&h.run(command, &[])), r(fail, 2));
        h.services.skill_queue_result = Gate::Unknown;
        unsupported(h.run(command, &[]));
        h.pc().zone = None;
        assert_eq!(replies(&h.run(command, &[])), r(fail, 2));
    }
}

#[test]
fn add_discovery_area_fields_and_unchecked_decode() {
    let mut h = H::new();
    h.queued(
        96,
        &[0xFE, 0xFF, 0xAA],
        QuestTask::AddDiscoveryArea { area: -2 },
    );
    h.queued(96, &[0, 0], QuestTask::AddDiscoveryArea { area: 0 });
    for body in [&[][..], &[1][..]] {
        unsupported(h.run(96, body));
    }
    h.pc().native_kind = 2;
    assert_eq!(replies(&h.run(96, &[])), r(98, 2));
    assert_eq!(replies(&h.run(96, &[1, 0])), r(98, 2));
    h.pc().zone = None;
    assert_eq!(replies(&h.run(96, &[])), r(98, 2));
    h.pc().zone = Some(9);
    h.pc().native_kind = 1;
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(96, &[1, 0])), r(98, 2));
    h.services.skill_queue_result = Gate::Unknown;
    unsupported(h.run(96, &[1, 0]));
}

#[test]
fn register_survey_count_and_element_failures() {
    let mut h = H::new();
    for body in [
        vec![],
        vec![0],
        vec![0xFF, 0xFF],
        vec![0, 0x80],
        vec![10, 0],
        vec![1, 0],
        vec![2, 0, 7],
    ] {
        assert_eq!(replies(&h.run(398, &body)), register_failure());
    }
    h.queued(398, &[0, 0], QuestTask::RegisterSurvey { orders: vec![] });
    let values = vec![0, 1, 2, 3, 4, 5, 6, 0x80, 0xFF];
    let mut body = orders(&values);
    body.push(0xAA); // The entry permits unused trailing bytes.
    h.queued(398, &body, QuestTask::RegisterSurvey { orders: values });
}

#[test]
fn register_survey_zone_kind_and_queue_failures() {
    let mut h = H::new();
    let body = orders(&[3]);
    h.pc().zone = None;
    assert_eq!(replies(&h.run(398, &body)), register_failure());
    h.pc().zone = Some(9);
    h.pc().native_kind = 2;
    assert_eq!(replies(&h.run(398, &body)), register_failure());
    h.pc().native_kind = 1;
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(398, &body)), register_failure());
    h.services.skill_queue_result = Gate::Unknown;
    unsupported(h.run(398, &body));
}

#[test]
fn answer_survey_fields_and_c_string_copy() {
    let mut h = H::new();
    h.queued(
        401,
        &string(&[0xFF, 0x80], &[0x41, 0, 0x42]),
        QuestTask::AnswerSurvey {
            survey: 0xFF,
            answer: 0x80,
            text: vec![0x41],
        },
    );
    h.queued(
        401,
        &string(&[1, 2], &[]),
        QuestTask::AnswerSurvey {
            survey: 1,
            answer: 2,
            text: vec![],
        },
    );
    let text = vec![0x42; 1899];
    h.queued(
        401,
        &string(&[1, 2], &text),
        QuestTask::AnswerSurvey {
            survey: 1,
            answer: 2,
            text,
        },
    );
}

#[test]
fn answer_survey_decode_zone_kind_and_queue_failures() {
    let mut h = H::new();
    for body in [
        vec![],
        vec![1],
        vec![1, 2],
        vec![1, 2, 1, 0],
        vec![1, 2, 0xFF, 0xFF],
        string(&[1, 2], &vec![0; 1900]),
    ] {
        assert_eq!(replies(&h.run(401, &body)), r(403, 2));
    }
    let body = string(&[1, 2], &[0x41]);
    h.pc().zone = None;
    assert_eq!(replies(&h.run(401, &body)), r(403, 2));
    h.pc().zone = Some(9);
    h.pc().native_kind = 2;
    assert_eq!(replies(&h.run(401, &body)), r(403, 2));
    h.pc().native_kind = 1;
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(401, &body)), r(403, 2));
    h.services.skill_queue_result = Gate::Unknown;
    unsupported(h.run(401, &body));
}

#[test]
fn extra_survey_replies_without_a_zone_or_task() {
    let mut h = H::new();
    h.pc().zone = None;
    h.pc().native_kind = 2;
    for body in [string(&[0xFF], &[]), string(&[1], &[0x41, 0, 0x42])] {
        let out = h.run(404, &body);
        assert_eq!(replies(&out), vec![(405, vec![])]);
        assert!(out.logs.contains(&"answer extra survey: client text log"));
        assert!(out.changes.is_empty());
    }
    for body in [
        vec![],
        vec![1],
        vec![1, 1, 0],
        vec![1, 0xFF, 0xFF],
        string(&[1], &vec![0; 1900]),
    ] {
        assert_eq!(replies(&h.run(404, &body)), r(406, 2));
    }
    assert_eq!(
        replies(&h.run(404, &string(&[1], &vec![0x41; 1899]))),
        vec![(405, vec![])]
    );
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn refresh_arena_before_configuration() {
    for arena in [2, 3, 5, 6] {
        let mut h = H::new();
        h.services.quest.config = QuestConfig::Unknown;
        h.pc().arena_state = arena;
        assert_eq!(replies(&h.run(1017, &[])), r(1019, 5));
        assert_eq!(h.services.quest.config_reads, 0);
    }
    let mut h = H::new();
    h.world.arena = true;
    h.rules().refresh_enabled = false;
    assert_eq!(replies(&h.run(1017, &[])), r(1019, 5));
    assert_eq!(h.services.quest.config_reads, 0);
}

#[test]
fn refresh_first_and_second_configuration_gates() {
    let mut h = H::new();
    h.rules().refresh_enabled = false;
    assert_eq!(replies(&h.run(1017, &[])), r(1019, 2));
    assert_eq!(h.services.quest.config_reads, 1);
    h.services.quest.config = QuestConfig::Unknown;
    unsupported(h.run(1017, &[]));
    let mut h = H::new();
    h.rules().refresh_enabled = false;
    h.services
        .quest
        .configs
        .push_back(QuestConfig::Known(rules()));
    assert_eq!(replies(&h.run(1017, &[])), r(1019, 2));
    assert_eq!(h.services.quest.config_reads, 2);
    let mut h = H::new();
    h.services.quest.config = QuestConfig::Unknown;
    h.services
        .quest
        .configs
        .push_back(QuestConfig::Known(rules()));
    unsupported(h.run(1017, &[]));
    assert_eq!(h.services.quest.config_reads, 2);
}

#[test]
fn refresh_zone_queue_and_no_body_field() {
    let mut h = H::new();
    h.pc().native_kind = 2;
    h.pc().t2_blocked = true;
    h.queued(1017, &[], QuestTask::RefreshSimpleQuestPack);
    h.queued(1017, &[0xAA], QuestTask::RefreshSimpleQuestPack);
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1017, &[])), r(1019, 2));
    h.pc().zone = Some(9);
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1017, &[])), r(1019, 2));
    h.services.skill_queue_result = Gate::Unknown;
    unsupported(h.run(1017, &[]));
}

#[test]
fn skip_region_decode_and_zone_order() {
    let mut h = H::new();
    h.services.quest.config = QuestConfig::Unknown;
    unsupported(h.run(1205, &[]));
    h.services.quest.config = QuestConfig::Known(rules());
    h.rules().region = 1;
    assert_eq!(replies(&h.run(1205, &[1, 0])), r(1207, 2627));
    h.rules().region = 2;
    h.pc().zone = None;
    for body in [&[][..], &[1][..]] {
        assert_eq!(replies(&h.run(1205, body)), r(1207, 2627));
    }
    assert_eq!(replies(&h.run(1205, &[0, 0xFF])), r(1207, 2));
}

#[test]
fn skip_signed_configuration_limits() {
    let mut h = H::new();
    for level in [0, 71, 0x80, 0xFF] {
        assert_eq!(replies(&h.run(1205, &[level, 0])), r(1207, 2630));
    }
    h.rules().skip_max_level = -1;
    assert_eq!(replies(&h.run(1205, &[1, 0])), r(1207, 2630));
    h.rules().skip_max_level = 70;
    for mastery in [51, 0x80, 0xFF] {
        assert_eq!(replies(&h.run(1205, &[1, mastery])), r(1207, 2630));
    }
    h.rules().skip_max_mastery = -1;
    assert_eq!(replies(&h.run(1205, &[1, 0])), r(1207, 2630));
    // Configuration checks occur before the task's native-kind check.
    h.pc().native_kind = 2;
    assert_eq!(replies(&h.run(1205, &[71, 0])), r(1207, 2630));
}

#[test]
fn skip_initializer_fixed_limits() {
    let mut h = H::new();
    h.rules().skip_max_level = 127;
    h.rules().skip_max_mastery = 127;
    assert_eq!(replies(&h.run(1205, &[71, 0])), r(1207, 2));
    assert_eq!(replies(&h.run(1205, &[70, 51])), r(1207, 2));
    h.rules().skip_mastery_enabled = false;
    for mastery in [51, 127, 0x80, 0xFF] {
        assert_eq!(replies(&h.run(1205, &[70, mastery])), r(1207, 2));
    }
    h.pc().native_kind = 2;
    assert_eq!(replies(&h.run(1205, &[1, 0])), r(1207, 2));
}

#[test]
fn skip_task_fields_and_queue_failures() {
    let mut h = H::new();
    // This entry has no arena or T2 gate.
    h.world.arena = true;
    h.pc().arena_state = 3;
    h.pc().t2_blocked = true;
    h.queued(
        1205,
        &[1, 0],
        QuestTask::SkipQuestTeleport {
            level: 1,
            mastery: 0,
        },
    );
    h.queued(
        1205,
        &[70, 50, 0xAA],
        QuestTask::SkipQuestTeleport {
            level: 70,
            mastery: 50,
        },
    );
    h.rules().skip_mastery_enabled = false;
    h.rules().skip_max_mastery = -1;
    h.queued(
        1205,
        &[1, 50],
        QuestTask::SkipQuestTeleport {
            level: 1,
            mastery: 50,
        },
    );
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1205, &[1, 0])), r(1207, 2));
    h.services.skill_queue_result = Gate::Unknown;
    unsupported(h.run(1205, &[1, 0]));
}

#[test]
fn replay_quest_fields_failure_reason_and_unchecked_decode() {
    let mut h = H::new();
    h.pc().native_kind = 2;
    h.pc().arena_state = 2;
    h.pc().t2_blocked = true;
    h.queued(
        1420,
        &[0xFF, 0xFF, 0xAA],
        QuestTask::ChangeToReplayQuest { quest: -1 },
    );
    h.queued(1420, &[0, 0], QuestTask::ChangeToReplayQuest { quest: 0 });
    for body in [&[][..], &[1][..]] {
        unsupported(h.run(1420, body));
    }
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1420, &[])), r(1422, 1));
    assert_eq!(replies(&h.run(1420, &[1, 0])), r(1422, 1));
    h.pc().zone = Some(9);
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1420, &[1, 0])), r(1422, 1));
    h.services.skill_queue_result = Gate::Unknown;
    unsupported(h.run(1420, &[1, 0]));
}

#[test]
fn quest_registration_and_dispatch_preconditions() {
    assert_eq!(
        QUEST_COMMANDS,
        &[86, 89, 93, 96, 398, 401, 404, 1017, 1205, 1420]
    );
    for &command in QUEST_COMMANDS {
        let mut h = H::new();
        h.session.authenticated = false;
        let out = h.run(command, &[]);
        assert!(out.replies.is_empty());
        assert!(out.logs.contains(&"missing authenticated PC"));
        h.session.authenticated = true;
        h.session.dispatch_permission = None;
        unsupported(h.run(command, &[]));
    }
    assert_eq!(MemoryServices::default().quest.config, QuestConfig::Unknown);
}
