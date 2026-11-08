//! Doc 12c part 2: ability, appearance, exchange, dress, item use and pouches.
use gamedaemon::handlers::*;

struct H {
    session: Session,
    world: World,
    services: MemoryServices,
}
impl H {
    fn new() -> Self {
        let pc = Pc {
            object_id: (1 << 48) | 0x11,
            db_id: 0x7700,
            zone: Some(9),
            native_kind: 1,
            user_present: true,
            ..Pc::default()
        };
        let world = World {
            ability_passing_enabled: true,
            weapon_appearance_enabled: true,
            custom_dress_enabled: true,
            ..World::default()
        };
        let mut services = MemoryServices::default();
        services.trade.item_queue_result = Gate::Pass;
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
    fn last(&self) -> ItemTaskKind {
        self.services.trade.item_tasks.last().unwrap().kind.clone()
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
fn rq(command: u16, reason: u16, q: u64) -> Vec<(u16, Vec<u8>)> {
    vec![(
        command,
        [reason.to_le_bytes().to_vec(), q.to_le_bytes().to_vec()].concat(),
    )]
}
fn unsupported(out: &Outcome) -> bool {
    matches!(out.status, Status::Unsupported(_))
}
fn cat(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
}

#[test]
fn passing_item_ability() {
    let mut h = H::new();
    let q = 0xABu64.to_le_bytes();
    let ok = cat(&[&q, &[1, 0, 2, 0, 3, 0], &[0, 0], &[1, 0, 4, 0, 5, 0]]);
    assert!(h.run(830, &ok).replies.is_empty());
    assert_eq!(
        h.last(),
        ItemTaskKind::Use(ItemUseTask::PassingItemAbility {
            prefix: 0xAB,
            groups: [vec![(2, 3)], vec![], vec![(4, 5)]]
        })
    );
    assert_eq!(replies(&h.run(830, &q[..4])), rq(832, 2013, 0));
    assert_eq!(
        replies(&h.run(830, &cat(&[&q, &[2, 0]]))),
        rq(832, 2017, 0xAB)
    );
    assert_eq!(
        replies(&h.run(830, &cat(&[&q, &[0, 0, 2, 0]]))),
        rq(832, 2021, 0xAB)
    );
    assert_eq!(
        replies(&h.run(830, &cat(&[&q, &[0, 0, 0, 0, 9, 0]]))),
        rq(832, 2023, 0xAB)
    );
    assert_eq!(
        replies(&h.run(830, &cat(&[&q, &[1, 0, 2]]))),
        rq(832, 2013, 0xAB)
    );
    h.services.trade.item_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(830, &ok)), rq(832, 2, 0xAB));
    h.pc().zone = None;
    assert_eq!(replies(&h.run(830, &ok)), rq(832, 2, 0));
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(830, &ok)), rq(832, 2030, 0));
    h.world.ability_passing_enabled = false;
    assert_eq!(replies(&h.run(830, &ok)), rq(832, 2013, 0));
    // 833 only logs.
    let out = h.run(833, &[]);
    assert!(out.replies.is_empty() && !out.logs.is_empty());
}

#[test]
fn appearance_exchange_dress() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(947, &[1, 0])), r(949, 2190));
    assert_eq!(replies(&h.run(947, &[1, 0, 0, 0, 0, 8, 0])), r(949, 2190));
    assert!(unsupported(&h.run(947, &[1, 0, 0, 0, 0, 0, 0])));
    h.world.arena = true;
    assert_eq!(replies(&h.run(947, &[])), r(949, 5));
    h.world.weapon_appearance_enabled = false;
    assert_eq!(replies(&h.run(947, &[])), r(949, 2189));

    let key = 5u64.to_le_bytes();
    assert_eq!(
        replies(&h.run(1010, &cat(&[&[4], &key, &[0, 0]]))),
        r(1012, 2293)
    );
    assert_eq!(
        replies(&h.run(1010, &cat(&[&[2], &key, &[5, 0]]))),
        r(1012, 2296)
    );
    assert!(unsupported(&h.run(1010, &cat(&[&[8], &key, &[0, 0]]))));
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(1010, &[])), r(1012, 2302));

    let dress = cat(&[&[3, 0], &[1, 0, 7, 8], &[1, 0, 1, 0, 2, 0]]);
    assert!(h.run(1050, &dress).replies.is_empty());
    assert_eq!(
        h.last(),
        ItemTaskKind::Use(ItemUseTask::MakeCustomDress {
            target: 3,
            first: vec![(7, 8)],
            pairs: vec![(1, 2)]
        })
    );
    assert_eq!(replies(&h.run(1050, &[3, 0, 9, 0])), r(1052, 2));
    assert_eq!(replies(&h.run(1050, &[3, 0, 0, 0, 17, 0])), r(1052, 2363));
    h.world.custom_dress_enabled = false;
    assert_eq!(replies(&h.run(1050, &dress)), r(1052, 2360));
}

#[test]
fn item_use_entries() {
    let mut h = H::new();
    let g = cat(&[
        &[1, 0],
        &2u32.to_le_bytes(),
        &3u64.to_le_bytes(),
        &[4, 0],
        &5u64.to_le_bytes(),
    ]);
    assert!(h.run(1233, &g).replies.is_empty());
    assert_eq!(
        h.last(),
        ItemTaskKind::Use(ItemUseTask::UseGrocery {
            a: 1,
            b: 2,
            c: 3,
            d: 4,
            e: 5
        })
    );
    assert!(unsupported(&h.run(1233, &g[..6])));
    h.run(1237, &[1, 0, 2, 0, 3, 0, 4, 0, 5, 0]);
    assert_eq!(
        h.last(),
        ItemTaskKind::Use(ItemUseTask::UseEquippedToGround {
            a: 1,
            position: [2, 3, 4],
            c: 5
        })
    );
    h.run(1242, &cat(&[&7u64.to_le_bytes(), &[1, 0, 2]]));
    assert_eq!(
        h.last(),
        ItemTaskKind::Use(ItemUseTask::UseFieldItem {
            target: 7,
            a: 1,
            b: 2
        })
    );
    for c in [1243, 1245, 1249] {
        assert!(unsupported(&h.run(c, &[0; 8])));
    }
    // 1247 wrong PC kind: assert then reason 1.
    h.pc().native_kind = 2;
    assert_eq!(replies(&h.run(1247, &9u64.to_le_bytes())), r(1248, 1));
    h.pc().native_kind = 1;
    h.run(1251, &9u64.to_le_bytes());
    assert_eq!(
        h.last(),
        ItemTaskKind::Use(ItemUseTask::PickupDeadBody { selector: 9 })
    );
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1236, &[0; 12])), r(1238, 1));
    assert_eq!(replies(&h.run(1234, &[0; 22])), r(1235, 1));
}

#[test]
fn pouch_entries() {
    let mut h = H::new();
    let pouch = (176u64 << 48) | 3;
    let p = pouch.to_le_bytes();
    h.run(1303, &p);
    assert_eq!(
        h.last(),
        ItemTaskKind::Use(ItemUseTask::ItemPouch { pouch, close: true })
    );
    h.services.trade.item_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1296, &p)), r(1298, 2));
    assert_eq!(replies(&h.run(1299, &p)), r(1301, 2));
    h.services.trade.item_queue_result = Gate::Pass;

    // 1306: decode 505, count 506, object kind → 507 then 1.
    assert_eq!(replies(&h.run(1306, &p)), r(1309, 505));
    assert_eq!(
        replies(&h.run(1306, &cat(&[&p, &[0, 17, 0]]))),
        r(1309, 506)
    );
    let bad = (5u64 << 48).to_le_bytes();
    let mut both = r(1309, 507);
    both.extend(r(1309, 1));
    assert_eq!(replies(&h.run(1306, &cat(&[&bad, &[0, 0, 0]]))), both);
    h.run(1306, &cat(&[&p, &[4, 1, 0, 9]]));
    assert_eq!(
        h.last(),
        ItemTaskKind::Use(ItemUseTask::PickupItem {
            pouch,
            selector: 4,
            entries: vec![9]
        })
    );

    let mut both = r(1313, 515);
    both.extend(r(1313, 1));
    assert_eq!(replies(&h.run(1311, &((32u64 << 48).to_le_bytes()))), both);

    // 1316: 16 entries → 524; wrong kind → 1309/507 then 1319/2.
    let name = [1u8, 0, 0x41, 0];
    assert_eq!(
        replies(&h.run(1316, &cat(&[&p, &name, &[16, 0]]))),
        r(1319, 524)
    );
    let mut both = r(1309, 507);
    both.extend(r(1319, 2));
    assert_eq!(replies(&h.run(1316, &cat(&[&bad, &name, &[0, 0]]))), both);
    assert!(h
        .run(1316, &cat(&[&p, &name, &[1, 0, 6]]))
        .replies
        .is_empty());

    assert_eq!(replies(&h.run(1321, &[1])), r(1323, 2));
    let mut both = r(1323, 536);
    both.extend(r(1323, 2));
    assert_eq!(replies(&h.run(1321, &bad)), both);
    h.run(1321, &p);
    assert_eq!(
        h.last(),
        ItemTaskKind::Use(ItemUseTask::CompleteDistribution { pouch })
    );
}
