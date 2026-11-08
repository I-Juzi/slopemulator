//! Checked fixed-storage edits. Text descriptors are never scalar-editable.
use super::{decode_fields, RawValue, Record};
use crate::{tabledef::TableDefinition, Error, Result};

#[derive(Clone, Debug)]
pub enum ScalarValue {
    Int(i64),
    Float(f32),
    Bool(bool),
    Enum(u64),
    Ref(Vec<u8>),
}
impl Record {
    /// Write one stored field. Validate all bounds before changing bytes.
    pub fn set_scalar(
        &mut self,
        definition: &TableDefinition,
        name: &str,
        repeat: usize,
        value: ScalarValue,
    ) -> Result<()> {
        let decoded = decode_fields(self, definition)?;
        let mut found = decoded
            .iter()
            .filter(|f| f.name == name && f.repeat_index == repeat);
        let current = found
            .next()
            .ok_or(Error::Invalid("scalar field not found"))?;
        if found.next().is_some() {
            return Err(Error::Invalid("ambiguous scalar field"));
        }
        let header = self.validate(Some(definition))?;
        let layout = definition.layout(header.element_type, header.subtype)?;
        let field = layout
            .fields()
            .find(|f| f.name == name && f.size != 0 && repeat < f.repeat)
            .ok_or(Error::Invalid("scalar attribute missing"))?;
        let bytes = match (&current.value, value) {
            (RawValue::Signed(_), ScalarValue::Int(v)) => {
                let fits = match field.size {
                    1 => i8::try_from(v).is_ok(),
                    2 => i16::try_from(v).is_ok(),
                    4 => i32::try_from(v).is_ok(),
                    8 => true,
                    _ => false,
                };
                if !fits {
                    return Err(Error::Invalid("integer exceeds stored width"));
                }
                v.to_le_bytes()[..field.size].to_vec()
            }
            (RawValue::Float32Bits(_), ScalarValue::Float(v)) => v.to_bits().to_le_bytes().to_vec(),
            (RawValue::Bool(_), ScalarValue::Bool(v)) => vec![u8::from(v)],
            (RawValue::Unsigned(_), ScalarValue::Enum(v)) => match field.size {
                1 => {
                    vec![u8::try_from(v).map_err(|_| Error::Invalid("enum exceeds stored width"))?]
                }
                2 => u16::try_from(v)
                    .map_err(|_| Error::Invalid("enum exceeds stored width"))?
                    .to_le_bytes()
                    .to_vec(),
                _ => return Err(Error::Invalid("invalid enum width")),
            },
            (RawValue::Ref(_), ScalarValue::Ref(v)) if v.len() == field.size => v,
            _ => return Err(Error::Invalid("scalar value type or size mismatch")),
        };
        if bytes.len() != field.size {
            return Err(Error::Invalid("scalar size mismatch"));
        }
        let start = repeat
            .checked_mul(field.size)
            .and_then(|n| field.offset.checked_add(n))
            .ok_or(Error::Invalid("scalar offset overflow"))?;
        if start < 6 {
            return Err(Error::Invalid("scalar overlaps record header"));
        }
        let end = start
            .checked_add(bytes.len())
            .ok_or(Error::Invalid("scalar end overflow"))?;
        self.fixed
            .get_mut(start..end)
            .ok_or(Error::Invalid("scalar exceeds record"))?
            .copy_from_slice(&bytes);
        Ok(())
    }
}
