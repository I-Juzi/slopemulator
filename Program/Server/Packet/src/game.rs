//! Build-381 game packets. This module does not implement a session state machine.
//! Formats include command but not size. Scalars retain unverified signed bits.
//! Unknown bodies are returned in `Error::Unsupported`, without interpretation.
//! Command 32 stays unsupported, including its unverified fragment metadata.

use bloodlib::packet::{self, DecodeLimits, PacketError, Value};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};
use thiserror::Error;

const DEFAULT_TABLE: &str = include_str!("../tables/game381.json");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    ClientToServer,
    ServerToClient,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub handler: String,
    pub fragment: bool,
    pub format: Option<String>,
}

/// A verified subset, not a complete native dispatch table. No lobby relay IDs.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tables {
    pub build: String,
    pub version: u32,
    pub client: BTreeMap<i16, Entry>,
    pub server: BTreeMap<i16, Entry>,
}

impl Tables {
    pub fn parse(json: &str) -> Result<Self, Error> {
        let table: Self = serde_json::from_str(json).map_err(|e| Error::Table(e.to_string()))?;
        if table.version != 1278 || table.build != "0.0.381" {
            return Err(Error::Table("expected build 381, protocol 1278".into()));
        }
        for (direction, entries) in [
            (Direction::ClientToServer, &table.client),
            (Direction::ServerToClient, &table.server),
        ] {
            for (&command, entry) in entries {
                let Some((expected_direction, format, fragment)) = wire_spec(command) else {
                    return Err(Error::Table(format!("unverified command {command}")));
                };
                if direction != expected_direction
                    || entry.format.as_deref() != format
                    || entry.fragment != fragment
                    || entry.handler.is_empty()
                {
                    return Err(Error::Table(format!("invalid entry for command {command}")));
                }
            }
        }
        Ok(table)
    }

    pub fn entry(&self, direction: Direction, command: i16) -> Option<&Entry> {
        match direction {
            Direction::ClientToServer => self.client.get(&command),
            Direction::ServerToClient => self.server.get(&command),
        }
    }
}

pub fn tables() -> &'static Tables {
    static TABLE: OnceLock<Tables> = OnceLock::new();
    TABLE.get_or_init(|| Tables::parse(DEFAULT_TABLE).expect("checked game381 table"))
}

fn wire_spec(command: i16) -> Option<(Direction, Option<&'static str>, bool)> {
    use Direction::{ClientToServer as C, ServerToClient as S};
    Some(match command {
        11 => (C, Some("hhqsqb"), false),
        12 => (S, Some("hb"), false),
        16 => (C, Some("h"), false),
        17 => (S, Some("hqq"), false),
        18 => (C, Some("hqq"), false),
        23 => (C, Some("hhqq"), false),
        24 => (S, None, false),
        31 => (C, Some("hic"), false),
        32 => (S, None, true),
        41 => (S, Some("hqhiiccKuuuhhh"), false),
        42 => (C, Some("hiuuuh"), false),
        43 => (S, Some("hq"), false),
        929 => (C, None, false),
        _ => return None,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawPacket {
    pub command: i16,
    /// Application bytes only. Includes any unknown native fragment metadata.
    pub body: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Packet(#[from] PacketError),
    #[error("unsupported game command {command}", command = .0.command)]
    Unsupported(RawPacket),
    #[error("wrong direction for game command {0}")]
    WrongDirection(i16),
    #[error("trailing bytes after game fields")]
    TrailingBytes,
    #[error("invalid game dispatch table: {0}")]
    Table(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hello {
    pub protocol_version: u16,
    pub property_version: u64,
    /// Raw UTF-16 units. No Unicode normalization or wire NUL.
    pub publisher: Vec<u16>,
    pub capabilities: u64,
    /// Opaque data only. This module does not process the handshake.
    pub handshake_blob: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenAuth {
    pub world_id: u16,
    pub client_key: u64,
    pub lobby_token: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrepareZone {
    pub zone_object: u64,
    /// Exact instance semantics are not verified.
    pub instance_bits: u16,
    pub zone_table_id: u32,
    pub correlation: u32,
    pub config_byte: u8,
    pub action_flag: u8,
    pub zone_block: Vec<u8>,
    /// Raw signed-short coordinate bits. Units remain unknown.
    pub position_bits: [u16; 3],
    pub yaw_bits: u16,
    pub player_transition: u16,
    pub summoned_transition: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnterZone {
    pub correlation: u32,
    pub summoned_position_bits: [u16; 3],
    pub summoned_yaw_bits: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GamePacket {
    Hello(Hello),
    HelloOk {
        handshake_blob: Vec<u8>,
    },
    TokenAuth(TokenAuth),
    EnterWorld {
        parameter: u32,
        flag: u8,
    },
    PrepareZone(PrepareZone),
    EnterZone(EnterZone),
    /// Opaque runtime spawn-action value. Not a verified character ID.
    EnterZoneOk {
        spawn_context: u64,
    },
    ClientPing,
    ServerPing {
        sequence: u64,
        timestamp: u64,
    },
    PingEcho {
        sequence: u64,
        timestamp: u64,
    },
    /// Encoding returns Unsupported and retains all bytes.
    Raw(RawPacket),
}

impl GamePacket {
    pub fn command(&self) -> i16 {
        match self {
            Self::Hello(_) => 11,
            Self::HelloOk { .. } => 12,
            Self::ClientPing => 16,
            Self::ServerPing { .. } => 17,
            Self::PingEcho { .. } => 18,
            Self::TokenAuth(_) => 23,
            Self::EnterWorld { .. } => 31,
            Self::PrepareZone(_) => 41,
            Self::EnterZone(_) => 42,
            Self::EnterZoneOk { .. } => 43,
            Self::Raw(raw) => raw.command,
        }
    }

    /// Produces a logical packet, not an encrypted outer frame.
    pub fn encode(&self, direction: Direction) -> Result<Vec<u8>, Error> {
        if let Self::Raw(raw) = self {
            return Err(Error::Unsupported(raw.clone()));
        }
        let command = self.command();
        let format = checked_format(direction, command)?.expect("typed command format");
        let mut values = vec![Value::I16(command)];
        self.body_values(&mut values);
        Ok(packet::compose(format, &values)?)
    }

    /// Reads exactly one logical packet. Does not advance authentication state.
    pub fn decode(direction: Direction, input: &[u8]) -> Result<Self, Error> {
        packet::packet_size(input, 4)?;
        if input.len() > packet::MAX_PACKET {
            return Err(PacketError::InvalidSize.into());
        }
        let command = i16::from_le_bytes([input[2], input[3]]);
        let Some(format) = checked_format(direction, command)? else {
            return Err(Error::Unsupported(RawPacket {
                command,
                body: input[4..].to_vec(),
            }));
        };
        let values = packet::decompose(format, input, DecodeLimits::default())?;
        // Generic decompose permits trailing fields. Reject them here.
        if packet::compose(format, &values)? != input {
            return Err(Error::TrailingBytes);
        }
        let mut fields = Fields(values.into_iter());
        fields.short()?; // command
        Ok(match command {
            11 => Self::Hello(Hello {
                protocol_version: fields.short()?,
                property_version: fields.quad()?,
                publisher: fields.string()?,
                capabilities: fields.quad()?,
                handshake_blob: fields.bytes()?,
            }),
            12 => Self::HelloOk {
                handshake_blob: fields.bytes()?,
            },
            16 => Self::ClientPing,
            17 => Self::ServerPing {
                sequence: fields.quad()?,
                timestamp: fields.quad()?,
            },
            18 => Self::PingEcho {
                sequence: fields.quad()?,
                timestamp: fields.quad()?,
            },
            23 => Self::TokenAuth(TokenAuth {
                world_id: fields.short()?,
                client_key: fields.quad()?,
                lobby_token: fields.quad()?,
            }),
            31 => Self::EnterWorld {
                parameter: fields.integer()?,
                flag: fields.byte()?,
            },
            41 => Self::PrepareZone(PrepareZone {
                zone_object: fields.quad()?,
                instance_bits: fields.short()?,
                zone_table_id: fields.integer()?,
                correlation: fields.integer()?,
                config_byte: fields.byte()?,
                action_flag: fields.byte()?,
                zone_block: fields.bytes()?,
                position_bits: [fields.short()?, fields.short()?, fields.short()?],
                yaw_bits: fields.short()?,
                player_transition: fields.short()?,
                summoned_transition: fields.short()?,
            }),
            42 => Self::EnterZone(EnterZone {
                correlation: fields.integer()?,
                summoned_position_bits: [fields.short()?, fields.short()?, fields.short()?],
                summoned_yaw_bits: fields.short()?,
            }),
            43 => Self::EnterZoneOk {
                spawn_context: fields.quad()?,
            },
            _ => unreachable!("checked format has a typed decoder"),
        })
    }

    fn body_values(&self, values: &mut Vec<Value>) {
        use Value::{Bytes as B, String as S, U16 as H, U32 as I, U64 as Q, U8 as C};
        match self {
            Self::Hello(p) => values.extend([
                H(p.protocol_version),
                Q(p.property_version),
                S(p.publisher.clone()),
                Q(p.capabilities),
                B(p.handshake_blob.clone()),
            ]),
            Self::HelloOk { handshake_blob } => values.push(B(handshake_blob.clone())),
            Self::TokenAuth(p) => values.extend([H(p.world_id), Q(p.client_key), Q(p.lobby_token)]),
            Self::EnterWorld { parameter, flag } => values.extend([I(*parameter), C(*flag)]),
            Self::PrepareZone(p) => values.extend([
                Q(p.zone_object),
                H(p.instance_bits),
                I(p.zone_table_id),
                I(p.correlation),
                C(p.config_byte),
                C(p.action_flag),
                B(p.zone_block.clone()),
                H(p.position_bits[0]),
                H(p.position_bits[1]),
                H(p.position_bits[2]),
                H(p.yaw_bits),
                H(p.player_transition),
                H(p.summoned_transition),
            ]),
            Self::EnterZone(p) => values.extend([
                I(p.correlation),
                H(p.summoned_position_bits[0]),
                H(p.summoned_position_bits[1]),
                H(p.summoned_position_bits[2]),
                H(p.summoned_yaw_bits),
            ]),
            Self::EnterZoneOk { spawn_context } => values.push(Q(*spawn_context)),
            Self::ServerPing {
                sequence,
                timestamp,
            }
            | Self::PingEcho {
                sequence,
                timestamp,
            } => {
                values.extend([Q(*sequence), Q(*timestamp)]);
            }
            Self::ClientPing | Self::Raw(_) => {}
        }
    }
}

fn checked_format(direction: Direction, command: i16) -> Result<Option<&'static str>, Error> {
    let table = tables();
    if let Some(entry) = table.entry(direction, command) {
        return Ok(entry.format.as_deref());
    }
    let opposite = match direction {
        Direction::ClientToServer => Direction::ServerToClient,
        Direction::ServerToClient => Direction::ClientToServer,
    };
    if table.entry(opposite, command).is_some() {
        return Err(Error::WrongDirection(command));
    }
    Ok(None)
}

struct Fields(std::vec::IntoIter<Value>);
impl Fields {
    fn short(&mut self) -> Result<u16, PacketError> {
        match self.0.next() {
            Some(Value::I16(n)) => Ok(n as u16),
            Some(Value::U16(n)) => Ok(n),
            _ => Err(PacketError::ValueType('h')),
        }
    }
    fn integer(&mut self) -> Result<u32, PacketError> {
        match self.0.next() {
            Some(Value::U32(n)) => Ok(n),
            _ => Err(PacketError::ValueType('i')),
        }
    }
    fn quad(&mut self) -> Result<u64, PacketError> {
        match self.0.next() {
            Some(Value::U64(n)) => Ok(n),
            _ => Err(PacketError::ValueType('q')),
        }
    }
    fn byte(&mut self) -> Result<u8, PacketError> {
        match self.0.next() {
            Some(Value::U8(n)) => Ok(n),
            _ => Err(PacketError::ValueType('c')),
        }
    }
    fn bytes(&mut self) -> Result<Vec<u8>, PacketError> {
        match self.0.next() {
            Some(Value::Bytes(n)) => Ok(n),
            _ => Err(PacketError::ValueType('b')),
        }
    }
    fn string(&mut self) -> Result<Vec<u16>, PacketError> {
        match self.0.next() {
            Some(Value::String(n)) => Ok(n),
            _ => Err(PacketError::ValueType('s')),
        }
    }
}
