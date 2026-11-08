//! EventOccuredNotify (game cmd 6) body parser (spec README.md).
//!
//! Layout after the 4-byte frame header: q time, q tid, q pcdbid, q hint,
//! u16 event id, u16 cluster id, then the fields of the event class in
//! `Fields` array order (see schema.rs).
use crate::schema::{schema, EventSchema};

/// `Fields` descriptor type values: 0 u8, 1 u16, 2 u32, 3 u64, 4 16-byte, 5 wstr.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    U8,
    U16,
    U32,
    U64,
    Id,
    /// u16 unit count, then that many UTF-16 units.
    WStr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldValue {
    Int(u64),
    Id([u8; 16]),
    Str(Vec<u16>),
}

/// Header plus decoded fields. Internal events (cmd 8 conversions, cluster
/// connect) use the same type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub time: i64,
    pub tid: i64,
    pub pcdbid: i64,
    pub hint: i64,
    pub id: u16,
    pub cluster_id: i16,
    pub values: Vec<FieldValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// Body shorter than the 38-byte header (`size - 2 < 0x26`).
    ShortHeader,
    /// Event id above 0x770 or without an event class: "invalid event".
    UnknownEvent(u16),
    /// A field runs past the packet end: "decompose failed".
    Decompose,
}

pub const HEADER_BYTES: usize = 4 + 8 * 4 + 2 + 2;
pub const MAX_EVENT_ID: u16 = 0x770;

/// Command 6 accepts 48 classes. Metadata also contains seven internal classes.
/// Native parser: 0x140013770.
pub fn wire_event_allowed(id: u16) -> bool {
    id <= MAX_EVENT_ID && schema(id).is_some() && !matches!(id, 1013 | 1014 | 1803..=1807)
}

struct Cursor<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ParseError> {
        let end = self.at.checked_add(n).ok_or(ParseError::Decompose)?;
        let bytes = self.data.get(self.at..end).ok_or(ParseError::Decompose)?;
        self.at = end;
        Ok(bytes)
    }
    fn u16(&mut self) -> Result<u16, ParseError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, ParseError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
}

impl Event {
    pub fn new(id: u16, tid: i64, pcdbid: i64, cluster_id: i16, values: Vec<FieldValue>) -> Self {
        Self {
            time: 0,
            tid,
            pcdbid,
            hint: 0,
            id,
            cluster_id,
            values,
        }
    }

    pub fn schema(&self) -> Option<&'static EventSchema> {
        schema(self.id)
    }

    /// Parse a complete cmd 6 frame (size and cmd included).
    pub fn parse(packet: &[u8]) -> Result<Self, ParseError> {
        if packet.len() < HEADER_BYTES {
            return Err(ParseError::ShortHeader);
        }
        let mut c = Cursor {
            data: packet,
            at: 4,
        };
        let time = c.u64()? as i64;
        let tid = c.u64()? as i64;
        let pcdbid = c.u64()? as i64;
        let hint = c.u64()? as i64;
        let id = c.u16()?;
        let cluster_id = c.u16()? as i16;
        let schema = match schema(id) {
            Some(s) if wire_event_allowed(id) => s,
            _ => return Err(ParseError::UnknownEvent(id)),
        };
        let mut values = Vec::with_capacity(schema.fields.len());
        for &(_, kind) in schema.fields {
            values.push(match kind {
                FieldType::U8 => FieldValue::Int(c.take(1)?[0] as u64),
                FieldType::U16 => FieldValue::Int(c.u16()? as u64),
                FieldType::U32 => {
                    FieldValue::Int(u32::from_le_bytes(c.take(4)?.try_into().unwrap()) as u64)
                }
                FieldType::U64 => FieldValue::Int(c.u64()?),
                FieldType::Id => FieldValue::Id(c.take(16)?.try_into().unwrap()),
                FieldType::WStr => {
                    let n = c.u16()? as usize;
                    let bytes = c.take(n * 2)?;
                    FieldValue::Str(
                        bytes
                            .as_chunks::<2>()
                            .0
                            .iter()
                            .map(|b| u16::from_le_bytes(*b))
                            .collect(),
                    )
                }
            });
        }
        Ok(Self {
            time,
            tid,
            pcdbid,
            hint,
            id,
            cluster_id,
            values,
        })
    }

    /// Build a cmd 6 frame. Used by tests and tools; values must match the schema.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0, 0, 6, 0];
        for v in [self.time, self.tid, self.pcdbid, self.hint] {
            out.extend(v.to_le_bytes());
        }
        out.extend(self.id.to_le_bytes());
        out.extend(self.cluster_id.to_le_bytes());
        let kinds = self.schema().map_or(&[][..], |s| s.fields);
        for (&(_, kind), value) in kinds.iter().zip(&self.values) {
            match (kind, value) {
                (FieldType::U8, FieldValue::Int(n)) => out.push(*n as u8),
                (FieldType::U16, FieldValue::Int(n)) => out.extend((*n as u16).to_le_bytes()),
                (FieldType::U32, FieldValue::Int(n)) => out.extend((*n as u32).to_le_bytes()),
                (FieldType::U64, FieldValue::Int(n)) => out.extend(n.to_le_bytes()),
                (FieldType::Id, FieldValue::Id(b)) => out.extend(b),
                (FieldType::WStr, FieldValue::Str(s)) => {
                    out.extend((s.len() as u16).to_le_bytes());
                    for u in s {
                        out.extend(u.to_le_bytes());
                    }
                }
                _ => panic!("event value does not match schema"),
            }
        }
        let size = out.len() as i16;
        out[..2].copy_from_slice(&size.to_le_bytes());
        out
    }

    fn index(&self, name: &str) -> Option<usize> {
        self.schema()?.fields.iter().position(|(n, _)| *n == name)
    }

    /// Integer field by name; 0 when absent.
    pub fn int(&self, name: &str) -> u64 {
        match self.index(name).and_then(|i| self.values.get(i)) {
            Some(FieldValue::Int(n)) => *n,
            _ => 0,
        }
    }

    pub fn str(&self, name: &str) -> Vec<u16> {
        match self.index(name).and_then(|i| self.values.get(i)) {
            Some(FieldValue::Str(s)) => s.clone(),
            _ => Vec::new(),
        }
    }
}

pub mod id {
    pub const INCREMENT_REGISTER_FROM_CLIENT: u16 = 1013;
    pub const COMPLETE_REQUEST_FROM_CLIENT: u16 = 1014;
    pub const PC_LEVEL_UP: u16 = 1015;
    pub const EXPAND_INVENTORY: u16 = 1006;
    pub const DUEL_RESULT: u16 = 1023;
    pub const PARTY_BATTLE_RESULT: u16 = 1028;
    pub const COMPLETE_TAKE_REWARD: u16 = 1108;
    pub const PC_LOGIN: u16 = 1801;
    pub const PC_LOGOUT: u16 = 1802;
    pub const CLUSTER_CONNECTED: u16 = 1803;
    pub const CLUSTER_DISCONNECTED: u16 = 1804;
    pub const PC_IS_LOGINED: u16 = 1807;
    pub const DEBUG_ADD_REGISTER: u16 = 1901;
    pub const DEBUG_SET_REGISTER: u16 = 1902;
    pub const DEBUG_DELETE_ACHIEVEMENT: u16 = 1903;
    pub const DEBUG_DELETE_ALL: u16 = 1904;
}
