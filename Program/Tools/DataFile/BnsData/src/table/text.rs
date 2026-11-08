//! Staged record text edits. The writer rebuilds only pools with real changes.
use super::{decode_fields_with_pool, DecodedField, RawValue, Record, TableBody, Variant};
use crate::{dat::Width, tabledef::TableDefinition, Error, Result};
use std::collections::BTreeMap;

type FieldKey = (String, usize);
#[derive(Clone, Debug)]
pub(super) struct Edits {
    width: Width,
    values: BTreeMap<FieldKey, Vec<u16>>,
}

/// Edit one record. Record indexes span compressed blocks in stored order.
/// Edits are staged in the table; `write` needs the same width and definition.
pub struct RecordEditor<'a> {
    table: &'a mut TableBody,
    definition: &'a TableDefinition,
    width: Width,
    index: usize,
}

fn record_pool(variant: &Variant, index: usize) -> Result<(&Record, &[u8])> {
    match variant {
        Variant::Plain(body) => body.records.get(index).map(|r| (r, body.extra.as_slice())),
        Variant::Compressed(body) => body
            .blocks
            .iter()
            .flat_map(|b| &b.records)
            .nth(index)
            .map(|r| (r, r.extra.as_slice())),
    }
    .ok_or(Error::Invalid("record index outside table"))
}

impl TableBody {
    pub fn edit_record<'a>(
        &'a mut self,
        index: usize,
        definition: &'a TableDefinition,
        width: Width,
    ) -> Result<RecordEditor<'a>> {
        let (record, pool) = record_pool(&self.variant, index)?;
        decode_fields_with_pool(record, definition, pool, width)?;
        if self.text_edits.values().any(|e| e.width != width) {
            return Err(Error::Invalid("text edit width mismatch"));
        }
        Ok(RecordEditor {
            table: self,
            definition,
            width,
            index,
        })
    }
}

impl RecordEditor<'_> {
    /// Set raw UTF-16 code units. Unpaired surrogates are retained; NUL is rejected.
    /// A same-value edit cancels an earlier edit of this field.
    pub fn set_text(&mut self, name: &str, repeat_index: usize, units: &[u16]) -> Result<()> {
        if units.contains(&0) {
            return Err(Error::Invalid("text contains a terminator"));
        }
        let (record, pool) = record_pool(&self.table.variant, self.index)?;
        let fields = decode_fields_with_pool(record, self.definition, pool, self.width)?;
        let mut matches = fields
            .iter()
            .filter(|f| f.name == name && f.repeat_index == repeat_index);
        let field = matches
            .next()
            .ok_or(Error::Invalid("text field not found"))?;
        if matches.next().is_some() {
            return Err(Error::Invalid("ambiguous text field name"));
        }
        let original = match &field.value {
            RawValue::Text { units, .. } | RawValue::Native { units, .. } => units,
            _ => return Err(Error::Invalid("field is not text")),
        };
        let key = (name.to_owned(), repeat_index);
        if original == units {
            if let Some(edits) = self.table.text_edits.get_mut(&self.index) {
                edits.values.remove(&key);
                if edits.values.is_empty() {
                    self.table.text_edits.remove(&self.index);
                }
            }
        } else {
            self.table
                .text_edits
                .entry(self.index)
                .or_insert_with(|| Edits {
                    width: self.width,
                    values: BTreeMap::new(),
                })
                .values
                .insert(key, units.to_vec());
        }
        Ok(())
    }

    pub fn set_text_str(&mut self, name: &str, repeat_index: usize, text: &str) -> Result<()> {
        self.set_text(name, repeat_index, &text.encode_utf16().collect::<Vec<_>>())
    }
}

fn apply_edits(fields: &mut [DecodedField], edits: Option<&Edits>, width: Width) -> Result<bool> {
    let Some(edits) = edits else { return Ok(false) };
    if edits.width != width {
        return Err(Error::Invalid("text edit width mismatch"));
    }
    let mut changed = false;
    for ((name, index), replacement) in &edits.values {
        let mut matches = fields
            .iter_mut()
            .filter(|f| &f.name == name && f.repeat_index == *index);
        let field = matches
            .next()
            .ok_or(Error::Invalid("edited field not found"))?;
        if matches.next().is_some() {
            return Err(Error::Invalid("ambiguous edited field"));
        }
        match &mut field.value {
            RawValue::Text { units, .. } => {
                if units != replacement {
                    *units = replacement.clone();
                    changed = true;
                }
            }
            RawValue::Native { a, units, .. } => {
                if units != replacement {
                    *a = native_size(replacement)?;
                    *units = replacement.clone();
                    changed = true;
                }
            }
            _ => return Err(Error::Invalid("edited field is not text")),
        }
    }
    Ok(changed)
}

/// `dr_native.size` for edited text: UTF-16 bytes including the terminator.
/// Client `DrElReaderImpl::readNative` sets `2 * chars + 2`; `AllocedSize` sums it as bytes.
/// Unedited records keep their stored value (it is the source-language size).
fn native_size(units: &[u16]) -> Result<u32> {
    units
        .len()
        .checked_add(1)
        .and_then(|n| n.checked_mul(2))
        .and_then(|n| u32::try_from(n).ok())
        .ok_or(Error::Invalid("native text too long"))
}

fn append_text(pool: &mut Vec<u8>, units: &[u16]) -> Result<u64> {
    let offset = u64::try_from(pool.len()).map_err(|_| Error::Invalid("pool offset overflow"))?;
    let size = units
        .len()
        .checked_add(1)
        .and_then(|n| n.checked_mul(2))
        .and_then(|n| pool.len().checked_add(n))
        .ok_or(Error::Invalid("text pool size overflow"))?;
    // Every table extra size is a u32; compressed spans have a tighter writer limit.
    u32::try_from(size).map_err(|_| Error::Invalid("text pool exceeds u32"))?;
    for unit in units {
        pool.extend_from_slice(&unit.to_le_bytes());
    }
    pool.extend_from_slice(&[0, 0]);
    Ok(offset)
}

fn rebuild_record(
    record: &mut Record,
    definition: &TableDefinition,
    fields: &[DecodedField],
    width: Width,
    pool: &mut Vec<u8>,
    mut dedup: Option<&mut BTreeMap<Vec<u16>, u64>>,
) -> Result<()> {
    let header = record.validate(Some(definition))?;
    let layout = definition.layout(header.element_type, header.subtype)?;
    // Field order within each kind is the base/subtype definition order, then repeat order.
    for native in [false, true] {
        let mut values = fields.iter();
        // Same field walk as the decoder: size-0 fields are not stored.
        for field in layout.fields().filter(|f| f.size != 0) {
            for index in 0..field.repeat {
                let value = &values
                    .next()
                    .ok_or(Error::Invalid("field count mismatch"))?
                    .value;
                let (units, native_a) = match (native, value) {
                    (false, RawValue::Text { units, .. }) => (units, None),
                    (true, RawValue::Native { a, units, .. }) => (units, Some(*a)),
                    _ => continue,
                };
                let offset =
                    if let Some(offset) = dedup.as_ref().and_then(|map| map.get(units)).copied() {
                        offset
                    } else {
                        let offset = append_text(pool, units)?;
                        if let Some(map) = dedup.as_mut() {
                            map.insert(units.clone(), offset);
                        }
                        offset
                    };
                let base = index
                    .checked_mul(field.size)
                    .and_then(|n| field.offset.checked_add(n))
                    .ok_or(Error::Invalid("field offset overflow"))?;
                if let Some(a) = native_a {
                    // dr_native: size at +0, data offset at +4.
                    record
                        .fixed
                        .get_mut(base..base + 4)
                        .ok_or(Error::Invalid("native size exceeds record"))?
                        .copy_from_slice(&a.to_le_bytes());
                }
                let start = base
                    .checked_add(if native { 4 } else { 0 })
                    .ok_or(Error::Invalid("field offset overflow"))?;
                let bytes = if native || width == Width::Bits32 {
                    u32::try_from(offset)
                        .map_err(|_| Error::Invalid("text offset exceeds u32"))?
                        .to_le_bytes()
                        .to_vec()
                } else {
                    offset.to_le_bytes().to_vec()
                };
                let end = start
                    .checked_add(bytes.len())
                    .ok_or(Error::Invalid("field end overflow"))?;
                record
                    .fixed
                    .get_mut(start..end)
                    .ok_or(Error::Invalid("text offset exceeds record"))?
                    .copy_from_slice(&bytes);
            }
        }
    }
    Ok(())
}

pub(super) fn rebuild(
    table: &mut TableBody,
    definition: &TableDefinition,
    width: Width,
) -> Result<()> {
    for index in table.text_edits.keys() {
        record_pool(&table.variant, *index)?;
    }
    match &mut table.variant {
        Variant::Plain(body) => {
            let mut decoded = Vec::with_capacity(body.records.len());
            let mut changed = false;
            for (index, record) in body.records.iter().enumerate() {
                let mut fields = decode_fields_with_pool(record, definition, &body.extra, width)?;
                changed |= apply_edits(&mut fields, table.text_edits.get(&index), width)?;
                decoded.push(fields);
            }
            if changed {
                let mut pool = Vec::new();
                let mut dedup = BTreeMap::new();
                for (record, fields) in body.records.iter_mut().zip(&decoded) {
                    rebuild_record(
                        record,
                        definition,
                        fields,
                        width,
                        &mut pool,
                        Some(&mut dedup),
                    )?;
                }
                body.extra = pool;
            }
        }
        Variant::Compressed(body) => {
            for (index, record) in body
                .blocks
                .iter_mut()
                .flat_map(|b| &mut b.records)
                .enumerate()
            {
                if let Some(edits) = table.text_edits.get(&index) {
                    let mut fields =
                        decode_fields_with_pool(record, definition, &record.extra, width)?;
                    if apply_edits(&mut fields, Some(edits), width)? {
                        let mut pool = Vec::new();
                        rebuild_record(record, definition, &fields, width, &mut pool, None)?;
                        record.extra = pool;
                    }
                }
            }
        }
    }
    table.text_edits.clear();
    Ok(())
}
