//! Bounded client table bodies. See `06-client-body.md` for the wire contract.
use crate::{dat::Width, tabledef::TableDefinition, Cursor, Error, Result};
use flate2::{write::ZlibEncoder, Compression, Decompress, FlushDecompress, Status};
use std::io::Write;

mod fields;
mod scalar;
pub use scalar::ScalarValue;
mod text;
pub use fields::{
    any_text_with_pool, decode_fields, decode_fields_with_pool, DecodedField, RawValue,
};
pub use text::RecordEditor;
#[cfg(test)]
mod strings_tests;
#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    /// Includes the six-byte DrEl header. Offsets remain integers.
    pub fixed: Vec<u8>,
    /// Compressed records only: bytes after DrEl.size in this record's span.
    pub extra: Vec<u8>,
    /// Optional writer override. Never inferred from an automatic slot position.
    pub key: Option<i64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordHeader {
    pub element_type: i8,
    pub subtype: i16,
    pub size: u16,
}
impl RecordHeader {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let mut c = Cursor::new(bytes);
        let element_type = c.u8()? as i8;
        c.u8()?;
        let subtype = c.u16()? as i16;
        let size = c.u16()?;
        if size < 6 {
            return Err(Error::Invalid("DrEl size is less than its header"));
        }
        Ok(Self {
            element_type,
            subtype,
            size,
        })
    }
}
impl Record {
    pub fn validate(&self, definition: Option<&TableDefinition>) -> Result<RecordHeader> {
        let header = RecordHeader::parse(&self.fixed)?;
        if usize::from(header.size) != self.fixed.len() {
            return Err(Error::Invalid("DrEl size differs from fixed record size"));
        }
        if let Some(def) = definition {
            if def.layout(header.element_type, header.subtype)?.size != self.fixed.len() {
                return Err(Error::Invalid("DrEl size differs from definition"));
            }
        }
        Ok(header)
    }
    fn writer_key(&self, definition: Option<&TableDefinition>) -> Result<i64> {
        let header = self.validate(definition)?;
        if let Some(key) = self.key {
            return Ok(key);
        }
        let def = definition.ok_or(Error::Invalid(
            "changed compressed records need a definition or explicit keys",
        ))?;
        let offset = def.layout(header.element_type, header.subtype)?.keyoffset;
        let end = offset
            .checked_add(8)
            .ok_or(Error::Invalid("key offset overflow"))?;
        let bytes = self
            .fixed
            .get(offset..end)
            .ok_or(Error::Invalid("eight-byte key exceeds record"))?;
        Ok(i64::from_le_bytes(bytes.try_into().unwrap()))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlainBody {
    /// Count as stored. Writing recomputes nonempty counts; empty kind 1 keeps 0 or 1.
    pub stored_count: u64,
    pub low_memory: u8,
    pub records: Vec<Record>,
    pub extra: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub first_key: i64,
    pub last_key: i64,
    pub records: Vec<Record>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompressedBody {
    pub constant: u16,
    pub blocks: Vec<Block>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Variant {
    Plain(PlainBody),
    Compressed(CompressedBody),
}
#[derive(Clone, Debug)]
pub struct TableBody {
    pub compressed: bool,
    pub variant: Variant,
    original: Option<Vec<u8>>,
    text_edits: std::collections::BTreeMap<usize, text::Edits>,
}
fn check_kind(kind: u8) -> Result<()> {
    if !matches!(kind, 1 | 2) {
        return Err(Error::Invalid("unknown table kind"));
    }
    Ok(())
}
fn check_count(count: u64, records: usize, kind: u8) -> Result<()> {
    let physical = if kind == 1 {
        count.saturating_sub(1)
    } else {
        count
    };
    if physical != records as u64 {
        return Err(Error::Invalid("stored count differs from physical records"));
    }
    Ok(())
}
fn u32_size(size: usize) -> Result<u32> {
    u32::try_from(size).map_err(|_| Error::Invalid("size exceeds u32"))
}
impl TableBody {
    pub fn new(variant: Variant) -> Self {
        Self {
            compressed: matches!(variant, Variant::Compressed(_)),
            variant,
            original: None,
            text_edits: Default::default(),
        }
    }
    pub fn discard_original(&mut self) {
        self.original = None;
    }
    pub fn parse(
        bytes: &[u8],
        width: Width,
        kind: u8,
        definition: Option<&TableDefinition>,
    ) -> Result<Self> {
        check_kind(kind)?;
        let mut c = Cursor::new(bytes);
        let flag = c.u8()?;
        let variant = match flag {
            0 => {
                let stored_count = if kind == 1 && width == Width::Bits64 {
                    c.u64()?
                } else {
                    u64::from(c.u32()?)
                };
                let fixed_size = c.u32()? as usize;
                let extra_size = c.u32()? as usize;
                let low_memory = c.u8()?;
                let mut fixed = Cursor::new(c.take(fixed_size)?);
                let extra = c.take(extra_size)?.to_vec();
                let mut records = Vec::new();
                while fixed.remaining() != 0 {
                    let header_bytes = fixed.take(6)?;
                    let header = RecordHeader::parse(header_bytes)?;
                    let mut bytes = header_bytes.to_vec();
                    bytes.extend_from_slice(fixed.take(usize::from(header.size) - 6)?);
                    let record = Record {
                        fixed: bytes,
                        extra: Vec::new(),
                        key: None,
                    };
                    record.validate(definition)?;
                    records.push(record);
                }
                check_count(stored_count, records.len(), kind)?;
                Variant::Plain(PlainBody {
                    stored_count,
                    low_memory,
                    records,
                    extra,
                })
            }
            1 => Variant::Compressed(read_compressed(&mut c, definition)?),
            _ => return Err(Error::Invalid("unknown table compression flag")),
        };
        c.finish()?;
        Ok(Self {
            compressed: flag == 1,
            variant,
            original: (flag == 1).then(|| bytes.to_vec()),
            text_edits: Default::default(),
        })
    }
    pub fn record_count(&self) -> usize {
        match &self.variant {
            Variant::Plain(body) => body.records.len(),
            Variant::Compressed(body) => body.blocks.iter().map(|b| b.records.len()).sum(),
        }
    }
    pub fn write(
        &self,
        width: Width,
        kind: u8,
        definition: Option<&TableDefinition>,
    ) -> Result<Vec<u8>> {
        check_kind(kind)?;
        if self.compressed != matches!(self.variant, Variant::Compressed(_)) {
            return Err(Error::Invalid("compression flag differs from variant"));
        }
        if !self.text_edits.is_empty() {
            let definition = definition.ok_or(Error::Invalid("text edits need a definition"))?;
            let mut rebuilt = self.clone();
            text::rebuild(&mut rebuilt, definition, width)?;
            return rebuilt.write(width, kind, Some(definition));
        }
        match &self.variant {
            Variant::Plain(body) => {
                let mut fixed_size = 0usize;
                for record in &body.records {
                    record.validate(definition)?;
                    if !record.extra.is_empty() {
                        return Err(Error::Invalid(
                            "plain records cannot have per-record extra bytes",
                        ));
                    }
                    fixed_size = fixed_size
                        .checked_add(record.fixed.len())
                        .ok_or(Error::Invalid("fixed size overflow"))?;
                }
                let count = if kind == 1 {
                    if body.records.is_empty() {
                        check_count(body.stored_count, 0, kind)?;
                        body.stored_count
                    } else {
                        (body.records.len() as u64)
                            .checked_add(1)
                            .ok_or(Error::Invalid("count overflow"))?
                    }
                } else {
                    body.records.len() as u64
                };
                let mut out = vec![0];
                if kind == 1 && width == Width::Bits64 {
                    out.extend_from_slice(&count.to_le_bytes());
                } else {
                    out.extend_from_slice(
                        &u32::try_from(count)
                            .map_err(|_| Error::Invalid("count exceeds u32"))?
                            .to_le_bytes(),
                    );
                }
                out.extend_from_slice(&u32_size(fixed_size)?.to_le_bytes());
                out.extend_from_slice(&u32_size(body.extra.len())?.to_le_bytes());
                out.push(body.low_memory);
                for record in &body.records {
                    out.extend_from_slice(&record.fixed);
                }
                out.extend_from_slice(&body.extra);
                Ok(out)
            }
            Variant::Compressed(body) => {
                for block in &body.blocks {
                    for record in &block.records {
                        record.validate(definition)?;
                    }
                }
                // Compare actual content, not a caller-managed dirty flag. Re-inflate one
                // original table rather than retain a second full record snapshot.
                if let Some(original) = &self.original {
                    let parsed = Self::parse(original, width, kind, None)?;
                    if parsed.variant == self.variant {
                        return Ok(original.clone());
                    }
                }
                write_compressed(body, definition)
            }
        }
    }
}
fn read_compressed(
    c: &mut Cursor<'_>,
    definition: Option<&TableDefinition>,
) -> Result<CompressedBody> {
    let count = c.u32()?;
    let constant = c.u16()?;
    let count = c.count(u64::from(count), 24)?;
    let mut blocks = Vec::with_capacity(count);
    let mut previous_last = None;
    for _ in 0..count {
        let first_key = c.u64()? as i64;
        let last_key = c.u64()? as i64;
        if first_key > last_key || previous_last.is_some_and(|last| last >= first_key) {
            return Err(Error::Invalid("invalid compressed block key range"));
        }
        previous_last = Some(last_key);
        let compressed_size = usize::from(c.u16()?);
        let compressed = c.take(compressed_size)?;
        let decompressed_size = usize::from(c.u16()?);
        let record_count = c.u32()?;
        let record_count = c.count(u64::from(record_count), 2)?;
        if record_count == 0 || record_count > decompressed_size / 6 {
            return Err(Error::Invalid("invalid compressed block record count"));
        }
        let mut offsets = Vec::with_capacity(record_count + 1);
        for _ in 0..record_count {
            offsets.push(usize::from(c.u16()?));
        }
        offsets.push(decompressed_size);
        if offsets[0] != 0 || offsets.windows(2).any(|w| w[0] >= w[1]) {
            return Err(Error::Invalid("invalid compressed record offsets"));
        }
        let mut decoder = Decompress::new(true);
        let mut raw = vec![0; decompressed_size + 1];
        let status = decoder
            .decompress(compressed, &mut raw, FlushDecompress::Finish)
            .map_err(|_| Error::Invalid("invalid zlib stream"))?;
        if status != Status::StreamEnd
            || decoder.total_out() != decompressed_size as u64
            || decoder.total_in() != compressed_size as u64
        {
            return Err(Error::Invalid("zlib block size or stream-end mismatch"));
        }
        raw.truncate(decompressed_size);
        let mut records = Vec::with_capacity(record_count);
        for offsets in offsets.windows(2) {
            let span = raw
                .get(offsets[0]..offsets[1])
                .ok_or(Error::Invalid("record offset exceeds block"))?;
            let header = RecordHeader::parse(span)?;
            let size = usize::from(header.size);
            let fixed = span
                .get(..size)
                .ok_or(Error::Invalid("DrEl size exceeds compressed span"))?
                .to_vec();
            let extra = span
                .get(size..)
                .ok_or(Error::Invalid("invalid record span"))?
                .to_vec();
            let record = Record {
                fixed,
                extra,
                key: None,
            };
            record.validate(definition)?;
            records.push(record);
        }
        blocks.push(Block {
            first_key,
            last_key,
            records,
        });
    }
    Ok(CompressedBody { constant, blocks })
}
fn write_compressed(
    body: &CompressedBody,
    definition: Option<&TableDefinition>,
) -> Result<Vec<u8>> {
    let mut out = vec![1, 0, 0, 0, 0];
    out.extend_from_slice(&body.constant.to_le_bytes());
    let mut block_count = 0u32;
    let mut pending: Vec<(&Record, i64)> = Vec::new();
    let mut raw_size = 0usize;
    let mut previous_key = None;
    for record in body.blocks.iter().flat_map(|block| &block.records) {
        let key = record.writer_key(definition)?;
        if previous_key.is_some_and(|previous| previous >= key) {
            return Err(Error::Invalid("compressed record keys must increase"));
        }
        previous_key = Some(key);
        let size = record
            .fixed
            .len()
            .checked_add(record.extra.len())
            .ok_or(Error::Invalid("record span overflow"))?;
        if size > usize::from(u16::MAX) {
            return Err(Error::Invalid("record span exceeds block limit"));
        }
        if raw_size + size > usize::from(u16::MAX) && !pending.is_empty() {
            emit_blocks(&pending, &mut out, &mut block_count)?;
            pending.clear();
            raw_size = 0;
        }
        pending.push((record, key));
        raw_size += size;
    }
    if !pending.is_empty() {
        emit_blocks(&pending, &mut out, &mut block_count)?;
    }
    out[1..5].copy_from_slice(&block_count.to_le_bytes());
    Ok(out)
}
fn emit_blocks(records: &[(&Record, i64)], out: &mut Vec<u8>, count: &mut u32) -> Result<()> {
    let mut raw = Vec::new();
    let mut offsets = Vec::new();
    for (record, _) in records {
        offsets.push(
            u16::try_from(raw.len()).map_err(|_| Error::Invalid("block offset exceeds u16"))?,
        );
        raw.extend_from_slice(&record.fixed);
        raw.extend_from_slice(&record.extra);
    }
    let ds = u16::try_from(raw.len()).map_err(|_| Error::Invalid("block exceeds u16"))?;
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&raw)?;
    let compressed = encoder.finish()?;
    let Ok(cs) = u16::try_from(compressed.len()) else {
        if records.len() < 2 {
            return Err(Error::Invalid("single compressed record exceeds u16"));
        }
        let mid = records.len() / 2;
        emit_blocks(&records[..mid], out, count)?;
        return emit_blocks(&records[mid..], out, count);
    };
    let first = records
        .first()
        .ok_or(Error::Invalid("empty compressed block"))?
        .1;
    let last = records
        .last()
        .ok_or(Error::Invalid("empty compressed block"))?
        .1;
    out.extend_from_slice(&first.to_le_bytes());
    out.extend_from_slice(&last.to_le_bytes());
    out.extend_from_slice(&cs.to_le_bytes());
    out.extend_from_slice(&compressed);
    out.extend_from_slice(&ds.to_le_bytes());
    out.extend_from_slice(&u32_size(records.len())?.to_le_bytes());
    for offset in offsets {
        out.extend_from_slice(&offset.to_le_bytes());
    }
    *count = count
        .checked_add(1)
        .ok_or(Error::Invalid("block count exceeds u32"))?;
    Ok(())
}
