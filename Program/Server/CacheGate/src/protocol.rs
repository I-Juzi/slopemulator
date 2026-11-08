//! Frames and packet builders (spec README.md section 4).
//!
//! Every frame starts with `size: i16` (whole frame, little-endian) and
//! `cmd: i16`. Fragment frames (cmd 9) add `mark: i32`.
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt};

/// CacheGatePacketServerSideHandlerList @0x1400592D0 (10 slots).
pub mod cmd {
    pub const HELLO: i16 = 1;
    pub const HELLO_OK: i16 = 2;
    pub const HELLO_FAIL: i16 = 3;
    pub const RELAY: i16 = 4;
    pub const RELAY_FAIL: i16 = 5;
    pub const RELAY_INVALID: i16 = 6;
    pub const THE_OTHER_SIDE_LIST_REQ: i16 = 7;
    pub const THE_OTHER_SIDE_LIST_ACK: i16 = 8;
    pub const FRAGMENT_RELAY: i16 = 9;
    /// Slots 0..10 exist; the native table has no bounds check.
    pub const SLOTS: i16 = 10;
}

/// Error codes sent in HelloFail / RelayFail.
pub mod reason {
    /// ClientManager::AddSession failed (session already registered).
    pub const SYSTEM: i16 = 2;
    /// The CacheD exists but its Hello is not acknowledged.
    pub const NOT_READY: i16 = 3;
    /// Packet or economy version mismatch.
    pub const VERSION: i16 = 5;
    /// The wid already has a client session.
    pub const DUPLICATE: i16 = 7;
    /// No session for the target wid.
    pub const NO_TARGET: i16 = 8;
}

pub const PACKET_VERSION: i16 = 4;
pub const ECONOMY_PACKET_VERSION: i16 = 10;

/// Size of {fromWid, toWid, tid, hint, reserved}.
pub const RELAY_HEADER_LEN: usize = 22;
/// Relay payload starts after size, cmd and the relay header.
pub const RELAY_PAYLOAD_AT: usize = 4 + RELAY_HEADER_LEN;
/// Fragment data starts after size, cmd and mark.
pub const FRAGMENT_DATA_AT: usize = 8;

pub fn invalid(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

pub fn command(frame: &[u8]) -> Option<i16> {
    (frame.len() >= 4).then(|| i16::from_le_bytes([frame[2], frame[3]]))
}

pub fn i16_at(b: &[u8], at: usize) -> Option<i16> {
    Some(i16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}
pub fn i32_at(b: &[u8], at: usize) -> Option<i32> {
    Some(i32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}
pub fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(at..at + 8)?.try_into().ok()?))
}

fn frame(cmd: i16, body: &[u8]) -> Vec<u8> {
    let size = 4 + body.len();
    let mut out = Vec::with_capacity(size);
    out.extend_from_slice(&(size as i16).to_le_bytes());
    out.extend_from_slice(&cmd.to_le_bytes());
    out.extend_from_slice(body);
    out
}

/// `hhhh`: cmd 1, packet version 4, economy version 10, wid.
pub fn hello(wid: u16) -> Vec<u8> {
    let mut body = Vec::with_capacity(6);
    body.extend_from_slice(&PACKET_VERSION.to_le_bytes());
    body.extend_from_slice(&ECONOMY_PACKET_VERSION.to_le_bytes());
    body.extend_from_slice(&wid.to_le_bytes());
    frame(cmd::HELLO, &body)
}

/// `h`: cmd 2, 4 bytes.
pub fn hello_ok() -> Vec<u8> {
    frame(cmd::HELLO_OK, &[])
}

/// `hE`: cmd 3 and an error code.
pub fn hello_fail(code: i16) -> Vec<u8> {
    frame(cmd::HELLO_FAIL, &code.to_le_bytes())
}

/// {fromWid, toWid, tid, hint, reserved}.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelayHeader {
    pub from_wid: u16,
    pub to_wid: u16,
    pub tid: u64,
    pub hint: u16,
    pub reserved: u64,
}

impl RelayHeader {
    /// Read the 22 header bytes at `at`.
    pub fn read(b: &[u8], at: usize) -> Option<Self> {
        Some(Self {
            from_wid: i16_at(b, at)? as u16,
            to_wid: i16_at(b, at + 2)? as u16,
            tid: u64_at(b, at + 4)?,
            hint: i16_at(b, at + 12)? as u16,
            reserved: u64_at(b, at + 14)?,
        })
    }

    pub fn write(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.from_wid.to_le_bytes());
        out.extend_from_slice(&self.to_wid.to_le_bytes());
        out.extend_from_slice(&self.tid.to_le_bytes());
        out.extend_from_slice(&self.hint.to_le_bytes());
        out.extend_from_slice(&self.reserved.to_le_bytes());
    }
}

/// `hhhqhqh`: cmd 4, header, payload.
pub fn relay(header: &RelayHeader, payload: &[u8]) -> Vec<u8> {
    let mut body = Vec::with_capacity(RELAY_HEADER_LEN + payload.len());
    header.write(&mut body);
    body.extend_from_slice(payload);
    frame(cmd::RELAY, &body)
}

/// RelayFail (cmd 5, 28 bytes) back to the sender: fromWid is the gate's
/// own wid and toWid is the original fromWid; tid, hint and reserved stay.
pub fn relay_fail(own_wid: u16, original: &RelayHeader, code: i16) -> Vec<u8> {
    let header = RelayHeader {
        from_wid: own_wid,
        to_wid: original.from_wid,
        ..*original
    };
    let mut body = Vec::with_capacity(RELAY_HEADER_LEN + 2);
    header.write(&mut body);
    body.extend_from_slice(&code.to_le_bytes());
    frame(cmd::RELAY_FAIL, &body)
}

/// Read one frame. `size` must be at least 4 (the native session layer
/// rejects shorter frames). The frame keeps its size prefix.
pub async fn read(reader: &mut (impl AsyncRead + Unpin)) -> io::Result<Option<Vec<u8>>> {
    let mut header = [0; 2];
    if reader.read(&mut header[..1]).await? == 0 {
        return Ok(None);
    }
    reader.read_exact(&mut header[1..]).await?;
    let size = i16::from_le_bytes(header);
    if size < 4 {
        return Err(invalid("invalid frame length"));
    }
    let mut packet = vec![0; size as usize];
    packet[..2].copy_from_slice(&header);
    reader.read_exact(&mut packet[2..]).await?;
    Ok(Some(packet))
}
