//! Build 381.8867. Synthetic live state only. Queueing does not execute tasks.
use gamedaemon::handlers::*;

const PC_ID: u64 = (1 << 48) | 0x11;
const TARGET: u64 = (32 << 48) | 0x22;

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
                dispatch_permission: Some(true),
                pc: Some(Pc {
                    object_id: PC_ID,
                    native_kind: 1,
                    zone: Some(9),
                    user_present: true,
                    ..Pc::default()
                }),
                ..Session::default()
            },
            world: World::default(),
            services: MemoryServices {
                skill_queue_result: Gate::Pass,
                skill_completion: MemorySkillCompletion {
                    state: Some(SkillCompletionState::default()),
                    schedule_result: Gate::Pass,
                    ..MemorySkillCompletion::default()
                },
                ..MemoryServices::default()
            },
        }
    }
    fn receive(&mut self, command: u16, body: &[u8]) -> Outcome {
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
    fn execute(&mut self, task: &SkillTask) -> Outcome {
        execute_skill_task(
            &mut Context {
                session: &mut self.session,
                world: &self.world,
                services: &mut self.services,
            },
            task,
        )
    }
    fn state(&mut self) -> &mut SkillCompletionState {
        self.services.skill_completion.state.as_mut().unwrap()
    }
    fn recall(&mut self) -> &mut RecallActionState {
        let SkillLookup::Found(state) = &mut self.state().recall else {
            panic!("recall fixture missing")
        };
        state
    }
    fn geometry(&mut self) -> &mut RecallGeometry {
        let SkillLookup::Found(state) = &mut self.state().recall_geometry else {
            panic!("geometry fixture missing")
        };
        state
    }
    fn gather(&mut self) -> &mut GatherCancelState {
        let SkillLookup::Found(state) = &mut self.state().gather else {
            panic!("gather fixture missing")
        };
        state
    }
    fn broadcast(&mut self) -> &mut GatherCancelBroadcast {
        let SkillLookup::Found(state) = &mut self.gather().broadcast else {
            panic!("broadcast fixture missing")
        };
        state
    }
    fn recall_fixture(&mut self) {
        self.state().recall = SkillLookup::Found(RecallActionState {
            current_action_class: 1,
            reply_owner: Some(PC_ID),
            owner_position: Some([0; 3]),
        });
        self.state().recall_geometry = SkillLookup::Found(RecallGeometry {
            environment: 0,
            environment_exists: false,
            environment_blocked: true,
            ground_z: -30000,
        });
    }
    fn gauge_fixture(&mut self, values: &[i32], property: Gate) {
        self.state().gauge = SkillLookup::Found(SpecialGaugeState {
            level: 50,
            minimum_level: 50,
        });
        let mock = &mut self.services.skill_completion;
        mock.gauge_values = values.iter().copied().collect();
        mock.gauge_property_result = property;
        mock.gauge_reads.clear();
        mock.gauge_property_calls.clear();
    }
    fn gather_fixture(&mut self) {
        self.state().gather = SkillLookup::Found(GatherCancelState {
            operation_class: 15,
            action_zone: 12,
            action_flags: 1,
            parent_attached: true,
            timer_present: true,
            timer_scheduler_active: true,
            timer_linked: true,
            target: TARGET,
            broadcast: SkillLookup::Found(GatherCancelBroadcast {
                zone_has_creatures: true,
                caster_found: true,
                personal_copy_enabled: true,
                personal_buffer_present: true,
                cell: 17,
                visibility: 3,
            }),
        });
    }
}
fn task(command: u16, kind: SkillTaskKind) -> SkillTask {
    SkillTask {
        command,
        caster: PC_ID,
        summon: command == 71,
        zone: if command == 71 { 4 } else { 9 },
        kind,
    }
}
fn recall_task(position: [i16; 3]) -> SkillTask {
    task(
        71,
        SkillTaskKind::RecallSummon {
            position: position.map(|v| v as u16),
        },
    )
}
fn message(command: u16, reason: u16) -> Vec<Message> {
    vec![Message {
        command,
        body: reason.to_le_bytes().to_vec(),
    }]
}
fn gauge_message(reason: u16, value: u16) -> Vec<Message> {
    vec![Message {
        command: 76,
        body: [reason.to_le_bytes(), value.to_le_bytes()].concat(),
    }]
}
fn unsupported(out: &Outcome) -> bool {
    matches!(out.status, Status::Unsupported(_))
}
fn gather_payload() -> Vec<u8> {
    let mut payload = vec![42];
    payload.extend(PC_ID.to_le_bytes());
    payload.extend(TARGET.to_le_bytes());
    payload.push(2);
    payload
}

#[test]
fn recall_idle_gate_precedes_geometry_and_distance() {
    let mut h = H::new();
    h.recall_fixture();
    h.recall().current_action_class = 15;
    h.recall().owner_position = None;
    h.state().recall_geometry = SkillLookup::Unknown;
    assert_eq!(
        h.execute(&recall_task([30000; 3])).replies,
        message(72, 178)
    );
    h.recall().current_action_class = 1;
    assert!(unsupported(&h.execute(&recall_task([0; 3]))));
    h.state().recall_geometry = SkillLookup::Missing;
    assert_eq!(h.execute(&recall_task([0; 3])).replies, message(72, 177));
}

#[test]
fn recall_environment_failures_precede_owner_range() {
    for (environment, exists, blocked, rejected) in [
        (-1, true, false, true),
        (0, false, true, false),
        (1, false, false, true),
        (1, true, true, true),
        (1, true, false, false),
    ] {
        let mut h = H::new();
        h.recall_fixture();
        let geo = h.geometry();
        geo.environment = environment;
        geo.environment_exists = exists;
        geo.environment_blocked = blocked;
        let out = h.execute(&recall_task([126, 0, 0]));
        assert_eq!(out.replies, message(72, if rejected { 177 } else { 176 }));
        assert_eq!(!out.logs.is_empty(), environment < 0);
    }
}

#[test]
fn recall_range_uses_signed_requested_xyz_before_ground_height() {
    let mut h = H::new();
    h.recall_fixture();
    for position in [[125, 0, 0], [-125, 0, 0], [0, 0, -125], [75, 100, 0]] {
        // Geometry height is -30000. It must not affect this range gate.
        let out = h.execute(&recall_task(position));
        assert!(unsupported(&out));
        assert!(out.replies.is_empty());
    }
    for position in [[126, 0, 0], [0, 0, -126], [75, 100, 1]] {
        assert_eq!(h.execute(&recall_task(position)).replies, message(72, 176));
    }
    h.recall().owner_position = Some([i16::MIN; 3]);
    assert_eq!(
        h.execute(&recall_task([i16::MAX; 3])).replies,
        message(72, 176)
    );
    h.recall().reply_owner = Some(PC_ID + 1);
    assert!(unsupported(&h.execute(&recall_task([i16::MAX; 3]))));
    h.recall().reply_owner = Some(PC_ID);
    h.recall().owner_position = None;
    assert!(unsupported(&h.execute(&recall_task([0; 3]))));
}

#[test]
fn recall_wrapper_keeps_summon_zone_and_does_not_run_action() {
    let mut h = H::new();
    h.session.pc.as_mut().unwrap().summon = Some(Summon {
        control_state: 2,
        native_kind: 2,
        zone: Some(4),
        ..Summon::default()
    });
    let body = [1, 0, 2, 0, 3, 0, 99, 0];
    let out = h.receive(71, &body);
    assert!(out.replies.is_empty());
    assert!(out.skill_effects.is_empty());
    assert_eq!(h.services.skill_tasks.last(), Some(&recall_task([1, 2, 3])));
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(h.receive(71, &body).replies, message(72, 2));
    h.session.pc.as_mut().unwrap().summon.as_mut().unwrap().zone = None;
    assert_eq!(h.receive(71, &body).replies, message(72, 2));
}

#[test]
fn gauge_wrapper_uses_pc_scheduler_and_ignores_submit_failure() {
    let mut h = H::new();
    // Body bytes do not change a command-only decode.
    let out = h.receive(74, &[0xFF]);
    assert_eq!(
        out.changes,
        vec![
            StateChange::Queued(74),
            StateChange::QueuedRequest {
                command: 74,
                body: vec![0xFF]
            }
        ]
    );
    assert!(out.replies.is_empty());
    assert!(h.services.skill_tasks.is_empty());
    assert_eq!(
        h.services.skill_completion.scheduled,
        vec![task(74, SkillTaskKind::SpecialSkillGaugeFull)]
    );
    assert!(h.services.skill_completion.gauge_reads.is_empty());
    h.services.skill_completion.schedule_result = Gate::Fail;
    assert_eq!(h.receive(74, &[]), Outcome::default());
    h.services.skill_completion.schedule_result = Gate::Unknown;
    assert!(unsupported(&h.receive(74, &[])));
    h.session.pc.as_mut().unwrap().zone = None;
    assert!(unsupported(&h.receive(74, &[])));
    h.session.pc.as_mut().unwrap().zone = Some(9);
    h.session.pc.as_mut().unwrap().native_kind = 2;
    assert!(unsupported(&h.receive(74, &[])));
}

#[test]
fn gauge_level_gate_uses_signed_level_and_nonzero_config() {
    let t = task(74, SkillTaskKind::SpecialSkillGaugeFull);
    for (level, minimum, reject) in [
        (50, 0, true),
        (49, 50, true),
        (-1, 1, true),
        (-1, -1, false),
    ] {
        let mut h = H::new();
        h.gauge_fixture(&[0, 7], Gate::Unknown);
        h.state().gauge = SkillLookup::Found(SpecialGaugeState {
            level,
            minimum_level: minimum,
        });
        assert_eq!(
            h.execute(&t).replies,
            if reject {
                gauge_message(1550, 0)
            } else {
                gauge_message(1549, 7)
            }
        );
        assert_eq!(
            h.services.skill_completion.gauge_reads.len(),
            if reject { 0 } else { 2 }
        );
        assert!(h.services.skill_completion.gauge_property_calls.is_empty());
    }
}

#[test]
fn gauge_failure_echoes_second_live_read_with_exact_short_width() {
    let mut h = H::new();
    let t = task(74, SkillTaskKind::SpecialSkillGaugeFull);
    h.gauge_fixture(&[9999, 10000], Gate::Pass);
    assert_eq!(h.execute(&t).replies, gauge_message(1549, 10000));
    assert!(h.services.skill_completion.gauge_property_calls.is_empty());
    h.gauge_fixture(&[-1, 0x12345], Gate::Pass);
    assert_eq!(h.execute(&t).replies, gauge_message(1549, 0x2345));
    h.gauge_fixture(&[9999], Gate::Pass);
    assert!(unsupported(&h.execute(&t)));
}

#[test]
fn gauge_property_failure_rereads_and_success_is_command_only() {
    let mut h = H::new();
    let t = task(74, SkillTaskKind::SpecialSkillGaugeFull);
    h.gauge_fixture(&[10000, 9998], Gate::Fail);
    assert_eq!(h.execute(&t).replies, gauge_message(2, 9998));
    assert_eq!(
        h.services.skill_completion.gauge_property_calls,
        vec![t.clone()]
    );
    h.gauge_fixture(&[10000], Gate::Pass);
    assert_eq!(
        h.execute(&t).replies,
        vec![Message {
            command: 75,
            body: vec![]
        }]
    );
    assert_eq!(h.services.skill_completion.gauge_reads.len(), 1);
    h.gauge_fixture(&[10000], Gate::Unknown);
    assert!(unsupported(&h.execute(&t)));
    h.gauge_fixture(&[10000], Gate::Fail);
    assert!(unsupported(&h.execute(&t)));
}

#[test]
fn gauge_missing_parent_logs_and_unknown_parent_stops() {
    let mut h = H::new();
    let t = task(74, SkillTaskKind::SpecialSkillGaugeFull);
    h.state().gauge = SkillLookup::Missing;
    let out = h.execute(&t);
    assert!(out.replies.is_empty());
    assert!(!out.logs.is_empty());
    assert!(h.services.skill_completion.gauge_reads.is_empty());
    h.state().gauge = SkillLookup::Unknown;
    assert!(unsupported(&h.execute(&t)));
    h.services.skill_completion.state = None;
    assert!(unsupported(&h.execute(&t)));
}

#[test]
fn gather_wrapper_failures_and_queue_do_not_cancel() {
    let mut h = H::new();
    h.gather_fixture();
    let out = h.receive(77, &[1, 2, 3]);
    assert_eq!(
        out.changes,
        vec![
            StateChange::Queued(77),
            StateChange::QueuedRequest {
                command: 77,
                body: vec![1, 2, 3]
            }
        ]
    );
    assert_eq!(h.gather().operation_class, 15);
    assert_eq!(h.gather().action_flags, 1);
    assert!(out.skill_effects.is_empty());
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(h.receive(77, &[]).replies, message(78, 1));
    h.services.skill_queue_result = Gate::Unknown;
    assert!(unsupported(&h.receive(77, &[])));
    h.session.pc.as_mut().unwrap().native_kind = 2;
    assert_eq!(h.receive(77, &[]).replies, message(78, 1));
    h.session.pc.as_mut().unwrap().zone = None;
    assert_eq!(h.receive(77, &[]).replies, message(78, 1));
}

#[test]
fn gather_non_gather_operation_fails_without_mutation() {
    let mut h = H::new();
    let t = task(77, SkillTaskKind::GatherCancel);
    h.gather_fixture();
    for class in [0, 1, 14, 16, 50] {
        h.gather().operation_class = class;
        let before = h.gather().clone();
        let out = h.execute(&t);
        assert_eq!(out.replies, message(78, 684));
        assert!(!out.logs.is_empty());
        assert!(out.skill_effects.is_empty());
        assert_eq!(*h.gather(), before);
    }
}

#[test]
fn gather_cancel_has_exact_event_bytes_and_native_effect_order() {
    let mut h = H::new();
    let t = task(77, SkillTaskKind::GatherCancel);
    h.gather_fixture();
    let out = h.execute(&t);
    assert_eq!(out.status, Status::Handled);
    assert_eq!(
        out.skill_effects,
        vec![
            SkillEffect::GatherCancelled,
            SkillEffect::AppendPersonalAction {
                caster: PC_ID,
                payload: gather_payload()
            },
            SkillEffect::AppendZoneAction {
                zone: 12,
                cell: 17,
                event_kind: 42,
                visibility: 3,
                payload: gather_payload()
            },
            SkillEffect::GatherTimerUnlinked,
            SkillEffect::GatherParentDetached,
            SkillEffect::GatherOperationReplaced,
        ]
    );
    assert!(out.replies.is_empty() && out.broadcasts.is_empty());
    assert!(out.daemon_requests.is_empty() && out.kicks.is_empty());
    assert_eq!(h.gather().operation_class, 1);
    assert_eq!(h.gather().action_flags, 5);
    assert!(!h.gather().timer_linked);
    assert!(!h.gather().parent_attached);
    assert_eq!(h.execute(&t).replies, message(78, 684));
}

#[test]
fn gather_already_cancelled_replaces_operation_without_repeating_effects() {
    let mut h = H::new();
    h.gather_fixture();
    h.gather().action_flags = 5;
    h.gather().broadcast = SkillLookup::Unknown;
    let out = h.execute(&task(77, SkillTaskKind::GatherCancel));
    assert_eq!(
        out.skill_effects,
        vec![SkillEffect::GatherOperationReplaced]
    );
    assert!(!out.logs.is_empty());
    assert!(h.gather().timer_linked);
    assert!(h.gather().parent_attached);
    assert_eq!(h.gather().operation_class, 1);
}

#[test]
fn gather_parent_detach_respects_both_deferred_flags() {
    for flags in [0, 1, 2, 3, 8, 9, 10, 11] {
        let mut h = H::new();
        h.gather_fixture();
        h.gather().action_flags = flags;
        let out = h.execute(&task(77, SkillTaskKind::GatherCancel));
        assert_eq!(h.gather().action_flags, flags | 4);
        assert_eq!(h.gather().parent_attached, flags & 0xA != 0);
        assert_eq!(
            out.skill_effects
                .contains(&SkillEffect::GatherParentDetached),
            flags & 0xA == 0
        );
        assert_eq!(h.gather().operation_class, 1);
    }
}

#[test]
fn gather_timer_unlinks_only_a_live_link_in_an_active_scheduler() {
    for present in [false, true] {
        for active in [false, true] {
            for linked in [false, true] {
                let mut h = H::new();
                h.gather_fixture();
                let gather = h.gather();
                gather.timer_present = present;
                gather.timer_scheduler_active = active;
                gather.timer_linked = linked;
                let out = h.execute(&task(77, SkillTaskKind::GatherCancel));
                let unlink = present && active && linked;
                assert_eq!(
                    out.skill_effects
                        .contains(&SkillEffect::GatherTimerUnlinked),
                    unlink
                );
                assert_eq!(h.gather().timer_linked, linked && !unlink);
            }
        }
    }
}

#[test]
fn gather_zone_and_personal_action_buffer_routes() {
    for creatures in [false, true] {
        for found in [false, true] {
            for personal in [false, true] {
                for buffer in [false, true] {
                    let mut h = H::new();
                    h.gather_fixture();
                    let b = h.broadcast();
                    b.zone_has_creatures = creatures;
                    b.caster_found = found;
                    b.personal_copy_enabled = personal;
                    b.personal_buffer_present = buffer;
                    let out = h.execute(&task(77, SkillTaskKind::GatherCancel));
                    let personal_events = out
                        .skill_effects
                        .iter()
                        .filter(|e| matches!(e, SkillEffect::AppendPersonalAction { .. }))
                        .count();
                    assert_eq!(
                        personal_events,
                        usize::from(creatures && found && personal && buffer)
                    );
                    let zone = out
                        .skill_effects
                        .iter()
                        .find(|e| matches!(e, SkillEffect::AppendZoneAction { .. }));
                    assert_eq!(zone.is_some(), creatures);
                    if let Some(SkillEffect::AppendZoneAction { visibility, .. }) = zone {
                        assert_eq!(*visibility, if found { 3 } else { 0 });
                    }
                    assert_eq!(
                        !out.logs.is_empty(),
                        creatures && found && personal && !buffer
                    );
                }
            }
        }
    }
}

#[test]
fn gather_unknown_buffer_preserves_only_the_completed_prefix() {
    let mut h = H::new();
    h.gather_fixture();
    h.gather().broadcast = SkillLookup::Unknown;
    let t = task(77, SkillTaskKind::GatherCancel);
    let out = h.execute(&t);
    assert!(unsupported(&out));
    assert_eq!(out.skill_effects, vec![SkillEffect::GatherCancelled]);
    assert_eq!(h.gather().action_flags, 5);
    assert_eq!(h.gather().operation_class, 15);
    assert!(h.gather().timer_linked && h.gather().parent_attached);
    h.state().gather = SkillLookup::Unknown;
    assert!(unsupported(&h.execute(&t)));
    h.state().gather = SkillLookup::Missing;
    assert!(unsupported(&h.execute(&t)));
}

#[test]
fn gather_missing_action_parent_stops_before_callback_and_idle_replacement() {
    let mut h = H::new();
    h.gather_fixture();
    h.gather().parent_attached = false;
    let out = h.execute(&task(77, SkillTaskKind::GatherCancel));
    assert!(unsupported(&out));
    assert_eq!(out.skill_effects, vec![SkillEffect::GatherCancelled]);
    assert!(out.replies.is_empty());
    assert_eq!(h.gather().operation_class, 15);
    assert!(h.gather().timer_linked);
}

#[test]
fn opaque_skill_defaults_never_approve_task_execution() {
    let mut h = H::new();
    h.services = MemoryServices::default();
    for t in [
        recall_task([0; 3]),
        task(74, SkillTaskKind::SpecialSkillGaugeFull),
        task(77, SkillTaskKind::GatherCancel),
    ] {
        let out = h.execute(&t);
        assert!(unsupported(&out));
        assert!(out.replies.is_empty());
        assert!(out.skill_effects.is_empty());
    }
    assert!(unsupported(&h.receive(74, &[])));
    assert!(h.services.skill_completion.scheduled.is_empty());
}

#[test]
fn task_execution_rejects_another_pc_or_a_mismatched_task_identity() {
    let mut h = H::new();
    h.gather_fixture();
    let mut t = task(77, SkillTaskKind::GatherCancel);
    t.caster += 1;
    assert!(unsupported(&h.execute(&t)));
    t.caster = PC_ID;
    t.command = 74;
    assert!(unsupported(&h.execute(&t)));
    t.command = 77;
    t.summon = true;
    assert!(unsupported(&h.execute(&t)));
}

fn list_fixture(h: &mut H) {
    h.state().skill_list = SkillLookup::Found(SkillListState {
        reply_owner: Some(PC_ID),
        ..SkillListState::default()
    });
}
fn list(h: &mut H) -> &mut SkillListState {
    let SkillLookup::Found(state) = &mut h.state().skill_list else {
        panic!("list fixture missing")
    };
    state
}
fn list_entry(id: u32, enabled: u8, bytes: [u8; 2]) -> Option<SkillListEntry> {
    Some(SkillListEntry { enabled, id, bytes })
}

#[test]
fn skill_list_wrapper_failure_and_queue_are_separate_from_list_send() {
    let mut h = H::new();
    list_fixture(&mut h);
    h.session.pc.as_mut().unwrap().zone = None;
    assert_eq!(h.receive(113, &[]).replies, message(115, 2));
    h.session.pc.as_mut().unwrap().zone = Some(9);
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(h.receive(113, &[]).replies, message(115, 2));
    h.services.skill_queue_result = Gate::Pass;
    // The wrapper has no explicit PC-kind gate or body check.
    h.session.pc.as_mut().unwrap().native_kind = 2;
    assert!(h.receive(113, &[0xFF]).replies.is_empty());
    assert_eq!(
        h.services.skill_tasks.last(),
        Some(&task(113, SkillTaskKind::SkillList))
    );
    let before = h.state().clone();
    assert_eq!(
        h.execute(&task(113, SkillTaskKind::SkillList)).replies,
        vec![Message {
            command: 114,
            body: vec![0, 0]
        }]
    );
    assert_eq!(*h.state(), before);
}

#[test]
fn skill_list_exact_nested_counts_filters_and_native_order() {
    let mut h = H::new();
    list_fixture(&mut h);
    list(&mut h).pages = [
        SkillListPage {
            flags: 1,
            map_size: 5,
            value: -1,
            entries: vec![
                list_entry(0x12345678, 2, [0x80, 0xFF]),
                None,
                list_entry(5, 0, [1, 2]),
                list_entry(1, 1, [3, 4]),
                list_entry(1, 1, [3, 4]),
            ],
        },
        // Bit 1 alone does not select a page.
        SkillListPage {
            flags: 2,
            map_size: 1,
            value: 2,
            entries: vec![list_entry(9, 1, [1, 2])],
        },
        // Zero native size suppresses a page even if the mock has entries.
        SkillListPage {
            flags: 1,
            map_size: 0,
            value: 3,
            entries: vec![list_entry(10, 1, [1, 2])],
        },
        SkillListPage {
            flags: 0xFF,
            map_size: 1,
            value: -128,
            entries: vec![None],
        },
        SkillListPage {
            flags: 1,
            map_size: 2,
            value: 127,
            entries: vec![list_entry(u32::MAX, 0xFF, [0, 0x80])],
        },
    ];
    let before = h.state().clone();
    let out = h.execute(&task(113, SkillTaskKind::SkillList));
    assert_eq!(
        out.replies,
        vec![Message {
            command: 114,
            body: vec![
                3, 0, // Three groups.
                1, 0xFF, 3, 0, // Group 1, value -1, three entries.
                0x78, 0x56, 0x34, 0x12, 0x80, 0xFF, 1, 0, 0, 0, 3, 4, 1, 0, 0, 0, 3,
                4, // No deduplication or new sort.
                4, 0x80, 0, 0, // Included page with no emitted entries.
                5, 127, 1, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0x80,
            ]
        }]
    );
    assert_eq!(*h.state(), before);
    assert!(out.skill_effects.is_empty() && out.broadcasts.is_empty());
    assert!(out.daemon_requests.is_empty() && out.kicks.is_empty());
}

#[test]
fn skill_list_all_five_groups_include_zero_entry_groups() {
    let mut h = H::new();
    list_fixture(&mut h);
    for page in &mut list(&mut h).pages {
        page.flags = 1;
        page.map_size = u64::MAX;
    }
    assert_eq!(
        h.execute(&task(113, SkillTaskKind::SkillList)).replies,
        vec![Message {
            command: 114,
            body: vec![5, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0],
        }]
    );
}

#[test]
fn skill_list_capacity_includes_size_command_and_both_counts() {
    let mut h = H::new();
    list_fixture(&mut h);
    let page = &mut list(&mut h).pages[0];
    page.flags = 1;
    page.map_size = 665;
    page.entries = vec![list_entry(7, 1, [1, 2]); 665];
    let t = task(113, SkillTaskKind::SkillList);
    let out = h.execute(&t);
    assert_eq!(out.replies.len(), 1);
    assert_eq!(out.replies[0].body.len() + 4, 4000);
    assert_eq!(&out.replies[0].body[..6], &[1, 0, 1, 0, 0x99, 2]);
    list(&mut h).pages[0].entries.push(list_entry(8, 1, [1, 2]));
    let out = h.execute(&t);
    assert!(unsupported(&out));
    assert!(out.replies.is_empty());
    // A second group alone also exceeds this exactly full packet.
    list(&mut h).pages[0].entries.pop();
    list(&mut h).pages[1].flags = 1;
    list(&mut h).pages[1].map_size = 1;
    assert!(unsupported(&h.execute(&t)));
}

#[test]
fn skill_list_unknown_or_changed_owner_does_not_send_an_empty_list() {
    let mut h = H::new();
    let t = task(113, SkillTaskKind::SkillList);
    assert!(unsupported(&h.execute(&t)));
    h.state().skill_list = SkillLookup::Missing;
    assert!(unsupported(&h.execute(&t)));
    list_fixture(&mut h);
    list(&mut h).reply_owner = None;
    assert!(unsupported(&h.execute(&t)));
    list(&mut h).reply_owner = Some(PC_ID + 1);
    assert!(unsupported(&h.execute(&t)));
    h.services = MemoryServices::default();
    assert!(unsupported(&h.execute(&t)));
}

fn exchange_task(amount: i32) -> SkillTask {
    task(
        83,
        CombatTask::ExchangeFactionScore {
            npc: TARGET,
            amount,
        }
        .into(),
    )
}
fn exchange_fixture(h: &mut H) {
    h.state().exchange = SkillLookup::Found(ExchangeFactionState {
        position: [0; 3],
        score: 20,
        npc: SkillLookup::Found(ExchangeFactionNpc {
            native_kind: 4,
            position: [100, 0, 0],
            record: Some(ExchangeFactionRecord {
                enabled: true,
                maximum_level: 5,
                multiplier: 2,
            }),
        }),
    });
    h.services.skill_completion.faction_level = Some(5);
}
fn exchange(h: &mut H) -> &mut ExchangeFactionState {
    let SkillLookup::Found(state) = &mut h.state().exchange else {
        panic!("exchange fixture missing")
    };
    state
}
fn exchange_npc(h: &mut H) -> &mut ExchangeFactionNpc {
    let SkillLookup::Found(npc) = &mut exchange(h).npc else {
        panic!("NPC fixture missing")
    };
    npc
}

#[test]
fn exchange_wrapper_gates_and_queue_do_not_run_the_task() {
    let mut h = H::new();
    exchange_fixture(&mut h);
    let body = [TARGET.to_le_bytes().as_slice(), &20i32.to_le_bytes()].concat();
    h.world.arena = true;
    h.session.pc.as_mut().unwrap().user_present = false;
    assert_eq!(h.receive(83, &[]).replies, message(85, 5));
    h.world.arena = false;
    assert_eq!(h.receive(83, &[]).replies, message(85, 336));
    h.session.pc.as_mut().unwrap().user_present = true;
    h.session.pc.as_mut().unwrap().zone = None;
    assert_eq!(h.receive(83, &body).replies, message(85, 1));
    h.session.pc.as_mut().unwrap().zone = Some(9);
    h.session.pc.as_mut().unwrap().native_kind = 2;
    assert_eq!(h.receive(83, &body).replies, message(85, 1));
    h.session.pc.as_mut().unwrap().native_kind = 1;
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(h.receive(83, &body).replies, message(85, 1));
    h.services.skill_queue_result = Gate::Pass;
    let out = h.receive(83, &body);
    assert!(out.replies.is_empty());
    assert_eq!(h.services.skill_tasks.last(), Some(&exchange_task(20)));
    assert!(h.services.skill_completion.faction_level_calls.is_empty());
}

#[test]
fn exchange_npc_and_record_gates_precede_level_and_range() {
    let t = exchange_task(-1);
    for case in 0..4 {
        let mut h = H::new();
        exchange_fixture(&mut h);
        match case {
            0 => exchange(&mut h).npc = SkillLookup::Missing,
            1 => exchange_npc(&mut h).native_kind = 2,
            2 => exchange_npc(&mut h).record = None,
            3 => exchange_npc(&mut h).record.as_mut().unwrap().enabled = false,
            _ => unreachable!(),
        }
        h.services.skill_completion.faction_level = None;
        assert_eq!(h.execute(&t).replies, message(85, 333));
        assert!(h.services.skill_completion.faction_level_calls.is_empty());
    }
}

#[test]
fn exchange_level_gate_is_positive_limit_and_strictly_greater() {
    for (maximum, level, rejected) in [
        (5, 6, true),
        (5, 5, false),
        (0, 100, false),
        (-1, 100, false),
        (5, -1, false),
    ] {
        let mut h = H::new();
        exchange_fixture(&mut h);
        exchange_npc(&mut h).record.as_mut().unwrap().maximum_level = maximum;
        exchange_npc(&mut h).position = [101, 0, 0];
        h.services.skill_completion.faction_level = Some(level);
        assert_eq!(
            h.execute(&exchange_task(0)).replies,
            message(85, if rejected { 333 } else { 334 })
        );
        assert_eq!(
            h.services.skill_completion.faction_level_calls,
            vec![exchange_task(0)]
        );
    }
}

#[test]
fn exchange_range_uses_signed_server_xyz_with_inclusive_3d_boundary() {
    let mut h = H::new();
    exchange_fixture(&mut h);
    for position in [[100, 0, 0], [-100, 0, 0], [0, 0, 100], [60, 80, 0]] {
        exchange_npc(&mut h).position = position;
        assert_eq!(h.execute(&exchange_task(0)).replies, message(85, 335));
    }
    for position in [[101, 0, 0], [60, 80, 1], [0, 0, -101]] {
        exchange_npc(&mut h).position = position;
        assert_eq!(h.execute(&exchange_task(0)).replies, message(85, 334));
    }
    exchange(&mut h).position = [i16::MIN; 3];
    exchange_npc(&mut h).position = [i16::MAX; 3];
    assert_eq!(h.execute(&exchange_task(20)).replies, message(85, 334));
}

#[test]
fn exchange_amount_precedes_signed_multiplier_and_stops_before_mutation() {
    for amount in [i32::MIN, -1, 0, 1, 20, 21, i32::MAX] {
        for multiplier in [-128, -1, 0, 1, 127] {
            let mut h = H::new();
            exchange_fixture(&mut h);
            exchange_npc(&mut h).record.as_mut().unwrap().multiplier = multiplier;
            let before = h.state().clone();
            let out = h.execute(&exchange_task(amount));
            if !(1..=20).contains(&amount) {
                assert_eq!(out.replies, message(85, 335));
            } else if multiplier <= 0 {
                assert_eq!(out.replies, message(85, 333));
            } else {
                assert!(unsupported(&out));
                assert!(out.replies.is_empty());
            }
            assert_eq!(*h.state(), before);
            assert!(out.skill_effects.is_empty() && out.broadcasts.is_empty());
            assert!(out.daemon_requests.is_empty() && out.kicks.is_empty());
        }
    }
    let mut h = H::new();
    exchange_fixture(&mut h);
    exchange(&mut h).score = -1;
    assert_eq!(h.execute(&exchange_task(1)).replies, message(85, 335));
}

#[test]
fn exchange_unknown_dependencies_do_not_invent_replies() {
    let mut h = H::new();
    let t = exchange_task(1);
    assert!(unsupported(&h.execute(&t)));
    h.state().exchange = SkillLookup::Missing;
    assert!(unsupported(&h.execute(&t)));
    exchange_fixture(&mut h);
    exchange(&mut h).npc = SkillLookup::Unknown;
    assert!(unsupported(&h.execute(&t)));
    exchange_fixture(&mut h);
    h.services.skill_completion.faction_level = None;
    assert!(unsupported(&h.execute(&t)));
    h.services = MemoryServices::default();
    assert!(unsupported(&h.execute(&t)));
}

fn mastery_task(shorts: [i16; 2], bytes: [i8; 5]) -> SkillTask {
    task(117, SkillTaskKind::SetMasteryStatPoint { shorts, bytes })
}
fn mastery_fixture(h: &mut H) {
    h.state().mastery = SkillLookup::Found(MasteryStatState {
        native_kind: 1,
        life_state: 0,
        combat_flags: 0,
        owner: SkillLookup::Found(MasteryStatOwner {
            object_id: PC_ID,
            native_kind: 1,
            table_key: -128,
            job: 0xFE,
            additional_points: 2,
        }),
    });
    h.services.skill_completion.mastery_limits = Some(
        [(
            -128,
            MasteryStatLimit {
                total: 8,
                attacker: 6,
                defender: 7,
            },
        )]
        .into(),
    );
    h.services.skill_completion.mastery_costs = Some(Default::default());
}
fn mastery(h: &mut H) -> &mut MasteryStatState {
    let SkillLookup::Found(state) = &mut h.state().mastery else {
        panic!("mastery fixture missing")
    };
    state
}
fn mastery_owner(h: &mut H) -> &mut MasteryStatOwner {
    let SkillLookup::Found(owner) = &mut mastery(h).owner else {
        panic!("mastery owner missing")
    };
    owner
}
fn mastery_body(shorts: [i16; 2], bytes: [i8; 5]) -> Vec<u8> {
    shorts
        .into_iter()
        .flat_map(i16::to_le_bytes)
        .chain(bytes.map(|b| b as u8))
        .collect()
}

#[test]
fn mastery_wrapper_zone_kind_values_and_queue_keep_native_order() {
    let mut h = H::new();
    let invalid = mastery_body([-1, -1], [-1; 5]);
    h.session.pc.as_mut().unwrap().zone = None;
    assert_eq!(h.receive(117, &[]).replies, message(119, 2));
    h.session.pc.as_mut().unwrap().zone = Some(9);
    h.session.pc.as_mut().unwrap().native_kind = 2;
    let out = h.receive(117, &invalid);
    assert_eq!(out.replies, message(119, 2));
    assert!(!out.logs.is_empty());
    assert_eq!(h.receive(117, &[]).replies, message(119, 2));
    h.session.pc.as_mut().unwrap().native_kind = 1;
    assert!(unsupported(&h.receive(117, &[])));
    for (shorts, bytes) in [
        ([-1, 0], [0; 5]),
        ([0, -1], [0; 5]),
        ([0; 2], [0, 0, -128, 0, 0]),
    ] {
        assert_eq!(
            h.receive(117, &mastery_body(shorts, bytes)).replies,
            message(119, 2)
        );
    }
    assert!(h.services.skill_tasks.is_empty());
    let body = mastery_body([2, 3], [1, 2, 3, 4, 127]);
    h.services.skill_queue_result = Gate::Fail;
    assert_eq!(h.receive(117, &body).replies, message(119, 2));
    h.services.skill_queue_result = Gate::Pass;
    assert!(h.receive(117, &body).replies.is_empty());
    assert_eq!(
        h.services.skill_tasks.last(),
        Some(&mastery_task([2, 3], [1, 2, 3, 4, 127]))
    );
    assert!(h.services.skill_completion.mastery_limit_calls.is_empty());
}

#[test]
fn mastery_task_kind_life_and_combat_gates_precede_owner_lookup() {
    let t = mastery_task([0; 2], [0; 5]);
    let mut h = H::new();
    mastery_fixture(&mut h);
    mastery(&mut h).owner = SkillLookup::Unknown;
    mastery(&mut h).native_kind = 2;
    mastery(&mut h).life_state = 1;
    mastery(&mut h).combat_flags = 1;
    assert_eq!(h.execute(&t), Outcome::default());
    mastery(&mut h).native_kind = 1;
    for life in [1, 2] {
        mastery(&mut h).life_state = life;
        let out = h.execute(&t);
        assert_eq!(out.replies, message(119, 2));
        assert!(!out.logs.is_empty());
    }
    for life in [0, 3, 255] {
        mastery(&mut h).life_state = life;
        assert_eq!(h.execute(&t).replies, message(119, 701));
    }
    mastery(&mut h).combat_flags = 0xFE;
    assert!(unsupported(&h.execute(&t)));
    assert!(h.services.skill_completion.mastery_limit_calls.is_empty());
}

#[test]
fn mastery_owner_and_limit_unknowns_do_not_invent_replies() {
    let t = mastery_task([0; 2], [0; 5]);
    let mut h = H::new();
    assert!(unsupported(&h.execute(&t)));
    h.state().mastery = SkillLookup::Missing;
    assert!(unsupported(&h.execute(&t)));
    for owner in [SkillLookup::Unknown, SkillLookup::Missing] {
        mastery_fixture(&mut h);
        mastery(&mut h).owner = owner;
        assert!(unsupported(&h.execute(&t)));
    }
    mastery_fixture(&mut h);
    mastery_owner(&mut h).native_kind = 2;
    assert_eq!(h.execute(&t).replies, message(119, 2));
    mastery_owner(&mut h).native_kind = 1;
    mastery_owner(&mut h).object_id += 1;
    assert!(unsupported(&h.execute(&t)));
    assert!(h.services.skill_completion.mastery_limit_calls.is_empty());
    mastery_fixture(&mut h);
    h.services.skill_completion.mastery_limits = None;
    assert!(unsupported(&h.execute(&t)));
    h.services.skill_completion.mastery_limits = Some(Default::default());
    assert_eq!(h.execute(&t).replies, message(119, 2));
    assert_eq!(
        h.services.skill_completion.mastery_limit_calls,
        vec![-128, -128]
    );
    h.services = MemoryServices::default();
    assert!(unsupported(&h.execute(&t)));
}

#[test]
fn mastery_allocation_limits_are_signed_and_precede_cost_lookup() {
    for (shorts, additional, rejected) in [
        ([6, 5], 2, true),  // Total 10.
        ([7, 0], 2, true),  // Attacker limit 6.
        ([0, 8], 2, true),  // Defender limit 7.
        ([6, 4], 2, false), // Inclusive total and attacker limits.
        ([3, 7], 2, false), // Inclusive defender limit.
        ([0, 0], -9, true), // Negative total, not unsigned.
        ([0, 0], -8, false),
        ([6, 7], i16::MAX, false), // No short-width sum overflow.
    ] {
        let mut h = H::new();
        mastery_fixture(&mut h);
        mastery_owner(&mut h).additional_points = additional;
        let out = h.execute(&mastery_task(shorts, [0; 5]));
        if rejected {
            assert_eq!(out.replies, message(119, 2));
        } else {
            assert!(unsupported(&out));
            assert!(out.replies.is_empty());
        }
        assert_eq!(h.services.skill_completion.mastery_limit_calls, vec![-128]);
        assert!(h.services.skill_completion.mastery_cost_calls.is_empty());
    }
    let mut h = H::new();
    mastery_fixture(&mut h);
    assert_eq!(
        h.execute(&mastery_task([7, 0], [1; 5])).replies,
        message(119, 2)
    );
    assert!(h.services.skill_completion.mastery_cost_calls.is_empty());
}

#[test]
fn mastery_positive_pick_keys_keep_job_bits_and_native_lookup_order() {
    let mut h = H::new();
    mastery_fixture(&mut h);
    let keys = [0xFE0100, 0xFE7F02, 0xFE0204];
    h.services.skill_completion.mastery_costs = Some(keys.map(|key| (key, 1)).into());
    // Negative picks are not reachable through receive, but the native helper skips them.
    let t = mastery_task([2, 3], [1, 0, 127, -128, 2]);
    let before = h.state().clone();
    let out = h.execute(&t);
    assert!(unsupported(&out));
    assert!(out.replies.is_empty());
    assert_eq!(h.services.skill_completion.mastery_cost_calls, keys);
    assert_eq!(*h.state(), before);
    assert!(out.skill_effects.is_empty() && out.broadcasts.is_empty());
    assert!(out.daemon_requests.is_empty() && out.kicks.is_empty());
}

#[test]
fn mastery_missing_pick_stops_before_later_lookups() {
    let mut h = H::new();
    mastery_fixture(&mut h);
    h.services.skill_completion.mastery_costs = Some([(0xFE0100, 1)].into());
    let out = h.execute(&mastery_task([0; 2], [1; 5]));
    assert_eq!(out.replies, message(119, 2));
    assert_eq!(
        h.services.skill_completion.mastery_cost_calls,
        vec![0xFE0100, 0xFE0101]
    );
    h.services.skill_completion.mastery_costs = None;
    let out = h.execute(&mastery_task([0; 2], [1; 5]));
    assert!(unsupported(&out));
    assert!(out.replies.is_empty());
    assert_eq!(
        h.services.skill_completion.mastery_cost_calls.last(),
        Some(&0xFE0100)
    );
}

#[test]
fn mastery_signed_cost_budget_keeps_equality_and_negative_costs() {
    for (costs, rejected) in [
        ([2, 4], true),
        ([2, 3], false),
        ([-1, 6], false),
        ([i16::MAX, i16::MAX], true),
        ([i16::MIN, i16::MIN], false),
    ] {
        let mut h = H::new();
        mastery_fixture(&mut h);
        h.services.skill_completion.mastery_costs =
            Some([(0xFE0100, costs[0]), (0xFE0101, costs[1])].into());
        let before = h.state().clone();
        let out = h.execute(&mastery_task([2, 3], [1, 1, 0, 0, 0]));
        if rejected {
            assert_eq!(out.replies, message(119, 2));
        } else {
            assert!(unsupported(&out));
            assert!(out.replies.is_empty());
        }
        assert_eq!(*h.state(), before);
        assert_eq!(
            h.services.skill_completion.mastery_cost_calls,
            vec![0xFE0100, 0xFE0101]
        );
        assert!(out.skill_effects.is_empty() && out.daemon_requests.is_empty());
    }
}
