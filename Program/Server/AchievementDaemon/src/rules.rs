//! Native build 348 rule loading and derived fields (0x140010600 / 0x140010AF0).
use crate::config::{parse_limited, ConfigError, Element};
use std::collections::{BTreeMap, HashMap};

const MAX_TABLE_XML: usize = 64 << 20;
pub const MAX_LINKS_PER_REGISTER: usize = 8;
pub const MAX_CONDITIONS: usize = 5;
pub type RecordKey = (u16, u16, u8);

/// Full native job table at 0x1400DDEA0. The index is the value.
pub const JOB_NAMES: [&str; 20] = [
    "job-none",
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
    "창술사",
    "뇌전술사",
    "pc-max",
    "소환수-루키",
    "소환수-striker",
    "소환수-defender",
    "소환수-controller",
];

pub fn job_from_name(name: &str) -> Option<u8> {
    if name.is_empty() {
        return Some(0);
    }
    if let Ok(n) = name.parse::<u8>() {
        return Some(n);
    }
    JOB_NAMES.iter().position(|j| *j == name).map(|i| i as u8)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    None,
    Above,
    Below,
    BitsOn,
}
impl Op {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "none" | "0" => Some(Self::None),
            "above" | "1" => Some(Self::Above),
            "below" | "2" => Some(Self::Below),
            "bits-on" | "3" => Some(Self::BitsOn),
            _ => None,
        }
    }
    pub fn holds(self, value: i64, v: i64) -> bool {
        match self {
            Self::None => false,
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
    pub version: u16,
    pub max_value: i64,
    pub persistant: bool,
    pub progress_show: bool,
    pub for_newbie_care: bool,
    pub increment_from_client: bool,
    /// Named duel-type or party-battle-type selector.
    pub sub_type: u8,
}
impl Register {
    pub fn visible(&self) -> bool {
        self.progress_show || self.for_newbie_care
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Condition {
    pub register: u16,
    pub op: Op,
    pub value: i64,
    pub clear: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Achievement {
    pub id: u16,
    pub step: u16,
    pub job: u8,
    pub score: i32,
    pub score_sum: i32,
    pub conditions: Vec<Condition>,
    pub deprecated: bool,
    pub progress_show: bool,
    pub complete_from_client: bool,
    /// Derived nonzero reward type at +146 (item=1, game cash=2).
    pub has_reward: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Rules {
    pub registers: BTreeMap<(u16, u8), Register>,
    pub achievements: BTreeMap<RecordKey, Achievement>,
    links: HashMap<(u16, u8), Vec<RecordKey>>,
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
fn selector(e: &Element, name: &str, names: &[&str]) -> Result<u8, ConfigError> {
    let Some(value) = e.attrs.get(name) else {
        return Ok(0);
    };
    names
        .iter()
        .position(|n| *n == value)
        .map(|n| n as u8)
        .ok_or_else(|| err(format!("bad {name}: {value}")))
}
fn table<'a>(root: &'a Element, kind: &str) -> Result<&'a [Element], ConfigError> {
    if root.name != "table" || root.attrs.get("type").map(String::as_str) != Some(kind) {
        return Err(err(format!("expected <table type='{kind}'>")));
    }
    Ok(&root.children)
}

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
            let sub_type = match kind.as_str() {
                "duel-grade" | "duel-win-count" => {
                    selector(r, "duel-type", &["none", "solo", "team"])?
                }
                "party-battle-grade"
                | "party-battle-win-count"
                | "party-battle-challenge-count" => selector(
                    r,
                    "party-battle-type",
                    &[
                        "none",
                        "occupation-war",
                        "capture-the-flag",
                        "lead-the-ball",
                    ],
                )?,
                _ => 0,
            };
            let transient = matches!(
                kind.as_str(),
                "duel-grade"
                    | "inventory-size"
                    | "wardrobe-size"
                    | "depot-size"
                    | "pc-level"
                    | "pc-mastery-level"
                    | "party-battle-grade"
            );
            let register = Register {
                id,
                job,
                version: num(r, "version")?.unwrap_or(1),
                max_value: num(r, "max-value")?.unwrap_or(0),
                persistant: flag(r, "persistant") && !transient,
                progress_show: flag(r, "progress-show"),
                for_newbie_care: flag(r, "for-newbie-care"),
                increment_from_client: kind == "client-only",
                sub_type,
                alias: alias.clone(),
                kind,
            };
            if !alias.is_empty() {
                by_alias.insert(alias, (id, job));
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
                    break;
                };
                if reference.is_empty() || reference == "0" {
                    break;
                }
                let register = match by_alias.get(reference) {
                    Some(&(id, ref_job)) => {
                        if ref_job != job {
                            return Err(err("achievement/register job mismatch"));
                        }
                        id
                    }
                    None => reference
                        .parse::<u16>()
                        .map_err(|_| err(format!("unknown register {reference}")))?,
                };
                let op = r
                    .attrs
                    .get(&format!("register-type-{n}"))
                    .map_or(Some(Op::None), |t| Op::parse(t))
                    .ok_or_else(|| err("bad register-type"))?;
                conditions.push(Condition {
                    register,
                    op,
                    value: num(r, &format!("register-value-{n}"))?.unwrap_or(0),
                    clear: flag(r, &format!("progress-step-register-clear-{n}")),
                });
            }
            let item_reward = (1..=2).any(|n| {
                r.attrs
                    .get(&format!("step-complete-reward-item-{n}"))
                    .is_some_and(|s| !s.is_empty() && s != "0")
            });
            let has_reward =
                item_reward || num::<i64>(r, "step-complete-reward-game-cash")?.unwrap_or(0) > 0;
            let record = Achievement {
                id,
                step,
                job,
                score: num(r, "current-step-score")?.unwrap_or(0),
                score_sum: 0,
                deprecated: flag(r, "deprecated"),
                progress_show: flag(r, "progress-show"),
                complete_from_client: flag(r, "complete-from-client"),
                has_reward,
                conditions,
            };
            if rules.achievements.insert((id, step, job), record).is_some() {
                return Err(err(format!("duplicate achievement {id} step {step}")));
            }
        }
        rules.normalize()?;
        Ok(rules)
    }

    /// Native preprocessing. Deprecation runs before links and score derivation.
    fn normalize(&mut self) -> Result<(), ConfigError> {
        let deprecated: Vec<_> = self
            .achievements
            .iter()
            .filter(|(_, r)| r.deprecated)
            .map(|(&k, _)| k)
            .collect();
        for (id, step, job) in deprecated {
            for (&(other_id, other_step, other_job), record) in &mut self.achievements {
                if other_id == id && other_job == job && other_step < step {
                    record.deprecated = true;
                }
            }
        }
        let mut sums: HashMap<(u16, u8), i16> = HashMap::new();
        for (&key, record) in &mut self.achievements {
            if record.deprecated {
                continue;
            }
            let sum = sums.entry((record.id, record.job)).or_default();
            *sum = sum.wrapping_add(record.score as i16);
            record.score_sum = i32::from(*sum);
            for condition in &record.conditions {
                let row_key = (condition.register, record.job);
                let Some(register) = self.registers.get_mut(&row_key) else {
                    break;
                };
                let links = self.links.entry(row_key).or_default();
                if links.len() == MAX_LINKS_PER_REGISTER {
                    return Err(err(format!(
                        "too many links for register {} job {}",
                        register.id, register.job
                    )));
                }
                links.push(key);
                register.progress_show |= record.progress_show;
            }
        }
        Ok(())
    }

    pub fn register(&self, id: u16, job: u8) -> Option<&Register> {
        self.registers
            .get(&(id, job))
            .or_else(|| self.registers.get(&(id, 0)))
    }
    pub fn linked(&self, register: u16, job: u8) -> &[RecordKey] {
        let Some(row) = self.register(register, job) else {
            return &[];
        };
        self.links
            .get(&(register, row.job))
            .map_or(&[], Vec::as_slice)
    }
    pub fn record(&self, id: u16, step: u16, job: u8) -> Option<&Achievement> {
        self.achievements
            .get(&(id, step, job))
            .or_else(|| self.achievements.get(&(id, step, 0)))
    }
}
