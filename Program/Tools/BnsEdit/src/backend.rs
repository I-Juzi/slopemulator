//! What the GUI needs from the data layer. `demo` implements it with synthetic data;
//! the real implementation wraps `bnsmodel`.

use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f32),
    Bool(bool),
    /// Index into the enum names; `names` is empty when the schema has none.
    Enum {
        index: u64,
        names: Vec<String>,
    },
    /// Link to another record. `title` is `None` when the target is not found.
    Ref {
        table: String,
        key: u64,
        title: Option<String>,
    },
    Text(String),
    /// Read-only bytes of a type the editor does not understand.
    Raw {
        kind: String,
        bytes: Vec<u8>,
    },
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub repeat: usize,
    pub value: Value,
    /// Value before any edit in this session, when changed.
    pub original: Option<Value>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub hidden: bool,
    /// Shown, but cannot be changed (unknown type or link).
    pub locked: bool,
}

impl Field {
    pub fn label(&self) -> String {
        let pretty = self.name.replace('-', " ");
        let mut c = pretty.chars();
        let base = match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => String::new(),
        };
        if self.repeat > 0 || self.name.ends_with(']') {
            format!("{base} {}", self.repeat + 1)
        } else {
            base
        }
    }
}

#[derive(Clone, Debug)]
pub struct TableInfo {
    pub name: String,
    pub records: usize,
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // `record` is kept for a future "jump to record" link.
pub struct Change {
    pub table: String,
    pub record: usize,
    pub title: String,
    pub field: String,
    pub old: Value,
    pub new: Value,
}

#[derive(Clone, Debug)]
pub struct Problem {
    pub table: String,
    pub record: usize,
    pub field: String,
    pub message: String,
    pub error: bool,
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // `files` is kept for a future file list.
pub struct GameInfo {
    pub folder: String,
    pub build: String,
    pub files: Vec<String>,
    pub schema: String,
}

#[derive(Clone, Debug)]
pub struct PackReport {
    pub files: Vec<(String, usize, u64)>,
}

#[allow(dead_code)] // undo/redo/changes are kept for tests and tools; the app keeps its own pack history.
pub trait Backend {
    fn game(&self) -> GameInfo;
    fn tables(&self) -> Vec<TableInfo>;
    fn search(&mut self, table: &str, query: &str, limit: usize) -> Vec<usize>;
    fn title(&mut self, table: &str, record: usize) -> String;
    fn subtitle(&mut self, table: &str, record: usize) -> String;
    fn fields(&mut self, table: &str, record: usize) -> Vec<Field>;
    fn set(
        &mut self,
        table: &str,
        record: usize,
        field: &str,
        repeat: usize,
        value: Value,
    ) -> Result<(), String>;
    /// Key that a link to this record stores. `None` if the record has no key.
    fn link_key(&mut self, table: &str, record: usize) -> Option<u64>;
    fn undo(&mut self) -> bool;
    fn redo(&mut self) -> bool;
    fn changes(&self) -> Vec<Change>;
    fn problems(&self) -> Vec<Problem>;
    fn is_changed(&self, table: &str, record: usize) -> bool;
    fn pack(&mut self, out_dir: &Path) -> Result<PackReport, String>;
    /// Fast read of a few fields (repeat 0) for the rule preview. Link titles may be empty.
    fn values(&mut self, table: &str, record: usize, names: &[String]) -> Vec<Option<Value>> {
        let fields = self.fields(table, record);
        names
            .iter()
            .map(|n| {
                fields
                    .iter()
                    .find(|f| f.name == *n && f.repeat == 0)
                    .map(|f| f.value.clone())
            })
            .collect()
    }
    /// Current value of one field.
    fn value(&mut self, table: &str, record: usize, field: &str, repeat: usize) -> Option<Value> {
        self.fields(table, record)
            .into_iter()
            .find(|f| f.name == field && f.repeat == repeat)
            .map(|f| f.value)
    }
}
