# 3. Tables, records and strings

Build: data 0.0.381.8884; code `Client.exe` 0.0.381 x86 (IDA copy `ida/lanes/cx/Client.exe.i64`); schemas: operator TableDefs 0.0.381.8876 x86/x64 and the client definition array at `0x194ece8`.
This chapter starts where chapter 2 ends: at the body of one table (after the 11-byte frame, 2.5.3).

## Quick find

| Question | Section | Source |
| --- | --- | --- |
| Body flag byte (plain or compressed) | 3.1 | `0x6228b0`; walk |
| Plain body layout | 3.2 | walk; `0x6228b0` |
| Compressed body layout | 3.3 | walk; `0x62f870`, `0x62fed0` |
| Record header (`DrEl`) and key | 3.4 | PDB `DrEl`; walk |
| Auto-key slot 0 | 3.4.2 | walk |
| Attribute type codes | 3.5 | PDB enum `DrAttrType` |
| `string` fields and the string pool | 3.6 | `08-strings.md`; `RelocateMemoryToDecompress` `0x62e4f0` |
| `native` fields | 3.7 | `0x62e4f0`; `AllocedSize` `0x62e030` |
| `ref` fields | 3.8 | definition array; journal |
| Size-0 and narrow fields | 3.9 | TableDefs scan |
| Full worked example | 3.10 | `map-area` + `text` layouts |
| Rust API per step | 3.11 | `bnsdata` |

## 3.1 The body flag

Each client table body starts with one byte. ✅ walk of all four files (`06-client-body.md`); ✅ client `DrMultiKeyTable::ReadFromBuffer` `0x6228b0`.

| Byte 0 | Variant | Tables per main file (x86 and x64) | Per local file |
| --- | --- | --- | --- |
| 0 | plain (3.2) | 319 (kind 1: 96, kind 2: 223) | 3 |
| 1 | block-compressed (3.3) | 29 (kind 1: 15, kind 2: 14) | 1 (`text`) |

Conflict: the server 348 reader (`0x1406fc640`, `0x1406fe7f0`) has no flag byte. For client files the client layout wins. ✅

## 3.2 Plain body (flag 0)

```
kind 2 and kind 1 (x86):
 0    1         5            9            13   14                       14+F            14+F+X
 +----+---------+------------+------------+----+------------------------+---------------+
 | 00 | count   | fixed size | extra size | LM | fixed region (F)       | extra region  |
 |    | u32     | F u32      | X u32      | u8 | records back to back   | (X) = pool    |
 +----+---------+------------+------------+----+------------------------+---------------+
kind 1 (x64): count is u64 at 1; F at 9; X at 13; LM at 17; fixed region at 18.
```

| Rule | Source |
| --- | --- |
| Body size = header + F + X exactly. No trailing byte. | ✅ walk, 319 tables per file |
| LM (low-memory flag) = 1 in every plain table. Keep it. | ✅ walk |
| The fixed region is a sequence of records. Each record is `DrEl.size` bytes. | ✅ walk |
| The extra region is the string pool of the whole table (3.6). | ✅ `08-strings.md` |
| Kind 2: records = count. Kind 1: records = count − 1 (3.4.2). | ✅ walk |
| Empty kind 1 tables store count 0 or 1 (it changes by file and width). Keep the stored value. | ✅ walk |

## 3.3 Compressed body (flag 1)

```
 0    1             5           7
 +----+-------------+-----------+---- block 0 ----+---- block 1 ----+ ...
 | 01 | block count | u16 = 8 ❓ |                 |                 |
 +----+-------------+-----------+-----------------+-----------------+

 block:
 +-----------+-----------+-----+-------------+-----+-----+--------------------+
 | first key | last key  | cs  | zlib bytes  | ds  | rc  | rc x u16 rec. offs |
 | i64       | i64       | u16 | cs bytes    | u16 | u32 | first = 0, rising  |
 +-----------+-----------+-----+-------------+-----+-----+--------------------+

 inflated block (ds bytes):
 +--------------------------------+------------------------------+----
 | record 0: DrEl (size S0)       | extra span of record 0       | record 1 ...
 +--------------------------------+------------------------------+----
 ^ offs[0] = 0                                                   ^ offs[1]
```

| Rule | Source |
| --- | --- |
| zlib (RFC 1950, header `78 01` seen). Each block inflates to exactly `ds` ≤ 65,535 bytes. | ✅ walk of 16,332 / 17,388 / 2,507 / 2,591 blocks |
| Keys rise across blocks; first ≤ last. | ✅ walk |
| Kind 1: last − first + 1 = rc in 3,062 of 3,067 blocks. So keys are not always contiguous. Do not make keys from positions. | ✅ walk |
| Record span = next offset − this offset (last record: ds − offset). Span ≥ `DrEl.size`. The bytes after `DrEl.size` are the record's own string pool (its "extra span"). | ✅ walk; ✅ client relocation base (3.6) |
| The client keeps blocks compressed and inflates one block on first use: `DrCacheData::decompressChunk` `0x62fed0` (`uncompress`), then `RelocateMemoryToDecompress` `0x62e4f0` for each record. | ✅ |
| Same records in both widths (824,534 per main file, 687,415 per local file). Block bounds differ by width. | ✅ walk |
| Meaning of the u16 value 8 at body offset 5 | ❓ |

## 3.4 Records

### 3.4.1 `DrEl` header and key

Every record (plain and compressed) starts with a 6-byte header. PDB type `DrEl` ✅; walk ✅.

```
 0      1      2            4            6       8                      16
 +------+------+------------+------------+-------+----------------------+---------------
 | type | pad  | subtype    | size       | pad ❓ | key (8-byte union)   | fields ...
 | i8   | u8   | i16        | u16        |       | at keyoffset = 8     |
 +------+------+------------+------------+-------+----------------------+---------------
```

- `size` = full record length in the fixed region, header included. The reader goes to the next record with it. ✅
- `subtype` selects a subtype definition when the table has subtypes; it decides which field list and which size apply. ✅ (definitions have subtype lists; walk checks each `DrEl` type, subtype and size against the definition).
- The 381 definitions put the key at offset 8 (`keyoffset` 8) for the tables in 3.10. ✅ `0x194ece8`. Other tables: read `keyoffset` from the definition.
- Legacy 8-byte prefix (`UseLegacyElHeader`): not in the 381 files. ✅ walk.
- ❓ Bytes 6–7: not named in the PDB fields that we read; the walk keeps them as data.
- `RelocateMemoryToDecompress` `0x62e4f0` does nothing when `type` = −1. ✅ ❓ meaning of type −1 (empty slot).

### 3.4.2 Auto-key tables (kind 1)

Kind 1 tables give each record an automatic number key. The stored count is one more than the number of records: slot 0 is not stored. ✅ walk (all plain kind 1 tables). This explains the special last loop in the server reader `0x1406fe7f0`. ✅

### 3.4.3 Record order

Keep the stored order. Compressed tables need rising keys across blocks (3.3). ❓ The client search rule for plain tables (binary search or map) was not traced.

## 3.5 Attribute type codes

PDB enum `DrAttrType` in the 381 client (values ✅ IDA type library). The TableDefs JSON uses the type names; the code value is in the client definition array.

| Code | Name | Stored as (x86 / x64) | Notes |
| ---: | --- | --- | --- |
| 0 | none | — | |
| 1 | int8 | 1 | |
| 2 | int16 | 2, or 1 when `size` = 1 (3.9) | |
| 3 | int32 | 4 | |
| 4 | int64 | 8 | |
| 5 | float32 | 4 | IEEE bits |
| 6 | bool | 1 | keep the raw byte |
| 7 | str (`string`) | 4 / 8 | 3.6 |
| 8 | seq | 1 | index into a value list |
| 9 | seq16 | 2 | |
| 10 | ref | 8 | 3.8 |
| 11 | tref | ❓ | typed ref |
| 12 | sub | ❓ | |
| 13 | su | ❓ | |
| 14 | vector16 | ❓ | |
| 15 | vector32 | ❓ | |
| 16 | icolor | ❓ | |
| 17 | fcolor | ❓ | |
| 18 | box | ❓ | |
| 19 | angle | ❓ | |
| 20 | msec | 1/2/4/8 by `size` | no unit change |
| 21 | distance | 1/2/4/8 by `size` | |
| 22 | velocity | ❓ | |
| 23 | prop_seq | ❓ | |
| 24 | prop_field | ❓ | |
| 25 | script_obj | ❓ | |
| 26 | native | 8 / 12 | 3.7 |
| 27 | version | ❓ | |
| 28 | icon | ❓ | |
| 29 | time32 | 4 ❓ | |
| 30 | time64 | 8 ❓ | |
| 31 | timeutc | ❓ | |
| 32 | asset | ❓ | |

Rule: always use the `size` of the attribute in the definition for the stored width. Do not use the width of the type name. ✅ (`bnsdata` field decoder, `06-client-body.md` "Raw fields and limits").

Each field can repeat. Repeat `i` is at `offset + i × size`. ✅ client `0x62e4f0` (loop `i < repeat` with stride = attribute size); ✅ `bnsdata`.

## 3.6 `string` fields and the string pool

A `string` field holds a byte offset into a pool. The text is UTF-16LE and ends with `00 00`. ✅ `08-strings.md` (456,218 / 456,218 plain and 14,967,774 / 14,967,774 compressed offsets start a string).

| Variant | Offset base (where offset 0 points) | Source |
| --- | --- | --- |
| Plain | Start of the table extra region | ✅ walk |
| Compressed | Start of the record extra span = record start + `DrEl.size` | ✅ walk; ✅ client `0x62e4f0`: for type 7, the stored u32 gets "record address + u16 size at record + 4" added |

| Width | Field size | Content |
| --- | --- | --- |
| x86 | 4 | u32 offset |
| x64 | 8 | u64 offset |

- Offset 0 is valid: it points to the first string of the pool. The compressed relocation adds the base also when the offset is 0. ✅ `0x62e4f0`.
- Plain tables can share one pool entry between records. Compressed records each own their span: do not share across records. ✅ `08-strings.md` writer rules.
- `AllocedSize` `0x62e030` counts `2 × length + 2` bytes for each non-null string. So the client copy size of a record depends on the string lengths. ✅
- The client skips a string attribute when a byte at attribute + 49 is not 0. ❓ meaning of that byte (in `0x62e4f0` and `0x62e030`).

## 3.7 `native` fields

| Width | Layout | Source |
| --- | --- | --- |
| x86 | u32 `a`, u32 `off` (8 bytes) | ✅ walk; ✅ `0x62e4f0` |
| x64 | u32 `a`, u32 `off`, u32 0 (12 bytes) | ✅ walk (687,477 / 687,477 third words are 0) |

- `off` uses the same base as `string` (3.6). In compressed records the client relocates `off` only when `off` ≠ 0; `off` = 0 stays a null text. ✅ `0x62e4f0` (type 26 path tests the word at field + 4).
- **New:** `AllocedSize` `0x62e030` adds the value of word `a` to the record copy size for each `native` field. So `a` is a byte size of the native text buffer. ✅ use. ❓ Whether `a` includes the 2-byte terminator. Writer rule until verified: when the text changes, set `a` to the new byte size by the same rule as the original records (compare `a` with the text length on real files first). The current Rust writer keeps `a` unchanged; that is wrong for longer text ❓ (journal said "meaning unknown").
- `native` text starts after a terminator and ends exactly at the end of the record span (`text` table, both widths). ✅ 687,415 / 687,415.

## 3.8 `ref` fields

A `ref` holds the 8-byte key of a record in another table. The definition gives the target table (`reftable`). Value 0 = no target. ✅ client definition array (`map-area.name2` → table 327 `text`, 3.10); journal ("ref = 8 key bytes").

- The key format is the key union of the target (3.4.1): for an auto-key target, the auto number; for a multi-key target, the key fields packed as in the target record at `keyoffset`. ❓ Key packing for multi-key targets with more than one key field was not traced.
- No relocation: `RelocateMemoryToDecompress` changes only types 7 and 26. ✅ `0x62e4f0`.
- The client resolves refs at use time. ❓ Not traced. `gamedata::dangling_references` finds refs with no target in our tools.
- `tref` (11): typed ref. ❓ layout.

## 3.9 Size-0 and narrow fields

| Rule | Source |
| --- | --- |
| An attribute with `size` 0 (and `offset` 0) is not in the client record. Skip it on read and on write. | ✅ 381 x86 TableDefs scan; `client_strings_resolve` on all four files |
| Example: `discovery-area.map-overlay` is a `ref` with size 0. | ✅ definition array |
| Size-0 attributes are server-only columns. | ❓ inference |
| An integer attribute can be narrower than its type (`int16` with `size` 1). Read and write by stored size. | ✅ same scan |

## 3.10 Worked example: one `map-area` record and its `text` target

**Synthetic values.** The layout is real (client definition array `0x194ece8`, x86). The byte values are made for this book; they are not game data.

Definitions (✅ `0x194ece8`):

| Table | Type | Version | Kind | Record size x86 | Fields used here |
| --- | ---: | --- | ---: | ---: | --- |
| `map-area` | 195 | 0.2 | 2 (multi-key) | 28 | `id` int16 key @8 size 2; `alias` string @16 size 4; `name2` ref @20 size 8 → `text` |
| `text` | 327 | 0.6 | 1 (auto-key) | 28 | key @8; `alias` string @16 size 4; `text` native @20 size 8 |

x64 (❓ inference from widths): `map-area` 32 bytes, `alias` @16 size 8, `name2` @24.

### 3.10.1 The `map-area` record (plain table)

Table frame (chapter 2): `02 | C3 00 | 00 00 | 02 00 | <body size>` (kind 2, type 195, version 0.2).
Body: `00` (plain), count, F, X, LM = 1, then the fixed region. Our record is the first record. Its pool (extra region) starts at body offset 14 + F.

```
record (28 bytes):
off  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F  10 11 12 13  14 15 16 17 18 19 1A 1B
     01 00 FF FF 1C 00 00 00  05 00 00 00 00 00 00 00  00 00 00 00  2A 00 00 00 00 00 00 00
     |  |  |___| |___| |___|  |___| |______________|  |_________|  |_____________________|
     |  |  subt  size  pad❓  id=5  rest of key = 0   alias off=0  name2 = key 42 of `text`
     |  pad
     type (synthetic value ❓)

extra region (pool of the table), offset 0:
     7A 00 6F 00 6E 00 65 00 5F 00 61 00 00 00     "zone_a" + 00 00
```

| Bytes | Field | Decode |
| --- | --- | --- |
| 0 | `DrEl.type` | 1 (synthetic) ❓ real value rule |
| 2–3 | `DrEl.subtype` | `FF FF` = −1 = no subtype (synthetic) ❓ |
| 4–5 | `DrEl.size` | `1C 00` = 28 = record size in the definition ✅ rule |
| 8–9 | `id` (int16, key) | 5 |
| 10–15 | rest of key union | 0 |
| 16–19 | `alias` | offset 0 → pool bytes 0–13 → `zone_a` |
| 20–27 | `name2` (ref) | 42 → the `text` record with key 42 |

### 3.10.2 The `text` record (compressed, local file)

`text` is the one compressed table in `localfile.bin` (3.1). Find the block whose first key ≤ 42 ≤ last key, inflate it, and find the record whose key is 42. In the inflated block:

```
record at block offset R (span = 44 bytes: 28 fixed + 16 extra):
off  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F  10 11 12 13  14 15 16 17  18 19 1A 1B
     01 00 FF FF 1C 00 00 00  2A 00 00 00 00 00 00 00  00 00 00 00  0A 00 00 00  06 00 00 00
                              key = 42                 alias off=0  a = 10      off = 6
extra span (base = R + 28):
     74 00 31 00 00 00                 "t1"   + 00 00   (span offsets 0-5)
     41 00 72 00 65 00 61 00 00 00     "Area" + 00 00   (span offsets 6-15, ends at span end ✅ rule)
```

| Field | Decode |
| --- | --- |
| key | 42 = the value in `map-area.name2` |
| `alias` | span offset 0 → `t1` |
| `text` (native) | `a` = 10 (synthetic; 4 units × 2 + 2 — see the ❓ in 3.7), `off` = 6 → `Area` |

After `RelocateMemoryToDecompress` `0x62e4f0`, the client has pointers: `alias` = R + 28 + 0, `text` = R + 28 + 6. ✅ rule. The whole decode gives: map area 5, alias `zone_a`, name text `Area`.

## 3.11 Rust API per step

| Step | API (crate `bnsdata`, `Program/Tools/DataFile/BnsData`) | State |
| --- | --- | --- |
| Load definitions | `tabledef::load_folder(path)` (operator TableDefs JSON) | ✅ |
| Parse a body | `table::TableBody::parse(bytes, width, kind, definition)` | ✅ |
| Raw fields | `table::decode_fields(record, definition)` | ✅ |
| Text fields | `table::decode_fields_with_pool(record, definition, pool, width)` → `RawValue::Text` / `RawValue::Native { a, offset, units }` | ✅ |
| Edit text | `TableBody::edit_record(index, definition, width)`, `RecordEditor::set_text` / `set_text_str` | ✅; ❓ native `a` (3.7) |
| Write a body | `TableBody::write(width, kind, Some(definition))` | ✅ |
| Whole project | `bnsmodel`: `files::open`, `Project::record`, `record_key`, `search`, `files::pack` | ✅ |
| Schemas from the client | `bnsschema`: definition array finder, `export::tables`, `layout::for_width` | ✅ |
| Ref checks | `gamedata::dangling_references` | ✅ |

Writer rules (✅ `06-client-body.md`, `08-strings.md`): write the flag first; keep each table's variant; plain: rebuild one table pool and recompute F and X; compressed: rebuild only the edited record's span, re-block with ds ≤ 65,535, write first/last key, rc and offsets; keep stored kind 1 counts for empty tables; compare inflated records, not zlib bytes, in tests.

## Journal links

Merged from: `Docs/DataFile/06-client-body.md` (all), `08-strings.md` (all), `02-tadboslb.md` ("Table bodies in the inspected server reader"), `07-client-exe-tabledefs.md` (definition array), journal `Docs/Client/client-datafiles.md`. New in this chapter (IDA, 2026-11-09): `DrAttrType` code table, client relocation base for `string`, `native` word `a` used as a byte size by `AllocedSize` `0x62e030`, null `native` offset rule.
