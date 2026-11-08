//! Doc 15 guild core handlers: reason codes, order and posted work.
use gamedaemon::handlers::*;

const PC_ID: u64 = 0x10001;
const DB_ID: u64 = 0x7700;
const GUILD: u64 = 0x5500;
const OPEN: u64 = 0x5501;

fn name(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}
fn s(text: &str) -> Vec<u8> {
    let units = name(text);
    let mut out = (units.len() as u16).to_le_bytes().to_vec();
    for u in units {
        out.extend(u.to_le_bytes());
    }
    out
}
fn q(v: u64) -> Vec<u8> {
    v.to_le_bytes().to_vec()
}

struct H {
    session: Session,
    world: World,
    services: MemoryServices,
}
impl H {
    fn new() -> Self {
        let pc = Pc {
            object_id: PC_ID,
            db_id: DB_ID,
            zone: Some(9),
            guild_id: GUILD,
            faction: 1,
            user_present: true,
            ..Pc::default()
        };
        let mut world = World::default();
        world.guild.guilds.insert(
            GUILD,
            Guild {
                guild_type: 1,
                name: name("[Old]Guild"),
                rename_in_progress: false,
                leader_db_id: DB_ID,
                craft_active: true,
                faction: 1,
                application_refusal: false,
            },
        );
        world.guild.guilds.insert(
            OPEN,
            Guild {
                guild_type: 2,
                name: name("Open"),
                rename_in_progress: false,
                leader_db_id: DB_ID,
                craft_active: true,
                faction: 1,
                application_refusal: false,
            },
        );
        for row in [1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 17] {
            world.guild.actions.insert(
                row,
                GuildActionRow {
                    normal: true,
                    open: true,
                },
            );
        }
        world.guild.config.rename_enabled = true;
        world.guild.config.rename_prefix = name("[Old]");
        world.guild.config.rename_by_item = true;
        world.guild.config.bank_enabled = true;
        world.guild.config.uniform_enabled = true;
        world.guild.config.bank_money_limit = 1000;
        world.guild.config.permission_limits = [[0xFF; 4], [0x0F; 4]];
        let mut services = MemoryServices::default();
        services.guild.post_result = Gate::Pass;
        services.guild.queue_result = Gate::Pass;
        services.guild.user_found = Some(true);
        services.guild.name_forbidden = Some(false);
        services.guild.rename_time_passed = Some(true);
        services.guild.join_ticket = Some(false);
        services.guild.leave_result = Some(Ok(()));
        services.guild.banish_result = Some(Ok(()));
        services.guild.flag_bearer_result = Some(0);
        services.guild.craft_contract_result = Gate::Pass;
        services.guild.craft_make_result = Some(0);
        services.guild.craft_abandon_result = Some(0);
        services.guild.uniform_name_ok = Some(true);
        services.guild.uniform_contract_result = Some(0);
        services.guild.uniform_op_result = Some(0);
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
}
fn reason(out: &Outcome) -> Option<(u16, u16)> {
    assert!(out.replies.len() <= 1, "{out:?}");
    out.replies
        .first()
        .map(|m| (m.command, u16::from_le_bytes([m.body[0], m.body[1]])))
}
fn posted(out: &Outcome) -> Option<&GuildWork> {
    out.changes.iter().find_map(|c| match c {
        StateChange::GuildWorkPosted(w) => Some(w),
        _ => None,
    })
}
fn create_body(t: u8, n: &str, v: u64) -> Vec<u8> {
    [vec![t], s(n), q(v)].concat()
}
fn rename_body(t: u8, n: &str, items: &[(u16, u16)]) -> Vec<u8> {
    let mut b = [vec![t], s(n)].concat();
    b.extend((items.len() as u16).to_le_bytes());
    for (a, c) in items {
        b.extend(a.to_le_bytes());
        b.extend(c.to_le_bytes());
    }
    b
}

#[test]
fn create_guild_checks_in_order() {
    let ok = create_body(1, "New", 7);
    let mut h = H::new();
    h.world.arena = true;
    assert_eq!(reason(&h.run(431, &ok)), Some((433, 5)));
    let mut h = H::new();
    h.pc().arena_state = 3;
    assert_eq!(reason(&h.run(431, &ok)), Some((433, 5)));
    let mut h = H::new();
    assert_eq!(reason(&h.run(431, &[1])), Some((433, 1352)));
    assert_eq!(
        reason(&h.run(431, &create_body(0, "New", 7))),
        Some((433, 1352))
    );
    h.world.guild.config.open_limit_enabled = true;
    h.world.guild.config.open_limit = 3;
    h.world.guild.config.open_guild_count = 3;
    assert_eq!(
        reason(&h.run(431, &create_body(2, "New", 7))),
        Some((433, 1352))
    );
    h.services.guild.user_found = Some(false);
    assert_eq!(reason(&h.run(431, &ok)), Some((433, 2)));
    h.services.guild.user_found = Some(true);
    h.services.guild.name_forbidden = Some(true);
    assert_eq!(reason(&h.run(431, &ok)), Some((433, 1356)));
    h.services.guild.name_forbidden = Some(false);
    assert_eq!(reason(&h.run(431, &ok)), Some((433, 1358)));
    h.pc().guild_id = 0;
    h.world.guild.config.faction_required = true;
    h.pc().faction = 0;
    assert_eq!(reason(&h.run(431, &ok)), Some((433, 2)));
    h.pc().faction = 1;
    h.pc().zone = None;
    assert_eq!(reason(&h.run(431, &ok)), Some((433, 2)));
    h.pc().zone = Some(9);
    assert_eq!(
        reason(&h.run(431, &create_body(1, "New", 0))),
        Some((433, 2))
    );
    let out = h.run(431, &ok);
    assert!(out.replies.is_empty());
    assert!(out.changes.iter().any(|c| matches!(
        c,
        StateChange::GuildTaskQueued(GuildTask {
            command: 431,
            kind: GuildTaskKind::Create {
                guild_type: 1,
                value: 7,
                ..
            },
            ..
        })
    )));
    h.services.guild.queue_result = Gate::Fail;
    assert_eq!(reason(&h.run(431, &ok)), Some((433, 2)));
}

#[test]
fn rename_guild_checks_in_order() {
    let by_prefix = rename_body(1, "New", &[]);
    let by_item = rename_body(2, "New", &[(4, 1)]);
    let mut h = H::new();
    h.pc().arena_state = 2;
    assert_eq!(reason(&h.run(434, &by_prefix)), Some((436, 5)));
    let mut h = H::new();
    h.world.guild.config.rename_enabled = false;
    assert_eq!(reason(&h.run(434, &by_prefix)), Some((436, 1363)));
    let mut h = H::new();
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(434, &by_prefix)), Some((436, 2)));
    h.pc().guild_id = 0x9999;
    assert_eq!(reason(&h.run(434, &by_prefix)), Some((436, 2)));
    let mut h = H::new();
    h.world.guild.actions.remove(&17);
    assert_eq!(reason(&h.run(434, &by_prefix)), Some((436, 2)));
    let mut h = H::new();
    h.world
        .guild
        .guilds
        .get_mut(&GUILD)
        .unwrap()
        .rename_in_progress = true;
    assert_eq!(reason(&h.run(434, &by_prefix)), Some((436, 2)));
    let mut h = H::new();
    assert_eq!(reason(&h.run(434, &[1])), Some((436, 2)));
    let too_many: Vec<_> = (0..17).map(|i| (i, 1)).collect();
    assert_eq!(
        reason(&h.run(434, &rename_body(2, "New", &too_many))),
        Some((436, 2))
    );
    assert_eq!(
        reason(&h.run(434, &rename_body(2, "New", &[(4, 0)]))),
        Some((436, 2))
    );
    // Type 1.
    assert_eq!(
        reason(&h.run(434, &rename_body(1, "New", &[(4, 1)]))),
        Some((436, 2))
    );
    h.world.guild.config.rename_prefix = name("[X]");
    assert_eq!(reason(&h.run(434, &by_prefix)), Some((436, 1368)));
    h.world.guild.config.rename_prefix.clear();
    assert_eq!(reason(&h.run(434, &by_prefix)), Some((436, 1363)));
    // Type 2.
    assert_eq!(
        reason(&h.run(434, &rename_body(2, "New", &[]))),
        Some((436, 2))
    );
    assert_eq!(
        reason(&h.run(434, &rename_body(2, "New", &[(4, 1), (5, 1)]))),
        Some((436, 2))
    );
    h.services.guild.rename_time_passed = Some(false);
    assert_eq!(reason(&h.run(434, &by_item)), Some((436, 1367)));
    h.services.guild.rename_time_passed = Some(true);
    h.world.guild.config.rename_by_item = false;
    assert_eq!(reason(&h.run(434, &by_item)), Some((436, 1363)));
    h.world.guild.config.rename_by_item = true;
    assert_eq!(
        reason(&h.run(434, &rename_body(3, "New", &[]))),
        Some((436, 2))
    );
    h.services.guild.name_forbidden = Some(true);
    assert_eq!(reason(&h.run(434, &by_item)), Some((436, 1371)));
    h.services.guild.name_forbidden = Some(false);
    let out = h.run(434, &by_item);
    assert!(out.replies.is_empty(), "{out:?}");
    assert_eq!(h.services.guild.tasks.last().unwrap().guild_id, GUILD);
}

#[test]
fn destroy_and_invite_always_fail() {
    let mut h = H::new();
    assert_eq!(reason(&h.run(437, &[])), Some((439, 1374)));
    assert_eq!(reason(&h.run(503, &s("x"))), Some((504, 1427)));
    h.pc().arena_state = 6;
    assert_eq!(reason(&h.run(437, &[])), Some((439, 5)));
    assert_eq!(reason(&h.run(503, &[])), Some((504, 5)));
}

#[test]
fn retrieve_info_and_members() {
    for (cmd, reply, none, op) in [(440, 442, 1384, 2), (477, 479, 1385, 3)] {
        let mut h = H::new();
        h.world.arena = true;
        assert_eq!(reason(&h.run(cmd, &[])), Some((reply, 5)));
        let mut h = H::new();
        // Form A only: the PC arena state does not block these.
        h.pc().arena_state = 2;
        let out = h.run(cmd, &[]);
        assert!(out.replies.is_empty());
        let w = posted(&out).unwrap();
        assert_eq!((w.op(), w.guild_id, w.requester), (Some(op), GUILD, PC_ID));
        h.services.guild.post_result = Gate::Fail;
        assert_eq!(reason(&h.run(cmd, &[])), Some((reply, 2)));
        h.pc().guild_id = 0;
        assert_eq!(reason(&h.run(cmd, &[])), Some((reply, none)));
    }
}

#[test]
fn edit_notice() {
    let mut h = H::new();
    assert_eq!(reason(&h.run(480, &[1])), Some((481, 1386)));
    assert_eq!(reason(&h.run(480, &s(""))), Some((481, 1386)));
    h.world.guild.actions.remove(&13);
    assert_eq!(reason(&h.run(480, &s("hi"))), Some((481, 2)));
    let mut h = H::new();
    let long = "x".repeat(200);
    let w = posted(&h.run(480, &s(&long))).unwrap().clone();
    assert_eq!(w.op(), Some(11));
    let GuildWorkKind::EditNotice { text } = w.kind else {
        panic!()
    };
    assert_eq!(text.len(), 121);
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(480, &s("hi"))), Some((481, 1389)));
}

#[test]
fn grade_and_delegate() {
    let grade = [vec![3], s("Target"), vec![4]].concat();
    let mut h = H::new();
    assert_eq!(reason(&h.run(491, &[3])), Some((492, 1403)));
    assert_eq!(
        reason(&h.run(491, &[vec![3], s(""), vec![4]].concat())),
        Some((492, 2))
    );
    let w = posted(&h.run(491, &grade)).unwrap().clone();
    assert_eq!(
        w.kind,
        GuildWorkKind::ChangeGrade {
            index: 3,
            grade: 4,
            name: name("Target")
        }
    );
    h.world.guild.actions.remove(&3);
    assert_eq!(reason(&h.run(491, &grade)), Some((492, 2)));
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(491, &grade)), Some((492, 1405)));

    let del = [vec![2], s("Target")].concat();
    let mut h = H::new();
    assert_eq!(reason(&h.run(494, &[])), Some((495, 1409)));
    assert_eq!(posted(&h.run(494, &del)).unwrap().op(), Some(25));
    h.world.guild.config.delegate_alt_op = true;
    assert_eq!(posted(&h.run(494, &del)).unwrap().op(), Some(29));
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(494, &del)), Some((495, 1410)));
}

#[test]
fn leader_vote_failure_cleans_up() {
    for (cmd, reply, op) in [(499, 500, 27), (501, 502, 26)] {
        let mut h = H::new();
        assert_eq!(posted(&h.run(cmd, &[])).unwrap().op(), Some(op));
        h.services.guild.post_result = Gate::Fail;
        let out = h.run(cmd, &[]);
        assert_eq!(reason(&out), Some((reply, 2)));
        assert!(out
            .changes
            .iter()
            .any(|c| matches!(c, StateChange::GuildLeaderVoteCleanup { .. })));
        h.pc().guild_id = 0;
        assert_eq!(reason(&h.run(cmd, &[])), Some((reply, 1410)));
    }
}

#[test]
fn confirm_invitation() {
    let mut h = H::new();
    assert_eq!(reason(&h.run(506, &[])), Some((507, 1440)));
    h.pc().pending_guild_id = 0x4242;
    assert_eq!(reason(&h.run(506, &[])), Some((507, 1439)));
    h.pc().pending_guild_id = OPEN;
    let w = posted(&h.run(506, &[])).unwrap().clone();
    assert_eq!(w.kind, GuildWorkKind::ConfirmInvitation { db_id: DB_ID });
    assert_eq!(h.pc().pending_guild_id, OPEN);
    h.services.guild.post_result = Gate::Fail;
    let out = h.run(506, &[]);
    assert!(out.replies.is_empty());
    assert_eq!(h.pc().pending_guild_id, 0);
}

#[test]
fn reject_invitation_never_replies() {
    let mut h = H::new();
    h.world.arena = true;
    h.pc().pending_guild_id = OPEN;
    assert!(h.run(508, &[]).replies.is_empty());
    assert_eq!(h.pc().pending_guild_id, OPEN);
    h.world.arena = false;
    let out = h.run(508, &[]);
    assert!(out.replies.is_empty());
    assert_eq!(posted(&out).unwrap().requester, DB_ID);
    assert_eq!(h.pc().pending_guild_id, 0);
    h.pc().pending_guild_id = 0x4242;
    assert!(h.run(508, &[]).replies.is_empty());
    assert_eq!(h.pc().pending_guild_id, 0);
}

#[test]
fn join_guild() {
    let mut h = H::new();
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(509, &[1])), Some((510, 1446)));
    assert_eq!(reason(&h.run(509, &q(0))), Some((510, 1447)));
    assert_eq!(reason(&h.run(509, &q(0x4242))), Some((510, 1448)));
    assert_eq!(reason(&h.run(509, &q(GUILD))), Some((510, 1449)));
    h.world.guild.config.join_renewal = true;
    h.services.guild.join_ticket = Some(true);
    assert_eq!(reason(&h.run(509, &q(OPEN))), Some((510, 1457)));
    h.services.guild.join_ticket = Some(false);
    let w = posted(&h.run(509, &q(OPEN))).unwrap().clone();
    assert_eq!((w.op(), w.guild_id), (None, OPEN));
    h.world.guild.actions.remove(&1);
    assert_eq!(reason(&h.run(509, &q(OPEN))), Some((510, 2)));
    h.pc().guild_id = GUILD;
    assert_eq!(reason(&h.run(509, &q(OPEN))), Some((510, 1451)));
    h.pc().arena_state = 5;
    assert_eq!(reason(&h.run(509, &q(OPEN))), Some((510, 5)));
}

#[test]
fn leave_and_banish() {
    let mut h = H::new();
    assert!(h.run(511, &[]).replies.is_empty());
    h.services.guild.leave_result = Some(Err(77));
    assert_eq!(reason(&h.run(511, &[])), Some((512, 2)));
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(511, &[])), Some((512, 1459)));

    let ban = [vec![1], s("Target")].concat();
    let mut h = H::new();
    assert_eq!(reason(&h.run(513, &[])), Some((514, 1420)));
    assert_eq!(
        reason(&h.run(513, &[vec![0x80], s("T")].concat())),
        Some((514, 1420))
    );
    assert!(h.run(513, &ban).replies.is_empty());
    h.services.guild.banish_result = Some(Err(1424));
    assert_eq!(reason(&h.run(513, &ban)), Some((514, 1424)));
    h.world.guild.actions.remove(&12);
    assert_eq!(reason(&h.run(513, &ban)), Some((514, 2)));
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(513, &ban)), Some((514, 1422)));
}

#[test]
fn query_guild_info_limits() {
    let mut h = H::new();
    assert_eq!(reason(&h.run(515, &[])), Some((517, 2)));
    let mut b = 51u16.to_le_bytes().to_vec();
    assert_eq!(reason(&h.run(515, &b)), Some((517, 2)));
    b = 1u16.to_le_bytes().to_vec();
    b.extend(q(0));
    assert_eq!(reason(&h.run(515, &b)), Some((517, 2)));
    b = 1u16.to_le_bytes().to_vec();
    b.extend(q(GUILD));
    assert!(matches!(h.run(515, &b).status, Status::Unsupported(_)));
}

#[test]
fn unknown_boundaries_are_not_approval() {
    let mut h = H::new();
    h.services.guild.post_result = Gate::Unknown;
    let out = h.run(440, &[]);
    assert!(matches!(out.status, Status::Unsupported(_)));
    assert!(out.replies.is_empty());
    let mut h = H::new();
    h.pc().guild_id = 0;
    h.services.guild.user_found = None;
    assert!(matches!(
        h.run(431, &create_body(1, "New", 7)).status,
        Status::Unsupported(_)
    ));
}

#[test]
fn level_up() {
    let mut h = H::new();
    assert_eq!(posted(&h.run(443, &[])).unwrap().op(), Some(6));
    h.services.guild.post_result = Gate::Fail;
    assert_eq!(reason(&h.run(443, &[])), Some((444, 2)));
    h.world.guild.actions.remove(&5);
    assert_eq!(reason(&h.run(443, &[])), Some((444, 2)));
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(443, &[])), Some((444, 1554)));
    h.world.guild.config.level_up_disabled = true;
    assert_eq!(reason(&h.run(443, &[])), Some((444, 2)));
    h.pc().arena_state = 2;
    assert_eq!(reason(&h.run(443, &[])), Some((444, 5)));
}

fn deposit(money: i32, items: &[(u16, u16)]) -> Vec<u8> {
    let mut b = money.to_le_bytes().to_vec();
    b.push(items.len() as u8);
    for (s, a) in items {
        b.extend(s.to_le_bytes());
        b.extend(a.to_le_bytes());
    }
    b
}

#[test]
fn deposit_in_guild_bank() {
    let mut h = H::new();
    let out = h.run(445, &deposit(500, &[]));
    assert!(out.replies.is_empty(), "{out:?}");
    assert!(matches!(
        h.services.guild.tasks.last().unwrap().kind,
        GuildTaskKind::Deposit { money: 500, .. }
    ));
    let out = h.run(445, &deposit(0, &[(3, 1), (4, 2)]));
    assert!(out.replies.is_empty(), "{out:?}");
    assert_eq!(reason(&h.run(445, &[1])), Some((446, 1553)));
    assert_eq!(reason(&h.run(445, &deposit(1001, &[]))), Some((446, 33)));
    assert_eq!(reason(&h.run(445, &deposit(-1, &[]))), Some((446, 33)));
    assert_eq!(reason(&h.run(445, &deposit(0, &[]))), Some((446, 1553)));
    let six: Vec<_> = (1..=6).map(|i| (i, 1)).collect();
    assert_eq!(reason(&h.run(445, &deposit(0, &six))), Some((446, 1553)));
    assert_eq!(
        reason(&h.run(445, &deposit(5, &[(3, 1)]))),
        Some((446, 1579))
    );
    assert_eq!(
        reason(&h.run(445, &deposit(0, &[(3, 1), (3, 2)]))),
        Some((446, 1553))
    );
    assert_eq!(reason(&h.run(445, &deposit(0, &[(3, 0)]))), Some((446, 2)));
    h.pc().zone = None;
    assert_eq!(reason(&h.run(445, &deposit(5, &[]))), Some((446, 2)));
    h.pc().zone = Some(9);
    h.world.guild.actions.remove(&6);
    assert_eq!(reason(&h.run(445, &deposit(5, &[]))), Some((446, 2)));
    h.pc().t2_blocked = true;
    assert_eq!(reason(&h.run(445, &deposit(5, &[]))), Some((446, 7)));
    h.pc().arena_state = 3;
    assert_eq!(reason(&h.run(445, &deposit(5, &[]))), Some((446, 5)));
    h.world.guild.config.bank_enabled = false;
    assert_eq!(reason(&h.run(445, &deposit(5, &[]))), Some((446, 2)));
}

#[test]
fn member_permissions() {
    let mut h = H::new();
    assert_eq!(posted(&h.run(483, &[])).unwrap().op(), Some(13));
    h.pc().guild_id = 0x4242;
    assert_eq!(reason(&h.run(483, &[])), Some((485, 2)));
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(483, &[])), Some((485, 1384)));

    let masks = |m: [u32; 4]| m.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>();
    let mut h = H::new();
    assert_eq!(reason(&h.run(486, &[1, 0, 0, 0])), Some((488, 1395)));
    let w = posted(&h.run(486, &masks([1, 2, 0xFF, 0])))
        .unwrap()
        .clone();
    assert_eq!(
        w.kind,
        GuildWorkKind::EditPermission {
            masks: [1, 2, 0xFF, 0]
        }
    );
    assert_eq!(
        reason(&h.run(486, &masks([0x100, 0, 0, 0]))),
        Some((488, 1396))
    );
    // Open guild uses the second limit set.
    h.pc().guild_id = OPEN;
    assert_eq!(
        reason(&h.run(486, &masks([0x10, 0, 0, 0]))),
        Some((488, 1396))
    );
    h.world.guild.actions.remove(&2);
    assert_eq!(reason(&h.run(486, &masks([1, 0, 0, 0]))), Some((488, 2)));
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(486, &masks([1, 0, 0, 0]))), Some((488, 1398)));
}

#[test]
fn change_flag_bearer() {
    let body = [vec![1], s("Target")].concat();
    let mut h = H::new();
    assert!(h.run(489, &body).replies.is_empty());
    h.services.guild.flag_bearer_result = Some(1555);
    assert_eq!(reason(&h.run(489, &body)), Some((490, 1555)));
    h.world.guild.actions.remove(&4);
    assert_eq!(reason(&h.run(489, &body)), Some((490, 2)));
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(489, &body)), Some((490, 1554)));
    assert_eq!(reason(&h.run(489, &[1])), Some((490, 1553)));
    h.pc().t2_blocked = true;
    assert_eq!(reason(&h.run(489, &body)), Some((490, 7)));
    h.world.arena = true;
    assert_eq!(reason(&h.run(489, &body)), Some((490, 5)));
}

fn recs(items: &[(u16, u16)]) -> Vec<u8> {
    let mut b = (items.len() as u16).to_le_bytes().to_vec();
    for (s, a) in items {
        b.extend(s.to_le_bytes());
        b.extend(a.to_le_bytes());
    }
    b
}
fn last_task(h: &H) -> &GuildTaskKind {
    &h.services.guild.tasks.last().unwrap().kind
}

#[test]
fn contract_guild_craft() {
    let body = |a: &[(u16, u16)], b: &[(u16, u16)]| {
        [
            5u32.to_le_bytes().to_vec(),
            q(9),
            s("Item"),
            recs(a),
            recs(b),
        ]
        .concat()
    };
    let mut h = H::new();
    assert!(h.run(447, &body(&[(1, 1)], &[(2, 1)])).replies.is_empty());
    let c = h.services.guild.contracts.last().unwrap().clone();
    assert_eq!((c.amount, c.value, c.list_a.len()), (5, 9, 1));
    assert_eq!(reason(&h.run(447, &[1, 0])), Some((448, 1553)));
    let six: Vec<_> = (1..=6).map(|i| (i, 1)).collect();
    assert_eq!(reason(&h.run(447, &body(&six, &[]))), Some((448, 1553)));
    assert_eq!(
        reason(&h.run(447, &body(&[(1, 1), (1, 2)], &[]))),
        Some((448, 1553))
    );
    assert_eq!(
        reason(&h.run(447, &body(&[(1, 1)], &[(1, 2)]))),
        Some((448, 1553))
    );
    assert_eq!(
        reason(&h.run(447, &body(&[], &[(3, 1), (3, 1)]))),
        Some((448, 1553))
    );
    h.services.guild.craft_contract_result = Gate::Fail;
    assert_eq!(reason(&h.run(447, &body(&[], &[]))), Some((448, 1553)));
    h.pc().zone = None;
    assert_eq!(reason(&h.run(447, &body(&[], &[]))), Some((448, 1553)));
    h.world.guild.actions.remove(&7);
    assert_eq!(reason(&h.run(447, &body(&[], &[]))), Some((448, 2)));
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(447, &body(&[], &[]))), Some((448, 2)));
    h.pc().user_present = false;
    assert_eq!(reason(&h.run(447, &body(&[], &[]))), Some((448, 1)));
    h.pc().t2_blocked = true;
    assert_eq!(reason(&h.run(447, &[])), Some((448, 7)));
    h.pc().arena_state = 2;
    assert_eq!(reason(&h.run(447, &[])), Some((448, 5)));
}

#[test]
fn assent_and_take_craft() {
    let mut h = H::new();
    assert!(h.run(449, &[q(4), vec![1]].concat()).replies.is_empty());
    assert_eq!(last_task(&h), &GuildTaskKind::CraftAssent { value: 4 });
    assert_eq!(reason(&h.run(449, &q(4))), Some((450, 1553)));
    h.pc().zone = None;
    assert_eq!(
        reason(&h.run(449, &[q(4), vec![1]].concat())),
        Some((450, 1553))
    );
    h.pc().user_present = false;
    assert_eq!(reason(&h.run(449, &[])), Some((450, 1)));

    let take = [q(8), vec![0, 0]].concat();
    let mut h = H::new();
    h.pc().user_present = false;
    assert!(h.run(457, &take).replies.is_empty());
    assert_eq!(last_task(&h), &GuildTaskKind::CraftTake { value: 8 });
    assert_eq!(reason(&h.run(457, &q(8))), Some((458, 1553)));
    h.pc().native_kind = 2;
    assert_eq!(reason(&h.run(457, &take)), Some((458, 2)));
    h.pc().zone = None;
    assert_eq!(reason(&h.run(457, &take)), Some((458, 2)));
}

#[test]
fn donate_craft_materials() {
    let normal = |v: u64, it: &[(u16, u16)]| [q(v), recs(it)].concat();
    let mut h = H::new();
    assert!(h.run(451, &normal(3, &[(1, 2)])).replies.is_empty());
    assert!(matches!(
        last_task(&h),
        GuildTaskKind::CraftDonateNormal { value: 3, .. }
    ));
    assert_eq!(reason(&h.run(451, &[1])), Some((452, 1553)));
    assert_eq!(
        reason(&h.run(451, &[q(3), vec![1]].concat())),
        Some((452, 1553))
    );
    assert_eq!(
        reason(&h.run(451, &normal(3, &[(1, 2), (1, 3)]))),
        Some((452, 1592))
    );
    assert_eq!(reason(&h.run(451, &normal(0, &[(1, 2)]))), Some((452, 2)));
    assert_eq!(reason(&h.run(451, &normal(3, &[]))), Some((452, 2)));
    assert_eq!(reason(&h.run(451, &normal(3, &[(1, 0)]))), Some((452, 2)));
    h.pc().user_present = false;
    assert_eq!(reason(&h.run(451, &normal(3, &[(1, 2)]))), Some((452, 1)));

    let brand = [q(3), 7u32.to_le_bytes().to_vec(), vec![1], recs(&[(1, 2)])].concat();
    let mut h = H::new();
    assert!(h.run(453, &brand).replies.is_empty());
    assert!(matches!(
        last_task(&h),
        GuildTaskKind::CraftDonateBrand {
            value: 3,
            extra: 7,
            flag: 1,
            ..
        }
    ));
    assert_eq!(reason(&h.run(453, &q(3))), Some((454, 1553)));
}

#[test]
fn make_and_abandon_craft() {
    let mut h = H::new();
    assert!(h.run(455, &q(1)).replies.is_empty());
    h.services.guild.craft_make_result = Some(1600);
    assert_eq!(reason(&h.run(455, &q(1))), Some((456, 1600)));
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(455, &q(1))), Some((456, 1554)));
    h.pc().user_present = false;
    assert_eq!(reason(&h.run(455, &q(1))), Some((456, 2)));
    assert_eq!(reason(&h.run(455, &[])), Some((456, 1553)));

    let mut h = H::new();
    assert!(h.run(459, &q(1)).replies.is_empty());
    h.services.guild.craft_abandon_result = Some(1601);
    assert_eq!(reason(&h.run(459, &q(1))), Some((460, 1601)));
    h.world.guild.guilds.get_mut(&GUILD).unwrap().craft_active = false;
    assert_eq!(reason(&h.run(459, &q(1))), Some((460, 1555)));
    h.world.guild.guilds.get_mut(&GUILD).unwrap().leader_db_id = 1;
    assert_eq!(reason(&h.run(459, &q(1))), Some((460, 1597)));
    h.pc().guild_id = 0x4242;
    assert_eq!(reason(&h.run(459, &q(1))), Some((460, 2)));
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(459, &q(1))), Some((460, 1554)));
    assert_eq!(reason(&h.run(459, &[])), Some((460, 1553)));
}

#[test]
fn contract_guild_uniform() {
    let list = |p: &[u8], r: &[u32]| {
        let mut b = (p.len() as u16).to_le_bytes().to_vec();
        b.extend(p);
        b.extend((r.len() as u16).to_le_bytes());
        for x in r {
            b.extend(x.to_le_bytes());
        }
        b
    };
    let body = |p: &[u8], r: &[u32]| [q(6), s("Robe"), list(p, r)].concat();
    let mut h = H::new();
    assert!(h.run(461, &body(&[1, 2], &[77])).replies.is_empty());
    let c = h.services.guild.uniform_contracts.last().unwrap().clone();
    assert_eq!((c.value, c.params, c.recipes), (6, vec![1, 2], vec![77]));
    h.services.guild.uniform_contract_result = Some(1610);
    assert_eq!(reason(&h.run(461, &body(&[], &[]))), Some((462, 1610)));
    h.world.guild.actions.remove(&8);
    assert_eq!(reason(&h.run(461, &body(&[], &[]))), Some((462, 2)));
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(461, &body(&[], &[]))), Some((462, 1554)));
    h.pc().uniform_faction_blocked = true;
    assert_eq!(reason(&h.run(461, &body(&[], &[]))), Some((462, 1600)));
    h.services.guild.uniform_name_ok = Some(false);
    assert_eq!(reason(&h.run(461, &body(&[], &[]))), Some((462, 1602)));
    assert_eq!(reason(&h.run(461, &body(&[0], &[]))), Some((462, 1553)));
    assert_eq!(reason(&h.run(461, &body(&[1; 29], &[]))), Some((462, 1553)));
    assert_eq!(reason(&h.run(461, &q(6))), Some((462, 1553)));
    h.pc().user_present = false;
    assert_eq!(reason(&h.run(461, &[])), Some((462, 1)));
    h.pc().t2_blocked = true;
    assert_eq!(reason(&h.run(461, &[])), Some((462, 7)));
    h.pc().arena_state = 2;
    assert_eq!(reason(&h.run(461, &[])), Some((462, 5)));
    h.world.guild.config.uniform_enabled = false;
    assert_eq!(reason(&h.run(461, &[])), Some((462, 1568)));
}

#[test]
fn donate_uniform_and_quota() {
    let donate = |v: u64, it: &[(u16, u16)]| [q(v), recs(it)].concat();
    let mut h = H::new();
    h.pc().user_present = false;
    assert!(h.run(463, &donate(5, &[(1, 2)])).replies.is_empty());
    assert_eq!(
        last_task(&h),
        &GuildTaskKind::UniformDonate {
            value: 5,
            items: vec![(1, 2)]
        }
    );
    assert_eq!(
        reason(&h.run(463, &donate(5, &[(1, 0xFFFF)]))),
        Some((464, 1553))
    );
    assert_eq!(
        reason(&h.run(463, &donate(5, &[(0, 1)]))),
        Some((464, 1553))
    );
    assert_eq!(
        reason(&h.run(463, &donate(5, &[(1, 1), (1, 2)]))),
        Some((464, 1592))
    );
    assert_eq!(reason(&h.run(463, &donate(0, &[(1, 1)]))), Some((464, 2)));
    assert_eq!(reason(&h.run(463, &donate(5, &[]))), Some((464, 2)));
    h.world.guild.config.uniform_enabled = false;
    assert_eq!(reason(&h.run(463, &[])), Some((464, 1568)));

    let quota = |it: &[(u16, u16)]| [vec![3], recs(it)].concat();
    let mut h = H::new();
    assert!(h.run(475, &quota(&[(4, 1)])).replies.is_empty());
    assert_eq!(
        last_task(&h),
        &GuildTaskKind::UniformQuota {
            flag: 3,
            items: vec![(4, 1)]
        }
    );
    assert_eq!(
        reason(&h.run(475, &quota(&[(4, 1), (4, 1)]))),
        Some((476, 1553))
    );
    assert_eq!(reason(&h.run(475, &quota(&[]))), Some((476, 2)));
    assert_eq!(reason(&h.run(475, &quota(&[(4, 0)]))), Some((476, 2)));
    h.pc().zone = None;
    assert_eq!(reason(&h.run(475, &quota(&[(4, 1)]))), Some((476, 2)));
    h.pc().user_present = false;
    assert_eq!(reason(&h.run(475, &[])), Some((476, 1)));
}

#[test]
fn uniform_helper_handlers() {
    for cmd in [465u16, 467, 469, 471] {
        let mut h = H::new();
        assert!(h.run(cmd, &q(9)).replies.is_empty());
        assert_eq!(h.services.guild.uniform_ops.last(), Some(&(cmd, 9, 0)));
        h.services.guild.uniform_op_result = Some(1620);
        assert_eq!(reason(&h.run(cmd, &q(9))), Some((cmd + 1, 1620)));
        h.pc().guild_id = 0x4242;
        assert_eq!(reason(&h.run(cmd, &q(9))), Some((cmd + 1, 2)));
        h.pc().guild_id = 0;
        assert_eq!(reason(&h.run(cmd, &q(9))), Some((cmd + 1, 1554)));
        assert_eq!(reason(&h.run(cmd, &[])), Some((cmd + 1, 1553)));
        h.world.guild.config.uniform_enabled = false;
        assert_eq!(reason(&h.run(cmd, &[])), Some((cmd + 1, 1568)));
    }
    let mut h = H::new();
    assert!(h.run(473, &[q(1), q(2)].concat()).replies.is_empty());
    assert_eq!(h.services.guild.uniform_ops.last(), Some(&(473, 1, 2)));
    assert_eq!(reason(&h.run(473, &q(1))), Some((474, 1553)));
}

fn unsupported(out: &Outcome) -> bool {
    matches!(out.status, Status::Unsupported(_))
}
fn ticket(h: &H) -> (u8, u64, u64, u64) {
    match last_task(h) {
        GuildTaskKind::JoinTicket { op, a, b, c } => (*op, *a, *b, *c),
        other => panic!("not a join ticket: {other:?}"),
    }
}
fn jr() -> H {
    let mut h = H::new();
    h.world.guild.config.join_renewal = true;
    h.world.guild.config.join_request_cooldown_secs = 10;
    h.world.tick = 100_000;
    h.world.guild.actions.insert(
        18,
        GuildActionRow {
            normal: true,
            open: false,
        },
    );
    h
}

#[test]
fn join_ticket_lists() {
    let mut h = jr();
    h.services.guild.permission = Some(false);
    assert_eq!(reason(&h.run(557, &q(0))), Some((559, 2)));
    h.services.guild.permission = Some(true);
    assert!(unsupported(&h.run(557, &q(0))));
    assert_eq!(reason(&h.run(557, &[])), Some((559, 2)));
    h.pc().guild_id = OPEN; // row 18 off for open guilds
    assert_eq!(reason(&h.run(557, &q(0))), Some((559, 2)));
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(557, &q(0))), Some((559, 2)));
    assert!(unsupported(&h.run(560, &[])));
    h.pc().guild_id = GUILD;
    assert_eq!(reason(&h.run(560, &[])), Some((562, 2)));
    h.world.arena = true;
    assert_eq!(reason(&h.run(557, &q(0))), Some((559, 5)));
    assert_eq!(reason(&h.run(560, &[])), Some((562, 5)));
    h.world.guild.config.join_renewal = false;
    assert_eq!(reason(&h.run(557, &q(0))), Some((559, 2)));
    assert_eq!(reason(&h.run(560, &[])), Some((562, 2)));
}

#[test]
fn send_guild_application() {
    let mut h = jr();
    h.pc().guild_id = 0;
    let body = s("[Old]Guild");
    assert!(h.run(563, &body).replies.is_empty());
    assert_eq!(ticket(&h), (1, DB_ID, DB_ID, GUILD));
    assert_eq!(h.pc().join_request_tick, 100_000);
    assert_eq!(reason(&h.run(563, &body)), Some((565, 1624)));
    h.world.tick = 110_000;
    h.pc().faction = 2;
    assert_eq!(reason(&h.run(563, &body)), Some((565, 1625)));
    h.pc().faction = 1;
    assert_eq!(reason(&h.run(563, &s("Open"))), Some((565, 1623)));
    assert_eq!(reason(&h.run(563, &s("Nobody"))), Some((565, 2)));
    assert_eq!(reason(&h.run(563, &s(""))), Some((565, 2)));
    h.services.guild.queue_result = Gate::Fail;
    assert_eq!(reason(&h.run(563, &body)), Some((565, 2)));
    assert_eq!(h.pc().join_request_tick, 100_000);
    h.pc().guild_id = GUILD;
    assert_eq!(reason(&h.run(563, &body)), Some((565, 1626)));
}

#[test]
fn application_and_invitation_tickets() {
    // (command, reply, op, needs guild)
    for (cmd, rep, op) in [
        (571u16, 573u16, 3u8),
        (575, 577, 4),
        (579, 581, 5),
        (584, 586, 6),
    ] {
        let mut h = jr();
        assert!(h.run(cmd, &q(0x99)).replies.is_empty());
        assert_eq!(ticket(&h), (op, DB_ID, 0x99, GUILD));
        h.world.tick += 20_000;
        assert_eq!(reason(&h.run(cmd, &q(0))), Some((rep, 2)));
        h.pc().guild_id = 0;
        assert_eq!(reason(&h.run(cmd, &q(0x99))), Some((rep, 2)));
    }
    let mut h = jr();
    assert!(h.run(579, &q(0x99)).replies.is_empty());
    assert_eq!(reason(&h.run(579, &q(0x99))), Some((581, 1644)));
    assert_eq!(reason(&h.run(583, &[])), Some((581, 1644)));
    h.world.tick += 20_000;
    assert_eq!(
        reason(&h.run(583, &[s("x"), q(1)].concat())),
        Some((581, 2))
    );
    for (cmd, rep, op, guildless) in [
        (567u16, 569u16, 2u8, true),
        (588, 590, 7, false),
        (592, 594, 8, true),
    ] {
        let mut h = jr();
        h.pc().guild_id = 0;
        assert!(h.run(cmd, &q(GUILD)).replies.is_empty());
        assert_eq!(ticket(&h), (op, DB_ID, DB_ID, GUILD));
        assert_eq!(reason(&h.run(cmd, &q(OPEN))), Some((rep, 2)));
        assert_eq!(reason(&h.run(cmd, &q(0))), Some((rep, 2)));
        h.pc().guild_id = GUILD;
        let out = h.run(cmd, &q(GUILD));
        assert_eq!(out.replies.is_empty(), !guildless);
        h.pc().zone = None;
        h.pc().guild_id = 0;
        assert_eq!(reason(&h.run(cmd, &q(GUILD))), Some((rep, 2)));
        h.world.arena = true;
        assert_eq!(reason(&h.run(cmd, &q(GUILD))), Some((rep, 5)));
    }
}

#[test]
fn refusal_flags() {
    let mut h = jr();
    h.pc().native_kind = 1;
    h.pc().name = name("Me");
    assert!(h.run(596, &q(1)).replies.is_empty());
    assert_eq!(
        last_task(&h),
        &GuildTaskKind::ApplicationRefusal {
            db_id: DB_ID,
            name: name("Me"),
            value: 1
        }
    );
    assert_eq!(reason(&h.run(596, &q(0))), Some((598, 2)));
    assert_eq!(reason(&h.run(596, &[])), Some((598, 2)));
    h.pc().native_kind = 0;
    assert_eq!(reason(&h.run(596, &q(1))), Some((598, 2)));
    h.pc().native_kind = 1;
    assert!(h.run(600, &q(1)).replies.is_empty());
    assert_eq!(
        last_task(&h),
        &GuildTaskKind::InvitationRefusal {
            db_id: DB_ID,
            value: 1
        }
    );
    h.pc().guild_invitation_refusal = true;
    assert_eq!(reason(&h.run(600, &q(1))), Some((602, 2)));
    assert!(h.run(600, &q(0)).replies.is_empty());
    h.world.arena = true;
    assert_eq!(reason(&h.run(596, &q(1))), Some((598, 5)));
    assert_eq!(reason(&h.run(600, &q(1))), Some((602, 5)));
}

#[test]
fn send_guild_invitation2() {
    let mut h = jr();
    h.world.guild.players.insert(
        0x42,
        GuildPlayer {
            name: name("Bob"),
            faction: 1,
            db_id: 0x8800,
        },
    );
    let body = [s("Bob"), q(0x42)].concat();
    assert!(h.run(583, &body).replies.is_empty());
    assert_eq!(ticket(&h), (5, DB_ID, 0x8800, GUILD));
    assert_eq!(h.pc().join_request_tick, 100_000);
    assert_eq!(reason(&h.run(583, &body)), Some((581, 1644)));
    h.world.tick += 20_000;
    assert_eq!(
        reason(&h.run(583, &[s("Ann"), q(0x42)].concat())),
        Some((581, 2))
    );
    assert_eq!(
        reason(&h.run(583, &[s("Bob"), q(0x43)].concat())),
        Some((581, 2))
    );
    assert_eq!(
        reason(&h.run(583, &[s("Bob"), q(0)].concat())),
        Some((581, 2))
    );
    assert_eq!(reason(&h.run(583, &s("Bob"))), Some((581, 2)));
    // Faction mismatch: 1645 first, then the handler continues.
    h.world.guild.players.get_mut(&0x42).unwrap().faction = 2;
    let out = h.run(583, &body);
    assert_eq!(out.replies.len(), 1);
    assert_eq!(reason(&out), Some((581, 1645)));
    assert!(out
        .changes
        .iter()
        .any(|c| matches!(c, StateChange::GuildTaskQueued(_))));
    h.world.tick += 20_000;
    h.pc().guild_id = OPEN; // row 18 off: 1645 then 2
    let out = h.run(583, &body);
    assert_eq!(out.replies.len(), 2);
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(583, &body)), Some((581, 2)));
    h.world.arena = true;
    assert_eq!(reason(&h.run(583, &body)), Some((581, 5)));
    h.world.guild.config.join_renewal = false;
    assert_eq!(reason(&h.run(583, &body)), Some((581, 2)));
}

#[test]
fn query_guild_info_to_gamed() {
    let mut h = H::new();
    assert!(unsupported(
        &h.run(518, &[[2u8].to_vec(), q(1), q(2)].concat())
    ));
    assert_eq!(
        reason(&h.run(518, &[[2u8].to_vec(), q(1), q(0)].concat())),
        Some((520, 2))
    );
    assert_eq!(
        reason(&h.run(518, &[[1u8].to_vec()].concat())),
        Some((520, 2))
    );
    assert_eq!(reason(&h.run(518, &[51u8])), Some((520, 2)));
    assert_eq!(reason(&h.run(518, &[])), Some((520, 2)));
    h.world.arena = true;
    assert_eq!(reason(&h.run(518, &[0u8])), Some((520, 5)));
}

fn gp() -> H {
    let mut h = H::new();
    h.pc().native_kind = 1;
    h.world.guild.config.level_up_disabled = true;
    h.world.guild.config.guild_point_cooldown_secs = 5;
    h.world.guild.config.benefit_enabled = true;
    h.world.tick = 50_000;
    h.world.guild.actions.insert(
        19,
        GuildActionRow {
            normal: true,
            open: false,
        },
    );
    h
}

#[test]
fn accumulate_guild_point() {
    let mut h = gp();
    let pts = 30u32.to_le_bytes();
    assert!(h.run(603, &pts).replies.is_empty());
    assert_eq!(
        last_task(&h),
        &GuildTaskKind::AccumulatePoint {
            db_id: DB_ID,
            points: 30
        }
    );
    assert_eq!(h.pc().guild_point_tick, 50_000);
    let out = h.run(603, &pts);
    assert_eq!(reason(&out), Some((605, 1658)));
    assert_eq!(out.replies[0].body.len(), 10);
    h.world.tick = 60_000;
    assert_eq!(reason(&h.run(603, &0u32.to_le_bytes())), Some((605, 2)));
    assert_eq!(reason(&h.run(603, &[1])), Some((605, 2)));
    h.pc().guild_id = 0x9999;
    assert_eq!(reason(&h.run(603, &pts)), Some((605, 1659)));
    h.pc().guild_id = OPEN;
    assert_eq!(reason(&h.run(603, &pts)), Some((605, 2)));
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(603, &pts)), Some((605, 2)));
    h.pc().guild_id = GUILD;
    h.services.guild.queue_result = Gate::Fail;
    assert_eq!(reason(&h.run(603, &pts)), Some((605, 2)));
    assert_eq!(h.pc().guild_point_tick, 50_000);
    h.world.arena = true;
    assert_eq!(reason(&h.run(603, &pts)), Some((605, 5)));
    h.world.guild.config.level_up_disabled = false;
    assert_eq!(reason(&h.run(603, &pts)), Some((605, 2)));
}

#[test]
fn level_up_and_benefit() {
    let mut h = gp();
    assert!(h.run(606, &[]).replies.is_empty());
    assert_eq!(
        last_task(&h),
        &GuildTaskKind::LevelUpWithPoint { db_id: DB_ID }
    );
    let body = |b: i16, k: i16, d: i32| {
        [
            b.to_le_bytes().to_vec(),
            k.to_le_bytes().to_vec(),
            d.to_le_bytes().to_vec(),
        ]
        .concat()
    };
    assert!(h.run(610, &body(7, 2, 3600)).replies.is_empty());
    assert_eq!(
        last_task(&h),
        &GuildTaskKind::PurchaseBenefit {
            db_id: DB_ID,
            benefit: 7,
            kind: 2,
            duration: 3600
        }
    );
    for bad in [
        body(0, 1, 1),
        body(7, 3, 1),
        body(7, 0, 1),
        body(7, 1, -1),
        vec![1, 0],
    ] {
        assert_eq!(reason(&h.run(610, &bad)), Some((612, 2)));
    }
    h.pc().native_kind = 0;
    assert_eq!(reason(&h.run(606, &[])), Some((608, 2)));
    assert_eq!(reason(&h.run(610, &body(7, 1, 1))), Some((612, 2)));
    h.pc().native_kind = 1;
    h.pc().zone = None;
    assert_eq!(reason(&h.run(606, &[])), Some((608, 2)));
    h.world.arena = true;
    assert_eq!(reason(&h.run(610, &body(7, 1, 1))), Some((612, 5)));
    h.world.guild.config.benefit_enabled = false;
    assert_eq!(reason(&h.run(610, &body(7, 1, 1))), Some((612, 2)));
    h.world.guild.config.level_up_disabled = false;
    assert_eq!(reason(&h.run(606, &[])), Some((608, 2)));
}

#[test]
fn guild_alliance() {
    let mut h = H::new();
    h.world.guild.config.alliance_enabled = true;
    let out = h.run(615, &s("Open"));
    assert!(out.replies.is_empty());
    assert_eq!(
        posted(&out).map(|w| w.kind.clone()),
        Some(GuildWorkKind::Alliance {
            op: 0,
            first_id: GUILD,
            first_name: name("[Old]Guild"),
            second_id: OPEN,
            second_name: name("Open"),
            reason: 0,
        })
    );
    assert_eq!(reason(&h.run(615, &s("Nobody"))), Some((617, 1668)));
    assert_eq!(reason(&h.run(615, &s("[Old]Guild"))), Some((617, 1669)));
    assert_eq!(reason(&h.run(615, &s(""))), Some((617, 2)));
    let out = h.run(619, &q(OPEN));
    match &posted(&out).unwrap().kind {
        GuildWorkKind::Alliance {
            op,
            first_id,
            second_id,
            ..
        } => {
            assert_eq!((*op, *first_id, *second_id), (1, OPEN, GUILD));
        }
        other => panic!("{other:?}"),
    }
    let out = h.run(623, &q(OPEN));
    match &posted(&out).unwrap().kind {
        GuildWorkKind::Alliance {
            op,
            first_id,
            second_id,
            ..
        } => {
            assert_eq!((*op, *first_id, *second_id), (3, GUILD, OPEN));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(reason(&h.run(619, &q(0x9999))), Some((620, 2)));
    assert_eq!(reason(&h.run(623, &[])), Some((625, 2)));
    assert!(unsupported(&h.run(621, &[q(OPEN), q(1)].concat())));
    assert_eq!(reason(&h.run(621, &[])), Some((622, 2)));
    h.services.guild.post_result = Gate::Fail;
    assert_eq!(reason(&h.run(615, &s("Open"))), Some((617, 2)));
    h.pc().guild_id = 0;
    assert_eq!(reason(&h.run(619, &q(OPEN))), Some((620, 2)));
    h.world.arena = true;
    assert_eq!(reason(&h.run(623, &q(OPEN))), Some((625, 5)));
    h.world.guild.config.alliance_enabled = false;
    assert_eq!(reason(&h.run(621, &q(OPEN))), Some((622, 2)));
}
