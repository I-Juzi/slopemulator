# 4. The schema: table definitions (TableDefs)

Build: data 0.0.381.8884; code `Client.exe` 0.0.381 x86 (IDA copy `ida/lanes/cx/Client.exe.i64`, names and types ported from the GM 419 PDB).
This chapter tells what a table definition is, where the client keeps it, its exact bytes, how our tools export it, and what each element and attribute type means. The list of all 362 tables is in [tables.tsv](tables.tsv) (section 4.9).

## Quick find

| Question | Section | Address / file |
| --- | --- | --- |
| What is a table definition, why does it exist? | 4.1 | — |
| Where is the definition array? | 4.2 | `0x194ece8` (363 slots) |
| `DrTableDef` bytes | 4.3.1 | example `map-area` at `0x1444c44` |
| `DrElDef` (elements) | 4.3.2 | `0x1444c68` |
| `DrElBodyDef` (record size, key offset) | 4.3.3 | `0x1444c30` |
| `DrAttrDef` (one field) | 4.3.4 | `0x1444b70` |
| `DrSubDef` (subtypes) | 4.3.5 | PDB type |
| Module flags (who uses a table) | 4.4 | enum `Data::ModuleType` |
| Element kinds and tree tables | 4.5 | 6 tables with more than 2 elements |
| Attribute type codes and their use counts | 4.6 | enum `DrAttrType` |
| Versions 8876 and 8884; server 348 | 4.7 | `client381-tables.json`; `0x624440` |
| How `bnsschema` exports the array | 4.8 | `Program/Tools/DataFile/BnsSchema` |
| Columns of `tables.tsv` | 4.9 | [tables.tsv](tables.tsv) |
| Open questions | 4.10 | — |

## 4.1 What a table definition is

The data files store records as raw C structures (chapter 3). The files do not describe their own fields. The only description of a record is compiled into the program that reads it. NCSoft calls this description a **table definition** (`DrTableDef`, "Dr" = data reader). ✅ PDB type names.

A table definition tells:

- the table name (`item`, `text`, ...) and its **type ID** (1–362 in build 381). The data file uses only the type ID (table frame, 2.5.3);
- the schema **version** (major, minor). The client refuses a file whose table version is different (4.7);
- the **kind**: auto-key (kind 1) or multi-key (kind 2), from the field `autokey`;
- the **elements**: element 0 is the table itself, element 1 is the record. Some tables have more elements (child nodes of a tree, 4.5);
- for each record element: the record size, the key offset, the list of **attributes** (fields) with type, offset, size, repeat count, reference target, and the **subtypes**.

The same definitions also drive the XML reader (`DrElReaderImpl`, chapter 5): the source data is XML; a tool converts it to the binary file with the same definitions. So one definition describes three forms of one table: the XML text, the binary record, and the record in memory. ✅ `DrElReaderImpl::readNative` `0x61cb90`, `readRef` `0x61c750` use `DrAttrDef`; ❓ the converter tool itself is not in our files (1.3).

Operator name: the folders `TableDefs_0.0.381.8876_x86 KR TEST` and `..._x64 KR TEST` are JSON dumps of these definitions (4.7, 4.8).

## 4.2 Where the definitions are

```
.data                                   .rdata (const)
0x194ece8  slot 0   = 0 (null)
0x194ecec  slot 1   ─────────────────> DrTableDef "abnormalcamera"  0x1453260
   ...
0x194ece8 + 4 × 195 ─────────────────> DrTableDef "map-area"        0x1444c44
   ...
0x194ece8 + 4 × 362 ─────────────────> DrTableDef "zonetriggereventstage" 0x14907f0 (last slot)
0x194f294  end of array (= 0x194ece8 + 4 × 363)
```

| Fact | Source |
| --- | --- |
| Array of 363 pointers. Slot 0 is null. Slot N points to the definition whose type ID is N, for N = 1–362. | ✅ IDA read 2026-11-09 (all 362 slots: `i16 type == N`) |
| The loader gets the array and its length 363 as arguments: `DrElReader` setup call `(**elreader)(elreader, 0x194ece8, 363, config, 1)`. | ✅ `Data::DataManager::Load` `0x5bc500` (decompiler line 109) |
| `Load` walks the same array up to `0x194f294` to make the set of all table types for the XML path. | ✅ `0x5bc500` (lines 217–227) |
| Each definition also has a named global, for example `Data::AcquireRouteTableDef`. `DataManager` passes it to `setupLoaderDef`. | ✅ `0x5b655d` |
| The definitions are constant data. Only the default records (4.3.3) are in writable memory and filled at run time. | ✅ `map-area` default record `0x1b1267c` is zero in the image ❓ who fills it |

## 4.3 Binary layout (x86, little-endian)

The structures are packed to 4 bytes. All pointers are 32-bit virtual addresses in the image. The x64 layout of the 2026 client is in 4.8.

### 4.3.1 `DrTableDef` (36 bytes)

| Offset | Size | Field | Meaning |
| ---: | ---: | --- | --- |
| 0 | 4 | `name` | pointer to UTF-16LE table name |
| 4 | 2 | `type` | type ID (i16), equal to the array slot |
| 6 | 2 | `major_ver` | i16 |
| 8 | 2 | `minor_ver` | i16 |
| 12 | 4 | `el` | pointer to an array of `elcount` × `DrElDef` |
| 16 | 1 | `elcount` | number of elements (u8) |
| 20 | 8 | `maxid` | u64; 0 in 296 tables; for example 10000 for `map-area` ❓ use |
| 28 | 1 | `autokey` | 1 = kind 1 (auto-key), 0 = kind 2 (multi-key) |
| 32 | 4 | `module` | bit set of `Data::ModuleType` (4.4) |

Source: PDB type `DrTableDef` (size 36) ✅; values ✅ IDA. The `bnsschema` reader reads only the low u32 of `maxid` (`10-schema-from-exe.md`); all 381 values fit in u32 ✅.

Real bytes, `map-area` (type 195) at `0x1444c44`:

```
01444c44  f0 d9 65 01 c3 00 00 00 02 00 00 00 68 4c 44 01   name=0x165d9f0  type=0x00c3=195  major=0  minor=2  el=0x1444c68
01444c54  02 00 00 00 10 27 00 00 00 00 00 00 00 00 00 00   elcount=2  maxid=0x2710=10000  autokey=0
01444c64  0c 00 00 00                                       module=0x0c = CLIENT | TOOL
```

### 4.3.2 `DrElDef` (44 bytes)

| Offset | Size | Field | Meaning |
| ---: | ---: | --- | --- |
| 0 | 4 | `name` | UTF-16LE element name (`table`, `record`, or a tree node name) |
| 4 | 1 | `type` | element index type; 0 for `table`, 1 for `record` ✅ |
| 8 | 4 | `body` | pointer to `DrElBodyDef`, or 0 |
| 12 | 32 | `child[32]` | element indexes of the child elements; 0 ends the list |

`map-area` element 1 (`record`) at `0x1444c94`:

```
01444c94  c4 c6 56 01 01 00 00 00 30 4c 44 01 00 00 00 00   name="record"  type=1  body=0x1444c30  child[0..3]=0
01444ca4  00 ... (rest of child[] = 0)
```

Element 0 (`table`) has `child[0]` = 1: the table node holds record nodes. ✅ IDA (`map-area` el 0 child list `[1, 0, ...]`).

**Use in the loader (new):** the first byte of each record (`DrEl.type`, 3.4.1) is the **element index**. The multi-key reader reads the key offset from `tabledef->el[DrEl.type].body->keyoffset`. ✅ `DrMultiKeyTable::ReadFromBuffer` `0x6228b0` (expression `44 * *record + el + 8`, then `+18`). So for a normal record, `DrEl.type` = 1. This replaces the "synthetic value" note in 3.10.

### 4.3.3 `DrElBodyDef` (20 bytes)

| Offset | Size | Field | Meaning |
| ---: | ---: | --- | --- |
| 0 | 4 | `attr` | pointer to `attrcount` × `DrAttrDef` |
| 4 | 4 | `sub` | pointer to `subcount` × `DrSubDef` |
| 8 | 4 | `defaultRecord` | pointer to a default record (`DrEl`) in writable memory |
| 12 | 2 | `attrcount` | i16 |
| 14 | 2 | `subcount` | i16 |
| 16 | 2 | `size` | record size in bytes, header included (base record without subtype) |
| 18 | 2 | `keyoffset` | byte offset of the 8-byte key in the record |

`map-area` record body at `0x1444c30`:

```
01444c30  70 4b 44 01 00 00 00 00 7c 26 b1 01 03 00 00 00   attr=0x1444b70  sub=0  default=0x1b1267c  attrcount=3  subcount=0
01444c40  1c 00 08 00                                       size=28  keyoffset=8
```

`keyoffset` is 8 in all 362 tables ✅ IDA. Element 0 (`table`) has a body with size 36 and no attributes ✅ (`abnormalcamera`, `map-area`, `text`).

### 4.3.4 `DrAttrDef` (64 bytes in 381)

| Offset | Size | Field | Meaning |
| ---: | ---: | --- | --- |
| 0 | 4 | `name` | UTF-16LE attribute name |
| 4 | 4 | `namehash` | pointer to `DrNameHash` (u32 hash, name pointer) |
| 8 | 1 | `type` | `DrAttrType` code (4.6) |
| 9 | 1 | `key` | 1 = part of the record key |
| 10 | 1 | `repeat` | number of repeats (1 = single value) |
| 11 | 1 | `required` | 1 = XML must give a value ❓ (XML reader only) |
| 12 | 4 | `defvalue` | UTF-16LE default text, or 0 |
| 16 | 8 | `min` | i64 lower limit |
| 24 | 8 | `max` | i64 upper limit |
| 32 | 4 | `fmin` | f32 |
| 36 | 4 | `fmax` | f32 |
| 40 | 4 | `seq` | pointer to `seqcount` UTF-16 names (enum values of `seq`, `seq16`, `prop_seq`) |
| 44 | 2 | `seqcount` | |
| 46 | 2 | `reftable` | target type ID of `ref` / `tref` |
| 48 | 1 | `refel` | target element index |
| 49 | 1 | `deprecated` | 1 = not in the binary record (4.6.2) |
| 50 | 1 | `hidden` | 1 = hidden field ❓ use |
| 52 | 2 | `offset` | byte offset in the record |
| 54 | 2 | `size` | byte size of one repeat |
| 56 | 8 | `propfield` | ❓ |

`map-area.id` at `0x1444b70`:

```
01444b70  48 5c 4c 01 00 40 99 01 02 01 01 01 00 00 00 00   name  namehash  type=2 int16  key=1  repeat=1  required=1  defvalue=0
01444b80  01 00 00 00 00 00 00 00 10 27 00 00 00 00 00 00   min=1  max=10000
01444b90  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00   fmin=0 fmax=0 seq=0 seqcount=0 reftable=0 refel=0 deprecated=0 hidden=0
01444ba0  00 00 00 00 08 00 02 00 00 00 00 00 00 00 00 00   offset=8  size=2  propfield=0
```

**Conflict: size 64 or 68.** The GM 419 PDB type `DrAttrDef` is 68 bytes and ends with `bool dynamic` after a u64 `propfield`. The 381 array has a stride of 64 bytes: with stride 64 the attribute names of `abnormalcamera` read correctly; with 68 they do not. ✅ IDA test 2026-11-09. **381 wins for 381 data:** stride 64, no `dynamic` field. The PDB type is for the 2021 build.

### 4.3.5 `DrSubDef` (16 bytes)

| Offset | Size | Field |
| ---: | ---: | --- |
| 0 | 4 | `name` (UTF-16LE subtype name) |
| 4 | 4 | `attr` (pointer to the extra attributes of this subtype) |
| 8 | 4 | `defaultRecord` |
| 12 | 2 | `attrcount` (u16) |
| 14 | 2 | `size` (record size with this subtype) |

Source: PDB type `DrSubDef` (16 bytes) ✅; `bnsschema` layout ✅.

A record with `DrEl.subtype` = s (s ≥ 0) has the base attributes **and** the attributes of subtype s, and its size is `sub[s].size`. `DrEl.subtype` = −1 means no subtype. ✅ `AllocedSize` walks the base attributes and then the subtype attributes (`client-datafiles.md`, "Extra data of packed elements"). 38 tables have subtypes; `effect` has 671 ✅ IDA.

## 4.4 Module flags

`DrTableDef.module` is a bit set. The enum is in the PDB (`Data::ModuleType`) ✅:

| Bit | Name | Bit | Name |
| --- | --- | --- | --- |
| `0x1` | TEST | `0x80` | MARKET |
| `0x2` | SERVER | `0x100` | ACHIEVEMENT |
| `0x4` | CLIENT | `0x200` | TESTHELPER |
| `0x8` | TOOL | `0x400` | MOBILEGATE |
| `0x10` | ENGINE | `0x800` | DUELBOT |
| `0x20` | CRAFT | `0x1000` | RANKING |
| `0x40` | ARENALOBBY | `0x2000` | ACCOUNTINVENTORY |

Facts ✅ IDA (all 362 definitions):

- Every definition in the client has the CLIENT bit. So the client exe has only client tables; the server exe has its own array (`tabledefs.json`, server 348).
- Most common values: `0x6` SERVER|CLIENT (137 tables), `0x4` CLIENT only (95), `0xc` CLIENT|TOOL (29), `0xe` (28).
- `text` has `0x41c` = CLIENT|TOOL|ENGINE|MOBILEGATE.
- ❓ The module bits do not choose the file. The four tables in `localfile.bin` are `petition-faq-list` (`0x4`), `survey` (`0x6`), `text` (`0x41c`), `user-command` (`0x4`).

## 4.5 Elements and tree tables

356 tables have two elements: `table` (index 0) and `record` (index 1). Six tables have more ✅ IDA:

| Table | Elements after `table` |
| --- | --- |
| `contextscript` (62) | contextscript, stance, layer, decision, condition, result |
| `quest` (239) | quest and 22 child nodes (mission, reward, ...) |
| `skill-training-sequence` (288) | skill-training-sequence, step |
| `summoned-sequence` (317) | summoned-sequence, action, melee, range, range-sim |
| `surveyquestions` (320) | surveyquestion, question, questionexample |
| `tutorialskillsequence` (331) | tutorialskillsequence, step |

None of these six has a table in the four `.bin` files ✅ (`client381-tables.json`). The client has a special loader object for each (`QuestTable`, `ContextScriptLoader`, `SurveyQuestionsLoader`, ... — members of `Data::DataManager`, PDB type) ✅ names. ❓ They load from XML in `xml.dat` through `LoadDocument` (chapter 5.6).

## 4.6 Attribute types

### 4.6.1 Codes and how often 381 uses them

Enum `DrAttrType` (PDB) ✅. Count = number of attributes of this type in the 362 record bodies plus subtypes ✅ IDA 2026-11-09. Layouts: chapter 3.5.

| Code | Name | Count | Code | Name | Count |
| ---: | --- | ---: | ---: | --- | ---: |
| 1 | int8 | 719 | 16 | icolor | 2 |
| 2 | int16 | 1,354 | 18 | box | 1 |
| 3 | int32 | 582 | 20 | msec | 121 |
| 4 | int64 | 59 | 21 | distance | 58 |
| 5 | float32 | 165 | 22 | velocity | 16 |
| 6 | bool | 549 | 23 | prop_seq | 205 |
| 7 | string | 1,788 | 24 | prop_field | 7 |
| 8 | seq | 607 | 25 | script_obj | 4 |
| 9 | seq16 | 71 | 26 | native | 2 |
| 10 | ref | 1,579 | 28 | icon | 53 |
| 11 | tref | 59 | 30 | time64 | 5 |
| 12 | sub | 2 | 15 | vector32 | 16 |

Not used in 381: 13 su, 14 vector16, 17 fcolor, 19 angle, 27 version, 29 time32, 31 timeutc, 32 asset. `native` is used only twice (one is `text.text`).

### 4.6.2 Attribute flags in numbers (base records, 6,697 attributes)

| Flag | Count | Rule |
| --- | ---: | --- |
| `key` = 1 | — | Key fields. Per table: 0 key fields in 119 tables (all auto-key), 1 in 147, 2 in 51, 3 in 25, up to 6 ✅ |
| `repeat` > 1 | 1,148 | Repeat `i` at `offset + i × size` (3.5) |
| `required` = 1 | 1,523 | ❓ XML only |
| `ref`/`tref` | 1,275 | Target in `reftable` (3.8, chapter 6) |
| `seqcount` > 0 | 728 | Enum names for `seq`-like fields |
| `deprecated` = 1 | 203 | **All 203 have `size` 0, and all size-0 attributes are deprecated** ✅ IDA. This explains 3.9: a size-0 field is a deprecated field. The client also skips it in `AllocedSize` (byte at attribute + 49) ✅ `client-datafiles.md`. This closes ❓ 9 of part 1 for the byte at + 49. |
| `hidden` = 1 | 143 | ❓ use |

### 4.6.3 How keys are packed

The record key is always the 8 bytes at `keyoffset` (8). The key fields are inside these 8 bytes at their own offsets. The reader reads the two u32 at `record + keyoffset` and `+ 4` as one u64. ✅ `0x6228b0` (`set(key_lo, key_hi, record)`).

| Table | Key fields (offset, size) | u64 key |
| --- | --- | --- |
| `zone` | `id` int32 (8, 4) | `id` |
| `item` | `id` int32 (8, 4), `level` int8 (12, 1) | `id + level × 2^32` |
| `skill3` | `id` int32 (8, 4), `variation-id` int8 (12, 1) | `id + variation-id × 2^32` |
| `achievement` | `id` int16 (8, 2), `step` int16 (10, 2), `job` prop_seq (12, 1) | `id + step × 2^16 + job × 2^32` |
| `text` (auto-key) | none | record number (chapter 6) |

Unused bytes in the 8-byte key are 0 ❓ (writer rule: keep them 0).

## 4.7 Versions: 8876, 8884 and the server

| Fact | Source |
| --- | --- |
| The operator TableDefs folders carry the number 0.0.381.**8876**. The data files are 0.0.381.**8884**. | folder names; `client381-tables.json` |
| The `Client.exe` array equals the 8876 x86 JSON: 362 tables, 250,383 fields, 0 differences. | ✅ `07-client-exe-tabledefs.md` |
| **New check:** all 352 tables in the four 8884 files have the same kind and version (major, minor) as the `Client.exe` definition. 0 differences. | ✅ IDA + `client381-tables.json`, 2026-11-09 |
| The client rejects the whole file when one table version differs (`major_ver`/`minor_ver` compare). So these files load with this exe. | ✅ `0x624440` (decompiler lines 137–139) |
| Conclusion: 8876 and 8884 have the same schema. The last number is a data build number, not a schema change. | inference from the two rows above ✅ |
| 10 definitions have no table in the files: the 6 tree tables (4.5) and `moveanimtransit` (210), `questrewardskill3` (241), `season-contents` (252), `zoneevent` (352). The last four also have no `setupLoaderDef` call, so no loader slot uses them. | ✅ IDA; chapter 5.3 |
| Server 348 (`tabledefs.json`) is a different schema: type IDs are renumbered. By ID: 19 match, 252 version differs, 133 kind differs, 77 missing. Do not use server definitions for client files. | ✅ `client381-2020-comparison.json` |
| The 2026 client (`BNSR_116.exe`, x64) has 519 tables and new type codes 31–33. | ✅ journal ("`BNSR_116.exe`") |

## 4.8 Export with `bnsschema`

`bnsschema` (`Program/Tools/DataFile/BnsSchema`) reads a PE file as bytes, finds the array, and writes one TableDefs JSON file per table. Full contract: `Docs/DataFile/10-schema-from-exe.md`. Summary:

```
 Client.exe bytes ─> PE parse ─> protection gate ─> width: PE32 → x86, PE32+ → x64p4
                                                    │
         find array: slot 0 null, slot i → def with type == i, name [a-z0-9_-]   (longest run, ≥ 16)
                                                    │
         walk DrTableDef → DrElDef → DrElBodyDef → DrAttrDef / DrSubDef
                                                    │
         <out>/<table name>.json   (one file per table; loaded by bnsdata::tabledef::load_folder)
```

| Structure | x86 stride / fields | x64p4 (2026) |
| --- | --- | --- |
| `DrTableDef` | 4.3.1 | type at 8, el at 16, elcount 24, maxid 28, autokey 36, module 40 |
| `DrElDef` | 44 | 52: type u32 at 8, body 12, child 20 |
| `DrElBodyDef` | 20 | 32: attrcount 24, subcount 26, size 28, keyoffset 30 |
| `DrSubDef` | 16 | 28 |
| `DrAttrDef` | 64 | 84: type 16, key 17, ..., offset 68, size 70, propfield 72 |

Results: `Client.exe` → array `0x194ece8`, 362 tables, 0 differences to the 8876 x86 JSON ✅ (journal, "Schema read from the exe in Rust"). `bnsschema` tests are synthetic; the real-file test is ignored by default (`STATUS-schema.md`) ❓ last run date.

No x64 381 exe exists in our files. The x64 TableDefs JSON is checked only against `datafile64.bin` record sizes ✅ (`STATUS.md`). The x64 record size of a table is not in [tables.tsv](tables.tsv); use the x64 JSON.

## 4.9 `tables.tsv`

One row per definition (362 rows). Tab-separated, UTF-8, header row first. Made by a read-only IDA script on 2026-11-09 from the array `0x194ece8` and `client381-tables.json`.

| Column | Meaning | Source |
| --- | --- | --- |
| `id` | type ID | `DrTableDef.type` |
| `name` | table name | `DrTableDef.name` |
| `kind` | 1 auto-key, 2 multi-key | `autokey` |
| `version` | major.minor | definition (= file, 4.7) |
| `file` | `datafile`, `localfile`, or `none` | `client381-tables.json` |
| `record_count` | `0` when the x86 body is the 14-byte empty plain body; `?` = not counted per table in our files ❓ | body size |
| `record_size_x86` | base record size (no subtype) | `DrElBodyDef.size` of element 1 |
| `key` | key field names; `(auto)` for auto-key | `DrAttrDef.key` |
| `keyoffset` | key offset | `DrElBodyDef.keyoffset` |
| `fields` | base attributes (deprecated ones included) | `attrcount` |
| `subtypes`, `sub_fields` | subtype count, sum of subtype attributes | `DrSubDef` |
| `elements` | elements after `table` (1 = only `record`) | `elcount − 1` |
| `module` | bit set (4.4) | `module` |
| `def_addr_381` | VA of the `DrTableDef` | array slot |
| `body_size_x86`, `body_size_x64` | stored body size in the file (`-` = not in a file) | `client381-tables.json` |
| `readers`, `reader_count` | client functions that read the table: `readers` = 2 example names and `…(+n)` for the top 60 tables, else `-`; `reader_count` for all (chapter 6.5) | IDA scan |

Record counts per table: our walks counted records per file, not per table (journal: `item` 144,439; `text` 687,415). A per-table count needs one more walk ❓ (`bnsmodel` can give it).

## 4.10 Open questions (❓)

1. `maxid` use (66 tables have a value). 
2. `hidden`, `required`, `propfield`, `refel` use in the binary path.
3. Who fills the default records (writable memory, zero in the image).
4. Do the module bits matter at run time in the client?
5. Unused bytes of a packed key: always 0?
6. Per-table record counts (4.9).

## Journal links

Merged from: `Docs/DataFile/03-tabledefs.md`, `04-tabledefs-2020.md`, `07-client-exe-tabledefs.md`, `10-schema-from-exe.md`, `STATUS-schema.md`, `client-datafiles.md` (sections "Operator table definitions", "Origin of the TableDefs JSON", "Proof", "Schema read from the exe in Rust", "GM client 2021 with PDB"). New in this chapter (IDA, 2026-11-09): hex dumps, `DrAttrDef` 64 vs 68, module enum, element-index meaning of `DrEl.type`, deprecated = size 0, version match of all 352 file tables, attribute type counts, key packing, the four unused definitions.
