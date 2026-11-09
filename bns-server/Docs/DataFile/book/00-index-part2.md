# The BnS client datafile book — part 2

Build: client data 0.0.381.8884; `Client.exe` 0.0.381 x86 (IDA copy `ida/lanes/cx/Client.exe.i64`).
Part 1 is in [00-index.md](00-index.md) (chapters 1–3). Part 2 adds chapters 4–6 and `tables.tsv`.

## Quick find

| Question | Go to |
| --- | --- |
| Where are the table definitions in the exe? | 4.2 (`0x194ece8`, 363 slots) |
| Binary layout of `DrTableDef`, `DrElDef`, `DrElBodyDef`, `DrAttrDef`, `DrSubDef` | 4.3 |
| Module flags | 4.4 |
| Tree tables (no binary table) | 4.5 |
| Attribute types, flags, key packing | 4.6 |
| Versions 8876 / 8884 / server | 4.7 |
| One row per table | 4.9, [tables.tsv](tables.tsv) |
| Call flow from start to tables ready | 5.0 |
| File names and where the file is read | 5.4, 5.7 |
| Checks on the file and on each table | 5.8 |
| Memory layout after load | 5.10 |
| Error codes and messages | 5.11 |
| Find a record by key / alias / iteration | 6.2, 6.3, 6.4 |
| Which code reads which table (top 60) | 6.5 |
| Server packet id → client record | 6.6 |

## Chapters

| File | Content |
| --- | --- |
| [04-schema.md](04-schema.md) | TableDefs in the exe, binary layout, element types, versions, `bnsschema` export |
| [05-client-loader.md](05-client-loader.md) | Client load flow with addresses, checks, memory layout, errors |
| [06-tables-in-code.md](06-tables-in-code.md) | Lookup API, reader scan, packet id to record |
| [tables.tsv](tables.tsv) | 362 rows: id, name, kind, version, file, sizes, key, fields, module, def address, readers |

## Part 1 open questions now answered

| Part 1 question | Answer | Source |
| --- | --- | --- |
| 9: `DrEl.type` meaning | element index (1 = `record`) | 4.3.2, 4.6.3 |
| 9: attribute byte at + 49 | `deprecated`; deprecated ⇔ size 0 (203 of 203) | 4.3.4 |

## Conflicts and which source wins

| Conflict | Winner |
| --- | --- |
| GM PDB: `DrAttrDef` is 68 bytes (with `dynamic`) | 381 array stride 64 (4.3.4) |
| Ported name `SaveBinaryDataFile` (reader `+0x3C`) | Code reads from memory (5.7) |
| Ported name "EnterReplay" for `0x1190f1e` | Code shows an error message (5.1) |

## Journal (old documents, not changed)

- `Docs/Client/client-datafiles.md`
- `Docs/DataFile/07-client-exe-tabledefs.md`, `10-schema-from-exe.md`, `STATUS-schema.md`, `STATUS.md`
