# LMXBOSLB: 2020 GameDaemon data

## Scope and method

This specification uses file bytes only. No external implementation or executable was used. Paths below are relative to `/home/ubuntu/Downloads/BnS_2020/GameDaemon/data/`. No game data is included.

## Header

All integers below use little-endian order.

| Offset | Bytes | Meaning | Evidence |
| --- | --- | --- | --- |
| 0x00 | 8 | ASCII `LMXBOSLB` | All initial samples |
| 0x08 | 4 | Version, observed value 3 | All initial samples |
| 0x0c | 4 | Total file size, including header | 789, 579, 645, 52,144, 369, 393, 4,399,375 bytes match file lengths |
| 0x10 | 64 | Reserved bytes, all zero in initial samples | All initial samples; ❓ individual field meanings |
| 0x50 | 1 | Opaque source flag; observed value 1; ❓ meaning | All binary files |
| 0x51 | 4 | Source-name length in UTF-16 code units | Initial sample names match lengths |
| 0x55 | variable | Transformed UTF-16LE source name | Initial sample names match filenames |

Initial sample files: `terraindata.3040.xml`, `quest/questdata.25208.xml`, `quest/questdata.6.xml`, `quest/questdata.272.xml`, `script/scriptdata.suwalpw_buyoudo.xml`, `script/actsequencedata.suwalpw_buyoudo.xml`, `script/scriptdata.r_tombofemperor.xml`.

## String transformation

XOR each string's UTF-16LE bytes with the repeating 16-byte sequence `a4 9f d8 b3 f6 8e 39 c2 2d e0 61 75 5c 4b 1a 07`. Reset the sequence at the start of each string. The known filename `terraindata.3040.xml` gives all 16 key bytes. The script and quest sample filenames independently match this transformation.

Lengths count UTF-16 code units. There is no NUL terminator. The full byte survey accepts all strings as UTF-16. Generated tests verify zero-length strings, Korean text, and surrogate pairs. ❓ The survey did not count empty strings or surrogate pairs in real data.

## Full-folder byte survey

The 2026-11-08 survey found 21,065 `.xml` paths. Of these, 21,064 have binary magic. Their total size is 1,674,199,807 bytes. All binary files use version 3. All size fields match the file length. All 64 reserved bytes are zero. All observed one-byte fields are 1. The survey parsed 1,786,132 elements and 2,202,819 text nodes. It consumed every binary file with no trailing bytes. Maximum node depth is 7 (root depth 1).

`accountleveldata.xml` is plain UTF-8 XML with a UTF-8 BOM (`ef bb bf`) and an XML declaration at offset 3. Its length is 114,375 bytes. It is not an LMXBOSLB file. The requested approximate folder counts do not match this local snapshot.

Additional header evidence: `social/social_etc/socialdata_etc_all_collect.xml` (395 bytes), `social/social_etc/socialdata_etc_all_nottracking.xml` (403), `social/social_etc/socialdata_etc_all_insdungeun.xml` (1,055,479), `zone/2080/zoneenv2placedata.2080.xml` (352), `zone/4310/zoneenv2placedata.4310.xml` (352), `zone/6442/zonepathwaydata.6442.xml` (445,499). The `terrain/` folder contains no XML paths in this snapshot; terrain files also occur at the data root.

## Body grammar

There is no alignment padding between body fields. All `u32` fields use little-endian order. `string` means `u32 code_unit_count` followed by exactly `2 * code_unit_count` transformed UTF-16LE bytes.

```text
file = header[80], source_flag:u8, source:string, root:element
node = kind:u32, (element if kind=1; text if kind=2)
element = attribute_count:u32, (key:string, value:string)*attribute_count,
          name_flag:u8, name:string, child_count:u32, source_line:u32,
          node*child_count
text = value:string, name_flag:u8, name:string,
       child_count:u32, source_line:u32, node*child_count
```

The root has no `kind` field. Its kind is implicitly element. Attributes retain their input order. Children use depth-first order. Text nodes retain whitespace. All 2,202,819 observed text names are `text`. All observed text nodes are leaves. The decoder must reject text children rather than silently discard them.

`source_line` never decreases during traversal of any binary file. For `terraindata.3040.xml`, root line is 1; the first whitespace and record have line 2; final whitespace has line 3. ❓ Its source-line meaning is inferred from whitespace and ordering, not from a source XML pair. Preserve this value without recalculation.

The one-byte source and name flags are always 1. ❓ Their semantic meaning is unknown. Preserve them as opaque fields; do not treat them as a verified transform selector.

### Offset evidence: terraindata.3040.xml

| Offset | Field |
| --- | --- |
| 0x50 | Source flag |
| 0x51 | Source string length |
| 0x7d | Root attribute count (4) |
| 0x11d | Root name flag |
| 0x11e | Root name string |
| 0x12c | Root child count (3) |
| 0x130 | Root source-line value (1) |
| 0x134 | First child kind (2, text) |
| 0x138 | First text string |
| 0x140 | First text name flag |
| 0x141 | First text name string |
| 0x14d | First text child count (0) |
| 0x151 | First text source-line value (2) |
| 0x155 | Second child kind (1, element) |
| 0x159 | Second child attribute count (10) |
| 0x2dd | Second child name flag |
| 0x2de | Second child name string |
| 0x2ee | Second child child count (0) |
| 0x2f2 | Second child source-line value (2) |
| 0x2f6 | Third child kind (2) |
| 0x315 | End of file |

No offset table, string table, compression, or additional data transformation are needed to parse this snapshot. The filename is metadata, not a filesystem instruction. A reader must not open it.

## Rust API and limits

The new `lmxb` crate has no dependencies.

- `decode(&[u8]) -> Result<Document, Error>` reports malformed input with a byte offset.
- `encode(&Document) -> Vec<u8>` writes version 3. Documents must fit the wire format's u32 file size.
- `to_xml_string(&Document) -> Result<String, Error>` escapes XML characters and preserves text whitespace. It rejects invalid XML characters, names, and duplicate attributes. It omits binary metadata.
- `element_count(&Document) -> usize` includes the root.

The fallible decode and render APIs avoid panics for malformed input. The decoder checks lengths and counts before reading or allocating. It rejects unknown versions, unknown node kinds, text children, invalid UTF-16, trailing bytes, and depth above 128. It does not infer meanings for opaque flags or reserved bytes. The encoder and renderer use iterative traversal. Supported decoded documents have at most 128 node levels.

XML output is semantic XML, not the original source text. Attribute order and text whitespace survive. The records do not have separate declaration or comment fields. ❓ The original source encoding, declaration, comment handling, entity spelling, and self-closing syntax were not verified against source XML pairs. Other builds may use additional node kinds or transformations.

## Reproduction

Run commands from the project directory. The examples read data only. Their output is not a repository fixture.

```sh
cargo run --manifest-path bns-server/Cargo.toml -p lmxb --example inspect -- <file-or-folder>
cargo run --release --manifest-path bns-server/Cargo.toml -p lmxb --example survey -- <data-folder>
cargo test --manifest-path bns-server/Cargo.toml -p lmxb -- --nocapture
cargo test --release --manifest-path bns-server/Cargo.toml -p lmxb --test real_data --config 'env.BNS_GAMEDATA="/home/ubuntu/Downloads/BnS_2020/GameDaemon/data"' -- --nocapture
cargo run --manifest-path bns-server/Cargo.toml -p lmxb --bin lmxb-dump -- <file>
```

Without `BNS_GAMEDATA`, the real-data test prints `SKIP`. It follows no directory symlinks. It reports nonbinary XML paths separately. A folder with no binary files fails the opt-in test.

The full opt-in codec run decoded 21,064 files and 1,786,132 elements. It reported 0 decode failures, 0 byte-exact round-trip failures, and 0 XML render failures. It skipped the one plain XML file. Binary groups were: root 1,098; quest 2,610; script 401; social 88; zone 16,867. The run read 1,674,199,807 binary bytes. The independent body survey counted 2,202,819 text nodes. All observed text nodes were leaves.

