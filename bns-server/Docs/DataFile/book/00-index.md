# The BnS client datafile book

Build: client data 0.0.381.8884; `Client.exe` 0.0.381 x86 (IDA copy `ida/lanes/cx/Client.exe.i64`).
This book replaces no old document. The old documents stay as the **journal** (the research log). The book gives the merged result, with a source for each fact.

The book has three parts:

| Part | Chapters | Index |
| --- | --- | --- |
| 1 | 1–3: files, container and protection, tables and records | this file |
| 2 | 4–6: schema, client loader, tables in code, `tables.tsv` | [00-index-part2.md](00-index-part2.md) |
| 3 | 7–9: versions and history, server side, tools and editing | this file |

## Quick find

| Question | Go to |
| --- | --- |
| What are `datafile.bin`, `datafile64.bin`, `localfile(64).bin`? Is there a `locale.bin`? | 1.1 |
| Where are the files on disk, and how does the client find them? | 1.5, 1.6 |
| What is 32-bit vs 64-bit? | 1.4 |
| Which layers protect the data, in which order? | 2.1 |
| `.dat` archive (`UOSEDALB`) header, index, entries | 2.2 |
| AES key source (`keyhere`) and mode | 2.3 |
| RSA signature check | 2.4 |
| `TADBOSLB` header and hex example | 2.5.1 |
| What the client checks in the container | 2.5.2, 5.8, 9.4 |
| Table frame, alias block | 2.5.3, 2.5.4 |
| Table compression | 2.6, 3.3 |
| Undo and redo of each layer | 2.7 |
| Record header, key, attribute type codes | 3.4, 3.5 |
| `string`, `native`, `ref`, size-0 fields | 3.6–3.9 |
| One full decoded record | 3.10 |
| Table definitions in the exe | 4.2, 4.3 (part 2) |
| Load flow, errors | 5.0, 5.11 (part 2) |
| Lookup by key and alias; packet id to record | 6.2–6.6 (part 2) |
| What changed between 381, GM 419 and 2026? | 7.2–7.8 |
| Is the game version in the header checked? | 7.3 (no) |
| Are table IDs the same in each build? | 7.9 (no; match by name) |
| What is LMXBOSLB? | 2.8, 8.2 |
| Client and server tables: what is the same? | 8.4 |
| What must our server keep in sync with the client? | 8.6 |
| Which tools exist; which can we use? | 9.1, 9.2 |
| Rules for a safe edit | 9.5 |
| Change one value in one table | 9.6 |
| Install and restore | 9.7 |

## Chapters and reading path

| File | Content |
| --- | --- |
| [01-what-and-where.md](01-what-and-where.md) | The files, their contents, who makes them, 32 vs 64, disk layout, load chain |
| [02-container-and-protection.md](02-container-and-protection.md) | All layers end to end: `.dat` archive, AES-128, RSA signature, zlib, `TADBOSLB`, table frame, undo/redo |
| [03-tables-records-strings.md](03-tables-records-strings.md) | Table bodies, records, type codes, string pools, `native`, `ref`, worked example |
| [00-index-part2.md](00-index-part2.md) | Index of part 2, answers to part 1 questions |
| [04-schema.md](04-schema.md) | TableDefs in the exe, binary layout, versions, `bnsschema` |
| [05-client-loader.md](05-client-loader.md) | Client load flow with addresses, checks, memory layout, errors |
| [06-tables-in-code.md](06-tables-in-code.md) | Lookup API, reader scan, packet id to record |
| [tables.tsv](tables.tsv) | 362 rows, one for each table |
| [07-versions-history.md](07-versions-history.md) | 381 vs GM 419 vs BNSR 2020/2026 vs server 348/2020, with evidence |
| [08-server-side.md](08-server-side.md) | LMXBOSLB server data, relation to the client tables, what to keep in sync |
| [09-tools-and-editing.md](09-tools-and-editing.md) | Public tools, our tools, safe edit rules, walkthrough, install and restore |

Reading paths:

| Reader | Path |
| --- | --- |
| New reader (all) | 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 |
| Tool writer (read and write files) | 1.4 → 2.1 → 2.5 → 2.7 → 3 → 4.3 → 5.8 → 9.3 → 9.5 |
| Data editor (change values) | 1.1 → 1.5 → 7.6 → 9.4 → 9.5 → 9.6 → 9.7 |
| Server developer | 1.1 → 2.8 → 7.9 → 8 → 6.6 → 9.8 |
| Researcher of a new build | 7 → 4.2 → 4.8 → 5.8 → 9.2.3 |

## Glossary

| Term | Meaning |
| --- | --- |
| `.dat` archive | Pack file `xml.dat`, `local.dat`, `config.dat`; signature `UOSEDALB` (2.2) |
| `TADBOSLB` | Signature of `datafile.bin` / `localfile.bin`; container version 5 (2.5) |
| `LMXBOSLB` | Server binary XML signature; version 3; strings XOR with `keyhere` (2.8, 8.2) |
| main / local file | `datafile*.bin` / `localfile*.bin`; only the main file has the alias block |
| width | x86 (32-bit) or x64 (64-bit) layout; the caller selects it |
| kind | 1 = auto-key table, 2 = multi-key table |
| table type / table ID | index of a table in the definition array of one build; not stable between builds (7.9) |
| table version | `major.minor` of a table; must be the same in the file and in the exe (5.8) |
| `_gameVersion` | u64 in the header; not checked by the 381 client (7.3) |
| `DrEl` | 6-byte record header (type, subtype, size) |
| `DrTableDef`, `DrElDef`, `DrAttrDef` | table, element and attribute definitions in the exe (4.3) |
| fixed region | the records of a plain table |
| extra region / extra span | string pool of a plain table / of one compressed record |
| `native` | string with a size word `a` and an offset; never change `a` (3.7) |
| `ref` | 8-byte key of a record in another table (3.8) |
| size-0 attribute | a deprecated field; it has no bytes (4.3.4) |
| `TIMEUTC`, `ASSET` | attribute types 31 and 32 in GM 419 and later (7.5) |
| `x64p4` | the 2026 x64 definition layout in `bnsschema` (7.6) |
| `DrTableSummary` | 72-byte table summary with three MD5 values; caller ❓ (8.7.2) |
| hidden field | a field that the client fills after load from other tables (8.4.3) |
| `release-module`, `release-side` | root attributes of a server table file (8.2.3) |
| `binreplace` | private client plugin that loads loose `.bin` files (9.7) |
| mirror | `bnsmodel` applies one edit to the x86 and the x64 file (9.2.2) |
| pack | write the changed project to a new folder (9.6) |
| round trip | read and write back with the same bytes (9.3) |
| journal | the old research documents; not changed |
| `keyhere` | a key or secret value that this book does not write |
| ✅ / ❓ | verified with the given source / not verified |

## Sources

| Source | Use |
| --- | --- |
| IDA `ida/lanes/cx/Client.exe.i64` (381 x86, PDB names) | `.dat` layers, client reader checks, version flow (7.3), XML names (8.4.2), MD5 summary (8.7.2) |
| IDA `Client_gm.exe.i64` (GM 419, full PDB; names and types only) | header struct, table count, type enum, struct sizes, writer name (ch. 7) |
| IDA `ida/datafile/gd348/GameDaemon.exe.i64` (server 348) | `TADBOSLB` x64 header, server readers |
| Operator walks of the four `.bin` files (counts only) | body variants, blocks, pools |
| Operator TableDefs 0.0.381.8876 x86/x64 | field layouts |
| Server data survey (`Docs/Server/GameDaemon/01-lmxboslb.md`) | LMXBOSLB format and counts (8.1, 8.2) |
| BnsBinTool, BnsPluginTables, BnsTools | cross-check only, no code copied (9.1) |
| `tools/rust_api.py` on our crates | tool APIs (9.2) |
| Rust tests (`bnsdata`, `bnsmodel`, `lmxb`) | round trip proof (9.3) |

## Conflicts and which source wins

| Conflict | Winner |
| --- | --- |
| Server 348 table body has no flag byte; client body has one | Client walk and client code (3.1) |
| Early notes: header version is u32; PDB and client check: u8 | u8 (2.5.1) |
| Early notes: `datafile.bin` can be encrypted | Not encrypted; flags 0 and not read; encryption is on the `.dat` archive (2.1) |
| A file named `locale.bin` | Does not exist; the local files are `localfile*.bin` (1.1) |
| Journal: native word `a` meaning unknown | Client `AllocedSize` uses `a` as a byte size (3.7); GM `readNative` size = 2 × chars + 2 (7.5) |
| GM `DrAttrDef` 68 B vs 381 stride 64 | 381 stride 64 for 381 data (4.3.4, 7.6) |
| Part 1 (3.9): size-0 fields are server-only | 4.3.4: size-0 fields are deprecated (8.4.4) |
| Client vs server table IDs | Neither; match by name (7.9) |
| TableDefs 8876 vs data 8884 | Same branch; versions match; use 8876 (7.11) |

## Open questions (❓)

Part 1 (some are answered in part 2; see [00-index-part2.md](00-index-part2.md)):

1. Which alias block form the 381 files use (2.5.4).
2. Meaning of the u16 value 8 in compressed bodies (3.3).
3. Rules for `_totalTableSize` and `_maxBufferSize` (2.5.1); the x86 client does not read them.
4. Native word `a`: does it include the terminator (3.7)? GM evidence says yes (7.5); 381 code not checked.
5. Does the client accept re-compressed tables and changed pools (2.7, 9.3)?
6. x64 `.dat` header and x64 client reader (2.2.1, 2.5.1).
7. Padding of AES blocks; RSA padding mode; hash input of the key blob check (2.2.4, 2.4).
8. Which tool makes the files (1.3)? GM has `BinaryDataFile::WriteToFile` (7.7).
9. `DrEl` bytes 6–7 and type −1 (3.4.1). (Byte at + 49: answered in 4.3.4.)
10. Layouts of types 11–19, 22–25, 27–32 (3.5).

Part 3:

11. GM version constant; does GM or 2026 check `_gameVersion` (7.12)?
12. Format of the BNSR 2020 UE4 `datafile*.bin` (7.12).
13. Who calls `GetTableSummary` (8.7.2)?
14. Does a server check client data at world entry (8.7.1)?
15. Rule of the hidden-field loaders (8.4.3).
16. Client acceptance of a packed file in a running client (9.9).

## Journal (old documents, not changed)

- `Docs/Client/client-datafiles.md`
- `Docs/DataFile/01-lmxboslb.md`, `02-tadboslb.md`, `03-tabledefs.md`, `04-tabledefs-2020.md`, `06-client-body.md`, `07-client-exe-tabledefs.md`, `08-strings.md`, `09-model.md`, `10-schema-from-exe.md`
- `Docs/DataFile/STATUS.md`, `STATUS-model.md`, `STATUS-schema.md`
- `Docs/Server/GameDaemon/01-lmxboslb.md`, `02-tables.md`, `STATUS.md`
