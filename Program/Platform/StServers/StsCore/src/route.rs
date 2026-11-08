//! Route protocol: commands, the `<Message>` body, locations and `t` targets.
//! Spec: Docs/Platform/StServers/13-routesrv.md.

use crate::xml::{parse_u32, Element};
use std::net::SocketAddr;

/// Provider location: socket address plus product, index, deployment, epoch.
/// Text: `<addr> <product> <index> <deployment> <epoch>[ <extra>]`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Location {
    pub addr: Option<SocketAddr>,
    pub product: u32,
    pub index: u32,
    pub deployment: u32,
    pub epoch: u32,
}

impl Location {
    pub fn format(&self, extra: u32) -> String {
        let addr = self.addr.map(|a| a.to_string()).unwrap_or_else(|| "0.0.0.0:0".into());
        let mut s = format!("{addr} {} {} {} {}", self.product, self.index, self.deployment, self.epoch);
        if extra != 0 {
            s.push_str(&format!(" {extra}"));
        }
        s
    }

    /// Returns the location and the optional 5th number (0 when missing).
    pub fn parse(s: &str) -> Option<(Self, u32)> {
        let mut it = s.split_ascii_whitespace();
        let addr = it.next()?.parse().ok()?;
        let nums: Vec<u32> = it.map(parse_u32).collect::<Option<_>>()?;
        if !(4..=5).contains(&nums.len()) {
            return None;
        }
        Some((
            Self { addr: Some(addr), product: nums[0], index: nums[1], deployment: nums[2], epoch: nums[3] },
            nums.get(4).copied().unwrap_or(0),
        ))
    }
}

macro_rules! commands {
    ($($id:literal $name:ident $mask:literal),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[repr(u32)]
        pub enum RouteCmd { $($name = $id),* }

        impl RouteCmd {
            pub fn from_id(id: u32) -> Option<Self> {
                match id { $($id => Some(Self::$name),)* _ => None }
            }
            pub fn name(self) -> &'static str {
                match self { $(Self::$name => stringify!($name)),* }
            }
            pub fn from_name(name: &str) -> Option<Self> {
                match name { $(stringify!($name) => Some(Self::$name),)* _ => None }
            }
            /// Field mask (AuthSrv `sub_140118EF0`).
            pub fn mask(self) -> u32 {
                match self { $(Self::$name => $mask),* }
            }
        }
    };
}

commands! {
    1 CheckRegistration 0,
    2 Snapshot 0x788,
    3 ShutdownServer 0,
    4 RegisterProvider 0x788,
    5 UnregisterProvider 0x788,
    6 RegisterForward 0x1A80,
    7 SubscribeService 0x80,
    8 SetBlocks 0x4780,
    9 GetBlocks 0x180,
    10 RouteXferRequest 0x7AA,
    11 RouteXferBlock 0x7A9,
    12 BlocksFound 0x2100,
    13 ProviderFound 0x88,
    14 ProviderLost 0x88,
    15 ServerShutdown 0xC,
    16 ServerDisconnect 0xC,
    17 ProviderChanged 0x27EC,
    18 ProviderShutdown 0x28C,
    19 ServiceSetForward 0x1280,
    20 MultiRouterChanged 0x10000,
    21 RouterConnect 0x204,
    22 ServiceChanged 0xF780,
    23 StartTls 0,
}

pub mod field {
    pub const T_DIRECT: u32 = 0x1;
    pub const T_BLOCK: u32 = 0x2;
    pub const ROUTER: u32 = 0x4;
    pub const PROVIDER: u32 = 0x8;
    pub const CLIENT: u32 = 0x10;
    pub const BLOCK: u32 = 0x20;
    pub const FLAGS: u32 = 0x40;
    pub const NAME: u32 = 0x80;
    pub const BLOCK_BITS: u32 = 0x100;
    pub const BUILD: u32 = 0x200;
    pub const TOPOLOGY: u32 = 0x400;
    pub const TEMPLATE: u32 = 0x800;
    pub const FORWARDS: u32 = 0x1000;
    pub const CURRENT: u32 = 0x2000;
    pub const DESIRED: u32 = 0x4000;
    pub const LISTS: u32 = 0x8000;
    pub const SUB_CMD: u32 = 0x10000;
    pub const PROTOCOL: u32 = 0x7780;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum Topology {
    #[default]
    Invalid = 0,
    Basic = 1,
    Star = 2,
    Grid = 3,
    Extern = 4,
}

impl Topology {
    pub fn name(self) -> &'static str {
        match self {
            Self::Invalid => "Invalid",
            Self::Basic => "Basic",
            Self::Star => "Star",
            Self::Grid => "Grid",
            Self::Extern => "Extern",
        }
    }
    pub fn from_name(s: &str) -> Self {
        match s {
            "Basic" => Self::Basic,
            "Star" => Self::Star,
            "Grid" | "Federation" => Self::Grid,
            "Extern" => Self::Extern,
            _ => Self::Invalid,
        }
    }
}

/// One block slot: provider id (24 bits) and two state digits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockSlot(pub u32);

impl BlockSlot {
    pub fn provider(self) -> u32 {
        self.0 & 0xFF_FFFF
    }
    fn format(self, out: &mut String) {
        if self.provider() != 0 {
            out.push_str(&self.provider().to_string());
        }
        let (a, b) = ((self.0 >> 24) & 7, (self.0 >> 27) & 0xF);
        if a != 0 || b != 0 {
            out.push('/');
            out.push(char::from_digit(a, 16).unwrap_or('0'));
            out.push(char::from_digit(b, 16).unwrap_or('0'));
        }
    }
    fn parse(s: &str) -> Option<Self> {
        let (id, st) = s.split_once('/').unwrap_or((s, ""));
        let id = if id.is_empty() { 0 } else { id.parse::<u32>().ok()? & 0xFF_FFFF };
        let mut v = id;
        if !st.is_empty() {
            let d: Vec<u32> = st.chars().map(|c| c.to_digit(16)).collect::<Option<_>>()?;
            if d.len() != 2 {
                return None;
            }
            v |= (d[0] & 7) << 24 | (d[1] & 0xF) << 27;
        }
        Some(Self(v))
    }
}

fn blocks_text(b: &[BlockSlot]) -> String {
    let mut s = String::new();
    for (i, slot) in b.iter().enumerate() {
        if i != 0 {
            s.push(',');
        }
        slot.format(&mut s);
    }
    s
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Forward {
    pub scope: String,
    pub name_db: String,
    pub not_found: u32,
    pub no_cache: bool,
    pub use_no_name: bool,
    pub use_local_center: bool,
    pub version: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Member {
    pub code: u32,
    pub location: Location,
    pub server_flags: u32,
    pub service_flags: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Connection {
    pub router_code: u32,
    pub code: u32,
    pub epoch: u32,
}

/// The `<Message>` body of every Route command.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RouteMessage {
    pub cmd: Option<RouteCmd>,
    pub sub_cmd: Option<RouteCmd>,
    pub router: Option<Location>,
    pub provider: Option<Location>,
    pub client: u32,
    pub block: u32,
    pub flags: u32,
    pub name: String,
    pub build: u32,
    pub topology: Topology,
    pub block_bits: u32,
    pub current: Vec<BlockSlot>,
    pub desired: Vec<BlockSlot>,
    pub forwards: Vec<Forward>,
    pub template: Option<(String, u32)>,
    pub providers: Vec<Member>,
    pub routers: Vec<Member>,
    pub connections: Vec<Connection>,
}

impl RouteMessage {
    pub fn new(cmd: RouteCmd) -> Self {
        Self { cmd: Some(cmd), ..Self::default() }
    }

    /// Mask for this message: the command mask, or for MultiRouterChanged
    /// the SubCmd mask plus 0x10000.
    pub fn mask(&self) -> u32 {
        match (self.cmd, self.sub_cmd) {
            (Some(RouteCmd::MultiRouterChanged), Some(sub)) => sub.mask() | field::SUB_CMD,
            (Some(c), _) => c.mask(),
            _ => 0,
        }
    }

    pub fn to_element(&self) -> Element {
        self.to_element_masked(self.mask())
    }

    /// Writer `sub_140118530`.
    pub fn to_element_masked(&self, m: u32) -> Element {
        use field::*;
        let mut e = Element::new("Message");
        if m & SUB_CMD != 0 {
            e.push(Element::text("SubCmd", self.sub_cmd.map_or("Invalid", RouteCmd::name)));
        }
        if m & ROUTER != 0 {
            e.push(Element::text("Router", self.router.unwrap_or_default().format(0)));
        }
        if m & PROVIDER != 0 {
            e.push(Element::text("Provider", self.provider.unwrap_or_default().format(0)));
        }
        if m & CLIENT != 0 {
            e.push_nonzero("Client", self.client);
        }
        if m & BLOCK != 0 {
            e.push_nonzero("Block", self.block);
        }
        if m & FLAGS != 0 {
            e.push_nonzero("Flags", self.flags);
        }
        if m & PROTOCOL != 0 {
            let mut p = Element::new("Protocol");
            if m & NAME != 0 {
                p.push_text_nonempty("Name", &self.name);
            }
            if m & BUILD != 0 {
                p.push(Element::int("Build", self.build as i32 as i64));
            }
            if m & TOPOLOGY != 0 {
                p.push(Element::text("Topology", self.topology.name()));
            }
            if m & BLOCK_BITS != 0 {
                p.push_nonzero("BlockBits", self.block_bits);
            }
            let n = if self.block_bits != 0 { 1usize << self.block_bits } else { 0 };
            if m & CURRENT != 0 && !self.current.is_empty() {
                p.push(Element::text("Current", blocks_text(&self.current[..n.min(self.current.len())])));
            }
            if m & DESIRED != 0 && !self.desired.is_empty() {
                p.push(Element::text("Desired", blocks_text(&self.desired[..n.min(self.desired.len())])));
            }
            if m & FORWARDS != 0 && !self.forwards.is_empty() {
                let mut fa = Element::array("Forwards");
                for f in &self.forwards {
                    let mut fe = Element::new("Forward");
                    fe.push(Element::text("Scope", f.scope.clone()));
                    fe.push(Element::text("NameDb", f.name_db.clone()));
                    fe.push_nonzero("NotFound", f.not_found);
                    for (on, n) in [(f.no_cache, "NoCache"), (f.use_no_name, "UseNoName"), (f.use_local_center, "UseLocalCenter")] {
                        if on {
                            fe.push(Element::new(n));
                        }
                    }
                    if let Some(v) = &f.version {
                        fe.push(Element::text("Version", v.clone()));
                    }
                    fa.push(fe);
                }
                p.push(fa);
            }
            e.push(p);
        }
        if m & TEMPLATE != 0 {
            if let Some((name, build)) = &self.template {
                if !name.is_empty() {
                    let mut t = Element::new("ProtocolTemplate");
                    t.push(Element::text("Name", name.clone()));
                    t.push_nonzero("Build", *build);
                    e.push(t);
                }
            }
        }
        if m & LISTS != 0 {
            for (list, item, members) in [("Providers", "Provider", &self.providers), ("Routers", "Router", &self.routers)] {
                let mut a = Element::array(list);
                for mb in members {
                    let mut it = Element::new(item)
                        .attr("code", mb.code.to_string())
                        .attr("location", mb.location.format(0));
                    if mb.server_flags != 0 {
                        it = it.attr("serverFlags", mb.server_flags.to_string());
                    }
                    if mb.service_flags != 0 {
                        it = it.attr("serviceFlags", mb.service_flags.to_string());
                    }
                    a.push(it);
                }
                e.push(a);
            }
            let mut a = Element::array("Connections");
            for c in &self.connections {
                a.push(
                    Element::new("Connection")
                        .attr("code", c.code.to_string())
                        .attr("epoch", c.epoch.to_string())
                        .attr("routerCode", c.router_code.to_string()),
                );
            }
            e.push(a);
        }
        e
    }

    /// Parser `sub_1400FA0B0`. `cmd` comes from the request line.
    pub fn from_element(cmd: Option<RouteCmd>, e: &Element) -> Option<Self> {
        if e.name != "Message" {
            return None;
        }
        let loc = |n: &str| -> Option<Option<Location>> {
            match e.get_text(n) {
                Some(t) if !t.is_empty() => Location::parse(t).map(|(l, _)| Some(l)),
                _ => Some(None),
            }
        };
        let mut m = Self {
            cmd,
            sub_cmd: e.get_text("SubCmd").and_then(RouteCmd::from_name),
            router: loc("Router")?,
            provider: loc("Provider")?,
            client: e.get_u32("Client", 0),
            block: e.get_u32("Block", 0),
            flags: e.get_u32("Flags", 0),
            ..Self::default()
        };
        if let Some(p) = e.get("Protocol") {
            m.name = p.get_text("Name").unwrap_or_default().to_owned();
            m.build = p.get_u32("Build", 0);
            m.topology = Topology::from_name(p.get_text("Topology").unwrap_or_default());
            m.block_bits = p.get_u32("BlockBits", 0);
            let blocks = |n| -> Option<Vec<BlockSlot>> {
                match p.get_text(n) {
                    Some(t) if !t.is_empty() => t.split(',').map(BlockSlot::parse).collect(),
                    _ => Some(vec![]),
                }
            };
            m.current = blocks("Current")?;
            m.desired = blocks("Desired")?;
            if let Some(fa) = p.get("Forwards") {
                for f in fa.all("Forward") {
                    m.forwards.push(Forward {
                        scope: f.get_text("Scope").unwrap_or_default().into(),
                        name_db: f.get_text("NameDb").unwrap_or_default().into(),
                        not_found: f.get_u32("NotFound", 0),
                        no_cache: f.has("NoCache"),
                        use_no_name: f.has("UseNoName"),
                        use_local_center: f.has("UseLocalCenter"),
                        version: f.get_text("Version").map(Into::into),
                    });
                }
            }
        }
        if let Some(t) = e.get("ProtocolTemplate") {
            m.template = Some((t.get_text("Name").unwrap_or_default().into(), t.get_u32("Build", 0)));
        }
        for (list, item, dst) in [("Providers", "Provider", &mut m.providers), ("Routers", "Router", &mut m.routers)] {
            if let Some(a) = e.get(list) {
                for it in a.all(item) {
                    dst.push(Member {
                        code: it.attr_u32("code", 0),
                        location: Location::parse(it.attr_value("location")?)?.0,
                        server_flags: it.attr_u32("serverFlags", 0),
                        service_flags: it.attr_u32("serviceFlags", 0),
                    });
                }
            }
        }
        if let Some(a) = e.get("Connections") {
            for c in a.all("Connection") {
                m.connections.push(Connection {
                    router_code: c.attr_u32("routerCode", 0),
                    code: c.attr_u32("code", 0),
                    epoch: c.attr_u32("epoch", 0),
                });
            }
        }
        Some(m)
    }

    /// Request line protocol and `t` header (`sub_140118B60`).
    pub fn request_target(&self) -> (String, Option<Target>) {
        let m = self.mask();
        if m & field::T_BLOCK != 0 {
            return (self.name.clone(), Some(Target::Block { center: 0, block: self.block, protocol: None }));
        }
        if m & field::T_DIRECT != 0 {
            return (self.name.clone(), Some(Target::Direct(self.provider.unwrap_or_default())));
        }
        ("Route".into(), None)
    }
}

/// `t` header (`sub_1400F9FC0`). Center is 0–63, 0 = local.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// `*<location>`
    Direct(Location),
    /// `<c>$<key>`
    Key { center: u32, key: String },
    /// `<c>@<scope>:<name>[;rest]`
    Forward { center: u32, address: String },
    /// `<c>#<hex block>`, optional `protocol` attribute.
    Block { center: u32, block: u32, protocol: Option<String> },
}

impl Target {
    pub fn parse(s: &str) -> Option<Self> {
        if let Some(rest) = s.strip_prefix('*') {
            return Some(Self::Direct(Location::parse(rest)?.0));
        }
        let split = s.find(|c: char| !c.is_ascii_alphanumeric())?;
        let center = parse_c(&s[..split])?;
        if center > 0x3F {
            return None;
        }
        let rest = &s[split + 1..];
        match s.as_bytes()[split] {
            b'$' => Some(Self::Key { center, key: rest.into() }),
            b'@' => Some(Self::Forward { center, address: rest.into() }),
            b'#' => {
                let (hex, attrs) = rest.split_once(';').unwrap_or((rest, ""));
                let protocol = attrs
                    .split(';')
                    .find_map(|kv| kv.trim().strip_prefix("protocol=").map(Into::into));
                Some(Self::Block { center, block: u32::from_str_radix(hex, 16).ok()?, protocol })
            }
            _ => None,
        }
    }

    pub fn format(&self) -> String {
        match self {
            Self::Direct(l) => format!("*{}", l.format(0)),
            Self::Key { center, key } => format!("{center}${key}"),
            Self::Forward { center, address } => format!("{center}@{address}"),
            Self::Block { center, block, protocol } => match protocol {
                Some(p) => format!("{center}#{block:X};protocol={p}"),
                None => format!("{center}#{block:X}"),
            },
        }
    }
}

/// `strtoul(s, _, 0)`: decimal, `0x` hex or leading-0 octal.
fn parse_c(s: &str) -> Option<u32> {
    if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(h, 16).ok()
    } else if s.len() > 1 && s.starts_with('0') {
        u32::from_str_radix(&s[1..], 8).ok()
    } else {
        s.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loc() -> Location {
        Location { addr: Some("10.1.2.3:6000".parse().unwrap()), product: 1, index: 2, deployment: 3, epoch: 4 }
    }

    #[test]
    fn register_provider_body() {
        let mut m = RouteMessage::new(RouteCmd::RegisterProvider);
        m.provider = Some(loc());
        m.name = "Auth".into();
        m.build = 381;
        m.topology = Topology::Grid;
        m.block_bits = 5;
        let x = m.to_element().to_xml();
        assert_eq!(
            x,
            "<Message>\n<Provider>10.1.2.3:6000 1 2 3 4</Provider>\n<Protocol>\n<Name>Auth</Name>\n<Build>381</Build>\n<Topology>Grid</Topology>\n<BlockBits>5</BlockBits>\n</Protocol>\n</Message>\n"
        );
        let back = RouteMessage::from_element(m.cmd, &Element::parse(x.as_bytes()).unwrap()).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn service_changed_lists() {
        let mut m = RouteMessage::new(RouteCmd::ServiceChanged);
        m.name = "Auth".into();
        m.block_bits = 1;
        m.topology = Topology::Basic;
        m.current = vec![BlockSlot(7), BlockSlot(7 | 1 << 24 | 2 << 27)];
        m.desired = m.current.clone();
        m.providers = vec![Member { code: 7, location: loc(), server_flags: 0, service_flags: 1 }];
        m.connections = vec![Connection { router_code: 1, code: 7, epoch: 4 }];
        let x = m.to_element().to_xml();
        assert!(x.contains("<Current>7,7/12</Current>"), "{x}");
        let back = RouteMessage::from_element(m.cmd, &Element::parse(x.as_bytes()).unwrap()).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn targets() {
        for s in ["*10.1.2.3:6000 1 2 3 4", "0$alice", "3@Name:x;y", "0#1F", "0#1F;protocol=Auth"] {
            assert_eq!(Target::parse(s).unwrap().format(), s);
        }
        assert!(Target::parse("64$x").is_none());
        assert_eq!(RouteCmd::from_id(13).unwrap().name(), "ProviderFound");
    }
}
