//! All frames and identities here are generated. No captured traffic.
use sts::{messages::AuthCommand, Codec, Error, Frame, Header, StartLine, Transaction};

fn request(body: Vec<u8>) -> Frame {
    Frame {
        start: StartLine::Request {
            protocol: "Sts".into(),
            command: "Ping".into(),
        },
        headers: vec![],
        body,
    }
}
fn round_trip(frame: &Frame) {
    let codec = Codec::default();
    let bytes = codec.encode(frame).unwrap();
    let (decoded, used) = codec.decode(&bytes).unwrap().unwrap();
    assert_eq!(used, bytes.len());
    assert_eq!(&decoded, frame);
}

#[test]
fn empty_ping_has_checked_framing() {
    let codec = Codec::default();
    let frame = request(vec![]);
    assert_eq!(
        codec.encode(&frame).unwrap(),
        b"POST /Sts/Ping STS/1.0\r\nl:0\r\n\r\n"
    );
    round_trip(&frame);
}
#[test]
fn opaque_binary_body_round_trip() {
    round_trip(&request((0..=255).collect()));
}
#[test]
fn xml_utf8_uses_byte_length() {
    let frame = request("<Generated>é水</Generated>".as_bytes().to_vec());
    let bytes = Codec::default().encode(&frame).unwrap();
    let header_end = bytes.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
    let head = std::str::from_utf8(&bytes[..header_end]).unwrap();
    assert!(head.ends_with(&format!("l:{}", frame.body.len())));
    round_trip(&frame);
}
#[test]
fn response_and_unknown_headers_round_trip() {
    let frame = Frame {
        start: StartLine::Response {
            status: 200,
            reason: "Generated success".into(),
        },
        headers: vec![
            Header {
                name: "generated-meta".into(),
                value: "opaque value".into(),
            },
            Transaction {
                id: 7,
                complete: Some(true),
                sequence: 0,
            }
            .to_header()
            .unwrap(),
        ],
        body: b"<Generated/>".to_vec(),
    };
    round_trip(&frame);
}
#[test]
fn every_split_position_is_incomplete_until_complete() {
    let codec = Codec::default();
    let frame = request(b"generated\r\n\r\nbinary\0body".to_vec());
    let bytes = codec.encode(&frame).unwrap();
    for split in 0..bytes.len() {
        assert_eq!(codec.decode(&bytes[..split]), Ok(None), "split {split}");
        let mut joined = bytes[..split].to_vec();
        joined.extend_from_slice(&bytes[split..]);
        assert_eq!(
            codec.decode(&joined).unwrap().unwrap(),
            (frame.clone(), bytes.len())
        );
    }
}
#[test]
fn coalesced_frames_keep_trailing_input() {
    let codec = Codec::default();
    let one = request(vec![3; 17]);
    let two = request(vec![5; 29]);
    let mut bytes = codec.encode(&one).unwrap();
    let first_len = bytes.len();
    bytes.extend(codec.encode(&two).unwrap());
    let (decoded, used) = codec.decode(&bytes).unwrap().unwrap();
    assert_eq!(decoded, one);
    assert_eq!(used, first_len);
    let (decoded, second_used) = codec.decode(&bytes[used..]).unwrap().unwrap();
    assert_eq!(decoded, two);
    assert_eq!(used + second_used, bytes.len());
}
#[test]
fn transaction_forms_round_trip() {
    for id in [1, 7, u32::MAX] {
        for complete in [None, Some(false), Some(true)] {
            for sequence in [0, 1, u32::MAX] {
                let transaction = Transaction {
                    id,
                    complete,
                    sequence,
                };
                if complete.is_none() && sequence != 0 {
                    assert_eq!(transaction.to_header(), Err(Error::InvalidTransaction));
                } else {
                    let header = transaction.to_header().unwrap();
                    assert_eq!(Transaction::parse(&header.value), Ok(transaction));
                }
            }
        }
    }
}
#[test]
fn invalid_transactions_fail() {
    for value in [
        "",
        "0",
        "0R",
        "1x",
        "1r-2",
        "1RR",
        "-1",
        "4294967296",
        "1R4294967296",
        "1 ",
    ] {
        assert!(Transaction::parse(value).is_err(), "{value}");
        let bytes = format!("POST /Sts/Ping STS/1.0\r\ns:{value}\r\nl:0\r\n\r\n");
        assert_eq!(
            Codec::default().decode(bytes.as_bytes()),
            Err(Error::InvalidTransaction)
        );
    }
}
#[test]
fn missing_and_invalid_lengths_fail() {
    let codec = Codec::default();
    assert_eq!(
        codec.decode(b"POST /Sts/Ping STS/1.0\r\n\r\n"),
        Err(Error::MissingLength)
    );
    for length in ["", "-1", "+1", "0x10", "1 ", " 1", "4294967296", "1a"] {
        let bytes = format!("POST /Sts/Ping STS/1.0\r\nl:{length}\r\n\r\n");
        assert_eq!(codec.decode(bytes.as_bytes()), Err(Error::InvalidLength));
    }
}
#[test]
fn duplicate_and_alias_headers_fail() {
    for headers in ["l:0\r\nL:0", "l:0\r\nt:a\r\nT:b"] {
        let bytes = format!("POST /Sts/Ping STS/1.0\r\n{headers}\r\n\r\n");
        assert_eq!(
            Codec::default().decode(bytes.as_bytes()),
            Err(Error::DuplicateHeader)
        );
    }
    assert_eq!(
        Codec::default().decode(b"POST /Sts/Ping STS/1.0\r\nContent-Length:0\r\n\r\n"),
        Err(Error::InvalidLength)
    );
}
#[test]
fn header_names_normalize_but_values_do_not() {
    let frame = Codec::default()
        .decode(b"POST /Sts/Ping STS/1.0\r\nL:0\r\nGenerated:MiXeD\r\n\r\n")
        .unwrap()
        .unwrap()
        .0;
    assert_eq!(
        frame.headers,
        vec![Header {
            name: "generated".into(),
            value: "MiXeD".into()
        }]
    );
    round_trip(&frame);
}
#[test]
fn chunked_encoding_is_not_supported() {
    for name in ["x", "Transfer-Encoding"] {
        let bytes = format!("POST /Sts/Ping STS/1.0\r\nl:0\r\n{name}:chunked\r\n\r\n");
        assert_eq!(
            Codec::default().decode(bytes.as_bytes()),
            Err(Error::UnsupportedEncoding)
        );
    }
}
#[test]
fn malformed_start_lines_fail() {
    for line in [
        "GET /Sts/Ping STS/1.0",
        "POST /Sts/Ping HTTP/1.1",
        "POST /Sts/Ping/Extra STS/1.0",
        "POST /Sts/Ping#7 STS/1.0",
        "POST /Sts/%50ing STS/1.0",
        "POST //Ping STS/1.0",
        "STS/1.0 -1 Bad",
        "STS/1.0 4294967296 Bad",
    ] {
        let bytes = format!("{line}\r\nl:0\r\n\r\n");
        assert_eq!(
            Codec::default().decode(bytes.as_bytes()),
            Err(Error::InvalidStartLine)
        );
    }
}
#[test]
fn malformed_headers_fail() {
    for header in [
        " folded",
        "bad name:value",
        ":value",
        "no-colon",
        "a:one\nother",
        "a:one\0other",
        "a:one\tother",
        "a:é",
    ] {
        let bytes = format!("POST /Sts/Ping STS/1.0\r\n{header}\r\nl:0\r\n\r\n");
        assert_eq!(
            Codec::default().decode(bytes.as_bytes()),
            Err(Error::InvalidHeader)
        );
    }
}
#[test]
fn encoder_rejects_line_injection() {
    let codec = Codec::default();
    let mut frame = request(vec![]);
    frame.headers.push(Header {
        name: "a".into(),
        value: "value\r\nl:0".into(),
    });
    assert_eq!(codec.encode(&frame), Err(Error::InvalidHeader));
    frame.headers.clear();
    frame.start = StartLine::Request {
        protocol: "Sts".into(),
        command: "Ping\r\n".into(),
    };
    assert_eq!(codec.encode(&frame), Err(Error::InvalidStartLine));
    frame.start = StartLine::Response {
        status: 200,
        reason: "OK\r\nl:0".into(),
    };
    assert_eq!(codec.encode(&frame), Err(Error::InvalidStartLine));
}
#[test]
fn encoder_rejects_explicit_length_and_duplicate_headers() {
    let codec = Codec::default();
    let mut frame = request(vec![]);
    frame.headers.push(Header {
        name: "l".into(),
        value: "0".into(),
    });
    assert_eq!(codec.encode(&frame), Err(Error::DuplicateHeader));
    frame.headers = vec![
        Header {
            name: "a".into(),
            value: "1".into(),
        },
        Header {
            name: "A".into(),
            value: "2".into(),
        },
    ];
    assert_eq!(codec.encode(&frame), Err(Error::DuplicateHeader));
}
#[test]
fn header_limits_apply_to_complete_and_incomplete_frames() {
    let frame = request(vec![]);
    let bytes = Codec::default().encode(&frame).unwrap();
    let exact = Codec {
        max_header_bytes: bytes.len(),
        max_body_bytes: 0,
    };
    assert!(exact.decode(&bytes).unwrap().is_some());
    assert_eq!(exact.encode(&frame).unwrap(), bytes);
    let short = Codec {
        max_header_bytes: bytes.len() - 1,
        max_body_bytes: 0,
    };
    assert_eq!(short.decode(&bytes), Err(Error::HeaderTooLarge));
    assert_eq!(short.encode(&frame), Err(Error::HeaderTooLarge));
    assert_eq!(
        short.decode(&vec![b'A'; bytes.len()]),
        Err(Error::HeaderTooLarge)
    );
}
#[test]
fn body_limits_apply_before_waiting_for_the_body() {
    let codec = Codec {
        max_header_bytes: 100,
        max_body_bytes: 2,
    };
    assert_eq!(
        codec.decode(b"POST /Sts/Ping STS/1.0\r\nl:3\r\n\r\n"),
        Err(Error::BodyTooLarge)
    );
    assert_eq!(codec.encode(&request(vec![0; 3])), Err(Error::BodyTooLarge));
    round_trip(&request(vec![0; 2]));
}
#[test]
fn truncated_body_waits() {
    assert_eq!(
        Codec::default().decode(b"POST /Sts/Ping STS/1.0\r\nl:2\r\n\r\nx"),
        Ok(None)
    );
}
#[test]
fn auth_names_round_trip_without_numeric_wire_ids() {
    for name in [
        "ConsumeGameToken",
        "KeyData",
        "LoginCancel",
        "LoginFinish",
        "LoginStart",
        "LoginWebStart",
        "LoginWebFinish",
        "RequestCryptKey",
        "RequestGameToken",
        "RequestSession",
        "RequestToken",
        "LoginTokenStart",
        "TokenKeyData",
    ] {
        assert_eq!(AuthCommand::from_wire_name(name).unwrap().wire_name(), name);
        let frame = Frame {
            start: StartLine::Request {
                protocol: "Auth".into(),
                command: name.into(),
            },
            headers: vec![],
            body: vec![],
        };
        round_trip(&frame);
    }
    assert_eq!(AuthCommand::from_wire_name("GeneratedUnknown"), None);
    assert_eq!(AuthCommand::from_wire_name("3101"), None);
}
