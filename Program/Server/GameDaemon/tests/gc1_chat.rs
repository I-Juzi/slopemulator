//! Doc 16 part 1: chat, talk-session, petition and QQ entry functions.
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
            chat_state: 1,
            db_id: 77,
            short_96: 3,
            user_present: true,
            ..Pc::default()
        };
        let world = World {
            item_message_enabled: true,
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

#[test]
fn log_only_and_message_gates() {
    let mut h = H::new();
    let out = h.run(50, &[]);
    assert!(out.replies.is_empty() && !out.logs.is_empty());
    let out = h.run(395, &[]);
    assert!(out.replies.is_empty() && !out.logs.is_empty());
    // 51: chat state 1 reaches the chat routine boundary.
    assert!(unsupported(&h.run(51, &[])));
    h.pc().chat_state = 0;
    let out = h.run(51, &[]);
    assert_eq!(replies(&out), r(54, 90));
    h.world.chat_rule = true;
    h.world.text_filter_present = true;
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(51, &[])), r(54, 90));
    h.pc().user_present = true;
    assert!(unsupported(&h.run(51, &[])));
}

#[test]
fn message_by_item_order() {
    let mut h = H::new();
    // s (empty), c, h.
    let body = [0u8, 0, 1, 2, 0];
    h.world.item_message_enabled = false;
    let out = h.run(55, &body);
    assert!(out.replies.is_empty() && !unsupported(&out));
    h.world.item_message_enabled = true;
    h.world.arena = true;
    assert_eq!(replies(&h.run(55, &body)), r(57, 5));
    h.world.arena = false;
    assert_eq!(replies(&h.run(55, &[])), r(57, 96));
    h.pc().chat_state = 2;
    assert_eq!(replies(&h.run(55, &body)), r(57, 93));
    h.pc().chat_state = 1;
    assert!(unsupported(&h.run(55, &body)));
}

#[test]
fn dice_range_and_queue() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(58, &[1, 0])), r(60, 2));
    let body = |a: i16, b: i16| [a.to_le_bytes(), b.to_le_bytes()].concat();
    assert_eq!(replies(&h.run(58, &body(5, 3))), r(60, 98));
    assert_eq!(replies(&h.run(58, &body(-1, 3))), r(60, 98));
    assert_eq!(replies(&h.run(58, &body(0, 10001))), r(60, 98));
    let out = h.run(58, &body(1, 100));
    assert!(out.replies.is_empty());
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Chat(ChatTask::Dice { min: 1, max: 100 })
    );
    h.pc().chat_state = 0;
    assert_eq!(replies(&h.run(58, &body(1, 100))), r(60, 97));
    h.pc().chat_state = 1;
    h.pc().zone = None;
    assert_eq!(replies(&h.run(58, &body(1, 100))), r(60, 2));
    h.pc().zone = Some(9);
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(58, &body(1, 100))), r(60, 2));
}

#[test]
fn talk_session_and_social() {
    let mut h = H::new();
    let q = 0x1234u64.to_le_bytes();
    h.run(1216, &q);
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Chat(ChatTask::TalkSessionOpen { session: 0x1234 })
    );
    h.run(61, &q);
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Chat(ChatTask::TalkSessionClose { session: 0x1234 })
    );
    h.run(68, &(-3i16).to_le_bytes());
    assert_eq!(
        h.last().kind,
        SkillTaskKind::Chat(ChatTask::SetPcSocial { social: -3 })
    );
    h.services.skill_queue_result = Gate::Fail;
    assert!(h.run(61, &q).replies.is_empty());
    assert_eq!(replies(&h.run(1216, &q)), r(1218, 2));
    assert_eq!(replies(&h.run(68, &[0, 0])), r(70, 2));
    h.pc().zone = None;
    assert!(h.run(61, &q).replies.is_empty());
    assert_eq!(replies(&h.run(1216, &[])), r(1218, 2));
    assert_eq!(replies(&h.run(68, &[])), r(70, 2));
}

#[test]
fn channel_and_petition_boundaries() {
    let mut h = H::new();
    assert!(unsupported(&h.run(342, &[4])));
    h.services.chat_result = Gate::Fail;
    assert_eq!(replies(&h.run(342, &[4])), r(344, 2));
    assert_eq!(replies(&h.run(345, &[])), r(347, 2));
    assert_eq!(
        h.services.chat_calls[1],
        ChatRequest::ChattingChannel {
            command: 342,
            object_id: PC_ID,
            leave: false,
            channel: 4
        }
    );
    assert_eq!(
        h.services.chat_calls[2],
        ChatRequest::ChattingChannel {
            command: 345,
            object_id: PC_ID,
            leave: true,
            channel: 0xFF
        }
    );
    h.services.chat_result = Gate::Pass;
    assert!(h.run(390, &[]).replies.is_empty());
    assert_eq!(
        h.services.chat_calls.last().unwrap(),
        &ChatRequest::CurrentPetition {
            world: 3,
            db_id: 77
        }
    );
    assert!(h.run(1560, &[]).replies.is_empty());
    assert!(unsupported(&h.run(393, &[])));
    // 387: decode failure logs only; unknown type → 389/2.
    let out = h.run(387, &[]);
    assert!(out.replies.is_empty() && !unsupported(&out));
    assert_eq!(replies(&h.run(387, &[3])), r(389, 2));
    assert!(unsupported(&h.run(387, &[1])));
    // Petition arena gates.
    h.world.arena = true;
    assert_eq!(replies(&h.run(1557, &[])), vec![(1559, vec![5, 0, 0, 0])]);
    assert_eq!(replies(&h.run(1560, &[])), vec![(1562, vec![5, 0, 0, 0])]);
    assert_eq!(replies(&h.run(1563, &[])), r(1565, 5));
    h.world.arena = false;
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(1557, &[])), vec![(1559, vec![2, 0, 0, 0])]);
}

#[test]
fn cancel_petition_keeps_owner_identity_in_one_world() {
    let mut h = H::new();
    h.services.chat_result = Gate::Pass;
    h.pc().name = vec![0x0041, 0xD83D, 0xDE00, 0, 0x0058];
    assert!(h.run(1560, &[]).replies.is_empty());
    let first = h.services.chat_calls.last().unwrap().clone();

    h.pc().object_id = PC_ID + 1;
    h.pc().db_id = 0x0807_0605_0403_0201;
    h.pc().name = vec![0x0042];
    assert!(h.run(1560, &[]).replies.is_empty());
    let second = h.services.chat_calls.last().unwrap().clone();
    assert_ne!(first, second);
    assert_eq!(
        first,
        ChatRequest::CancelPetition {
            world: 3,
            identity: vec![0x0041, 0xD83D, 0xDE00, 0, 0x0058],
            db_id: 77,
        }
    );
    assert_eq!(
        second,
        ChatRequest::CancelPetition {
            world: 3,
            identity: vec![0x0042],
            db_id: 0x0807_0605_0403_0201,
        }
    );
    // u1 opcode, then two identical s2/i8 pairs. Do not send the NUL or suffix.
    let first_pair = [3, 0, 0x41, 0, 0x3D, 0xD8, 0, 0xDE, 77, 0, 0, 0, 0, 0, 0, 0];
    let second_pair = [1, 0, 0x42, 0, 1, 2, 3, 4, 5, 6, 7, 8];
    assert_eq!(
        first.cancel_petition_packet().unwrap(),
        [&[154][..], &first_pair, &first_pair].concat()
    );
    assert_eq!(
        second.cancel_petition_packet().unwrap(),
        [&[154][..], &second_pair, &second_pair].concat()
    );
    assert!(ChatRequest::CurrentPetition {
        world: 3,
        db_id: 77,
    }
    .cancel_petition_packet()
    .is_none());
}

#[test]
fn submit_petition_unchecked_decode_stays_at_boundary() {
    let mut h = H::new();
    // Empty body, then integer + first string only. The second string is absent.
    for body in [&[][..], &[1, 0, 0, 0, 0, 0][..]] {
        let out = h.run(1557, body);
        assert!(unsupported(&out));
        assert!(out.replies.is_empty());
        assert!(h.services.chat_calls.is_empty());
    }
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(1557, &[])), vec![(1559, vec![2, 0, 0, 0])]);
}

#[test]
fn board_gacha_reward_and_qq() {
    let mut h = H::new();
    let count = 2i16.to_le_bytes();
    assert_eq!(replies(&h.run(1185, &count)), r(1187, 2));
    h.world.board_gacha_enabled = true;
    h.pc().trial_jumping = true;
    assert_eq!(replies(&h.run(1185, &count)), r(1187, 2));
    h.pc().trial_jumping = false;
    h.pc().party_user_state = 1;
    assert_eq!(replies(&h.run(1185, &count)), r(1187, 2));
    h.pc().party_user_state = 0;
    assert_eq!(replies(&h.run(1185, &[])), r(1187, 2));
    assert_eq!(replies(&h.run(1185, &0i16.to_le_bytes())), r(1187, 3));
    assert!(unsupported(&h.run(1185, &count)));
    h.services.trade.board_gacha_result = Gate::Pass;
    assert!(h.run(1185, &count).replies.is_empty());
    assert_eq!(
        h.services.trade.board_gacha.last().unwrap(),
        &BoardGachaRequest::RewardTransaction {
            command: 1185,
            count: 2,
            object_id: PC_ID
        }
    );
    h.services.trade.board_gacha_result = Gate::Fail;
    assert_eq!(replies(&h.run(1185, &count)), r(1187, 2));
    // QQ group: same as 887. No interface → native false.
    for id in [844u16, 869, 893] {
        let a = h.run(id, &[]);
        let b = h.run(887, &[]);
        assert_eq!(a, b);
    }
    h.world.qq_interface_present = true;
    assert_eq!(h.run(857, &[]), h.run(887, &[]));
}
