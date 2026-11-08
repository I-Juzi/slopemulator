//! Opt-in, byte-only table survey. Writes schema names/counts, not game records.
use lmxb::{Element, Node};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
};
#[derive(Default)]
struct Field {
    n: usize,
    int: usize,
    float: usize,
    boolean: usize,
    values: BTreeSet<String>,
    overflow: bool,
}
impl Field {
    fn add(&mut self, v: &str) {
        self.n += 1;
        self.int += usize::from(v.parse::<i128>().is_ok());
        self.float += usize::from(v.parse::<f64>().is_ok_and(f64::is_finite));
        self.boolean += usize::from(matches!(v, "true" | "false"));
        if !self.overflow {
            self.values.insert(v.into());
            if self.values.len() > 20 {
                self.values.clear();
                self.overflow = true;
            }
        }
    }
    fn kind(&self) -> &str {
        if self.boolean == self.n {
            "bool"
        } else if self.int == self.n {
            "int"
        } else if self.float == self.n {
            "float"
        } else if !self.overflow
            && self
                .values
                .iter()
                .all(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_lowercase() || c == '-'))
        {
            "enum-like string ❓"
        } else {
            "string"
        }
    }
}
#[derive(Default)]
struct Table {
    files: usize,
    records: usize,
    metadata: BTreeMap<String, Field>,
    fields: BTreeMap<(String, String), Field>,
    ids: BTreeSet<String>,
    aliases: BTreeSet<String>,
    refs: BTreeMap<String, BTreeMap<String, usize>>,
}
fn attr<'a>(e: &'a Element, key: &str) -> Option<&'a str> {
    e.attributes
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}
fn scan(e: &Element, path: &str, t: &mut Table) {
    for (k, v) in &e.attributes {
        t.fields.entry((path.into(), k.clone())).or_default().add(v);
        if !path.contains('/') {
            if k == "id" {
                t.ids.insert(v.clone());
            }
            if k == "alias" {
                t.aliases.insert(v.clone());
            }
        }
        if k != "id" && k != "alias" {
            *t.refs
                .entry(k.clone())
                .or_default()
                .entry(v.clone())
                .or_default() += 1;
        }
    }
    for child in &e.children {
        if let Node::Element(c) = child {
            scan(c, &format!("{path}/{}", c.name), t);
        }
    }
}
fn walk(p: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let m = fs::symlink_metadata(p)?;
    if m.file_type().is_symlink() {
        return Ok(());
    }
    if m.is_dir() {
        for e in fs::read_dir(p)? {
            walk(&e?.path(), out)?;
        }
    } else if p.extension().is_some_and(|e| e == "xml") {
        out.push(p.into());
    }
    Ok(())
}
fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(root) = env::var_os("BNS_GAMEDATA") else {
        println!("SKIP: set BNS_GAMEDATA");
        return Ok(());
    };
    let args: Vec<_> = env::args().skip(1).collect();
    let mut paths = Vec::new();
    walk(Path::new(&root), &mut paths)?;
    paths.sort();
    let mut tables: BTreeMap<String, Table> = BTreeMap::new();
    let mut plain = 0;
    let mut non_table = BTreeMap::<String, usize>::new();
    for p in &paths {
        let b = fs::read(p)?;
        if !b.starts_with(lmxb::MAGIC) {
            plain += 1;
            continue;
        }
        let d = lmxb::decode(&b)?;
        if d.root.name != "table" {
            *non_table.entry(d.root.name).or_default() += 1;
            continue;
        }
        let name = attr(&d.root, "type").unwrap_or("<missing>").to_string();
        if args.first().is_some_and(|a| a == "--focus")
            && [
                "zone",
                "Terrain",
                "ZoneEnv",
                "ZoneEnv2",
                "ZoneEnv2Place",
                "ZoneEnv2Spawn",
                "ZonePcSpawn",
                "PcInitialSetting",
                "script",
            ]
            .contains(&name.as_str())
        {
            // Only the first file of each type, and at most three records, reaches stdout.
            if !tables.contains_key(&name) {
                println!(
                    "FOCUS {name} {} {:?}",
                    p.strip_prefix(Path::new(&root))?.display(),
                    d.root.attributes
                );
                for n in d
                    .root
                    .children
                    .iter()
                    .filter_map(|n| {
                        if let Node::Element(e) = n {
                            Some(e)
                        } else {
                            None
                        }
                    })
                    .take(3)
                {
                    println!("RECORD {} {:?}", n.name, n.attributes);
                    for c in &n.children {
                        if let Node::Element(e) = c {
                            println!("CHILD {} {:?}", e.name, e.attributes);
                        }
                    }
                }
            }
        }
        let t = tables.entry(name).or_default();
        t.files += 1;
        for (k, v) in &d.root.attributes {
            t.metadata.entry(k.clone()).or_default().add(v);
        }
        for n in &d.root.children {
            if let Node::Element(e) = n {
                t.records += 1;
                scan(e, &e.name, t);
            }
        }
    }
    let mut md = String::from("# GameDaemon table survey: 2020 data\n\nSource: decoded LMXBOSLB bytes from the local 2020 GameDaemon data folder. No executable or external code was used. Counts describe the 2026-11-08 snapshot. Records are direct element children of `table`. Attribute paths include nested elements.\n\nTypes are lexical inferences, not a server schema. `bool` means literal true/false. Numeric 0/1 stays int. Enum-like sets contain at most 20 lowercase/hyphen names; they can still be aliases. ❓ Reference targets and optional/default semantics require further checks.\n\n");
    md += &format!("XML paths: {}. Binary table types: {}. Nonbinary files skipped: {plain}. Other binary root names/counts: {non_table:?}.\n\n",paths.len(),tables.len());
    md += "## Table counts and root attribute names\n\n| Table type | Files | Records | Root attribute names |\n| --- | ---: | ---: | --- |\n";
    for (name, t) in &tables {
        md += &format!(
            "| {name} | {} | {} | {} |\n",
            t.files,
            t.records,
            t.metadata.keys().cloned().collect::<Vec<_>>().join(", ")
        );
    }
    md += "\n## Root metadata sets\n\nThese are enum-like names or numeric metadata counts. Version entries show the number of distinct versions, not their values.\n\n| Table | release-module names | release-side names | Distinct versions |\n| --- | --- | --- | ---: |\n";
    for (name, t) in &tables {
        let names = |k: &str| {
            t.metadata
                .get(k)
                .map(|f| {
                    if f.overflow {
                        ">20 names".into()
                    } else {
                        f.values.iter().cloned().collect::<Vec<_>>().join(", ")
                    }
                })
                .unwrap_or_default()
        };
        let versions = t
            .metadata
            .get("version")
            .map(|f| {
                if f.overflow {
                    ">20".into()
                } else {
                    f.values.len().to_string()
                }
            })
            .unwrap_or("0".into());
        md += &format!(
            "| {name} | {} | {} | {versions} |\n",
            names("release-module"),
            names("release-side")
        );
    }
    md += "\n## Attribute names and inferred types\n";
    for (name, t) in &tables {
        md += &format!("\n### {name}\n\n| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |\n| --- | --- | ---: | --- | --- |\n");
        for ((path, k), f) in &t.fields {
            let names = if f.kind().starts_with("enum-like") {
                f.values.iter().cloned().collect::<Vec<_>>().join(", ")
            } else {
                String::new()
            };
            md += &format!("| {path} | {k} | {} | {} | {names} |\n", f.n, f.kind());
        }
    }
    md += "\n## Cross-file reference candidates\n\n❓ These are byte-derived candidates, not confirmed runtime links. Normalize an attribute name by removing nonletters; match a table name, with optional id/alias suffix. Count nonempty occurrences, and exact id/alias matches across all files of that target table. Zero and empty defaults are not resolved here. Other fields can contain references that this name rule misses.\n\n| Source table | Attribute | Candidate target | Nonempty occurrences | Id matches | Alias matches |\n| --- | --- | --- | ---: | ---: | ---: |\n";
    for (name, t) in &tables {
        for (k, vs) in &t.refs {
            let norm = normalize(k);
            for (target, tt) in &tables {
                let tn = normalize(target);
                if norm == tn || norm == format!("{tn}id") || norm == format!("{tn}alias") {
                    let count: usize = vs
                        .iter()
                        .filter(|(v, _)| !v.is_empty())
                        .map(|(_, n)| n)
                        .sum();
                    let ids: usize = vs
                        .iter()
                        .filter(|(v, _)| tt.ids.contains(*v))
                        .map(|(_, n)| n)
                        .sum();
                    let aliases: usize = vs
                        .iter()
                        .filter(|(v, _)| tt.aliases.contains(*v))
                        .map(|(_, n)| n)
                        .sum();
                    md += &format!("| {name} | {k} | {target} | {count} | {ids} | {aliases} |\n");
                }
            }
        }
    }
    if let Some(i) = args.iter().position(|a| a == "--out") {
        let out = Path::new(args.get(i + 1).ok_or("--out needs a path")?);
        fs::write(out, md)?;
    }
    println!(
        "SUMMARY paths={} tables={} files={} records={} nonbinary={plain} other={non_table:?}",
        paths.len(),
        tables.len(),
        tables.values().map(|t| t.files).sum::<usize>(),
        tables.values().map(|t| t.records).sum::<usize>()
    );
    Ok(())
}
