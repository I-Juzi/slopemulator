use super::*;
use bnsdata::{
    dat::{Header, Table},
    table::{Block, CompressedBody, PlainBody, ScalarValue},
};
use serde_json::json;
use std::path::PathBuf;
fn definition(width: Width) -> TableDefinition {
    let attrs = json!([
        {"name":"id","type":"int64","offset":8,"size":8,"repeat":1,"key":true},
        {"name":"n","type":"int16","offset":16,"size":1,"repeat":1,"min":-5,"max":5,"defvalue":2},
        {"name":"b","type":"bool","offset":17,"size":1,"repeat":1},
        {"name":"e","type":"seq","offset":18,"size":1,"repeat":1,"seq":["off","on"]},
        {"name":"f","type":"float32","offset":20,"size":4,"repeat":1,"fmin":-2.0,"fmax":2.0},
        {"name":"link","type":"ref","offset":24,"size":8,"repeat":1,"reftable":7,"refel":1},
        {"name":"alias","type":"string","offset":32,"size":if width == Width::Bits32 {4} else {8},"repeat":1,"required":1},
        {"name":"native","type":"native","offset":40,"size":if width == Width::Bits32 {8} else {12},"repeat":1},
        {"name":"raw","type":"unknown","offset":52,"size":2,"repeat":1,"hidden":true,"deprecated":true},
        {"name":"skip","type":"int64","offset":0,"size":0,"repeat":1},
        {"name":"repeat","type":"int8","offset":54,"size":1,"repeat":2}
    ]);
    TableDefinition::from_json(&serde_json::to_vec(&json!({"name":"synthetic","type":7,"major_ver":1,"minor_ver":0,"autokey":false,"maxid":0,"module":0,"elcount":1,"el":[{"name":"record","type":1,"body":{"size":56,"keyoffset":8,"attrcount":11,"subcount":0,"attr":attrs}}]})).unwrap()).unwrap()
}
fn pool() -> Vec<u8> {
    "Alpha"
        .encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect()
}
fn container(width: Width, role: Role, compressed: bool) -> Container {
    let def = definition(width);
    let records = (1i64..=2)
        .map(|id| {
            let mut fixed = vec![0; 56];
            fixed[0] = 1;
            fixed[2..4].copy_from_slice(&(-1i16).to_le_bytes());
            fixed[4..6].copy_from_slice(&56u16.to_le_bytes());
            fixed[8..16].copy_from_slice(&id.to_le_bytes());
            fixed[16] = 2;
            fixed[17] = 1;
            fixed[18] = 1;
            fixed[20..24].copy_from_slice(&1f32.to_le_bytes());
            if id == 1 {
                fixed[24..32].copy_from_slice(&2i64.to_le_bytes());
            }
            fixed[40..44]
                .copy_from_slice(&(if width == Width::Bits32 { 99u32 } else { 100 }).to_le_bytes());
            fixed[52..54].copy_from_slice(&[42, 43]);
            fixed[54..56].copy_from_slice(&[3, 4]);
            Record {
                fixed,
                extra: if compressed { pool() } else { Vec::new() },
                key: None,
            }
        })
        .collect::<Vec<_>>();
    let variant = if compressed {
        Variant::Compressed(CompressedBody {
            constant: 9,
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
            extra: pool(),
        })
    };
    Container {
        width,
        role,
        header: Header {
            game_version: [0; 4],
            total_table_size: 777,
            total_table_count: 1,
            alias_size: 0,
            alias_count: 0,
            max_buffer_size: 888,
            save_time: 0,
            compressed: 0,
            encrypted: 0,
            reserved: [0; 56],
        },
        aliases: None,
        tables: vec![Table {
            kind: 2,
            type_id: 7,
            major: 1,
            minor: 0,
            body: TableBody::new(variant).write(width, 2, Some(&def)).unwrap(),
        }],
    }
}
fn project(compressed: bool, mirror: bool) -> Project {
    let mut containers = vec![container(Width::Bits32, Role::Main, compressed)];
    if mirror {
        containers.push(container(Width::Bits64, Role::Main, !compressed));
    }
    Project::from_containers(
        containers,
        Some(Definitions::from_definitions([definition(Width::Bits32)]).unwrap()),
        Some(Definitions::from_definitions([definition(Width::Bits64)]).unwrap()),
        Width::Bits32,
    )
    .unwrap()
}
fn edit(p: &Project, field: &str, new: Value) -> Edit {
    let old = p
        .record(TableId(0), 0)
        .unwrap()
        .into_iter()
        .find(|f| f.name == field)
        .unwrap()
        .value;
    Edit {
        table: TableId(0),
        record: 0,
        field: field.into(),
        repeat: 0,
        old,
        new,
    }
}
fn temp() -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "bnsmodel-synthetic-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ))
}
#[test]
fn typed_fields_widths_variants_and_lazy_cache() {
    for compressed in [false, true] {
        for width in [Width::Bits32, Width::Bits64] {
            let p = Project::from_containers(
                vec![container(width, Role::Main, compressed)],
                Some(Definitions::from_definitions([definition(Width::Bits32)]).unwrap()),
                Some(Definitions::from_definitions([definition(Width::Bits64)]).unwrap()),
                width,
            )
            .unwrap();
            assert!(p.cache.borrow().is_empty());
            assert_eq!(p.tables()[0].record_count, 2);
            let fields = p.record(TableId(0), 0).unwrap();
            assert_eq!(fields.len(), 11);
            assert!(fields.iter().all(|f| f.name != "skip"));
            assert_eq!(
                fields.iter().find(|f| f.name == "n").unwrap().value,
                Value::Int(2)
            );
            assert!(fields
                .iter()
                .find(|f| f.name == "raw")
                .is_some_and(|f| !f.editable && f.hidden && f.deprecated));
            assert!(fields
                .iter()
                .find(|f| f.name == "link")
                .is_some_and(|f| matches!(
                    f.value,
                    Value::Ref {
                        resolved: Some(RecordId { index: 1, .. }),
                        ..
                    }
                )));
            assert_eq!(p.record_title(TableId(0), 0).unwrap(), "Alpha");
            assert_eq!(p.search(TableId(0), "aLP", 1).unwrap(), vec![0]);
            assert_eq!(p.search(TableId(0), "2", 9).unwrap(), vec![1]);
            assert!(p.search(TableId(0), "", 0).unwrap().is_empty());
        }
    }
}
#[test]
fn checked_scalar_writes_all_types_and_repeats() {
    for width in [Width::Bits32, Width::Bits64] {
        let def = definition(width);
        let c = container(width, Role::Main, false);
        let mut body = TableBody::parse(&c.tables[0].body, width, 2, Some(&def)).unwrap();
        let Variant::Plain(b) = &mut body.variant else {
            unreachable!()
        };
        let r = &mut b.records[0];
        for (name, repeat, value) in [
            ("n", 0, ScalarValue::Int(-128)),
            ("b", 0, ScalarValue::Bool(false)),
            ("e", 0, ScalarValue::Enum(255)),
            ("f", 0, ScalarValue::Float(-0.0)),
            ("link", 0, ScalarValue::Ref(vec![0; 8])),
            ("repeat", 1, ScalarValue::Int(-5)),
        ] {
            r.set_scalar(&def, name, repeat, value).unwrap();
        }
        assert_eq!(r.fixed[16], 128);
        assert_eq!(r.fixed[55], 251);
        let before = r.clone();
        for (name, repeat, value) in [
            ("n", 0, ScalarValue::Int(128)),
            ("e", 0, ScalarValue::Enum(256)),
            ("link", 0, ScalarValue::Ref(vec![0; 4])),
            ("alias", 0, ScalarValue::Int(0)),
            ("repeat", 2, ScalarValue::Int(0)),
            ("skip", 0, ScalarValue::Int(0)),
        ] {
            assert!(r.set_scalar(&def, name, repeat, value).is_err());
            assert_eq!(*r, before);
        }
    }
}
#[test]
fn edits_mirror_undo_redo_discard_and_native_a() {
    let mut p = project(false, true);
    p.apply(edit(&p, "n", Value::Int(-3))).unwrap();
    assert_eq!(p.changes().unwrap().len(), 1);
    assert_eq!(
        p.current_fields(1, 0)
            .unwrap()
            .into_iter()
            .find(|f| f.name == "n")
            .unwrap()
            .value,
        Value::Int(-3)
    );
    assert!(p.undo().unwrap());
    assert!(p.changes().unwrap().is_empty());
    assert!(p.redo().unwrap());
    let change = p.changes().unwrap().remove(0);
    p.discard(&change).unwrap();
    assert!(p.changes().unwrap().is_empty());
    assert!(p.undo().unwrap());
    let text = "Edited".encode_utf16().collect::<Vec<_>>();
    p.apply(edit(
        &p,
        "native",
        Value::Native {
            a: 99,
            text: text.clone(),
        },
    ))
    .unwrap();
    assert_eq!(
        p.current_fields(1, 0)
            .unwrap()
            .into_iter()
            .find(|f| f.name == "native")
            .unwrap()
            .value,
        Value::Native { a: 100, text }
    );
    assert!(p
        .apply(edit(&p, "native", Value::Native { a: 5, text: vec![] }))
        .is_err());
}
#[test]
fn validation_and_stale_edits() {
    let mut p = project(true, false);
    for (field, value) in [
        ("n", Value::Int(6)),
        ("f", Value::Float(3.0)),
        (
            "e",
            Value::Enum {
                index: 2,
                name: None,
            },
        ),
        ("alias", Value::Text(vec![])),
    ] {
        let e = edit(&p, field, value);
        assert!(p
            .validate(&e)
            .unwrap()
            .iter()
            .any(|p| p.severity == Severity::Error));
        assert!(p.apply(e).is_err());
    }
    let e = edit(&p, "n", Value::Int(3));
    p.apply(e.clone()).unwrap();
    assert!(p.apply(e).is_err());
    let e = edit(
        &p,
        "link",
        Value::Ref {
            table: 7,
            key: 99i64.to_le_bytes().to_vec(),
            resolved: None,
        },
    );
    assert_eq!(p.validate(&e).unwrap()[0].severity, Severity::Warning);
    p.apply(e).unwrap();
    assert!(p
        .problems()
        .unwrap()
        .iter()
        .any(|p| p.severity == Severity::Warning));
    assert!(p.apply(edit(&p, "alias", Value::Text(vec![0]))).is_err());
    assert!(p.record(TableId(50), 0).is_err());
    assert!(p.record(TableId(0), 50).is_err());
}
#[test]
fn mirror_failure_is_atomic() {
    let mut p = project(false, true);
    p.definitions[1].as_mut().unwrap().0.get_mut(&7).unwrap().el[0]
        .body
        .attr[1]
        .max = 2;
    assert!(p.apply(edit(&p, "n", Value::Int(3))).is_err());
    assert!(p.changes().unwrap().is_empty());
    assert!(!p.undo().unwrap());
    p.definitions[1]
        .as_mut()
        .unwrap()
        .0
        .get_mut(&7)
        .unwrap()
        .name = "other".into();
    assert!(p.apply(edit(&p, "n", Value::Int(1))).is_err());
    assert!(p.changes().unwrap().is_empty());
}
#[test]
fn pack_unchanged_all_files_is_identical_and_never_overwrites() {
    let mut containers = Vec::new();
    for width in [Width::Bits32, Width::Bits64] {
        for role in [Role::Main, Role::Local] {
            containers.push(container(width, role, role == Role::Main));
        }
    }
    let originals: Vec<_> = containers.iter().map(|c| c.write().unwrap()).collect();
    let p = Project::from_containers(
        containers,
        Some(Definitions::from_definitions([definition(Width::Bits32)]).unwrap()),
        Some(Definitions::from_definitions([definition(Width::Bits64)]).unwrap()),
        Width::Bits32,
    )
    .unwrap();
    let dir = temp();
    let report = p.pack(&dir).unwrap();
    assert_eq!(report.files.len(), 4);
    for (file, original) in report.files.iter().zip(originals) {
        assert!(file.changed_tables.is_empty());
        assert_eq!(std::fs::read(&file.path).unwrap(), original);
    }
    assert!(p.pack(&dir).is_err());
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn pack_changed_widths_variants_text_and_scalar() {
    for compressed in [false, true] {
        let mut p = project(compressed, true);
        p.apply(edit(&p, "n", Value::Int(-4))).unwrap();
        p.apply(edit(
            &p,
            "alias",
            Value::Text("Beta".encode_utf16().collect()),
        ))
        .unwrap();
        p.apply(edit(
            &p,
            "native",
            Value::Native {
                a: 99,
                text: "Gamma".encode_utf16().collect(),
            },
        ))
        .unwrap();
        let dir = temp();
        let report = p.pack(&dir).unwrap();
        assert_eq!(report.files.len(), 2);
        for file in report.files {
            assert_eq!(file.changed_tables, vec!["synthetic"]);
            let width = if file.path.file_name().unwrap() == "datafile.bin" {
                Width::Bits32
            } else {
                Width::Bits64
            };
            let c =
                Container::parse(&std::fs::read(file.path).unwrap(), width, Role::Main).unwrap();
            let reopened = Project::from_containers(
                vec![c],
                Some(Definitions::from_definitions([definition(Width::Bits32)]).unwrap()),
                Some(Definitions::from_definitions([definition(Width::Bits64)]).unwrap()),
                width,
            )
            .unwrap();
            assert_eq!(reopened.record_title(TableId(0), 0).unwrap(), "Beta");
            assert_eq!(reopened.record_title(TableId(0), 1).unwrap(), "Alpha");
            assert_eq!(
                reopened
                    .record(TableId(0), 0)
                    .unwrap()
                    .into_iter()
                    .find(|f| f.name == "n")
                    .unwrap()
                    .value,
                Value::Int(-4)
            );
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}
#[test]
fn unknown_enum_and_noncanonical_bool_remain_visible() {
    let mut p = project(false, false);
    let Variant::Plain(body) = &mut p.physical[0].body.variant else {
        unreachable!()
    };
    body.records[0].fixed[18] = 250;
    body.records[0].fixed[17] = 7;
    let fields = p.record(TableId(0), 0).unwrap();
    assert!(fields.iter().any(|f| matches!(
        f.value,
        Value::Enum {
            index: 250,
            name: None
        }
    )));
    assert!(fields.iter().any(|f| f.name == "b" && !f.editable));
    let old = fields.iter().find(|f| f.name == "e").unwrap().value.clone();
    p.apply(Edit {
        table: TableId(0),
        record: 0,
        field: "e".into(),
        repeat: 0,
        old: old.clone(),
        new: old,
    })
    .unwrap();
    assert!(p.changes().unwrap().is_empty());
}
#[test]
fn reference_index_tracks_key_edits_and_undo() {
    let mut p = project(false, false);
    assert!(matches!(
        p.record(TableId(0), 0)
            .unwrap()
            .into_iter()
            .find(|f| f.name == "link")
            .unwrap()
            .value,
        Value::Ref {
            resolved: Some(_),
            ..
        }
    ));
    let old = p
        .record(TableId(0), 1)
        .unwrap()
        .into_iter()
        .find(|f| f.name == "id")
        .unwrap()
        .value;
    p.apply(Edit {
        table: TableId(0),
        record: 1,
        field: "id".into(),
        repeat: 0,
        old,
        new: Value::Int(3),
    })
    .unwrap();
    assert!(matches!(
        p.record(TableId(0), 0)
            .unwrap()
            .into_iter()
            .find(|f| f.name == "link")
            .unwrap()
            .value,
        Value::Ref { resolved: None, .. }
    ));
    p.undo().unwrap();
    assert!(p.changes().unwrap().is_empty());
}
#[test]
fn malformed_inputs_return_errors() {
    let mut c = container(Width::Bits32, Role::Main, false);
    c.tables[0].body.truncate(3);
    assert!(Project::from_containers(
        vec![c],
        Some(Definitions::from_definitions([definition(Width::Bits32)]).unwrap()),
        None,
        Width::Bits32
    )
    .is_err());
    let p = project(false, false);
    assert!(p.search(TableId(99), "a", 1).is_err());
    assert!(p.record_title(TableId(0), usize::MAX).is_err());
    let mut def = definition(Width::Bits32);
    def.el[0].body.attr[1].offset = usize::MAX;
    let c = container(Width::Bits32, Role::Main, false);
    let p = Project::from_containers(
        vec![c],
        Some(Definitions::from_definitions([def]).unwrap()),
        None,
        Width::Bits32,
    )
    .unwrap();
    assert!(p.record(TableId(0), 0).is_err());
}
#[test]
fn folder_open_both_schemas_and_source_protection() {
    let root = temp();
    let schemas = root.join("schemas");
    let schema = br#"{"name":"synthetic","type":7,"major_ver":1,"minor_ver":0,"autokey":false,"maxid":0,"module":0,"elcount":1,"el":[{"name":"record","type":1,"body":{"size":56,"keyoffset":8,"attrcount":0,"subcount":0,"attr":[]}}]}"#;
    for (width, sub, name) in [
        (Width::Bits32, "x86", "datafile.bin"),
        (Width::Bits64, "x64", "datafile64.bin"),
    ] {
        std::fs::create_dir_all(schemas.join(sub)).unwrap();
        std::fs::write(schemas.join(sub).join("synthetic.json"), schema).unwrap();
        std::fs::write(
            root.join(name),
            container(width, Role::Main, true).write().unwrap(),
        )
        .unwrap();
    }
    let p = Project::open(&root, Width::Bits32, &schemas).unwrap();
    assert_eq!(p.tables().len(), 1);
    assert_eq!(p.search(TableId(0), "2", 10).unwrap(), vec![1]);
    let before = std::fs::read(root.join("datafile.bin")).unwrap();
    assert!(p.pack(&root).is_err());
    assert_eq!(std::fs::read(root.join("datafile.bin")).unwrap(), before);
    assert!(
        Project::open_with_schemas(&root, Width::Bits32, Some(&schemas.join("x86")), None).is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn scalar_integer_storage_matrix_and_seq16() {
    let c = container(Width::Bits32, Role::Main, false);
    let original = TableBody::parse(
        &c.tables[0].body,
        Width::Bits32,
        2,
        Some(&definition(Width::Bits32)),
    )
    .unwrap();
    for (kind, max_size) in [
        ("int8", 1),
        ("int16", 2),
        ("int32", 4),
        ("int64", 8),
        ("msec", 8),
        ("distance", 8),
    ] {
        for size in [1, 2, 4, 8].into_iter().filter(|n| *n <= max_size) {
            let mut def = definition(Width::Bits32);
            def.el[0].body.attr[1].kind = kind.into();
            def.el[0].body.attr[1].size = size;
            let mut body = original.clone();
            let Variant::Plain(b) = &mut body.variant else {
                unreachable!()
            };
            let r = &mut b.records[0];
            let low = match size {
                1 => i64::from(i8::MIN),
                2 => i64::from(i16::MIN),
                4 => i64::from(i32::MIN),
                _ => i64::MIN,
            };
            r.set_scalar(&def, "n", 0, ScalarValue::Int(low)).unwrap();
            if size < 8 {
                assert!(r
                    .set_scalar(&def, "n", 0, ScalarValue::Int(low - 1))
                    .is_err());
            }
        }
    }
    let mut def = definition(Width::Bits32);
    def.el[0].body.attr[3].kind = "seq16".into();
    def.el[0].body.attr[3].size = 2;
    let mut body = original;
    let Variant::Plain(b) = &mut body.variant else {
        unreachable!()
    };
    let r = &mut b.records[0];
    r.set_scalar(&def, "e", 0, ScalarValue::Enum(65535))
        .unwrap();
    assert!(r
        .set_scalar(&def, "e", 0, ScalarValue::Enum(65536))
        .is_err());
}
#[test]
fn missing_target_and_zero_reference_keep_storage() {
    let mut p = project(false, false);
    p.definitions[0].as_mut().unwrap().0.get_mut(&7).unwrap().el[0]
        .body
        .attr[5]
        .reftable = 999;
    let fields = p.record(TableId(0), 0).unwrap();
    let value = &fields.iter().find(|f| f.name == "link").unwrap().value;
    assert_eq!(
        *value,
        Value::Ref {
            table: 999,
            key: 2i64.to_le_bytes().to_vec(),
            resolved: None
        }
    );
    assert!(matches!(
        p.record(TableId(0), 1)
            .unwrap()
            .into_iter()
            .find(|f| f.name == "link")
            .unwrap()
            .value,
        Value::Ref { resolved: None, .. }
    ));
    assert_eq!(
        p.validate(&edit(&p, "link", value.clone())).unwrap()[0].severity,
        Severity::Warning
    );
}
#[test]
fn same_value_pack_nan_bits_and_raw_read_only() {
    let mut p = project(true, true);
    p.apply(edit(&p, "n", Value::Int(2))).unwrap();
    assert!(p.changes().unwrap().is_empty());
    assert!(!p.undo().unwrap());
    let dir = temp();
    let report = p.pack(&dir).unwrap();
    for (file, source) in report.files.iter().zip(&p.sources) {
        assert!(file.changed_tables.is_empty());
        assert_eq!(
            std::fs::read(&file.path).unwrap(),
            source.container.write().unwrap()
        );
    }
    std::fs::remove_dir_all(dir).unwrap();
    assert_ne!(Value::Float(0.0), Value::Float(-0.0));
    let nan = Value::Float(f32::from_bits(0x7fc00042));
    assert_eq!(nan, nan.clone());
    let mut p = project(false, false);
    let Variant::Plain(body) = &mut p.physical[0].body.variant else {
        unreachable!()
    };
    body.records[0].fixed[17] = 7;
    assert!(p.apply(edit(&p, "b", Value::Bool(true))).is_err());
}
#[test]
fn preferred_64_view_mirrors_and_roles_stay_separate() {
    let mut containers = Vec::new();
    for width in [Width::Bits32, Width::Bits64] {
        for role in [Role::Main, Role::Local] {
            containers.push(container(width, role, width == Width::Bits64));
        }
    }
    let mut p = Project::from_containers(
        containers,
        Some(Definitions::from_definitions([definition(Width::Bits32)]).unwrap()),
        Some(Definitions::from_definitions([definition(Width::Bits64)]).unwrap()),
        Width::Bits64,
    )
    .unwrap();
    assert!(p.tables().iter().all(|t| t.width == Width::Bits64));
    p.apply(edit(&p, "b", Value::Bool(false))).unwrap();
    for (physical, body) in p.physical.iter().enumerate() {
        let role = p.sources[body.file].container.role;
        let value = p
            .current_fields(physical, 0)
            .unwrap()
            .into_iter()
            .find(|f| f.name == "b")
            .unwrap()
            .value;
        assert_eq!(value, Value::Bool(role == Role::Local));
    }
    let dir = temp();
    let report = p.pack(&dir).unwrap();
    assert_eq!(
        report
            .files
            .iter()
            .filter(|f| !f.changed_tables.is_empty())
            .count(),
        2
    );
    std::fs::remove_dir_all(dir).unwrap();
}
