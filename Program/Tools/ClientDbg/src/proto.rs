//! Wire protocol: one JSON object per line.
//!
//! * Request: `{"id":n,"cmd":"ping", ...args}`
//! * Reply: `{"id":n,"ok":<value>}` or `{"id":n,"err":"text"}`
//! * Event: `{"ev":"hbp","t":<ms since plugin start>, ...}`
//! * First line from the plugin: `{"hello":{...},"token":"keyhere"}`

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// Hardware breakpoint kind.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Exec,
    Write,
    Rw,
}

/// Command with its arguments (the part of a request after the id).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Command {
    Ping,
    Modules,
    Threads,
    Rebase,
    Read {
        addr: u32,
        len: u32,
    },
    Write {
        addr: u32,
        hex: String,
    },
    Regs {
        tid: u32,
    },
    Hbp {
        addr: u32,
        kind: Kind,
        len: u8,
        slot: u8,
        #[serde(default)]
        capture: String,
        /// Stop recording after this many hits. 0 means no limit.
        #[serde(default)]
        limit: u32,
    },
    HbpClear {
        /// `None` clears all slots.
        #[serde(default)]
        slot: Option<u8>,
    },
}

/// Names of all commands, for error reports.
pub const COMMANDS: &[&str] = &[
    "ping",
    "modules",
    "threads",
    "rebase",
    "read",
    "write",
    "regs",
    "hbp",
    "hbp_clear",
];

/// A request line: id plus command.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub id: u64,
    #[serde(flatten)]
    pub cmd: Command,
}

/// A reply line.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Reply {
    Ok { id: u64, ok: Value },
    Err { id: u64, err: String },
}

impl Reply {
    pub fn ok<T: Serialize>(id: u64, v: &T) -> Reply {
        Reply::Ok {
            id,
            ok: serde_json::to_value(v).unwrap_or(Value::Null),
        }
    }
    pub fn err(id: u64, msg: impl Into<String>) -> Reply {
        Reply::Err {
            id,
            err: msg.into(),
        }
    }
    pub fn id(&self) -> u64 {
        match self {
            Reply::Ok { id, .. } | Reply::Err { id, .. } => *id,
        }
    }
}

/// General registers of one thread.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegSet {
    pub eax: u32,
    pub ecx: u32,
    pub edx: u32,
    pub ebx: u32,
    pub esp: u32,
    pub ebp: u32,
    pub esi: u32,
    pub edi: u32,
    pub eip: u32,
    pub eflags: u32,
}

/// Result of the `regs` command.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegDump {
    #[serde(flatten)]
    pub regs: RegSet,
    pub dr0: u32,
    pub dr1: u32,
    pub dr2: u32,
    pub dr3: u32,
    pub dr6: u32,
    pub dr7: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ModuleInfo {
    pub name: String,
    pub base: u32,
    pub size: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ThreadInfo {
    pub tid: u32,
    /// Win32 start address, 0 when unknown.
    pub start: u32,
    /// Current EIP, `None` when the thread could not be read.
    pub eip: Option<u32>,
}

/// Result of the `rebase` command.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RebaseInfo {
    pub base: u32,
}

/// One captured item of a breakpoint hit.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CapOut {
    /// Item in spec form, for example `mem:ecx+0x10:64`.
    pub item: String,
    /// Start address (`deref`: the pointer value).
    pub ptr: u32,
    pub hex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub err: Option<String>,
}

/// Event line from the plugin.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "ev", rename_all = "snake_case")]
pub enum Event {
    Hbp {
        t: u64,
        slot: u8,
        tid: u32,
        /// Address of the breakpoint as set.
        addr: u32,
        regs: RegSet,
        #[serde(default)]
        caps: Vec<CapOut>,
    },
    Log {
        t: u64,
        msg: String,
    },
    /// Sent when events were dropped because the ring was full.
    Stats {
        t: u64,
        dropped: u64,
    },
}

/// Information the plugin sends in its first line.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Hello {
    pub pid: u32,
    pub exe: String,
    pub base: u32,
    pub size: u32,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HelloMsg {
    pub hello: Hello,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtoError {
    BadJson(String),
    MissingId,
    MissingCmd,
    UnknownCmd(String),
    BadArgs { cmd: String, msg: String },
}

impl fmt::Display for ProtoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtoError::BadJson(m) => write!(f, "bad json: {m}"),
            ProtoError::MissingId => write!(f, "missing id"),
            ProtoError::MissingCmd => write!(f, "missing cmd"),
            ProtoError::UnknownCmd(c) => write!(f, "unknown cmd: {c}"),
            ProtoError::BadArgs { cmd, msg } => write!(f, "bad arguments for {cmd}: {msg}"),
        }
    }
}

impl std::error::Error for ProtoError {}

/// Encode a value as one JSON line (without the line end).
pub fn encode<T: Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "null".to_string())
}

pub fn decode_request(line: &str) -> Result<Request, ProtoError> {
    let v: Value =
        serde_json::from_str(line.trim()).map_err(|e| ProtoError::BadJson(e.to_string()))?;
    let obj = v
        .as_object()
        .ok_or_else(|| ProtoError::BadJson("not an object".into()))?;
    let id = obj
        .get("id")
        .and_then(Value::as_u64)
        .ok_or(ProtoError::MissingId)?;
    let cmd = obj
        .get("cmd")
        .and_then(Value::as_str)
        .ok_or(ProtoError::MissingCmd)?
        .to_string();
    match serde_json::from_value::<Command>(v.clone()) {
        Ok(cmd) => Ok(Request { id, cmd }),
        Err(e) if COMMANDS.contains(&cmd.as_str()) => Err(ProtoError::BadArgs {
            cmd,
            msg: e.to_string(),
        }),
        Err(_) => Err(ProtoError::UnknownCmd(cmd)),
    }
}

pub fn decode_reply(line: &str) -> Result<Reply, ProtoError> {
    serde_json::from_str(line.trim()).map_err(|e| ProtoError::BadJson(e.to_string()))
}

pub fn decode_event(line: &str) -> Result<Event, ProtoError> {
    serde_json::from_str(line.trim()).map_err(|e| ProtoError::BadJson(e.to_string()))
}

pub fn decode_hello(line: &str) -> Result<HelloMsg, ProtoError> {
    serde_json::from_str(line.trim()).map_err(|e| ProtoError::BadJson(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_requests() -> Vec<Request> {
        let cmds = vec![
            Command::Ping,
            Command::Modules,
            Command::Threads,
            Command::Rebase,
            Command::Read {
                addr: 0x5fd770,
                len: 64,
            },
            Command::Write {
                addr: 0x1000,
                hex: "90cc".into(),
            },
            Command::Regs { tid: 42 },
            Command::Hbp {
                addr: 0x5fd770,
                kind: Kind::Exec,
                len: 1,
                slot: 0,
                capture: "regs,stack:4".into(),
                limit: 10,
            },
            Command::Hbp {
                addr: 8,
                kind: Kind::Rw,
                len: 4,
                slot: 3,
                capture: String::new(),
                limit: 0,
            },
            Command::HbpClear { slot: Some(2) },
            Command::HbpClear { slot: None },
        ];
        cmds.into_iter()
            .enumerate()
            .map(|(i, cmd)| Request {
                id: i as u64 + 1,
                cmd,
            })
            .collect()
    }

    #[test]
    fn proto_roundtrip() {
        for r in all_requests() {
            let line = encode(&r);
            assert!(!line.contains('\n'));
            assert_eq!(decode_request(&line).unwrap(), r, "{line}");
        }
        let regs = RegSet {
            eax: 1,
            esp: 0x12f000,
            eip: 0x5fd770,
            eflags: 0x246,
            ..Default::default()
        };
        let events = vec![
            Event::Hbp {
                t: 5,
                slot: 1,
                tid: 7,
                addr: 0x5fd770,
                regs,
                caps: vec![
                    CapOut {
                        item: "stack:2".into(),
                        ptr: 0x12f000,
                        hex: "0011223344556677".into(),
                        err: None,
                    },
                    CapOut {
                        item: "deref:eax".into(),
                        ptr: 0,
                        hex: String::new(),
                        err: Some("unreadable".into()),
                    },
                ],
            },
            Event::Log {
                t: 1,
                msg: "hi".into(),
            },
            Event::Stats { t: 9, dropped: 3 },
        ];
        for e in events {
            assert_eq!(decode_event(&encode(&e)).unwrap(), e);
        }
        for rep in [Reply::ok(1, &"pong"), Reply::err(2, "bad")] {
            assert_eq!(decode_reply(&encode(&rep)).unwrap(), rep);
        }
        let h = HelloMsg {
            hello: Hello {
                pid: 1,
                exe: "C:\\Client.exe".into(),
                base: 0x400000,
                size: 0x1000,
                version: "0.0.381.8884".into(),
            },
            token: "keyhere".into(),
        };
        assert_eq!(decode_hello(&encode(&h)).unwrap(), h);
        let d = RegDump {
            regs,
            dr7: 0x401,
            ..Default::default()
        };
        assert_eq!(serde_json::from_str::<RegDump>(&encode(&d)).unwrap(), d);
    }

    #[test]
    fn proto_bad_line() {
        assert_eq!(
            decode_request(r#"{"id":1,"cmd":"nope"}"#),
            Err(ProtoError::UnknownCmd("nope".into()))
        );
        assert!(matches!(decode_request("{"), Err(ProtoError::BadJson(_))));
        assert_eq!(
            decode_request(r#"{"cmd":"ping"}"#),
            Err(ProtoError::MissingId)
        );
        assert_eq!(decode_request(r#"{"id":1}"#), Err(ProtoError::MissingCmd));
        assert!(matches!(
            decode_request(r#"{"id":1,"cmd":"read","addr":1}"#),
            Err(ProtoError::BadArgs { .. })
        ));
    }

    #[test]
    fn proto_wire_shape() {
        assert_eq!(
            encode(&Request {
                id: 3,
                cmd: Command::Ping
            }),
            r#"{"id":3,"cmd":"ping"}"#
        );
        assert_eq!(encode(&Reply::ok(3, &"pong")), r#"{"id":3,"ok":"pong"}"#);
        assert_eq!(encode(&Reply::err(3, "x")), r#"{"id":3,"err":"x"}"#);
    }
}
