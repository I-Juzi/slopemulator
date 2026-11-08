//! Operator-only checks. These tests never write files or print their contents.
use bnsdata::{
    dat::{Container, Role, Width},
    lmx,
    table::{decode_fields_with_pool, RawValue, TableBody, Variant},
    tabledef::Definitions,
};

#[test]
#[ignore = "requires BNSDATA_SERVER_XML and BNSDATA_KEY_FILE; reads operator game data"]
fn server_roundtrip() {
    let key = lmx::load_key().expect("BNSDATA_KEY_FILE must select a valid LMX key");
    let path = std::env::var_os("BNSDATA_SERVER_XML").expect("set BNSDATA_SERVER_XML");
    let bytes = std::fs::read(path).expect("read server XML, read-only");
    let counts = lmx::walk(&bytes).expect("walk server XML");
    let parsed = lmx::Document::parse(&bytes, &key).expect("parse server XML");
    let written = parsed.write(&key).expect("serialize server XML in memory");
    // Do not use assert_eq!: a failure must not dump game bytes.
    assert!(bytes == written, "LMX byte round trip differs");
    println!(
        "elements={} text_nodes={} attributes={} bytes={}",
        counts.elements, counts.text_nodes, counts.attributes, counts.binary_size
    );
}

#[test]
#[ignore = "requires BNSDATA_CLIENT_DAT, BNSDATA_DAT_WIDTH and BNSDATA_DAT_ROLE; reads operator game data"]
fn client_roundtrip() {
    let width = match std::env::var("BNSDATA_DAT_WIDTH").as_deref() {
        Ok("32") => Width::Bits32,
        Ok("64") => Width::Bits64,
        _ => panic!("set BNSDATA_DAT_WIDTH to 32 or 64"),
    };
    let role = match std::env::var("BNSDATA_DAT_ROLE").as_deref() {
        Ok("main") => Role::Main,
        Ok("local") => Role::Local,
        _ => panic!("set BNSDATA_DAT_ROLE to main or local"),
    };
    let path = std::env::var_os("BNSDATA_CLIENT_DAT").expect("set BNSDATA_CLIENT_DAT");
    let bytes = std::fs::read(path).expect("read client DAT, read-only");
    let parsed = Container::parse(&bytes, width, role).expect("parse client DAT");
    let written = parsed.write().expect("serialize client DAT in memory");
    assert!(bytes == written, "DAT byte round trip differs");
    println!(
        "tables={} alias_bytes={} bytes={}",
        parsed.tables.len(),
        parsed.aliases.as_ref().map_or(0, Vec::len),
        bytes.len()
    );
}

#[test]
#[ignore = "requires BNSDATA_CLIENT_DAT, BNSDATA_DAT_WIDTH, BNSDATA_DAT_ROLE and BNSDATA_TABLEDEFS; reads operator data"]
fn client_table_bodies_roundtrip() {
    let width = match std::env::var("BNSDATA_DAT_WIDTH").as_deref() {
        Ok("32") => Width::Bits32,
        Ok("64") => Width::Bits64,
        _ => panic!("set BNSDATA_DAT_WIDTH to 32 or 64"),
    };
    let role = match std::env::var("BNSDATA_DAT_ROLE").as_deref() {
        Ok("main") => Role::Main,
        Ok("local") => Role::Local,
        _ => panic!("set BNSDATA_DAT_ROLE to main or local"),
    };
    let definitions = Definitions::load_folder(
        std::env::var_os("BNSDATA_TABLEDEFS").expect("set BNSDATA_TABLEDEFS"),
    )
    .expect("load width-specific definitions");
    let bytes =
        std::fs::read(std::env::var_os("BNSDATA_CLIENT_DAT").expect("set BNSDATA_CLIENT_DAT"))
            .expect("read client DAT, read-only");
    let mut container = Container::parse(&bytes, width, role).expect("parse client DAT");
    let mut plain = 0usize;
    let mut compressed = 0usize;
    let mut records = 0usize;
    let mut blocks = 0usize;
    for table in &mut container.tables {
        let definition = definitions
            .get(table.type_id)
            .expect("find table definition");
        assert!(
            definition.major_ver == table.major && definition.minor_ver == table.minor,
            "table definition version mismatch"
        );
        assert!(
            definition.autokey == (table.kind == 1),
            "table definition kind mismatch"
        );
        let parsed = TableBody::parse(&table.body, width, table.kind, Some(definition))
            .expect("parse table and validate every DrEl size");
        records += parsed.record_count();
        match &parsed.variant {
            Variant::Plain(_) => plain += 1,
            Variant::Compressed(body) => {
                compressed += 1;
                blocks += body.blocks.len();
            }
        }
        let written = parsed
            .write(width, table.kind, Some(definition))
            .expect("write table in memory");
        // Do not use assert_eq!: failures must never dump game data.
        assert!(table.body == written, "table body byte round trip differs");
        table.body = written;
    }
    let written = container.write().expect("write container in memory");
    assert!(bytes == written, "container byte round trip differs");
    println!(
        "tables={} plain={} compressed={} blocks={} records={} bytes={}",
        container.tables.len(),
        plain,
        compressed,
        blocks,
        records,
        bytes.len()
    );
}

#[test]
#[ignore = "requires BNSDATA_CLIENT_DAT, BNSDATA_DAT_WIDTH, BNSDATA_DAT_ROLE and BNSDATA_TABLEDEFS; reads operator data"]
fn client_strings_resolve() {
    use bnsdata::{
        table::{Record, RecordHeader},
        tabledef::TableDefinition,
    };

    // A bad slot must not hide the remaining slots. Retry individually on failure.
    fn resolve_record(
        record: &Record,
        pool: &[u8],
        definition: &TableDefinition,
        width: Width,
        successes: &mut usize,
        failures: &mut usize,
    ) -> Option<(String, usize, Vec<u16>)> {
        if let Ok(fields) = decode_fields_with_pool(record, definition, pool, width) {
            let mut first = None;
            for field in fields {
                match field.value {
                    RawValue::Text { units, .. } => {
                        *successes += 1;
                        if first.is_none() {
                            first = Some((field.name, field.repeat_index, units));
                        }
                    }
                    RawValue::Native { .. } => *successes += 1,
                    _ => {}
                }
            }
            return first;
        }
        let header = RecordHeader::parse(&record.fixed).expect("read checked record header");
        let layout = definition
            .layout(header.element_type, header.subtype)
            .expect("find checked layout");
        let mut standalone = record.clone();
        standalone.fixed[2..4].copy_from_slice(&(-1i16).to_le_bytes());
        let failures_before = *failures;
        for field in layout
            .fields()
            .filter(|f| matches!(f.kind.as_str(), "string" | "native"))
        {
            for index in 0..field.repeat {
                let Some(offset) = index
                    .checked_mul(field.size)
                    .and_then(|n| field.offset.checked_add(n))
                else {
                    *failures += 1;
                    continue;
                };
                let mut single = definition.clone();
                let element = single
                    .el
                    .iter_mut()
                    .find(|e| e.type_id == header.element_type)
                    .expect("find checked element");
                let mut attribute = field.clone();
                attribute.offset = offset;
                attribute.repeat = 1;
                element.body.size = record.fixed.len();
                element.body.attr = vec![attribute];
                element.body.attrcount = 1;
                if decode_fields_with_pool(&standalone, &single, pool, width).is_ok() {
                    *successes += 1;
                } else {
                    *failures += 1;
                }
            }
        }
        // Count a non-text/schema failure too, but do not double-count text failures.
        if *failures == failures_before {
            *failures += 1;
        }
        None
    }

    let width = match std::env::var("BNSDATA_DAT_WIDTH").as_deref() {
        Ok("32") => Width::Bits32,
        Ok("64") => Width::Bits64,
        _ => panic!("set BNSDATA_DAT_WIDTH to 32 or 64"),
    };
    let role = match std::env::var("BNSDATA_DAT_ROLE").as_deref() {
        Ok("main") => Role::Main,
        Ok("local") => Role::Local,
        _ => panic!("set BNSDATA_DAT_ROLE to main or local"),
    };
    let definitions = Definitions::load_folder(
        std::env::var_os("BNSDATA_TABLEDEFS").expect("set BNSDATA_TABLEDEFS"),
    )
    .expect("load width-specific definitions");
    let bytes =
        std::fs::read(std::env::var_os("BNSDATA_CLIENT_DAT").expect("set BNSDATA_CLIENT_DAT"))
            .expect("read client DAT, read-only");
    let container = Container::parse(&bytes, width, role).expect("parse client DAT");
    let mut successes = 0usize;
    let mut failures = 0usize;
    let mut records = 0usize;
    let mut checked_plain = false;
    let mut checked_compressed = false;
    let mut noop_tables = 0usize;
    for table in &container.tables {
        let definition = definitions
            .get(table.type_id)
            .expect("find table definition");
        assert!(
            definition.major_ver == table.major && definition.minor_ver == table.minor,
            "table definition version mismatch"
        );
        assert!(
            definition.autokey == (table.kind == 1),
            "table definition kind mismatch"
        );
        let mut parsed = TableBody::parse(&table.body, width, table.kind, Some(definition))
            .expect("parse table");
        let mut candidate = None;
        let is_compressed = parsed.compressed;
        let already_checked = if is_compressed {
            checked_compressed
        } else {
            checked_plain
        };
        let spans: Box<dyn Iterator<Item = (&Record, &[u8])> + '_> = match &parsed.variant {
            Variant::Plain(body) => {
                Box::new(body.records.iter().map(|r| (r, body.extra.as_slice())))
            }
            Variant::Compressed(body) => Box::new(
                body.blocks
                    .iter()
                    .flat_map(|b| &b.records)
                    .map(|r| (r, r.extra.as_slice())),
            ),
        };
        for (index, (record, pool)) in spans.enumerate() {
            records += 1;
            let first = resolve_record(
                record,
                pool,
                definition,
                width,
                &mut successes,
                &mut failures,
            );
            if !already_checked && candidate.is_none() {
                candidate = first.map(|(name, repeat, units)| (index, name, repeat, units));
            }
        }
        if let Some((index, name, repeat, units)) = candidate {
            parsed
                .edit_record(index, definition, width)
                .expect("open text editor")
                .set_text(&name, repeat, &units)
                .expect("set identical text");
            let written = parsed
                .write(width, table.kind, Some(definition))
                .expect("write no-op edit in memory");
            assert!(
                written == table.body,
                "same-value text edit changed table bytes"
            );
            if is_compressed {
                checked_compressed = true;
            } else {
                checked_plain = true;
            }
            noop_tables += 1;
        }
    }
    println!(
        "tables={} records={} resolved_fields={} failures={} noop_tables={}",
        container.tables.len(),
        records,
        successes,
        failures,
        noop_tables
    );
    assert!(failures == 0, "text resolution failed");
    assert!(
        checked_plain && checked_compressed,
        "missing plain or compressed string no-op check"
    );
}
