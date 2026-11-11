# 11 — Designer tools in the GM client (data load, reload, edit)

Source: `Client_gm.exe` (PE32, link 2021-05-24) and `Client_gm.pdb`. Branch 419NC. This build is newer than 381. It has 386 table definitions; build 381 has 362.

Mark ❓ = not verified.

## Summary

NCSoft designers edited the XML source of the data tables. The GM client loads that XML directly, checks it, reloads it while the game runs, and can pack it to `datafile.bin`. No Excel and no separate form editor occur in the client or the PDB.

```
XML source in ..\contents\local\<locale>\data\
        │
        ▼
GM client: Load (bin first, XML second) ──► validators
        │
        ├── reload: builder command 0x12, reload-table, auto-refresh-table
        ├── test:   Data Browser, Builder commands (GameBuilderSession)
        ▼
save-table datafile.bin  (❓ handler not found)
```

## Data load

| Fact | Address | State |
| --- | --- | --- |
| `Data::DataManager::Load` calls `loadBinaryDataFile` first | `0x6ab580`, `0x6acbd0` | ✅ |
| If the binary file does not load, it loads XML: `loadAlias`, `loadTable` (or `LoadAliasPieceByPiece` / `LoadTablePieceByPiece`), then `constructDataOnlyForXML` for all tables | `0x6ab580` | ✅ |
| Schema setup: `elreader->Setup(Data::TableDefs, 386, ...)` | `0x6ab580` | ✅ |
| After load: `constructData`, `DataValidator::CheckDataValidation`, `validateQuestData`, `validateCustomizingData` | `0x6ab580` | ✅ |
| File masks per table, for example `AchievementData*.xml`. One table = many XML files. | strings | ✅ |

## Reload

| Fact | Address | State |
| --- | --- | --- |
| `BNSClient::HandleBuilderCommand`, command type `0x12`: log "start local table reload", then `Data::DataManager::Reload` | `0x1426290` | ✅ |
| Data path: `BNSClient::GetDataDirectoryPath` gives `..\contents\local\<locale>\data\` | `0x13df1b0` | ✅ |
| `HandleBuilderCommand` is the only caller of `DataManager::Reload` | `0x6abca0` | ✅ |
| Name of builder command `0x12` is probably `reload-local-data` | string `0x19b0fc0` | ❓ |

## Debug commands (`DebugCommandDefList`, `0x16a4710`)

| Command | `_debuggerCommandType` | Argument |
| --- | --- | --- |
| `reload-table` | 0x4C | data table name |
| `reload-localize-table` | 0x4D | — |
| `auto-refresh-table` | 0x4E | y or n |
| `save-table` | 0x50 | [optional] file name, for example `datafile.bin` |

- Source: disassembly at `0x16AA61F`–`0x16AA770`. ✅
- The handlers are not in `BNSClient::HandleDebugCommand` (`0x1469540`). ❓ Location.
- `DrDataReaderImpl::SaveBinaryDataFile` is linked (`0x789a90`, vtable `??_7DrDataReaderImpl@@6B@` +0x40). ❓ It is the `save-table` path.

## Data Browser

| Fact | Address | State |
| --- | --- | --- |
| `DataBrowser::OnContextEdit` opens `DataBrowserEditingDialog` ("Edit Value") | `0x64cf10` | ✅ |
| It parses the text by attribute type and copies the value with `memcpy` into the record in memory. Then it calls `RefreshPropertyList`. | `0x64cf10` | ✅ |
| It does not write a file and does not send a packet | `0x64cf10` | ✅ |

## Builder session

`GameBuilderSession` is a separate client session for builder and QA commands to the server. The PDB has 155 symbols. Examples: SpawnNpc, SpawnZoneNpc, CompleteQuest, ResetQuest, SetAbility, InvokeEffect, MoveToArena, TestBot, UseQaCmd. ❓ Protocol and server side.

## Open items

1. Find the handlers for debug commands 0x4C–0x50.
2. Find the same features in the 2020 `Client.exe` (no PDB) by signature.
3. Find the server side of `GameBuilderSession`.
