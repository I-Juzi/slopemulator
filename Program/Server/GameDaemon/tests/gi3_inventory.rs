//! Doc 12b part 2: closet, inventory, repair, decompose, gem and seal entries.
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
            routing: 0x55,
            zone: Some(9),
            native_kind: 1,
            user_present: true,
            ..Pc::default()
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
            world: World {
                unseal_enabled: true,
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
    fn last(&self) -> InventoryTask {
        match &self.services.trade.item_tasks.last().unwrap().kind {
            ItemTaskKind::Inventory(t) => t.clone(),
            other => panic!("{other:?}"),
        }
    }
    fn queued(&self) -> usize {
        self.services.trade.item_tasks.len()
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
fn cat(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
}

#[test]
fn closet_entries() {
    let mut h = H::new();
    assert_eq!(body(&h.run(934, &[1])), r(936, 2168));
    assert_eq!(body(&h.run(934, &s(0xFFFF))), r(936, 2));
    h.run(934, &s(4));
    assert_eq!(h.last(), InventoryTask::PutIntoCloset { slot: 4 });
    assert_eq!(body(&h.run(937, &[1, 0, 0])), r(939, 2173));
    assert_eq!(
        body(&h.run(937, &cat(&[&0i32.to_le_bytes(), &s(1)]))),
        r(939, 2)
    );
    assert_eq!(
        body(&h.run(937, &cat(&[&3i32.to_le_bytes(), &s(0)]))),
        r(939, 2)
    );
    h.run(937, &cat(&[&3i32.to_le_bytes(), &s(1)]));
    assert_eq!(
        h.last(),
        InventoryTask::TakeFromCloset {
            selector: 3,
            dst_slot: 1
        }
    );
    // 1118: count 1..16, initializer failure → 2.
    assert_eq!(body(&h.run(1118, &s(0))), r(1120, 2));
    assert_eq!(body(&h.run(1118, &s(17))), r(1120, 2));
    assert_eq!(body(&h.run(1118, &cat(&[&s(1), &[1, 0]]))), r(1120, 2));
    h.run(
        1118,
        &cat(&[&s(2), &(-1i32).to_le_bytes(), &(-1i32).to_le_bytes()]),
    );
    assert_eq!(
        h.last(),
        InventoryTask::SetClosetItemUsed {
            selectors: vec![-1, -1]
        }
    );
    // Lists have no PC kind test.
    h.pc().native_kind = 2;
    h.run(1121, &[]);
    assert_eq!(h.last(), InventoryTask::CollectingList);
    h.run(1125, &[]);
    assert_eq!(h.last(), InventoryTask::FishingList);
    let out = h.run(934, &s(4));
    assert_eq!(body(&out), r(936, 2));
    assert!(!out.logs.is_empty());
    h.pc().native_kind = 1;
    h.pc().zone = None;
    assert_eq!(body(&h.run(1121, &[])), r(1123, 2));
    assert_eq!(body(&h.run(1125, &[])), r(1127, 2));
    assert_eq!(
        body(&h.run(937, &cat(&[&3i32.to_le_bytes(), &s(1)]))),
        r(939, 2)
    );
}

#[test]
fn complete_collecting_failures_are_boundaries() {
    let mut h = H::new();
    let entry = cat(&[&[1, 2], &s(3), &[4], &5u32.to_le_bytes()]);
    for bad in [
        cat(&[&0i32.to_le_bytes(), &s(0)]),
        cat(&[&1i32.to_le_bytes(), &s(9)]),
    ] {
        let out = h.run(1428, &bad);
        assert!(
            out.replies.is_empty() && matches!(out.status, Status::Unsupported(_)),
            "{out:?}"
        );
    }
    h.run(1428, &cat(&[&7i32.to_le_bytes(), &s(1), &entry]));
    assert_eq!(
        h.last(),
        InventoryTask::CompleteClosetCollecting {
            collection: 7,
            entries: vec![(1, 2, 3, 4, 5)]
        }
    );
}

#[test]
fn inventory_list_dispose_move() {
    let mut h = H::new();
    h.run(1331, &[]);
    assert_eq!(h.last(), InventoryTask::InventoryList);
    // Wrong kind logs without a reply and posts nothing.
    h.pc().native_kind = 2;
    let n = h.queued();
    let out = h.run(1331, &[]);
    assert!(out.replies.is_empty() && !out.logs.is_empty());
    assert_eq!(h.queued(), n);
    h.pc().native_kind = 1;
    assert!(matches!(
        h.run(1334, &[1, 0]).status,
        Status::Unsupported(_)
    ));
    h.run(1334, &cat(&[&s(5), &s(0xFFFF)]));
    assert_eq!(h.last(), InventoryTask::Dispose { slot: 5, count: -1 });
    h.services.trade.item_queue_result = Gate::Fail;
    assert_eq!(
        body(&h.run(1334, &cat(&[&s(5), &s(1)]))),
        (1335, vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    );
    let mv = cat(&[&[2], &s(3), &s(4), &[2], &s(6), &9u64.to_le_bytes()]);
    assert_eq!(body(&h.run(1336, &mv)), r(1337, 1));
    h.services.trade.item_queue_result = Gate::Pass;
    h.run(1336, &mv);
    assert_eq!(
        h.last(),
        InventoryTask::Move {
            src_selector: 2,
            src_slot: 3,
            count: 4,
            dst_selector: 2,
            dst_slot: 6,
            token: 9
        }
    );
}

#[test]
fn move_list_and_expand() {
    let mut h = H::new();
    let pre = cat(&[&7u64.to_le_bytes(), &[2, 3]]);
    assert_eq!(body(&h.run(1338, &pre[..5])), r(1337, 292));
    assert_eq!(body(&h.run(1338, &cat(&[&pre, &s(97)]))), r(1337, 293));
    assert_eq!(
        body(&h.run(1338, &cat(&[&pre, &s(0), &s(0xFFFF)]))),
        r(1337, 293)
    );
    assert_eq!(
        body(&h.run(1338, &cat(&[&pre, &s(1), &s(1)]))),
        r(1337, 292)
    );
    h.run(1338, &cat(&[&pre, &s(1), &s(1), &s(2), &s(3), &s(0)]));
    assert_eq!(
        h.last(),
        InventoryTask::MoveList {
            token: 7,
            src_selector: 2,
            dst_selector: 3,
            first: vec![(1, 2, 3)],
            second: vec![]
        }
    );
    h.pc().native_kind = 2;
    assert_eq!(body(&h.run(1338, &cat(&[&pre, &s(0), &s(0)]))), r(1337, 2));
    h.pc().native_kind = 1;
    // 1328: prefix → 1330/1; count → 1344/349; pair → 1344/348.
    let pre = cat(&[&[3], &5u64.to_le_bytes()]);
    assert_eq!(body(&h.run(1328, &[3])), r(1330, 1));
    assert_eq!(body(&h.run(1328, &cat(&[&pre, &s(16)]))), r(1344, 349));
    assert_eq!(body(&h.run(1328, &cat(&[&pre, &s(1), &[1]]))), r(1344, 348));
    h.run(1328, &cat(&[&pre, &s(1), &s(8), &s(9)]));
    assert_eq!(
        h.last(),
        InventoryTask::ExpandInventory {
            selector: 3,
            npc: 5,
            pairs: vec![(8, 9)]
        }
    );
}

#[test]
fn repair_and_decompose() {
    let mut h = H::new();
    let npc = 5u64.to_le_bytes();
    assert_eq!(body(&h.run(1385, &[1])), r(1386, 448));
    assert_eq!(body(&h.run(1385, &cat(&[&npc, &s(97)]))), r(1386, 449));
    assert_eq!(body(&h.run(1385, &cat(&[&npc, &s(1), &[1]]))), r(1386, 448));
    h.run(1385, &cat(&[&npc, &s(2), &[1], &s(3), &[2], &s(3)]));
    assert_eq!(
        h.last(),
        InventoryTask::Repair {
            npc: 5,
            entries: vec![(1, 3), (2, 3)]
        }
    );
    assert!(matches!(h.run(1387, &npc).status, Status::Unsupported(_)));
    h.run(1387, &cat(&[&npc, &s(4)]));
    assert_eq!(
        h.last(),
        InventoryTask::RepairWithCampfire {
            campfire: 5,
            material_slot: 4
        }
    );
    // Decompose.
    let pre = |c: u8, m: u8| cat(&[&[c], &s(1), &[m]]);
    assert_eq!(body(&h.run(1395, &[2, 1])), r(1397, 480));
    assert_eq!(body(&h.run(1395, &pre(4, 0))), r(1397, 482));
    assert_eq!(body(&h.run(1395, &pre(2, 7))), r(1397, 482));
    assert_eq!(
        body(&h.run(1395, &cat(&[&pre(8, 6), &s(17)]))),
        r(1397, 482)
    );
    assert_eq!(
        body(&h.run(1395, &cat(&[&pre(8, 6), &s(0), &[1]]))),
        r(1397, 480)
    );
    assert_eq!(
        body(&h.run(1395, &cat(&[&pre(8, 6), &s(0), &[1, 2], &s(9)]))),
        r(1397, 483)
    );
    assert_eq!(
        body(&h.run(1395, &cat(&[&pre(8, 6), &s(0), &[1, 2], &s(1), &[1]]))),
        r(1397, 483)
    );
    h.run(
        1395,
        &cat(&[
            &pre(6, 0),
            &s(1),
            &s(4),
            &s(5),
            &[1, 2],
            &s(1),
            &[3],
            &9u64.to_le_bytes(),
        ]),
    );
    assert_eq!(
        h.last(),
        InventoryTask::Decompose {
            container: 6,
            slot: 1,
            mode: 0,
            pairs: vec![(4, 5)],
            q1: 1,
            q2: 2,
            auxiliary: vec![(3, 9)]
        }
    );
    h.pc().zone = None;
    assert_eq!(
        body(&h.run(1395, &cat(&[&pre(2, 0), &s(0), &[0, 0], &s(0)]))),
        r(1397, 1)
    );
    assert_eq!(body(&h.run(1385, &cat(&[&npc, &s(0)]))), r(1386, 2));
    // U0 failures come before decode.
    h.pc().party_user_blocked = true;
    assert_eq!(body(&h.run(1385, &[])), r(1386, 455));
    assert_eq!(body(&h.run(1387, &[])), r(1388, 464));
    assert_eq!(body(&h.run(1395, &[])), r(1397, 489));
    h.pc().party_user_blocked = false;
    h.pc().user_present = false;
    assert_eq!(body(&h.run(1385, &[])), r(1386, 455));
}

#[test]
fn gem_entries() {
    let mut h = H::new();
    let e = |i: i8, k: u8| cat(&[&[1], &s(2), &s(3), &[i as u8, k]]);
    for bad in [
        s(4),
        cat(&[&s(1), &e(8, 0)]),
        cat(&[&s(1), &e(2, 1)]),
        cat(&[&s(1), &e(-1, 0)]),
    ] {
        let out = h.run(1364, &bad);
        assert!(out.replies.is_empty() && matches!(out.status, Status::Unsupported(_)));
    }
    h.run(1364, &cat(&[&s(2), &e(7, 0), &e(1, 5)]));
    assert_eq!(
        h.last(),
        InventoryTask::AttachGem {
            entries: vec![(1, 2, 3, 7, 0), (1, 2, 3, 1, 5)]
        }
    );
    h.run(1364, &s(0));
    assert_eq!(h.last(), InventoryTask::AttachGem { entries: vec![] });
    // Detach: missing socket kind decodes as 0.
    assert_eq!(body(&h.run(1367, &[1, 2, 0, 8])), r(1368, 382));
    assert_eq!(body(&h.run(1367, &[1, 2, 0, 2, 1])), r(1368, 382));
    h.run(1367, &[1, 2, 0, 7]);
    assert_eq!(
        h.last(),
        InventoryTask::DetachGem {
            container: 1,
            slot: 2,
            index: 7,
            socket_kind: 0
        }
    );
    // Open.
    assert_eq!(body(&h.run(1369, &[1])), r(1371, 386));
    assert_eq!(
        body(&h.run(1369, &cat(&[&[1], &s(2), &s(17)]))),
        r(1371, 387)
    );
    assert_eq!(
        body(&h.run(1369, &cat(&[&[1], &s(2), &s(1)]))),
        r(1371, 386)
    );
    // Expand / add.
    assert!(matches!(
        h.run(1372, &[1, 2, 0]).status,
        Status::Unsupported(_)
    ));
    h.run(1372, &cat(&[&[1], &s(2), &s(3)]));
    assert_eq!(
        h.last(),
        InventoryTask::ExpandGemSlot {
            container: 1,
            slot: 2,
            operand: 3
        }
    );
    h.run(1375, &cat(&[&[1], &s(2), &s(3)]));
    assert_eq!(
        h.last(),
        InventoryTask::AddGemSlot {
            container: 1,
            slot: 2,
            operand: 3
        }
    );
    // Delete: byte B ≤ 3; byte A unbounded.
    assert_eq!(body(&h.run(1378, &[1, 2, 0, 0xFF])), r(1380, 408));
    assert_eq!(body(&h.run(1378, &[1, 2, 0, 0xFF, 4])), r(1380, 411));
    assert_eq!(
        body(&h.run(1378, &cat(&[&[1, 2, 0, 0xFF, 3], &s(17)]))),
        r(1380, 411)
    );
    h.run(1378, &cat(&[&[1, 2, 0, 0xFF, 3], &s(0)]));
    assert_eq!(
        h.last(),
        InventoryTask::DeleteGemSlot {
            container: 1,
            slot: 2,
            a: 0xFF,
            b: 3,
            pairs: vec![]
        }
    );
    // U0 reasons.
    h.pc().user_present = false;
    assert!(matches!(h.run(1364, &s(0)).status, Status::Unsupported(_)));
    assert_eq!(body(&h.run(1367, &[])), r(1368, 385));
    assert_eq!(body(&h.run(1369, &[])), r(1371, 392));
    assert_eq!(body(&h.run(1372, &[])), r(1374, 400));
    assert_eq!(body(&h.run(1375, &[])), r(1377, 406));
    assert_eq!(body(&h.run(1378, &[])), r(1380, 414));
    h.pc().user_present = true;
    h.pc().zone = None;
    assert_eq!(body(&h.run(1372, &cat(&[&[1], &s(2), &s(3)]))), r(1374, 1));
}

#[test]
fn identify_seal_unseal() {
    let mut h = H::new();
    // 1381 has no U0 gate; T2 → 7.
    h.pc().user_present = false;
    h.run(1381, &cat(&[&s(1), &s(2)]));
    assert_eq!(
        h.last(),
        InventoryTask::Identify {
            first: 1,
            second: 2
        }
    );
    h.pc().t2_blocked = true;
    assert_eq!(body(&h.run(1381, &[])), r(1382, 7));
    h.pc().t2_blocked = false;
    assert_eq!(body(&h.run(1392, &[])), r(1394, 478));
    assert_eq!(body(&h.run(1383, &[])), r(1384, 446));
    h.pc().user_present = true;
    // Seal: count 1..16.
    assert_eq!(body(&h.run(1392, &[1])), r(1394, 473));
    assert_eq!(body(&h.run(1392, &cat(&[&s(1), &s(0)]))), r(1394, 475));
    assert_eq!(body(&h.run(1392, &cat(&[&s(1), &s(17)]))), r(1394, 475));
    assert_eq!(
        body(&h.run(1392, &cat(&[&s(1), &s(1), &[1]]))),
        r(1394, 473)
    );
    h.run(1392, &cat(&[&s(1), &s(1), &s(2), &s(3)]));
    assert_eq!(
        h.last(),
        InventoryTask::Seal {
            target: 1,
            pairs: vec![(2, 3)]
        }
    );
    // Unseal.
    assert_eq!(body(&h.run(1383, &s(1))), r(1384, 441));
    assert_eq!(body(&h.run(1383, &cat(&[&s(1), &[4]]))), r(1384, 443));
    assert_eq!(
        body(&h.run(1383, &cat(&[&s(1), &[3], &s(17)]))),
        r(1384, 443)
    );
    h.run(1383, &cat(&[&s(1), &[3], &s(0)]));
    assert_eq!(
        h.last(),
        InventoryTask::Unseal {
            target: 1,
            mode: 3,
            pairs: vec![]
        }
    );
    // Feature byte zero: log only, no decode or reply, even for U0 failure.
    h.world.unseal_enabled = false;
    h.pc().user_present = false;
    let out = h.run(1383, &[]);
    assert!(out.replies.is_empty() && !out.logs.is_empty());
}
