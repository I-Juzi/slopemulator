#![allow(dead_code)]

use bloodlib::crypto::{ripemd128_381, RsaKey};
use rand::{CryptoRng, RngCore, SeedableRng};
use rsa::{
    traits::{PrivateKeyParts, PublicKeyParts},
    RsaPrivateKey,
};

pub fn field(bytes: &[u8], output: &mut Vec<u8>) {
    output.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    output.extend_from_slice(bytes);
}

pub fn rsa_keys(bits: usize) -> (RsaKey, RsaKey, Vec<u8>) {
    // Generate test material at runtime. No production key is read or embedded.
    let mut rng = rand::rngs::StdRng::seed_from_u64(bits as u64);
    let key = RsaPrivateKey::new(&mut rng, bits).unwrap();
    let n = key.n().to_bytes_le();
    let mut public_blob = Vec::new();
    field(&n, &mut public_blob);
    let mut private_blob = public_blob.clone();
    let mut d = key.d().to_bytes_le();
    d.resize(bits / 8, 0);
    field(&d, &mut private_blob);
    (
        RsaKey::from_public_blob(&public_blob).unwrap(),
        RsaKey::from_private_blob(&private_blob).unwrap(),
        n,
    )
}

/// Deterministic RNG for tests only.
pub struct FixedRng(pub u64);
impl RngCore for FixedRng {
    fn next_u32(&mut self) -> u32 {
        self.0 as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.0
    }
    fn fill_bytes(&mut self, output: &mut [u8]) {
        for (i, byte) in output.iter_mut().enumerate() {
            *byte = self.0.to_le_bytes()[i % 8];
        }
    }
    fn try_fill_bytes(&mut self, output: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(output);
        Ok(())
    }
}
impl CryptoRng for FixedRng {}

fn mask(input: &[u8], len: usize) -> Vec<u8> {
    let mut out = Vec::new();
    let mut counter = 0u32;
    while out.len() < len {
        let mut message = input.to_vec();
        message.extend_from_slice(&counter.to_le_bytes());
        out.extend_from_slice(&ripemd128_381(&message));
        counter += 1;
    }
    out.truncate(len);
    out
}

/// Build a conceptual OAEP block independently, then reverse it for the wire.
pub fn oaep_with_db(leading: u8, seed: [u8; 16], db: &[u8]) -> Vec<u8> {
    let masked_db: Vec<_> = db
        .iter()
        .zip(mask(&seed, db.len()))
        .map(|(a, b)| a ^ b)
        .collect();
    let masked_seed: Vec<_> = seed
        .iter()
        .zip(mask(&masked_db, 16))
        .map(|(a, b)| a ^ b)
        .collect();
    let mut em = vec![leading];
    em.extend(masked_seed);
    em.extend(masked_db);
    em.reverse();
    em
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
