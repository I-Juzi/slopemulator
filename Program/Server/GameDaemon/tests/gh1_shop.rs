//! Doc 22: shop and store entry functions.
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
        let mut services = MemoryServices {
            skill_queue_result: Gate::Pass,
            ..MemoryServices::default()
        };
        for check in [
            ShopCheck::MileageEnabled,
            ShopCheck::MileageWithinLimit { count: 1 },
            ShopCheck::UserNotFatigued { pc: PC_ID },
            ShopCheck::NoLeaveAction { pc: PC_ID },
            ShopCheck::ExpandRecord { store: 1, slot: 3 },
            ShopCheck::ExpandPaymentEnabled { store: 1, slot: 3 },
            ShopCheck::LiveServer,
            ShopCheck::CollectionEnabled,
            ShopCheck::CollectionGroup {
                group: 10,
                item: 20,
            },
            ShopCheck::CollectionItem { item: 20 },
            ShopCheck::CollectionGroupMatch {
                group: 10,
                item: 20,
            },
            ShopCheck::CollectionChargeItem { group: 10 },
        ] {
            services.shop.results.insert(check, Gate::Pass);
        }
        Self {
            session: Session {
                authenticated: true,
                pc: Some(pc),
                dispatch_permission: Some(true),
                ..Session::default()
            },
            world: World {
                account_inventory_enabled: true,
                ..World::default()
            },
            services,
        }
    }
    fn pc(&mut self) -> &mut Pc {
        self.session.pc.as_mut().unwrap()
    }
    fn set(&mut self, check: ShopCheck, result: Gate) {
        self.services.shop.results.insert(check, result);
    }
    fn run(&mut self, command: u16, body: &[u8]) -> Outcome {
        dispatch(
            &mut Context {
                session: &mut self.session,
                world: &self.world,
                services: &mut self.services,
            },
            command,
            body,
        )
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
    e(command, reason, &[])
}
fn e(command: u16, reason: u16, fields: &[u8]) -> Vec<(u16, Vec<u8>)> {
    let mut body = reason.to_le_bytes().to_vec();
    body.extend(fields);
    vec![(command, body)]
}
fn unsupported(out: Outcome) {
    assert!(matches!(out.status, Status::Unsupported(_)), "{out:?}");
    assert!(out.replies.is_empty() && out.changes.is_empty());
}
fn pairs(items: &[(u16, u16)]) -> Vec<u8> {
    let mut b = (items.len() as u16).to_le_bytes().to_vec();
    for (a, c) in items {
        b.extend(a.to_le_bytes());
        b.extend(c.to_le_bytes());
    }
    b
}
fn mileage() -> Vec<u8> {
    let mut b = vec![1, 0, 1, 0];
    b.extend(pairs(&[(3, 4)]));
    b.extend(pairs(&[(5, 6)]));
    b
}
fn sell(items: &[(u16, u16)]) -> Vec<u8> {
    let mut b = vec![2];
    b.extend(pairs(items));
    b
}
fn mysterious(items: &[(u16, u16)]) -> Vec<u8> {
    let mut b = 55u64.to_le_bytes().to_vec();
    b.push(7);
    b.extend(pairs(items));
    b
}
fn restore() -> Vec<u8> {
    let mut b = vec![0xFE, 2, 0, 3, 0];
    b.extend(44u64.to_le_bytes());
    b
}
fn random_prefix(bytes: &[u8]) -> Vec<u8> {
    let mut b = bytes.to_vec();
    b.extend(42i32.to_le_bytes());
    b.extend([8, 0]);
    b.extend(1f64.to_le_bytes());
    b
}
fn draw(payment: u8, items: &[(u16, u16)]) -> Vec<u8> {
    let mut b = random_prefix(&[1, payment, 5]);
    b.extend(pairs(items));
    b
}
fn expand(payment: u8, items: &[(u16, u16)]) -> Vec<u8> {
    let mut b = random_prefix(&[1, 3, payment]);
    b.extend(pairs(items));
    b
}
fn buy(items: &[(u16, u16)]) -> Vec<u8> {
    let mut b = vec![1, 5, 8];
    b.extend(pairs(items));
    b
}
fn collection(items: &[(i32, u64, u64)]) -> Vec<u8> {
    let mut b = vec![8, 0];
    b.extend(1f64.to_le_bytes());
    b.extend((items.len() as u16).to_le_bytes());
    for (goods, group, item) in items {
        b.extend(goods.to_le_bytes());
        b.extend([8, 0]);
        b.extend(1f64.to_le_bytes());
        b.extend(group.to_le_bytes());
        b.extend(item.to_le_bytes());
    }
    b
}

#[test]
fn all_commands_registered() {
    assert_eq!(
        SHOP_COMMANDS,
        &[80, 827, 940, 944, 1008, 1345, 1436, 1439, 1566, 1569, 1572, 1575, 1578, 1584]
    );
    for &command in SHOP_COMMANDS {
        let mut h = H::new();
        let out = h.run(command, &[]);
        assert!(!out.logs.contains(&"unknown command"), "{command}");
        h.session.authenticated = false;
        assert_eq!(h.run(command, &[]).logs, vec!["missing authenticated PC"]);
    }
}

#[test]
fn restore_vitality_only_logs() {
    let mut h = H::new();
    h.pc().zone = None;
    h.pc().native_kind = 9;
    let out = h.run(827, &[1, 2, 3]);
    assert_eq!(out.status, Status::Handled);
    assert!(out.replies.is_empty() && out.changes.is_empty() && h.services.skill_tasks.is_empty());
    assert_eq!(out.logs, vec!["vitality: deprecated packet"]);
}

#[test]
fn rebuy_failure_reason_is_one() {
    let mut h = H::new();
    h.pc().zone = None;
    assert_eq!(replies(&h.run(80, &[])), r(82, 1));
    h.pc().zone = Some(9);
    h.pc().native_kind = 2;
    assert_eq!(replies(&h.run(80, &[])), r(82, 1));
    h.pc().native_kind = 1;
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(80, &[])), r(82, 1));
}

#[test]
fn queued_task_fields_and_raw_request() {
    let cases = vec![
        (80, vec![], ShopTask::RebuyItemList),
        (
            944,
            77u64.to_le_bytes().to_vec(),
            ShopTask::RetrieveMysteriousStore { key: 77 },
        ),
        (
            1008,
            sell(&[(4, 0xFFFF)]),
            ShopTask::SellOldItem {
                depot: 2,
                items: vec![(4, 0xFFFF)],
            },
        ),
        (
            1345,
            mysterious(&[(0, 0xFFFF)]),
            ShopTask::MysteriousBuy {
                seller: 55,
                index: 7,
                items: vec![(0, 0xFFFF)],
            },
        ),
        (
            1436,
            restore(),
            ShopTask::RestoreEnchantFailedItem {
                depot: 0xFE,
                first: 2,
                second: 3,
                key: 44,
            },
        ),
        (
            1439,
            vec![0xFF, 0xFE, 0xFF],
            ShopTask::ResetEnchantRestoreInfo {
                depot: 0xFF,
                item: 0xFFFE,
            },
        ),
        (1566, vec![], ShopTask::RandomStoreInfo),
        (
            1569,
            draw(2, &[(0, 2)]),
            ShopTask::RandomStoreDraw {
                store: 1,
                payment: 2,
                draws: 5,
                items: vec![(0, 2)],
            },
        ),
        (
            1572,
            expand(2, &[(0, 0xFFFF)]),
            ShopTask::ExpandRandomStoreSlot {
                store: 1,
                slot: 3,
                payment: 2,
                items: vec![(0, 0xFFFF)],
            },
        ),
        (
            1575,
            buy(&[(0, 0)]),
            ShopTask::BuyRandomStoreItem {
                store: 1,
                draw: 5,
                slot: 8,
                items: vec![(0, 0)],
            },
        ),
        (
            1578,
            vec![2, 127],
            ShopTask::TakeRandomStoreDrawReward {
                store: 2,
                reward: 127,
            },
        ),
    ];
    for (command, mut body, expected) in cases {
        let mut h = H::new();
        body.extend([0xEE, 0xFF]);
        let out = h.run(command, &body);
        assert_eq!(out.status, Status::Handled, "{command}");
        assert!(out.replies.is_empty());
        assert_eq!(
            h.last(),
            SkillTask {
                command,
                caster: PC_ID,
                summon: false,
                zone: 9,
                kind: SkillTaskKind::Shop(expected)
            }
        );
        assert_eq!(
            out.changes,
            vec![
                StateChange::Queued(command),
                StateChange::QueuedRequest { command, body }
            ]
        );
    }
}

#[test]
fn list_and_retrieve_no_native_kind_gate() {
    let mut h = H::new();
    h.pc().native_kind = 9;
    for command in [944, 1566] {
        assert!(h.run(command, &[]).replies.is_empty());
    }
    assert_eq!(h.services.skill_tasks.len(), 2);
    assert_eq!(
        h.services.skill_tasks[0].kind,
        SkillTaskKind::Shop(ShopTask::RetrieveMysteriousStore { key: 0 })
    );
    h.pc().zone = None;
    assert_eq!(replies(&h.run(944, &[7])), e(946, 2, &[0; 8]));
    assert_eq!(replies(&h.run(1566, &[])), r(1568, 2));
    h.pc().zone = Some(9);
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(
        replies(&h.run(944, &88u64.to_le_bytes())),
        e(946, 2, &88u64.to_le_bytes())
    );
    assert_eq!(replies(&h.run(1566, &[])), r(1568, 2));
}

#[test]
fn mileage_gates_before_decode() {
    let mut h = H::new();
    h.services.shop.results.clear();
    h.world.arena = true;
    assert_eq!(replies(&h.run(940, &[])), r(942, 5));
    assert!(h.services.shop.calls.is_empty());
    h.world.arena = false;
    for state in [2, 3, 5, 6] {
        h.pc().arena_state = state;
        assert_eq!(replies(&h.run(940, &[])), r(942, 5));
    }
    h.pc().arena_state = 0;
    unsupported(h.run(940, &[]));
    h.set(ShopCheck::MileageEnabled, Gate::Fail);
    assert_eq!(replies(&h.run(940, &[])), r(942, 2177));
    h.set(ShopCheck::MileageEnabled, Gate::Pass);
    for b in [vec![], vec![1, 0], vec![0, 0, 1, 0], vec![1, 0, 0, 0]] {
        assert_eq!(replies(&h.run(940, &b)), r(942, 2177));
    }
    unsupported(h.run(940, &mileage()));
    h.set(ShopCheck::MileageWithinLimit { count: 1 }, Gate::Fail);
    assert_eq!(replies(&h.run(940, &mileage())), r(942, 2179));
}

#[test]
fn mileage_lists_and_boundary() {
    let mut h = H::new();
    for n in [0u16, 6, 0xFFFF] {
        let mut b = vec![1, 0, 1, 0];
        b.extend(n.to_le_bytes());
        assert_eq!(replies(&h.run(940, &b)), r(942, 2182));
        let mut b = mileage()[..10].to_vec();
        b.extend(n.to_le_bytes());
        assert_eq!(replies(&h.run(940, &b)), r(942, 2184));
    }
    for (len, reason) in [
        (4, 2182),
        (6, 2177),
        (9, 2177),
        (10, 2184),
        (12, 2177),
        (15, 2177),
    ] {
        assert_eq!(replies(&h.run(940, &mileage()[..len])), r(942, reason));
    }
    h.pc().zone = None;
    assert_eq!(replies(&h.run(940, &mileage())), r(942, 2));
    h.pc().zone = Some(9);
    h.pc().native_kind = 2;
    assert_eq!(replies(&h.run(940, &mileage())), r(942, 2));
    h.pc().native_kind = 1;
    unsupported(h.run(940, &mileage()));
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn fatigue_precedes_decode_for_sell_and_mysterious() {
    for (command, reply, code) in [(1008, 1009, 2291), (1345, 1347, 355)] {
        let mut h = H::new();
        h.services.shop.results.clear();
        unsupported(h.run(command, &[]));
        h.pc().user_present = false;
        h.services.shop.calls.clear();
        assert_eq!(replies(&h.run(command, &[])), r(reply, code));
        assert!(h.services.shop.calls.is_empty());
        h.pc().user_present = true;
        h.set(ShopCheck::UserNotFatigued { pc: PC_ID }, Gate::Fail);
        assert_eq!(replies(&h.run(command, &[])), r(reply, code));
    }
}

#[test]
fn sell_depots_and_list_limits() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(1008, &[])), r(1009, 2286));
    for depot in [0, 1, 4, 5, 7, 9, 255] {
        assert_eq!(replies(&h.run(1008, &[depot])), r(1009, 2286));
    }
    for depot in [2, 3, 6, 8] {
        assert!(h.run(1008, &[depot, 0, 0]).replies.is_empty());
    }
    for n in [16u16, 0xFFFF] {
        let mut b = vec![2];
        b.extend(n.to_le_bytes());
        assert_eq!(replies(&h.run(1008, &b)), r(1009, 2287));
    }
    assert_eq!(replies(&h.run(1008, &[2])), r(1009, 2287));
    assert_eq!(replies(&h.run(1008, &[2, 1, 0, 3, 0])), r(1009, 2286));
    assert!(h.run(1008, &sell(&[(0, 0); 15])).replies.is_empty());
}

#[test]
fn mysterious_decode_and_list_limits() {
    let mut h = H::new();
    for len in [0, 7, 8, 9, 10] {
        assert_eq!(replies(&h.run(1345, &mysterious(&[])[..len])), r(1347, 351));
    }
    let mut b = mysterious(&[]);
    b[9..11].copy_from_slice(&17u16.to_le_bytes());
    assert_eq!(replies(&h.run(1345, &b)), r(1347, 351));
    b[9..11].copy_from_slice(&0xFFFFu16.to_le_bytes());
    assert_eq!(replies(&h.run(1345, &b)), r(1347, 351));
    let mut b = mysterious(&[(1, 1)]);
    b.pop();
    assert_eq!(replies(&h.run(1345, &b)), r(1347, 351));
    for n in [0, 16] {
        assert!(h
            .run(1345, &mysterious(&vec![(0, 0); n]))
            .replies
            .is_empty());
    }
}

#[test]
fn pc_task_failure_paths() {
    for (command, reply, body) in [
        (1008, 1009, sell(&[])),
        (1345, 1347, mysterious(&[])),
        (1436, 1438, restore()),
        (1439, 1441, vec![1, 2, 0]),
    ] {
        let mut h = H::new();
        h.pc().zone = None;
        assert_eq!(replies(&h.run(command, &body)), r(reply, 2));
        h.pc().zone = Some(9);
        h.pc().native_kind = 2;
        assert_eq!(replies(&h.run(command, &body)), r(reply, 2));
        h.pc().native_kind = 1;
        h.services.skill_queue_result = Gate::Fail;
        assert_eq!(replies(&h.run(command, &body)), r(reply, 2));
        h.services.skill_queue_result = Gate::Unknown;
        unsupported(h.run(command, &body));
    }
}

#[test]
fn enchant_decode_failures() {
    for (command, reply, b) in [(1436, 1438, restore()), (1439, 1441, vec![1, 2, 0])] {
        for n in 0..b.len() {
            let mut h = H::new();
            assert_eq!(replies(&h.run(command, &b[..n])), r(reply, 2));
            assert!(h.services.skill_tasks.is_empty());
        }
    }
}

#[test]
fn draw_decode_and_pair_failures_echo_store() {
    let mut h = H::new();
    let b = draw(2, &[(3, 2)]);
    for n in 0..17 {
        assert_eq!(
            replies(&h.run(1569, &b[..n])),
            e(1571, 2115, &[if n == 0 { 0 } else { 1 }])
        );
    }
    for len in [17, 18] {
        assert_eq!(replies(&h.run(1569, &b[..len])), e(1571, 2115, &[1]));
    }
    for len in [19, 20, 21, 22] {
        assert_eq!(replies(&h.run(1569, &b[..len])), e(1571, 1553, &[1]));
    }
    for n in [17u16, 0xFFFF] {
        let mut b = random_prefix(&[1, 2, 5]);
        b.extend(n.to_le_bytes());
        assert_eq!(replies(&h.run(1569, &b)), e(1571, 2115, &[1]));
    }
    for amount in [0, 0x8000, 0xFFFF] {
        assert_eq!(
            replies(&h.run(1569, &draw(2, &[(3, amount)]))),
            e(1571, 1553, &[1])
        );
    }
    // This path uses reason 1553, not the decompiler's missing argument.
    let mut b = draw(2, &[(3, 0)]);
    b[0] = 9;
    assert_eq!(replies(&h.run(1569, &b)), e(1571, 1553, &[9]));
}

#[test]
fn draw_parameter_and_state_gate_order() {
    let mut h = H::new();
    let valid = draw(2, &[(3, 2)]);
    for (idx, value) in [(0, 0), (0, 3), (1, 0), (1, 5)] {
        let mut b = valid.clone();
        b[idx] = value;
        assert_eq!(replies(&h.run(1569, &b)), e(1571, 2116, &[b[0]]));
    }
    assert_eq!(replies(&h.run(1569, &draw(2, &[]))), e(1571, 2116, &[1]));
    h.pc().instant_payment = true;
    assert_eq!(replies(&h.run(1569, &valid)), e(1571, 2126, &[1]));
    h.pc().instant_payment = false;
    h.set(ShopCheck::NoLeaveAction { pc: PC_ID }, Gate::Fail);
    assert_eq!(replies(&h.run(1569, &valid)), e(1571, 2125, &[1]));
    h.set(ShopCheck::NoLeaveAction { pc: PC_ID }, Gate::Pass);
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(1569, &valid)), e(1571, 2118, &[1]));
    h.pc().user_present = true;
    h.set(ShopCheck::UserNotFatigued { pc: PC_ID }, Gate::Fail);
    assert_eq!(replies(&h.run(1569, &valid)), e(1571, 2118, &[1]));
    h.set(ShopCheck::UserNotFatigued { pc: PC_ID }, Gate::Pass);
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1569, &valid)), e(1571, 2, &[1]));
    h.pc().zone = Some(9);
    for count in [0, 2, 4, 6, 255] {
        let mut b = valid.clone();
        b[2] = count;
        assert_eq!(replies(&h.run(1569, &b)), e(1571, 2116, &[1]));
    }
    h.pc().native_kind = 9;
    h.pc().t2_blocked = true;
    h.pc().arena_state = 3;
    for payment in [1, 3] {
        assert!(h.run(1569, &draw(payment, &[])).replies.is_empty());
    }
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1569, &valid)), e(1571, 2, &[1]));
}

#[test]
fn instant_draw_stays_at_transaction_boundary() {
    let mut h = H::new();
    let b = draw(4, &[]);
    h.set(ShopCheck::LiveServer, Gate::Fail);
    h.world.arena = true;
    assert_eq!(replies(&h.run(1569, &b)), e(1571, 15, &[1]));
    h.set(ShopCheck::LiveServer, Gate::Pass);
    assert_eq!(replies(&h.run(1569, &b)), e(1571, 5, &[1]));
    h.world.arena = false;
    h.pc().arena_state = 3;
    unsupported(h.run(1569, &b));
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn expansion_decode_and_record_precedence() {
    let mut h = H::new();
    let b = expand(2, &[(3, 1)]);
    for n in 0..17 {
        assert_eq!(
            replies(&h.run(1572, &b[..n])),
            e(1574, 2130, &[if n == 0 { 0 } else { 1 }, 255])
        );
    }
    let mut bad = b.clone();
    bad[0] = 0;
    assert_eq!(replies(&h.run(1572, &bad)), e(1574, 2131, &[0, 3]));
    h.set(ShopCheck::ExpandRecord { store: 1, slot: 3 }, Gate::Fail);
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(1572, &b)), e(1574, 2139, &[1, 3]));
    h.set(ShopCheck::ExpandRecord { store: 1, slot: 3 }, Gate::Pass);
    assert_eq!(replies(&h.run(1572, &b)), e(1574, 2, &[1, 3]));
    h.pc().user_present = true;
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1572, &b)), e(1574, 2, &[1, 3]));
    h.pc().zone = Some(9);
    assert_eq!(
        replies(&h.run(1572, &expand(1, &[]))),
        e(1574, 2131, &[1, 3])
    );
}

#[test]
fn expansion_item_lists_and_queue() {
    let mut h = H::new();
    for n in [0u16, 17, 0xFFFF] {
        let mut b = random_prefix(&[1, 3, 2]);
        b.extend(n.to_le_bytes());
        assert_eq!(replies(&h.run(1572, &b)), e(1574, 2133, &[1, 3]));
    }
    assert_eq!(
        replies(&h.run(1572, &random_prefix(&[1, 3, 2]))),
        e(1574, 2133, &[1, 3])
    );
    let mut b = expand(2, &[(1, 0)]);
    b.pop();
    assert_eq!(replies(&h.run(1572, &b)), e(1574, 2130, &[1, 3]));
    h.pc().native_kind = 9;
    h.pc().t2_blocked = true;
    h.world.arena = true;
    h.set(ShopCheck::UserNotFatigued { pc: PC_ID }, Gate::Fail);
    let b = expand(2, &[(0, 0); 16]);
    assert!(h.run(1572, &b).replies.is_empty());
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1572, &b)), e(1574, 2, &[1, 3]));
}

#[test]
fn instant_expansion_config_order_and_boundary() {
    let mut h = H::new();
    let b = expand(4, &[]);
    h.set(ShopCheck::LiveServer, Gate::Fail);
    assert_eq!(replies(&h.run(1572, &b)), e(1574, 15, &[1, 3]));
    h.set(ShopCheck::LiveServer, Gate::Pass);
    h.set(
        ShopCheck::ExpandPaymentEnabled { store: 1, slot: 3 },
        Gate::Fail,
    );
    assert_eq!(replies(&h.run(1572, &b)), e(1574, 15, &[1, 3]));
    h.set(
        ShopCheck::ExpandPaymentEnabled { store: 1, slot: 3 },
        Gate::Pass,
    );
    h.world.arena = true;
    assert_eq!(replies(&h.run(1572, &b)), e(1574, 5, &[1, 3]));
    h.world.arena = false;
    unsupported(h.run(1572, &b));
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn buy_random_prefix_echo_and_parameters() {
    let mut h = H::new();
    for (b, fields) in [
        (vec![], vec![0, 0, 0]),
        (vec![2], vec![2, 0, 0]),
        (vec![2, 3], vec![2, 3, 0]),
    ] {
        assert_eq!(replies(&h.run(1575, &b)), e(1577, 2145, &fields));
    }
    for fields in [
        [0, 1, 1],
        [3, 1, 1],
        [1, 0, 1],
        [1, 6, 1],
        [1, 1, 0],
        [1, 1, 9],
    ] {
        assert_eq!(replies(&h.run(1575, &fields)), e(1577, 2146, &fields));
    }
}

#[test]
fn buy_random_fatigue_lists_and_queue() {
    let mut h = H::new();
    let fields = [1, 5, 8];
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(1575, &fields)), e(1577, 2148, &fields));
    h.pc().user_present = true;
    h.set(ShopCheck::UserNotFatigued { pc: PC_ID }, Gate::Fail);
    assert_eq!(replies(&h.run(1575, &fields)), e(1577, 2148, &fields));
    h.set(ShopCheck::UserNotFatigued { pc: PC_ID }, Gate::Pass);
    assert_eq!(replies(&h.run(1575, &fields)), e(1577, 2145, &fields));
    for n in [17u16, 0xFFFF] {
        let mut b = fields.to_vec();
        b.extend(n.to_le_bytes());
        assert_eq!(replies(&h.run(1575, &b)), e(1577, 2145, &fields));
    }
    let mut b = buy(&[(1, 1)]);
    b.pop();
    assert_eq!(replies(&h.run(1575, &b)), e(1577, 2145, &fields));
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1575, &buy(&[]))), e(1577, 2, &fields));
    h.pc().zone = Some(9);
    h.pc().native_kind = 9;
    assert!(h.run(1575, &buy(&[(0, 0); 16])).replies.is_empty());
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1575, &buy(&[]))), e(1577, 2, &fields));
}

#[test]
fn reward_echo_and_signed_index() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(1578, &[])), e(1580, 2157, &[0, 0]));
    assert_eq!(replies(&h.run(1578, &[2])), e(1580, 2157, &[2, 0]));
    for fields in [[0, 0], [3, 0], [1, 128], [2, 255]] {
        assert_eq!(replies(&h.run(1578, &fields)), e(1580, 2158, &fields));
    }
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1578, &[1, 4])), e(1580, 2, &[1, 4]));
    h.pc().zone = Some(9);
    h.pc().native_kind = 9;
    assert!(h.run(1578, &[1, 4]).replies.is_empty());
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1578, &[1, 4])), e(1580, 2, &[1, 4]));
}

#[test]
fn collection_configuration_order() {
    let mut h = H::new();
    h.services.shop.results.clear();
    h.world.arena = true;
    assert_eq!(replies(&h.run(1584, &[])), r(1586, 5));
    assert!(h.services.shop.calls.is_empty());
    h.world.arena = false;
    h.world.account_inventory_enabled = false;
    assert_eq!(replies(&h.run(1584, &[])), r(1586, 15));
    assert!(h.services.shop.calls.is_empty());
    h.world.account_inventory_enabled = true;
    unsupported(h.run(1584, &[]));
    h.set(ShopCheck::LiveServer, Gate::Fail);
    assert_eq!(replies(&h.run(1584, &[])), r(1586, 15));
    h.set(ShopCheck::LiveServer, Gate::Pass);
    h.set(ShopCheck::CollectionEnabled, Gate::Fail);
    assert_eq!(replies(&h.run(1584, &[])), r(1586, 20));
    h.set(ShopCheck::CollectionEnabled, Gate::Pass);
    h.pc().zone = None;
    h.pc().instant_payment = true;
    assert_eq!(replies(&h.run(1584, &[])), r(1586, 2));
    h.pc().zone = Some(9);
    assert_eq!(replies(&h.run(1584, &[])), r(1586, 29));
    h.pc().instant_payment = false;
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(1584, &[])), r(1586, 2));
}

#[test]
fn collection_decode_count_and_blob_lengths() {
    let mut h = H::new();
    let b = collection(&[(1, 10, 20)]);
    for len in 0..10 {
        assert_eq!(replies(&h.run(1584, &b[..len])), r(1586, 2504));
    }
    for len in [10, 11] {
        assert_eq!(replies(&h.run(1584, &b[..len])), r(1586, 2505));
    }
    for len in 12..b.len() {
        assert_eq!(replies(&h.run(1584, &b[..len])), r(1586, 2504));
    }
    for n in [11u16, 0xFFFF] {
        let mut b = collection(&[]);
        b[10..12].copy_from_slice(&n.to_le_bytes());
        assert_eq!(replies(&h.run(1584, &b)), r(1586, 2505));
    }
    for n in [9u16, 0xFFFF] {
        let mut b = b.clone();
        b[..2].copy_from_slice(&n.to_le_bytes());
        assert_eq!(replies(&h.run(1584, &b)), r(1586, 2504));
        let mut b = collection(&[(1, 10, 20)]);
        b[16..18].copy_from_slice(&n.to_le_bytes());
        assert_eq!(replies(&h.run(1584, &b)), r(1586, 2504));
    }
}

#[test]
fn collection_local_and_table_failures() {
    let mut h = H::new();
    for goods in [0, -1] {
        assert_eq!(
            replies(&h.run(1584, &collection(&[(goods, 0, 0)]))),
            r(1586, 2506)
        );
    }
    assert_eq!(
        replies(&h.run(1584, &collection(&[(1, 0, 0)]))),
        r(1586, 2508)
    );
    assert_eq!(
        replies(&h.run(1584, &collection(&[(1, 10, 0)]))),
        r(1586, 2509)
    );
    for (check, code) in [
        (
            ShopCheck::CollectionGroup {
                group: 10,
                item: 20,
            },
            2508,
        ),
        (ShopCheck::CollectionItem { item: 20 }, 2509),
        (
            ShopCheck::CollectionGroupMatch {
                group: 10,
                item: 20,
            },
            2510,
        ),
        (ShopCheck::CollectionChargeItem { group: 10 }, 2508),
    ] {
        h.set(check.clone(), Gate::Fail);
        assert_eq!(
            replies(&h.run(1584, &collection(&[(1, 10, 20)]))),
            r(1586, code)
        );
        h.set(check, Gate::Pass);
    }
    h.pc().native_kind = 9;
    h.pc().arena_state = 3;
    h.pc().t2_blocked = true;
    unsupported(h.run(1584, &collection(&[(1, 10, 20)])));
    unsupported(h.run(1584, &collection(&[])));
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn unknown_adapter_states_never_approve() {
    for (command, b, check) in [
        (
            1569,
            draw(2, &[(1, 1)]),
            ShopCheck::NoLeaveAction { pc: PC_ID },
        ),
        (1569, draw(1, &[]), ShopCheck::UserNotFatigued { pc: PC_ID }),
        (1569, draw(4, &[]), ShopCheck::LiveServer),
        (
            1572,
            expand(2, &[(1, 1)]),
            ShopCheck::ExpandRecord { store: 1, slot: 3 },
        ),
        (
            1572,
            expand(4, &[]),
            ShopCheck::ExpandPaymentEnabled { store: 1, slot: 3 },
        ),
        (1584, collection(&[]), ShopCheck::CollectionEnabled),
        (
            1584,
            collection(&[(1, 10, 20)]),
            ShopCheck::CollectionGroup {
                group: 10,
                item: 20,
            },
        ),
    ] {
        let mut h = H::new();
        h.set(check.clone(), Gate::Unknown);
        unsupported(h.run(command, &b));
        assert_eq!(h.services.shop.calls.last(), Some(&check));
        assert!(h.services.skill_tasks.is_empty());
    }
    let mut h = H::new();
    h.services.skill_queue_result = Gate::Unknown;
    for (c, b) in [
        (80, vec![]),
        (944, vec![]),
        (1566, vec![]),
        (1569, draw(1, &[])),
        (1572, expand(2, &[(1, 1)])),
        (1575, buy(&[])),
        (1578, vec![1, 0]),
    ] {
        unsupported(h.run(c, &b));
    }
}
