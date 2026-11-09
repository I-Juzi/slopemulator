# 00 — Client.exe 381: binary inventory and method

Summary:

1. `Client.exe` 0.0.381 is a 32-bit MSVC 8.0 (Visual C++ 2005) PE file. Image base `0x400000`. It has 45,392 functions.
2. The exe holds the game logic only. The Unreal engine is in `bsengine_Shipping.dll`. `LoadEngine` (`0x114c6d0`) loads it at run time.
3. Static libraries in the exe: OpenSSL 1.1.x, zlib (3 copies), JsonCpp, MSVC STL. They use about 8,800 functions.
4. 36,971 functions (81.4 %) have a name from the 419 GM PDB. 32,116 of them have a high confidence.
5. [`classes.tsv`](classes.tsv) (3,650 classes) and `functions-1..5.tsv` (45,392 rows) are the lookup tables. §5 gives the script that makes them.

```
 Client.exe (game logic, 0x401000-0x139a000)
   │  start 0x703cfa (loader stub) ──► _WinMain@16 0x1190f60
   │                                      └─► BNSClient::Initialize 0x116f550
   │                                            ├─► XmlReader_cl32.dll  (string 0x14fdd38)
   │                                            └─► LoadEngine 0x114c6d0 ──► bsengine_Shipping.dll (string 0x14fe00c)
   ├─ static: OpenSSL 0x633d30-0x6f0f70, zlib, JsonCpp 0x707a40-0x711680, STL
   └─ run-time DLLs: StsMsgCli32.dll (0xbc90a0), TerSafe/Tencent, NCTalk.exe, game_presence
```

Sources: all facts come from the 381 database `ida/lanes/cx/Client.exe.i64` (MD5 of the input file `874b3ab4a2a3332a8bb913222e8b9374`). "419 GM" marks a fact from `Client_gm.exe.i64`. ❓ marks a fact that we did not verify.

---

## Quick find

| Task or question | Function (381 name) | 381 address | 419 GM address (same name) | Section |
| --- | --- | --- | --- | --- |
| Program entry (loader stub, not game code) | `start` | `0x703cfa` | — | §1.5 |
| Main function | `_WinMain@16` | `0x1190f60` | `0x14728f0` | [01-startup](01-startup.md) |
| Client start, loads the XML reader DLL | `BNSClient::Initialize` | `0x116f550` | `0x146bd10` | §1.5, [01-startup](01-startup.md) |
| Where the engine DLL loads | `LoadEngine` | `0x114c6d0` | `0x1433c60` | §1.5, [15-engine](15-engine.md) |
| Where the STS client DLL loads | `NPStsMsgLib::Create` | `0xbc90a0` | `0xe2a280` | [03-sts-client](03-sts-client.md) |
| Lobby session object | `LobbyGateSession::LobbyGateSession` (vtable `0x15e9a80`) | `0xc673e0` | `0xed33e0` | [04-lobbygate](04-lobbygate.md) |
| Game session object | `GameSession::GameSession` (vtable `0x15896b8`) | `0xefbf40` | `0x1199da0` | [05-game-dispatch](05-game-dispatch.md) |
| Game ping send | `GameSession::SendPing` | `0x47d7a0` | `0x48b330` | [05-game-dispatch](05-game-dispatch.md) |
| Packet encryption entry (no key data here) | `GamePacketProcessor::Encrypt` / `Decrypt` | `0x608f10` / `0x608fb0` | `0x6f3150` / `0x6f3240` | [02-transport](02-transport.md) |
| Session key set | `GamePacketProcessor::SetSessionKey` | `0x608d10` | `0x6f2f40` | [02-transport](02-transport.md) |
| Byte stream to packets | `PacketParser::ParseData` | `0x5fd770` | `0x6e3570` | [02-transport](02-transport.md) |
| Where the client sends a move | `MoveListMaker::sendWalkPacket` | `0xcc4ce0` | `0xf23340` | [06-movement](06-movement.md) |
| Move list tick | `MoveListMaker::Tick` | `0xcc93d0` | `0xf264a0` | [06-movement](06-movement.md) |
| Sprint, drift, boost tick | `HyperMove::Tick` | `0xe82b90` | `0x11121b0` | [06-movement](06-movement.md) |
| Player object (constructor, tick) | `BPTPlayer::BPTPlayer` / `BPTPlayer::Tick` (vtable `0x16187a0`) | `0xae3140` / `0xb36040` | `0xcaa400` / `0xcf0670` | [08-creature](08-creature.md) |
| Info server reconnect | `InfoGateSession::ReconnectInfoGate` | `0x1131520` | `0x10b4590` | [12-infogate-arena](12-infogate-arena.md) |
| Datafile native string read | `DrElReaderImpl::readNative` | `0x61cb90` | `0x7a04a0` | [13-datafile](13-datafile.md) |
| Client log file | `LogManager::CreateLogFile` | `0x5e6630` | `0x6d7160` | [16-strings-errors](16-strings-errors.md) |
| Auto fishing stop | `Fishing::StopAutomaticFishing` | `0xf7b2c0` | `0x1219000` | [10-quests-npc](10-quests-npc.md) ❓ |
| Tencent TCLS proxy load | `CreateTclsProxy` | `0x54ca60` | `0x565ce0` | §1.5 |
| Any function by name or address | `grep` in `functions-*.tsv` | — | — | §4 |
| Any class, its vtable and constructor | `grep` in `classes.tsv` | — | — | §2 |

Grep examples:

```
grep -h $'\tMoveListMaker\t' Docs/Client/RE/functions-*.tsv      # all methods of a class
grep -h '^0xcc4ce0' Docs/Client/RE/functions-*.tsv               # one address
grep -h 'StsMsgCli32' Docs/Client/RE/functions-*.tsv             # function by string anchor
awk -F'\t' '$8=="04-lobbygate"' Docs/Client/RE/functions-*.tsv  # one area
```

---

## 1. Binary map

### 1.1 Image

| Item | Value | Source |
| --- | --- | --- |
| File | `Client.exe`, PE32, x86 | IDA root file name |
| Image base | `0x400000` | `get_imagebase()` |
| Entry point | `start` `0x703cfa`. It calls `_PDLoaderEntryForExe` (`0x703d06`), then `_OnModuleLoad` (`0x704249`). | entry table, call list of `start` |
| Main function | `_WinMain@16` `0x1190f60` (365 bytes) | name, 419 GM same name at `0x14728f0` |
| Exports | none (only the entry point) | `get_entry_qty() = 1` |
| Functions | 45,392 | `idautils.Functions()` |
| Build PDB path (string) | `e:\DailyBuild02\Client-Common-0.0.381\depot\Branch\381NC\Program\output\client\bin\Client.pdb` | `0x1871ef4` |
| Other PDB path (string) | `d:\work\Bloodlust\Program\library\RestrictLib\tool\output\wiper.pdb` | `0x14922f0` |
| Vendor strings | `NCsoft Corp.` `0x186e558`, `NCSOFT Corporation` `0x1870994` | strings |

The internal project name is "Bloodlust" (PDB path `0x14922f0`).

### 1.2 Sections

| Section | Start | End | Size | Access | Content |
| --- | --- | --- | --- | --- | --- |
| `.text` | `0x401000` | `0x139a000` | `0xf99000` | R X | all code |
| `.idata` | `0x139a000` | `0x139ab24` | `0xb24` | R | import address table (§1.4) |
| `.rdata` | `0x139ab24` | `0x193e000` | `0x5a34dc` | R | vtables, strings, constant tables |
| `.data` | `0x193e000` | `0x1b3b000` | `0x1fd000` | R W | globals |
| `.tls` | `0x1b3b000` | `0x1b3c000` | `0x1000` | R W | thread local data |
| `.idata` (2nd) | `0x1b98000` | `0x1b9f000` | `0x7000` | R | a second copy of the import names and DLL names (for example `WS2_32.dll` at `0x1b9e4b0`). It belongs to the loader stub ❓ |

419 GM sections for comparison: `.text 0x401000`, `.idata 0x17fc000`, `.rdata 0x17fdbc4`, `.data 0x1f9a000`, `.tls 0x24f8000`, `.tdata 0x24f9000`. 419 GM has 63,457 functions.

### 1.3 Compiler and runtime

| Item | Value | Source |
| --- | --- | --- |
| Compiler | Microsoft Visual C++ 8.0 (Visual Studio 2005). Service pack level ❓ (Rich header not read). | imports `MSVCR80` (205 names), `MSVCP80` (179 names); DLL name `game_presence-32-vc_v80-MD` |
| C runtime | `MSVCR80.dll`, dynamic (`/MD`) | import table |
| C++ runtime | `MSVCP80.dll` | import table |
| Name mangling | MSVC. 34,220 function names start with `?`. | function names |
| Security cookie | `/GS` is on: `___security_init_cookie` `0x6f23e4`, `___report_gsfailure` `0x6f202c` | FLIRT names |
| SEH helpers | `__SEH_prolog4` `0x6f2130`, `__SEH_epilog4` `0x6f2175`, `__chkstk` `0x6f1c30`, `__alloca_probe` `0x1491f68` | FLIRT names |
| RTTI | Almost all removed. Only 92 `.?AV`/`.?AU` type names remain, mostly `std::` (first: `0x1946100`). | strings |

### 1.4 Import table (19 DLLs)

| DLL | Names | IAT range | Use (from the import names) |
| --- | ---: | --- | --- |
| `KERNEL32` | 152 | in `0x139a000-0x139ab24` | base API. Includes `LoadLibraryW` `0x139a0e4`, `LoadLibraryA` `0x139a18c`, `GetProcAddress` `0x139a0e8`, `CreateFileW` `0x139a12c` |
| `MSVCR80` | 205 | — | C runtime |
| `MSVCP80` | 179 | — | C++ runtime |
| `ADVAPI32` | 41 | `0x139a000-0x139a0a0` | registry, services, Windows CryptoAPI (`CryptEncrypt`, `CryptImportKey`, `CryptVerifySignatureA` …) |
| `CRYPT32` | 11 | `0x139a0a8-0x139a0d0` | certificate store read (`CertOpenStore`, `CryptQueryObject` …) |
| `GDI32` | 2 | `0x139a0d8-0x139a0dc` | `AddFontResourceW`, `RemoveFontResourceW` |
| `MSWSOCK` | 1 | `0x139a950` | `AcceptEx` |
| `NETAPI32` | 1 | `0x139a958` | `Netbios` (MAC address, see `GameSession::MakeMacAddress` `0xefba60`) ❓ |
| `RPCRT4` | 7 | `0x139a960-0x139a978` | UUID functions |
| `SHELL32` | 5 | `0x139a980-0x139a990` | `CommandLineToArgvW`, `ShellExecuteW`, `SHGetFolderPathW` … |
| `SHLWAPI` | 1 | `0x139a998` | `PathFileExistsW` |
| `USER32` | 7 | `0x139a9a0-0x139a9b8` | `MessageBoxW`, `PostMessageW` … (no window class code: the engine DLL owns the window ❓) |
| `VERSION` | 3 | `0x139a9c0-0x139a9c8` | file version read |
| `WININET` | 14 | `0x139a9d0-0x139aa04` | HTTP client (web shop, web login) |
| `WINMM` | 13 | `0x139aa0c-0x139aa3c` | mixer and `timeGetTime` |
| `WS2_32` | 43 | `0x139aa44-0x139aaec` | sockets (`WSASocketW`, `WSAConnect`, `WSARecv`, `WSASend` …) |
| `game_presence-32-vc_v80-MD` | 2 | `0x139aaf4-0x139aaf8` | `RegisterGamePresence`, `UnregisterGamePresence` |
| `IPHLPAPI` | 2 | `0x139ab00-0x139ab04` | `GetAdaptersInfo`, `GetNetworkParams` |
| `ole32` | 5 | `0x139ab0c-0x139ab1c` | COM start and stop |

There is no D3D, no sound and no engine import. The engine comes in at run time (§1.5).

### 1.5 Modules loaded at run time

All rows: callers of `LoadLibraryW` / `LoadLibraryA` / `GetProcAddress` (xrefs to the IAT slots in §1.4).

| Module | Loader function | 381 address | Anchors |
| --- | --- | --- | --- |
| `bsengine_Shipping.dll` (Unreal engine) | `LoadEngine` | `0x114c6d0` | wide strings `bsengine_Shipping` `0x14fe00c`, `.dll` `0x14fe030`, `Engine Version invalid. (Current Version : %s . Loaded Version : %s )` `0x14fd900`. Imported by name: `EInterfaceGetInstance`, `EInterfaceInitialize`, `EInterfaceCleanup`, `EInterfaceSetClientInfo`, `EInterfaceGetClientInfoCount`, `EInterfaceGetClientInfo`, `UIEngineInterfaceGetInstance`, `EInterfaceAddEngineObjectRef`, `EInterfaceRemoveEngineObjectRef`, `EInterfaceGetInterfaceVersion`, `EInterfaceGetPlatformBinaryVersion`, `EInterfaceErrorReport`, `EInterfaceMalloc`, `EInterfaceRealloc`, `EInterfaceFree` |
| engine allocator | `sub_5D21A0` | `0x5d21a0` | `EInterfaceMalloc`, `EInterfaceRealloc`, `EInterfaceFree` |
| `XmlReader_cl32.dll` (datafile and XML reader) | `BNSClient::Initialize` | `0x116f550` | wide strings `XmlReader_cl32.dll` `0x14fdd38`, `xml-reader Version invalid. …` `0x14fddb0`, `config.dat` `0x14fde48`, `xml.dat` `0x14fde60`, `local.dat` `0x14fde70`. Imported by name: `GetInterfaceVersion`, `SetMemAllocator`, `CreateXmlReader`, `DestroyXmlReader` |
| `StsMsgCli32.dll` (STS client) | `NPStsMsgLib::Create` | `0xbc90a0` | wide string `./StsMsgCli32.dll` `0x15f2674` |
| Tencent TCLS proxy | `CreateTclsProxy` | `0x54ca60` | `Invoke` |
| `TerSafe.dll` (Tencent client protection) | `AntiBot::CreateAntiObj` | `0x54cf00` | `TerSafe.dll` `0x1667614` |
| `gameShieldDll.dll` | `shieldSecurityDll::loadLibrary` | `0x6337a0` | `\frost\gameShieldDll.dll` `0x139d958`, `frostInitialize` `0x139da50` |
| GameGuard | `CGameGuard::InitNPSC` | `0x6f81f0` | — |
| NC guard interface | `_LoadNCGuardClientInterface@8` | `0x12c56d6` | — |
| WinSock helper | `_WspiapiLoad@4` | `0x5b11c0` | `freeaddrinfo` |
| OpenSSL service check | `_OPENSSL_isservice` | `0x66b5f0` | — |
| process list and symbol read (crash report ❓) | `sub_6D37C0`, `sub_6D3240`, `sub_6D33F0` | `0x6d37c0`, `0x6d3240`, `0x6d33f0` | `CreateToolhelp32Snapshot`, `filename(`, `symname(` |
| other (no anchor) | `sub_6F46C0`, `sub_6F7070`, `CLogServer::_InitSafeFunction` | `0x6f46c0`, `0x6f7070`, `0x6fc8e0` | — |

Other program names in strings: `./NCTalk.exe` `0x14f4e04`, `./Talk/VC/NCTalk.exe` `0x14f568c` (NC chat client), `..\bin\Version.ini` `0x14fd9c4`, `..\bin\SystemText.ini` `0x14fdfac`.

This chapter lists the protection modules by name and address only. It does not describe them.

### 1.6 Static libraries in the exe

| Library | Version | Evidence (381 address) | Code range | Functions (area tag) |
| --- | --- | --- | --- | --- |
| OpenSSL | 1.1.x; 1.1.1 because ED448 and X448 exist only from 1.1.1 ❓ | `OpenSSL ED448 algorithm` `0x13b2d58`, `OpenSSL X448 algorithm` `0x13b2d8c`, `C:\Program Files (x86)\OpenSSL\lib\engines-1_1` `0x13b3da8`, `openssl.cnf` `0x13bb364`, `OpenSSL NIST SP 800-90A DRBG` `0x13bb2f8` | `0x633d30-0x6f0f70` (726 functions refer to OpenSSL strings) | 2,649 (`lib-openssl`) |
| zlib (inflate) | 1.2.3 | ` inflate 1.2.3 Copyright 1995-2005 Mark Adler ` `0x139b828` | ❓ | — |
| zlib (inflate) + minizip unzip | 1.1.3 + unzip 0.15 | ` inflate 1.1.3 Copyright 1995-1998 Mark Adler ` `0x13f3054`, ` - unzip 0.15 Copyright 1998 Gilles Vollant ` `0x13f3677` | ❓ | — |
| zlib (deflate) | 1.2.8 | ` deflate 1.2.8 Copyright 1995-2013 Jean-loup Gailly and Mark Adler ` `0x13f7568` | named zlib functions `0x6092b0-0x633440` | 18 (`lib-zlib`) |
| OpenSSL zlib BIO | — | `bio_zlib_read` `0x13cf70c`, `zlib inflate error` `0x13cf6c4` | in OpenSSL range | — |
| JsonCpp | 1.x (class `Json::OurReader` exists) ❓ | class names `Json::Value`, `Json::Reader`, `Json::OurReader` | `0x707a40-0x711680` (see `classes.tsv`) | 154 |
| MSVC STL (`std::`) | VC 8.0 | 2,251 functions in class `std` | spread over `.text` | 6,137 (`lib-stl`, includes template instances) |
| BigInteger | ❓ (own or third party) | class `BigInteger` | see `classes.tsv` | 45 |
| C runtime (static parts) | VC 8.0 | FLIRT names (§1.3) | `0x6f1c30-0x6f23e4`, `0x1491f68` | 199 (`lib-crt`) |

Not in the exe: Scaleform, PhysX, Bink, Lua and Unreal code. No version strings for them exist in the exe. Bink is used through the engine: UI widget names such as `CreateCharacterInfomationHolder_Bink` `0x152ac78` and `lobby-play-bink` `0x17ffd5c`. See [15-engine](15-engine.md).

### 1.7 `.text` layout (1 MB blocks)

Counts are functions per block. The top 3 area tags come from `functions-*.tsv` (§4).

| Block | Functions | Top area tags |
| --- | ---: | --- |
| `0x400000` | 6,745 | `lib-stl` 3,887, unnamed 1,168, `09-items` 415 |
| `0x500000` | 5,234 | `lib-stl` 2,111, unnamed 1,916, `other` 255 |
| `0x600000` | 3,751 | `lib-openssl` 2,649, `other` 372, unnamed 186 |
| `0x700000` | 3,312 | `other` 1,122, `14-ui` 611, unnamed 521 |
| `0x800000` | 2,365 | `14-ui` 599, unnamed 454, `11-social` 299 |
| `0x900000` | 2,459 | `07-combat` 795, unnamed 505, `14-ui` 374 |
| `0xa00000` | 2,976 | `10-quests-npc` 836, unnamed 541, `08-creature` 536 |
| `0xb00000` | 2,474 | `08-creature` 492, `14-ui` 450, `11-social` 349 |
| `0xc00000` | 2,280 | `14-ui` 421, unnamed 416, `09-items` 363 |
| `0xd00000` | 1,603 | `09-items` 1,161, unnamed 210, `07-combat` 71 |
| `0xe00000` | 2,288 | `11-social` 747, `09-items` 679, unnamed 317 |
| `0xf00000` | 2,704 | `09-items` 584, unnamed 558, `11-social` 500 |
| `0x1000000` | 2,406 | `07-combat` 436, `09-items` 391, `14-ui` 338 |
| `0x1100000` | 2,820 | `11-social` 750, unnamed 492, `09-items` 318 |
| `0x1200000` | 1,710 | `10-quests-npc` 524, `12-infogate-arena` 238, `04-lobbygate` 196 |
| `0x1300000` | 249 | unnamed 228 |
| `0x1400000` | 16 | unnamed 15 (`__alloca_probe` `0x1491f68` and thunks) |

The game code is not sorted by feature. The linker order follows the object files. Use the TSV files, not the address, to find a feature.

---

## 2. Class inventory: `classes.tsv`

One row per class (or namespace) that has at least one named function. 3,650 rows. The script in §5 writes it.

| Column | Meaning |
| --- | --- |
| `class` | demangled scope of the function name (`A::B` for nested). `(global)` = functions with no scope. `std` = STL. |
| `functions` | count of named functions with this scope |
| `vtable` | the last `mov [reg], offset X` (or `mov [reg+0], offset X`) in the first constructor (`??0`) where `X` is in `.rdata` and the first two dwords point into `.text`. `-` = no constructor or no store. This is a heuristic ❓ (see the limits below). |
| `vt_slots` | dwords from `vtable` that point into `.text`, up to the next address that has an xref |
| `vt_named_ratio` | part of the slots that point to the start of a named function |
| `ctor` | first constructor address |
| `first`, `last` | lowest and highest address of a named function of the class |
| `area` | most common area tag of the class functions (§4.3) |

Results:

| Item | Value |
| --- | --- |
| Classes | 3,650 |
| Classes with a vtable found | 801 |
| Mean `vt_named_ratio` | 0.78 |
| Classes per area | `lib-stl` 914, `09-items` 468, `14-ui` 367, `other` 361, `07-combat` 309, `10-quests-npc` 289, `11-social` 264, `13-datafile` 133, `08-creature` 109, `12-infogate-arena` 103, `05-game-dispatch` 74, `04-lobbygate` 58, `06-movement` 57, `16-strings-errors` 43, `15-engine` 42, `02-transport` 25, `03-sts` 23, `01-startup` 11 |

Largest game classes: `UiStateGame` 520 (vtable `0x14d08c8`), `BQuestJournal` 393, `BUiWorld` 326 (`0x1574250`), `ItemGrowth2Panel` 290 (`0x15bf7e8`), `MinimapPanel` 281 (`0x1675230`), `World` 280, `BPTCreature` 264, `NewInventoryPanel2` 208, `Inventory` 172, `Guild` 170, `WardrobePanel` 163, `BPTPlayer` 155 (`0x16187a0`, 194 slots), `BPlayer` 143.

Limits of the vtable column:

- Slot 0 is often not a function in IDA. Example: `BPTPlayer` slot 0 = `0x4f0280` and `GameSession` slot 0 = `0x47d7b0` are `loc_` labels. They are scalar deleting destructors that IDA did not make into functions (`MoveListMaker` slot 0 `0x4d6c20` starts with `test byte ptr [esp+4], 1`). So the ratio is low for small vtables.
- Identical code folding (the linker merges equal functions) puts one name on many slots. Example: `LobbyGateSession` vtable `0x15e9a80` slot 1 has the name `G_AddMasteryStatPointFailPacket::PacketSync`. The name of a folded function is one of many correct names.
- Many UI classes show 19 slots. This can be the base `UIPanel` part or a stop at a shared xref ❓.
- `MoveListMaker` (`0x15e98dc`, 2 slots, ratio 0.00) is a wrong result ❓. Check the constructor `0xc61730` by hand.
- The 629 IDA names `??_7…@@6B@` (from `datamatch.py`) are not used. Some are in `.text` (for example `BSkillBook` at `0x408ac0`), so they are false.

---

## 3. Method: names for a binary from a PDB of another build

The old notes ([`client-exe-reversing.md` §1](../client-exe-reversing.md)) give the tool chain. This section makes it a recipe for StServers and GameDaemon binaries. The tools `match.py`, `match2.py`, `datamatch.py` and `mkpdb.py` are in the operator scratch area, not in this repo ❓.

```
 reference build + PDB ──► feature dump A ─┐
                                            ├─► match.py ─► match2.py ─► datamatch.py ─► mkpdb.py ─► target.pdb
 target build (no PDB) ──► feature dump B ─┘                                                         │
                                                                                                      ▼
                                                     IDA: load target.pdb ─► §5 script ─► functions-*.tsv, classes.tsv
```

### 3.1 Recipe

| Step | Action | Output | Check |
| --- | --- | --- | --- |
| 1 | Find a reference build of the same program with a PDB. Prefer the nearest version (381 used 419 GM, about one year later). | reference `.exe` + `.pdb` | same compiler (MSVC 8.0 here, §1.3) |
| 2 | Open both in IDA. Run full auto-analysis. Load the PDB in the reference only. | two `.i64` | function counts (381: 45,392; 419 GM: 63,457) |
| 3 | Dump features per function from both: bytes with relocations masked, strings, constants, CFG shape (blocks, edges), call list, vtable slot place. | two JSON dumps | — |
| 4 | `match.py`: match in this order: vtables (same slot layout), exact bytes, unique strings, CFG plus constants, call graph from matched neighbours. | first match set with a `how` tag per match | each `how` has a count |
| 5 | `match2.py`: use matched functions as ordered anchors. Align the gaps between two anchors (linker order is stable). Use a fuzzy call graph for the rest. | larger match set (`fuzzy`, `vtslot`) | spot-check 20 `fuzzy` matches by hand |
| 6 | `datamatch.py`: name globals and vtables through the data refs of matched code. | global names | reject names that fall in `.text` (§2 limits) |
| 7 | `mkpdb.py`: write a v3 PDB with the mangled names for the target. | `target.pdb` | IDA loads it; names demangle |
| 8 | Run the §5 scripts: a name dump of the reference, then the inventory script on the target. | `functions-*.tsv`, `classes.tsv` | confidence counts (§4.2) |
| 9 | Mark feature names with no string in the target as ❓ (old notes §6.3: three false vehicle matches). | — | — |

### 3.2 Confidence rule used here

The `how` tag of step 4 and 5 is not stored in the `.i64`. So the §5 script makes its own confidence. It compares the function size in the target with the size of the function with the same name in the reference:

| Confidence | Rule |
| --- | --- |
| `high` | name found in the reference and size ratio 0.80 to 1.25, or an IDA FLIRT name |
| `medium` | size ratio 0.50 to 2.00 |
| `low` | other ratio, or the name is not in the reference name list |

Signs of a false match: the method does not fit the class (for example `MoveListMaker::ReqAcquireQuest` `0xc99a70` and `MoveListMaker::getPortalDesc` `0xc99130` ❓), a large size difference (for example `BUiWorld::isAttractionType` `0x74ff90`: 37 bytes in 381, 285 in 419 GM), or no string anchor for a feature that has strings in the reference.

### 3.3 Notes for server binaries

- StServers and GameDaemon also use MSVC. Check the runtime DLL names first (§1.3 method).
- Server binaries have more log strings. Step 4 (unique strings) gives more matches there.
- Use the same area tags (§4.3) but replace the client chapter list with the server spec list in `Docs/Server/`.

---

## 4. Global function index: `functions-1.tsv` … `functions-5.tsv`

One row per function in 381, in address order. 10,000 rows per file (file 5 has 5,392). The script in §5 writes them.

| File | First address | Rows |
| --- | --- | ---: |
| `functions-1.tsv` | `0x401000` | 10,000 |
| `functions-2.tsv` | `0x57ef40` | 10,000 |
| `functions-3.tsv` | `0x8660c0` | 10,000 |
| `functions-4.tsv` | `0xc59710` | 10,000 |
| `functions-5.tsv` | `0x10add30` | 5,392 |

### 4.1 Columns

| Column | Meaning |
| --- | --- |
| `address` | function start, `0x` + hex |
| `size` | bytes (end − start) |
| `name` | IDA name (MSVC mangled for PDB names; `sub_…` for no name) |
| `class` | demangled scope (empty for global or unnamed) |
| `method` | demangled function name without scope |
| `source` | `pdb419` = name from the 419 GM PDB and found in the GM name list; `pdb419-nogm` = a name that is not in the GM name list (IDA thunks `j_…`, compiler names `??_…`, IDA auto names such as `pDtor`); `ida-flirt` = FLIRT library name; `none` = no name |
| `confidence` | `high`, `medium`, `low` (§3.2), `-` for `none` |
| `area` | chapter tag (§4.3) |
| `anchor` | the most unique string that the function refers to (fewest referring functions, then the longest), first 60 characters. UTF-16 and ASCII. Empty if the function refers to no string. |

### 4.2 Counts

| Source | Confidence | Functions |
| --- | --- | ---: |
| `pdb419` | `high` | 32,110 |
| `pdb419` | `medium` | 3,255 |
| `pdb419` | `low` | 57 |
| `pdb419-nogm` | `low` | 1,543 |
| `ida-flirt` | `high` | 6 |
| `none` | `-` | 8,421 |
| total | | 45,392 |

Functions with a string anchor: 7,735. Most game strings are text keys (for example `UI.ExitGame.Kick.XignCode.Title` `0x1633c38`) or datafile names; many are used through tables, so a function does not always refer to them directly.

The old notes give 37,543 matched functions (83.8 %). This database has 36,965 names with source `pdb419` or `pdb419-nogm` and 8,421 `sub_` names. The difference is from functions that IDA made later and from merged names ❓.

### 4.3 Area tags

The tag is the chapter number of `Docs/Client/RE/`. The script tests the class name (or the method name of a global function) against these rules. The first match wins. So a `GuildPanel` gets `11-social`, not `14-ui`.

| Order | Tag | Pattern (regular expression on the class) | Functions |
| ---: | --- | --- | ---: |
| 0 | `lib-stl` | class starts with `std` | 6,137 |
| 0 | `lib-openssl` | OpenSSL C name prefix, or address in `0x633d30-0x6f0f80` | 2,649 |
| 0 | `lib-zlib` | zlib function name | 18 |
| 0 | `lib-crt` | FLIRT, or a global `_lowercase` name | 199 |
| 1 | `03-sts` | `Sts`, `^Auth`, `Login` | 87 |
| 2 | `04-lobbygate` | `Lobby`, `^L_` | 1,008 |
| 3 | `12-infogate-arena` | `InfoGate`, `Arena`, `BattleField`, `BattleRoyal`, `Ranking`, `Tournament`, `^I_` | 950 |
| 4 | `05-game-dispatch` | `GameSession`, `GamePacket`, `PacketProcessor`, `^G_`, `Packet` | 180 |
| 5 | `02-transport` | `Session`, `Socket`, `Network`, `Connection`, `Stream`, `Parser`, `Iocp`, `Tcp`, `Udp`, `Crypt`, `Cipher`, `Net[A-Z]` | 162 |
| 6 | `06-movement` | `Move`, `Glide`, `Jump`, `Path`, `Navi`, `Mount`, `Vehicle`, `Grab`, `Swim`, `Fly`, `Teleport`, `Warp` | 246 |
| 7 | `07-combat` | `Skill`, `Combat`, `Effect`, `Buff`, `Damage`, `Attack`, `Duel`, `Pvp`, `Target`, `Stance`, `Combo`, `Cast` | 2,473 |
| 8 | `09-items` | `Item`, `Inventory`, `Equip`, `Shop`, `Store`, `Market`, `Auction`, `Wardrobe`, `Gem`, `Badge`, `Costume`, `Craft`, `Recipe`, `Money`, `Goods`, `Coupon`, `Warehouse`, `Accessory`, `Weapon`, `Delivery` | 5,567 |
| 9 | `10-quests-npc` | `Quest`, `Npc`, `Achievement`, `Mission`, `Cinematic`, `Dungeon`, `Zone`, `Contents`, `Attendance`, `Reward`, `Talk` | 3,086 |
| 10 | `11-social` | `Party`, `Guild`, `Chat`, `Friend`, `Mail`, `Faction`, `Clan`, `Whisper`, `Team`, `Mentor`, `Social`, `Emotion`, `Raid`, `Community` | 3,612 |
| 11 | `08-creature` | `Creature`, `Player`, `^B?PT`, `Pc[A-Z]`, `Character`, `Appearance`, `Summon`, `Pet`, `Brain`, `Job`, `Actor`, `Avatar` | 1,602 |
| 12 | `13-datafile` | `^Data$`, `^Data::`, `Record`, `Table`, `DrEl`, `Reader`, `Datafile`, `DataFile`, `Xml`, `XML` | 416 |
| 13 | `14-ui` | `Panel`, `^UI`, `^Ui`, `Widget`, `Button`, `Window`, `Tooltip`, `Slot`, `Icon`, `ComboBox`, `List`, `Dialog`, `Scroll`, `Tab`, `Menu`, `Hud`, `HUD`, `Notifier`, `Notification`, `Edit`, `Font` | 4,284 |
| 14 | `01-startup` | `Config`, `Option`, `Startup`, `Launcher`, `CommandLine`, `Version`, `Setting`, `BNSClient`, `Application`, `Main` | 193 |
| 15 | `16-strings-errors` | `Log`, `Error`, `Msg`, `Message`, `String`, `Exception`, `Assert`, `Crash`, `Report` | 247 |
| 16 | `15-engine` | `^E[A-Z]`, `Engine`, `Render`, `Mesh`, `Anim`, `Camera`, `Sound`, `Audio`, `Particle`, `Material`, `Texture`, `Scene`, `Light`, `Physics`, `Collision`, `Thread`, `Lock`, `Timer`, `Memory`, `Alloc`, `Pool`, `Handle`, `Interface`, `Vector`, `Matrix`, `Rotator`, `Quat`, `Math`, `Event`, `Signal`, `Task` | 182 |
| 17 | `other` | named, no rule matches | 3,754 |
| — | `-` | unnamed (`sub_`) and not in a library range | 8,340 |

The tag is a first guess from the name. It is not verified per function ❓. Known weak points:

- `other` holds large world classes: `BUiWorld` 326, `World` 280, `BContextProcess` 117, `BControlledPlayingActions` 116, `PresentationWorld` 90, `BEnvObject` 78, `Production2` 75, `QQMessenger` 52, `BoardGachaManager` 51, `BAutoModeSystem` 49, `BigInteger` 45, `Vip` 42, `Json::Value` 38.
- `PacketParser` gets `05-game-dispatch` (rule 4 `Packet` is before rule 5 `Parser`). Its chapter is [02-transport](02-transport.md).
- Methods with a false class name (§3.2) get the tag of the false class.

---

## 5. Scripts

Run both in IDA 9 (idalib or GUI) with IDAPython. Run them from the repo root, because the output path is relative.

### 5.1 Name dump of the reference (419 GM `.i64`)

```python
import idautils, ida_funcs, idc, json
d = {}
for f in idautils.Functions():
    nm = idc.get_func_name(f)
    if not nm.startswith("sub_"):
        d.setdefault(nm, [f, ida_funcs.get_func(f).end_ea - f])
json.dump(d, open("/tmp/cx00_gm_names.json", "w"))   # 62,342 names for 419 GM
```

### 5.2 Inventory script (381 `.i64`)

It writes `functions-N.tsv` and `classes.tsv`. Run time on 381: 15 seconds.

```python
# cx00_inventory.py - run in IDA 9 (idalib or GUI) on the 381 Client.exe.i64.
# Input : GM name dump (JSON name -> [ea, size]) made from the 419 GM .i64.
# Output: functions-N.tsv (<= 10000 rows each) and classes.tsv in OUT.
import idautils, ida_funcs, ida_name, ida_bytes, ida_segment, idc, json, re, collections, time
OUT = "Docs/Client/RE/"; GMJSON = "/tmp/cx00_gm_names.json"
GM = json.load(open(GMJSON))
seg = {ida_segment.get_segm_name(ida_segment.getseg(s)): (s, ida_segment.getseg(s).end_ea) for s in idautils.Segments()}
TX = seg[".text"]; RD = seg[".rdata"]; DT = (RD[0], seg[".data"][1])
def inr(x, r): return r[0] <= x < r[1]

def qual(n):                      # mangled name -> "A::B::m" (short form)
    d = ida_name.demangle_name(n, ida_name.MNG_SHORT_FORM) if n.startswith("?") else None
    if not d: return None
    d = re.sub(r"^(public|private|protected): ", "", d); d = re.sub(r"^(static |virtual )*", "", d)
    p = d.find("("); head = d[:p] if p > 0 else d
    depth = cut = 0
    for i, c in enumerate(head):
        depth += (c == "<") - (c == ">")
        if c == " " and depth == 0: cut = i + 1
    return head[cut:]
def split_scope(q):              # last top-level "::"
    depth = 0; last = -1; i = 0
    while i < len(q):
        c = q[i]; depth += (c == "<") - (c == ">")
        if depth == 0 and q[i:i+2] == "::": last = i; i += 1
        i += 1
    return (q[:last], q[last+2:]) if last >= 0 else ("", q)

SCACHE = {}
def lit(r):                       # own string decoder: UTF-16LE or ASCII, >= 4 chars
    if r in SCACHE: return SCACHE[r]
    b = ida_bytes.get_bytes(r, 240) or b""; s = None
    w = []
    for i in range(0, len(b) - 1, 2):
        c = b[i] | (b[i+1] << 8)
        if c == 0: break
        if not (32 <= c < 127): w = None; break
        w.append(chr(c))
    if w and len(w) >= 4: s = "".join(w)
    else:
        a = []
        for c in b:
            if c == 0: break
            if not (32 <= c < 127): a = None; break
            a.append(chr(c))
        if a and len(a) >= 4: s = "".join(a)
    SCACHE[r] = s; return s

LIBRE = re.compile(r"^_*(EVP|BN|RSA|SSL|X509|ASN1|OPENSSL|CRYPTO|EC|ECDSA|BIO|ERR|PEM|DH|DSA|OBJ|RAND|ENGINE|CONF|OSSL|PKCS7|PKCS12|HMAC|CMAC|SHA\d*|MD5|AES|DES|i2d|d2i|UI|OCSP|CMS|TS|CT|SRP|COMP|ossl|tls1?|ssl3|dtls1?)_")
ZRE = re.compile(r"^_*(inflate|deflate|adler32|crc32|unz|zip|_tr_|compress|uncompress|gz)")
AREAS = [  # first match wins; tested on the class, then on the method for global functions
 ("03-sts", r"Sts|^Auth|Login"),
 ("04-lobbygate", r"Lobby|^L_"),
 ("12-infogate-arena", r"InfoGate|Arena|BattleField|BattleRoyal|Ranking|Tournament|^I_"),
 ("05-game-dispatch", r"GameSession|GamePacket|PacketProcessor|^G_|Packet"),
 ("02-transport", r"Session|Socket|Network|Connection|Stream|Parser|Iocp|Tcp|Udp|Crypt|Cipher|Net[A-Z]"),
 ("06-movement", r"Move|Glide|Jump|Path|Navi|Mount|Vehicle|Grab|Swim|Fly|Teleport|Warp"),
 ("07-combat", r"Skill|Combat|Effect|Buff|Damage|Attack|Duel|Pvp|PvP|Target|Stance|Combo|Cast"),
 ("09-items", r"Item|Inventory|Equip|Shop|Store|Market|Auction|Wardrobe|Gem|Badge|Costume|Craft|Recipe|Money|Goods|Coupon|Warehouse|Accessory|Weapon|Delivery"),
 ("10-quests-npc", r"Quest|Npc|Achievement|Mission|Cinematic|Dungeon|Zone|Contents|Attendance|Reward|Talk"),
 ("11-social", r"Party|Guild|Chat|Friend|Mail|Faction|Clan|Whisper|Team|Mentor|Social|Emotion|Raid|Community"),
 ("08-creature", r"Creature|Player|^B?PT|Pc[A-Z]|Character|Appearance|Summon|Pet|Brain|Job|Actor|Avatar"),
 ("13-datafile", r"^Data$|^Data::|Record|Table|DrEl|Reader|Datafile|DataFile|Xml|XML"),
 ("14-ui", r"Panel|^UI|^Ui|Widget|Button|Window|Tooltip|Slot|Icon|ComboBox|List|Dialog|Scroll|Tab|Menu|Hud|HUD|Notifier|Notification|Edit|Font"),
 ("01-startup", r"Config|Option|Startup|Launcher|CommandLine|Version|Setting|BNSClient|Application|Main"),
 ("16-strings-errors", r"Log|Error|Msg|Message|String|Exception|Assert|Crash|Report"),
 ("15-engine", r"^E[A-Z]|Engine|Render|Mesh|Anim|Camera|Sound|Audio|Particle|Material|Texture|Scene|Light|Physics|Collision|Thread|Lock|Timer|Memory|Alloc|Pool|Handle|Interface|Vector|Matrix|Rotator|Quat|Math|Event|Signal|Task"),
]
AREAS = [(t, re.compile(p)) for t, p in AREAS]
def area(ea, name, cls, meth, flags):
    if cls.startswith("std") or cls.startswith("stdext"): return "lib-stl"
    if LIBRE.match(name) or inr(ea, (0x633d30, 0x6f0f80)): return "lib-openssl"
    if ZRE.match(name): return "lib-zlib"
    if flags & ida_funcs.FUNC_LIB: return "lib-crt"
    key = cls or (meth if name.startswith("?") else "")
    if not key: return "lib-crt" if re.match(r"^_+[a-z]", name) else "-"
    for t, rx in AREAS:
        if rx.search(key): return t
    return "other"

def conf(n, sz):
    g = GM.get(n)
    if not g: return "low"
    r = sz / float(g[1] or 1)
    return "high" if 0.8 <= r <= 1.25 else ("medium" if 0.5 <= r <= 2 else "low")

t0 = time.time()
FUN = []; F2S = {}; SREF = collections.Counter()
for f in idautils.Functions():
    fn = ida_funcs.get_func(f); strs = set()
    for h in idautils.FuncItems(f):
        for r in idautils.DataRefsFrom(h):
            if inr(r, DT):
                s = lit(r)
                if s: strs.add(r)
    for r in strs: SREF[r] += 1
    F2S[f] = strs
    FUN.append((f, fn.end_ea - f, idc.get_func_name(f), fn.flags))
def anchor(f):
    c = F2S.get(f)
    if not c: return ""
    b = min(c, key=lambda e: (SREF[e], -min(len(lit(e)), 60)))
    return lit(b)[:60].replace("\t", " ")

ROWS = []; CLS = collections.defaultdict(list)
for ea, sz, n, fl in FUN:
    q = qual(n) if n.startswith("?") else None
    cls, meth = split_scope(q) if q else ("", "")
    if n.startswith("sub_"): src, cf = "none", "-"
    elif fl & ida_funcs.FUNC_LIB: src, cf = "ida-flirt", "high"
    else: src, cf = ("pdb419" if n in GM else "pdb419-nogm"), conf(n, sz)
    ar = area(ea, n, cls, meth, fl)
    ROWS.append((ea, sz, n, cls, meth, src, cf, ar, anchor(ea)))
    if q: CLS[cls or "(global)"].append((ea, n, ar))

hdr = "address\tsize\tname\tclass\tmethod\tsource\tconfidence\tarea\tanchor\n"
for k in range(0, len(ROWS), 10000):
    with open(OUT + "functions-%d.tsv" % (k // 10000 + 1), "w") as fo:
        fo.write(hdr)
        for r in ROWS[k:k+10000]:
            fo.write("0x%x\t%d\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n" % r)

def vt_ok(x): return inr(x, RD) and inr(ida_bytes.get_dword(x), TX) and inr(ida_bytes.get_dword(x + 4), TX)
def ctor_vt(fea):                 # last "mov [reg], offset vtable" in a constructor
    last = None
    for h in idautils.FuncItems(fea):
        if idc.print_insn_mnem(h) != "mov" or idc.get_operand_type(h, 1) != 5: continue
        t = idc.get_operand_type(h, 0)
        if t == 3 or (t == 4 and idc.get_operand_value(h, 0) == 0):
            v = idc.get_operand_value(h, 1)
            if vt_ok(v): last = v
    return last
def vt_slots(v):
    i = 0
    while i < 2000 and inr(ida_bytes.get_dword(v + 4*i), TX) and (i == 0 or not ida_bytes.has_xref(ida_bytes.get_flags(v + 4*i))): i += 1
    return [ida_bytes.get_dword(v + 4*j) for j in range(i)]
named = lambda p: ida_funcs.get_func(p) is not None and ida_funcs.get_func(p).start_ea == p and not idc.get_func_name(p).startswith("sub_")
with open(OUT + "classes.tsv", "w") as fo:
    fo.write("class\tfunctions\tvtable\tvt_slots\tvt_named_ratio\tctor\tfirst\tlast\tarea\n")
    for c in sorted(CLS, key=lambda c: (-len(CLS[c]), c)):
        L = CLS[c]; ct = [ea for ea, n, a in L if n.startswith("??0")]
        v = None
        for x in ct:
            v = ctor_vt(x)
            if v: break
        sl = vt_slots(v) if v else []
        ratio = "%.2f" % (sum(1 for p in sl if named(p)) / float(len(sl))) if sl else "-"
        ar = collections.Counter(a for _, _, a in L).most_common(1)[0][0]
        fo.write("%s\t%d\t%s\t%s\t%s\t%s\t0x%x\t0x%x\t%s\n" % (c, len(L), "0x%x" % v if v else "-", len(sl) if sl else "-",
                 ratio, "0x%x" % ct[0] if ct else "-", min(e for e, _, _ in L), max(e for e, _, _ in L), ar))
print("rows", len(ROWS), "classes", len(CLS), "secs", int(time.time() - t0))
```

---

## 6. Open items ❓

| # | Item | Address | Next step |
| ---: | --- | --- | --- |
| 1 | Second `.idata` section: owner and use | `0x1b98000-0x1b9f000` | read only as far as needed for the import list; no protection details |
| 2 | Exact OpenSSL version letter | strings `0x13b2d58`…`0x13bb458` | find the version text or a known 1.1.1 letter change |
| 3 | Code ranges of the three zlib copies | strings `0x139b828`, `0x13f3054`, `0x13f7568` | find the inflate table refs |
| 4 | JsonCpp version | `Json::OurReader` near `0x70b7f0` | compare with JsonCpp releases |
| 5 | MSVC service pack | PE Rich header | read the header from the file |
| 6 | Wrong vtable for `MoveListMaker` and the 19-slot UI cluster | `0xc61730`, `0x15e98dc` | check constructors by hand; improve the slot end rule |
| 7 | Scalar deleting destructors that are not functions | for example `0x4f0280`, `0x47d7b0`, `0x4d6c20` | make functions from all vtable slot targets, then run §5 again |
| 8 | Area tags for `other` and unnamed functions | §4.3 | tag by caller and callee majority, or by the string anchor |
| 9 | Match `how` tags (`vtable`, `bytes`, `strings`, `fuzzy`, `vtslot`) | `c20_named.json` (scratch) | add them as a column when the file is in reach |
| 10 | 37,543 (old notes) against 36,965 names now | — | compare the two name lists |
