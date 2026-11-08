//! Doc 21: quest progress entry functions. Build 381.8867.
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
            job: 6,
            db_id: 77,
            user_present: true,
            ..Pc::default()
        };
        let services = MemoryServices {
            skill_queue_result: Gate::Pass,
            quest_progress: MemoryQuestProgress {
                additional_reward_enabled: Gate::Fail,
                reward_item: Gate::Pass,
                cost_group: Gate::Pass,
                reset_restricted: Gate::Fail,
                summoned_name_service_present: Gate::Pass,
                summoned_name_allowed: Gate::Pass,
                summoned_name_filter: Gate::Pass,
                ..MemoryQuestProgress::default()
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
    fn last(&self) -> QuestProgressTask {
        match &self.services.skill_tasks.last().unwrap().kind {
            SkillTaskKind::QuestProgress(t) => t.clone(),
            other => panic!("wrong task: {other:?}"),
        }
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
fn unsupported(out: Outcome) {
    assert!(matches!(out.status, Status::Unsupported(_)), "{out:?}");
    assert!(out.replies.is_empty() && out.changes.is_empty());
}
fn queued(out: &Outcome, command: u16, body: &[u8]) {
    assert_eq!(out.status, Status::Handled);
    assert!(out.replies.is_empty());
    assert!(out.changes.contains(&StateChange::Queued(command)));
    assert!(out.changes.contains(&StateChange::QueuedRequest {
        command,
        body: body.to_vec()
    }));
}
fn short(v: &mut Vec<u8>, n: u16) {
    v.extend(n.to_le_bytes());
}
fn pos(v: &mut Vec<u8>, p: (u8, u16)) {
    v.push(p.0);
    short(v, p.1);
}
fn positions(v: &mut Vec<u8>, list: &[(u8, u16)]) {
    short(v, list.len() as u16);
    for &p in list {
        pos(v, p);
    }
}
fn items(v: &mut Vec<u8>, list: &[(u8, u16, u16)]) {
    short(v, list.len() as u16);
    for &(d, p, n) in list {
        pos(v, (d, p));
        short(v, n);
    }
}
fn acquire_head() -> Vec<u8> {
    let mut v = 42u16.to_le_bytes().to_vec();
    v.extend([3, 4]);
    v.extend(0x1234u64.to_le_bytes());
    short(&mut v, 0xfedc);
    v
}
fn acquire(brands: &[(u8, u16)], orders: &[(u8, u16, u16)]) -> Vec<u8> {
    let mut v = acquire_head();
    positions(&mut v, brands);
    items(&mut v, orders);
    v
}
fn mission() -> QuestMission {
    QuestMission {
        quest: 42,
        mission: 2,
        case: 3,
        selector: 4,
        target: 0x1234,
    }
}
fn mission_body(v: &mut Vec<u8>, m: &QuestMission) {
    short(v, m.quest);
    v.extend([m.mission, m.case, m.selector]);
    v.extend(m.target.to_le_bytes());
}
fn progress_head() -> Vec<u8> {
    let mut v = Vec::new();
    mission_body(&mut v, &mission());
    short(&mut v, 0xfedc);
    v
}
fn progress(
    brands: &[(u8, u16)],
    gems: &[(u8, u16)],
    orders: &[(u8, u16, u16)],
    reward: (u8, u64, u64),
) -> Vec<u8> {
    let mut v = progress_head();
    positions(&mut v, brands);
    positions(&mut v, gems);
    items(&mut v, orders);
    v.push(reward.0);
    v.extend(reward.1.to_le_bytes());
    v.extend(reward.2.to_le_bytes());
    v
}
fn summoned(name: &[u16]) -> Vec<u8> {
    let mut v = 42u16.to_le_bytes().to_vec();
    v.extend([2, 3]);
    v.extend(0x1234u64.to_le_bytes());
    short(&mut v, name.len() as u16);
    for &c in name {
        short(&mut v, c);
    }
    v
}
fn reset(orders: &[(u16, u16)]) -> Vec<u8> {
    let mut v = 42u16.to_le_bytes().to_vec();
    short(&mut v, orders.len() as u16);
    for &(a, b) in orders {
        short(&mut v, a);
        short(&mut v, b);
    }
    v
}

#[test]
fn command_set_and_queue_envelopes() {
    assert_eq!(
        QUEST2_COMMANDS,
        &[1448, 1450, 1452, 1454, 1458, 1465, 1467, 1469, 1472, 1475, 1478]
    );
    let mut h = H::new();
    let cases = [
        (1448, acquire(&[], &[])),
        (1450, progress(&[], &[], &[], (0, 0, 0))),
        (1452, vec![0, 0]),
        (1454, summoned(&[0x41])),
        (1458, vec![0, 0]),
        (1465, vec![42, 0, 2, 3]),
        (1467, vec![42, 0, 2]),
        (1469, reset(&[])),
        (1475, vec![1, 42, 0]),
        (1478, vec![42, 0, 0xff]),
    ];
    for (c, b) in cases {
        queued(&h.run(c, &b), c, &b);
        let t = h.services.skill_tasks.last().unwrap();
        assert_eq!(
            (t.command, t.caster, t.zone, t.summon),
            (c, PC_ID, 9, false)
        );
    }
    assert_eq!(replies(&h.run(1472, &[])), r(1474, 2));
}

#[test]
fn acquire_fixed_and_brand_decode_failures() {
    let mut h = H::new();
    for n in 0..acquire_head().len() {
        assert_eq!(replies(&h.run(1448, &acquire_head()[..n])), r(1449, 955));
    }
    for count in [17, 0xffff] {
        let mut b = acquire_head();
        short(&mut b, count);
        assert_eq!(replies(&h.run(1448, &b)), r(1449, 955));
    }
    let mut b = acquire_head();
    short(&mut b, 1);
    b.extend([1, 2]);
    assert_eq!(replies(&h.run(1448, &b)), r(1449, 955));
    assert_eq!(replies(&h.run(1448, &acquire_head())), r(1449, 955));
}

#[test]
fn acquire_duplicate_brand_before_item_decode() {
    let mut h = H::new();
    let mut b = acquire_head();
    positions(&mut b, &[(1, 8), (1, 8)]);
    assert_eq!(replies(&h.run(1448, &b)), r(1449, 965));
}

#[test]
fn acquire_item_failures_use_progress_reply() {
    let mut h = H::new();
    for count in [17, 0xffff] {
        let mut b = acquire_head();
        positions(&mut b, &[]);
        short(&mut b, count);
        assert_eq!(replies(&h.run(1448, &b)), r(1451, 1001));
    }
    let mut b = acquire_head();
    positions(&mut b, &[]);
    assert_eq!(replies(&h.run(1448, &b)), r(1451, 1001));
    short(&mut b, 1);
    b.extend([1, 2, 0, 3]);
    assert_eq!(replies(&h.run(1448, &b)), r(1451, 1001));
    assert_eq!(
        replies(&h.run(1448, &acquire(&[], &[(1, 8, 2), (1, 8, 99)]))),
        r(1451, 1013)
    );
}

#[test]
fn acquire_fields_zone_and_queue() {
    let mut h = H::new();
    let b = acquire(&[(1, 8), (2, 8)], &[(3, 9, 0x8001)]);
    queued(&h.run(1448, &b), 1448, &b);
    assert_eq!(
        h.last(),
        QuestProgressTask::Acquire {
            quest: 42,
            case: 3,
            selector: 4,
            target: 0x1234,
            value: 0xfedc,
            brands: vec![
                QuestPosition {
                    depot: 1,
                    position: 8
                },
                QuestPosition {
                    depot: 2,
                    position: 8
                }
            ],
            items: vec![QuestItemOrder {
                position: QuestPosition {
                    depot: 3,
                    position: 9
                },
                count: 0x8001
            }],
        }
    );
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1448, &b)), r(1449, 2));
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1448, &b)), r(1449, 2));
    // Decode failure precedes the zone check.
    assert_eq!(replies(&h.run(1448, &[])), r(1449, 955));
}

#[test]
fn progress_fixed_and_list_read_failures() {
    let mut h = H::new();
    for n in 0..progress_head().len() {
        assert_eq!(replies(&h.run(1450, &progress_head()[..n])), r(1451, 1001));
    }
    for stage in 0..3 {
        let mut b = progress_head();
        for _ in 0..stage {
            short(&mut b, 0);
        }
        assert_eq!(replies(&h.run(1450, &b)), r(1451, 1001));
        for count in [17, 0xffff] {
            let mut bad = b.clone();
            short(&mut bad, count);
            assert_eq!(replies(&h.run(1450, &bad)), r(1451, 1001));
        }
        short(&mut b, if stage == 1 { 8 } else { 1 });
        b.extend([1, 2]);
        assert_eq!(replies(&h.run(1450, &b)), r(1451, 1001));
    }
    for count in [1, 7, 9] {
        let mut b = progress_head();
        short(&mut b, 0);
        short(&mut b, count);
        assert_eq!(replies(&h.run(1450, &b)), r(1451, 1001));
    }
}

#[test]
fn progress_duplicate_paths_precede_later_fields() {
    let mut h = H::new();
    let mut b = progress_head();
    positions(&mut b, &[(1, 8), (1, 8)]);
    assert_eq!(replies(&h.run(1450, &b)), r(1451, 1013));
    let gems = vec![(1, 8); 8];
    assert_eq!(
        replies(&h.run(1450, &progress(&[], &gems, &[], (0, 0, 0)))),
        r(1451, 1013)
    );
    assert_eq!(
        replies(&h.run(
            1450,
            &progress(&[], &[], &[(1, 8, 1), (1, 8, 2)], (0, 0, 0))
        )),
        r(1451, 1013)
    );
    assert!(h.services.quest_progress.calls.is_empty());
}

#[test]
fn progress_reward_decode_and_disabled_feature() {
    let mut h = H::new();
    let b = progress(&[], &[], &[], (0, 0, 0));
    for n in b.len() - 17..b.len() {
        assert_eq!(replies(&h.run(1450, &b[..n])), r(1451, 1001));
    }
    for reward in [(1, 0, 0), (0, 1, 0), (0, 0, 1)] {
        assert_eq!(
            replies(&h.run(1450, &progress(&[], &[], &[], reward))),
            r(1451, 1001)
        );
    }
    assert!(h
        .services
        .quest_progress
        .calls
        .iter()
        .all(|c| *c == QuestProgressCheck::AdditionalRewardEnabled));
}

#[test]
fn progress_enabled_reward_range_keys_and_lookups() {
    let mut h = H::new();
    h.services.quest_progress.additional_reward_enabled = Gate::Pass;
    for reward in [(5, 1, 2), (255, 1, 2), (1, 0, 2), (1, 1, 0)] {
        assert_eq!(
            replies(&h.run(1450, &progress(&[], &[], &[], reward))),
            r(1451, 1001)
        );
    }
    let b = progress(&[], &[], &[], (4, 11, 22));
    h.services.quest_progress.calls.clear();
    h.services.quest_progress.reward_item = Gate::Fail;
    assert_eq!(replies(&h.run(1450, &b)), r(1451, 1001));
    assert_eq!(
        h.services.quest_progress.calls,
        vec![
            QuestProgressCheck::AdditionalRewardEnabled,
            QuestProgressCheck::RewardItem { key: 11 }
        ]
    );
    h.services.quest_progress.calls.clear();
    h.services.quest_progress.reward_item = Gate::Pass;
    h.services.quest_progress.cost_group = Gate::Fail;
    assert_eq!(replies(&h.run(1450, &b)), r(1451, 1001));
    assert_eq!(
        h.services.quest_progress.calls.last(),
        Some(&QuestProgressCheck::CostGroup { key: 22 })
    );
    h.services.quest_progress.cost_group = Gate::Pass;
    queued(&h.run(1450, &b), 1450, &b);
}

#[test]
fn progress_index_zero_skips_lookup_even_with_nonzero_keys() {
    let mut h = H::new();
    h.services.quest_progress.additional_reward_enabled = Gate::Pass;
    h.services.quest_progress.reward_item = Gate::Unknown;
    h.services.quest_progress.cost_group = Gate::Unknown;
    let b = progress(&[], &[], &[], (0, 11, 22));
    queued(&h.run(1450, &b), 1450, &b);
    assert_eq!(
        h.services.quest_progress.calls,
        vec![QuestProgressCheck::AdditionalRewardEnabled]
    );
}

#[test]
fn progress_task_fields_and_zone_order() {
    let mut h = H::new();
    let gems = (0..8).map(|n| (2, n)).collect::<Vec<_>>();
    let b = progress(&[(1, 9)], &gems, &[(3, 8, 7)], (0, 0, 0));
    queued(&h.run(1450, &b), 1450, &b);
    let QuestProgressTask::Progress {
        mission: m,
        value,
        brands,
        gems: g,
        items,
        reward,
    } = h.last()
    else {
        panic!()
    };
    assert_eq!(m, mission());
    assert_eq!(value, 0xfedc);
    assert_eq!(
        brands[0],
        QuestPosition {
            depot: 1,
            position: 9
        }
    );
    assert_eq!(g.len(), 8);
    assert_eq!(g[7].position, 7);
    assert_eq!(items[0].count, 7);
    assert_eq!(
        reward,
        QuestAdditionalReward {
            index: 0,
            item_key: 0,
            cost_group_key: 0
        }
    );
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1450, &b)), r(1451, 2));
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1450, &b)), r(1451, 2));
    h.services.quest_progress.additional_reward_enabled = Gate::Unknown;
    unsupported(h.run(1450, &b)); // Configuration precedes the zone gate.
}

#[test]
fn progress_unknown_checks_never_approve() {
    for stage in 0..3 {
        let mut h = H::new();
        h.services.quest_progress.additional_reward_enabled = Gate::Pass;
        match stage {
            0 => h.services.quest_progress.additional_reward_enabled = Gate::Unknown,
            1 => h.services.quest_progress.reward_item = Gate::Unknown,
            _ => h.services.quest_progress.cost_group = Gate::Unknown,
        }
        unsupported(h.run(1450, &progress(&[], &[], &[], (1, 11, 22))));
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn progress_all_failure_body_for_count_read_zone_and_queue() {
    let fail = vec![(1453, vec![2, 0, 0, 0, 0, 0])];
    let mut h = H::new();
    for b in [vec![], vec![9, 0], vec![0xff, 0xff], vec![1, 0, 42, 0, 1]] {
        assert_eq!(replies(&h.run(1452, &b)), fail);
    }
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1452, &[0, 0])), fail);
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1452, &[0, 0])), fail);
}

#[test]
fn progress_all_keeps_fields_and_accepts_zero_or_eight() {
    let mut h = H::new();
    let mut b = vec![8, 0];
    for _ in 0..8 {
        mission_body(&mut b, &mission());
    }
    queued(&h.run(1452, &b), 1452, &b);
    assert_eq!(
        h.last(),
        QuestProgressTask::ProgressAll {
            missions: vec![mission(); 8]
        }
    );
    queued(&h.run(1452, &[0, 0]), 1452, &[0, 0]);
    assert_eq!(
        h.last(),
        QuestProgressTask::ProgressAll { missions: vec![] }
    );
}

#[test]
fn summoned_no_zone_then_job_before_unset_decode() {
    let mut h = H::new();
    h.pc().zone = None;
    h.pc().job = 4;
    assert_eq!(replies(&h.run(1454, &[])), r(1455, 1));
    h.pc().zone = Some(9);
    assert_eq!(replies(&h.run(1454, &[])), r(1455, 1060));
    h.pc().job = 6;
    unsupported(h.run(1454, &[]));
    assert!(h.services.quest_progress.calls.is_empty());
}

#[test]
fn summoned_user_and_service_rejection_paths() {
    let mut h = H::new();
    let b = summoned(&[0x41]);
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(1454, &b)), r(1455, 2));
    h.pc().user_present = true;
    h.pc().party_user_blocked = true;
    assert_eq!(replies(&h.run(1454, &b)), r(1455, 2));
    h.pc().party_user_blocked = false;
    h.services.quest_progress.summoned_name_allowed = Gate::Fail;
    assert_eq!(replies(&h.run(1454, &b)), r(1455, 1062));
    assert!(!h
        .services
        .quest_progress
        .calls
        .iter()
        .any(|c| matches!(c, QuestProgressCheck::SummonedNameFilter { .. })));
}

#[test]
fn summoned_filter_rejection_and_absent_service() {
    let mut h = H::new();
    let b = summoned(&[0x41]);
    h.services.quest_progress.summoned_name_service_present = Gate::Fail;
    h.pc().user_present = false;
    h.services.quest_progress.summoned_name_filter = Gate::Fail;
    assert_eq!(replies(&h.run(1454, &b)), r(1455, 1062));
    assert_eq!(
        h.services.quest_progress.calls,
        vec![
            QuestProgressCheck::SummonedNameServicePresent,
            QuestProgressCheck::SummonedNameFilter { name: vec![0x41] }
        ]
    );
    h.services.quest_progress.summoned_name_filter = Gate::Pass;
    queued(&h.run(1454, &b), 1454, &b);
}

#[test]
fn summoned_task_nul_prefix_and_queue_failure() {
    let mut h = H::new();
    let b = summoned(&[0x41, 0, 0x42]);
    queued(&h.run(1454, &b), 1454, &b);
    assert_eq!(
        h.last(),
        QuestProgressTask::AcquireSummoned {
            quest: 42,
            mission: 2,
            case: 3,
            target: 0x1234,
            name: vec![0x41]
        }
    );
    assert_eq!(
        h.services.quest_progress.calls,
        vec![
            QuestProgressCheck::SummonedNameServicePresent,
            QuestProgressCheck::SummonedNameAllowed {
                pc: PC_ID,
                name: vec![0x41]
            },
            QuestProgressCheck::SummonedNameFilter { name: vec![0x41] },
        ]
    );
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1454, &b)), r(1455, 1));
}

#[test]
fn summoned_name_capacity_and_unknown_boundaries() {
    let mut h = H::new();
    let b = summoned(&[0x41; 20]);
    queued(&h.run(1454, &b), 1454, &b);
    unsupported(h.run(1454, &summoned(&[0x41; 21])));
    for stage in 0..3 {
        let mut h = H::new();
        match stage {
            0 => h.services.quest_progress.summoned_name_service_present = Gate::Unknown,
            1 => h.services.quest_progress.summoned_name_allowed = Gate::Unknown,
            _ => h.services.quest_progress.summoned_name_filter = Gate::Unknown,
        }
        unsupported(h.run(1454, &summoned(&[])));
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn dispose_all_failure_paths_and_maximum() {
    let mut h = H::new();
    for b in [vec![], vec![35, 0], vec![0xff, 0xff], vec![1, 0, 1]] {
        assert_eq!(replies(&h.run(1458, &b)), r(1459, 2));
    }
    let mut b = vec![34, 0];
    for n in 0..34 {
        short(&mut b, n);
    }
    queued(&h.run(1458, &b), 1458, &b);
    assert_eq!(
        h.last(),
        QuestProgressTask::Dispose {
            quests: (0..34).collect()
        }
    );
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1458, &b)), r(1459, 2));
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1458, &b)), r(1459, 2));
}

#[test]
fn rollback_task_and_failure_paths() {
    let mut h = H::new();
    let b = [42, 0, 0xfe, 0xff];
    queued(&h.run(1465, &b), 1465, &b);
    assert_eq!(
        h.last(),
        QuestProgressTask::Rollback {
            quest: 42,
            mission: 0xfe,
            step: 0xff
        }
    );
    for n in 0..b.len() {
        unsupported(h.run(1465, &b[..n]));
    }
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1465, &b)), r(1466, 2));
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1465, &[])), r(1466, 2));
}

#[test]
fn time_expired_task_and_failure_paths() {
    let mut h = H::new();
    let b = [42, 0, 0xff];
    queued(&h.run(1467, &b), 1467, &b);
    assert_eq!(
        h.last(),
        QuestProgressTask::TimeExpired {
            quest: 42,
            mission: 0xff
        }
    );
    for n in 0..b.len() {
        unsupported(h.run(1467, &b[..n]));
    }
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1467, &b)), r(1468, 2));
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1467, &[])), r(1468, 2));
}

#[test]
fn reset_decode_reasons_and_exclusive_maximum() {
    let mut h = H::new();
    for b in [vec![], vec![42]] {
        assert_eq!(replies(&h.run(1469, &b)), r(1471, 1091));
    }
    for b in [vec![42, 0], vec![42, 0, 16, 0], vec![42, 0, 0xff, 0xff]] {
        assert_eq!(replies(&h.run(1469, &b)), r(1471, 1092));
    }
    for b in [vec![42, 0, 1, 0], vec![42, 0, 1, 0, 1, 0, 2]] {
        assert_eq!(replies(&h.run(1469, &b)), r(1471, 1091));
    }
    let orders = vec![(0xfffe, 0xffff); 15];
    let b = reset(&orders);
    queued(&h.run(1469, &b), 1469, &b);
    assert_eq!(
        h.last(),
        QuestProgressTask::ResetCompletionTime { quest: 42, orders }
    );
}

#[test]
fn reset_zone_restriction_kind_queue_in_native_order() {
    let mut h = H::new();
    let b = reset(&[]);
    h.pc().zone = None;
    h.services.quest_progress.reset_restricted = Gate::Pass;
    assert_eq!(replies(&h.run(1469, &b)), r(1471, 2));
    assert!(h.services.quest_progress.calls.is_empty());
    h.pc().zone = Some(9);
    h.pc().native_kind = 2;
    assert_eq!(replies(&h.run(1469, &b)), r(1471, 1098));
    assert_eq!(
        h.services.quest_progress.calls.last(),
        Some(&QuestProgressCheck::ResetRestricted { pc: PC_ID })
    );
    h.services.quest_progress.reset_restricted = Gate::Fail;
    assert_eq!(replies(&h.run(1469, &b)), r(1471, 2));
    h.pc().native_kind = 1;
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1469, &b)), r(1471, 2));
    h.services.quest_progress.reset_restricted = Gate::Unknown;
    unsupported(h.run(1469, &b));
}

#[test]
fn renewal_always_disabled() {
    let mut h = H::new();
    h.pc().zone = None;
    h.pc().native_kind = 2;
    for b in [&[][..], &[0xff, 0xff][..]] {
        let out = h.run(1472, b);
        assert_eq!(replies(&out), r(1474, 2));
        assert!(out.logs.contains(&"acquire renewal quest: disabled"));
        assert!(out.changes.is_empty());
    }
    assert!(h.services.skill_tasks.is_empty() && h.services.quest_progress.calls.is_empty());
}

#[test]
fn challenge_quest_default_then_zone_kind_subtype_and_queue() {
    let mut h = H::new();
    h.pc().zone = None;
    for b in [vec![], vec![1], vec![1, 42], vec![1, 0, 0]] {
        assert_eq!(replies(&h.run(1475, &b)), r(1477, 1111));
    }
    assert_eq!(replies(&h.run(1475, &[1, 42, 0])), r(1477, 2));
    h.pc().zone = Some(9);
    h.pc().native_kind = 2;
    assert_eq!(replies(&h.run(1475, &[1, 42, 0])), r(1477, 2));
    h.pc().native_kind = 1;
    for sub_type in [2, 0xff] {
        assert_eq!(replies(&h.run(1475, &[sub_type, 42, 0])), r(1477, 2));
    }
    for sub_type in [0, 1] {
        let b = [sub_type, 42, 0];
        queued(&h.run(1475, &b), 1475, &b);
        assert_eq!(
            h.last(),
            QuestProgressTask::ChallengeAcquire {
                sub_type,
                quest: 42
            }
        );
    }
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(replies(&h.run(1475, &[1, 42, 0])), r(1477, 2));
}

#[test]
fn skip_uses_one_byte_and_does_not_invent_failure_body() {
    let mut h = H::new();
    let b = [42, 0, 0xff];
    queued(&h.run(1478, &b), 1478, &b);
    assert_eq!(
        h.last(),
        QuestProgressTask::SkipMissionStep {
            quest: 42,
            reason: 0xff
        }
    );
    for n in 0..b.len() {
        unsupported(h.run(1478, &b[..n]));
    }
    h.services.skill_queue_result = Gate::Fail;
    unsupported(h.run(1478, &b));
    h.pc().zone = None;
    unsupported(h.run(1478, &b));
    unsupported(h.run(1478, &[]));
}

#[test]
fn queue_unknown_is_not_success_for_each_task() {
    let cases = [
        (1448, acquire(&[], &[])),
        (1450, progress(&[], &[], &[], (0, 0, 0))),
        (1452, vec![0, 0]),
        (1454, summoned(&[])),
        (1458, vec![0, 0]),
        (1465, vec![42, 0, 1, 2]),
        (1467, vec![42, 0, 1]),
        (1469, reset(&[])),
        (1475, vec![0, 42, 0]),
        (1478, vec![42, 0, 1]),
    ];
    for (c, b) in cases {
        let mut h = H::new();
        h.services.skill_queue_result = Gate::Unknown;
        unsupported(h.run(c, &b));
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn no_extra_arena_trial_or_pc_kind_gates() {
    let cases = [
        (1448, acquire(&[], &[])),
        (1450, progress(&[], &[], &[], (0, 0, 0))),
        (1452, vec![0, 0]),
        (1454, summoned(&[])),
        (1458, vec![0, 0]),
        (1465, vec![42, 0, 1, 2]),
        (1467, vec![42, 0, 1]),
        (1478, vec![42, 0, 1]),
    ];
    for (c, mut b) in cases {
        let mut h = H::new();
        h.pc().native_kind = 2;
        h.pc().arena_state = 3;
        h.pc().t2_blocked = true;
        h.world.arena = true;
        b.extend([0xde, 0xad]);
        queued(&h.run(c, &b), c, &b);
    }
    for (c, b) in [(1469, reset(&[])), (1475, vec![1, 42, 0])] {
        let mut h = H::new();
        h.world.arena = true;
        h.pc().arena_state = 6;
        h.pc().t2_blocked = true;
        queued(&h.run(c, &b), c, &b);
    }
}

#[test]
fn opaque_state_defaults_are_unknown() {
    let mut h = H::new();
    h.services.quest_progress = MemoryQuestProgress::default();
    unsupported(h.run(1450, &progress(&[], &[], &[], (0, 0, 0))));
    unsupported(h.run(1454, &summoned(&[])));
    unsupported(h.run(1469, &reset(&[])));
    assert!(h.services.skill_tasks.is_empty());
}
