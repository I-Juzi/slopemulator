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
            },
        );
        for row in [1, 2, 3, 4, 5, 6, 7, 9, 11, 12, 13, 17] {
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
