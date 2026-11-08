//! Opt-in byte-derived reference diagnostics. Does not write game records.
use gamedata::{Data, TableKind, World};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(root) = std::env::var_os("BNS_GAMEDATA") else {
        println!("SKIP: set BNS_GAMEDATA");
        return Ok(());
    };
    let w = World::load_dir(root)?;
    let zones = w.table(TableKind::Zone).ok_or("no zones")?;
    for r in zones
        .records
        .iter()
        .filter(|r| matches!(&r.data,Data::Zone(z) if z.reentry.is_some()))
        .take(5)
    {
        if let Data::Zone(z) = &r.data {
            println!(
                "ZONE {:?} reentry={:?} respawn={:?}",
                r.id, z.reentry, z.respawn
            );
        }
    }
    let places = w.table(TableKind::EnvPlace).ok_or("no places")?;
    let mut missing_scope = 0;
    let mut present_elsewhere = 0;
    for r in &w.table(TableKind::EnvSpawn).ok_or("no spawns")?.records {
        if let Data::EnvSpawn(e) = &r.data {
            if let Some(a) = &e.place {
                if places.by_alias(r.zone, a).is_none() {
                    missing_scope += 1;
                    present_elsewhere +=
                        usize::from(places.records.iter().any(|r| r.alias.as_ref() == Some(a)));
                }
            }
        }
    }
    println!("PLACE missing_in_scope={missing_scope} present_elsewhere={present_elsewhere}");
    Ok(())
}
