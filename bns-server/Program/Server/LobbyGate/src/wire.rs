//! Bounded client transport. Spec 03 §§1, 2, 4.
use bloodlib::{frames::payload_len, packet::MAX_PACKET};
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
pub const MAX_WIRE: usize = 4008;
pub fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// Read exactly one frame. A clean EOF is distinct from a truncated frame.
/// The header is checked before the payload allocation.
pub async fn read_frame(reader: &mut (impl AsyncRead + Unpin)) -> io::Result<Option<Vec<u8>>> {
    let mut header = [0; 2];
    if reader.read(&mut header[..1]).await? == 0 {
        return Ok(None);
    }
    reader.read_exact(&mut header[1..]).await?;
    let h = u16::from_le_bytes(header);
    let len = payload_len(h);
    if h >> 14 == 0 {
        return Err(invalid("plain client frame is not accepted"));
    }
    if len + 2 > MAX_WIRE {
        return Err(invalid("frame exceeds receive buffer"));
    }
    let mut frame = vec![0; len + 2];
    frame[..2].copy_from_slice(&header);
    reader.read_exact(&mut frame[2..]).await?;
    Ok(Some(frame))
}
pub async fn write_frame(writer: &mut (impl AsyncWrite + Unpin), frame: &[u8]) -> io::Result<()> {
    if frame.len() < 2 || frame.len() > MAX_WIRE {
        return Err(invalid("invalid output frame size"));
    }
    let header = u16::from_le_bytes([frame[0], frame[1]]);
    if header >> 14 == 0 || payload_len(header) + 2 != frame.len() {
        return Err(invalid("invalid output frame header"));
    }
    writer.write_all(frame).await?;
    writer.flush().await
}
/// Remove cipher padding only by the inner signed size field.
pub fn inner_packet(plain: &[u8]) -> io::Result<&[u8]> {
    if plain.len() < 2 {
        return Err(invalid("missing inner size"));
    }
    let size = i16::from_le_bytes([plain[0], plain[1]]);
    if size < 4 || size as usize > plain.len() || size as usize > MAX_PACKET {
        return Err(invalid("invalid inner packet size"));
    }
    Ok(&plain[..size as usize])
}
