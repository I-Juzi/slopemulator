//! Crypto from spec 02, sections 4, 6, 8 and 9. No built-in server key.

use aes::{
    cipher::{Block, BlockDecrypt, BlockEncrypt, KeyInit},
    Aes128,
};
use num_bigint::BigUint;
use rand::{CryptoRng, RngCore};
use ripemd::{Digest, Ripemd128};
use thiserror::Error;

const HASH_LEN: usize = 16;
const OAEP_OVERHEAD: usize = 34;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CryptoError {
    #[error("OAEP block must have at least 34 bytes")]
    OaepBlockTooSmall,
    #[error("OAEP message exceeds block capacity")]
    MessageTooLong,
    #[error("OAEP delimiter is absent or invalid (RSA error 17)")]
    InvalidOaepDelimiter,
    #[error("invalid length-prefixed integer layout")]
    InvalidIntegerLayout,
    #[error("invalid RSA modulus or exponent")]
    InvalidRsaKey,
    #[error("RSA input must contain whole fixed-width blocks")]
    IncompleteRsaBlock,
    #[error("RSA integer is not less than the modulus")]
    RsaIntegerOutOfRange,
    #[error("invalid DH field length")]
    InvalidDhLength,
    #[error("DH modulus must exceed one")]
    InvalidDhModulus,
    #[error("invalid session code {0:#x} (crypt error 7)")]
    InvalidSessionCode(u8),
    #[error("session key slot {0} is not configured")]
    MissingSessionKey(usize),
    #[error("AES ciphertext must contain whole 16-byte blocks")]
    IncompleteAesBlock,
    #[error("output length overflow")]
    LengthOverflow,
}

fn digest_381(input: &[u8]) -> [u8; HASH_LEN] {
    let mut message = input.to_vec();
    let tail = message.len() / 64 * 64;
    let last = message.len().checked_sub(1);
    for (offset, byte) in message.iter_mut().enumerate().skip(tail) {
        if offset % 4 != 3 && Some(offset) != last {
            *byte = 0;
        }
    }
    Ripemd128::digest(&message).into()
}

/// Return the build 381 digest, including its nonstandard partial-block rule.
pub fn ripemd128_381(input: &[u8]) -> [u8; HASH_LEN] {
    digest_381(input)
}

fn mgf(input: &[u8], len: usize) -> Vec<u8> {
    let mut output = Vec::with_capacity(len);
    for counter in 0..len.div_ceil(HASH_LEN) {
        let mut message = input.to_vec();
        message.extend_from_slice(&(counter as u32).to_le_bytes());
        let digest = digest_381(&message);
        let take = HASH_LEN.min(len - output.len());
        output.extend_from_slice(&digest[..take]);
    }
    output
}

fn xor(bytes: &mut [u8], mask: &[u8]) {
    for (byte, mask) in bytes.iter_mut().zip(mask) {
        *byte ^= mask;
    }
}

/// Return the reversed OAEP block, ready for little-endian RSA import.
pub fn oaep_encode(
    message: &[u8],
    k: usize,
    rng: &mut (impl RngCore + CryptoRng),
) -> Result<Vec<u8>, CryptoError> {
    let capacity = k
        .checked_sub(OAEP_OVERHEAD)
        .ok_or(CryptoError::OaepBlockTooSmall)?;
    if message.len() > capacity {
        return Err(CryptoError::MessageTooLong);
    }
    let mut seed = [0; HASH_LEN];
    rng.fill_bytes(&mut seed);
    let mut db = vec![0; k - HASH_LEN - 1];
    db[..HASH_LEN].copy_from_slice(&digest_381(&[0; HASH_LEN]));
    let delimiter = db.len() - message.len() - 1;
    db[delimiter] = 1;
    db[delimiter + 1..].copy_from_slice(message);
    let mask = mgf(&seed, db.len());
    xor(&mut db, &mask);
    xor(&mut seed, &mgf(&db, HASH_LEN));
    let mut block = Vec::with_capacity(k);
    block.push(0);
    block.extend_from_slice(&seed);
    block.extend_from_slice(&db);
    block.reverse();
    Ok(block)
}

/// Decode the little-endian block. Deliberately ignore its leading byte and
/// recovered label hash, as required by the legacy decoder.
pub fn oaep_decode(block_le: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if block_le.len() < OAEP_OVERHEAD {
        return Err(CryptoError::OaepBlockTooSmall);
    }
    let em: Vec<_> = block_le.iter().rev().copied().collect();
    let mut seed = em[1..17].to_vec();
    let mut db = em[17..].to_vec();
    xor(&mut seed, &mgf(&db, HASH_LEN));
    let mask = mgf(&seed, db.len());
    xor(&mut db, &mask);
    let delimiter = db[HASH_LEN..]
        .iter()
        .position(|&byte| byte != 0)
        .map(|offset| offset + HASH_LEN)
        .ok_or(CryptoError::InvalidOaepDelimiter)?;
    if db[delimiter] != 1 {
        return Err(CryptoError::InvalidOaepDelimiter);
    }
    Ok(db[delimiter + 1..].to_vec())
}

fn integer_field<'a>(input: &mut &'a [u8]) -> Result<&'a [u8], CryptoError> {
    let prefix: [u8; 4] = input
        .get(..4)
        .ok_or(CryptoError::InvalidIntegerLayout)?
        .try_into()
        .expect("four-byte prefix");
    let len = u32::from_le_bytes(prefix) as usize;
    let rest = &input[4..];
    let value = rest.get(..len).ok_or(CryptoError::InvalidIntegerLayout)?;
    *input = &rest[len..];
    Ok(value)
}

/// A raw RSA operation key. Its exponent is either 65537 or the supplied d.
/// Do not log or persist private-key blobs.
#[derive(Clone)]
pub struct RsaKey {
    modulus: BigUint,
    exponent: BigUint,
    block_len: usize,
}

impl RsaKey {
    /// Parse `[u32 n_length][n_le]`; the public exponent is always 65537.
    pub fn from_public_blob(mut blob: &[u8]) -> Result<Self, CryptoError> {
        let modulus = integer_field(&mut blob)?;
        if !blob.is_empty() {
            return Err(CryptoError::InvalidIntegerLayout);
        }
        Self::new(modulus, BigUint::from(65537u32))
    }

    /// Parse `[u32 n_length][n_le][u32 d_length][d_le]` from caller bytes.
    pub fn from_private_blob(mut blob: &[u8]) -> Result<Self, CryptoError> {
        let modulus = integer_field(&mut blob)?;
        let exponent = integer_field(&mut blob)?;
        if !blob.is_empty() {
            return Err(CryptoError::InvalidIntegerLayout);
        }
        Self::new(modulus, BigUint::from_bytes_le(exponent))
    }

    fn new(modulus_le: &[u8], exponent: BigUint) -> Result<Self, CryptoError> {
        let modulus = BigUint::from_bytes_le(modulus_le);
        let block_len = modulus_le.len();
        if block_len < OAEP_OVERHEAD
            || modulus.bits().div_ceil(8) != block_len as u64
            || exponent == BigUint::from(0u8)
        {
            return Err(CryptoError::InvalidRsaKey);
        }
        Ok(Self {
            modulus,
            exponent,
            block_len,
        })
    }

    pub fn block_len(&self) -> usize {
        self.block_len
    }

    pub fn capacity(&self) -> usize {
        self.block_len - OAEP_OVERHEAD
    }

    /// Apply this key's exponent to one fixed-width little-endian integer.
    pub fn apply_block(&self, block: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if block.len() != self.block_len {
            return Err(CryptoError::IncompleteRsaBlock);
        }
        let integer = BigUint::from_bytes_le(block);
        if integer >= self.modulus {
            return Err(CryptoError::RsaIntegerOutOfRange);
        }
        let mut output = integer.modpow(&self.exponent, &self.modulus).to_bytes_le();
        output.resize(self.block_len, 0);
        Ok(output)
    }

    /// OAEP-encode each chunk and apply this key's exponent.
    pub fn encrypt(
        &self,
        plaintext: &[u8],
        rng: &mut (impl RngCore + CryptoRng),
    ) -> Result<Vec<u8>, CryptoError> {
        // A zero-capacity key cannot encode a nonempty chunk.
        if self.capacity() == 0 && !plaintext.is_empty() {
            return Err(CryptoError::MessageTooLong);
        }
        let mut output = Vec::new();
        for chunk in plaintext.chunks(self.capacity().max(1)) {
            let block = oaep_encode(chunk, self.block_len, rng)?;
            output.extend_from_slice(&self.apply_block(&block)?);
        }
        Ok(output)
    }

    /// Apply this key's exponent and OAEP-decode every complete block.
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if !ciphertext.len().is_multiple_of(self.block_len) {
            return Err(CryptoError::IncompleteRsaBlock);
        }
        let mut output = Vec::new();
        for block in ciphertext.chunks_exact(self.block_len) {
            output.extend_from_slice(&oaep_decode(&self.apply_block(block)?)?);
        }
        Ok(output)
    }
}

/// Client-supplied factors, in g, p, A order. The unresolved g/p comparison in
/// spec 02 line 67 is not imposed here. No primality test is specified.
#[derive(Clone)]
pub struct DhParameters {
    generator: BigUint,
    modulus: BigUint,
    peer_public: BigUint,
}

pub struct DhExchange {
    /// Minimal-length little-endian B. There is no internal length prefix.
    pub reply: Vec<u8>,
    pub session_key: [u8; 16],
}

impl DhParameters {
    pub fn from_blob(mut blob: &[u8]) -> Result<Self, CryptoError> {
        let g = integer_field(&mut blob)?;
        let p = integer_field(&mut blob)?;
        let a = integer_field(&mut blob)?;
        if g.is_empty() || g.len() > 512 || p.is_empty() || p.len() > 32 || a.is_empty() {
            return Err(CryptoError::InvalidDhLength);
        }
        if !blob.is_empty() {
            return Err(CryptoError::InvalidIntegerLayout);
        }
        let modulus = BigUint::from_bytes_le(p);
        // 02 §4.1: the modulus must be larger than the generator.
        if modulus <= BigUint::from_bytes_le(g) {
            return Err(CryptoError::InvalidDhModulus);
        }
        if modulus <= BigUint::from(1u8) {
            return Err(CryptoError::InvalidDhModulus);
        }
        Ok(Self {
            generator: BigUint::from_bytes_le(g),
            modulus,
            peer_public: BigUint::from_bytes_le(a),
        })
    }

    pub fn exchange(&self, rng: &mut (impl RngCore + CryptoRng)) -> DhExchange {
        let exponent = BigUint::from(rng.next_u64() | (1u64 << 63) | 1);
        let reply = self
            .generator
            .modpow(&exponent, &self.modulus)
            .to_bytes_le();
        let shared = self
            .peer_public
            .modpow(&exponent, &self.modulus)
            .to_bytes_le();
        // BigUint represents zero as [0], so cyclic expansion is defined for it.
        let mut session_key = [0; 16];
        for (index, byte) in session_key.iter_mut().enumerate() {
            *byte = shared[index % shared.len()];
        }
        DhExchange { reply, session_key }
    }
}

/// Fixed session slots. Slot selection is explicit; there is no rotation state.
#[derive(Clone)]
pub struct SessionCrypt {
    slots: [Option<Aes128>; 2],
}

impl SessionCrypt {
    pub fn new(key: [u8; 16]) -> Self {
        Self::with_keys(key, None)
    }

    pub fn with_keys(slot0: [u8; 16], slot1: Option<[u8; 16]>) -> Self {
        Self {
            slots: [
                Some(Aes128::new(&slot0.into())),
                slot1.map(|k| Aes128::new(&k.into())),
            ],
        }
    }

    fn cipher(&self, code: u8, encrypt: bool) -> Result<&Aes128, CryptoError> {
        let slot = match code {
            0x8 => 0,
            0x9 => 1,
            0xa if !encrypt => 0,
            0xb if !encrypt => 1,
            _ => return Err(CryptoError::InvalidSessionCode(code)),
        };
        self.slots[slot]
            .as_ref()
            .ok_or(CryptoError::MissingSessionKey(slot))
    }

    /// Encrypt complete blocks and zero-pad only a final partial block.
    pub fn encrypt(&self, plaintext: &[u8], code: u8) -> Result<Vec<u8>, CryptoError> {
        let cipher = self.cipher(code, true)?;
        let len = plaintext
            .len()
            .checked_add(15)
            .ok_or(CryptoError::LengthOverflow)?
            / 16
            * 16;
        let mut output = plaintext.to_vec();
        output.resize(len, 0);
        for bytes in output.as_chunks_mut::<16>().0 {
            let mut block = Block::<Aes128>::default();
            block.copy_from_slice(bytes);
            cipher.encrypt_block(&mut block);
            bytes.copy_from_slice(&block);
        }
        Ok(output)
    }

    /// Return padded plaintext. Only the frame's inner size removes padding.
    /// Codes A/B use slots 0/1 without a third-party step (spec 03 section 8).
    pub fn decrypt(&self, ciphertext: &[u8], code: u8) -> Result<Vec<u8>, CryptoError> {
        let cipher = self.cipher(code, false)?;
        if !ciphertext.len().is_multiple_of(16) {
            return Err(CryptoError::IncompleteAesBlock);
        }
        let mut output = ciphertext.to_vec();
        for bytes in output.as_chunks_mut::<16>().0 {
            let mut block = Block::<Aes128>::default();
            block.copy_from_slice(bytes);
            cipher.decrypt_block(&mut block);
            bytes.copy_from_slice(&block);
        }
        Ok(output)
    }
}
