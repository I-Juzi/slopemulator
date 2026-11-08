//! Local opaque single-use game tokens. Not vendor token cryptography.
use crate::password::{Binding, Identity, LoginError};
use rand::{CryptoRng, RngCore};
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

/// Owner tag of launcher tokens (spec 09). Game-token consumers refuse it,
/// and launcher consumers accept only it, so the two kinds never cross.
const LAUNCHER_OWNER: [u8; 16] = [0xff; 16];

struct Entry {
    identity: Identity,
    binding: Binding,
    owner: [u8; 16],
    deadline: Instant,
}
pub struct TokenAuthority {
    entries: Mutex<HashMap<[u8; 16], Entry>>,
    lifetime: Duration,
    capacity: usize,
}
impl TokenAuthority {
    pub fn new(lifetime: Duration, capacity: usize) -> Result<Self, LoginError> {
        if lifetime.is_zero() || capacity == 0 {
            return Err(LoginError::InvalidInput);
        }
        Ok(Self {
            entries: Mutex::default(),
            lifetime,
            capacity,
        })
    }
    // Only an authenticated Session can issue through the public API.
    pub(crate) fn issue<R: RngCore + CryptoRng>(
        &self,
        identity: Identity,
        binding: Binding,
        owner: [u8; 16],
        rng: &mut R,
        now: Instant,
    ) -> Result<[u8; 16], LoginError> {
        let deadline = now
            .checked_add(self.lifetime)
            .ok_or(LoginError::InvalidInput)?;
        let mut entries = self.entries.lock().map_err(|_| LoginError::Unavailable)?;
        entries.retain(|_, e| e.deadline > now);
        if entries.len() >= self.capacity {
            return Err(LoginError::Unavailable);
        }
        for _ in 0..8 {
            let mut token = [0; 16];
            rng.fill_bytes(&mut token);
            if token == [0; 16] || entries.contains_key(&token) {
                continue;
            }
            entries.insert(
                token,
                Entry {
                    identity,
                    binding,
                    owner,
                    deadline,
                },
            );
            return Ok(token);
        }
        Err(LoginError::Unavailable)
    }
    /// Wrong bindings do not consume. A successful check removes the token atomically.
    pub fn consume(
        &self,
        token: [u8; 16],
        portal: [u8; 16],
        binding: &Binding,
        now: Instant,
    ) -> Result<Identity, LoginError> {
        let mut entries = self.entries.lock().map_err(|_| LoginError::Unavailable)?;
        let entry = entries.get(&token).ok_or(LoginError::Authentication)?;
        if entry.deadline <= now {
            entries.remove(&token);
            return Err(LoginError::Authentication);
        }
        if entry.owner == LAUNCHER_OWNER
            || entry.identity.portal_id != portal
            || entry.binding != *binding
        {
            return Err(LoginError::Authentication);
        }
        Ok(entries
            .remove(&token)
            .ok_or(LoginError::Authentication)?
            .identity)
    }
    /// Issue a single-use launcher token for the token login path (spec 09).
    /// Call only from the trusted launcher issuer, after it authenticated the user.
    pub fn issue_launcher<R: RngCore + CryptoRng>(
        &self,
        identity: Identity,
        binding: Binding,
        rng: &mut R,
        now: Instant,
    ) -> Result<[u8; 16], LoginError> {
        if !identity.valid() || !binding.valid() {
            return Err(LoginError::InvalidInput);
        }
        self.issue(identity, binding, LAUNCHER_OWNER, rng, now)
    }
    /// Consume a launcher token at TokenKeyData. The portal is not known yet on
    /// this path, so only the binding is checked. Wrong bindings do not consume.
    pub(crate) fn consume_launcher(
        &self,
        token: [u8; 16],
        binding: &Binding,
        now: Instant,
    ) -> Result<Identity, LoginError> {
        let mut entries = self.entries.lock().map_err(|_| LoginError::Unavailable)?;
        let entry = entries.get(&token).ok_or(LoginError::Authentication)?;
        if entry.deadline <= now {
            entries.remove(&token);
            return Err(LoginError::Authentication);
        }
        if entry.owner != LAUNCHER_OWNER || entry.binding != *binding {
            return Err(LoginError::Authentication);
        }
        Ok(entries
            .remove(&token)
            .ok_or(LoginError::Authentication)?
            .identity)
    }
    pub fn expire(&self, now: Instant) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.retain(|_, e| e.deadline > now);
        }
    }
    pub(crate) fn revoke(&self, owner: [u8; 16]) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.retain(|_, e| e.owner != owner);
        }
    }
}
