//! Config files (spec 01-config.md): smartcoupond.config.xml,
//! game-event.config.xml and gamed-list.config.xml.
use quick_xml::{events::Event, Reader};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::net::Ipv4Addr;

const MAX_CONFIG_BYTES: usize = 1 << 20;
const MAX_EVENT_NAME: usize = 260;

/// SmartCouponDaemonTypeDesc @0x1400835F8: "none", "gamed".
pub const DAEMON_TYPE_DESC: [&str; 2] = ["none", "gamed"];
pub const DAEMON_TYPE_GAMED: i8 = 1;

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
    if xml.len() > MAX_CONFIG_BYTES {
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

/// smartcoupond.config.xml: `<config module='smartcoupond'><option name value/>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonConfig {
    pub max_world: i32,
    pub np_enabled: bool,
    pub np_address: String,
    pub np_port: u16,
    pub manage_listen_port: u16,
    pub daemon_listen_port: u16,
    pub service_world_id: i32,
    pub np_reward_changeover_enable: bool,
    /// Every option, including the ones this port does not use.
    pub options: BTreeMap<String, String>,
}

impl Default for DaemonConfig {
    /// Values from the shipped 348 sample file.
    fn default() -> Self {
        Self {
            max_world: 100,
            np_enabled: true,
            np_address: "10.10.110.55".into(),
            np_port: 6601,
            manage_listen_port: 32000,
            daemon_listen_port: 22000,
            service_world_id: 1,
            np_reward_changeover_enable: true,
            options: BTreeMap::new(),
        }
    }
}

fn yes(value: &str) -> bool {
    value.eq_ignore_ascii_case("yes") || value.eq_ignore_ascii_case("true") || value == "1"
}

impl DaemonConfig {
    pub fn parse(xml: &str) -> Result<Self, ConfigError> {
        let root = parse(xml)?;
        if root.name != "config" {
            return Err(error("root element must be config"));
        }
        if let Some(module) = root.attrs.get("module") {
            if module != "smartcoupond" {
                return Err(error("wrong config module"));
            }
        }
        let mut config = Self::default();
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
            if config.options.insert(name.clone(), value.clone()).is_some() {
                return Err(error("duplicate option"));
            }
        }
        let number = |name: &str| -> Result<Option<i64>, ConfigError> {
            config
                .options
                .get(name)
                .map(|v| {
                    v.trim()
                        .parse::<i64>()
                        .map_err(|_| error(format!("bad {name}")))
                })
                .transpose()
        };
        let port = |n: Option<i64>, default: u16, name: &str| -> Result<u16, ConfigError> {
            match n {
                None => Ok(default),
                Some(n) => u16::try_from(n).map_err(|_| error(format!("bad {name}"))),
            }
        };
        let max_world = number("max-world")?.map_or(config.max_world, |n| n as i32);
        let np_port = port(number("np-port")?, config.np_port, "np-port")?;
        let manage = port(
            number("manage-listen-port")?,
            config.manage_listen_port,
            "manage-listen-port",
        )?;
        let daemon = port(
            number("daemon-listen-port")?,
            config.daemon_listen_port,
            "daemon-listen-port",
        )?;
        let world = number("service-world-id")?.map_or(config.service_world_id, |n| n as i32);
        let np_enabled = config
            .options
            .get("np-enabled")
            .map_or(config.np_enabled, |v| yes(v));
        let changeover = config
            .options
            .get("np-reward-changeover-enable")
            .map_or(config.np_reward_changeover_enable, |v| yes(v));
        if let Some(address) = config.options.get("np-address") {
            config.np_address = address.clone();
        }
        config.max_world = max_world;
        config.np_port = np_port;
        config.manage_listen_port = manage;
        config.daemon_listen_port = daemon;
        config.service_world_id = world;
        config.np_enabled = np_enabled;
        config.np_reward_changeover_enable = changeover;
        Ok(config)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameEventInfo {
    pub id: i32,
    pub name: String,
    /// 1 account, 2 character.
    pub event_type: i32,
    pub daemon_type: i8,
    pub enable: bool,
}

/// game-event.config.xml (GameEventConfig::SetupGameEventListFromXml @0x140002C80).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GameEventConfig {
    pub events: HashMap<i32, GameEventInfo>,
}

fn daemon_type(name: &str) -> i8 {
    DAEMON_TYPE_DESC
        .iter()
        .position(|d| d.eq_ignore_ascii_case(name))
        .unwrap_or(0) as i8
}

impl GameEventConfig {
    pub fn parse(xml: &str) -> Result<Self, ConfigError> {
        let root = parse(xml)?;
        let mut config = Self::default();
        for group in &root.children {
            let kind = daemon_type(&group.name);
            if kind == 0 {
                continue;
            }
            for event in &group.children {
                let get = |key: &str| {
                    event
                        .attrs
                        .get(key)
                        .filter(|v| !v.is_empty())
                        .ok_or_else(|| error(format!("event needs {key}")))
                };
                let id = get("id")?;
                let name = get("name")?;
                let event_type = get("type")?;
                let enable = get("enable")?;
                if name.encode_utf16().count() >= MAX_EVENT_NAME {
                    return Err(error("event name too long"));
                }
                let info = GameEventInfo {
                    id: id.trim().parse().map_err(|_| error("bad event id"))?,
                    name: name.clone(),
                    event_type: event_type
                        .trim()
                        .parse()
                        .map_err(|_| error("bad event type"))?,
                    daemon_type: kind,
                    enable: enable.eq_ignore_ascii_case("true"),
                };
                config.events.insert(info.id, info);
            }
        }
        Ok(config)
    }
    pub fn get(&self, id: i32) -> Option<&GameEventInfo> {
        self.events.get(&id)
    }
}

/// GameEventDaemonId: type byte and world id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DaemonId {
    pub kind: i8,
    pub wid: i16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonInfo {
    pub id: DaemonId,
    pub ip: Ipv4Addr,
    pub port: u16,
}

/// gamed-list.config.xml (DChannelMgr::SetupDaemonListFromXml @0x14000CAC0).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GamedList {
    pub daemons: HashMap<DaemonId, DaemonInfo>,
}

impl GamedList {
    /// Unknown elements and `gamed` entries with an empty attribute are skipped.
    pub fn parse(xml: &str) -> Result<Self, ConfigError> {
        let root = parse(xml)?;
        let mut list = Self::default();
        for child in &root.children {
            if !child.name.eq_ignore_ascii_case("gamed") {
                continue;
            }
            let attr = |k: &str| child.attrs.get(k).filter(|v| !v.is_empty());
            let (Some(wid), Some(ip), Some(port)) =
                (attr("wid"), attr("ip-address"), attr("internal-port"))
            else {
                continue;
            };
            let id = DaemonId {
                kind: daemon_type(&child.name),
                wid: wid.trim().parse().map_err(|_| error("bad wid"))?,
            };
            let ip: Ipv4Addr = ip.trim().parse().map_err(|_| error("bad ip-address"))?;
            let port: u16 = port
                .trim()
                .parse()
                .map_err(|_| error("bad internal-port"))?;
            list.daemons.insert(id, DaemonInfo { id, ip, port });
        }
        Ok(list)
    }
    pub fn get(&self, id: &DaemonId) -> Option<&DaemonInfo> {
        self.daemons.get(id)
    }
}
