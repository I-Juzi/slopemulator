# dfx-1 handoff — Datafile book part 1

## Done
- `Docs/DataFile/book/00-index.md`: quick find, glossary, sources, conflicts, ❓ list, journal links.
- `01-what-and-where.md`: files, contents, makers, 32 vs 64, `.dat` disk layout, load chain (IDA).
- `02-container-and-protection.md`: all layers in order. New IDA facts: `.dat` (`UOSEDALB` v3) header, index and entry header; AES-128 ECB with a built-in key (`keyhere`, `0x7044d0`); RSA-1024 + SHA-256 signature (`0x7060b0`); client `TADBOSLB` reader checks only signature, version 5 and counts (`0x624440`). Rebuilt header hex example. Undo/redo table.
- `03-tables-records-strings.md`: body variants, `DrEl`, `DrAttrType` codes (PDB enum), string base confirmed in client `0x62e4f0`, native word `a` used as byte size by `AllocedSize` `0x62e030`, worked `map-area` → `text` example (synthetic values, real layout).
- No key values, no game text, no Hex-Rays code. No change under `Program/`. Old docs not changed.

## Not done
- No raw `.dat` or `.bin` hex dump: the files are outside the worktree. The header hex is rebuilt from journal values.
- No Rust `.dat` unpacker exists; the book describes the method only.
- GM `Client_gm.exe.i64` not opened (not needed).

## ❓
- Alias block form in 381; u16 constant 8; `_totalTableSize` / `_maxBufferSize` rules.
- Native word `a`: terminator included? Rust writer keeps `a`; check before editing longer text.
- AES padding byte, RSA padding mode, key blob hash input.
- x64 `.dat` and x64 client reader; `DrEl` bytes 6–7; types 11–32 layouts.
- Client acceptance of repacked tables.

## Blocked
- None.
