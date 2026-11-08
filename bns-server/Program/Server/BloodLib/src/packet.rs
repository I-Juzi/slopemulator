//! Format-driven packet fields (spec 03 sections 2, 5 and 6).
//! Strings contain UTF-16 code units, not Unicode scalar values.

use thiserror::Error;

pub const MAX_PACKET: usize = 4000;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    U8(u8),
    I8(i8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    F64(f64),
    I64(i64),
    U64(u64),
    Id([u8; 16]),
    Bytes(Vec<u8>),
    String(Vec<u16>),
    Property {
        type_id: u8,
        count: i16,
        data: Vec<u8>,
    },
    Repeat(Vec<Vec<Value>>),
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum PacketError {
    #[error("unknown format code {0}")]
    UnknownCode(char),
    #[error("repeat nesting exceeds three")]
    RepeatDepth,
    #[error("unmatched repeat delimiter")]
    RepeatDelimiter,
    #[error("wrong value type for {0}")]
    ValueType(char),
    #[error("value count does not match format")]
    ValueCount,
    #[error("repeat count exceeds u16")]
    RepeatCount,
    #[error("length cannot be written as a nonnegative i16")]
    LengthRange,
    #[error("compose exceeds packet capacity")]
    ComposeOverflow,
    #[error("field {code} exceeds input bounds")]
    Bounds { code: char },
    #[error("src size underflow")]
    BlobLengthUnderflow,
    #[error("not enough buffer")]
    BlobDestination,
    #[error("length underflow")]
    StringLengthUnderflow,
    #[error("string buffer is too small")]
    StringDestination,
    #[error("length overflow")]
    StringLengthOverflow,
    #[error("invalid property type id")]
    PropertyType,
    #[error("property count underflow")]
    PropertyCount,
    #[error("property length underflow")]
    PropertyLength,
    #[error("property destination is too small")]
    PropertyDestination,
    #[error("invalid packet size")]
    InvalidSize,
    #[error("declared size differs from input length")]
    PacketLength,
    #[error("fragment format must start with hi")]
    FragmentFormat,
}

/// Destination capacities. String capacity includes room for the trailing NUL
/// used by the original caller, although that NUL is absent on the wire.
#[derive(Clone, Copy, Debug)]
pub struct DecodeLimits {
    pub blob_bytes: usize,
    pub string_units: usize,
    pub property_bytes: usize,
}
impl Default for DecodeLimits {
    fn default() -> Self {
        Self {
            blob_bytes: usize::MAX,
            string_units: usize::MAX,
            property_bytes: usize::MAX,
        }
    }
}

#[derive(Debug)]
pub(crate) enum Field {
    Code(char),
    Repeat(Vec<Field>),
}

pub(crate) fn parse(format: &str) -> Result<Vec<Field>, PacketError> {
    fn group(chars: &mut std::str::Chars<'_>, depth: usize) -> Result<Vec<Field>, PacketError> {
        let mut fields = Vec::new();
        while let Some(code) = chars.next() {
            match code {
                ';' if depth > 0 => return Ok(fields),
                ';' => return Err(PacketError::RepeatDelimiter),
                'C' => {
                    if depth == 3 {
                        return Err(PacketError::RepeatDepth);
                    }
                    fields.push(Field::Repeat(group(chars, depth + 1)?));
                }
                'c' | 'Q' | 'h' | 'u' | 'E' | 'U' | 'd' | 'i' | 'f' | 'q' | 'o' | 'b' | 'K'
                | 's' | 'P' => fields.push(Field::Code(code)),
                _ => return Err(PacketError::UnknownCode(code)),
            }
        }
        if depth != 0 {
            return Err(PacketError::RepeatDelimiter);
        }
        Ok(fields)
    }
    group(&mut format.chars(), 0)
}

fn length(n: usize) -> Result<[u8; 2], PacketError> {
    Ok(i16::try_from(n)
        .map_err(|_| PacketError::LengthRange)?
        .to_le_bytes())
}

// Each vector is one indivisible wire field. Counts remain at their original
// field index while rows are emitted, even when later fields cross fragments.
#[doc(hidden)]
pub fn fields(
    format: &str,
    values: &[Value],
    capacity: usize,
) -> Result<Vec<Vec<u8>>, PacketError> {
    fn emit(
        nodes: &[Field],
        values: &[Value],
        cap: usize,
        out: &mut Vec<Vec<u8>>,
    ) -> Result<(), PacketError> {
        if nodes.len() != values.len() {
            return Err(PacketError::ValueCount);
        }
        for (node, value) in nodes.iter().zip(values) {
            if let Field::Repeat(body) = node {
                let Value::Repeat(rows) = value else {
                    return Err(PacketError::ValueType('C'));
                };
                if rows.len() > u16::MAX as usize {
                    return Err(PacketError::RepeatCount);
                }
                let index = out.len();
                out.push(vec![0, 0]);
                for (i, row) in rows.iter().enumerate() {
                    emit(body, row, cap, out)?;
                    out[index].copy_from_slice(&((i + 1) as i16).to_le_bytes());
                }
                continue;
            }
            let Field::Code(code) = node else {
                unreachable!()
            };
            let bytes = match (*code, value) {
                ('c', Value::U8(n)) => vec![*n],
                ('Q', Value::I8(n)) => vec![*n as u8],
                ('h' | 'u' | 'E' | 'U', Value::I16(n)) => n.to_le_bytes().to_vec(),
                ('h' | 'u' | 'E' | 'U', Value::U16(n)) => n.to_le_bytes().to_vec(),
                ('d' | 'i', Value::I32(n)) => n.to_le_bytes().to_vec(),
                ('d' | 'i', Value::U32(n)) => n.to_le_bytes().to_vec(),
                ('f', Value::F64(n)) => (*n as f32).to_le_bytes().to_vec(),
                ('q', Value::I64(n)) => n.to_le_bytes().to_vec(),
                ('q', Value::U64(n)) => n.to_le_bytes().to_vec(),
                ('o', Value::Id(n)) => n.to_vec(),
                ('b' | 'K', Value::Bytes(data)) => {
                    let prefix = length(data.len())?;
                    if data.len() > cap.saturating_sub(2) {
                        return Err(PacketError::ComposeOverflow);
                    }
                    let mut bytes = prefix.to_vec();
                    bytes.extend(data);
                    bytes
                }
                ('s', Value::String(units)) => {
                    let prefix = length(units.len())?;
                    if units.len() > cap.saturating_sub(2) / 2 {
                        return Err(PacketError::ComposeOverflow);
                    }
                    let mut bytes = prefix.to_vec();
                    for unit in units {
                        bytes.extend(unit.to_le_bytes());
                    }
                    bytes
                }
                (
                    'P',
                    Value::Property {
                        type_id,
                        count,
                        data,
                    },
                ) => {
                    if *type_id == 0 {
                        return Err(PacketError::PropertyType);
                    }
                    if *count < 0 {
                        return Err(PacketError::PropertyCount);
                    }
                    let prefix = length(data.len())?;
                    if data.len() > cap.saturating_sub(5) {
                        return Err(PacketError::ComposeOverflow);
                    }
                    let mut bytes = vec![*type_id];
                    bytes.extend(count.to_le_bytes());
                    bytes.extend(prefix);
                    bytes.extend(data);
                    bytes
                }
                _ => return Err(PacketError::ValueType(*code)),
            };
            if bytes.len() > cap {
                return Err(PacketError::ComposeOverflow);
            }
            out.push(bytes);
        }
        Ok(())
    }
    let mut output = Vec::new();
    emit(&parse(format)?, values, capacity, &mut output)?;
    Ok(output)
}

/// Compose includes the size prefix. The format includes cmd, normally `h`.
pub fn compose(format: &str, values: &[Value]) -> Result<Vec<u8>, PacketError> {
    let fields = fields(format, values, MAX_PACKET - 2)?;
    let mut output = vec![0, 0];
    for field in fields {
        if field.len() > MAX_PACKET - output.len() {
            return Err(PacketError::ComposeOverflow);
        }
        output.extend(field);
    }
    let size = output.len() as i16;
    output[..2].copy_from_slice(&size.to_le_bytes());
    Ok(output)
}

#[doc(hidden)]
pub fn packet_size(input: &[u8], minimum: usize) -> Result<(), PacketError> {
    let size = i16::from_le_bytes(
        input
            .get(..2)
            .ok_or(PacketError::InvalidSize)?
            .try_into()
            .unwrap(),
    );
    if size < minimum as i16 || size as usize > MAX_PACKET {
        return Err(PacketError::InvalidSize);
    }
    if size as usize != input.len() {
        return Err(PacketError::PacketLength);
    }
    Ok(())
}

pub fn decompose(
    format: &str,
    packet: &[u8],
    limits: DecodeLimits,
) -> Result<Vec<Value>, PacketError> {
    packet_size(packet, 2)?;
    decode_fields(format, &packet[2..], limits)
}

/// Decode joined fragment data with the portion of the format after `hi`.
/// Extra input is allowed, as in Decompose; callers can decode a prefix.
pub fn decode_fields(
    format: &str,
    input: &[u8],
    limits: DecodeLimits,
) -> Result<Vec<Value>, PacketError> {
    struct Reader<'a> {
        input: &'a [u8],
        limits: DecodeLimits,
    }
    impl Reader<'_> {
        fn take(&mut self, n: usize, code: char) -> Result<&[u8], PacketError> {
            let bytes = self.input.get(..n).ok_or(PacketError::Bounds { code })?;
            self.input = &self.input[n..];
            Ok(bytes)
        }
        fn short(&mut self, code: char) -> Result<i16, PacketError> {
            Ok(i16::from_le_bytes(self.take(2, code)?.try_into().unwrap()))
        }
        fn decode(&mut self, nodes: &[Field]) -> Result<Vec<Value>, PacketError> {
            let mut values = Vec::new();
            for node in nodes {
                let code = match node {
                    Field::Repeat(body) => {
                        let count = self.short('C')? as u16;
                        let mut rows = Vec::new();
                        for _ in 0..count {
                            rows.push(self.decode(body)?);
                        }
                        values.push(Value::Repeat(rows));
                        continue;
                    }
                    Field::Code(code) => *code,
                };
                let value = match code {
                    'c' => Value::U8(self.take(1, code)?[0]),
                    'Q' => Value::I8(self.take(1, code)?[0] as i8),
                    'h' | 'E' => Value::I16(self.short(code)?),
                    'u' | 'U' => Value::U16(self.short(code)? as u16),
                    'd' => Value::I32(i32::from_le_bytes(self.take(4, code)?.try_into().unwrap())),
                    'i' => Value::U32(u32::from_le_bytes(self.take(4, code)?.try_into().unwrap())),
                    'f' => Value::F64(
                        f32::from_le_bytes(self.take(4, code)?.try_into().unwrap()) as f64
                    ),
                    'q' => Value::U64(u64::from_le_bytes(self.take(8, code)?.try_into().unwrap())),
                    'o' => Value::Id(self.take(16, code)?.try_into().unwrap()),
                    'b' | 'K' => {
                        let n = self.short(code)?;
                        if n < 0 {
                            return Err(PacketError::BlobLengthUnderflow);
                        }
                        if n as usize > self.limits.blob_bytes {
                            return Err(PacketError::BlobDestination);
                        }
                        Value::Bytes(self.take(n as usize, code)?.to_vec())
                    }
                    's' => {
                        let n = self.short(code)?;
                        if n < 0 {
                            return Err(PacketError::StringLengthUnderflow);
                        }
                        if n as usize >= self.limits.string_units {
                            return Err(PacketError::StringDestination);
                        }
                        let bytes = self
                            .take(n as usize * 2, code)
                            .map_err(|_| PacketError::StringLengthOverflow)?;
                        Value::String(
                            bytes
                                .as_chunks::<2>()
                                .0
                                .iter()
                                .map(|p| u16::from_le_bytes([p[0], p[1]]))
                                .collect(),
                        )
                    }
                    'P' => {
                        let type_id = self.take(1, code)?[0];
                        if type_id == 0 {
                            return Err(PacketError::PropertyType);
                        }
                        let count = self.short(code)?;
                        if count < 0 {
                            return Err(PacketError::PropertyCount);
                        }
                        let n = self.short(code)?;
                        if n < 0 {
                            return Err(PacketError::PropertyLength);
                        }
                        if n as usize > self.limits.property_bytes {
                            return Err(PacketError::PropertyDestination);
                        }
                        Value::Property {
                            type_id,
                            count,
                            data: self.take(n as usize, code)?.to_vec(),
                        }
                    }
                    _ => unreachable!(),
                };
                values.push(value);
            }
            Ok(values)
        }
    }
    Reader { input, limits }.decode(&parse(format)?)
}
