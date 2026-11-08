//! Commands, reasons, frame reader and packet helpers (spec README.md
//! sections 3-6). Reply command names are inferred from the table order ❓.
use bloodlib::packet::{compose, decompose, DecodeLimits, Value};
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt};

/// Game port (RankingGamePacketRankingSideHandlerList @0x1400E1860).
pub mod game_cmd {
    pub const HELLO_REQ: i16 = 1;
    pub const HELLO_ACK: i16 = 2;
    pub const HELLO_FAIL: i16 = 3;
    pub const PING_REQ: i16 = 4;
    pub const PING_ACK: i16 = 5;
    pub const RETRIEVE_PLAYER_RANKING_INFO_REQ: i16 = 6;
    pub const RETRIEVE_PLAYER_RANKING_INFO_OK: i16 = 7;
    pub const RETRIEVE_PLAYER_RANKING_INFO_FAIL: i16 = 8;
    pub const RETRIEVE_PC_RANKING_RESULT_LIST_REQ: i16 = 9;
    pub const RETRIEVE_PC_RANKING_RESULT_LIST_ACK: i16 = 10;
    pub const RETRIEVE_PC_RANKING_RESULT_LIST_FAIL: i16 = 11;
    pub const LOAD_GUILD_RANKING_RESULT_LIST_REQ: i16 = 12;
    pub const LOAD_GUILD_RANKING_RESULT_LIST_ACK: i16 = 13;
    pub const LOAD_GUILD_RANKING_RESULT_LIST_FAIL: i16 = 14;
    pub const LOAD_PLAYER_SEASON_RESULT_REQ: i16 = 15;
    pub const LOAD_PLAYER_SEASON_RESULT_ACK: i16 = 16;
    pub const LOAD_PLAYER_SEASON_RESULT_FAIL: i16 = 17;
    pub const INSERT_PLAYER_RATING_SCORE_REQ: i16 = 18;
    pub const INSERT_PLAYER_RATING_SCORE_OK: i16 = 19;
    pub const INSERT_PLAYER_RATING_SCORE_FAIL: i16 = 20;
    pub const INSERT_RANKING_SCORE_REQ: i16 = 21;
    pub const INSERT_RANKING_SCORE_OK: i16 = 22;
    pub const INSERT_RANKING_SCORE_FAIL: i16 = 23;
    pub const DELETE_PLAYER_NOTIFY: i16 = 24;
}

/// InfoGate port (table @0x1400E17F0, 14 slots).
pub mod infogate_cmd {
    pub const HELLO_REQ: i16 = 1;
    pub const HELLO_ACK: i16 = 2;
    pub const HELLO_FAIL: i16 = 3;
    pub const RETRIEVE_RANKING_LIST_REQ: i16 = 5;
    pub const RETRIEVE_RANKING_LIST_ACK: i16 = 6;
    pub const RETRIEVE_RANKING_LIST_FAIL: i16 = 7;
    pub const RETRIEVE_TIME_ATTACK_BEST_RECORD_REQ: i16 = 8;
    pub const RETRIEVE_TIME_ATTACK_BEST_RECORD_OK: i16 = 9;
    pub const RETRIEVE_TIME_ATTACK_BEST_RECORD_FAIL: i16 = 10;
    pub const RETRIEVE_CHALLENGE_PARTY_LOG_SUMMARY_REQ: i16 = 11;
    pub const RETRIEVE_CHALLENGE_PARTY_LOG_SUMMARY_OK: i16 = 12;
    pub const RETRIEVE_CHALLENGE_PARTY_LOG_SUMMARY_FAIL: i16 = 13;
}

/// Manage port (RankingManagePacketRankingSideHandlerList @0x1400E1750, 1..9).
pub mod manage_cmd {
    pub const HELLO_REQ: i16 = 1;
    pub const HELLO_ACK: i16 = 2;
    pub const HELLO_FAIL: i16 = 3;
    pub const DELETE_BLACK_LIST_REQ: i16 = 4;
    pub const DELETE_BLACK_LIST_OK: i16 = 5;
    pub const DELETE_BLACK_LIST_FAIL: i16 = 6;
    pub const INSERT_BLACK_LIST_REQ: i16 = 7;
    pub const INSERT_BLACK_LIST_OK: i16 = 8;
    pub const INSERT_BLACK_LIST_FAIL: i16 = 9;
    pub const MAX: i16 = 10;
}

/// Hello fail reasons (read from the `r9d` argument of each Send).
pub mod reason {
    pub const SYSTEM: i16 = 2;
    pub const SEASON_LIST: i16 = 5;
    pub const VERSION: i16 = 7;
    pub const IP: i16 = 9;
    pub const PORT: i16 = 10;
    pub const DUPLICATE: i16 = 11;
    pub const MANAGE_DECOMPOSE: i16 = 1;
    pub const MANAGE_VERSION: i16 = 6;
}

/// Job fail reasons (handleJob / verifyParameter).
pub mod job_reason {
    pub const DEFAULT: i16 = 1;
    pub const NO_BOARD: i16 = 2;
    pub const DB: i16 = 4;
    pub const PARAMETER: i16 = 6;
    pub const DB_DELETE: i16 = 3;
    pub const NOT_IN_BLACK_LIST: i16 = 7;
    pub const ALREADY_IN_BLACK_LIST: i16 = 8;
    pub const RATING_EXPIRED: i16 = 16;
    pub const BLOCKED: i16 = 17;
    pub const DB_RESULT_2: i16 = 18;
    pub const RANKING_EXPIRED: i16 = 20;
}

pub const GAME_PACKET_VERSION: i16 = 10;
pub const INFOGATE_PACKET_VERSION: i16 = 6;
pub const MANAGE_PACKET_VERSION: i16 = 1;
/// Cluster ids from 10000 are arenas (`FindGame((id >= 10000) + 1, id)`).
pub const ARENA_ID_BASE: i16 = 10000;
pub const MAX_PACKET: i16 = 4000;

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
            string_units: 401,
            property_bytes: 0,
        },
    )
    .ok()
}

pub fn command(packet: &[u8]) -> Option<i16> {
    (packet.len() >= 4).then(|| i16::from_le_bytes([packet[2], packet[3]]))
}

/// Read one frame: i16 size (4..=4000) then the rest.
pub async fn read(reader: &mut (impl AsyncRead + Unpin)) -> io::Result<Option<Vec<u8>>> {
    let mut header = [0; 2];
    if reader.read(&mut header[..1]).await? == 0 {
        return Ok(None);
    }
    reader.read_exact(&mut header[1..]).await?;
    let size = i16::from_le_bytes(header);
    if !(4..=MAX_PACKET).contains(&size) {
        return Err(invalid("invalid frame size"));
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
pub fn i8_at(values: &[Value], i: usize) -> i8 {
    match values.get(i) {
        Some(Value::I8(n)) => *n,
        Some(Value::U8(n)) => *n as i8,
        _ => 0,
    }
}
pub fn u8_at(values: &[Value], i: usize) -> u8 {
    i8_at(values, i) as u8
}
pub fn i32_at(values: &[Value], i: usize) -> i32 {
    match values.get(i) {
        Some(Value::I32(n)) => *n,
        Some(Value::U32(n)) => *n as i32,
        _ => 0,
    }
}
pub fn u64_at(values: &[Value], i: usize) -> u64 {
    match values.get(i) {
        Some(Value::U64(n)) => *n,
        Some(Value::I64(n)) => *n as u64,
        _ => 0,
    }
}

pub fn wide(text: &str) -> Value {
    Value::String(text.encode_utf16().collect())
}
