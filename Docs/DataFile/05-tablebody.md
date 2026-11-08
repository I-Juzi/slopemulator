# Table body framing — build 348 guide, partial specification

## Scope

Source: `ida/datafile/gd348/GameDaemon.exe.i64`, with PDB names and types. These are server build 348 reader facts. **❓ They have not been checked on client 0.0.381.8884 bodies.** The operator metadata lists body lengths, not internal bytes. The 2020 server definitions do not match the client ID order. Do not use either exported server schema to decode client fields by ID.

No Rust `table` module was added in this step. The auto-key final-slot rule and exact body end rule remain unresolved. A safe implementation must not guess these rules.

## Source addresses

| Function | Address | Role |
| --- | --- | --- |
| BinaryDataFile::readFromMem | `0x1406fa370` | Selects kind, checks type and schema version, calls per-table reader |
| DrMultiKeyTable::ReadFromBuffer | `0x1406fc640` | Kind 2 body header, region copies, record map |
| DrAutoKeyTable::ReadFromBuffer | `0x1406fe7f0` | Kind 1 body header, region copies, slot map |
| DrElReaderImpl::RelocateAddress | `0x1406edcb0` | Adds extra-buffer base to stored string/native offsets |
| DrMultiKeyTable::constructExtraBuffer | `0x1406fc3f0` | Sums variable allocation sizes and calls RelocateMemory |
| DrAutoKeyTable::constructExtraBuffer | `0x1406fd4c0` | Builds extra buffer for non-null slots |
| DrElReaderImpl::RelocateMemory | `0x1406f0dc0` | Delegates to RelocateElMemory; detailed encoding not inspected |
| DrDataTableImpl::UseLegacyElHeader | `0x1406fd290` | Named legacy-header switch; activation not proved |

The outer container record is already specified in `02-tadboslb.md`. Its body-size u32 starts after kind, type, major, and minor. The dispatcher advances by this declared size. It does not require the table reader to consume the body exactly. The dispatcher supplies the remaining container buffer size, not just the current body size. A new parser must bound each table by its declared body size before it reads any header or region.

## Body headers

All integers use little-endian order. The following offsets start at the body pointer passed to ReadFromBuffer.

| Field | Kind 2 offset/size | Kind 1 offset/size (64-bit guide) |
| --- | --- | --- |
| Record count / slot count | 0 / u32 | 0 / u64 |
| `_tableSize`, fixed-region byte count | 4 / u32 | 8 / u32 |
| `_tableExtraSize`, extra-region byte count | 8 / u32 | 12 / u32 |
| `_uselowmemory` | 12 / u8 | 16 / u8 |
| Fixed region starts | 13 | 17 |
| Extra region starts | 13 + tableSize | 17 + tableSize |

✅ The guide copies each region separately. `_uselowmemory` controls reconstruction and relocation. These functions do not decrypt or decompress the regions.

❓ The 32-bit kind 1 count is expected to be u32, with a 13-byte header. This follows its pointer-sized type and the observed four-byte empty-body difference. It was not independently checked in a 32-bit reader.

❓ Exact body-end rule: the reader does not test equality between body length and header + tableSize + tableExtraSize. Operator metadata contains empty kind 2 bodies of 14 bytes and empty kind 1 bodies of 14/18 bytes. These are each one byte longer than the candidate header. The extra byte may belong to the extra region. Its ownership and value are not proved. Do not invent a terminator or discard a trailing byte.

## Record header and definition lookup

The PDB `DrEl` header occupies six meaningful bytes. Padding can follow it.

| Offset | Type | Meaning |
| --- | --- | --- |
| 0 | signed byte | Element type; -1 is an unused-entry marker |
| 1 | byte | Padding/reserved; meaning not proved |
| 2 | i16 | Subtype; negative means no subtype |
| 4 | u16 | Serialized fixed-record size |

The normal record base is the current fixed-region cursor. With `_useLegacyElHeader`, the DrEl base is cursor + 8. The reader then advances by DrEl.size + 8. **❓ The eight legacy bytes and activation rule are not specified.** They are not proved to be a key.

Select `tabledef.el[element_type].body`. A nonnegative subtype selects `body.sub[subtype]`. Base and subtype field offsets both start at the DrEl base. The base definition supplies the key offset. A record-size check must use the subtype's full size when present. A future parser must check type, subtype, size, field extents, and cursor bounds before it forms a record slice.

## Kind 2: explicit-key records

✅ `0x1406fc640` loops over the u32 count. It reads a u64 key at DrEl base + the selected element body's key offset. It inserts the record in the key map. After a successful insertion, it advances by the size in the DrEl header, plus the optional eight legacy bytes.

The key is part of the fixed record. It is not a separate framing field. A field marked `key` in the schema describes table indexing, not encryption. The reader does not establish uniqueness or end-of-region safety for malformed input. A new implementation must reject duplicate-key policy conflicts and size/count inconsistencies explicitly.

❓ Width-specific key representation and schema compatibility still need a matching target definition. This guide uses an eight-byte composite/index key, even when the first declared key field is smaller.

## Kind 1: automatic slots — unresolved count rule

✅ `0x1406fe7f0` allocates an element map with the header count. It starts its slot cursor at 1. It only inserts when slot < count and the record type is not -1. It advances the record cursor and subtracts record size on every iteration **except the last iteration**. It then requires the remaining fixed-region size to be zero.

This is not a proved ordinary record count. Slot zero is not inserted by this loop. The last iteration does not consume a record. **❓ The source and storage of that final marker, the relation between slot count and physical records, and the zero-count case require a writer or a real-body check.** Do not implement count ordinary records from this header. Do not silently change count by one.

Automatic keys come from slot positions, not a generic key field in every record. Empty/deleted slots can use element type -1. The exact serialized marker extent is not proved.

## Extra data, repeats, and relocation

✅ `0x1406edcb0` walks the base and subtype attribute lists. It skips deprecated attributes. For each repeat index, it uses field offset + index × item size. Repetition is inline storage with a schema-defined stride. It is not evidence of a general variable-array descriptor.

| Attribute type in build 348 | Pointer location | Read-time operation |
| --- | --- | --- |
| `DR_ATTR_STR` = 7 | Record base + field offset + repeat index × size | Add extra-buffer base to the stored 64-bit value |
| `DR_ATTR_NATIVE` = 26 | Same field base + 8 | Add extra-buffer base to the stored 64-bit value |

The base-field pass processes key fields only for element type 1. The subtype pass has its own list. The stored values in these locations are relative to the extra-buffer base, not the record or file base, when low-memory relocation applies. The Rust implementation must retain them as integers and checked slices. It must not turn untrusted file offsets into native pointers.

✅ Both constructExtraBuffer functions sum AllocedSize and call RelocateMemory for records. This confirms a separate variable-data allocation region. **❓ String termination, null-offset semantics, native payload layout, alignment, packing order, and variable-length array rules remain unproved.** RelocateMemory delegates to RelocateElMemory; that implementation was not inspected in this budget.

## Why widths differ

The kind 1 count itself changes width. String/native pointer slots also change width. This can change record padding, field offsets, fixed-region sizes, and extra-region placement. The server exports contain 64-bit offsets only. Do not subtract four from each record size or reuse the 64-bit definitions for 32-bit bodies. Kind 2 uses a u32 count in the 64-bit guide; its header does not grow merely because the executable is 64-bit.

## Implementation gate and next check

1. Obtain permitted definitions with the client ID order and versions for both widths.
2. Ask the operator for a read-only header/region check on the existing client files. Print counts and size totals only.
3. Prove the kind 1 slot/end-marker rule from a writer or a matching reader.
4. Prove the exact body-end rule and legacy-header state.
5. Add a bounded borrowed-slice record splitter with synthetic tests.
6. Decode fields only after the corresponding width-specific schema is verified.

No client acceptance signature, hash, or integrity check was encountered in these readers. No such boundary was bypassed. No key bytes or game-data contents were copied into this specification.
