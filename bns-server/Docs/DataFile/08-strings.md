# String and native fields — verified encoding (client 0.0.381.8884)

Source: read-only statistical walks of `datafile.bin`, `datafile64.bin`, `localfile.bin`, `localfile64.bin` with the operator TableDefs, 2026-11-07. Tools: `client-exe/strprobe.py`, `client-exe/natprobe.py`. The tools print counts only.

## Offset base

| Variant | Base of every string/native offset |
| --- | --- |
| Plain (flag 0) | Start of the table extra region (after the fixed region) |
| Compressed (flag 1) | Start of the record's extra span: record offset + DrEl size, up to the next record offset (or the block end) |

## Field layouts

| Type | x86 | x64 | Meaning |
| --- | --- | --- | --- |
| `string` | u32 | u64 | Byte offset to UTF-16LE text, ended by `00 00`. 0 is valid. |
| `native` | u32 a, u32 off | u32 a, u32 off, u32 0 | `off` = byte offset to UTF-16LE text ended by `00 00`. `a` ❓ unknown; keep it. |

## Checks

| Check | Result |
| --- | --- |
| Plain `string` offsets that start a string (datafile x86) | ✅ 456,218 / 456,218 |
| Compressed `string` offsets that start a string (datafile x86) | ✅ 14,967,774 / 14,967,774 |
| `native` text starts after a terminator and ends exactly at the record span end (`text`, both widths) | ✅ 687,415 / 687,415 |
| x64 `native` third word = 0 | ✅ 687,477 / 687,477 |

## Writer rules

1. Rebuild the pool: write each string once in UTF-16LE with `00 00`, and store its byte offset.
2. Plain: the pool is the table extra region. Recompute the extra size.
3. Compressed: the pool is the record extra span. The record span changes, so block offsets change.
4. `native`: keep word `a` as read ❓. Client acceptance of a changed `a`/text pair is not tested.
5. Do not deduplicate strings across records in compressed tables; each record owns its span.

## Rust API — step 5 (2026-11-07)

The code uses the verified layouts above. Tests use synthetic data only.

| API | Function |
| --- | --- |
| `table::decode_fields(record, definition)` | Keep the step 4 raw API. Text descriptors remain `Unresolved`. No pool is read. |
| `table::decode_fields_with_pool(record, definition, pool, width)` | Decode base/subtype fields and repeats. Return `RawValue::Text { offset: u64, units: Vec<u16> }` or `RawValue::Native { a: u32, offset: u64, units: Vec<u16> }`. Use table extra bytes for plain tables. Use the record extra bytes for compressed tables. |
| `RawValue::text_string()` | Return a `String` without replacement characters. Reject invalid UTF-16 or a non-text value. |
| `TableBody::edit_record(index, definition, width)` | Open a record editor. The zero-based index spans compressed blocks in stored order. Validate the record and its text pool. |
| `RecordEditor::set_text(name, repeat_index, units)` | Stage UTF-16 code units for one string/native field. Keep unpaired surrogates. Reject embedded NUL, missing names, ambiguous names, and invalid repeats. Keep native word `a`. |
| `RecordEditor::set_text_str(name, repeat_index, text)` | Encode a Rust string as UTF-16 and stage the edit. |
| `TableBody::write(width, kind, Some(definition))` | Apply staged edits to a temporary model. Rebuild affected pools. Use the existing bounded block writer for compressed output. |

### Bounds and preservation

The pool decoder rejects odd offsets, offsets outside the pool, and missing two-byte terminators.
It checks descriptor sizes against the selected width.
It rejects a nonzero x64 native third word.
Offset zero is valid. A terminated empty string is valid.
The raw API retains invalid surrogate code units. Only the `String` helper rejects them.

The editor stores pending changes separately from `variant`.
Raw record reads still show the original values until output is parsed again.
Use the same width and definition for editing and writing.
Writing pending edits without a definition returns an error.
Writing does not change the source table model.
Repeated writes produce the same output.
A same-value edit does not stage a change.
Setting a field back to its original units cancels its pending change.

With no text change, the writer retains unknown and unreferenced pool bytes.
It also retains original zlib bytes and block boundaries for unchanged parsed tables.
With a plain text change, the writer builds one new table pool.
It deduplicates identical code-unit sequences across all string/native fields and records.
It relocates every known text descriptor and retains every unedited text value.
Unreferenced bytes are not retained in a rebuilt pool.
With a compressed text change, the writer rebuilds only the edited record's pool.
It writes string fields first, then native fields, in definition/repeat order.
It does not deduplicate compressed pool entries.
Unedited compressed records retain their fixed and extra bytes.
Changed table zlib bytes and block boundaries can differ.
Native word `a` and fixed bytes outside relocated offsets remain unchanged.
Record span overflow returns an error through the existing re-blocking writer.

### Operator-only check

`tests/real_files.rs::client_strings_resolve` is ignored by default.
It resolves every string/native field and prints counts only.
It checks a same-value edit in the first eligible plain and compressed table.
It writes no files and never prints text content or game bytes.
The test uses the same four environment variables as step 4.
❓ This job did not run real-file checks. Shell approval also blocked workspace verification.
❓ Client acceptance of changed pools and the meaning of native word `a` remain open.
The outer container's total-size and maximum-buffer rules remain unchanged.

Run from `/home/ubuntu/Documents/Codex/private-project`:

```sh
BNSDATA_CLIENT_DAT='/home/ubuntu/Downloads/Bin/datafile.bin' BNSDATA_DAT_WIDTH=32 BNSDATA_DAT_ROLE=main BNSDATA_TABLEDEFS='/home/ubuntu/Downloads/TableDefs_0.0.381.8876_x86 KR TEST' CARGO_NET_OFFLINE=true cargo test --manifest-path bns-server/Cargo.toml -p bnsdata --test real_files client_strings_resolve -- --ignored --exact --nocapture
BNSDATA_CLIENT_DAT='/home/ubuntu/Downloads/Bin/datafile64.bin' BNSDATA_DAT_WIDTH=64 BNSDATA_DAT_ROLE=main BNSDATA_TABLEDEFS='/home/ubuntu/Downloads/TableDefs_0.0.381.8876_x64 KR TEST' CARGO_NET_OFFLINE=true cargo test --manifest-path bns-server/Cargo.toml -p bnsdata --test real_files client_strings_resolve -- --ignored --exact --nocapture
BNSDATA_CLIENT_DAT='/home/ubuntu/Downloads/Bin/localfile.bin' BNSDATA_DAT_WIDTH=32 BNSDATA_DAT_ROLE=local BNSDATA_TABLEDEFS='/home/ubuntu/Downloads/TableDefs_0.0.381.8876_x86 KR TEST' CARGO_NET_OFFLINE=true cargo test --manifest-path bns-server/Cargo.toml -p bnsdata --test real_files client_strings_resolve -- --ignored --exact --nocapture
BNSDATA_CLIENT_DAT='/home/ubuntu/Downloads/Bin/localfile64.bin' BNSDATA_DAT_WIDTH=64 BNSDATA_DAT_ROLE=local BNSDATA_TABLEDEFS='/home/ubuntu/Downloads/TableDefs_0.0.381.8876_x64 KR TEST' CARGO_NET_OFFLINE=true cargo test --manifest-path bns-server/Cargo.toml -p bnsdata --test real_files client_strings_resolve -- --ignored --exact --nocapture
```

## Field-walk rules found on real files (2026-11-07)

| Rule | Source | State |
| --- | --- | --- |
| Attributes with `size` 0 (always `offset` 0) are not stored in the client record. Skip them when reading and writing. | 381 x86 TableDefs scan; `client_strings_resolve` on all four files | ✅ |
| An integer attribute can be stored narrower than its type (`int16` with `size` 1). Read it by stored size. | same | ✅ |
| Size-0 attributes are server-only columns | inference | ❓ |
