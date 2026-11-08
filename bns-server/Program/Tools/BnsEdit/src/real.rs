//! Backend over real game files, through `bnsmodel`.

use crate::backend::*;
use bnsdata::dat::{Role, Width};
use bnsmodel::{Edit, FieldView, Project, Severity, TableId};
use std::path::{Path, PathBuf};

/// Where the game files and schemas are.
#[derive(Clone, Debug, Default)]
pub struct OpenSpec {
    pub game: PathBuf,
    pub schema32: Option<PathBuf>,
    pub schema64: Option<PathBuf>,
}

pub struct Real {
    project: Project,
    spec: OpenSpec,
    /// GUI name, model id, record count.
    tables: Vec<(String, TableId, usize)>,
    files: Vec<String>,
    /// Row titles. Cleared on every change.
    titles: std::collections::HashMap<(String, usize), String>,
    /// Last record shown. Cleared on every change.
    last: Option<((String, usize), Vec<Field>)>,
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// Removes markup like `<font name="x">` and trims.
fn plain(s: &str) -> String {
    let mut out = String::new();
    let mut tag = false;
    for c in s.chars() {
        match c {
            '<' => tag = true,
            '>' if tag => tag = false,
            c if !tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn short_name(p: &Option<PathBuf>) -> Option<String> {
    p.as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
}

impl Real {
    /// Opens the game folder. Slow for big files: call it on a worker thread.
    pub fn open(spec: OpenSpec) -> Result<Self, String> {
        if spec.schema32.is_none() && spec.schema64.is_none() {
            return Err("Add a schema first (a TableDefs folder).".into());
        }
        let preferred = if spec.schema64.is_some() {
            Width::Bits64
        } else {
            Width::Bits32
        };
        let project = Project::open_with_schemas(
            &spec.game,
            preferred,
            spec.schema32.as_deref(),
            spec.schema64.as_deref(),
        )
        .map_err(err)?;
        let infos = project.tables();
        let mut tables: Vec<(String, TableId, usize)> = Vec::new();
        for t in &infos {
            let clash = infos.iter().filter(|o| o.name == t.name).count() > 1;
            let name = if clash && t.role == Role::Local {
                format!("{} (local)", t.name)
            } else {
                t.name.clone()
            };
            tables.push((name, t.id, t.record_count));
        }
        tables.sort_by(|a, b| a.0.cmp(&b.0));
        let files = [
            "datafile.bin",
            "datafile64.bin",
            "localfile.bin",
            "localfile64.bin",
        ]
        .iter()
        .filter(|f| spec.game.join(f).is_file())
        .map(|f| f.to_string())
        .collect();
        Ok(Self {
            project,
            spec,
            tables,
            files,
            titles: Default::default(),
            last: None,
        })
    }

    fn id(&self, table: &str) -> Option<(TableId, usize)> {
        self.tables
            .iter()
            .find(|t| t.0 == table)
            .map(|t| (t.1, t.2))
    }

    fn name(&self, id: TableId) -> String {
        self.tables
            .iter()
            .find(|t| t.1 == id)
            .map_or_else(|| format!("table {}", id.0), |t| t.0.clone())
    }

    fn view(&self, table: &str, record: usize) -> Result<Vec<FieldView>, String> {
        let (id, _) = self.id(table).ok_or("table not found")?;
        self.project.record(id, record).map_err(err)
    }

    fn to_gui(&self, f: &FieldView) -> Value {
        self.convert(f, true)
    }

    /// `titles` false skips the slow lookup of link titles.
    fn convert(&self, f: &FieldView, titles: bool) -> Value {
        match &f.value {
            bnsmodel::Value::Int(v) => Value::Int(*v),
            bnsmodel::Value::Float(v) => Value::Float(*v),
            bnsmodel::Value::Bool(v) => Value::Bool(*v),
            bnsmodel::Value::Enum { index, .. } => Value::Enum {
                index: *index,
                names: f.hint.enum_names.clone(),
            },
            bnsmodel::Value::Ref {
                table,
                key,
                resolved,
            } => {
                let mut k = [0u8; 8];
                for (d, s) in k.iter_mut().zip(key) {
                    *d = *s;
                }
                let key = u64::from_le_bytes(k);
                Value::Ref {
                    table: f
                        .hint
                        .target_table
                        .clone()
                        .unwrap_or_else(|| format!("type {table}")),
                    key,
                    title: if key == 0 {
                        Some("(none)".into())
                    } else if titles {
                        resolved.and_then(|r| self.nice_title(r.table, r.index))
                    } else {
                        None
                    },
                }
            }
            bnsmodel::Value::Text(u) | bnsmodel::Value::Native { text: u, .. } => {
                Value::Text(String::from_utf16_lossy(u))
            }
            bnsmodel::Value::Raw { kind, bytes } => Value::Raw {
                kind: kind.clone(),
                bytes: bytes.clone(),
            },
        }
    }

    fn build_fields(&mut self, table: &str, record: usize) -> Vec<Field> {
        let Ok(views) = self.view(table, record) else {
            return Vec::new();
        };
        let changes = self.project.changes().unwrap_or_default();
        let (id, _) = self.id(table).unwrap_or((TableId(usize::MAX), 0));
        views
            .iter()
            .map(|f| {
                let original = changes
                    .iter()
                    .find(|c| {
                        c.table == id
                            && c.record == record
                            && c.field == f.name
                            && c.repeat == f.repeat_index
                    })
                    .map(|c| {
                        let mut old = f.clone();
                        old.value = c.old.clone();
                        self.to_gui(&old)
                    });
                let (min, max) = match (f.hint.int_range, f.hint.float_range) {
                    (Some((lo, hi)), _) => (Some(lo as f64), Some(hi as f64)),
                    (_, Some((lo, hi))) => (Some(lo as f64), Some(hi as f64)),
                    _ => (None, None),
                };
                Field {
                    name: f.name.clone(),
                    repeat: f.repeat_index,
                    value: self.to_gui(f),
                    original,
                    min,
                    max,
                    hidden: f.hidden || f.deprecated,
                    locked: !f.editable,
                }
            })
            .collect()
    }

    fn set_inner(
        &mut self,
        table: &str,
        record: usize,
        field: &str,
        repeat: usize,
        value: Value,
    ) -> Result<(), String> {
        let (id, _) = self.id(table).ok_or("table not found")?;
        let views = self.project.record(id, record).map_err(err)?;
        let view = views
            .iter()
            .find(|f| f.name == field && f.repeat_index == repeat)
            .ok_or("field not found")?;
        let edit = Edit {
            table: id,
            record,
            field: field.into(),
            repeat,
            old: view.value.clone(),
            new: Self::to_model(&view.value, value)?,
        };
        if let Some(p) = self
            .project
            .validate(&edit)
            .map_err(err)?
            .into_iter()
            .find(|p| p.severity == Severity::Error)
        {
            return Err(format!("{}: {}", p.field, p.message));
        }
        self.project.apply(edit).map_err(err)
    }

    /// A name for people: the linked name text when there is one, else the code name.
    fn nice_title(&self, id: TableId, record: usize) -> Option<String> {
        let views = self.project.record(id, record).ok()?;
        for name in ["name2", "name", "text"] {
            let Some(f) = views.iter().find(|f| f.name == name && f.repeat_index == 0) else {
                continue;
            };
            let text = match &f.value {
                bnsmodel::Value::Ref {
                    resolved: Some(r), ..
                } => self.project.record(r.table, r.index).ok().and_then(|tv| {
                    tv.iter()
                        .find(|t| t.name == "text")
                        .and_then(|t| t.value.text().ok())
                }),
                v => v.text().ok(),
            };
            if let Some(t) = text.map(|t| plain(&t)).filter(|t| !t.is_empty()) {
                return Some(t);
            }
        }
        self.project.record_title(id, record).ok()
    }

    fn invalidate(&mut self) {
        self.titles.clear();
        self.last = None;
    }

    /// Converts a GUI value back. Keeps parts the GUI does not show (native word `a`).
    fn to_model(old: &bnsmodel::Value, new: Value) -> Result<bnsmodel::Value, String> {
        use bnsmodel::Value as M;
        Ok(match (old, new) {
            (M::Int(_), Value::Int(v)) => M::Int(v),
            (M::Float(_), Value::Float(v)) => M::Float(v),
            (M::Bool(_), Value::Bool(v)) => M::Bool(v),
            (M::Enum { .. }, Value::Enum { index, .. }) => M::Enum { index, name: None },
            (M::Text(_), Value::Text(s)) => M::Text(s.encode_utf16().collect()),
            (M::Native { a, .. }, Value::Text(s)) => M::Native {
                a: *a,
                text: s.encode_utf16().collect(),
            },
            (
                M::Ref {
                    table, key: old, ..
                },
                Value::Ref { key, .. },
            ) => {
                let bytes = key.to_le_bytes();
                if old.len() > 8 || bytes[old.len()..].iter().any(|b| *b != 0) {
                    return Err("This link has an unusual size and cannot be changed.".into());
                }
                M::Ref {
                    table: *table,
                    key: bytes[..old.len()].to_vec(),
                    resolved: None,
                }
            }
            _ => return Err("This field cannot be changed.".into()),
        })
    }
}

impl Backend for Real {
    fn game(&self) -> GameInfo {
        let schema = [
            short_name(&self.spec.schema32),
            short_name(&self.spec.schema64),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" + ");
        GameInfo {
            folder: self.spec.game.file_name().map_or_else(
                || self.spec.game.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            ),
            build: format!("{} files", self.files.len()),
            files: self.files.clone(),
            schema,
        }
    }

    fn tables(&self) -> Vec<TableInfo> {
        self.tables
            .iter()
            .map(|(n, _, c)| TableInfo {
                name: n.clone(),
                records: *c,
            })
            .collect()
    }

    fn search(&mut self, table: &str, query: &str, limit: usize) -> Vec<usize> {
        let Some((id, count)) = self.id(table) else {
            return Vec::new();
        };
        let q = query.trim();
        if q.is_empty() {
            return (0..count.min(limit)).collect();
        }
        self.project
            .search(id, q, limit.min(2000))
            .unwrap_or_default()
    }

    fn title(&mut self, table: &str, record: usize) -> String {
        let key = (table.to_string(), record);
        if let Some(t) = self.titles.get(&key) {
            return t.clone();
        }
        let t = self
            .id(table)
            .and_then(|(id, _)| self.nice_title(id, record))
            .unwrap_or_else(|| format!("row {record}"));
        self.titles.insert(key, t.clone());
        t
    }

    fn fields(&mut self, table: &str, record: usize) -> Vec<Field> {
        let key = (table.to_string(), record);
        if let Some((k, f)) = &self.last
            && *k == key
        {
            return f.clone();
        }
        let fields = self.build_fields(table, record);
        self.last = Some((key, fields.clone()));
        fields
    }

    fn set(
        &mut self,
        table: &str,
        record: usize,
        field: &str,
        repeat: usize,
        value: Value,
    ) -> Result<(), String> {
        self.invalidate();
        self.set_inner(table, record, field, repeat, value)
    }

    fn undo(&mut self) -> bool {
        self.invalidate();
        self.project.undo().unwrap_or(false)
    }

    fn redo(&mut self) -> bool {
        self.invalidate();
        self.project.redo().unwrap_or(false)
    }

    fn link_key(&mut self, table: &str, record: usize) -> Option<u64> {
        let (id, _) = self.id(table)?;
        self.project
            .record_key(id, record)
            .ok()
            .map(u64::from_le_bytes)
    }

    fn subtitle(&mut self, table: &str, record: usize) -> String {
        let code = self
            .id(table)
            .and_then(|(id, _)| self.project.record_title(id, record).ok())
            .unwrap_or_default();
        if code.is_empty() || code == self.title(table, record) {
            format!("{table} · row {record}")
        } else {
            format!("{code} · row {record}")
        }
    }

    fn values(&mut self, table: &str, record: usize, names: &[String]) -> Vec<Option<Value>> {
        let Ok(views) = self.view(table, record) else {
            return vec![None; names.len()];
        };
        names
            .iter()
            .map(|n| {
                views
                    .iter()
                    .find(|f| f.name == *n && f.repeat_index == 0)
                    .map(|f| self.convert(f, false))
            })
            .collect()
    }

    fn value(&mut self, table: &str, record: usize, field: &str, repeat: usize) -> Option<Value> {
        let views = self.view(table, record).ok()?;
        views
            .iter()
            .find(|f| f.name == field && f.repeat_index == repeat)
            .map(|f| self.convert(f, false))
    }

    fn changes(&self) -> Vec<Change> {
        self.project
            .changes()
            .unwrap_or_default()
            .into_iter()
            .map(|c| Change {
                table: self.name(c.table),
                record: c.record,
                title: String::new(),
                field: c.field,
                old: Value::Raw {
                    kind: String::new(),
                    bytes: Vec::new(),
                },
                new: Value::Raw {
                    kind: String::new(),
                    bytes: Vec::new(),
                },
            })
            .collect()
    }

    fn problems(&self) -> Vec<Problem> {
        self.project
            .problems()
            .unwrap_or_default()
            .into_iter()
            .map(|p| Problem {
                table: self.name(p.table),
                record: p.record,
                field: p.field,
                message: p.message,
                error: p.severity == Severity::Error,
            })
            .collect()
    }

    fn is_changed(&self, table: &str, record: usize) -> bool {
        let Some((id, _)) = self.id(table) else {
            return false;
        };
        self.project
            .changes()
            .unwrap_or_default()
            .iter()
            .any(|c| c.table == id && c.record == record)
    }

    fn pack(&mut self, out_dir: &Path) -> Result<PackReport, String> {
        let report = self.project.pack(out_dir).map_err(err)?;
        Ok(PackReport {
            files: report
                .files
                .into_iter()
                .map(|f| {
                    (
                        f.path.display().to_string(),
                        f.changed_tables.len(),
                        f.bytes,
                    )
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Operator-only. Needs BNSEDIT_GAME, BNSEDIT_SCHEMA32, BNSEDIT_SCHEMA64. Prints counts only.
    #[test]
    #[ignore = "needs real game files"]
    fn edit_pack_reopen() {
        let var = |k: &str| std::env::var_os(k).map(PathBuf::from);
        let spec = OpenSpec {
            game: var("BNSEDIT_GAME").expect("BNSEDIT_GAME"),
            schema32: var("BNSEDIT_SCHEMA32"),
            schema64: var("BNSEDIT_SCHEMA64"),
        };
        let mut real = Real::open(spec.clone()).expect("open");
        let price = |r: &mut Real| match r.fields("item", 0).into_iter().find(|f| f.name == "price")
        {
            Some(Field {
                value: Value::Int(v),
                ..
            }) => v,
            _ => panic!("no int price field"),
        };
        let before = price(&mut real);
        real.set("item", 0, "price", 0, Value::Int(before + 1))
            .expect("set");
        assert_eq!(price(&mut real), before + 1);
        assert_eq!(real.changes().len(), 1);
        assert!(real.is_changed("item", 0));
        assert!(real.undo());
        assert_eq!(price(&mut real), before);
        assert!(real.redo());
        assert_eq!(price(&mut real), before + 1);
        // Change the first resolved link to another record of the same target table.
        let link = real
            .fields("item", 0)
            .into_iter()
            .find(|f| matches!(&f.value, Value::Ref { key, title: Some(_), table } if *key != 0 && real.id(table).is_some()))
            .expect("item 0 has a resolved link");
        let Value::Ref {
            table: target,
            key: old_key,
            ..
        } = link.value.clone()
        else {
            unreachable!()
        };
        let pick = (0..10)
            .find(|&r| {
                real.link_key(&target, r)
                    .is_some_and(|k| k != 0 && k != old_key)
            })
            .expect("other target");
        let new_key = real.link_key(&target, pick).unwrap();
        real.set(
            "item",
            0,
            &link.name,
            link.repeat,
            Value::Ref {
                table: target.clone(),
                key: new_key,
                title: None,
            },
        )
        .expect("set link");
        let link_now = |r: &mut Real| {
            r.fields("item", 0)
                .into_iter()
                .find(|f| f.name == link.name && f.repeat == link.repeat)
                .map(|f| f.value)
        };
        assert!(
            matches!(link_now(&mut real), Some(Value::Ref { key, title: Some(_), .. }) if key == new_key),
            "link resolves"
        );
        println!("link field changed and resolves (target table {target})");
        let out = std::env::temp_dir().join(format!("bnsedit-pack-{}", std::process::id()));
        let report = real.pack(&out).expect("pack");
        println!(
            "files={} changed_tables={:?}",
            report.files.len(),
            report.files.iter().map(|f| f.1).collect::<Vec<_>>()
        );
        drop(real);
        for (file, w32) in [("datafile.bin", true), ("datafile64.bin", false)] {
            let one = out.join(if w32 { "only32" } else { "only64" });
            std::fs::create_dir(&one).expect("subfolder");
            std::fs::hard_link(out.join(file), one.join(file)).expect("link");
            let mut again = Real::open(OpenSpec {
                game: one,
                schema32: spec.schema32.clone().filter(|_| w32),
                schema64: spec.schema64.clone().filter(|_| !w32),
            })
            .expect("reopen packed");
            assert_eq!(price(&mut again), before + 1, "packed value in {file}");
            assert!(
                matches!(link_now(&mut again), Some(Value::Ref { key, .. }) if key == new_key),
                "packed link in {file}"
            );
        }
        std::fs::remove_dir_all(&out).ok();
        println!("price edit packed and reopened in both widths");
    }
}
