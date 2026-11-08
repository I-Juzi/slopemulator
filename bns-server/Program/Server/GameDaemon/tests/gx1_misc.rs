//! Doc 19: misc entry functions.
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
            misc: MiscWorld {
                bot_report_min_level: 10,
                trace_route_log: true,
                slate_amp_pages: 2,
            },
            utc_seconds: 1234,
            tick: 10_000,
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
const T_ID: u64 = (1 << 48) | 0x22;

fn report(kind: u8, target: u64, name: &[u16]) -> Vec<u8> {
    let mut v = vec![kind];
    v.extend(target.to_le_bytes());
    v.extend((name.len() as u16).to_le_bytes());
    for c in name {
        v.extend(c.to_le_bytes());
    }
    v
}
fn eq(code: u16, kind: u8) -> Vec<(u16, Vec<u8>)> {
    let mut v = code.to_le_bytes().to_vec();
    v.push(kind);
    vec![(421, v)]
}

#[test]
fn stubs_do_nothing() {
    let mut h = H::new();
    for c in [356, 359, 360, 362] {
        let out = h.run(c, &[1, 2, 3]);
        assert!(out.replies.is_empty() && out.changes.is_empty());
        assert!(matches!(out.status, Status::Handled));
    }
}

#[test]
fn stop_cinematic() {
    let mut h = H::new();
    let out = h.run(165, &[]);
    assert!(replies(&out).is_empty());
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Misc(MiscTask::StopCinematic { value: 0 })
    );
    h.run(165, &7i32.to_le_bytes());
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Misc(MiscTask::StopCinematic { value: 7 })
    );
    h.pc().native_kind = 2;
    assert_eq!(replies(&h.run(165, &[])), r(167, 2));
    h.pc().zone = None;
    assert_eq!(replies(&h.run(165, &[])), r(167, 2));
    h.pc().zone = Some(9);
    h.pc().native_kind = 1;
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(165, &[])), r(167, 2));
}

#[test]
fn write_client_log() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(407, &[1, 0, 0x41, 0])), vec![(408, vec![])]);
    assert_eq!(replies(&h.run(407, &[5, 0])), r(409, 2));
}

#[test]
fn report_bot_spammer_order() {
    let mut h = H::new();
    h.pc().level = 20;
    let t = Pc {
        object_id: T_ID,
        name: vec![0x41, 0x42],
        ..Pc::default()
    };
    h.world.mentoring.players.insert(T_ID, t);
    h.world.mentoring.names.insert(vec![0x41, 0x42], T_ID);
    assert_eq!(replies(&h.run(419, &[2])), eq(2224, 2));
    assert_eq!(replies(&h.run(419, &report(3, T_ID, &[]))), eq(2224, 3));
    assert_eq!(replies(&h.run(419, &report(1, 5, &[]))), eq(2226, 1));
    h.pc().level = 9;
    assert_eq!(replies(&h.run(419, &report(1, T_ID, &[]))), eq(2225, 1));
    h.pc().level = 20;
    let ok = |n: &[u16], k: u8| {
        let mut v = vec![k];
        v.extend((n.len() as u16).to_le_bytes());
        for c in n {
            v.extend(c.to_le_bytes());
        }
        vec![(420, v)]
    };
    assert_eq!(
        replies(&h.run(419, &report(1, 0, &[0x41, 0x42]))),
        ok(&[0x41, 0x42], 1)
    );
    assert_eq!(h.pc().bot_report_tick, 10_000);
    // Rate limit: silent drop.
    let out = h.run(419, &report(1, T_ID, &[]));
    assert!(out.replies.is_empty());
    // Arena lobby: ID must be 0 and the name is used as given.
    h.pc().bot_report_tick = 0;
    h.pc().arena_state = 3;
    assert_eq!(replies(&h.run(419, &report(2, T_ID, &[0x43]))), eq(2224, 2));
    assert_eq!(replies(&h.run(419, &report(2, 0, &[0x43]))), ok(&[0x43], 2));
}

#[test]
fn trace_route_ip() {
    let mut h = H::new();
    let mut body = 2u16.to_le_bytes().to_vec();
    body.extend([1, 0, 0, 0, 2, 0, 0, 0]);
    let out = h.run(820, &body);
    assert!(out.replies.is_empty() && matches!(out.status, Status::Handled));
    assert!(out.logs.contains(&"trace route ip: game log"));
    let out = h.run(820, &6u16.to_le_bytes());
    assert!(matches!(out.status, Status::NativeFalse));
    let out = h.run(820, &[2, 0, 1, 0, 0, 0]);
    assert!(matches!(out.status, Status::NativeFalse));
}

#[test]
fn challenge_lists_and_secret_treasure() {
    let mut h = H::new();
    assert!(replies(&h.run(1130, &[])).is_empty());
    assert_eq!(h.last().kind, SkillTaskKind::Misc(MiscTask::ChallengeLists));
    assert!(replies(&h.run(1137, &4u16.to_le_bytes())).is_empty());
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Misc(MiscTask::SecretTreasureExpiration { request: 4 })
    );
    assert_eq!(replies(&h.run(1137, &[1])), r(1139, 2));
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1137, &[4, 0])), r(1028, 2));
    assert_eq!(replies(&h.run(1130, &[])), r(1132, 2));
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1137, &[4, 0])), r(1139, 2));
}

#[test]
fn set_slate_amp_slot_order() {
    let mut h = H::new();
    let body = [1u8, 3, 2, 0];
    h.pc().arena_state = 2;
    assert_eq!(replies(&h.run(1188, &body)), r(1190, 5));
    h.pc().arena_state = 0;
    h.pc().t2_blocked = true;
    assert_eq!(replies(&h.run(1188, &body)), r(1190, 7));
    h.pc().t2_blocked = false;
    h.world.misc.slate_amp_pages = 0;
    assert_eq!(replies(&h.run(1188, &body)), r(1190, 2537));
    h.world.misc.slate_amp_pages = 1;
    assert_eq!(replies(&h.run(1188, &[1, 3])), r(1190, 2539));
    assert_eq!(replies(&h.run(1188, &[2, 3, 2, 0])), r(1190, 2539));
    assert_eq!(replies(&h.run(1188, &[1, 7, 2, 0])), r(1190, 2539));
    assert_eq!(replies(&h.run(1188, &[1, 3, 0, 0])), r(1190, 2539));
    assert!(replies(&h.run(1188, &body)).is_empty());
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Misc(MiscTask::SetSlateAmpSlot {
            page: 1,
            position: 3,
            order: 0x1_0002
        })
    );
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1188, &body)), r(1190, 2));
}

#[test]
fn challenge_list_take_reward() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(1423, &[])), r(1425, 2));
    assert_eq!(replies(&h.run(1423, &[2])), r(1425, 2));
    assert!(replies(&h.run(1423, &[1])).is_empty());
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Misc(MiscTask::ChallengeListTakeReward { sub_type: 1 })
    );
    h.pc().native_kind = 0;
    assert_eq!(replies(&h.run(1423, &[1])), r(1425, 2));
}
