use gamedata::{Data, TableKind, World};
use std::collections::BTreeMap;
#[test]
fn minimum_world_from_real_folder() {
    let Some(root) = std::env::var_os("BNS_GAMEDATA") else {
        println!("SKIP: set BNS_GAMEDATA to read real GameDaemon data");
        return;
    };
    let w = World::load_dir(root).expect("load minimum-world tables");
    assert_eq!(w.tables.len(), 12, "expected all selected table types");
    for (k, t) in &w.tables {
        println!(
            "TABLE {} files={} records={}",
            k.name(),
            t.files.len(),
            t.records.len()
        );
    }
    println!(
        "SKIPPED plain={} tables={} roots={}",
        w.skipped_plain, w.skipped_tables, w.skipped_roots
    );
    let dangling = w.dangling_references();
    let mut groups = BTreeMap::new();
    for d in &dangling {
        *groups
            .entry((d.source_table, d.attribute.clone(), d.target))
            .or_insert(0usize) += 1;
    }
    for ((from, attr, to), n) in groups {
        println!("DANGLING {}.{attr} -> {} count={n}", from.name(), to.name());
    }
    println!("DANGLING_TOTAL {}", dangling.len());
    let mut starts = BTreeMap::new();
    for r in &w.table(TableKind::Initial).unwrap().records {
        if let Data::Initial(i) = &r.data {
            if i.level == 1 {
                let zone = w
                    .table(TableKind::Zone)
                    .unwrap()
                    .by_alias(None, &i.zone)
                    .expect("level-one start zone");
                if let Data::Zone(z) = &zone.data {
                    *starts.entry((zone.id, z.terrain)).or_insert(0usize) += 1;
                }
            }
        }
    }
    assert!(!starts.is_empty());
    println!("LEVEL_ONE_ZONE_TERRAIN_COUNTS {starts:?}");
}
