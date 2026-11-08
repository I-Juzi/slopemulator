//! Doc 23: post entry functions, build 381.8867.
use gamedaemon::handlers::*;

const PC_ID: u64 = (1 << 48) | 0x11;
const POST_ID: u64 = 0x8877_6655_4433_2211;

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
            routing: 123,
            user_present: true,
            ..Pc::default()
        };
        let services = MemoryServices {
            post: MemoryPost {
                cached_result: PostResult::Submitted,
                register_result: PostResult::Submitted,
                money_limit: Some(1000),
                ..MemoryPost::default()
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
    fn calls(&self) -> &[PostOperation] {
        &self.services.post.calls
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
fn reason_key(command: u16, code: u16, kind: u8, id: u64) -> Vec<(u16, Vec<u8>)> {
    let mut body = code.to_le_bytes().to_vec();
    body.push(kind);
    body.extend(id.to_le_bytes());
    vec![(command, body)]
}
fn handled(out: Outcome) {
    assert_eq!(out.status, Status::Handled);
    assert!(out.replies.is_empty());
    assert!(out.changes.is_empty());
}
fn unsupported(out: Outcome) {
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert!(out.replies.is_empty());
}
fn key() -> Vec<u8> {
    let mut body = vec![0xA5];
    body.extend(POST_ID.to_le_bytes());
    body
}
fn detach() -> Vec<u8> {
    let mut body = key();
    body.extend(91u64.to_le_bytes());
    body.extend(92u64.to_le_bytes());
    body
}
fn string(body: &mut Vec<u8>, text: &[u16]) {
    body.extend((text.len() as u16).to_le_bytes());
    for c in text {
        body.extend(c.to_le_bytes());
    }
}
fn prefix(money: i64) -> Vec<u8> {
    let mut body = vec![2];
    body.extend(12345u64.to_le_bytes());
    body.extend(0x8012u16.to_le_bytes());
    string(&mut body, &[0x41, 0xD83D, 0xDE00]);
    string(&mut body, &[0x42]);
    string(&mut body, &[0x43, 0, 0x44]);
    body.extend(money.to_le_bytes());
    body
}
fn list(body: &mut Vec<u8>, items: &[(u16, u16)]) {
    body.extend((items.len() as u16).to_le_bytes());
    for (first, second) in items {
        body.extend(first.to_le_bytes());
        body.extend(second.to_le_bytes());
    }
}
fn deliver(money: i64, attached: &[(u16, u16)], charged: &[(u16, u16)]) -> Vec<u8> {
    let mut body = prefix(money);
    list(&mut body, attached);
    list(&mut body, charged);
    body
}

#[test]
fn count_and_brief_arena_precedes_user() {
    for (command, failure) in [(368, 370), (371, 373)] {
        for state in [2, 3, 5, 6] {
            let mut h = H::new();
            h.pc().arena_state = state;
            h.pc().user_present = false;
            assert_eq!(replies(&h.run(command, &[])), reason(failure, 5));
            assert!(h.calls().is_empty());
        }
        let mut h = H::new();
        h.world.arena = true;
        assert_eq!(replies(&h.run(command, &[])), reason(failure, 5));
    }
}

#[test]
fn count_and_brief_cached_fields_and_no_extra_gates() {
    for (command, backend) in [(368, 69), (371, 72)] {
        let mut h = H::new();
        h.pc().t2_blocked = true; // Neither entry checks trial play.
        h.pc().zone = None;
        h.pc().native_kind = 2;
        handled(h.run(command, &[0xFF])); // Neither entry decodes the request.
        assert_eq!(
            h.calls(),
            &[PostOperation::Cached {
                command: backend,
                routing: 123,
                object_id: PC_ID,
                post: None,
            }]
        );
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn cached_missing_user_or_blocked_user_is_log_only() {
    for (command, body) in [(368, vec![]), (371, vec![]), (374, key()), (1500, key())] {
        let mut h = H::new();
        h.pc().user_present = false;
        let out = h.run(command, &body);
        assert!(!out.logs.is_empty());
        handled(out);
        h.pc().user_present = true;
        h.pc().party_user_blocked = true;
        handled(h.run(command, &body));
        assert!(h.calls().is_empty());
    }
}

#[test]
fn cached_unknown_invalid_db_id_and_ignored_send_failure() {
    for (command, body) in [(368, vec![]), (371, vec![]), (374, key()), (1500, key())] {
        let mut h = H::new();
        h.services.post.cached_result = PostResult::Unknown;
        unsupported(h.run(command, &body));
        h.services.post.cached_result = PostResult::InvalidUserDbId;
        let out = h.run(command, &body);
        assert_eq!(out.logs, ["post: cached user DB ID is invalid"]);
        handled(out);
        h.services.post.cached_result = PostResult::Rejected;
        handled(h.run(command, &body));
    }
}

#[test]
fn retrieve_and_archive_cached_request_fields() {
    for (command, backend) in [(374, 75), (1500, 93)] {
        let mut h = H::new();
        let mut body = key();
        body.extend([1, 2, 3]); // Trailing bytes do not fail the decode.
        handled(h.run(command, &body));
        assert_eq!(
            h.calls(),
            &[PostOperation::Cached {
                command: backend,
                routing: 123,
                object_id: PC_ID,
                post: Some(PostKey {
                    kind: 0xA5,
                    id: POST_ID,
                }),
            }]
        );
    }
}

#[test]
fn retrieve_and_archive_unset_reply_tails_are_not_fabricated() {
    let mut h = H::new();
    h.world.arena = true;
    for command in [374, 1500] {
        unsupported(h.run(command, &key()));
        unsupported(h.run(command, &[]));
    }
    h.world.arena = false;
    h.pc().t2_blocked = true;
    unsupported(h.run(1500, &key()));
    handled(h.run(374, &key())); // Retrieve has no trial gate.
    assert_eq!(h.calls().len(), 1);
}

#[test]
fn retrieve_and_archive_unchecked_short_requests() {
    for command in [374, 1500] {
        let mut h = H::new();
        for n in 0..key().len() {
            unsupported(h.run(command, &key()[..n]));
        }
        assert!(h.calls().is_empty());
        // Decode failure is ignored natively. Missing user is still a known path.
        h.pc().user_present = false;
        handled(h.run(command, &[]));
    }
}

#[test]
fn db_id_reply_is_counted_blob_and_ignores_other_gates() {
    let mut h = H::new();
    h.world.arena = true;
    h.pc().t2_blocked = true;
    h.pc().user_present = false;
    h.pc().zone = None;
    h.pc().db_id = 0xFEDC_BA98_7654_3210;
    let mut body = 8u16.to_le_bytes().to_vec();
    body.extend(0xFEDC_BA98_7654_3210u64.to_le_bytes());
    assert_eq!(replies(&h.run(785, &[0xFF])), vec![(786, body)]);
    assert!(h.calls().is_empty());
}

#[test]
fn mark_express_replies_follow_native_order_and_echo_key() {
    let mut h = H::new();
    h.world.arena = true;
    h.pc().t2_blocked = true;
    h.pc().user_present = false;
    assert_eq!(
        replies(&h.run(1503, &key())),
        reason_key(1505, 5, 0xA5, POST_ID)
    );
    h.world.arena = false;
    assert_eq!(
        replies(&h.run(1503, &key())),
        reason_key(1505, 7, 0xA5, POST_ID)
    );
    h.pc().t2_blocked = false;
    handled(h.run(1503, &key()));
    assert!(h.calls().is_empty());
    h.pc().user_present = true;
    h.services.post.register_result = PostResult::Rejected;
    assert_eq!(
        replies(&h.run(1503, &key())),
        reason_key(1505, 2, 0xA5, POST_ID)
    );
    assert_eq!(
        h.calls(),
        &[PostOperation::Register {
            object_id: PC_ID,
            task: PostTask::MarkExpress
        }]
    );
}

#[test]
fn mark_express_short_decode_never_supplies_reply_defaults() {
    let mut h = H::new();
    for n in 0..key().len() {
        unsupported(h.run(1503, &key()[..n]));
        h.world.arena = true;
        unsupported(h.run(1503, &key()[..n]));
        h.world.arena = false;
    }
    assert!(h.calls().is_empty());
    h.pc().user_present = false;
    handled(h.run(1503, &[]));
}

#[test]
fn detach_decode_precedes_arena_and_preserves_partial_key() {
    let mut h = H::new();
    h.world.arena = true;
    h.pc().t2_blocked = true;
    h.pc().user_present = false;
    for n in 0..detach().len() {
        let kind = if n >= 1 { 0xA5 } else { 0 };
        let id = if n >= 9 { POST_ID } else { 0 };
        assert_eq!(
            replies(&h.run(1506, &detach()[..n])),
            reason_key(1508, 2, kind, id),
            "prefix {n}"
        );
    }
    assert_eq!(
        replies(&h.run(1506, &detach())),
        reason_key(1508, 5, 0xA5, POST_ID)
    );
    h.world.arena = false;
    assert_eq!(
        replies(&h.run(1506, &detach())),
        reason_key(1508, 7, 0xA5, POST_ID)
    );
    h.pc().t2_blocked = false;
    handled(h.run(1506, &detach()));
    assert!(h.calls().is_empty());
}

#[test]
fn detach_registration_failure_has_exact_body() {
    let mut h = H::new();
    h.services.post.register_result = PostResult::Rejected;
    assert_eq!(
        replies(&h.run(1506, &detach())),
        reason_key(1508, 2, 0xA5, POST_ID)
    );
    assert_eq!(
        h.calls(),
        &[PostOperation::Register {
            object_id: PC_ID,
            task: PostTask::DetachAttachment
        }]
    );
}

#[test]
fn account_transaction_submitted_and_unknown_are_not_zone_tasks() {
    for (command, body) in [(1503, key()), (1506, detach())] {
        let mut h = H::new();
        h.pc().zone = None;
        h.pc().native_kind = 2;
        handled(h.run(command, &body));
        h.services.post.register_result = PostResult::Unknown;
        unsupported(h.run(command, &body));
        assert!(h.services.skill_tasks.is_empty());
        h.pc().party_user_blocked = true;
        let before = h.calls().len();
        handled(h.run(command, &body));
        assert_eq!(h.calls().len(), before);
    }
}

#[test]
fn deliver_arena_trial_and_user_precede_decode() {
    let mut h = H::new();
    h.world.arena = true;
    h.pc().t2_blocked = true;
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(1509, &[])), reason(1511, 5));
    h.world.arena = false;
    assert_eq!(replies(&h.run(1509, &[])), reason(1511, 7));
    h.pc().t2_blocked = false;
    assert_eq!(h.run(1509, &[]).logs, ["deliver post: user is not found"]);
    h.pc().user_present = true;
    h.pc().party_user_blocked = true;
    handled(h.run(1509, &[]));
    assert!(h.calls().is_empty());
}

#[test]
fn deliver_prefix_decode_failures_are_log_only() {
    let mut h = H::new();
    let body = prefix(0);
    for n in 0..body.len() {
        let out = h.run(1509, &body[..n]);
        assert_eq!(out.logs, ["deliver post: invalid packet"], "prefix {n}");
        handled(out);
    }
    assert!(h.calls().is_empty());
}

#[test]
fn deliver_string_capacity_and_negative_length_are_log_only() {
    for (index, capacity) in [(0, 33), (1, 65), (2, 1025)] {
        for count in [capacity, 0xFFFF] {
            let mut body = vec![2];
            body.extend(1u64.to_le_bytes());
            body.extend(0u16.to_le_bytes());
            for _ in 0..index {
                string(&mut body, &[]);
            }
            body.extend((count as u16).to_le_bytes());
            let mut h = H::new();
            let out = h.run(1509, &body);
            assert_eq!(out.logs, ["deliver post: invalid packet"]);
            handled(out);
            assert!(h.calls().is_empty());
        }
    }
}

#[test]
fn deliver_attached_count_errors_are_1287() {
    for count in [
        vec![],
        vec![1],
        4u16.to_le_bytes().to_vec(),
        0xFFFFu16.to_le_bytes().to_vec(),
    ] {
        let mut h = H::new();
        let mut body = prefix(0);
        body.extend(count);
        assert_eq!(replies(&h.run(1509, &body)), reason(1511, 1287));
        assert!(h.calls().is_empty());
    }
}

#[test]
fn deliver_attached_truncation_is_silent_and_duplicates_precede_charge_count() {
    let mut h = H::new();
    for n in 0..4 {
        let mut body = prefix(0);
        body.extend(1u16.to_le_bytes());
        body.extend(&[1, 0, 2, 0][..n]);
        let out = h.run(1509, &body);
        assert_eq!(out.logs, ["deliver post: invalid packet"]);
        handled(out);
    }
    let mut body = prefix(0);
    list(&mut body, &[(0xFFFF, 1), (0xFFFF, 2)]);
    // No charge count. Duplicate attached positions still select 1292, not a count read.
    assert_eq!(replies(&h.run(1509, &body)), reason(1511, 1292));
    assert!(h.calls().is_empty());
}

#[test]
fn deliver_charge_count_and_duplicate_errors_are_1292() {
    let mut h = H::new();
    for count in [
        vec![],
        vec![1],
        7u16.to_le_bytes().to_vec(),
        0xFFFFu16.to_le_bytes().to_vec(),
    ] {
        let mut body = prefix(0);
        list(&mut body, &[]);
        body.extend(count);
        assert_eq!(replies(&h.run(1509, &body)), reason(1511, 1292));
    }
    let body = deliver(-1, &[], &[(5, 1), (5, 2)]);
    // Duplicate check precedes negative money.
    assert_eq!(replies(&h.run(1509, &body)), reason(1511, 1292));
    assert!(h.calls().is_empty());
}

#[test]
fn deliver_charge_pair_truncation_is_log_only() {
    let mut h = H::new();
    for n in 0..4 {
        let mut body = prefix(0);
        list(&mut body, &[]);
        body.extend(1u16.to_le_bytes());
        body.extend(&[1, 0, 2, 0][..n]);
        let out = h.run(1509, &body);
        assert_eq!(out.logs, ["deliver post: invalid packet"]);
        handled(out);
    }
    assert!(h.calls().is_empty());
}

#[test]
fn deliver_money_reply_33_and_unknown_limit() {
    let mut h = H::new();
    h.services.post.money_limit = None;
    assert_eq!(
        replies(&h.run(1509, &deliver(-1, &[], &[]))),
        reason(1511, 33)
    );
    assert!(h.calls().is_empty()); // Negative amount short-circuits the global read.
    unsupported(h.run(1509, &deliver(0, &[], &[])));
    assert_eq!(h.calls(), &[PostOperation::MoneyLimit]);
    h.services.post.money_limit = Some(1000);
    assert_eq!(
        replies(&h.run(1509, &deliver(1001, &[], &[]))),
        reason(1511, 33)
    );
    h.services.post.money_limit = Some(-1);
    assert_eq!(
        replies(&h.run(1509, &deliver(0, &[], &[]))),
        reason(1511, 33)
    );
}

#[test]
fn deliver_registration_reply_2_and_unknown_boundary() {
    let mut h = H::new();
    h.services.post.register_result = PostResult::Rejected;
    let body = deliver(0, &[], &[]);
    assert_eq!(replies(&h.run(1509, &body)), reason(1511, 2));
    assert!(matches!(
        h.calls(),
        [
            PostOperation::MoneyLimit,
            PostOperation::Register {
                task: PostTask::Deliver(_),
                ..
            }
        ]
    ));
    h.services.post.register_result = PostResult::Unknown;
    unsupported(h.run(1509, &body));
}

#[test]
fn deliver_retains_fields_and_pairs_without_comparing_between_lists() {
    let mut h = H::new();
    h.pc().zone = None;
    h.pc().native_kind = 2;
    let attached = [(1, 9), (2, 9), (0xFFFF, 9)];
    let charged = [(1, 9), (2, 9), (3, 9), (4, 9), (5, 9), (0xFFFF, 9)];
    let mut body = deliver(1000, &attached, &charged);
    body.extend([0xFF, 0xFF]);
    handled(h.run(1509, &body));
    let PostOperation::Register {
        object_id,
        task: PostTask::Deliver(delivery),
    } = h.calls().last().unwrap()
    else {
        panic!("missing delivery transaction");
    };
    assert_eq!(*object_id, PC_ID);
    assert_eq!(
        **delivery,
        PostDelivery {
            kind: 2,
            reference: 12345,
            value: 0x8012,
            receiver: vec![0x41, 0xD83D, 0xDE00],
            subject: vec![0x42],
            content: vec![0x43, 0, 0x44],
            money: 1000,
            attached: attached.to_vec(),
            charged: charged.to_vec(),
        }
    );
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn post_dispatch_keeps_authentication_and_rights_preconditions() {
    assert_eq!(POST_COMMANDS, [368, 371, 374, 785, 1500, 1503, 1506, 1509]);
    for command in POST_COMMANDS {
        let mut h = H::new();
        h.session.authenticated = false;
        let out = h.run(*command, &[]);
        assert_eq!(out.logs, ["missing authenticated PC"]);
        assert!(out.replies.is_empty());
        h.session.authenticated = true;
        h.session.dispatch_permission = None;
        unsupported(h.run(*command, &[]));
        assert!(h.calls().is_empty());
    }
}

#[test]
fn default_post_service_results_are_unknown() {
    let post = MemoryPost::default();
    assert_eq!(post.cached_result, PostResult::Unknown);
    assert_eq!(post.register_result, PostResult::Unknown);
    assert_eq!(post.money_limit, None);
    assert!(post.calls.is_empty());
}
