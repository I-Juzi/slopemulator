//! "Change many at once": a rule picks records by up to two conditions and changes one field.

use crate::backend::Value;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    Is,
    IsNot,
    AtLeast,
    AtMost,
    Has,
}

impl Op {
    pub const ALL: [Op; 5] = [Op::Is, Op::IsNot, Op::AtLeast, Op::AtMost, Op::Has];
    pub fn label(self) -> &'static str {
        match self {
            Op::Is => "is",
            Op::IsNot => "is not",
            Op::AtLeast => "is at least",
            Op::AtMost => "is at most",
            Op::Has => "has the text",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum How {
    /// Set to this value.
    Set,
    /// Multiply by this percent. 50 = half.
    Percent,
    /// Add this number (a negative number takes away).
    Add,
}

impl How {
    pub const ALL: [How; 3] = [How::Set, How::Percent, How::Add];
    pub fn label(self) -> &'static str {
        match self {
            How::Set => "set it to",
            How::Percent => "make it this % of now",
            How::Add => "add",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cond {
    pub field: String,
    pub op: Op,
    pub value: String,
}

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Int(i) => Some(*i as f64),
        Value::Float(f) => Some(*f as f64),
        Value::Bool(b) => Some(*b as u8 as f64),
        Value::Enum { index, .. } => Some(*index as f64),
        Value::Ref { key, .. } => Some(*key as f64),
        _ => None,
    }
}

fn text_of(v: &Value) -> String {
    match v {
        Value::Text(t) => t.clone(),
        Value::Enum { index, names } => names.get(*index as usize).cloned().unwrap_or_default(),
        Value::Ref { title, key, .. } => title.clone().unwrap_or_else(|| key.to_string()),
        Value::Bool(b) => if *b { "yes" } else { "no" }.into(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Raw { .. } => String::new(),
    }
}

/// Parses what the user typed for a field of this kind.
pub fn parse(like: &Value, s: &str) -> Option<Value> {
    let s = s.trim();
    match like {
        Value::Int(_) => s.parse::<i64>().ok().map(Value::Int),
        Value::Float(_) => s.parse::<f32>().ok().map(Value::Float),
        Value::Bool(_) => match s.to_lowercase().as_str() {
            "yes" | "true" | "1" | "on" => Some(Value::Bool(true)),
            "no" | "false" | "0" | "off" => Some(Value::Bool(false)),
            _ => None,
        },
        Value::Enum { names, .. } => names
            .iter()
            .position(|n| n.eq_ignore_ascii_case(s))
            .map(|i| i as u64)
            .or_else(|| s.parse().ok())
            .map(|index| Value::Enum {
                index,
                names: names.clone(),
            }),
        Value::Ref { table, .. } => s.parse().ok().map(|key| Value::Ref {
            table: table.clone(),
            key,
            title: None,
        }),
        Value::Text(_) => Some(Value::Text(s.into())),
        Value::Raw { .. } => None,
    }
}

impl Cond {
    pub fn matches(&self, v: &Value) -> bool {
        let want = self.value.trim();
        match self.op {
            Op::Has => text_of(v).to_lowercase().contains(&want.to_lowercase()),
            Op::Is | Op::IsNot => {
                let eq = match parse(v, want) {
                    Some(w) => crate::packs::same(&w, v),
                    None => text_of(v).eq_ignore_ascii_case(want),
                };
                eq == (self.op == Op::Is)
            }
            Op::AtLeast | Op::AtMost => {
                let (Some(a), Ok(b)) = (num(v), want.parse::<f64>()) else {
                    return false;
                };
                if self.op == Op::AtLeast { a >= b } else { a <= b }
            }
        }
    }
}

/// The new value for `cur`, or `None` when the rule does not change it.
pub fn apply(how: How, input: &str, cur: &Value, min: Option<f64>, max: Option<f64>) -> Option<Value> {
    let clamp = |x: f64| {
        let x = min.map_or(x, |m| x.max(m));
        max.map_or(x, |m| x.min(m))
    };
    let out = match (how, cur) {
        (How::Set, _) => parse(cur, input)?,
        (How::Percent, Value::Int(i)) => {
            Value::Int(clamp((*i as f64 * input.trim().parse::<f64>().ok()? / 100.0).round()) as i64)
        }
        (How::Percent, Value::Float(f)) => {
            Value::Float(clamp(*f as f64 * input.trim().parse::<f64>().ok()? / 100.0) as f32)
        }
        (How::Add, Value::Int(i)) => {
            Value::Int(clamp(*i as f64 + input.trim().parse::<f64>().ok()?.round()) as i64)
        }
        (How::Add, Value::Float(f)) => {
            Value::Float(clamp(*f as f64 + input.trim().parse::<f64>().ok()?) as f32)
        }
        _ => return None,
    };
    (!crate::packs::same(&out, cur)).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditions() {
        let c = |op, v: &str| Cond { field: "level".into(), op, value: v.into() };
        assert!(c(Op::Is, "50").matches(&Value::Int(50)));
        assert!(!c(Op::IsNot, "50").matches(&Value::Int(50)));
        assert!(c(Op::AtLeast, "50").matches(&Value::Int(60)));
        assert!(!c(Op::AtMost, "50").matches(&Value::Int(60)));
        let grade = Value::Enum { index: 2, names: vec!["Common".into(), "Rare".into(), "Heroic".into()] };
        assert!(c(Op::Is, "heroic").matches(&grade));
        assert!(c(Op::Is, "2").matches(&grade));
        assert!(c(Op::Has, "SWO").matches(&Value::Text("Long sword".into())));
        assert!(c(Op::Is, "yes").matches(&Value::Bool(true)));
    }

    #[test]
    fn changes() {
        assert_eq!(apply(How::Percent, "50", &Value::Int(1001), None, None), Some(Value::Int(501)));
        assert_eq!(apply(How::Percent, "100", &Value::Int(7), None, None), None);
        assert_eq!(apply(How::Add, "-20", &Value::Int(10), Some(0.0), None), Some(Value::Int(0)));
        assert_eq!(apply(How::Set, "no", &Value::Bool(true), None, None), Some(Value::Bool(false)));
        assert_eq!(apply(How::Set, "x", &Value::Int(1), None, None), None);
        assert_eq!(apply(How::Percent, "50", &Value::Text("a".into()), None, None), None);
    }
}
