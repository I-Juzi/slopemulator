use gamedata::{Data, Position, ReferenceKey, TableKind as K, World};
use lmxb::{Document, Element, Node};
use std::path::PathBuf;
fn element(name: &str, attrs: &[(&str, &str)], children: Vec<Node>) -> Element {
    Element {
        name_flag: 1,
        name: name.into(),
        attributes: attrs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        source_line: 7,
        children,
    }
}
fn doc(kind: K, rows: Vec<Vec<(&str, &str)>>) -> Document {
    let d = Document {
        reserved: [0; 64],
        source_flag: 1,
        source: "generated.xml".into(),
        root: element(
            "table",
            &[
                ("type", kind.name()),
                ("release-module", "Generated"),
                ("release-side", "server"),
                ("version", "test"),
            ],
            rows.iter()
                .map(|a| Node::Element(element("record", a, vec![])))
                .collect(),
        ),
    };
    lmxb::decode(&lmxb::encode(&d)).unwrap()
}
fn one(kind: K, a: &[(&str, &str)]) -> Document {
    doc(kind, vec![a.to_vec()])
}
fn zone() -> Document {
    one(
        K::Zone,
        &[
            ("id", "41"),
            ("alias", "test-zone"),
            ("terrain", "51"),
            ("tutorial", "y"),
            ("sector-xmin", "-1"),
            ("sector-xmax", "2"),
            ("sector-ymin", "-3"),
            ("sector-ymax", "4"),
            ("default-phase-pc-spawn-id", "1"),
            ("reentrance-pcspawn", "test-spawn"),
            ("respawn", "test-respawn"),
            ("script", "test-script"),
            ("future", "retained"),
        ],
    )
}
fn fixture() -> Vec<(PathBuf, Document)> {
    vec![
        ("zone/41/zone.xml".into(), zone()),
        (
            "terrain.xml".into(),
            one(
                K::Terrain,
                &[
                    ("id", "51"),
                    ("alias", "test-terrain"),
                    ("terrain-start-x", "-1"),
                    ("terrain-start-y", "2"),
                    ("terrain-origin-x", "3"),
                    ("terrain-origin-y", "4"),
                    ("count-x", "5"),
                    ("count-y", "6"),
                    ("umap-name", "unused-resource"),
                ],
            ),
        ),
        (
            "spawn.xml".into(),
            one(
                K::PcSpawn,
                &[
                    ("id", "1"),
                    ("alias", "test-spawn"),
                    ("zone", "41"),
                    ("pos", "-1, 2.5,3"),
                    ("yaw", "90"),
                    ("summon-pos", "0,0,0"),
                ],
            ),
        ),
        (
            "respawn.xml".into(),
            one(
                K::Respawn,
                &[
                    ("id", "1"),
                    ("alias", "test-respawn"),
                    ("zone", "41"),
                    ("area", "41.8"),
                ],
            ),
        ),
        (
            "area.xml".into(),
            one(
                K::Area,
                &[("id", "8"), ("alias", "test-area"), ("zone", "41")],
            ),
        ),
        (
            "initial.xml".into(),
            one(
                K::Initial,
                &[
                    ("race", "test-race"),
                    ("job", "test-job"),
                    ("level", "1"),
                    ("inventory-size", "16"),
                    ("zone", "test-zone"),
                    ("quest-1", "test-quest"),
                ],
            ),
        ),
        (
            "script.xml".into(),
            one(K::Script, &[("alias", "test-script")]),
        ),
        (
            "quest.xml".into(),
            one(K::Quest, &[("alias", "test-quest")]),
        ),
        (
            "env.xml".into(),
            one(
                K::Env,
                &[
                    ("id", "1"),
                    ("alias", "test-env"),
                    ("zone", "41"),
                    ("type", "gate"),
                    ("action-point", "1,2,3"),
                    ("script", "test-script"),
                    ("transit-zone", "41"),
                    ("transit-pc-spawn", "1"),
                ],
            ),
        ),
        (
            "env2.xml".into(),
            one(
                K::Env2,
                &[
                    ("alias", "test-env2"),
                    ("type", "gate"),
                    ("transit-zone", "0"),
                    ("transit-pc-spawn", "0"),
                ],
            ),
        ),
        (
            "zone/41/place.xml".into(),
            one(
                K::EnvPlace,
                &[
                    ("alias", "test-place"),
                    ("type", "gate"),
                    ("action-point", "1,2,3"),
                ],
            ),
        ),
        (
            "envspawn.xml".into(),
            one(
                K::EnvSpawn,
                &[
                    ("id", "1"),
                    ("zone", "41"),
                    ("alias", "test-envspawn"),
                    ("env2", "test-env2"),
                    ("env2place", "test-place"),
                ],
            ),
        ),
    ]
}
#[test]
fn all_typed_tables_and_forward_references() {
    let w = World::from_documents(fixture()).unwrap();
    assert_eq!(w.tables.len(), 12);
    assert!(w.dangling_references().is_empty());
    let z = w.table(K::Zone).unwrap().by_id(None, 41).unwrap();
    assert_eq!(z.unknown.get("future").unwrap(), "retained");
    assert_eq!(
        w.table(K::Zone).unwrap().files[0].metadata["version"],
        "test"
    );
    assert!(matches!(&z.data,Data::Zone(z) if z.sectors==[-1,2,-3,4] && z.tutorial));
    assert!(w
        .table(K::Initial)
        .unwrap()
        .initial("test-race", "test-job", 1)
        .is_some());
    assert!(
        matches!(&w.table(K::PcSpawn).unwrap().by_alias(Some(41),"test-spawn").unwrap().data,Data::PcSpawn(p) if p.pos.y==2.5)
    );
}
#[test]
fn scoped_ids_and_aliases_do_not_collide() {
    let rows = vec![
        vec![
            ("id", "1"),
            ("alias", "same"),
            ("zone", "41"),
            ("pos", "0,0,0"),
            ("yaw", "0"),
        ],
        vec![
            ("id", "1"),
            ("alias", "same"),
            ("zone", "42"),
            ("pos", "0,0,0"),
            ("yaw", "0"),
        ],
    ];
    let w = World::from_documents([("s.xml".into(), doc(K::PcSpawn, rows))]).unwrap();
    let t = w.table(K::PcSpawn).unwrap();
    assert!(t.by_id(Some(41), 1).is_some());
    assert!(t.by_alias(Some(42), "same").is_some());
    assert!(t.by_id(None, 1).is_none());
}
#[test]
fn duplicate_identity_is_an_error() {
    let d = one(K::Script, &[("alias", "duplicate")]);
    let e = World::from_documents([("a.xml".into(), d.clone()), ("b.xml".into(), d)]).unwrap_err();
    assert!(e.message.contains("duplicate alias"));
    assert_eq!(e.line, Some(7));
}
#[test]
fn duplicate_id_and_initial_key_are_errors() {
    let d = one(K::Area, &[("id", "1"), ("zone", "41")]);
    assert!(
        World::from_documents([("a.xml".into(), d.clone()), ("b.xml".into(), d)])
            .unwrap_err()
            .message
            .contains("duplicate id")
    );
    let d = fixture().remove(5).1;
    assert!(
        World::from_documents([("a.xml".into(), d.clone()), ("b.xml".into(), d)])
            .unwrap_err()
            .message
            .contains("duplicate initial")
    );
}
#[test]
fn dangling_checks_keep_source_and_scope() {
    let w = World::from_documents([("zone.xml".into(), zone())]).unwrap();
    let ds = w.dangling_references();
    assert_eq!(ds.len(), 5);
    let d = ds
        .iter()
        .find(|d| d.attribute == "default-phase-pc-spawn-id")
        .unwrap();
    assert_eq!(d.scope, Some(41));
    assert_eq!(d.key, ReferenceKey::Id(1));
    assert_eq!(d.source, PathBuf::from("zone.xml"));
    assert_eq!(d.line, 7);
}
#[test]
fn cross_zone_reentry_and_composite_respawn() {
    let mut docs = fixture();
    if let Node::Element(e) = &mut docs[0].1.root.children[0] {
        e.attributes
            .iter_mut()
            .find(|(k, _)| k == "respawn")
            .unwrap()
            .1 = "41.1".into();
    }
    if let Node::Element(e) = &mut docs[2].1.root.children[0] {
        e.attributes
            .iter_mut()
            .find(|(k, _)| k == "zone")
            .unwrap()
            .1 = "42".into();
    }
    let w = World::from_documents(docs).unwrap();
    let d = w.dangling_references();
    assert!(!d
        .iter()
        .any(|d| d.attribute == "reentrance-pcspawn" || d.attribute == "respawn"));
}
#[test]
fn quest_names_and_script_extras_survive() {
    let mut q = one(K::Quest, &[("alias", "q")]);
    if let Node::Element(e) = &mut q.root.children[0] {
        e.name = "quest".into();
    }
    let mut s = one(K::Script, &[("alias", "s")]);
    if let Node::Element(e) = &mut s.root.children[0] {
        e.name = "Script".into();
    }
    s.root.children.push(Node::Element(element(
        "decision",
        &[("event", "generated")],
        vec![],
    )));
    s.root.children.push(Node::Text(lmxb::Text {
        name_flag: 1,
        name: "text".into(),
        source_line: 9,
        value: "non-whitespace".into(),
    }));
    let s = lmxb::decode(&lmxb::encode(&s)).unwrap();
    let q = lmxb::decode(&lmxb::encode(&q)).unwrap();
    let w = World::from_documents([("q.xml".into(), q), ("s.xml".into(), s)]).unwrap();
    assert_eq!(w.table(K::Script).unwrap().files[0].extra_nodes.len(), 2);
    assert!(w.table(K::Quest).unwrap().by_alias(None, "q").is_some());
}
#[test]
fn strict_positions() {
    for s in ["1,2", "1,2,3,4", "NaN,2,3", "1,inf,3", "a,2,3"] {
        assert!(s.parse::<Position>().is_err(), "{s}");
    }
    assert!("-1,2.5,0".parse::<Position>().is_ok());
}
#[test]
fn bad_or_missing_typed_fields_fail_with_context() {
    for attrs in [
        vec![("id", "1"), ("zone", "41"), ("pos", "1,2"), ("yaw", "0")],
        vec![("id", "1"), ("zone", "41"), ("pos", "0,0,0")],
        vec![("id", "-1"), ("zone", "41"), ("pos", "0,0,0"), ("yaw", "0")],
    ] {
        let e = World::from_documents([("bad.xml".into(), one(K::PcSpawn, &attrs))]).unwrap_err();
        assert_eq!(e.path, PathBuf::from("bad.xml"));
        assert_eq!(e.line, Some(7));
    }
    let mut d = zone();
    if let Node::Element(e) = &mut d.root.children[0] {
        e.attributes
            .iter_mut()
            .find(|(k, _)| k == "tutorial")
            .unwrap()
            .1 = "maybe".into();
    }
    assert!(World::from_documents([("bad.xml".into(), d)]).is_err());
}
#[test]
fn duplicate_attributes_are_not_dropped() {
    let d = one(K::Script, &[("alias", "a"), ("alias", "b")]);
    assert!(World::from_documents([("bad.xml".into(), d)])
        .unwrap_err()
        .message
        .contains("duplicate attribute"));
}
#[test]
fn children_and_unknown_attributes_survive_encoding() {
    let mut d = one(K::Script, &[("alias", "a"), ("new-field", "new-value")]);
    if let Node::Element(e) = &mut d.root.children[0] {
        e.children.push(Node::Element(element(
            "future-child",
            &[("x", "y")],
            vec![],
        )));
    }
    let d = lmxb::decode(&lmxb::encode(&d)).unwrap();
    let w = World::from_documents([("a.xml".into(), d)]).unwrap();
    let r = &w.table(K::Script).unwrap().records[0];
    assert_eq!(r.children.len(), 1);
    assert_eq!(r.unknown["new-field"], "new-value");
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "gamedata-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn directory_loader_uses_metadata_and_reports_skips() {
    let t = Temp::new();
    std::fs::write(
        t.0.join("arbitrary.xml"),
        lmxb::encode(&one(K::Script, &[("alias", "a")])),
    )
    .unwrap();
    std::fs::write(t.0.join("plain.xml"), b"<table/>").unwrap();
    let mut other = one(K::Script, &[]);
    other.root.attributes[0].1 = "future-table".into();
    std::fs::write(t.0.join("other.xml"), lmxb::encode(&other)).unwrap();
    let w = World::load_dir(&t.0).unwrap();
    assert_eq!(w.skipped_plain, 1);
    assert_eq!(w.skipped_tables, 1);
    assert_eq!(w.table(K::Script).unwrap().records.len(), 1);
}
#[test]
fn empty_and_malformed_directories_fail() {
    let t = Temp::new();
    assert!(World::load_dir(&t.0).is_err());
    std::fs::write(t.0.join("bad.xml"), b"LMXBOSLB").unwrap();
    assert!(World::load_dir(&t.0)
        .unwrap_err()
        .message
        .contains("truncated"));
}
#[cfg(unix)]
#[test]
fn symlinks_are_not_followed() {
    use std::os::unix::fs::symlink;
    let t = Temp::new();
    std::fs::write(
        t.0.join("ok.xml"),
        lmxb::encode(&one(K::Script, &[("alias", "a")])),
    )
    .unwrap();
    symlink(t.0.join("ok.xml"), t.0.join("link.xml")).unwrap();
    symlink(&t.0, t.0.join("loop")).unwrap();
    let w = World::load_dir(&t.0).unwrap();
    assert_eq!(w.table(K::Script).unwrap().records.len(), 1);
    assert!(World::load_dir(t.0.join("loop")).is_err());
}
