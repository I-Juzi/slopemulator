//! Trusted and client frames from spec 03, sections 3 and 4.

use crate::crypto::{CryptoError, RsaKey, SessionCrypt};
use rand::{CryptoRng, RngCore};
use thiserror::Error;

pub const MAX_TRUSTED_SIZE: usize = 4000;
pub const MAX_ENCRYPTED_PAYLOAD: usize = 0x0fff * 4;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FrameError {
    #[error("plain client frames are not accepted")]
    PlainNotAccepted,
    #[error("invalid crypto module mode {0}; expected 5")]
    InvalidMode(u8),
    #[error("invalid session encryption code {0:#x}")]
    InvalidEncryptionCode(u8),
    #[error("invalid handshake code {actual:#x}; expected {expected:#x} (crypt error 7)")]
    InvalidHandshakeCode { expected: u8, actual: u8 },
    #[error("frame payload length {0} exceeds its wire limit")]
    PayloadTooLarge(usize),
    #[error("encrypted payload length {0} is not a multiple of four")]
    UnalignedPayload(usize),
    #[error("invalid inner packet size {size}; decrypted length is {available}")]
    InvalidPacketSize { size: i16, available: usize },
    #[error("invalid trusted packet size {0}; expected 2..=4000")]
    InvalidTrustedSize(i16),
    #[error("packet header size {declared} differs from input length {actual}")]
    PacketLengthMismatch { declared: usize, actual: usize },
    #[error("handshake decryption failed (crypt error 13): {0}")]
    HandshakeDecrypt(CryptoError),
    #[error(transparent)]
    Crypto(#[from] CryptoError),
}

#[derive(Debug, PartialEq, Eq)]
pub struct DecodedFrame {
    /// One plain packet, including its size field, without cipher padding.
    pub packet: Vec<u8>,
    /// Full wire frame length. Bytes after it belong to the next frame.
    pub consumed: usize,
}

/// Decode one trusted frame. `None` means that more bytes are needed.
pub fn decode_trusted(input: &[u8]) -> Result<Option<DecodedFrame>, FrameError> {
    let Some(size) = read_size(input) else {
        return Ok(None);
    };
    if !(2..=MAX_TRUSTED_SIZE as i16).contains(&size) {
        return Err(FrameError::InvalidTrustedSize(size));
    }
    let size = size as usize;
    if input.len() < size {
        return Ok(None);
    }
    Ok(Some(DecodedFrame {
        packet: input[..size].to_vec(),
        consumed: size,
    }))
}

pub fn encode_trusted(packet: &[u8]) -> Result<Vec<u8>, FrameError> {
    let size = read_size(packet).unwrap_or(0);
    if !(2..=MAX_TRUSTED_SIZE as i16).contains(&size) {
        return Err(FrameError::InvalidTrustedSize(size));
    }
    exact_packet_length(packet, size as usize)?;
    Ok(packet.to_vec())
}

/// Header arithmetic for both encrypted and rejected plain frames.
pub fn payload_len(header: u16) -> usize {
    if header >> 14 == 0 {
        usize::from(header & 0x3fff)
    } else {
        usize::from(header & 0x0fff) * 4
    }
}

/// Build an encrypted header without silent length truncation.
pub fn encrypted_header(code: u8, len: usize) -> Result<u16, FrameError> {
    if !(0x4..=0xf).contains(&code) {
        return Err(FrameError::InvalidEncryptionCode(code));
    }
    if len > MAX_ENCRYPTED_PAYLOAD {
        return Err(FrameError::PayloadTooLarge(len));
    }
    if !len.is_multiple_of(4) {
        return Err(FrameError::UnalignedPayload(len));
    }
    Ok((u16::from(code) << 12) | (len / 4) as u16)
}

pub enum ClientEncrypt<'a> {
    /// Public-key encryption uses code 4 or 5. Private-key failure output uses 4.
    Handshake {
        key: &'a RsaKey,
        code: u8,
    },
    Session {
        crypt: &'a SessionCrypt,
        code: u8,
    },
}

pub enum ClientDecrypt<'a> {
    /// No current crypt. Server input expects code 4; peer reply input uses 5.
    Handshake { key: &'a RsaKey, expected_code: u8 },
    /// A current session crypt is valid only in mode 5.
    Session { crypt: &'a SessionCrypt, mode: u8 },
}

pub fn encode_client(
    packet: &[u8],
    crypt: ClientEncrypt<'_>,
    rng: &mut (impl RngCore + CryptoRng),
) -> Result<Vec<u8>, FrameError> {
    let size = validate_inner(packet)?;
    exact_packet_length(packet, size)?;
    let (code, ciphertext) = match crypt {
        ClientEncrypt::Handshake { key, code } => {
            check_handshake_code(code, code)?;
            (code, key.encrypt(packet, rng)?)
        }
        ClientEncrypt::Session { crypt, code } => (code, crypt.encrypt(packet, code)?),
    };
    let header = encrypted_header(code, ciphertext.len())?;
    let mut output = Vec::with_capacity(ciphertext.len() + 2);
    output.extend_from_slice(&header.to_le_bytes());
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

/// Decode one client frame. `None` means that more bytes are needed.
/// Check completeness before checking crypto state, as in the original handler.
pub fn decode_client(
    input: &[u8],
    crypt: ClientDecrypt<'_>,
) -> Result<Option<DecodedFrame>, FrameError> {
    let Some(bytes) = input.get(..2) else {
        return Ok(None);
    };
    let header = u16::from_le_bytes([bytes[0], bytes[1]]);
    let len = payload_len(header);
    // Spec 03 step 1 is unreachable for a u16 header, but retain the guard.
    if len > 0xffff {
        return Err(FrameError::PayloadTooLarge(len));
    }
    let consumed = len + 2;
    if input.len() < consumed {
        return Ok(None);
    }
    let payload = &input[2..consumed];
    let code = (header >> 12) as u8;
    let plaintext = match crypt {
        ClientDecrypt::Handshake { key, expected_code } => {
            if header >> 14 == 0 {
                return Err(FrameError::PlainNotAccepted);
            }
            check_handshake_code(code, expected_code)?;
            key.decrypt(payload).map_err(FrameError::HandshakeDecrypt)?
        }
        ClientDecrypt::Session { crypt, mode } => {
            if mode != 5 {
                return Err(FrameError::InvalidMode(mode));
            }
            if !(0x8..=0xb).contains(&code) {
                return Err(FrameError::InvalidEncryptionCode(code));
            }
            crypt.decrypt(payload, code)?
        }
    };
    let size = validate_inner(&plaintext)?;
    Ok(Some(DecodedFrame {
        packet: plaintext[..size].to_vec(),
        consumed,
    }))
}

fn read_size(packet: &[u8]) -> Option<i16> {
    let bytes = packet.get(..2)?;
    Some(i16::from_le_bytes([bytes[0], bytes[1]]))
}

fn validate_inner(packet: &[u8]) -> Result<usize, FrameError> {
    let size = read_size(packet).unwrap_or(0);
    if size < 2 || size as usize > packet.len() {
        return Err(FrameError::InvalidPacketSize {
            size,
            available: packet.len(),
        });
    }
    Ok(size as usize)
}

fn exact_packet_length(packet: &[u8], declared: usize) -> Result<(), FrameError> {
    if declared != packet.len() {
        return Err(FrameError::PacketLengthMismatch {
            declared,
            actual: packet.len(),
        });
    }
    Ok(())
}

fn check_handshake_code(actual: u8, expected: u8) -> Result<(), FrameError> {
    if !matches!(expected, 0x4 | 0x5) || actual != expected {
        return Err(FrameError::InvalidHandshakeCode { expected, actual });
    }
    Ok(())
}
