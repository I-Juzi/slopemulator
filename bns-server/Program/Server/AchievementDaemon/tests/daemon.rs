use achievementdaemon::config::{DaemonConfig, IpList};
use achievementdaemon::event::{id, Event, FieldValue, ParseError};
use achievementdaemon::pc::take;
use achievementdaemon::protocol::{self, game_cmd, manager_cmd};
use achievementdaemon::rules::{job_from_name, Op, Rules};
use achievementdaemon::schema::EVENTS;
use achievementdaemon::service::{Daemon, GameConn, ManagerConn, Out};
use achievementdaemon::store::{MemoryStore, Store};
use bloodlib::packet::Value;
use std::net::Ipv4Addr;
use std::sync::atomic::Ordering;

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

fn connected(d: &Daemon<MemoryStore>) -> GameConn {
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
fn login(d: &Daemon<MemoryStore>, conn: &mut GameConn, job: u64, level: u64) -> Vec<Out> {
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
