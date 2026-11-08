# Table definitions in the client executable (x86)

## Result

The operator JSON folder `TableDefs_0.0.381.8876_x86 KR TEST` is a dump of a static array in the client executable. ✅ Proved: an export from `Client.exe` equals the JSON in 104,340 compared values, with 0 differences.

Source: `Client.exe` in the project root. PE32 (x86), image base `0x400000`, link time 2020-06-04. Normal sections (`.text` entropy 6.62); no packer seen. Passive static read only: no execution, no change, no protection work. 2026-11-07.

## Method

1. Find the UTF-16 table name `abnormalcamera` (VA `0x17e0ce8`).
2. Find the 4-byte pointer to it. The single hit is DrTableDef ID 1 at VA `0x1453260` (`.rdata`).
3. Find the pointer to that DrTableDef in `.data`. It is slot 1 of the pointer array at VA `0x194ece8`: 363 slots, slot 0 null, slots 1–362 = type IDs 1–362.
4. Walk each definition with the x86 layouts below and compare with the JSON.

Scripts: `client-exe/` (`pe.py`, `findtd.py`, `findarr.py`, `cmpexe.py`). They print counts and metadata only.

## x86 layouts (all offsets in bytes, little-endian)

| Struct | Stride | Fields |
| --- | ---: | --- |
| DrTableDef | — | name ptr 0; type i16 4; major i16 6; minor i16 8; el ptr 12; elcount u8 16; module u32 32 ❓ (maxid/autokey offsets not proved) |
| DrElDef | 44 | name ptr 0; type u8 4; body ptr 8; child[32] u8 12 |
| DrElBodyDef | 20 | attr ptr 0; sub ptr 4; default ptr 8; attrcount i16 12; subcount i16 14; size u16 16; keyoffset u16 18 |
| DrSubDef | 16 | name ptr 0; attr ptr 4; default ptr 8; attrcount i16 12; size u16 14 |
| DrAttrDef | 64 | name ptr 0; type u8 8; key u8 9; repeat u8 10; required u8 11; seq ptr 40; seqcount u16 44; reftable i16 46; refel u8 48; deprecated u8 49; hidden u8 50; offset u16 52; size u16 54 |

Compared fields: table name, ID, version, element count; element name, type, child list; body size, key offset, subtype count; subtype name and size; attribute name, type, key, repeat, required, seqcount, reftable, refel, deprecated, hidden, offset, size.

## Attribute type codes (one-to-one, from the comparison)

| Code | Type | Code | Type | Code | Type |
| ---: | --- | ---: | --- | ---: | --- |
| 1 | int8 | 9 | seq16 | 21 | distance |
| 2 | int16 | 10 | ref | 22 | velocity |
| 3 | int32 | 11 | tref | 23 | prop_seq |
| 4 | int64 | 12 | sub | 24 | prop_field |
| 5 | float32 | 15 | vector32 | 25 | script_obj |
| 6 | bool | 16 | icolor | 26 | native |
| 7 | string | 18 | box | 28 | icon |
| 8 | seq | 20 | msec | 30 | time64 |

Codes 7 (string) and 26 (native) agree with the build 348 PDB enum (`05-tablebody.md`).

## Limits

- Not compared: default values, min/max, fmin/fmax, namehash, seq value strings, maxid, autokey.
- x64: no 64-bit client executable is present. The x64 JSON is verified against `datafile64.bin` record sizes only (`STATUS.md`).
- The `Client.exe` build number is not read from the file. Its definitions equal the 8876 JSON, and its link date (2020-06-04) is before the data save time (2020-06-10). ❓ exact build.

## Exporter (2026-11-07)

`client-exe/exportdefs.py EXE ARRAY_VA LAYOUT OUTDIR` writes one TableDefs-format JSON file per table. Layouts: `x86` (0.0.381 `Client.exe`) and `x64p4` (2026 `BNSR_116.exe`). `client-exe/diffdefs.py REF NEW [--same-version]` compares two folders and prints counts only.

| Check | Result |
| --- | --- |
| `Client.exe` x86 export vs operator x86 TableDefs | ✅ 362 tables, 250,383 fields, 0 differences |
| `BNSR_116.exe` x64p4 export vs 381 x64 TableDefs, same-version tables | ✅ 178 tables, 57,528 equal fields; remaining differences explained (renumbered IDs, grown enums, changed `table` element) |
| New 2026 type codes 31, 32, 33 | ❓ names and value encoding |

Exported folders are game data. Keep them outside the repository.
