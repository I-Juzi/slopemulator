//! Achievement rule tables (spec README.md): `achievement-register` and
//! `achievement` tables, from plain XML or LMXBOSLB files.
use crate::config::{parse_limited, ConfigError, Element};
use std::collections::{BTreeMap, HashMap};

const MAX_TABLE_XML: usize = 64 << 20;
/// AchievementRegister keeps at most 8 achievement links per register.
pub const MAX_LINKS_PER_REGISTER: usize = 8;
/// Achievement records hold register-ref-1 .. register-ref-5.
pub const MAX_CONDITIONS: usize = 5;

/// Job names in table order. Index + 1 is the Pc job byte.
/// ❓ Order taken from the client job enum, not read from the daemon.
pub const JOB_NAMES: [&str; 12] = [
    "검사",
    "권사",
    "기공사",
    "격사",
    "역사",
    "소환사",
    "암살자",
    "귀검사",
    "주술사",
    "기권사",
    "투사",
    "궁사",
];

pub fn job_from_name(name: &str) -> Option<u8> {
    if name.is_empty() || name == "job-none" {
        return Some(0);
    }
    if let Ok(n) = name.parse::<u8>() {
        return Some(n);
    }
    JOB_NAMES
        .iter()
        .position(|j| *j == name)
        .map(|i| i as u8 + 1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// 1 `above`: value >= v.
    Above,
    /// 2 `below`: value <= v.
    Below,
    /// 3 `bits-on`: the low v bits of value are all set.
    BitsOn,
}

impl Op {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "above" | "1" => Some(Self::Above),
            "below" | "2" => Some(Self::Below),
            "bits-on" | "3" => Some(Self::BitsOn),
            _ => None,
        }
    }
    pub fn holds(self, value: i64, v: i64) -> bool {
        match self {
            Self::Above => value >= v,
            Self::Below => value <= v,
            Self::BitsOn => {
                if v <= 0 {
                    return true;
                }
                let mask = if v >= 64 { u64::MAX } else { (1u64 << v) - 1 };
                (value as u64) & mask == mask
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Register {
    pub id: u16,
    pub alias: String,
    pub kind: String,
    pub job: u8,
    /// 0 means no upper clamp.
    pub max_value: i64,
    pub persistant: bool,
    pub progress_show: bool,
    pub increment_from_client: bool,
    /// Grade selector for `duel-grade` (0 max, 1 solo, 2 team) and
    /// `party-battle-grade` (1..=3). ❓ Attribute name not seen in the 0.42 data.
    pub sub_type: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Condition {
    pub register: u16,
    pub op: Op,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Achievement {
    pub id: u16,
    pub step: u16,
    pub job: u8,
    pub score: i32,
    pub conditions: Vec<Condition>,
    pub deprecated: bool,
    pub complete_from_client: bool,
    pub has_reward: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Rules {
    /// Keyed by (id, job): one register id can have a row per job.
    pub registers: BTreeMap<(u16, u8), Register>,
    pub achievements: BTreeMap<(u16, u16, u8), Achievement>,
    links: HashMap<u16, Vec<u16>>,
}

fn err(message: impl Into<String>) -> ConfigError {
    ConfigError(message.into())
}

fn flag(e: &Element, name: &str) -> bool {
    matches!(
        e.attrs.get(name).map(String::as_str),
        Some("y" | "yes" | "true" | "1")
    )
}

fn num<T: std::str::FromStr>(e: &Element, name: &str) -> Result<Option<T>, ConfigError> {
    e.attrs
        .get(name)
        .map(|v| {
            v.trim()
                .parse::<T>()
                .map_err(|_| err(format!("bad {name}: {v}")))
        })
        .transpose()
}

fn table<'a>(root: &'a Element, kind: &str) -> Result<&'a [Element], ConfigError> {
    if root.name != "table" || root.attrs.get("type").map(String::as_str) != Some(kind) {
        return Err(err(format!("expected <table type='{kind}'>")));
    }
    Ok(&root.children)
}

/// Decode an LMXBOSLB file to XML text; plain XML passes through.
pub fn table_text(bytes: &[u8]) -> Result<String, ConfigError> {
    if bytes.starts_with(b"LMXBOSLB") {
        let doc = lmxb::decode(bytes).map_err(|e| err(format!("lmxb: {e}")))?;
        lmxb::to_xml_string(&doc).map_err(|e| err(format!("lmxb: {e}")))
    } else {
        String::from_utf8(bytes.to_vec()).map_err(|_| err("table is not UTF-8"))
    }
}

impl Rules {
    pub fn from_files(register: &[u8], achievement: &[u8]) -> Result<Self, ConfigError> {
        Self::from_xml(&table_text(register)?, &table_text(achievement)?)
    }

    pub fn from_xml(register_xml: &str, achievement_xml: &str) -> Result<Self, ConfigError> {
        let mut rules = Rules::default();
        let mut by_alias = HashMap::new();
        let root = parse_limited(register_xml, MAX_TABLE_XML)?;
        for r in table(&root, "achievement-register")? {
            let id: u16 = num(r, "id")?.ok_or_else(|| err("register without id"))?;
            let alias = r.attrs.get("alias").cloned().unwrap_or_default();
            let kind = r.attrs.get("type").cloned().unwrap_or_default();
            let job = job_from_name(r.attrs.get("job").map_or("", String::as_str))
                .ok_or_else(|| err("bad register job"))?;
            let register = Register {
                id,
                job,
                max_value: num(r, "max-value")?.unwrap_or(0),
                persistant: flag(r, "persistant"),
                progress_show: flag(r, "progress-show"),
                increment_from_client: flag(r, "increment-from-client") || kind == "client-only",
                sub_type: num(r, "sub-type")?.unwrap_or(0),
                alias: alias.clone(),
                kind,
            };
            if !alias.is_empty() {
                by_alias.insert(alias, id);
            }
            if rules.registers.insert((id, job), register).is_some() {
                return Err(err(format!("duplicate register {id} job {job}")));
            }
        }
        let root = parse_limited(achievement_xml, MAX_TABLE_XML)?;
        for r in table(&root, "achievement")? {
            let id: u16 = num(r, "id")?.ok_or_else(|| err("achievement without id"))?;
            let step: u16 = num(r, "step")?.unwrap_or(1);
            let job = job_from_name(r.attrs.get("job").map_or("", String::as_str))
                .ok_or_else(|| err("bad achievement job"))?;
            let mut conditions = Vec::new();
            for n in 1..=MAX_CONDITIONS {
                let Some(reference) = r.attrs.get(&format!("register-ref-{n}")) else {
                    continue;
                };
                let register = match by_alias.get(reference) {
                    Some(&id) => id,
                    None => reference
                        .parse::<u16>()
                        .map_err(|_| err(format!("unknown register {reference}")))?,
                };
                let op = r
                    .attrs
                    .get(&format!("register-type-{n}"))
                    .map_or(Some(Op::Above), |t| Op::parse(t))
                    .ok_or_else(|| err("bad register-type"))?;
                let value = num(r, &format!("register-value-{n}"))?.unwrap_or(0);
                conditions.push(Condition {
                    register,
                    op,
                    value,
                });
            }
            let has_reward = r
                .attrs
                .keys()
                .any(|k| k.starts_with("step-complete-reward"));
            let record = Achievement {
                id,
                step,
                job,
                score: num(r, "current-step-score")?.unwrap_or(0),
                deprecated: flag(r, "deprecated"),
                complete_from_client: flag(r, "complete-from-client"),
                has_reward,
                conditions,
            };
            for c in &record.conditions {
                let links = rules.links.entry(c.register).or_default();
                if !links.contains(&id) && links.len() < MAX_LINKS_PER_REGISTER {
                    links.push(id);
                }
            }
            if rules.achievements.insert((id, step, job), record).is_some() {
                return Err(err(format!("duplicate achievement {id} step {step}")));
            }
        }
        Ok(rules)
    }

    /// Register row for a Pc job: (id, job), then (id, job-none).
    pub fn register(&self, id: u16, job: u8) -> Option<&Register> {
        self.registers
            .get(&(id, job))
            .or_else(|| self.registers.get(&(id, 0)))
    }

    /// Achievement ids that use this register (at most 8).
    pub fn linked(&self, register: u16) -> &[u16] {
        self.links.get(&register).map_or(&[], Vec::as_slice)
    }

    /// Lookup order of the daemon: (id, step, job), then (id, step, job-none).
    pub fn record(&self, id: u16, step: u16, job: u8) -> Option<&Achievement> {
        self.achievements
            .get(&(id, step, job))
            .or_else(|| self.achievements.get(&(id, step, 0)))
    }
}
