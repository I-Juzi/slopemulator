use bloodlib::packet::{self as blood_packet, DecodeLimits, Value};
use packet::game_session::{LobbyConsumeToken, LobbyTokenAuthOk};

#[test]
fn token_ok_has_exact_verified_field_order_and_widths() {
    let value = LobbyTokenAuthOk {
        charge_apply_type: 255,
        remaining_play_time: -2,
        charge_base_type: 128,
        charge_item_id: i32::MIN,
        reservations: 65535,
    };
    let encoded = value.encode().unwrap();
    assert_eq!(
        encoded,
        vec![16, 0, 24, 0, 255, 254, 255, 255, 255, 128, 0, 0, 0, 128, 255, 255]
    );
    assert_eq!(LobbyTokenAuthOk::decode(&encoded).unwrap(), value);
}
#[test]
fn token_ok_roundtrips_generated_signed_and_raw_byte_values() {
    for seed in 0..=255u8 {
        let value = LobbyTokenAuthOk {
            charge_apply_type: seed,
            remaining_play_time: (i32::from(seed) - 128) * 1_000_000,
            charge_base_type: !seed,
            charge_item_id: i32::MIN + i32::from(seed),
            reservations: u16::from(seed) * 257,
        };
        assert_eq!(
            LobbyTokenAuthOk::decode(&value.encode().unwrap()).unwrap(),
            value
        );
    }
}
#[test]
fn token_ok_structural_decoder_checks_command_size_and_truncation() {
    let value = LobbyTokenAuthOk {
        charge_apply_type: 0,
        remaining_play_time: 0,
        charge_base_type: 0,
        charge_item_id: 0,
        reservations: 0,
    };
    let encoded = value.encode().unwrap();
    for len in 0..encoded.len() {
        assert!(LobbyTokenAuthOk::decode(&encoded[..len]).is_err());
    }
    let mut wrong = encoded.clone();
    wrong[2] = 25;
    assert!(LobbyTokenAuthOk::decode(&wrong).is_err());
    let mut wrong = encoded;
    wrong[0] = 15;
    assert!(LobbyTokenAuthOk::decode(&wrong).is_err());
}
#[test]
fn lobby_consume_request_has_no_routing_world_in_body() {
    let encoded = LobbyConsumeToken {
        transaction_id: 1,
        client_key: u64::MAX,
        lobby_token: 2,
        accepted_address_bits: 0x80000000,
    }
    .encode()
    .unwrap();
    assert_eq!(encoded.len(), 32);
    assert_eq!(
        blood_packet::decompose("hqqqi", &encoded, DecodeLimits::default()).unwrap(),
        vec![
            Value::I16(5),
            Value::U64(1),
            Value::U64(u64::MAX),
            Value::U64(2),
            Value::U32(0x80000000),
        ]
    );
}
