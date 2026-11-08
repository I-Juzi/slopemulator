# Client 0.0.381.8884 table bodies — verified framing

## Scope and source

Source: read-only structural walks of the operator files in `/home/ubuntu/Downloads/Bin/` (`datafile.bin`, `datafile64.bin`, `localfile.bin`, `localfile64.bin`, game version 0.0.381.8884), 2026-11-07. The walks printed counts and sizes only. No record content was written to the repository. Cross-check (facts only, no code copied): BnsBinTool `TableHeader.cs`, `TableReader.cs`, `RecordCompressedReader.cs` (no licence).

This supersedes the "extra byte" and "auto-key final slot" questions in `05-tablebody.md`. The build 348 server reader has no leading flag byte. The client format has one.

## Body = 1 flag byte + variant

| Offset | Size | Field |
| --- | --- | --- |
| 0 | u8 | `compressed`: 0 = plain variant, 1 = block-compressed variant |
| 1 | — | Variant body |

| File | Tables | Plain (kind 1 / 2) | Compressed (kind 1 / 2) |
| --- | ---: | --- | --- |
| datafile.bin | 348 | 319 (96 / 223) | 29 (15 / 14) |
| datafile64.bin | 348 | 319 (96 / 223) | 29 (15 / 14) |
| localfile.bin | 4 | 3 (1 / 2) | 1 (1 / 0) |
| localfile64.bin | 4 | 3 (1 / 2) | 1 (1 / 0) |

✅ Every table in all four files ends exactly at its declared body size under the rules below.

## Plain variant (flag 0)

| Field | Kind 2 | Kind 1, 32-bit | Kind 1, 64-bit |
| --- | --- | --- | --- |
| count | u32 @1 | u32 @1 | **u64** @1 |
| fixed-region size | u32 @5 | u32 @5 | u32 @9 |
| extra-region size | u32 @9 | u32 @9 | u32 @13 |
| low-memory flag | u8 @13 | u8 @13 | u8 @17 |
| fixed region starts | 14 | 14 | 18 |

- ✅ Body size = header + fixed size + extra size, exactly, for all 319 plain tables per file. No trailing byte.
- ✅ Low-memory flag = 1 in every plain table.
- ✅ The fixed region is a sequence of records. Each record starts with the 6-byte DrEl header (i8 type, u8 pad, i16 subtype, u16 size) and is `size` bytes long. No legacy 8-byte prefix occurs. The walk ends exactly at the region end in every table.
- ✅ Kind 2: physical records = count.
- ✅ Kind 1: physical records = count − 1 when count > 0. Slot 0 is not stored. This explains the "special final iteration" in build 348 `0x1406fe7f0`.
- ✅ Empty kind 1 tables: 32-bit files store count 0 (9 tables) or 1 (type 9). The 64-bit file stores count 1 for all of them. **Preserve the stored count. Do not recompute it for empty tables.**
- No 6-byte (header-only) records occur in plain tables.

## Compressed variant (flag 1)

| Offset | Size | Field |
| --- | --- | --- |
| 1 | u32 | block count |
| 5 | u16 | constant 8 (all 30 compressed tables per file) ❓ meaning |
| 7 | — | blocks, back to back |

Each block:

| Size | Field |
| --- | --- |
| i64 | first record key in block |
| i64 | last record key in block |
| u16 | compressed size `cs` |
| `cs` | zlib stream (`78 01` header seen) |
| u16 | decompressed size `ds` (≤ 65,535) |
| u32 | record count `rc` in block |
| `rc` × u16 | record start offsets in the decompressed block; first is 0, ascending |

- ✅ All 16,332 / 17,388 / 2,507 / 2,591 blocks (datafile / datafile64 / localfile / localfile64) decompress with plain zlib to exactly `ds` bytes. Walk ends exactly at body end.
- ✅ Block key ranges are ascending across blocks, and first ≤ last, for both kinds.
- ✅ Kind 1: last − first + 1 = `rc` in 3,062 of 3,067 blocks (32-bit). Keys are not always contiguous. Do not derive keys from positions in compressed auto tables.
- ✅ Each record span (next offset − this offset, or `ds` − offset for the last) is at least the DrEl size. In most records it is larger: the remainder is per-record extra data (string pool). BnsBinTool treats it as a per-record string buffer ❓ not checked by us.
- ✅ Same records in both widths: 824,534 compressed records per datafile, 687,415 per localfile. Block boundaries differ between widths.

## Writer rules derived (for repacking)

1. Write the flag byte first. Keep each table's original variant unless a client test proves the other is accepted ❓.
2. Plain: recompute fixed and extra sizes from content. Keep the stored count for kind 1 empty tables.
3. Compressed: split records into blocks with `ds` ≤ 65,535. Write first/last key, zlib stream, `ds`, `rc` and offsets. Block split policy and zlib level are free choices ❓ (client acceptance not tested).
4. A byte-identical re-compress is not expected. The round-trip test for changed tables must compare decompressed records, not raw bytes.

## Still open

| Question | State |
| --- | --- |
| Field layouts (schemas) per type for 0.0.381 | ✅ Operator supplied build 0.0.381.8876 x86/x64 JSON folders. The operator verified every DrEl size against client 0.0.381.8884. This job did not read the client files. |
| Meaning of the u16 constant 8 | ❓ |
| Extra-data encoding (strings, natives) | ✅ `string` = byte offset to UTF-16LE zero-terminated text; base = table extra region (plain) or record extra span (compressed). `native` = u32 a ❓ + u32 text offset (same base) [+ u32 0 in x64]. See `08-strings.md`. |
| `_totalTableSize`, `_maxBufferSize` header rules | ❓ |
| Client accepts a re-compressed table | ❓ Not tested |

## Rust API — step 4 (2026-11-07)

The code is in `crates/bnsdata/`. It uses fresh Rust code and the layout above.
No operator definitions or record content are embedded.

| API | Function |
| --- | --- |
| `tabledef::Definitions::load_folder(path)` | Load immediate JSON files into a map keyed by table type ID. Reject duplicate IDs and inconsistent metadata counts. Select the width-specific folder explicitly. |
| `TableDefinition::from_json(bytes)` | Load one definition. Unknown metadata, including default record contents, is not retained. Null attribute/subtype lists become empty lists. |
| `TableDefinition::layout(element_type, subtype)` | Return full size, base key offset, base fields, and subtype fields. Negative subtype selects the base layout. Nonnegative subtype indexes `body.sub`. |
| `table::TableBody::parse(bytes, width, kind, definition)` | Parse one bounded body. The optional definition checks every DrEl type, subtype, and fixed size. |
| `TableBody { compressed, variant, .. }` | Keep the original variant. `Variant::Plain` holds stored count, low-memory byte, fixed records, and table extra bytes. `Variant::Compressed` holds the constant and blocks with first/last keys and records. |
| `Record { fixed, extra, key }` | Keep the DrEl header in `fixed`. Keep each compressed record's remaining span in `extra`. Optional `key` overrides key extraction for changed output. Parsed records have no override. |
| `TableBody::write(width, kind, definition)` | Recompute sizes and nonempty plain counts. Keep the empty kind 1 stored count. Validate records before writing. |
| `TableBody::discard_original()` | Remove the original compressed bytes. The next write must re-compress the records. |
| `table::decode_fields(record, definition)` | Return named raw values and zero-based repeat indexes from base and subtype fields. |

### Writer checks

Plain records cannot contain per-record extra bytes. Store those bytes in the table extra region.
The writer preserves the low-memory byte and all record padding.
The plain parser checks the exact physical record count and both region ends.

The compressed parser checks ascending block ranges and strictly increasing offsets.
Each block must have at least one record. A table can have zero blocks.
Inflation is limited to the declared size plus one byte. The declared size cannot exceed 65,535.
The parser requires zlib `StreamEnd`, including the checksum trailer.
It rejects short streams, oversized output, extra compressed bytes, and invalid spans.

For unchanged compressed input, the writer compares the current model with the original model.
It re-inflates the original bytes for this comparison. It does not trust a caller-managed dirty flag.
It then emits the original bytes without a zlib change.
This retains nondefault compression levels and block boundaries.
The comparison costs one extra inflation and a temporary record model.

For changed compressed input, the writer uses each explicit `Record.key` when present.
Otherwise, it reads eight little-endian bytes at the selected base layout's `keyoffset`.
Source: the build 348 reader in `05-tablebody.md`, `0x1406fc640`.
❓ Independent client proof of this key rule, especially for auto-key tables, remains open.
Use explicit keys if the schema key representation does not fit that rule.
The writer never derives a key from the record position or block range.
It requires strictly increasing keys. It does not sort records or repair duplicates.
It recomputes block ranges, record counts, offsets, and the block count.
It keeps the u16 constant.

A new block contains at most 65,535 decompressed bytes.
If zlib output exceeds the u16 compressed-size limit, the writer splits the block again.
A single record that cannot fit either limit returns an error.
Changed block boundaries and zlib output need not match the original bytes.
❓ Client acceptance of these changes is not established.

### Raw fields and limits

| Types | Raw result |
| --- | --- |
| `int8`, `int16`, `int32`, `int64` | Signed little-endian integer with checked storage width |
| `bool` | Raw byte; noncanonical values are not normalized |
| `float32`, `float` | IEEE storage bits; NaN payloads and signed zero stay intact |
| `seq`, `seq16` | Unsigned 8-bit / 16-bit value; no label lookup |
| `msec`, `distance` | Signed integer at the schema's 1/2/4/8-byte width; no unit conversion |
| `ref` | Raw integer bytes; no target lookup |
| `string`, `native`, all other types | `Unresolved` with type name and raw offset/descriptor bytes |

❓ This job does not prove signedness or units for semantic scalar aliases beyond their raw integer storage.
Each repeat uses `offset + repeat_index * size`. All field extents use checked arithmetic.
Deprecated fields remain visible as raw storage.
No offset becomes a native pointer. No string encoding is assumed.
The API does not relocate offsets, rebuild string pools, resolve references, or convert widths.
The outer container's unknown total-size and maximum-buffer fields remain unchanged.

### Operator-only byte check

`tests/real_files.rs::client_table_bodies_roundtrip` is ignored by default.
It checks table versions, kinds, every fixed-record size, each unchanged body, and the full container.
It reads only paths from the four specified environment variables.
It writes no files. It prints counts only.
The dat4 job did not run it. ✅ The main session ran it on 2026-11-07: all four files pass (see `STATUS.md`, "dat4 verification").

Run these commands from `lobbygate-rs`'s parent directory:

```sh
BNSDATA_CLIENT_DAT='/home/ubuntu/Downloads/Bin/datafile.bin' BNSDATA_DAT_WIDTH=32 BNSDATA_DAT_ROLE=main BNSDATA_TABLEDEFS='/home/ubuntu/Downloads/TableDefs_0.0.381.8876_x86 KR TEST' CARGO_NET_OFFLINE=true cargo test --manifest-path lobbygate-rs/Cargo.toml -p bnsdata --test real_files client_table_bodies_roundtrip -- --ignored --exact --nocapture
BNSDATA_CLIENT_DAT='/home/ubuntu/Downloads/Bin/datafile64.bin' BNSDATA_DAT_WIDTH=64 BNSDATA_DAT_ROLE=main BNSDATA_TABLEDEFS='/home/ubuntu/Downloads/TableDefs_0.0.381.8876_x64 KR TEST' CARGO_NET_OFFLINE=true cargo test --manifest-path lobbygate-rs/Cargo.toml -p bnsdata --test real_files client_table_bodies_roundtrip -- --ignored --exact --nocapture
BNSDATA_CLIENT_DAT='/home/ubuntu/Downloads/Bin/localfile.bin' BNSDATA_DAT_WIDTH=32 BNSDATA_DAT_ROLE=local BNSDATA_TABLEDEFS='/home/ubuntu/Downloads/TableDefs_0.0.381.8876_x86 KR TEST' CARGO_NET_OFFLINE=true cargo test --manifest-path lobbygate-rs/Cargo.toml -p bnsdata --test real_files client_table_bodies_roundtrip -- --ignored --exact --nocapture
BNSDATA_CLIENT_DAT='/home/ubuntu/Downloads/Bin/localfile64.bin' BNSDATA_DAT_WIDTH=64 BNSDATA_DAT_ROLE=local BNSDATA_TABLEDEFS='/home/ubuntu/Downloads/TableDefs_0.0.381.8876_x64 KR TEST' CARGO_NET_OFFLINE=true cargo test --manifest-path lobbygate-rs/Cargo.toml -p bnsdata --test real_files client_table_bodies_roundtrip -- --ignored --exact --nocapture
```
