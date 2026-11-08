//! Config files (spec README.md section 2): cachegate.config.xml and
//! cached-list.config.xml.
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

/// CacheGateConfig::_Def @0x1400596C0 (13 entries, module `cacheGate`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateConfig {
    pub root_path: String,
    pub log_path: String,
    pub config_path: String,
    pub worker_thread: i32,
    pub dump_type: i32,
    pub max_cached_session_count: i32,
    pub max_public_session_count: i32,
    pub public_port: u16,
    pub wid: u16,
    /// Every option, including the ones this port does not use.
    pub options: BTreeMap<String, String>,
}

/// CACHEGATE_PUBLIC_PORT_VALUE @0x14004611C.
pub const DEFAULT_PUBLIC_PORT: u16 = 20800;
/// DEFAULT_DUMPTYPE @0x140046314.
pub const DEFAULT_DUMP_TYPE: i32 = 1;
pub const MODULE: &str = "cacheGate";
pub const CONFIG_FILE: &str = "cachegate.config.xml";
pub const CACHED_LIST_FILE: &str = "cached-list.config.xml";

impl GateConfig {
    /// `[REQ]` options: the three paths and both max session counts.
    pub fn parse(xml: &str) -> Result<Self, ConfigError> {
        let root = parse(xml)?;
        if root.name != "config" {
            return Err(error("root element must be config"));
        }
        if root.attrs.get("module").is_some_and(|m| m != MODULE) {
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
        let required = |name: &str| {
            options
                .get(name)
                .cloned()
                .ok_or_else(|| error(format!("missing {name}")))
        };
        let int = |name: &str, value: Option<String>, default: i64| -> Result<i64, ConfigError> {
            match value {
                None => Ok(default),
                Some(v) => v.trim().parse().map_err(|_| error(format!("bad {name}"))),
            }
        };
        let i32_of =
            |name: &str, v: i64| i32::try_from(v).map_err(|_| error(format!("bad {name}")));
        let u16_of =
            |name: &str, v: i64| u16::try_from(v).map_err(|_| error(format!("bad {name}")));
        let get = |name: &str| options.get(name).cloned();
        Ok(Self {
            root_path: required("root-path")?,
            log_path: required("log-path")?,
            config_path: required("config-path")?,
            worker_thread: i32_of(
                "worker-thread",
                int("worker-thread", get("worker-thread"), 0)?,
            )?,
            dump_type: i32_of(
                "dump-type",
                int("dump-type", get("dump-type"), DEFAULT_DUMP_TYPE.into())?,
            )?,
            max_cached_session_count: i32_of(
                "max-cached-session-count",
                int(
                    "max-cached-session-count",
                    Some(required("max-cached-session-count")?),
                    0,
                )?,
            )?,
            max_public_session_count: i32_of(
                "max-public-session-count",
                int(
                    "max-public-session-count",
                    Some(required("max-public-session-count")?),
                    0,
                )?,
            )?,
            public_port: u16_of(
                "public-port",
                int(
                    "public-port",
                    get("public-port"),
                    DEFAULT_PUBLIC_PORT.into(),
                )?,
            )?,
            wid: u16_of("wid", int("wid", get("wid"), 0)?)?,
            options,
        })
    }
}

/// One `<cached>` row (CacheDList::LoadCachedListConfigFile @0x140002B20).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CachedInfo {
    pub wid: u16,
    pub ip: Ipv4Addr,
    pub port: u16,
}

/// `_wtoi` semantics: leading blanks, optional sign, digits; else 0.
fn wtoi(text: &str) -> i32 {
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

/// Parse cached-list.config.xml. Every child of the root must be a
/// `cached` element (case-sensitive), or the load fails. `wid` and
/// `economy-port` use `_wtoi` (bad text is 0); a bad `ip-address` is
/// 255.255.255.255 (inet_addr INADDR_NONE). `internal-port` is not read.
/// Rows keep file order; a later row with the same wid replaces the first.
pub fn parse_cached_list(xml: &str) -> Result<Vec<CachedInfo>, ConfigError> {
    let root = parse(xml)?;
    let mut rows: Vec<CachedInfo> = Vec::new();
    for child in &root.children {
        if child.name != "cached" {
            return Err(error("unexpected element in cached-list"));
        }
        let attr = |k: &str| child.attrs.get(k).map_or("", String::as_str);
        let row = CachedInfo {
            wid: wtoi(attr("wid")) as u16,
            ip: attr("ip-address")
                .trim()
                .parse()
                .unwrap_or(Ipv4Addr::BROADCAST),
            port: wtoi(attr("economy-port")) as u16,
        };
        match rows.iter_mut().find(|r| r.wid == row.wid) {
            Some(old) => *old = row,
            None => rows.push(row),
        }
    }
    Ok(rows)
}

/// CacheDList::writeSampleConfigFormat @0x140003230.
pub const SAMPLE_CACHED_LIST: &str = "<?xml version='1.0' encoding='UTF-8'?>\r\n<cached-list>\r\n    <!-- <cached wid='[world-id]' ip-address='[internal-IP]' internal-port='[internal-port]' economy-port='[economy-port]'/> -->\r\n</cached-list>\r\n";
