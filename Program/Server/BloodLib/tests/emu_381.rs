//! Black-box vectors from the unmodified 0.0.381.572 functions in Unicorn.
//! Regenerate with tools/emu/generate_381.py. All key material is synthetic.

use bloodlib::crypto::*;
use bloodlib::session::SessionPacketCrypt;

fn stateful_session(case: &Value) -> SessionPacketCrypt {
    let mut crypt = SessionPacketCrypt::new(bytes(case, "key0").try_into().unwrap());
    if case["before"]["pending"] == 1 || case["before"]["send"] == 1 {
        crypt
            .prepare_key_change(bytes(case, "key1").try_into().unwrap())
            .unwrap();
    }
    if case["before"]["send"] == 1 {
        crypt.encrypt(&[0; 16], 18, false).unwrap();
    }
    if case["before"]["receive"] == 1 {
        let packet = crypt.encrypt(&[0; 16], 18, false).unwrap();
        crypt.decrypt(&packet, 16).unwrap();
    }
    assert_eq!(
        crypt.state(),
        (
            case["before"]["send"].as_u64().unwrap() as usize,
            case["before"]["receive"].as_u64().unwrap() as usize,
            case["before"]["pending"] == 1
        )
    );
    crypt
}
use rand::{CryptoRng, RngCore};
use serde_json::Value;

fn vectors() -> Vec<Value> {
    let root: Value = serde_json::from_str(include_str!("vectors/emu_381.json")).unwrap();
    assert_eq!(root["build"], "0.0.381.572");
    assert_eq!(root["pe_timestamp"], "0x5eb5328b");
    root["cases"].as_array().unwrap().clone()
}

fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn bytes(case: &Value, key: &str) -> Vec<u8> {
    unhex(case[key].as_str().unwrap())
}

fn evidence(case: &Value) {
    assert!(case["calls"].as_array().unwrap().iter().any(|call| {
        call["address"] == case["source"]
            && call["sentinel"] == true
            && call["instructions"].as_u64().unwrap() > 1
    }));
}

/// A finite stream. Any unexpected random consumption fails the test.
struct StreamRng {
    bytes: Vec<u8>,
    offset: usize,
}

impl StreamRng {
    fn new(bytes: Vec<u8>) -> Self {
        Self { bytes, offset: 0 }
    }
}

impl RngCore for StreamRng {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0; 4];
        self.fill_bytes(&mut bytes);
        u32::from_le_bytes(bytes)
    }
    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0; 8];
        self.fill_bytes(&mut bytes);
        u64::from_le_bytes(bytes)
    }
    fn fill_bytes(&mut self, output: &mut [u8]) {
        output.copy_from_slice(&self.bytes[self.offset..self.offset + output.len()]);
        self.offset += output.len();
    }
    fn try_fill_bytes(&mut self, output: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(output);
        Ok(())
    }
}
impl CryptoRng for StreamRng {}

fn hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn original_381_ripemd128_empty() {
    let case = vectors()
        .into_iter()
        .find(|c| c["kind"] == "ripemd128" && c["case"] == "empty")
        .unwrap();
    evidence(&case);
    assert_eq!(
        ripemd128_381(&bytes(&case, "input")).as_slice(),
        bytes(&case, "output")
    );
}

#[test]
fn original_381_ripemd128_differences() {
    let cases: Vec<_> = vectors()
        .into_iter()
        .filter(|c| c["kind"] == "ripemd128")
        .collect();
    assert_eq!(cases.len(), 3);
    let mut mismatches = 0;
    for case in cases {
        evidence(&case);
        let input = bytes(&case, "input");
        let expected = bytes(&case, "output");
        let actual = ripemd128_381(&input);
        assert_ne!(input, expected);
        if actual.as_slice() != expected {
            println!(
                "{}: original={} rust={}",
                case["case"],
                hex(&expected),
                hex(&actual)
            );
            mismatches += 1;
        }
    }
    assert_eq!(mismatches, 0, "381 RIPEMD differences");
}

#[test]
fn original_381_mgf_through_production_oaep() {
    let cases: Vec<_> = vectors()
        .into_iter()
        .filter(|c| c["kind"] == "mgf")
        .collect();
    assert_eq!(cases.len(), 3);
    let mut mismatches = 0;
    for case in cases {
        evidence(&case);
        let input = bytes(&case, "input");
        let expected = bytes(&case, "output");
        let n = case["length"].as_u64().unwrap() as usize;
        // OAEP's masked DB is DB XOR MGF(seed, k-17). Recover the private
        // production MGF output through this public boundary, not a copy of MGF.
        let k = (n + 17).max(34);
        let mut rng = StreamRng::new(input);
        let mut em = oaep_encode(&[], k, &mut rng).unwrap();
        em.reverse();
        let mut db = vec![0; k - 17];
        db[..16].copy_from_slice(&ripemd128_381(&[0; 16]));
        *db.last_mut().unwrap() = 1;
        let actual: Vec<_> = em[17..]
            .iter()
            .zip(db)
            .map(|(a, b)| a ^ b)
            .take(n)
            .collect();
        if actual != expected {
            println!(
                "{}: input={} original={} rust={}",
                case["case"],
                case["input"],
                hex(&expected),
                hex(&actual)
            );
            mismatches += 1;
        }
        assert_eq!(rng.offset, 16);
    }
    assert_eq!(mismatches, 0, "381 MGF differences");
}

#[test]
fn original_381_oaep_encode() {
    let mut mismatches = 0;
    let cases: Vec<_> = vectors()
        .into_iter()
        .filter(|c| c["kind"] == "oaep_encode")
        .collect();
    assert_eq!(cases.len(), 8);
    for case in cases {
        evidence(&case);
        let input = bytes(&case, "input");
        let mut rng = StreamRng::new(bytes(&case, "seed"));
        let actual = oaep_encode(&input, case["k"].as_u64().unwrap() as usize, &mut rng);
        if case["result"] == 1 {
            let expected = bytes(&case, "output");
            assert_ne!(expected, input);
            let actual = actual.unwrap();
            if actual != expected {
                println!(
                    "OAEP mismatch: case={} input={} seed={} original={} rust={}",
                    case["case"],
                    case["input"],
                    case["seed"],
                    hex(&expected),
                    hex(&actual)
                );
                mismatches += 1;
            }
        } else {
            assert_eq!(actual, Err(CryptoError::MessageTooLong), "{}", case["case"]);
        }
        assert_eq!(rng.offset as u64, case["random_used"].as_u64().unwrap());
    }
    assert_eq!(mismatches, 0, "381 OAEP encode differences");
}

fn rsa_evidence(case: &Value) {
    evidence(case);
    let setup = case["setup_calls"].as_array().unwrap();
    for address in ["0x1400668e0", "0x140066b50"] {
        assert!(setup.iter().any(|c| c["address"] == address
            && c["sentinel"] == true
            && c["instructions"].as_u64().unwrap() > 1));
    }
}

#[test]
fn original_381_rsa_encrypt() {
    let cases: Vec<_> = vectors()
        .into_iter()
        .filter(|c| c["kind"] == "rsa_encrypt")
        .collect();
    assert_eq!(cases.len(), 2);
    let mut mismatches = 0;
    for case in cases {
        rsa_evidence(&case);
        let key = RsaKey::from_public_blob(&bytes(&case, "public_blob")).unwrap();
        let input = bytes(&case, "input");
        let mut rng = StreamRng::new(bytes(&case, "seed"));
        let actual = key.encrypt(&input, &mut rng).unwrap();
        let expected = bytes(&case, "output");
        assert_ne!(input, expected);
        assert_eq!(rng.offset as u64, case["random_used"].as_u64().unwrap());
        if actual != expected {
            println!(
                "RSA mismatch: case={} input={} seed={} key=vector.public_blob original={} rust={}",
                case["case"],
                case["input"],
                case["seed"],
                hex(&expected),
                hex(&actual)
            );
            mismatches += 1;
        }
        // The raw integer operation must still match the original. Recover
        // each original OAEP block with the generated private exponent.
        let private = RsaKey::from_private_blob(&bytes(&case, "private_blob")).unwrap();
        for block in expected.as_chunks::<128>().0 {
            let em = private.apply_block(block).unwrap();
            assert_eq!(key.apply_block(&em).unwrap(), block);
        }
    }
    assert_eq!(mismatches, 0, "381 RSA encrypt differences");
}

#[test]
fn original_381_rsa_decrypt() {
    let cases: Vec<_> = vectors()
        .into_iter()
        .filter(|c| c["kind"] == "rsa_decrypt")
        .collect();
    assert_eq!(cases.len(), 4);
    for case in cases {
        rsa_evidence(&case);
        let key = RsaKey::from_private_blob(&bytes(&case, "private_blob")).unwrap();
        let input = bytes(&case, "input");
        let actual = key.decrypt(&input);
        if case["result"] == 1 {
            let expected = bytes(&case, "output");
            assert_ne!(input, expected);
            assert_eq!(actual.unwrap(), expected, "{}", case["case"]);
        } else if case["case"] == "incomplete" {
            assert_eq!(actual, Err(CryptoError::IncompleteRsaBlock));
        } else {
            assert_eq!(case["case"], "bad_oaep");
            assert!(case["object_changes"]
                .as_object()
                .unwrap()
                .values()
                .any(|v| *v == 17));
            assert_eq!(actual, Err(CryptoError::InvalidOaepDelimiter));
        }
    }
}

#[test]
fn original_381_rsa_bad_oaep() {
    let case = vectors()
        .into_iter()
        .find(|c| c["kind"] == "rsa_decrypt" && c["case"] == "bad_oaep")
        .expect("No verified original bad-OAEP vector");
    rsa_evidence(&case);
    assert_eq!(case["result"], 0);
    assert!(case["object_changes"]
        .as_object()
        .unwrap()
        .values()
        .any(|v| *v == 17));
    let key = RsaKey::from_private_blob(&bytes(&case, "private_blob")).unwrap();
    assert_eq!(
        key.decrypt(&bytes(&case, "input")),
        Err(CryptoError::InvalidOaepDelimiter)
    );
}

#[test]
fn rust_only_rsa_zero_ciphertext() {
    // No claim about the original: its zero-base path did not reach the sentinel.
    let case = vectors()
        .into_iter()
        .find(|c| c["kind"] == "rsa_encrypt")
        .unwrap();
    let key = RsaKey::from_private_blob(&bytes(&case, "private_blob")).unwrap();
    assert_eq!(
        key.decrypt(&[0; 128]),
        Err(CryptoError::InvalidOaepDelimiter)
    );
}

#[test]
fn original_381_dh_exchange() {
    let cases: Vec<_> = vectors()
        .into_iter()
        .filter(|c| c["kind"] == "dh")
        .collect();
    assert_eq!(cases.len(), 3);
    for case in cases {
        evidence(&case);
        for address in ["0x140067850", "0x140067a10", "0x140067660"] {
            assert!(case["calls"].as_array().unwrap().iter().any(|call| {
                call["address"] == address
                    && call["sentinel"] == true
                    && call["instructions"].as_u64().unwrap() > 1
            }));
        }
        let input = bytes(&case, "input");
        let params = DhParameters::from_blob(&input).unwrap();
        let seed = bytes(&case, "seed");
        let exponent = u64::from_le_bytes(seed.as_slice().try_into().unwrap()) | (1u64 << 63) | 1;
        assert_eq!(bytes(&case, "exponent"), exponent.to_le_bytes());
        assert_eq!(case["exponent_bits"], 64);
        let mut rng = StreamRng::new(seed);
        let actual = params.exchange(&mut rng);
        let expected = bytes(&case, "output");
        let expected_key = bytes(&case, "session_key");
        assert_ne!(expected, input);
        assert_ne!(expected_key, input);
        assert_eq!(actual.reply, expected, "DH reply: {}", case["case"]);
        assert_eq!(
            actual.session_key.as_slice(),
            expected_key,
            "DH key: {}",
            case["case"]
        );
        assert_eq!(rng.offset as u64, case["random_used"].as_u64().unwrap());
        if case["case"] == "short_shared" {
            for pair in expected_key.as_chunks::<2>().0 {
                assert_eq!(pair, &expected_key[..2]);
            }
        }
    }
}

#[test]
fn original_381_dh_reject_modulus() {
    let cases: Vec<_> = vectors()
        .into_iter()
        .filter(|c| c["kind"] == "dh_reject")
        .collect();
    assert_eq!(cases.len(), 2);
    for case in cases {
        evidence(&case);
        assert_eq!(case["result"], 0);
        assert!(matches!(
            DhParameters::from_blob(&bytes(&case, "input")),
            Err(CryptoError::InvalidDhModulus)
        ));
    }
}

fn session(case: &Value) -> SessionCrypt {
    SessionCrypt::with_keys(
        bytes(case, "key0").try_into().unwrap(),
        Some(bytes(case, "key1").try_into().unwrap()),
    )
}

fn session_packet(case: &Value, code: u8) -> Vec<u8> {
    let body = session(case).encrypt(&bytes(case, "input"), code).unwrap();
    let tag = code + if case["flag"] == true { 2 } else { 0 };
    let mut packet = (((tag as u16) << 12) | (body.len() as u16 / 4))
        .to_le_bytes()
        .to_vec();
    packet.extend(body);
    packet
}

#[test]
fn original_381_session_encrypt_explicit_slots() {
    let cases: Vec<_> = vectors()
        .into_iter()
        .filter(|c| c["kind"] == "session_encrypt" && c["result"] == 0)
        .collect();
    assert_eq!(cases.len(), 9);
    for case in cases {
        evidence(&case);
        let code = 8 + case["before"]["send"].as_u64().unwrap() as u8;
        assert_eq!(
            session_packet(&case, code),
            bytes(&case, "output"),
            "{}",
            case["case"]
        );
        assert_ne!(bytes(&case, "input"), bytes(&case, "output"));
        if case["before"]["pending"] == 1 {
            assert_eq!(case["after"]["send"], 1);
            assert_eq!(case["after"]["pending"], 0);
        }
    }
}

#[test]
fn original_381_session_decrypt_explicit_slots() {
    let cases: Vec<_> = vectors()
        .into_iter()
        .filter(|c| c["kind"] == "session_decrypt" && c["result"] == 0)
        .collect();
    assert_eq!(cases.len(), 9);
    for case in cases {
        evidence(&case);
        let packet = bytes(&case, "input");
        let header = u16::from_le_bytes(packet[..2].try_into().unwrap());
        let tag = (header >> 12) as u8;
        assert_eq!(usize::from(header & 0xFFF) * 4 + 2, packet.len());
        assert_eq!(case["flag"].as_u64().unwrap(), u64::from(tag >= 10));
        assert_eq!(
            session(&case).decrypt(&packet[2..], tag).unwrap(),
            bytes(&case, "output"),
            "{}",
            case["case"]
        );
        assert_ne!(packet[2..], bytes(&case, "output"));
        if case["case"] == "rotate_send_new" || case["case"] == "rotate_reply_new" {
            assert_eq!(case["before"]["receive"], 0);
            assert_eq!(case["after"]["receive"], 1);
        }
    }
}

#[test]
fn original_381_session_errors() {
    let cases: Vec<_> = vectors()
        .into_iter()
        .filter(|c| c["kind"].as_str().unwrap().starts_with("session_") && c["result"] != 0)
        .collect();
    assert_eq!(cases.len(), 7);
    for case in cases {
        evidence(&case);
        assert_ne!(bytes(&case, "error_output"), bytes(&case, "input"));
        match case["case"].as_str().unwrap() {
            "wrong_tag" => {
                assert_eq!(case["result"], 7);
                let packet = bytes(&case, "input");
                assert_eq!(
                    session(&case).decrypt(&packet[2..], 12),
                    Err(CryptoError::InvalidSessionCode(12))
                );
            }
            "truncated" => {
                assert_eq!(case["result"], 10);
                let packet = bytes(&case, "input");
                assert_eq!(
                    session(&case).decrypt(&packet[2..], 8),
                    Err(CryptoError::IncompleteAesBlock)
                );
            }
            "no_key" => assert_eq!(case["result"], 3), // Rust cannot represent no configured slot 0.
            "wrong_active_slot" => assert_eq!(case["result"], 7),
            "too_large" => assert_eq!(case["result"], 8),
            "small_buffer" => assert_eq!(case["result"], 9),
            other => panic!("Unknown error case: {other}"),
        }
    }
}

#[test]
fn session_gap_outputs() {
    let mut report = Vec::new();
    for case in vectors().into_iter().filter(|c| {
        (c["kind"] == "session_encrypt"
            && ["too_large", "small_buffer", "rotate_send_new"]
                .iter()
                .any(|name| c["case"] == *name))
            || (c["kind"] == "session_decrypt" && c["case"] == "wrong_active_slot")
    }) {
        evidence(&case);
        let input = bytes(&case, "input");
        let actual = if case["case"] == "wrong_active_slot" {
            assert_eq!(case["before"]["send"], 0);
            assert_eq!(case["before"]["receive"], 0);
            session(&case).decrypt(&input[2..], 9).unwrap()
        } else if case["case"] == "rotate_send_new" {
            session_packet(&case, 8) // No automatic transition exists in Rust.
        } else {
            session(&case).encrypt(&input, 8).unwrap()
        };
        if case["case"] == "too_large" {
            assert_eq!(actual.len(), 4016);
        }
        if case["case"] == "small_buffer" {
            assert_eq!(actual.len(), 32);
        }
        if case["result"] == 0 {
            assert_ne!(actual, bytes(&case, "output"));
        }
        report.push(serde_json::json!({
            "case": case["case"], "input": case["input"],
            "key0": case["key0"], "key1": case["key1"],
            "before": case["before"], "capacity": case["capacity"],
            "flag": case["flag"], "original_result": case["result"],
            "original_output": case["output"], "rust_result": "Ok",
            "rust_output": hex(&actual), "rust_output_length": actual.len()
        }));
    }
    assert_eq!(report.len(), 4);
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}

#[test]
fn original_381_session_size_limit() {
    let case = vectors()
        .into_iter()
        .find(|c| c["kind"] == "session_encrypt" && c["case"] == "too_large")
        .unwrap();
    evidence(&case);
    let mut crypt = stateful_session(&case);
    let before = crypt.state();
    let err = crypt
        .encrypt(
            &bytes(&case, "input"),
            case["capacity"].as_u64().unwrap() as usize,
            false,
        )
        .unwrap_err();
    assert_eq!(
        u64::from(err.code().unwrap()),
        case["result"].as_u64().unwrap()
    );
    assert_eq!(crypt.state(), before);
}

#[test]
fn original_381_session_small_buffer() {
    let case = vectors()
        .into_iter()
        .find(|c| c["kind"] == "session_encrypt" && c["case"] == "small_buffer")
        .unwrap();
    evidence(&case);
    let mut crypt = stateful_session(&case);
    let before = crypt.state();
    let err = crypt
        .encrypt(
            &bytes(&case, "input"),
            case["capacity"].as_u64().unwrap() as usize,
            false,
        )
        .unwrap_err();
    assert_eq!(
        u64::from(err.code().unwrap()),
        case["result"].as_u64().unwrap()
    );
    assert_eq!(crypt.state(), before);
}

#[test]
fn original_381_session_wrong_active_slot() {
    let case = vectors()
        .into_iter()
        .find(|c| c["kind"] == "session_decrypt" && c["case"] == "wrong_active_slot")
        .unwrap();
    evidence(&case);
    let packet = bytes(&case, "input");
    let mut crypt = stateful_session(&case);
    let before = crypt.state();
    assert_eq!(crypt.decrypt(&packet, 4000).unwrap_err().code(), Some(7));
    assert_eq!(crypt.state(), before);
}

#[test]
fn original_381_session_automatic_key_change() {
    let case = vectors()
        .into_iter()
        .find(|c| c["kind"] == "session_encrypt" && c["case"] == "rotate_send_new")
        .unwrap();
    evidence(&case);
    assert_eq!(
        stateful_session(&case)
            .encrypt(
                &bytes(&case, "input"),
                case["capacity"].as_u64().unwrap() as usize,
                case["flag"] == true
            )
            .unwrap(),
        bytes(&case, "output")
    );
}

#[test]
fn original_381_oaep_decode() {
    let cases: Vec<_> = vectors()
        .into_iter()
        .filter(|c| c["kind"] == "oaep_decode")
        .collect();
    assert_eq!(cases.len(), 12);
    for case in cases {
        evidence(&case);
        let input = bytes(&case, "input");
        let actual = oaep_decode(&input);
        if case["result"] == 1 {
            let expected = bytes(&case, "output");
            assert_ne!(expected, input);
            assert_eq!(actual.unwrap(), expected, "{}", case["case"]);
        } else {
            assert_eq!(
                actual,
                Err(CryptoError::InvalidOaepDelimiter),
                "{}",
                case["case"]
            );
        }
    }
}
