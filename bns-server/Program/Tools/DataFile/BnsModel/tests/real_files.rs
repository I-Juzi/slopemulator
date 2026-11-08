//! Operator-only check. Read source data only. Print counts, never content.
use bnsdata::{
    dat::{Container, Role, Width},
    tabledef::Definitions,
};
use bnsmodel::{Edit, Project};
#[test]
#[ignore = "requires BNSDATA_CLIENT_DAT, BNSDATA_DAT_WIDTH, BNSDATA_DAT_ROLE, BNSDATA_TABLEDEFS"]
fn model_open_search_pack() {
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
    let source = std::env::var_os("BNSDATA_CLIENT_DAT").expect("set BNSDATA_CLIENT_DAT");
    let original = std::fs::read(source).expect("read source");
    let definitions = Definitions::load_folder(
        std::env::var_os("BNSDATA_TABLEDEFS").expect("set BNSDATA_TABLEDEFS"),
    )
    .expect("load definitions");
    let container = Container::parse(&original, width, role).expect("parse container");
    let (x86, x64) = if width == Width::Bits32 {
        (Some(definitions), None)
    } else {
        (None, Some(definitions))
    };
    let mut project =
        Project::from_containers(vec![container], x86, x64, width).expect("open project");
    let tables = project.tables();
    let largest = tables
        .iter()
        .max_by_key(|t| t.record_count)
        .expect("at least one table");
    let hits = project.search(largest.id, "a", 10).expect("search");
    let mut edited = false;
    'tables: for table in &tables {
        if table.record_count == 0 {
            continue;
        }
        for field in project.record(table.id, 0).expect("decode record") {
            if field.editable {
                project
                    .apply(Edit {
                        table: table.id,
                        record: 0,
                        field: field.name,
                        repeat: field.repeat_index,
                        old: field.value.clone(),
                        new: field.value,
                    })
                    .expect("same-value edit");
                edited = true;
                break 'tables;
            }
        }
    }
    assert!(edited, "no editable field");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let out = std::env::temp_dir().join(format!("bnsmodel-real-{}-{stamp}", std::process::id()));
    let report = project
        .pack(&out)
        .expect("pack to separate temporary folder");
    assert!(report.files.len() == 1, "expected one output");
    let written = std::fs::read(&report.files[0].path).expect("read output");
    assert!(written == original, "packed bytes differ");
    println!(
        "tables={} hits={} edits={} files={} bytes={}",
        tables.len(),
        hits.len(),
        usize::from(edited),
        report.files.len(),
        report.bytes
    );
    std::fs::remove_dir_all(out).expect("remove temporary output");
}
