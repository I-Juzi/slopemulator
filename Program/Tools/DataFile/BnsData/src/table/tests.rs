use super::*;
use crate::tabledef::{tests::definition, Attribute};

fn record(key: i64, extra_size: usize) -> Record {
    let mut fixed = vec![1, 73, 255, 255, 16, 0, 0, 0];
    fixed.extend_from_slice(&key.to_le_bytes());
    Record {
        fixed,
        extra: vec![42; extra_size],
        key: None,
    }
}
fn plain(count: u64, records: Vec<Record>) -> TableBody {
    TableBody::new(Variant::Plain(PlainBody {
        stored_count: count,
        low_memory: 7,
        records,
        extra: vec![19, 0, 21],
    }))
}
fn compressed(records: Vec<Record>) -> TableBody {
    TableBody::new(Variant::Compressed(CompressedBody {
        constant: 8,
        blocks: vec![Block {
            first_key: 0,
            last_key: 0,
            records,
        }],
    }))
}
#[test]
fn plain_kinds_and_widths_roundtrip() {
    let def = definition();
    for width in [Width::Bits32, Width::Bits64] {
        for kind in [1, 2] {
            let source = plain(99, vec![record(3, 0), record(5, 0)]);
            let bytes = source.write(width, kind, Some(&def)).unwrap();
            let parsed = TableBody::parse(&bytes, width, kind, Some(&def)).unwrap();
            assert_eq!(parsed.record_count(), 2);
            let Variant::Plain(body) = &parsed.variant else {
                panic!()
            };
            assert_eq!(body.stored_count, if kind == 1 { 3 } else { 2 });
            assert_eq!(body.low_memory, 7);
            assert_eq!(body.extra, [19, 0, 21]);
            assert_eq!(bytes, parsed.write(width, kind, Some(&def)).unwrap());
        }
    }
}
#[test]
fn empty_auto_preserves_both_counts() {
    for width in [Width::Bits32, Width::Bits64] {
        for count in [0, 1] {
            let bytes = plain(count, vec![]).write(width, 1, None).unwrap();
            let parsed = TableBody::parse(&bytes, width, 1, None).unwrap();
            let Variant::Plain(body) = &parsed.variant else {
                panic!()
            };
            assert_eq!(body.stored_count, count);
            assert_eq!(bytes, parsed.write(width, 1, None).unwrap());
        }
    }
    assert!(plain(2, vec![]).write(Width::Bits64, 1, None).is_err());
}
#[test]
fn compressed_multiblock_roundtrip_and_mutation() {
    let def = definition();
    for width in [Width::Bits32, Width::Bits64] {
        for kind in [1, 2] {
            let source = compressed(vec![record(3, 40000), record(9, 40000), record(15, 12)]);
            let bytes = source.write(width, kind, Some(&def)).unwrap();
            let mut parsed = TableBody::parse(&bytes, width, kind, Some(&def)).unwrap();
            assert_eq!(parsed.record_count(), 3);
            let Variant::Compressed(body) = &mut parsed.variant else {
                panic!()
            };
            assert_eq!(body.blocks.len(), 2);
            assert_eq!((body.blocks[0].first_key, body.blocks[1].last_key), (3, 15));
            assert_eq!(bytes, parsed.write(width, kind, Some(&def)).unwrap());
            let Variant::Compressed(body) = &mut parsed.variant else {
                panic!()
            };
            body.blocks[0].records[0].extra[0] = 99;
            body.blocks[0].records[0].fixed[8..16].copy_from_slice(&4i64.to_le_bytes());
            let changed = parsed.write(width, kind, Some(&def)).unwrap();
            assert_ne!(bytes, changed);
            let reparsed = TableBody::parse(&changed, width, kind, Some(&def)).unwrap();
            let Variant::Compressed(body) = &reparsed.variant else {
                panic!()
            };
            assert_eq!(body.blocks[0].first_key, 4);
            assert_eq!(body.blocks[0].records[0].extra[0], 99);
            assert_eq!(changed, reparsed.write(width, kind, None).unwrap());
        }
    }
}
#[test]
fn original_zlib_stream_is_preserved() {
    let def = definition();
    let source = record(7, 128);
    let mut raw = source.fixed.clone();
    raw.extend_from_slice(&source.extra);
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::none());
    encoder.write_all(&raw).unwrap();
    let zlib = encoder.finish().unwrap();
    let mut bytes = vec![1];
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&27u16.to_le_bytes());
    bytes.extend_from_slice(&7i64.to_le_bytes());
    bytes.extend_from_slice(&7i64.to_le_bytes());
    bytes.extend_from_slice(&(zlib.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&zlib);
    bytes.extend_from_slice(&(raw.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    let mut parsed = TableBody::parse(&bytes, Width::Bits32, 2, Some(&def)).unwrap();
    assert_eq!(bytes, parsed.write(Width::Bits32, 2, None).unwrap());
    parsed.discard_original();
    let changed = parsed.write(Width::Bits32, 2, Some(&def)).unwrap();
    assert_ne!(bytes, changed);
    assert_eq!(
        TableBody::parse(&bytes, Width::Bits32, 2, None)
            .unwrap()
            .variant,
        TableBody::parse(&changed, Width::Bits32, 2, None)
            .unwrap()
            .variant
    );
}
#[test]
fn rejects_truncations_and_bad_plain_sizes() {
    let def = definition();
    for width in [Width::Bits32, Width::Bits64] {
        for kind in [1, 2] {
            for source in [
                plain(2, vec![record(7, 0)]),
                compressed(vec![record(7, 20)]),
            ] {
                let bytes = source.write(width, kind, Some(&def)).unwrap();
                for end in 0..bytes.len() {
                    assert!(
                        TableBody::parse(&bytes[..end], width, kind, Some(&def)).is_err(),
                        "end={end}"
                    );
                }
                let mut trailing = bytes;
                trailing.push(0);
                assert!(TableBody::parse(&trailing, width, kind, None).is_err());
            }
        }
    }
    let bytes = plain(1, vec![record(7, 0)])
        .write(Width::Bits32, 2, None)
        .unwrap();
    for size in [0u16, 5, 17, u16::MAX] {
        let mut bad = bytes.clone();
        bad[18..20].copy_from_slice(&size.to_le_bytes());
        assert!(TableBody::parse(&bad, Width::Bits32, 2, None).is_err());
    }
    let mut bad = bytes;
    bad[1..5].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(TableBody::parse(&bad, Width::Bits32, 2, None).is_err());
}
#[test]
fn rejects_bad_blocks_and_overflow() {
    let def = definition();
    let source = compressed(vec![record(7, 12), record(9, 10)]);
    let bytes = source.write(Width::Bits32, 2, Some(&def)).unwrap();
    let cs = usize::from(u16::from_le_bytes(bytes[23..25].try_into().unwrap()));
    let ds_pos = 25 + cs;
    for (pos, replacement) in [
        (ds_pos, 1u16),         // Wrong inflate size.
        (ds_pos + 6, 1),        // First offset must be zero.
        (ds_pos + 8, u16::MAX), // Offset exceeds decompressed size.
        (ds_pos + 8, 0),        // Duplicate offsets.
    ] {
        let mut bad = bytes.clone();
        bad[pos..pos + 2].copy_from_slice(&replacement.to_le_bytes());
        assert!(TableBody::parse(&bad, Width::Bits32, 2, None).is_err());
    }
    let mut bad = bytes.clone();
    bad[7..15].copy_from_slice(&10i64.to_le_bytes());
    assert!(TableBody::parse(&bad, Width::Bits32, 2, None).is_err());
    let mut bad = bytes.clone();
    bad[25] ^= 255;
    assert!(TableBody::parse(&bad, Width::Bits32, 2, None).is_err());
    let mut bad = bytes;
    bad[ds_pos + 2..ds_pos + 6].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(TableBody::parse(&bad, Width::Bits32, 2, None).is_err());
    assert!(compressed(vec![record(1, 65520)])
        .write(Width::Bits32, 1, Some(&def))
        .is_err());
    assert!(compressed(vec![record(2, 0), record(1, 0)])
        .write(Width::Bits32, 1, Some(&def))
        .is_err());
}
#[test]
fn rejects_unfinished_zlib_stream_and_compressed_span_overrun() {
    let def = definition();
    let bytes = compressed(vec![record(7, 0)])
        .write(Width::Bits32, 2, Some(&def))
        .unwrap();
    let cs = usize::from(u16::from_le_bytes(bytes[23..25].try_into().unwrap()));
    for remove in 1..=4 {
        let mut bad = bytes.clone();
        bad.drain(25 + cs - remove..25 + cs);
        bad[23..25].copy_from_slice(&((cs - remove) as u16).to_le_bytes());
        assert!(TableBody::parse(&bad, Width::Bits32, 2, None).is_err());
    }
    let mut oversized = record(7, 0).fixed;
    oversized[4..6].copy_from_slice(&17u16.to_le_bytes());
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&oversized).unwrap();
    let zlib = encoder.finish().unwrap();
    let mut bad = bytes[..23].to_vec();
    bad.extend_from_slice(&(zlib.len() as u16).to_le_bytes());
    bad.extend_from_slice(&zlib);
    bad.extend_from_slice(&16u16.to_le_bytes());
    bad.extend_from_slice(&1u32.to_le_bytes());
    bad.extend_from_slice(&0u16.to_le_bytes());
    assert!(TableBody::parse(&bad, Width::Bits32, 2, None).is_err());
}
#[test]
fn definitions_validate_type_subtype_and_size() {
    let def = definition();
    let mut source = record(1, 0);
    assert!(source.validate(Some(&def)).is_ok());
    source.fixed[0] = 2;
    assert!(source.validate(Some(&def)).is_err());
    source.fixed[0] = 1;
    source.fixed[2..4].copy_from_slice(&0i16.to_le_bytes());
    assert!(source.validate(Some(&def)).is_err());
    source.fixed.extend_from_slice(&33i32.to_le_bytes());
    source.fixed[4..6].copy_from_slice(&20u16.to_le_bytes());
    assert!(source.validate(Some(&def)).is_ok());
    let fields = decode_fields(&source, &def).unwrap();
    assert_eq!(fields[1].value, RawValue::Signed(33));
}
#[test]
fn explicit_keys_and_missing_schema() {
    let mut source = compressed(vec![record(2, 0)]);
    assert!(source.write(Width::Bits32, 1, None).is_err());
    let Variant::Compressed(body) = &mut source.variant else {
        panic!()
    };
    body.blocks[0].records[0].key = Some(100);
    let bytes = source.write(Width::Bits32, 1, None).unwrap();
    let parsed = TableBody::parse(&bytes, Width::Bits32, 1, None).unwrap();
    let Variant::Compressed(body) = parsed.variant else {
        panic!()
    };
    assert_eq!(body.blocks[0].first_key, 100);
    source.compressed = false;
    assert!(source.write(Width::Bits32, 1, None).is_err());
    assert!(source.write(Width::Bits32, 0, None).is_err());
}
fn attr(kind: &str, offset: usize, size: usize, repeat: usize) -> Attribute {
    Attribute {
        name: kind.into(),
        kind: kind.into(),
        offset,
        size,
        repeat,
        key: false,
        required: 0,
        deprecated: false,
        hidden: false,
        seq: None,
        reftable: 0,
        refel: 0,
        min: 0,
        max: 0,
        fmin: 0.0,
        fmax: 0.0,
        defvalue: serde_json::Value::Null,
    }
}
#[test]
fn scalar_fields_repeats_and_unresolved_offsets() {
    let mut def = definition();
    let fields = vec![
        attr("int8", 8, 1, 2),
        attr("int16", 10, 2, 1),
        attr("int32", 12, 4, 1),
        attr("int64", 16, 8, 1),
        attr("bool", 24, 1, 1),
        attr("float32", 25, 4, 1),
        attr("seq", 29, 1, 1),
        attr("seq16", 30, 2, 1),
        attr("msec", 32, 4, 1),
        attr("distance", 36, 2, 1),
        attr("ref", 38, 8, 1),
        attr("string", 46, 8, 1),
        attr("native", 54, 12, 1),
        attr("unknown", 66, 4, 1),
    ];
    def.el[0].body.size = 70;
    def.el[0].body.attrcount = fields.len();
    def.el[0].body.attr = fields;
    let mut record = record(0, 0);
    record.fixed.resize(70, 0);
    record.fixed[4..6].copy_from_slice(&70u16.to_le_bytes());
    record.fixed[8..10].copy_from_slice(&[255, 127]);
    record.fixed[10..12].copy_from_slice(&(-123i16).to_le_bytes());
    record.fixed[12..16].copy_from_slice(&i32::MIN.to_le_bytes());
    record.fixed[16..24].copy_from_slice(&i64::MIN.to_le_bytes());
    record.fixed[24] = 2;
    record.fixed[25..29].copy_from_slice(&0x7fc00001u32.to_le_bytes());
    record.fixed[29] = 255;
    record.fixed[30..32].copy_from_slice(&65535u16.to_le_bytes());
    let values = decode_fields(&record, &def).unwrap();
    assert_eq!(values.len(), 15);
    assert_eq!(
        (values[0].repeat_index, &values[0].value),
        (0, &RawValue::Signed(-1))
    );
    assert_eq!(
        (values[1].repeat_index, &values[1].value),
        (1, &RawValue::Signed(127))
    );
    assert_eq!(values[2].value, RawValue::Signed(-123));
    assert_eq!(values[3].value, RawValue::Signed(i64::from(i32::MIN)));
    assert_eq!(values[4].value, RawValue::Signed(i64::MIN));
    assert_eq!(values[5].value, RawValue::Bool(2));
    assert_eq!(values[6].value, RawValue::Float32Bits(0x7fc00001));
    assert_eq!(values[7].value, RawValue::Unsigned(255));
    assert_eq!(values[8].value, RawValue::Unsigned(65535));
    assert!(matches!(values[11].value, RawValue::Ref(_)));
    assert!(matches!(values[12].value, RawValue::Unresolved { .. }));
    assert!(matches!(values[13].value, RawValue::Unresolved { .. }));
    assert!(matches!(values[14].value, RawValue::Unresolved { .. }));
}
#[test]
fn maximum_block_and_incompressible_reblocking() {
    use rand::{RngCore, SeedableRng};
    let def = definition();
    let source = compressed(vec![record(1, 65535 - 16)]);
    let bytes = source.write(Width::Bits64, 1, Some(&def)).unwrap();
    assert_eq!(
        TableBody::parse(&bytes, Width::Bits64, 1, Some(&def))
            .unwrap()
            .record_count(),
        1
    );

    let mut rng = rand::rngs::StdRng::seed_from_u64(781);
    let mut first = record(2, 32768 - 16);
    let mut second = record(3, 32767 - 16);
    rng.fill_bytes(&mut first.extra);
    rng.fill_bytes(&mut second.extra);
    let source = compressed(vec![first.clone(), second.clone()]);
    let bytes = source.write(Width::Bits32, 2, Some(&def)).unwrap();
    let parsed = TableBody::parse(&bytes, Width::Bits32, 2, Some(&def)).unwrap();
    let Variant::Compressed(body) = parsed.variant else {
        panic!()
    };
    assert_eq!(body.blocks.len(), 2);
    assert_eq!(body.blocks[0].records, [first]);
    assert_eq!(body.blocks[1].records, [second]);
}
#[test]
fn rejects_field_extent_overflow_and_wrong_scalar_width() {
    let mut def = definition();
    let record = record(1, 0);
    for field in [
        attr("int64", usize::MAX, 8, 1),
        attr("int64", 8, 8, usize::MAX),
        attr("int64", 8, 3, 1),
        attr("int16", 8, 4, 1),
        attr("int8", 8, 2, 1),
        attr("ref", 15, 2, 1),
    ] {
        def.el[0].body.attr = vec![field];
        assert!(decode_fields(&record, &def).is_err());
    }
    // Size-0 fields are not stored in client records and are skipped.
    def.el[0].body.attr = vec![attr("int64", 0, 0, 1)];
    assert!(decode_fields(&record, &def).unwrap().is_empty());
    // A narrower integer (int16 stored in 1 byte) decodes by its stored size.
    def.el[0].body.attr = vec![attr("int16", 8, 1, 1)];
    assert_eq!(decode_fields(&record, &def).unwrap().len(), 1);
}
