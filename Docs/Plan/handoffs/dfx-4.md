# Handoff dfx-4: one XML source for client and server data

## Done
- `Docs/DataFile/book/10-unified-xml.md`: 10.1 data flow; 10.2 client XML form (element and attribute names, repeat names `name-n`, value text per type, ref forms, alias, merged file form, 7 quirks); 10.3 table/record/field match, classes, worked examples `terrain`, `achievement`, `item`; 10.4 derived fields (281 hidden attrs, 38 loaders, 6 rule rows); 10.5 rebuild paths and round-trip checks R1–R7; 10.6 open questions.
- `Docs/DataFile/book/merge-rules.tsv`: 15 class rows (table, record, field) with winner and report column.
- `Docs/DataFile/book/00-index.md`: 3 Quick find rows, 2 chapter rows, 2 conflict rows (52 → 38 loaders, 143 → 281 hidden attrs).
- `Docs/Plan/cards/dfx-4-rust.md`: new crate `bnsxml`, commands dump-client, dump-server, merge, with `merge-report.tsv` and `conflicts.tsv`.

## Not done
- build-client, build-server and verify are too large for one Sonnet session. Next card `dfx-5-rust`: record encoder from field text plus string pool (10.5.1), derived-field rules (10.4.2), LMXBOSLB `Document` from merged XML (10.5.2), checks R1–R6 (10.5.3).
- Real merge-report counts for `terrain`, `achievement`, `item`: the Rust session writes them (test `real_dump_merge`).

## ❓
- Int range compare signed or unsigned (`quirk_int_range_unsigned`, `sub_61B430`).
- Zone list field of the 17 attraction loaders; `skill3` write conditions (`0x5bd020`); match rule in `0x5c34e0`.
- Root `type`/`version` not read by the client XML code; `XmlReader_cl32.dll` not in IDA.
- Binary record order (sorted by key or file order).
- The original packer computed the hidden values (packer not in our files).
- GameDaemon dependence on attribute order and `source_line`.

## Blocked
- Bash was denied. `tools/spec_index.py` and `tools/rust_api.py` did not run. I used Grep for headings and `pub` lines only (no Rust bodies read). Chapter size was estimated (255 lines), not measured with `wc -c`; the gate must check < 40,000 bytes.
- No IDA changes; the database was closed without a save.
