//! Original XML configuration, spec 01 §§2–5.
use quick_xml::{events::Event, Reader};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    net::Ipv4Addr,
    path::{Path, PathBuf},
};
use thiserror::Error;

pub const MAX_CONFIG_BYTES: usize = 1024 * 1024;
#[derive(Debug, Error)]
#[error("configuration: {0}")]
pub struct ConfigError(pub String);
fn error(message: impl Into<String>) -> ConfigError {
    ConfigError(message.into())
}

pub fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, ConfigError> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| error(format!("cannot open {}: {e}", path.display())))?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| error(e.to_string()))?;
    if bytes.len() > limit {
        return Err(error("file exceeds size limit"));
    }
    Ok(bytes)
}

/// Reject DTDs, nested options, duplicate options, and unexpected XML structure.
/// Unknown option names remain valid and are ignored by the typed loader.
pub fn options(xml: &str, module: &str) -> Result<BTreeMap<String, String>, ConfigError> {
    if xml.len() > MAX_CONFIG_BYTES {
        return Err(error("XML exceeds size limit"));
    }
    let mut reader = Reader::from_str(xml);
    let mut result = BTreeMap::new();
    let mut depth = 0;
    let mut root_seen = false;
    loop {
        let event = reader.read_event().map_err(|e| error(e.to_string()))?;
        let empty = matches!(event, Event::Empty(_));
        match event {
            Event::Start(e) | Event::Empty(e) => {
                let mut attrs = BTreeMap::new();
                for a in e.attributes() {
                    let a = a.map_err(|e| error(e.to_string()))?;
                    let name = String::from_utf8(a.key.as_ref().to_vec())
                        .map_err(|_| error("invalid attribute"))?;
                    let value = a
                        .decode_and_unescape_value(reader.decoder())
                        .map_err(|e| error(e.to_string()))?
                        .into_owned();
                    if attrs.insert(name, value).is_some() {
                        return Err(error("duplicate attribute"));
                    }
                }
                if depth == 0 && !root_seen && e.name().as_ref() == b"config" {
                    if attrs.len() != 1 || attrs.get("module").map(String::as_str) != Some(module) {
                        return Err(error("wrong config module"));
                    }
                    root_seen = true;
                } else if depth == 1 && e.name().as_ref() == b"option" {
                    if attrs.len() != 2 {
                        return Err(error("option needs name and value"));
                    }
                    let name = attrs
                        .remove("name")
                        .ok_or_else(|| error("option has no name"))?;
                    let value = attrs
                        .remove("value")
                        .ok_or_else(|| error("option has no value"))?;
                    if result.insert(name, value).is_some() {
                        return Err(error("duplicate option"));
                    }
                } else {
                    return Err(error("unexpected XML element"));
                }
                if !empty {
                    depth += 1;
                }
            }
            Event::End(_) => {
                if depth == 0 {
                    return Err(error("unexpected closing tag"));
                }
                depth -= 1;
            }
            Event::Text(t) if t.as_ref().iter().all(u8::is_ascii_whitespace) => (),
            Event::Decl(_) | Event::Comment(_) => (),
            Event::Eof => break,
            _ => return Err(error("unsupported XML content")),
        }
    }
    if !root_seen || depth != 0 {
        return Err(error("incomplete config"));
    }
    Ok(result)
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GateConfig {
    pub root_path: PathBuf,
    pub log_path: PathBuf,
    pub config_path: PathBuf,
    // Accept the cert path only. Do not open or validate it.
    pub cert: String,
    pub worker_thread: i32,
    pub max_client: i32,
    pub dump_type: i32,
    pub timer_interval: i32,
    pub timer_max_time: i32,
    pub init_client_session_count: i32,
    pub client_session_hash_count: i32,
    pub predefined_world_id: i32,
    pub client_close_timeout: i32,
    pub internal_port: u16,
    pub client_port: u16,
    pub lobby_daemon_addr: Ipv4Addr,
    pub lobby_daemon_port: u16,
    pub max_idle_time_enable: bool,
    pub disconnect_after_idle_time: bool,
    pub max_idle_time: i32,
    pub use_channel_delayed_release: bool,
    pub alert_send_address: String,
    pub cooldown_complete_delay: i32,
    pub allow_client_ip_address_in_logs: bool,
}
impl GateConfig {
    pub fn worker_count(&self) -> usize {
        if self.worker_thread == 0 {
            std::thread::available_parallelism().map_or(2, |n| n.get().saturating_mul(2))
        } else {
            self.worker_thread as usize
        }
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CommonConfig {
    pub p_type: String,
    pub ranking_service_mode: i32,
    pub np_program_id: i32,
    pub np_game_code: String,
    pub np_app_id: Option<String>,
    pub np_client_app_id: Option<String>,
}
#[derive(Clone, Debug, Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ContentsConfig {
    pub dummy_data: i32,
    pub use_time_based_play_schedule: bool,
    pub enable_play_warrior: bool,
    pub enable_fish_system: bool,
    pub enable_account_contents: bool,
    pub account_contents_required_pc_level: i32,
    pub account_contents_required_pc_mastery_level: i32,
    pub account_contents_required_account_top_pc_level: i32,
    pub account_contents_required_account_top_pc_mastery_level: i32,
    pub account_exp_quota_per_day: i32,
    pub account_exp_quota_reset_time: i32,
    pub account_level_max: i32,
    pub enable_guild_join_renewal: bool,
    pub enable_job: String,
}
#[derive(Clone, Debug)]
pub struct Config {
    pub gate: GateConfig,
    pub common: CommonConfig,
    pub contents: ContentsConfig,
}

// A missing required key with no default is an error. Listed defaults apply to
// required keys too. TODO(spec 01 §4): verify the original required-key policy.
#[derive(Clone, Copy)]
enum Kind {
    Int(i32, i32),
    Bool,
    Text,
    Guid,
}
struct Rule(&'static str, Kind, Option<&'static str>);
use Kind::*;
const GATE: &[Rule] = &[
    Rule("root-path", Text, None),
    Rule("log-path", Text, None),
    Rule("config-path", Text, None),
    Rule("cert", Text, None),
    Rule("worker-thread", Int(0, 1024), Some("0")),
    Rule("max-client", Int(1, i32::MAX), Some("200000")),
    Rule("dump-type", Int(i32::MIN, i32::MAX), Some("1")),
    Rule("timer-interval", Int(0, i32::MAX), Some("1000")),
    Rule("timer-max-time", Int(0, i32::MAX), Some("20")),
    Rule("init-client-session-count", Int(0, i32::MAX), Some("0")),
    Rule("client-session-hash-count", Int(0, i32::MAX), Some("1")),
    Rule("predefined-world-id", Int(i32::MIN, i32::MAX), Some("4001")),
    Rule("client-close-timeout", Int(0, i32::MAX), None),
    Rule("internal-port", Int(0, 65535), Some("20900")),
    Rule("client-port", Int(0, 65535), Some("10900")),
    Rule("lobby-daemon-addr", Text, Some("127.0.0.1")),
    Rule("lobby-daemon-port", Int(0, 65535), Some("21801")),
    Rule("max-idle-time-enable", Bool, Some("yes")),
    Rule("disconnect-after-idle-time", Bool, Some("no")),
    Rule("max-idle-time", Int(0, i32::MAX), Some("300")),
    Rule("use-channel-delayed-release", Bool, Some("no")),
    Rule("alert-send-address", Text, Some("")),
    Rule("cooldown-complete-delay", Int(0, i32::MAX), Some("180")),
    Rule("allow-client-ip-address-in-logs", Bool, Some("yes")),
];
const COMMON: &[Rule] = &[
    Rule("p-type", Text, Some("live")),
    Rule("ranking-service-mode", Int(1, 11), Some("1")),
    Rule("np-program-id", Int(0, 10000), Some("303")),
    Rule("np-game-code", Text, Some("bnsprog")),
    Rule("np-app-id", Guid, None),
    Rule("np-client-app-id", Guid, None),
];
const CONTENTS: &[Rule] = &[
    Rule("dummy-data", Int(0, 1234), Some("123")),
    Rule("use-time-based-play-schedule", Bool, Some("no")),
    Rule("enable-play-warrior", Bool, Some("yes")),
    Rule("enable-fish-system", Bool, Some("no")),
    Rule("enable-account-contents", Bool, Some("no")),
    Rule(
        "account-contents-required-pc-level",
        Int(0, 1000),
        Some("0"),
    ),
    Rule(
        "account-contents-required-pc-mastery-level",
        Int(0, 1000),
        Some("0"),
    ),
    Rule(
        "account-contents-required-account-top-pc-level",
        Int(0, 1000),
        Some("0"),
    ),
    Rule(
        "account-contents-required-account-top-pc-mastery-level",
        Int(0, 1000),
        Some("0"),
    ),
    Rule("account-exp-quota-per-day", Int(0, i32::MAX), Some("0")),
    Rule("account-exp-quota-reset-time", Int(0, 23), Some("6")),
    Rule("account-level-max", Int(0, 1000), Some("50")),
    Rule("enable-guild-join-renewal", Bool, Some("no")),
    Rule("enable-job", Text, Some(";")),
];
fn parse<T: serde::de::DeserializeOwned>(
    xml: &str,
    module: &str,
    rules: &[Rule],
) -> Result<T, ConfigError> {
    let options = options(xml, module)?;
    let mut values = Map::new();
    for Rule(name, kind, default) in rules {
        let raw = options.get(*name).map(String::as_str).or(*default);
        let value = match (kind, raw) {
            (Guid, None) => Value::Null,
            (_, None) => return Err(error(format!("missing required key {name}"))),
            (Int(min, max), Some(s)) => {
                let n: i32 = s
                    .parse()
                    .map_err(|_| error(format!("invalid integer for {name}")))?;
                if n < *min || n > *max {
                    return Err(error(format!("out of range: {name}")));
                }
                Value::from(n)
            }
            (Bool, Some("yes")) => Value::Bool(true),
            (Bool, Some("no")) => Value::Bool(false),
            (Bool, _) => return Err(error(format!("expected yes or no for {name}"))),
            (Guid, Some(s)) => {
                if s.len() != 36
                    || !s.bytes().enumerate().all(|(i, b)| {
                        if [8, 13, 18, 23].contains(&i) {
                            b == b'-'
                        } else {
                            b.is_ascii_hexdigit()
                        }
                    })
                {
                    return Err(error(format!("invalid GUID for {name}")));
                }
                Value::String(s.into())
            }
            (Text, Some(s)) => Value::String(s.into()),
        };
        values.insert((*name).into(), value);
    }
    serde_json::from_value(Value::Object(values)).map_err(|e| error(e.to_string()))
}
impl Config {
    pub fn from_xml(gate: &str, common: &str, contents: &str) -> Result<Self, ConfigError> {
        Ok(Self {
            gate: parse(gate, "lobbygate", GATE)?,
            common: parse(common, "bns-common-config", COMMON)?,
            contents: parse(contents, "lobbygate-contents-rule-config", CONTENTS)?,
        })
    }
    pub fn load(dir: &Path) -> Result<Self, ConfigError> {
        let read = |name| {
            String::from_utf8(read_bounded(&dir.join(name), MAX_CONFIG_BYTES)?)
                .map_err(|_| error("config is not UTF-8"))
        };
        Self::from_xml(
            &read("lobbygate.config.xml")?,
            &read("bns.common.config.xml")?,
            &read("ContentsRule.lobbygate.config.xml")?,
        )
    }
}
