mod common;

use bloodlib::crypto::*;
use common::*;
use num_bigint::BigUint;
use ripemd::{Digest, Ripemd128};

#[test]
fn ripemd128_known_answers() {
    for (message, expected) in [
        ("", "cdf26213a150dc3ecb610f18f6b38b46"),
        ("a", "86be7afa339d0fc7cfc785e72f578d33"),
        ("abc", "c14a12199c66e4ba84636b0f69144c77"),
        ("message digest", "9e327b3d6e523062afc1132d7df9d1b8"),
        (
            "abcdefghijklmnopqrstuvwxyz",
            "fd2aa607f71dc8f510714922b371834e",
        ),
    ] {
        assert_eq!(hex(&Ripemd128::digest(message.as_bytes())), expected);
    }
}

#[test]
fn oaep_round_trips_and_capacity() {
    for k in [64, 128] {
        for len in [0, 1, k - 35, k - 34] {
            let message: Vec<_> = (0..len).map(|i| i as u8).collect();
            let block = oaep_encode(&message, k, &mut FixedRng(42)).unwrap();
            assert_eq!(block.len(), k);
            assert_eq!(block[k - 1], 0);
            assert_eq!(oaep_decode(&block).unwrap(), message);
        }
        assert_eq!(
            oaep_encode(&vec![0; k - 33], k, &mut FixedRng(0)),
            Err(CryptoError::MessageTooLong)
        );
    }
    assert_eq!(
        oaep_encode(&[], 33, &mut FixedRng(0)),
        Err(CryptoError::OaepBlockTooSmall)
    );
    assert_eq!(oaep_decode(&[0; 33]), Err(CryptoError::OaepBlockTooSmall));
    assert_eq!(
        oaep_decode(&oaep_encode(&[], 34, &mut FixedRng(0)).unwrap()).unwrap(),
        Vec::<u8>::new()
    );
}

#[test]
fn oaep_exact_label_mask_counter_and_reversal() {
    for k in [64, 128] {
        let mut db = vec![0; k - 17];
        db[..16].copy_from_slice(&ripemd128_381(&[0; 16]));
        let n = db.len();
        db[n - 4..].copy_from_slice(&[1, b'a', b'b', b'c']);
        let expected = oaep_with_db(0, [0; 16], &db);
        assert_eq!(oaep_encode(b"abc", k, &mut FixedRng(0)).unwrap(), expected);
    }
}

#[test]
fn oaep_permissive_leading_byte_and_label_hash() {
    for k in [64, 128] {
        let valid = oaep_encode(b"hello", k, &mut FixedRng(0)).unwrap();
        let mut wrong_leading = valid.clone();
        wrong_leading[k - 1] = 0xa7;
        assert_eq!(oaep_decode(&wrong_leading).unwrap(), b"hello");

        let mut db = vec![0; k - 17];
        db[..16].fill(0x42); // Not H([FF;16]).
        let n = db.len();
        db[n - 6..].copy_from_slice(b"\x01hello");
        for leading in [0, 0xa7] {
            let wrong_hash = oaep_with_db(leading, [0; 16], &db);
            assert_eq!(oaep_decode(&wrong_hash).unwrap(), b"hello");
        }
    }
}

#[test]
fn oaep_rejects_missing_or_non_one_delimiter() {
    for k in [64, 128] {
        for delimiter in [0, 2, 0xff] {
            let mut db = vec![0; k - 17];
            db[..16].fill(0x33);
            db[20] = delimiter;
            assert_eq!(
                oaep_decode(&oaep_with_db(0, [0; 16], &db)),
                Err(CryptoError::InvalidOaepDelimiter)
            );
        }
    }
}

#[test]
fn rsa_chunking_in_both_directions_and_fixed_width() {
    for bits in [512, 1024] {
        let (public, private, modulus) = rsa_keys(bits);
        let k = bits / 8;
        let capacity = k - 34;
        assert_eq!(public.block_len(), k);
        assert_eq!(private.capacity(), capacity);
        assert_eq!(public.apply_block(&vec![0; k]).unwrap(), vec![0; k]);
        let mut one = vec![0; k];
        one[0] = 1;
        assert_eq!(public.apply_block(&one).unwrap(), one);
        assert_eq!(
            public.apply_block(&modulus),
            Err(CryptoError::RsaIntegerOutOfRange)
        );
        for len in [0, 1, capacity, capacity + 1, capacity * 2, capacity * 3 + 7] {
            let message: Vec<_> = (0..len).map(|i| i as u8).collect();
            for (sender, receiver) in [(&public, &private), (&private, &public)] {
                let ciphertext = sender.encrypt(&message, &mut FixedRng(42)).unwrap();
                assert_eq!(ciphertext.len(), len.div_ceil(capacity) * k);
                assert_eq!(receiver.decrypt(&ciphertext).unwrap(), message);
            }
        }
        assert_eq!(
            private.decrypt(&vec![0; k - 1]),
            Err(CryptoError::IncompleteRsaBlock)
        );
        assert_eq!(
            public.apply_block(&vec![0; k + 1]),
            Err(CryptoError::IncompleteRsaBlock)
        );
    }
}

#[test]
fn rsa_blob_validation() {
    for blob in [
        vec![],
        vec![1, 0, 0],
        vec![128, 0, 0, 0],
        vec![0, 0, 0, 0, 0],
    ] {
        assert!(matches!(
            RsaKey::from_public_blob(&blob),
            Err(CryptoError::InvalidIntegerLayout)
        ));
    }
    let mut short = Vec::new();
    field(&[3], &mut short);
    assert!(matches!(
        RsaKey::from_public_blob(&short),
        Err(CryptoError::InvalidRsaKey)
    ));
    let mut private = Vec::new();
    field(&[0xff; 64], &mut private);
    field(&[0], &mut private);
    assert!(matches!(
        RsaKey::from_private_blob(&private),
        Err(CryptoError::InvalidRsaKey)
    ));
    private.push(0);
    assert!(matches!(
        RsaKey::from_private_blob(&private),
        Err(CryptoError::InvalidIntegerLayout)
    ));
}

fn dh_blob(g: &[u8], p: &[u8], a: &[u8]) -> Vec<u8> {
    let mut blob = Vec::new();
    for bytes in [g, p, a] {
        field(bytes, &mut blob);
    }
    blob
}

#[test]
fn dh_agreement_field_order_exponent_bits_and_reply() {
    let g = BigUint::from(3u8);
    let p = BigUint::from(65539u32);
    let client_secret = BigUint::from(123u8);
    let a = g.modpow(&client_secret, &p);
    let params = DhParameters::from_blob(&dh_blob(
        &g.to_bytes_le(),
        &p.to_bytes_le(),
        &a.to_bytes_le(),
    ))
    .unwrap();
    for random in [0, 2, 0x0123456789abcdef, u64::MAX] {
        let exchange = params.exchange(&mut FixedRng(random));
        let exponent = BigUint::from(random | (1u64 << 63) | 1);
        assert_eq!(exchange.reply, g.modpow(&exponent, &p).to_bytes_le());
        let b = BigUint::from_bytes_le(&exchange.reply);
        let shared = b.modpow(&client_secret, &p).to_bytes_le();
        let expected: Vec<_> = shared.iter().copied().cycle().take(16).collect();
        assert_eq!(exchange.session_key.as_slice(), expected);
    }
}

#[test]
fn dh_short_shared_secret_expands_cyclically() {
    // x = 2^63 + 1, so x == 1 mod 65536. Modulo 65537, S == A.
    for a in [vec![2], vec![2, 1], vec![0]] {
        let params = DhParameters::from_blob(&dh_blob(&[3], &[1, 0, 1], &a)).unwrap();
        let exchange = params.exchange(&mut FixedRng(0));
        assert_eq!(exchange.reply, [3]);
        let expected: Vec<_> = a.iter().copied().cycle().take(16).collect();
        assert_eq!(exchange.session_key.as_slice(), expected);
    }
}

#[test]
fn dh_long_shared_secret_is_truncated() {
    // A = 1 and p = 2^255 - 19 give S = 1. Also test a full-width result.
    let p = (BigUint::from(1u8) << 255usize) - BigUint::from(19u8);
    let a = &p - BigUint::from(1u8); // Odd x gives S = p-1.
    let params =
        DhParameters::from_blob(&dh_blob(&[2], &p.to_bytes_le(), &a.to_bytes_le())).unwrap();
    let exchange = params.exchange(&mut FixedRng(0));
    assert_eq!(&exchange.session_key, &a.to_bytes_le()[..16]);
    assert!(exchange.reply.len() <= 32);
}

#[test]
fn dh_rejects_bad_layout_lengths_and_modulus() {
    assert!(matches!(
        DhParameters::from_blob(&[0]),
        Err(CryptoError::InvalidIntegerLayout)
    ));
    for (g, p, a) in [
        (vec![], vec![23], vec![1]),
        (vec![1; 513], vec![23], vec![1]),
        (vec![2], vec![], vec![1]),
        (vec![2], vec![1; 33], vec![1]),
        (vec![2], vec![23], vec![]),
    ] {
        assert!(matches!(
            DhParameters::from_blob(&dh_blob(&g, &p, &a)),
            Err(CryptoError::InvalidDhLength)
        ));
    }
    for p in [0, 1] {
        assert!(matches!(
            DhParameters::from_blob(&dh_blob(&[2], &[p], &[1])),
            Err(CryptoError::InvalidDhModulus)
        ));
    }
    let mut trailing = dh_blob(&[2], &[23], &[3]);
    trailing.push(0);
    assert!(matches!(
        DhParameters::from_blob(&trailing),
        Err(CryptoError::InvalidIntegerLayout)
    ));
}

#[test]
fn aes_known_answer_padding_slots_and_alias_codes() {
    let crypt = SessionCrypt::with_keys([0; 16], Some([1; 16]));
    let ciphertext = crypt.encrypt(&[0; 16], 8).unwrap();
    assert_eq!(hex(&ciphertext), "66e94bd4ef8a2c3b884cfa59ca342b2e");
    assert_eq!(crypt.encrypt(&[0; 1], 8).unwrap(), ciphertext);
    assert_eq!(crypt.encrypt(&[0; 17], 8).unwrap(), ciphertext.repeat(2));
    for code in [8, 9] {
        for len in [0, 1, 15, 16, 17, 32] {
            let plaintext = vec![0x42; len];
            let encoded = crypt.encrypt(&plaintext, code).unwrap();
            assert_eq!(encoded.len(), len.div_ceil(16) * 16);
            for receive_code in [code, code + 2] {
                let decoded = crypt.decrypt(&encoded, receive_code).unwrap();
                assert_eq!(&decoded[..len], plaintext);
                assert!(decoded[len..].iter().all(|&b| b == 0));
            }
        }
    }
    assert_ne!(
        crypt.encrypt(&[0; 16], 8).unwrap(),
        crypt.encrypt(&[0; 16], 9).unwrap()
    );
    assert_eq!(
        crypt.decrypt(&[0; 15], 8),
        Err(CryptoError::IncompleteAesBlock)
    );
    assert_eq!(
        crypt.encrypt(&[], 10),
        Err(CryptoError::InvalidSessionCode(10))
    );
    assert_eq!(
        crypt.decrypt(&[], 7),
        Err(CryptoError::InvalidSessionCode(7))
    );
    assert_eq!(
        SessionCrypt::new([0; 16]).decrypt(&[], 9),
        Err(CryptoError::MissingSessionKey(1))
    );
}

#[test]
fn dh_rejects_modulus_not_larger_than_generator() {
    let blob = dh_blob(&[23], &[23], &[5]);
    assert_eq!(
        DhParameters::from_blob(&blob).err(),
        Some(CryptoError::InvalidDhModulus)
    );
    let blob = dh_blob(&[24], &[23], &[5]);
    assert_eq!(
        DhParameters::from_blob(&blob).err(),
        Some(CryptoError::InvalidDhModulus)
    );
}
