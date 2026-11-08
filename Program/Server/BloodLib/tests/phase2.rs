use bloodlib::{dispatch::*, fragments::*, packet::*};
use std::borrow::Cow;

fn packet(cmd: i16, body: &[u8]) -> Vec<u8> {
    let mut out = ((body.len() + 4) as i16).to_le_bytes().to_vec();
    out.extend(cmd.to_le_bytes());
    out.extend(body);
    out
}
fn fragment(cmd: i16, mark: i32, data: &[u8]) -> Vec<u8> {
    let mut body = mark.to_le_bytes().to_vec();
    body.extend(data);
    packet(cmd, &body)
}
fn wire_fields(format: &str, values: &[Value], expected: &[u8]) {
    let mut full = ((expected.len() + 2) as i16).to_le_bytes().to_vec();
    full.extend(expected);
    assert_eq!(compose(format, values).unwrap(), full, "{format}");
    let decoded = decompose(format, &full, DecodeLimits::default()).unwrap();
    assert_eq!(compose(format, &decoded).unwrap(), full, "{format}");
}

#[test]
fn byte_exact_all_seventeen_codes() {
    wire_fields("c", &[Value::U8(254)], &[254]);
    wire_fields("Q", &[Value::I8(-2)], &[254]);
    for code in ["h", "u", "E", "U"] {
        wire_fields(code, &[Value::U16(0x8123)], &[0x23, 0x81]);
    }
    wire_fields("d", &[Value::I32(-2)], &[254, 255, 255, 255]);
    wire_fields("i", &[Value::U32(0x81234567)], &[0x67, 0x45, 0x23, 0x81]);
    wire_fields("f", &[Value::F64(1.5)], &[0, 0, 0xc0, 0x3f]);
    wire_fields(
        "q",
        &[Value::I64(-2)],
        &[254, 255, 255, 255, 255, 255, 255, 255],
    );
    wire_fields("o", &[Value::Id([0xa5; 16])], &[0xa5; 16]);
    for code in ["b", "K"] {
        wire_fields(code, &[Value::Bytes(vec![0, 255, 3])], &[3, 0, 0, 255, 3]);
    }
    wire_fields(
        "s",
        &[Value::String(vec![0x41, 0xd800])],
        &[2, 0, 0x41, 0, 0, 0xd8],
    );
    wire_fields(
        "P",
        &[Value::Property {
            type_id: 7,
            count: 2,
            data: vec![9, 8, 0],
        }],
        &[7, 2, 0, 3, 0, 9, 8, 0],
    );
    wire_fields(
        "Cc;",
        &[Value::Repeat(vec![vec![Value::U8(7)], vec![Value::U8(8)]])],
        &[2, 0, 7, 8],
    );
}
#[test]
fn float_narrows_and_integer_aliases_keep_bits() {
    assert_eq!(
        &compose("f", &[Value::F64(1.0 + f64::EPSILON)]).unwrap()[2..],
        &1f32.to_le_bytes()
    );
    assert_eq!(
        decompose("h", &[4, 0, 255, 255], DecodeLimits::default()).unwrap(),
        vec![Value::I16(-1)]
    );
    assert_eq!(
        decompose("u", &[4, 0, 255, 255], DecodeLimits::default()).unwrap(),
        vec![Value::U16(65535)]
    );
}
#[test]
fn repeat_depth_empty_skip_and_unsigned_read_signed_write() {
    wire_fields(
        "CCCc;;;",
        &[Value::Repeat(vec![vec![Value::Repeat(vec![vec![
            Value::Repeat(vec![vec![Value::U8(9)]]),
        ]])]])],
        &[1, 0, 1, 0, 1, 0, 9],
    );
    wire_fields("CCs;;c", &[Value::Repeat(vec![]), Value::U8(9)], &[0, 0, 9]);
    wire_fields("C;", &[Value::Repeat(vec![vec![]; 65535])], &[255, 255]);
    for format in ["CCCCc;;;;", "C", ";", "C;;", "x"] {
        assert!(compose(format, &[]).is_err());
        assert!(decode_fields(format, &[], DecodeLimits::default()).is_err());
    }
    assert_eq!(compose("CCCC;;;;", &[]), Err(PacketError::RepeatDepth));
    assert_eq!(
        compose("C;", &[Value::Repeat(vec![vec![]; 65536])]),
        Err(PacketError::RepeatCount)
    );
}
fn example(cmd: i16, format: &str, values: &[Value], expected: &[u8]) {
    let output = compose_fragments(cmd, format, values).unwrap();
    assert_eq!(output, vec![fragment(cmd, expected.len() as i32, expected)]);
    let joined = FragmentReader::default().add(&output[0]).unwrap().unwrap();
    assert!(matches!(joined.data, Cow::Borrowed(_)));
    let decoded = decode_fields(&format[2..], &joined.data, DecodeLimits::default()).unwrap();
    assert_eq!(compose_fragments(cmd, format, &decoded).unwrap(), output);
}
#[test]
fn section6_character_list_example() {
    let row = vec![
        Value::U64(2),
        Value::Id([3; 16]),
        Value::I16(-4),
        Value::String(vec![65]),
        Value::Bytes(vec![6]),
        Value::U64(7),
        Value::I8(-8),
        Value::I8(9),
        Value::U32(10),
    ];
    let values = vec![Value::U64(1), Value::I16(-2), Value::Repeat(vec![row])];
    let mut expected = vec![
        1, 0, 0, 0, 0, 0, 0, 0, 254, 255, 1, 0, 2, 0, 0, 0, 0, 0, 0, 0,
    ];
    expected.extend([3; 16]);
    expected.extend([
        252, 255, 1, 0, 65, 0, 1, 0, 6, 7, 0, 0, 0, 0, 0, 0, 0, 248, 9, 10, 0, 0, 0,
    ]);
    example(28, "hiqhCqohsbqQQi;", &values, &expected);
}
#[test]
fn section6_nested_example() {
    let row = vec![
        Value::U64(1),
        Value::U32(2),
        Value::I8(3),
        Value::I8(4),
        Value::U64(5),
        Value::U64(6),
        Value::String(vec![]),
        Value::String(vec![]),
        Value::U32(7),
        Value::String(vec![]),
        Value::String(vec![]),
        Value::Bytes(vec![]),
        Value::Bytes(vec![]),
        Value::Repeat(vec![vec![
            Value::U32(8),
            Value::U32(9),
            Value::U32(10),
            Value::Bytes(vec![11]),
            Value::Bytes(vec![12]),
        ]]),
    ];
    let mut expected = vec![1, 0];
    expected.extend(1u64.to_le_bytes());
    expected.extend(2u32.to_le_bytes());
    expected.extend([3, 4]);
    expected.extend(5u64.to_le_bytes());
    expected.extend(6u64.to_le_bytes());
    expected.extend([0; 4]);
    expected.extend(7u32.to_le_bytes());
    expected.extend([0; 8]);
    expected.extend([1, 0]);
    expected.extend(8u32.to_le_bytes());
    expected.extend(9u32.to_le_bytes());
    expected.extend(10u32.to_le_bytes());
    expected.extend([1, 0, 11, 1, 0, 12]);
    example(
        1,
        "hiCqiQQqqssissbbCiiibb;;",
        &[Value::Repeat(vec![row])],
        &expected,
    );
}
#[test]
fn section6_sequential_groups_example() {
    let values = vec![
        Value::U64(1),
        Value::Repeat(vec![vec![
            Value::U32(2),
            Value::U32(3),
            Value::U32(4),
            Value::U32(5),
            Value::U32(6),
        ]]),
        Value::Repeat(vec![vec![
            Value::U32(7),
            Value::U32(8),
            Value::I8(-9),
            Value::U32(10),
            Value::U64(11),
            Value::U64(12),
        ]]),
        Value::U32(13),
        Value::U64(14),
        Value::U32(15),
        Value::U64(16),
    ];
    let mut expected = 1u64.to_le_bytes().to_vec();
    expected.extend([1, 0]);
    for n in 2u32..=6 {
        expected.extend(n.to_le_bytes());
    }
    expected.extend([1, 0]);
    expected.extend(7u32.to_le_bytes());
    expected.extend(8u32.to_le_bytes());
    expected.push(247);
    expected.extend(10u32.to_le_bytes());
    expected.extend(11u64.to_le_bytes());
    expected.extend(12u64.to_le_bytes());
    expected.extend(13u32.to_le_bytes());
    expected.extend(14u64.to_le_bytes());
    expected.extend(15u32.to_le_bytes());
    expected.extend(16u64.to_le_bytes());
    example(1, "hiqCiiiii;CiiQiqq;iqiq", &values, &expected);
}
#[test]
fn compose_limits_and_typed_errors() {
    assert_eq!(
        compose("b", &[Value::Bytes(vec![1; 3996])]).unwrap().len(),
        4000
    );
    assert_eq!(
        compose("b", &[Value::Bytes(vec![1; 3997])]),
        Err(PacketError::ComposeOverflow)
    );
    assert_eq!(
        compose(
            "bb",
            &[Value::Bytes(vec![1; 2000]), Value::Bytes(vec![1; 2000])]
        ),
        Err(PacketError::ComposeOverflow)
    );
    assert_eq!(
        compose("b", &[Value::Bytes(vec![1; 32768])]),
        Err(PacketError::LengthRange)
    );
    assert_eq!(
        compose("s", &[Value::String(vec![1; 2000])]),
        Err(PacketError::ComposeOverflow)
    );
    assert_eq!(
        compose(
            "P",
            &[Value::Property {
                type_id: 1,
                count: 0,
                data: vec![0; 3994]
            }]
        ),
        Err(PacketError::ComposeOverflow)
    );
    assert_eq!(compose("c", &[]), Err(PacketError::ValueCount));
    assert_eq!(
        compose("c", &[Value::I16(1)]),
        Err(PacketError::ValueType('c'))
    );
    assert_eq!(
        compose("C;", &[Value::U8(0)]),
        Err(PacketError::ValueType('C'))
    );
    assert_eq!(compose("!", &[]), Err(PacketError::UnknownCode('!')));
    assert_eq!(
        compose(
            "P",
            &[Value::Property {
                type_id: 0,
                count: 0,
                data: vec![]
            }]
        ),
        Err(PacketError::PropertyType)
    );
    assert_eq!(
        compose(
            "P",
            &[Value::Property {
                type_id: 1,
                count: -1,
                data: vec![]
            }]
        ),
        Err(PacketError::PropertyCount)
    );
}
#[test]
fn every_decompose_check() {
    let defaults = DecodeLimits::default();
    for (code, valid) in [
        ("c", vec![1]),
        ("Q", vec![1]),
        ("h", vec![1, 0]),
        ("u", vec![1, 0]),
        ("E", vec![1, 0]),
        ("U", vec![1, 0]),
        ("d", vec![1, 0, 0, 0]),
        ("i", vec![1, 0, 0, 0]),
        ("f", vec![0; 4]),
        ("q", vec![0; 8]),
        ("o", vec![0; 16]),
        ("b", vec![1, 0, 7]),
        ("K", vec![1, 0, 7]),
        ("s", vec![1, 0, 65, 0]),
        ("P", vec![1, 0, 0, 1, 0, 7]),
        ("Cc;", vec![1, 0, 7]),
    ] {
        for cut in 0..valid.len() {
            assert!(
                decode_fields(code, &valid[..cut], defaults).is_err(),
                "{code} at {cut}"
            );
        }
    }
    for code in ["b", "K"] {
        assert_eq!(
            decode_fields(code, &[255, 255], defaults),
            Err(PacketError::BlobLengthUnderflow)
        );
        assert_eq!(
            decode_fields(
                code,
                &[1, 0, 7],
                DecodeLimits {
                    blob_bytes: 0,
                    ..defaults
                }
            ),
            Err(PacketError::BlobDestination)
        );
        assert_eq!(
            decode_fields(code, &[2, 0, 7], defaults),
            Err(PacketError::Bounds {
                code: code.chars().next().unwrap()
            })
        );
    }
    assert_eq!(
        decode_fields("s", &[255, 255], defaults),
        Err(PacketError::StringLengthUnderflow)
    );
    for capacity in [0, 1] {
        assert_eq!(
            decode_fields(
                "s",
                &[1, 0, 65, 0],
                DecodeLimits {
                    string_units: capacity,
                    ..defaults
                }
            ),
            Err(PacketError::StringDestination)
        );
    }
    assert!(decode_fields(
        "s",
        &[1, 0, 65, 0],
        DecodeLimits {
            string_units: 2,
            ..defaults
        }
    )
    .is_ok());
    assert_eq!(
        decode_fields("s", &[1, 0, 65], defaults),
        Err(PacketError::StringLengthOverflow)
    );
    assert_eq!(
        decode_fields("P", &[0], defaults),
        Err(PacketError::PropertyType)
    );
    assert_eq!(
        decode_fields("P", &[1, 255, 255], defaults),
        Err(PacketError::PropertyCount)
    );
    assert_eq!(
        decode_fields("P", &[1, 0, 0, 255, 255], defaults),
        Err(PacketError::PropertyLength)
    );
    assert_eq!(
        decode_fields(
            "P",
            &[1, 0, 0, 1, 0, 7],
            DecodeLimits {
                property_bytes: 0,
                ..defaults
            }
        ),
        Err(PacketError::PropertyDestination)
    );
    assert_eq!(
        decode_fields("P", &[1, 0, 0, 1, 0], defaults),
        Err(PacketError::Bounds { code: 'P' })
    );
    assert_eq!(
        decompose("", &[1, 0], defaults),
        Err(PacketError::InvalidSize)
    );
    assert_eq!(
        decompose("", &[3, 0], defaults),
        Err(PacketError::PacketLength)
    );
    assert_eq!(
        decompose("", &[0xa1, 0x0f], defaults),
        Err(PacketError::InvalidSize)
    );
    assert_eq!(decompose("", &[], defaults), Err(PacketError::InvalidSize));
}
#[test]
fn field_aligned_fragments_counts_marks_and_round_trip() {
    let values = [Value::Repeat(vec![
        vec![Value::Bytes(vec![1; 3990])],
        vec![Value::Bytes(vec![2; 3990])],
    ])];
    let output = compose_fragments(28, "hiCb;", &values).unwrap();
    assert_eq!(output.len(), 3);
    assert_eq!(output[0], fragment(28, 7986, &[2, 0]));
    for (index, mark) in [(1, -1i32), (2, 0)] {
        assert_eq!(output[index].len(), 4000);
        assert_eq!(&output[index][4..8], &mark.to_le_bytes());
        assert_eq!(&output[index][8..10], &3990i16.to_le_bytes());
        assert!(output[index][10..].iter().all(|b| *b == index as u8));
    }
    let mut reader = FragmentReader::default();
    assert_eq!(reader.add(&output[0]).unwrap(), None);
    assert_eq!(reader.add(&output[1]).unwrap(), None);
    let joined = reader.add(&output[2]).unwrap().unwrap();
    assert_eq!(joined.cmd, 28);
    assert!(matches!(joined.data, Cow::Owned(_)));
    assert_eq!(
        decode_fields("Cb;", &joined.data, DecodeLimits::default()).unwrap(),
        values
    );
    assert_eq!(
        reader.add(&fragment(7, 1, &[8])).unwrap(),
        Some(Joined {
            cmd: 7,
            data: vec![8].into()
        })
    );
    assert_eq!(
        compose_fragments(1, "hib", &[Value::Bytes(vec![0; 3991])]),
        Err(PacketError::ComposeOverflow)
    );
    assert_eq!(
        compose_fragments(1, "h", &[]),
        Err(PacketError::FragmentFormat)
    );
}
#[test]
fn count_moves_to_new_fragment_and_nested_counts_survive() {
    let values = [
        Value::Bytes(vec![1; 3989]),
        Value::Repeat(vec![vec![Value::Repeat(vec![
            vec![Value::U8(2)],
            vec![Value::U8(3)],
        ])]]),
    ];
    let fragments = compose_fragments(1, "hibCCc;;", &values).unwrap();
    assert_eq!(fragments.len(), 2);
    assert_eq!(fragments[0].len(), 3999);
    assert_eq!(&fragments[1][8..], &[1, 0, 2, 0, 2, 3]);
    let mut reader = FragmentReader::default();
    assert_eq!(reader.add(&fragments[0]).unwrap(), None);
    let joined = reader.add(&fragments[1]).unwrap().unwrap();
    assert_eq!(
        decode_fields("bCCc;;", &joined.data, DecodeLimits::default()).unwrap(),
        values
    );
}
#[test]
fn fragment_reader_every_table_row_and_failure() {
    let mut reader = FragmentReader::default();
    assert_eq!(
        reader.add(&fragment(1, -2, &[])),
        Err(FragmentError::NegativeTotal(-2))
    );
    assert_eq!(
        reader.add(&fragment(1, -1, &[1])),
        Err(FragmentError::EmptyContinuation)
    );
    assert_eq!(
        reader.add(&fragment(1, 1, &[1, 2])),
        Err(FragmentError::ExceedsTotal)
    );
    assert_eq!(reader.add(&fragment(1, 4, &[1])).unwrap(), None);
    assert_eq!(
        reader.add(&fragment(2, -1, &[2])),
        Err(FragmentError::CommandMismatch)
    );
    assert_eq!(
        reader.add(&fragment(2, 1, &[2])),
        Err(FragmentError::CommandMismatch)
    );
    assert_eq!(reader.add(&fragment(1, -1, &[2])).unwrap(), None);
    assert_eq!(
        reader.add(&fragment(1, -1, &[3, 4, 5])),
        Err(FragmentError::ExceedsTotal)
    );
    assert_eq!(
        reader.add(&fragment(1, 0, &[3])),
        Err(FragmentError::IncompleteTotal {
            actual: 3,
            total: 4
        })
    );
    assert_eq!(
        reader.add(&fragment(1, 0, &[3, 4])).unwrap(),
        Some(Joined {
            cmd: 1,
            data: vec![1, 2, 3, 4].into()
        })
    );
    assert_eq!(
        reader.add(&fragment(1, 1, &[9])).unwrap(),
        Some(Joined {
            cmd: 1,
            data: vec![9].into()
        })
    );
    assert_eq!(reader.add(&fragment(1, 3, &[1])).unwrap(), None);
    assert_eq!(reader.add(&fragment(1, 4, &[2])).unwrap(), None);
    assert_eq!(
        reader
            .add(&fragment(1, 0, &[3, 4]))
            .unwrap()
            .unwrap()
            .data
            .as_ref(),
        &[1, 2, 3, 4]
    );
    assert_eq!(reader.add(&fragment(1, 3, &[])).unwrap(), None);
    assert_eq!(
        reader.add(&fragment(1, -1, &[1])),
        Err(FragmentError::EmptyContinuation)
    );
    reader.clear();
    assert_eq!(
        reader.add(&fragment(1, 0, &[1])),
        Err(FragmentError::ExceedsTotal)
    );
    assert!(matches!(
        reader.add(&packet(1, &[])),
        Err(FragmentError::Packet(PacketError::InvalidSize))
    ));
    let mut malformed = fragment(1, 1, &[1]);
    malformed[0] = 8;
    assert!(matches!(
        reader.add(&malformed),
        Err(FragmentError::Packet(PacketError::PacketLength))
    ));
}
#[test]
fn empty_fragment_round_trip() {
    let output = compose_fragments(28, "hi", &[]).unwrap();
    assert_eq!(output, vec![fragment(28, 0, &[])]);
    assert_eq!(
        FragmentReader::default().add(&output[0]).unwrap(),
        Some(Joined {
            cmd: 28,
            data: vec![].into()
        })
    );
}
#[test]
fn dispatch_all_data_and_quirks() {
    let tables = Tables::parse(DEFAULT_TABLES).unwrap();
    assert_eq!(tables.build, "0.0.381.572");
    assert_eq!(tables.version, 58);
    assert_eq!(tables.any_authed.len(), 2);
    assert_eq!(tables.not_authed.len(), 3);
    assert_eq!(tables.authed.len(), 30);
    assert_eq!(tables.bypass.len(), 54);
    assert_eq!(tables.c2d.len(), 28);
    // 04:120..127 lists 52 pairs, not the 51 claimed at 04:129.
    assert_eq!(tables.d2c.len(), 52);
    assert_ne!(tables.d2c.len(), 51);
    assert_eq!(tables.lobbyd.len(), 21);
    assert_eq!(
        tables
            .lobbyd
            .iter()
            .filter(|(_, e)| e.fragment)
            .map(|(c, _)| *c)
            .collect::<Vec<_>>(),
        vec![2, 4, 10, 16, 36, 68, 71, 74, 80]
    );
    for cmd in &tables.bypass {
        assert_eq!(tables.lobbyd_route(*cmd), Route::Relay);
    }
    for (cmd, entry) in &tables.lobbyd {
        assert_eq!(
            tables.lobbyd_route(*cmd),
            Route::Handler {
                name: &entry.handler,
                fragment: entry.fragment
            }
        );
    }
    for authed in [false, true] {
        assert_eq!(
            tables.client_route(9, authed),
            Route::Handler {
                name: "PingReq",
                fragment: false
            }
        );
        assert_eq!(
            tables.client_route(106, authed),
            Route::Handler {
                name: "ContentsRuleListReq",
                fragment: false
            }
        );
        for cmd in [-1, 111, 256] {
            assert_eq!(tables.client_route(cmd, authed), Route::Reject);
        }
    }
    for (cmd, name) in &tables.not_authed {
        assert_eq!(
            tables.client_route(*cmd, false),
            Route::Handler {
                name,
                fragment: false
            }
        );
    }
    for (cmd, name) in &tables.authed {
        assert_eq!(
            tables.client_route(*cmd, true),
            if name == "relay" {
                Route::Relay
            } else {
                Route::Handler {
                    name,
                    fragment: false,
                }
            }
        );
    }
    assert_eq!(tables.client_route(33, true), Route::Relay);
    assert_eq!(tables.map_c2d(33), 0);
    assert_eq!(tables.client_route(41, true), Route::Reject);
    assert_eq!(tables.map_c2d(41), 41);
    assert_eq!(tables.map_d2c(42), 0);
    assert_eq!(tables.map_d2c(43), 0);
    assert_eq!(tables.lobbyd_route(1), Route::Reject);
    assert_eq!(tables.client_route(24, false), Route::Reject);
    for [src, dst] in &tables.c2d {
        assert_eq!(tables.map_c2d(i16::from(*src) + 256), i16::from(*dst));
    }
    for [src, dst] in &tables.d2c {
        assert_eq!(tables.map_d2c(i16::from(*src) + 256), i16::from(*dst));
    }
    let mut fallback = Tables::parse(DEFAULT_TABLES).unwrap();
    fallback.any_authed.remove(&9);
    assert_eq!(fallback.client_route(9, true), Route::Ping);
}
#[test]
fn relay_byte_exact_both_directions_and_limits() {
    let tables = Tables::parse(DEFAULT_TABLES).unwrap();
    let key = 0x0807060504030201;
    let body = [0, 255, 7, 0, 128];
    let mut expected = vec![17, 0, 23, 0, 1, 2, 3, 4, 5, 6, 7, 8];
    expected.extend(body);
    assert_eq!(
        tables
            .relay_to_lobbyd(&packet(12, &body), key, true)
            .unwrap(),
        expected
    );
    for cmd in [33, 12, 41, 268] {
        let out = tables
            .relay_to_lobbyd(&packet(cmd, &body), key, true)
            .unwrap();
        assert_eq!(&out[2..4], &tables.map_c2d(cmd).to_le_bytes());
        assert_eq!(&out[12..], &body);
    }
    for cmd in [26, 42, 43, 282] {
        let mut input = key.to_le_bytes().to_vec();
        input.extend(body);
        let (actual_key, out) = tables.relay_to_client(&packet(cmd, &input)).unwrap();
        assert_eq!(actual_key, key);
        assert_eq!(out, packet(tables.map_d2c(cmd), &body));
    }
    assert_eq!(
        tables.relay_to_lobbyd(&packet(12, &[]), 0, true),
        Err(RelayError::InvalidClientKey)
    );
    assert_eq!(
        tables.relay_to_lobbyd(&packet(12, &[]), key, false),
        Err(RelayError::NotConnected)
    );
    assert!(tables
        .relay_to_lobbyd(&packet(12, &vec![0; 3987]), key, true)
        .is_ok());
    for len in [3988, 3996] {
        assert_eq!(
            tables.relay_to_lobbyd(&packet(12, &vec![0; len]), key, true),
            Err(RelayError::OversizedBody)
        );
    }
    let mut largest = key.to_le_bytes().to_vec();
    largest.extend(vec![5; 3988]);
    assert_eq!(
        tables.relay_to_client(&packet(26, &largest)).unwrap().1,
        packet(15, &vec![5; 3988])
    );
    assert!(tables.relay_to_client(&packet(26, &[0; 7])).is_err());
    assert!(tables.relay_to_lobbyd(&[], key, true).is_err());
}
#[test]
fn each_fragment_is_an_independent_client_frame() {
    use bloodlib::{crypto::SessionCrypt, frames::*};
    use rand::SeedableRng;
    let crypt = SessionCrypt::new([7; 16]);
    let mut rng = rand::rngs::StdRng::seed_from_u64(1);
    let values = [Value::Repeat(vec![
        vec![Value::Bytes(vec![1; 3000])],
        vec![Value::Bytes(vec![2; 3000])],
    ])];
    let fragments = compose_fragments(28, "hiCb;", &values).unwrap();
    assert_eq!(fragments.len(), 2);
    let mut reader = FragmentReader::default();
    for (index, fragment) in fragments.iter().enumerate() {
        let frame = encode_client(
            fragment,
            ClientEncrypt::Session {
                crypt: &crypt,
                code: 8,
            },
            &mut rng,
        )
        .unwrap();
        let decoded = decode_client(
            &frame,
            ClientDecrypt::Session {
                crypt: &crypt,
                mode: 5,
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(decoded.consumed, frame.len());
        assert_eq!(&decoded.packet, fragment);
        let joined = reader.add(&decoded.packet).unwrap();
        if index == 0 {
            assert!(joined.is_none());
        } else {
            assert_eq!(
                decode_fields("Cb;", &joined.unwrap().data, DecodeLimits::default()).unwrap(),
                values
            );
        }
    }
}

#[test]
fn tables_startup_once_and_replacement_data() {
    assert!(matches!(tables(), Err(TableError::NotInitialized)));
    let replacement = DEFAULT_TABLES
        .replace("0.0.381.572", "test-build")
        .replace("\"version\": 58", "\"version\": 59");
    let initialized = initialize_tables(&replacement).unwrap();
    assert_eq!(initialized.version, 59);
    assert_eq!(initialized.build, "test-build");
    assert!(std::ptr::eq(initialized, tables().unwrap()));
    assert!(matches!(
        initialize_tables(DEFAULT_TABLES),
        Err(TableError::AlreadyInitialized)
    ));
    assert!(Tables::parse("{}").is_err());
    assert!(matches!(
        Tables::parse(&DEFAULT_TABLES.replace("[12,23]", "[13,23]")),
        Err(TableError::InvalidEntries)
    ));
}
