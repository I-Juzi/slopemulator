# 7. Versions and history

This chapter compares the data format of four client builds and two server builds. For each change, it gives the evidence. The base build of this book is client 0.0.381.8884 (2020). Chapters 1–6 describe that build.

## Quick find

| Question | Section | Address or file offset |
| --- | --- | --- |
| Which builds do we have, and from which year? | 7.1 | — |
| Did the `TADBOSLB` header change from 381 to 419? | 7.2 | GM `_BinaryDataFileHeader` (99 B); 381 reader `0x624440` |
| Does the client compare the file game version with its own version? | 7.3 | `GetClientVersion` `0x74ef10`; `0x624440` |
| How many tables does each build have? | 7.4 | 381 array (ch. 4); GM `Data::TableDefs` `0x1fa5f88`; 2026 `0x147484eb0` |
| Which attribute type codes are new? | 7.5 | GM enum `DrAttrType` (31 TIMEUTC, 32 ASSET) |
| Did the definition structs change size? | 7.6 | GM `DrAttrDef` 68 B; 381 stride 64 B |
| Is there a file writer in a client? | 7.7 | GM `BinaryDataFile::WriteToFile` `0x78c7a0` |
| Did the protection change? | 7.8 | ch. 2; journal |
| Do table IDs stay the same between builds? | 7.9 | 2026 `abnormalcamera` id 2 |
| What is the server history (348, 2020)? | 7.10 | server 348 array `0x140e99970`; 2020 `0x140e73670` |
| Which source wins when two builds disagree? | 7.11 | — |

## 7.1 The builds

| Build | Year | Binary | Data we have | Names | Source |
| --- | --- | --- | --- | --- | --- |
| Client 0.0.381.8884 | 2020 | `Client.exe` x86 + x64 | the four `.bin` files (ch. 1) | ported PDB, 83.8 % of functions | IDA `ida/lanes/cx/Client.exe.i64`; journal `Docs/Client/client-datafiles.md` |
| TableDefs 0.0.381.8876 | 2020 | — | operator definition export | — | journal; see the conflict in 7.11 |
| GM client, branch 419NC | 2021 | `Client_gm.exe`, link time 2021-05-24 | no `.bin` files | full PDB (115 MB) | IDA `Client_gm.exe.i64`; journal |
| BNSR UE4 archive | 2020-01-14 | `BNSR.exe` + PDB | `datafile.bin` 92,666,369 B; `datafile64.bin` 108,861,977 B | PDB | journal; format ❓ |
| BNSR_116 | 2026 | `BNSR_116.exe` x64, link time 2026-10-13, not packed | none | export by `bnsschema` | journal |
| NEO client (EU/KR) | later than BNSR_116 | — | — | BnsPluginTables C++ structs | journal; cross-check only |
| Server GameDaemon 348 | 2019 | `GameDaemon.exe` x64 | LMXBOSLB server XML | PDB | IDA `ida/datafile/gd348/GameDaemon.exe.i64`; `Docs/DataFile/03-tabledefs.md` |
| Server GameDaemon 2020 | 2020 | `GameDaemon.exe` x64 | 21,064 LMXBOSLB files | no PDB | `Docs/DataFile/04-tabledefs-2020.md` |

A build number has the form `0.0.<branch>.<build>`. The branch number (381, 419) grows with time.

```text
2019            2020                    2021           2026
 |               |                        |              |
 GD 348 --- client 381 (8876 defs, 8884 data) --- GM 419 --- BNSR_116 (UE4)
                 |  GD 2020 data           |
                 BNSR UE4 archive (2020-01-14)
```

## 7.2 The container header did not change (381 → 419)

The GM PDB gives the struct `BinaryDataFile::_BinaryDataFileHeader`. It is 99 bytes. Its fields have the same offsets as the 381 header in 2.5.1.

| Offset | Size | GM PDB field | 381 (2.5.1) | Same? |
| --- | --- | --- | --- | --- |
| 0 | 8 | `_binaryDataFileIdentifier` | signature `TADBOSLB` | ✅ |
| 8 | 1 | `_binaryDataFileVersion` (u8) | version 5 | ✅ field; GM value ❓ |
| 9 | 8 | `_gameVersion` (u64) | game version | ✅ |
| 17 | 4 | `_totalTableSize` | same | ✅ |
| 21 | 4 | `_totalTableCount` | same | ✅ |
| 25 | 4 | `_aliasSize` | same | ✅ |
| 29 | 4 | `_aliasCount` | same | ✅ |
| 33 | 4 | `_maxBufferSize` | same | ✅ |
| 37 | 4 | `_saveTime` | same | ✅ |
| 41 | 1 | `_compressed` | same | ✅ |
| 42 | 1 | `_encrypted` | same | ✅ |
| 43 | 56 | `_reserved[56]` | same | ✅ |

The GM build also has the name `BINARYDATAFILE_IDENTIFIER` at `0x1e8cf20` and the reader `BinaryDataFile::IsCorrectHeader` at `0x78b9c0`. We did not read the GM code. Thus the GM version constant is ❓. The struct layout is the same, so a 381 reader can read the GM header bytes.

New reader functions in GM (names only, code not read ❓):

| GM name | Address | Note |
| --- | --- | --- |
| `BinaryDataFile::ReadFromFile` | `0x78ba60` | also in 381 by behaviour |
| `BinaryDataFile::ReadFromMem` / `readFromMem` | `0x78be80` / `0x78bf20` | reads from a memory buffer |
| `BinaryDataFile::readFromFileStream` | `0x78ce80` | stream read |
| `DrDataReaderImpl::ReadBinaryDataFileFromMem` | `0x789a50` | new reader slot; 381 has only `+0x34` read and `+0x3C` save |
| `DrDataReaderImpl::SaveBinaryDataFile` | `0x789a90` | |
| `DrDataReaderImpl::ReplaceL10NData` | `0x789eb0` | replaces the local text data |
| `Data::DataManager::loadBinaryDataFile` | `0x6acbd0` | same argument list as 381 `0x5bca00`, with a u64 version |

## 7.3 The game version in the header is not checked (381)

The header has the field `_gameVersion` (u64). The 381 client does not compare it with its own version.

Evidence (381, `Client.exe.i64`):

1. `GetClientVersion` `0x74ef10` reads the file version resource of the exe. It keeps the result in a global cache.
2. `BNSClient::LoadData` `0x114bf80` calls `GetClientVersion` at `0x114c07e`. It gives the u64 to `DataManager::Load`, and then to `loadBinaryDataFile` `0x5bca00`.
3. The version goes to the reader slots `+0x34` (`ReadBinaryDataFile` `0x618210`) and `+0x3C` (`SaveBinaryDataFile` `0x6182f0`). These call `0x6243c0`, which calls `0x624440`.
4. `0x624440` has 8 arguments: element reader, alias map, buffer, size, loader definitions, count, local flag and legacy flag. None of them is the version. The version stops at `0x6243c0` ✅.
5. The other callers of `GetClientVersion` are `ClientExpireCheck::CheckClientRestrict` `0x115bf84` and `BNSClient::Initialize` `0x116f6c3`. They do not read the data file.

Result: you can open a `datafile.bin` from a different build if the table versions match (see 7.6 and 5.8). The game version field is only a label. GM keeps the same `loadBinaryDataFile` signature (7.2). Whether GM uses the version is ❓.

## 7.4 Number of tables

| Build | Tables | Last enum value | Evidence |
| --- | --- | --- | --- |
| Server 348 | 265 | — | array `0x140e99970` (`Docs/DataFile/03-tabledefs.md`) |
| Server 2020 | 277 | — | array `0x140e73670` (`04-tabledefs-2020.md`) |
| Client 381 | 362 | 363 slots | `tables.tsv` (362 rows); array `0x194ece8` (4.2) |
| GM 419 | 385 | `TABLE_COUNT` = 386 | `Data::TableDefs` `0x1fa5f88` is `const DrTableDef*[386]`; enum `Data::TableType` has 387 values ✅ |
| BNSR_116 (2026) | 519 | — | array `0x147484eb0` (journal) |
| NEO (BnsPluginTables) | 528 | — | journal; cross-check only |

The GM array has 386 slots. Slot 0 is the "none" type. Thus 385 tables are real.

The 2026 build removed 11 tables that 381 has (journal).

## 7.5 Attribute type codes

| Code | Name | 381 | GM 419 | 2026 | Evidence |
| --- | --- | --- | --- | --- | --- |
| 0–30 | see 3.5 | ✅ | ✅ | ✅ | ch. 3 |
| 31 | `TIMEUTC` | ❓ | ✅ | ✅ | GM enum `DrAttrType` |
| 32 | `ASSET` | ❓ | ✅ | ✅ | GM enum `DrAttrType` |
| 33 | `TYPE_COUNT` in GM; a real type in 2026 | — | end marker | new type | GM enum (34 values); journal for 2026 |

The journal says that the 2026 layout has new type codes 31, 32 and 33. In GM, 33 is the count, so 33 is new after GM.

GM `dr_native` is 8 bytes: `{size, data}`. GM `readNative` sets size = 2 × chars + 2 (journal). This agrees with the 381 rule in 3.7, where the word `a` is a byte size. The 2 extra bytes are the terminator. This is evidence for open question 4 of part 1, but from GM, not from 381 ❓.

## 7.6 Definition struct sizes

| Struct | 381 x86 | GM 419 (PDB) | 2026 x64p4 | Evidence |
| --- | --- | --- | --- | --- |
| `DrTableDef` | 36 | 36 | ❓ | GM PDB; 4.3.1 |
| `DrElDef` | 44 | 44 | ❓ | GM PDB; 4.3.2 |
| `DrElBodyDef` | 20 | 20 | ❓ | GM PDB; 4.3.3 |
| `DrAttrDef` | stride 64 | 68 (new field `dynamic` at 64) | 84 | GM PDB; 4.3.4 (IDA stride test); `bnsschema` layout `X64P4` |
| `DrSubDef` | 16 | 16 | ❓ | GM PDB; 4.3.5 |
| `DrLoaderDef` | ❓ | 52 | — | GM PDB; server stride 96 (`03-tabledefs.md`) |
| `DrEl` | 6 | 6 | — | GM PDB; 3.4 |
| `DrTableSummary` | — | 72 | — | GM PDB; see 8.7 |

`bnsschema` knows two layouts: `X86` (381) and `X64P4` (2026). It does not know the GM layout (68-byte `DrAttrDef`) ❓.

The client loads a table only if the table version in the file is the same as the compiled definition (5.8). Thus the definition layout and the data must come from the same build. The journal tested this: the 2026 layout does not open the 381 data.

## 7.7 A writer exists in GM

The 381 client has no named `BinaryDataFile::WriteToFile`. Its names are ported from GM. The GM PDB has:

- `BinaryDataFile::WriteToFile` at `0x78c7a0`.
- `DrDataReaderImpl::SaveBinaryDataFile` at `0x789a90`.
- `DrUtil::GenerateMD5Checksum` at `0x7a7b20`.
- `DataBrowser` and `ShowDataInspector` names (a data inspector UI in the GM build).

This shows that the game company could write `.bin` files with a client build. It does not show which tool made the 381 files (part 1, open question 8) ❓. We did not read the writer code. The server 348 alias `WriteToBuffer` is a stub (`Docs/DataFile/02-tadboslb.md`).

## 7.8 Protection history

| Layer | 381 | GM 419 | BNSR 2020 / 2026 | Evidence |
| --- | --- | --- | --- | --- |
| `.dat` archive `UOSEDALB` + AES-128 (`keyhere`) | ✅ | ❓ | ❓ | ch. 2.2, 2.3 |
| RSA signature on the archive | ✅ | ❓ | ❓ | ch. 2.4 |
| zlib in tables | ✅ | ✅ field `_compressed` | ❓ | 2.6; GM header |
| Encryption of `datafile.bin` | flag `_encrypted` is 0 and not read | field is present | ❓ | 2.5.1; GM header |
| Packer on the exe | ❓ | none (PDB names fit) | not packed (BNSR_116) | journal; `bnsschema` `protect::check` |
| Server LMXBOSLB string XOR (`keyhere`) | — | — | 2019 and 2020 servers use version 3 | `Docs/DataFile/01-lmxboslb.md` |

The server key is in a table `cipherKeys`. The 2019 table is at `0x1801ba930`. The 2020 table is at `0x1800dc4b0` (journal). This book does not write the key. Write `keyhere`.

The BNSR 2020 archive has no `locale.bin`. Its Korean text is in `.x16` files (journal). The format of its `datafile*.bin` files is ❓. We did not open them.

## 7.9 Table IDs are not stable

A table ID is the index in the definition array of one build. It is not a fixed number.

| Table | 381 client | 2026 client | Server 348 | Source |
| --- | --- | --- | --- | --- |
| `abnormalcamera` | 1 | 2 | ❓ | journal |
| `account-level` | ❓ | ❓ | 1 | `03-tabledefs.md` |
| `item` | 159 (1.398) | ❓ | 114 (1.387) | `tables.tsv`; `03-tabledefs.md` |
| `table` element count | 44 | 60 | — | journal |

The server 2020 definitions compared with client 381 by ID: 19 match, 252 differ in version, 77 are missing, 133 differ in kind (`04-tabledefs-2020.md`). The IDs are given in a different way. Thus a match by ID is not correct. Always match tables by **name**, then compare the version.

Rule for tools: never store a table ID in a file that must work with more than one build. Store the table name.

## 7.10 Server history

| Item | Server 348 (2019) | Server 2020 | Evidence |
| --- | --- | --- | --- |
| Definition array | `0x140e99970`, 265 tables | `0x140e73670`, 277 tables | `03-tabledefs.md`; `04-tabledefs-2020.md` |
| `TADBOSLB` reader | `0x1406fa370` (no flag byte in body) | ❓ | `02-tadboslb.md`; 3.1 |
| Data form | LMXBOSLB v3 | LMXBOSLB v3 (21,064 of 21,065 `.xml` paths) | `Docs/Server/GameDaemon/01-lmxboslb.md` |
| `GameData_RNCS.zip` version file | — | `GameData-0.0.5002.50 / GameDaemon-0.0.356.8492` | journal; the trailing base64 is ❓ |

The 2020 data is newer than the 348 daemon. For example, it has the field `mastery-exp-best-record`, which the RankingDaemon 348 does not know (journal).

## 7.11 Conflicts and which source wins

| Conflict | Sources | Winner |
| --- | --- | --- |
| TableDefs build 8876 vs data build 8884 | operator export vs file names | Both are branch 381. The table versions match (`bnsdata` round trip tests pass). Use the 8876 definitions for 8884 data ✅ |
| Client vs server table IDs | `tables.tsv` vs `03-tabledefs.md` | Neither. Match by name (7.9) |
| Server body has no flag byte; client has one | server 348 code vs client walk | Client form for client files (3.1) |
| GM `DrAttrDef` 68 B vs 381 stride 64 B | GM PDB vs 381 array walk | Each is right for its build. Do not mix them |
| Type 33 | GM: count marker; 2026: real type | Each is right for its build |
| BnsPluginTables offsets vs BNSR_116 export | 926 offsets match in 73 tables | Our export wins for BNSR_116. Their build is later |

## 7.12 Open questions

1. The GM `_binaryDataFileVersion` constant (7.2).
2. Does GM or 2026 compare `_gameVersion` (7.3)?
3. The format of the BNSR 2020 UE4 `datafile*.bin` files (7.1, 7.8).
4. Do types 31 and 32 occur in 381 data (7.5)?
5. Which tool made the 381 files; is it `WriteToFile` (7.7)?
6. The 2026 `.dat` protection (7.8).
