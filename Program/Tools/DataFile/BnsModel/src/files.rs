use super::*;
use std::{
    fs::{self, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug)]
pub struct PackFile {
    pub path: PathBuf,
    pub changed_tables: Vec<String>,
    pub bytes: u64,
}
#[derive(Clone, Debug)]
pub struct PackReport {
    pub files: Vec<PackFile>,
    pub bytes: u64,
}
fn filename(width: Width, role: Role) -> &'static str {
    match (width, role) {
        (Width::Bits32, Role::Main) => "datafile.bin",
        (Width::Bits64, Role::Main) => "datafile64.bin",
        (Width::Bits32, Role::Local) => "localfile.bin",
        (Width::Bits64, Role::Local) => "localfile64.bin",
    }
}
impl Project {
    /// Open a folder. Use schema/x86 and schema/x64 when both widths are present.
    /// With a flat schema folder, only the selected width can be opened.
    pub fn open(
        folder: impl AsRef<Path>,
        preferred: Width,
        schema: impl AsRef<Path>,
    ) -> Result<Self> {
        let schema = schema.as_ref();
        if schema.join("x86").is_dir() || schema.join("x64").is_dir() {
            let x86 = schema.join("x86");
            let x64 = schema.join("x64");
            Self::open_with_schemas(
                folder,
                preferred,
                x86.is_dir().then_some(x86.as_path()),
                x64.is_dir().then_some(x64.as_path()),
            )
        } else {
            Self::open_with_schemas(
                folder,
                preferred,
                (preferred == Width::Bits32).then_some(schema),
                (preferred == Width::Bits64).then_some(schema),
            )
        }
    }
    pub fn open_with_schemas(
        folder: impl AsRef<Path>,
        preferred: Width,
        schema32: Option<&Path>,
        schema64: Option<&Path>,
    ) -> Result<Self> {
        let definitions32 = schema32.map(Definitions::load_folder).transpose()?;
        let definitions64 = schema64.map(Definitions::load_folder).transpose()?;
        let mut containers = Vec::new();
        for width in [Width::Bits32, Width::Bits64] {
            for role in [Role::Main, Role::Local] {
                let path = folder.as_ref().join(filename(width, role));
                match fs::read(path) {
                    Ok(bytes) => containers.push(Container::parse(&bytes, width, role)?),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                    Err(e) => return Err(e.into()),
                }
            }
        }
        Self::from_containers(containers, definitions32, definitions64, preferred)
    }
    /// Pack all present files. Existing output files are never overwritten.
    /// Create-new semantics also protect source files, hard links, and symlinks.
    /// Validation errors stop packing. Unresolved references remain warnings.
    pub fn pack(&self, out_dir: impl AsRef<Path>) -> Result<PackReport> {
        if self
            .problems()?
            .iter()
            .any(|p| p.severity == Severity::Error)
        {
            return Err(bad("changed records fail validation"));
        }
        // Encode every changed table before creating output files.
        let mut outputs = Vec::new();
        for (file, source) in self.sources.iter().enumerate() {
            let mut container = source.container.clone();
            let mut changed_tables = Vec::new();
            for (p, physical) in self
                .physical
                .iter()
                .enumerate()
                .filter(|(_, p)| p.file == file)
            {
                let edits: Vec<_> = self
                    .overlay
                    .iter()
                    .filter(|(key, _)| key.physical == p)
                    .collect();
                if edits.is_empty() {
                    continue;
                }
                let mut body = physical.body.clone();
                let def = self.definition(p)?;
                for (key, value) in &edits {
                    if matches!(value, Value::Text(_) | Value::Native { .. }) {
                        continue;
                    }
                    let record = match &mut body.variant {
                        Variant::Plain(b) => b.records.get_mut(key.record),
                        Variant::Compressed(b) => {
                            physical
                                .locations
                                .get(key.record)
                                .and_then(|&(block, slot)| {
                                    b.blocks.get_mut(block)?.records.get_mut(slot)
                                })
                        }
                    }
                    .ok_or(bad("edited record missing"))?;
                    edits::set_scalar(record, def, &key.field, key.repeat, value)?;
                }
                for (key, value) in edits {
                    if let Value::Text(units) | Value::Native { text: units, .. } = value {
                        body.edit_record(key.record, def, container.width)?
                            .set_text(&key.field, key.repeat, units)?;
                    }
                }
                let table = &mut container.tables[physical.slot];
                table.body = body.write(container.width, table.kind, Some(def))?;
                changed_tables.push(def.name.clone());
            }
            outputs.push((
                out_dir
                    .as_ref()
                    .join(filename(container.width, container.role)),
                container.write()?,
                changed_tables,
            ));
        }
        fs::create_dir_all(out_dir)?;
        // Reserve all names first. Do not truncate an existing target.
        let mut handles = Vec::new();
        for (path, _, _) in &outputs {
            match OpenOptions::new().write(true).create_new(true).open(path) {
                Ok(file) => handles.push(file),
                Err(e) => {
                    for (path, _, _) in outputs.iter().take(handles.len()) {
                        let _ = fs::remove_file(path);
                    }
                    return Err(e.into());
                }
            }
        }
        let mut report = PackReport {
            files: Vec::new(),
            bytes: 0,
        };
        for ((path, bytes, changed_tables), mut file) in outputs.into_iter().zip(handles) {
            file.write_all(&bytes)?;
            file.sync_all()?;
            let size = bytes.len() as u64;
            report.bytes += size;
            report.files.push(PackFile {
                path,
                changed_tables,
                bytes: size,
            });
        }
        Ok(report)
    }
}
