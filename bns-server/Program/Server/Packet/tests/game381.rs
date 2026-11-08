//! Generated fixtures only. No client captures, keys, accounts or game records.
use packet::game::{
    self, Direction, EnterZone, Error, GamePacket, Hello, PrepareZone, RawPacket, TokenAuth,
};
use rand::{rngs::StdRng, Rng, SeedableRng};

fn fixture(command: i16, seed: u64) -> (Direction, GamePacket, Vec<u8>) {
    use Direction::{ClientToServer as C, ServerToClient as S};
    let mut rng = StdRng::seed_from_u64(seed);
    let a: u16 = rng.gen();
    let b: u32 = rng.gen();
    let c: u64 = rng.gen();
    let d: u64 = rng.gen();
    let byte: u8 = rng.gen();
    let blob: Vec<u8> = (0..seed % 31).map(|_| rng.gen()).collect();
    let pos = [a, a.wrapping_add(1), a.wrapping_add(2)];
    let mut body = Vec::new();
    let (direction, packet) = match command {
        11 => {
            let units = vec![a, 0, 0xd800, 0xffff];
            body.extend(a.to_le_bytes());
            body.extend(c.to_le_bytes());
            body.extend(4u16.to_le_bytes());
            for unit in &units {
                body.extend(unit.to_le_bytes());
            }
            body.extend(d.to_le_bytes());
            body.extend((blob.len() as u16).to_le_bytes());
            body.extend(&blob);
            (
                C,
                GamePacket::Hello(Hello {
                    protocol_version: a,
                    property_version: c,
                    publisher: units,
                    capabilities: d,
                    handshake_blob: blob,
                }),
            )
        }
        12 => {
            body.extend((blob.len() as u16).to_le_bytes());
            body.extend(&blob);
            (
                S,
                GamePacket::HelloOk {
                    handshake_blob: blob,
                },
            )
        }
        16 => (C, GamePacket::ClientPing),
        17 | 18 => {
            body.extend(c.to_le_bytes());
            body.extend(d.to_le_bytes());
            if command == 17 {
                (
                    S,
                    GamePacket::ServerPing {
                        sequence: c,
                        timestamp: d,
                    },
                )
            } else {
                (
                    C,
                    GamePacket::PingEcho {
                        sequence: c,
                        timestamp: d,
                    },
                )
            }
        }
        23 => {
            body.extend(a.to_le_bytes());
            body.extend(c.to_le_bytes());
            body.extend(d.to_le_bytes());
            (
                C,
                GamePacket::TokenAuth(TokenAuth {
                    world_id: a,
                    client_key: c,
                    lobby_token: d,
                }),
            )
        }
        31 => {
            body.extend(b.to_le_bytes());
            body.push(byte);
            (
                C,
                GamePacket::EnterWorld {
                    parameter: b,
                    flag: byte,
                },
            )
        }
        41 => {
            body.extend(c.to_le_bytes());
            body.extend(a.to_le_bytes());
            body.extend(b.to_le_bytes());
            body.extend(b.wrapping_add(1).to_le_bytes());
            body.extend([byte, byte.wrapping_add(1)]);
            body.extend((blob.len() as u16).to_le_bytes());
            body.extend(&blob);
            for n in pos {
                body.extend(n.to_le_bytes());
            }
            body.extend(a.wrapping_add(3).to_le_bytes());
            body.extend(a.wrapping_add(4).to_le_bytes());
            body.extend(a.wrapping_add(5).to_le_bytes());
            (
                S,
                GamePacket::PrepareZone(PrepareZone {
                    zone_object: c,
                    instance_bits: a,
                    zone_table_id: b,
                    correlation: b.wrapping_add(1),
                    config_byte: byte,
                    action_flag: byte.wrapping_add(1),
                    zone_block: blob,
                    position_bits: pos,
                    yaw_bits: a.wrapping_add(3),
                    player_transition: a.wrapping_add(4),
                    summoned_transition: a.wrapping_add(5),
                }),
            )
        }
        42 => {
            body.extend(b.to_le_bytes());
            for n in pos {
                body.extend(n.to_le_bytes());
            }
            body.extend(a.wrapping_add(3).to_le_bytes());
            (
                C,
                GamePacket::EnterZone(EnterZone {
                    correlation: b,
                    summoned_position_bits: pos,
                    summoned_yaw_bits: a.wrapping_add(3),
                }),
            )
        }
        43 => {
            body.extend(c.to_le_bytes());
            (S, GamePacket::EnterZoneOk { spawn_context: c })
        }
        _ => unreachable!(),
    };
    let mut expected = ((body.len() + 4) as u16).to_le_bytes().to_vec();
    expected.extend(command.to_le_bytes());
    expected.extend(body);
    (direction, packet, expected)
}

macro_rules! roundtrip {
    ($name:ident, $command:literal) => {
        #[test]
        fn $name() {
            for seed in 0..256 {
                let (direction, packet, expected) = fixture($command, seed);
                assert_eq!(packet.command(), $command);
                assert_eq!(packet.encode(direction).unwrap(), expected);
                assert_eq!(GamePacket::decode(direction, &expected).unwrap(), packet);
            }
        }
    };
}
roundtrip!(hello_11, 11);
roundtrip!(hello_ok_12, 12);
roundtrip!(client_ping_16, 16);
roundtrip!(server_ping_17, 17);
roundtrip!(echo_18, 18);
roundtrip!(token_23, 23);
roundtrip!(enter_world_31, 31);
roundtrip!(prepare_zone_41, 41);
roundtrip!(enter_zone_42, 42);
roundtrip!(enter_zone_ok_43, 43);

#[test]
fn all_truncations_and_trailing_bytes_fail() {
    for command in [11, 12, 16, 17, 18, 23, 31, 41, 42, 43] {
        let (direction, _, wire) = fixture(command, 29);
        for len in 0..wire.len() {
            assert!(GamePacket::decode(direction, &wire[..len]).is_err());
            if len >= 4 {
                let mut short = wire[..len].to_vec();
                short[..2].copy_from_slice(&(len as u16).to_le_bytes());
                assert!(GamePacket::decode(direction, &short).is_err());
            }
        }
        let mut trailing = wire.clone();
        trailing.push(0);
        let len = trailing.len() as u16;
        trailing[..2].copy_from_slice(&len.to_le_bytes());
        assert!(matches!(
            GamePacket::decode(direction, &trailing),
            Err(Error::TrailingBytes)
        ));
        let mut mismatch = wire;
        mismatch[0] ^= 1;
        assert!(GamePacket::decode(direction, &mismatch).is_err());
    }
}

#[test]
fn directions_are_not_lobby_aliases() {
    for command in [11, 12, 16, 17, 18, 23, 31, 41, 42, 43] {
        let (direction, packet, wire) = fixture(command, 0);
        let other = match direction {
            Direction::ClientToServer => Direction::ServerToClient,
            Direction::ServerToClient => Direction::ClientToServer,
        };
        assert!(matches!(packet.encode(other), Err(Error::WrongDirection(c)) if c == command));
        assert!(
            matches!(GamePacket::decode(other, &wire), Err(Error::WrongDirection(c)) if c == command)
        );
    }
}

#[test]
fn unsupported_retains_every_body_byte() {
    for command in [24i16, 32, 929, 1500] {
        let direction = if command == 929 {
            Direction::ClientToServer
        } else {
            Direction::ServerToClient
        };
        let raw = RawPacket {
            command,
            body: (0..=255).collect(),
        };
        let mut wire = ((raw.body.len() + 4) as u16).to_le_bytes().to_vec();
        wire.extend(command.to_le_bytes());
        wire.extend(&raw.body);
        let err = GamePacket::decode(direction, &wire).unwrap_err();
        assert!(err.to_string().contains("unsupported"));
        assert!(matches!(err, Error::Unsupported(ref p) if p == &raw));
        assert!(
            matches!(GamePacket::Raw(raw.clone()).encode(direction), Err(Error::Unsupported(p)) if p == raw)
        );
    }
    // A header-only unknown success must not become a typed empty success.
    for command in [24i16, 32] {
        let mut wire = vec![4, 0];
        wire.extend(command.to_le_bytes());
        assert!(matches!(
            GamePacket::decode(Direction::ServerToClient, &wire),
            Err(Error::Unsupported(_))
        ));
    }
}

#[test]
fn counted_fields_reject_negative_and_short_lengths() {
    use Direction::{ClientToServer as C, ServerToClient as S};
    // Logical offsets: Hello publisher count=14; HelloOk blob count=4;
    // PrepareZone K count=24. These are wire offsets, not native object offsets.
    for (command, direction, offset) in [(11, C, 14), (12, S, 4), (41, S, 24)] {
        let (_, _, wire) = fixture(command, 7);
        for count in [0x8000u16, 0xffff, 0x7fff] {
            let mut broken = wire.clone();
            broken[offset..offset + 2].copy_from_slice(&count.to_le_bytes());
            assert!(GamePacket::decode(direction, &broken).is_err());
        }
    }
}

#[test]
fn maximum_packet_and_count_bounds() {
    let direction = Direction::ServerToClient;
    let exact = GamePacket::HelloOk {
        handshake_blob: vec![0xa5; 3994],
    };
    let wire = exact.encode(direction).unwrap();
    assert_eq!(wire.len(), 4000);
    assert_eq!(GamePacket::decode(direction, &wire).unwrap(), exact);
    for count in [3995, 32768] {
        assert!(GamePacket::HelloOk {
            handshake_blob: vec![0; count]
        }
        .encode(direction)
        .is_err());
    }
    let mut huge = vec![0; 4001];
    huge[..2].copy_from_slice(&4001u16.to_le_bytes());
    huge[2..4].copy_from_slice(&1500i16.to_le_bytes());
    assert!(GamePacket::decode(direction, &huge).is_err());
}

#[test]
fn game_table_is_separate_and_validated() {
    let tables = game::tables();
    assert_eq!(tables.version, 1278);
    let lobby: serde_json::Value =
        serde_json::from_str(include_str!("../tables/381.json")).unwrap();
    assert_eq!(lobby["version"], 58);
    assert_eq!(tables.client.len(), 7);
    assert_eq!(tables.server.len(), 6);
    let init = tables.entry(Direction::ServerToClient, 32).unwrap();
    assert!(init.fragment);
    assert!(init.format.is_none());
    assert_eq!(
        tables
            .entry(Direction::ServerToClient, 43)
            .unwrap()
            .format
            .as_deref(),
        Some("hq")
    );
    assert!(tables.entry(Direction::ClientToServer, 1).is_none());
    let source = include_str!("../tables/game381.json");
    for invalid in [
        source.replace("1278", "58"),
        source.replace("hiuuuh", "hi"),
        source.replace("\"fragment\":true", "\"fragment\":false"),
        source.replace("\"client\":", "\"lobbyd\":"),
        source.replace("\"929\"", "\"930\""),
    ] {
        assert!(game::Tables::parse(&invalid).is_err());
    }
}
