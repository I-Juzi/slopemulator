//! Doc 13 part 1: skill entry functions.
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
    fn summon(&mut self, control_state: u8) {
        self.pc().summon = Some(Summon {
            control_state,
            native_kind: 2,
            zone: Some(4),
            ..Summon::default()
        });
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
fn rb(command: u16, reason: u16, extra: &[u8]) -> Vec<(u16, Vec<u8>)> {
    vec![(command, [&reason.to_le_bytes()[..], extra].concat())]
}
fn cat(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
}
fn unsupported(out: &Outcome) -> bool {
    matches!(out.status, Status::Unsupported(_))
}

fn use_skill(selector: u8, groups: &[u8]) -> Vec<u8> {
    cat(&[
        &[selector],
        &77u32.to_le_bytes(),
        &[2],
        &5u64.to_le_bytes(),
        &[3, 0],
        &[0xFF],
        groups,
    ])
}

#[test]
fn use_skill_entries() {
    let mut h = H::new();
    let groups = cat(&[&[1, 0, 4], &[1, 0], &8u64.to_le_bytes(), &[0, 0], &[0, 0]]);
    assert!(h.run(1226, &use_skill(0, &groups)).replies.is_empty());
    let t = h.last();
    assert_eq!((t.command, t.summon, t.zone), (1226, false, 9));
    assert_eq!(
        t.kind,
        SkillTaskKind::UseSkill {
            skill: 77,
            variation: 2,
            target: 5,
            request: 3,
            item_selector: 0xFF,
            modifiers: SkillModifiers {
                equip_positions: vec![4],
                set_item_keys: vec![8],
                item_skill_positions: vec![],
                badge_positions: vec![],
            }
        }
    );
    let ident = cat(&[&[0], &77u32.to_le_bytes(), &[2]]);
    // Scalar failure zeros identity; group bound 9 > 8 keeps it.
    assert_eq!(replies(&h.run(1226, &[0, 1])), rb(1227, 1, &[0; 6]));
    assert_eq!(
        replies(&h.run(1226, &use_skill(0, &[9, 0]))),
        rb(1227, 1, &ident)
    );
    assert_eq!(
        replies(&h.run(1226, &use_skill(0, &[0, 0, 39, 0]))),
        rb(1227, 1, &ident)
    );
    // Selector 1 without a controlled summon fails; with one, the summon zone is used.
    let zero = [0u8; 8];
    let mut ident1 = ident.clone();
    ident1[0] = 1;
    assert_eq!(
        replies(&h.run(1226, &use_skill(1, &zero))),
        rb(1227, 1, &ident1)
    );
    h.summon(2);
    h.run(1226, &use_skill(1, &zero));
    assert_eq!((h.last().summon, h.last().zone), (true, 4));
    // Ground: three position shorts replace the target.
    let ground = cat(&[
        &[0],
        &77u32.to_le_bytes(),
        &[2],
        &[1, 0, 2, 0, 3, 0],
        &[3, 0],
        &[0],
        &zero,
    ]);
    h.run(1228, &ground);
    assert!(matches!(
        h.last().kind,
        SkillTaskKind::UseSkillToGround {
            position: [1, 2, 3],
            ..
        }
    ));
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(
        replies(&h.run(1226, &use_skill(0, &zero))),
        rb(1227, 1, &ident)
    );
}

#[test]
fn cancel_trigger_gather_loot() {
    let mut h = H::new();
    h.run(1229, &[0, 3]);
    assert_eq!(h.last().kind, SkillTaskKind::CancelAction { option: 3 });
    assert_eq!(replies(&h.run(1231, &[1, 0])), rb(1232, 1, &[1]));
    assert!(unsupported(&h.run(1229, &[0])));
    let q = 7u64.to_le_bytes();
    h.run(1270, &q);
    assert_eq!(h.last().kind, SkillTaskKind::Gather { target: 7 });
    h.pc().user_present = false;
    assert_eq!(replies(&h.run(1270, &q)), r(1271, 683));
    h.pc().user_present = true;
    h.pc().native_kind = 2;
    assert_eq!(replies(&h.run(77, &[])), r(78, 1));
    h.pc().native_kind = 1;
    let mut both = r(1273, 537);
    both.extend(r(1273, 1));
    assert_eq!(replies(&h.run(1272, &q)), both);
    let loot = ((176u64 << 48) | 1).to_le_bytes();
    h.run(1272, &loot);
    assert!(matches!(h.last().kind, SkillTaskKind::Loot { .. }));
    h.pc().zone = None;
    assert_eq!(replies(&h.run(1270, &q)), r(1271, 1));
    // 134: missing zone logs only.
    let out = h.run(134, &q);
    assert!(out.replies.is_empty() && !out.logs.is_empty());
}

#[test]
fn summon_exhaustion_respawn_vitality() {
    let mut h = H::new();
    let pos = [1, 0, 2, 0, 3, 0, 9, 0];
    assert_eq!(replies(&h.run(71, &pos)), r(72, 2));
    h.summon(1);
    h.pc().summon.as_mut().unwrap().native_kind = 3;
    assert_eq!(replies(&h.run(71, &pos)), r(72, 175));
    h.summon(1);
    {
        let s = h.pc().summon.as_mut().unwrap();
        s.has_entry = true;
        s.race = 11;
    }
    assert_eq!(replies(&h.run(71, &pos)), r(72, 179));
    h.summon(1);
    h.run(71, &pos);
    assert_eq!(
        h.last().kind,
        SkillTaskKind::RecallSummon {
            position: [1, 2, 3]
        }
    );
    assert_eq!(h.last().zone, 4);
    assert!(unsupported(&h.run(74, &[])));
    h.run(135, &[]);
    assert_eq!(h.last().kind, SkillTaskKind::DieInExhaustion);
    // 139: summon caster kind 2 passes; queue failure keeps the selector.
    h.run(139, &cat(&[&[1], &3u64.to_le_bytes()]));
    assert!(h.last().summon);
    h.run(142, &[2]);
    assert_eq!(h.last().kind, SkillTaskKind::Respawn { option: 2 });
    assert_eq!(h.run(821, &[]).replies.len(), 0);
    let utc = 1234u64.to_le_bytes();
    h.run(824, &[]);
    assert_eq!(h.last().kind, SkillTaskKind::RechargeVitality);
    h.world.vitality_recharge_enabled = false;
    assert_eq!(replies(&h.run(824, &[])), rb(826, 1, &utc));
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(
        replies(&h.run(139, &cat(&[&[0], &3u64.to_le_bytes()]))),
        rb(141, 1, &[0])
    );
    assert_eq!(replies(&h.run(137, &[])), r(138, 1));
}

#[test]
fn list_mastery_pages() {
    let mut h = H::new();
    h.run(113, &[]);
    assert_eq!(h.last().kind, SkillTaskKind::SkillList);
    assert_eq!(
        replies(&h.run(117, &[0, 0, 0xFF, 0xFF, 0, 0, 0, 0, 0])),
        r(119, 2)
    );
    assert_eq!(
        replies(&h.run(117, &[0, 0, 0, 0, 0, 0, 0x80, 0, 0])),
        r(119, 2)
    );
    h.run(117, &[1, 0, 2, 0, 1, 2, 3, 4, 5]);
    assert_eq!(
        h.last().kind,
        SkillTaskKind::SetMasteryStatPoint {
            shorts: [1, 2],
            bytes: [1, 2, 3, 4, 5]
        }
    );
    h.run(1488, &[2]);
    assert_eq!(h.last().kind, SkillTaskKind::ChangeSkillPage { page: 2 });
    let name = [1u8, 0, 0x41, 0];
    // 1491: fixed decode → 1330/1; bad count → 1493/693; wrong kind is silent.
    assert_eq!(replies(&h.run(1491, &[1])), r(1330, 1));
    assert_eq!(
        replies(&h.run(1491, &cat(&[&[1], &name, &[16, 0]]))),
        r(1493, 693)
    );
    h.run(1491, &cat(&[&[1], &name, &[1, 0, 2, 0, 3, 0]]));
    assert_eq!(
        h.last().kind,
        SkillTaskKind::ExpandSkillPage {
            page: 1,
            name: vec![0x41],
            pairs: vec![(2, 3)]
        }
    );
    h.pc().native_kind = 2;
    assert_eq!(
        h.run(1491, &cat(&[&[1], &name, &[0, 0]])),
        Outcome::default()
    );
    assert_eq!(replies(&h.run(1494, &cat(&[&[1], &name]))), r(1496, 2));
    assert_eq!(replies(&h.run(1485, &[])), r(1487, 2));
    h.pc().native_kind = 1;
    // 1389: count 16 → 470, T2 → 7.
    assert_eq!(replies(&h.run(1389, &[16, 0])), r(1391, 470));
    h.pc().t2_blocked = true;
    assert_eq!(replies(&h.run(1389, &[0, 0])), r(1391, 7));
}

#[test]
fn train_trait() {
    let mut h = H::new();
    h.services.trait_levels = Some([(5, 3)].into_iter().collect());
    let fixed = [2u8, 9, 1, 2, 3, 0, 0];
    let rec = |id: u32, level: u8| cat(&[&id.to_le_bytes(), &[level]]);
    let ok = cat(&[&fixed, &[1, 0], &rec(5, 3)]);
    let out = h.run(1497, &ok);
    assert!(out.replies.is_empty());
    assert_eq!(out.changes[0], StateChange::TraitTrainTick(1000));
    assert_eq!(
        h.last().kind,
        SkillTaskKind::TrainTrait {
            selectors: [2, 9],
            build_up: [1, 1, 2, 3, 0, 0],
            traits: vec![(5, 3)]
        }
    );
    // Rate gate: 1000 + 100 > 1099 rejects; equality at 1100 passes.
    h.world.tick = 1099;
    assert_eq!(replies(&h.run(1497, &ok)), rb(1499, 707, &[0xFF, 0xFF]));
    let step = |h: &mut H, body: &[u8]| {
        h.world.tick += 100;
        replies(&h.run(1497, body))
    };
    h.world.tick = 1000;
    assert!(step(&mut h, &ok).is_empty());
    assert_eq!(step(&mut h, &[1]), rb(1499, 2, &[0xFF, 0xFF]));
    assert_eq!(
        step(&mut h, &[2, 10, 0, 0, 0, 0, 0]),
        rb(1499, 711, &[2, 10])
    );
    assert_eq!(step(&mut h, &[6, 0, 0, 0, 0, 0, 0]), rb(1499, 706, &[6, 0]));
    assert_eq!(step(&mut h, &[1, 0, 4, 0, 0, 0, 0]), rb(1499, 710, &[1, 0]));
    assert_eq!(
        step(&mut h, &cat(&[&fixed, &[51, 0]])),
        rb(1499, 712, &[2, 9])
    );
    assert_eq!(
        step(&mut h, &cat(&[&fixed, &[1, 0], &rec(0, 1)])),
        rb(1499, 713, &[2, 9])
    );
    assert_eq!(
        step(&mut h, &cat(&[&fixed, &[1, 0], &rec(6, 1)])),
        rb(1499, 713, &[2, 9])
    );
    assert_eq!(
        step(&mut h, &cat(&[&fixed, &[1, 0], &rec(5, 4)])),
        rb(1499, 717, &[2, 9])
    );
    assert_eq!(
        step(&mut h, &cat(&[&fixed, &[2, 0], &rec(5, 1)])),
        rb(1499, 702, &[2, 9])
    );
    h.pc().user_present = false;
    assert_eq!(step(&mut h, &ok), rb(1499, 2, &[0xFF, 0xFF]));
}
