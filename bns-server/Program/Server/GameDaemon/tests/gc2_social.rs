//! Doc 16 part 2: achievement and time-based play entry functions.
use gamedaemon::handlers::*;
use std::collections::BTreeMap;

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
            achievement_enabled: true,
            achievement_service_usable: true,
            time_play_enabled: true,
            time_play_schedules: [Some((1000, 2000)), None],
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

fn hh(a: u16, b: u16) -> Vec<u8> {
    let mut v = a.to_le_bytes().to_vec();
    v.extend(b.to_le_bytes());
    v
}
fn table(h: &mut H) {
    let mut t = BTreeMap::new();
    t.insert(
        (5, 1, 0),
        AchievementRecord {
            deprecated: false,
            completed_effect: 9,
        },
    );
    t.insert(
        (6, 1, 4),
        AchievementRecord {
            deprecated: true,
            completed_effect: 9,
        },
    );
    t.insert(
        (7, 1, 4),
        AchievementRecord {
            deprecated: false,
            completed_effect: 0,
        },
    );
    h.world.achievements = Some(t);
}

#[test]
fn complete_is_gated_and_silent() {
    let mut h = H::new();
    h.world.achievement_enabled = false;
    let out = h.run(896, &[]);
    assert!(out.replies.is_empty() && !unsupported(&out));
    h.world.achievement_enabled = true;
    let out = h.run(896, &[]);
    assert!(out.replies.is_empty() && unsupported(&out));
}

#[test]
fn title_install_order() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(898, &[1])), r(900, 2));
    // No table → boundary.
    assert!(unsupported(&h.run(898, &hh(5, 1))));
    table(&mut h);
    assert_eq!(replies(&h.run(898, &hh(9, 1))), r(900, 2090));
    assert_eq!(replies(&h.run(898, &hh(6, 1))), r(900, 2091));
    // Job 4 misses, job 0 hits.
    let out = h.run(898, &hh(5, 1));
    assert!(out.replies.is_empty() && !unsupported(&out));
    assert_eq!(h.last().command, 898);
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Chat(ChatTask::AchievementTitleInstall { id: 5, step: 1 })
    );
    // Zero step clears both fields and skips the table.
    h.run(898, &hh(9, 0));
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Chat(ChatTask::AchievementTitleInstall { id: 0, step: 0 })
    );
    h.pc().zone = None;
    assert_eq!(replies(&h.run(898, &hh(0, 0))), r(900, 2));
    h.pc().zone = Some(9);
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(898, &hh(0, 0))), r(900, 2));
}

#[test]
fn ability_install_codes() {
    let mut h = H::new();
    table(&mut h);
    assert_eq!(replies(&h.run(901, &hh(9, 1))), r(903, 2094));
    assert_eq!(replies(&h.run(901, &hh(6, 1))), r(903, 2095));
    assert_eq!(replies(&h.run(901, &hh(7, 1))), r(903, 2096));
    h.run(901, &hh(5, 1));
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Chat(ChatTask::AchievementAbilityInstall { id: 5, step: 1 })
    );
}

#[test]
fn game_grade_arena_and_lobby() {
    let mut h = H::new();
    let out = h.run(904, &[1]);
    assert!(out.replies.is_empty() && !out.logs.is_empty());
    let body = [vec![2u8], 300i32.to_le_bytes().to_vec()].concat();
    h.pc().arena_state = 3;
    let mut want = 5u16.to_le_bytes().to_vec();
    want.extend(&body);
    assert_eq!(replies(&h.run(904, &body)), vec![(906, want)]);
    h.pc().arena_state = 0;
    assert!(unsupported(&h.run(904, &body)));
    h.services.chat_result = Gate::Pass;
    let out = h.run(904, &body);
    assert!(out.replies.is_empty() && !unsupported(&out));
    assert_eq!(
        h.services.chat_calls.last(),
        Some(&ChatRequest::ActivateGameGrade {
            world: 3,
            db_id: 77,
            key_type: 2,
            grade: 300
        })
    );
}

#[test]
fn achievement_reward_order() {
    let mut h = H::new();
    assert!(h.run(1041, &[1]).replies.is_empty());
    h.pc().user_present = false;
    assert!(h.run(1041, &hh(5, 1)).replies.is_empty());
    h.pc().user_present = true;
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1041, &hh(5, 1))), r(1043, 2));
    h.pc().zone = Some(9);
    h.world.achievement_service_usable = false;
    assert_eq!(replies(&h.run(1041, &hh(5, 1))), r(1043, 2260));
    h.world.achievement_service_usable = true;
    h.services.chat_result = Gate::Fail;
    assert_eq!(replies(&h.run(1041, &hh(5, 1))), r(1043, 2));
    h.services.chat_result = Gate::Pass;
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1041, &hh(5, 1))), r(1043, 1));
    h.services.skill_queue_result = Gate::Pass;
    let out = h.run(1041, &hh(5, 1));
    assert!(out.replies.is_empty());
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Chat(ChatTask::TakeAchievementReward { id: 5, step: 1 })
    );
}

#[test]
fn time_based_play() {
    let mut h = H::new();
    assert!(unsupported(&h.run(1044, &[])));
    h.world.arena = true;
    assert_eq!(replies(&h.run(1044, &[])), r(1046, 2274));
    assert_eq!(replies(&h.run(1047, &[0])), r(1049, 2275));
    h.world.arena = false;
    h.world.time_play_enabled = false;
    assert_eq!(replies(&h.run(1044, &[])), r(1046, 2274));
    h.world.time_play_enabled = true;
    assert!(h.run(1047, &[]).replies.is_empty());
    assert_eq!(replies(&h.run(1047, &[2])), r(1049, 2276));
    assert_eq!(replies(&h.run(1047, &[1])), r(1049, 2277));
    h.world.utc_seconds = 2001;
    assert_eq!(replies(&h.run(1047, &[0])), r(1049, 2278));
    h.world.utc_seconds = 1500;
    assert!(unsupported(&h.run(1047, &[0])));
    h.services.chat_result = Gate::Fail;
    assert_eq!(replies(&h.run(1047, &[0])), r(1049, 1));
    h.services.chat_result = Gate::Pass;
    assert!(h.run(1047, &[0]).replies.is_empty());
}
