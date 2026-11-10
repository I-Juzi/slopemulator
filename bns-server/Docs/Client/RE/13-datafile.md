# 13 — Client.exe 381: datafile (link page)

The datafile chapter of this reference is the Datafile book in [`Docs/DataFile/book/`](../../DataFile/book/00-index.md).
This page only links to it. It does not repeat its facts.
Old client-side notes stay in [`../client-datafiles.md`](../client-datafiles.md).
Classes of the reader in 381 that no other chapter names are listed in [17 §2.3](17-open-items.md#23-short-entries-for-large-classes-that-no-chapter-names).

| Task or question | Book chapter |
| --- | --- |
| What the data files are and where they live | [01-what-and-where](../../DataFile/book/01-what-and-where.md) |
| Container format and file checks | [02-container-and-protection](../../DataFile/book/02-container-and-protection.md) |
| Tables, records, strings, compressed bodies | [03-tables-records-strings](../../DataFile/book/03-tables-records-strings.md) |
| Table schema (tabledefs) | [04-schema](../../DataFile/book/04-schema.md) |
| Client loader: start to tables ready (`Data::DataManager::Load` `0x5bc500`, `loadBinaryDataFile` `0x5bca00`, checks `0x6243c0` / `0x624440`) | [05-client-loader](../../DataFile/book/05-client-loader.md) |
| Table access in code (get table, find by key or alias, reader scan) | [06-tables-in-code](../../DataFile/book/06-tables-in-code.md) |
| Versions and history | [07-versions-history](../../DataFile/book/07-versions-history.md) |
| Server-side data files | [08-server-side](../../DataFile/book/08-server-side.md) |
| Tools and safe edits | [09-tools-and-editing](../../DataFile/book/09-tools-and-editing.md) |
| All tables (one row each) | [`tables.tsv`](../../DataFile/book/tables.tsv) |

## Reader classes in 381 (from [`classes.tsv`](classes.tsv))

| Class | Functions | 381 first function | Role (from method names) |
| --- | ---: | --- | --- |
| `Data::DataManager` | 51 | `0x5b6370` (constructor) | Owns all tables. `Load`, `LoadHiddenField*`. |
| `DrDataReaderImpl` | 20 | `0x617480` | `LoadTable`, `LoadTableMT`, `ReadBinaryDataFile`, `LoadDocument` (XML fallback), alias maps. |
| `DrAliasMapImpl` | 28 | `0x624d20` | Alias (name) to record map: `Find`, `Insert`, `ReadFromBuffer`. |
| `DrCacheData` | 22 | `0x621100` | Table record cache: `BuildCacheData`, `compressChunk`, `SetTableCacheSize`. |
| `QuestTableImpl` | 20 | `0x403a90` | Quest table loader (`loadAcquisition`, `loadMission`, `loadCompletion`). Also see [10](10-quests-npc.md). |
| `DrElReaderImpl` | 24 | `0x619660` | Record element reader. `readNative` (`0x61cb90`) sets the native size to `2*chars+2` (old notes §8). |
