# Client 381 reversing notes

These notes record what we reversed from the BnS client, build 0.0.381 (2020).
They support the server port. They contain no keys and no game data.

| File | Content |
| --- | --- |
| [`client-exe-reversing.md`](client-exe-reversing.md) | `Client.exe`: symbols, networking, command tables, packet layouts, framing, session classes, jobs, movement, summoner, mounts, fishing, open items |
| [`client-datafiles.md`](client-datafiles.md) | `datafile.bin`, `datafile64.bin`, `localfile.bin`: format, table definitions, record layout, loading |
| [`client-4047-crash.md`](client-4047-crash.md) | Error 4047 at launch: `pathengine.dll` null dereference |
| [`client-381-game-commands.tsv`](client-381-game-commands.tsv) | Game command IDs and names (1600 rows) |

Left out of this public copy:

- Client protection removal work. It is not published.
- Operator paths and addresses. Placeholders: `<server-ip>`, `<operator-downloads>`, `<ext>`, `<scratch>`.

❓ marks a fact that we did not verify.
