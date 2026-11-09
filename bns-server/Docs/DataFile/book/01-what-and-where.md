# 1. What the data files are, and where they live

Build: client 0.0.381.8884 (data), `Client.exe` 0.0.381 x86 (code, IDA copy `ida/lanes/cx/Client.exe.i64`).
Legend: ✅ verified (source given). ❓ not verified. "Journal" = `Docs/Client/client-datafiles.md`.

## Quick find

| Question | Section | Address or file offset |
| --- | --- | --- |
| Which files exist? | 1.1 | — |
| What game data is in them? | 1.2 | table array `0x194ece8` |
| Who makes them? | 1.3 | header `_saveTime` at file offset 37 / 57 |
| How do 32-bit and 64-bit files differ? | 1.4 | header offset 21 (x86) / 25 (x64) |
| Where are they on disk? | 1.5 | `BNSClient::GetDataDirectoryPath` `0x1129d30` |
| How does the client find and open them? | 1.6 | `BNSClient::Initialize` `0x116fe4d`, `0x1170541`, `0x1170696`; `BNSClient::LoadData` `0x114bf80` |
| Is there a `locale.bin`? | 1.1 | string search, no hit |
| What does the private-server client change? | 1.7 | `bin\plugins\binreplace.ini` |

## 1.1 The files

The client reads its static game data (items, skills, quests, zones, texts) from two typed binary files. Each file exists in a 32-bit and a 64-bit form.

| File | Role | Size (bytes) | Tables | Source |
| --- | --- | ---: | ---: | --- |
| `datafile.bin` | main data, 32-bit client | 141,456,443 | 348 | journal "Client files received" ✅ |
| `datafile64.bin` | main data, 64-bit client | 152,163,903 | 348 | same ✅ |
| `localfile.bin` | localized data (texts), 32-bit | 17,344,957 | 4 | same ✅ |
| `localfile64.bin` | localized data, 64-bit | 17,538,602 | 4 | same ✅ |

- All four files start with the ASCII signature `TADBOSLB` and the format version byte 5. ✅ Journal; client check `BinaryDataFile::_BinaryDataFileHeader::IsCorrectHeader` `0x624130` (8-byte compare with `0x1870e60`, then version byte == 5).
- `TADBOSLB` read backward is `BLSOBDAT` ("BLS" binary data). The server binary XML signature `LMXBOSLB` read backward is `BLSOBXML`. The `.dat` archive signature `UOSEDALB` (section 1.6) read backward is `BLADESOU`. They are one naming family. ✅ bytes; the meaning of the letters is ❓.
- **`locale.bin` does not exist in build 381.** The 381 `Client.exe` has no ASCII or UTF-16 string `locale.bin`. The only `locale` strings are UI option names (`locale`, `locale-time`). ✅ IDA byte search, 2026-11-09. The 2020 UE4 client `BNSR` also has no `locale.bin`; it keeps Korean text in loose `.x16` files. ✅ Journal "Source: BNSR 2020 UE4 client archive". Treat "locale.bin" in old notes as a wrong name for `localfile.bin`.
- The 4 tables in `localfile.bin` have type IDs 233 (`petition-faq-list`), 319, 327 (`text`), 335. ✅ Journal "Operator-side validation". `text` holds 687,415 records. ✅ Journal "Record size facts".

## 1.2 What game data is in them

A data file is a list of **tables**. Each table holds the records of one data type, for example `item` (144,439 records) or `text` (687,415 records). ✅ Journal "Record size facts".

- Type IDs go from 1 to 362. The client has a static array of 363 definition pointers at VA `0x194ece8`. Slot 0 is null. Slot N is the definition of type N. ✅ `Docs/DataFile/07-client-exe-tabledefs.md`; IDA read of the array, 2026-11-09.
- Examples: type 1 `abnormalcamera`, type 195 `map-area` v0.2, type 327 `text` v0.6. ✅ IDA, definitions at `0x194ece8` + 4 × type.
- The main file holds 348 of the 362 types. Some types have no table in the file. The client skips a table whose type it does not know (section 2.5). ✅ Journal; `0x624440`.
- Each table carries a schema version (major, minor). The client rejects the file when a table version differs from its compiled definition. ✅ `0x624440` (compare with `tabledef->major_ver` / `minor_ver`, then return 0).
- One table can have more than one record shape: an element type (byte 0 of each record) and a subtype (bytes 2–3). Chapter 3 explains this.

## 1.3 Who makes them

The files come from the NCSoft data build. Evidence, from strongest to weakest:

| Evidence | Fact | State |
| --- | --- | --- |
| Header `_gameVersion` | 0.0.381.8884, the same build as our LobbyGate target | ✅ journal |
| Header `_saveTime` | 2020-06-10 23:28:58 UTC (x86) / 23:33:24 UTC (x64). The x64 file was saved 4.5 minutes later. | ✅ journal; field name from PDB type `BinaryDataFile::_BinaryDataFileHeader` |
| Build path strings in `Client.exe` | `E:\DailyBuild02\Client-Common-0.0.381\depot\Branch\381NC\...` | ✅ `BNSClient::LoadData` `0x114bf80` |
| Server table data | 2020 GameDaemon data is binary XML (`LMXBOSLB`, 21,064 files). The same data types (for example `achievement`, `season-contents-schedule`) exist there in XML form. | ✅ journal |
| Server 348 PDB names | `DataPacker::GAMEDATA_BINFILENAME`, `DataPacker::GAMEDATA_VERSION_FILENAME`, `DrDataReaderImpl::SaveBinaryDataFile` | ✅ names; the server save slot is a stub that returns false (`Docs/DataFile/02-tadboslb.md`) |
| Client save slot | `DrDataReaderImpl::SaveBinaryDataFile` `0x6182f0` forwards to `0x6243c0`, the same helper that the read path calls (`0x6241d0`). No writer code was found. | ✅ address; ❓ folded code |
| Client XML path | `Data::DataManager::Load` `0x5bc500` calls `loadBinaryDataFile` `0x5bca00`, then `constructDataOnlyForXML`, `constructData` and `DataValidator::CheckDataValidation`. The `DrDataReaderImpl` class also has `LoadDocument` `0x618a80` and `LoadTable` `0x6180b0`, which read XML through an `XmlReader`. | ✅ names and call order |

Probable pipeline (❓ inferred, no packer binary found):

```
designer XML (one file per data type, "release-module" groups)
        |  NCSoft data build ("DataPacker" ❓)
        |  - read each XML with the same Dr* library and the same table definitions
        |  - write typed records, string pools, alias map
        v
datafile.bin / localfile.bin  (TADBOSLB v5, one per width)    server copy: LMXBOSLB XML per type
        |  pack tool ❓
        v
xml.dat / local.dat  (UOSEDALB v3 archive: RSA-signed, AES-128, zlib)
```

The client and the server share the `Dr*` data library (`DrDataReaderImpl`, `DrMultiKeyTable`, `DrAliasMapImpl`). The server 348 build reads the same `TADBOSLB` container, but its table bodies have no leading compressed byte (chapter 3). ✅ `Docs/DataFile/06-client-body.md`.

## 1.4 32-bit and 64-bit files

The 32-bit client loads `datafile.bin`. The 64-bit client loads `datafile64.bin`. ❓ The 64-bit file name is not in the x86 `Client.exe`; we have no 64-bit 381 executable in IDA. The name comes from the operator files.

| Item | 32-bit | 64-bit | Source |
| --- | --- | --- | --- |
| Header size | 99 bytes | 119 bytes | journal walk ✅; PDB type size 0x63 = 99 ✅ (`Client.exe` types) |
| Size and count fields in the header | u32 | u64 | `Docs/DataFile/02-tadboslb.md` ✅ |
| Auto-key table count | u32 | u64 | `06-client-body.md` ✅ |
| `string` field | 4-byte offset | 8-byte offset | journal "Operator table definitions x64" ✅ |
| `native` field | 8 bytes | 12 bytes | same ✅ |
| Record sizes | x86 layout | x64 layout (234 of 348 tables differ) | same ✅ |
| Tables, records, keys, texts | same | same | 1,081,505 records per main file in both widths ✅ |
| Compressed block boundaries | differ | differ | `06-client-body.md` ✅ |

The two widths carry the same game data. Only the binary layout differs, because the client copies records into memory as C structs, and pointer-size fields change size.

## 1.5 Where they live on disk

### 1.5.1 Retail layout (`.dat` archives)

In a retail client, `datafile.bin` and `localfile.bin` are not loose files. They are entries inside encrypted `.dat` archives:

| Archive | Holds | Client slot | Opened at | Source |
| --- | --- | --- | --- | --- |
| `config.dat` | `release.config2.xml` and other config XML | `EInterfaceSetClientInfo` + `0x200` (slot 128) | `0x116fe5b` | ✅ IDA |
| `xml.dat` | `datafile.bin` | + `0x1FC` (slot 127) | `0x1170549` | ✅ IDA |
| `local.dat` | `localfile.bin` | + `0x204` (slot 129) | `0x117069e` | ✅ IDA |
| `config64.dat`, `xml64.dat`, `local64.dat` | 64-bit forms | — | — | ❓ `config64.dat` seen on disk (journal); the other two names are inferred |

Folder: `BNSClient::GetDataDirectoryPath` `0x1129d30` builds the path from these parts:

```
"..\contents\"  +  ("local\" or "data\")  +  publisher or locale path  +  "data\"
     fixed          selected by the            LocalizeUnit::PublisherPath   fixed
                    publisher value ❓         or LocalizeUnit::FullPath
```

The path is relative to the `bin\` folder of the client. On the operator client the result is `contents\local\NCSoft\data\` (journal "Client-side loading", `config.dat` found there). ✅ path parts; ❓ which branch gives which part (the decompiled condition looks inverted).

The third argument of `GetDataDirectoryPath` selects `FullPath` (with the language part) or `PublisherPath`:

- `config.dat`: publisher path. ✅ `0x116fe3f` (argument 0).
- `xml.dat`: full path when the locale flag byte `0x1955c44` is 0, else publisher path. ✅ `0x1170521`–`0x1170541`.
- `local.dat`: only when the locale flag is 1; full path. ✅ `0x117067a`–`0x1170696`.

The locale flag `0x1955c44` is written by `BNSClient::SetDefaultLocaleInfo` `0x1129992` and `BNSClient::DoCommandLine` `0x113f987`. ✅ xrefs. ❓ Which locales set it to 1.

A plain ASCII string `../contents/data/xml.dat` at `0x14f3f9c` is used only by `Wiper::Execute` `0x5b04d0` (an unrelated tool path). ✅ xref.

### 1.5.2 Loose file layout

`Data::DataManager::loadBinaryDataFile` `0x5bca00` has two paths (section 1.6). When no archive reader is set, the client reads `datafile.bin` and `localfile.bin` from disk with `CreateFileW` (`0x6241d0`). In that path the file names have no folder (both prefixes are empty strings in `LoadXmlData` `0x114ca4b`–`0x114ca73`), so the path is relative to the working folder. ✅ IDA. ❓ When this path is active in a retail client.

## 1.6 How the client finds and opens them

```
BNSClient::Initialize                                           0x116f550 region
 ├─ GetDataDirectoryPath + "config.dat" ─> new PackXmlReaderIO ─> Initialize(path, 1, salt, key, checksum)   0x116feeb
 ├─ GetDataDirectoryPath + "xml.dat"    ─> new PackXmlReaderIO ─> Initialize(...)                           0x11705d7
 └─ (locale flag == 1) + "local.dat"    ─> new PackXmlReaderIO ─> Initialize(...)                           0x117072c
      PackXmlReaderIO::Initialize 0x7076b0
        PackEnc::Initialize 0x7044d0          build the AES-128 key, chapter 2
        DataPackFile::Open 0x7063e0
        DataPackFile::ReadFromFile 0x7068b0   header + file index, lazy contents
        PackSignature::CheckSignature 0x7060b0  RSA signature over the archive

LoadXmlData 0x114c9f0
 └─ BNSClient::LoadData("", "") 0x114bf80
      sub_5b6000(dataIO = slot 127, localIO = slot 129)   creates Data::DataManager 0x5b6370
      name1 = "" + "datafile.bin"
      name2 = "" + "localfile.bin"   only when locale flag == 1
      Data::DataManager::Load 0x5bc500
        loadBinaryDataFile 0x5bca00
          if dataIO:  open "datafile.bin" in xml.dat, read all bytes, parse as main  (isLocal = 0)
          if localIO: open "localfile.bin" in local.dat, read all bytes, parse as local (isLocal = 1)
          else:       DrDataReaderImpl::ReadBinaryDataFile 0x618210 -> read from disk 0x6241d0
        BinaryDataFile read-from-memory 0x624440   chapter 2 and 3
```

Facts:

- All three archives get the same three key blobs: a 32-byte salt (`0x14b186c`), a 610-byte key blob (`0x14b1890`), a 32-byte checksum (`0x14b1af4`). The type `SignatureKey` names them `keySalt`, `key`, `keyChecksum`. ✅ `0x116fe9c`–`0x116feaf`; PDB type `SignatureKey`. Values: `keyhere`.
- The second `Initialize` argument is 1 for all three archives. It is stored at `PackXmlReaderIO + 328` and passed to `DataPackFile::ReadFromFile`. With 1, the client reads only the index at start and decodes each entry on first use. With 0, it decodes every entry at once. ✅ `0x7068b0` (loop that calls `LoadFileContents` only when the argument is 0).
- `loadBinaryDataFile` opens the entry through the reader interface (`vtbl[0]` open with the filter string, `vtbl[2]` size, `vtbl[1]` read, `vtbl[5]` close). ✅ `0x5bca00`.
- The main file must load first. The read-from-memory function stores the main header in a global (`0x19a44d8`, 99 bytes). A local file load fails when the saved main alias count (`0x19a44f5`) is 0. ✅ `0x624440`.
- If the load fails, the client logs `failed to load data.` (or `failed to initialize data manager.`) with source `Client.cpp`. ✅ `0x114bf80`.

## 1.7 The private-server client (operator laptop)

The operator client is the retail 381.8884 x86 client with a plugin stack. A proxy `winmm.dll` loads every DLL in `bin\plugins\`. The plugin `binreplace.dll` reads `bin\plugins\binreplace.ini`, which maps `datafile` and `localfile` to `..\contents\local\NCSoft\data\datafile.bin` and `...\localfile.bin`. ✅ Journal "Client-side loading".

So on that client the `.bin` files are loose files and the plugin gives them to the client instead of the archive entries. ❓ How the plugin does this (hook on the archive reader or on the disk path) was not checked; `binreplace.dll` has no IDA copy.

Consequence for us: an edited `datafile.bin` needs no `.dat` repack and no RSA signature on that client. A clean retail client needs both (chapter 2).

## 1.8 Engine-side facts not checked

`bsengine_Shipping.dll` has no IDA copy. These items can sit in it and stay ❓:

1. The 64-bit file and archive names (`datafile64.bin`, `xml64.dat`, `local64.dat`).
2. Any second check of the `.dat` signature or of the data version at world entry.
3. The UE file system mount of `contents\` (if the engine and not `Client.exe` resolves the relative path).

## Journal links

Merged from: journal sections "Facts", "Client files received", "External tool: BnsBinTool", "Origin of the TableDefs JSON", "Source: BNSR 2020 UE4 client archive", "Client-side loading of `datafile.bin` / `localfile.bin`"; `Docs/DataFile/02-tadboslb.md` (sources table). Those sections stay unchanged as the research journal.
