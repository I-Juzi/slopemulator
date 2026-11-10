# Card dfx-4: one XML source for client and server data (dump, merge, rebuild)

## Goal
Write the spec for one merged XML data set that holds the client tables and the server tables together, and the Rust card for a tool that makes it and builds both sides from it.

## Context
- The game company made the client data (`datafile*.bin`, `localfile*.bin`, TADBOSLB) and the server data (`GameDaemon/data/*.xml`, LMXBOSLB binary XML) from one designer XML source ❓ (book 8.4.2). Each side got only its own fields.
- The client knows the server XML file names (`setupLoaderDef` patterns, `DataManager` ctor `0x5b6370`). It can load a table from XML (`constructDataOnlyForXML`, book 5.6).
- The operator heard this advice: "dump the client datafiles to XML and merge them with the server ones". The result is one editable source. One edit then changes both the client and the server, and the two sides cannot go out of sync (book 8.6).
- Version gap: the server data is `GameData-0.0.5002.50` / `GameDaemon-0.0.356.8492`. The client is 381. Some table versions differ (book 8.4.1). The merge must report this and not hide it.
- Existing tools: `Program/Tools/DataFile/BnsData` (TADBOSLB, `.dat`, LMX), `BnsSchema`, `BnsModel`, `Program/Server/Data/Lmxb` (LMXBOSLB), `Program/Server/Data/GameData`. Use `python3 tools/rust_api.py <crate dir>`.

## Work (4 topics)
1. **Client XML form (IDA).** How the client reads a table from XML: `constructDataOnlyForXML`, the XML reader, element and attribute names, how an attribute name maps to a table field, how keys, aliases, references and enums are written as text, and the `release-side` (or similar) attribute ❓. Compare with the server LMXBOSLB element form. Result: the exact XML form that both sides accept.
2. **Field mapping rules.** For a table that both sides have: match by table name and version (not ID), match records by key, and match fields by name. Define the classes client-only, server-only, both-equal, both-conflict, and version-gap. Define which side wins in each class, and why. Use 3 real tables as worked examples (for example `item`, `achievement`, `terrain`), from the book and the tools, not from guesses.
3. **Derived and hidden fields.** `DataManager::loadHiddenFieldFrom*` (52 in 381) fill fields from other tables after load (book 8.4.3). Find the rule for at least 5 of them. Rule for the merged XML: a derived field is not stored; the build step makes it again.
4. **Rebuild paths.** Merged XML → client `datafile*.bin`/`localfile*.bin` (and the `.dat` archive with its signature limit, book chapter 2), and merged XML → server LMXBOSLB files. List what the existing crates already do and what is missing. List the checks that prove a round trip (dump → merge → build gives byte-equal or field-equal output).

## Rust card (write it early, before topic 4 is complete)
Write `Docs/Plan/cards/dfx-4-rust.md` for a tool `bnsxml` (new crate `Program/Tools/DataFile/BnsXml`, or commands in an existing crate if the spec shows that it fits better). Commands: `dump-client`, `dump-server`, `merge` (writes the merged XML and a `merge-report.tsv` with one row per table: counts per class from topic 2), `build-client`, `build-server`, `verify` (round trip). Give exact spec line ranges, signatures and tests. Keep it to what one Sonnet session can build. If it is too large, write `dfx-4-rust.md` for dump and merge, and list build and verify as the next card in the handoff.

## Inputs
- Book: `Docs/DataFile/book/` chapters 2, 3, 4, 5, 8, 9 and `tables.tsv`. Use `python3 tools/spec_index.py <file>` and read only the sections you need.
- Server data survey: `Docs/Server/GameDaemon/01-lmxboslb.md`, `02-tables.md`.
- IDA: 381 `/home/ubuntu/Documents/Codex/private-project/ida/lanes/cx/Client.exe.i64` (main), 419 GM `/home/ubuntu/Documents/Codex/private-project/ida/lanes/cx/Client_gm.exe.i64` (names and types only).

## Style
- Book chapter style: "Quick find" table first, ASCII diagrams, a source for each fact, ❓ for facts that you did not verify, hex or XML examples (no key bytes: `keyhere`; no game text dumps, small examples only).
- Each file stays under 40,000 bytes.

## Out of scope
- Do not change files under `Program/`. Do not copy table data beyond small examples.

## Files
- `Docs/DataFile/book/10-unified-xml.md` (and `11-*.md` if needed)
- `Docs/DataFile/book/merge-rules.tsv`
- `Docs/DataFile/book/00-index.md` (add the new chapter rows only)
- `Docs/Plan/cards/dfx-4-rust.md`
