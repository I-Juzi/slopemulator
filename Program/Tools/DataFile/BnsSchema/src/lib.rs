//! Passive schema export from unprotected PE files.
//!
//! PE parsing and the protection gate always precede schema reads or output.
//! A refusal is a normal result. This crate cannot unpack protected files.
mod export;
pub mod find;
pub mod layout;
pub mod pe;
pub mod protect;
pub use pe::Width32or64;
use serde_json::Value;
use std::{collections::BTreeSet, fmt, fs, io::Write, path::Path};

#[derive(Debug)]
pub enum Error {
    Refused(String),
    NotFound(String),
    Io(std::io::Error),
    Format(String),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(s) => write!(f, "refused: {s}"),
            Self::NotFound(s) => write!(f, "not found: {s}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Format(s) => write!(f, "format error: {s}"),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<protect::Refused> for Error {
    fn from(e: protect::Refused) -> Self {
        Self::Refused(e.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub width: Width32or64,
    pub layout: &'static str,
    pub array_va: u64,
    pub tables: usize,
    pub unknown_types: Vec<u8>,
}
fn analyze(bytes: &[u8], array_va: Option<u64>) -> Result<(Found, Vec<Value>), Error> {
    let pe = pe::Pe::parse(bytes)?;
    protect::check(&pe)?;
    let layout = layout::for_width(pe.width);
    let array = find::find(&pe, layout, array_va)?;
    let (tables, unknown_types) = export::tables(&pe, layout, &array)?;
    Ok((
        Found {
            width: pe.width,
            layout: layout.name,
            array_va: array.va,
            tables: tables.len(),
            unknown_types,
        },
        tables,
    ))
}
/// Inspect the file. Run the protection gate and find the array. Write nothing.
pub fn inspect(exe: &Path) -> Result<Found, Error> {
    let bytes = fs::read(exe)?;
    analyze(&bytes, None).map(|(found, _)| found)
}
/// Write one JSON file per table. Never overwrite an existing file.
/// An I/O failure can leave a partial output folder. Use a fresh folder to retry.
pub fn export(exe: &Path, array_va: Option<u64>, out_dir: &Path) -> Result<Found, Error> {
    let bytes = fs::read(exe)?;
    let (found, tables) = analyze(&bytes, array_va)?;
    let mut names = BTreeSet::new();
    let mut files = Vec::new();
    for table in tables {
        let name = table
            .get("name")
            .and_then(Value::as_str)
            .filter(|name| find::safe_name(name))
            .ok_or_else(|| Error::Format("invalid table file name".into()))?;
        if !names.insert(name.to_owned()) {
            return Err(Error::Format("duplicate table file name".into()));
        }
        let path = out_dir.join(format!("{name}.json"));
        match fs::symlink_metadata(&path) {
            Ok(_) => {
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "output file already exists",
                )))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        let bytes = serde_json::to_vec_pretty(&table).map_err(|e| Error::Format(e.to_string()))?;
        files.push((path, bytes));
    }
    fs::create_dir_all(out_dir)?;
    for (path, bytes) in files {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
    }
    Ok(found)
}
#[cfg(test)]
mod tests;
