//! Doc 29: NPC trade, board gacha and dungeon-step entry functions.
use gamedaemon::handlers::*;

const PC_ID: u64 = (1 << 48) | 0x11;
const NPC: u64 = 0x0102_0304_0506_0708;

struct H {
    session: Session,
    world: World,
    services: MemoryServices,
}
impl H {
    fn new() -> Self {
        Self {
            session: Session {
                authenticated: true,
                pc: Some(Pc {
                    object_id: PC_ID,
                    db_id: 77,
                    zone: Some(9),
                    native_kind: 1,
                    user_present: true,
                    ..Pc::default()
                }),
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
    fn gacha() -> Self {
        let mut h = Self::new();
        h.services.npc_trade.enabled = Gate::Pass;
        h.services.npc_trade.cost = Gate::Pass;
        h.services.npc_trade.instant = Gate::Fail;
        h.services.npc_trade.payment_item = Gate::Pass;
        h
    }
    fn pc(&mut self) -> &mut Pc {
        self.session.pc.as_mut().unwrap()
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
    fn task(&self) -> &SkillTask {
        self.services.skill_tasks.last().unwrap()
    }
}

fn reason(out: &Outcome, command: u16, code: u16) {
    assert_eq!(
        out.replies,
        vec![Message {
            command,
            body: code.to_le_bytes().to_vec()
        }],
        "{out:?}"
    );
    assert!(out.changes.is_empty());
}
fn unsupported(out: &Outcome) {
    assert!(
        matches!(out.status, Status::Unsupported(why) if why.starts_with("❓")),
        "{out:?}"
    );
    assert!(out.replies.is_empty());
}
fn queued(h: &H, out: &Outcome, command: u16, kind: NpcTradeTask) {
    assert_eq!(out.status, Status::Handled, "{out:?}");
    assert!(out.replies.is_empty());
    assert_eq!(
        h.task(),
        &SkillTask {
            command,
            caster: PC_ID,
            summon: false,
            zone: 9,
            kind: SkillTaskKind::NpcTrade(kind)
        }
    );
}
fn pairs(count: i16, items: &[(u16, u16)]) -> Vec<u8> {
    let mut b = count.to_le_bytes().to_vec();
    for (a, c) in items {
        b.extend(a.to_le_bytes());
        b.extend(c.to_le_bytes());
    }
    b
}
fn npc() -> Vec<u8> {
    NPC.to_le_bytes().to_vec()
}

#[test]
fn command_list_has_eight_ids_and_no_doc22_store_ids() {
    let mut got = NPC_TRADE_COMMANDS.to_vec();
    got.sort_unstable();
    assert_eq!(got, vec![1176, 1179, 1341, 1343, 1348, 1350, 1352, 1456]);
    for id in [80, 827, 940, 944, 1008, 1345, 1566, 1584] {
        assert!(!NPC_TRADE_COMMANDS.contains(&id));
    }
}

#[test]
fn buy_user_decode_count_zone_and_task() {
    let mut body = npc();
    body.extend(1i16.to_le_bytes());
    body.extend([3u8]);
    body.extend(5u16.to_le_bytes());

    let mut h = H::new();
    h.pc().party_user_blocked = true;
    reason(&h.run(1341, &body), 1342, 314);

    let mut h = H::new();
    reason(&h.run(1341, &[1, 2]), 1342, 303);
    let mut short = npc();
    short.extend(16i16.to_le_bytes());
    reason(&h.run(1341, &short), 1342, 304);
    let mut missing = npc();
    missing.extend(1i16.to_le_bytes());
    missing.push(3);
    reason(&h.run(1341, &missing), 1342, 303);

    h.pc().zone = None;
    reason(&h.run(1341, &body), 1342, 2);
    h.pc().zone = Some(9);
    h.pc().native_kind = 2;
    reason(&h.run(1341, &body), 1342, 2);
    h.pc().native_kind = 1;

    let out = h.run(1341, &body);
    queued(
        &h,
        &out,
        1341,
        NpcTradeTask::Buy {
            npc: NPC,
            items: vec![(3, 5)],
        },
    );
}

#[test]
fn buy_queue_failure_replies_reason_2() {
    let mut h = H::new();
    h.services.skill_queue_result = Gate::Fail;
    let mut body = npc();
    body.extend(0i16.to_le_bytes());
    reason(&h.run(1341, &body), 1342, 2);
}

#[test]
fn buy2_index_limit_and_task() {
    let head = |index: u8| {
        let mut b = npc();
        b.extend([index, 4]);
        b.extend(600u16.to_le_bytes());
        b
    };
    let mut h = H::new();
    h.pc().party_user_state = 1;
    reason(&h.run(1343, &head(0)), 1344, 350);

    let mut h = H::new();
    reason(&h.run(1343, &npc()), 1344, 348);
    let mut bad = head(120);
    bad.extend(pairs(0, &[]));
    reason(&h.run(1343, &bad), 1344, 348);
    let mut over = head(1);
    over.extend(pairs(-1, &[]));
    reason(&h.run(1343, &over), 1344, 349);

    let mut ok = head(119);
    ok.extend(pairs(2, &[(1, 2), (3, 4)]));
    let out = h.run(1343, &ok);
    queued(
        &h,
        &out,
        1343,
        NpcTradeTask::Buy2 {
            npc: NPC,
            index: 119,
            selector: 4,
            store: 600,
            items: vec![(1, 2), (3, 4)],
        },
    );
}

#[test]
fn sell_reasons_and_task() {
    let mut h = H::new();
    h.pc().user_present = false;
    reason(&h.run(1348, &npc()), 1349, 322);

    let mut h = H::new();
    reason(&h.run(1348, &[]), 1349, 315);
    let mut over = npc();
    over.extend(pairs(16, &[]));
    reason(&h.run(1348, &over), 1349, 316);
    let mut ok = npc();
    ok.extend(pairs(1, &[(7, 8)]));
    let out = h.run(1348, &ok);
    queued(
        &h,
        &out,
        1348,
        NpcTradeTask::Sell {
            npc: NPC,
            items: vec![(7, 8)],
        },
    );
}

#[test]
fn rebuy_uses_reason_1_and_unchecked_decode_is_unsupported() {
    let mut h = H::new();
    h.pc().party_user_blocked = true;
    reason(&h.run(1350, &[]), 1351, 332);

    let mut h = H::new();
    let mut ok = npc();
    ok.push(6);
    h.pc().zone = None;
    reason(&h.run(1350, &ok), 1351, 1);
    h.pc().zone = Some(9);
    h.pc().native_kind = 0;
    reason(&h.run(1350, &ok), 1351, 1);
    h.pc().native_kind = 1;
    unsupported(&h.run(1350, &npc()));
    assert!(h.services.skill_tasks.is_empty());

    let out = h.run(1350, &ok);
    queued(&h, &out, 1350, NpcTradeTask::Rebuy { npc: NPC, slot: 6 });
    h.services.skill_queue_result = Gate::Fail;
    reason(&h.run(1350, &ok), 1351, 1);
}

#[test]
fn trade_with_npc_reasons_and_task() {
    let mut h = H::new();
    h.pc().party_user_state = 2;
    reason(&h.run(1352, &[]), 1353, 347);

    let mut h = H::new();
    reason(&h.run(1352, &npc()), 1353, 337);
    let mut head = npc();
    head.push(1);
    head.extend(77u16.to_le_bytes());
    let mut over = head.clone();
    over.extend(pairs(20, &[]));
    reason(&h.run(1352, &over), 1353, 338);
    let mut ok = head;
    ok.extend(pairs(1, &[(9, 1)]));
    let out = h.run(1352, &ok);
    queued(
        &h,
        &out,
        1352,
        NpcTradeTask::TradeWithNpc {
            npc: NPC,
            selector: 1,
            store: 77,
            items: vec![(9, 1)],
        },
    );
}

fn gacha_item_body(count: i16, items: &[(u16, u16)]) -> Vec<u8> {
    let mut b = 5u64.to_le_bytes().to_vec();
    b.push(1);
    b.extend(pairs(count, items));
    b
}

#[test]
fn board_gacha_prefix_order_and_unknown() {
    let body = gacha_item_body(1, &[(1, 1)]);
    let mut h = H::new();
    unsupported(&h.run(1176, &body));
    assert_eq!(
        h.services.npc_trade.calls,
        vec![NpcTradeCheck::BoardGachaEnabled]
    );

    let mut h = H::gacha();
    h.services.npc_trade.enabled = Gate::Fail;
    reason(&h.run(1176, &body), 1178, 2);
    let mut h = H::gacha();
    h.services.npc_trade.cost = Gate::Fail;
    reason(&h.run(1176, &body), 1178, 2);
    // Item draw needs instant payment off; instant draw needs it on.
    let mut h = H::gacha();
    h.services.npc_trade.instant = Gate::Pass;
    reason(&h.run(1176, &body), 1178, 2518);
    let mut h = H::gacha();
    reason(&h.run(1179, &[]), 1181, 2);
    assert_eq!(
        h.services.npc_trade.calls,
        vec![
            NpcTradeCheck::BoardGachaEnabled,
            NpcTradeCheck::BoardCostExists,
            NpcTradeCheck::InstantPaymentEnabled
        ]
    );
    let mut h = H::gacha();
    h.pc().user_present = false;
    reason(&h.run(1176, &body), 1178, 2520);
}

#[test]
fn board_gacha_by_item_decode_and_boundary() {
    let mut h = H::gacha();
    reason(&h.run(1176, &[1, 2, 3]), 1178, 2519);
    reason(&h.run(1176, &gacha_item_body(0, &[])), 1178, 2519);
    reason(&h.run(1176, &gacha_item_body(11, &[])), 1178, 2519);
    reason(&h.run(1176, &gacha_item_body(2, &[(1, 1)])), 1178, 2519);
    h.pc().zone = None;
    reason(&h.run(1176, &gacha_item_body(1, &[(1, 1)])), 1178, 2);
    h.pc().zone = Some(9);
    unsupported(&h.run(1176, &gacha_item_body(10, &[(1, 1); 10])));
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn board_gacha_by_instant_payment_reasons_and_boundary() {
    let mut body = 5u64.to_le_bytes().to_vec();
    body.push(1);
    body.extend(100i32.to_le_bytes());
    body.extend([0u8; 8]);

    let mut h = H::gacha();
    h.services.npc_trade.instant = Gate::Pass;
    h.pc().trial_jumping = true;
    reason(&h.run(1179, &body), 1181, 32);
    h.pc().trial_jumping = false;
    reason(&h.run(1179, &body[..12]), 1181, 2);
    h.services.npc_trade.payment_item = Gate::Fail;
    reason(&h.run(1179, &body), 1181, 2);
    h.services.npc_trade.payment_item = Gate::Unknown;
    unsupported(&h.run(1179, &body));
    h.services.npc_trade.payment_item = Gate::Pass;
    // No zone check on this entry.
    h.pc().zone = None;
    unsupported(&h.run(1179, &body));
}

#[test]
fn dungeon_progress_uses_1451_for_task_failures() {
    let mut head = 3u16.to_le_bytes().to_vec();
    head.extend(4u16.to_le_bytes());
    head.push(2);
    head.extend(npc());

    let mut h = H::new();
    reason(&h.run(1456, &head[..5]), 1457, 1067);
    let mut over = head.clone();
    over.extend(pairs(16, &[]));
    reason(&h.run(1456, &over), 1457, 1068);
    let mut ok = head;
    ok.extend(pairs(1, &[(5, 6)]));
    h.pc().zone = None;
    reason(&h.run(1456, &ok), 1457, 2);
    h.pc().zone = Some(9);
    h.pc().native_kind = 3;
    reason(&h.run(1456, &ok), 1451, 2);
    h.pc().native_kind = 1;
    h.services.skill_queue_result = Gate::Fail;
    reason(&h.run(1456, &ok), 1451, 2);
    h.services.skill_queue_result = Gate::Pass;
    let out = h.run(1456, &ok);
    queued(
        &h,
        &out,
        1456,
        NpcTradeTask::DungeonProgressToStep {
            dungeon: 3,
            step: 4,
            selector: 2,
            npc: NPC,
            items: vec![(5, 6)],
        },
    );
}

#[test]
fn parity_tags_resolve_to_doc_headings() {
    let source = include_str!("../src/handlers/npc_trade.rs");
    let doc = include_str!("../../../../Docs/Server/GameDaemon/29-npc-trade-handlers.md");
    for &id in NPC_TRADE_COMMANDS {
        let prefix = format!("## {id} — ");
        let heading = doc.lines().find(|l| l.starts_with(&prefix)).unwrap();
        let slug = heading
            .trim_start_matches("## ")
            .replace('—', "")
            .replace(' ', "-")
            .to_lowercase();
        let tag = format!("/// parity: {id} 29-npc-trade-handlers.md#{slug} partial");
        assert!(source.contains(&tag), "missing tag {tag}");
    }
}
