# Client.exe 381 — Chapter 01: Process start, command line, config, main loop

Summary:

1. `WinMain` (0x1190f60) sets the locale globals to 0, reads `Local.ini`, parses the command line, loads `SystemText.ini`, then loads `bsengine_Shipping.dll`.
2. `BNSClient::MainFramework` (0x11904c0) checks the locale, makes `BNSClient::Initialize` (0x116f550), starts the engine with `..\contents\`, checks engine version `812`, and loads the XML data.
3. The engine DLL owns the frame loop. The client gets `BNSClient::Tick` (0x1167630) and `BNSClient::PostTick` (0x11902b0) calls each frame.
4. Start errors show a `ClientSystemText` message with a code (2000, 2001, 2002, 10005, 10006, 10007) and stop the start.
5. At exit `WinMain` calls `BNSClient::Cleanup` (0x116dc90), `EInterfaceCleanup`, `UnloadEngine` and `CoUninitialize`.

```
WinMain 0x1190f60
 ├─ SetWorkingDirectory 0x1129670        (cwd = folder of Client.exe)
 ├─ LoadLocaleInfo 0x11297c0             (Local.ini [Locale])
 ├─ SaveLocaleInfo 0x11313b0             (Local.ini, some publishers only)
 ├─ DoCommandLine 0x113ed60 ──► return 1 (--version) ─► exit
 ├─ LanguageLocale::Create, LoadSystemText 0x114cbe0 (..\bin\SystemText.ini)
 ├─ /NPGE set? ─► GameGuard-only loop (Sleep 10000 ms) ─► exit
 ├─ LoadEngine 0x114c6d0                 (bsengine_Shipping.dll, 15 exports)
 ├─ MainFramework 0x11904c0              (see §3)
 │    └─ EInterfaceInitialize(..\contents\) ─► engine frame loop ─► BNSClient::Tick
 └─ Cleanup 0x116dc90 ─► EInterfaceCleanup ─► UnloadEngine ─► CoUninitialize
```

Rules: "381" = `Client.exe` 0.0.381 (x86). "419 GM" = the 2021 GM client with PDB, used for names only.
Names come from the 419 GM PDB match. ❓ marks a fact or a name that we did not verify.
Old notes: [`../client-exe-reversing.md`](../client-exe-reversing.md), [`../client-datafiles.md`](../client-datafiles.md).

---

## Quick find

| Task or question | Function / item | 381 address | GM name | Section |
| --- | --- | --- | --- | --- |
| Process entry (CRT) | `start` | `0x703cfa` | same | §3.1 |
| Where the game starts | `WinMain` | `0x1190f60` | same | §3.1 |
| Where the command line is parsed | `BNSClient::DoCommandLine` | `0x113ed60` | same | §1 |
| Publisher name → enum | `sub_5D0460` (table `0x19498ec`) | `0x5d0460` | ❓ | §1.3 |
| Language name → enum | `sub_5D04D0` (table `0x1949928`) | `0x5d04d0` | ❓ | §1.3 |
| Text language code → enum | `sub_5D0540` (table `0x1949958`) | `0x5d0540` | ❓ | §1.3 |
| Is this publisher + language pair valid | `sub_5D05B0` (table `0x19a4260`) | `0x5d05b0` | ❓ | §1.4 |
| Where `Local.ini` is read | `BNSClient::LoadLocaleInfo` | `0x11297c0` | same | §2.1 |
| Where `Local.ini` is written | `BNSClient::SaveLocaleInfo` | `0x11313b0` | same | §2.1 |
| Where `SystemText.ini` is read | `BNSClient::LoadSystemText` | `0x114cbe0` | same | §2.2 |
| Where the engine DLL is loaded | `LoadEngine` | `0x114c6d0` | same | §3.2 |
| Main start sequence | `BNSClient::MainFramework` | `0x11904c0` | same | §3.3 |
| Client subsystem init | `BNSClient::Initialize` | `0x116f550` | same | §3.4 |
| Per-frame client tick | `BNSClient::Tick` | `0x1167630` | same | §3.5 |
| Shutdown | `BNSClient::Cleanup` | `0x116dc90` | same | §3.6 |
| Start error text (set) | `ClientSystemText::SetAdditionalMsg` | `0x10a0110` | same | §3.7 |
| Start error text (show at exit) | `sub_114CAF0` (false IDA name) | `0x114caf0` | ❓ | §3.6 |
| Error code → GameGuard result (4047) | `ClientGameGuard::GetGameGuardInitializeError` | `0x114b560` | same | §3.7 |
| Engine exports | `LoadEngine` globals | `0x1a4be5c`, `0x1a4c0b8`–`0x1a4c0ec` | same | §3.2 |
| `BNSClient` object pointer | global | `0x1a4c0f8` | ❓ | §3.4 |
| Where the server address file loads | `sub_10A9DE0` (`release.config2.xml`) | `0x10a9de0` | ❓ | §2.4 |
| Data folder path | `BNSClient::GetDataDirectoryPath` | `0x1129d30` | same | §2.2 |
| User option file path | `getLocalOptionFilePath` | `0x1130100` | same | §2.1 |

All named functions of this chapter (155 rows): [`01-startup-functions.tsv`](01-startup-functions.tsv).
| Session key from launcher | global `0x1a823d0` | `0x1a823d0` | `BNSClient::_webLoginSessionKey` ❓ | §1.2 |
| Region (lobby address set 1 or 2) | global `Value` | `0x1955c40` | ❓ | §1.2 |
| Publisher / language globals | `publisher`, `language` | `0x1955c38`, `0x1955c3c` | same | §1.2 |

---

## 1. Command line

### 1.1 Parser

`BNSClient::DoCommandLine` (0x113ed60, ends 0x113f9cd):

- Gets the command line with `GetCommandLineW` and splits it with `CommandLineToArgvW` (0x113ed60).
- If there is only one argument (the exe), it does nothing and returns 0.
- It examines `argv[1..n-1]` in order. Each argument is compared with `_wcsnicmp` (prefix, case-insensitive) or `_wcsicmp` (full, case-insensitive).
- Values come after the prefix, with no space: `/SessKey:<value>`. A value cannot be a separate argument.
- An argument can match more than one test: after a publisher, language, region or session test the code continues at the `--version` test (label `LABEL_31`).
- Unknown arguments are ignored.
- At the end it frees `argv` with `LocalFree`.
- Return value: 1 if `--version` or `--version:console` was seen, else 0. `WinMain` stops the process when the value is 1 (0x1190fc4).

Search anchors: the UTF-16 strings in the table below. They sit at `0x14fe260`–`0x14fe3e0`.

### 1.2 Switches

All switch names are case-insensitive. "Prefix" = compare only the first characters.

| Switch | String addr | Test site | Match | Value | Effect | Default |
| --- | --- | --- | --- | --- | --- | --- |
| `--publisher:<name>` | `0x14fe30c` | `0x113ee05` | prefix 12 | publisher name (§1.3) | local publisher = `sub_5D0460(value)` | not set |
| `-publisher:<name>` | `0x14fe328` | `0x113ee2b` | prefix 11 | same | same | not set |
| `--language:<name>` | `0x14fe340` | `0x113ee51` | prefix 11 | language name (§1.3) | local language = `sub_5D04D0(value)` | not set |
| `-lang:<name>` | `0x14fe358` | `0x113ee77` | prefix 6 | language name | local publisher = current `publisher`; local language = `sub_5D04D0(value)` | not set |
| `/TextLang:<code>` | `0x14fe368` | `0x113eea7` | prefix 10 | `ko`, `en`, … (§1.3) | local publisher = current `publisher`; local language = `sub_5D0540(value)` | not set |
| `-region:<n>` | (aR_3) | `0x113eed6` | prefix 8 | decimal | local region = `_wtoi(value)` | 0 |
| `/GameRegion:<name>` | `0x14fe394` | `0x113eefd` | prefix 12 | `eu` or `vietnam` (case-insensitive) | local region = 1 for `eu` or `vietnam`, else 0 | 0 |
| `/SessKey:<key>` | (aS_23) | `0x113efe3` | prefix | string | copies the value to wstring `0x1a823d0` | empty |
| `/AuthnToken:<token>` | (aA_7) | `0x113f0c1` | prefix | string | copies the value to the same wstring `0x1a823d0` | empty |
| `/LoginMode` | `0x14fe3c8` | `0x113f1c6` | prefix 10 | none | `byte_1A4C310` = 1 | 0 |
| `--version` | `0x14fe3e0` | `0x113f1e2` | full | none | return 1 → process exits after parse | — |
| `--version:console` | — | `0x113f1fb` | full | none | calls `executeVersionCommand` (0x752030), return 1 | — |
| `/UserName:<name>` | `0x14fe260` | `0x113f219` | prefix | string | `BNSClient::SetWebLoginUserName` (0x4404b0) → wstring `0x1a823ec` | empty |
| `/Password:<pw>` | `0x14fe278` | `0x113f2f8` | prefix | string | `BNSClient::SetWebLoginTempPassword` (0x440510) → wstring `0x1a822d4` | empty |
| `/GarenaToken:<t>` | `0x14fe290` | `0x113f3d8` | prefix | string | `BNSClient::SetWebLoginGarenaToken` (0x440570) | empty |
| `-lite:<n>` | `0x14fe2ac` | `0x113f4b8` | prefix | decimal | `ArgList` (0x1a4c440) = `_wtoi(value)` | 0 |
| `/UserAge:<n>` | `0x14fe2bc` | `0x113f59a` | prefix | decimal | if value < 18: `byte_1A4C444` = 1 (minor flag) | 0 |
| `/UidHash:<h>` | `0x14fe2d0` | `0x113f685` | prefix | string | copies the value to wstring `0x1a4db24` | empty |
| `/NPGE` | (aN_8) | `0x113f771` | prefix | none | `byte_1A4C311` = 1 (GameGuard-only mode, §3.1) | 0 |
| `/PresenceId:<x>` | `0x14fe2f0` | `0x113f7ab` | prefix | string | `UnregisterGamePresence(value)`; text after the first `:` goes to wstring `0x1a8237c` | empty |

Notes:

- `/sesskey`, `/launchbylauncher` and `/ProxyIP` from the card are **not** in 381. A search of all C and UTF-16 strings found no `launchbylauncher`, `ProxyIP` or `LoginMode` other than `/LoginMode` (string scan, 89,719 strings). The match is case-insensitive, so `/sesskey:` works the same as `/SessKey:`.
- `/SessKey:` and `/AuthnToken:` write the same global (0x113f19c, `mov ecx, offset 0x1a823d0`). The last one on the line wins.
- The server address is **not** a command-line option. It comes from `release.config2.xml` (§2.4).
- Global names from the dynamic initializers (0x1397d00–0x1397e80) ❓ (the initializer list can be shifted by one entry):

| Global | Initializer | Name (419 GM match) | Written by | Read by |
| --- | --- | --- | --- | --- |
| `0x1a823d0` | `0x1397d60` | `BNSClient::_webLoginSessionKey` | `/SessKey:`, `/AuthnToken:` | `NPSession::SendWebLoginTokenKeyData` (0xc080a8), `NPSession::ReceiveTokenSuccess` (0xc24440), `InGameWeb::MakeGoodsStoreChargeUri` (0xe3e243) |
| `0x1a8237c` | `0x1397d80` | `BNSClient::_npUserKey` ❓ | `/PresenceId:` | — |
| `0x1a4db24` | `0x1397dc0` | `BNSClient::_ncTvUidHash` | `/UidHash:` | `MainFramework` 0x1190e06 (window title text `" (" + hash`) |
| `0x1a823ec` | `0x1397e00` | `BNSClient::_webLoginUserName` | `/UserName:` | `NPSession::SendWebLoginTokenKeyData` (0xc07bbb) |
| `0x1a822d4` | `0x1397e20` | `BNSClient::_webLoginTempPassword` | `/Password:` | `NPSession::SendWebLoginTokenKeyData` (0xc07bce) |
| `0x1a82398` | `0x1397e60` | `BNSClient::_webLoginGarenaToken` | `/GarenaToken:` ❓ | — |

Other flag globals:

| Global | Set by | Read by | Meaning |
| --- | --- | --- | --- |
| `byte_1A4C310` | `/LoginMode` | `MainFramework` 0x11907f5 | When 1, the client skips the Tencent client-security start (only for Tencent publishers). |
| `byte_1A4C311` | `/NPGE` | `WinMain` 0x1190ff1, 0x114cb24 | GameGuard-only mode: no engine, no game. |
| `byte_1A4C444` | `/UserAge:` < 18; `MainFramework` 0x11909ff | `UiStateCinema` (0x7ea213), `BPlayer::CheckRequestEnterArena` (0xb960b7), `ClientUserLog` (0x10c78cd) | Minor (age < 18) flag. Also passed to the engine (vtable +304, 0x1190a42 area). |
| `ArgList` `0x1a4c440` | `-lite:` | `World::EnterWorld` (0x74274a), `World::IsDestZoneResource` (0x72af3c), level-download panels | Lite (partial download) client level. ❓ exact value meaning. |
| `Value` `0x1955c40` | `-region:`, `/GameRegion:` | `World::IsUseMyLanguage` (0x722740), `NPSession`, `BNSClient::Initialize` | Region index. > 0: use lobby address set 2 (see [old notes](../client-datafiles.md), `release.config2.xml`). |
| `dword_1955C44` | `DoCommandLine` 0x113f987 | `BNSClient::LoadData` (0x114c059), `BNSClient::Initialize` (0x1170521) | Copied with the locale pair. ❓ meaning. |

### 1.3 Name tables

Each parser walks a table of UTF-16 string pointers. It compares `_wcsnicmp(table[i], value, len(table[i]))`. The result is the index, or -1 (`NONE`) if no entry matches. So the value must **start with** a table name: `koreanXYZ` gives `korean`.

Publisher (`sub_5D0460`, table `0x19498ec`, 15 entries). `PUBLISHER_NONE` = -1.

| Value | Name | Value | Name | Value | Name |
| ---: | --- | ---: | --- | ---: | --- |
| 0 | `ncsoft` | 5 | `nctaiwan` | 10 | `rncjapan` |
| 1 | `tencent` | 6 | `ncwest` | 11 | `rgarena` |
| 2 | `innova` | 7 | `rncs` | 12 | `rnctaiwan` |
| 3 | `ncjapan` | 8 | `rtencent` | 13 | `rncwest` |
| 4 | `garena` | 9 | `rinnova` | 14 | `empty` |

Language (`sub_5D04D0`, table `0x1949928`) and text-language code (`sub_5D0540`, table `0x1949958`), 12 entries each. Same index = same `Language` value. `LANGUAGE_NONE` = -1.

| Value | `--language:` / `-lang:` | `/TextLang:` |
| ---: | --- | --- |
| 0 | `korean` | `ko` |
| 1 | `chineses` | `zh-CN` |
| 2 | `chineset` | `zh-TW` |
| 3 | `japanese` | `ja` |
| 4 | `english` | `en` |
| 5 | `russian` | `ru` |
| 6 | `french` | `fr` |
| 7 | `german` | `de` |
| 8 | `bportuguese` | `pt` |
| 9 | `thai` | `th` |
| 10 | `vietnamese` | `vn` |
| 11 | `empty` | `empty` |

### 1.4 Commit of the locale values

At the end of the parse (0x113f960–0x113f990):

1. The local values start as publisher = `PUBLISHER_NONE`, language = -1, region = 0.
2. `sub_5D05B0` (0x5d05b0) looks for the pair (publisher, language) in the table at `0x19a4260` (32 entries of 16 bytes: `+0` publisher, `+4` language, `+8` 0, `+12` byte flag, all flags 1).
3. If the pair is found: `Value` (region), `publisher`, `language` and `dword_1955C44` get the local values. Then, if the engine client info is loaded, it calls `WebSessionManager::UpdateResourcePathCookie`.
4. If the pair is not found: the globals keep their old values (from `WinMain`: `ncsoft`, `korean`, region 0).

Result: to change the region you must also give a valid publisher and language on the same line. Example: `--publisher:ncwest --language:english -region:1`.

Valid pairs (table `0x19a4260`). Full list in [`01-startup-locale-pairs.tsv`](01-startup-locale-pairs.tsv).

| Publisher | Languages |
| --- | --- |
| `ncsoft` (0), `rncs` (7) | korean, english, chineses, japanese, chineset, russian |
| `tencent` (1), `rtencent` (8) | chineses |
| `innova` (2), `rinnova` (9) | russian |
| `ncjapan` (3), `rncjapan` (10) | japanese |
| `nctaiwan` (5), `rnctaiwan` (12) | chineset |
| `ncwest` (6), `rncwest` (13) | english, french, german, bportuguese |
| `garena` (4), `rgarena` (11) | thai, vietnamese |

`MainFramework` checks the pair again at 0x1190592 (§3.3 step 3, error 10007).

---

## 2. Config files and registry

Base folder: `BNSClient::SetWorkingDirectory` (0x1129670) gets the exe path with `GetModuleFileNameW`, cuts it at the last `\`, copies it to `word_1A4C100` and calls `SetCurrentDirectoryW`. All relative paths below start at the exe folder (`bin\`). `%s%s` paths use `word_1A4C100` as the first part.

### 2.1 File list (read at start)

| File (path from `bin\`) | Reader | 381 address | Section / keys | Default | Effect |
| --- | --- | --- | --- | --- | --- |
| `Local.ini` | `BNSClient::LoadLocaleInfo` | `0x11297c0` | `[Locale]` `Publisher`, `Language` | — | Used only if `publisher` / `language` = -1. `WinMain` sets them to 0 first (0x1190f9c), so in 381 the values in `Local.ini` have no effect ❓ (no other path found). If one is still -1 at the end: `ncsoft` + `korean`. |
| `Local.ini` | `BNSClient::SaveLocaleInfo` | `0x11313b0` | writes `[Locale]` `Publisher`, `Language`, `Universe` (= `Value`, region) | — | Writes only when the publisher is **not** ncsoft, rncs, ncwest, rncwest, garena or rgarena. |
| `Local.ini` | `ClientGameSecurity::InitGameGuard` | `0x10e24c0` (read 0x10e25c9) | `[GameGuard]` `Country` | `KR` | GameGuard country code. Out of scope. |
| `..\bin\SystemText.ini` | `BNSClient::LoadSystemText` → `ClientSystemText::SetFile` (0x10a4df0) → `ClientSystemText::ReadAll` (0x10b7bf0) | `0x114cbe0` | sections read with `GetPrivateProfileSectionW` in `readAll` (0x10b77d0); key form `%s_%d` (for example `ErrID_4047`) | `Client Unknown Error(%d)` | Error and system texts for §3.7. `ReadAll` reads a second section first if one is set, then the default ❓. |
| `..\bin\Version.ini` | `BNSClient::MainFramework` | `0x1190b0f` | `[Version]` `ProductVersion` | empty | Text on the splash screen (`Splash.bmp`). Also read by `BUiWorld::ReInit` (0x7c3650), `BUiLobby::ReInit` (0x818560), screenshot EXIF text. |
| `Splash.bmp` | engine vtable +1740 call | `0x1190b1f` | — | — | Splash image. |
| `extraMode.ini` | `BNSClient::LoadDuelExtraMode` | `0x112a300` | `[Mode]` 17 keys | 0 | Duel observer / telecast modes. Keys in TSV. |
| `battleRoyalExtraMode.ini` | `BNSClient::LoadBattleRoyalExtraMode` | `0x1129ef0` | `[Mode]`, `[ShortCut]` | 0 | Battle royal observer modes. |
| `battleRoyalPlayerMode.ini` | `BNSClient::LoadBattleRoyalPlayerMode` | `0x112a180` | `[Mode]` | 0 | Battle royal player modes. |
| `extraMode.ini` / `battleRoyalPlayerMode.ini` | `SkillTracer::LoadIni` | `0x906b80` | `[SkillTrace]` `Enabled`=0, `TraceFailureByClient`=0, `TraceSuccess`=0, `LatencyThresholdMillisecond`=100, `EncryptLogFile`=0, `MaxMemorySizeMB`=10, `ExcludedClientFailReason%d` | see keys | Skill trace log. Called from `Initialize` 0x1170d68. |
| `XmlReader_cl32.dll` | `BNSClient::Initialize` | `0x116fc89` | exports `GetInterfaceVersion`, `SetMemAllocator`, `CreateXmlReader`, `DestroyXmlReader` (globals `0x1a16b94`, `0x1a16b98`, `0x1a16b9c`, `0x1a16ba0`; handle `0x1a16b90`) | — | Version must be `"15"` (0x116fd43). Else error 1001. |
| `config.dat` | `BNSClient::Initialize` → `PackXmlReaderIO` | `0x116fe5b` | — | — | Config package in the data folder. `release.config2.xml` comes from it (§2.4). |
| `xml.dat` | `BNSClient::Initialize` → `PackXmlReaderIO::Initialize` | `0x1170549` | — | — | Data XML package (held at `BNSClient+0x1fc`). Fail → error 1002. |
| `local.dat` | `BNSClient::Initialize` → `PackXmlReaderIO` | `0x117069e` | — | — | Local (language) XML package (held at `BNSClient+0x204`). |
| `bsengine_Shipping.dll` | `LoadEngine` | `0x114c6d0` | 15 exports (§3.2) | — | Engine. Fail → error 2000. |
| `ClientConfiguration.xml` | `getLocalOptionFilePath` | `0x1130100` | — | — | User options. Path: `<My Documents>\BnS\<sub>\ClientConfiguration.xml` (`SHGetFolderPathW` CSIDL 5). `<sub>` comes from `GReleaseConfig` field `clientConfigrationSubDirectoryName`, or the publisher name when the publisher is not ncsoft / rncs ❓. Read by `LoadOption` (0x115e820). |

All `GetPrivateProfile*` calls in the exe (116 sites, with section, key, default and file): [`01-startup-ini-reads.tsv`](01-startup-ini-reads.tsv). The "file" column lists the `.ini` names that the same function uses.

### 2.2 Data folder

`BNSClient::GetDataDirectoryPath` (0x1129d30): `..\contents\` + `local\` + `LocalizeUnit::PublisherPath(publisher)` + `data\`, when the third argument is true. Else `..\contents\data\`. Example for `ncsoft`: `..\contents\local\NCSoft\data\` (matches [`client-datafiles.md`](../client-datafiles.md), live client notes). `GetWebDataDirectoryPath` (0x1129bf0) uses `web\` in place of `data\`.

### 2.3 Registry

381 does **not** keep game settings in the registry. All registry API calls (`RegOpenKeyExA`, `RegQueryValueExA`, `RegCreateKeyExA`, `RegSetValueExA`, `RegCreateKeyA`) are in GameGuard code (`CGameGuard::GG*LastError` 0x6f35b6–0x6f3636, `RegSetErrlog` 0x6f5a91, `CheckIrregularGamemonDie` 0x6f7f75, `sub_6F46C0`, `sub_6F5150`, `sub_6F5360`) or in `HTTPClient::getContentType` (0xe36a9b, MIME type lookup). GameGuard details are out of scope.

### 2.4 Server address (`release.config2.xml`)

`sub_10A9DE0` (0x10a9de0) loads `release.config2.xml` from the config package. It is called from `BNSClient::Initialize` at 0x116ff1d. The result is `GReleaseConfig` (0x194f654). Field layout and the built-in default address: see [`../client-datafiles.md`](../client-datafiles.md) "Server address". The lobby set (1 or 2) depends on `Value` (region, §1.2). Then `ConfigLoader::LoadRuleConfig` (0x116ff67), `InGameWeb::LoadFromConfig` (0x1170019) and `ContentsControl::InitializeAfterLoadConfig` (0x117001e) run.

---

## 3. Start order and main loop

### 3.1 `WinMain` (0x1190f60)

| Step | Address | Call / action |
| ---: | --- | --- |
| 1 | `0x1190f9c` | `publisher` = 0, `language` = 0, `Value` = 0, `dword_1955C44` = 0 |
| 2 | `0x1190fb5` | `BNSClient::SetWorkingDirectory` (0x1129670) |
| 3 | `0x1190fba` | `BNSClient::LoadLocaleInfo` (0x11297c0) |
| 4 | `0x1190fbf` | `BNSClient::SaveLocaleInfo` (0x11313b0) |
| 5 | `0x1190fc4` | `BNSClient::DoCommandLine` (0x113ed60). Return 1 → go to step 12 without engine. |
| 6 | `0x1190fd1` | `LanguageLocale::Destroy`, `LanguageLocale::Create(language)` |
| 7 | `0x1190fe1` | `BNSClient::LoadSystemText` (0x114cbe0) |
| 8 | `0x1190ff1` | If `/NPGE` (`byte_1A4C311` = 1): get the game-security object (`sub_1131520`, IDA name `InfoGateSession::ReconnectInfoGate` is false ❓). If not NULL, loop: `ClientGameGuard::Tick` (0x115bd30), stop when `byte_1A4C446` = 1, else `Sleep(10000)` (0x2710 ms). Then go to step 12. |
| 9 | `0x119102a` | `LoadEngine` (0x114c6d0). Fail → step 12. |
| 10 | `0x1191045` | `BNSClient::MainFramework(hInstance, nCmdShow)` (0x11904c0). It returns after the engine loop ends ❓ (the engine run call blocks; not traced into the DLL). |
| 11 | `0x1191053` | If `BNSClient+0x54` (account) is set: `sub_128C510(account)` ❓ |
| 12 | `0x119107d` | `ItemGraphUserData::clear`, `BNSClient::Cleanup` (0x116dc90) |
| 13 | `0x1191099` | If engine init was OK (`byte_1A4C0F4`): `EInterfaceCleanup` (0x1a4c0bc) |
| 14 | `0x119109b` | `UnloadEngine` (0x1122f80), release COM object, `CoUninitialize`. Return 0. |

`WinMain` always returns 0. The process exit code is 0 for a normal exit and for a start error ❓ (exit through the CRT).

`byte_1A4C446` is the "exit now" flag. It is set by `NPGameMonCallback` (0x115b88d …), `ClientGameSecurity::AppExitImmediate` (0x10a03b0) and `sub_10D1750` (0x10d17b9).

### 3.2 `LoadEngine` (0x114c6d0)

Loads `bsengine_Shipping.dll` (name at `0x14fe00c` + `.dll` at `0x14fe030`; handle `0x1a4c0f0`). On load error: text `" last error: %u"` and error 2000. Then `GetProcAddress` for each export. A missing export returns false (no message).

| Export | Global (381) | Store site |
| --- | --- | --- |
| `EInterfaceGetInstance` | `0x1a4be5c` | `0x114c852` |
| `EInterfaceInitialize` | `0x1a4c0b8` | `0x114c869` |
| `EInterfaceCleanup` | `0x1a4c0bc` | `0x114c87f` |
| `EInterfaceSetClientInfo` | `0x1a4c0c0` | `0x114c896` |
| `EInterfaceGetClientInfoCount` | `0x1a4c0c4` | `0x114c8ad` |
| `EInterfaceGetClientInfo` | `0x1a4c0c8` | `0x114c8c3` |
| `UIEngineInterfaceGetInstance` | `0x1a4c0cc` | `0x114c8de` |
| `EInterfaceAddEngineObjectRef` | `0x1a4c0d0` | `0x114c8f9` |
| `EInterfaceRemoveEngineObjectRef` | `0x1a4c0d4` | `0x114c913` |
| `EInterfaceGetInterfaceVersion` | `0x1a4c0d8` | `0x114c92e` |
| `EInterfaceGetPlatformBinaryVersion` | `0x1a4c0dc` | `0x114c949` |
| `EInterfaceErrorReport` | `0x1a4c0e0` | `0x114c963` |
| `EInterfaceMalloc` | `0x1a4c0e4` | `0x114c97e` |
| `EInterfaceRealloc` | `0x1a4c0e8` | `0x114c999` |
| `EInterfaceFree` | `0x1a4c0ec` | `0x114c9b3` |

Note: IDA shows the global `0x1a4c0f8` as `EInterfaceSetClientInfo+1`. It is the pointer to the one `BNSClient` object (see §3.4).

### 3.3 `BNSClient::MainFramework` (0x11904c0)

| Step | Address | Action | Fail |
| ---: | --- | --- | --- |
| 1 | `0x1190529` | `publisher` = -1? | error 10005, return 0 |
| 2 | `0x1190566` | `language` = -1? | error 10006 |
| 3 | `0x1190592` | `sub_5D05B0(publisher, language)` pair check (§1.4) | error 10007 |
| 4 | `0x11905d0` | `ContentsControl::InitializeBeforeLoadConfig` (0x10a28d0) | return 0 |
| 5 | `0x11905ed` | `sub_10E21A0`: `ClientExpireCheck::CheckClientRestrict(publisher)` and more | error 3000 / 3002 |
| 6 | `0x11905fe` | `GetClientVersionString`; build `  <game-version>…</game-version>\n`; pass it to `EInterfaceSetClientInfo` (0x119071c) | — |
| 7 | `0x119075e` | `BNSClient::Initialize(hInstance)` (0x116f550, §3.4) | return 0 |
| 8 | `0x1190770` | `sub_10D17E0` | error 3002 |
| 9 | `0x11907b7` | If the client-info flag is set: `InitializeTCLS` (0x114c110, Tencent) | error 6000 |
| 10 | `0x11907f5` | Tencent publishers and not `/LoginMode`: Tencent client security, account, cross | — |
| 11 | `0x1190862` | `EInterfaceInitialize(nullsub_4, L"..\\contents\\", xml.dat IO, local.dat IO, item graphs)` → `byte_1A4C0F4` | error 2001 |
| 12 | `0x11908b7` | `ClientExecutionLog::AddSystemInfo`, `ClientExecutionLog::SendLog` (if flag) | — |
| 13 | `0x11908da` | Engine interface version must be `"812"`. Else text `Engine Version invalid. (Current Version : %s . Loaded Version : %s)` | error 2002 |
| 14 | `0x11909a9` | Engine console command `SHOW SCREENDEBUGMSGS OFF` (vtable +72) | — |
| 15 | `0x11909b0` | `GetVirtualMemoryMB` < 2500 (0x9c4): set the engine low-memory flag (vtable +0x1ac) | — |
| 16 | `0x11909de` | `LoadDuelExtraMode`, `LoadBattleRoyalExtraMode`, `LoadBattleRoyalPlayerMode` | — |
| 17 | `0x11909ed` | If `byte_1A4C328` and `byte_1A4C314`: minor flag `byte_1A4C444` = 1 ❓ | — |
| 18 | `0x1190a3d` | `TaskSession::InitializeLatencyLogModule` (if `byte_1A4C329`) | — |
| 19 | `0x1190a42` | Pass the minor flag to the engine (vtable +304) | — |
| 20 | `0x1190b0f` | Read `..\bin\Version.ini`; show splash `Splash.bmp` with version, publisher, language (UI engine vtable +1740) | — |
| 21 | `0x1190b66` | `LoadXmlData` (0x114c9f0) | error 10001 |
| 22 | `0x1190be9` | `ItemTabularGraph::LoadGraphs`, `SetUpDefaultChatTabTemplet`, `SetUpKeyConfigEnabledByClientConfig`, `SetUpVipGrade`, `LoadOption` (0x115e820) | — |
| 23 | `0x1190c1d`–`0x1190d83` | Engine settings: 0.8, (32, 32), config and `GRuleConfig` values | — |
| 24 | `0x1190e06` | Window title: format text, add `" (" + UidHash + ")"` if `/UidHash:` was set | — |
| 25 | `0x1190edc` | Engine run call (vtable +8) with `hInstance`, `nCmdShow`, **`BNSClient::Tick`, `BNSClient::PostTick`, `BNSClient::ReplayTick`**, publisher name, language name, title. This call holds the frame loop. Return 1 at the end. | — |

### 3.4 `BNSClient::Initialize` (0x116f550)

Hex-Rays fails on this function. Order from the call list:

```
InitializeClientModuleNames 0x1123160  (log tags, table 0x1a4be60)
BNSClient::BNSClient 0x114d230 ─► global 0x1a4c0f8
GlobalMemoryStatusEx; ClientLog::Initialize 0x10dce00; GetClientVersion
NPSession, LobbyGateSession, GameSession (x2), WebProcessManager, TaskSession, BAccount, Timer, Clock
ClientExecutionLog::Startup 0x1131030
XmlReader_cl32.dll (1000 / 1001)
config.dat ─► sub_10A9DE0 release.config2.xml ─► ConfigLoader::LoadRuleConfig ─► sub_5D05B0
InGameWeb::LoadFromConfig; ContentsControl::InitializeAfterLoadConfig
CoInitialize; CoCreateInstance
xml.dat (1002); local.dat
STSMSGLIB_CHECK_DLL_VERSION; ShaLiWrapper; TencentClientSecurity; InnovaClientSecurity; TencentCross
SetUpDefaultTemplet; mouse button commands (LBtnCmd … HybridModeRBtnCmd); LoadOption
TaskSession::Initialize; WebUtil::AddWebFontResource; SkillTracer::LoadIni; SkillTracer
```

`BNSClient` object fields (from the getters; object pointer at `0x1a4c0f8`):

| Offset | Getter | 381 address |
| --- | --- | --- |
| `+0x24` | `GetGameSession`, `IsConnectedGame` | `0x1122f10`, `0x11230b0` |
| `+0x28` | `GetArenaSession`, `IsConnectedArena` | `0x1122f00`, `0x11230a0` |
| `+0x2c` | `IsConnectedNP` (Tick calls `NPSession::Tick` on it) | `0x11230d0` |
| `+0x3c` | `GetWebSessionManager` | `0x41c0f0` |
| `+0x40` | `GetWebProcessManager` | `0x4a38e0` |
| `+0x44` | `GetUi` | `0x401600` |
| `+0x48` | `GetLobby` | `0x401610` |
| `+0x4c` | `GetWorld` | `0x401620` |
| `+0x50` | `GetReplay` | `0x520c50` |
| `+0x54` | `GetAccount` | `0x401630` |
| `+0x58` | `GetPresentationWorld` | `0x401680` |
| `+0x5c` | `GetNPSession` ❓ (conflicts with `+0x2c`) | `0x401640` |
| `+0x60` | `GetLobbyGateSession` | `0x401650` |
| `+0x64` | `GetInfoGateSession` | `0x4404a0` |
| `+0x68` | `GetXmlReader` | `0x401670` |
| `+0x80` | `GetDisconnectWSALastError` | `0x520c60` |
| `+0x1a4` | frame counter (Tick adds 1) | `0x1167630` |
| `+0x1fc` | `GetPackDataXmlReaderIO` (xml.dat) | `0x4405d0` |
| `+0x204` | `GetPackLocalXmlReaderIO` (local.dat) | `0x4405f0` |
| `+0x210` | `GetTencentClientSecurity` | `0x440610` |
| `+0x214` | `GetTencentSafeMode` | `0x440630` |
| `+0x218` | `GetTencentCross` | `0x1122ee0` |
| `+0x21c` | `GetInnovaClientSecurity` | `0x440650` |
| `+0x224` | `GetDuelBotBrain` | `0x4016a0` |
| `+0x228` | `GetWebRequester` | `0x47c690` |
| `+0x234` | `GetClientDebugger` | `0x4a38f0` |
| `+0x238` | `GetSpammerCheck` | `0x45d3d0` |
| `+0x23c` | `GetBnsTv` | `0x41c100` |
| `+0x240` | `GetSkillTracer` | `0x532aa0` |
| `+0x244` | `GetItemGraphs` | `0x440670` |

Log module tags: 143 entries, `[lobbys]`, `[NP]`, `[gameds]`, `[infogate]`, … in [`01-startup-log-modules.tsv`](01-startup-log-modules.tsv). Slot address = `0x1a4be60` + 4 × index.

### 3.5 Frame tick: `BNSClient::Tick(float dt)` (0x1167630)

The engine calls it once per frame (§3.3 step 25). Order:

1. `BNSClient+0x1a4` += 1; `QueryPerformanceCounter`.
2. If running (`byte_1A4C0F5` ≠ 0): read engine log events (engine vtable +3408). Pass each to `+0x44` (Ui), `+0x58`, `+0x4c` and `+0x218`. Event type 55 → client log `[Engine] Critical Log: %s` (source line 6217).
3. If `+0x248` is set: pop `ClientEventQueue` (`ClientEventQueue::Pop`) and send each event to the receivers.
4. `sub_5D5650` ❓; World and InfoGate ticks through vtables.
5. Game security: `ClientGameGuard::Tick` (0x115bd30, checks every interval with `timeGetTime` ❓). If `byte_1A4C446` = 1: log `exit by security. (%s)` (line 6325), `saveLocalOption`, `saveSyncOption`, `Ui::SetOutroScene`, engine exit (vtable +292, +68), `byte_1A4C0F5` = 0.
6. `NPSession::Tick` if `+0x2c` is set and `+0x2c→+472` = 0.
7. `sub_E54130` if `+0x34` is set and `GRuleConfig+16` ≠ 0 ❓.
8. Game session (`+0x24`) and arena session (`+0x28`) ticks: `sub_5D2840` ❓.
9. `DelayedCall::Tick(dt)`; `qword_1A4C448` = `PreciseTimer::GetElapsedTimeMilliSecond`.

`BNSClient::PostTick(float dt)` (0x11902b0), call order: `World::Tick` (0x11902e4), `Ui::PostTick` (0x1190301), `Timer::Tick` (0x119031f), `BAccount::Tick` (0x1190336), `WebSessionManager::Tick` (0x1190355), `WebSessionManager::UpdateResourcePathCookie` / `SetWebLanguageCookie` (0x1190375, 0x119038b), `PresentationWorld::CleanUpDeferedRemoveObjectsList` (0x11903a9), `Ui::ExitGame` (0x11903d8, on a condition ❓), `PreciseTimer::GetElapsedTime`, `FloatingPanelManager::deleteFloatingList` (0x1190462), `QueryPerformanceCounter`. Conditions per call ❓.

`BNSClient::ReplayTick(float dt)` (0x116f530): only `Replay::update(dt)` (0x116f54a).

`LoadXmlData` (0x114c9f0): times the load with `QueryPerformanceCounter` and calls `BNSClient::LoadData` (0x114bf80, at 0x114ca7a). Table order inside `LoadData` ❓ (datafile chapter).

From the tick to the network: `NPSession::Tick` and the session ticks; see the network chapters in `Docs/Client/RE/` when they exist, and [`../client-exe-reversing.md`](../client-exe-reversing.md) §2.

### 3.6 Shutdown: `BNSClient::Cleanup` (0x116dc90)

Order (call list): `SkillTracer::Report("_unknown_")`, release config objects (`sub_10A3FA0` … `sub_10A3EB0`), engine object release, close the three `DataPackFile`s (0x116df72, 0x116dfbd, 0x116e008), `frostFinalize`, security objects, `_RegisterGamePresence`, `FreeLibrary` (XmlReader DLL, 0x116e2dc), `BannedWordFilterLoader::Clear`, `WebUtil::RemoveWebFontResource`, `Log::RemoveMedia` (x2), `ClientUserLog::Cleanup`.

At 0x116e267 it calls `0x114caf0` (IDA name `ChoiceRewardSelectConfirmPanel::~…` is false). That function gets the pending system text. If there is one, it shows `MessageBoxW` with it, then `ClientSystemText::ResetSystemText`. In `/NPGE` mode it does not show the box. So a start error is shown **after** cleanup, just before the process ends.

### 3.7 Start errors

Mechanism: the failing step calls `0x113f9d0` (IDA name `BNSClient::EnterReplay` is false; it gets the `ClientSystemText` object) and then `ClientSystemText::SetAdditionalMsg(code, extra text)` (0x10a0110). The step returns 0, the start stops, and §3.6 shows the text for the code from `SystemText.ini`.

| Code | Function | Site | Cause |
| ---: | --- | --- | --- |
| 1000 | `BNSClient::Initialize` | `0x116fd28` | `LoadLibraryW(XmlReader_cl32.dll)` failed |
| 1001 | `BNSClient::Initialize` | `0x116fdb7`, `0x1170e87` | xml-reader version not `"15"` |
| 1002 | `BNSClient::Initialize` | `0x1170601` | `xml.dat` failed to open |
| 2000 | `LoadEngine` | `0x114c828` | `bsengine_Shipping.dll` failed to load |
| 2001 | `MainFramework` | `0x1190893` | `EInterfaceInitialize` failed |
| 2002 | `MainFramework` | `0x119094e` | engine version not `"812"` |
| 3000 | `sub_10E21A0` | `0x10e221e` | `ClientExpireCheck::CheckClientRestrict` |
| 3002 | `sub_10E21A0`, `sub_10D17E0` | `0x10e2262`, `0x10d184f` | other start checks ❓ |
| 4000–4050 | `ClientGameGuard::GetGameGuardInitializeError` | `0x114b560` | GameGuard init result → code. **4047 is the default for an unknown result** (0x114bb83). See [`../client-4047-crash.md`](../client-4047-crash.md): on the laptop, 4047 came from a crash in `pathengine.dll`, so 4047 can have more than one cause. |
| 4040–4046 | `NPGameMonCallback` | `0x115b876`–`0x115bae0` | GameGuard run-time messages |
| 4048 | `sub_10D1750` | `0x10d17a5` | after `_SetD3DDeviceInfo` ❓ |
| 4049 | `sub_115EA90` | `0x115eae5` | `ClientGameSecurity::InitGameGuard` failed ❓ |
| 5000–5002 | `UiTransitControl::loadZoneMap` | `0x790344`… | zone map load (not start) |
| 6000 | `InitializeTCLS` | `0x114c684` | Tencent TCLS start failed |
| 10001 | `MainFramework` | `0x1190b8e` | `LoadXmlData` failed |
| 10005 | `MainFramework` | `0x119053f` | publisher is NONE |
| 10006 | `MainFramework` | `0x119057c` | language is NONE |
| 10007 | `MainFramework` | `0x11905bf` | publisher + language pair is not valid |

Full list with the GameGuard result → code map (41 rows): [`01-startup-error-codes.tsv`](01-startup-error-codes.tsv).

---

## 4. Worked example

Launcher line: `Client.exe /SessKey:keyhere /LoginMode --publisher:ncwest --language:english -region:1`

1. `DoCommandLine`: `0x1a823d0` = `keyhere`; `byte_1A4C310` = 1; local publisher = 6, language = 4, region = 1.
2. Pair (6, 4) is row 20 of `0x19a4260` → `publisher` = 6 (`ncwest`), `language` = 4, `Value` = 1.
3. `BNSClient::Initialize` → `Universe::SetUniverseInfo` gets lobby set 2 (`Value` > 0), see [`../client-datafiles.md`](../client-datafiles.md).
4. Data folder: `..\contents\local\<PublisherPath(ncwest)>\data\` ❓ (name from `LocalizeUnit::PublisherPath`).
5. At login, `NPSession::SendWebLoginTokenKeyData` (0xc080a8) sends the key from `0x1a823d0`.

---

## 5. Open items ❓

- `PUBLISHER_NONE` path in `LoadLocaleInfo`: no path found that leaves `publisher` = -1 before the read.
- `sub_10D17E0`, `sub_10E21A0` (3002): exact checks.
- `BNSClient::PostTick` (0x11902b0): the condition for each call.
- `-lite:` value meaning (`ArgList` 0x1a4c440).
- `dword_1955C44` meaning.
- `LoadXmlData` (0x114c9f0) table load order; `LoadOption` (0x115e820) XML keys.
- The path from engine loop start to the login screen (`UiStateLogin` / `TitleLoginPanel::loginReq` 0x7ff420) is in the UI chapter.
