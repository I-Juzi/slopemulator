# Client 381 reversing notes

These notes record what we reversed from the BnS client, build 0.0.381 (2020).
They support the server port. They contain no keys and no game data.

Start here: [`RE/INDEX.md`](RE/INDEX.md) — chapter list, "how to find anything" guide (grep the TSV files, then `idc.jumpto(0x…)` in IDA) and the A–Z keyword index.

| File | Content |
| --- | --- |
| [`RE/INDEX.md`](RE/INDEX.md) | Entry point and A–Z index of the full reference |
| [`RE/`](RE/) | The full reference: chapters 00–17 (`NN-*.md`) and their TSV files. Chapter 18 (GM debugger) is not written yet. |
| [`RE/17-open-items.md`](RE/17-open-items.md) | Coverage audit, all open items (❓) with next actions, facts moved from the old notes |
| [`client-exe-reversing.md`](client-exe-reversing.md) | Short overview of `Client.exe`: client map, where each old section went, key facts for the port |
| [`client-datafiles.md`](client-datafiles.md) | `datafile.bin`, `datafile64.bin`, `localfile.bin`: format, table definitions, record layout, loading. Full book: [`../DataFile/book/`](../DataFile/book/00-index.md) |
| [`client-4047-crash.md`](client-4047-crash.md) | Error 4047 at launch: `pathengine.dll` null dereference |
| [`client-381-game-commands.tsv`](client-381-game-commands.tsv) | Game command IDs and names (1600 rows). Full table with handlers: [`RE/game-commands.tsv`](RE/game-commands.tsv) |

Left out of this public copy:

- Client protection removal work. It is not published.
- Operator paths and addresses. Placeholders: `<server-ip>`, `<operator-downloads>`, `<ext>`, `<scratch>`.

❓ marks a fact that we did not verify.
