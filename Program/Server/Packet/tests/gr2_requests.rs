use bloodlib::packet::{self as wire, DecodeLimits, Value};
use packet::game_requests::{format, BackendRequest, Body, Error, Request, COMMANDS};

fn fields(command: u16) -> Vec<Value> {
    use Value::{
        Bytes as B, Repeat as R, String as S, I8 as Q, U16 as H, U32 as I, U64 as L, U8 as C,
    };
    match command {
        11 => vec![
            H(1278),
            L(0x12D261D120403),
            S(vec![0xd800]),
            L(0x40000000000),
            B(vec![7, 8]),
        ],
        13 => vec![H(1278), S(vec![]), R(vec![vec![H(2), H(301)]])],
        18 | 414 => vec![L(u64::MAX), L(1)],
        23 => vec![H(1), L(2), L(3)],
        31 => vec![I(u32::MAX), C(9)],
        42 => vec![I(123), H(1), H(0xffff), H(3), H(359)],
        64 => {
            let mut v = vec![C(0)];
            v.extend((0..6).map(H));
            v.extend((0..6).map(I));
            v.extend([H(10), H(359), H(42), C(1)]);
            v.push(R(vec![vec![
                Q(1),
                C(255),
                C(1),
                C(0),
                C(2),
                C(3),
                C(4),
                H(0x8000),
            ]]));
            v.push(R(vec![vec![H(1), H(2), H(3)]]));
            v
        }
        66 => vec![C(0), H(359)],
        102 | 303 | 1239 => vec![H(0xffff)],
        298 => vec![H(1), H(2), H(3), H(4)],
        305 => vec![Q(1), B(vec![1, 2, 3])],
        416 => vec![Q(-1), I(1), I(2), I(3), I(4), I(u32::MAX)],
        809 => vec![L(1), L(2), I(3)],
        981 => vec![Q(1), L(u64::MAX), C(2), R(vec![vec![H(1), H(0xffff)]])],
        984 => vec![],
        1149 => vec![L(1), L(2), L(0xdeadbeef12345678), C(3)],
        1210 => vec![C(1), C(2)],
        1219 => vec![L(u64::MAX)],
        1223 => vec![H(1), H(2)],
        _ => unreachable!(),
    }
}
fn sample(command: u16) -> Request {
    Request {
        command,
        body: if format(command).is_some() {
            Body::Fields(fields(command))
        } else {
            Body::Unused(vec![0xff, 0x80, 0])
        },
    }
}
#[test]
fn all_43_requests_round_trip_and_truncate() {
    for command in COMMANDS {
        let bytes = sample(command).encode().unwrap();
        let decoded = Request::decode(&bytes).unwrap();
        assert_eq!(decoded.encode().unwrap(), bytes, "command {command}");
        // Invalid logical size always fails, even for unused-body commands.
        assert!(Request::decode(&bytes[..3]).is_err(), "command {command}");
        if format(command).is_some_and(|f| f != "h") {
            let malformed = [4, 0, command as u8, (command >> 8) as u8];
            assert!(Request::decode(&malformed).is_err(), "command {command}");
        }
    }
}
#[test]
fn repeat_count_is_two_bytes_and_preserves_raw_shorts() {
    let bytes = sample(13).encode().unwrap();
    assert_eq!(&bytes[8..], &[1, 0, 2, 0, 0x2d, 1]);
    let bytes = sample(981).encode().unwrap();
    assert_eq!(&bytes[14..], &[1, 0, 1, 0, 0xff, 0xff]);
}
#[test]
fn unchecked_decoder_failure_is_structural_only() {
    for command in [23, 42, 66, 102, 298, 303, 809, 1219, 1223, 1239] {
        let input = [4, 0, command as u8, (command >> 8) as u8];
        assert!(matches!(Request::decode(&input), Err(Error::Packet(_))));
        // Codec errors are NOT native handler failure reasons.
    }
}
#[test]
fn prefix_decoders_allow_trailing_bytes() {
    for command in COMMANDS {
        let mut bytes = sample(command).encode().unwrap();
        bytes.extend([0xde, 0xad]);
        let len = bytes.len() as u16;
        bytes[..2].copy_from_slice(&len.to_le_bytes());
        assert!(Request::decode(&bytes).is_ok(), "command {command}");
    }
}
#[test]
fn hello_capacity_and_negative_blob_fail() {
    let mut v = fields(11);
    v[2] = Value::String(vec![0; 40]);
    let req = Request {
        command: 11,
        body: Body::Fields(v),
    };
    assert!(Request::decode(&req.encode().unwrap()).is_err());
    let mut bytes = sample(305).encode().unwrap();
    bytes[5..7].copy_from_slice(&0xffffu16.to_le_bytes());
    assert!(Request::decode(&bytes).is_err());
}
#[test]
fn backend_commands_have_independent_exact_fixtures() {
    let clean = BackendRequest::LobbyClean {
        db_id: 0x0102030405060708,
    }
    .encode()
    .unwrap();
    assert_eq!(clean, vec![14, 0, 17, 0, 8, 7, 6, 5, 4, 3, 2, 1, 0, 1]);
    let returned = BackendRequest::ReturnToGame {
        db_id: 1,
        object_id: 2,
    }
    .encode()
    .unwrap();
    assert_eq!(&returned[..5], &[21, 0, 15, 1, 2]);
    assert_eq!(&returned[5..13], &1u64.to_le_bytes());
    assert_eq!(&returned[13..], &2u64.to_le_bytes());
    let options = BackendRequest::SaveClientOption {
        db_id: 1,
        option_type: 2,
        data: vec![0xaa, 0xbb],
    }
    .encode()
    .unwrap();
    let decoded = wire::decompose("hQCqqchb;", &options, DecodeLimits::default()).unwrap();
    assert_eq!(decoded[1], Value::I8(0));
    let Value::Repeat(rows) = &decoded[2] else {
        panic!("repeat")
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][1], Value::U64(2));
    assert_eq!(rows[0][2], Value::U8(14));
    assert_eq!(rows[0][3], Value::I16(2));
    assert_eq!(rows[0][4], Value::Bytes(vec![2, 0, 2, 0, 0xaa, 0xbb]));
}
#[test]
fn unsupported_and_wrong_body_are_explicit() {
    assert!(matches!(
        Request::decode(&[4, 0, 0xff, 0x7f]),
        Err(Error::Unsupported(32767))
    ));
    assert_eq!(
        Request {
            command: 18,
            body: Body::Unused(vec![])
        }
        .encode(),
        Err(Error::WrongBody)
    );
}
