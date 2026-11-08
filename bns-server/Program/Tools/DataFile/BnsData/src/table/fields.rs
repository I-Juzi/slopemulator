use super::Record;
use crate::{dat::Width, tabledef::TableDefinition, Cursor, Error, Result};

/// Storage values. Text uses UTF-16 code units; references remain unresolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RawValue {
    Signed(i64),
    Unsigned(u64),
    /// Keep noncanonical values; do not silently normalize them to true.
    Bool(u8),
    /// Preserve NaN payloads and signed zero.
    Float32Bits(u32),
    Ref(Vec<u8>),
    Text {
        offset: u64,
        units: Vec<u16>,
    },
    Native {
        a: u32,
        offset: u64,
        units: Vec<u16>,
    },
    /// Stored bytes. Text stays unresolved when no pool is supplied.
    Unresolved {
        kind: String,
        raw: Vec<u8>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedField {
    pub name: String,
    pub repeat_index: usize,
    pub value: RawValue,
}
fn signed(bytes: &[u8]) -> Result<i64> {
    Ok(match bytes.len() {
        1 => i64::from(bytes[0] as i8),
        2 => i64::from(i16::from_le_bytes(bytes.try_into().unwrap())),
        4 => i64::from(i32::from_le_bytes(bytes.try_into().unwrap())),
        8 => i64::from_le_bytes(bytes.try_into().unwrap()),
        _ => return Err(Error::Invalid("unsupported scalar width")),
    })
}
/// Decode base and subtype fields. Repeats use offset + index * size.
/// Deprecated fields remain visible because this function reports raw storage.
pub fn decode_fields(record: &Record, definition: &TableDefinition) -> Result<Vec<DecodedField>> {
    decode(record, definition, None)
}

/// Resolve text against the table extra bytes (plain) or record extra bytes (compressed).
pub fn decode_fields_with_pool(
    record: &Record,
    definition: &TableDefinition,
    pool: &[u8],
    width: Width,
) -> Result<Vec<DecodedField>> {
    decode(record, definition, Some((pool, width)))
}

/// Calls `found` with the UTF-16 units of each `string` and `native` field, in field order.
/// Stops early when `found` returns true; the result says whether it did.
/// Faster than `decode_fields_with_pool` for search: other fields are not decoded.
pub fn any_text_with_pool(
    record: &Record,
    definition: &TableDefinition,
    pool: &[u8],
    width: Width,
    mut found: impl FnMut(&[u16]) -> bool,
) -> Result<bool> {
    let header = record.validate(Some(definition))?;
    let layout = definition.layout(header.element_type, header.subtype)?;
    for field in layout
        .fields()
        .filter(|f| f.size != 0 && matches!(f.kind.as_str(), "string" | "native"))
    {
        for repeat_index in 0..field.repeat {
            let start = repeat_index
                .checked_mul(field.size)
                .and_then(|n| field.offset.checked_add(n))
                .ok_or(Error::Invalid("field extent overflow"))?;
            let bytes = record
                .fixed
                .get(start..start.saturating_add(field.size))
                .filter(|b| b.len() == field.size)
                .ok_or(Error::Invalid("field exceeds record"))?;
            if let RawValue::Text { units, .. } | RawValue::Native { units, .. } =
                decode_text(bytes, field.kind == "native", pool, width)?
            {
                if found(&units) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

impl RawValue {
    /// Convert text without replacement characters. Invalid UTF-16 returns an error.
    pub fn text_string(&self) -> Result<String> {
        let units = match self {
            Self::Text { units, .. } | Self::Native { units, .. } => units,
            _ => return Err(Error::Invalid("field is not resolved text")),
        };
        String::from_utf16(units).map_err(|_| Error::Invalid("invalid UTF-16 text"))
    }
}

fn text_units(pool: &[u8], offset: u64) -> Result<Vec<u16>> {
    if !offset.is_multiple_of(2) {
        return Err(Error::Invalid("odd text offset"));
    }
    let start = usize::try_from(offset).map_err(|_| Error::Invalid("text offset overflow"))?;
    let tail = pool
        .get(start..)
        .ok_or(Error::Invalid("text offset outside pool"))?;
    if tail.len() < 2 {
        return Err(Error::Invalid("text offset outside pool"));
    }
    let mut units = Vec::new();
    let (pairs, _) = tail.as_chunks::<2>();
    for pair in pairs {
        let unit = u16::from_le_bytes([pair[0], pair[1]]);
        if unit == 0 {
            return Ok(units);
        }
        units.push(unit);
    }
    Err(Error::Invalid("missing text terminator"))
}

fn decode_text(bytes: &[u8], native: bool, pool: &[u8], width: Width) -> Result<RawValue> {
    let expected = match (native, width) {
        (false, Width::Bits32) => 4,
        (false, Width::Bits64) | (true, Width::Bits32) => 8,
        (true, Width::Bits64) => 12,
    };
    if bytes.len() != expected {
        return Err(Error::Invalid("text descriptor size differs from width"));
    }
    let mut c = Cursor::new(bytes);
    if native {
        let a = c.u32()?;
        let offset = u64::from(c.u32()?);
        if width == Width::Bits64 && c.u32()? != 0 {
            return Err(Error::Invalid("nonzero native third word"));
        }
        Ok(RawValue::Native {
            a,
            offset,
            units: text_units(pool, offset)?,
        })
    } else {
        let offset = if width == Width::Bits64 {
            c.u64()?
        } else {
            u64::from(c.u32()?)
        };
        Ok(RawValue::Text {
            offset,
            units: text_units(pool, offset)?,
        })
    }
}

fn decode(
    record: &Record,
    definition: &TableDefinition,
    pool: Option<(&[u8], Width)>,
) -> Result<Vec<DecodedField>> {
    let header = record.validate(Some(definition))?;
    let layout = definition.layout(header.element_type, header.subtype)?;
    let mut result = Vec::new();
    // Size-0 fields have no bytes in client records (server-only columns): skip them.
    for field in layout.fields().filter(|f| f.size != 0) {
        let extent = field
            .size
            .checked_mul(field.repeat)
            .and_then(|size| field.offset.checked_add(size))
            .ok_or(Error::Invalid("field extent overflow"))?;
        if extent > record.fixed.len() {
            return Err(Error::Invalid("field exceeds record"));
        }
        for repeat_index in 0..field.repeat {
            // The checked full extent above bounds every repeat.
            let start = field.offset + repeat_index * field.size;
            let bytes = record
                .fixed
                .get(start..start + field.size)
                .ok_or(Error::Invalid("field exceeds record"))?;
            let expected = match field.kind.as_str() {
                "int8" | "bool" | "seq" => Some(1),
                "int16" | "seq16" => Some(2),
                "int32" | "float32" | "float" => Some(4),
                "int64" => Some(8),
                _ => None,
            };
            // Client builds may store an integer narrower than its type (int16 in 1 byte).
            let narrower_int = matches!(field.kind.as_str(), "int16" | "int32" | "int64")
                && matches!(bytes.len(), 1 | 2 | 4)
                && expected.is_some_and(|size| bytes.len() < size);
            if !narrower_int && expected.is_some_and(|size| bytes.len() != size) {
                return Err(Error::Invalid("scalar size differs from definition type"));
            }
            let value = match field.kind.as_str() {
                "int8" | "int16" | "int32" | "int64" | "msec" | "distance" => {
                    RawValue::Signed(signed(bytes)?)
                }
                "seq" => RawValue::Unsigned(u64::from(bytes[0])),
                "seq16" => {
                    RawValue::Unsigned(u64::from(u16::from_le_bytes(bytes.try_into().unwrap())))
                }
                "bool" => RawValue::Bool(bytes[0]),
                "float32" | "float" => {
                    RawValue::Float32Bits(u32::from_le_bytes(bytes.try_into().unwrap()))
                }
                "ref" => RawValue::Ref(bytes.to_vec()),
                "string" | "native" if pool.is_some() => {
                    let (pool, width) = pool.ok_or(Error::Invalid("missing text pool"))?;
                    decode_text(bytes, field.kind == "native", pool, width)?
                }
                _ => RawValue::Unresolved {
                    kind: field.kind.clone(),
                    raw: bytes.to_vec(),
                },
            };
            result.push(DecodedField {
                name: field.name.clone(),
                repeat_index,
                value,
            });
        }
    }
    Ok(result)
}
