//! Command line parsing for `clientdbg <cmd> [args]`.

use serde_json::{json, Map, Value};
use std::collections::HashMap;

/// Flags that take a value.
const VALUE_FLAGS: &[&str] = &[
    "ctl",
    "listen",
    "token-file",
    "log-dir",
    "capture",
    "limit",
    "n",
    "filter",
];

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Opts {
    pub pos: Vec<String>,
    pub flags: HashMap<String, String>,
    pub ida: bool,
}

pub fn parse_args(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts::default();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if let Some(name) = a.strip_prefix("--") {
            if name == "ida" {
                o.ida = true;
            } else if VALUE_FLAGS.contains(&name) {
                let v = it.next().ok_or_else(|| format!("--{name} needs a value"))?;
                o.flags.insert(name.to_string(), v.clone());
            } else {
                return Err(format!("unknown flag --{name}"));
            }
        } else {
            o.pos.push(a.clone());
        }
    }
    Ok(o)
}

/// Parse a decimal or `0x` hex number.
pub fn parse_u32(s: &str) -> Result<u32, String> {
    let r = match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Some(h) => u32::from_str_radix(h, 16),
        None => s.parse::<u32>(),
    };
    r.map_err(|_| format!("bad number: {s}"))
}

fn arg<'a>(o: &'a Opts, i: usize, name: &str) -> Result<&'a str, String> {
    o.pos
        .get(i)
        .map(String::as_str)
        .ok_or_else(|| format!("missing argument: {name}"))
}

/// Build the control request (JSON object, no id) for a command line.
pub fn build_request(o: &Opts) -> Result<Value, String> {
    let cmd = arg(o, 0, "cmd")?.replace('-', "_");
    let mut m = Map::new();
    match cmd.as_str() {
        "ping" | "status" | "modules" | "threads" | "rebase" => {}
        "read" => {
            m.insert("addr".into(), json!(parse_u32(arg(o, 1, "addr")?)?));
            m.insert("len".into(), json!(parse_u32(arg(o, 2, "len")?)?));
        }
        "write" => {
            m.insert("addr".into(), json!(parse_u32(arg(o, 1, "addr")?)?));
            m.insert("hex".into(), json!(arg(o, 2, "hex")?));
        }
        "regs" => {
            m.insert("tid".into(), json!(parse_u32(arg(o, 1, "tid")?)?));
        }
        "hbp" => {
            m.insert("addr".into(), json!(parse_u32(arg(o, 1, "addr")?)?));
            let kind = arg(o, 2, "kind")?;
            if !matches!(kind, "exec" | "write" | "rw") {
                return Err("kind must be exec, write or rw".into());
            }
            m.insert("kind".into(), json!(kind));
            m.insert("len".into(), json!(parse_u32(arg(o, 3, "len")?)?));
            m.insert("slot".into(), json!(parse_u32(arg(o, 4, "slot")?)?));
            if let Some(c) = o.flags.get("capture") {
                m.insert("capture".into(), json!(c));
            }
            if let Some(l) = o.flags.get("limit") {
                m.insert("limit".into(), json!(parse_u32(l)?));
            }
        }
        "hbp_clear" => match o.pos.get(1).map(String::as_str) {
            None | Some("all") => {}
            Some(s) => {
                m.insert("slot".into(), json!(parse_u32(s)?));
            }
        },
        other => return Err(format!("unknown command: {other}")),
    }
    m.insert("cmd".into(), json!(cmd));
    if o.ida {
        m.insert("ida".into(), json!(true));
    }
    Ok(Value::Object(m))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(s: &str) -> Opts {
        let a: Vec<String> = s.split_whitespace().map(str::to_string).collect();
        parse_args(&a).unwrap()
    }

    #[test]
    fn cli_requests() {
        let v = build_request(&opts("read 0x5fd770 64 --ida")).unwrap();
        assert_eq!(v, json!({"cmd":"read","addr":0x5fd770,"len":64,"ida":true}));
        let v = build_request(&opts("hbp 0x1000 rw 4 1 --capture regs,stack:4 --limit 5")).unwrap();
        assert_eq!(
            v,
            json!({"cmd":"hbp","addr":0x1000,"kind":"rw","len":4,"slot":1,"capture":"regs,stack:4","limit":5})
        );
        assert_eq!(
            build_request(&opts("hbp-clear")).unwrap(),
            json!({"cmd":"hbp_clear"})
        );
        assert_eq!(
            build_request(&opts("hbp_clear 2")).unwrap(),
            json!({"cmd":"hbp_clear","slot":2})
        );
        assert!(build_request(&opts("read 1")).is_err());
        assert!(build_request(&opts("hbp 1 bad 1 0")).is_err());
        assert!(build_request(&opts("bogus")).is_err());
        assert!(parse_args(&["--nope".to_string()]).is_err());
    }
}
