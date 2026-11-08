//! Doc 12b part 1: equipment, craft and item lock entry functions.
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
            short_96: -2,
            zone: Some(9),
            native_kind: 1,
            user_present: true,
            ..Pc::default()
        };
        let world = World {
            item_lock_enabled: true,
            utc_seconds: 1000,
            arena_value: 7,
            ..World::default()
        };
        let mut services = MemoryServices::default();
        services.trade.item_queue_result = Gate::Pass;
        services.trade.recipe_exists = Some(true);
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
fn body(out: &Outcome) -> (u16, Vec<u8>) {
    assert_eq!(out.replies.len(), 1, "{out:?}");
    (out.replies[0].command, out.replies[0].body.clone())
}
fn r(command: u16, reason: u16) -> (u16, Vec<u8>) {
    (command, reason.to_le_bytes().to_vec())
}
fn mv(a: u8, b: u16, c: u8, d: u16) -> Vec<u8> {
    [
        vec![a],
        b.to_le_bytes().to_vec(),
        vec![c],
        d.to_le_bytes().to_vec(),
    ]
    .concat()
}

#[test]
fn manufacture_stub() {
    let mut h = H::new();
    assert_eq!(h.run(1325, &[9; 4]), Outcome::default());
}

#[test]
fn equip_and_unequip_entries() {
    let mut h = H::new();
    assert_eq!(body(&h.run(1354, &mv(2, 1, 2, 3))), r(1355, 1));
    assert_eq!(body(&h.run(1354, &mv(3, 1, 1, 3))), r(1355, 1));
    assert!(h.run(1354, &mv(9, 4, 11, 2)).replies.is_empty());
    assert_eq!(
        h.last(),
        ItemTaskKind::Equip {
            src_selector: 9,
            src_slot: 4,
            dst_selector: 11,
            dst_slot: 2,
            operation: 1
        }
    );
    assert_eq!(body(&h.run(1359, &mv(1, 1, 1, 3))), r(1360, 1));
    h.run(1359, &mv(1, 2, 9, 3));
    assert!(matches!(
        h.last(),
        ItemTaskKind::Unequip { operation: 0, .. }
    ));
    // Zone / queue failure → reason 1. Wrong PC kind logs only.
    h.pc().zone = None;
    assert_eq!(body(&h.run(1354, &mv(1, 1, 1, 1))), r(1355, 1));
    h.pc().zone = Some(9);
    h.pc().native_kind = 2;
    let out = h.run(1359, &mv(1, 1, 2, 1));
    assert!(out.replies.is_empty() && !out.logs.is_empty());
    h.pc().native_kind = 1;
    h.services.trade.item_queue_result = Gate::Fail;
    assert_eq!(body(&h.run(1354, &mv(1, 1, 1, 1))), r(1355, 1));
}

#[test]
fn equip_lists() {
    let mut h = H::new();
    // Count 16 and negative count → 2. Zero passes.
    assert_eq!(body(&h.run(1356, &16u16.to_le_bytes())), r(1355, 2));
    assert_eq!(body(&h.run(1361, &(-1i16).to_le_bytes())), r(1360, 2));
    assert_eq!(body(&h.run(1356, &[1, 0, 2])), r(1355, 2));
    h.run(1356, &0u16.to_le_bytes());
    assert_eq!(h.last(), ItemTaskKind::EquipList { entries: vec![] });
    h.run(1361, &[1, 0, 4, 0, 9, 0]);
    assert_eq!(
        h.last(),
        ItemTaskKind::UnequipList {
            entries: vec![(4, 9)]
        }
    );
}

#[test]
fn register_take_and_union() {
    let mut h = H::new();
    let fail = (1517, vec![2, 0, 0, 0, 0, 0]);
    let reg =
        |recipe: i32, n: i16| [recipe.to_le_bytes().to_vec(), n.to_le_bytes().to_vec()].concat();
    assert_eq!(body(&h.run(1515, &reg(0, 0))), fail);
    assert_eq!(body(&h.run(1515, &reg(5, 10))), fail);
    assert_eq!(body(&h.run(1515, &reg(-5, 0))), fail);
    h.run(1515, &[reg(5, 1), vec![3, 0, 4, 0]].concat());
    assert_eq!(
        h.last(),
        ItemTaskKind::RegisterCraft {
            recipe: 5,
            count: 1,
            entries: vec![(3, 4)]
        }
    );
    h.services.trade.recipe_exists = Some(false);
    assert_eq!(body(&h.run(1515, &reg(5, 0))), fail);
    h.pc().t2_blocked = true;
    assert_eq!(body(&h.run(1515, &reg(5, 0))), fail);
    // Join/leave: T2 → 7 with selector 0, before decode.
    assert_eq!(body(&h.run(1524, &[3])), (1526, vec![7, 0, 0]));
    assert_eq!(body(&h.run(1527, &[3])), (1529, vec![7, 0, 0]));
    h.pc().t2_blocked = false;
    let join = |s: u8, t: u64| [vec![s], t.to_le_bytes().to_vec()].concat();
    assert_eq!(body(&h.run(1524, &join(7, 1))), (1526, vec![2, 0, 7]));
    assert_eq!(body(&h.run(1524, &join(15, 0))), (1526, vec![2, 0, 15]));
    h.run(1524, &join(15, 1));
    assert_eq!(
        h.last(),
        ItemTaskKind::JoinCraftUnion {
            selector: 15,
            target: 1
        }
    );
    h.run(1527, &[7]);
    assert_eq!(h.last(), ItemTaskKind::LeaveCraftUnion { selector: 7 });
    assert_eq!(body(&h.run(1527, &[0x80])), (1529, vec![2, 0, 0x80]));
    // Take: short body → ID 0 → 1520 reason 2, ID 0.
    assert_eq!(
        body(&h.run(1518, &[1])),
        (1520, [vec![2, 0], vec![0; 8]].concat())
    );
    h.run(1518, &9u64.to_le_bytes());
    assert_eq!(h.last(), ItemTaskKind::TakeCraft { craft_id: 9 });
}

#[test]
fn abort_craft() {
    let mut h = H::new();
    let rec = |id, state, start, end| {
        Some(CraftRecord {
            id,
            state,
            start,
            end,
        })
    };
    h.services.trade.crafts = Some(vec![
        None,
        rec(5, 3, 1001, 2000),
        rec(5, 3, 900, 1000),
        rec(5, 3, 1000, 1001),
        rec(6, 2, 0, 9999),
    ]);
    let fail = |id: u64| (1523, [vec![3, 0], id.to_le_bytes().to_vec()].concat());
    assert_eq!(body(&h.run(1521, &0u64.to_le_bytes())), fail(0));
    assert_eq!(body(&h.run(1521, &6u64.to_le_bytes())), fail(6));
    let out = h.run(1521, &5u64.to_le_bytes());
    assert!(out.replies.is_empty());
    assert_eq!(
        out.daemon_requests,
        vec![DaemonRequest::CraftAbort {
            routing: 0x55,
            short_96: -2,
            restriction: 7,
            object_id: PC_ID,
            db_id: 0x7700,
            craft_id: 5
        }]
    );
    let crafts = h.services.trade.crafts.as_ref().unwrap();
    assert_eq!(crafts[3].unwrap().state, 6);
    assert_eq!(crafts[1].unwrap().state, 3);
    // Now state 6: the first match (state 3, bad time) continues, then state 6 stops.
    assert_eq!(body(&h.run(1521, &5u64.to_le_bytes())), fail(5));
}

#[test]
fn lock_unlock_expiry_used() {
    let mut h = H::new();
    let cs = [1u8, 3, 0];
    h.run(1414, &cs);
    assert_eq!(
        h.last(),
        ItemTaskKind::Lock {
            container: 1,
            slot: 3
        }
    );
    assert_eq!(body(&h.run(1414, &[1])), r(1415, 2));
    h.pc().zone = None;
    assert_eq!(body(&h.run(1414, &cs)), r(1415, 1));
    assert_eq!(body(&h.run(1418, &cs)), r(1419, 1));
    assert_eq!(body(&h.run(1416, &cs)), (1417, vec![2, 0, 0, 0, 0, 0]));
    assert_eq!(body(&h.run(1426, &[1, 0, 1, 1, 0])), r(1427, 2));
    h.pc().zone = Some(9);
    h.world.item_lock_enabled = false;
    assert_eq!(body(&h.run(1418, &cs)), r(1419, 2));
    assert_eq!(body(&h.run(1416, &cs)), (1417, vec![2, 0, 0, 0, 0, 0]));
    // 1426 does not use the lock feature byte. Count 1..16.
    assert_eq!(body(&h.run(1426, &[0, 0])), r(1427, 2));
    assert_eq!(body(&h.run(1426, &[17, 0])), r(1427, 2));
    h.run(1426, &[2, 0, 1, 1, 0, 1, 1, 0]);
    assert_eq!(
        h.last(),
        ItemTaskKind::SetUsed {
            entries: vec![(1, 1), (1, 1)]
        }
    );
}
