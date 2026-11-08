use achievementdaemon::config::{DaemonConfig, IpList};
use achievementdaemon::event::{id, Event, FieldValue, ParseError};
use achievementdaemon::pc::{take, Done, Pc, Profile, Slot, CLEAN, INSERT, UPDATE};
use achievementdaemon::protocol::{self, game_cmd, manager_cmd};
use achievementdaemon::rules::{job_from_name, Op, Rules};
use achievementdaemon::schema::EVENTS;
use achievementdaemon::service::{Daemon, GameConn, ManagerConn, Out};
use achievementdaemon::store::{MemoryStore, PcRows, Store, StoreError};
use bloodlib::packet::Value;
use std::net::Ipv4Addr;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

const REGISTERS: &str = r#"<table type="achievement-register" version="0.42">
  <record alias="1_kill" id="1" job="job-none" max-value="10" persistant="y" progress-show="y" type="kill-npc"/>
  <record alias="2_temp" id="2" job="job-none" max-value="0" type="kill-npc"/>
  <record alias="3_bits" id="3" job="job-none" max-value="0" persistant="y" type="kill-npc"/>
  <record alias="4_level" id="4" job="job-none" max-value="0" persistant="y" type="pc-level"/>
  <record alias="5_client" id="5" job="job-none" max-value="0" persistant="y" type="client-only"/>
  <record alias="6_blade" id="6" job="검사" max-value="0" persistant="y" type="kill-npc"/>
  <record alias="7_inv" id="7" job="job-none" max-value="0" type="inventory-size"/>
</table>"#;

const ACHIEVEMENTS: &str = r#"<table type="achievement" version="0.19">
  <record alias="a1s1" current-step-score="10" id="100" job="job-none" register-ref-1="1_kill" register-type-1="above" register-value-1="2" step="1" step-complete-reward-item-1="x"/>
  <record alias="a1s2" current-step-score="20" id="100" job="job-none" register-ref-1="1_kill" register-type-1="above" register-value-1="5" step="2" step-complete-reward-item-1="x"/>
  <record alias="bits" current-step-score="5" id="101" job="job-none" register-ref-1="3_bits" register-type-1="bits-on" register-value-1="3" step="1"/>
  <record alias="lvl" current-step-score="1" id="102" job="job-none" register-ref-1="4_level" register-type-1="above" register-value-1="10" step="1"/>
  <record alias="cfc" current-step-score="3" id="103" job="job-none" complete-from-client="y" step="1"/>
  <record alias="dep" current-step-score="3" id="104" job="job-none" deprecated="y" register-ref-1="1_kill" register-type-1="above" register-value-1="1" step="1"/>
  <record alias="two" current-step-score="7" id="105" job="job-none" register-ref-1="1_kill" register-type-1="above" register-value-1="3" register-ref-2="5_client" register-type-2="above" register-value-2="1" step="1"/>
</table>"#;

const IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);
const PC: i64 = 0x1234;
const TID: i64 = 77;

fn daemon() -> Daemon<MemoryStore> {
    let rules = Rules::from_xml(REGISTERS, ACHIEVEMENTS).unwrap();
    let iplist =
        IpList::parse("<cluster-list><cluster ip='10.0.0.1' cluster-id='1'/></cluster-list>")
            .unwrap();
    Daemon::new(DaemonConfig::default(), iplist, rules, MemoryStore::new())
}

fn hello(cluster: i16, pv: i32, ev: i32) -> Vec<u8> {
    protocol::packet(
        "hiih",
        &[
            Value::I16(1),
            Value::I32(pv),
            Value::I32(ev),
            Value::I16(cluster),
        ],
    )
}

fn connected<S: Store>(d: &Daemon<S>) -> GameConn {
    let mut conn = GameConn::new(IP);
    let out = d.game_packet(&mut conn, &hello(1, 10, 33), 0);
    assert_eq!(out, vec![Out::Reply(protocol::hh(game_cmd::HELLO_ACK, 1))]);
    conn
}

fn event(id: u16, values: Vec<u64>) -> Vec<u8> {
    internal_event(id, values).encode()
}

fn internal_event(id: u16, values: Vec<u64>) -> Event {
    let schema = achievementdaemon::schema::schema(id).unwrap();
    let values = schema
        .fields
        .iter()
        .zip(values.into_iter().chain(std::iter::repeat(0)))
        .map(|(&(_, kind), v)| match kind {
            achievementdaemon::event::FieldType::WStr => {
                FieldValue::Str("Pc".encode_utf16().collect())
            }
            _ => FieldValue::Int(v),
        })
        .collect();
    Event {
        time: 5,
        tid: TID,
        pcdbid: PC,
        hint: 9,
        id,
        cluster_id: 1,
        values,
    }
}

/// PcLogin with job, level, inventory.
fn login<S: Store>(d: &Daemon<S>, conn: &mut GameConn, job: u64, level: u64) -> Vec<Out> {
    // worldId level mastery faction faction2 inventory depot wardrobe race sex job pcid ...
    d.game_packet(
        conn,
        &event(
            id::PC_LOGIN,
            vec![1, level, 0, 0, 0, 60, 0, 0, 1, 1, job, 0x99],
        ),
        1000,
    )
}

fn cluster_packets(out: &[Out]) -> Vec<Vec<u8>> {
    out.iter()
        .filter_map(|o| match o {
            Out::Cluster(1, p) => Some(p.clone()),
            _ => None,
        })
        .collect()
}

fn cmd(p: &[u8]) -> i16 {
    i16::from_le_bytes([p[2], p[3]])
}

#[test]
fn schema_covers_all_event_ids() {
    assert_eq!(EVENTS.len(), 55);
    assert!(EVENTS.windows(2).all(|w| w[0].id < w[1].id));
    assert!(EVENTS.iter().all(|e| e.id <= 0x770));
    let login = achievementdaemon::schema::schema(1801).unwrap();
    assert_eq!(login.fields.len(), 18);
    assert_eq!(login.fields[17].0, "pcname");
}

#[test]
fn event_parse_roundtrip_and_errors() {
    let p = event(id::COMPLETE_TAKE_REWARD, vec![42, 100, 1, 0]);
    let ev = Event::parse(&p).unwrap();
    assert_eq!((ev.tid, ev.pcdbid, ev.hint, ev.cluster_id), (TID, PC, 9, 1));
    assert_eq!(ev.int("transaction_id"), 42);
    assert_eq!(ev.int("id"), 100);
    assert_eq!(Event::parse(&p[..39]), Err(ParseError::ShortHeader));
    assert_eq!(Event::parse(&p[..p.len() - 1]), Err(ParseError::Decompose));
    let mut bad = p.clone();
    bad[36..38].copy_from_slice(&1018u16.to_le_bytes());
    assert_eq!(Event::parse(&bad), Err(ParseError::UnknownEvent(1018)));
    let login = Event::parse(&event(id::PC_LOGIN, vec![])).unwrap();
    assert_eq!(String::from_utf16(&login.str("pcname")).unwrap(), "Pc");
}

#[test]
fn config_defaults_and_options() {
    let c = DaemonConfig::parse(
        "<config module='achievement-daemon'><option name='internal-port' value='1234'/><option name='update-cycle' value='500'/><option name='ping-enable' value='no'/></config>",
    )
    .unwrap();
    assert_eq!(
        (
            c.internal_port,
            c.manage_port,
            c.update_cycle_ms,
            c.ping_enable
        ),
        (1234, 32200, 500, false)
    );
    assert!(DaemonConfig::parse("<config module='x'/>").is_err());
    assert!(
        DaemonConfig::parse("<config><option name='update-cycle' value='0'/></config>").is_err()
    );
    let ips =
        IpList::parse("<cluster-list><cluster ip='1.2.3.4' cluster-id='10001'/></cluster-list>")
            .unwrap();
    assert!(ips.allows(Ipv4Addr::new(1, 2, 3, 4), 10001));
    assert!(!ips.allows(Ipv4Addr::new(1, 2, 3, 4), 1));
}

#[test]
fn rules_parse_and_ops() {
    let r = Rules::from_xml(REGISTERS, ACHIEVEMENTS).unwrap();
    assert_eq!(r.registers.len(), 7);
    assert_eq!(r.register(6, 1).unwrap().job, 1);
    assert!(r.register(6, 2).is_none());
    assert!(r.register(5, 3).unwrap().increment_from_client);
    assert_eq!(r.linked(1, 0), &[(100, 1, 0), (100, 2, 0), (105, 1, 0)]);
    assert_eq!(r.record(100, 2, 7).unwrap().score, 20);
    assert_eq!(job_from_name("궁사"), Some(12));
    assert!(Op::BitsOn.holds(0b0111, 3));
    assert!(!Op::BitsOn.holds(0b0101, 3));
    assert!(Op::Below.holds(3, 3) && !Op::Below.holds(4, 3));
    assert!(Rules::from_xml(
        REGISTERS,
        r#"<table type="achievement"><record id="1" register-ref-1="nope"/></table>"#
    )
    .is_err());
}

#[test]
fn hello_checks() {
    let d = daemon();
    let mut c = GameConn::new(IP);
    let fail = |r| vec![Out::Reply(protocol::hh(game_cmd::HELLO_FAIL, r))];
    assert_eq!(d.game_packet(&mut c, &hello(1, 9, 33), 0), fail(3));
    assert_eq!(d.game_packet(&mut c, &hello(1, 10, 32), 0), fail(3));
    assert_eq!(d.game_packet(&mut c, &hello(2, 10, 33), 0), fail(4));
    assert_eq!(
        d.game_packet(&mut c, &protocol::packet("h", &[Value::I16(1)]), 0),
        fail(2)
    );
    let mut other = GameConn::new(Ipv4Addr::new(10, 0, 0, 2));
    assert_eq!(d.game_packet(&mut other, &hello(1, 10, 33), 0), fail(4));
    let _ok = connected(&d);
    let mut dup = GameConn::new(IP);
    assert_eq!(d.game_packet(&mut dup, &hello(1, 10, 33), 0), fail(2));
}

#[test]
fn commands_need_a_cluster_and_ping_replies() {
    let d = daemon();
    let mut c = GameConn::new(IP);
    let ping = protocol::packet("hqh", &[Value::I16(4), Value::I64(8), Value::I16(1)]);
    assert!(d.game_packet(&mut c, &ping, 0).is_empty());
    let mut c = connected(&d);
    let out = d.game_packet(&mut c, &ping, 0);
    assert_eq!(
        out,
        vec![Out::Reply(protocol::packet(
            "hqh",
            &[Value::I16(5), Value::I64(8), Value::I16(1)]
        ))]
    );
    assert!(d
        .game_packet(&mut c, &protocol::packet("h", &[Value::I16(13)]), 0)
        .is_empty());
}

#[test]
fn login_sends_all_data_and_sets_profile_registers() {
    let d = daemon();
    let mut c = connected(&d);
    let out = login(&d, &mut c, 0, 12);
    let packets = cluster_packets(&out);
    assert_eq!(cmd(&packets[0]), game_cmd::DATA);
    // Level 12 >= 10 completes achievement 102 -> CompletedNotify.
    assert!(packets.iter().any(|p| cmd(p) == game_cmd::COMPLETED_NOTIFY));
    let pc = d.pc(PC).unwrap();
    assert_eq!(pc.register(4), 12);
    assert_eq!(pc.register(7), 60);
    assert_eq!(pc.completed_step(102), 1);
    assert_eq!(pc.score, 1);
    // Data packet: fragment header (8 bytes) + tid, pcdbid, hint.
    let p = &packets[0];
    assert_eq!(i64::from_le_bytes(p[8..16].try_into().unwrap()), TID);
    assert_eq!(i64::from_le_bytes(p[16..24].try_into().unwrap()), PC);
    assert_eq!(i64::from_le_bytes(p[24..32].try_into().unwrap()), 9);
}

#[test]
fn events_without_pc_or_with_wrong_tid_are_dropped() {
    let d = daemon();
    let mut c = connected(&d);
    let add = event(id::DEBUG_ADD_REGISTER, vec![1, 3]);
    assert!(d.game_packet(&mut c, &add, 0).is_empty());
    login(&d, &mut c, 0, 1);
    let mut wrong = Event::parse(&add).unwrap();
    wrong.tid = TID + 1;
    d.game_packet(&mut c, &wrong.encode(), 0);
    assert_eq!(d.pc(PC).unwrap().register(1), 0);
}

#[test]
fn add_register_clamps_and_completes_steps() {
    let d = daemon();
    let mut c = connected(&d);
    login(&d, &mut c, 0, 1);
    let out = d.game_packet(&mut c, &event(id::DEBUG_ADD_REGISTER, vec![1, 100]), 0);
    let pc = d.pc(PC).unwrap();
    assert_eq!(pc.register(1), 10); // max-value
    assert_eq!(pc.completed_step(100), 2);
    assert_eq!(pc.completed_step(104), 0); // deprecated
    assert_eq!(pc.score, 30);
    let notes: Vec<_> = cluster_packets(&out)
        .into_iter()
        .filter(|p| cmd(p) == 11)
        .collect();
    assert_eq!(notes.len(), 2);
    let v = protocol::fields("hsqqqhhhc", &notes[1]).unwrap();
    assert_eq!(v[1], Value::String("Pc".encode_utf16().collect()));
    assert_eq!(
        (
            protocol::i16_at(&v, 5),
            protocol::i16_at(&v, 6),
            protocol::i16_at(&v, 7)
        ),
        (100, 1, 2)
    );
    assert_eq!(v[8], Value::U8(1)); // last step
    d.game_packet(
        &mut c,
        &event(id::DEBUG_ADD_REGISTER, vec![1, (-50i64) as u64]),
        0,
    );
    assert_eq!(d.pc(PC).unwrap().register(1), 0); // clamp >= 0
}

#[test]
fn job_filter_and_bits_and_two_conditions() {
    let d = daemon();
    let mut c = connected(&d);
    login(&d, &mut c, 2, 1);
    d.game_packet(&mut c, &event(id::DEBUG_ADD_REGISTER, vec![6, 1]), 0);
    assert_eq!(d.pc(PC).unwrap().register(6), 0); // register for job 1 only
    d.game_packet(&mut c, &event(id::DEBUG_SET_REGISTER, vec![3, 0b101]), 0);
    assert_eq!(d.pc(PC).unwrap().completed_step(101), 0);
    d.game_packet(&mut c, &event(id::DEBUG_SET_REGISTER, vec![3, 0b111]), 0);
    assert_eq!(d.pc(PC).unwrap().completed_step(101), 1);
    d.game_packet(&mut c, &event(id::DEBUG_SET_REGISTER, vec![1, 3]), 0);
    assert_eq!(d.pc(PC).unwrap().completed_step(105), 0);
    d.post(
        internal_event(id::INCREMENT_REGISTER_FROM_CLIENT, vec![5, 1]),
        0,
    );
    assert_eq!(d.pc(PC).unwrap().completed_step(105), 1);
    // IncrementFromClient is refused for a register without the flag.
    d.post(
        internal_event(id::INCREMENT_REGISTER_FROM_CLIENT, vec![3, 1]),
        0,
    );
    assert_eq!(d.pc(PC).unwrap().register(3), 0b111);
}

#[test]
fn complete_from_client_notify_converts_rows() {
    let d = daemon();
    let mut c = connected(&d);
    login(&d, &mut c, 0, 1);
    let p = protocol::packet(
        "hqqqqhChh;Chh;",
        &[
            Value::I16(8),
            Value::I64(0),
            Value::I64(TID),
            Value::I64(PC),
            Value::I64(0),
            Value::I16(1),
            Value::Repeat(vec![vec![Value::I16(5), Value::I16(2)]]),
            Value::Repeat(vec![
                vec![Value::I16(103), Value::I16(1)],
                vec![Value::I16(100), Value::I16(1)],
            ]),
        ],
    );
    d.game_packet(&mut c, &p, 0);
    let pc = d.pc(PC).unwrap();
    assert_eq!(pc.register(5), 2);
    assert_eq!(pc.completed_step(103), 1);
    assert_eq!(pc.completed_step(100), 0); // not complete-from-client
}

#[test]
fn take_reward_reasons_and_replies() {
    let d = daemon();
    let mut c = connected(&d);
    login(&d, &mut c, 0, 1);
    let take_req =
        |tx: u64, id: u64, step: u64| event(id::COMPLETE_TAKE_REWARD, vec![tx, id, step, 0]);
    let reply = |out: Vec<Out>| cluster_packets(&out).pop().unwrap();
    let r = reply(d.game_packet(&mut c, &take_req(1, 100, 1), 0));
    assert_eq!(cmd(&r), game_cmd::TAKE_REWARD_FAIL);
    assert_eq!(
        protocol::fields("hEqqhh", &r).unwrap()[1],
        Value::I16(take::NOT_AVAILABLE)
    );
    let r = reply(d.game_packet(&mut c, &take_req(1, 999, 1), 0));
    assert_eq!(
        protocol::fields("hEqqhh", &r).unwrap()[1],
        Value::I16(take::NOT_FOUND)
    );
    let r = reply(d.game_packet(&mut c, &take_req(1, 101, 1), 0));
    assert_eq!(
        protocol::fields("hEqqhh", &r).unwrap()[1],
        Value::I16(take::NOT_AVAILABLE)
    );
    d.game_packet(&mut c, &event(id::DEBUG_SET_REGISTER, vec![1, 2]), 0);
    let r = reply(d.game_packet(&mut c, &take_req(5, 100, 1), 0));
    let v = protocol::fields("hqqhh", &r).unwrap();
    assert_eq!(cmd(&r), game_cmd::TAKE_REWARD_OK);
    assert_eq!(
        (
            protocol::i64_at(&v, 1),
            protocol::i64_at(&v, 2),
            protocol::i16_at(&v, 3),
            protocol::i16_at(&v, 4)
        ),
        (5, PC, 100, 1)
    );
    let r = reply(d.game_packet(&mut c, &take_req(5, 100, 1), 0));
    assert_eq!(
        protocol::fields("hEqqhh", &r).unwrap()[1],
        Value::I16(take::ALREADY_TAKEN)
    );
    // Transaction id 0: no reply.
    assert!(cluster_packets(&d.game_packet(&mut c, &take_req(0, 100, 1), 0)).is_empty());
}

#[test]
fn tick_writes_store_and_sends_changed_data() {
    let d = daemon();
    let mut c = connected(&d);
    login(&d, &mut c, 0, 1);
    d.tick();
    d.game_packet(&mut c, &event(id::DEBUG_SET_REGISTER, vec![1, 2]), 0);
    d.game_packet(&mut c, &event(id::DEBUG_SET_REGISTER, vec![2, 4]), 0);
    let out = d.tick();
    let packets = cluster_packets(&out);
    assert_eq!(packets.len(), 1);
    assert_eq!(packets[0][38], 1); // changed flag after tid/pcdbid/hint/score/cluster
    let rows = d.store.rows(PC);
    assert_eq!(rows.registers.get(&1), Some(&(1, 2)));
    assert_eq!(rows.registers.get(&2), None); // not persistant
    assert_eq!(rows.completed.get(&100).map(|c| c.0), Some(1));
    assert!(d.tick().is_empty());
    // Failure keeps the markers.
    d.game_packet(&mut c, &event(id::DEBUG_SET_REGISTER, vec![1, 3]), 0);
    d.store.fail.store(true, Ordering::Relaxed);
    d.tick();
    d.store.fail.store(false, Ordering::Relaxed);
    assert!(!d.tick().is_empty());
    assert_eq!(d.store.rows(PC).registers.get(&1), Some(&(1, 3)));
}

#[test]
fn logout_flushes_and_relogin_reloads() {
    let d = daemon();
    let mut c = connected(&d);
    login(&d, &mut c, 0, 1);
    d.game_packet(&mut c, &event(id::DEBUG_SET_REGISTER, vec![1, 9]), 0);
    d.game_packet(&mut c, &event(id::PC_LOGOUT, vec![1]), 0);
    assert!(d.pc(PC).is_none());
    login(&d, &mut c, 0, 1);
    let pc = d.pc(PC).unwrap();
    assert_eq!(pc.register(1), 9);
    assert_eq!(pc.completed_step(100), 2);
    assert_eq!(pc.score, 30);
}

#[test]
fn debug_delete_commands() {
    let d = daemon();
    let mut c = connected(&d);
    login(&d, &mut c, 0, 1);
    d.game_packet(&mut c, &event(id::DEBUG_SET_REGISTER, vec![1, 9]), 0);
    d.tick();
    let out = d.game_packet(&mut c, &event(id::DEBUG_DELETE_ACHIEVEMENT, vec![100]), 0);
    assert_eq!(cmd(&cluster_packets(&out)[0]), game_cmd::DATA);
    let pc = d.pc(PC).unwrap();
    assert_eq!((pc.completed_step(100), pc.score), (0, 0));
    assert!(!d.store.rows(PC).completed.contains_key(&100));
    d.game_packet(&mut c, &event(id::DEBUG_DELETE_ALL, vec![]), 0);
    assert!(d.pc(PC).unwrap().registers.is_empty());
    assert_eq!(d.store.load(PC).unwrap(), Default::default());
}

#[test]
fn disconnect_drops_cluster_pcs() {
    let d = daemon();
    let mut c = connected(&d);
    login(&d, &mut c, 0, 1);
    d.game_packet(&mut c, &event(id::DEBUG_SET_REGISTER, vec![1, 4]), 0);
    d.game_disconnect(&c, 0);
    assert!(d.pc(PC).is_none());
    assert_eq!(d.store.rows(PC).registers.get(&1), Some(&(1, 4)));
    // The cluster id is free again.
    let _again = connected(&d);
}

#[test]
fn manager_port() {
    let d = daemon();
    let mut m = ManagerConn::default();
    let check = protocol::packet("hqq", &[Value::I16(6), Value::I64(PC), Value::I64(TID)]);
    assert!(d.manager_packet(&mut m, &check).is_empty());
    let bad = protocol::packet("hi", &[Value::I16(1), Value::I32(3)]);
    assert_eq!(
        d.manager_packet(&mut m, &bad),
        vec![Out::Reply(protocol::hh(manager_cmd::HELLO_FAIL, 3))]
    );
    let ok = protocol::packet("hi", &[Value::I16(1), Value::I32(4)]);
    assert_eq!(
        d.manager_packet(&mut m, &ok),
        vec![Out::Reply(protocol::packet("h", &[Value::I16(2)]))]
    );
    let ack = |logined: u8| {
        vec![Out::Managers(protocol::packet(
            "hqqc",
            &[
                Value::I16(7),
                Value::I64(PC),
                Value::I64(TID),
                Value::U8(logined),
            ],
        ))]
    };
    assert_eq!(d.manager_packet(&mut m, &check), ack(0));
    let mut c = connected(&d);
    login(&d, &mut c, 0, 1);
    assert_eq!(d.manager_packet(&mut m, &check), ack(1));
    let ping = protocol::packet("hqq", &[Value::I16(4), Value::I64(1), Value::I64(2)]);
    assert_eq!(
        d.manager_packet(&mut m, &ping),
        vec![Out::Reply(protocol::packet(
            "hqq",
            &[Value::I16(5), Value::I64(1), Value::I64(2)]
        ))]
    );
}

/// Loads the real GameDaemon tables when the (git-ignored) data is present.
#[test]
fn real_tables_load_when_present() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../BnS_2020/GameDaemon/data");
    let (Ok(reg), Ok(ach)) = (
        std::fs::read(dir.join("achievementregisterdata.xml")),
        std::fs::read(dir.join("achievementdata.xml")),
    ) else {
        return;
    };
    let rules = Rules::from_files(&reg, &ach).unwrap();
    assert_eq!(rules.registers.len(), 1067);
    assert_eq!(
        rules.register(901, 12).unwrap().alias,
        "901_1_AR_use-grocery-n"
    );
    assert_eq!(rules.achievements.len(), 1558);
}

// Build-348 regression cases. Native addresses are in the AchievementDaemon spec.
fn with_rules(registers: &str, achievements: &str) -> Daemon<MemoryStore> {
    let mut d = daemon();
    d.rules = Rules::from_xml(registers, achievements).unwrap();
    d
}

fn loaded_pc(rules: &Rules, rows: PcRows) -> Pc {
    Pc::new(PC, TID, 1, Profile::default(), rows, rules)
}

fn client_completion(achievement: i16, step: i16) -> Vec<u8> {
    protocol::packet(
        "hqqqqhChh;Chh;",
        &[
            Value::I16(game_cmd::COMPLETE_FROM_CLIENT_NOTIFY),
            Value::I64(5),
            Value::I64(TID),
            Value::I64(PC),
            Value::I64(9),
            Value::I16(1),
            Value::Repeat(vec![]),
            Value::Repeat(vec![vec![Value::I16(achievement), Value::I16(step)]]),
        ],
    )
}

fn data_fields(packets: &[Vec<u8>]) -> Vec<Value> {
    // These small fixtures fit in one fragment. Decode its header and all lists.
    assert_eq!(packets.len(), 1);
    assert_eq!(cmd(&packets[0]), game_cmd::DATA);
    assert_eq!(
        i32::from_le_bytes(packets[0][4..8].try_into().unwrap()) as usize,
        packets[0].len() - 8
    );
    protocol::fields("hiqqqihcChq;Chhq;Chh;", &packets[0]).unwrap()
}

#[test]
fn reward_replies_put_id_before_step_in_both_forms() {
    let d = daemon();
    let mut c = connected(&d);
    login(&d, &mut c, 0, 1);
    let request = event(id::COMPLETE_TAKE_REWARD, vec![123, 100, 1, 0]);
    assert_eq!(
        cluster_packets(&d.game_packet(&mut c, &request, 0)),
        vec![protocol::packet(
            "hEqqhh",
            &[
                Value::I16(10),
                Value::I16(5),
                Value::I64(123),
                Value::I64(PC),
                Value::I16(100),
                Value::I16(1),
            ],
        )]
    );
    d.game_packet(&mut c, &event(id::DEBUG_SET_REGISTER, vec![1, 2]), 0);
    assert_eq!(
        cluster_packets(&d.game_packet(&mut c, &request, 0)),
        vec![protocol::packet(
            "hqqhh",
            &[
                Value::I16(9),
                Value::I64(123),
                Value::I64(PC),
                Value::I16(100),
                Value::I16(1),
            ],
        )]
    );
}

#[test]
fn transaction_zero_replies_only_for_missing_reward_record() {
    let d = daemon();
    let mut c = connected(&d);
    login(&d, &mut c, 0, 1);
    assert_eq!(
        cluster_packets(&d.game_packet(
            &mut c,
            &event(id::COMPLETE_TAKE_REWARD, vec![0, 999, 3, 0]),
            0,
        )),
        vec![protocol::packet(
            "hEqqhh",
            &[
                Value::I16(10),
                Value::I16(6),
                Value::I64(0),
                Value::I64(PC),
                Value::I16(999),
                Value::I16(3),
            ],
        )]
    );
    let found = event(id::COMPLETE_TAKE_REWARD, vec![0, 100, 1, 0]);
    // Suppress both a found-record failure and a found-record success.
    assert!(d.game_packet(&mut c, &found, 0).is_empty());
    d.game_packet(&mut c, &event(id::DEBUG_SET_REGISTER, vec![1, 2]), 0);
    assert!(d.game_packet(&mut c, &found, 0).is_empty());
    assert_eq!(
        d.pc(PC).unwrap().rewards[&100],
        Slot {
            value: 1,
            modified: UPDATE
        }
    );
}

#[test]
fn reward_gates_follow_native_order() {
    let rules = Rules::from_xml(REGISTERS, ACHIEVEMENTS).unwrap();
    // name, completed, marker, deprecated, client, no condition, expected reason.
    let cases = [
        (
            "no progress before completion gate",
            0,
            None,
            false,
            false,
            false,
            5,
        ),
        (
            "no progress before deprecation",
            0,
            None,
            true,
            false,
            false,
            5,
        ),
        ("marker zero is present", 0, Some(0), true, false, false, 7),
        (
            "deprecation before client and taken",
            1,
            Some(2),
            true,
            true,
            true,
            7,
        ),
        (
            "client before incomplete and taken",
            1,
            Some(2),
            false,
            true,
            false,
            5,
        ),
        (
            "no condition before incomplete",
            1,
            Some(2),
            false,
            false,
            true,
            5,
        ),
        (
            "incomplete before taken",
            1,
            Some(2),
            false,
            false,
            false,
            9,
        ),
        (
            "taken before earlier-record scan",
            2,
            Some(2),
            false,
            false,
            false,
            8,
        ),
    ];
    for (name, completed, marker, deprecated, client, no_condition, expected) in cases {
        let mut rows = PcRows::default();
        rows.completed.insert(100, (completed, 5));
        if let Some(marker) = marker {
            rows.rewards.insert(100, marker);
        }
        let pc = loaded_pc(&rules, rows);
        let mut record = rules.record(100, 2, 0).unwrap().clone();
        record.deprecated = deprecated;
        record.complete_from_client = client;
        if no_condition {
            record.conditions.clear();
        }
        assert_eq!(
            pc.available_take_reward(&rules, &record, 0),
            expected,
            "{name}"
        );
    }
}

#[test]
fn reward_scan_checks_earlier_reward_types_backward_and_uses_request_job() {
    let item = "step-complete-reward-item-1='item'";
    let cash = "step-complete-reward-game-cash='1'";
    let zero_cash = "step-complete-reward-game-cash='0'";
    let cases = [
        ("missing earlier record", vec![(3, "", 0)], None, 0, 5),
        (
            "earlier item reward",
            vec![(1, "", 0), (2, item, 0), (3, "", 0)],
            None,
            0,
            9,
        ),
        (
            "earlier cash reward",
            vec![(1, "", 0), (2, cash, 0), (3, "", 0)],
            None,
            0,
            9,
        ),
        (
            "zero cash and rewardless target",
            vec![(1, "", 0), (2, zero_cash, 0), (3, "", 0)],
            None,
            0,
            0,
        ),
        (
            "near reward before distant gap",
            vec![(2, item, 0), (3, "", 0)],
            None,
            0,
            9,
        ),
        (
            "near gap before distant reward",
            vec![(1, item, 0), (3, "", 0)],
            None,
            0,
            5,
        ),
        (
            "do not rescan claimed reward",
            vec![(2, item, 0), (3, "", 0)],
            Some(2),
            0,
            0,
        ),
        (
            "request-job record overrides default",
            vec![(1, "", 0), (2, "", 0), (2, item, 1), (3, "", 0)],
            None,
            1,
            9,
        ),
        (
            "request-job falls back to default",
            vec![(1, "", 0), (2, item, 0), (3, "", 0)],
            None,
            1,
            9,
        ),
    ];
    for (name, steps, marker, job, expected) in cases {
        let mut xml = String::from("<table type='achievement'>");
        for (step, reward, record_job) in steps {
            xml.push_str(&format!(
                "<record id='200' step='{step}' job='{record_job}' register-ref-1='1' register-type-1='above' {reward}/>"
            ));
        }
        xml.push_str("</table>");
        let rules = Rules::from_xml(REGISTERS, &xml).unwrap();
        let mut rows = PcRows::default();
        rows.completed.insert(200, (3, 5));
        if let Some(marker) = marker {
            rows.rewards.insert(200, marker);
        }
        let mut pc = loaded_pc(&rules, rows);
        let record = rules.record(200, 3, job).unwrap();
        let before = pc.rewards.clone();
        assert_eq!(pc.take_reward(&rules, record, job), expected, "{name}");
        if expected == 0 {
            assert_eq!(
                pc.rewards[&200],
                Slot {
                    value: 3,
                    modified: UPDATE
                },
                "{name}"
            );
        } else {
            assert_eq!(pc.rewards, before, "failed gate changed marker: {name}");
        }
    }
}

#[test]
fn client_completion_checks_only_the_requested_exact_record() {
    let xml = r#"<table type='achievement'>
        <record id='200' step='1' current-step-score='2' register-ref-1='1_kill' register-type-1='above' register-value-1='99'/>
        <record id='200' step='2' current-step-score='3' complete-from-client='y'/>
        <record id='200' step='3' current-step-score='5' complete-from-client='y'/>
    </table>"#;
    for (step, score) in [(2, 5), (3, 10)] {
        let d = with_rules(REGISTERS, xml);
        let mut c = connected(&d);
        login(&d, &mut c, 0, 1);
        let out = cluster_packets(&d.game_packet(&mut c, &client_completion(200, step), 20));
        assert_eq!(out.len(), 1);
        let fields = protocol::fields("hsqqqhhhc", &out[0]).unwrap();
        assert_eq!(protocol::i16_at(&fields, 0), 11);
        assert_eq!(protocol::i16_at(&fields, 5), 200);
        assert_eq!(protocol::i16_at(&fields, 6), 0);
        assert_eq!(protocol::i16_at(&fields, 7), step);
        assert_eq!(fields[8], Value::U8(u8::from(step == 3)));
        let pc = d.pc(PC).unwrap();
        assert_eq!(
            pc.completed[&200].value,
            Done {
                step: step as u16,
                time: 20
            }
        );
        assert_eq!(pc.score, score);
        // No duplicate completion and no automatic advance to the next client step.
        assert!(d
            .game_packet(&mut c, &client_completion(200, step), 21)
            .is_empty());
        assert_eq!(d.pc(PC).unwrap().completed_step(200), step as u16);
    }
}

#[test]
fn register_links_keep_exact_steps_and_reject_the_ninth_link() {
    let xml = |count| {
        let mut xml = String::from("<table type='achievement'>");
        for step in 1..=count {
            xml.push_str(&format!(
                "<record id='200' step='{step}' register-ref-1='1_kill' register-type-1='above' register-value-1='{step}'/>"
            ));
        }
        xml.push_str("</table>");
        xml
    };
    let rules = Rules::from_xml(REGISTERS, &xml(8)).unwrap();
    let expected: Vec<_> = (1..=8).map(|step| (200, step, 0)).collect();
    assert_eq!(rules.linked(1, 0), expected);
    let mut pc = loaded_pc(&rules, PcRows::default());
    let mut out = Vec::new();
    pc.set_register(&rules, 1, 8, 5, &mut out);
    assert_eq!(pc.completed_step(200), 8);
    assert_eq!(out.len(), 8);
    let error = Rules::from_xml(REGISTERS, &xml(9)).unwrap_err();
    assert!(error
        .to_string()
        .contains("too many links for register 1 job 0"));
}

#[test]
fn all_five_progress_clear_flags_reset_only_selected_register_and_check_links() {
    let mut registers = String::from("<table type='achievement-register'>");
    for id in 1..=5 {
        registers.push_str(&format!(
            "<record id='{id}' alias='r{id}' type='kill-npc' persistant='y'/>"
        ));
    }
    registers.push_str("</table>");
    for clear in 1..=5 {
        let mut xml = String::from("<table type='achievement'><record id='200' step='1'");
        for n in 1..=5 {
            let flag = if n == clear { "y" } else { "n" };
            xml.push_str(&format!(
                " register-ref-{n}='r{n}' register-type-{n}='above' register-value-{n}='1' progress-step-register-clear-{n}='{flag}'"
            ));
        }
        // This linked record can complete only when SetRegister(0) runs after 200.
        xml.push_str(&format!(
            "/><record id='201' step='1' register-ref-1='r{clear}' register-type-1='below' register-value-1='0' register-ref-2='r{}' register-type-2='above' register-value-2='1'/></table>",
            if clear == 1 { 2 } else { 1 }
        ));
        let rules = Rules::from_xml(&registers, &xml).unwrap();
        let mut pc = loaded_pc(&rules, PcRows::default());
        let mut out = Vec::new();
        // Set the selected register first so 201 cannot complete before the reset.
        let order = std::iter::once(clear).chain((1..=5).filter(|n| *n != clear));
        for n in order {
            pc.set_register(&rules, n, 1, 5, &mut out);
        }
        assert_eq!(pc.completed_step(200), 1, "clear flag {clear}");
        assert_eq!(pc.completed_step(201), 1, "reset must check links: {clear}");
        assert_eq!(out.len(), 2, "reset must not recursively complete 200");
        for n in 1..=5 {
            assert_eq!(
                pc.register(n),
                i64::from(n != clear),
                "clear {clear}, register {n}"
            );
            assert_eq!(pc.registers[&n].modified, INSERT);
        }
    }
}

#[test]
fn register_normalization_limits_client_increment_and_clears_transient_persistence() {
    let kinds = [
        "duel-grade",
        "inventory-size",
        "wardrobe-size",
        "depot-size",
        "pc-level",
        "pc-mastery-level",
        "party-battle-grade",
        "kill-npc",
        "client-only",
    ];
    let mut xml = String::from("<table type='achievement-register'>");
    for (n, kind) in kinds.iter().enumerate() {
        // The explicit permission must be ignored on every non-client-only kind.
        let flag = if *kind == "client-only" { "n" } else { "y" };
        xml.push_str(&format!(
            "<record id='{}' type='{kind}' persistant='y' increment-from-client='{flag}'/>",
            n + 1
        ));
    }
    xml.push_str("</table>");
    let rules = Rules::from_xml(&xml, "<table type='achievement'/>").unwrap();
    let mut pc = loaded_pc(&rules, PcRows::default());
    for (n, kind) in kinds.iter().enumerate() {
        let id = (n + 1) as u16;
        let register = rules.register(id, 0).unwrap();
        assert_eq!(
            register.increment_from_client,
            *kind == "client-only",
            "{kind}"
        );
        assert_eq!(register.persistant, n >= 7, "{kind}");
        assert_eq!(register.version, 1);
        pc.set_register(&rules, id, 4, 5, &mut Vec::new());
        assert_eq!(
            pc.registers[&id].modified,
            if n >= 7 { INSERT } else { CLEAN },
            "{kind}"
        );
    }
    let mut d = with_rules(&xml, "<table type='achievement'/>");
    let mut c = connected(&d);
    login(&d, &mut c, 0, 1);
    for id in 1..=9 {
        let before = d.pc(PC).unwrap().register(id);
        d.post(
            internal_event(id::INCREMENT_REGISTER_FROM_CLIENT, vec![u64::from(id), 2]),
            5,
        );
        assert_eq!(
            d.pc(PC).unwrap().register(id),
            before + if id == 9 { 2 } else { 0 }
        );
    }
    // Also normalize a client-only register with no explicit increment flag.
    d.rules = Rules::from_xml(REGISTERS, ACHIEVEMENTS).unwrap();
    assert!(d.rules.register(5, 0).unwrap().increment_from_client);
}

#[test]
fn deprecation_propagates_backward_only_within_the_exact_job() {
    let xml = r#"<table type='achievement'>
        <record id='200' step='1' job='job-none' current-step-score='10' register-ref-1='1_kill' register-type-1='above' progress-show='y'/>
        <record id='200' step='2' job='job-none' current-step-score='20' deprecated='y'/>
        <record id='200' step='3' job='job-none' current-step-score='7' complete-from-client='y'/>
        <record id='200' step='1' job='검사' current-step-score='11' complete-from-client='y'/>
    </table>"#;
    let rules = Rules::from_xml(REGISTERS, xml).unwrap();
    assert!(rules.record(200, 1, 0).unwrap().deprecated);
    assert!(rules.record(200, 2, 0).unwrap().deprecated);
    assert!(!rules.record(200, 3, 0).unwrap().deprecated);
    assert!(!rules.record(200, 1, 1).unwrap().deprecated);
    assert!(rules.linked(1, 0).is_empty());
    assert_eq!(rules.record(200, 1, 0).unwrap().score_sum, 0);
    assert_eq!(rules.record(200, 3, 0).unwrap().score_sum, 7);
    assert_eq!(rules.record(200, 1, 1).unwrap().score_sum, 11);
    let mut pc = loaded_pc(&rules, PcRows::default());
    let mut out = Vec::new();
    pc.check_complete(&rules, rules.record(200, 1, 0).unwrap(), 5, &mut out);
    assert!(out.is_empty());
    pc.check_complete(&rules, rules.record(200, 3, 0).unwrap(), 5, &mut out);
    assert_eq!(pc.score, 7);
}

#[test]
fn cumulative_score_is_used_on_load_and_completion_with_signed_16_bit_sum() {
    let xml = r#"<table type='achievement'>
        <record id='200' step='1' current-step-score='30000' complete-from-client='y'/>
        <record id='200' step='2' current-step-score='3000' complete-from-client='y'/>
        <record id='200' step='3' current-step-score='7' complete-from-client='y'/>
        <record id='201' step='1' current-step-score='9' complete-from-client='y'/>
    </table>"#;
    let rules = Rules::from_xml(REGISTERS, xml).unwrap();
    assert_eq!(rules.record(200, 1, 0).unwrap().score_sum, 30000);
    assert_eq!(rules.record(200, 2, 0).unwrap().score_sum, -32536);
    assert_eq!(rules.record(200, 3, 0).unwrap().score_sum, -32529);
    let mut rows = PcRows::default();
    rows.completed.insert(200, (2, 5));
    rows.completed.insert(201, (1, 5));
    let mut pc = loaded_pc(&rules, rows);
    assert_eq!(pc.score, -32527);
    pc.check_complete(&rules, rules.record(200, 3, 0).unwrap(), 6, &mut Vec::new());
    assert_eq!(pc.score, -32520); // Replace the old cumulative score, do not add it again.
}

#[test]
fn missing_register_type_defaults_to_none_and_never_completes() {
    let rules = Rules::from_xml(
        REGISTERS,
        r#"<table type='achievement'>
        <record id='200' register-ref-1='1_kill' register-value-1='1'/>
    </table>"#,
    )
    .unwrap();
    let record = rules.record(200, 1, 0).unwrap();
    assert_eq!(record.conditions[0].op, Op::None);
    let mut pc = loaded_pc(&rules, PcRows::default());
    let mut out = Vec::new();
    pc.set_register(&rules, 1, 100, 5, &mut out);
    assert_eq!(pc.completed_step(200), 0);
    assert!(out.is_empty());
}

#[test]
fn full_and_changed_register_lists_hide_hidden_insert_and_update_rows() {
    let registers = r#"<table type='achievement-register'>
        <record id='1' type='kill-npc' persistant='y'/>
        <record id='2' type='kill-npc' persistant='y'/>
        <record id='3' type='kill-npc' progress-show='y'/>
        <record id='4' type='kill-npc' for-newbie-care='y'/>
        <record id='5' type='kill-npc'/>
    </table>"#;
    let xml = r#"<table type='achievement'>
        <record id='200' progress-show='y' register-ref-1='5' register-type-1='above' register-value-1='100'/>
    </table>"#;
    let rules = Rules::from_xml(registers, xml).unwrap();
    assert!(rules.register(5, 0).unwrap().progress_show);
    let mut rows = PcRows::default();
    rows.registers.insert(2, (1, 10));
    let mut pc = loaded_pc(&rules, rows);
    for id in 1..=5 {
        pc.set_register(&rules, id, 20, 5, &mut Vec::new());
    }
    assert_eq!(pc.registers[&1].modified, INSERT);
    assert_eq!(pc.registers[&2].modified, UPDATE);
    assert_eq!(pc.registers[&4].modified, CLEAN); // Newbie visibility alone is not tracked.
    for (changed, ids) in [(false, vec![3, 4, 5]), (true, vec![3, 5])] {
        let fields = data_fields(&pc.data_packets(changed));
        assert_eq!(fields[7], Value::U8(u8::from(changed)));
        let actual: Vec<_> = protocol::rows_at(&fields, 8)
            .iter()
            .map(|row| (protocol::i16_at(row, 0), protocol::i64_at(row, 1)))
            .collect();
        assert_eq!(
            actual,
            ids.into_iter().map(|id| (id, 20)).collect::<Vec<_>>()
        );
    }
    // Once visible markers clear, hidden DB changes must not emit an empty packet.
    pc.registers.get_mut(&3).unwrap().modified = CLEAN;
    pc.registers.get_mut(&5).unwrap().modified = CLEAN;
    assert!(pc.data_packets(true).is_empty());
    assert_eq!(pc.changes().insert_registers, vec![(1, 1, 20)]);
    assert_eq!(pc.changes().update_registers, vec![(2, 1, 20)]);
}

#[test]
fn command_six_rejects_all_internal_classes_but_accepts_an_allowed_event() {
    use achievementdaemon::event::wire_event_allowed;
    assert_eq!(
        EVENTS.iter().filter(|e| wire_event_allowed(e.id)).count(),
        48
    );
    let d = daemon();
    let mut c = connected(&d);
    login(&d, &mut c, 0, 1);
    for internal in [1013, 1014, 1803, 1804, 1805, 1806, 1807] {
        let packet = event(internal, vec![5, 3]);
        assert_eq!(
            Event::parse(&packet),
            Err(ParseError::UnknownEvent(internal))
        );
        assert!(d.game_packet(&mut c, &packet, 5).is_empty());
    }
    assert_eq!(d.pc(PC).unwrap().register(5), 0);
    let allowed = event(id::DEBUG_ADD_REGISTER, vec![5, 3]);
    assert_eq!(Event::parse(&allowed).unwrap().id, id::DEBUG_ADD_REGISTER);
    d.game_packet(&mut c, &allowed, 5);
    assert_eq!(d.pc(PC).unwrap().register(5), 3);
}

#[test]
fn status_check_echoes_supplied_tid_and_uses_pcdbid_routing_membership() {
    let d = daemon();
    let mut c = connected(&d);
    let mut manager = ManagerConn::default();
    d.manager_packet(
        &mut manager,
        &protocol::packet("hi", &[Value::I16(1), Value::I32(4)]),
    );
    let check = |pcdbid, tid, present| {
        assert_eq!(
            d.manager_packet(
                &mut manager.clone(),
                &protocol::packet("hqq", &[Value::I16(6), Value::I64(pcdbid), Value::I64(tid)],)
            ),
            vec![Out::Managers(protocol::packet(
                "hqqc",
                &[
                    Value::I16(7),
                    Value::I64(pcdbid),
                    Value::I64(tid),
                    Value::U8(present)
                ],
            ))]
        );
    };
    check(PC, TID + 100, 0);
    login(&d, &mut c, 0, 1);
    check(PC, TID + 100, 1); // A tid mismatch does not change membership.
    check(PC + 1, TID, 0);
    // A failed load still creates routing membership, without a loaded Pc.
    d.store.fail.store(true, Ordering::Relaxed);
    let mut failed_login = internal_event(id::PC_LOGIN, vec![]);
    failed_login.pcdbid = PC + 1;
    d.post(failed_login, 5);
    assert!(d.pc(PC + 1).is_none());
    check(PC + 1, -987, 1);
}

#[test]
fn manager_ping_requires_hello_and_game_ping_echoes_request_cluster() {
    let d = daemon();
    let mut manager = ManagerConn::default();
    let ping = protocol::packet("hqq", &[Value::I16(4), Value::I64(123), Value::I64(456)]);
    assert!(d.manager_packet(&mut manager, &ping).is_empty());
    assert!(!manager.hello);
    d.manager_packet(
        &mut manager,
        &protocol::packet("hi", &[Value::I16(1), Value::I32(4)]),
    );
    assert_eq!(
        d.manager_packet(&mut manager, &ping),
        vec![Out::Reply(protocol::packet(
            "hqq",
            &[Value::I16(5), Value::I64(123), Value::I64(456)],
        ))]
    );
    let mut c = connected(&d);
    let ping = protocol::packet("hqh", &[Value::I16(4), Value::I64(321), Value::I16(999)]);
    assert_eq!(
        d.game_packet(&mut c, &ping, 5),
        vec![Out::Reply(protocol::packet(
            "hqh",
            &[Value::I16(5), Value::I64(321), Value::I16(999)],
        ))]
    );
}

#[test]
fn check_ping_expires_only_after_timeout_and_fresh_ping_keeps_channel() {
    let mut d = daemon();
    d.config.check_ping_time_ms = 100;
    let mut c = GameConn::new(IP);
    d.game_packet_at(&mut c, &hello(1, 10, 33), 0, 1000);
    assert!(d.check_ping(1100).is_empty()); // Strict >, not >=.
                                            // Event time does not refresh the monotonic Ping clock.
    let ping = protocol::packet("hqh", &[Value::I16(4), Value::I64(1), Value::I16(1)]);
    d.game_packet_at(&mut c, &ping, 99999, 1090);
    assert!(d.check_ping(1101).is_empty());
    assert!(d.check_ping(1190).is_empty());
    // A malformed Ping must not refresh last_ping_ms.
    d.game_packet_at(&mut c, &protocol::packet("h", &[Value::I16(4)]), 0, 1190);
    assert_eq!(
        d.check_ping(1191),
        vec![Out::Expired {
            cluster: 1,
            token: c.token,
            close: true
        }]
    );
    assert!(d.state.lock().unwrap().clusters.is_empty());
    assert!(d.check_ping(1192).is_empty());
}

#[test]
fn ping_timeout_honors_disable_and_socket_close_options() {
    let mut d = daemon();
    d.config.check_ping_time_ms = 100;
    d.config.ping_enable = false;
    let c = connected(&d);
    login(&d, &mut c.clone(), 0, 1);
    assert!(d.check_ping(101).is_empty());
    assert!(d.state.lock().unwrap().clusters.contains_key(&1));
    d.config.ping_enable = true;
    d.config.disconnect_when_ping_timeout = false;
    assert_eq!(
        d.check_ping(101),
        vec![Out::Expired {
            cluster: 1,
            token: c.token,
            close: false
        }]
    );
    assert!(d.pc(PC).is_some()); // Removal without socket close keeps Pcs loaded.
    assert!(d.state.lock().unwrap().clusters.is_empty());
    d.game_disconnect(&c, 0);
    assert!(d.pc(PC).is_none()); // The expired owner can still close later.
}

#[test]
fn stale_disconnect_after_reconnect_keeps_new_channel_pcs_and_routes() {
    let mut d = daemon();
    d.config.check_ping_time_ms = 100;
    let mut old = connected(&d);
    login(&d, &mut old, 0, 1);
    assert_eq!(
        d.check_ping(101),
        vec![Out::Expired {
            cluster: 1,
            token: old.token,
            close: true
        }]
    );
    let mut new = GameConn::new(IP);
    assert_eq!(
        d.game_packet_at(&mut new, &hello(1, 10, 33), 0, 101),
        vec![Out::Reply(protocol::hh(game_cmd::HELLO_ACK, 1))]
    );
    assert_ne!(old.token, new.token);
    let mut relogin = internal_event(id::PC_LOGIN, vec![]);
    relogin.tid = TID + 1;
    d.post(relogin, 5);
    let before = d.pc(PC).unwrap();
    // Old Ping cannot refresh the new channel's timer.
    let ping = protocol::packet("hqh", &[Value::I16(4), Value::I64(1), Value::I16(1)]);
    d.game_packet_at(&mut old, &ping, 0, 200);
    assert_eq!(d.state.lock().unwrap().clusters[&1].last_ping_ms, 101);
    d.game_disconnect(&old, 5);
    assert_eq!(d.pc(PC), Some(before));
    {
        let state = d.state.lock().unwrap();
        assert_eq!(state.clusters[&1].token, new.token);
        assert_eq!(state.routes.get(&PC), Some(&1));
    }
    assert!(d.check_ping(201).is_empty());
    assert_eq!(
        d.check_ping(202),
        vec![Out::Expired {
            cluster: 1,
            token: new.token,
            close: true
        }]
    );
    // Even after the replacement expires, an older token must not own its Pcs.
    d.game_disconnect(&old, 5);
    assert!(d.pc(PC).is_some());
    d.game_disconnect(&new, 5);
    assert!(d.pc(PC).is_none());
    assert!(!d.state.lock().unwrap().routes.contains_key(&PC));
}

#[test]
fn register_version_mismatch_resets_marks_update_and_sends_command_twelve() {
    let registers = r#"<table type='achievement-register'>
        <record id='1' type='kill-npc' version='3' persistant='y' progress-show='y'/>
        <record id='2' type='kill-npc' version='4' persistant='y' progress-show='y'/>
    </table>"#;
    let d = with_rules(registers, "<table type='achievement'/>");
    d.store
        .insert_registers(PC, &[(1, 2, 123456), (2, 4, 789), (99, 1, 10)])
        .unwrap();
    let mut c = connected(&d);
    let out = cluster_packets(&login(&d, &mut c, 0, 1));
    assert_eq!(out.len(), 2);
    assert_eq!(
        out[0],
        protocol::packet(
            "hsqqqhhhqq",
            &[
                Value::I16(12),
                Value::String("Pc".encode_utf16().collect()),
                Value::I64(PC),
                Value::I64(0x99),
                Value::I64(TID),
                Value::I16(1),
                Value::I16(2),
                Value::I16(3),
                Value::I64(123456),
                Value::I64(0),
            ],
        )
    );
    let data = data_fields(&out[1..]);
    assert_eq!(
        protocol::rows_at(&data, 8),
        &[
            vec![Value::I16(1), Value::U64(0)],
            vec![Value::I16(2), Value::U64(789)],
        ]
    );
    let pc = d.pc(PC).unwrap();
    assert_eq!(pc.registers[&1].version, 3);
    assert_eq!(pc.registers[&1].modified, UPDATE);
    assert_eq!(pc.registers[&2].modified, CLEAN);
    assert!(!pc.registers.contains_key(&99));
    assert_eq!(pc.changes().update_registers, vec![(1, 3, 0)]);
    d.tick();
    assert_eq!(d.pc(PC).unwrap().registers[&1].modified, CLEAN);
    assert_eq!(d.store.rows(PC).registers[&1], (3, 0));
    d.game_packet(&mut c, &event(id::PC_LOGOUT, vec![1]), 5);
    let reloaded = cluster_packets(&login(&d, &mut c, 0, 1));
    assert_eq!(reloaded.len(), 1);
    assert_eq!(cmd(&reloaded[0]), game_cmd::DATA);
    assert_eq!(d.pc(PC).unwrap().registers[&1].modified, CLEAN);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriteGroup {
    InsertRegisters,
    UpdateRegisters,
    InsertCompleted,
    UpdateCompleted,
    Rewards,
}

#[derive(Default)]
struct GroupFailStore {
    memory: MemoryStore,
    failing: Mutex<Option<WriteGroup>>,
    calls: Mutex<Vec<WriteGroup>>,
}

impl GroupFailStore {
    fn write(
        &self,
        group: WriteGroup,
        empty: bool,
        write: impl FnOnce() -> Result<(), StoreError>,
    ) -> Result<(), StoreError> {
        if empty {
            return Ok(());
        }
        self.calls.lock().unwrap().push(group);
        if *self.failing.lock().unwrap() == Some(group) {
            Err(StoreError)
        } else {
            write()
        }
    }
}

impl Store for GroupFailStore {
    fn load(&self, pcdbid: i64) -> Result<PcRows, StoreError> {
        self.memory.load(pcdbid)
    }
    fn insert_registers(&self, pcdbid: i64, rows: &[(u16, u16, i64)]) -> Result<(), StoreError> {
        self.write(WriteGroup::InsertRegisters, rows.is_empty(), || {
            self.memory.insert_registers(pcdbid, rows)
        })
    }
    fn update_registers(&self, pcdbid: i64, rows: &[(u16, u16, i64)]) -> Result<(), StoreError> {
        self.write(WriteGroup::UpdateRegisters, rows.is_empty(), || {
            self.memory.update_registers(pcdbid, rows)
        })
    }
    fn insert_completed(&self, pcdbid: i64, rows: &[(u16, u16, i64)]) -> Result<(), StoreError> {
        self.write(WriteGroup::InsertCompleted, rows.is_empty(), || {
            self.memory.insert_completed(pcdbid, rows)
        })
    }
    fn update_completed(&self, pcdbid: i64, rows: &[(u16, u16, i64)]) -> Result<(), StoreError> {
        self.write(WriteGroup::UpdateCompleted, rows.is_empty(), || {
            self.memory.update_completed(pcdbid, rows)
        })
    }
    fn take_step_reward(&self, pcdbid: i64, rows: &[(u16, u16)]) -> Result<(), StoreError> {
        self.write(WriteGroup::Rewards, rows.is_empty(), || {
            self.memory.take_step_reward(pcdbid, rows)
        })
    }
    fn delete_completed(&self, pcdbid: i64, id: u16) -> Result<(), StoreError> {
        self.memory.delete_completed(pcdbid, id)
    }
    fn delete_all(&self, pcdbid: i64) -> Result<(), StoreError> {
        self.memory.delete_all(pcdbid)
    }
}

#[test]
fn update_pc_attempts_all_groups_and_retries_only_failed_markers() {
    let groups = [
        WriteGroup::InsertRegisters,
        WriteGroup::UpdateRegisters,
        WriteGroup::InsertCompleted,
        WriteGroup::UpdateCompleted,
        WriteGroup::Rewards,
    ];
    for failing in groups {
        let template = daemon();
        let store = GroupFailStore::default();
        let mut rows = PcRows::default();
        rows.registers.insert(3, (1, 0));
        rows.completed.insert(101, (0, 1));
        store.memory.pcs.lock().unwrap().insert(PC, rows.clone());
        *store.failing.lock().unwrap() = Some(failing);
        let d = Daemon::new(template.config, template.iplist, template.rules, store);
        let mut pc = loaded_pc(&d.rules, rows);
        pc.set_register(&d.rules, 1, 2, 5, &mut Vec::new()); // register + completed INSERT
        pc.set_register(&d.rules, 3, 7, 5, &mut Vec::new()); // register + completed UPDATE
        assert_eq!(
            pc.take_reward(&d.rules, d.rules.record(100, 1, 0).unwrap(), 0),
            take::OK
        );
        let pending = pc.changes();
        assert_eq!(pending.insert_registers, vec![(1, 1, 2)]);
        assert_eq!(pending.update_registers, vec![(3, 1, 7)]);
        assert_eq!(pending.insert_completed, vec![(100, 1, 5)]);
        assert_eq!(pending.update_completed, vec![(101, 1, 5)]);
        assert_eq!(pending.rewards, vec![(100, 1)]);
        d.state.lock().unwrap().pcs.insert(PC, pc);
        d.tick();
        assert_eq!(
            *d.store.calls.lock().unwrap(),
            groups,
            "failure in {failing:?}"
        );
        let pc = d.pc(PC).unwrap();
        let markers = [
            pc.registers[&1].modified,
            pc.registers[&3].modified,
            pc.completed[&100].modified,
            pc.completed[&101].modified,
            pc.rewards[&100].modified,
        ];
        for ((group, original), marker) in groups
            .into_iter()
            .zip([INSERT, UPDATE, INSERT, UPDATE, UPDATE])
            .zip(markers)
        {
            assert_eq!(
                marker,
                if group == failing { original } else { CLEAN },
                "failed {failing:?}, group {group:?}"
            );
        }
        let stored = d.store.memory.rows(PC);
        assert_eq!(
            stored.registers.get(&1).copied(),
            if failing == WriteGroup::InsertRegisters {
                None
            } else {
                Some((1, 2))
            }
        );
        assert_eq!(
            stored.registers[&3],
            (
                1,
                if failing == WriteGroup::UpdateRegisters {
                    0
                } else {
                    7
                }
            )
        );
        assert_eq!(
            stored.completed.get(&100).copied(),
            if failing == WriteGroup::InsertCompleted {
                None
            } else {
                Some((1, 5))
            }
        );
        assert_eq!(
            stored.completed[&101],
            if failing == WriteGroup::UpdateCompleted {
                (0, 1)
            } else {
                (1, 5)
            }
        );
        assert_eq!(
            stored.rewards.get(&100).copied(),
            if failing == WriteGroup::Rewards {
                None
            } else {
                Some(1)
            }
        );
        d.store.calls.lock().unwrap().clear();
        *d.store.failing.lock().unwrap() = None;
        d.tick();
        assert_eq!(*d.store.calls.lock().unwrap(), vec![failing]);
        assert!(d.pc(PC).unwrap().changes().is_empty());
        let stored = d.store.memory.rows(PC);
        assert_eq!(stored.registers[&1], (1, 2));
        assert_eq!(stored.registers[&3], (1, 7));
        assert_eq!(stored.completed[&100], (1, 5));
        assert_eq!(stored.completed[&101], (1, 5));
        assert_eq!(stored.rewards[&100], 1);
        d.store.calls.lock().unwrap().clear();
        assert!(d.tick().is_empty());
        assert!(d.store.calls.lock().unwrap().is_empty());
    }
}
