//! TADBOSLB v5 framing. Bodies and aliases remain opaque.
//! Width and main/local role are explicit: the header does not identify them.
use crate::{Cursor, Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width {
    Bits32,
    Bits64,
}
impl Width {
    pub fn header_size(self) -> usize {
        match self {
            Self::Bits32 => 99,
            Self::Bits64 => 119,
        }
    }
    fn read(self, c: &mut Cursor<'_>) -> Result<u64> {
        match self {
            Self::Bits32 => c.u32().map(u64::from),
            Self::Bits64 => c.u64(),
        }
    }
    fn put(self, out: &mut Vec<u8>, value: u64) -> Result<()> {
        match self {
            Self::Bits32 => out.extend_from_slice(
                &u32::try_from(value)
                    .map_err(|_| Error::Invalid("DAT field exceeds 32-bit width"))?
                    .to_le_bytes(),
            ),
            Self::Bits64 => out.extend_from_slice(&value.to_le_bytes()),
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Main,
    Local,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub game_version: [u16; 4],
    /// Unknown sum rule. Always preserved, even after replacing tables.
    pub total_table_size: u64,
    pub total_table_count: u64,
    pub alias_size: u64,
    pub alias_count: u64,
    /// Unknown maximum rule. Always preserved.
    pub max_buffer_size: u64,
    pub save_time: u32,
    pub compressed: u8,
    pub encrypted: u8,
    pub reserved: [u8; 56],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    pub kind: u8,
    pub type_id: u16,
    pub major: u16,
    pub minor: u16,
    pub body: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Container {
    pub width: Width,
    pub role: Role,
    pub header: Header,
    /// Present only for main files with nonzero alias size AND count.
    /// Alias count is opaque. Do not change aliases without a matching count.
    pub aliases: Option<Vec<u8>>,
    pub tables: Vec<Table>,
}
fn supported_flags(h: &Header) -> Result<()> {
    if h.compressed != 0 || h.encrypted != 0 {
        return Err(Error::Invalid(
            "DAT container compression/encryption is not specified",
        ));
    }
    Ok(())
}
fn supported_kind(kind: u8) -> Result<()> {
    if !matches!(kind, 1 | 2) {
        return Err(Error::Invalid("unknown DAT table kind"));
    }
    Ok(())
}
impl Container {
    pub fn parse(bytes: &[u8], width: Width, role: Role) -> Result<Self> {
        let mut c = Cursor::new(bytes);
        if c.take(8)? != b"TADBOSLB" || c.u8()? != 5 {
            return Err(Error::Invalid("not TADBOSLB version 5"));
        }
        let mut game_version = [0; 4];
        for part in &mut game_version {
            *part = c.u16()?;
        }
        let header = Header {
            game_version,
            total_table_size: width.read(&mut c)?,
            total_table_count: width.read(&mut c)?,
            alias_size: width.read(&mut c)?,
            alias_count: width.read(&mut c)?,
            max_buffer_size: width.read(&mut c)?,
            save_time: c.u32()?,
            compressed: c.u8()?,
            encrypted: c.u8()?,
            reserved: c.take(56)?.try_into().unwrap(),
        };
        supported_flags(&header)?;
        let aliases = if role == Role::Main && header.alias_size != 0 && header.alias_count != 0 {
            let size = usize::try_from(header.alias_size)
                .map_err(|_| Error::Invalid("DAT alias size overflow"))?;
            Some(c.take(size)?.to_vec())
        } else {
            None
        };
        let count = c.count(header.total_table_count, 11)?;
        let mut tables = Vec::with_capacity(count);
        for _ in 0..count {
            let kind = c.u8()?;
            supported_kind(kind)?;
            let type_id = c.u16()?;
            let major = c.u16()?;
            let minor = c.u16()?;
            let size = c.u32()? as usize;
            let body = c.take(size)?.to_vec();
            tables.push(Table {
                kind,
                type_id,
                major,
                minor,
                body,
            });
        }
        c.finish()?;
        Ok(Self {
            width,
            role,
            header,
            aliases,
            tables,
        })
    }
    /// Update proved fields only. This is not a schema-aware client packer.
    /// The undocumented total-table-size and max-buffer-size fields stay unchanged.
    pub fn replace_tables(&mut self, tables: Vec<Table>) {
        self.tables = tables;
        self.header.total_table_count = self.tables.len() as u64;
    }
    /// Recompute table count, each body length, and a present main alias block's size.
    /// Preserve other fields. Local files retain the main header's alias metadata.
    pub fn write(&self) -> Result<Vec<u8>> {
        supported_flags(&self.header)?;
        let alias_size = match (&self.aliases, self.role) {
            (Some(bytes), Role::Main) if !bytes.is_empty() && self.header.alias_count != 0 => {
                bytes.len() as u64
            }
            (None, Role::Local) => self.header.alias_size,
            (None, Role::Main) if self.header.alias_size == 0 || self.header.alias_count == 0 => {
                self.header.alias_size
            }
            _ => {
                return Err(Error::Invalid(
                    "DAT aliases disagree with role or alias metadata",
                ))
            }
        };
        let mut out = Vec::new();
        out.extend_from_slice(b"TADBOSLB\x05");
        for part in self.header.game_version {
            out.extend_from_slice(&part.to_le_bytes());
        }
        for value in [
            self.header.total_table_size,
            self.tables.len() as u64,
            alias_size,
            self.header.alias_count,
            self.header.max_buffer_size,
        ] {
            self.width.put(&mut out, value)?;
        }
        out.extend_from_slice(&self.header.save_time.to_le_bytes());
        out.push(self.header.compressed);
        out.push(self.header.encrypted);
        out.extend_from_slice(&self.header.reserved);
        if let Some(bytes) = &self.aliases {
            out.extend_from_slice(bytes);
        }
        for table in &self.tables {
            supported_kind(table.kind)?;
            let size = u32::try_from(table.body.len())
                .map_err(|_| Error::Invalid("DAT table body exceeds u32"))?;
            out.push(table.kind);
            for value in [table.type_id, table.major, table.minor] {
                out.extend_from_slice(&value.to_le_bytes());
            }
            out.extend_from_slice(&size.to_le_bytes());
            out.extend_from_slice(&table.body);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample(width: Width, role: Role) -> Container {
        Container {
            width,
            role,
            header: Header {
                game_version: [0, 0, 381, 8884],
                total_table_size: 12345,
                total_table_count: 2,
                alias_size: 7,
                alias_count: 5,
                max_buffer_size: 8765,
                save_time: 123,
                compressed: 0,
                encrypted: 0,
                reserved: [73; 56],
            },
            aliases: (role == Role::Main).then(|| vec![3; 7]),
            tables: vec![
                Table {
                    kind: 1,
                    type_id: 4,
                    major: 2,
                    minor: 3,
                    body: vec![8; 29],
                },
                Table {
                    kind: 2,
                    type_id: 319,
                    major: 1,
                    minor: 0,
                    body: vec![],
                },
            ],
        }
    }
    #[test]
    fn roundtrips_widths_and_roles() {
        for width in [Width::Bits32, Width::Bits64] {
            for role in [Role::Main, Role::Local] {
                let source = sample(width, role);
                let bytes = source.write().unwrap();
                assert_eq!(
                    bytes.len(),
                    width.header_size() + if role == Role::Main { 7 } else { 0 } + 22 + 29
                );
                let parsed = Container::parse(&bytes, width, role).unwrap();
                assert_eq!(source, parsed);
                assert_eq!(bytes, parsed.write().unwrap());
            }
        }
    }
    #[test]
    fn replacements_update_only_proved_fields() {
        let mut source = sample(Width::Bits32, Role::Local);
        let mut table = source.tables[0].clone();
        table.body.resize(100, 2);
        source.replace_tables(vec![table]);
        let parsed = Container::parse(&source.write().unwrap(), source.width, source.role).unwrap();
        assert_eq!(parsed.header.total_table_count, 1);
        assert_eq!(parsed.tables[0].body.len(), 100);
        assert_eq!(parsed.header.total_table_size, 12345);
        assert_eq!(parsed.header.max_buffer_size, 8765);
        assert_eq!(
            (parsed.header.alias_size, parsed.header.alias_count),
            (7, 5)
        );
    }
    #[test]
    fn main_without_alias_and_alias_size_update() {
        let mut source = sample(Width::Bits64, Role::Main);
        source.aliases = None;
        source.header.alias_count = 0;
        let bytes = source.write().unwrap();
        assert_eq!(
            Container::parse(&bytes, source.width, source.role).unwrap(),
            source
        );
        source.header.alias_count = 5;
        source.aliases = Some(vec![1; 13]);
        let parsed = Container::parse(&source.write().unwrap(), source.width, source.role).unwrap();
        assert_eq!(parsed.header.alias_size, 13);
    }
    #[test]
    fn rejects_bad_framing_flags_and_width_overflow() {
        let source = sample(Width::Bits32, Role::Local);
        let bytes = source.write().unwrap();
        for end in 0..bytes.len() {
            assert!(Container::parse(&bytes[..end], source.width, source.role).is_err());
        }
        let mut bad = bytes.clone();
        bad.push(0);
        assert!(Container::parse(&bad, source.width, source.role).is_err());
        let mut bad = bytes.clone();
        bad[99] = 7;
        assert!(Container::parse(&bad, source.width, source.role).is_err());
        let mut bad = bytes.clone();
        bad[106..110].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(Container::parse(&bad, source.width, source.role).is_err());
        let mut bad = bytes;
        bad[41] = 1;
        assert!(Container::parse(&bad, source.width, source.role).is_err());
        let mut bad = source;
        bad.header.total_table_size = u64::MAX;
        assert!(bad.write().is_err());
    }
}
