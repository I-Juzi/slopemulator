# Card dfx-3: Datafile book part 3 — history, server side, tools and editing

## Goal
Write part of the Datafile book in `Docs/DataFile/book/`: one complete, deep reference for the BnS client data files (build 0.0.381.8884), merged from all our research plus new IDA work. It replaces the scattered notes as the place to read first.

## Work (3 topics)
1. Versions and history: 381 (2020) vs 419 GM (2021) vs BNSR 2026 (UE4 era): what changed in format, protection and schema; evidence for each change.
2. Server side: LMXBOSLB / TADBOSLB server data, how it relates to the client files (same tables, different fields), what our server must load and keep in sync (ids, versions), GameDaemon and other daemons' data use.
3. Tools and editing: the public tools (BnsBinTool, BnsPluginTables, BnsTools) and our Rust tools (`BnsData`, `BnsModel`, `BnsSchema`, `Lmxb`): what each does, how to read, edit, rebuild and verify a file safely, what the client checks on load, and a step-by-step "change one value in one table" walkthrough. Finish `00-index.md` with the full reading path and a glossary.

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
- `Docs/DataFile/book/07-versions-history.md`
- `Docs/DataFile/book/08-server-side.md`
- `Docs/DataFile/book/09-tools-and-editing.md`
- `Docs/DataFile/book/00-index.md`
- `Docs/DataFile/book/` (other new files of this part)
