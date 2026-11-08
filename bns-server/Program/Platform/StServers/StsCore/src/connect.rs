//! Connection identity sent first on every STS socket (`POST /Sts/Connect`).
//! RouteSrv `sub_1401?` parser at RouteSrv.c ~207150 (build 381).
//!
//! Three forms:
//! 1. Header `X-Sts-Connect: <ConnType>; product=; index=; address=;
//!    deployment=; epoch=; notify=; version=; process=; program=; build=`.
//!    Required non-zero: ConnType, process, program, build. `address` must parse.
//! 2. Body `<Connect>` with ConnType, Process (required), ProductType,
//!    AppIndex, Deployment, NotifyFlags, VersionFlags, then `Address` +
//!    Program, Build, Epoch — or `Location` = `addr;program;build;epoch[;x]`.
//!    Optional ConnProductType, ConnAppIndex, ConnDeployment, ConnEpoch.
//! 3. No identity (game client): ConnType 300, 301 with `Authorization`,
//!    302 with `Upgrade: websocket`.
//!
//! `program == -1` means "my own program id".

use crate::route::Location;
use crate::xml::Element;
use sts::Frame;
use std::net::SocketAddr;

pub const CONN_CLIENT: u32 = 300;
pub const CONN_CLIENT_AUTH: u32 = 301;
pub const CONN_WEBSOCKET: u32 = 302;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConnectInfo {
    pub conn_type: u32,
    pub location: Location,
    pub process: u32,
    pub program: u32,
    pub build: u32,
    pub notify_flags: u32,
    pub version_flags: u32,
    pub conn_product_type: u32,
    pub conn_app_index: u32,
    pub conn_deployment: u32,
    pub conn_epoch: u32,
}

fn header<'a>(frame: &'a Frame, name: &str) -> Option<&'a str> {
    frame.headers.iter().find(|h| h.name.eq_ignore_ascii_case(name)).map(|h| h.value.as_str())
}

/// Value of `key=` inside a `;`-separated header value. `None` key → the
/// leading token without `=`.
pub fn header_param<'a>(value: &'a str, key: Option<&str>) -> Option<&'a str> {
    value.split(';').map(str::trim).find_map(|part| match (key, part.split_once('=')) {
        (None, None) if !part.is_empty() => Some(part),
        (Some(k), Some((n, v))) if n.trim().eq_ignore_ascii_case(k) => Some(v.trim()),
        _ => None,
    })
}

fn num(s: Option<&str>) -> u32 {
    s.and_then(crate::xml::parse_u32).unwrap_or(0)
}

impl ConnectInfo {
    /// Reads the identity of a peer. `own_program` replaces `-1`.
    /// `None` means the identity is invalid and the socket must close.
    pub fn from_frame(frame: &Frame, own_program: u32) -> Option<Self> {
        let is_connect = matches!(&frame.start,
            sts::StartLine::Request { protocol, command } if protocol == "Sts" && command == "Connect");
        if !is_connect {
            return Self::from_body(&Element::parse(&frame.body).ok()?, own_program);
        }
        let Some(h) = header(frame, "x-sts-connect") else {
            let ws = header(frame, "upgrade").is_some_and(|v| v.eq_ignore_ascii_case("websocket"));
            let conn_type = if ws {
                CONN_WEBSOCKET
            } else if header(frame, "authorization").is_some() {
                CONN_CLIENT_AUTH
            } else {
                CONN_CLIENT
            };
            return Some(Self { conn_type, ..Self::default() });
        };
        let p = |k| num(header_param(h, Some(k)));
        let addr: SocketAddr = header_param(h, Some("address"))?.parse().ok()?;
        let mut program = p("program");
        if program == u32::MAX {
            program = own_program;
        }
        let info = Self {
            conn_type: num(header_param(h, None)),
            location: Location {
                addr: Some(addr),
                product: p("product"),
                index: p("index"),
                deployment: p("deployment"),
                epoch: p("epoch"),
            },
            process: p("process"),
            program,
            build: p("build"),
            notify_flags: p("notify"),
            version_flags: p("version"),
            ..Self::default()
        };
        (info.conn_type != 0 && info.process != 0 && info.build != 0 && info.program != 0).then_some(info)
    }

    pub fn from_body(e: &Element, own_program: u32) -> Option<Self> {
        if e.name != "Connect" {
            return None;
        }
        let conn_type = e.get_u32("ConnType", 0);
        let process = e.get_u32("Process", 0);
        if conn_type == 0 || process == 0 {
            return None;
        }
        let mut location = Location {
            product: e.get_u32("ProductType", 0),
            index: e.get_u32("AppIndex", 0),
            deployment: e.get_u32("Deployment", 0),
            ..Location::default()
        };
        let (mut program, build);
        if let Some(a) = e.get_text("Address") {
            location.addr = Some(a.parse().ok()?);
            program = e.get_u32("Program", 0);
            build = e.get_u32("Build", 0);
            location.epoch = e.get_u32("Epoch", 0);
        } else {
            let loc = e.get_text("Location")?;
            let (addr, rest) = loc.split_once(';')?;
            location.addr = Some(addr.parse().ok()?);
            let nums: Vec<u32> = rest.split(';').map(crate::xml::parse_u32).collect::<Option<_>>()?;
            if !(3..=4).contains(&nums.len()) {
                return None;
            }
            program = nums[0];
            build = nums[1];
            location.epoch = nums[2];
        }
        if program == u32::MAX {
            program = own_program;
        }
        if build == 0 || program == 0 {
            return None;
        }
        Some(Self {
            conn_type,
            location,
            process,
            program,
            build,
            notify_flags: e.get_u32("NotifyFlags", 0),
            version_flags: e.get_u32("VersionFlags", 0),
            conn_product_type: e.get_u32("ConnProductType", 0),
            conn_app_index: e.get_u32("ConnAppIndex", 0),
            conn_deployment: e.get_u32("ConnDeployment", 0),
            conn_epoch: e.get_u32("ConnEpoch", 0),
        })
    }

    /// Header form, as a service sends it.
    pub fn to_header_value(&self) -> String {
        let l = &self.location;
        let addr = l.addr.map(|a| a.to_string()).unwrap_or_default();
        format!(
            "{}; product={}; index={}; address={}; deployment={}; epoch={}; notify={}; version={}; process={}; program={}; build={}",
            self.conn_type, l.product, l.index, addr, l.deployment, l.epoch,
            self.notify_flags, self.version_flags, self.process, self.program as i32, self.build
        )
    }

    pub fn is_client(&self) -> bool {
        (CONN_CLIENT..=CONN_WEBSOCKET).contains(&self.conn_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sts::{Header, StartLine};

    fn connect(headers: Vec<(&str, &str)>) -> Frame {
        Frame {
            start: StartLine::Request { protocol: "Sts".into(), command: "Connect".into() },
            headers: headers.into_iter().map(|(n, v)| Header { name: n.into(), value: v.into() }).collect(),
            body: vec![],
        }
    }

    #[test]
    fn header_form_roundtrip() {
        let info = ConnectInfo {
            conn_type: 2,
            location: Location { addr: Some("10.0.0.1:6600".parse().unwrap()), product: 1, index: 2, deployment: 3, epoch: 4 },
            process: 99,
            program: 101,
            build: 381,
            ..ConnectInfo::default()
        };
        let f = connect(vec![("x-sts-connect", &info.to_header_value())]);
        assert_eq!(ConnectInfo::from_frame(&f, 0), Some(info));
    }

    #[test]
    fn client_forms() {
        assert_eq!(ConnectInfo::from_frame(&connect(vec![]), 0).unwrap().conn_type, 300);
        assert_eq!(ConnectInfo::from_frame(&connect(vec![("authorization", "x")]), 0).unwrap().conn_type, 301);
        assert_eq!(ConnectInfo::from_frame(&connect(vec![("upgrade", "websocket")]), 0).unwrap().conn_type, 302);
    }

    #[test]
    fn body_location_form() {
        let e = Element::new("Connect")
            .child(Element::int("ConnType", 2))
            .child(Element::int("Process", 5))
            .child(Element::text("Location", "127.0.0.1:1;-1;381;7"));
        let i = ConnectInfo::from_body(&e, 103).unwrap();
        assert_eq!((i.program, i.build, i.location.epoch), (103, 381, 7));
    }
}
