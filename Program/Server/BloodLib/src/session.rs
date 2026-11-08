//! Stateful session packets. Spec 02 §12; no live rotation trigger.
use crate::crypto::{CryptoError, SessionCrypt};
use thiserror::Error;

pub const MAX_CIPHERTEXT: usize = 4000;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SessionError {
    #[error("session key is not set (crypt error 3)")]
    NoKey,
    #[error("inactive or invalid tag (crypt error 7)")]
    InvalidTag,
    #[error("padded packet exceeds 4000 bytes (crypt error 8)")]
    TooLarge,
    #[error("output capacity is too small (crypt error 9)")]
    Capacity,
    #[error("incomplete packet (crypt error 10)")]
    Incomplete,
    #[error("key change is already pending or receive has not caught up")]
    ChangePending,
    #[error(transparent)]
    Crypto(#[from] CryptoError),
}
impl SessionError {
    pub fn code(&self) -> Option<u8> {
        match self {
            Self::NoKey => Some(3),
            Self::InvalidTag => Some(7),
            Self::TooLarge => Some(8),
            Self::Capacity => Some(9),
            Self::Incomplete => Some(10),
            _ => None,
        }
    }
}

#[derive(Clone, Default)]
pub struct SessionPacketCrypt {
    keys: [Option<[u8; 16]>; 2],
    send: usize,
    receive: usize,
    pending: bool,
}
impl SessionPacketCrypt {
    pub fn new(key: [u8; 16]) -> Self {
        Self {
            keys: [Some(key), None],
            ..Self::default()
        }
    }
    pub fn state(&self) -> (usize, usize, bool) {
        if self.keys.iter().all(Option::is_none) {
            (u32::MAX as usize, u32::MAX as usize, self.pending)
        } else {
            (self.send, self.receive, self.pending)
        }
    }
    /// Supply the replacement key. The server does not call this method.
    pub fn prepare_key_change(&mut self, key: [u8; 16]) -> Result<(), SessionError> {
        if self.keys[self.send].is_none() {
            return Err(SessionError::NoKey);
        }
        if self.pending || self.send != self.receive {
            return Err(SessionError::ChangePending);
        }
        self.keys[1 - self.send] = Some(key);
        self.pending = true;
        Ok(())
    }
    fn crypt(&self, slot: usize) -> Result<SessionCrypt, SessionError> {
        Ok(SessionCrypt::new(
            self.keys[slot].ok_or(SessionError::NoKey)?,
        ))
    }
    pub fn encrypt(
        &mut self,
        input: &[u8],
        capacity: usize,
        third_party: bool,
    ) -> Result<Vec<u8>, SessionError> {
        let crypt = self.crypt(self.send)?;
        let padded = input.len().checked_add(15).ok_or(SessionError::TooLarge)? / 16 * 16;
        if padded > MAX_CIPHERTEXT {
            return Err(SessionError::TooLarge);
        }
        if capacity < padded + 2 {
            return Err(SessionError::Capacity);
        }
        let code = 8 + self.send as u16 + if third_party { 2 } else { 0 };
        let mut output = ((code << 12) | (padded as u16 / 4)).to_le_bytes().to_vec();
        output.extend(crypt.encrypt(input, 8)?);
        if self.pending {
            self.send = 1 - self.send;
            self.pending = false;
        }
        Ok(output)
    }
    /// Decode one complete packet. Return padded bytes and consumed wire bytes.
    pub fn decrypt(
        &mut self,
        input: &[u8],
        capacity: usize,
    ) -> Result<(Vec<u8>, usize), SessionError> {
        if input.len() < 2 {
            return Err(SessionError::Incomplete);
        }
        let header = u16::from_le_bytes([input[0], input[1]]);
        let len = usize::from(header & 0xfff) * 4;
        if input.len() < len + 2 || !len.is_multiple_of(16) {
            return Err(SessionError::Incomplete);
        }
        let tag = header >> 12;
        if !(8..=11).contains(&tag) {
            return Err(SessionError::InvalidTag);
        }
        let slot = (tag & 1) as usize;
        if slot != self.receive && slot != self.send {
            return Err(SessionError::InvalidTag);
        }
        if len > MAX_CIPHERTEXT {
            return Err(SessionError::TooLarge);
        }
        if capacity < len {
            return Err(SessionError::Capacity);
        }
        let output = self.crypt(slot)?.decrypt(&input[2..len + 2], 8)?;
        if slot != self.receive {
            self.keys[self.receive] = None;
            self.receive = slot;
        }
        Ok((output, len + 2))
    }
}
