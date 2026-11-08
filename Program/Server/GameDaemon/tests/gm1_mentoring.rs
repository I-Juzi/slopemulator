//! Doc 17: mentoring entry functions.
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
            job: 4,
            chat_state: 1,
            db_id: 77,
            short_96: 3,
            user_present: true,
            ..Pc::default()
        };
        let world = World {
            mentoring: MentoringWorld {
                enabled: true,
                max_mentees: Some(3),
                ..MentoringWorld::default()
            },
            utc_seconds: 1234,
            tick: 1000,
            trait_train_interval: 100,
            ..World::default()
        };
        let services = MemoryServices {
            skill_queue_result: Gate::Pass,
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
    matches!(out.status, Status::Unsupported(_))
}

const T_ID: u64 = (1 << 48) | 0x22;

fn add_target(h: &mut H) {
    let t = Pc {
        object_id: T_ID,
        db_id: 88,
        zone: Some(10),
        native_kind: 1,
        mentoring_type: 1,
        ..Pc::default()
    };
    h.world.mentoring.players.insert(T_ID, t);
    h.world.mentoring.names.insert(vec![0x41, 0x42], T_ID);
}
fn name_body(kind: u8, name: &[u16]) -> Vec<u8> {
    let mut v = vec![kind];
    v.extend((name.len() as u16).to_le_bytes());
    for c in name {
        v.extend(c.to_le_bytes());
    }
    v
}
fn qq(a: u64, b: u64) -> Vec<u8> {
    [a.to_le_bytes(), b.to_le_bytes()].concat()
}
fn qh(target: u64) -> Vec<u8> {
    let mut v = target.to_le_bytes().to_vec();
    v.extend([0, 0]);
    v
}
fn eqh(command: u16, reason: u16, target: u64) -> Vec<(u16, Vec<u8>)> {
    let mut v = reason.to_le_bytes().to_vec();
    v.extend(target.to_le_bytes());
    v.extend([0, 0]);
    vec![(command, v)]
}

#[test]
fn change_type_order() {
    let mut h = H::new();
    h.world.mentoring.enabled = false;
    assert_eq!(replies(&h.run(1098, &[1])), r(1099, 2436));
    h.world.mentoring.enabled = true;
    h.pc().arena_state = 2;
    assert_eq!(replies(&h.run(1098, &[1])), r(1099, 5));
    h.pc().arena_state = 0;
    assert_eq!(replies(&h.run(1098, &[])), r(1099, 2));
    assert_eq!(replies(&h.run(1098, &[3])), r(1099, 2435));
    assert_eq!(replies(&h.run(1098, &[0])), r(1099, 2434));
    let out = h.run(1098, &[2]);
    assert!(out.replies.is_empty());
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Mentoring(MentoringTask::ChangeType { mentoring_type: 2 })
    );
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1098, &[2])), r(1099, 2));
}

#[test]
fn invite_order() {
    let mut h = H::new();
    let ab = [0x41u16, 0x42];
    assert_eq!(replies(&h.run(1100, &[1])), r(1102, 2));
    assert_eq!(replies(&h.run(1100, &name_body(3, &ab))), r(1102, 3));
    assert_eq!(replies(&h.run(1100, &name_body(1, &ab))), r(1102, 2447));
    add_target(&mut h);
    h.world.mentoring.players.get_mut(&T_ID).unwrap().db_id = 77;
    assert_eq!(replies(&h.run(1100, &name_body(1, &ab))), r(1102, 2447));
    h.world.mentoring.players.get_mut(&T_ID).unwrap().db_id = 88;
    h.world.party_config.arena_enabled = true;
    h.world
        .mentoring
        .players
        .get_mut(&T_ID)
        .unwrap()
        .arena_state = 5;
    assert_eq!(replies(&h.run(1100, &name_body(1, &ab))), r(1102, 2448));
    h.world
        .mentoring
        .players
        .get_mut(&T_ID)
        .unwrap()
        .arena_state = 0;
    h.world.mentoring.players.get_mut(&T_ID).unwrap().zone = None;
    assert_eq!(replies(&h.run(1100, &name_body(1, &ab))), r(1102, 2449));
    h.world.mentoring.players.get_mut(&T_ID).unwrap().zone = Some(10);
    let out = h.run(1100, &name_body(2, &ab));
    assert!(out.replies.is_empty());
    assert_eq!(h.last().zone, 9);
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Mentoring(MentoringTask::Invite {
            target: T_ID,
            target_zone: 10,
            inviter_db_id: 77,
            inviter_object_id: PC_ID,
            mentoring_type: 2,
        })
    );
}

#[test]
fn confirm_order() {
    let mut h = H::new();
    h.world.mentoring.max_mentees = None;
    assert_eq!(replies(&h.run(1103, &qq(T_ID, 88))), r(1104, 2454));
    h.world.mentoring.max_mentees = Some(3);
    assert_eq!(replies(&h.run(1103, &[1])), r(1104, 2));
    assert_eq!(replies(&h.run(1103, &qq(T_ID, 88))), r(1104, 2));
    add_target(&mut h);
    assert_eq!(replies(&h.run(1103, &qq(T_ID, 99))), r(1104, 2));
    // PC type 0 is neither mentor nor mentee.
    assert_eq!(replies(&h.run(1103, &qq(T_ID, 88))), r(1104, 2));
    h.pc().mentoring_type = 2;
    assert!(unsupported(&h.run(1103, &qq(T_ID, 88))));
    h.services.mentoring_registry_result = MentoringRegistry::AlreadyMentored;
    assert_eq!(replies(&h.run(1103, &qq(T_ID, 88))), r(1104, 2452));
    h.services.mentoring_registry_result = MentoringRegistry::MentorFull;
    assert_eq!(replies(&h.run(1103, &qq(T_ID, 88))), r(1104, 2453));
    h.services.mentoring_registry_result = MentoringRegistry::Clear;
    let out = h.run(1103, &qq(T_ID, 88));
    assert!(out.replies.is_empty());
    // Mentor = PC, mentee = target; the task goes to the mentee zone.
    assert_eq!(h.last().zone, 10);
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Mentoring(MentoringTask::Confirm {
            inviter_object_id: T_ID,
            inviter_db_id: 88,
            mentor_db_id: 77,
            mentee_db_id: 88,
            max_mentees: 3,
        })
    );
    h.pc().mentoring_type = 1;
    h.run(1103, &qq(T_ID, 88));
    assert_eq!(h.last().zone, 9);
}

#[test]
fn reject_and_leave() {
    let mut h = H::new();
    h.pc().arena_state = 6;
    assert_eq!(replies(&h.run(1105, &qq(1, 2))), r(1104, 5));
    h.pc().arena_state = 0;
    assert!(h.run(1105, &[1]).replies.is_empty());
    let out = h.run(1105, &qq(1, 2));
    assert!(out.replies.is_empty());
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Mentoring(MentoringTask::Reject {
            first: 1,
            second: 2
        })
    );
    h.world.mentoring.enabled = false;
    assert!(h.run(1105, &qq(1, 2)).replies.is_empty());
    assert_eq!(replies(&h.run(1107, &[])), r(1109, 2455));
    h.world.mentoring.enabled = true;
    assert_eq!(replies(&h.run(1107, &[1])), r(1109, 3));
    let body = [vec![1u8, 0], 5u64.to_le_bytes().to_vec()].concat();
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1107, &body)), r(1109, 2));
    h.pc().zone = Some(9);
    assert!(unsupported(&h.run(1107, &body)));
}

#[test]
fn call_and_warp() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(1111, &[1])), eqh(1113, 3, 0));
    assert_eq!(replies(&h.run(1111, &qh(0))), eqh(1113, 2458, 0));
    assert_eq!(replies(&h.run(1111, &qh(T_ID))), eqh(1113, 2458, T_ID));
    add_target(&mut h);
    h.world.party_config.arena_enabled = true;
    h.world
        .mentoring
        .players
        .get_mut(&T_ID)
        .unwrap()
        .arena_state = 3;
    assert_eq!(replies(&h.run(1111, &qh(T_ID))), eqh(1113, 6, T_ID));
    h.world
        .mentoring
        .players
        .get_mut(&T_ID)
        .unwrap()
        .arena_state = 0;
    assert!(h.run(1111, &qh(T_ID)).replies.is_empty());
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Mentoring(MentoringTask::Call { target: T_ID })
    );
    h.pc().instant_payment = true;
    assert_eq!(replies(&h.run(1115, &qh(T_ID))), eqh(1117, 2, T_ID));
    h.pc().instant_payment = false;
    h.pc().achievement_reward = true;
    assert_eq!(replies(&h.run(1115, &qh(T_ID))), eqh(1117, 2, T_ID));
    h.pc().achievement_reward = false;
    assert!(h.run(1115, &qh(T_ID)).replies.is_empty());
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Mentoring(MentoringTask::Warp { target: T_ID })
    );
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1115, &qh(T_ID))), eqh(1117, 2, T_ID));
    h.world.mentoring.enabled = false;
    assert_eq!(replies(&h.run(1115, &qh(T_ID))), eqh(1117, 2455, T_ID));
}
