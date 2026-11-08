mod common;

use common::*;
use bloodlib::{crypto::*, frames::*};

fn wire(code: u8, payload: &[u8]) -> Vec<u8> {
    let mut frame = encrypted_header(code, payload.len())
        .unwrap()
        .to_le_bytes()
        .to_vec();
    frame.extend_from_slice(payload);
    frame
}

fn session_decode(input: &[u8], crypt: &SessionCrypt) -> Result<Option<DecodedFrame>, FrameError> {
    decode_client(input, ClientDecrypt::Session { crypt, mode: 5 })
}

#[test]
fn all_header_lengths_and_limits() {
    // The spec's len > 0xFFFF receive error cannot occur for any u16 header.
    for header in 0..=u16::MAX {
        let len = payload_len(header);
        assert!(len <= 0xffff);
        if header >> 14 == 0 {
            assert_eq!(len, usize::from(header & 0x3fff));
        } else {
            assert_eq!(len, usize::from(header & 0xfff) * 4);
            assert_eq!(encrypted_header((header >> 12) as u8, len).unwrap(), header);
        }
    }
    assert_eq!(encrypted_header(4, 128).unwrap(), 0x4020);
    assert_eq!(encrypted_header(5, 64).unwrap(), 0x5010);
    assert_eq!(encrypted_header(8, 16).unwrap(), 0x8004);
    assert_eq!(encrypted_header(9, MAX_ENCRYPTED_PAYLOAD).unwrap(), 0x9fff);
    assert_eq!(
        encrypted_header(8, 16384),
        Err(FrameError::PayloadTooLarge(16384))
    );
    assert_eq!(encrypted_header(8, 3), Err(FrameError::UnalignedPayload(3)));
    assert_eq!(
        encrypted_header(3, 4),
        Err(FrameError::InvalidEncryptionCode(3))
    );
}

#[test]
fn trusted_round_trip_stream_boundaries_and_minimum() {
    for len in [2, 4, 4000] {
        let mut packet = vec![0x42; len];
        packet[..2].copy_from_slice(&(len as i16).to_le_bytes());
        let encoded = encode_trusted(&packet).unwrap();
        assert_eq!(encoded, packet);
        for cut in 0..len {
            assert_eq!(decode_trusted(&encoded[..cut]).unwrap(), None);
        }
        let combined = encoded.repeat(2);
        let decoded = decode_trusted(&combined).unwrap().unwrap();
        assert_eq!(decoded.packet, packet);
        assert_eq!(decoded.consumed, len);
    }
}

#[test]
fn trusted_errors() {
    for size in [i16::MIN, -1, 0, 1, 4001, i16::MAX] {
        let bytes = size.to_le_bytes();
        assert_eq!(
            decode_trusted(&bytes),
            Err(FrameError::InvalidTrustedSize(size))
        );
        assert_eq!(
            encode_trusted(&bytes),
            Err(FrameError::InvalidTrustedSize(size))
        );
    }
    assert_eq!(encode_trusted(&[]), Err(FrameError::InvalidTrustedSize(0)));
    for packet in [vec![2, 0, 0], vec![4, 0, 0]] {
        assert_eq!(
            encode_trusted(&packet),
            Err(FrameError::PacketLengthMismatch {
                declared: packet[0] as usize,
                actual: 3
            })
        );
    }
}

#[test]
fn session_round_trip_padding_codes_and_stream_boundaries() {
    let crypt = SessionCrypt::with_keys([0; 16], Some([1; 16]));
    for len in [2usize, 4, 15, 16, 17, 4000] {
        let mut packet = vec![0x42; len];
        packet[..2].copy_from_slice(&(len as i16).to_le_bytes());
        for code in [8, 9] {
            let encoded = encode_client(
                &packet,
                ClientEncrypt::Session {
                    crypt: &crypt,
                    code,
                },
                &mut FixedRng(0),
            )
            .unwrap();
            assert_eq!(encoded.len(), len.div_ceil(16) * 16 + 2);
            assert_eq!(encoded[1] >> 4, code);
            for cut in 0..encoded.len() {
                assert_eq!(session_decode(&encoded[..cut], &crypt).unwrap(), None);
            }
            let combined = encoded.repeat(2);
            let decoded = session_decode(&combined, &crypt).unwrap().unwrap();
            assert_eq!(decoded.packet, packet);
            assert_eq!(decoded.consumed, encoded.len());
            let mut third_party = encoded.clone();
            third_party[1] += 0x20;
            assert_eq!(
                session_decode(&third_party, &crypt)
                    .unwrap()
                    .unwrap()
                    .packet,
                packet
            );
        }
    }
}

#[test]
fn client_plain_mode_and_code_errors_in_receive_order() {
    let (public, _, _) = rsa_keys(512);
    let crypt = SessionCrypt::new([0; 16]);
    let handshake = || ClientDecrypt::Handshake {
        key: &public,
        expected_code: 4,
    };
    let plain = [2, 0, 2, 0];
    assert_eq!(decode_client(&plain[..3], handshake()).unwrap(), None);
    assert_eq!(
        decode_client(&plain, handshake()),
        Err(FrameError::PlainNotAccepted)
    );
    for code in [5, 6, 8, 15] {
        assert_eq!(
            decode_client(&wire(code, &[]), handshake()),
            Err(FrameError::InvalidHandshakeCode {
                expected: 4,
                actual: code
            })
        );
    }
    assert_eq!(
        decode_client(
            &wire(8, &[]),
            ClientDecrypt::Session {
                crypt: &crypt,
                mode: 3
            }
        ),
        Err(FrameError::InvalidMode(3))
    );
    assert_eq!(
        decode_client(
            &wire(4, &[]),
            ClientDecrypt::Session {
                crypt: &crypt,
                mode: 3
            }
        ),
        Err(FrameError::InvalidMode(3))
    );
    for code in [0, 1, 2, 3, 4, 5, 6, 7, 12, 13, 14, 15] {
        let bytes = (u16::from(code) << 12).to_le_bytes();
        let mut input = bytes.to_vec();
        input.resize(payload_len(u16::from_le_bytes(bytes)) + 2, 0);
        assert_eq!(
            session_decode(&input, &crypt),
            Err(FrameError::InvalidEncryptionCode(code))
        );
    }
}

#[test]
fn client_inner_size_and_encode_errors() {
    let crypt = SessionCrypt::new([0; 16]);
    for size in [i16::MIN, -1, 0, 1, 17, i16::MAX] {
        let encrypted = crypt.encrypt(&size.to_le_bytes(), 8).unwrap();
        assert_eq!(
            session_decode(&wire(8, &encrypted), &crypt),
            Err(FrameError::InvalidPacketSize {
                size,
                available: 16
            })
        );
    }
    assert_eq!(
        session_decode(&wire(8, &[]), &crypt),
        Err(FrameError::InvalidPacketSize {
            size: 0,
            available: 0
        })
    );
    for packet in [vec![], vec![0], vec![1, 0], vec![4, 0]] {
        assert!(matches!(
            encode_client(
                &packet,
                ClientEncrypt::Session {
                    crypt: &crypt,
                    code: 8
                },
                &mut FixedRng(0)
            ),
            Err(FrameError::InvalidPacketSize { .. })
        ));
    }
    assert_eq!(
        encode_client(
            &[2, 0, 0],
            ClientEncrypt::Session {
                crypt: &crypt,
                code: 8
            },
            &mut FixedRng(0)
        ),
        Err(FrameError::PacketLengthMismatch {
            declared: 2,
            actual: 3
        })
    );
}

#[test]
fn client_crypto_errors() {
    let (public, private, _) = rsa_keys(512);
    let crypt = SessionCrypt::new([0; 16]);
    assert_eq!(
        session_decode(&wire(8, &[0; 4]), &crypt),
        Err(FrameError::Crypto(CryptoError::IncompleteAesBlock))
    );
    assert_eq!(
        session_decode(&wire(9, &[]), &crypt),
        Err(FrameError::Crypto(CryptoError::MissingSessionKey(1)))
    );
    assert_eq!(
        encode_client(
            &[2, 0],
            ClientEncrypt::Session {
                crypt: &crypt,
                code: 10
            },
            &mut FixedRng(0)
        ),
        Err(FrameError::Crypto(CryptoError::InvalidSessionCode(10)))
    );
    assert_eq!(
        encode_client(
            &[2, 0],
            ClientEncrypt::Handshake {
                key: &public,
                code: 6
            },
            &mut FixedRng(0)
        ),
        Err(FrameError::InvalidHandshakeCode {
            expected: 6,
            actual: 6
        })
    );
    assert_eq!(
        decode_client(
            &wire(4, &[0; 4]),
            ClientDecrypt::Handshake {
                key: &private,
                expected_code: 4
            }
        ),
        Err(FrameError::HandshakeDecrypt(
            CryptoError::IncompleteRsaBlock
        ))
    );
    let mut db = vec![0; 47];
    db[20] = 2;
    let invalid = public.apply_block(&oaep_with_db(0, [0; 16], &db)).unwrap();
    assert_eq!(
        decode_client(
            &wire(4, &invalid),
            ClientDecrypt::Handshake {
                key: &private,
                expected_code: 4
            }
        ),
        Err(FrameError::HandshakeDecrypt(
            CryptoError::InvalidOaepDelimiter
        ))
    );
}

#[test]
fn handshake_frames_in_all_rsa_directions() {
    let (public, private, _) = rsa_keys(1024);
    let (client_public, client_private, _) = rsa_keys(512);
    let mut packet = vec![0x42; 205];
    packet[..2].copy_from_slice(&205i16.to_le_bytes());
    for (sender, receiver, code) in [
        (&public, &private, 4),
        (&private, &public, 4),
        (&client_public, &client_private, 5),
    ] {
        let encoded = encode_client(
            &packet,
            ClientEncrypt::Handshake { key: sender, code },
            &mut FixedRng(0),
        )
        .unwrap();
        for cut in 0..encoded.len() {
            assert_eq!(
                decode_client(
                    &encoded[..cut],
                    ClientDecrypt::Handshake {
                        key: receiver,
                        expected_code: code
                    }
                )
                .unwrap(),
                None
            );
        }
        let decoded = decode_client(
            &encoded,
            ClientDecrypt::Handshake {
                key: receiver,
                expected_code: code,
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(decoded.packet, packet);
        assert_eq!(decoded.consumed, encoded.len());
    }
}
