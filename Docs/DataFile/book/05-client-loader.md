# Chapter 5 — The client loader: from start to tables ready

Build: `Client.exe` 0.0.381 x86, IDA copy `ida/lanes/cx/Client.exe.i64` (PDB names from the GM build, ported).
All addresses are in this copy. ✅ = verified in this copy. ❓ = not verified.
Some PDB names are ported from another build. When a ported name does not agree with the code, this chapter gives the behaviour and marks the name "wrong port".

## Quick find

| Question | Section | Address |
| --- | --- | --- |
| Where does the load start? | 5.1 | `BNSClient::MainFramework` call at `0x1190b66` |
| Which DLL reads XML, and which functions does the client get from it? | 5.2 | `0x116fca7` |
| Where are the table slots made (one per table)? | 5.3 | `0x5b6000`, `0x5b6370`, `0x5bc1f0` |
| How are the file names made? | 5.4 | `0x114c03a`, `0x114c062` |
| Main load function | 5.5 | `Data::DataManager::Load` `0x5bc500` |
| What happens when the binary file fails? (XML fallback) | 5.6 | `0x5bc500`, `0x618a80` |
| How is the file read from the archive or from the disk? | 5.7 | `0x5bca00` |
| Which checks are on the file, the header and each table? | 5.8 | `0x624440`, `0x624130` |
| How does a table body go into memory? | 5.9 | `0x6228b0`, `0x62ce60`, `0x62f870` |
| Memory layout after load | 5.10 | `dword_1A4D5AC` |
| Error codes and messages | 5.11 | `0x3E8`, `0x2711` |
| What is in the engine DLL (not verified)? | 5.12 | — |

## 5.0 Call flow (overview)

```
BNSClient::Initialize
  LoadLibraryW "XmlReader_cl32.dll" ............ 0x116fca7   (5.2)
  GetProcAddress x4 -> 0x1A16B94..0x1A16BA0
BNSClient::MainFramework
  BNSClient::LoadXmlData 0x114c9f0 ............. call at 0x1190b66
    QueryPerformanceCounter (load time)
    BNSClient::LoadData("", "") 0x114bf80 ...... call at 0x114ca7a
      sub_5B6000 (make DataManager) ............ call at 0x114bfbb  (5.3)
        DataReader setup, version "34" check
        operator new(0x5024) -> DataManager ctor 0x5b6370
          setupLoaderDef 0x5bc1f0  x358
      "%s%s" prefix + "datafile.bin" ........... 0x114c03a       (5.4)
      [locale flag] prefix + "localfile.bin" ... 0x114c062
      Data::DataManager::Load 0x5bc500 ......... call at 0x114c0b0 (5.5)
        CreateXmlReader (main IO, local IO)
        ElReader setup (0x194ece8, 363)
        aliasMap UseLegacyCode(0)
        loadBinaryDataFile 0x5bca00 ............ (5.7)
          read file into buffer (IO object)
          DrDataReaderImpl +0x3C -> 0x6243c0 -> 0x624440 (5.8)
            header check 0x624130
            alias block -> aliasMap
            per table frame: version check, new table, ReadFromBuffer (5.9)
        [binary OK]  LoadDocument (XML for tables not loaded) (5.6)
                     LoadTable per table (0x5bc9c0)
        [binary bad] constructDataOnlyForXML (all tables from XML)
        constructData 0x5bcca0 (hidden fields, MapUnitRelationTable)
        CheckDataValidation (if validator is set)
        DestroyXmlReader
        sub_5BCE50 ❓
        per table GetTableCacheInfoLogStr (log)
  false -> error 0x2711 (10001) ................ 0x1190f1e      (5.11)
```

The engine (`bsengine_Shipping.dll`) opens the `.dat` archives and gives the IO objects. The client code above does not decrypt. See 2.1 for the layers and 5.12 for the engine part.

## 5.1 Start

| Step | Address | Behaviour |
| --- | --- | --- |
| 1 | `0x1190b66` in `BNSClient::MainFramework` | Calls `BNSClient::LoadXmlData`. ✅ |
| 2 | `0x114c9f0` `BNSClient::LoadXmlData` | Reads `QueryPerformanceCounter` (start time). Makes two empty path prefixes. Calls `LoadData(prefix, localPrefix)` at `0x114ca7a`. ✅ |
| 3 | result false | `MainFramework` sets error code `0x2711` (10001) and goes to `0x1190f1e`. That block calls `ClientSystemText::SetAdditionalMsg`. The ported name of the target ("EnterReplay") is a wrong port. ✅ code, ❓ name |

The two prefixes are empty in this build. Thus the client gives the bare names `datafile.bin` and `localfile.bin` to the IO objects (5.4). The IO objects find the names in the archives (1.6). ❓ for the archive lookup itself (engine).

## 5.2 The XML reader DLL

`BNSClient::Initialize` loads the XML reader DLL. ✅

| Address | Behaviour |
| --- | --- |
| `0x116fca7` | `LoadLibraryW("XmlReader_cl32.dll")` |
| after | `GetProcAddress` for `GetInterfaceVersion` → `0x1A16B94`, `SetMemAllocator` → `0x1A16B98`, `CreateXmlReader` → `0x1A16B9C` (store at `0x116fcf5`), `DestroyXmlReader` → `0x1A16BA0` |
| `0x116fd28` | A null DLL or a null function gives error code `0x3E8` (1000). |
| `0x116e2f3` | Cleanup sets the four pointers to zero. |

`DataManager::Load` calls `CreateXmlReader` two times: once for the main IO object and once for the local IO object (5.5). The reader object opens a file through the IO object. The internals of `XmlReader_cl32.dll` are not in this IDA copy ❓.

## 5.3 DataManager creation and loader slots

`BNSClient::LoadData` (`0x114bf80`) calls `sub_5B6000` at `0x114bfbb`. Arguments: path, the main IO object (client object slot `+0x1FC`) and the local IO object (slot `+0x204`). ✅

`sub_5B6000` (`0x5b6000`) does these steps only if the global `dword_1A4D5AC` (the DataManager) is null: ✅

1. Fills a function table for the data reader: `SetupDataReader`, `CloseDataReader`, `CreateDataReader`, `ReleaseDataReader`, logger (`0x1A39678`..`0x1A3968C`). Sets `dword_1A4D5B0` = 18.
2. Gets the data-reader interface version string and compares it with `"34"` (`sub_617130`, `wcsicmp`). A different string is a failure.
3. Calls `SetupDataReader(100, &0x1A925C4, 0, 0)`.
4. Makes the reader, the alias map and the el reader.
5. `operator new(0x5024)` (20,516 bytes) and the constructor `0x5b6370`. The result goes into `dword_1A4D5AC`.

Failure → `LoadData` writes the message `failed to initialize data manager.` through `sub_5D0860` (log, with the source path `Client.cpp`) and returns false. ✅

### 5.3.1 The constructor and `setupLoaderDef`

The constructor `0x5b6370` has 358 calls to `setupLoaderDef` (`0x5bc1f0`). Each call uses registers: `edx` = the `DrTableDef` (4.2), `esi` = `&_loaderDefs[type]` = `this + 52 + 52 × type`. Each call also pushes an XML file glob, for example `AcquireRouteData*.xml`. After the call, the constructor can add a check function with `DataValidator::AddValidationFun`. ✅

17 tables also keep the loader slot address in a global, for example `dword_1B3598C` (acquire-route), `0x1b22810` (effect), `0x1b25534` (equip-gem-piece). ✅

The 363 − 1 − 358 = 4 types with no `setupLoaderDef` call: `moveanimtransit` 210, `questrewardskill3` 241, `season-contents` 252, `zoneevent` 352. They also are not in the files (4.7). ✅ (from 4.7 scan)

### 5.3.2 One loader slot (`DrLoaderDef`, 52 bytes)

```
+0   table          DrDataTable*   (set by the load)
+4   tabledef       DrTableDef*    (4.2)
+8   path           wchar*
+12  xml            XML glob (example "AcquireRouteData*.xml")
+16  subdir
+20  xsd
+24  postproc       function after load
+28  xmldoc
+32  l10ndoc
...
+44  loaded         bool
+48  tableCacheSize
```

Source: GM PDB layout, checked against the store offsets in `0x5bc1f0` and the `loaded` reset in `0x6243c0`. ✅ for `+0`, `+4`, `+44`; ❓ for the other field positions in 381.

## 5.4 File names

In `BNSClient::LoadData` (`0x114bf80`): ✅

| Address | Behaviour |
| --- | --- |
| `0x114c03a` | `StringCchPrintfW("%s%s", prefix, "datafile.bin")` (string at `0x13f3b7c`) |
| `0x114c062` | If the byte `0x1955c44` is 1: `"%s%s"` with `localfile.bin` (string at `0x13f3b60`). Else the local name is empty. |
| — | `GetClientVersion` |
| `0x114c0b0` | `Data::DataManager::Load(...)`. Two bool arguments come from `GRuleConfig + 0x30` and `GRuleConfig + 0x148 == 0`. ❓ for their meaning. |
| fail | Log message `failed to load data.`; returns false. |

The x86 client uses the names without `64`. The x64 client uses `datafile64.bin` and `localfile64.bin` (1.4) ❓ for the x64 code (no IDA copy).

## 5.5 `Data::DataManager::Load` (`0x5bc500`)

Steps in order: ✅

| # | Behaviour |
| --- | --- |
| 1 | `CreateXmlReader` for the main IO and for the local IO. Opens the reader through `vtbl[0]` with a file filter. |
| 2 | El reader setup: `(**elreader)(elreader, 0x194ece8, 363, cfg, 1)`. The el reader now knows all 363 definitions (4.1). |
| 3 | Alias map `UseLegacyCode(0)` (alias map `vtbl + 0x24`). |
| 4 | `loadBinaryDataFile` (`0x5bca00`, 5.7). |
| 5a | `_recordCountPerPiece` (`+20508`) ≤ 0 and binary OK: reader `+0x20` `LoadDocument` (5.6), then for each table `vtbl + 0x30`, then `sub_5BC9C0`: `timeGetTime`, reader `+0x24` `LoadTable(xmlReader, elreader, defs, count, aliasMap)`. |
| 5b | `_recordCountPerPiece` > 0: piece path; reader `+0x1C` (load alias, piece by piece) and `+0x28` (load table, piece by piece). |
| 6 | A slot with no table gets the stub table `&off_1981EF8` (vtable `0x17c86c8`). Thus a lookup on that table gives no record and does not crash. |
| 7 | Binary failed: `constructDataOnlyForXML` for all types (5.6). |
| 8 | `constructData` (`0x5bcca0`): `loadHiddenFieldFromDungeonQuestRelated`, `…TencentQuest`, `…MasteryLevel`, `…Quest`, `Log::Write`, `MapUnitRelationTable` constructor. |
| 9 | `CheckDataValidation` only if `_valiator` (`+20496`) is not null. |
| 10 | `DestroyXmlReader` for both readers. |
| 11 | `sub_5BCE50` when the 6th argument is false and the binary is OK. ❓ behaviour. |
| 12 | For each table: `vtbl + 0x38` `GetTableCacheInfoLogStr` into a 1024-character buffer (log). |

## 5.6 XML fallback (`LoadDocument`)

Two cases use XML files:

1. **Binary OK.** `LoadDocument` (`0x618a80`, reader `+0x20`) and `LoadTable` (`0x6180b0`, reader `+0x24`) run for the tables that are **not** loaded from the binary (slot `loaded` = false). These are the 6 tree tables (4.5) and each table whose body failed (5.8 step 9). ✅ for the call; ❓ for the exact skip test in `0x618a80`.
2. **Binary failed.** `constructDataOnlyForXML` loads all types from XML. ✅

The XML files come from the XML reader DLL (5.2) through the IO object, with the glob from the loader slot (5.3.2). The retail client has no loose XML files, thus this path only works if the archive has the XML ❓.

## 5.7 Read the file (`loadBinaryDataFile`, `0x5bca00`)

✅

```
if localName != "" and no local IO and !PathFileExistsW(localName): localName = ""
main IO present:
   io.open(filter, "datafile.bin")
   size = io.vtbl+8 ; buf = new[size] ; io.vtbl+4 read
   reader.vtbl+0x3C(buf, size, elreader, defs, count, isLocal=0, aliasMap, ...)
   free buf ; io.vtbl+0x14 close
   failure -> return 0
local IO present: same, isLocal=1
no IO:  reader+0x34 ReadBinaryDataFile (disk path, 0x618210 -> 0x6241d0:
        CreateFileW, GetFileSize, ReadFile)
     or reader+0x38 ReadBinaryDataFileStream (0x618280 -> 0x624310), chosen by bool arg 5
```

Reader `+0x3C` has the ported name `SaveBinaryDataFile` (`0x6182f0`). The code reads from memory (`0x6243c0` → `0x624440`). The name is a wrong port. ✅

### 5.7.1 DrDataReaderImpl vtable (`0x1870f88`)

| Offset | Address | Behaviour |
| --- | --- | --- |
| +0x00 | `0x6174f0` | destructor ❓ |
| +0x04 | | ReleaseElReader |
| +0x08 / +0x0C | | CreateAliasMap / ReleaseAliasMap |
| +0x10 / +0x14 | | CreateQuestTable / ReleaseTable |
| +0x1C | `0x617a10` | load alias piece by piece |
| +0x20 | `0x618a80` | LoadDocument |
| +0x24 | `0x6180b0` | LoadTable |
| +0x28 | `0x618810` | load table piece by piece |
| +0x2C / +0x30 | | LoadAliasMT / LoadTableMT |
| +0x34 | `0x618210` | read file from disk |
| +0x38 | `0x618280` | read file from stream |
| +0x3C | `0x6182f0` | read file from memory (ported name wrong) |

## 5.8 Checks (`0x6243c0`, `0x624440`)

`0x6243c0` calls `0x624440`. If `0x624440` fails, `0x6243c0` clears the alias map (`vtbl + 12`), releases each table of kind 1 or 2 (reader `+0x14`) and sets `loaded` = false. ✅

`0x624440` does these steps: ✅

| # | Check or action | On failure |
| --- | --- | --- |
| 1 | Buffer pointer not null, size ≥ header size | return 0 |
| 2 | Copy the 99-byte header (2.5.1) | — |
| 3 | Local file: the saved alias count (`0x19A44F5`) must be set (main file read first) | return 0 |
| 4 | Main file: save the header at `0x19A44D8` | — |
| 5 | `IsCorrectHeader` (`0x624130`): signature `TADBOSLB` and version (2.5.2) | return 0 |
| 6 | Main file, `header[29] != 0` and `header[25] != 0`: alias map `ReadFromBuffer` (`vtbl + 24`)(buf + 99, rest, count, 1); skip `header[25]` bytes | return 0 |
| 7 | Loop `header[21]` tables. Frame: u8 kind, u16 type, u16 major, u16 minor, u32 size (2.5.3). Find the loader slot by type. | — |
| 8 | Unknown type → skip the frame. Version different from `DrTableDef` → **return 0** (all data fails). Kind 1 → `DrAutoKeyTable`, kind 2 → `DrMultiKeyTable` (64 bytes from `DrBaseStaticMemory`), other kind → skip. | see left |
| 9 | `UseLegacyElHeader`, `Setup(tabledef)`, `ReadFromBuffer` (5.9). Body failure is **not** fatal: the table is freed, the loop continues, `loaded` stays false. | table from XML (5.6) |

No hash, no signature and no game-version check is on this file. The RSA check is on the `.dat` archive only (2.4). ✅ for the client code; the archive check is in the engine ❓.

Note for tool writers: an unknown table type is safe (skip). A known type with a different version is not safe: the whole file fails (`0x624440`). Thus a changed file must keep the 381 version of each table (4.7).

## 5.9 Table body into memory

| Function | Address | Behaviour |
| --- | --- | --- |
| `DrMultiKeyTable::ReadFromBuffer` | `0x6228b0` | Plain body: copy records to `_tableBuffer` and strings to `_tableExtraBuffer`. Build the key map: a set of the u64 at `record + keyoffset` (keyoffset is 8, 4.3.3, 4.6.3). ✅ |
| `DrAutoKeyTable::ReadFromBuffer` | `0x62ce60` | Plain body: key i starts at 1, skips records of type `0xFF`. Fails if the record count or the left-over byte count is wrong. ✅ |
| both | — | Relocate string fields: el reader `vtbl + 36`, base `_tableExtraBuffer` (3.6). ✅ |
| both | — | Flag byte "LM" → `_uselowmemory` → `sub_622F80` (rebuild ❓). |
| `DrCacheData::ReadFromBuffer` | `0x62f870` | Compressed body: keeps compressed chunks; a chunk is decompressed when a lookup needs it (6.2). ✅ |

## 5.10 Memory layout after load

```
dword_1A4D5AC ──► DataManager (0x5024 bytes)
  +0   QuestTable*                 (tree tables, special loaders)
  +4   SummonedSequenceLoader*
  +8   ContextScriptLoader*
  +12  SurveyQuestionsLoader*
  +16  MapUnitRelationTable*
  +20  TutorialSkillSequenceLoader*
  +24  SkillTrainingSequenceLoader*
  +28  _reader       DrDataReader*
  +32  _xmlReader
  +36  _readerIO     (main IO)
  +40  _readerLocalIO
  +44  _aliasMap     (vtable 0x18716d0)
  +48  _elreader
  +52  _loaderDefs[363] x 52 B ──► slot N at +52+52*N
                         +0 table* ──► DrMultiKeyTable / DrAutoKeyTable / stub
  +18928 _loaderDefCount
  +18932 _gameXmlDir[260], localXmlDir, reloadLocalXmlDir
  +20496 _valiator
  +20508 _recordCountPerPiece
```

Source: GM PDB layout minus 4 bytes, checked against the offsets in `0x5bc500` and the readers (6.1). ✅ for `+0`, `+28`..`+52`, `+20496`, `+20508`; ❓ for the other fields.

Table objects (x86):

```
DrDataTableImpl (20 B)            DrMultiKeyTable (64 B)          DrAutoKeyTable (64 B)
+0  vtbl                          +0..19 DrDataTableImpl          +0..19 DrDataTableImpl
+4  _tabledef                     +20 _elMap (DrMultiKeyElMap,     +20 _elMap (vector<DrEl*>,
+8  _uselowmemory (u8)                28 B)                            elCount)
+9  _useLegacyElHeader (u8)       +48 _tableBuffer                 +40 _tableBuffer
+10 _useTableCache (u8)           +52 _tableExtraBuffer            +44 _tableExtraBuffer
+12 _cacheData (DrCacheData*)     +56/+60 buffer sizes             +48/+52 buffer sizes
+16 _tableCacheInfo                                                +56 _nextAutoKey

DrMultiKeyElMap (28 B): vtbl, std::map<u64,DrEl*>, reconstructed flag,
                        elArray (12 B each: u64 key, DrEl*), elCount
DrCacheData (68 B):     chunkCount, maxCacheSize, chunkSize, idOffset,
                        dataChunkArray, chunk list, tabledef, cacheinfo, name
DataChunk (56 B):       startId, endId, compressedSize, compressedData,
                        rawDataSize, rawData, elCount, elArray (u16 offsets),
                        elPtrArray, refCount, next, prev
```

Vtables: multi-key `0x1871588`, auto-key `0x1871748`, stub `0x17c86c8`. ✅ Source: PDB types plus the field use in `0x6228b0`, `0x62ce60`, `0x622d50`, `0x62d3d0`, `0x62e920`.

## 5.11 Errors and messages

| Code / text | Where | Cause |
| --- | --- | --- |
| `0x3E8` (1000) | `0x116fd28` | `XmlReader_cl32.dll` or one of its four functions not found ✅ |
| `failed to initialize data manager.` | `LoadData`, after `0x114bfbb` | `sub_5B6000` failed (interface version not `"34"`, setup failed) ✅ |
| `failed to load data.` | `LoadData`, after `0x114c0b0` | `DataManager::Load` false ✅ |
| `0x2711` (10001) | `0x1190b66` → `0x1190f1e` | `LoadXmlData` false; message box text through `ClientSystemText::SetAdditionalMsg` ✅ |
| (no message) | `0x624440` | bad header, version mismatch → whole binary fails, XML fallback (5.6) ✅ |
| (no message) | `0x624440` step 9 | one table body bad → that table only from XML ✅ |

## 5.12 Engine side (not verified, ❓)

The engine `bsengine_Shipping.dll` has no IDA copy in this slice. These facts come from the journal only:

1. ❓ The engine opens `xml.dat` / `local.dat`, does the RSA check, AES decryption and zlib (2.1–2.4), and gives the IO objects at client slots `+0x1FC` / `+0x204`.
2. ❓ The IO object vtable: `+0` open, `+4` read, `+8` size, `+0x14` close (seen from the client calls in `0x5bca00`; the code is in the engine).
3. ❓ The meaning of locale flag byte `0x1955c44` (local file on or off).
4. ❓ `XmlReader_cl32.dll` internals (5.2).
5. ❓ The x64 client: same flow with `datafile64.bin`, `localfile64.bin`.
6. ❓ The bool arguments from `GRuleConfig + 0x30` and `+ 0x148`.

## Journal (old documents, not changed)

- `Docs/Client/client-datafiles.md` (load chain notes)
- `Docs/DataFile/07-client-exe-tabledefs.md`, `10-schema-from-exe.md`, `STATUS-schema.md`
