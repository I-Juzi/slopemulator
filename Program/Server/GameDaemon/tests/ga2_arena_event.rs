//! Doc 26: arena-event (GM) and arena dungeon recruit entry functions.
use gamedaemon::handlers::*;

const PC_ID: u64 = (1 << 48) | 0x11;
const T_ID: u64 = (1 << 48) | 0x22;
const NAME: [u16; 2] = [0x41, 0x42];

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
            short_96: 3,
            gm_grade: 1,
            user_present: true,
            ..Pc::default()
        };
        let mut world = World {
            arena: true,
            arena_event: ArenaEventWorld {
                recruit_enabled: true,
                attraction: 2,
            },
            ..World::default()
        };
        world.party_config.arena_enabled = true;
        let t = Pc {
            object_id: T_ID,
            zone: Some(9),
            name: NAME.to_vec(),
            ..Pc::default()
        };
        world.mentoring.players.insert(T_ID, t);
        world.mentoring.names.insert(NAME.to_vec(), T_ID);
        let mut services = MemoryServices {
            skill_queue_result: Gate::Pass,
            ..MemoryServices::default()
        };
        services.arena_event.portal = Gate::Pass;
        services.arena_event.geo_zone = Gate::Pass;
        services.arena_event.send = Gate::Pass;
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
    fn lobby() -> Self {
        let mut h = Self::new();
        h.world.arena = false;
        h
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
fn s(text: &[u16]) -> Vec<u8> {
    let mut v = (text.len() as u16).to_le_bytes().to_vec();
    for c in text {
        v.extend(c.to_le_bytes());
    }
    v
}
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}
fn kind(t: &SkillTask) -> ArenaEventTask {
    match &t.kind {
        SkillTaskKind::ArenaEvent(k) => k.clone(),
        other => panic!("unexpected task {other:?}"),
    }
}

#[test]
fn teleport_gates_and_task() {
    let mut h = H::new();
    h.pc().gm_grade = 0;
    assert_eq!(replies(&h.run(723, &s(&NAME))), r(725, 2));
    h.pc().gm_grade = 8;
    assert_eq!(replies(&h.run(723, &s(&NAME))), r(725, 2));
    h.pc().gm_grade = 7;
    assert_eq!(replies(&h.run(723, &[])), r(725, 2));
    assert_eq!(replies(&h.run(723, &s(&wide("ZZ")))), r(725, 1916));
    h.pc().zone = None;
    assert_eq!(replies(&h.run(723, &s(&NAME))), r(725, 2));
    h.pc().zone = Some(9);
    let out = h.run(723, &s(&NAME));
    assert!(out.replies.is_empty());
    assert_eq!(kind(&h.last()), ArenaEventTask::TeleportTo { target: T_ID });
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(723, &s(&NAME))), r(725, 2));
}

#[test]
fn recall_zone_rules_and_success_reply() {
    let mut h = H::new();
    h.world.arena = false;
    assert_eq!(replies(&h.run(726, &s(&NAME))), r(728, 2));
    h.world.arena = true;
    assert_eq!(replies(&h.run(726, &s(&wide("ZZ")))), r(728, 1917));
    h.world.mentoring.players.get_mut(&T_ID).unwrap().zone = Some(5);
    assert_eq!(replies(&h.run(726, &s(&NAME))), r(728, 1918));
    h.world.mentoring.players.get_mut(&T_ID).unwrap().zone = Some(9);
    let out = h.run(726, &s(&NAME));
    assert_eq!(replies(&out), vec![(727, vec![])]);
    assert_eq!(kind(&h.last()), ArenaEventTask::Recall { target: T_ID });
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(726, &s(&NAME))), r(728, 2));
}

#[test]
fn warp_to_pos_decode_geo_zone_and_reply() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(729, &[1, 0, 2, 0])), r(731, 2));
    h.services.arena_event.geo_zone = Gate::Fail;
    let body = [1, 0, 2, 0, 3, 0];
    assert_eq!(replies(&h.run(729, &body)), r(731, 2));
    h.services.arena_event.geo_zone = Gate::Unknown;
    assert!(matches!(h.run(729, &body).status, Status::Unsupported(_)));
    h.services.arena_event.geo_zone = Gate::Pass;
    assert_eq!(replies(&h.run(729, &body)), vec![(730, vec![])]);
    assert_eq!(
        kind(&h.last()),
        ArenaEventTask::WarpToPos { x: 1, y: 2, z: 3 }
    );
}

#[test]
fn duel_game_start_replies_282() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(732, &[])), r(282, 2));
    h.world.arena_event.attraction = 1;
    h.pc().gm_grade = 0;
    assert_eq!(replies(&h.run(732, &[])), r(282, 2));
    h.pc().gm_grade = 3;
    assert!(h.run(732, &[]).replies.is_empty());
    assert_eq!(kind(&h.last()), ArenaEventTask::DuelGameStart);
    h.world.party_config.arena_enabled = false;
    assert_eq!(replies(&h.run(732, &[])), r(282, 2));
}

#[test]
fn find_arena_session_maps_attraction_and_sends() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(733, &[0])), r(735, 5));
    let mut h = H::lobby();
    let mut body = vec![3];
    body.extend(s(&NAME));
    assert_eq!(replies(&h.run(733, &body)), r(735, 2));
    body[0] = 1;
    assert!(h.run(733, &body).replies.is_empty());
    let mut want = 77u64.to_le_bytes().to_vec();
    want.extend(PC_ID.to_le_bytes());
    want.push(2);
    want.extend(s(&NAME));
    assert_eq!(
        h.services.arena_event.calls.last(),
        Some(&ArenaEventOperation::Send {
            world: 3,
            command: 140,
            descriptor: "hqqQs",
            body: want,
        })
    );
    h.services.arena_event.send = Gate::Fail;
    assert_eq!(replies(&h.run(733, &body)), r(735, 1919));
    h.world.party_config.arena_enabled = false;
    assert_eq!(replies(&h.run(733, &body)), r(735, 1919));
}

#[test]
fn move_to_arena_state_checks() {
    let mut h = H::lobby();
    assert_eq!(replies(&h.run(736, &0i32.to_le_bytes())), r(738, 2));
    let id = 5i32.to_le_bytes();
    assert_eq!(replies(&h.run(736, &id)), r(738, 1922));
    h.pc().arena_state = 3;
    assert_eq!(replies(&h.run(736, &id)), r(738, 1923));
    h.pc().arena_qwords[0] = 1;
    assert!(h.run(736, &id).replies.is_empty());
    h.services.arena_event.send = Gate::Fail;
    assert_eq!(replies(&h.run(736, &id)), r(738, 1921));
    h.world.party_config.arena_enabled = false;
    assert_eq!(replies(&h.run(736, &id)), r(738, 1921));
}

#[test]
fn return_to_arena_lobby_portal() {
    let mut h = H::new();
    let key = 44u64.to_le_bytes();
    h.services.arena_event.portal = Gate::Fail;
    assert_eq!(replies(&h.run(739, &key)), r(741, 1927));
    h.services.arena_event.portal = Gate::Pass;
    assert!(h.run(739, &key).replies.is_empty());
    assert_eq!(
        kind(&h.last()),
        ArenaEventTask::ReturnToArenaLobby { portal_key: 44 }
    );
    h.world.arena = false;
    assert_eq!(replies(&h.run(739, &key)), r(741, 1929));
    h.world.party_config.arena_enabled = false;
    assert_eq!(replies(&h.run(739, &key)), r(741, 1928));
}

#[test]
fn return_all_pc_quirk_reply_741() {
    let mut h = H::new();
    let key = 44u64.to_le_bytes();
    assert_eq!(replies(&h.run(742, &key)), vec![(743, vec![])]);
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(742, &key)), r(741, 2));
    h.services.arena_event.portal = Gate::Fail;
    assert_eq!(replies(&h.run(742, &key)), r(744, 2));
}

#[test]
fn retrieve_pc_asset_runs_on_target() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(745, &s(&wide("ZZ")))), r(747, 1932));
    h.world.mentoring.players.get_mut(&T_ID).unwrap().zone = None;
    assert_eq!(replies(&h.run(745, &s(&NAME))), r(747, 2));
    h.world.mentoring.players.get_mut(&T_ID).unwrap().zone = Some(4);
    assert!(h.run(745, &s(&NAME)).replies.is_empty());
    let t = h.last();
    assert_eq!((t.caster, t.zone), (T_ID, 4));
    assert_eq!(
        kind(&t),
        ArenaEventTask::RetrievePcAsset { requester: PC_ID }
    );
}

#[test]
fn warp_to_duel_round_start_param() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(748, &[2])), r(750, 2));
    assert!(h.run(748, &[1]).replies.is_empty());
    assert_eq!(
        kind(&h.last()),
        ArenaEventTask::WarpToDuelRoundStartPcSpawn { flag: 1 }
    );
    h.world.arena = false;
    assert_eq!(replies(&h.run(748, &[1])), r(750, 1934));
    h.world.party_config.arena_enabled = false;
    assert_eq!(replies(&h.run(748, &[1])), r(750, 1933));
}

#[test]
fn battle_royal_state_keywords() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(751, &s(&wide("other")))), r(753, 2));
    assert_eq!(
        replies(&h.run(751, &s(&wide("StandBy")))),
        vec![(752, vec![])]
    );
    assert_eq!(
        kind(&h.last()),
        ArenaEventTask::SetBattleRoyalState { state: 1 }
    );
    h.run(751, &s(&wide("FINISH")));
    assert_eq!(
        kind(&h.last()),
        ArenaEventTask::SetBattleRoyalState { state: 4 }
    );
}

#[test]
fn zone_event_tasks() {
    let mut h = H::new();
    assert!(h.run(754, &[]).replies.is_empty());
    assert_eq!(kind(&h.last()), ArenaEventTask::InZonePcLoc);
    assert!(h.run(757, &[]).replies.is_empty());
    assert_eq!(kind(&h.last()), ArenaEventTask::BattleRoyalScore);
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(757, &[])), r(759, 2));
    h.pc().zone = None;
    assert_eq!(replies(&h.run(754, &[])), r(756, 2));
}

#[test]
fn recruit_gates_and_modes() {
    let mut h = H::new();
    let key = 44u64.to_le_bytes();
    assert!(h.run(959, &key).replies.is_empty());
    assert_eq!(
        kind(&h.last()),
        ArenaEventTask::CheckPortal {
            portal_key: 44,
            mode: 2
        }
    );
    assert!(h.run(963, &key).replies.is_empty());
    assert_eq!(
        kind(&h.last()),
        ArenaEventTask::CheckPortal {
            portal_key: 44,
            mode: 3
        }
    );
    assert_eq!(replies(&h.run(959, &[1])), r(962, 2));
    h.services.arena_event.portal = Gate::Fail;
    assert_eq!(replies(&h.run(963, &key)), r(966, 2216));
    h.world.arena_event.attraction = 1;
    assert_eq!(replies(&h.run(959, &key)), r(962, 2214));
    h.world.arena_event.recruit_enabled = false;
    assert_eq!(replies(&h.run(959, &key)), r(962, 2213));
    h.world.arena = false;
    assert_eq!(replies(&h.run(959, &key)), r(962, 2215));
    h.world.party_config.arena_enabled = false;
    assert_eq!(replies(&h.run(963, &key)), r(966, 2214));
}
