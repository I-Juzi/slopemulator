# Card dfx-4-rust: `bnsxml` dump and merge (client + server XML)

## Goal
A new crate `bnsxml` dumps the client tables and the server tables to one XML form, and merges them into one XML set with a `merge-report.tsv`. Build and verify are the next card (dfx-5-rust, see the handoff).

## Context
- The XML form, the value text rules and the quirks are in the spec. Do not guess a form that the spec does not give.
- The client field list comes from a TableDefs folder (`bnsdata::tabledef::Definitions::load_folder`). The client records come from `bnsmodel::Project::open_with_schemas` (`tables()`, `record()`, `record_key()`). `FieldView` has `name`, `repeat_index`, `value: Value`, `hidden`, `deprecated`, `hint.enum_names`, `hint.target_table`.
- Server files: `lmxb::decode(&[u8]) -> Result<Document, Error>`. `Document { source_flag, source, root: Element }`, `Element { name_flag, name, attributes: Vec<(String, String)>, source_line, children: Vec<Node> }`, `Node::Element | Node::Text`.
- Run `python3 tools/rust_api.py Program/Tools/DataFile/BnsModel`, `.../BnsData`, `Program/Server/Data/Lmxb` for the exact interfaces. Do not read their source.
- No new external crate. Write a small XML writer and reader for the 10.2.5 form only (elements, attributes, element text for `native`; escape `& < > " '`).

## Inputs
| What | Path | Lines or section |
| --- | --- | --- |
| XML form, attributes, value text | Docs/DataFile/book/10-unified-xml.md | §10.2.1–10.2.5, lines 48–149 |
| Quirks to keep | same | §10.2.6, lines 150–161 |
| Table, record, field match; classes | same | §10.3.1–10.3.2, lines 162–206 |
| Worked examples (test data shape) | same | §10.3.3, lines 207–234 |
| Derived fields | same | §10.4.1, §10.4.3, lines 235–250 and 264–272 |
| Class list with report columns | Docs/DataFile/book/merge-rules.tsv | all rows |
| Existing code | Program/Tools/DataFile/BnsModel, BnsData, Program/Server/Data/Lmxb | `Project`, `FieldView`, `Value`, `TableDefinition`, `lmxb::decode` |

## Work
1. Add the crate `Program/Tools/DataFile/BnsXml` (lib + bin `bnsxml`). Add it to the workspace `members`.
2. Module `form`: the text rules of §10.2.3 and §10.2.2 (functions below). Write bools as `y`/`n`. Write enums with the `seq` name. Write refs as the target alias; if the target record has no alias, the dotted key; key 0 → `null`.
3. Module `xml`: `XmlTable`, `XmlElement`, `write_table`, `read_table` for the §10.2.5 form. Names that start with `_` are tool attributes.
4. Module `dump`: `dump_client` writes `client/<table>.xml` (one file per client table, records in file order) and `derived/<table>.tsv` (key, field, value) for `hidden` fields. Skip `deprecated` fields. Write all repeats up to the last non-default value (`quirk_required_stops_repeats`). Build the ref alias map from the `alias` field of each target table.
5. `dump_server` reads all LMXBOSLB files under a folder. It groups them by root `type`. It writes `server/<type>.xml` with the root attributes, and on each record `_server-file` (path relative to the folder) and `_line` (`source_line`). Keep the attribute order. Keep child elements under their record.
6. Module `merge`: implement the classes of `merge-rules.tsv` and §10.3.2. Table match: §10.3.1 (case-insensitive, `-` removed). Record key: dotted key text of the key fields; auto-key → `alias`. Field compare on normalized text. `both-conflict` → server value (default), `--prefer client` changes it. Write `merged/<table>.xml`, `conflicts.tsv` (table, key, field, client, server), and `merge-report.tsv`.
7. `merge-report.tsv`: one row per table. Columns: `table`, `class` (both-table, version-gap, client-only-table, server-only-table), `client_version`, `server_version`, then the `report_column` names of `merge-rules.tsv` in file order.
8. Bin commands: `bnsxml dump-client <game dir> <schema x86> <out>`, `bnsxml dump-server <data dir> <out>`, `bnsxml merge <client dir> <server dir> <schema x86> <out> [--prefer client|server]`.

## Interfaces
```rust
// form.rs
pub fn table_match_key(name: &str) -> String;                 // lower case, '-' removed
pub fn repeat_name(name: &str, repeat: usize, index: usize) -> String; // repeat 1 -> name; else "name-{index+1}"
pub fn parse_bool(text: &str) -> Option<bool>;                // 10.2.3 eight words, case-insensitive
pub fn bool_text(v: bool) -> &'static str;                    // "y" / "n"
pub enum RefText { Null, Number(u64), Dotted(Vec<String>), Alias(String) }
pub fn parse_ref(text: &str) -> RefText;                      // 10.2.3 ref table, blanks removed
pub fn normalize(kind: &str, text: &str, enum_names: &[String]) -> Option<String>;

// xml.rs
pub struct XmlElement { pub name: String, pub attrs: Vec<(String, String)>, pub text: Option<String>, pub children: Vec<XmlElement> }
pub struct XmlTable { pub root: XmlElement }                  // root.name == "table"
pub fn write_table(table: &XmlTable) -> String;
pub fn read_table(text: &str) -> Result<XmlTable, XmlError>;

// dump.rs
pub struct DumpReport { pub tables: usize, pub records: usize, pub files: usize }
pub fn dump_client(project: &bnsmodel::Project, out: &std::path::Path) -> anyhow::Result<DumpReport>;
pub fn dump_server(data_dir: &std::path::Path, out: &std::path::Path) -> anyhow::Result<DumpReport>;

// merge.rs
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Side { Client, Server }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum TableClass { Both, VersionGap, ClientOnly, ServerOnly }
#[derive(Default, Debug, PartialEq)] pub struct MergeCounts {
    pub records_both: usize, pub records_client_only: usize, pub records_server_only: usize, pub records_no_key: usize,
    pub fields_equal: usize, pub fields_conflict: usize, pub fields_client_only: usize,
    pub fields_server_only: usize, pub fields_derived: usize, pub fields_deprecated: usize }
pub struct Conflict { pub table: String, pub key: String, pub field: String, pub client: String, pub server: String }
pub struct TableMerge { pub name: String, pub class: TableClass, pub client_version: Option<String>,
    pub server_version: Option<String>, pub counts: MergeCounts, pub merged: XmlTable, pub conflicts: Vec<Conflict> }
pub fn merge_table(client: Option<&XmlTable>, server: Option<&XmlTable>,
    def: Option<&bnsdata::tabledef::TableDefinition>, prefer: Side) -> TableMerge;
pub fn merge_dirs(client_dir: &std::path::Path, server_dir: &std::path::Path, schema_dir: &std::path::Path,
    out: &std::path::Path, prefer: Side) -> anyhow::Result<Vec<TableMerge>>;
pub fn report_tsv(merges: &[TableMerge]) -> String;
```
If `anyhow` is not in the workspace, use a crate `Error` enum. Use the error type of the workspace crates.

## Files
- `Program/Tools/DataFile/BnsXml/`
- `Cargo.toml` (workspace `members` only)
- `Cargo.lock`

## Acceptance
| Test name | Input | Expected result |
| --- | --- | --- |
| `repeat_names` | `repeat_name("filter-equip", 4, 0)`, `(…, 4, 3)`, `("alias", 1, 0)` | `filter-equip-1`, `filter-equip-4`, `alias` |
| `bool_words` | `y`, `YES`, `t`, `True`, `n`, `no`, `F`, `false`, `1` | true ×4, false ×4, `None` for `1` (`quirk_bool_strict_text`) |
| `ref_forms` | ``, `NULL`, `1234`, `12.3`, `.5`, `5.`, `Item_A` | Null, Null, Number(1234), Dotted([12,3]), Alias, Alias, Alias |
| `table_match` | `Terrain` vs `terrain`; `FielditemDrop` vs `fielditemdrop`; `zone-env2` vs `ZoneEnv2` | equal keys |
| `xml_roundtrip` | table with `&`, `<`, `"` in values, one `native` text, one child element | `read_table(write_table(t)) == t` |
| `merge_terrain_shape` | synthetic client + server `terrain` (10 equal fields, 2 records; 1 record only on the server) | class Both; records_both 1, records_server_only 1; fields_equal 10; fields_client_only 0; fields_server_only 0 |
| `merge_conflict_server_wins` | one field differs | merged has the server value; 1 `Conflict`; with `Side::Client` the client value |
| `merge_derived_not_written` | client field with `hidden` = true | not in merged XML; fields_derived 1 |
| `merge_version_gap` | client 1.398, server 1.387 | class VersionGap; both record sets with `_side`; no field compare |
| `merge_achievement_key` | key `id`, `step`, `job` (prop_seq) | records match by dotted key `id.step.job-name` |
| `report_columns` | `report_tsv` on 2 merges | header = `table class client_version server_version` + report columns of `merge-rules.tsv` in order; 2 rows |
| `real_dump_merge` (`#[ignore]`) | env `BNSXML_GAME`, `BNSXML_SCHEMA`, `BNSXML_SERVER` | `terrain` row: class Both, fields_server_only 0; write the report rows of `terrain`, `achievement`, `item` in the handoff |

## Out of scope
- `build-client`, `build-server`, `verify` (next card).
- The `.dat` archive, AES, RSA.
- Computing derived fields (§10.4.2); the dump only stores them.
- Changes to `bnsdata`, `bnsmodel`, `lmxb`.
