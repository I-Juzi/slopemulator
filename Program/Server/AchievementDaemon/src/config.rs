//! Config files (spec README.md): achievementd.config.xml and
//! achievementd-iplist.xml. The XML tree reader is shared with rules.rs.
use quick_xml::{events::Event, Reader};
use std::collections::BTreeMap;
use std::fmt;
use std::net::Ipv4Addr;

const MAX_CONFIG_BYTES: usize = 1 << 20;

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

/// Same as [`parse`] with a caller-chosen size limit (rule tables are large).
pub fn parse_limited(xml: &str, limit: usize) -> Result<Element, ConfigError> {
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

fn yes(value: &str) -> bool {
    value.eq_ignore_ascii_case("yes") || value.eq_ignore_ascii_case("true") || value == "1"
}

/// achievementd.config.xml: `<config module='achievement-daemon'><option name value/>`.
/// Defaults are the values the 348 binary uses when an option is missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonConfig {
    pub internal_port: u16,
    pub manage_port: u16,
    pub ping_enable: bool,
    pub check_ping_time_ms: u64,
    pub disconnect_when_ping_timeout: bool,
    pub max_pc_count: usize,
    pub update_cycle_ms: u64,
    pub log_register_change: bool,
    pub data_path: String,
    /// Every option, including the ones this port does not use.
    pub options: BTreeMap<String, String>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            internal_port: 22200,
            manage_port: 32200,
            ping_enable: true,
            check_ping_time_ms: 30000,
            disconnect_when_ping_timeout: true,
            max_pc_count: 400_000,
            update_cycle_ms: 3000,
            log_register_change: false,
            data_path: "data".into(),
            options: BTreeMap::new(),
        }
    }
}

impl DaemonConfig {
    pub fn parse(xml: &str) -> Result<Self, ConfigError> {
        let root = parse(xml)?;
        if root.name != "config" {
            return Err(error("root element must be config"));
        }
        if let Some(module) = root.attrs.get("module") {
            if module != "achievement-daemon" {
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
        let options = config.options.clone();
        let number = |name: &str| -> Result<Option<u64>, ConfigError> {
            options
                .get(name)
                .map(|v| {
                    v.trim()
                        .parse::<u64>()
                        .map_err(|_| error(format!("bad {name}")))
                })
                .transpose()
        };
        let port = |name: &str, default: u16| -> Result<u16, ConfigError> {
            match number(name)? {
                None => Ok(default),
                Some(n) => u16::try_from(n).map_err(|_| error(format!("bad {name}"))),
            }
        };
        config.internal_port = port("internal-port", config.internal_port)?;
        config.manage_port = port("manage-port", config.manage_port)?;
        if let Some(n) = number("check-ping-time")? {
            config.check_ping_time_ms = n;
        }
        if let Some(n) = number("max-pc-count")? {
            config.max_pc_count = n as usize;
        }
        if let Some(n) = number("update-cycle")? {
            if n == 0 {
                return Err(error("bad update-cycle"));
            }
            config.update_cycle_ms = n;
        }
        if let Some(v) = options.get("ping-enable") {
            config.ping_enable = yes(v);
        }
        if let Some(v) = options.get("disconnect-when-ping-timeout") {
            config.disconnect_when_ping_timeout = yes(v);
        }
        if let Some(v) = options.get("log-register-change") {
            config.log_register_change = yes(v);
        }
        if let Some(v) = options.get("data-path") {
            config.data_path = v.clone();
        }
        Ok(config)
    }
}

/// achievementd-iplist.xml: `<cluster-list><cluster ip='a.b.c.d' cluster-id='n'/>`.
/// A Hello is accepted only when its (remote IP, cluster id) pair is listed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IpList {
    pub clusters: Vec<(Ipv4Addr, i16)>,
}

impl IpList {
    pub fn parse(xml: &str) -> Result<Self, ConfigError> {
        let root = parse(xml)?;
        if root.name != "cluster-list" {
            return Err(error("root element must be cluster-list"));
        }
        let mut clusters = Vec::new();
        for child in &root.children {
            if child.name != "cluster" {
                return Err(error("unexpected element in cluster-list"));
            }
            let ip = child
                .attrs
                .get("ip")
                .and_then(|v| v.trim().parse::<Ipv4Addr>().ok())
                .ok_or_else(|| error("cluster has no valid ip"))?;
            let id = child
                .attrs
                .get("cluster-id")
                .and_then(|v| v.trim().parse::<i16>().ok())
                .ok_or_else(|| error("cluster has no valid cluster-id"))?;
            clusters.push((ip, id));
        }
        Ok(Self { clusters })
    }

    pub fn allows(&self, ip: Ipv4Addr, cluster_id: i16) -> bool {
        self.clusters
            .iter()
            .any(|&(i, c)| i == ip && c == cluster_id)
    }
}
