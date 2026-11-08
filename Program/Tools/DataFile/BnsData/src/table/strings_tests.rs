use super::*;
use crate::tabledef::{tests::definition, Attribute, TableDefinition};

fn attribute(kind: &str, offset: usize, size: usize, repeat: usize) -> Attribute {
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
fn sizes(width: Width) -> (usize, usize) {
    match width {
        Width::Bits32 => (4, 8),
        Width::Bits64 => (8, 12),
    }
}
fn fixture(width: Width, compressed: bool) -> (TableDefinition, Vec<u8>) {
    let (ss, ns) = sizes(width);
    let size = 16 + ns + 2 * ss;
    let mut def = definition();
    def.el[0].body.size = size;
    // Native comes first in the schema. The output pool must still put strings first.
    def.el[0].body.attr = vec![
        attribute("int64", 8, 8, 1),
        attribute("native", 16, ns, 1),
        attribute("string", 16 + ns, ss, 2),
    ];
    def.el[0].body.attrcount = 3;
    let pool = vec![65, 0, 0, 0, 66, 0, 67, 0, 0, 0, 78, 0, 0, 0, 250];
    let records = [1i64, 2]
        .into_iter()
        .map(|key| {
            let mut fixed = vec![0; size];
            fixed[0] = 1;
            fixed[1] = 73;
            fixed[2..4].copy_from_slice(&(-1i16).to_le_bytes());
            fixed[4..6].copy_from_slice(&(size as u16).to_le_bytes());
            fixed[8..16].copy_from_slice(&key.to_le_bytes());
            fixed[16..20].copy_from_slice(&0xfedcba98u32.to_le_bytes());
            fixed[20..24].copy_from_slice(&10u32.to_le_bytes());
            let start = 16 + ns + ss;
            fixed[start..start + 4].copy_from_slice(&4u32.to_le_bytes());
            Record {
                fixed,
                extra: if compressed { pool.clone() } else { vec![] },
                key: None,
            }
        })
        .collect();
    let variant = if compressed {
        Variant::Compressed(CompressedBody {
            constant: 8,
            blocks: vec![Block {
                first_key: 1,
                last_key: 2,
                records,
            }],
        })
    } else {
        Variant::Plain(PlainBody {
            stored_count: 2,
            low_memory: 7,
            records,
            extra: pool,
        })
    };
    let bytes = TableBody::new(variant).write(width, 2, Some(&def)).unwrap();
    (def, bytes)
}
fn records(table: &TableBody) -> Vec<(&Record, &[u8])> {
    match &table.variant {
        Variant::Plain(body) => body
            .records
            .iter()
            .map(|r| (r, body.extra.as_slice()))
            .collect(),
        Variant::Compressed(body) => body
            .blocks
            .iter()
            .flat_map(|b| &b.records)
            .map(|r| (r, r.extra.as_slice()))
            .collect(),
    }
}
fn fields(
    table: &TableBody,
    def: &TableDefinition,
    width: Width,
    index: usize,
) -> Vec<DecodedField> {
    let all = records(table);
    decode_fields_with_pool(all[index].0, def, all[index].1, width).unwrap()
}
fn text_units(value: &RawValue) -> &[u16] {
    match value {
        RawValue::Text { units, .. } | RawValue::Native { units, .. } => units,
        _ => panic!(),
    }
}
fn text_offset(value: &RawValue) -> u64 {
    match value {
        RawValue::Text { offset, .. } | RawValue::Native { offset, .. } => *offset,
        _ => panic!(),
    }
}

#[test]
fn resolve_both_widths_and_variants_and_preserve_noops() {
    for width in [Width::Bits32, Width::Bits64] {
        for compressed in [false, true] {
            let (def, bytes) = fixture(width, compressed);
            let mut table = TableBody::parse(&bytes, width, 2, Some(&def)).unwrap();
            let values = fields(&table, &def, width, 0);
            assert_eq!(
                values[2].value,
                RawValue::Text {
                    offset: 0,
                    units: vec![65]
                }
            );
            assert_eq!(
                values[1].value,
                RawValue::Native {
                    a: 0xfedcba98,
                    offset: 10,
                    units: vec![78]
                }
            );
            assert_eq!(values[3].value.text_string().unwrap(), "BC");
            assert_eq!(bytes, table.write(width, 2, Some(&def)).unwrap());
            let raw = decode_fields(records(&table)[0].0, &def).unwrap();
            assert!(matches!(raw[1].value, RawValue::Unresolved { .. }));
            let mut editor = table.edit_record(0, &def, width).unwrap();
            editor.set_text("string", 0, &[65]).unwrap();
            editor.set_text("native", 0, &[78]).unwrap();
            assert_eq!(bytes, table.write(width, 2, Some(&def)).unwrap());
            table
                .edit_record(0, &def, width)
                .unwrap()
                .set_text_str("string", 0, "changed")
                .unwrap();
            table
                .edit_record(0, &def, width)
                .unwrap()
                .set_text("string", 0, &[65])
                .unwrap();
            assert_eq!(bytes, table.write(width, 2, Some(&def)).unwrap());
        }
    }
}

#[test]
fn target_edit_plain_dedup_and_compressed_order() {
    for width in [Width::Bits32, Width::Bits64] {
        for compressed in [false, true] {
            let (def, bytes) = fixture(width, compressed);
            let mut table = TableBody::parse(&bytes, width, 2, Some(&def)).unwrap();
            let untouched = records(&table)[1].0.clone();
            table
                .edit_record(0, &def, width)
                .unwrap()
                .set_text("string", 1, &[65])
                .unwrap();
            let changed = table.write(width, 2, Some(&def)).unwrap();
            assert_ne!(bytes, changed);
            assert_eq!(changed, table.write(width, 2, Some(&def)).unwrap());
            let parsed = TableBody::parse(&changed, width, 2, Some(&def)).unwrap();
            let first = fields(&parsed, &def, width, 0);
            let second = fields(&parsed, &def, width, 1);
            assert_eq!(text_units(&first[2].value), [65]);
            assert_eq!(text_units(&first[3].value), [65]);
            assert_eq!(text_units(&second[3].value), [66, 67]);
            assert!(matches!(
                first[1].value,
                RawValue::Native { a: 0xfedcba98, .. }
            ));
            assert_eq!(first[0].value, RawValue::Signed(1));
            assert_eq!(second[0].value, RawValue::Signed(2));
            if compressed {
                assert_eq!(records(&parsed)[1].0, &untouched);
                assert_eq!(text_offset(&first[2].value), 0);
                assert_eq!(text_offset(&first[3].value), 4);
                assert_eq!(text_offset(&first[1].value), 8);
            } else {
                assert_eq!(text_offset(&first[2].value), text_offset(&first[3].value));
                assert_eq!(text_offset(&first[2].value), text_offset(&second[2].value));
                // A, N, BC: only three distinct terminated strings remain.
                assert_eq!(records(&parsed)[0].1.len(), 14);
            }
        }
    }
}

#[test]
fn native_edit_sets_size_and_keeps_other_text() {
    for width in [Width::Bits32, Width::Bits64] {
        for compressed in [false, true] {
            let (def, bytes) = fixture(width, compressed);
            let mut table = TableBody::parse(&bytes, width, 2, Some(&def)).unwrap();
            let second_a = match fields(&table, &def, width, 1)[1].value {
                RawValue::Native { a, .. } => a,
                _ => panic!("not native"),
            };
            table
                .edit_record(0, &def, width)
                .unwrap()
                .set_text_str("native", 0, "new native")
                .unwrap();
            let changed = table.write(width, 2, Some(&def)).unwrap();
            let parsed = TableBody::parse(&changed, width, 2, Some(&def)).unwrap();
            let first = fields(&parsed, &def, width, 0);
            assert!(matches!(
                first[1].value,
                RawValue::Native { a: 22, .. }
            ));
            // Unedited records keep their stored size.
            assert!(matches!(
                fields(&parsed, &def, width, 1)[1].value,
                RawValue::Native { a, .. } if a == second_a
            ));
            assert_eq!(first[1].value.text_string().unwrap(), "new native");
            assert_eq!(first[2].value.text_string().unwrap(), "A");
            assert_eq!(first[3].value.text_string().unwrap(), "BC");
            assert_eq!(
                fields(&parsed, &def, width, 1)[1]
                    .value
                    .text_string()
                    .unwrap(),
                "N"
            );
        }
    }
}

#[test]
fn bad_offsets_terminators_width_and_native_padding() {
    for width in [Width::Bits32, Width::Bits64] {
        let (def, bytes) = fixture(width, false);
        let table = TableBody::parse(&bytes, width, 2, Some(&def)).unwrap();
        let (source, pool) = records(&table)[0];
        let (ss, ns) = sizes(width);
        for offset in [1u64, 16, u64::from(u32::MAX)] {
            let mut record = source.clone();
            let start = 16 + ns;
            record.fixed[start..start + ss].copy_from_slice(&offset.to_le_bytes()[..ss]);
            assert!(decode_fields_with_pool(&record, &def, pool, width).is_err());
            // The raw API does not interpret or validate text descriptors.
            assert!(decode_fields(&record, &def).is_ok());
        }
        assert!(decode_fields_with_pool(source, &def, &[], width).is_err());
        let mut no_terminator = pool.to_vec();
        no_terminator[12..14].copy_from_slice(&1u16.to_le_bytes());
        assert!(decode_fields_with_pool(source, &def, &no_terminator, width).is_err());
        let other_width = if width == Width::Bits32 {
            Width::Bits64
        } else {
            Width::Bits32
        };
        assert!(decode_fields_with_pool(source, &def, pool, other_width).is_err());
        if width == Width::Bits64 {
            let mut record = source.clone();
            record.fixed[24] = 1;
            assert!(decode_fields_with_pool(&record, &def, pool, width).is_err());
        }
        let mut native_bad = source.clone();
        native_bad.fixed[20..24].copy_from_slice(&1u32.to_le_bytes());
        assert!(decode_fields_with_pool(&native_bad, &def, pool, width).is_err());
    }
}

#[test]
fn surrogate_units_are_lossless_and_nul_is_rejected() {
    for width in [Width::Bits32, Width::Bits64] {
        for compressed in [false, true] {
            let (def, bytes) = fixture(width, compressed);
            let mut table = TableBody::parse(&bytes, width, 2, Some(&def)).unwrap();
            let units = [0xd800, 65, 0xdc00];
            table
                .edit_record(0, &def, width)
                .unwrap()
                .set_text("string", 0, &units)
                .unwrap();
            let changed = table.write(width, 2, Some(&def)).unwrap();
            let mut parsed = TableBody::parse(&changed, width, 2, Some(&def)).unwrap();
            let values = fields(&parsed, &def, width, 0);
            assert_eq!(text_units(&values[2].value), units);
            assert!(values[2].value.text_string().is_err());
            assert!(RawValue::Signed(1).text_string().is_err());
            let mut editor = parsed.edit_record(0, &def, width).unwrap();
            assert!(editor.set_text("string", 0, &[65, 0, 66]).is_err());
            editor.set_text_str("string", 0, "😀").unwrap();
            editor.set_text("native", 0, &[]).unwrap();
            let changed = parsed.write(width, 2, Some(&def)).unwrap();
            let parsed = TableBody::parse(&changed, width, 2, Some(&def)).unwrap();
            let values = fields(&parsed, &def, width, 0);
            assert_eq!(values[2].value.text_string().unwrap(), "😀");
            assert!(text_units(&values[1].value).is_empty());
        }
    }
}

#[test]
fn editor_errors_are_checked_and_do_not_change_output() {
    for width in [Width::Bits32, Width::Bits64] {
        let (def, bytes) = fixture(width, false);
        let mut table = TableBody::parse(&bytes, width, 2, Some(&def)).unwrap();
        assert!(table.edit_record(2, &def, width).is_err());
        let mut editor = table.edit_record(0, &def, width).unwrap();
        assert!(editor.set_text("missing", 0, &[]).is_err());
        assert!(editor.set_text("string", 2, &[]).is_err());
        assert!(editor.set_text("int64", 0, &[]).is_err());
        assert_eq!(bytes, table.write(width, 2, Some(&def)).unwrap());
        table
            .edit_record(0, &def, width)
            .unwrap()
            .set_text("string", 0, &[90])
            .unwrap();
        assert!(table.write(width, 2, None).is_err());
        let other = if width == Width::Bits32 {
            Width::Bits64
        } else {
            Width::Bits32
        };
        assert!(table.write(other, 2, Some(&def)).is_err());
        let mut ambiguous = def.clone();
        let duplicate = ambiguous.el[0].body.attr[2].clone();
        ambiguous.el[0].body.attr.push(duplicate);
        assert!(table
            .edit_record(0, &ambiguous, width)
            .unwrap()
            .set_text("string", 0, &[])
            .is_err());
    }
}

#[test]
fn text_growth_reblocks_and_rejects_oversized_record() {
    for width in [Width::Bits32, Width::Bits64] {
        let (def, bytes) = fixture(width, true);
        let mut table = TableBody::parse(&bytes, width, 2, Some(&def)).unwrap();
        for index in 0..2 {
            table
                .edit_record(index, &def, width)
                .unwrap()
                .set_text("string", 0, &[65; 20000])
                .unwrap();
        }
        let changed = table.write(width, 2, Some(&def)).unwrap();
        let parsed = TableBody::parse(&changed, width, 2, Some(&def)).unwrap();
        let Variant::Compressed(body) = &parsed.variant else {
            panic!()
        };
        assert_eq!(body.blocks.len(), 2);
        assert_eq!(
            text_units(&fields(&parsed, &def, width, 0)[2].value).len(),
            20000
        );
        table
            .edit_record(0, &def, width)
            .unwrap()
            .set_text("string", 0, &[65; 32768])
            .unwrap();
        assert!(table.write(width, 2, Some(&def)).is_err());
    }
}

#[test]
fn subtype_text_fields_are_resolved_and_rebuilt() {
    for width in [Width::Bits32, Width::Bits64] {
        let (mut def, bytes) = fixture(width, false);
        let mut table = TableBody::parse(&bytes, width, 2, Some(&def)).unwrap();
        let (ss, _) = sizes(width);
        let base_size = def.el[0].body.size;
        def.el[0].body.sub[0].size = base_size + ss;
        def.el[0].body.sub[0].attr = vec![attribute("subtext", base_size, ss, 1)];
        def.el[0].body.sub[0].attr[0].kind = "string".into();
        let Variant::Plain(body) = &mut table.variant else {
            panic!()
        };
        for record in &mut body.records {
            record.fixed.resize(base_size + ss, 0);
            record.fixed[2..4].copy_from_slice(&0i16.to_le_bytes());
            record.fixed[4..6].copy_from_slice(&((base_size + ss) as u16).to_le_bytes());
        }
        table
            .edit_record(0, &def, width)
            .unwrap()
            .set_text_str("subtext", 0, "subtype")
            .unwrap();
        let changed = table.write(width, 2, Some(&def)).unwrap();
        let parsed = TableBody::parse(&changed, width, 2, Some(&def)).unwrap();
        assert_eq!(
            fields(&parsed, &def, width, 0)[4]
                .value
                .text_string()
                .unwrap(),
            "subtype"
        );
        assert_eq!(
            fields(&parsed, &def, width, 1)[4]
                .value
                .text_string()
                .unwrap(),
            "A"
        );
    }
}
