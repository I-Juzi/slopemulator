//! Doc 13 part 2: faction, manipulate, summon, energy, training, skin, auto combat.
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
            user_present: true,
            ..Pc::default()
        };
        let world = World {
            vitality_recharge_enabled: true,
            utc_seconds: 1234,
            tick: 1000,
            trait_train_interval: 100,
            ..World::default()
        };
        let services = MemoryServices {
            skill_queue_result: Gate::Pass,
            ..MemoryServices::default()
        };
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
    vec![(command, reason.to_le_bytes().to_vec())]
}
fn cat(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
}
fn unsupported(out: &Outcome) -> bool {
    matches!(out.status, Status::Unsupported(_))
}

fn kind(h: &H) -> CombatTask {
    match h.last().kind {
        SkillTaskKind::Combat(t) => t,
        other => panic!("{other:?}"),
    }
}
fn q(v: u64) -> [u8; 8] {
    v.to_le_bytes()
}
const NPC32: u64 = (32 << 48) | 5;
const NPC4: u64 = (4 << 48) | 5;

#[test]
fn faction_entries() {
    let mut h = H::new();
    h.world.faction_enabled = true;
    let ex = cat(&[&q(7), &(-3i32).to_le_bytes()]);
    // U0 (user missing) → 336; then zone/kind logs.
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(83, &ex)), r(85, 336));
    h.pc().user_present = true;
    h.run(83, &ex);
    assert_eq!(
        kind(&h),
        CombatTask::ExchangeFactionScore { npc: 7, amount: -3 }
    );
    assert!(unsupported(&h.run(83, &[1])));
    h.pc().native_kind = 2;
    let out = h.run(83, &ex);
    assert_eq!(replies(&out), r(85, 1));
    assert!(!out.logs.is_empty());
    h.pc().native_kind = 1;
    h.run(125, &cat(&[&[2], &q(9)]));
    assert_eq!(kind(&h), CombatTask::JoinFaction { faction: 2, npc: 9 });
    h.run(127, &q(9));
    assert_eq!(kind(&h), CombatTask::LeaveFaction { npc: 9 });
    h.run(129, &cat(&[&[1], &q(9), &[4]]));
    assert_eq!(
        kind(&h),
        CombatTask::TransferFaction {
            faction: 1,
            npc: 9,
            option: 4
        }
    );
    // Config gate applies to 125/127/129, not 83.
    h.world.faction_enabled = false;
    assert_eq!(replies(&h.run(127, &q(9))), r(128, 2));
    assert!(h.run(83, &ex).replies.is_empty());
    // T1 comes first (5) for all four.
    h.world.arena = true;
    assert_eq!(replies(&h.run(129, &[])), r(130, 5));
    assert_eq!(replies(&h.run(83, &[])), r(85, 5));
    // 131 ignores T1 and every gate.
    h.world.faction_statistics = [1, 2, 3];
    h.pc().zone = None;
    let out = h.run(131, &[9]);
    assert_eq!(replies(&out), vec![(132, cat(&[&q(1), &q(2), &q(3)]))]);
    assert_eq!(out.changes, vec![StateChange::FactionStatisticsCalls(1)]);
    h.world.faction_statistics_calls = 100;
    let out = h.run(131, &[]);
    assert_eq!(out.changes, vec![StateChange::FactionStatisticsCalls(1)]);
    assert!(!out.logs.is_empty());
    h.world.arena = false;
    h.pc().zone = Some(9);
    h.world.faction_enabled = true;
    assert_eq!(replies(&h.run(125, &cat(&[&[2], &q(9)]))), vec![]);
}

#[test]
fn manipulate_refiner_fish() {
    let mut h = H::new();
    let rec = |c: u8, p: u16, n: u16| cat(&[&[c], &p.to_le_bytes(), &n.to_le_bytes()]);
    let man = |target: u64, recs: &[Vec<u8>]| {
        cat(&[
            &q(target),
            &[3, 0, 1],
            &(recs.len() as u16).to_le_bytes(),
            &recs.concat(),
        ])
    };
    h.run(1253, &man(NPC32, &[rec(1, 2, 3), rec(1, 3, 3)]));
    assert_eq!(
        kind(&h),
        CombatTask::Manipulate {
            target: NPC32,
            operation: 3,
            option: 1,
            records: vec![(1, 2, 3), (1, 3, 3)]
        }
    );
    assert_eq!(
        replies(&h.run(1253, &man(NPC32, &[rec(1, 2, 3), rec(1, 2, 9)]))),
        r(1254, 2)
    );
    assert_eq!(replies(&h.run(1253, &man(NPC4, &[]))), r(1254, 2));
    assert_eq!(
        replies(&h.run(1253, &cat(&[&q(NPC32), &[3, 0, 1], &[17, 0]]))),
        r(1254, 2)
    );
    // 1264: U0 676 before decode; target kind 4; no operation option byte.
    h.pc().party_user_blocked = true;
    assert_eq!(replies(&h.run(1264, &[])), r(1265, 676));
    h.pc().party_user_blocked = false;
    h.run(1264, &cat(&[&q(NPC4), &[3, 0], &[1, 0], &rec(1, 2, 3)]));
    assert_eq!(
        kind(&h),
        CombatTask::NpcManipulate {
            target: NPC4,
            operation: 3,
            records: vec![(1, 2, 3)]
        }
    );
    assert_eq!(
        replies(&h.run(1264, &cat(&[&q(NPC32), &[3, 0, 0, 0]]))),
        r(1265, 2)
    );
    // Refiner: target kind 32 and PC kind.
    h.run(1255, &q(NPC32));
    assert_eq!(
        kind(&h),
        CombatTask::GiveFactionScoreToRefiner { target: NPC32 }
    );
    assert_eq!(replies(&h.run(1258, &q(NPC4))), r(1259, 2));
    assert!(unsupported(&h.run(1258, &[1])));
    // Fish.
    let fish = |action: u8| cat(&[&q(5), &[1, 0, 2, 0, 3, 0], &[action], &[7, 0], &[9]]);
    assert_eq!(replies(&h.run(1266, &fish(0))), r(1269, 2));
    h.world.fish_enabled = true;
    h.run(1266, &fish(0));
    assert_eq!(
        kind(&h),
        CombatTask::FishAction {
            target: 5,
            operation: 7,
            position: [1, 2, 3]
        }
    );
    h.run(1266, &fish(2));
    assert_eq!(
        kind(&h),
        CombatTask::FishTask {
            action: 2,
            option: 9
        }
    );
    let out = h.run(1266, &fish(3));
    assert!(out.replies.is_empty() && !out.logs.is_empty());
    assert_eq!(replies(&h.run(1266, &fish(0)[..10])), r(1269, 2));
}

#[test]
fn summon_management() {
    let mut h = H::new();
    let change = |n: u16, bytes: &[u8]| cat(&[&q(4), &n.to_le_bytes(), bytes]);
    assert_eq!(replies(&h.run(630, &change(2, &[1, 2]))), r(632, 1332));
    h.pc().job = 6;
    h.run(630, &change(2, &[1, 2]));
    assert_eq!(
        kind(&h),
        CombatTask::ChangeSummoned {
            key: 4,
            changes: vec![1, 2]
        }
    );
    assert_eq!(replies(&h.run(630, &change(7, &[0; 7]))), r(1342, 1337));
    assert_eq!(replies(&h.run(630, &change(2, &[1]))), r(632, 1331));
    assert_eq!(replies(&h.run(630, &[1])), r(632, 1331));
    // 633: config, count 16 → 1346, zero second short → 1342, life → 2, then the name service.
    let name = [1u8, 0, 0x41, 0];
    let ren = |pairs: &[(u16, u16)]| {
        let mut b = cat(&[&name, &(pairs.len() as u16).to_le_bytes()]);
        for (a, c) in pairs {
            b.extend(a.to_le_bytes());
            b.extend(c.to_le_bytes());
        }
        b
    };
    assert_eq!(replies(&h.run(633, &ren(&[]))), r(635, 1342));
    h.world.summon_rename_enabled = true;
    assert_eq!(replies(&h.run(633, &ren(&[(1, 0)]))), r(635, 1342));
    assert_eq!(replies(&h.run(633, &cat(&[&name, &[16, 0]]))), r(635, 1346));
    h.pc().life_state = 1;
    assert_eq!(replies(&h.run(633, &ren(&[(1, 2)]))), r(635, 2));
    h.pc().life_state = 0;
    assert!(unsupported(&h.run(633, &ren(&[(1, 2)]))));
    h.pc().job = 1;
    assert_eq!(replies(&h.run(633, &ren(&[]))), r(635, 1343));
    // 636 / 788.
    h.run(636, &[]);
    assert_eq!(kind(&h), CombatTask::KillSummoned);
    h.pc().native_kind = 2;
    assert_eq!(replies(&h.run(788, &[])), r(790, 2));
}

#[test]
fn energy_and_pet_food() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(837, &[3])), r(839, 2614));
    h.world.energy_enabled = true;
    assert!(unsupported(&h.run(837, &[3])));
    h.world.energy_multiple_mask = Some(1 << 3);
    assert_eq!(replies(&h.run(837, &[1])), r(839, 2615));
    assert_eq!(replies(&h.run(837, &[64])), r(839, 2615));
    assert_eq!(replies(&h.run(837, &[4])), r(839, 2615));
    assert_eq!(replies(&h.run(837, &[])), r(839, 2));
    h.run(837, &[3]);
    assert_eq!(kind(&h), CombatTask::SetEnergyMultiple { multiple: 3 });
    // 840: only code 2 is framed (E=1, Q=2, i=ID).
    let reward = |id: i32, opt: u8, v: i16| cat(&[&id.to_le_bytes(), &[opt], &v.to_le_bytes()]);
    assert_eq!(
        replies(&h.run(840, &[1])),
        vec![(842, vec![1, 0, 2, 0, 0, 0, 0])]
    );
    assert!(unsupported(&h.run(840, &reward(5, 9, 0))));
    assert!(unsupported(&h.run(840, &reward(5, 8, -1))));
    h.run(840, &reward(5, 8, 0));
    assert_eq!(
        kind(&h),
        CombatTask::OpenEnergyPointReward {
            reward: 5,
            option: 8,
            value: 0
        }
    );
    h.pc().zone = None;
    assert_eq!(
        replies(&h.run(840, &reward(5, 0, 0))),
        vec![(842, vec![1, 0, 2, 5, 0, 0, 0])]
    );
    assert_eq!(replies(&h.run(952, &[1, 2, 0, 3, 0, 4])), r(953, 2));
    h.pc().zone = Some(9);
    h.run(952, &[1, 2, 0, 3, 0, 4]);
    assert_eq!(
        kind(&h),
        CombatTask::FeedPetFood {
            selector: 1,
            shorts: [2, 3],
            option: 4
        }
    );
    // Scheduler return is ignored.
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(h.run(952, &[1, 2, 0, 3, 0, 4]), Outcome::default());
}

#[test]
fn training_skin_auto_combat() {
    let mut h = H::new();
    assert_eq!(replies(&h.run(1026, &[1, 0, 0])), r(1028, 2324));
    h.run(1026, &cat(&[&(-1i32).to_le_bytes(), &[2]]));
    assert_eq!(
        kind(&h),
        CombatTask::TrainingRoomCombatStart { id: -1, option: 2 }
    );
    assert_eq!(replies(&h.run(1029, &[])), r(1031, 2329));
    h.run(1053, &[]);
    assert_eq!(kind(&h), CombatTask::ResetCombatMode);
    h.pc().native_kind = 2;
    let out = h.run(1038, &[]);
    assert_eq!(replies(&out), r(1040, 2));
    assert!(!out.logs.is_empty());
    // Skin: 1062 has no kind gate.
    assert_eq!(replies(&h.run(1062, &[])), r(1064, 2378));
    h.world.skill_skin_enabled = true;
    h.run(1062, &[]);
    assert_eq!(kind(&h), CombatTask::SkillSkinAcquiredList);
    h.pc().native_kind = 1;
    h.pc().current_skin = 4;
    assert_eq!(replies(&h.run(1065, &4u16.to_le_bytes())), r(1067, 2384));
    assert_eq!(replies(&h.run(1065, &(-1i16).to_le_bytes())), r(1067, 2));
    assert_eq!(replies(&h.run(1065, &[1])), r(1067, 2));
    h.run(1065, &0u16.to_le_bytes());
    assert_eq!(kind(&h), CombatTask::ApplySkillSkin { skin: 0 });
    // Auto combat: failures log only.
    h.run(1208, &[3]);
    assert_eq!(kind(&h), CombatTask::AutoCombatStart { option: 3 });
    h.run(1209, &[1, 4]);
    assert_eq!(
        kind(&h),
        CombatTask::AutoCombatStop {
            option: 1,
            reason: 4
        }
    );
    let out = h.run(1209, &[1]);
    assert!(out.replies.is_empty() && !out.logs.is_empty());
    h.services.skill_queue_result = Gate::Fail;
    let out = h.run(1208, &[3]);
    assert!(out.replies.is_empty() && !out.logs.is_empty());
}
