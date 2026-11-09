# The BnS client datafile book — part 1

Build: client data 0.0.381.8884; `Client.exe` 0.0.381 x86 (IDA copy `ida/lanes/cx/Client.exe.i64`).
This book replaces no old document. The old documents stay as the **journal** (the research log). The book gives the merged result, with a source for each fact.

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
| What the client checks in the container | 2.5.2 |
| Table frame, alias block | 2.5.3, 2.5.4 |
| Table compression | 2.6, 3.3 |
| Undo and redo of each layer | 2.7 |
| Record header, key, attribute type codes | 3.4, 3.5 |
| `string`, `native`, `ref`, size-0 fields | 3.6–3.9 |
| One full decoded record | 3.10 |

## Chapters

| File | Content |
| --- | --- |
| [01-what-and-where.md](01-what-and-where.md) | The files, their contents, who makes them, 32 vs 64, disk layout, load chain |
| [02-container-and-protection.md](02-container-and-protection.md) | All layers end to end: `.dat` archive, AES-128, RSA signature, zlib, `TADBOSLB`, table frame, undo/redo |
| [03-tables-records-strings.md](03-tables-records-strings.md) | Table bodies, records, type codes, string pools, `native`, `ref`, worked example |

Reading order: 1 → 2 → 3. For a tool writer: 2.1, 2.5, 2.7, then chapter 3.

## Glossary

| Term | Meaning |
| --- | --- |
| `.dat` archive | Pack file `xml.dat`, `local.dat`, `config.dat`; signature `UOSEDALB` (2.2) |
| `TADBOSLB` | Signature of `datafile.bin` / `localfile.bin` (2.5) |
| `LMXBOSLB` | Server binary XML signature (2.8) |
| main / local file | `datafile*.bin` / `localfile*.bin`; only the main file has the alias block |
| width | x86 (32-bit) or x64 (64-bit) layout; the caller selects it |
| kind | 1 = auto-key table, 2 = multi-key table |
| `DrEl` | 6-byte record header (type, subtype, size) |
| fixed region | the records of a plain table |
| extra region / extra span | string pool of a plain table / of one compressed record |
| `keyhere` | a key or secret value that this book does not write |
| ✅ / ❓ | verified with the given source / not verified |

## Sources

| Source | Use |
| --- | --- |
| IDA `ida/lanes/cx/Client.exe.i64` (381 x86, PDB names) | `.dat` layers, client reader checks, relocation, type enum |
| IDA `ida/datafile/gd348/GameDaemon.exe.i64` (server 348) | `TADBOSLB` x64 header, server readers |
| Operator walks of the four `.bin` files (counts only) | body variants, blocks, pools |
| Operator TableDefs 0.0.381.8876 x86/x64 | field layouts |
| BnsBinTool (no licence) | cross-check only, no code copied |
| Rust tests (`bnsdata` `tests/real_files.rs`) | round trip proof |

## Conflicts and which source wins

| Conflict | Winner |
| --- | --- |
| Server 348 table body has no flag byte; client body has one | Client walk and client code (3.1) |
| Early notes: header version is u32; PDB and client check: u8 | u8 (2.5.1) |
| Early notes: `datafile.bin` can be encrypted | Not encrypted; flags 0 and not read; encryption is on the `.dat` archive (2.1) |
| A file named `locale.bin` | Does not exist; the local files are `localfile*.bin` (1.1) |
| Journal: native word `a` meaning unknown | Client `AllocedSize` uses `a` as a byte size (3.7); exact rule ❓ |

## Open questions (❓)

1. Which alias block form the 381 files use (2.5.4).
2. Meaning of the u16 value 8 in compressed bodies (3.3).
3. Rules for `_totalTableSize` and `_maxBufferSize` (2.5.1); the x86 client does not read them.
4. Native word `a`: does it include the terminator (3.7)?
5. Does the client accept re-compressed tables and changed pools (2.7)?
6. x64 `.dat` header and x64 client reader (2.2.1, 2.5.1).
7. Padding of AES blocks; RSA padding mode; hash input of the key blob check (2.2.4, 2.4).
8. Which tool makes the files (1.3).
9. `DrEl` bytes 6–7 and type −1 (3.4.1); attribute byte at + 49 (3.6).
10. Layouts of types 11–19, 22–25, 27–32 (3.5).

## Journal (old documents, not changed)

- `Docs/Client/client-datafiles.md`
- `Docs/DataFile/01-lmxboslb.md`, `02-tadboslb.md`, `06-client-body.md`, `07-client-exe-tabledefs.md`, `08-strings.md`
- `Docs/DataFile/STATUS.md`
