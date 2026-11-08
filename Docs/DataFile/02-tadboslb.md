# TADBOSLB version 5 — framing specification

## Step 2 implementation contract

The operator validated the framing on all four client files, build 0.0.381.8884. Each walk ended at the exact file size. Both container flags are zero. There is no container transform in these files. This supersedes the earlier uncertainty below about container-level compression and encryption. Table-internal content remains opaque.

The caller must select 32- or 64-bit width and main or local role. The header has no proved discriminator. Main files have the alias block when both alias size and alias count are nonzero. Local files have no alias block, even when their headers repeat the main alias metadata. Sources: operator validation in `STATUS.md`; build 348 `BinaryDataFile::readFromMem` at `0x1406fa370` explicitly skips the alias block when `isLocalData` is true. The caller at `0x1406ebac0` loads the main file before the local file.

The Rust writer updates table count, each table body length, and the size of a present main alias block. These fields directly bound the read loops and byte spans at build 348 `0x1406fa370` and `0x1406fa7b0`. It preserves `_totalTableSize`, `_maxBufferSize`, alias count, game version, save time, flags, and reserved bytes. The exact rules for the first two fields remain unknown. Thus replacing tables is supported at the framing level, not as a proved client-accepted repack. Alias bytes remain opaque; changing their content requires an independently correct alias count.

Nonzero container compression or encryption flags are unsupported. The reader rejects them. It does not guess transforms. Unknown table kinds, truncation, trailing bytes, and field-width overflow are errors. No client acceptance check was found or bypassed.

Operator tests use `BNSDATA_CLIENT_DAT`, plus `BNSDATA_DAT_WIDTH=32|64` and `BNSDATA_DAT_ROLE=main|local`. Run the ignored `client_roundtrip` test once per file. It compares bytes in memory and prints counts only. No key is required for this container.

The historical research sections below retain their original uncertainty markers. The operator checks above supersede the real-file validation and next-work sections.

## Scope and sources

Build: supplied `ida/datafile/gd348/GameDaemon.exe.i64`, GameDaemon build 348 with PDB types. This specifies the inspected SERVER reader. It does not yet prove that the supplied client files use the same body representation.

| Source | Address | Result |
| --- | --- | --- |
| `BinaryDataFile::readFromMem` | `0x1406fa370` | 119-byte packed header; alias block; sequential typed tables |
| `BinaryDataFile::readFromFileStream` | `0x1406fa7b0` | Same header and 11-byte table framing |
| `BinaryDataFile::ReadFromFile` | `0x1406fade0` | Reads bytes without a transform, then calls the memory reader |
| `DrDataReaderImpl::ReadBinaryDataFile` | `0x1406ebac0` | Loads main file before optional local file |
| `DrMultiKeyTable::ReadFromBuffer` | `0x1406fc640` | Raw element buffer and extra-data buffer |
| `DrAutoKeyTable::ReadFromBuffer` | `0x1406fe7f0` | Raw element buffer and extra-data buffer |
| `DrAliasMapImpl::ReadFromBuffer` | `0x1406f5fa0` | Two alias representations |
| `DrAliasStringTable::ReadFromBuffer` | `0x1406f3900` | Array plus string pool |
| `DrAliasMapImpl::WriteToBuffer` | `0x1406f33a0` | Stub: returns whether buffer is non-null |
| `SaveBinaryDataFile` virtual slot | `0x140723db0` | Points to `0x140455420`, which returns false |
| `BinaryDataFile::_gameDataBinaryDataFileHeader` initializer | `0x140721e90` | Clears reserved state |
| `BINARYDATAFILE_IDENTIFIER` symbol | `0x140723c70` | Signature symbol |
| `DataPacker::GAMEDATA_VERSION_FILENAME` symbol | `0x1407443b8` | Name exists; packer code not established |
| `DataPacker::GAMEDATA_BINFILENAME` symbol | `0x140744b68` | Name exists; packer code not established |
| `DrUtil::GenerateMD5Checksum` symbol | `0x1406fed30` | No container checksum call established |

Do not treat PDB method declarations as proof of a working packer. The `DrDataReaderImpl_vtbl` type puts `SaveBinaryDataFile` at byte offset 128. The vtable starts at `0x140723d30`. Its slot at `0x140723db0` points to a shared false-return function. IDA labels that function with an unrelated folded name. The target body is the evidence.

## Header

Integers use little-endian byte order. The header is packed. The version is ONE BYTE, not u32. Bytes 9 onward belong to game version.

The 64-bit layout is verified from PDB type `BinaryDataFile::_BinaryDataFileHeader` and both readers. The 32-bit layout below is ❓ inferred. It agrees with offsets reported in the operator's existing header notes. No 32-bit reader was inspected.

| Field | Width, 32 / 64 | Offset, 32 / 64 |
| --- | --- | --- |
| ASCII `TADBOSLB` | 8 / 8 | 0 / 0 |
| `_binaryDataFileVersion`, value 5 | 1 / 1 | 8 / 8 |
| `_gameVersion` | 8 / 8 | 9 / 9 |
| `_totalTableSize` | 4 / 8 | 17 / 17 |
| `_totalTableCount` | 4 / 8 | 21 / 25 |
| `_aliasSize` | 4 / 8 | 25 / 33 |
| `_aliasCount` | 4 / 8 | 29 / 41 |
| `_maxBufferSize` | 4 / 8 | 33 / 49 |
| `_saveTime` | 4 / 4 | 37 / 57 |
| `_compressed` | 1 / 1 | 41 / 61 |
| `_encrypted` | 1 / 1 | 42 / 62 |
| `_reserved` | 56 / 56 | 43 / 63 |
| End of header | — | 99 / 119 |

The field at 0x11 is total TABLE size, not total file size. The following count is a TABLE count. The supplied client datafile count 348 is not proof of 348 named inner XML files. The field at 37/57 is named `_saveTime`, consistent with the timestamps in the existing notes.

❓ The exact sum represented by `_totalTableSize` is not established by these readers. They use nonzero as a condition and advance by individual table lengths. Do not recalculate it from an unverified formula.

## Body framing and index

There is no separate named-entry index in the inspected server path. A reader can construct a structural index by scanning table headers.

1. Start at byte 119 in the verified 64-bit format.
2. Read the alias block of `_aliasSize` bytes when `_aliasCount` and `_aliasSize` are nonzero.
3. Read `_totalTableCount` table records in sequence.

Each table record starts with:

| Relative offset | Width | Field |
| --- | --- | --- |
| 0 | 1 | Table kind: 1 = auto-key; 2 = multi-key |
| 1 | 2 | Data type ID |
| 3 | 2 | Major schema version |
| 5 | 2 | Minor schema version |
| 7 | 4 | Body size, excluding this 11-byte header |
| 11 | body size | Table body |

A table has no stored filename or absolute offset here. Its offset follows from the preceding record sizes. It has no per-record compression flag in this header. Source: `0x1406fa370` and `0x1406fa7b0`.

The server matches the data type ID against `DrLoaderDef::tabledef`. It requires matching major and minor schema versions. These are format/schema compatibility conditions, not a discovered client integrity check. The client-side condition remains ❓.

## Table bodies in the inspected server reader

| Offset | Multi-key kind 2 | Auto-key kind 1 |
| --- | --- | --- |
| 0 | u32 element count | u64 count/map extent |
| 4 / 8 | u32 element-buffer size | u32 element-buffer size |
| 8 / 12 | u32 extra-buffer size | u32 extra-buffer size |
| 12 / 16 | u8 low-memory flag | u8 low-memory flag |
| 13 / 17 | Raw element buffer | Raw element buffer |
| next | Raw extra-data buffer | Raw extra-data buffer |

Source: `0x1406fc640` and `0x1406fe7f0`. The auto-key loop starts its map index at 1 and has special last-record handling. ❓ Do not equate its first u64 with a simple user-visible row count until that loop is fully checked.

PDB type `DrEl` has a type byte at 0, a subtype u16 at 2, and a size u16 at 4. The reader advances through elements using that size. `_useLegacyElHeader` adds an 8-byte prefix when active. Relocation uses the schema definitions and the extra buffer. ❓ Full element schemas, padding, relocation fields, and legacy selection remain open.

These are typed binary records. The inspected path does NOT parse inner LMXBOSLB documents. A server XML tree cannot be inserted as a table body without a schema-aware conversion.

## Alias block

`DrAliasMapImpl::_useStringTable` selects the representation. ❓ The selector is not established from the file header.

- Map representation: each alias stores a u64 key, u8 UTF-16 code-unit count, then that many UTF-16LE code units. Source: `0x1406f5fa0`.
- String-table representation: u32 array count, then 16 bytes per array item, then u32 pool byte size, then that many pool bytes. Source: `0x1406f3900`. The reader relocates each item's alias pointer/offset against the pool base. ❓ Individual item layout and pool encoding were not fully checked.

Preserve the complete alias block until its representation is known. Do not assume aliases name table files.

## Compression, encryption, checksums, and writing

The inspected file wrapper, memory reader, and table readers do not decrypt or decompress the input. The header has compressed and encrypted flags, but those flags do not select transforms in the inspected functions.

❓ The high-entropy supplied client bodies can use a different loader path or body variant. No zlib stream framing, AES mode, padding, IV, or key source was established. Do not implement a guessed transform. No encryption key or XOR table was located for this container. A future implementation must use `BNSDATA_KEY_FILE` for required runtime key material and must fail clearly without it.

❓ No working `SaveBinaryDataFile` or DataPacker writer was recovered in this build. No writer field order, alignment policy, checksum coverage, or MD5 placement was proved. The existence of an MD5 utility is not evidence of a checksum in this format.

No signature, hash, or integrity check that determines CLIENT acceptance was found in the inspected code. Client acceptance remains unknown. If later research finds such a check, record its address as a boundary and stop that branch. Do not infer that a header match is enough for acceptance.

## Real-file validation and next work

The existing operator notes report four client headers with signature, size-like fields, counts, and timestamps. This job did not independently validate those files. Shell reads required approval and returned no file results. No game body was copied into the repository.

Next: run read-only header checks on all four client files. Read a bounded alias prefix and one table header to test this server layout. Stop if the layout does not match. Find a working data packer build before specifying compression or encryption. Do not inspect excluded client executables or protection modules.
