# dfx-2 handoff — datafile book part 2

## Done
- `Docs/DataFile/book/04-schema.md`: TableDefs at `0x194ece8` (363 slots), x86 layouts, module flags, tree tables, attribute types, key packing, versions, export notes.
- `Docs/DataFile/book/tables.tsv`: 362 rows, 19 columns; all 352 file tables match exe kind/version.
- `Docs/DataFile/book/05-client-loader.md`: flow `MainFramework` → `LoadXmlData` → `LoadData` → `DataManager::Load` → `0x624440`, checks, memory layout, errors.
- `Docs/DataFile/book/06-tables-in-code.md`: Find by key (`vtbl+0x64`), alias (`vtbl+0x60`), iterators, reader scan, top 60, packet id → record examples.
- `Docs/DataFile/book/00-index-part2.md`: index for part 2 (old `00-index.md` not changed).
- Part 1 ❓ 9 answered: `DrEl.type` = element index; attr +49 = deprecated.

## Not done
- Per-table record counts (`record_count` = `?`); needs a per-table walk.
- `tables.tsv` `readers` keeps 2 names per table (40,000-byte limit).
- `tools/rust_api.py` and `tools/spec_index.py` not run: Bash denied in this session.
- No Rust card (card says so).

## ❓
- Engine side (`bsengine_Shipping.dll`): archive open, RSA, AES, IO object (5.12).
- `XmlReader_cl32.dll` internals; x64 client flow.
- `sub_5BCE50`, `sub_622F80`, `GRuleConfig +0x30/+0x148` args, flag `0x1955c44`.
- Stub table methods; iterator order; readers the scan misses.

## Blocked
- None.
