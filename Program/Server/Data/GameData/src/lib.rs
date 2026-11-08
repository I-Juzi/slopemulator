//! Byte-derived minimum-world tables. See spec/gamed/03-minimum-world.md.
//! This crate does not execute scripts or choose an initial player spawn.
use lmxb::{Document, Element, Node};
use std::{
    collections::BTreeMap,
    fmt, fs,
    path::{Path, PathBuf},
    str::FromStr,
};

pub type Attributes = BTreeMap<String, String>;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TableKind {
    Initial,
    Zone,
    Terrain,
    PcSpawn,
    Respawn,
    Area,
    Env,
    Env2,
    EnvPlace,
    EnvSpawn,
    Script,
    Quest,
}
impl TableKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Initial => "PcInitialSetting",
            Self::Zone => "zone",
            Self::Terrain => "Terrain",
            Self::PcSpawn => "ZonePcSpawn",
            Self::Respawn => "ZoneRespawn",
            Self::Area => "ZoneArea",
            Self::Env => "ZoneEnv",
            Self::Env2 => "ZoneEnv2",
            Self::EnvPlace => "ZoneEnv2Place",
            Self::EnvSpawn => "ZoneEnv2Spawn",
            Self::Script => "script",
            Self::Quest => "quest",
        }
    }
    pub fn from_name(s: &str) -> Option<Self> {
        [
            Self::Initial,
            Self::Zone,
            Self::Terrain,
            Self::PcSpawn,
            Self::Respawn,
            Self::Area,
            Self::Env,
            Self::Env2,
            Self::EnvPlace,
            Self::EnvSpawn,
            Self::Script,
            Self::Quest,
        ]
        .into_iter()
        .find(|k| k.name() == s)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl FromStr for Position {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: Vec<_> = s
            .split(',')
            .map(|x| x.trim().parse::<f64>())
            .collect::<Result<_, _>>()
            .map_err(|_| "invalid position")?;
        if v.len() != 3 || v.iter().any(|x| !x.is_finite()) {
            return Err("position needs three finite components");
        }
        Ok(Self {
            x: v[0],
            y: v[1],
            z: v[2],
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AreaRef {
    pub zone: u64,
    pub id: u64,
}
impl FromStr for AreaRef {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (z, i) = s.split_once('.').ok_or("area needs zone.id")?;
        Ok(Self {
            zone: z.parse().map_err(|_| "invalid area zone")?,
            id: i.parse().map_err(|_| "invalid area id")?,
        })
    }
}
#[derive(Clone, Debug)]
pub struct Initial {
    pub race: String,
    pub job: String,
    pub level: u32,
    pub inventory_size: u32,
    pub zone: String,
    pub quest: Option<String>,
}
#[derive(Clone, Debug)]
pub struct Zone {
    pub terrain: u64,
    pub sectors: [i64; 4],
    pub tutorial: bool,
    pub default_spawn: Option<u64>,
    pub reentry: Option<String>,
    pub respawn: Option<String>,
    pub script: Option<String>,
    pub phase_zones: BTreeMap<String, String>,
}
#[derive(Clone, Debug)]
pub struct Terrain {
    pub start: [i64; 2],
    pub origin: [i64; 2],
    pub count: [u64; 2],
    pub umap: String,
}
#[derive(Clone, Debug)]
pub struct PcSpawn {
    pub pos: Position,
    pub yaw: i64,
    pub summon_pos: Option<Position>,
}
#[derive(Clone, Debug)]
pub struct Environment {
    pub kind: String,
    pub action_point: Option<Position>,
    pub script: Option<String>,
    pub transit_zone: Option<u64>,
    pub transit_spawn: Option<u64>,
}
#[derive(Clone, Debug)]
pub struct EnvSpawn {
    pub environment: Option<String>,
    pub place: Option<String>,
}
#[derive(Clone, Debug)]
pub enum Data {
    Initial(Initial),
    Zone(Zone),
    Terrain(Terrain),
    PcSpawn(PcSpawn),
    Respawn(AreaRef),
    Area,
    Env(Environment),
    Env2(Environment),
    EnvPlace(Environment),
    EnvSpawn(EnvSpawn),
    Script,
    Quest,
}
#[derive(Clone, Debug)]
pub struct Record {
    pub id: Option<u64>,
    pub alias: Option<String>,
    pub zone: Option<u64>,
    pub source: PathBuf,
    pub source_line: u32,
    pub data: Data,
    pub unknown: Attributes,
    pub children: Vec<Node>,
}
#[derive(Clone, Debug)]
pub struct TableFile {
    pub path: PathBuf,
    pub source_name: String,
    pub metadata: Attributes,
    pub records: usize,
    pub extra_nodes: Vec<Node>,
}
#[derive(Clone, Debug, Default)]
pub struct Table {
    pub files: Vec<TableFile>,
    pub records: Vec<Record>,
    by_id: BTreeMap<(Option<u64>, u64), usize>,
    by_alias: BTreeMap<(Option<u64>, String), usize>,
    initial: BTreeMap<(String, String, u32), usize>,
}
impl Table {
    pub fn by_id(&self, zone: Option<u64>, id: u64) -> Option<&Record> {
        self.by_id.get(&(zone, id)).map(|&i| &self.records[i])
    }
    pub fn by_alias(&self, zone: Option<u64>, alias: &str) -> Option<&Record> {
        self.by_alias
            .get(&(zone, alias.to_owned()))
            .map(|&i| &self.records[i])
    }
    /// Resolve an alias only if it identifies exactly one record across scopes.
    pub fn by_alias_any(&self, alias: &str) -> Option<&Record> {
        let mut matches = self
            .by_alias
            .iter()
            .filter(|((_, a), _)| a == alias)
            .map(|(_, &i)| &self.records[i]);
        let first = matches.next()?;
        if matches.next().is_some() {
            None
        } else {
            Some(first)
        }
    }
    pub fn initial(&self, race: &str, job: &str, level: u32) -> Option<&Record> {
        self.initial
            .get(&(race.into(), job.into(), level))
            .map(|&i| &self.records[i])
    }
    fn insert(&mut self, r: Record) -> Result<(), Error> {
        let fail = |message| Error {
            path: r.source.clone(),
            line: Some(r.source_line),
            message,
        };
        if let Some(id) = r.id {
            if self.by_id.contains_key(&(r.zone, id)) {
                return Err(fail(format!("duplicate id in scope: {id}")));
            }
        }
        if let Some(a) = &r.alias {
            if self.by_alias.contains_key(&(r.zone, a.clone())) {
                return Err(fail(format!("duplicate alias in scope: {a}")));
            }
        }
        if let Data::Initial(i) = &r.data {
            if self
                .initial
                .contains_key(&(i.race.clone(), i.job.clone(), i.level))
            {
                return Err(fail("duplicate initial setting".into()));
            }
        }
        let n = self.records.len();
        if let Some(id) = r.id {
            self.by_id.insert((r.zone, id), n);
        }
        if let Some(a) = &r.alias {
            self.by_alias.insert((r.zone, a.clone()), n);
        }
        if let Data::Initial(i) = &r.data {
            self.initial
                .insert((i.race.clone(), i.job.clone(), i.level), n);
        }
        self.records.push(r);
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct Error {
    pub path: PathBuf,
    pub line: Option<u32>,
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.path.display(),
            self.line.unwrap_or(0),
            self.message
        )
    }
}
impl std::error::Error for Error {}
fn error(path: &Path, message: impl ToString) -> Error {
    Error {
        path: path.into(),
        line: None,
        message: message.to_string(),
    }
}
fn attributes(e: &Element) -> Result<Attributes, String> {
    let mut a = Attributes::new();
    for (k, v) in &e.attributes {
        if a.insert(k.clone(), v.clone()).is_some() {
            return Err(format!("duplicate attribute: {k}"));
        }
    }
    Ok(a)
}
fn required(a: &mut Attributes, k: &str) -> Result<String, String> {
    a.remove(k).ok_or_else(|| format!("missing {k}"))
}
fn value<T: FromStr>(a: &mut Attributes, k: &str) -> Result<T, String> {
    required(a, k)?.parse().map_err(|_| format!("invalid {k}"))
}
fn optional<T: FromStr>(a: &mut Attributes, k: &str) -> Result<Option<T>, String> {
    a.remove(k)
        .map(|v| v.parse().map_err(|_| format!("invalid {k}")))
        .transpose()
}
fn link(a: &mut Attributes, k: &str) -> Option<String> {
    a.remove(k).filter(|s| !s.is_empty())
}
fn yn(a: &mut Attributes, k: &str) -> Result<bool, String> {
    match required(a, k)?.as_str() {
        "y" => Ok(true),
        "n" => Ok(false),
        _ => Err(format!("invalid y/n {k}")),
    }
}
fn environment(a: &mut Attributes) -> Result<Environment, String> {
    Ok(Environment {
        kind: required(a, "type")?,
        action_point: optional(a, "action-point")?,
        script: link(a, "script"),
        transit_zone: optional(a, "transit-zone")?,
        transit_spawn: optional(a, "transit-pc-spawn")?,
    })
}
fn parse_record(kind: TableKind, e: Element, path: &Path) -> Result<Record, String> {
    let mut a = attributes(&e)?;
    let id = optional(&mut a, "id")?;
    let alias = link(&mut a, "alias");
    let zone = match kind {
        TableKind::PcSpawn
        | TableKind::Respawn
        | TableKind::Area
        | TableKind::Env
        | TableKind::EnvSpawn => Some(value(&mut a, "zone")?),
        TableKind::EnvPlace => Some(
            path.parent()
                .and_then(Path::file_name)
                .and_then(|s| s.to_str())
                .and_then(|s| s.parse().ok())
                .ok_or("place needs numeric parent-zone directory")?,
        ),
        _ => None,
    };
    if matches!(
        kind,
        TableKind::Zone
            | TableKind::Terrain
            | TableKind::PcSpawn
            | TableKind::Respawn
            | TableKind::Area
            | TableKind::Env
            | TableKind::EnvSpawn
    ) && id.is_none()
    {
        return Err("missing id".into());
    }
    if matches!(
        kind,
        TableKind::Zone
            | TableKind::Terrain
            | TableKind::Env2
            | TableKind::EnvPlace
            | TableKind::Script
            | TableKind::Quest
    ) && alias.is_none()
    {
        return Err("missing alias".into());
    }
    let data = match kind {
        TableKind::Initial => Data::Initial(Initial {
            race: required(&mut a, "race")?,
            job: required(&mut a, "job")?,
            level: value(&mut a, "level")?,
            inventory_size: value(&mut a, "inventory-size")?,
            zone: required(&mut a, "zone")?,
            quest: link(&mut a, "quest-1"),
        }),
        TableKind::Zone => {
            let phase_zones = a
                .keys()
                .filter(|k| {
                    k.strip_prefix("phase-zone-")
                        .is_some_and(|n| n.parse::<u32>().is_ok())
                })
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .map(|k| {
                    let v = a.remove(&k).unwrap_or_default();
                    (k, v)
                })
                .collect();
            Data::Zone(Zone {
                terrain: value(&mut a, "terrain")?,
                sectors: [
                    value(&mut a, "sector-xmin")?,
                    value(&mut a, "sector-xmax")?,
                    value(&mut a, "sector-ymin")?,
                    value(&mut a, "sector-ymax")?,
                ],
                tutorial: yn(&mut a, "tutorial")?,
                default_spawn: optional(&mut a, "default-phase-pc-spawn-id")?,
                reentry: link(&mut a, "reentrance-pcspawn"),
                respawn: link(&mut a, "respawn"),
                script: link(&mut a, "script"),
                phase_zones,
            })
        }
        TableKind::Terrain => Data::Terrain(Terrain {
            start: [
                value(&mut a, "terrain-start-x")?,
                value(&mut a, "terrain-start-y")?,
            ],
            origin: [
                value(&mut a, "terrain-origin-x")?,
                value(&mut a, "terrain-origin-y")?,
            ],
            count: [value(&mut a, "count-x")?, value(&mut a, "count-y")?],
            umap: required(&mut a, "umap-name")?,
        }),
        TableKind::PcSpawn => Data::PcSpawn(PcSpawn {
            pos: value(&mut a, "pos")?,
            yaw: value(&mut a, "yaw")?,
            summon_pos: optional(&mut a, "summon-pos")?,
        }),
        TableKind::Respawn => Data::Respawn(value(&mut a, "area")?),
        TableKind::Area => Data::Area,
        TableKind::Env => Data::Env(environment(&mut a)?),
        TableKind::Env2 => Data::Env2(environment(&mut a)?),
        TableKind::EnvPlace => Data::EnvPlace(environment(&mut a)?),
        TableKind::EnvSpawn => Data::EnvSpawn(EnvSpawn {
            environment: link(&mut a, "env2"),
            place: link(&mut a, "env2place"),
        }),
        TableKind::Script => Data::Script,
        TableKind::Quest => Data::Quest,
    };
    Ok(Record {
        id,
        alias,
        zone,
        source: path.into(),
        source_line: e.source_line,
        data,
        unknown: a,
        children: e.children,
    })
}
#[derive(Clone, Debug, Default)]
pub struct World {
    pub tables: BTreeMap<TableKind, Table>,
    pub skipped_plain: usize,
    pub skipped_tables: usize,
    pub skipped_roots: usize,
}
impl World {
    pub fn table(&self, kind: TableKind) -> Option<&Table> {
        self.tables.get(&kind)
    }
    pub fn from_documents(
        docs: impl IntoIterator<Item = (PathBuf, Document)>,
    ) -> Result<Self, Error> {
        let mut world = Self::default();
        for (p, d) in docs {
            world.add_document(p, d)?;
        }
        Ok(world)
    }
    // Validate a complete file before inserting it. This method is private so
    // callers cannot retain an incomplete world after any error.
    fn add_document(&mut self, path: PathBuf, d: Document) -> Result<(), Error> {
        if d.root.name != "table" {
            self.skipped_roots += 1;
            return Ok(());
        }
        let metadata = attributes(&d.root).map_err(|e| error(&path, e))?;
        let name = metadata
            .get("type")
            .ok_or_else(|| error(&path, "missing table type"))?;
        let Some(kind) = TableKind::from_name(name) else {
            self.skipped_tables += 1;
            return Ok(());
        };
        let mut records = Vec::new();
        let mut extra_nodes = Vec::new();
        for n in d.root.children {
            match n {
                Node::Element(e)
                    if kind == TableKind::Script
                        && !matches!(e.name.as_str(), "script" | "Script" | "record") =>
                {
                    extra_nodes.push(Node::Element(e))
                }
                Node::Element(e) => {
                    let accepted = match kind {
                        TableKind::Quest => e.name == "quest" || e.name == "record",
                        TableKind::Script => true,
                        _ => e.name == "record",
                    };
                    if !accepted {
                        return Err(error(&path, "unexpected table child"));
                    }
                    let line = e.source_line;
                    records.push(parse_record(kind, e, &path).map_err(|message| Error {
                        path: path.clone(),
                        line: Some(line),
                        message,
                    })?);
                }
                Node::Text(t) => extra_nodes.push(Node::Text(t)),
            }
        }
        let count = records.len();
        let table = self.tables.entry(kind).or_default();
        for r in records {
            table.insert(r)?;
        }
        table.files.push(TableFile {
            path,
            source_name: d.source,
            metadata,
            records: count,
            extra_nodes,
        });
        Ok(())
    }
    pub fn load_dir(root: impl AsRef<Path>) -> Result<Self, Error> {
        let root = root.as_ref();
        if !fs::symlink_metadata(root)
            .map_err(|e| error(root, e))?
            .is_dir()
        {
            return Err(error(root, "expected nonsymlink data directory"));
        }
        let mut paths = Vec::new();
        xml_paths(root, &mut paths)?;
        paths.sort();
        let mut w = Self::default();
        for path in paths {
            let bytes = fs::read(&path).map_err(|e| error(&path, e))?;
            if !bytes.starts_with(lmxb::MAGIC) {
                w.skipped_plain += 1;
                continue;
            }
            let doc = lmxb::decode(&bytes).map_err(|e| error(&path, e))?;
            w.add_document(
                path.strip_prefix(root).map_err(|e| error(&path, e))?.into(),
                doc,
            )?;
        }
        if w.tables.is_empty() {
            return Err(error(root, "no minimum-world tables"));
        }
        Ok(w)
    }
    pub fn dangling_references(&self) -> Vec<DanglingReference> {
        let mut out = Vec::new();
        for (&kind, t) in &self.tables {
            for r in &t.records {
                let mut check =
                    |attribute: &str, target: TableKind, scope: Option<u64>, key: ReferenceKey| {
                        let found = self.tables.get(&target).is_some_and(|t| match &key {
                            ReferenceKey::Id(i) => t.by_id(scope, *i).is_some(),
                            ReferenceKey::Alias(a) => {
                                if scope.is_none() {
                                    t.by_alias_any(a).is_some()
                                } else {
                                    t.by_alias(scope, a).is_some()
                                }
                            }
                        });
                        if !found {
                            out.push(DanglingReference {
                                source_table: kind,
                                source: r.source.clone(),
                                line: r.source_line,
                                attribute: attribute.into(),
                                target,
                                scope,
                                key,
                            });
                        }
                    };
                if let Some(z) = r.zone {
                    check("zone", TableKind::Zone, None, ReferenceKey::Id(z));
                }
                match &r.data {
                    Data::Initial(i) => {
                        check(
                            "zone",
                            TableKind::Zone,
                            None,
                            ReferenceKey::Alias(i.zone.clone()),
                        );
                        if let Some(q) = &i.quest {
                            check(
                                "quest-1",
                                TableKind::Quest,
                                None,
                                ReferenceKey::Alias(q.clone()),
                            );
                        }
                    }
                    Data::Zone(z) => {
                        check(
                            "terrain",
                            TableKind::Terrain,
                            None,
                            ReferenceKey::Id(z.terrain),
                        );
                        if let Some(i) = z.default_spawn.filter(|&i| i != 0) {
                            check(
                                "default-phase-pc-spawn-id",
                                TableKind::PcSpawn,
                                r.id,
                                ReferenceKey::Id(i),
                            );
                        }
                        if let Some(a) = &z.reentry {
                            check(
                                "reentrance-pcspawn",
                                TableKind::PcSpawn,
                                None,
                                ReferenceKey::Alias(a.clone()),
                            );
                        }
                        if let Some(a) = &z.respawn {
                            if let Ok(key) = a.parse::<AreaRef>() {
                                check(
                                    "respawn",
                                    TableKind::Respawn,
                                    Some(key.zone),
                                    ReferenceKey::Id(key.id),
                                );
                            } else {
                                check(
                                    "respawn",
                                    TableKind::Respawn,
                                    None,
                                    ReferenceKey::Alias(a.clone()),
                                );
                            }
                        }
                        if let Some(a) = &z.script {
                            check(
                                "script",
                                TableKind::Script,
                                None,
                                ReferenceKey::Alias(a.clone()),
                            );
                        }
                        for (k, a) in &z.phase_zones {
                            if !a.is_empty() {
                                check(k, TableKind::Zone, None, ReferenceKey::Alias(a.clone()));
                            }
                        }
                    }
                    Data::Respawn(a) => check(
                        "area",
                        TableKind::Area,
                        Some(a.zone),
                        ReferenceKey::Id(a.id),
                    ),
                    Data::Env(e) | Data::Env2(e) | Data::EnvPlace(e) => {
                        if let Some(a) = &e.script {
                            check(
                                "script",
                                TableKind::Script,
                                None,
                                ReferenceKey::Alias(a.clone()),
                            );
                        }
                        if let Some(i) = e.transit_zone.filter(|&i| i != 0) {
                            check("transit-zone", TableKind::Zone, None, ReferenceKey::Id(i));
                        }
                        if let Some(i) = e.transit_spawn.filter(|&i| i != 0) {
                            check(
                                "transit-pc-spawn",
                                TableKind::PcSpawn,
                                e.transit_zone.filter(|&i| i != 0).or(r.zone),
                                ReferenceKey::Id(i),
                            );
                        }
                    }
                    Data::EnvSpawn(e) => {
                        if let Some(a) = &e.environment {
                            check(
                                "env2",
                                TableKind::Env2,
                                None,
                                ReferenceKey::Alias(a.clone()),
                            );
                        }
                        if let Some(a) = &e.place {
                            check(
                                "env2place",
                                TableKind::EnvPlace,
                                r.zone,
                                ReferenceKey::Alias(a.clone()),
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
        out
    }
}
fn xml_paths(p: &Path, out: &mut Vec<PathBuf>) -> Result<(), Error> {
    let m = fs::symlink_metadata(p).map_err(|e| error(p, e))?;
    if m.file_type().is_symlink() {
        return Ok(());
    }
    if m.is_dir() {
        for entry in fs::read_dir(p).map_err(|e| error(p, e))? {
            xml_paths(&entry.map_err(|e| error(p, e))?.path(), out)?;
        }
    } else if m.is_file() && p.extension().is_some_and(|e| e == "xml") {
        out.push(p.into());
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceKey {
    Id(u64),
    Alias(String),
}
#[derive(Clone, Debug)]
pub struct DanglingReference {
    pub source_table: TableKind,
    pub source: PathBuf,
    pub line: u32,
    pub attribute: String,
    pub target: TableKind,
    pub scope: Option<u64>,
    pub key: ReferenceKey,
}
