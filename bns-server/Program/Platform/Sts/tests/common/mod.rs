//! Independent test client, transcribed from section 07. No server helpers.
#![allow(dead_code)]
use num_bigint::BigUint;
use rand::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};

fn h(parts: &[&[u8]]) -> Vec<u8> {
    let mut hash = Sha256::new();
    for bytes in parts {
        hash.update(bytes);
    }
    hash.finalize().to_vec()
}
fn number(hash: &[u8]) -> BigUint {
    let words: Vec<u32> = hash
        .as_chunks::<4>()
        .0
        .iter()
        .map(|w| u32::from_le_bytes(*w))
        .rev()
        .collect();
    BigUint::new(words)
}
fn export(n: &BigUint) -> Vec<u8> {
    let mut bytes = n.to_bytes_le();
    if *n == BigUint::from(0u32) {
        return Vec::new();
    }
    bytes.resize(bytes.len().div_ceil(4) * 4, 0);
    bytes
}
fn n() -> BigUint {
    BigUint::parse_bytes(
        concat!(
            "EEAF0AB9ADB38DD69C33F80AFA8FC5E86072618775FF3C0B9EA2314C9C256576",
            "D674DF7496EA81D3383B4813D692C6E0E0D5D8E250B98BE48E495C1D6089DAD15",
            "DC7D7B46154D6B6CE8EF4AD69B15D4982559B297BCF1885C529F566660E57EC68",
            "EDBC3C05726CC02FD4CBF4976EAA9AFD5138FE8376435B9FC61D2FC0EB06E3"
        )
        .as_bytes(),
        16,
    )
    .unwrap()
}
pub struct Client {
    pub public: Vec<u8>,
    pub proof: Vec<u8>,
    pub server_proof: Vec<u8>,
    pub key: [u8; 64],
}
impl Client {
    pub fn new<R: RngCore + CryptoRng>(
        login: &str,
        password: &[u8],
        salt: &[u8],
        public_b: &[u8],
        rng: &mut R,
    ) -> Self {
        let modulus = n();
        let g = BigUint::from(2u32);
        let b = BigUint::from_bytes_le(public_b);
        assert!(b > BigUint::from(0u32) && b < modulus);
        let b_bytes = export(&b);
        let salt = export(&BigUint::from_bytes_le(salt));
        let mut random = [0u8; 32];
        rng.fill_bytes(&mut random);
        let a = BigUint::from_bytes_le(&random);
        let public = export(&g.modpow(&a, &modulus));
        let k = number(&h(&[&export(&modulus), &[2, 0, 0, 0]]));
        let inner = h(&[login.as_bytes(), b":", password]);
        let x = number(&h(&[&salt, &inner]));
        let u = number(&h(&[&public, &b_bytes]));
        let subtract = (k * g.modpow(&x, &modulus)) % &modulus;
        let base = (b + &modulus - subtract) % &modulus;
        let secret = base.modpow(&(a + u * x), &modulus);
        let mut bytes = export(&secret);
        while bytes.len() > 4 && bytes[0] == 0 {
            bytes.remove(0);
        }
        if bytes.len() % 2 == 1 {
            bytes.remove(0);
        }
        bytes.reverse();
        let mut even = Vec::new();
        let mut odd = Vec::new();
        for pair in bytes.as_chunks::<2>().0.iter() {
            even.push(pair[0]);
            odd.push(pair[1]);
        }
        let h_even = h(&[&even]);
        let h_odd = h(&[&odd]);
        let mut key = [0; 64];
        for (i, pair) in key.as_chunks_mut::<2>().0.iter_mut().enumerate() {
            pair[0] = h_even[i];
            pair[1] = h_odd[i];
        }
        let hn = h(&[&export(&modulus)]);
        let hg = h(&[&export(&g)]);
        let xor: Vec<u8> = hn.iter().zip(hg).map(|(a, b)| a ^ b).collect();
        let proof = h(&[
            &xor,
            &h(&[login.as_bytes()]),
            &salt,
            &public,
            &b_bytes,
            &key,
        ]);
        let server_proof = h(&[&public, &proof, &key]);
        Self {
            public,
            proof,
            server_proof,
            key,
        }
    }
}
pub fn generated<R: RngCore>(rng: &mut R) -> String {
    let mut bytes = [0; 16];
    rng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
