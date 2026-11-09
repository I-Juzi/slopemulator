# dfx-3 handoff: datafile book part 3

## Done
- `Docs/DataFile/book/07-versions-history.md`: 381 vs GM 419 vs BNSR 2020/2026 vs server 348/2020. Header is the same (GM PDB, 99 B). Table counts are 362 / 385 / 519. Types 31–33. Struct sizes. The writer `WriteToFile` is named in GM.
- IDA 381: the exe version goes to `loadBinaryDataFile`, but `0x624440` does not get it. Thus `_gameVersion` is not checked (7.3).
- `08-server-side.md`: LMXBOSLB header (rebuilt hex, `keyhere`). Same names and versions, other IDs. Client XML name patterns (`0x5b6370`). Hidden-field loaders. Daemon data use. Sync list. `DrTableSummary` MD5.
- `09-tools-and-editing.md`: public tools (cross-check only), our crates and BnsEdit, the client load checks, 11 safe-edit rules, a 10-step walkthrough (item price), install and restore, server edit.
- `00-index.md`: all 3 parts, the reading paths, the full glossary, conflicts, open questions 1–16.
- Each file is under 40,000 bytes. No key bytes are written.

## Not done
- No Rust card (the card says "No Rust card for this slice").
- No running-client test of a packed file.

## ❓
- GM version constant; does GM/2026 check `_gameVersion`?
- The caller of `GetTableSummary` (vtable `0x1871588` +0x2C).
- World-entry data check on the server.
- Hidden-field loader rule (52 in 381, 62 in GM).
- BNSR 2020 UE4 `datafile*.bin` format; the 2026 `.dat` protection.
- Walkthrough: price field name, x64 exe name, header fields that `pack` changes.
- BnsEdit GUI flow was not run (API names only).

## Blocked
- None.
