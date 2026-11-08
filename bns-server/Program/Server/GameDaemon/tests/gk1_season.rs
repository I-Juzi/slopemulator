use gamedaemon::handlers::*;

const PC_ID: u64 = (1 << 48) | 0x11;
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
    fn pc(&mut self) -> &mut Pc {
        self.session.pc.as_mut().unwrap()
    }
    fn gate(&mut self, check: SeasonCheck, result: Gate) {
        self.services.season.results.insert(check, result);
    }
    fn allow(&mut self, check: SeasonCheck) {
        self.gate(check, Gate::Pass);
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
    fn last(&self) -> &SkillTask {
        self.services.skill_tasks.last().unwrap()
    }
    fn unbind_checks(&mut self) {
        for check in [
            SeasonCheck::RaidEnabled,
            SeasonCheck::RaidExists(42),
            SeasonCheck::UnbindGeoPresent(9),
            SeasonCheck::UnbindGeoAllowed(9),
            SeasonCheck::UnbindTrialAllowed,
        ] {
            self.allow(check);
        }
    }
}
fn reply(out: &Outcome, command: u16, body: &[u8]) {
    assert_eq!(out.status, Status::Handled);
    assert_eq!(
        out.replies,
        vec![Message {
            command,
            body: body.to_vec()
        }]
    );
    assert!(out.changes.is_empty());
}
fn reason(out: &Outcome, command: u16, code: u16) {
    reply(out, command, &code.to_le_bytes());
}
fn unsupported(out: &Outcome) {
    assert!(
        matches!(out.status, Status::Unsupported(why) if why.starts_with("❓")),
        "{out:?}"
    );
    assert!(out.replies.is_empty());
    assert!(out.changes.is_empty());
}
fn queued(h: &H, out: &Outcome, command: u16, body: &[u8], kind: SeasonTask) {
    assert_eq!(out.status, Status::Handled);
    assert!(out.replies.is_empty());
    assert_eq!(
        h.last(),
        &SkillTask {
            command,
            caster: PC_ID,
            summon: false,
            zone: 9,
            kind: SkillTaskKind::Season(kind)
        }
    );
    assert_eq!(
        out.changes,
        vec![
            StateChange::Queued(command),
            StateChange::QueuedRequest {
                command,
                body: body.to_vec()
            }
        ]
    );
}
fn raid_request(count: i16, pairs: &[(u16, u16)]) -> Vec<u8> {
    let mut body = 42u64.to_le_bytes().to_vec();
    body.extend(count.to_le_bytes());
    for (a, b) in pairs {
        body.extend(a.to_le_bytes());
        body.extend(b.to_le_bytes());
    }
    body
}

#[test]
fn command_set_has_all_17_unique_ids() {
    let mut got = SEASON_COMMANDS.to_vec();
    got.sort_unstable();
    got.dedup();
    let mut want = vec![
        105, 108, 913, 916, 921, 924, 980, 985, 989, 992, 995, 998, 1001, 1059, 1088, 1134, 1213,
    ];
    want.sort_unstable();
    assert_eq!(got, want);
    assert_eq!(SEASON_COMMANDS.len(), 17);
}

#[test]
fn authentication_and_permission_gates_still_precede_season_dispatch() {
    let mut h = H::new();
    h.session.authenticated = false;
    let out = h.run(108, &[]);
    assert!(h.services.skill_tasks.is_empty());
    assert!(out.replies.is_empty());
    h.session.authenticated = true;
    h.session.dispatch_permission = Some(false);
    let _ = h.run(108, &[]);
    assert!(h.services.skill_tasks.is_empty());
    assert!(h.services.season.checks.is_empty());
}

#[test]
fn initialize_rejects_short_zero_negative_and_invalid_types() {
    for body in [
        vec![],
        vec![1],
        vec![0, 0],
        vec![255, 255],
        vec![1, 0, 0],
        vec![1, 0, 2],
        vec![1, 0, 17],
        vec![1, 0, 255],
        vec![2, 0, 1],
    ] {
        let mut h = H::new();
        reason(&h.run(105, &body), 107, 3);
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn initialize_accepts_type16_duplicates_and_seventeen_safe_entries() {
    let mut body = vec![17, 0];
    body.extend([16; 17]);
    let mut h = H::new();
    h.pc().native_kind = 0; // No kind gate in this initializer.
    let out = h.run(105, &body);
    queued(
        &h,
        &out,
        105,
        &body,
        SeasonTask::InitializeContents {
            types: vec![16; 17],
        },
    );
}

#[test]
fn initialize_stops_at_native_buffer_and_lobby_boundaries() {
    let mut h = H::new();
    unsupported(&h.run(105, &[18, 0]));
    h.pc().arena_state = 3;
    h.pc().zone = None;
    unsupported(&h.run(105, &[1, 0, 1])); // Lobby path precedes zone.
    reason(&h.run(105, &[1, 0, 2]), 107, 3); // Types precede lobby.
    h.pc().arena_state = 0;
    reason(&h.run(105, &[1, 0, 1]), 107, 2);
}

#[test]
fn simple_season_tasks_keep_fields_and_ignore_unused_body() {
    for (id, kind) in [
        (108, SeasonTask::ContentsInfo),
        (980, SeasonTask::ReviveByAttraction),
        (985, SeasonTask::TakeAttractionReward),
        (1059, SeasonTask::PublicRaidScore),
        (1213, SeasonTask::PvpResults),
    ] {
        let mut h = H::new();
        let body = 0x1122334455667788u64.to_le_bytes();
        let out = h.run(id, &body);
        queued(&h, &out, id, &body, kind);
    }
}

#[test]
fn all_new_queue_failures_send_the_exact_entry_reply() {
    for (id, fail, body) in [
        (105, 107, vec![1, 0, 1]),
        (108, 110, vec![]),
        (913, 915, 42u64.to_le_bytes().to_vec()),
        (916, 918, raid_request(0, &[])),
        (924, 926, vec![]),
        (985, 987, 42u64.to_le_bytes().to_vec()),
        (1059, 1061, vec![]),
        (1134, 1136, vec![2]),
        (1213, 1215, vec![]),
    ] {
        let mut h = H::new();
        h.unbind_checks();
        h.services.skill_queue_result = Gate::Fail;
        reason(&h.run(id, &body), fail, 2);
        assert!(h.services.skill_tasks.is_empty());
    }
    let mut h = H::new();
    h.services.skill_queue_result = Gate::Fail;
    let out = h.run(980, &[]);
    assert_eq!(out.status, Status::Handled);
    assert!(out.replies.is_empty());
    assert!(!out.logs.is_empty());
}

#[test]
fn unknown_queue_results_do_not_claim_a_queued_task() {
    for (id, body) in [
        (105, vec![1, 0, 1]),
        (108, vec![]),
        (913, 42u64.to_le_bytes().to_vec()),
        (916, raid_request(0, &[])),
        (924, vec![]),
        (980, vec![]),
        (985, 42u64.to_le_bytes().to_vec()),
        (1059, vec![]),
        (1134, vec![1]),
        (1213, vec![]),
    ] {
        let mut h = H::new();
        h.unbind_checks();
        h.services.skill_queue_result = Gate::Unknown;
        unsupported(&h.run(id, &body));
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn missing_zone_sends_no_synthetic_success() {
    for (id, fail, body) in [
        (108, 110, vec![]),
        (913, 915, 42u64.to_le_bytes().to_vec()),
        (916, 918, raid_request(0, &[])),
        (924, 926, vec![]),
        (985, 987, 42u64.to_le_bytes().to_vec()),
        (1059, 1061, vec![]),
        (1134, 1136, vec![1]),
        (1213, 1215, vec![]),
    ] {
        let mut h = H::new();
        h.unbind_checks();
        h.pc().zone = None;
        reason(&h.run(id, &body), fail, 2);
        assert!(h.services.skill_tasks.is_empty());
    }
    let mut h = H::new();
    h.pc().zone = None;
    let out = h.run(980, &[]);
    assert!(out.replies.is_empty());
    assert!(!out.logs.is_empty());
}

#[test]
fn native_kind_gates_exist_only_on_native_paths() {
    for (id, fail, body) in [
        (913, 915, 42u64.to_le_bytes().to_vec()),
        (916, 918, raid_request(0, &[])),
        (924, 926, vec![]),
        (985, 987, 42u64.to_le_bytes().to_vec()),
        (1059, 1061, vec![]),
    ] {
        let mut h = H::new();
        h.unbind_checks();
        h.pc().native_kind = 2;
        reason(&h.run(id, &body), fail, 2);
    }
    for (id, body) in [(108, vec![]), (1134, vec![1]), (1213, vec![])] {
        let mut h = H::new();
        h.pc().native_kind = 2;
        let out = h.run(id, &body);
        assert_eq!(out.status, Status::Handled);
        assert_eq!(h.services.skill_tasks.len(), 1);
    }
    let mut h = H::new();
    h.pc().native_kind = 2;
    let out = h.run(980, &[]);
    assert!(out.replies.is_empty());
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn raid_arena_form_b_precedes_feature_and_decode() {
    for arena in [true, false] {
        for (id, fail) in [(913, 915), (916, 918), (921, 923)] {
            let mut h = H::new();
            h.world.arena = arena;
            if !arena {
                h.pc().arena_state = 3;
            }
            reason(&h.run(id, &[]), fail, 5);
            assert!(h.services.season.checks.is_empty());
        }
        let mut h = H::new();
        h.world.arena = arena;
        if !arena {
            h.pc().arena_state = 3;
        }
        let out = h.run(924, &[]);
        assert!(out.replies.is_empty());
        assert!(!out.logs.is_empty());
        assert!(h.services.season.checks.is_empty());
    }
}

#[test]
fn raid_feature_unknown_and_disabled_are_distinct() {
    for (id, fail) in [(913, 915), (916, 918), (921, 923), (924, 926)] {
        let mut h = H::new();
        unsupported(&h.run(id, &[]));
        assert_eq!(h.services.season.checks, vec![SeasonCheck::RaidEnabled]);
        h.gate(SeasonCheck::RaidEnabled, Gate::Fail);
        reason(&h.run(id, &[]), fail, 2);
    }
}

#[test]
fn bound_raid_task_retains_the_qword() {
    let mut h = H::new();
    h.allow(SeasonCheck::RaidEnabled);
    let body = 0x1122334455667788u64.to_le_bytes();
    let out = h.run(913, &body);
    queued(
        &h,
        &out,
        913,
        &body,
        SeasonTask::BoundRaidInfo {
            raid: 0x1122334455667788,
        },
    );
    reason(&h.run(913, &body[..7]), 915, 2);
}

#[test]
fn unbind_lookup_precedes_count_and_zone() {
    let mut h = H::new();
    h.allow(SeasonCheck::RaidEnabled);
    h.gate(SeasonCheck::RaidExists(42), Gate::Fail);
    h.pc().zone = None;
    reason(&h.run(916, &42u64.to_le_bytes()), 918, 2099);
    assert_eq!(
        h.services.season.checks,
        vec![SeasonCheck::RaidEnabled, SeasonCheck::RaidExists(42)]
    );
    h.allow(SeasonCheck::RaidExists(42));
    reason(&h.run(916, &42u64.to_le_bytes()), 918, 2);
    reason(&h.run(916, &[]), 918, 2);
}

#[test]
fn unbind_count_pair_and_geo_gates_keep_native_order() {
    for body in [
        raid_request(-1, &[]),
        raid_request(16, &[]),
        raid_request(1, &[]),
    ] {
        let mut h = H::new();
        h.unbind_checks();
        reason(&h.run(916, &body), 918, 2);
        assert_eq!(
            h.services.season.checks,
            vec![SeasonCheck::RaidEnabled, SeasonCheck::RaidExists(42)]
        );
    }
    let gates = [
        (SeasonCheck::UnbindGeoPresent(9), 2),
        (SeasonCheck::UnbindGeoAllowed(9), 2103),
        (SeasonCheck::UnbindTrialAllowed, 2104),
    ];
    for (gate, code) in gates {
        let mut h = H::new();
        h.unbind_checks();
        h.gate(gate.clone(), Gate::Fail);
        h.pc().native_kind = 0;
        reason(&h.run(916, &raid_request(0, &[])), 918, code);
        assert_eq!(h.services.season.checks.last(), Some(&gate));
    }
}

#[test]
fn unbind_task_keeps_all_pairs_in_order() {
    let mut h = H::new();
    h.unbind_checks();
    let pairs = vec![(0, 65535), (42, 7)];
    let body = raid_request(2, &pairs);
    let out = h.run(916, &body);
    queued(
        &h,
        &out,
        916,
        &body,
        SeasonTask::UnbindRaid { raid: 42, pairs },
    );
    assert_eq!(
        h.services.season.checks,
        vec![
            SeasonCheck::RaidEnabled,
            SeasonCheck::RaidExists(42),
            SeasonCheck::UnbindGeoPresent(9),
            SeasonCheck::UnbindGeoAllowed(9),
            SeasonCheck::UnbindTrialAllowed
        ]
    );
}

#[test]
fn unbind_accepts_fifteen_pairs_but_not_sixteen() {
    let mut h = H::new();
    h.unbind_checks();
    let body = raid_request(15, &[(1, 2); 15]);
    assert_eq!(h.run(916, &body).status, Status::Handled);
    assert_eq!(h.services.skill_tasks.len(), 1);
    reason(&h.run(916, &raid_request(16, &[(1, 2); 16])), 918, 2);
}

#[test]
fn online_raid_property_lookup_and_snapshot_are_distinct() {
    let mut h = H::new();
    h.allow(SeasonCheck::RaidEnabled);
    reason(&h.run(921, &[]), 923, 2);
    h.gate(SeasonCheck::RaidProperty(42), Gate::Fail);
    reason(&h.run(921, &42u64.to_le_bytes()), 923, 2110);
    h.allow(SeasonCheck::RaidProperty(42));
    h.gate(SeasonCheck::RaidExists(42), Gate::Fail);
    reason(&h.run(921, &42u64.to_le_bytes()), 923, 2109);
    h.allow(SeasonCheck::RaidExists(42));
    unsupported(&h.run(921, &42u64.to_le_bytes()));
    assert!(h.services.skill_tasks.is_empty());
}

#[test]
fn expiration_sets_mode_one() {
    let mut h = H::new();
    h.allow(SeasonCheck::RaidEnabled);
    let out = h.run(924, &[]);
    queued(&h, &out, 924, &[], SeasonTask::RaidExpiration { mode: 1 });
}

#[test]
fn attraction_reward_decode_precedes_zone_and_does_not_reach_task() {
    let mut h = H::new();
    reason(&h.run(985, &[1; 7]), 987, 2);
    assert!(h.services.skill_tasks.is_empty());
    let out = h.run(985, &[255; 8]);
    queued(&h, &out, 985, &[255; 8], SeasonTask::TakeAttractionReward);
}

#[test]
fn ranking_feature_precedes_arena_and_decode() {
    for (id, fail, disabled) in [
        (989, 991, 2227),
        (992, 994, 2229),
        (995, 997, 2232),
        (998, 1000, 2229),
        (1001, 1003, 2241),
        (1088, 1090, 2235),
    ] {
        let mut h = H::new();
        h.world.arena = true;
        unsupported(&h.run(id, &[]));
        h.gate(SeasonCheck::RankingEnabled, Gate::Fail);
        reason(&h.run(id, &[]), fail, disabled);
        h.allow(SeasonCheck::RankingEnabled);
        reason(&h.run(id, &[]), fail, 5);
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn global_arena_ranking_gates_do_not_add_pc_arena_gates() {
    for (id, body) in [
        (989, vec![]),
        (992, vec![1]),
        (995, vec![8]),
        (998, vec![]),
        (1088, vec![13]),
    ] {
        let mut h = H::new();
        h.allow(SeasonCheck::RankingEnabled);
        h.allow(SeasonCheck::GuildExists(0));
        h.allow(SeasonCheck::ChallengePartyExists);
        h.pc().arena_state = 3;
        unsupported(&h.run(id, &body));
    }
    let mut h = H::new();
    h.allow(SeasonCheck::RankingEnabled);
    h.pc().arena_state = 3;
    reason(&h.run(1001, &[]), 1003, 5);
}

#[test]
fn own_ranking_distinguishes_decode_zero_range_and_cache() {
    let mut h = H::new();
    h.allow(SeasonCheck::RankingEnabled);
    reason(&h.run(992, &[]), 994, 2);
    reason(&h.run(992, &[0]), 994, 2230);
    for contents in [17, 128, 255] {
        reason(&h.run(992, &[contents]), 994, 2231);
    }
    for contents in [1, 2, 16] {
        unsupported(&h.run(992, &[contents]));
    }
}

#[test]
fn guild_ranking_lookup_precedes_zero_snapshot() {
    let mut h = H::new();
    h.allow(SeasonCheck::RankingEnabled);
    h.pc().guild_id = 123;
    reason(&h.run(995, &[]), 997, 2);
    reason(&h.run(995, &[0]), 997, 2233);
    assert_eq!(
        h.services.season.checks,
        vec![SeasonCheck::RankingEnabled; 2]
    );
    unsupported(&h.run(995, &[1]));
    h.gate(SeasonCheck::GuildExists(123), Gate::Fail);
    reason(&h.run(995, &[1]), 997, 1554);
    h.allow(SeasonCheck::GuildExists(123));
    for contents in [1, 2, 7, 9, 16, 17, 255] {
        let mut body = vec![0; 35];
        body[0] = contents;
        reply(&h.run(995, &[contents]), 996, &body);
    }
    unsupported(&h.run(995, &[8]));
}

#[test]
fn reward_cache_paths_do_not_invent_backend_data() {
    for id in [989, 998] {
        let mut h = H::new();
        h.allow(SeasonCheck::RankingEnabled);
        unsupported(&h.run(id, &[]));
        assert!(h.services.skill_tasks.is_empty());
    }
}

#[test]
fn ranking_reward_zero_fields_precede_reward_map_boundary() {
    let mut h = H::new();
    h.allow(SeasonCheck::RankingEnabled);
    reason(&h.run(1001, &[1, 1, 0, 0, 0]), 1003, 2);
    for body in [[0, 1, 0, 0, 0, 1], [1, 0, 0, 0, 0, 1], [1, 1, 0, 0, 0, 0]] {
        reason(&h.run(1001, &body), 1003, 2242);
    }
    unsupported(&h.run(1001, &[1, 255, 255, 255, 255, 1]));
    // No type-range gate belongs before the native reward-map lookup.
    unsupported(&h.run(1001, &[255, 1, 0, 0, 0, 255]));
}

#[test]
fn challenge_party_ranking_has_exact_type_and_lookup_reasons() {
    let mut h = H::new();
    h.allow(SeasonCheck::RankingEnabled);
    reason(&h.run(1088, &[]), 1090, 2);
    for contents in [0, 1, 12, 14, 255] {
        reason(&h.run(1088, &[contents]), 1090, 2236);
    }
    h.gate(SeasonCheck::ChallengePartyExists, Gate::Fail);
    reason(&h.run(1088, &[13]), 1090, 2238);
    h.allow(SeasonCheck::ChallengePartyExists);
    unsupported(&h.run(1088, &[13]));
}

#[test]
fn pvp_mode_is_a_boolean_byte_not_a_qword() {
    for value in [0, 1, 2, 128, 255] {
        let mut h = H::new();
        let out = h.run(1134, &[value]);
        queued(
            &h,
            &out,
            1134,
            &[value],
            SeasonTask::SetPvpMode {
                enabled: value != 0,
            },
        );
    }
    let mut h = H::new();
    reason(&h.run(1134, &[]), 1136, 2);
}

#[test]
fn parity_tags_resolve_to_all_17_doc_headings() {
    let source = include_str!("../src/handlers/season.rs");
    let doc = include_str!("../../../../Docs/Server/GameDaemon/28-season-ranking-handlers.md");
    for &id in SEASON_COMMANDS {
        let prefix = format!("## {id} — ");
        let heading = doc.lines().find(|line| line.starts_with(&prefix)).unwrap();
        let slug = heading
            .trim_start_matches("## ")
            .replace('—', "")
            .replace(' ', "-")
            .to_lowercase();
        let tag = format!("/// parity: {id} 28-season-ranking-handlers.md#{slug} partial");
        assert!(source.contains(&tag), "missing tag {tag}");
    }
}
