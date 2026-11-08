//! Doc 12c part 1: BoardGacha, badge and slate entry functions.
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
            db_id: 0x7700,
            zone: Some(9),
            native_kind: 1,
            user_present: true,
            ..Pc::default()
        };
        let mut services = MemoryServices::default();
        services.trade.item_queue_result = Gate::Pass;
        services.trade.board_gacha_result = Gate::Pass;
        Self {
            session: Session {
                authenticated: true,
                pc: Some(pc),
                dispatch_permission: Some(true),
                ..Session::default()
            },
            world: World {
                board_gacha_enabled: true,
                badge_enabled: true,
                slate_pages: 2,
                ..World::default()
            },
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
    fn last(&self) -> BadgeSlateTask {
        match &self.services.trade.item_tasks.last().unwrap().kind {
            ItemTaskKind::BadgeSlate(t) => t.clone(),
            other => panic!("{other:?}"),
        }
    }
}
fn body(out: &Outcome) -> (u16, Vec<u8>) {
    assert_eq!(out.replies.len(), 1, "{out:?}");
    (out.replies[0].command, out.replies[0].body.clone())
}
fn r(command: u16, reason: u16) -> (u16, Vec<u8>) {
    (command, reason.to_le_bytes().to_vec())
}
fn s(v: u16) -> Vec<u8> {
    v.to_le_bytes().to_vec()
}

#[test]
fn board_gacha_entries() {
    let mut h = H::new();
    h.run(1160, &[]);
    assert_eq!(
        h.services.trade.board_gacha.last().unwrap(),
        &BoardGachaRequest::Manager {
            command: 1160,
            kind: 57,
            object_id: PC_ID,
            identity: 0x7700
        }
    );
    h.run(1182, &[]);
    assert_eq!(
        h.services.trade.board_gacha.last().unwrap(),
        &BoardGachaRequest::Transaction {
            command: 1182,
            start_event: 70,
            initial: 5251338,
            object_id: PC_ID
        }
    );
    // Missing user: transaction entries log only; manager entries do not check it.
    h.pc().user_present = false;
    let out = h.run(1167, &[]);
    assert!(out.replies.is_empty() && !out.logs.is_empty());
    h.run(1173, &[]);
    assert!(matches!(
        h.services.trade.board_gacha.last().unwrap(),
        BoardGachaRequest::Manager { kind: 59, .. }
    ));
    h.services.trade.board_gacha_result = Gate::Fail;
    assert_eq!(body(&h.run(1152, &[])), r(1154, 2));
    h.pc().user_present = true;
    assert_eq!(body(&h.run(1170, &[])), r(1172, 2));
    h.world.board_gacha_enabled = false;
    for (c, f) in [
        (1152, 1154),
        (1160, 1161),
        (1162, 1164),
        (1165, 1166),
        (1167, 1169),
        (1170, 1172),
        (1173, 1175),
        (1182, 1184),
    ] {
        assert_eq!(body(&h.run(c, &[])), r(f, 2));
    }
}

#[test]
fn badge_entries() {
    let mut h = H::new();
    h.run(1140, &[3]);
    assert_eq!(h.last(), BadgeSlateTask::ChangeBadgePage { page: 3 });
    h.pc().native_kind = 2;
    assert_eq!(body(&h.run(1140, &[3])), r(1142, 2));
    h.pc().native_kind = 1;
    // 1143: count 1..8, pairs 0..16, all → 2469.
    assert_eq!(body(&h.run(1143, &s(0))), r(1145, 2469));
    assert_eq!(body(&h.run(1143, &s(9))), r(1145, 2469));
    let t = [vec![1], s(2), s(3)].concat();
    assert_eq!(
        body(&h.run(1143, &[s(1), t.clone(), s(17)].concat())),
        r(1145, 2469)
    );
    h.run(1143, &[s(1), t.clone(), s(1), s(4), s(5)].concat());
    assert_eq!(
        h.last(),
        BadgeSlateTask::SynthesisBadge {
            first: vec![(1, 2, 3)],
            second: vec![(4, 5)]
        }
    );
    // Zone and U0 precede decode.
    h.pc().zone = None;
    assert_eq!(body(&h.run(1143, &[])), r(1145, 2));
    h.pc().user_present = false;
    assert_eq!(body(&h.run(1143, &[])), r(1145, 2482));
    h.world.badge_enabled = false;
    assert_eq!(body(&h.run(1143, &[])), r(1145, 1));
    assert_eq!(body(&h.run(1140, &[1])), r(1142, 2));
    assert_eq!(body(&h.run(1146, &[])), r(1148, 1));
}

#[test]
fn slate_entries() {
    let mut h = H::new();
    h.run(1194, &[2]);
    assert_eq!(h.last(), BadgeSlateTask::SetSlateStone { page: 2 });
    h.run(1197, &[1]);
    assert_eq!(h.last(), BadgeSlateTask::GiveUpSlateStone { page: 1 });
    // Decode failure, then page failure (1200 uses 2557 for both).
    assert_eq!(body(&h.run(1194, &[])), r(1196, 2549));
    assert_eq!(body(&h.run(1194, &[3])), r(1196, 2));
    assert_eq!(body(&h.run(1197, &[0])), r(1199, 2));
    assert_eq!(body(&h.run(1200, &[])), r(1202, 2557));
    assert_eq!(body(&h.run(1200, &[3])), r(1202, 2557));
    h.world.slate_pages = 1;
    assert_eq!(body(&h.run(1200, &[2])), r(1202, 2557));
    h.run(1200, &[1]);
    assert_eq!(h.last(), BadgeSlateTask::ChangeSlatePage { page: 1 });
    h.pc().zone = None;
    assert_eq!(body(&h.run(1194, &[1])), r(1196, 2));
    // Gates: T1 5, T2 7, disabled.
    h.world.slate_pages = 0;
    assert_eq!(body(&h.run(1191, &[])), r(1193, 2542));
    assert_eq!(body(&h.run(1197, &[])), r(1199, 2551));
    h.pc().t2_blocked = true;
    assert_eq!(body(&h.run(1200, &[])), r(1202, 7));
    h.world.arena = true;
    assert_eq!(body(&h.run(1194, &[])), r(1196, 5));
}
