//! Doc 12 part 1: profession stubs, PC trade and account inventory.
use gamedaemon::handlers::*;
use gamedaemon::Position;

const A: u64 = (1 << 48) | 0x11;
const B: u64 = (1 << 48) | 0x22;

struct H {
    session: Session,
    world: World,
    services: MemoryServices,
}
impl H {
    fn new() -> Self {
        let pc = Pc {
            object_id: A,
            db_id: 0x7700,
            zone: Some(9),
            user_present: true,
            ..Pc::default()
        };
        let world = World {
            account_inventory_enabled: true,
            utc_seconds: 1000,
            ..World::default()
        };
        let mut services = MemoryServices::default();
        services.trade.queue_result = Gate::Pass;
        services.trade.account_result = Gate::Pass;
        let mut state = TradeState {
            offer_max: 1_000_000,
            money_max: 2_000_000,
            ..TradeState::default()
        };
        for (id, x) in [(A, 0), (B, 10)] {
            state.players.insert(
                id,
                TradePlayer {
                    object_id: id,
                    zone: Some(9),
                    position: Position { x, y: 0, z: 0 },
                    money: 5000,
                    usable_limit: 40,
                    bag_slots: 40,
                    ..TradePlayer::default()
                },
            );
        }
        services.trade.state = Some(state);
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
    fn st(&mut self) -> &mut TradeState {
        self.services.trade.state.as_mut().unwrap()
    }
    fn p(&mut self, id: u64) -> &mut TradePlayer {
        self.st().players.get_mut(&id).unwrap()
    }
    fn run(&mut self, command: u16, body: &[u8]) -> Outcome {
        let mut ctx = Context {
            session: &mut self.session,
            world: &self.world,
            services: &mut self.services,
        };
        dispatch(&mut ctx, command, body)
    }
    fn task(&mut self, caster: u64, kind: TradeTaskKind) -> Outcome {
        let task = TradeTask {
            command: 0,
            caster,
            zone: 9,
            kind,
        };
        let mut ctx = Context {
            session: &mut self.session,
            world: &self.world,
            services: &mut self.services,
        };
        execute_trade_task(&mut ctx, &task)
    }
    /// Invite + join: both states 1.
    fn open(&mut self) -> u64 {
        assert!(self
            .task(A, TradeTaskKind::Invite { guest: B })
            .replies
            .is_empty());
        let out = self.task(B, TradeTaskKind::Join { inviter: A });
        assert!(out.replies.is_empty(), "{out:?}");
        self.st().next_trade
    }
}
fn reason(out: &Outcome) -> Option<(u16, u16)> {
    assert!(out.replies.len() <= 1, "{out:?}");
    out.replies
        .first()
        .map(|m| (m.command, u16::from_le_bytes([m.body[0], m.body[1]])))
}
fn sends(out: &Outcome) -> Vec<(u64, u16, Vec<u8>)> {
    out.trade_effects
        .iter()
        .filter_map(|e| match e {
            TradeEffect::Send { to, message } => Some((*to, message.command, message.body.clone())),
            _ => None,
        })
        .collect()
}
fn q(v: u64) -> Vec<u8> {
    v.to_le_bytes().to_vec()
}

#[test]
fn profession_stubs_do_nothing() {
    let mut h = H::new();
    for c in [121, 123] {
        assert_eq!(h.run(c, &[1, 2, 3]), Outcome::default());
    }
}

#[test]
fn request_gates_and_queue() {
    let mut h = H::new();
    let out = h.run(311, &q(B));
    assert!(out.replies.is_empty());
    assert_eq!(
        h.services.trade.tasks[0].kind,
        TradeTaskKind::Invite { guest: B }
    );
    // T1 → 5 before decode.
    h.pc().arena_state = 3;
    assert_eq!(reason(&h.run(311, &[])), Some((312, 5)));
    h.pc().arena_state = 0;
    // T2 → 7 for 311..330, not for 333/336/339.
    h.pc().t2_blocked = true;
    for (c, r) in [
        (311, 312),
        (314, 315),
        (317, 318),
        (320, 321),
        (324, 323),
        (327, 323),
        (330, 323),
    ] {
        assert_eq!(reason(&h.run(c, &[0; 16])), Some((r, 7)), "{c}");
    }
    for c in [333, 336, 339] {
        assert!(h.run(c, &[]).replies.is_empty(), "{c}");
    }
    h.pc().t2_blocked = false;
    // Short body: boundary.
    assert!(matches!(h.run(311, &[1]).status, Status::Unsupported(_)));
    // Zone null → 2, queue failure → 2.
    h.pc().zone = None;
    assert_eq!(reason(&h.run(333, &[])), Some((334, 2)));
    h.pc().zone = Some(9);
    h.services.trade.queue_result = Gate::Fail;
    assert_eq!(reason(&h.run(336, &[])), Some((337, 2)));
    // Decoded fields.
    h.services.trade.queue_result = Gate::Pass;
    h.run(317, &[q(B), vec![7]].concat());
    h.run(324, &[3, 0, 0xFF, 0xFF]);
    h.run(330, &q(u64::MAX));
    let kinds: Vec<_> = h
        .services
        .trade
        .tasks
        .iter()
        .map(|t| t.kind.clone())
        .collect();
    assert!(kinds.contains(&TradeTaskKind::Decline {
        inviter: B,
        decline: 7
    }));
    assert!(kinds.contains(&TradeTaskKind::Append { slot: 3, count: -1 }));
    assert!(kinds.contains(&TradeTaskKind::Money { amount: -1 }));
}

#[test]
fn invite_order_and_success() {
    let mut h = H::new();
    let inv = |g| TradeTaskKind::Invite { guest: g };
    assert_eq!(reason(&h.task(A, inv(0x22))), Some((312, 907)));
    assert_eq!(reason(&h.task(A, inv(A))), Some((312, 907)));
    h.p(B).zone = Some(8);
    assert_eq!(reason(&h.task(A, inv(B))), Some((312, 904)));
    h.p(B).zone = Some(9);
    h.p(B).t2_blocked = true;
    assert_eq!(reason(&h.task(A, inv(B))), Some((312, 8)));
    h.p(B).t2_blocked = false;
    h.st().timed_gate = true;
    h.p(A).timed_window = Some((1000, 1000));
    assert_eq!(reason(&h.task(A, inv(B))), Some((312, 2)));
    h.p(A).timed_window = None;
    h.p(B).timed_window = Some((900, 1000));
    assert_eq!(reason(&h.task(A, inv(B))), Some((312, 911)));
    h.p(B).timed_window = None;
    h.p(A).busy = true;
    assert_eq!(reason(&h.task(A, inv(B))), Some((312, 906)));
    h.p(A).busy = false;
    h.p(B).trade = Some(99);
    assert_eq!(reason(&h.task(A, inv(B))), Some((312, 909)));
    h.p(B).trade = None;
    h.p(B).position = Position { x: 63, y: 0, z: 0 };
    assert_eq!(reason(&h.task(A, inv(B))), Some((312, 908)));
    h.p(B).position = Position { x: 62, y: 0, z: 0 };
    h.st().restriction = Some(RestrictionRecord {
        active: true,
        level: 10,
        required: 0,
    });
    assert_eq!(reason(&h.task(A, inv(B))), Some((312, 2)));
    h.st().restriction = None;
    let out = h.task(A, inv(B));
    let body = [q(A), q(B)].concat();
    assert_eq!(sends(&out), vec![(A, 313, body.clone()), (B, 313, body)]);
    assert_eq!(h.p(A).trade, Some(1));
    assert_eq!(reason(&h.task(A, inv(B))), Some((312, 905)));
}

#[test]
fn join_and_decline() {
    let mut h = H::new();
    h.task(A, TradeTaskKind::Invite { guest: B });
    assert_eq!(
        reason(&h.task(B, TradeTaskKind::Join { inviter: 0x11 })),
        Some((315, 915))
    );
    // Inviter must be in the caller zone.
    h.p(B).zone = Some(8);
    assert_eq!(
        reason(&h.task(B, TradeTaskKind::Join { inviter: A })),
        Some((315, 915))
    );
    h.p(B).zone = Some(9);
    h.p(B).busy = true;
    assert_eq!(
        reason(&h.task(B, TradeTaskKind::Join { inviter: A })),
        Some((315, 914))
    );
    h.p(B).busy = false;
    let out = h.task(B, TradeTaskKind::Join { inviter: A });
    assert_eq!(sends(&out).len(), 2);
    assert_eq!(h.st().trades[&1].states, [1, 1]);
    // Joined: inviter state ≠ 0 → decline 919.
    let out = h.task(
        B,
        TradeTaskKind::Decline {
            inviter: A,
            decline: 4,
        },
    );
    assert_eq!(reason(&out), Some((318, 919)));

    let mut h = H::new();
    h.task(A, TradeTaskKind::Invite { guest: B });
    let out = h.task(
        B,
        TradeTaskKind::Decline {
            inviter: A,
            decline: 200,
        },
    );
    let body = [q(A), q(B), vec![200]].concat();
    assert_eq!(sends(&out), vec![(A, 319, body.clone()), (B, 319, body)]);
    assert!(h.st().trades.is_empty());
    assert_eq!(h.p(A).trade, None);
}

#[test]
fn decline_clears_guest_link_to_another_trade() {
    let mut h = H::new();
    h.task(A, TradeTaskKind::Invite { guest: B });
    let t = h.st().next_trade;
    let u = t + 1;
    let other = h.st().trades[&t].clone();
    h.st().trades.insert(u, other);
    h.p(B).trade = Some(u);
    let out = h.task(
        B,
        TradeTaskKind::Decline {
            inviter: A,
            decline: 200,
        },
    );
    assert!(out.replies.is_empty());
    let body = [q(A), q(B), vec![200]].concat();
    assert_eq!(sends(&out), vec![(A, 319, body.clone()), (B, 319, body)]);
    assert_eq!(
        out.trade_effects.last(),
        Some(&TradeEffect::Unlinked { trade: t })
    );
    assert!(!h.st().trades.contains_key(&t));
    assert!(h.st().trades.contains_key(&u));
    assert_eq!(h.p(A).trade, None);
    assert_eq!(h.p(B).trade, None);
}

#[test]
fn suggestions_allow_inviter_link_to_another_trade() {
    let mut h = H::new();
    let t = h.open();
    let u = t + 1;
    let other = h.st().trades[&t].clone();
    h.st().trades.insert(u, other);
    h.p(A).trade = Some(u);
    assert_eq!(h.p(B).trade, Some(t));
    let out = h.task(B, TradeTaskKind::Money { amount: 500 });
    assert!(out.replies.is_empty());
    assert!(out.logs.is_empty());
    assert_eq!(h.st().trades[&t].offers, [0, 500]);
    assert_eq!(h.st().trades[&u].offers, [0, 0]);
    assert_eq!(
        sends(&out),
        vec![(B, 332, q(500)), (A, 331, [q(B), q(500)].concat())]
    );
    // Append and remove must also pass the suggestion link gate.
    assert_eq!(
        reason(&h.task(B, TradeTaskKind::Append { slot: 3, count: 0 })),
        Some((323, 926))
    );
    assert_eq!(
        reason(&h.task(B, TradeTaskKind::Remove { slot: 1 })),
        Some((323, 931))
    );
    // Automatic retract keeps its stronger equality check.
    h.st().trades.get_mut(&t).unwrap().states = [2, 1];
    let out = h.task(B, TradeTaskKind::Money { amount: 0 });
    assert_eq!(h.st().trades[&t].states, [2, 1]);
    assert!(sends(&out).iter().all(|(_, command, _)| *command != 338));
    // A null inviter link still fails the suggestion check.
    h.p(A).trade = None;
    let out = h.task(B, TradeTaskKind::Money { amount: 1 });
    assert!(sends(&out).is_empty());
    assert!(!out.logs.is_empty());
    assert_eq!(h.st().trades[&t].offers, [0, 0]);
}

#[test]
fn money_submit_retract_confirm_money_only() {
    let mut h = H::new();
    h.open();
    let money = |a| TradeTaskKind::Money { amount: a };
    assert_eq!(reason(&h.task(A, money(-1))), Some((323, 930)));
    assert_eq!(reason(&h.task(A, money(5001))), Some((323, 926)));
    h.p(B).money = 1_999_500;
    assert_eq!(reason(&h.task(A, money(501))), Some((323, 928)));
    let out = h.task(A, money(500));
    assert_eq!(
        sends(&out),
        vec![(A, 332, q(500)), (B, 331, [q(A), q(500)].concat())]
    );
    h.p(B).money = 100;
    // Submit, then peer money suggestion auto-retracts with Q = 1.
    assert!(h.task(A, TradeTaskKind::Submit).replies.is_empty());
    assert_eq!(reason(&h.task(A, TradeTaskKind::Submit)), Some((334, 934)));
    let out = h.task(B, money(0));
    assert_eq!(sends(&out)[0], (A, 338, [q(B), vec![1]].concat()));
    assert_eq!(h.st().trades[&1].states, [1, 1]);
    // Explicit retract: Q = 0.
    h.task(A, TradeTaskKind::Submit);
    assert_eq!(reason(&h.task(B, TradeTaskKind::Retract)), Some((337, 938)));
    let out = h.task(A, TradeTaskKind::Retract);
    assert_eq!(sends(&out)[1], (B, 338, [q(A), vec![0]].concat()));
    // Confirm twice: money moves.
    h.task(A, TradeTaskKind::Submit);
    assert_eq!(reason(&h.task(A, TradeTaskKind::Confirm)), Some((340, 941)));
    h.task(B, TradeTaskKind::Submit);
    let out = h.task(A, TradeTaskKind::Confirm);
    assert_eq!(sends(&out), vec![(A, 341, q(A)), (B, 341, q(A))]);
    let out = h.task(B, TradeTaskKind::Confirm);
    assert_eq!(h.p(A).money, 4500);
    assert_eq!(h.p(B).money, 600);
    assert_eq!(
        out.trade_effects[..2],
        [
            TradeEffect::MoneyChanged {
                pc: A,
                delta: -500,
                reason: 551
            },
            TradeEffect::MoneyChanged {
                pc: B,
                delta: 500,
                reason: 151
            },
        ]
    );
    assert_eq!(
        out.trade_effects.last(),
        Some(&TradeEffect::Unlinked { trade: 1 })
    );
    assert_eq!(h.p(B).trade, None);
    // No trade: 321 with the task reason.
    assert_eq!(reason(&h.task(A, TradeTaskKind::Confirm)), Some((321, 940)));
    assert_eq!(reason(&h.task(A, TradeTaskKind::Submit)), Some((321, 933)));
    assert_eq!(reason(&h.task(A, money(1))), Some((323, 924)));
}

#[test]
fn confirm_final_check_cancels() {
    let mut h = H::new();
    h.open();
    h.task(A, TradeTaskKind::Money { amount: 300 });
    h.task(A, TradeTaskKind::Submit);
    h.task(B, TradeTaskKind::Submit);
    h.task(A, TradeTaskKind::Confirm);
    // Inviter money now below offer: cancel actor inviter, byte 5.
    h.p(A).money = 299;
    let out = h.task(B, TradeTaskKind::Confirm);
    let body = [q(A), vec![5]].concat();
    assert_eq!(sends(&out), vec![(A, 322, body.clone()), (B, 322, body)]);
    assert!(h.st().trades.is_empty());
}

#[test]
fn cancel_and_escrow_boundaries() {
    let mut h = H::new();
    h.open();
    h.p(B).escrow[0] = Some(EscrowItem { native_ok: true });
    assert!(matches!(
        h.task(A, TradeTaskKind::Cancel).status,
        Status::Unsupported(_)
    ));
    assert_eq!(
        reason(&h.task(A, TradeTaskKind::Remove { slot: 1 })),
        Some((323, 931))
    );
    assert_eq!(
        reason(&h.task(A, TradeTaskKind::Remove { slot: 9 })),
        Some((323, 931))
    );
    assert!(matches!(
        h.task(B, TradeTaskKind::Remove { slot: 1 }).status,
        Status::Unsupported(_)
    ));
    assert_eq!(
        reason(&h.task(A, TradeTaskKind::Append { slot: 3, count: 0 })),
        Some((323, 926))
    );
    h.p(B).usable_limit = 0;
    assert_eq!(
        reason(&h.task(A, TradeTaskKind::Append { slot: 3, count: 1 })),
        Some((323, 927))
    );
    h.p(B).escrow[0] = None;
    let out = h.task(B, TradeTaskKind::Cancel);
    let body = [q(B), vec![1]].concat();
    assert_eq!(sends(&out), vec![(A, 322, body.clone()), (B, 322, body)]);
    assert_eq!(reason(&h.task(B, TradeTaskKind::Cancel)), Some((321, 920)));
}

#[test]
fn account_inventory() {
    let mut h = H::new();
    assert!(h.run(793, &[]).replies.is_empty());
    assert_eq!(
        h.services.trade.transactions[0].kind,
        AccountTransactionKind::GoodsCount
    );
    assert_eq!(reason(&h.run(796, &[0x80])), Some((798, 2)));
    h.run(805, &[[2].as_slice(), &q(77), &[1]].concat());
    assert_eq!(
        h.services.trade.transactions[1].kind,
        AccountTransactionKind::WarehouseList {
            selector: 2,
            token: 77,
            list: 1
        }
    );
    // 802 failure echoes selector and token.
    h.world.arena = true;
    let out = h.run(802, &[[3].as_slice(), &q(55)].concat());
    assert_eq!(out.replies[0].command, 804);
    assert_eq!(out.replies[0].body, [vec![5, 0, 3], q(55)].concat());
    h.world.arena = false;
    h.world.account_inventory_enabled = false;
    assert_eq!(reason(&h.run(799, &[0; 9])), Some((801, 2)));
    h.world.account_inventory_enabled = true;
    h.pc().user_present = false;
    let out = h.run(793, &[]);
    assert!(out.replies.is_empty() && !out.logs.is_empty());
    h.pc().user_present = true;
    h.services.trade.account_result = Gate::Fail;
    assert_eq!(reason(&h.run(793, &[])), Some((795, 2)));
}
