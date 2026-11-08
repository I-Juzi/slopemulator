//! Config and data files (spec README.md section 2): rankingd.config.xml,
//! game-list.config.xml, infogate-list.config.xml and
//! SeasonContentsScheduleData.xml.
use quick_xml::{events::Event, Reader};
use std::collections::BTreeMap;
use std::fmt;
use std::net::Ipv4Addr;

const MAX_CONFIG_BYTES: usize = 1 << 20;
/// Data tables are larger (the 2020 season table is 1.9 MB).
const MAX_DATA_BYTES: usize = 64 << 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError(pub String);
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "config: {}", self.0)
    }
}
impl std::error::Error for ConfigError {}
fn error(message: impl Into<String>) -> ConfigError {
    ConfigError(message.into())
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Element {
    pub name: String,
    pub attrs: BTreeMap<String, String>,
    pub children: Vec<Element>,
}

/// Parse a small XML document into an element tree. DTDs, CDATA and
/// non-blank text are refused.
pub fn parse(xml: &str) -> Result<Element, ConfigError> {
    parse_limited(xml, MAX_CONFIG_BYTES)
}

fn parse_limited(xml: &str, limit: usize) -> Result<Element, ConfigError> {
    if xml.len() > limit {
        return Err(error("XML exceeds size limit"));
    }
    let mut reader = Reader::from_str(xml);
    let mut stack: Vec<Element> = Vec::new();
    let mut root: Option<Element> = None;
    loop {
        let event = reader.read_event().map_err(|e| error(e.to_string()))?;
        let empty = matches!(event, Event::Empty(_));
        match event {
            Event::Start(e) | Event::Empty(e) => {
                if root.is_some() {
                    return Err(error("content after root element"));
                }
                let name = String::from_utf8(e.name().as_ref().to_vec())
                    .map_err(|_| error("invalid element name"))?;
                let mut attrs = BTreeMap::new();
                for a in e.attributes() {
                    let a = a.map_err(|e| error(e.to_string()))?;
                    let key = String::from_utf8(a.key.as_ref().to_vec())
                        .map_err(|_| error("invalid attribute"))?;
                    let value = a
                        .decode_and_unescape_value(reader.decoder())
                        .map_err(|e| error(e.to_string()))?
                        .into_owned();
                    if attrs.insert(key, value).is_some() {
                        return Err(error("duplicate attribute"));
                    }
                }
                let element = Element {
                    name,
                    attrs,
                    children: Vec::new(),
                };
                if empty {
                    match stack.last_mut() {
                        Some(parent) => parent.children.push(element),
                        None => root = Some(element),
                    }
                } else {
                    stack.push(element);
                }
            }
            Event::End(_) => {
                let element = stack.pop().ok_or_else(|| error("unexpected closing tag"))?;
                match stack.last_mut() {
                    Some(parent) => parent.children.push(element),
                    None => root = Some(element),
                }
            }
            Event::Text(t) if t.as_ref().iter().all(u8::is_ascii_whitespace) => (),
            Event::Decl(_) | Event::Comment(_) => (),
            Event::Eof => break,
            _ => return Err(error("unsupported XML content")),
        }
    }
    if !stack.is_empty() {
        return Err(error("unclosed element"));
    }
    root.ok_or_else(|| error("no root element"))
}

pub const MODULE: &str = "rankingd";
pub const CONFIG_FILE: &str = "rankingd.config.xml";
pub const GAME_LIST_FILE: &str = "game-list.config.xml";
pub const INFOGATE_LIST_FILE: &str = "infogate-list.config.xml";
pub const SEASON_DATA_FILE: &str = "SeasonContentsScheduleData.xml";

/// `Data::SEASONCONTENTSSCHEDULE_SERVICE_MODE_NAMES` @0x1400E14E0.
pub const SERVICE_MODE_NAMES: [&str; 12] = [
    "none",
    "program",
    "internal",
    "qa-1-reserve",
    "qa-2-nextlive",
    "qa-3-live",
    "live-rc",
    "live-test",
    "live-1",
    "live-2",
    "addition-1",
    "addition-2",
];
/// `Data::SEASONCONTENTS_SEASON_CONTENTS_TYPE_NAMES` @0x1400E15C0 (0..16).
pub const SEASON_CONTENTS_TYPE_NAMES: [&str; 16] = [
    "invalid",
    "duel-death-match-1vs1",
    "duel-death-match-3vs3",
    "duel-tag-match-3vs3",
    "party-battle-occupation-war",
    "solo-dungeon-1-best-record",
    "infinite-tower-best-record",
    "party-battle-capture-the-flag",
    "guild-combat",
    "solo-dungeon-2-best-record",
    "ingame-championship-duel-death-match-1vs1",
    "ingame-championship-duel-tag-match-3vs3",
    "party-battle-lead-the-ball",
    "challenge-party-best-record",
    "battle-royal-free-for-all",
    "battle-royal-free-for-all-party",
];
/// `SEASONCONTENTSSCHEDULE_SEASON_TYPE_NAMES` @0x1400E14C8.
pub const SEASON_TYPE_NAMES: [&str; 3] = ["none", "free", "normal"];
/// `SEASONCONTENTSSCHEDULE_PLAYER_TYPE_NAMES` @0x1400E1540.
pub const PLAYER_TYPE_NAMES: [&str; 4] = ["none", "pc", "guild", "challenge-party"];

fn name_index(names: &[&str], value: &str) -> Option<usize> {
    names.iter().position(|n| *n == value)
}

/// `RankingConfig::_Def` @0x1400E2410 (module `rankingd`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankingConfig {
    pub root_path: String,
    pub data_path: String,
    pub log_path: String,
    pub config_path: String,
    pub transaction_log_path: String,
    pub cert_file: String,
    pub worker_thread: i32,
    pub max_world: i32,
    pub dump_type: i32,
    pub timer_interval: i32,
    pub timer_max_time: i32,
    pub service_mode: i32,
    pub world_listen_port: u16,
    pub infogate_listen_port: u16,
    pub manage_listen_port: u16,
    pub service_world_id: i32,
    pub max_ranking_list_count_per_contents: i32,
    pub max_ranking_count: i32,
    pub max_search_count_one_time: i32,
    pub pc_list_hash_count: i32,
    pub ranking_list_update_period: i32,
    pub accumulate_season_interim_score_period: i32,
    pub skip_list_ex_max_level: i32,
    pub skip_list_ex_prob: i32,
    pub skip_list_ex_max_node_count: i32,
    /// Every option, including the ones this port does not use.
    pub options: BTreeMap<String, String>,
}

fn options_of(root: &Element, module: &str) -> Result<BTreeMap<String, String>, ConfigError> {
    if root.name != "config" {
        return Err(error("root element must be config"));
    }
    if root.attrs.get("module").is_some_and(|m| m != module) {
        return Err(error("wrong config module"));
    }
    let mut options = BTreeMap::new();
    for child in &root.children {
        if child.name != "option" {
            return Err(error("unexpected element in config"));
        }
        let name = child
            .attrs
            .get("name")
            .ok_or_else(|| error("option has no name"))?;
        let value = child
            .attrs
            .get("value")
            .ok_or_else(|| error("option has no value"))?;
        if options.insert(name.clone(), value.clone()).is_some() {
            return Err(error("duplicate option"));
        }
    }
    Ok(options)
}

impl RankingConfig {
    pub fn parse(xml: &str) -> Result<Self, ConfigError> {
        let options = options_of(&parse(xml)?, MODULE)?;
        let text = |name: &str| {
            options
                .get(name)
                .cloned()
                .ok_or_else(|| error(format!("missing {name}")))
        };
        let int = |name: &str, default: Option<i32>| -> Result<i32, ConfigError> {
            match (options.get(name), default) {
                (Some(v), _) => v.trim().parse().map_err(|_| error(format!("bad {name}"))),
                (None, Some(d)) => Ok(d),
                (None, None) => Err(error(format!("missing {name}"))),
            }
        };
        let port = |name: &str, default: i32| -> Result<u16, ConfigError> {
            u16::try_from(int(name, Some(default))?).map_err(|_| error(format!("bad {name}")))
        };
        let service_mode = int("service-mode", Some(1))?;
        if !(1..SERVICE_MODE_NAMES.len() as i32).contains(&service_mode) {
            return Err(error("bad service-mode"));
        }
        Ok(Self {
            root_path: text("root-path")?,
            data_path: text("data-path")?,
            log_path: text("log-path")?,
            config_path: text("config-path")?,
            transaction_log_path: text("transaction-log-path")?,
            cert_file: text("cert-file")?,
            worker_thread: int("worker-thread", Some(0))?,
            max_world: int("max-world", Some(100))?,
            dump_type: int("dump-type", Some(1))?,
            timer_interval: int("timer-interval", Some(1000))?,
            timer_max_time: int("timer-max-time", Some(30))?,
            service_mode,
            world_listen_port: port("world-listen-port", 12510)?,
            infogate_listen_port: port("infogate-listen-port", 12520)?,
            manage_listen_port: port("manage-listen-port", 32510)?,
            service_world_id: int("service-world-id", None)?,
            max_ranking_list_count_per_contents: int(
                "max-ranking-list-count-per-contents",
                Some(10),
            )?,
            max_ranking_count: int("max-ranking-count", Some(100_000))?,
            max_search_count_one_time: int("max-search-count-one-time", Some(1000))?,
            pc_list_hash_count: int("pc-list-hash-count", None)?,
            ranking_list_update_period: int("ranking-list-update-period", None)?,
            accumulate_season_interim_score_period: int(
                "accumulate-season-interim-score-period",
                Some(600),
            )?,
            skip_list_ex_max_level: int("skip-list-ex-max-level", Some(15))?,
            skip_list_ex_prob: int("skip-list-ex-prob", Some(36))?,
            skip_list_ex_max_node_count: int("skip-list-ex-max-node-count", Some(100_000))?,
            options,
        })
    }
}

/// `_wtoi`: leading digits, 0 for bad text.
pub fn wtoi(text: &str) -> i32 {
    let t = text.trim_start();
    let (neg, digits) = match t.as_bytes().first() {
        Some(b'-') => (true, &t[1..]),
        Some(b'+') => (false, &t[1..]),
        _ => (false, t),
    };
    let mut n: i64 = 0;
    for b in digits.bytes().take_while(u8::is_ascii_digit) {
        n = (n * 10 + i64::from(b - b'0')).min(i64::from(i32::MAX) + 1);
    }
    let n = if neg { -n } else { n };
    n.clamp(i32::MIN.into(), i32::MAX.into()) as i32
}

fn ip_of(text: Option<&String>) -> Ipv4Addr {
    text.and_then(|t| t.trim().parse().ok())
        .unwrap_or(Ipv4Addr::BROADCAST)
}

/// Service type of a game-list row: 1 game (cluster), 2 arena.
pub const SERVICE_GAME: i16 = 1;
pub const SERVICE_ARENA: i16 = 2;

/// One `game` row (GameChannelMgr::SetupGameListFromXml @0x140008AE0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameInfo {
    pub service_type: i16,
    pub id: i16,
    pub ip: Ipv4Addr,
    pub port: u16,
}

/// One `infogate` row (InfoGateChannelMgr::SetupInfoGateListFromXml @0x14000C3B0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InfoGateInfo {
    pub wid: i16,
    pub ip: Ipv4Addr,
    pub port: u16,
}

fn list_rows<'a>(xml: &'a Element, child: &str) -> Result<&'a [Element], ConfigError> {
    if let Some(bad) = xml.children.iter().find(|c| c.name != child) {
        return Err(error(format!("unexpected element {}", bad.name)));
    }
    Ok(&xml.children)
}

pub fn parse_game_list(xml: &str) -> Result<Vec<GameInfo>, ConfigError> {
    let root = parse(xml)?;
    list_rows(&root, "game")?
        .iter()
        .map(|row| {
            let n = |k: &str| row.attrs.get(k).map_or(0, |v| wtoi(v));
            let service_type = n("service-type") as i16;
            if service_type != SERVICE_GAME && service_type != SERVICE_ARENA {
                return Err(error("bad service-type"));
            }
            Ok(GameInfo {
                service_type,
                id: n("id") as i16,
                ip: ip_of(row.attrs.get("ip-address")),
                port: n("internal-port") as u16,
            })
        })
        .collect()
}

pub fn parse_infogate_list(xml: &str) -> Result<Vec<InfoGateInfo>, ConfigError> {
    let root = parse(xml)?;
    Ok(list_rows(&root, "infogate")?
        .iter()
        .map(|row| {
            let n = |k: &str| row.attrs.get(k).map_or(0, |v| wtoi(v));
            InfoGateInfo {
                wid: n("wid") as i16,
                ip: ip_of(row.attrs.get("ip-address")),
                port: n("internal-port") as u16,
            }
        })
        .collect())
}

/// One season result schedule (RankingSeasonResultScheduleInfo, 64 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultSchedule {
    pub number: u8,
    pub ranking_result_start_time: i64,
    pub closing_noti_time: i64,
    /// Not sent in FetchSeasonList; kept for the DB and the accumulate task.
    pub determine_time: i64,
    pub reward_start_time: i64,
    pub result_expiration_time: i64,
    pub is_finality_result: bool,
}

/// One SeasonContentsScheduleData record, converted to RankingSeason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Season {
    pub service_mode: i32,
    pub season_contents_type: u8,
    pub schedule_number: i32,
    pub name: String,
    pub season_type: u8,
    pub player_type: i16,
    pub start_time: i64,
    pub end_time: i64,
    pub result_expiration_time: i64,
    pub reward_group_id: i16,
    pub default_rating_score: i32,
    pub ranking_minimum_score: i64,
    /// closing-notice-period (record +64), minutes.
    pub closing_notice_period: i16,
    /// accumulation-period (record +66), minutes.
    pub accumulation_period: i16,
    /// enable-weekly-ending-schedule (+68), weekly-ending-day (+69, sun = 0),
    /// weekly-ending-hour (+70).
    pub weekly_ending: bool,
    pub weekly_ending_day: u8,
    pub weekly_ending_hour: u8,
    /// enable-weekly-ending-schedule-for-qa (+80) and
    /// weekly-ending-schedule-for-qa-1..20 (+88).
    pub weekly_ending_for_qa: bool,
    pub qa_schedule: Vec<i64>,
    /// forced-termination (+249).
    pub forced_termination: bool,
    pub results: Vec<ResultSchedule>,
}

/// SEASONCONTENTSSCHEDULE weekly-ending-day names (0x1400E1560), same
/// order as `tm_wday`.
pub const WEEKDAY_NAMES: [&str; 7] = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
/// weekly-ending-schedule-for-qa-1..20.
pub const MAX_QA_SCHEDULE: usize = 20;
const WEEK: i64 = 604_800;

/// Days from 1970-01-01 to a civil date (proleptic Gregorian).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// `YYYY-MM-DD H:MM:SS` as seconds, read as UTC. Native reads local time
/// (`_localtime64_s` in loadHiddenFieldFromSeasonContentsSchedule
/// @0x140004480); [`parse_season_data_at`] subtracts the local offset.
pub fn parse_time(text: &str) -> Option<i64> {
    let (date, time) = text.trim().split_once(' ')?;
    let mut d = date.split('-').map(|p| p.parse::<i64>().ok());
    let (y, mo, da) = (d.next()??, d.next()??, d.next()??);
    let mut t = time.split(':').map(|p| p.parse::<i64>().ok());
    let (h, mi, s) = (t.next()??, t.next()??, t.next()??);
    if d.next().is_some() || t.next().is_some() {
        return None;
    }
    if !(1..=12).contains(&mo) || !(1..=31).contains(&da) || h > 23 || mi > 59 || s > 59 {
        return None;
    }
    Some(days_from_civil(y, mo, da) * 86_400 + h * 3600 + mi * 60 + s)
}

/// Parse SeasonContentsScheduleData.xml (`table` of `record` rows).
/// Result schedules come from [`load_result_schedules`]. The exporter leaves out attributes that hold
/// their default, so a missing number or time is 0 and a missing name is index 0 ❓.
/// A time that is present but does not parse fails the load.
/// A record with a name the 348 enums do not know (newer data, for example
/// `mastery-exp-best-record`) is skipped ❓ (native DrDataReader path not
/// traced).
pub fn parse_season_data(xml: &str) -> Result<Vec<Season>, ConfigError> {
    parse_season_data_at(xml, 0)
}

/// [`parse_season_data`] with the server local time offset (seconds east
/// of UTC). Every time becomes `wall clock - utc_offset`.
pub fn parse_season_data_at(xml: &str, utc_offset: i64) -> Result<Vec<Season>, ConfigError> {
    let root = parse_limited(xml, MAX_DATA_BYTES)?;
    if root.name != "table" {
        return Err(error("root element must be table"));
    }
    let mut seasons = Vec::new();
    for row in list_rows(&root, "record")? {
        let get = |k: &str| row.attrs.get(k).map(String::as_str);
        let named = |k: &str, names: &[&str]| match get(k) {
            None => Some(0),
            Some(v) => name_index(names, v),
        };
        let time = |k: &str| match get(k) {
            None => Ok(0),
            Some(v) => parse_time(v)
                .map(|t| t - utc_offset)
                .ok_or_else(|| error(format!("bad {k}"))),
        };
        let flag = |k: &str| get(k).is_some_and(|v| v.trim().eq_ignore_ascii_case("y"));
        let num = |k: &str| -> Result<i64, ConfigError> {
            match get(k) {
                None => Ok(0),
                Some(v) => v.trim().parse().map_err(|_| error(format!("bad {k}"))),
            }
        };
        let (Some(service_mode), Some(contents), Some(season_type), Some(player_type), Some(day)) = (
            named("service-mode", &SERVICE_MODE_NAMES),
            named("season-contents-type", &SEASON_CONTENTS_TYPE_NAMES),
            named("season-type", &SEASON_TYPE_NAMES),
            named("player-type", &PLAYER_TYPE_NAMES),
            named("weekly-ending-day", &WEEKDAY_NAMES),
        ) else {
            continue;
        };
        // The QA list count (+248) is the number of filled entries from
        // qa-1 up ❓ (count field set by the generic reader, not traced).
        let mut qa_schedule = Vec::new();
        for n in 1..=MAX_QA_SCHEDULE {
            if get(&format!("weekly-ending-schedule-for-qa-{n}")).is_none() {
                break;
            }
            qa_schedule.push(time(&format!("weekly-ending-schedule-for-qa-{n}"))?);
        }
        seasons.push(Season {
            service_mode: service_mode as i32,
            season_contents_type: contents as u8,
            schedule_number: num("season-contents-schedule-number")? as i32,
            name: get("season-name").unwrap_or_default().to_owned(),
            season_type: season_type as u8,
            player_type: player_type as i16,
            start_time: time("season-start-time")?,
            end_time: time("season-end-time")?,
            result_expiration_time: time("season-reward-expiration-time")?,
            reward_group_id: num("reward-group-data")? as i16,
            default_rating_score: num("default-rating-score")? as i32,
            ranking_minimum_score: num("ranking-minimum-score")?,
            closing_notice_period: num("closing-notice-period")? as i16,
            accumulation_period: num("accumulation-period")? as i16,
            weekly_ending: flag("enable-weekly-ending-schedule"),
            weekly_ending_day: day as u8,
            weekly_ending_hour: num("weekly-ending-hour")? as u8,
            weekly_ending_for_qa: flag("enable-weekly-ending-schedule-for-qa"),
            qa_schedule,
            forced_termination: flag("forced-termination"),
            results: Vec::new(),
        });
    }
    Ok(seasons)
}

/// Week day (0 = sunday) and hour of an epoch time in local time.
fn local_wday_hour(time: i64, utc_offset: i64) -> (i64, i64) {
    let local = time + utc_offset;
    let days = local.div_euclid(86_400);
    ((days + 4).rem_euclid(7), local.rem_euclid(86_400) / 3600)
}

/// Data::DataManager::loadHiddenFieldFromSeasonContentsSchedule @0x140004480:
/// the reward times (record +256, count +250). A free season also moves
/// its result expiration (+56) to end + 3600.
pub fn result_times(season: &mut Season, utc_offset: i64) -> Result<Vec<i64>, ConfigError> {
    let fail = |what: &str| {
        error(format!(
            "season {}/{}: {what}",
            season.season_contents_type, season.schedule_number
        ))
    };
    match season.season_type {
        // normal
        2 if season.weekly_ending => {
            let (wday, hour) = local_wday_hour(season.start_time, utc_offset);
            let day = i64::from(season.weekly_ending_day);
            let h = i64::from(season.weekly_ending_hour);
            let first = if wday == day {
                season.start_time + 3600 * (h - hour + 168)
            } else if wday > day {
                season.start_time + 86_400 * (day - wday + 7) + 3600 * (h - hour)
            } else {
                season.start_time + 3600 * (h + 24 * (day - wday) - hour)
            };
            if wday < day && first > season.end_time {
                // DataManager.cpp line 581: validation failure.
                return Err(fail("first weekly ending is after the season end"));
            }
            let mut times = Vec::new();
            let mut t = first;
            while t < season.end_time {
                times.push(t);
                t += WEEK;
            }
            times.push(season.end_time);
            Ok(times)
        }
        2 => Ok(vec![season.end_time]),
        // free
        1 => {
            season.result_expiration_time = season.end_time + 3600;
            Ok(vec![season.end_time])
        }
        _ => Err(fail("season type is not free or normal")),
    }
}

/// convertSeasonDataToRankingSeason @0x14002A7E0 result schedule rows.
/// The QA list (+88) replaces the derived list when its flag (+80) is set.
/// Expiration: GetWeeklyResultExpirationTime @0x140007AA0.
pub fn result_schedules(season: &Season, derived: &[i64]) -> Vec<ResultSchedule> {
    let times = if season.weekly_ending_for_qa {
        &season.qa_schedule[..]
    } else {
        derived
    };
    let acc = 60 * i64::from(season.accumulation_period);
    let notice = 60 * i64::from(season.closing_notice_period);
    let n = times.len();
    times
        .iter()
        .enumerate()
        .map(|(i, &reward)| ResultSchedule {
            number: (i + 1) as u8,
            ranking_result_start_time: if i == 0 {
                season.start_time
            } else {
                times[i - 1]
            },
            closing_noti_time: reward - acc - notice,
            determine_time: reward - acc,
            reward_start_time: reward,
            result_expiration_time: if i + 1 == n {
                season.result_expiration_time
            } else {
                times[i + 1] - acc
            },
            is_finality_result: i + 1 == n,
        })
        .collect()
}

/// Derive the result schedules of every season in `service_mode`
/// (loadHiddenFieldFromSeasonContentsSchedule runs only on those). A
/// validation failure fails the load, as in native.
pub fn load_result_schedules(
    seasons: &mut [Season],
    service_mode: i32,
    utc_offset: i64,
) -> Result<(), ConfigError> {
    for season in seasons
        .iter_mut()
        .filter(|s| s.service_mode == service_mode)
    {
        let derived = result_times(season, utc_offset)?;
        season.results = result_schedules(season, &derived);
    }
    Ok(())
}
