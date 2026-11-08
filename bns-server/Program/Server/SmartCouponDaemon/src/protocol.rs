//! Trusted frames and packet builders (spec 02-wire.md).
use bloodlib::packet::{compose, decompose, DecodeLimits, Value};
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt};

/// GameDaemon side (DChannelMgr::PacketHandler @0x14000DFE0).
pub mod daemon_cmd {
    pub const HELLO_REQ: i16 = 1;
    pub const HELLO_ACK: i16 = 2;
    pub const HELLO_FAIL: i16 = 3;
    pub const BY_CHARACTER_REQ: i16 = 4;
    pub const BY_CHARACTER_ACK: i16 = 5;
    pub const BY_CHARACTER_FAIL: i16 = 6;
    pub const BY_ACCOUNT_REQ: i16 = 7;
    pub const BY_ACCOUNT_ACK: i16 = 8;
    pub const BY_ACCOUNT_FAIL: i16 = 9;
}

/// Manager side (ManagerSessionHandler::PacketHandler @0x14000E980).
pub mod manager_cmd {
    pub const HELLO_REQ: i16 = 1;
    pub const HELLO_ACK: i16 = 2;
    pub const HELLO_FAIL: i16 = 3;
    pub const RETRY_REQ: i16 = 4;
    pub const RETRY_OK: i16 = 5;
    pub const RETRY_FAIL: i16 = 6;
    pub const LOAD_RETRY_REQ: i16 = 7;
    pub const LOAD_RETRY_ACK: i16 = 8;
    pub const LOAD_RETRY_FAIL: i16 = 9;
}

/// Reason values seen in the binary. Names describe where each value is used.
pub mod reason {
    /// EventNotiJob::finalize when the reason was never set.
    pub const UNKNOWN: i16 = 1;
    /// Decode failure, missing world, or a failed internal post.
    pub const SYSTEM: i16 = 2;
    /// NP reply timed out (eventNotiExpire).
    pub const EXPIRED: i16 = 3;
    /// A store load failed.
    pub const DB_LOAD: i16 = 4;
    /// No NP channel, or the NP send failed.
    pub const NP_SEND: i16 = 5;
    /// Register failed, or NP rejected the publication.
    pub const PROCESS: i16 = 6;
    /// A request parameter is out of range.
    pub const INVALID_PARAM: i16 = 7;
    /// Service not available, or manager hello repeated.
    pub const STATE: i16 = 8;
    /// Packet version mismatch.
    pub const VERSION: i16 = 9;
    /// The publication has no coupon event.
    pub const NO_COUPON: i16 = 10;
    /// Remote IP differs from the gamed-list entry.
    pub const IP: i16 = 11;
    /// The world already has a session; both sessions close.
    pub const DUPLICATE_SESSION: i16 = 13;
}

pub const DAEMON_PACKET_VERSION: i16 = 1;
pub const MANAGER_PACKET_VERSION: i16 = 2;

pub fn invalid(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

pub fn packet(format: &str, fields: &[Value]) -> Vec<u8> {
    // Formats in this crate hold only fixed-size codes, so compose cannot fail.
    compose(format, fields).expect("fixed-size packet")
}

pub fn fields(format: &str, packet: &[u8]) -> Option<Vec<Value>> {
    decompose(
        format,
        packet,
        DecodeLimits {
            blob_bytes: 0,
            string_units: 0,
            property_bytes: 0,
        },
    )
    .ok()
}

pub fn command(packet: &[u8]) -> Option<i16> {
    if packet.len() < 4
        || packet.len() > 4000
        || i16::from_le_bytes([packet[0], packet[1]]) as usize != packet.len()
    {
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

pub fn ack(cmd: i16) -> Vec<u8> {
    packet("h", &[Value::I16(cmd)])
}
pub fn hello_fail(cmd: i16, reason: i16) -> Vec<u8> {
    packet("hh", &[Value::I16(cmd), Value::I16(reason)])
}
/// The tid the native decoder already wrote when a later field is short
/// (0x140008360, 0x140008E80, 0x14000F2F0): the decoder fills fields in
/// order, so a complete 8-byte tid after the 4-byte header survives.
pub fn partial_tid(packet: &[u8]) -> i64 {
    packet
        .get(4..12)
        .map_or(0, |b| i64::from_le_bytes(b.try_into().unwrap_or([0; 8])))
}

pub fn tid_ok(cmd: i16, tid: i64, value: i64) -> Vec<u8> {
    packet(
        "hqq",
        &[Value::I16(cmd), Value::I64(tid), Value::I64(value)],
    )
}
pub fn tid_fail(cmd: i16, tid: i64, reason: i16) -> Vec<u8> {
    packet(
        "hqE",
        &[Value::I16(cmd), Value::I64(tid), Value::I16(reason)],
    )
}
pub fn tid_only(cmd: i16, tid: i64) -> Vec<u8> {
    packet("hq", &[Value::I16(cmd), Value::I64(tid)])
}

pub fn i16_at(values: &[Value], i: usize) -> i16 {
    match values.get(i) {
        Some(Value::I16(n)) => *n,
        Some(Value::U16(n)) => *n as i16,
        _ => 0,
    }
}
pub fn i8_at(values: &[Value], i: usize) -> i8 {
    match values.get(i) {
        Some(Value::I8(n)) => *n,
        Some(Value::U8(n)) => *n as i8,
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
pub fn id_at(values: &[Value], i: usize) -> [u8; 16] {
    match values.get(i) {
        Some(Value::Id(n)) => *n,
        _ => [0; 16],
    }
}
