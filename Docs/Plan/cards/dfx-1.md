# Card dfx-1: Datafile book part 1 — what it is, where it lives, how it is built and protected

## Goal
Write part of the Datafile book in `Docs/DataFile/book/`: one complete, deep reference for the BnS client data files (build 0.0.381.8884), merged from all our research plus new IDA work. It replaces the scattered notes as the place to read first.

## Work (3 topics)
1. Overview: what `datafile.bin`, `datafile64.bin`, `localfile.bin` (and `localfile64.bin`, `locale.bin` if it exists) are; what game data is in them; who makes them (NCSoft build pipeline from server XML / LMXBOSLB to client binary, as far as evidence shows); 32 vs 64 differences; where they live on disk (client folder paths, packaging inside .dat archives or upk if so), and how the client finds them.
2. Container format end to end: outer header, compression, encryption or obfuscation ("what they did to it"): each layer, the algorithm, where the key or seed comes from (`keyhere` for values), the order of the layers, and how we undo and redo each layer. Byte diagrams and a header hex example.
3. Inner format: dat1/dat2/table bodies, record layout, string pools, `native` and `string` encodings, link (`ref`) fields, size-0 and narrow fields. One full worked example: decode one small record from raw bytes to fields.

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
- `Docs/DataFile/book/00-index.md`
- `Docs/DataFile/book/01-what-and-where.md`
- `Docs/DataFile/book/02-container-and-protection.md`
- `Docs/DataFile/book/03-tables-records-strings.md`
- `Docs/DataFile/book/` (other new files of this part)
