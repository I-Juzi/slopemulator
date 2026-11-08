//! Doc 26: 705 G_EnterArenaReqCmd and 955 G_ChoiceReturnToArenaLobbyReqCmd.
use gamedaemon::handlers::*;

const PC_ID: u64 = (1 << 48) | 0x11;

struct H {
    session: Session,
    world: World,
    services: MemoryServices,
}
impl H {
    /// Lobby PC session for 955.
    fn lobby() -> Self {
        let mut world = World::default();
        world.party_config.arena_enabled = true;
        let mut services = MemoryServices {
            skill_queue_result: Gate::Pass,
            ..MemoryServices::default()
        };
        services.arena_event.portal = Gate::Pass;
        services.arena_event.send = Gate::Pass;
        services.arena_event.choice_pending = Gate::Fail;
        Self {
            session: Session {
                authenticated: true,
                pc: Some(Pc {
                    object_id: PC_ID,
                    db_id: 77,
                    zone: Some(9),
                    native_kind: 1,
                    short_96: 3,
                    user_present: true,
                    ..Pc::default()
                }),
                dispatch_permission: Some(true),
                ..Session::default()
            },
            world,
            services,
        }
    }
    /// Fresh connection for 705: no PC, not authenticated.
    fn connection() -> Self {
        let mut h = Self::lobby();
        h.session.authenticated = false;
        h.session.pc = None;
        h.services.arena_event.entry_attach = Gate::Pass;
        h.services.arena_event.entry_name = Gate::Pass;
        h
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
}
fn unsupported(out: &Outcome) {
    assert!(
        matches!(out.status, Status::Unsupported(why) if why.starts_with("❓")),
        "{out:?}"
    );
    assert!(out.replies.is_empty());
}

fn key() -> ArenaEntryKey {
    ArenaEntryKey {
        key_a: 0x1111,
        world: 4,
        key_b: 0x2222,
    }
}
fn entry_body(k: &ArenaEntryKey) -> Vec<u8> {
    let mut b = k.key_a.to_le_bytes().to_vec();
    b.extend(k.world.to_le_bytes());
    b.extend(k.key_b.to_le_bytes());
    b
}
fn entry_pc() -> Option<Box<Pc>> {
    Some(Box::new(Pc {
        object_id: PC_ID,
        user_present: true,
        ..Pc::default()
    }))
}

#[test]
fn enter_arena_needs_an_unauthenticated_connection() {
    let mut h = H::lobby();
    let out = h.run(705, &entry_body(&key()));
    assert!(out.replies.is_empty());
    assert!(h.services.arena_event.entry_lookups.is_empty());
}

#[test]
fn enter_arena_decode_failure_logs_only() {
    let mut h = H::connection();
    let out = h.run(705, &[1, 2, 3]);
    assert_eq!(out.status, Status::NativeFalse);
    assert!(out.replies.is_empty());
    assert!(h.services.arena_event.entry_lookups.is_empty());
}

#[test]
fn enter_arena_unknown_missing_and_userless_entries() {
    let mut h = H::connection();
    unsupported(&h.run(705, &entry_body(&key())));
    assert_eq!(h.services.arena_event.entry_lookups, vec![key()]);

    h.services.arena_event.pending_entries = Some(vec![]);
    let out = h.run(705, &entry_body(&key()));
    assert_eq!(out.status, Status::NativeFalse);
    reason(&out, 707, 1887);

    h.services.arena_event.pending_entries = Some(vec![ArenaPendingEntry {
        key: key(),
        pc: None,
    }]);
    reason(&h.run(705, &entry_body(&key())), 707, 2);
    // A found entry is consumed even when it fails.
    assert_eq!(
        h.services
            .arena_event
            .pending_entries
            .as_ref()
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn enter_arena_key_must_match_all_three_fields() {
    let mut h = H::connection();
    h.services.arena_event.pending_entries = Some(vec![ArenaPendingEntry {
        key: key(),
        pc: entry_pc(),
    }]);
    let mut other = key();
    other.world = 5;
    reason(&h.run(705, &entry_body(&other)), 707, 1887);
    assert_eq!(
        h.services
            .arena_event
            .pending_entries
            .as_ref()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn enter_arena_attaches_pc_then_name_table() {
    let mut h = H::connection();
    h.services.arena_event.pending_entries = Some(vec![ArenaPendingEntry {
        key: key(),
        pc: entry_pc(),
    }]);
    h.services.arena_event.entry_name = Gate::Fail;
    reason(&h.run(705, &entry_body(&key())), 707, 1);
    assert_eq!(h.session.pc.as_ref().unwrap().object_id, PC_ID);

    let mut h = H::connection();
    h.services.arena_event.pending_entries = Some(vec![ArenaPendingEntry {
        key: key(),
        pc: entry_pc(),
    }]);
    // Success stops before the restricted probe and world entry.
    unsupported(&h.run(705, &entry_body(&key())));
    assert_eq!(h.session.pc.as_ref().unwrap().object_id, PC_ID);

    let mut h = H::connection();
    h.services.arena_event.entry_attach = Gate::Unknown;
    h.services.arena_event.pending_entries = Some(vec![ArenaPendingEntry {
        key: key(),
        pc: entry_pc(),
    }]);
    unsupported(&h.run(705, &entry_body(&key())));
    assert!(h.session.pc.is_none());
}

#[test]
fn choice_world_gates_and_decode() {
    let mut h = H::lobby();
    h.world.party_config.arena_enabled = false;
    reason(&h.run(955, &[0]), 958, 2206);
    let mut h = H::lobby();
    h.world.arena = true;
    reason(&h.run(955, &[0]), 958, 5);
    let mut h = H::lobby();
    reason(&h.run(955, &[]), 958, 2);
}

#[test]
fn choice_pending_entry_stops_silently() {
    let mut h = H::lobby();
    h.services.arena_event.choice_pending = Gate::Pass;
    let out = h.run(955, &[1]);
    assert!(out.replies.is_empty());
    assert!(h.services.arena_event.choice_queries.is_empty());
    h.services.arena_event.choice_pending = Gate::Unknown;
    unsupported(&h.run(955, &[1]));
}

#[test]
fn choice_stay_sends_to_destination_daemon() {
    let mut h = H::lobby();
    h.services.arena_event.choice_session = Some(None);
    reason(&h.run(955, &[0]), 958, 2208);

    h.services.arena_event.choice_session = Some(Some(ArenaStayRecord {
        route: 9,
        destination: 12,
    }));
    let out = h.run(955, &[0]);
    assert_eq!(out.status, Status::Handled);
    assert!(out.replies.is_empty());
    assert_eq!(
        h.services.arena_event.choice_queries.last(),
        Some(&ArenaChoiceQuery::Session { db_id: 77 })
    );
    h.services.arena_event.send = Gate::Fail;
    reason(&h.run(955, &[0]), 958, 5);
}

#[test]
fn choice_return_finds_portal_and_queues_check() {
    let mut h = H::lobby();
    h.session.pc.as_mut().unwrap().party_user_state = 1;
    reason(&h.run(955, &[1]), 958, 2209);

    let mut h = H::lobby();
    h.services.arena_event.last_attraction = Some(None);
    reason(&h.run(955, &[1]), 958, 2208);

    h.services.arena_event.last_attraction = Some(Some(8));
    h.services.arena_event.choice_portals = Some(vec![
        ArenaChoicePortal {
            key: 40,
            portal_type: 1,
        },
        ArenaChoicePortal {
            key: 41,
            portal_type: 2,
        },
    ]);
    h.services.arena_event.portal = Gate::Fail;
    reason(&h.run(955, &[1]), 958, 1687);
    h.services.arena_event.portal = Gate::Pass;
    h.session.pc.as_mut().unwrap().zone = None;
    reason(&h.run(955, &[1]), 958, 2);
    h.session.pc.as_mut().unwrap().zone = Some(9);
    let out = h.run(955, &[1]);
    assert_eq!(out.status, Status::Handled, "{out:?}");
    let t = h.services.skill_tasks.last().unwrap();
    assert_eq!(t.command, 955);
    assert_eq!(
        t.kind,
        SkillTaskKind::ArenaEvent(ArenaEventTask::CheckPortal {
            portal_key: 41,
            mode: 1
        })
    );
    h.services.skill_queue_result = Gate::Fail;
    reason(&h.run(955, &[1]), 958, 2);
}

#[test]
fn choice_portal_scan_uses_attraction_groups() {
    let mut h = H::lobby();
    h.services.arena_event.choice_portals = Some(vec![
        ArenaChoicePortal {
            key: 40,
            portal_type: 1,
        },
        ArenaChoicePortal {
            key: 41,
            portal_type: 2,
        },
    ]);
    let scan = |h: &mut H, attraction| {
        h.services
            .arena_choice(&ArenaChoiceQuery::Portal { attraction })
    };
    assert_eq!(scan(&mut h, 3), ArenaChoiceResult::PortalKey(40));
    assert_eq!(scan(&mut h, 10), ArenaChoiceResult::PortalKey(41));
    assert_eq!(scan(&mut h, 9), ArenaChoiceResult::PortalKey(0));
}
