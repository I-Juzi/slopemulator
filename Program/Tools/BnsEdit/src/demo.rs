//! Synthetic data for running the GUI without game files. No game data.

use crate::backend::*;
use std::collections::BTreeMap;
use std::path::Path;

const GRADES: [&str; 5] = ["Common", "Uncommon", "Rare", "Heroic", "Legendary"];

/// One undo step: field key and (before, after) values.
type Step = ((String, usize, String, usize), Option<(Value, Value)>);

pub struct Demo {
    tables: Vec<(String, usize)>,
    edits: BTreeMap<(String, usize, String, usize), (Value, Value)>,
    undo: Vec<Step>,
    redo: Vec<Step>,
}

impl Demo {
    pub fn new() -> Self {
        let tables = [
            ("item", 5000),
            ("item-brand-tooltip", 300),
            ("npc", 2000),
            ("creatureappearance", 900),
            ("skill3", 3000),
            ("quest", 1200),
            ("store2", 150),
            ("text", 20000),
            ("zone", 400),
            ("mapinfo", 300),
            ("icontexture", 800),
            ("effect", 4000),
        ];
        Self {
            tables: tables.iter().map(|(n, c)| (n.to_string(), *c)).collect(),
            edits: BTreeMap::new(),
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    fn base(&self, table: &str, r: usize) -> Vec<Field> {
        let f = |name: &str, value: Value, min: Option<f64>, max: Option<f64>| Field {
            name: name.into(),
            repeat: 0,
            value,
            original: None,
            min,
            max,
            hidden: false,
            locked: false,
        };
        let mut v = vec![
            f("alias", Value::Text(format!("{table}_{r:05}")), None, None),
            f(
                "name",
                Value::Text(format!("Sample {table} {r}")),
                None,
                None,
            ),
        ];
        match table {
            "item" | "item-brand-tooltip" => v.extend([
                f(
                    "grade",
                    Value::Enum {
                        index: (r % 5) as u64,
                        names: GRADES.map(String::from).to_vec(),
                    },
                    None,
                    None,
                ),
                f(
                    "price",
                    Value::Int((r as i64 % 97) * 100),
                    Some(0.0),
                    Some(2147483647.0),
                ),
                f("can-sell", Value::Bool(!r.is_multiple_of(3)), None, None),
                f(
                    "icon",
                    Value::Ref {
                        table: "icontexture".into(),
                        key: (r % 800) as u64,
                        title: Some(format!("Sample icontexture {}", r % 800)),
                    },
                    None,
                    None,
                ),
                f("weight", Value::Float(1.5), Some(0.0), Some(100.0)),
                f(
                    "level",
                    Value::Int((r % 60) as i64 + 1),
                    Some(1.0),
                    Some(60.0),
                ),
            ]),
            _ => v.extend([
                f(
                    "level",
                    Value::Int((r % 60) as i64 + 1),
                    Some(1.0),
                    Some(60.0),
                ),
                f("enabled", Value::Bool(true), None, None),
            ]),
        }
        let mut raw = f(
            "unknown-31",
            Value::Raw {
                kind: "type_31".into(),
                bytes: vec![0, 1, 2, 3, 0, 0, 0, 0],
            },
            None,
            None,
        );
        raw.hidden = true;
        raw.locked = true;
        v.push(raw);
        v
    }
}

impl Backend for Demo {
    fn game(&self) -> GameInfo {
        GameInfo {
            folder: "Demo data (no game folder)".into(),
            build: "demo".into(),
            files: vec![
                "datafile.bin".into(),
                "datafile64.bin".into(),
                "localfile.bin".into(),
                "localfile64.bin".into(),
            ],
            schema: "demo schema".into(),
        }
    }
    fn tables(&self) -> Vec<TableInfo> {
        self.tables
            .iter()
            .map(|(n, c)| TableInfo {
                name: n.clone(),
                records: *c,
            })
            .collect()
    }
    fn search(&mut self, table: &str, query: &str, limit: usize) -> Vec<usize> {
        let count = self
            .tables
            .iter()
            .find(|(n, _)| n == table)
            .map_or(0, |t| t.1);
        let q = query.to_lowercase();
        (0..count)
            .filter(|&r| {
                q.is_empty()
                    || r.to_string() == q
                    || self.title(table, r).to_lowercase().contains(&q)
            })
            .take(limit)
            .collect()
    }
    fn title(&mut self, table: &str, record: usize) -> String {
        match self.edits.get(&(table.into(), record, "name".into(), 0)) {
            Some((_, Value::Text(t))) => t.clone(),
            _ => format!("Sample {table} {record}"),
        }
    }
    fn subtitle(&mut self, table: &str, record: usize) -> String {
        format!("{table} · id {record}")
    }
    fn fields(&mut self, table: &str, record: usize) -> Vec<Field> {
        let mut v = self.base(table, record);
        for f in &mut v {
            if let Some((old, new)) =
                self.edits
                    .get(&(table.into(), record, f.name.clone(), f.repeat))
            {
                f.original = Some(old.clone());
                f.value = new.clone();
            }
        }
        v
    }
    fn set(
        &mut self,
        table: &str,
        record: usize,
        field: &str,
        repeat: usize,
        value: Value,
    ) -> Result<(), String> {
        let key = (table.to_string(), record, field.to_string(), repeat);
        let old = self
            .base(table, record)
            .into_iter()
            .find(|f| f.name == field && f.repeat == repeat)
            .ok_or("field not found")?
            .value;
        let prev = self.edits.get(&key).cloned();
        if value == old {
            self.edits.remove(&key);
        } else {
            self.edits.insert(key.clone(), (old, value));
        }
        self.undo.push((key, prev));
        self.redo.clear();
        Ok(())
    }
    fn link_key(&mut self, _table: &str, record: usize) -> Option<u64> {
        Some(record as u64)
    }
    fn undo(&mut self) -> bool {
        let Some((key, prev)) = self.undo.pop() else {
            return false;
        };
        let cur = self.edits.get(&key).cloned();
        match prev {
            Some(p) => self.edits.insert(key.clone(), p),
            None => self.edits.remove(&key),
        };
        self.redo.push((key, cur));
        true
    }
    fn redo(&mut self) -> bool {
        let Some((key, next)) = self.redo.pop() else {
            return false;
        };
        let cur = self.edits.get(&key).cloned();
        match next {
            Some(n) => self.edits.insert(key.clone(), n),
            None => self.edits.remove(&key),
        };
        self.undo.push((key, cur));
        true
    }
    fn changes(&self) -> Vec<Change> {
        self.edits
            .iter()
            .map(|((t, r, f, _), (o, n))| Change {
                table: t.clone(),
                record: *r,
                title: format!("Sample {t} {r}"),
                field: f.clone(),
                old: o.clone(),
                new: n.clone(),
            })
            .collect()
    }
    fn problems(&self) -> Vec<Problem> {
        self.edits
            .iter()
            .filter_map(|((t, r, f, _), (_, n))| match n {
                Value::Int(v) if *v < 0 => Some(Problem {
                    table: t.clone(),
                    record: *r,
                    field: f.clone(),
                    message: "must be 0 or more".into(),
                    error: true,
                }),
                Value::Text(s) if s.is_empty() && f == "name" => Some(Problem {
                    table: t.clone(),
                    record: *r,
                    field: f.clone(),
                    message: "name is empty".into(),
                    error: false,
                }),
                _ => None,
            })
            .collect()
    }
    fn is_changed(&self, table: &str, record: usize) -> bool {
        self.edits
            .keys()
            .any(|(t, r, _, _)| t == table && *r == record)
    }
    fn pack(&mut self, _out_dir: &Path) -> Result<PackReport, String> {
        Err("Demo data cannot be packed. Open a game folder first.".into())
    }
}
