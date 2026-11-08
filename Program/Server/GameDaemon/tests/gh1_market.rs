//! Doc 24: market entry functions. Build 381.8867.
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
            short_96: 3,
            user_present: true,
            ..Pc::default()
        };
        let services = MemoryServices {
            skill_queue_result: Gate::Pass,
            market: MemoryMarket {
                register_sale_enabled: Some(true),
                sale_price_limit: Some(1000),
                bid_price_limit: Some(2000),
                auction_escrow_enabled: Some(true),
                submit_result: Gate::Pass,
                ..MemoryMarket::default()
            },
            ..MemoryServices::default()
        };
        Self {
            session: Session {
                authenticated: true,
                pc: Some(pc),
                dispatch_permission: Some(true),
                ..Session::default()
            },
            world: World::default(),
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
    fn last_task(&self) -> SkillTask {
        self.services.skill_tasks.last().unwrap().clone()
    }
    fn last_request(&self) -> MarketRequest {
        let MarketCheck::Submit(request) = self.services.market.calls.last().unwrap() else {
            panic!("expected market submission")
        };
        request.clone()
    }
}
fn replies(out: &Outcome) -> Vec<(u16, Vec<u8>)> {
    out.replies
        .iter()
        .map(|m| (m.command, m.body.clone()))
        .collect()
}
fn reason(command: u16, code: u16) -> Vec<(u16, Vec<u8>)> {
    vec![(command, code.to_le_bytes().to_vec())]
}
fn silent(out: &Outcome) {
    assert_eq!(out.status, Status::Handled);
    assert!(out.replies.is_empty());
}
fn unsupported(out: &Outcome) {
    assert!(matches!(out.status, Status::Unsupported(_)), "{out:?}");
    assert!(out.replies.is_empty());
}
fn quads(values: &[u64]) -> Vec<u8> {
    values.iter().flat_map(|n| n.to_le_bytes()).collect()
}
fn sale() -> Vec<u8> {
    let mut v = 0x1234u64.to_le_bytes().to_vec();
    v.push(2);
    v.extend(900i64.to_le_bytes());
    v.extend((-4i32).to_le_bytes());
    v.extend((-2i16).to_le_bytes());
    v.extend(8i16.to_le_bytes());
    v
}
fn auction(currency: i16, price: i64) -> Vec<u8> {
    let mut v = (0..16).collect::<Vec<_>>();
    v.extend(currency.to_le_bytes());
    v.extend(price.to_le_bytes());
    v
}
fn search() -> Vec<u8> {
    let mut v = vec![3, 2, 1, 2];
    v.extend((-7i32).to_le_bytes());
    v.extend(quads(&[11, 22, 33]));
    v
}

#[test]
fn command_membership_and_dispatch_preconditions() {
    assert_eq!(
        MARKET_COMMANDS,
        &[378, 381, 384, 1013, 1530, 1533, 1536, 1539, 1542, 1545, 1548, 1551, 1554]
    );
    let mut h = H::new();
    for &command in MARKET_COMMANDS {
        h.session.authenticated = false;
        let out = h.run(command, &[]);
        assert!(out.logs.contains(&"missing authenticated PC"));
        h.session.authenticated = true;
        h.session.dispatch_permission = None;
        unsupported(&h.run(command, &[]));
        h.session.dispatch_permission = Some(true);
    }
    assert!(h.services.market.calls.is_empty());
}

#[test]
fn arena_b_replies_and_priority() {
    for world_arena in [false, true] {
        for state in [2, 3, 5, 6] {
            let mut h = H::new();
            h.world.arena = world_arena;
            h.pc().arena_state = state;
            h.pc().t2_blocked = true;
            for (command, failure) in [
                (381, 383),
                (384, 386),
                (1013, 1015),
                (1530, 1532),
                (1533, 1535),
                (1536, 1538),
                (1539, 1541),
                (1542, 1544),
                (1545, 1547),
                (1548, 1550),
            ] {
                assert_eq!(replies(&h.run(command, &[])), reason(failure, 5));
            }
            let mut body = 5u16.to_le_bytes().to_vec();
            body.extend([0; 11]);
            assert_eq!(replies(&h.run(378, &[])), vec![(380, body)]);
        }
    }
    let mut h = H::new();
    h.world.arena = true;
    h.pc().arena_state = 0;
    assert_eq!(replies(&h.run(381, &[])), reason(383, 5));
    h.services.market.register_sale_enabled = Some(false);
    assert_eq!(replies(&h.run(1530, &[])), reason(1532, 1223));
}

#[test]
fn trial_play_replies_before_decode() {
    let mut h = H::new();
    h.pc().t2_blocked = true;
    for (command, failure) in [
        (1530, 1532),
        (1533, 1535),
        (1536, 1538),
        (1539, 1541),
        (1542, 1544),
        (1545, 1547),
        (1548, 1550),
    ] {
        assert_eq!(replies(&h.run(command, &[])), reason(failure, 7));
    }
    assert!(h.services.skill_tasks.is_empty());
    assert_eq!(
        h.services.market.calls,
        vec![MarketCheck::RegisterSaleEnabled]
    );
}

#[test]
fn retrieve_sale_list_fields_and_unchecked_decode() {
    let mut h = H::new();
    let mut body = vec![0x81, 0x82];
    body.extend(55u64.to_le_bytes());
    body.push(0xff);
    for n in 0..body.len() {
        unsupported(&h.run(378, &body[..n]));
    }
    silent(&h.run(378, &body));
    assert_eq!(
        h.last_request(),
        MarketRequest {
            object_id: PC_ID,
            db_id: 77,
            world: 3,
            operation: MarketOperation::RetrieveSaleList {
                first: 0x81,
                second: 0x82,
                cursor: 55,
                direction: -1,
            },
        }
    );
    h.pc().t2_blocked = true;
    h.services.market.submit_result = Gate::Fail;
    silent(&h.run(378, &body)); // No trial gate; dealer failure is ignored.
}

#[test]
fn retrieve_bid_list_ignores_body_and_trial() {
    let mut h = H::new();
    h.pc().t2_blocked = true;
    silent(&h.run(381, &[0xff]));
    assert_eq!(h.last_request().operation, MarketOperation::RetrieveBidList);
    h.services.market.submit_result = Gate::Fail;
    silent(&h.run(381, &[]));
}

#[test]
fn retrieve_rank_count_and_element_errors() {
    let mut h = H::new();
    for body in [
        vec![],
        vec![1],
        vec![0xff, 0xff],
        vec![33, 0],
        vec![1, 0],
        vec![2, 0, 1, 0, 0, 0, 0, 0, 0, 0],
    ] {
        assert_eq!(replies(&h.run(384, &body)), reason(386, 2));
    }
    for count in [0u16, 1, 32] {
        let ids: Vec<_> = (0..u64::from(count)).collect();
        let mut body = count.to_le_bytes().to_vec();
        body.extend(quads(&ids));
        body.push(0x99); // Trailing bytes are allowed.
        silent(&h.run(384, &body));
        assert_eq!(
            h.last_request().operation,
            MarketOperation::RetrieveBidRankStatus { sales: ids }
        );
    }
}

#[test]
fn dealer_missing_user_and_blocked_user_are_silent() {
    let mut sale_list = vec![1, 2];
    sale_list.extend(3u64.to_le_bytes());
    sale_list.push(4);
    for blocked in [false, true] {
        let mut h = H::new();
        h.pc().user_present = blocked;
        h.pc().party_user_blocked = blocked;
        for (command, body) in [
            (378, sale_list.clone()),
            (381, vec![]),
            (384, vec![0, 0]),
            (1533, quads(&[7])),
            (1536, quads(&[7, 8])),
            (1542, quads(&[7, 8])),
        ] {
            silent(&h.run(command, &body));
        }
        assert!(!h
            .services
            .market
            .calls
            .iter()
            .any(|c| matches!(c, MarketCheck::Submit(_))));
    }
}

#[test]
fn auction_parameter_and_configuration_replies() {
    let mut h = H::new();
    let body = auction(4, 800);
    for n in 0..body.len() {
        assert_eq!(replies(&h.run(1013, &body[..n])), reason(1015, 2));
    }
    for (currency, price) in [(0, 1), (-1, 1), (1, 0), (1, -1), (i16::MIN, i64::MIN)] {
        assert_eq!(
            replies(&h.run(1013, &auction(currency, price))),
            reason(1015, 2303)
        );
    }
    h.pc().zone = None;
    h.services.market.auction_escrow_enabled = Some(false);
    assert_eq!(replies(&h.run(1013, &body)), reason(1015, 2304));
    h.services.market.auction_escrow_enabled = None;
    unsupported(&h.run(1013, &body));
    h.services.market.auction_escrow_enabled = Some(true);
    assert_eq!(replies(&h.run(1013, &body)), reason(1015, 2));
}

#[test]
fn auction_initializer_and_queue_failures() {
    let mut h = H::new();
    let body = auction(4, 800);
    for world in [0, -1] {
        h.pc().short_96 = world;
        assert_eq!(replies(&h.run(1013, &body)), reason(1015, 2));
    }
    h.pc().short_96 = 3;
    h.pc().db_id = 0;
    assert_eq!(replies(&h.run(1013, &body)), reason(1015, 2));
    h.pc().db_id = 77;
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1013, &body)), reason(1015, 2));
    h.services.skill_queue_result = Gate::Unknown;
    unsupported(&h.run(1013, &body));
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn auction_task_fields_and_raw_request() {
    let mut h = H::new();
    let body = auction(4, 800);
    h.pc().t2_blocked = true;
    h.pc().native_kind = 2; // The native entry has neither of these gates.
    let out = h.run(1013, &body);
    silent(&out);
    let task = h.last_task();
    assert_eq!(
        (task.command, task.caster, task.zone, task.summon),
        (1013, PC_ID, 9, false)
    );
    assert_eq!(
        task.kind,
        SkillTaskKind::Market(MarketTask::AuctionBid {
            world: 3,
            db_id: 77,
            auction: std::array::from_fn(|i| i as u8),
            currency_group: 4,
            price: 800,
        })
    );
    assert!(out.changes.contains(&StateChange::QueuedRequest {
        command: 1013,
        body
    }));
}

#[test]
fn register_sale_decode_zone_kind_and_queue_replies() {
    let mut h = H::new();
    let body = sale();
    for n in 0..body.len() {
        assert_eq!(replies(&h.run(1530, &body[..n])), reason(1532, 1213));
    }
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1530, &body)), reason(1532, 1));
    h.pc().zone = Some(9);
    h.pc().native_kind = 2;
    assert_eq!(replies(&h.run(1530, &body)), reason(1532, 1));
    h.pc().native_kind = 1;
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1530, &body)), reason(1532, 1));
    h.services.skill_queue_result = Gate::Unknown;
    unsupported(&h.run(1530, &body));
    h.services.market.register_sale_enabled = None;
    unsupported(&h.run(1530, &body));
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn register_sale_action_fields_without_invented_value_checks() {
    let mut h = H::new();
    let mut body = sale();
    // The entry does not validate sale type, price or count.
    body[8] = 0xff;
    body[9..17].copy_from_slice(&(-1i64).to_le_bytes());
    let out = h.run(1530, &body);
    silent(&out);
    let task = h.last_task();
    assert_eq!(
        (task.command, task.caster, task.zone, task.summon),
        (1530, PC_ID, 9, false)
    );
    assert_eq!(
        task.kind,
        SkillTaskKind::Market(MarketTask::RegisterSale {
            item: 0x1234,
            sale_type: 0xff,
            price: -1,
            count: -4,
            first: -2,
            second: 8,
        })
    );
    assert!(out.changes.contains(&StateChange::QueuedRequest {
        command: 1530,
        body
    }));
}

#[test]
fn abandon_sale_and_bid_unchecked_fields_and_send() {
    let mut h = H::new();
    for (command, body, expected) in [
        (
            1533,
            quads(&[55]),
            MarketOperation::AbandonSale { sale: 55 },
        ),
        (
            1542,
            quads(&[55, 66]),
            MarketOperation::AbandonBid { sale: 55, bid: 66 },
        ),
    ] {
        for n in 0..body.len() {
            unsupported(&h.run(command, &body[..n]));
        }
        h.pc().zone = None; // Dealer sends do not need a zone.
        h.pc().short_96 = -1;
        h.services.market.submit_result = Gate::Fail;
        silent(&h.run(command, &body));
        assert_eq!(h.last_request().operation, expected);
        assert_eq!(h.last_request().world, u16::MAX);
    }
}

#[test]
fn update_sale_price_bounds_and_zero_default() {
    let mut h = H::new();
    for n in 0..8 {
        unsupported(&h.run(1536, &quads(&[55])[..n]));
    }
    for price in [u64::MAX, 1001] {
        assert_eq!(
            replies(&h.run(1536, &quads(&[55, price]))),
            reason(1538, 33)
        );
    }
    for price in [0, 1000] {
        silent(&h.run(1536, &quads(&[55, price])));
        assert_eq!(
            h.last_request().operation,
            MarketOperation::UpdateSalePrice {
                sale: 55,
                price: price as i64
            }
        );
    }
    let body = quads(&[55, 900]);
    for n in 8..16 {
        silent(&h.run(1536, &body[..n]));
        assert_eq!(
            h.last_request().operation,
            MarketOperation::UpdateSalePrice { sale: 55, price: 0 }
        );
    }
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(1536, &quads(&[55, 1001]))), reason(1538, 33));
    h.services.market.sale_price_limit = None;
    unsupported(&h.run(1536, &quads(&[55, 5])));
    // Negative price rejection does not need an upper limit.
    assert_eq!(
        replies(&h.run(1536, &quads(&[55, u64::MAX]))),
        reason(1538, 33)
    );
}

#[test]
fn transaction_price_and_registration_replies() {
    for (command, failure, limit, fail_reason) in [
        (1539, 1541, 2000, 1),
        (1545, 1547, 2000, 1),
        (1548, 1550, 1000, 2),
    ] {
        let mut h = H::new();
        let body = |price| {
            if command == 1545 {
                quads(&[u64::MAX, 66, price])
            } else {
                quads(&[u64::MAX, price])
            }
        };
        for price in [u64::MAX, limit + 1] {
            assert_eq!(replies(&h.run(command, &body(price))), reason(failure, 33));
        }
        for price in [0, limit] {
            silent(&h.run(command, &body(price)));
            let expected = match command {
                1539 => MarketOperation::RegisterBid {
                    sale: u64::MAX,
                    price: price as i64,
                },
                1545 => MarketOperation::UpdateBidPrice {
                    sale: u64::MAX,
                    bid: 66,
                    price: price as i64,
                },
                _ => MarketOperation::RegisterPurchase {
                    sale: u64::MAX,
                    price: price as i64,
                },
            };
            assert_eq!(h.last_request().operation, expected);
        }
        h.services.market.submit_result = Gate::Fail;
        assert_eq!(
            replies(&h.run(command, &body(5))),
            reason(failure, fail_reason)
        );
        h.services.market.submit_result = Gate::Unknown;
        unsupported(&h.run(command, &body(5)));
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn transaction_missing_user_order_is_native() {
    for blocked in [false, true] {
        let mut h = H::new();
        h.pc().user_present = blocked;
        h.pc().party_user_blocked = blocked;
        assert_eq!(replies(&h.run(1539, &quads(&[55, 2001]))), reason(1541, 33));
        assert_eq!(replies(&h.run(1548, &quads(&[55, 1001]))), reason(1550, 33));
        let calls = h.services.market.calls.len();
        silent(&h.run(1545, &quads(&[55, 66, u64::MAX])));
        assert_eq!(h.services.market.calls.len(), calls); // User precedes the limit check.
        silent(&h.run(1539, &quads(&[55, 1])));
        silent(&h.run(1548, &quads(&[55, 1])));
        assert!(!h
            .services
            .market
            .calls
            .iter()
            .any(|c| matches!(c, MarketCheck::Submit(_))));
    }
}

#[test]
fn transaction_unchecked_decode_keeps_zero_defaults() {
    for command in [1539, 1545, 1548] {
        let mut h = H::new();
        let body = if command == 1545 {
            quads(&[55, 66, 77])
        } else {
            quads(&[55, 77])
        };
        for n in 0..=body.len() {
            silent(&h.run(command, &body[..n]));
            let sale = if n >= 8 { 55 } else { 0 };
            let expected = match command {
                1539 => MarketOperation::RegisterBid {
                    sale,
                    price: if n >= 16 { 77 } else { 0 },
                },
                1545 => MarketOperation::UpdateBidPrice {
                    sale,
                    bid: if n >= 16 { 66 } else { 0 },
                    price: if n >= 24 { 77 } else { 0 },
                },
                _ => MarketOperation::RegisterPurchase {
                    sale,
                    price: if n >= 16 { 77 } else { 0 },
                },
            };
            assert_eq!(h.last_request().operation, expected);
        }
    }
}

#[test]
fn search_decode_and_all_type_rejections() {
    let mut h = H::new();
    let body = search();
    for n in 0..body.len() {
        assert_eq!(replies(&h.run(1551, &body[..n])), reason(1553, 2));
    }
    for index in 0..4 {
        let max = if index == 0 { 3 } else { 2 };
        for invalid in [0, max + 1, 0x80, 0xff] {
            let mut body = search();
            body[index] = invalid;
            assert_eq!(replies(&h.run(1551, &body)), reason(1553, 1955));
        }
    }
    assert!(h.services.market.calls.is_empty());
}

#[test]
fn search_arena_a_user_and_send_replies() {
    let mut h = H::new();
    let body = search();
    h.world.arena = true;
    assert_eq!(replies(&h.run(1551, &[])), reason(1553, 5));
    h.world.arena = false;
    h.pc().arena_state = 3;
    h.pc().t2_blocked = true;
    silent(&h.run(1551, &body));
    assert_eq!(
        h.last_request().operation,
        MarketOperation::SearchItem {
            search_type: 3,
            order_type: 2,
            price_type: 1,
            sale_type: 2,
            item: -7,
            first: 11,
            second: 22,
            third: 33,
        }
    );
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(1551, &body)), reason(1553, 2));
    h.pc().user_present = true;
    h.pc().party_user_blocked = true;
    assert_eq!(replies(&h.run(1551, &body)), reason(1553, 2));
    h.pc().party_user_blocked = false;
    h.services.market.submit_result = Gate::Fail;
    assert_eq!(replies(&h.run(1551, &body)), reason(1553, 2));
    h.services.market.submit_result = Gate::Unknown;
    unsupported(&h.run(1551, &body));
}

#[test]
fn lowest_price_undefined_reply_tail_is_not_fabricated() {
    let mut h = H::new();
    h.world.arena = true;
    let out = h.run(1554, &[]);
    unsupported(&out);
    assert!(matches!(out.status, Status::Unsupported(s) if s.contains("reason 5")));
    h.world.arena = false;
    for n in 0..4 {
        let out = h.run(1554, &[0; 4][..n]);
        unsupported(&out);
        assert!(matches!(out.status, Status::Unsupported(s) if s.contains("reason 2")));
    }
    h.pc().user_present = false;
    unsupported(&h.run(1554, &[0; 4]));
    h.pc().user_present = true;
    h.pc().party_user_blocked = true;
    unsupported(&h.run(1554, &[0; 4]));
    h.pc().party_user_blocked = false;
    h.services.market.submit_result = Gate::Fail;
    unsupported(&h.run(1554, &[0; 4]));
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn lowest_price_success_uses_arena_a_only() {
    let mut h = H::new();
    h.pc().arena_state = 6;
    h.pc().t2_blocked = true;
    h.pc().zone = None;
    silent(&h.run(1554, &(-77i32).to_le_bytes()));
    assert_eq!(
        h.last_request().operation,
        MarketOperation::LowestPrice { item: -77 }
    );
    h.services.market.submit_result = Gate::Unknown;
    unsupported(&h.run(1554, &[0; 4]));
}

#[test]
fn unknown_market_configuration_and_submission_do_not_approve() {
    let mut h = H::new();
    h.services.market = MemoryMarket::default();
    unsupported(&h.run(1530, &sale()));
    unsupported(&h.run(1539, &quads(&[1, 2])));
    unsupported(&h.run(1545, &quads(&[1, 2, 3])));
    unsupported(&h.run(1548, &quads(&[1, 2])));
    unsupported(&h.run(381, &[]));
    assert!(h.services.skill_tasks.is_empty());
}
