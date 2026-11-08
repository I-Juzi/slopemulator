use lmxb::{decode, element_count, encode, to_xml_string, MAGIC};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};
fn collect(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        // Do not follow symlinks from a data folder.
        let kind = entry.file_type()?;
        if kind.is_dir() {
            collect(&entry.path(), out)?;
        } else if kind.is_file() && entry.path().extension().is_some_and(|x| x == "xml") {
            out.push(entry.path());
        }
    }
    Ok(())
}
#[test]
fn opt_in_real_data_round_trip() {
    let Some(folder) = env::var_os("BNS_GAMEDATA") else {
        println!("SKIP: BNS_GAMEDATA is not set");
        return;
    };
    let folder = Path::new(&folder);
    let mut paths = Vec::new();
    collect(folder, &mut paths).expect("read BNS_GAMEDATA");
    paths.sort();
    assert!(!paths.is_empty(), "BNS_GAMEDATA contains no XML files");
    let mut decoded = 0u64;
    let mut elements = 0u64;
    let mut bytes = 0u64;
    let mut failures = 0u64;
    let mut round_trip_failures = 0u64;
    let mut xml_failures = 0u64;
    let mut nonbinary = 0u64;
    let mut groups: BTreeMap<String, u64> = BTreeMap::new();
    for path in &paths {
        let b = fs::read(path).expect("read data file");
        if !b.starts_with(MAGIC) {
            let plain = b.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&b);
            if plain.starts_with(b"<?xml ") && std::str::from_utf8(plain).is_ok() {
                nonbinary += 1;
                println!(
                    "SKIP plain UTF-8 XML: {}",
                    path.strip_prefix(folder).unwrap().display()
                );
            } else {
                failures += 1;
                println!("DECODE FAIL {}: unknown file magic", path.display());
            }
            continue;
        }
        match decode(&b) {
            Ok(doc) => {
                decoded += 1;
                bytes += b.len() as u64;
                elements += element_count(&doc) as u64;
                let relative = path.strip_prefix(folder).unwrap();
                let group = if relative.components().count() == 1 {
                    "(root)".into()
                } else {
                    relative
                        .components()
                        .next()
                        .unwrap()
                        .as_os_str()
                        .to_string_lossy()
                        .into_owned()
                };
                *groups.entry(group).or_default() += 1;
                if encode(&doc) != b {
                    round_trip_failures += 1;
                    println!("ROUND TRIP FAIL: {}", relative.display());
                }
                if let Err(e) = to_xml_string(&doc) {
                    xml_failures += 1;
                    if xml_failures <= 20 {
                        println!("XML FAIL {}: {e}", relative.display());
                    }
                }
            }
            Err(e) => {
                failures += 1;
                if failures <= 20 {
                    println!("DECODE FAIL {}: {e}", path.display());
                }
            }
        }
    }
    println!("REAL DATA: paths={} decoded={decoded} bytes={bytes} elements={elements} decode_failures={failures} round_trip_failures={round_trip_failures} xml_failures={xml_failures} nonbinary={nonbinary} groups={groups:?}",paths.len());
    assert!(decoded > 0, "no binary files tested");
    assert_eq!((failures, round_trip_failures, xml_failures), (0, 0, 0));
}
