use packet::{fragments::FragmentReader, world_entry::*};

#[test]
fn all_422_descriptor_kinds_match_the_binary_definition_table() {
    // The spec table comes from the separate native FieldDef array, not from
    // NATIVE_DESCRIPTOR. Compare every ID, including ability and packed types.
    let spec = include_str!("../../../../Docs/Server/GameDaemon/10-session-handlers.md");
    let table = spec
        .split("<!-- gw1-creature-fields-start -->")
        .nth(1)
        .unwrap()
        .split("<!-- gw1-creature-fields-end -->")
        .next()
        .unwrap();
    let mut seen = 0;
    for row in table.lines() {
        let columns: Vec<_> = row.split('|').map(str::trim).collect();
        let Some(id) = columns.get(1).and_then(|s| s.parse::<usize>().ok()) else {
            continue;
        };
        let kind: u8 = columns[2].parse().unwrap();
        seen += 1;
        assert_eq!(id, seen);
        let expected = match kind {
            1 => 'c',
            2 | 7 => 'h',
            3 | 8 => 'i',
            4 | 9 => 'q',
            10 => 'B',
            11 => 'F',
            12 => 's',
            13 | 14 => 'b',
            other => panic!("unexpected native kind {other}"),
        };
        let actual = property_code(id).unwrap();
        assert!(
            actual == expected || (expected == 'q' && actual == '?'),
            "property {id} {}: native kind {kind}, wire {actual}",
            columns[3]
        );
    }
    assert_eq!(seen, 422);
}

fn properties() -> CreatureProperties {
    CreatureProperties::new(std::array::from_fn(|i| {
        match property_code(i + 1).unwrap() {
            'c' => PropertyValue::Byte(0),
            'h' => PropertyValue::Short(0),
            'i' => PropertyValue::Int(0),
            'q' | '?' => PropertyValue::Long(0),
            's' => PropertyValue::String(vec![]),
            'b' => PropertyValue::Bytes(vec![]),
            'B' | 'F' => PropertyValue::Bit(false),
            other => panic!("unexpected property kind {other}"),
        }
    }))
    .unwrap()
}
fn fixture() -> EnterWorldOk {
    EnterWorldOk {
        header: EntryHeader {
            world_id: 7,
            object_id: 0x0807060504030201,
            jumping_character_state: 9,
            chat_address_bits: 0x14131211,
            chat_port: 21,
            chat_world_id: 22,
            manage_port: 23,
            time_zone_bias: -60,
        },
        properties: properties(),
        extra_properties: vec![0xA1, 0xA2],
        opaque_broadcast_block: vec![],
        recycle_remaining: [None; 25],
        acquired_skills: [vec![0xB1], vec![0xB2, 0xB3]],
        config_bytes: [31, 32, 33, 34],
        pc_cafe_publisher: 35,
        pc_db_id: 36,
        content_bans_enabled: true,
        user: None,
        test_server_enabled: true,
        craft_unions: [
            CraftUnion { id: 1, level: -2 },
            CraftUnion { id: 14, level: 3 },
        ],
        slate_pages: std::array::from_fn(|i| SlatePage {
            page: (i + 1) as u8,
            items: [SlateItem {
                item_id: 0x04030201,
                level: 5,
            }; 6],
            stones: [6; 9],
            new_stone_id: 7,
            new_stone_position: 8,
        }),
    }
}
fn joined(entry: &EnterWorldOk) -> Vec<u8> {
    let fragments = entry.encode_fragments().unwrap();
    let mut reader = FragmentReader::default();
    let mut result = None;
    for fragment in &fragments {
        if let Some(packet) = reader.add(fragment).unwrap() {
            assert!(result.is_none());
            assert_eq!(packet.cmd, 32);
            result = Some(packet.data.into_owned());
        }
    }
    result.expect("complete fragment chain")
}
fn suffix(entry: &EnterWorldOk) -> Vec<u8> {
    joined(entry)[25 + entry.properties.encode().len()..].to_vec()
}
fn time() -> TimeBasedPlay {
    TimeBasedPlay {
        schedule_number: 41,
        expire_time_utc: 42,
        last_reset_time_utc: 43,
        duration: 44,
        acknowledged_play_time: 45,
        rewards: [46, 47, 48, 49],
    }
}
#[test]
fn descriptor_and_native_property_anchors() {
    assert_eq!(NATIVE_DESCRIPTOR.len(), 448);
    assert_eq!(&NATIVE_DESCRIPTOR[..11], "hihqQihhhhh");
    assert_eq!(
        &NATIVE_DESCRIPTOR[384..],
        "bKCci;CQb;cccccqCQqq;QCch;CQiqqihcccc;Ccicicicicicichhhhhhhhhhc;"
    );
    for (id, code) in [
        (1, 'h'),
        (2, 'c'),
        (96, 'c'),
        (97, 's'),
        (98, 'i'),
        (99, 'h'),
        (103, 'c'),
        (104, 'i'),
        (106, 'q'),
        (150, 'b'),
        (165, 'c'),
        (166, 'B'),
        (173, 'B'),
        (174, 'q'),
        (217, 'i'),
        (218, 'B'),
        (241, 'B'),
        (242, 'c'),
        (243, 'i'),
        (244, '?'),
        (245, 'i'),
        (246, '?'),
        (247, 'i'),
        (344, 'F'),
        (367, 'F'),
        (422, 'h'),
    ] {
        assert_eq!(property_code(id), Some(code), "property {id}");
    }
    assert_eq!(property_code(0), None);
    assert_eq!(property_code(423), None);
    assert_eq!(property_code(usize::MAX), None);
    assert_eq!(
        (1..=422)
            .filter(|id| property_code(*id) == Some('B'))
            .count(),
        32
    );
    assert_eq!(
        (1..=422)
            .filter(|id| property_code(*id) == Some('F'))
            .count(),
        24
    );
    assert_eq!(properties().get(422), Some(&PropertyValue::Short(0)));
}
#[test]
fn every_property_requires_its_declared_type_and_valid_id() {
    let mut p = properties();
    let before = p.clone();
    for id in 1..=422 {
        let wrong = if property_code(id as usize) == Some('c') {
            PropertyValue::Int(0)
        } else {
            PropertyValue::Byte(0)
        };
        assert!(p.set(id, wrong).is_err(), "property {id}");
    }
    assert_eq!(p, before);
    for id in [0, 423, u16::MAX] {
        assert!(p.get(id).is_none());
        assert!(p.set(id, PropertyValue::Byte(0)).is_err());
    }
    assert!(CreatureProperties::new(std::array::from_fn(|_| PropertyValue::Byte(0))).is_err());
}
#[test]
fn name_and_survey_limits_and_native_nul() {
    let mut p = properties();
    let base = p.encode();
    p.set(97, PropertyValue::String(vec![0x1234; 32])).unwrap();
    assert_eq!(p.encode().len(), base.len() + 64);
    assert!(p.set(97, PropertyValue::String(vec![1; 33])).is_err());
    p.set(97, PropertyValue::String(vec![0x1234, 0, 0x5678]))
        .unwrap();
    let b = p.encode();
    // World ID (2) and race/sex/job/appearance (95 bytes) precede the name.
    assert_eq!(&b[97..101], &[1, 0, 0x34, 0x12]);
    assert_eq!(b.len(), base.len() + 2);
    p.set(150, PropertyValue::Bytes(vec![7; 10])).unwrap();
    assert_eq!(p.encode().len(), base.len() + 12);
    assert!(p.set(150, PropertyValue::Bytes(vec![7; 11])).is_err());
}
#[test]
fn native_bit_and_flag_groups_have_different_semantics() {
    for start in [166, 218, 226, 234] {
        let mut p = properties();
        let base = p.encode();
        for bit in [0, 2, 7] {
            p.set(start + bit, PropertyValue::Bit(true)).unwrap();
        }
        let changes: Vec<_> = p
            .encode()
            .into_iter()
            .zip(base)
            .filter(|(a, b)| a != b)
            .collect();
        assert_eq!(changes, vec![(0x85, 0)]);
    }
    for start in [344, 352, 360] {
        let mut p = properties();
        let base = p.encode();
        for bit in 1..8 {
            p.set(start + bit, PropertyValue::Bit(true)).unwrap();
        }
        assert_eq!(p.encode(), base); // The native getter repeats only the first member.
        p.set(start, PropertyValue::Bit(true)).unwrap();
        let changes: Vec<_> = p
            .encode()
            .into_iter()
            .zip(base)
            .filter(|(a, b)| a != b)
            .collect();
        assert_eq!(changes, vec![(255, 0)]);
    }
}
#[test]
fn ability_question_slots_are_eight_byte_values() {
    let mut p = properties();
    let base = p.encode();
    p.set(244, PropertyValue::Long(u64::MAX)).unwrap();
    p.set(246, PropertyValue::Long(u64::MAX)).unwrap();
    let encoded = p.encode();
    assert_eq!(encoded.len(), base.len());
    assert_eq!(encoded.iter().filter(|b| **b == 255).count(), 16);
}
#[test]
fn complete_prefix_and_suffix_without_user() {
    let mut e = fixture();
    e.recycle_remaining[0] = Some(-4);
    e.recycle_remaining[24] = Some(99);
    let body = joined(&e);
    assert_eq!(
        &body[..25],
        &[
            7, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0x11, 0x12, 0x13, 0x14, 21, 0, 22, 0, 23, 0, 196, 255,
            0, 0
        ]
    );
    let mut expected = vec![
        2, 0, 0xA1, 0xA2, 0, 0, 2, 0, 1, 0, 0, 0, 0, 25, 99, 0, 0, 0, 2, 0, 1, 1, 0, 0xB1, 2, 2, 0,
        0xB2, 0xB3, 31, 32, 33, 34, 35,
    ];
    expected.extend(36u64.to_le_bytes());
    expected.extend([0, 0, 1, 2, 0, 1, 254, 255, 14, 3, 0, 0, 0, 2, 0]);
    for page in [1, 2] {
        expected.push(page);
        for _ in 0..6 {
            expected.extend([1, 2, 3, 4, 5]);
        }
        for _ in 0..9 {
            expected.extend([6, 0]);
        }
        expected.extend([7, 0, 8]);
    }
    assert_eq!(suffix(&e), expected);
}
#[test]
fn user_bans_time_rows_and_rights() {
    let mut e = fixture();
    let base = joined(&e).len();
    let mut bans = [ContentBan::default(); 9];
    bans[0].effective_from = -1;
    bans[8].effective_to = 123;
    e.user = Some(EntryUser {
        game_packet_rights: 0x1234,
        content_bans: bans,
        time_based_play: [time(), time()],
    });
    let b = joined(&e);
    assert_eq!(&b[23..25], &[0x34, 0x12]);
    assert_eq!(b.len(), base + 34 + 62);
    let s = suffix(&e);
    // Two time rows follow their count, immediately before the slate count/pages.
    let time_start = s.len() - 106 - 64;
    assert_eq!(&s[time_start..time_start + 2], &[2, 0]);
    let mut row = vec![0];
    row.extend(41i32.to_le_bytes());
    row.extend(42i64.to_le_bytes());
    row.extend(43i64.to_le_bytes());
    row.extend(44i32.to_le_bytes());
    row.extend(45i16.to_le_bytes());
    row.extend([46, 47, 48, 49]);
    assert_eq!(row.len(), 31);
    assert_eq!(&s[time_start + 2..time_start + 33], row);
    row[0] = 1;
    assert_eq!(&s[time_start + 33..time_start + 64], row);
    // Bans precede the test flag, craft count and two craft rows.
    let ban_start = time_start - 9 - 36;
    let mut expected = vec![2, 0, 0];
    expected.extend((-1i64).to_le_bytes());
    expected.extend(0i64.to_le_bytes());
    expected.push(8);
    expected.extend(0i64.to_le_bytes());
    expected.extend(123i64.to_le_bytes());
    assert_eq!(&s[ban_start..ban_start + 36], expected);
    e.content_bans_enabled = false;
    assert_eq!(joined(&e).len(), base + 62);
}
#[test]
fn craft_filter_matches_native_range() {
    let mut e = fixture();
    let base = joined(&e).len();
    for id in 0..=255 {
        e.craft_unions = [CraftUnion { id, level: 3 }; 2];
        assert_eq!(
            joined(&e).len(),
            base - if (1..=14).contains(&id) { 0 } else { 6 }
        );
    }
}
#[test]
fn fragment_groups_marks_and_reassembly() {
    let mut e = fixture();
    e.extra_properties = vec![0xA5; 3990];
    e.opaque_broadcast_block = vec![0x5A; 3990];
    let fragments = e.encode_fragments().unwrap();
    assert_eq!(fragments.len(), 4);
    assert_eq!(&fragments[1][8..10], &3990u16.to_le_bytes());
    assert_eq!(&fragments[1][10..], &e.extra_properties);
    assert_eq!(&fragments[2][10..], &e.opaque_broadcast_block);
    assert_eq!(fragments[1].len(), 4000);
    let total: usize = fragments.iter().map(|p| p.len() - 8).sum();
    for (i, p) in fragments.iter().enumerate() {
        assert!(p.len() <= 4000);
        assert_eq!(
            u16::from_le_bytes(p[..2].try_into().unwrap()) as usize,
            p.len()
        );
        assert_eq!(&p[2..4], &[32, 0]);
        assert_eq!(
            i32::from_le_bytes(p[4..8].try_into().unwrap()),
            if i == 0 {
                total as i32
            } else if i == 3 {
                0
            } else {
                -1
            }
        );
    }
    assert_eq!(joined(&e).len(), total);
    let small = fixture().encode_fragments().unwrap();
    assert_eq!(small.len(), 1);
    assert_eq!(
        i32::from_le_bytes(small[0][4..8].try_into().unwrap()) as usize,
        small[0].len() - 8
    );
}
#[test]
fn indivisible_groups_and_count_overflow_are_errors() {
    let mut e = fixture();
    e.extra_properties = vec![0; 3991];
    assert!(e.encode_fragments().is_err());
    e.extra_properties.clear();
    e.opaque_broadcast_block = vec![0; 32768];
    assert!(e.encode_fragments().is_err());
    e.opaque_broadcast_block.clear();
    e.acquired_skills[0] = vec![0; 3990]; // Page + byte count + data = 3993.
    assert!(e.encode_fragments().is_err());
}
