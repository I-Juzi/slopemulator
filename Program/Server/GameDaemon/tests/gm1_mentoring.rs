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
            requester_object_id: PC_ID,
            inviter_object_id: T_ID,
            inviter_db_id: 88,
            mentor_db_id: 77,
            mentee_db_id: 88,
            max_mentees: 3,
            mentor_job: 4,
            mentee_job: 0,
            mentor_name: [0; 33],
            mentee_name: [0; 33],
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

#[test]
fn confirm_task_owner_is_mentee_for_both_types() {
    for (kind, owner, zone) in [(1, PC_ID, 9), (2, T_ID, 10)] {
        let mut h = H::new();
        add_target(&mut h);
        h.pc().mentoring_type = kind;
        h.services.mentoring_registry_result = MentoringRegistry::Clear;
        assert!(h.run(1103, &qq(T_ID, 88)).replies.is_empty());
        let task = h.last();
        assert_eq!((task.caster, task.zone), (owner, zone));
        let SkillTaskKind::Mentoring(MentoringTask::Confirm {
            requester_object_id,
            inviter_object_id,
            ..
        }) = task.kind
        else {
            panic!("expected confirm task");
        };
        assert_eq!(requester_object_id, PC_ID);
        assert_eq!(inviter_object_id, T_ID);
    }
}

#[test]
fn confirm_captures_signed_jobs_and_fixed_name_snapshots() {
    for kind in [1, 2] {
        let mut h = H::new();
        add_target(&mut h);
        h.pc().mentoring_type = kind;
        h.pc().job = 0xff;
        h.pc().name = vec![0x41, 0x42, 0, 0x43];
        let target = h.world.mentoring.players.get_mut(&T_ID).unwrap();
        target.job = 0x80;
        target.name = vec![0x44; 34];
        h.services.mentoring_registry_result = MentoringRegistry::Clear;
        assert!(h.run(1103, &qq(T_ID, 88)).replies.is_empty());
        h.pc().job = 7;
        h.pc().name.clear();
        h.world
            .mentoring
            .players
            .get_mut(&T_ID)
            .unwrap()
            .name
            .clear();
        let SkillTaskKind::Mentoring(MentoringTask::Confirm {
            mentor_job,
            mentee_job,
            mentor_name,
            mentee_name,
            ..
        }) = h.last().kind
        else {
            panic!("expected confirm task");
        };
        let mut pc_name = [0; 33];
        pc_name[..2].copy_from_slice(&[0x41, 0x42]);
        let target_name = [0x44; 33];
        let expected = if kind == 2 {
            (-1, -128, pc_name, target_name)
        } else {
            (-128, -1, target_name, pc_name)
        };
        assert_eq!((mentor_job, mentee_job, mentor_name, mentee_name), expected);
    }
}

#[test]
fn target_rights_gate_precedes_later_target_checks() {
    let mut h = H::new();
    add_target(&mut h);
    h.pc().mentoring_type = 2;
    h.world.party_config.arena_enabled = true;
    h.services.mentoring_registry_result = MentoringRegistry::Clear;
    let target = h.world.mentoring.players.get_mut(&T_ID).unwrap();
    target.arena_state = 3;
    for rights in [0, 1, 8] {
        h.world.mentoring.players.get_mut(&T_ID).unwrap().rights = rights;
        assert_eq!(
            replies(&h.run(1100, &name_body(1, &[0x41, 0x42]))),
            r(1102, 2447)
        );
        assert_eq!(replies(&h.run(1103, &qq(T_ID, 88))), r(1104, 2));
        assert_eq!(replies(&h.run(1111, &qh(T_ID))), eqh(1113, 2458, T_ID));
        assert_eq!(replies(&h.run(1115, &qh(T_ID))), eqh(1117, 2458, T_ID));
    }
    assert!(h.services.skill_tasks.is_empty());
    for rights in [2, 4, 6] {
        h.world.mentoring.players.get_mut(&T_ID).unwrap().rights = rights;
        assert_eq!(
            replies(&h.run(1100, &name_body(1, &[0x41, 0x42]))),
            r(1102, 2448)
        );
        assert_eq!(replies(&h.run(1111, &qh(T_ID))), eqh(1113, 6, T_ID));
        assert_eq!(replies(&h.run(1115, &qh(T_ID))), eqh(1117, 6, T_ID));
        assert!(h.run(1103, &qq(T_ID, 88)).replies.is_empty());
    }
}

#[test]
fn object_lookup_uses_configured_slot_and_keeps_wire_alias() {
    let mut h = H::new();
    add_target(&mut h);
    h.pc().mentoring_type = 2;
    h.services.mentoring_registry_result = MentoringRegistry::Clear;
    h.world.mentoring.slot_mask = 0xffff0;
    h.world.mentoring.slot_shift = 4;
    let occupant = (1 << 48) | 0x220;
    let alias = occupant | (1 << 32) | 0xf;
    let mut pc = h.world.mentoring.players.remove(&T_ID).unwrap();
    pc.object_id = occupant;
    h.world.mentoring.players.insert(occupant, pc.clone());
    assert!(h.run(1103, &qq(alias, 88)).replies.is_empty());
    let task = h.last();
    assert_eq!((task.caster, task.zone), (occupant, 10));
    let SkillTaskKind::Mentoring(MentoringTask::Confirm {
        inviter_object_id, ..
    }) = task.kind
    else {
        panic!("expected confirm task");
    };
    assert_eq!(inviter_object_id, alias);
    for command in [1111, 1115] {
        assert!(h.run(command, &qh(alias)).replies.is_empty());
        let expected = if command == 1111 {
            MentoringTask::Call { target: alias }
        } else {
            MentoringTask::Warp { target: alias }
        };
        assert_eq!(h.last().kind, SkillTaskKind::Mentoring(expected));
        h.services.skill_queue_result = Gate::Fail;
        assert_eq!(
            replies(&h.run(command, &qh(alias))),
            eqh(command + 2, 2, alias)
        );
        h.services.skill_queue_result = Gate::Pass;
    }
    for slot in [19999, 20000] {
        let id = (1 << 48) | (slot << 4);
        h.world.mentoring.players.insert(id, pc.clone());
        assert_eq!(h.world.mentoring.by_id(id, 6).is_some(), slot < 20000);
    }
    let invalid_ids = [0, 0x220, (2 << 48) | 0x220, (1 << 48) | 0x230];
    for id in invalid_ids {
        assert!(h.world.mentoring.by_id(id, 6).is_none());
        assert_eq!(replies(&h.run(1103, &qq(id, 88))), r(1104, 2));
        assert_eq!(replies(&h.run(1111, &qh(id))), eqh(1113, 2458, id));
        assert_eq!(replies(&h.run(1115, &qh(id))), eqh(1117, 2458, id));
    }
}

#[test]
fn invite_name_lookup_uses_nonempty_nul_prefix() {
    let mut h = H::new();
    add_target(&mut h);
    assert!(h
        .run(1100, &name_body(1, &[0x41, 0x42, 0, 0x43]))
        .replies
        .is_empty());
    assert!(matches!(
        h.last().kind,
        SkillTaskKind::Mentoring(MentoringTask::Invite { target: T_ID, .. })
    ));
    assert_eq!(
        replies(&h.run(1100, &name_body(1, &[0, 0x41]))),
        r(1102, 2447)
    );
    // A NUL does not hide a missing counted unit from the decoder.
    let mut truncated = name_body(1, &[0x41, 0x42, 0, 0x43]);
    truncated.truncate(truncated.len() - 2);
    assert_eq!(replies(&h.run(1100, &truncated)), r(1102, 2));
}
