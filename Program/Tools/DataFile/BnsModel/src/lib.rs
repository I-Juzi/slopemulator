//! Typed, GUI-independent edits over read-only client containers.
//! Record indexes use stored order. Table IDs are local to one Project.
use bnsdata::{
    dat::{Container, Role, Width},
    table::{decode_fields_with_pool, RawValue, Record, TableBody, Variant},
    tabledef::{Attribute, Definitions, TableDefinition},
};
use std::{cell::RefCell, collections::BTreeMap};
mod edits;
mod files;
pub use bnsdata::{Error, Result};
pub use files::{PackFile, PackReport};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableId(pub usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordId {
    pub table: TableId,
    pub index: usize,
}
#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Float(f32),
    Bool(bool),
    Enum {
        index: u64,
        name: Option<String>,
    },
    Ref {
        table: i32,
        key: Vec<u8>,
        resolved: Option<RecordId>,
    },
    Text(Vec<u16>),
    Native {
        a: u32,
        text: Vec<u16>,
    },
    Raw {
        kind: String,
        bytes: Vec<u8>,
    },
}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a.to_bits() == b.to_bits(),
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Enum { index: a, .. }, Self::Enum { index: b, .. }) => a == b,
            (
                Self::Ref {
                    table: a, key: x, ..
                },
                Self::Ref {
                    table: b, key: y, ..
                },
            ) => a == b && x == y,
            (Self::Text(a), Self::Text(b)) => a == b,
            (Self::Native { a, text: x }, Self::Native { a: b, text: y }) => a == b && x == y,
            (Self::Raw { kind: a, bytes: x }, Self::Raw { kind: b, bytes: y }) => a == b && x == y,
            _ => false,
        }
    }
}
impl Eq for Value {}
impl Value {
    pub fn text(&self) -> Result<String> {
        let units = match self {
            Self::Text(v) | Self::Native { text: v, .. } => v,
            _ => return Err(Error::Invalid("value is not text")),
        };
        String::from_utf16(units).map_err(|_| Error::Invalid("invalid UTF-16 text"))
    }
}
#[derive(Clone, Debug)]
pub struct Hint {
    pub int_range: Option<(i64, i64)>,
    pub float_range: Option<(f32, f32)>,
    pub enum_names: Vec<String>,
    pub target_table: Option<String>,
    pub target_element: i32,
}
#[derive(Clone, Debug)]
pub struct FieldView {
    pub name: String,
    pub repeat_index: usize,
    pub kind: String,
    pub value: Value,
    pub editable: bool,
    pub hint: Hint,
    pub deprecated: bool,
    pub hidden: bool,
}
#[derive(Clone, Debug)]
pub struct TableInfo {
    pub id: TableId,
    pub type_id: u16,
    pub name: String,
    pub record_count: usize,
    pub compressed: bool,
    pub role: Role,
    pub width: Width,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    pub table: TableId,
    pub record: usize,
    pub field: String,
    pub repeat: usize,
    pub old: Value,
    pub new: Value,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}
#[derive(Clone, Debug)]
pub struct Problem {
    pub table: TableId,
    pub record: usize,
    pub field: String,
    pub repeat: usize,
    pub severity: Severity,
    pub message: String,
}
struct Source {
    container: Container,
}
struct Physical {
    file: usize,
    slot: usize,
    body: TableBody,
    locations: Vec<(usize, usize)>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct FieldKey {
    physical: usize,
    record: usize,
    field: String,
    repeat: usize,
}
#[derive(Clone)]
struct Write {
    key: FieldKey,
    old: Value,
    new: Value,
}
#[derive(Clone)]
struct Transaction {
    writes: Vec<Write>,
}
type Cache = BTreeMap<(usize, usize), Vec<FieldView>>;
type KeyIndex = BTreeMap<[u8; 8], usize>;
pub struct Project {
    sources: Vec<Source>,
    definitions: [Option<Definitions>; 2],
    physical: Vec<Physical>,
    logical: Vec<usize>,
    cache: RefCell<Cache>,
    keys: RefCell<BTreeMap<usize, KeyIndex>>,
    overlay: BTreeMap<FieldKey, Value>,
    undo: Vec<Transaction>,
    redo: Vec<Transaction>,
}
fn wi(width: Width) -> usize {
    usize::from(width == Width::Bits64)
}
fn bad(message: &'static str) -> Error {
    Error::Invalid(message)
}
impl Physical {
    fn record_pool(&self, index: usize) -> Result<(&Record, &[u8])> {
        match &self.body.variant {
            Variant::Plain(b) => b.records.get(index).map(|r| (r, b.extra.as_slice())),
            Variant::Compressed(b) => self
                .locations
                .get(index)
                .and_then(|&(block, slot)| b.blocks.get(block)?.records.get(slot))
                .map(|r| (r, r.extra.as_slice())),
        }
        .ok_or(bad("record index outside table"))
    }
}
impl Project {
    /// Construct from owned containers and width-specific definitions. No typed fields are decoded.
    pub fn from_containers(
        containers: Vec<Container>,
        definitions32: Option<Definitions>,
        definitions64: Option<Definitions>,
        preferred: Width,
    ) -> Result<Self> {
        if containers.is_empty() {
            return Err(bad("project has no containers"));
        }
        let mut project = Self {
            sources: Vec::new(),
            definitions: [definitions32, definitions64],
            physical: Vec::new(),
            logical: Vec::new(),
            cache: RefCell::new(BTreeMap::new()),
            keys: RefCell::new(BTreeMap::new()),
            overlay: BTreeMap::new(),
            undo: Vec::new(),
            redo: Vec::new(),
        };
        for container in containers {
            if project
                .sources
                .iter()
                .any(|s| s.container.width == container.width && s.container.role == container.role)
            {
                return Err(bad("duplicate container role and width"));
            }
            let file = project.sources.len();
            let defs = project.definitions[wi(container.width)]
                .as_ref()
                .ok_or(bad("missing width-specific definitions"))?;
            let mut names = std::collections::BTreeSet::new();
            for (slot, table) in container.tables.iter().enumerate() {
                let def = defs.get(table.type_id)?;
                if !names.insert(def.name.clone()) {
                    return Err(bad("duplicate table name in container"));
                }
                if def.major_ver != table.major
                    || def.minor_ver != table.minor
                    || def.autokey != (table.kind == 1)
                {
                    return Err(bad("table schema version or kind mismatch"));
                }
                let body = TableBody::parse(&table.body, container.width, table.kind, Some(def))?;
                let locations = match &body.variant {
                    Variant::Plain(_) => Vec::new(),
                    Variant::Compressed(b) => b
                        .blocks
                        .iter()
                        .enumerate()
                        .flat_map(|(i, b)| (0..b.records.len()).map(move |j| (i, j)))
                        .collect(),
                };
                project.physical.push(Physical {
                    file,
                    slot,
                    body,
                    locations,
                });
            }
            project.sources.push(Source { container });
        }
        for (i, p) in project.physical.iter().enumerate() {
            let source = &project.sources[p.file].container;
            let name = &project.definition(i)?.name;
            let preferred_exists = project.physical.iter().enumerate().any(|(j, q)| {
                project.sources[q.file].container.width == preferred
                    && project.sources[q.file].container.role == source.role
                    && project.definition(j).is_ok_and(|d| &d.name == name)
            });
            if source.width == preferred || !preferred_exists {
                project.logical.push(i);
            }
        }
        Ok(project)
    }
    fn physical_id(&self, table: TableId) -> Result<usize> {
        self.logical
            .get(table.0)
            .copied()
            .ok_or(bad("unknown table ID"))
    }
    fn definition(&self, physical: usize) -> Result<&TableDefinition> {
        let p = self
            .physical
            .get(physical)
            .ok_or(bad("unknown physical table"))?;
        let c = &self.sources[p.file].container;
        self.definitions[wi(c.width)]
            .as_ref()
            .ok_or(bad("missing definitions"))?
            .get(c.tables[p.slot].type_id)
    }
    pub fn tables(&self) -> Vec<TableInfo> {
        self.logical
            .iter()
            .enumerate()
            .filter_map(|(id, &physical)| {
                let p = &self.physical[physical];
                let c = &self.sources[p.file].container;
                Some(TableInfo {
                    id: TableId(id),
                    type_id: c.tables[p.slot].type_id,
                    name: self.definition(physical).ok()?.name.clone(),
                    record_count: p.body.record_count(),
                    compressed: p.body.compressed,
                    role: c.role,
                    width: c.width,
                })
            })
            .collect()
    }
    fn attribute(&self, p: usize, index: usize, name: &str, repeat: usize) -> Result<&Attribute> {
        let (r, _) = self.physical[p].record_pool(index)?;
        let def = self.definition(p)?;
        let h = r.validate(Some(def))?;
        let layout = def.layout(h.element_type, h.subtype)?;
        let mut attrs = layout
            .base_fields
            .iter()
            .chain(layout.subtype_fields.iter())
            .filter(|a| a.name == name && a.size != 0 && repeat < a.repeat);
        let attr = attrs.next().ok_or(bad("field not found"))?;
        if attrs.next().is_some() {
            return Err(bad("ambiguous field name"));
        }
        Ok(attr)
    }
    fn base_fields(&self, p: usize, index: usize) -> Result<Vec<FieldView>> {
        if let Some(fields) = self.cache.borrow().get(&(p, index)) {
            return Ok(fields.clone());
        }
        let physical = &self.physical[p];
        let c = &self.sources[physical.file].container;
        let def = self.definition(p)?;
        let (record, pool) = physical.record_pool(index)?;
        let h = record.validate(Some(def))?;
        let layout = def.layout(h.element_type, h.subtype)?;
        // Name -> attributes, built once per record (a field scan per field is quadratic).
        let mut by_name: std::collections::HashMap<&str, Vec<&Attribute>> =
            std::collections::HashMap::new();
        for a in layout
            .base_fields
            .iter()
            .chain(layout.subtype_fields.iter())
            .filter(|a| a.size != 0)
        {
            by_name.entry(a.name.as_str()).or_default().push(a);
        }
        let mut fields = Vec::new();
        for f in decode_fields_with_pool(record, def, pool, c.width)? {
            let mut matches = by_name
                .get(f.name.as_str())
                .into_iter()
                .flatten()
                .filter(|a| f.repeat_index < a.repeat);
            let a = *matches.next().ok_or(bad("field not found"))?;
            if matches.next().is_some() {
                return Err(bad("ambiguous field name"));
            }
            let value = match f.value {
                RawValue::Signed(v) => Value::Int(v),
                RawValue::Float32Bits(v) => Value::Float(f32::from_bits(v)),
                RawValue::Bool(v) if v <= 1 => Value::Bool(v != 0),
                RawValue::Bool(v) => Value::Raw {
                    kind: a.kind.clone(),
                    bytes: vec![v],
                },
                RawValue::Unsigned(index) => Value::Enum {
                    index,
                    name: usize::try_from(index)
                        .ok()
                        .and_then(|i| a.seq.as_ref()?.get(i).cloned()),
                },
                RawValue::Ref(key) => Value::Ref {
                    table: a.reftable,
                    key,
                    resolved: None,
                },
                RawValue::Text { units, .. } => Value::Text(units),
                RawValue::Native { a, units, .. } => Value::Native { a, text: units },
                RawValue::Unresolved { kind, raw } => Value::Raw { kind, bytes: raw },
            };
            fields.push(FieldView {
                name: f.name,
                repeat_index: f.repeat_index,
                kind: a.kind.clone(),
                editable: !matches!(value, Value::Raw { .. }),
                value,
                hint: Hint {
                    int_range: (a.min != 0 || a.max != 0).then_some((a.min, a.max)),
                    float_range: (a.fmin != 0.0 || a.fmax != 0.0).then_some((a.fmin, a.fmax)),
                    enum_names: a.seq.clone().unwrap_or_default(),
                    target_table: u16::try_from(a.reftable).ok().and_then(|id| {
                        self.definitions[wi(c.width)]
                            .as_ref()?
                            .0
                            .get(&id)
                            .map(|d| d.name.clone())
                    }),
                    target_element: a.refel,
                },
                deprecated: a.deprecated,
                hidden: a.hidden,
            });
        }
        let mut cache = self.cache.borrow_mut();
        if cache.len() >= 128 {
            cache.clear();
        }
        cache.insert((p, index), fields.clone());
        Ok(fields)
    }
    fn current_fields(&self, p: usize, index: usize) -> Result<Vec<FieldView>> {
        let mut fields = self.base_fields(p, index)?;
        for f in &mut fields {
            let key = FieldKey {
                physical: p,
                record: index,
                field: f.name.clone(),
                repeat: f.repeat_index,
            };
            if let Some(value) = self.overlay.get(&key) {
                f.value = value.clone();
            }
            if let Value::Enum { index, name } = &mut f.value {
                *name = usize::try_from(*index)
                    .ok()
                    .and_then(|i| f.hint.enum_names.get(i).cloned());
            }
        }
        Ok(fields)
    }
    pub fn record(&self, table: TableId, index: usize) -> Result<Vec<FieldView>> {
        let p = self.physical_id(table)?;
        let mut fields = self.current_fields(p, index)?;
        for f in &mut fields {
            if let Value::Ref {
                table: target,
                key,
                resolved,
            } = &mut f.value
            {
                *resolved = self.resolve(p, *target, key)?;
            }
        }
        Ok(fields)
    }
    /// Current key bytes of a record (with edits). A `Ref` to this record stores these bytes.
    pub fn record_key(&self, table: TableId, index: usize) -> Result<[u8; 8]> {
        self.key(self.physical_id(table)?, index)
    }
    fn key(&self, p: usize, index: usize) -> Result<[u8; 8]> {
        let (r, _) = self.physical[p].record_pool(index)?;
        let d = self.definition(p)?;
        let h = r.validate(Some(d))?;
        let start = d.layout(h.element_type, h.subtype)?.keyoffset;
        let end = start.checked_add(8).ok_or(bad("key offset overflow"))?;
        let raw_key = || {
            r.fixed
                .get(start..end)
                .ok_or(bad("key exceeds record"))?
                .try_into()
                .map_err(|_| bad("invalid key size"))
        };
        let mut changed = None;
        for (k, v) in self
            .overlay
            .iter()
            .filter(|(k, _)| k.physical == p && k.record == index)
        {
            let a = self.attribute(p, index, &k.field, k.repeat)?;
            let field_start = k
                .repeat
                .checked_mul(a.size)
                .and_then(|n| a.offset.checked_add(n))
                .ok_or(bad("key field offset overflow"))?;
            let field_end = field_start
                .checked_add(a.size)
                .ok_or(bad("key field end overflow"))?;
            if field_start < end
                && start < field_end
                && !matches!(v, Value::Text(_) | Value::Native { .. })
            {
                edits::set_scalar(
                    changed.get_or_insert_with(|| r.clone()),
                    d,
                    &k.field,
                    k.repeat,
                    v,
                )?;
            }
        }
        match changed {
            Some(record) => record
                .fixed
                .get(start..end)
                .ok_or(bad("key exceeds record"))?
                .try_into()
                .map_err(|_| bad("invalid key size")),
            None => raw_key(),
        }
    }
    fn resolve(&self, source: usize, target: i32, key: &[u8]) -> Result<Option<RecordId>> {
        if key.iter().all(|b| *b == 0) {
            return Ok(None);
        }
        let Ok(key) = <[u8; 8]>::try_from(key) else {
            return Ok(None);
        };
        let role = self.sources[self.physical[source].file].container.role;
        let mut candidates = self
            .tables()
            .into_iter()
            .filter(|t| i32::from(t.type_id) == target);
        let target = candidates
            .clone()
            .find(|t| t.role == role)
            .or_else(|| candidates.next());
        let Some(target) = target else {
            return Ok(None);
        };
        let p = self.physical_id(target.id)?;
        if !self.keys.borrow().contains_key(&p) {
            let mut index = BTreeMap::new();
            for i in 0..self.physical[p].body.record_count() {
                index.entry(self.key(p, i)?).or_insert(i);
            }
            self.keys.borrow_mut().insert(p, index);
        }
        Ok(self
            .keys
            .borrow()
            .get(&p)
            .and_then(|m| m.get(&key))
            .map(|&index| RecordId {
                table: target.id,
                index,
            }))
    }
    pub fn record_title(&self, table: TableId, index: usize) -> Result<String> {
        let p = self.physical_id(table)?;
        let fields = self.current_fields(p, index)?;
        for f in fields
            .iter()
            .filter(|f| f.name.eq_ignore_ascii_case("alias"))
            .chain(fields.iter())
        {
            if let Some(text) = f.value.text().ok().filter(|text| !text.is_empty()) {
                return Ok(text);
            }
        }
        Ok(i64::from_le_bytes(self.key(p, index)?).to_string())
    }
    /// True when a text field of the record contains `query` (already lower case).
    /// Unedited records skip building field views.
    fn text_match(&self, p: usize, index: usize, query: &str) -> Result<bool> {
        let contains = |units: &[u16]| {
            String::from_utf16_lossy(units)
                .to_lowercase()
                .contains(query)
        };
        let edited = self
            .overlay
            .range(
                FieldKey {
                    physical: p,
                    record: index,
                    field: String::new(),
                    repeat: 0,
                }..,
            )
            .next()
            .is_some_and(|(k, _)| k.physical == p && k.record == index);
        if edited {
            return Ok(self
                .current_fields(p, index)?
                .iter()
                .any(|f| match &f.value {
                    Value::Text(u) | Value::Native { text: u, .. } => contains(u),
                    _ => false,
                }));
        }
        let physical = &self.physical[p];
        let width = self.sources[physical.file].container.width;
        let (record, pool) = physical.record_pool(index)?;
        bnsdata::table::any_text_with_pool(record, self.definition(p)?, pool, width, contains)
    }
    pub fn search(&self, table: TableId, query: &str, limit: usize) -> Result<Vec<usize>> {
        let p = self.physical_id(table)?;
        let query = query.to_lowercase();
        let mut hits = Vec::new();
        if limit == 0 {
            return Ok(hits);
        }
        for i in 0..self.physical[p].body.record_count() {
            let key = self.key(p, i)?;
            let key_match = i64::from_le_bytes(key).to_string().contains(&query)
                || u64::from_le_bytes(key).to_string().contains(&query);
            if key_match || self.text_match(p, i, &query)? {
                hits.push(i);
                if hits.len() == limit {
                    break;
                }
            }
        }
        Ok(hits)
    }
}
#[cfg(test)]
mod tests;
