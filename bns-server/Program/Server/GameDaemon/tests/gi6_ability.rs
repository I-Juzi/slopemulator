//! Doc 12c part 3: ability modification, warehouse take and transform.
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
            item_growth_enabled: true,
            account_inventory_enabled: true,
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

fn pairs(p: &[(u16, u16)]) -> Vec<u8> {
    let mut v = (p.len() as u16).to_le_bytes().to_vec();
    for (a, b) in p {
        v.extend(a.to_le_bytes());
        v.extend(b.to_le_bytes());
    }
    v
}

#[test]
fn reveal_attach_enchant() {
    let mut h = H::new();
    let ok = cat(&[&[1, 2, 0, 3, 0, 4], &pairs(&[(5, 6)])]);
    assert!(h.run(1398, &ok).replies.is_empty());
    assert_eq!(
        h.last(),
        ItemTaskKind::Ability(AbilityTask::RevealItemAbility {
            selector: 1,
            a: 2,
            b: 3,
            c: 4,
            pairs: vec![(5, 6)]
        })
    );
    assert_eq!(replies(&h.run(1398, &[1, 2])), r(1399, 565));
    assert_eq!(
        replies(&h.run(1398, &[1, 2, 0, 3, 0, 4, 16, 0])),
        r(1399, 569)
    );
    // Wrong PC kind: assert, no reply.
    h.pc().native_kind = 2;
    assert!(h.run(1398, &ok).replies.is_empty());
    h.pc().native_kind = 1;
    h.services.trade.item_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1398, &ok)), r(1399, 1));
    assert_eq!(replies(&h.run(1400, &[1, 2, 0, 3, 0, 0, 0])), r(1401, 1));
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1400, &[1, 2, 0, 3, 0, 0, 0])), r(1401, 2));
    h.pc().zone = Some(9);
    h.services.trade.item_queue_result = Gate::Pass;
    assert_eq!(replies(&h.run(1400, &[1, 2, 0, 3, 0, 16, 0])), r(1401, 580));
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(1398, &ok)), r(1399, 575));
    assert_eq!(replies(&h.run(1400, &ok)), r(1401, 585));
    // 1430/1433 have no U0 gate.
    let q = 7u64.to_le_bytes();
    assert_eq!(
        replies(&h.run(1430, &cat(&[&[1, 0, 0], &q]))),
        r(1432, 2563)
    );
    assert_eq!(replies(&h.run(1430, &[1, 1, 0])), r(1432, 2));
    h.run(1430, &cat(&[&[1, 3, 0], &q]));
    assert_eq!(
        h.last(),
        ItemTaskKind::Ability(AbilityTask::EnchantToItem {
            selector: 1,
            slot: 3,
            value: 7
        })
    );
    h.run(1433, &[4, 0, 0]);
    assert_eq!(
        h.last(),
        ItemTaskKind::Ability(AbilityTask::ResetEnchantItemInfo {
            selector: 4,
            slot: 0
        })
    );
    assert_eq!(replies(&h.run(1433, &[4])), r(1435, 2));
}

#[test]
fn polish_rules() {
    let mut h = H::new();
    let tail = cat(&[&1u64.to_le_bytes(), &2u64.to_le_bytes()]);
    assert_eq!(replies(&h.run(1442, &[1, 0, 0, 4, 0])), r(1444, 3));
    assert_eq!(
        replies(&h.run(1442, &cat(&[&[1, 0, 0, 1, 0, 6], &tail]))),
        r(1444, 2595)
    );
    assert_eq!(
        replies(&h.run(1442, &cat(&[&[1, 0, 0, 2, 0, 3, 3], &tail]))),
        r(1444, 2595)
    );
    assert_eq!(replies(&h.run(1442, &[1, 0, 0, 1, 0, 3])), r(1444, 2));
    h.run(1442, &cat(&[&[1, 9, 0, 2, 0, 5, 1], &tail]));
    assert_eq!(
        h.last(),
        ItemTaskKind::Ability(AbilityTask::PolishToItem {
            selector: 1,
            slot: 9,
            list: vec![5, 1],
            first: 1,
            second: 2
        })
    );
}

#[test]
fn growth_awakening_feeding_fusion() {
    let mut h = H::new();
    let pre = |c: u8, a: u16, b: u16, x: u16| {
        cat(&[&[c], &a.to_le_bytes(), &b.to_le_bytes(), &x.to_le_bytes()])
    };
    assert_eq!(
        replies(&h.run(1402, &cat(&[&pre(2, 1, 1, 3), &pairs(&[])]))),
        r(1404, 2)
    );
    assert_eq!(
        replies(&h.run(1402, &cat(&[&pre(2, 1, 2, 3), &pairs(&[(1, 9)])]))),
        r(1404, 2)
    );
    assert!(h
        .run(1402, &cat(&[&pre(3, 1, 1, 1), &pairs(&[(1, 9)])]))
        .replies
        .is_empty());
    assert_eq!(
        replies(&h.run(1402, &cat(&[&pre(3, 1, 1, 1), &[16, 0]]))),
        r(1404, 598)
    );

    let two = |c: u8, a: u16, b: u16, g1: &[(u16, u16)], g2: &[(u16, u16)]| {
        cat(&[
            &[c],
            &a.to_le_bytes(),
            &b.to_le_bytes(),
            &pairs(g1),
            &pairs(g2),
        ])
    };
    h.run(1408, &two(2, 1, 2, &[(3, 4)], &[]));
    assert_eq!(
        h.last(),
        ItemTaskKind::Ability(AbilityTask::TwoGroup {
            feeding: true,
            container: 2,
            targets: [1, 2],
            first: vec![(3, 4)],
            second: vec![]
        })
    );
    assert_eq!(
        replies(&h.run(1405, &two(2, 1, 2, &[], &[(1, 0)]))),
        r(1407, 2)
    );
    assert_eq!(
        replies(&h.run(1405, &cat(&[&pre(0, 1, 2, 0), &[16, 0]]))),
        r(1407, 619)
    );
    assert_eq!(
        replies(&h.run(1408, &cat(&[&pre(0, 1, 2, 16)]))),
        r(1410, 635)
    );
    h.world.item_growth_enabled = false;
    assert_eq!(replies(&h.run(1402, &[])), r(1404, 586));
    assert_eq!(replies(&h.run(1405, &[])), r(1407, 586));
    assert_eq!(replies(&h.run(1408, &[])), r(1410, 626));

    assert_eq!(replies(&h.run(1445, &pairs(&[]))), r(1447, 2598));
    assert_eq!(replies(&h.run(1445, &pairs(&[(1, 2)]))), r(1447, 2598));
    h.run(1445, &cat(&[&pairs(&[(1, 2)]), &[7, 0]]));
    assert_eq!(
        h.last(),
        ItemTaskKind::Ability(AbilityTask::ProcessItemFusion {
            pairs: vec![(1, 2)],
            operand: 7
        })
    );
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1445, &[])), r(1447, 2));
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(1445, &[])), r(1447, 2612));
}

#[test]
fn warehouse_and_transform() {
    let mut h = H::new();
    let w = |v: u32| cat(&[&[0; 16], &v.to_le_bytes()]);
    assert_eq!(replies(&h.run(1411, &w(1))), r(1413, 1964));
    assert_eq!(replies(&h.run(1411, &w(999))), r(1413, 1964));
    assert!(unsupported(&h.run(1411, &w(0))));
    assert!(unsupported(&h.run(1411, &w(1000))));
    h.world.arena = true;
    assert_eq!(replies(&h.run(1411, &w(5))), r(1413, 5));
    h.world.arena = false;
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(1411, &w(5))), r(1413, 1));
    assert_eq!(replies(&h.run(1581, &[])), rq(1583, 2012, 0));
    h.pc().user_present = true;
    h.world.account_inventory_enabled = false;
    assert_eq!(replies(&h.run(1411, &w(5))), r(1413, 2));

    let key = 0x55u64.to_le_bytes();
    assert_eq!(replies(&h.run(1581, &[1])), rq(1583, 1980, 0));
    assert_eq!(
        replies(&h.run(1581, &cat(&[&key, &[17, 0]]))),
        rq(1583, 1987, 0x55)
    );
    assert_eq!(
        replies(&h.run(1581, &cat(&[&key, &[0; 6], &[17, 0]]))),
        rq(1583, 2002, 0x55)
    );
    let groups = [0u8; 8];
    assert_eq!(
        replies(&h.run(1581, &cat(&[&key, &groups, &[0], &[0; 4], &[9, 0]]))),
        rq(1583, 1980, 0x55)
    );
    assert!(unsupported(
        &h.run(1581, &cat(&[&key, &groups, &[1], &[0; 4], &[0, 0]]))
    ));
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1581, &key)), rq(1583, 2, 0));
}
