# Table definitions — GameDaemon build 348

## Scope and evidence

Source: `ida/datafile/gd348/GameDaemon.exe.i64`, with build 348 PDB types. This job read schema metadata only. It did not read default records or table data values. It did not change or save the database.

`tabledefs.json` lists all 265 named `DrTableDef` objects found in this database. It includes IDs 1 through 265, names, versions, kinds, element definitions, base fields, and subtype fields. The export includes 4,420 base-field definitions and 1,340 subtype definitions. Repeated use of one definition can produce repeated metadata in this export.

❓ The client files belong to build 0.0.381.8884, not build 348. Do not use these definitions as target schemas. The operator observed target type 2 at version 0.39; build 348 type 2 is 0.1. Target type 3 is 0.3; build 348 type 3 is 0.19. Target type 4 is 0.1; build 348 type 4 is 0.41. These are schema-version differences, not a proved name mapping between builds. Source for target versions: operator validation in `notes/client-datafiles.md`, section 76 onward.

## Definition array and lookup

| Source | Address | Evidence |
| --- | --- | --- |
| `Data::TableDefs` | `0x140e99970` | Pointer array; 8-byte slots; 266 slots observed |
| `Data::DataManager::loadData` | `0x14061f8f0` | Passes array and count 266 to `DrElReaderImpl::Setup` |
| `DrElReaderImpl::Setup` | `0x1406f2760` | Stores definition pointer and count |
| `Data::DataManager` constructor | `0x140618710` | Initializes loader definitions; reference to zone definition at `0x14061cff4` |
| `BinaryDataFile::readFromMem` | `0x1406fa370` | Searches loader definitions by `tabledef->type`; compares major and minor |
| `DrElReaderImpl::ReadHeader` | `0x1406f2f90` | Finds an XML element type and selects `tabledef->el[type]` |
| `DrElReaderImpl::readEl` | `0x1406f2c10` | Reads base fields, then subtype fields |
| `DrElReaderImpl::RelocateAddress` | `0x1406edcb0` | Uses field offsets, sizes, repeats, and pointer-bearing types |

Enumeration procedure:

1. Read 266 u64 pointers from `0x140e99970`.
2. Skip the raw null pointer in slot zero.
3. Read each remaining pointer as `DrTableDef`.
4. Use the type ID in the definition, not an assumed array index.
5. Follow the element pointer for `elcount` records.
6. Follow each body definition to base fields and subtype definitions.

All 265 nonzero array slots matched named `DrTableDef` objects during export. The first slots correspond to IDs 1, 2, 3, and so on. ❓ Slot zero has a null raw value, but IDA also reports a reference from that address to `ZoneTableDef`. Startup state for slot zero was not proved. Do not treat slot zero as a valid table definition.

`DrLoaderDef` is a runtime record, not a serialized DAT table header. Its stride is 96 bytes. The table pointer is at 0; the definition pointer is at 8. The remaining fields include XML paths, a postprocessor, document handles, and loaded state. The DAT reader advances 12 pointer widths between definition-pointer fields. No fixed global address for the runtime loader array was proved; the data manager holds `_loaderDefs`.

The `autokey` flag maps to kind 1 when true and kind 2 when false. The JSON kind is derived from this flag. The DAT reader independently selects kind 1 or 2 from the file. A schema match alone does not prove that the client accepts the file.

## Definition layouts, 64-bit PDB

All offsets and sizes below are bytes. Integers use little-endian byte order.

| Record | Stride | Fields |
| --- | --- | --- |
| `DrTableDef` | 48 | name pointer 0; i16 type ID 8; i16 major 10; i16 minor 12; element pointer 16; byte element count 24; u64 maximum ID 32; bool autokey 40; int module 44 |
| `DrElDef` | 56 | name pointer 0; type byte 8; body pointer 16; 32 child-type bytes at 24 |
| `DrElBodyDef` | 32 | attribute pointer 0; subtype pointer 8; default-record pointer 16; i16 attribute count 24; i16 subtype count 26; u16 record size 28; u16 key offset 30 |
| `DrSubDef` | 32 | name pointer 0; attribute pointer 8; default-record pointer 16; i16 attribute count 24; u16 record size 26 |
| `DrAttrDef` | 88 | name pointer 0; name-hash pointer 8; type byte 16; key bool 17; repeat byte 18; required byte 19; default-value pointer 24; integer bounds 32/40; float bounds 48/52; sequence pointer 56; i16 sequence count 64; i16 reference table 66; reference-element byte 68; deprecated bool 69; hidden bool 70; u16 offset 72; u16 size 74; u64 property field 80 |

The JSON deliberately excludes default values and default records. Schema strings are plain UTF-16LE. The export reads field names, not key material. `key` in an attribute definition means a table-index field. It does not mean an encryption key.

`DrEl` has a type byte at 0, an i16 subtype at 2, and a u16 serialized record size at 4. The containing record can include padding. Do not infer the first field offset from the six occupied header bytes. Source: PDB `DrEl` and the readers above.

A base field starts at `record_base + offset`. Repetition advances by `size` bytes. `size` is one repeated item's stride, not necessarily the whole field extent. A nonnegative `DrEl.subtype` selects a `DrSubDef`. Subtype field offsets also use the same record base. Source: `0x1406f2c10` and `0x1406edcb0`.

The relocation code handles `DR_ATTR_STR` (7) as a pointer at the field offset. It handles `DR_ATTR_NATIVE` (26) as a structure with a pointer at field offset + 8 in this build. It skips deprecated fields. These facts describe build 348 relocation only. They do not define the target client's table-body encoding.

## Field example

Build 348 type 1, `account-level`, version 0.3, kind 2:

- Definition: `0x1407d1100`.
- Element array: `0x1407d1090`.
- `table` element body: `0x1407d0c30`, record size 48.
- `record` element body: `0x1407d1070`, record size 72, key offset 8.

| Field | Type | Offset64 | Size64 | Repeat |
| --- | --- | --- | --- | --- |
| level | INT16 | 8 | 2 | 1 |
| exp | INT64 | 16 | 8 | 1 |
| ability | SEQ | 24 | 1 | 8 |
| ability-value | INT32 | 32 | 4 | 8 |
| badge-slot-quota-category1-premium | INT8 | 64 | 1 | 1 |
| badge-slot-quota-category1-normal | INT8 | 65 | 1 | 1 |

The field definition addresses start at `0x1407d0c50` and advance by 88 bytes. The full set is in `tabledefs.json`.

## Selected table IDs

The JSON contains the complete 265-entry build 348 list. This table gives common examples only.

| ID | Name | Major.minor | Kind | Definition address |
| --- | --- | --- | --- | --- |
| 1 | account-level | 0.3 | 2 | `0x1407d1100` |
| 2 | account-post-charge | 0.1 | 1 | `0x1407d13a0` |
| 3 | achievement | 0.19 | 2 | `0x1407d1b60` |
| 4 | achievement-register | 0.41 | 2 | `0x1407d3820` |
| 5 | act-sequence | 0.13 | 1 | `0x1407d5040` |
| 41 | combat-sequence | 0.86 | 1 | `0x1407c9c78` |
| 75 | effect | 0.313 | 2 | `0x1407bd4c0` |
| 88 | filter | 0.68 | 2 | `0x1407c21d0` |
| 114 | item | 1.387 | 2 | `0x1407a50d0` |
| 124 | itemskill | 0.13 | 1 | `0x1407a5730` |
| 132 | job | 0.50 | 2 | `0x1407820b0` |
| 152 | npc-brain-parameters | 0.25 | 1 | `0x140789240` |
| 154 | npc | 0.252 | 1 | `0x14078f550` |
| 160 | pc | 0.55 | 2 | `0x140795010` |
| 167 | quest | 1.287 | 2 | `0x14076dfb8` |
| 181 | script | 1.174 | 1 | `0x1407789b0` |
| 188 | skill3 | 3.276 | 2 | `0x14077f060` |
| 192 | skillcastcondition3 | 1.42 | 1 | `0x140780f10` |
| 195 | skill | 2.190 | 1 | `0x14075af00` |
| 224 | summoned | 0.38 | 2 | `0x14074d180` |
| 233 | terrain | 0.8 | 2 | `0x14074ffc0` |
| 247 | zone | 0.146 | 2 | `0x140743700` |
| 258 | zonepcspawn | 0.49 | 2 | `0x1407439f0` |
| 265 | zonetriggereventstage | 0.16 | 2 | `0x140745cd0` |

## 32-bit versus 64-bit records

✅ This PDB and all exported offsets describe the 64-bit server. Its definition pointers and native string pointers are 8 bytes. The operator observed different body sizes in the target 32- and 64-bit files.

❓ No corresponding 32-bit definition array was inspected. The JSON uses `offset64` and `size64`; it leaves the 32-bit layout unknown. Pointer width changes can also change padding and later field offsets. Do not subtract four from all record sizes or reuse the 64-bit offsets for 32-bit files.

❓ Scalar storage rules, pointer relocation forms, native structures, and subtype padding require target-build checks before schema-aware writing. The container writer does not reinterpret any record body.

## Reproducibility and boundary

`export_tabledefs.py` collects the schema metadata through IDA's read-only byte and type APIs. Call `export(db, output_path)` from an IDA Python session on this database. The output file must not exist. The script does not save the database. The checked export also records the lookup source and the slot-zero uncertainty.

No client signature, hash, or integrity check was found in this branch. Client acceptance remains unverified. If such a check is found, record its address and stop that branch. Do not bypass it.

Next: obtain build 381 schema definitions for both widths. Then specify table-internal records, aliases, and the two unknown header size rules before implementing an XML-to-table compiler.
