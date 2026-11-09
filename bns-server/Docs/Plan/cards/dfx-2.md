# Card dfx-2: Datafile book part 2 — schema, client loader and table use in code

## Goal
Write part of the Datafile book in `Docs/DataFile/book/`: one complete, deep reference for the BnS client data files (build 0.0.381.8884), merged from all our research plus new IDA work. It replaces the scattered notes as the place to read first.

## Work (3 topics)
1. Schema (TableDefs): where the definitions live in the exe, their binary layout, how `BnsSchema` exports them, versions (8876 vs 8884), and every element type. Write `Docs/DataFile/book/tables.tsv`: one row per table (name, id, record count, record size, key, number of fields, which file, 381 definition address).
2. Client loader in IDA: the full call flow from start to tables ready (`BNSClient::LoadData` 0x114bf80, `LoadXmlData`, DrElReaderImpl and the others): reading, layer removal, checks (hash, version, size), memory layout after load, errors and their messages. Give addresses for every step.
3. Table use in code: the record lookup API (by id, by alias, iteration), and for the 60 most used tables the client functions that read them (add columns `readers` and `reader_count` to tables.tsv). How a server reply id maps to a client table record.

## Sources to merge (read with `python3 tools/spec_index.py <file>`, then only the sections you need)
- `Docs/Client/client-datafiles.md` (research journal, about 12.5k tokens), `Docs/DataFile/01..10-*.md`, `Docs/DataFile/STATUS*.md`.
- Rust tools: `Program/Tools/DataFile/BnsData`, `BnsModel`, `BnsSchema`, `Program/Server/Data/Lmxb`, `Program/Server/Data/GameData`. Use `python3 tools/rust_api.py <crate dir>`; do not read Rust source files in full.
- Earlier book chapters in `Docs/DataFile/book/`.
- IDA: 381 `/home/ubuntu/Documents/Codex/private-project/ida/lanes/cx/Client.exe.i64` (main), 419 GM `/home/ubuntu/Documents/Codex/private-project/ida/lanes/cx/Client_gm.exe.i64` (names and types only). The loader may also be in `bsengine_Shipping.dll`; it has no IDA copy, so mark engine-side facts ❓ and list them.

## Style (the operator wants a complete, deep write-up)
- This is a book, not a note. Explain each thing so a new reader understands it without other files: what it is, why it exists, how it works, with byte-level examples.
- Every fact has a source (address, file offset, file, tool, test). Mark facts that you did not verify with ❓. Keep conflicts between sources visible, and say which source wins and why.
- Start each chapter with a "Quick find" table (question → section → address or file offset). Use diagrams (ASCII) for layouts and flows. Use hex dumps of real headers (no key bytes; write `keyhere`).
- Each chapter file stays under 40,000 bytes. Split into more files if needed.
- Do not delete the old docs. At the end of each old doc section that you merged, do not edit it; the book links to it as "journal".

## Out of scope
- No Rust card for this slice. No key values, no game text dumps, no copied table data beyond small examples. Do not change files under `Program/`.

## Files
- `Docs/DataFile/book/04-schema.md`
- `Docs/DataFile/book/05-client-loader.md`
- `Docs/DataFile/book/06-tables-in-code.md`
- `Docs/DataFile/book/tables.tsv`
- `Docs/DataFile/book/` (other new files of this part)
