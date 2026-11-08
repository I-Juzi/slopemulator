# Client data files: `datafile.bin`, `datafile64.bin`, `locale.bin`

Research log. Add each fact with its source and build. Mark facts that are not verified with ❓.

## Searches

| Date | Where | Search | Result |
| --- | --- | --- | --- |
| 2026-11-07 | `ida/LobbyGate/LobbyGate.c` (348 full decompile) | `datafile`, `locale.bin` | no match |
| 2026-11-07 | all `.exe` / `.dll` (< 200 MB) under `BnS_2020/` and `Service_348_8349/` (ASCII and UTF-16 strings) | `datafile(64).bin`, `locale.bin` | no match |
| 2026-11-07 | file names under `BnS_2020/` and `Service_348_8349/` | `*datafile*`, `locale*.bin` | no files |

Conclusion so far: LobbyGate does not read or write these files. The server sets in this directory do not name them.

## Facts

(none yet)

## Server-side binary XML (LMXBOSLB) — 2020 server package

- `BnS_2020/GameDaemon/data/*.xml`: 1,098 of 1,099 files start with the 8-byte signature `LMXBOSLB` (binary XML, body looks obfuscated). One file is plain XML. Source: `head -c 8` on each file, 2026-11-07. Build: 2020 server package (GameDaemon.exe dated 2020-07-04).
- File families (count): effectdata 119, effectgroupdata 116, skill3data 91, skillcastcondition3data 81, zonenpcspawndata 77, zonenpcsquaddata 75, itemskilldata 24, npcdata 18, npcbrainparametersdata 18.
- ❓ Same binary XML format as the XML inside the client `datafile.bin` / `datafile64.bin`. Not verified.
- ❓ The decoder is in GameDaemon's data loader. Not located yet. Guide: GameDaemon build 348 has `GameDaemon.pdb` (`Service_348_8349/Service/GameDaemon/bin/`).
- `GameData_RNCS.zip` (same directory): 23,074 entries, 3.3 GB unpacked, contains `gamedata_version.txt` dated 2019-11-11. Contents not checked.
- LMXBOSLB file count by server (recursive, `*.xml`/`*.dat`, 2026-11-07): GameDaemon 21,064 of 21,073 XML; LobbyGate 0 of 6; LobbyDaemon 0 of 12 (only plain-text `config/*.xml`). Lobby servers do not load binary XML.
- The ASCII string `LMXBOSLB` does not occur in `GameDaemon/bin/*.exe` (2020). ❓ The loader may check the signature as an integer constant or sit in a DLL.
- The `LMXBOSLB` reader is in `XmlReader_sv64.dll` (exports `CreateXmlReader` / `DestroyXmlReader`). The ASCII signature occurs 3 times in it. Also in `XmlReader2019_sv64.dll`. GameDaemon.exe only contains `OSLB` (3 hits). Source: `grep -a`, 2026-11-07, 2020 package `BnS_2020/GameDaemon/bin/`.
- A matching PDB exists: `BnS_2020/InfoGateDaemon/bin/XmlReader_sv64.pdb` (2019-01-08). Its GUID `1e4f1ca42cc70841b32a781dbe6ec68d` matches InfoGateDaemon's own `XmlReader_sv64.dll` (sha1 00bdd10ab54e…). It does NOT match GameDaemon's 2020 copy (sha1 f81c0d0d9231…). Names in it: `XmlDoc::IsBinaryFile`, `XmlReaderImpl::IsBinary`, `XmlDocImpl::BinarySize`, `XmlElementImpl::BinarySize`, `XmlTextNodeImpl::BinarySize`. Text XML goes through libxml2/expat.
- ❓ The 2019 and 2020 DLLs use the same binary format. Not verified.
- GameDaemon 348 PDB (`Service_348_8349/Service/GameDaemon/bin/GameDaemon.pdb`) names a binary data-file reader: `BinaryDataFile::ReadFromFile`, `ReadFromMem`, `ReadFromFileStream`, `BINARYDATAFILE_IDENTIFIER`, `_gameDataBinaryDataFileHeader`, a local `_version` in `readFromFileStream`, and `DrDataReaderImpl::ReadBinaryDataFile` / `SaveBinaryDataFile`. Also `DataPacker::GAMEDATA_VERSION_FILENAME` and `DrUtil::GenerateMD5Checksum`. ❓ Related to the client `datafile.bin` format. Not verified.
- `gamedata_version.txt` in `BnS_2020/GameDaemon/data/GameData_RNCS.zip`: the first line reads `GameData-0.0.5002.50|GameDaemon-0.0.356.8492`, then a base64 block (❓ maybe a signature or hash list). The zip entry is dated 2019-11-11.
- No evidence yet that any server compares the client's `datafile.bin` with its own. The only client version check found so far is the LobbyGate HelloReq protocol version (58, `bns-server/Docs/Server/LobbyGate/02-handshake.md`). ❓ A data-version or hash check in the world entry. Not searched.

## Client files received (2026-11-07)

Location: `<operator-downloads>/Bin/` (operator copy; not in the repo). Read-only header checks only.

| File | Size | File date |
| --- | --- | --- |
| `datafile.bin` | 141,456,443 | 2022-06-14 |
| `datafile64.bin` | 152,163,903 | 2022-06-15 |
| `localfile.bin` | 17,344,957 | 2022-06-14 |
| `localfile64.bin` | 17,538,602 | 2022-06-15 |

- All four start with the signature `TADBOSLB`, then u32 format version **5**. Source: `xxd -l 64`.
- Signature family: `TADBOSLB` reversed is `BLSOBDAT`. The server XML signature `LMXBOSLB` reversed is `BLSOBXML`. Same naming family; the container (DAT, version 5) differs from the single XML file (XML, version 3). Source: header bytes of both.
- Field at 0x11 (u32 in 32-bit files, u64 in 64-bit files) ❓ size: 121,952,001 / 127,540,087 (datafile), 14,182,211 / 14,354,010 (localfile). Next field ❓ entry count: **348** (datafile), **4** (localfile).
- u32 at offset 37 (32-bit) and 57 (64-bit) reads as Unix time **2020-06-10 23:28:58 / 23:33:24 UTC** ❓ build time. This fits the 2020 server package (XmlReader DLLs 2020-06-11, GameDaemon.exe 2020-07-04). The 2022 file dates are copy dates. ❓
- `datafile64.bin` body: entropy about 7.93 bits/byte in the middle (compressed or encrypted); about 3.0 near the start and 1.4 near the end (❓ index tables). Inner `LMXBOSLB` and `<?xml` do not occur in plain form. zlib-like byte pairs (`78 9c` 749 times, `78 da` 733 times) occur, but may be chance. ❓
- Not done: entry decoding. It may need a key built into the client. Stopped here and asked the operator first (project rule: no client protection work without approval).

## External tool: BnsBinTool (notscuffed)

Source: https://github.com/notscuffed/BnsBinTool, last commit 2022-04-13, archived 2022-09-14. Local clone (outside the repo): `<ext>/BnsBinTool`. **No licence file**: do not copy its code into bns-server. Use it only as a cross-check of facts.

- Header layout (`Src/BnsBinTool.Core/Models/DatafileHeader.cs`), checked against our `datafile.bin` / `datafile64.bin` bytes ✅:

| Offset | Size (32 / 64-bit) | Field | Our files |
| --- | --- | --- | --- |
| 0 | 8 | magic `TADBOSLB` | ✅ |
| 8 | 1 (u8, not u32) | datafile version | 5 |
| 9 | 4 × u16 | client version | **0.0.381.8884** (bytes `00 00 00 00 7d 01 b4 22`) |
| 17 | 4 / 8 | total table size | 121,952,001 / 127,540,087 |
| 21 / 25 | 4 / 8 | table count | 348 (datafile), 4 (localfile) |
| … | 4 / 8 each | alias map size, alias count, max buffer size | — |
| 37 / 57 | u32 | created at (Unix time) | 2020-06-10 23:28:58 / 23:33:24 UTC |
| 41 / 61 | 58 | reserved | — |

- Client version **0.0.381** matches the LobbyGate target build 0.0.381 ✅. The data belongs to the same build as our LobbyGate.
- The tool has **no AES/decrypt code** (grep `Aes|Rijndael|Decrypt|Xor` in `Src`: no match). `datafile.bin` itself is not encrypted. The high body entropy is compression: the tool has `RecordCompressedReader`/`RecordCompressedWriter` and uses Intel ISA-L (`Deps/isalib`, deflate). ❓ Exact compression format, not checked by us.
- In the live client, `datafile.bin` / `localfile.bin` come from inside `xml.dat` / `local.dat` (the tool takes "extracted xml.dat" paths for aliases). ❓ Those `.dat` archives are the encrypted layer. Our operator files are already extracted.
- Reading tables needs **table definitions per client version** (`TableDef <version>` folder of `*.json`, `DatafileDefinition.cs` line 49). The repo does not ship them. ❓ For 0.0.381 they must come from elsewhere; GameDaemon's `DrLoaderDef` / `DrElReader` (348 PDB) may provide the server-side equivalent.
- The tool can dump to XML and compile XML back to `datafile.bin` (`bnsxml dump` / `compile` / `transform`, `-64` switch). So repacking without a client-side signature is known to work for that era ❓ (not tested by us).

## Operator-side validation of the dat1 spec (2026-11-07, read-only Python walks)

- TADBOSLB layout from `bns-server/Docs/DataFile/02-tadboslb.md` (GameDaemon 348 `BinaryDataFile::readFromMem` `0x1406fa370`) ✅ on all four client files:

| File | Header | Alias block (size / count) | Tables | Kinds (1 auto / 2 multi) | Walk ends at file size |
| --- | --- | --- | --- | --- | --- |
| `datafile.bin` | 99 | 32,350,773 / 1,727,972 | 348 | 111 / 237 | ✅ exact |
| `datafile64.bin` | 119 | 39,244,150 / 1,710,494 | 348 | 111 / 237 | ✅ exact |
| `localfile.bin` | 99 | none in file (header repeats the datafile values) | 4 (types 233, 319, 327, 335) | 2 / 2 | ✅ exact |
| `localfile64.bin` | 119 | none in file | 4 | 2 / 2 | ✅ exact |

- `_compressed` = 0 and `_encrypted` = 0 in all four headers. **No container-level encryption or compression.** The high entropy is the table and alias content itself. ✅
- Localfile header copies the datafile alias fields, but its body has no alias block. Reader rule: read the alias block for the main datafile only (matches `DrDataReaderImpl::ReadBinaryDataFile` `0x1406ebac0` loading main before local). ✅ by walk; ❓ by code.
- First tables (datafile): type 1 v0.3, type 2 v0.39, type 3 v0.3, type 4 v0.1. Body sizes differ between 32- and 64-bit files (pointer-size records). ✅
- LMXBOSLB layout from `01-lmxboslb.md` ✅ on 60 random server files in `BnS_2020/GameDaemon/data`: version byte 3, size at offset 12 = file size, tree walk (counts and tags are not XORed) ends exactly at file size. Strings stay XORed (key table in `XmlReader_sv64.dll` 2020 at `0x1800dc4b0`, not read).

## dat2 results and real-file round trip (2026-11-07)

| Fact | Source | State |
| --- | --- | --- |
| Rust `bnsdata::dat` reads and rewrites all four client files **byte-identically** (datafile 348 tables / alias 32,350,773 B; datafile64 348 / 39,244,150 B; localfile and localfile64 4 tables, no alias) | `Program/Tools/DataFile/BnsData/tests/real_files.rs` `client_roundtrip`, run by operator with `BNSDATA_DAT_WIDTH` and `BNSDATA_DAT_ROLE` | ✅ |
| Table bodies and alias block are kept opaque. `_totalTableSize` and `_maxBufferSize` are preserved, not recomputed (rule unknown) | `Docs/DataFile/02-tadboslb.md`, STATUS | ❓ rule |
| Build 348 table-definition array `DrTableDef[266]` at `0x140e99970`, set up by `Data::DataManager::loadData` `0x14061f8f0` → `DrElReaderImpl::Setup` `0x1406f2760`; field reader `0x1406f2c10` | GameDaemon 348 PDB | ✅ |
| Build 348 has 265 table types; client 0.0.381 has 348 tables with type ids 1–362 (with gaps). Schema versions differ (type 2: 0.1 in 348, 0.39 in client) → build 348 schemas do not fit the client | `Docs/DataFile/03-tabledefs.md`, `client381-tables.json` | ✅ |
| `DrTableDef` 64-bit layout: name ptr 0, i16 type 8, i16 major 10, i16 minor 12, element ptr 16, u8 element count 24, u64 max id 32, bool autokey 40, int module 44 (size 48). `DrAttrDef` 88 bytes with type, offset 72, size 74 | GameDaemon 348 PDB | ✅ (348 only) |
| 2020 server GameDaemon (`BnS_2020/GameDaemon/bin/GameDaemon.exe`, 15,953,920 B, 2020-07-04) is not packed. Table names are stored as UTF-16 (`account-level` found). It is the best source for 0.0.381-era schemas | operator PE section and string scan | ✅ not packed; ❓ build match to 381 |
| Client table metadata list (type, kind, version, body size; no content) saved for matching | `bns-server/Docs/DataFile/client381-tables.json` | ✅ |

## Client table bodies (2026-11-07, main session, read-only walk of build 0.0.381.8884 files)

| Fact | Source | State |
| --- | --- | --- |
| Each table body starts with a u8 compressed flag (0 plain, 1 block-compressed). Server build 348 reader has no such byte. | walk of all 4 files; BnsBinTool `TableHeader.cs` cross-check | ✅ |
| Per datafile: 319 plain (96 kind 1, 223 kind 2), 29 compressed (15 / 14). Per localfile: 3 plain, 1 compressed. Same in both widths. | walk | ✅ |
| Plain header: count (u32; u64 for kind 1 in 64-bit), fixed size u32, extra size u32, low-memory u8 (always 1). Body size = header + fixed + extra exactly. | walk | ✅ |
| Fixed region = DrEl records (6-byte header, u16 size at +4). No legacy 8-byte prefix. | walk | ✅ |
| Kind 1: records = count − 1 (slot 0 not stored). Empty kind 1 tables: count 0 or 1 in 32-bit, 1 in 64-bit. | walk | ✅ |
| Compressed: u32 block count, u16 8 (❓ meaning), then blocks of i64 first key, i64 last key, u16 csize, zlib, u16 dsize, u32 record count, u16 offsets. All blocks inflate to dsize. | walk; BnsBinTool `RecordCompressedReader.cs` | ✅ |
| Compressed records: 824,534 per datafile, 687,415 per localfile. Most carry per-record extra data after the DrEl size. | walk | ✅ count; ❓ extra encoding |
| Full spec | `bns-server/Docs/DataFile/06-client-body.md` | — |

## Operator table definitions: `TableDefs_0.0.381.8876_x86 KR TEST` (2026-11-07)

Location: `<operator-downloads>/TableDefs_0.0.381.8876_x86 KR TEST/` (operator copy, outside the repo). 362 JSON files, 8.4 MB, one per table type (IDs 1–362). Keys: name, type, major_ver, minor_ver, autokey, maxid, module, elcount, el[] (body/sub with attr list: name, namehash, type, key, repeat, offset, size, seq, reftable, …). Origin of the dump ❓ not known.

| Fact | Source | State |
| --- | --- | --- |
| All 348 datafile tables and 4 localfile tables match a definition by ID **and** major/minor version (both widths) | `scratchpad/defcheck.py` against `client381-tables.json` | ✅ |
| 32-bit files: every record DrEl size = definition body/subtype size (datafile 1,081,505 records, localfile 688,221; 0 mismatches) | walk of `datafile.bin`, `localfile.bin` | ✅ |
| 64-bit files: sizes differ in 234 of 348 tables. These definitions are x86 only. | walk of `datafile64.bin` | ✅ |
| A simple rule (string 4→8, native 8→16, natural alignment) gives the 64-bit size for only 98 of 1,115 record shapes | `scratchpad/relayout.py` | ✅ rule rejected; real 64-bit rule ❓ |
| Attribute types used (count): string 1797, ref 1735, int16 1382, int8 787, seq 677, int32 638, bool 596, prop_seq 228, float32 174, msec 124, tref 86, seq16 82, int64 64, distance 61, icon 53, others < 20 | definitions | ✅ |
| Build label 8876 is older than the client data 8884, but every table version matches | definitions vs client header | ✅ versions; ❓ field changes between 8876 and 8884 |

## Operator table definitions: `TableDefs_0.0.381.8876_x64 KR TEST` (2026-11-07)

Location: `<operator-downloads>/TableDefs_0.0.381.8876_x64 KR TEST/` (outside the repo). 362 JSON files, same format as the x86 folder.

| Fact | Source | State |
| --- | --- | --- |
| All 352 client tables match by ID and version | `scratchpad/defcheck64.py` | ✅ |
| 64-bit files: every record size = x64 definition size (datafile64 1,081,505 records, localfile64 688,221; 0 mismatches) | walk of `datafile64.bin`, `localfile64.bin` | ✅ |
| x86 and x64 definitions have the same elements, fields, types and repeats in all 1,946 bodies/subtypes. Only offsets and sizes differ: string 4→8 (1,694 fields), native 8→12 (2 fields) | field-by-field compare | ✅ |
| Both widths now have a verified schema source. Field decoding is unblocked for all four files. | — | ✅ framing/sizes; ❓ extra-data (string/native) encoding |

## Origin of the TableDefs JSON (2026-11-07)

| Fact | Source | State |
| --- | --- | --- |
| The JSON keys copy the in-memory definition structs one to one: `DrTableDef` (name, type, major/minor, el, elcount, maxid, autokey, module), `DrElDef` (name, type, body, child[32]), `DrElBodyDef` (attr, sub, defaultRecord, attrcount, subcount, size, keyoffset), `DrAttrDef` (offset, size, repeat, key, required, min/max, reftable, refel, deprecated, hidden, propfield). Same structs as GameDaemon 348 PDB (`Docs/DataFile/03-tabledefs.md`). | JSON keys vs PDB types | ✅ |
| Table and field names (`abnormalcamera`, `idle-type`, `attractionrewardsummary`) occur once each as UTF-16 in `Client.exe` and `Client (2).exe`. They do not occur in 2020 `GameDaemon.exe`. | byte search, both exes and `ida/datafile/gd2020/GameDaemon.exe` | ✅ |
| The definitions are compiled into the client executable as a static array. The x86 and x64 folders were dumped from the 32-bit and 64-bit client of the KR TEST build 0.0.381.8876. | struct match + string location | ❓ inferred; the dumping tool is not known |
| BnsBinTool uses its own definition format, not this JSON. The dump comes from another tool. | `<ext>/BnsBinTool/Src` grep | ✅ |
| The build of `Client.exe` / `Client (2).exe` in the project root | — | ❓ not checked |

## Proof: TableDefs JSON = dump of the `Client.exe` definition array (2026-11-07)

| Fact | Source | State |
| --- | --- | --- |
| `Client.exe` (project root): PE32 x86, link time 2020-06-04, not packed (normal sections) | PE header read | ✅ |
| Definition pointer array at VA `0x194ece8` (`.data`), 363 slots, slot 0 null; DrTableDef ID 1 `abnormalcamera` at `0x1453260` | pointer search from UTF-16 name `0x17e0ce8` | ✅ |
| Export of all 362 definitions equals the x86 JSON: 104,340 values, 0 differences | `bns-server/Docs/DataFile/client-exe/cmpexe.py` | ✅ |
| x86 struct layouts (TableDef, ElDef 44, ElBodyDef 20, SubDef 16, AttrDef 64) and type codes 1–30 | `bns-server/Docs/DataFile/07-client-exe-tabledefs.md` | ✅ |
| The dump tool is still unknown, but the method is now reproducible by us for any unpacked client build | — | ✅ method; ❓ tool |
| `Client (2).exe`: PE32 x86, link time 2017-07-12. An older build; not a source for 381. | PE header read | ✅ |

## `BNSR_116.exe` (2026 client) — definition array (2026-11-07, passive static read)

| Fact | Source | State |
| --- | --- | --- |
| `BNSR_116.exe` (project root, same bytes as `<operator-downloads>/BNSR_116.exe`, 129,910,272 B): PE32+ x64, link time 2026-10-13. Normal sections (`.text` entropy 6.70), no `.ncg0`. **Not packed**, unlike `BNSR.exe`. | PE header read | ✅ |
| Definition pointer array at VA `0x147484eb0` (`.data`), 520 slots, slot 0 null, IDs 1–519 = slot index | `bns-server/Docs/DataFile/client-exe/arr64.py` | ✅ |
| DrTableDef x64 header unchanged: name ptr 0, type i16 8, major 10, minor 12, el ptr 16, elcount 24. Example: 1 `ability-list` v6.0, 2 `abnormalcamera` v0.3, 519 `zonetriggereventstage` v2.1 | same | ✅ |
| Table IDs differ from 381: `abnormalcamera` is 2 (381: 1). 519 tables (381: 362). Definitions from 381 do not fit 2026 data. | same | ✅ |
| DrElDef layout changed: 4-byte packed, stride 52, name ptr 0, type 8 ❓, body ptr 12, child[32] 20 ❓. Body, sub and attr layouts for 2026 not derived yet. | byte dump of ID 2 elements at `0x14667aaa0` | ✅ stride and body ptr; ❓ rest |
| No 2026 data file to check against yet | — | ❓ |

## Rust table reader/writer verified on real files (2026-11-07, dat4)

| Fact | Source | State |
| --- | --- | --- |
| `bnsdata::table` + `tabledef` parse every table body of all four 0.0.381.8884 files, check every record size against the TableDefs, and write each file back byte-identically | ignored test `client_table_bodies_roundtrip`, run by main session | ✅ |
| Scalar fields decode as raw values; `string` and `native` stay unresolved offsets | `Program/Tools/DataFile/BnsData/src/table/fields.rs` | ✅ code; ❓ string encoding |

## Schema exporter `exportdefs.py` — own TableDefs from executables (2026-11-07)

Tool: `bns-server/Docs/DataFile/client-exe/exportdefs.py` (passive PE read, writes TableDefs-format JSON). Compare tool: `diffdefs.py`.

| Fact | Source | State |
| --- | --- | --- |
| Export of `Client.exe` x86 (array `0x194ece8`, layout `x86`) equals the operator x86 TableDefs 0.0.381.8876 in **all 362 tables, 250,383 fields, 0 differences** (incl. namehash, defvalue, min/max, fmin/fmax, seq lists, propfield, defaultRecord) | `diffdefs.py` run | ✅ |
| x86 DrTableDef tail: maxid u64 +20, autokey u8 +28, module u32 +32 | same | ✅ |
| x86 DrAttrDef (64): name 0, namehash ptr 4 → {u32 hash 0, name ptr 4}, type 8, key 9, repeat u8 10, required 11, defvalue wstr ptr 12, min i64 16, max i64 24, fmin f32 32, fmax f32 36, seq ptr 40, seqcount u16 44, reftable 46, refel 48, deprecated 49, hidden 50, offset 52, size 54, propfield u32 56 | same | ✅ |
| `BNSR_116.exe` layout `x64p4` (4-byte packed): TableDef name 0, type 8, major 10, minor 12, el 16, elcount u8 24, maxid u64 28, autokey 36, module 40. ElDef 52: name 0, type u32 8, body 12, child[32] 20. Body 32: attr 0, sub 8, defrec 16, attrcount 24, subcount 26, size 28, keyoffset 30. Sub 28: name 0, attr 8, defrec 16, attrcount 24, size 26. Attr 84: name 0, namehash 8, type 16, key 17, repeat 18, required 19, defvalue 20, min 28, max 36, fmin 44, fmax 48, seq 52, seqcount 60, reftable 62, refel 64, deprecated 65, hidden 66, offset 68, size 70, propfield 72 | export + compare with 381 x64 JSON: 178 same-version tables, 57,528 fields equal | ✅ |
| 2026 export: 519 tables written to `tabledefs/BNSR_116_x64/` (outside the repo). Loads in `bnsdata::tabledef`. | Rust load check | ✅ |
| The 2026 `BNSR_116.exe` layout does **not** open the 2022 `datafile64.bin` (build 381). First mismatch: `ability-list` is v6.0 in the 2026 layout and v0.3 in the file (kind 2). The 381 x64 TableDefs folder opens it, and edit → pack → reopen passes in both widths. Rule: the layout must come from the same build as the data files. | `bnsmodel::Project` open check (`Program/Tools/DataFile/BnsModel/src/lib.rs`, version/kind compare), bnsedit test `edit_pack_reopen`, 2026-11-07 | ✅ |
| 2026 vs 381 (same-version tables): all 362 `reftable` values point to the same table **name** (only IDs renumbered); 32 seq lists reordered or with inserted values, 165 only appended; `table` element size 44 → 60 in every table; 4 tables changed field offsets without a version change (`skill-training-sequence`, `summoned-sequence`, `surveyquestions`, `tutorialskillsequence`) | same | ✅ |
| 11 tables removed in 2026: exhaustionanim, formula-constant, friend-noti-location, game-tip, item-random-effect-group, item-random-option, itempouchmesh, phantomsword, secondary-stat, skillshow2, skillvariation | same | ✅ |
| New type codes in 2026: 31 (size 8, date fields such as `racoon-store.start-date`), 32 (size 8 or 0; replaces `string` in 429 attributes, max 256), 33 (size 1, enum with seq `NONE, NCK, NCA, NCJ, NCT, INV, TX, NCK_NEO_GOLD`, field `publisher`). Exported as `type_31/32/33`. | same | ✅ codes; ❓ names and encoding |
| 2026 `table` element has a null defaultRecord pointer; 381 points to a zero header | same | ✅ |

## `string` and `native` field encoding (2026-11-07, client 0.0.381.8884, all four files)

| Fact | Source | State |
| --- | --- | --- |
| `string` field = byte offset (u32 in x86, u64 in x64) to a zero-terminated UTF-16LE string | `client-exe/strprobe.py` | ✅ |
| Plain tables: offset is relative to the start of the table **extra region** (datafile x86: 456,218 non-zero values, all start a string) | same | ✅ |
| Compressed tables: offset is relative to the start of the record's **own extra span** (the bytes after the fixed record up to the next record offset) (datafile x86: 14,967,774 values, all start a string). Record-relative base fails (2,985,655 bad). | same | ✅ |
| Offset 0 is a valid offset (first string in the pool) | same | ✅ |
| `native` occurs in two tables only: `text` (type 327, localfile, compressed, 687,415 records) and `petition-faq-list` (233, plain, 62) | x86 TableDefs scan | ✅ |
| `native` x86 (8 B) = u32 **a** + u32 **text offset**; x64 (12 B) = same + u32 0 (always 0, 687,477 records) | `client-exe/natprobe.py` | ✅ |
| Text offset uses the same base as `string`. In `text` the record extra span = alias string, then native text; the text ends with its terminator exactly at the span end (687,415 / 687,415, both widths) | same | ✅ |
| Native text is plain UTF-16LE (96 % zero high bytes, rest in Hangul range, entropy ~3.1 bit/byte). It is not encrypted. It can contain markup. | byte statistics | ✅ |
| Meaning of word **a**: not chars, bytes, or markup-free length in most records. Keep it unchanged; what to write after a text edit is unknown. | same | superseded, see next row and "Extra data" section |
| Word **a** in `localfile.bin` x86 `text` (687,477 records): always even and ≥ 2 (687,477 / 687,477). `a == 2*chars+2` (bytes of the stored text) in only 133,700 (19.4 %); `a < chars` in 263,295; `a > 2*chars+2` in 20,161. So **a** is a byte count, but not of the stored (localized) text. | scratchpad `natprobe2.py`, build 381 | ✅ counts; ❓ it is the byte size of the source-language text |
| bnsdata writer (2026-11-07): unedited records keep the stored **a**. An edited native field gets `a = 2*chars+2`, as `readNative` sets it. Real-file test: localfile x86/x64, 1,374,954 fields resolve, no-op tables stay byte-identical. | `Program/Tools/DataFile/BnsData/src/table/text.rs` `native_size`; `tests/real_files.rs` | ✅ code; ❓ client behaviour with a changed **a** (not tested in game) |
| No type-name string list (`int8` … `time64`) exists in `Client.exe` or `BNSR_116.exe` (only CRT imports match `time64`). The type names in the TableDefs JSON come from the dump tool. Names for 2026 codes 31/32/33 cannot be read from the client. | byte search ASCII + UTF-16, both exes | ✅ |

## Field-walk rules and string verification in Rust (2026-11-07, client 0.0.381.8884, TableDefs 0.0.381.8876)

| Fact | Source | State |
| --- | --- | --- |
| About 250 attributes in the 381 x86 TableDefs have `size` 0 and `offset` 0 (for example `npctalkmessage.name`, `mapinfo.origin-x`, `quest.register-mask`). They take no bytes in client records. | TableDefs scan; scratchpad `strdiag` per-table error count | ✅ |
| These size-0 attributes are server-only columns | inference | ❓ |
| 15 `int16` attributes have `size` 1 (for example `jumpingcharacter.jumping-level`) | TableDefs scan | ✅ |
| With both rules, every record decodes: datafile 1,081,505, localfile 688,221 records (both widths) | `bnsdata` `tests/real_files.rs` `client_strings_resolve` | ✅ |
| All `string`/`native` fields resolve: datafile 16,244,116, localfile 1,374,954; a same-value text edit rewrites one plain and one compressed table byte-identically | same | ✅ |


## Record size facts seen in the editor (build 0.0.381.8884, 2026-11-07)

| Fact | Source |
| --- | --- |
| `item` has 144,439 records; one record decodes to 798 field values (repeats counted) | bnsmodel `Project::record`, datafile64.bin + TableDefs 0.0.381.8876 x64 |
| `text` (largest table) has 687,415 records | same |
| datafile/datafile64 hold 348 tables each; localfile/localfile64 hold 4 tables each | bnsmodel `model_open_search_pack` |
| Opening all four files with both schemas takes about 6 s and 3.5 GB RAM | bnsedit / searchbench, release build |
| One edit to `item` changes 1 table in datafile.bin and 1 in datafile64.bin; the localfiles are copied unchanged | bnsedit `edit_pack_reopen` |

## Link (`ref`) fields: write test (build 0.0.381.8884, 2026-11-07)

| Fact | Source |
| --- | --- |
| A `ref` value stores the 8 key bytes of the target record (`fixed[keyoffset..+8]` of the target table, chosen by `reftable`). Changing it to another target's key bytes makes it resolve to that record. Model reading ✅; the client's own reading ❓ (not tested in the client) | bnsedit `real::tests::edit_pack_reopen`: item row 0, first resolved link (target `text`), repacked and reopened in datafile.bin and datafile64.bin |
| Key 0 means "no link" | bnsmodel `resolve` (all-zero bytes) |

## Schema read from the exe in Rust (`bnsschema`, 2026-11-07)

| Fact | Source |
| --- | --- |
| `bnsschema` finds the definition pointer array without a known address: longest run of slots where slot i points to a definition with `type == i` and a UTF-16 name. | `bns-server/Program/Tools/DataFile/BnsSchema/src/find.rs` |
| Client.exe (PE32, 0.0.381): array found at VA `0x194ece8`, 362 tables (same as the manual search). Export equals TableDefs 0.0.381.8876 x86: 250,383 fields, 0 differences. Takes 0.33 s. | `bnsschema` CLI + `client-exe/diffdefs.py` |
| BNSR_116.exe (PE32+, 2026): array at VA `0x147484eb0`, 519 tables, unknown type codes 31, 32, 33. All 519 JSON files equal the Python `exportdefs.py` x64p4 output. Passed the protection gate. | `bnsschema` CLI vs `exportdefs.py` |
| bnsedit opens datafile.bin with the Client.exe export as the x86 schema; edit, pack and reopen pass. | `apps/bnsedit` `edit_pack_reopen` with `BNSEDIT_SCHEMA32` = export folder |

## GM client 2021 with PDB (`Client_gm.exe`, 2026-11-07)

Files: `<operator-downloads>/GM/bin` (x86) and `bin64` (x64); copies in `ida/gm_client/`. `Client_gm.exe` = `client_gm.bak` (same bytes).

| Fact | Source | State |
| --- | --- | --- |
| `Client_gm.exe` PE32, link time 2021-05-24 (`0x60aa7934`); `Client_gm64.exe` PE32+, 2021-05-23. Both have normal sections (`.text` entropy 6.56 / 5.51). They are the unprotected input files; `client_gm.exe.log` is the WinLicense log for a separate output file. | PE header read; map header; log | ✅ |
| `Client_gm.pdb` (115 MB) is MSF 7.00 with 1 KB blocks; its directory needs 439 blocks, so `llvm-pdbutil` refuses it ("Too many directory blocks"). `bns-server/Docs/DataFile/client-exe/msf_rebloc.py` rewrites it with 4 KB blocks; then llvm-pdbutil reads it. GUID `{A238962B-A299-4CA9-B591-9695722CFA0F}`, age 1. | llvm-pdbutil summary | ✅ |
| Field type enum `DrAttrType`: 0 NONE, 1 INT8, 2 INT16, 3 INT32, 4 INT64, 5 FLOAT32, 6 BOOL, 7 STR, 8 SEQ, 9 SEQ16, 10 REF, 11 TREF, 12 SUB, 13 SU, 14 VECTOR16, 15 VECTOR32, 16 ICOLOR, 17 FCOLOR, 18 BOX, 19 ANGLE, 20 MSEC, 21 DISTANCE, 22 VELOCITY, 23 PROP_SEQ, 24 PROP_FIELD, 25 SCRIPT_OBJ, 26 NATIVE, 27 VERSION, 28 ICON, 29 TIME32, 30 TIME64, **31 TIMEUTC, 32 ASSET**, 33 TYPE_COUNT | PDB type `0x565C6` (`DR_ATTR_*` enumerators) | ✅ 2021 build |
| 2026 `BNSR_116.exe` code 31 (size 8, date fields) fits TIMEUTC. Code 32 (size 8 or 0, replaces `string` in 429 attributes) fits ASSET. Code 33 is past TYPE_COUNT in 2021, so it is a newer type. | name match with 2026 export facts above | ❓ 31/32 inferred, 33 unknown |
| `dr_native` (8 bytes, x86) = `int size` at 0 + `wchar_t* data` at 4. So native word **a** is a size. | PDB type `0x41F50` | ✅ struct; unit: see readNative row |
| `Data::TextRecord` (28 B, x86): base `DrEl` 0, `key` 8 (union, 8 B), `alias` wchar_t* 16, `text` dr_native 20 | PDB type `0x42C94` | ✅ 2021 |
| Other constants: DR_ALIAS_LEN 70, DR_ALIAS_LEN_INTERNAL 200, DR_TABLE_TYPE_LEN 30, DR_TABLE_VERSION_LEN 30, DR_SUB_TYPE_LEN 60, DR_INVALID_EL -1, DR_TABLE_EL 0, DR_MAIN_EL 1 | PDB enumerators | ✅ |
| XML reader for native text: `DrElReaderImpl::readNative` at `0x7a04a0`; also `DeleteElementExtraNativePointer` `0x79bde0`, `PrintErrorNative` `0x79c910` | `Client_gm.map` | ✅ addresses |
| `readNative` (XML input path): `size = 2 * wcslen(text) + 2`, i.e. **bytes of UTF-16 text including the NUL**. Buffer is allocated with that size, zeroed, then text copied. | Hex-Rays decompile of `0x7a04a0`, GM build (branch `419nc`, path in `StlMemory.h` string) | ✅ for the XML reader in GM 2021; ❓ same unit in packed `.bin` records of build 381 (natprobe earlier did not fit plain chars/bytes) |
| GM build is branch **419NC** (Jenkins path `binary_419\platformbinary\depot\branch\419nc`) | string in `readNative` | ✅ |

### Symbols ported to the 2020 Client.exe (2026-11-07)

| Fact | Source | Status |
| --- | --- | --- |
| 2020 exe asks for `Client.pdb` GUID `{47DD5EA4-6D7C-4292-B472-368A4B56D339}` age 1 (RSDS) | `Client.exe` RSDS; path `e:\DailyBuild02\Client-Common-0.0.381\...\Client.pdb` | ✅ |
| Rebuilt PDB: `ida/client2020/Client.pdb` (43 MB, v3: + 835 vtables, 2,072 data names, 132 typed globals; 36,658 functions named in IDA). GM types (TPI) kept; new DBI, publics, globals, one module with S_GPROC32 at 2020 addresses | `scratchpad/mkpdb.py` | ✅ llvm-pdbutil reads it; ✅ IDA 9.5 pdb plugin loads it |
| Functions in 2020 exe: 44,732. Matched to GM: 37,464 (83.8 %). Named after PDB load: 36,579; with function type: 34,335. v1 (exact keys only) had 23,870 | `match.py` + `match2.py` (linker-order gap alignment, call-graph fuzzy), `pdbtest.py` | ✅ |
| Linker order is kept: 87 % of matched pairs are in the same order in both builds | `match2.py` LIS check | ✅ |
| Source path strings differ: 2020 `E:\\DailyBuild02\\Client-Common-0.0.381\\...`, GM `D:\\Jenkins\\workspace\\NCSoft\\Binary_419\\...` | function strings | ✅ |
| Unmatched 7,268: 1,007 < 16 B, 1,557 < 64 B, 4,453 < 1 KB, 251 large; 561 use strings | `m3.json` | ✅ counts; ❓ how many are removed in 2021 |
| Locals copied only for byte-identical functions (8,926) | `mkpdb.py` | ✅ |
| Match list (GM VA → 2020 VA, method): `ida/client2020/Client_symbol_matches.json` | — | ✅ |
| `DrElReaderImpl::readNative` matched in 2020 by call-graph fuzzy (2020 string `e:\\dailybuild01\\platformbinary-common-0.0.0\\...`) | `m3.json` | ✅ match; ❓ decompile 2020 version |
| Works in WinDbg / Visual Studio (DIA) | — | ❓ not tested |

### Extra data of packed elements: STR and NATIVE (build 381, 2020 `Client.exe`)

Functions found through the ported PDB. The 2020 vtable `??_7DrElReaderImpl@@6B@` is at `0x187112c` (10 slots; GM 2021 has more). Slot order 0–9 is the same as in GM.

| Fact | Source | Status |
| --- | --- | --- |
| `DrElReaderImpl::AllocedSize` (slot 5, `0x619eb0` → `0x619ed0`): extra bytes of one element = Σ over STR (type 7) fields `2*wcslen+2` + Σ over NATIVE (type 26) fields `dr_native.size`. It walks the main table attrs and then the sub-type attrs (`el+2` = sub index, `-1` = none). | Hex-Rays, 2020 `Client.exe` | ✅ |
| So native word **a** (`dr_native.size`) = **byte count** of the native data in the extra buffer (in-memory elements). In packed build-381 `.bin` records the stored value often differs from the stored text (see the `a` measurement row). From XML it is set to `2*chars+2` (`readNative` 2020 `0x61cb90`, GM `0x7a04a0`). | same + readNative | ✅ |
| `RelocateAddress` (slot 9, `0x61ab50`): adds the buffer base to the dword at the STR field offset, and to the dword at NATIVE field offset **+4** (data pointer). NATIVE `+0` (size) is not changed. So packed STR / NATIVE data pointers are offsets into the element extra buffer. | Hex-Rays 2020 | ✅ |
| Repeat fields: count = attr byte `+10`, offset = attr u16 `+52`, stride = attr u16 `+54`; type = attr byte `+8`; attr def size 64; attr skipped when byte `+49` != 0, or when byte `+9` != 0 and table byte `+4` != 1 | same | ✅ offsets; ❓ meaning of `+9`, `+49`, table `+4` |
| Element skipped when `el[0] == 0xFF` or u16 `el+4` == 0 | same | ✅ code; ❓ meaning |
| `DrMultiKeyTable::constructExtraBuffer` (`0x621e80`): pass 1 sums `AllocedSize` for all elements, allocates one buffer, pass 2 calls `RelocateMemory` (slot 6, `0x61a0a0`) to copy into it | Hex-Rays 2020 | ✅ |

## Cross-check: leanleon93/BnsPluginTables (checked 2026-11-07)

| Fact | Value | Source |
| --- | --- | --- |
| What it is | Generated C++ record structs for the current **NEO** client, EU and KR (528 tables each). Auto-committed by a bot; last commit 2026-11-03. | github.com/leanleon93/BnsPluginTables, `Generated/include/{EU,KR}/<table>/AAA_<table>_RecordBase.h`, `PluginTableCount.h` |
| Record header | `DrEl { char type; int16 subtype; uint16 size; }`, then an 8-byte key union at offset 8. | `Generated/include/DrEl.h` |
| Version / id | `TableVersion(major, minor)` and `TableId()` per record struct. | each `AAA_*_RecordBase.h` |
| Vs. our `tabledefs/BNSR_116_x64` | 75 tables have the same id and version. In 73 of them, **926 field offsets all match** our export (0 mismatches). 444 tables have newer versions in their repo, and ids shift by +1 from about id 81 (`contributionreward` 82 vs 81). 44 tables exist only in theirs. So their headers are from a **later** NEO build than BNSR_116. | scratchpad script `cmp_bpt.py`, run 2026-11-07 |
| Use for build 381 (2020, UE3) | None directly: it covers NEO only. It confirms that our layout-export method gives correct offsets. | — |
| Licence | "Non-Commercial, No Monetization License". Use it as a cross-check only; do not copy its files into the repo. | `LICENSE` |

## Cross-check: war100ck/BnsTools (checked 2026-11-07)

| Fact | Value | Source |
| --- | --- | --- |
| What it is | C# WPF (Prism) editor, Chinese UI. 0 stars, last push 2024-04-09. No LICENSE file, so do not copy its code. | github.com/war100ck/BnsTools |
| Server data it edits | **Plain** XML in a server folder (`Config.ServerPath`, `*.xml`, `XDocument.Load`). It expects already-decoded files. | `Modules/ServerEditor/ViewModels/EditingFileViewModel.cs:499`, `Modules/BatchBoostJob/ViewModels/BatchBoostJobViewModel.cs:76` |
| Our 2020 GameDaemon data | 1,108 top-level files, 4.8 GB, 21,073 `*.xml` in total (zone 18,319, quest 2,610, script 401, terrain 165, social 88). The files start with `LMXBOSLB` (binary XML), not plain XML. | `<operator-downloads>/BnS_2020/GameDaemon/data/`, e.g. `effectdata_forcemaster.xml` |
| Old client `.dat` + binary XML code | `Modules/Translate/lib/BNSDat.cs` and `BDat.cs` pack/unpack the old client `.dat` archives and convert binary XML. Not read in detail. ❓ Whether it matches the server `LMXBOSLB` files is not checked. | BnsTools repo |
| Shop DB | `Modules/ExtractItem/DAO/GoodsDb/Models/*` (21 models: Goods, GoodsItems, GameItems, ...) map the GoodsDb tables. | BnsTools repo |
| Launcher | None. | BnsTools repo (no `Process.Start`) |

## Server address: `release.config2.xml` (client build 381, Client.exe 32-bit)

Source: static read of `ida/client2020/Client.exe` (PDB names). Date 2026-11-07.

- `sub_10A9DE0` loads `release.config2.xml` through `sub_10A3C80` (XmlConfigReader DLL: `CreateXmlConfigReader`, `DecodeFromConfigPacket`). The result replaces the global `GReleaseConfig` (0x194F654).
- The package handle comes from `EInterfaceSetClientInfo` table entry [128]. Which `.dat` holds the file (probably `config.dat` / `config64.dat`) ❓.
- Built-in default when no file loads: lobby gate and NP address `70.102.0.96`, port 6600 (0x19C8). Source: data at `*GReleaseConfig` = 0x1493130.
- `ClientReleaseConfig` fields used for login: `_lobbyGateAddress` +0x0, `_lobbyGatePort` +0x4, `_npAddress` +0x8, `_npPort` +0xC, plus a second set `_lobbyGateAddress2` +0x34 … `_npPort2` +0x40.
- `BNSClient::Initialize` (0x116F550) passes one set to `Universe::SetUniverseInfo` (0x440150). It uses set 2 when the global `Value` (0x1955C40) > 0, else set 1. `Value` is written in `BNSClient::DoCommandLine` next to `publisher` (0x113F96B). The command-line option that sets it ❓.
- The XML option names are not in Client.exe as plain strings ❓ (the reader is generic; names likely come from the reader DLL schema).
- `TitleLoginPanel::loginReq` (0x7FF420) and `NPSession` read the address from `Universe`, not from the command line.

## LMXBOSLB readers in bns-server: BnsData vs Lmxb (2026-11-08)

Source: code comparison by Sol, cleanup job 1. Paths are in bns-server/.
| Property | BnsData | Lmxb | Source |
| --- | --- | --- | --- |
| Strings | Keeps raw UTF-16 units, including unpaired surrogates | Converts to `String` and rejects invalid UTF-16 | `Program/Tools/DataFile/BnsData/src/lmx.rs`: `Utf16`, `read_string`; `Program/Server/Data/Lmxb/src/lib.rs`: `Reader::string` |
| Header | Keeps three padding bytes after the version byte | Reads a u32 version that must equal 3; padding must be zero | BnsData: `header`, `Document::write`; Lmxb: `decode`, `encode` |
| Text records | Keeps a full node, including children | Rejects children on text records | BnsData: `Text`, `read_node`, `put_node`; Lmxb: `Text`, `Reader::text` |
| Depth | Allows depth through 256, with root depth 0 | Uses a stack limit of 128 | BnsData: `MAX_DEPTH`, `depth_ok`; Lmxb: `MAX_DEPTH`, `decode` |
| Key input | `Document::parse` and `write` take a key from the caller | `decode` and `encode` have no key argument | Public function signatures in the same source files |
| Structure scan | `walk` counts nodes and strings without decoding string payloads | No equivalent public structure-only scan in the compared reader | BnsData: `Structure`, `walk`; Lmxb: reader API |

These differences prevent a behavior-preserving replacement of BnsData with Lmxb.
Both implementations remain unchanged.

❓ Not checked against an original binary or build. Relation to datafile.bin / datafile64.bin / locale.bin not established.

## Source: BNSR 2020 UE4 client archive (operator archive)

Source: `unrar` listing of the archive, 2026-11-08. All archive entries are dated 2020-01-14 18:08:16.

| File in archive | Size (bytes) | Note |
| --- | ---: | --- |
| `BNSR/Content/local/ncsoft/data/datafile.bin` | 92 666 369 | Extracted to `BNSR_2020_client/BNSR/Content/local/ncsoft/data/`. ❓ Format version and key not checked. |
| `BNSR/Content/local/ncsoft/data/datafile64.bin` | 108 861 977 | Extracted to `BNSR_2020_client/BNSR/Content/local/ncsoft/data/`. ❓ Format version and key not checked. |
| `locale.bin` | — | Not in this client. The Korean text is 2486 loose files in `BNSR/Content/local/ncsoft/Korean/data/`: `text/TextData.<name>.x16` files, `UserCommandData.x16`, and chat/name filter `.txt` files. ❓ Relation of `.x16` to the `locale.bin` format not checked. |

The archive also holds `BNSR/Binaries/Win64/BNSR.exe` with a matching `BNSR.pdb` (PDB GUID f3f1c4a98f5640218a0e5371c82ac851, exe age 1, PDB age 2). The exe has normal sections (no packer section names). These are extracted in `ida/sts-client/bnsr2020/`. The PDB can name the client code that reads the data files. ❓ Not analysed yet.

## Server-side achievement tables (AchievementDaemon 0.0.348, GameDaemon data, 2026-11-08)

These are server tables, not client data files. They share the `release-module="ItemData"` table family. ❓ Whether the client `datafile.bin` holds the same `achievement` / `achievement-register` tables is not checked.

| Fact | Source |
| --- | --- |
| AchievementDaemon reads `AchievementData*.xml` and `AchievementRegisterData*.xml` from the `data-path` config option. | `ida/small/AchievementDaemon.exe` (build path `AchievementDaemon-Common-0.0.348`), config `BnS_2020/AchievementDaemon/config/achievementd.config.xml` |
| The daemon sample files `BnS_2020/AchievementDaemon/data/AchievementData.xml` (3 records, `type="achievement"` v0.19) and `AchievementRegisterData.xml` (v0.41) are plain XML. | File contents |
| The GameDaemon copies `BnS_2020/GameDaemon/data/achievementdata.xml` (770 651 bytes) and `achievementregisterdata.xml` (366 417 bytes) are LMXBOSLB binary. The `lmxb` crate decodes both. | `lmxb::decode` + `to_xml_string` run, 2026-11-08 |
| `achievement-register` v0.42: 1067 records. Attributes: `id`, `alias`, `type`, `job`, `max-value`, `version`, `persistant`, plus type-specific keys (`boss-npc`, `npc-N`, `quest`, `env2-N`, `item-brand`, `skill-training-room-group`, `gadget`, `team-broadcast`, `for-newbie-care`, `tendency-id`, `env2-state-N`, `seed-item-brand`, `item`, `equip-gem-only`). 52 distinct `type` values (top: `equip-item-brand-n` 216, `kill-boss-npc` 210, `use-grocery` 189). | Decoded `achievementregisterdata.xml` |
| `achievement` v0.19: 1558 records (one per id+step+job; steps 1..7). Attributes: `id`, `step`, `job`, `alias`, `current-step-score`, `register-ref-1..5` (register alias), `register-type-N` (`above` 1344, `bits-on` 420), `register-value-N`, `deprecated`, `complete-from-client` (3 records), `completed-effect`, `step-complete-reward-item-N`, `step-complete-reward-item-count-N`, `step-complete-reward-skill-build-up-point`, `step-complete-reward-game-cash`. | Decoded `achievementdata.xml` |
| `job` values are `job-none` or Korean job names (검사, 권사, 기공사, 역사, 암살자, 소환사, 귀검사, 주술사, 기권사, 격사, 투사, 궁사), 64 records each. | Decoded `achievementdata.xml` |
| Condition ops in the daemon: 1 = value ≥ v (`above`), 2 = value ≤ v (❓ name `below`, not in data), 3 = low v bits all set (`bits-on`). | `checkComplete` in AchievementDaemon 348 |

## Server-side ranking tables (RankingDaemon 0.0.348, 2026-11-08)

These are server tables in the `release-module="ItemData"` family. ❓ Whether the client `datafile.bin` holds the same `season-contents-schedule` table is not checked.

| Fact | Source |
| --- | --- |
| RankingDaemon reads `SeasonContentsScheduleData.xml` from `data-path`. | `ida/small/RankingDaemon.exe` 348 (PDB names); `Docs/Server/RankingDaemon/README.md` |
| `BnS_2020/RankingDaemon/data/SeasonContentsScheduleData.xml`: plain XML, `release-side="ranking"`, `type="season-contents-schedule"`, version 0.4, 3012 records. | File contents, counted 2026-11-08 |
| `BnS_2020/ArenaLobby/data/SeasonContentsScheduleData.xml`: same type and version, `release-side="arenalobby"`, 2832 records. | File contents |
| `BnS_2020/RankingDaemon/data/RankingRewardGroupData.xml`: `type="RankingRewardGroup"`, version 0.6, 72 records. | File contents |
| The GameDaemon copies `seasoncontentsscheduledata.xml` (3 901 681 bytes) and `rankingrewardgroupdata.xml` (253 147 bytes) are LMXBOSLB binary. | File header |
| Record keys used by the daemon: `service-mode`, `season-contents-type`, `season-contents-schedule-number`, `season-name`, `season-type`, `player-type`, `season-start-time`, `season-end-time`, `season-reward-expiration-time`, `reward-group-data`, `default-rating-score`, `ranking-minimum-score`. Other keys exist (`accumulation-period`, `closing-notice-period`, `enable-weekly-ending-schedule…`). | File contents; `rankingdaemon::config::parse_season_data` |
| Time format `YYYY-MM-DD H:MM:SS` in server **local** time (verified: `_localtime64_s` on start time for weekday/hour). The port uses `RANKINGD_UTC_OFFSET_MINUTES`. | RankingDaemon 348 `loadHiddenFieldFromSeasonContentsSchedule` @0x140004480, `insertSeasonFromData` @0x140029600 |
| Full record field order (field table @0x1400E1D38): … season-reward-expiration-time, closing-notice-period (+64, i16 min), accumulation-period (+66), enable-weekly-ending-schedule (+68), weekly-ending-day (+69, sun = 0), weekly-ending-hour (+70), reward-group-data (+72), enable-weekly-ending-schedule-for-qa (+80), weekly-ending-schedule-for-qa-1..20 (+88), weekly-ending-schedule-count-for-qa (+248), forced-termination (+249), hidden season-result-schedule-count/1.. (+250/+256). | RankingDaemon 348 IDA, 2026-11-08 |
| 2020 RankingDaemon table: 2189 records have the QA list flag `y`, 239 the weekly flag, 91 `forced-termination="y"`; all `weekly-ending-day` values are `wed`. | Attribute counts, 2026-11-08 |
| Enums: season-contents-type 0..15 (`duel-death-match-1vs1` = 1, `guild-combat` = 8, `challenge-party-best-record` = 13, battle-royal 14/15); season-type none/free/normal; player-type none/pc/guild/challenge-party; service-mode 0 none … 6 `live-rc` … 11 `addition-2`. | RankingDaemon 348 enum name tables (IDA) |
| Max ranking list index by contents type: 1, 3–7, 9–12, 14, 15 → 14; 8 → 2; 13 → 1; else 0. | `convertSeasonContentsTypeToMaxRankingListIndex` @0x14001BAE0 |
| The exporter leaves out attributes at their default: `reward-group-data` is in 2433 of 3012 records; `weekly-ending-day` in 238. The ArenaLobby copy has no `season-reward-expiration-time` at all (a ranking-side field). | Attribute counts, 2026-11-08 |
| The 2020 table has contents type `mastery-exp-best-record` (8 records). The 348 RankingDaemon enum @0x1400E15C0 has only 16 names and lacks it, so the 2020 data is newer than build 348. | IDA RankingDaemon 348; file contents |
| The `rankingdaemon` parser loads the RankingDaemon table as 3004 seasons (8 skipped) and the ArenaLobby table as 2832. | `cargo run -p rankingdaemon --example check_seasons`, 2026-11-08 |

## Client-side loading of `datafile.bin` / `localfile.bin` (live client, 2026-11-08)

Observed on the operator laptop: packed retail client, build **381.8884**, 32-bit,
at `...\BNS_Client\bin` (`Client.exe`, Themida-packed). It carries a private-server
plugin stack loaded through a proxy DLL.

| Fact | Source |
| --- | --- |
| A proxy `winmm.dll` (256 512 bytes, **no Microsoft version info**) sits in `bin\`. The client imports `winmm.dll` for audio; `winmm` is not SxS-managed, so the proxy loads from the app dir and then loads every DLL in `bin\plugins\`. `loader.dll` and `version.dll` (both no MS version info) are the same loader family. | `bin\winmm.dll`/`loader.dll`/`version.dll` VersionInfo empty; crash-dump module list |
| **`binreplace.ini`** tells `binreplace.dll` which files to substitute into the client at load:  `datafile=..\contents\local\NCSoft\data\datafile.bin` and `localfile=..\contents\local\NCSoft\data\localfile.bin` (paths relative to `bin\`). So the client reads `datafile.bin` and `localfile.bin` from `contents\local\NCSoft\data\`, injected by the `binreplace` plugin. | `bin\plugins\binreplace.ini` |
| Base client config lives in `contents\local\NCSoft\data\config.dat` / `config64.dat` (~3.16 MB binary, not plaintext XML). Server address is overridden on top of it by `bin\patches.xml` / `patches_2.xml` (`<patch file="release.config2.xml">` transforms). | `bin\patches.xml`; file listing |
| Client server target set by us: `lobby-gate-address` and `np-address` → `<server-ip>` in both `patches.xml` and `patches_2.xml`. No port option in the transforms, so `lobby-gate-port` stays at the base-config retail value (10900), which matches our LobbyGate `client-port` default. | Edited 2026-11-08; `Docs/Server/LobbyGate/03-framing.md` |
| ❓ The on-disk `datafile.bin`/`localfile.bin` in `contents\local\NCSoft\data\` were not opened or version-checked in this session. Their format vs. the in-client table is unverified. | — |
| ❓ A direct `Client.exe` launch crashed in `pathengine.dll` (`0xc0000005`, offset `0x79d0`), build 381.8884. Cause unverified (standalone launch vs. no reachable server vs. data mismatch). | WER event id 1000, 2026-11-08 |
