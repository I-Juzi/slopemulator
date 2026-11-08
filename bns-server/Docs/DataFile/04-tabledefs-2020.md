# 2020 server definitions and client 381 comparison

## Result

The 2020 package does **not** supply the client 0.0.381.8884 schema by numeric ID. Do not use this export to decode or compile client fields. A matching version alone does not prove a matching table name or field layout.

| Client file | Tables | Version match | Version differs | Missing | Kind differs among present IDs |
| --- | ---: | ---: | ---: | ---: | ---: |
| datafile.bin | 348 | 19 | 252 | 77 | 133 |
| datafile64.bin | 348 | 19 | 252 | 77 | 133 |
| localfile.bin | 4 | 0 | 1 | 3 | 1 |
| localfile64.bin | 4 | 0 | 1 | 3 | 1 |

`client381-2020-comparison.json` records every row for every file. It includes the ID, state, both versions, both kinds, server name, and client body size. Source: operator metadata in `client381-tables.json`. This job did not open the real client files.

Example: server ID 2 is `account-post-charge`, v0.1, kind 1. Client ID 2 is v0.39, kind 2. Server ID 4 is `achievement-register`, v0.42, kind 2. Client ID 4 is v0.1, kind 1. The differences include ID assignment, not just a possible version change. Client IDs above 277 have no definition in this array.

## Source and array

Source: `ida/datafile/gd2020/GameDaemon.exe.i64`, from the GameDaemon executable in the 2020-07-04 package. No PDB is present. The package date is not proof of its internal game-data version.

| Item | Address | Evidence |
| --- | --- | --- |
| UTF-16 `account-level` | `0x1407fc5d8` | Name pointer in first definition |
| First DrTableDef | `0x14072ee80` | ID 1, major 0, minor 3; element count 2 |
| Definition pointer array | `0x140e73670` | Slot zero null; slots 1–277 hold matching stored IDs |
| Array end | `0x140e73f20` | 278 slots of eight bytes; also a loop bound in `0x1406272a0` |
| loadData counterpart | `0x140627600` | Calls reader Setup through vtable slot zero with array, count 278, auxiliary pointer `0x1407e4fb0`, and flag 1 |
| Data manager constructor reference | `0x140620d85` | The inspected constructor references the first definition here; entry address not recorded |

The export has 277 tables, 4,521 base-field definitions, and 1,380 subtype definitions. Counts include repeated use of a body definition. All nonzero slots match their stored IDs. Every exported field extent fits the corresponding record size. The exporter reads schema names and layout metadata only. It does not read default records or default-value contents.

## Layout evidence and limits

Offsets below use bytes. Pointer width is eight bytes. Integer order is little-endian. These layouts agree with the build 348 PDB and give consistent metadata across the full 2020 export.

**❓ Independent consumer proof of every 2020 offset is incomplete.** Structural consistency proves that this is a coherent definition graph. It does not independently prove every inherited field meaning. In particular, numeric attribute type labels, reference semantics, autokey-to-kind mapping, unused bounds, and property fields still rely on the build 348 guide. The exporter retains numeric type IDs. It does not claim 32-bit offsets.

| Record | Stride | Exported fields and offsets | Structural check |
| --- | ---: | --- | --- |
| DrTableDef | 48 | name ptr 0; ID i16 8; major i16 10; minor i16 12; element ptr 16; count u8 24; autokey u8 40 | IDs match all array slots; flags are 0 or 1; all names decode |
| DrElDef | 56 | name ptr 0; type u8 8; body ptr 16; 32 child-type bytes 24 | Element arrays give valid names, bodies, and bounded fields |
| DrElBodyDef | 32 | attr ptr 0; subtype ptr 8; attr count i16 24; subtype count i16 26; size u16 28; key offset u16 30 | Counts walk valid metadata; all field extents fit sizes |
| DrSubDef | 32 | name ptr 0; attr ptr 8; attr count i16 24; size u16 26 | All 1,380 subtype entries walk valid names and fields |
| DrAttrDef | 88 | name ptr 0; type u8 16; key u8 17; repeat u8 18; required u8 19; reference-table i16 66; reference-element u8 68; deprecated u8 69; hidden u8 70; offset u16 72; item size u16 74 | Repeats and item sizes fit each layout; names decode |

The inherited layout also places default pointers at body/subtype +16 and attr +24. The exporter deliberately skips these pointers and their contents. Unused DrTableDef maximum-ID/module fields and DrAttrDef bounds/property fields were not exported or independently proved.

Concrete ID 1 evidence:

| Metadata | Address | Value |
| --- | --- | --- |
| element array | `0x14072b2f0` | `table`, then `record` at +56 |
| table body | `0x14072d7e0` | Size 48, zero attributes |
| record body | `0x140728460` | Size 72, key offset 8, 12 attributes |
| first attribute | `0x14072d3c0` | `level`, type 2, offset 8, size 2, key flag 1 |
| second attribute | `0x14072d418` | `exp`, type 4, offset 16, size 8 |
| third attribute | `0x14072d470` | `ability`, type 8, offset 24, item size 1, repeat 8 |

These addresses prove the observed stride and offset reads. Semantic names for types 2/4/8 come from the build 348 PDB enum.

## Reproduction and next step

Run `export_tabledefs_2020.py` inside the supplied IDA database. Call `export(db, output_path)` with a new output path under this specification directory. The script uses `tabledefs.json` only for PDB enum labels. It refuses ID-slot mismatches and out-of-record field extents. It does not save the database.

Obtain a permitted schema source with the client ID order and versions. Do not infer missing schemas from this server date. Continue generic body framing with the named build 348 reader as a guide. Keep target-specific field decoding disabled until the schema source is verified.

No client acceptance signature or integrity check was found in this branch. No check was bypassed. No game key bytes were read or written.
