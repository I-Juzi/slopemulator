use bloodlib::packet::{decode_fields, DecodeLimits, Value};
use packet::fragments::FragmentReader;
use rankingdaemon::config::{
    load_result_schedules, parse_game_list, parse_infogate_list, parse_season_data,
    parse_season_data_at, parse_time, RankingConfig, ResultSchedule,
};
use rankingdaemon::protocol::{fields, game_cmd, infogate_cmd, manage_cmd, packet};
use rankingdaemon::service::{max_ranking_list_index, Daemon, Flow, Out, Peer};
use std::net::Ipv4Addr;

const CONFIG: &str = r#"<config module='rankingd'>
  <option name='root-path' value='.'/>
  <option name='data-path' value='data'/>
  <option name='log-path' value='log'/>
  <option name='config-path' value='config'/>
  <option name='transaction-log-path' value='tlog'/>
  <option name='cert-file' value='cert'/>
  <option name='service-mode' value='6'/>
  <option name='world-listen-port' value='22501'/>
  <option name='service-world-id' value='11050'/>
  <option name='pc-list-hash-count' value='10'/>
  <option name='ranking-list-update-period' value='600000'/>
</config>"#;

const GAMES: &str = r#"<?xml version='1.0' encoding='UTF-8'?>
<game-list>
  <!-- comment -->
  <game service-type='1' id='1' ip-address='10.0.0.5' internal-port='20100'/>
  <game service-type='2' id='10001' ip-address='10.0.0.6' internal-port='21900'/>
</game-list>"#;

const INFOGATES: &str = r#"<infogate-list>
  <infogate wid='1' ip-address='10.0.0.7' internal-port='22100'/>
</infogate-list>"#;

const SEASONS: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<table release-module="ItemData" release-side="ranking" type="season-contents-schedule" version="0.4">
  <record default-rating-score="130000" player-type="pc" ranking-minimum-score="160000" reward-group-data="14" season-contents-schedule-number="46" season-contents-type="duel-death-match-1vs1" season-end-time="2022-08-21 7:00:00" season-name="Duel 1vs1" season-reward-expiration-time="2022-09-18 4:00:00" season-start-time="2022-05-21 7:00:00" season-type="normal" service-mode="live-rc"/>
  <record default-rating-score="0" player-type="guild" ranking-minimum-score="0" reward-group-data="4006" season-contents-schedule-number="3" season-contents-type="guild-combat" season-end-time="2022-08-21 7:00:00" season-name="Guild" season-reward-expiration-time="2022-09-18 4:00:00" season-start-time="2022-05-21 7:00:00" season-type="free" service-mode="live-rc"/>
  <record default-rating-score="0" player-type="pc" ranking-minimum-score="0" reward-group-data="1" season-contents-schedule-number="9" season-contents-type="guild-combat" season-end-time="2022-08-21 7:00:00" season-name="Other mode" season-reward-expiration-time="2022-09-18 4:00:00" season-start-time="2022-05-21 7:00:00" season-type="free" service-mode="live-1"/>
</table>"#;

const GAME_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 5);
const INFOGATE_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 7);

fn t(text: &str) -> i64 {
    parse_time(text).unwrap()
}

fn daemon() -> Daemon {
    let config = RankingConfig::parse(CONFIG).unwrap();
    let mut seasons = parse_season_data(SEASONS).unwrap();
    load_result_schedules(&mut seasons, config.service_mode, 0).unwrap();
    Daemon::new(
        &config,
        &parse_game_list(GAMES).unwrap(),
        &parse_infogate_list(INFOGATES).unwrap(),
        seasons,
    )
}

fn in_season() -> i64 {
    t("2022-06-01 0:00:00")
}

fn sent(out: &[Out]) -> Vec<Vec<u8>> {
    out.iter()
        .filter_map(|o| match o {
            Out::Send(_, p) => Some(p.clone()),
            Out::Close(_) => None,
        })
        .collect()
}

fn game_hello(d: &mut Daemon, id: u64, cluster: i16, version: i16) -> (Vec<Out>, Flow) {
    let p = packet(
        "hhh",
        &[Value::I16(1), Value::I16(version), Value::I16(cluster)],
    );
    d.handle(Peer::Game(id), GAME_IP, &p, in_season())
}

#[test]
fn config_defaults_and_required() {
    let c = RankingConfig::parse(CONFIG).unwrap();
    assert_eq!(c.world_listen_port, 22501);
    assert_eq!(c.infogate_listen_port, 12520);
    assert_eq!(c.manage_listen_port, 32510);
    assert_eq!(c.max_world, 100);
    assert_eq!(c.timer_max_time, 30);
    assert_eq!(c.skip_list_ex_prob, 36);
    assert_eq!(c.service_mode, 6);
    let missing = CONFIG.replace("<option name='cert-file' value='cert'/>", "");
    assert!(RankingConfig::parse(&missing).is_err());
    let wrong = CONFIG.replace("module='rankingd'", "module='other'");
    assert!(RankingConfig::parse(&wrong).is_err());
    let mode = CONFIG.replace("value='6'", "value='12'");
    assert!(RankingConfig::parse(&mode).is_err());
}

#[test]
fn lists_parse_and_reject_other_elements() {
    let games = parse_game_list(GAMES).unwrap();
    assert_eq!(games.len(), 2);
    assert_eq!(
        (games[1].service_type, games[1].id, games[1].port),
        (2, 10001, 21900)
    );
    assert!(parse_game_list("<game-list><server id='1'/></game-list>").is_err());
    assert!(parse_game_list("<game-list><game service-type='3' id='1'/></game-list>").is_err());
    let gates = parse_infogate_list(INFOGATES).unwrap();
    assert_eq!(
        (gates[0].wid, gates[0].ip, gates[0].port),
        (1, INFOGATE_IP, 22100)
    );
}

#[test]
fn season_data_parses_names_and_times() {
    let s = parse_season_data(SEASONS).unwrap();
    assert_eq!(s.len(), 3);
    assert_eq!(s[0].season_contents_type, 1);
    assert_eq!(s[0].season_type, 2);
    assert_eq!(s[0].player_type, 1);
    assert_eq!(s[0].service_mode, 6);
    assert_eq!(s[1].player_type, 2);
    assert_eq!(s[2].service_mode, 8);
    assert_eq!(s[0].start_time, 1_653_116_400);
    assert_eq!(
        parse_season_data(&SEASONS.replace("live-rc", "bogus"))
            .unwrap()
            .len(),
        1
    );
    let no_reward = SEASONS.replace(r#"reward-group-data="14" "#, "");
    assert_eq!(parse_season_data(&no_reward).unwrap()[0].reward_group_id, 0);
    assert!(parse_season_data(&SEASONS.replace("2022-08-21 7:00:00", "never")).is_err());
    assert_eq!(parse_time("1970-01-01 00:00:00"), Some(0));
    assert_eq!(parse_time("2022-13-01 0:00:00"), None);
    // Only seasons for the configured service mode stay.
    assert_eq!(daemon().seasons().len(), 2);
}

#[test]
fn max_ranking_list_index_table() {
    assert_eq!(max_ranking_list_index(1), 14);
    assert_eq!(max_ranking_list_index(2), 0);
    assert_eq!(max_ranking_list_index(8), 2);
    assert_eq!(max_ranking_list_index(13), 1);
    assert_eq!(max_ranking_list_index(15), 14);
    assert_eq!(max_ranking_list_index(16), 0);
}

#[test]
fn game_hello_failures() {
    let mut d = daemon();
    let (out, flow) = game_hello(&mut d, 1, 1, 9);
    assert_eq!(flow, Flow::Close);
    assert_eq!(
        sent(&out),
        vec![packet("hh", &[Value::I16(3), Value::I16(7)])]
    );
    let (out, _) = game_hello(&mut d, 1, 2, 10);
    assert_eq!(
        sent(&out),
        vec![packet("hh", &[Value::I16(3), Value::I16(2)])]
    );
    // Arena id 10001 is service type 2 but the ip is 10.0.0.6.
    let (out, flow) = game_hello(&mut d, 1, 10001, 10);
    assert_eq!(flow, Flow::Close);
    assert_eq!(
        sent(&out),
        vec![packet("hh", &[Value::I16(3), Value::I16(9)])]
    );
    d.service_available = false;
    let (out, _) = game_hello(&mut d, 1, 1, 10);
    assert_eq!(
        sent(&out),
        vec![packet("hh", &[Value::I16(3), Value::I16(2)])]
    );
    let (out, _) = d.handle(Peer::Game(1), GAME_IP, &[4, 0, 1, 0], 0);
    assert_eq!(
        sent(&out),
        vec![packet("hh", &[Value::I16(3), Value::I16(2)])]
    );
}

#[test]
fn game_hello_sends_season_list_fragments() {
    let mut d = daemon();
    let (out, flow) = game_hello(&mut d, 1, 1, 10);
    assert_eq!(flow, Flow::Keep);
    assert_eq!(d.game_session(1, 1), Some(1));
    let frames = sent(&out);
    let mut reader = FragmentReader::default();
    let mut joined = None;
    for f in &frames {
        joined = reader.add(f).unwrap().map(|j| (j.cmd, j.data.into_owned()));
    }
    let (cmd, data) = joined.unwrap();
    assert_eq!(cmd, game_cmd::HELLO_ACK);
    let limits = DecodeLimits {
        blob_bytes: 0,
        string_units: 64,
        property_bytes: 0,
    };
    let v = decode_fields("CcischqqqhiCcqqqqQ;;CcischqqqhiCcqqqqQ;;", &data, limits).unwrap();
    let Value::Repeat(active) = &v[0] else {
        panic!()
    };
    let Value::Repeat(result) = &v[1] else {
        panic!()
    };
    assert_eq!(active.len(), 2);
    assert!(result.is_empty());
    assert_eq!(active[0][0], Value::U8(1));
    assert_eq!(active[0][1], Value::U32(46));
    assert_eq!(active[0][4], Value::I16(1));
    assert_eq!(active[0][8], Value::I16(14));
    assert_eq!(active[0][9], Value::U32(130000));
    // Normal season without weekly ending: one final result row.
    assert_eq!(
        active[0][10],
        Value::Repeat(vec![vec![
            Value::U8(1),
            Value::U64(t("2022-05-21 7:00:00") as u64),
            Value::U64(t("2022-08-21 7:00:00") as u64),
            Value::U64(t("2022-08-21 7:00:00") as u64),
            Value::U64(t("2022-09-18 4:00:00") as u64),
            Value::I8(1),
        ]])
    );
}

#[test]
fn game_hello_after_end_moves_season_to_result_list() {
    let d = daemon();
    let frames = d.season_list(t("2022-09-01 0:00:00")).unwrap();
    let data = &frames[0][8..];
    let limits = DecodeLimits {
        blob_bytes: 0,
        string_units: 64,
        property_bytes: 0,
    };
    let v = decode_fields("CcischqqqhiCcqqqqQ;;CcischqqqhiCcqqqqQ;;", data, limits).unwrap();
    assert_eq!(v[0], Value::Repeat(vec![]));
    let Value::Repeat(result) = &v[1] else {
        panic!()
    };
    // The free season expires at end + 3600, so only the normal one stays.
    assert_eq!(result.len(), 1);
    assert_eq!(result[0][1], Value::U32(46));
}

#[test]
fn game_duplicate_hello_closes_both() {
    let mut d = daemon();
    game_hello(&mut d, 1, 1, 10);
    let (out, flow) = game_hello(&mut d, 2, 1, 10);
    assert_eq!(flow, Flow::Close);
    assert!(out.contains(&Out::Close(Peer::Game(1))));
    assert_eq!(
        sent(&out),
        vec![packet("hh", &[Value::I16(3), Value::I16(11)])]
    );
    assert_eq!(d.game_session(1, 1), None);
    // The slot is free again.
    let (_, flow) = game_hello(&mut d, 3, 1, 10);
    assert_eq!(flow, Flow::Keep);
}

#[test]
fn game_gate_and_ping() {
    let mut d = daemon();
    let ping = packet("hqh", &[Value::I16(4), Value::U64(7), Value::I16(1)]);
    let (out, flow) = d.handle(Peer::Game(1), GAME_IP, &ping, 0);
    assert_eq!((out.is_empty(), flow), (true, Flow::Close));
    game_hello(&mut d, 1, 1, 10);
    let (out, flow) = d.handle(Peer::Game(1), GAME_IP, &ping, 0);
    assert_eq!(flow, Flow::Keep);
    assert_eq!(
        sent(&out),
        vec![packet(
            "hqh",
            &[Value::I16(5), Value::U64(7), Value::I16(1)]
        )]
    );
    // Second hello on an authed session, and an unknown command.
    let (_, flow) = game_hello(&mut d, 1, 1, 10);
    assert_eq!(flow, Flow::Close);
    let (_, flow) = d.handle(Peer::Game(1), GAME_IP, &[4, 0, 25, 0], 0);
    assert_eq!(flow, Flow::Close);
    d.closed(Peer::Game(1));
    assert_eq!(d.game_session(1, 1), None);
}

fn rating(kind: u8, schedule: u32, job: u8) -> Vec<u8> {
    packet(
        "hqQqhQiQqcisqcqss",
        &[
            Value::I16(18),
            Value::U64(100),
            Value::I8(1),
            Value::U64(555),
            Value::I16(1),
            Value::I8(3),
            Value::U32(10),
            Value::I8(1),
            Value::U64(1_654_000_000),
            Value::U8(kind),
            Value::U32(schedule),
            Value::String("pc".encode_utf16().collect()),
            Value::U64(130000),
            Value::U8(job),
            Value::U64(0),
            Value::String(vec![]),
            Value::String(vec![]),
        ],
    )
}

fn rating_fail(kind: u8, schedule: u32, code: i16) -> Vec<u8> {
    packet(
        "hqQqhQciE",
        &[
            Value::I16(20),
            Value::U64(100),
            Value::I8(1),
            Value::U64(555),
            Value::I16(1),
            Value::I8(3),
            Value::U8(kind),
            Value::U32(schedule),
            Value::I16(code),
        ],
    )
}

#[test]
fn insert_player_rating_score_fail_reasons() {
    let mut d = daemon();
    game_hello(&mut d, 1, 1, 10);
    let mut check = |p: Vec<u8>, want: Vec<u8>| {
        let (out, flow) = d.handle(Peer::Game(1), GAME_IP, &p, in_season());
        assert_eq!(flow, Flow::Keep);
        assert_eq!(sent(&out), vec![want]);
    };
    check(rating(1, 46, 15), rating_fail(1, 46, 6));
    check(rating(16, 46, 1), rating_fail(16, 46, 6));
    check(rating(1, 0, 1), rating_fail(1, 0, 6));
    check(rating(1, 47, 1), rating_fail(1, 47, 2));
    check(rating(1, 46, 1), rating_fail(1, 46, 4));
}

#[test]
fn insert_ranking_score_fail_layout() {
    let mut d = daemon();
    game_hello(&mut d, 1, 1, 10);
    let p = packet(
        "hqqhQqqqciscqss",
        &[
            Value::I16(21),
            Value::U64(9),
            Value::U64(555),
            Value::I16(1),
            Value::I8(2),
            Value::U64(0),
            Value::U64(10),
            Value::U64(1_654_000_000),
            Value::U8(6),
            Value::U32(1),
            Value::String(vec![]),
            Value::U8(0),
            Value::U64(0),
            Value::String(vec![]),
            Value::String(vec![]),
        ],
    );
    let (out, _) = d.handle(Peer::Game(1), GAME_IP, &p, in_season());
    let want = packet(
        "hciqqhQE",
        &[
            Value::I16(23),
            Value::U8(6),
            Value::U32(1),
            Value::U64(9),
            Value::U64(555),
            Value::I16(1),
            Value::I8(2),
            Value::I16(6),
        ],
    );
    assert_eq!(sent(&out), vec![want]);
}

#[test]
fn delete_player_notify_black_lists_without_reply() {
    let mut d = daemon();
    game_hello(&mut d, 1, 1, 10);
    let p = packet("hQq", &[Value::I16(24), Value::I8(1), Value::U64(77)]);
    let (out, flow) = d.handle(Peer::Game(1), GAME_IP, &p, 0);
    assert_eq!((out.is_empty(), flow), (true, Flow::Keep));
    assert!(d.is_black_listed(1, 77));
    let p = packet("hQq", &[Value::I16(24), Value::I8(3), Value::U64(78)]);
    d.handle(Peer::Game(1), GAME_IP, &p, 0);
    assert!(!d.is_black_listed(3, 78));
}

fn infogate_hello(d: &mut Daemon, id: u64, port: i16) -> (Vec<Out>, Flow) {
    let p = packet(
        "hhhh",
        &[
            Value::I16(1),
            Value::I16(6),
            Value::I16(1),
            Value::I16(port),
        ],
    );
    d.handle(Peer::InfoGate(id), INFOGATE_IP, &p, in_season())
}

#[test]
fn infogate_hello_checks_and_current_seasons() {
    let mut d = daemon();
    let (out, flow) = infogate_hello(&mut d, 1, 22101);
    assert_eq!(flow, Flow::Close);
    assert_eq!(
        sent(&out),
        vec![packet("hh", &[Value::I16(3), Value::I16(10)])]
    );
    let (out, flow) = d.handle(
        Peer::InfoGate(1),
        GAME_IP,
        &packet(
            "hhhh",
            &[
                Value::I16(1),
                Value::I16(6),
                Value::I16(1),
                Value::I16(22100),
            ],
        ),
        0,
    );
    assert_eq!(flow, Flow::Close);
    assert_eq!(
        sent(&out),
        vec![packet("hh", &[Value::I16(3), Value::I16(9)])]
    );
    let (out, flow) = infogate_hello(&mut d, 1, 22100);
    assert_eq!(flow, Flow::Keep);
    let want = packet(
        "hCcc;",
        &[
            Value::I16(infogate_cmd::HELLO_ACK),
            Value::Repeat(vec![
                vec![Value::U8(1), Value::U8(15)],
                vec![Value::U8(8), Value::U8(3)],
            ]),
        ],
    );
    assert_eq!(sent(&out), vec![want]);
    let (out, flow) = infogate_hello(&mut d, 2, 22100);
    assert_eq!(flow, Flow::Close);
    assert!(out.contains(&Out::Close(Peer::InfoGate(1))));
    assert_eq!(d.infogate_session(1), None);
}

#[test]
fn infogate_requests_need_hello() {
    let mut d = daemon();
    let p = packet("hcc", &[Value::I16(5), Value::U8(1), Value::U8(0)]);
    let (_, flow) = d.handle(Peer::InfoGate(1), INFOGATE_IP, &p, 0);
    assert_eq!(flow, Flow::Close);
    infogate_hello(&mut d, 1, 22100);
    let (out, flow) = d.handle(Peer::InfoGate(1), INFOGATE_IP, &p, 0);
    assert_eq!(flow, Flow::Keep);
    assert_eq!(
        sent(&out),
        vec![packet(
            "hccE",
            &[Value::I16(7), Value::U8(1), Value::U8(0), Value::I16(2)]
        )]
    );
}

fn manage(d: &mut Daemon, p: Vec<u8>, now: i64) -> (Vec<Vec<u8>>, Flow) {
    let (out, flow) = d.handle(Peer::Manage(1), Ipv4Addr::LOCALHOST, &p, now);
    (sent(&out), flow)
}

fn black(cmd: i16, tid: u64, player: u64, kind: i8) -> Vec<u8> {
    packet(
        "hqqQ",
        &[
            Value::I16(cmd),
            Value::U64(tid),
            Value::U64(player),
            Value::I8(kind),
        ],
    )
}

#[test]
fn manage_hello_and_black_list() {
    let mut d = daemon();
    let now = in_season();
    let (_, flow) = manage(&mut d, black(7, 1, 2, 1), now);
    assert_eq!(flow, Flow::Close);
    let hello = |v: i16| packet("hhh", &[Value::I16(1), Value::I16(5), Value::I16(v)]);
    let (out, flow) = manage(&mut d, hello(2), now);
    assert_eq!(
        (out, flow),
        (
            vec![packet("hh", &[Value::I16(3), Value::I16(6)])],
            Flow::Close
        )
    );
    let (out, flow) = manage(&mut d, hello(1), now);
    assert_eq!(
        (out, flow),
        (vec![packet("h", &[Value::I16(2)])], Flow::Keep)
    );
    let ok = |cmd: i16| packet("hq", &[Value::I16(cmd), Value::U64(1)]);
    let fail =
        |cmd: i16, code: i16| packet("hqE", &[Value::I16(cmd), Value::U64(1), Value::I16(code)]);
    assert_eq!(
        manage(&mut d, black(7, 1, 2, 1), now).0,
        vec![ok(manage_cmd::INSERT_BLACK_LIST_OK)]
    );
    assert!(d.is_black_listed(1, 2));
    assert_eq!(manage(&mut d, black(7, 1, 2, 1), now).0, vec![fail(9, 8)]);
    assert_eq!(manage(&mut d, black(7, 1, 2, 3), now).0, vec![fail(9, 1)]);
    assert_eq!(manage(&mut d, black(4, 1, 2, 1), now).0, vec![ok(5)]);
    assert_eq!(manage(&mut d, black(4, 1, 2, 1), now).0, vec![fail(6, 7)]);
    // No active season: the DB insert stays, the reply fails with 2.
    assert_eq!(manage(&mut d, black(7, 1, 3, 1), 0).0, vec![fail(9, 2)]);
    assert!(d.is_black_listed(1, 3));
    let (_, flow) = manage(&mut d, packet("h", &[Value::I16(10)]), now);
    assert_eq!(flow, Flow::Close);
}

#[test]
fn decode_limits_reject_long_strings() {
    let long: Vec<u16> = vec![b'a' as u16; 401];
    let p = packet("hs", &[Value::I16(1), Value::String(long)]);
    assert!(fields("hs", &p).is_none());
}

fn season_xml(extra: &str, season_type: &str, start: &str, end: &str) -> String {
    format!(
        r#"<table><record season-contents-type="duel-death-match-1vs1" season-contents-schedule-number="5" season-type="{season_type}" service-mode="live-rc" player-type="pc" season-start-time="{start}" season-end-time="{end}" season-reward-expiration-time="2020-06-10 15:00:00" closing-notice-period="5" accumulation-period="10" {extra}/></table>"#
    )
}

fn schedules(xml: &str, offset: i64) -> Result<Vec<ResultSchedule>, String> {
    let mut s = parse_season_data_at(xml, offset).map_err(|e| e.0)?;
    load_result_schedules(&mut s, 6, offset).map_err(|e| e.0)?;
    Ok(s.remove(0).results)
}

#[test]
fn weekly_normal_season_has_one_result_per_week() {
    // 2020-04-15 is a wednesday; same day and hour jumps one full week.
    let xml = season_xml(
        r#"enable-weekly-ending-schedule="y" weekly-ending-day="wed" weekly-ending-hour="10""#,
        "normal",
        "2020-04-15 10:00:00",
        "2020-06-03 10:00:00",
    );
    let r = schedules(&xml, 0).unwrap();
    let weeks: Vec<i64> = r.iter().map(|x| x.reward_start_time).collect();
    let mut want: Vec<i64> = (0..6)
        .map(|w| t("2020-04-22 10:00:00") + w * 604_800)
        .collect();
    want.push(t("2020-06-03 10:00:00"));
    assert_eq!(weeks, want);
    assert_eq!(r[0].ranking_result_start_time, t("2020-04-15 10:00:00"));
    assert_eq!(r[1].ranking_result_start_time, want[0]);
    assert_eq!(r[0].closing_noti_time, want[0] - 15 * 60);
    assert_eq!(r[0].determine_time, want[0] - 10 * 60);
    assert_eq!(r[0].result_expiration_time, want[1] - 10 * 60);
    assert_eq!(r[6].result_expiration_time, t("2020-06-10 15:00:00"));
    assert_eq!(
        r.iter().map(|x| x.is_finality_result).collect::<Vec<_>>(),
        [false, false, false, false, false, false, true]
    );
    assert_eq!(
        r.iter().map(|x| x.number).collect::<Vec<_>>(),
        [1, 2, 3, 4, 5, 6, 7]
    );
}

#[test]
fn weekly_ending_day_before_and_after_start_day() {
    // Start friday 2020-04-17 12:00, ending wednesday 10:00: wday > day.
    let xml = season_xml(
        r#"enable-weekly-ending-schedule="y" weekly-ending-day="wed" weekly-ending-hour="10""#,
        "normal",
        "2020-04-17 12:00:00",
        "2020-05-06 10:00:00",
    );
    let r = schedules(&xml, 0).unwrap();
    assert_eq!(r[0].reward_start_time, t("2020-04-22 10:00:00"));
    assert_eq!(r.len(), 3);
    // Start monday 2020-04-13, ending wednesday 10:00: wday < day.
    let xml = season_xml(
        r#"enable-weekly-ending-schedule="y" weekly-ending-day="wed" weekly-ending-hour="10""#,
        "normal",
        "2020-04-13 9:00:00",
        "2020-04-29 10:00:00",
    );
    let r = schedules(&xml, 0).unwrap();
    assert_eq!(r[0].reward_start_time, t("2020-04-15 10:00:00"));
    // wday < day and the first ending is after the end: the load fails.
    let xml = season_xml(
        r#"enable-weekly-ending-schedule="y" weekly-ending-day="wed" weekly-ending-hour="10""#,
        "normal",
        "2020-04-13 9:00:00",
        "2020-04-14 9:00:00",
    );
    assert!(schedules(&xml, 0).is_err());
}

#[test]
fn weekly_ending_uses_local_time() {
    // 2020-04-15 01:00 KST is tuesday 16:00 UTC. With +540 minutes the
    // local day is wednesday, so the first ending is one week later.
    let xml = season_xml(
        r#"enable-weekly-ending-schedule="y" weekly-ending-day="wed" weekly-ending-hour="1""#,
        "normal",
        "2020-04-15 1:00:00",
        "2020-05-01 0:00:00",
    );
    let kst = 540 * 60;
    let r = schedules(&xml, kst).unwrap();
    assert_eq!(r[0].reward_start_time, t("2020-04-22 1:00:00") - kst);
    let utc = schedules(&xml, 0).unwrap();
    assert_eq!(utc[0].reward_start_time, t("2020-04-22 1:00:00"));
}

#[test]
fn free_and_one_week_seasons_and_qa_list() {
    let xml = season_xml("", "free", "2020-04-15 10:00:00", "2020-04-22 10:00:00");
    let r = schedules(&xml, 0).unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].reward_start_time, t("2020-04-22 10:00:00"));
    assert_eq!(r[0].result_expiration_time, t("2020-04-22 11:00:00"));
    assert!(r[0].is_finality_result);
    let xml = season_xml("", "normal", "2020-04-15 10:00:00", "2020-04-22 10:00:00");
    let r = schedules(&xml, 0).unwrap();
    assert_eq!(r[0].result_expiration_time, t("2020-06-10 15:00:00"));
    assert!(schedules(
        &season_xml("", "none", "2020-04-15 10:00:00", "2020-04-22 10:00:00"),
        0
    )
    .is_err());
    // QA list replaces the derived list; it stops at the first gap.
    let xml = season_xml(
        r#"enable-weekly-ending-schedule-for-qa="y" weekly-ending-schedule-for-qa-1="2020-04-16 10:00:00" weekly-ending-schedule-for-qa-2="2020-04-17 10:00:00" weekly-ending-schedule-for-qa-4="2020-04-19 10:00:00""#,
        "normal",
        "2020-04-15 10:00:00",
        "2020-04-22 10:00:00",
    );
    let r = schedules(&xml, 0).unwrap();
    assert_eq!(r.len(), 2);
    assert_eq!(r[1].ranking_result_start_time, t("2020-04-16 10:00:00"));
    assert_eq!(r[0].result_expiration_time, t("2020-04-17 10:00:00") - 600);
}

#[test]
fn forced_termination_seasons_are_not_loaded() {
    let config = RankingConfig::parse(CONFIG).unwrap();
    let xml = season_xml(
        r#"forced-termination="y""#,
        "free",
        "2020-04-15 10:00:00",
        "2020-04-22 10:00:00",
    );
    let mut s = parse_season_data(&xml).unwrap();
    assert!(s[0].forced_termination);
    load_result_schedules(&mut s, 6, 0).unwrap();
    let d = Daemon::new(&config, &[], &[], s);
    assert!(d.seasons().is_empty());
}
