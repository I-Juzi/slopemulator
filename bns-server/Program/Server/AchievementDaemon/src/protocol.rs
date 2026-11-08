//! Frames and packet builders (spec README.md).
use bloodlib::packet::{compose, decompose, DecodeLimits, Value};
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt};

/// Game port (AchievementGameChannel::PacketHandler @0x14000C1D0).
pub mod game_cmd {
    pub const HELLO_REQ: i16 = 1;
    pub const HELLO_ACK: i16 = 2;
    pub const HELLO_FAIL: i16 = 3;
    pub const PING: i16 = 4;
    pub const PONG: i16 = 5;
    pub const EVENT_OCCURED_NOTIFY: i16 = 6;
    pub const DATA: i16 = 7;
    pub const COMPLETE_FROM_CLIENT_NOTIFY: i16 = 8;
    pub const TAKE_REWARD_OK: i16 = 9;
    pub const TAKE_REWARD_FAIL: i16 = 10;
    pub const COMPLETED_NOTIFY: i16 = 11;
    pub const MAX: i16 = 12;
}

/// Manage port (ManagerSessionHandler::PacketHandler @0x14000E090).
pub mod manager_cmd {
    pub const HELLO_REQ: i16 = 1;
    pub const HELLO_ACK: i16 = 2;
    pub const HELLO_FAIL: i16 = 3;
    pub const PING: i16 = 4;
    pub const PONG: i16 = 5;
    pub const CHECK_EVENT_PROCESSING_REQ: i16 = 6;
    pub const CHECK_EVENT_PROCESSING_ACK: i16 = 7;
}

pub mod reason {
    pub const SYSTEM: i16 = 2;
    pub const VERSION: i16 = 3;
    pub const IP: i16 = 4;
}

pub const GAME_PACKET_VERSION: i32 = 10;
pub const GAME_EVENT_VERSION: i32 = 33;
pub const MANAGER_PACKET_VERSION: i32 = 4;

pub fn invalid(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

pub fn packet(format: &str, fields: &[Value]) -> Vec<u8> {
    compose(format, fields).expect("packet fits")
}

pub fn fields(format: &str, packet: &[u8]) -> Option<Vec<Value>> {
    decompose(
        format,
        packet,
        DecodeLimits {
            blob_bytes: 0,
            string_units: 4000,
            property_bytes: 0,
        },
    )
    .ok()
}

pub fn command(packet: &[u8]) -> Option<i16> {
    if packet.len() < 4 {
        return None;
    }
    Some(i16::from_le_bytes([packet[2], packet[3]]))
}

pub async fn read(reader: &mut (impl AsyncRead + Unpin)) -> io::Result<Option<Vec<u8>>> {
    let mut header = [0; 2];
    if reader.read(&mut header[..1]).await? == 0 {
        return Ok(None);
    }
    reader.read_exact(&mut header[1..]).await?;
    let size = i16::from_le_bytes(header);
    if !(4..=4000).contains(&size) {
        return Err(invalid("invalid trusted length"));
    }
    let mut packet = vec![0; size as usize];
    packet[..2].copy_from_slice(&header);
    reader.read_exact(&mut packet[2..]).await?;
    Ok(Some(packet))
}

pub fn hh(cmd: i16, value: i16) -> Vec<u8> {
    packet("hh", &[Value::I16(cmd), Value::I16(value)])
}

pub fn i16_at(values: &[Value], i: usize) -> i16 {
    match values.get(i) {
        Some(Value::I16(n)) => *n,
        Some(Value::U16(n)) => *n as i16,
        _ => 0,
    }
}
pub fn i32_at(values: &[Value], i: usize) -> i32 {
    match values.get(i) {
        Some(Value::I32(n)) => *n,
        Some(Value::U32(n)) => *n as i32,
        _ => 0,
    }
}
pub fn i64_at(values: &[Value], i: usize) -> i64 {
    match values.get(i) {
        Some(Value::I64(n)) => *n,
        Some(Value::U64(n)) => *n as i64,
        _ => 0,
    }
}
pub fn rows_at(values: &[Value], i: usize) -> &[Vec<Value>] {
    match values.get(i) {
        Some(Value::Repeat(rows)) => rows,
        _ => &[],
    }
}
