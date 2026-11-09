# 8. Server side data

The client reads its tables from `datafile*.bin` and `localfile*.bin` (chapters 1–6). The servers read the same tables from other files. This chapter tells what these server files are, how they relate to the client files, and what our server must load and keep in sync.

## Quick find

| Question | Section | Address or file offset |
| --- | --- | --- |
| Which server files hold game data? | 8.1 | `GameDaemon/data/` |
| What is the LMXBOSLB header? | 8.2 | file offset 0x00–0x55 |
| Can the server read `TADBOSLB` too? | 8.3 | server 348 reader `0x1406fa370` |
| Are the server tables the same as the client tables? | 8.4 | `tables.tsv`; `Docs/DataFile/03-tabledefs.md` |
| Why do client and server XML file names look the same? | 8.4.2 | client `setupLoaderDef` calls in `0x5b6370` |
| Which daemon reads which data? | 8.5 | — |
| What must our server keep in sync with the client? | 8.6 | — |
| Does any server check the client data file? | 8.7 | LobbyGate protocol 58; client `0x624440` |
| What is `DrTableSummary` (MD5)? | 8.7.2 | client `0x62dcb0`; GM struct 72 B |

## 8.1 The server data files

The 2020 server set has a folder `GameDaemon/data/`. It has 21,065 `.xml` paths (`Docs/Server/GameDaemon/01-lmxboslb.md`).

| Item | Value | Source |
| --- | --- | --- |
| Files with the `LMXBOSLB` signature | 21,064 | full-folder survey, 01-lmxboslb.md |
| Plain XML files | 1 (`accountleveldata.xml`, UTF-8 with BOM) | same |
| Total binary size | 1,674,199,807 bytes | same |
| Elements / text nodes | 1,786,132 / 2,202,819 | same |
| Groups | root 1,098; quest 2,610; script 401; social 88; zone 16,867 | same |
| Packed copy | `GameData_RNCS.zip`, 23,074 entries | journal `Docs/Client/client-datafiles.md` |
| Version file in the zip | `gamedata_version.txt` = `GameData-0.0.5002.50` and `GameDaemon-0.0.356.8492`, then a base64 value | journal; base64 meaning ❓ |

The file names are lower case, for example `effectdata_*.xml`, `zone/2080/zoneenv2placedata.2080.xml`, `quest/questdata.25208.xml`.

```text
GameDaemon/data/
  accountleveldata.xml            <- plain XML (only one)
  terraindata.3040.xml            <- LMXBOSLB
  effectdata_*.xml ...            <- LMXBOSLB, root group (1,098)
  quest/questdata.<id>.xml        <- one quest per file (2,610)
  script/scriptdata.<zone>.xml    <- (401)
  social/...                      <- (88)
  zone/<zone id>/zone*data.<id>.xml  <- (16,867)
```

## 8.2 LMXBOSLB: the server binary XML

LMXBOSLB is a binary form of an XML tree. It is not a table file. One file holds one XML document. The strings in it are UTF-16LE, and a 16-byte XOR hides them. This book does not write the XOR bytes: `keyhere`. The same sequence starts again at each string.

### 8.2.1 Header

| Offset | Size | Field | Value seen |
| --- | --- | --- | --- |
| 0x00 | 8 | signature | ASCII `LMXBOSLB` |
| 0x08 | 4 | version | 3 (all files) |
| 0x0C | 4 | file size, with the header | equal to the file length (all files) |
| 0x10 | 64 | reserved | all zero (all files) |
| 0x50 | 1 | source flag | 1 (all files); meaning ❓ |
| 0x51 | 4 | length of the source name, in UTF-16 units | |
| 0x55 | 2 × length | source name, UTF-16LE with XOR (`keyhere`) | |

Source: `Docs/Server/GameDaemon/01-lmxboslb.md` (byte survey of all 21,064 files) ✅.

Example: the header of `terraindata.3040.xml` (789 bytes; name 20 units). This dump is rebuilt from the field table and the file size. The name bytes are XOR output, so the dump shows `xx` for them.

```text
offset    00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F
00000000  4C 4D 58 42 4F 53 4C 42  03 00 00 00 15 03 00 00   LMXBOSLB........
00000010  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00   (reserved, 64 B)
  ...     00 ...                                             (to 0x4F)
00000050  01 14 00 00 00 xx xx xx  xx xx xx xx xx xx xx xx   flag, len 20, name
  ...     (40 name bytes, XOR with keyhere)
0000007D  04 00 00 00 ...                                    root: 4 attributes
```

The root attribute count at 0x7D (= 0x55 + 40) is 4. This agrees with the offset table in 01-lmxboslb.md ✅.

### 8.2.2 Body

```text
file    = header[80] flag:u8 source:string root:element
string  = units:u32 + 2*units bytes (UTF-16LE, XOR keyhere)
node    = kind:u32  (1 = element, 2 = text) + body
element = attr_count:u32 (key:string value:string)*
          name_flag:u8 name:string child_count:u32 source_line:u32 node*
text    = value:string name_flag:u8 name:string("text")
          child_count:u32(=0) source_line:u32
```

The root has no `kind` field. There is no padding, no offset table and no compression. The maximum depth seen is 7 ✅ (01-lmxboslb.md).

### 8.2.3 The root element

Each table document has a root element with four attributes (`Docs/Server/GameDaemon/02-tables.md`):

| Attribute | Meaning | Example |
| --- | --- | --- |
| `release-module` | the data module of the table | `ItemData`, `ZoneData0`, `CombatData`, `AIData`, `LocalizationData` |
| `release-side` | who gets the data | always `server` in the GameDaemon copies |
| `type` | the table name | `terrain` |
| `version` | the table version, `major.minor` | `0.8` |

The table name and the version are the link to the client table. See 8.4.

## 8.3 The server can read `TADBOSLB` too

The server 348 has a `TADBOSLB` reader at `0x1406fa370` (`Docs/DataFile/02-tadboslb.md`). The x64 header is 119 bytes (2.5.1). The server body has no flag byte; the client body has one (3.1). The server 348 PDB also has the names `DataPacker::GAMEDATA_BINFILENAME` (`0x140744b68`) and `DataPacker::GAMEDATA_VERSION_FILENAME` (`0x1407443b8`). The packer code was not found ❓. The save slot is a stub that returns false.

Our 2020 server data is all LMXBOSLB. Thus we do not need the server `TADBOSLB` reader. The `bnsdata` crate has a server round trip test (`server_roundtrip`) for one LMXBOSLB file (see 9.3).

## 8.4 Same tables, other IDs and fields

### 8.4.1 What is the same and what is different

| Item | Client 381 | Server (348 / 2020) | Same? |
| --- | --- | --- | --- |
| Table names | `tables.tsv` | root `type` attribute; server definition array | ✅ same names |
| Table versions | `tables.tsv` | root `version` attribute | often equal; see the table below |
| Table IDs | array index, 381 build | array index, server build | ❌ different (7.9) |
| Kind (auto-key / multi-key) | `tables.tsv` | server definitions | 133 differ (2020 by ID; `04-tabledefs-2020.md`) ❓ by name |
| Fields | client definition | server definition | the server has fields that the client does not get |
| Form | binary table records | XML tree, one file per table or per zone | different |

Version check by name (client `tables.tsv` vs server data):

| Table | Client 381 | Server data | Source of the server value |
| --- | --- | --- | --- |
| `achievement` | 0.19 | 0.19 (1,558 records) | GameDaemon LMXBOSLB copy (journal) ✅ |
| `achievement-register` | 0.42 | 0.42 (1,067 records); 0.41 in the AchievementDaemon sample | journal ✅ |
| `rankingrewardgroup` | 0.6 | 0.6 (72 records) | journal ✅ |
| `terrain` | 0.8 | 0.8 | server 348 definition ✅ |
| `item` | 1.398 | 1.387 (server 348) | `03-tabledefs.md`; the 2020 data value ❓ |
| `season-contents-schedule` | no table in the file | 0.4 (3,012 records) | journal |

Rule: the table name and version identify a table. The ID does not.

### 8.4.2 The client knows the server file names

The client `DataManager` constructor (`0x5b6370`) fills one loader slot per table with `setupLoaderDef` (5.3.1). Each slot has an XML file name pattern. The patterns use the same names as the server files:

| Push address | Pattern |
| --- | --- |
| `0x5b652e` | `AcquireRouteData*.xml` |
| `0x5b65cc` | `CreatureAppearanceData*.xml` |
| `0x5b661b` | `EffectData*.xml` (server: `effectdata_*.xml`) |
| `0x5b6e8e` | `EquipGemPieceData*.xml` |
| `0x5b6edd` | `IconTextureData*.xml` |
| `0x5b6f2d` | `IndicatorSocialData*.xml` |
| `0x5b6f80` | `WeaponData*.xml;CostumeData*.xml;GroceryData*.xml;GemData*.xml;QuestItemData*.xml;AccessoryData*.xml` (the `item` table) |

`Data::DataManager::Load` (`0x5bc500`) calls `loadBinaryDataFile`, then `constructDataOnlyForXML`, then `constructData`, then `DataValidator::CheckDataValidation` ✅. The client can thus load a table from XML instead of the binary file (5.6). The game company made the client files and the server files from one XML source ❓ (inference from the names). The `release-side` attribute selects the fields for each side ❓.

### 8.4.3 Hidden fields

The 381 client has 52 functions `DataManager::loadHiddenFieldFrom*` (GM has 62). For example, `loadHiddenFieldFromFieldZone` is at `0x5bdfb0`. `constructDataOnlyForXML` calls it at `0x5bcd76`. These functions fill fields after load from other tables. The exact rule is ❓.

### 8.4.4 Size-0 attributes

Some client attributes have size 0. Part 1 (3.9) guessed that they are server-only columns. Chapter 4 (4.3.4) shows that all 203 size-0 attributes are `deprecated`. **4.3.4 wins:** a size-0 attribute is a deprecated field. It is not proof of a server-only field.

## 8.5 Who reads which data

### 8.5.1 The original 2020 daemons

| Daemon | Data | Form | Source |
| --- | --- | --- | --- |
| GameDaemon | 21,064 files in `data/` | LMXBOSLB v3 | 01-lmxboslb.md |
| AchievementDaemon | `AchievementData*.xml` (0.19), `AchievementRegisterData*.xml` (0.41) from its data path | plain XML | journal |
| RankingDaemon | `SeasonContentsScheduleData.xml` (0.4; 3,012 records; ArenaLobby copy 2,832), `RankingRewardGroupData` (0.6; 72 records) | plain XML | journal |
| LobbyGate | no table data seen; it checks protocol 58 in HelloReq | — | journal |

The GameDaemon folder also has LMXBOSLB copies of the achievement and ranking tables.

### 8.5.2 Our Rust server

| Crate or daemon | Reads | API | Source |
| --- | --- | --- | --- |
| `lmxb` | any LMXBOSLB file | `decode`, `encode`, `to_xml_string`, `element_count`; `MAX_DEPTH` = 128 | `tools/rust_api.py`; 01-lmxboslb.md |
| `gamedata` | 12 table kinds | `World::load_dir`, `World::from_documents`, `World::table`, `World::dangling_references`; `Table::by_id`, `by_alias`, `initial` | `tools/rust_api.py`; `Docs/Server/GameDaemon/STATUS.md` |
| AchievementDaemon | achievement data | uses `lmxb` (the only daemon that does) | `AchievementDaemon/Cargo.toml` |
| RankingDaemon | ranking data | plain XML | journal |

The `gamedata` load result (`Docs/Server/GameDaemon/STATUS.md`): 12 kinds, 10,608 files, 138,440 records.

| Kind | Records | Client table (version) |
| --- | --- | --- |
| `PcInitialSetting` | 93 | `pcinitialsetting` (0.12) |
| `zone` | 1,533 | `zone` (0.149) |
| `Terrain` | 164 | `terrain` (0.8) |
| `ZonePcSpawn` | 2,951 | `zonepcspawn` (0.49) |
| `ZoneRespawn` | 1,652 | `zonerespawn` (0.46) |
| `ZoneArea` | 76,936 | `zonearea` (0.55) |
| `ZoneEnv` | 98 | ❓ no client row seen |
| `ZoneEnv2` | 11,626 | `zoneenv2` (0.47) |
| `ZoneEnv2Place` | 11,682 | `zoneenv2place` (0.9) |
| `ZoneEnv2Spawn` | 11,832 | `zoneenv2spawn` (0.7) |
| `script` | 17,263 | ❓ |
| `quest` | 2,610 | `quest` (1.291; tree table, not in the binary file) |

The load found 8,500 links with no target (`STATUS.md`). Some are real gaps in the data ❓.

## 8.6 What our server must keep in sync

The client and the server talk with IDs and keys. The client finds the record for an ID in its own tables (6.6). If the two sides disagree, the client shows the wrong thing or nothing.

| Item | Why | How to check |
| --- | --- | --- |
| Record keys (for example `item` id + level, `achievement` id + step + job) | packets carry keys; the client looks them up (6.2, 6.6) | compare key sets of the server XML and the client table (`tables.tsv` column `key`) |
| Aliases | some packets and scripts use alias names (`Table::by_alias`) | compare alias lists; the client alias block is in the main file only (2.5.4) |
| Table versions | a different version means a different field list | compare root `version` with `tables.tsv`; see 8.4.1 |
| Text IDs | the client text is in `localfile*.bin` (`text`, 0.6) | the server sends IDs only |
| Zone, spawn and terrain IDs | the client loads the map for a zone ID | `gamedata` kinds vs client `zone`, `terrain` |
| Initial character setup | race, job and level keys | `pcinitialsetting` (key race, job, level) |

If you change a value that both sides use (for example an item price), change it in both places. Chapter 9 shows the client side. For the server side, edit the server XML (or decode, edit and encode with `lmxb`).

```text
              same source XML (❓)
               /                \
  client build (release-side=client)   server build (release-side=server)
       |                                     |
  datafile*.bin / localfile*.bin        GameDaemon/data/*.xml (LMXBOSLB)
       |                                     |
  Client.exe (table by name+version)    our Rust server (lmxb, gamedata)
       \______ packets carry IDs and keys ______/
```

## 8.7 Version checks between client and server

### 8.7.1 What is checked

| Check | Where | Result |
| --- | --- | --- |
| Protocol version | LobbyGate HelloReq, protocol 58 | the only version check seen (journal) ✅ |
| Client file `_gameVersion` vs exe version | client `0x624440` | not checked (7.3) ✅ |
| Table version vs compiled definition | client `0x624440` | checked; a mismatch fails the whole file (5.8) ✅ |
| Server compares the client data file | — | not found ❓; the world-entry path was not searched |

### 8.7.2 `DrTableSummary` and MD5

The GM PDB has a struct `DrTableSummary` (72 bytes):

| Offset | Field |
| --- | --- |
| 0 | `tableType` |
| 4 | `tableElementCount` |
| 8 | `tableElementEffectiveCount` |
| 12 | `tableElementSize` |
| 16 | `tableTotalSize` |
| 20 | `tableExtraSize` |
| 24 | `keyChecksum[16]` (MD5) |
| 40 | `tableChecksum[16]` (MD5) |
| 56 | `extraChecksum[16]` (MD5) |

In 381, `DrUtil::GenerateMD5Checksum` (`0x62dcb0`) has callers only in the `GetTableSummary` functions: QuestTableImpl `0x620da0`, DrMultiKeyTable `0x622c40`, DrAutoKeyTable `0x62d250`. `GetTableSummary` is vtable slot 11 (`+0x2C`) of the multi-key vtable `0x1871588`. A scan found no direct caller ❓. A summary of this form can let a client and a server compare their tables. We did not find this exchange ❓. The load does not use it.

## 8.8 Open questions

1. Meaning of the LMXBOSLB source flag and name flag (8.2).
2. The base64 value in `gamedata_version.txt` (8.1).
3. Who calls `GetTableSummary`; does a packet carry it (8.7.2)?
4. Does the world-entry path check client data (8.7.1)?
5. The rule of the hidden-field loaders (8.4.3).
6. The `item` version in the 2020 server data (8.4.1).
7. `ZoneEnv` and `script`: which client tables match (8.5.2)?

## Journal (old documents, not changed)

- `Docs/Server/GameDaemon/01-lmxboslb.md`, `02-tables.md`, `STATUS.md`
- `Docs/DataFile/01-lmxboslb.md`, `02-tadboslb.md`, `03-tabledefs.md`, `04-tabledefs-2020.md`
- `Docs/Client/client-datafiles.md`
