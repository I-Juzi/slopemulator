//! Doc 27 part 1: duel and party-battle zone tasks, and the 358 stub.
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
        Self {
            session: Session {
                authenticated: true,
                pc: Some(pc),
                dispatch_permission: Some(true),
                ..Session::default()
            },
            world: World::default(),
            services: MemoryServices {
                skill_queue_result: Gate::Pass,
                ..MemoryServices::default()
            },
        }
    }
    fn run(&mut self, command: u16, body: &[u8]) -> Outcome {
        let mut ctx = Context {
            session: &mut self.session,
            world: &self.world,
            services: &mut self.services,
        };
        dispatch(&mut ctx, command, body)
    }
    fn last(&self) -> DuelTask {
        match &self.services.skill_tasks.last().unwrap().kind {
            SkillTaskKind::Duel(k) => k.clone(),
            other => panic!("unexpected task {other:?}"),
        }
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

#[test]
fn choose_duel_order() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(277, &[])), r(279, 2));
    assert_eq!(replies(&h.run(277, &[0])), r(279, 898));
    assert_eq!(replies(&h.run(277, &[0x80])), r(279, 898));
    assert!(h.run(277, &[3]).replies.is_empty());
    assert_eq!(h.last(), DuelTask::ChooseDuelOrder { order: 3 });
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(277, &[3])), r(279, 2));
    h.session.pc.as_mut().unwrap().zone = None;
    assert_eq!(replies(&h.run(277, &[3])), r(279, 2));
}

#[test]
fn duel_and_party_battle_modes() {
    let mut h = H::new();
    for (cmd, want) in [
        (280, DuelTask::Duel { mode: 1 }),
        (283, DuelTask::Duel { mode: 3 }),
        (289, DuelTask::PartyBattle { mode: 0 }),
        (292, DuelTask::PartyBattle { mode: 1 }),
    ] {
        assert!(h.run(cmd, &[]).replies.is_empty());
        assert_eq!(h.last(), want);
    }
    h.services.skill_queue_result = Gate::Fail;
    for (cmd, fail) in [(280, 282), (283, 285), (289, 291), (292, 294)] {
        assert_eq!(replies(&h.run(cmd, &[])), r(fail, 2));
    }
    h.services.skill_queue_result = Gate::Pass;
    h.session.pc.as_mut().unwrap().zone = None;
    assert_eq!(replies(&h.run(292, &[])), r(294, 2));
}

#[test]
fn party_battle_vote() {
    let mut h = H::new();
    let target = (1u64 << 48) | 5;
    assert_eq!(replies(&h.run(1022, &[1])), r(1024, 2));
    assert_eq!(replies(&h.run(1022, &5u64.to_le_bytes())), r(1024, 2));
    assert!(h.run(1022, &target.to_le_bytes()).replies.is_empty());
    assert_eq!(h.last(), DuelTask::PartyBattleVote { target });
    h.session.pc.as_mut().unwrap().object_id = (2 << 48) | 1;
    assert_eq!(replies(&h.run(1022, &target.to_le_bytes())), r(1024, 2));
}

#[test]
fn dp_arena_stub_does_nothing() {
    let mut h = H::new();
    let out = h.run(358, &[1, 2, 3]);
    assert!(out.replies.is_empty());
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn duel_bot_challenge() {
    let mut h = H::new();
    h.session.pc.as_mut().unwrap().gm_grade = 1;
    assert_eq!(replies(&h.run(968, &7u64.to_le_bytes())), r(970, 2));
    h.session.pc.as_mut().unwrap().gm_grade = 0x80; // signed: not > 0
    assert!(h.run(968, &7u64.to_le_bytes()).replies.is_empty());
    assert_eq!(h.last(), DuelTask::DuelBotChallengeStart { bot: 7 });
    assert_eq!(replies(&h.run(968, &[1])), r(970, 2));
    assert!(h.run(973, &[4]).replies.is_empty());
    assert_eq!(
        h.last(),
        DuelTask::StrategicTool {
            op: 0,
            tool: 0,
            slot: 4
        }
    );
    assert!(h.run(977, &9u64.to_le_bytes()).replies.is_empty());
    assert_eq!(
        h.last(),
        DuelTask::StrategicTool {
            op: 1,
            tool: 9,
            slot: 0
        }
    );
    assert_eq!(replies(&h.run(973, &[])), r(975, 2));
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(977, &9u64.to_le_bytes())), r(979, 2));
}

fn pick(bot: u32, level: u8) -> Vec<u8> {
    let mut v = bot.to_le_bytes().to_vec();
    v.push(level);
    v
}

#[test]
fn training_room() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(1587, &[1, 0])), r(1589, 2251));
    assert_eq!(replies(&h.run(1587, &pick(0, 1))), r(1589, 2));
    assert_eq!(replies(&h.run(1587, &pick(15, 1))), r(1589, 2));
    assert_eq!(replies(&h.run(1587, &pick(14, 11))), r(1589, 2));
    assert!(h.run(1587, &pick(14, 10)).replies.is_empty());
    assert_eq!(h.last(), DuelTask::TrainingRoomPick { bot: 14, level: 10 });
    let bot = (240u64 << 48) | 3;
    assert_eq!(replies(&h.run(1590, &3u64.to_le_bytes())), r(1592, 2));
    assert!(h.run(1590, &bot.to_le_bytes()).replies.is_empty());
    assert_eq!(h.last(), DuelTask::TrainingRoomStart { bot });
    h.session.pc.as_mut().unwrap().gm_grade = 2;
    assert_eq!(replies(&h.run(1590, &bot.to_le_bytes())), r(1592, 2));
    assert!(h.run(1594, &[1, 2, 3]).replies.is_empty());
    assert_eq!(h.last(), DuelTask::TrainingRoomReward { a: 1, b: 2, c: 3 });
    assert_eq!(replies(&h.run(1594, &[1, 2])), r(1596, 2));
    h.session.pc.as_mut().unwrap().zone = None;
    assert_eq!(replies(&h.run(1587, &pick(1, 1))), r(1589, 2));
}
