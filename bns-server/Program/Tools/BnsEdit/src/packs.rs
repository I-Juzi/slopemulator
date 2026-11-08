//! Change packs: named lists of changes, like mods. Each pack can be on or off.
//!
//! The packs are the source of truth. The data layer only holds the result: for every
//! changed field, the value of the last pack that is on (the active pack wins), or the
//! original value when no pack that is on changes it.

use crate::backend::Value;
use serde_json::{Value as J, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// One field: table, record, field, repeat.
pub type Key = (String, usize, String, usize);

#[derive(Clone, Debug, PartialEq)]
pub struct PackChange {
    pub table: String,
    pub record: usize,
    /// Record key when it was changed. A different key on load means the row moved.
    pub key: Option<u64>,
    pub title: String,
    pub field: String,
    pub repeat: usize,
    /// Friendly field name for the list.
    pub label: String,
    pub old: Value,
    pub new: Value,
}

impl PackChange {
    pub fn key(&self) -> Key {
        (
            self.table.clone(),
            self.record,
            self.field.clone(),
            self.repeat,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pack {
    pub name: String,
    pub on: bool,
    pub changes: Vec<PackChange>,
}

impl Pack {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            on: true,
            changes: Vec::new(),
        }
    }

    /// Adds or replaces the change for one field. A change back to the old value removes it.
    pub fn put(&mut self, c: PackChange) {
        let key = c.key();
        let old = match self.changes.iter().position(|x| x.key() == key) {
            Some(i) => self.changes.remove(i).old,
            None => c.old.clone(),
        };
        if !same(&old, &c.new) {
            self.changes.push(PackChange { old, ..c });
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Packs {
    pub list: Vec<Pack>,
    pub active: usize,
}

impl Default for Packs {
    fn default() -> Self {
        Self {
            list: vec![Pack::new("My changes")],
            active: 0,
        }
    }
}

impl Packs {
    pub fn active_mut(&mut self) -> &mut Pack {
        if self.list.is_empty() {
            self.list.push(Pack::new("My changes"));
        }
        self.active = self.active.min(self.list.len() - 1);
        &mut self.list[self.active]
    }

    pub fn active_name(&self) -> String {
        self.list
            .get(self.active)
            .map_or_else(|| "My changes".into(), |p| p.name.clone())
    }

    pub fn total(&self) -> usize {
        self.list.iter().map(|p| p.changes.len()).sum()
    }

    /// The value each touched field must have: the new value of the last pack that is on
    /// (the active pack last, so it wins), else the old value.
    pub fn desired(&self) -> Vec<(PackChange, Value)> {
        let mut order: Vec<usize> = (0..self.list.len()).filter(|&i| i != self.active).collect();
        if self.active < self.list.len() {
            order.push(self.active);
        }
        let mut keys: Vec<Key> = Vec::new();
        let mut map: HashMap<Key, (PackChange, Value)> = HashMap::new();
        for i in order {
            let p = &self.list[i];
            for c in &p.changes {
                let k = c.key();
                let entry = map.entry(k.clone()).or_insert_with(|| {
                    keys.push(k);
                    (c.clone(), c.old.clone())
                });
                if p.on {
                    *entry = (c.clone(), c.new.clone());
                }
            }
        }
        keys.into_iter().filter_map(|k| map.remove(&k)).collect()
    }

    /// A free name like "New pack 2".
    pub fn free_name(&self, base: &str) -> String {
        let taken = |n: &str| self.list.iter().any(|p| p.name == n);
        if !taken(base) {
            return base.into();
        }
        (2..)
            .map(|i| format!("{base} {i}"))
            .find(|n| !taken(n))
            .unwrap()
    }

    pub fn to_json(&self) -> J {
        json!({ "active": self.active, "packs": self.list.iter().map(pack_json).collect::<Vec<_>>() })
    }

    pub fn from_json(v: &J) -> Option<Self> {
        let list: Vec<Pack> = v
            .get("packs")?
            .as_array()?
            .iter()
            .filter_map(pack_from)
            .collect();
        if list.is_empty() {
            return None;
        }
        let active = v.get("active").and_then(J::as_u64).unwrap_or(0) as usize;
        Some(Self {
            active: active.min(list.len() - 1),
            list,
        })
    }

    pub fn file() -> Option<PathBuf> {
        crate::settings::config_dir().map(|d| d.join("packs.json"))
    }

    pub fn load() -> Self {
        Self::file()
            .and_then(|f| std::fs::read(f).ok())
            .and_then(|b| serde_json::from_slice::<J>(&b).ok())
            .and_then(|v| Self::from_json(&v))
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let file = Self::file().ok_or("No settings folder on this computer.")?;
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let tmp = file.with_extension("json.tmp");
        std::fs::write(&tmp, self.to_json().to_string()).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &file).map_err(|e| e.to_string())
    }
}

/// Same value for the game. Ignores names and titles that are only for display.
pub fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Enum { index: x, .. }, Value::Enum { index: y, .. }) => x == y,
        (Value::Ref { key: x, .. }, Value::Ref { key: y, .. }) => x == y,
        (Value::Float(x), Value::Float(y)) => x.to_bits() == y.to_bits(),
        _ => a == b,
    }
}

pub fn value_json(v: &Value) -> J {
    match v {
        Value::Int(i) => json!({ "int": i }),
        Value::Float(f) => json!({ "float": f }),
        Value::Bool(b) => json!({ "bool": b }),
        Value::Enum { index, names } => json!({ "enum": index, "names": names }),
        Value::Ref { table, key, title } => json!({ "ref": table, "key": key, "title": title }),
        Value::Text(t) => json!({ "text": t }),
        Value::Raw { kind, .. } => json!({ "raw": kind }),
    }
}

pub fn value_from(v: &J) -> Option<Value> {
    if let Some(i) = v.get("int") {
        return i.as_i64().map(Value::Int);
    }
    if let Some(f) = v.get("float") {
        return f.as_f64().map(|f| Value::Float(f as f32));
    }
    if let Some(b) = v.get("bool") {
        return b.as_bool().map(Value::Bool);
    }
    if let Some(i) = v.get("enum") {
        let names = v
            .get("names")
            .and_then(J::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|n| n.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        return i.as_u64().map(|index| Value::Enum { index, names });
    }
    if let Some(t) = v.get("ref") {
        return Some(Value::Ref {
            table: t.as_str()?.into(),
            key: v.get("key")?.as_u64()?,
            title: v.get("title").and_then(J::as_str).map(String::from),
        });
    }
    if let Some(t) = v.get("text") {
        return t.as_str().map(|t| Value::Text(t.into()));
    }
    None
}

fn pack_json(p: &Pack) -> J {
    json!({
        "name": p.name,
        "on": p.on,
        "changes": p.changes.iter().map(|c| json!({
            "table": c.table, "record": c.record, "key": c.key, "title": c.title,
            "field": c.field, "repeat": c.repeat, "label": c.label,
            "old": value_json(&c.old), "new": value_json(&c.new),
        })).collect::<Vec<_>>(),
    })
}

fn pack_from(v: &J) -> Option<Pack> {
    let s = |k: &str| v.get(k).and_then(J::as_str).map(String::from);
    let changes = v
        .get("changes")
        .and_then(J::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|c| {
                    let s = |k: &str| c.get(k).and_then(J::as_str).map(String::from);
                    Some(PackChange {
                        table: s("table")?,
                        record: c.get("record")?.as_u64()? as usize,
                        key: c.get("key").and_then(J::as_u64),
                        title: s("title").unwrap_or_default(),
                        field: s("field")?,
                        repeat: c.get("repeat").and_then(J::as_u64).unwrap_or(0) as usize,
                        label: s("label").unwrap_or_default(),
                        old: value_from(c.get("old")?)?,
                        new: value_from(c.get("new")?)?,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Some(Pack {
        name: s("name")?,
        on: v.get("on").and_then(J::as_bool).unwrap_or(true),
        changes,
    })
}

/// Writes one pack as a file to share.
pub fn export(p: &Pack, file: &Path) -> Result<(), String> {
    let v = json!({ "bnsedit-pack": 1, "pack": pack_json(p) });
    std::fs::write(file, serde_json::to_string_pretty(&v).unwrap_or_default())
        .map_err(|e| format!("Cannot write {}: {e}", file.display()))
}

/// Reads a shared pack file.
pub fn import(file: &Path) -> Result<Pack, String> {
    let bytes = std::fs::read(file).map_err(|e| format!("Cannot read {}: {e}", file.display()))?;
    let v: J = serde_json::from_slice(&bytes).map_err(|_| "This is not a bnsedit pack file.")?;
    v.get("pack")
        .and_then(pack_from)
        .ok_or_else(|| "This is not a bnsedit pack file.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch(record: usize, old: i64, new: i64) -> PackChange {
        PackChange {
            table: "item".into(),
            record,
            key: Some(record as u64 + 100),
            title: format!("Item {record}"),
            field: "price".into(),
            repeat: 0,
            label: "Sell price".into(),
            old: Value::Int(old),
            new: Value::Int(new),
        }
    }

    #[test]
    fn put_replaces_and_removes() {
        let mut p = Pack::new("a");
        p.put(ch(1, 10, 20));
        p.put(ch(1, 20, 30));
        assert_eq!(p.changes.len(), 1);
        assert_eq!(p.changes[0].old, Value::Int(10));
        assert_eq!(p.changes[0].new, Value::Int(30));
        p.put(ch(1, 30, 10));
        assert!(p.changes.is_empty());
    }

    #[test]
    fn desired_follows_on_off_and_active() {
        let mut packs = Packs::default();
        packs.list[0].put(ch(1, 10, 20));
        let mut b = Pack::new("b");
        b.put(ch(1, 10, 50));
        b.put(ch(2, 5, 6));
        packs.list.push(b);
        packs.active = 0;
        let d = packs.desired();
        let get = |d: &[(PackChange, Value)], r| d.iter().find(|x| x.0.record == r).unwrap().1.clone();
        assert_eq!(get(&d, 1), Value::Int(20), "active pack wins");
        assert_eq!(get(&d, 2), Value::Int(6));
        packs.active = 1;
        assert_eq!(get(&packs.desired(), 1), Value::Int(50));
        packs.list[1].on = false;
        let d = packs.desired();
        assert_eq!(get(&d, 1), Value::Int(20));
        assert_eq!(get(&d, 2), Value::Int(5), "off pack gives the old value back");
    }

    #[test]
    fn json_round_trip() {
        let mut packs = Packs::default();
        packs.list[0].put(ch(1, 10, 20));
        let mut c = ch(2, 0, 0);
        c.old = Value::Enum { index: 1, names: vec!["a".into(), "b".into()] };
        c.new = Value::Enum { index: 0, names: vec!["a".into(), "b".into()] };
        packs.list[0].put(c);
        let mut c = ch(3, 0, 0);
        c.old = Value::Ref { table: "text".into(), key: 7, title: Some("x".into()) };
        c.new = Value::Text("hello \"there\"".into());
        packs.list[0].put(c);
        packs.list.push(Pack { name: "off".into(), on: false, changes: vec![] });
        let back = Packs::from_json(&packs.to_json()).unwrap();
        assert_eq!(back, packs);
        let file = std::env::temp_dir().join(format!("bnsedit-pack-{}.json", std::process::id()));
        export(&packs.list[0], &file).unwrap();
        assert_eq!(import(&file).unwrap(), packs.list[0]);
        std::fs::remove_file(file).unwrap();
    }
}
