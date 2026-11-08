# 00 — LobbyDaemon overview

## Evidence and builds

✅ means seen in build 0.0.307 code or data. ❓ means guide-only evidence or an unchecked claim.
A recovered guide name does not prove full behavioral equivalence.

| Item | Evidence | Source |
| --- | --- | --- |
| Target build 0.0.307 | ✅ Build text in PDB path | 307 0x1401E0068; include-path string 0x14011BD40 |
| Target function count: 3047 | ✅ IDA function enumeration | Target database; first code segment 0x140001000 |
| Guide build 0.0.348 | ❓ Guide build path | 348 0x140120D40 |
| Guide function count: 3087 | ❓ IDA function enumeration | Guide database; first code segment 0x140001000 |
| Windows x64 process | ✅ PE imports, 64-bit pointers and entry code | 307 wmain 0x140096E00; .idata 0x14011B000 |

## Process model

| Function | 348 name and source | 307 source | Result |
| --- | --- | --- | --- |
| Entry | ❓ wmain; guide entry address not checked | ✅ wmain 0x140096E00 | Constructs daemon; calls 0x1400DCE20; destroys daemon |
| Service support | ❓ Detailed branch not checked | ✅ ADVAPI32 imports in .idata 0x14011B000 | StartServiceCtrlDispatcherW and RegisterServiceCtrlHandlerW are imported; ❓ branch and callbacks not mapped |
| Console modes | ❓ Detailed mode switch not checked | ✅ help routine 0x1400DCBC0; text 0x1401D3170, 0x1401D31C0, 0x1401D3218 | Window console, exit command and text console modes are listed |
| Initialize | LobbyDaemon::initialize, 348 0x14009B300 | ✅ 0x140097EB0 | Sets up DB, registries, job pools and NP |
| Network workers | LobbyDaemon::initNetwork, 348 0x14009D520 | ✅ 0x14009A0B0 calls 0x1400E12C0 | Worker count loads from config +40; ❓ thread count default not checked |
| Start | LobbyDaemon::onStart, 348 0x14009DCA0 | ✅ 0x14009A830 | Starts tick/timers, connects upstream services, then opens listeners |
| Timer interval | LobbyDaemon::GetTimerInterval, 348 0x14009AF60 | ✅ 0x140097B10 | Loads u32 at config +48 |
| Timer maximum | LobbyDaemon::GetTimerMaxTime, 348 0x14009AF70 | ✅ 0x140097B20 | Loads u32 at config +52 |
| Gate packet jobs | ❓ Guide job details not checked | ✅ 0x1400BC750 | Queues channel packets with PostQueuedCompletionStatus; joins fragments when flag +9 is set |

## Listeners and outbound links

Port IDs below are internal network identifiers. They are not TCP port numbers.
No endpoint values are stored here.

| Link | Direction and condition | 348 class | 307 main object or routine | 348 source |
| --- | --- | --- | --- | --- |
| LobbyGate | ✅ listener; port ID 4 | LobbyGateChannelManager | ✅ 0x14009A0B0; daemon +2032; vtable 0x1401BB5C8; port config +196 | ❓ 0x14009D520 |
| GameDaemon | ✅ listener; port ID 5 | GamedChannelManager | ✅ 0x14009A0B0; daemon +2024; vtable 0x14017A308; port config +192 | ❓ 0x14009D520 |
| InfoGate | ✅ optional listener; port ID 6; config flag +388 | InfoGateChannelManager | ✅ 0x14009A0B0; daemon +2048; vtable 0x1401413E8; port config +200 | ❓ 0x14009D520 |
| Manage | ✅ listener; port ID 1 | ManageSessionHandler | ✅ 0x14009A0B0; daemon +2040; vtable 0x140197E28; port config +204 | ❓ 0x14009D520 |
| AccountInventory | ✅ outbound; port ID 3; config flag +374 | AccountInventoryChannel; AccountInventorySessionHandler | ✅ 0x14009A830; connect wrapper 0x140005860; address/port config +376/+380 | ❓ 0x14009DCA0 |
| PostOffice | ✅ outbound; port ID 7; config flag +520 | PostOfficeChannel; PostOfficeSessionHandler | ✅ 0x14009A830; handler object daemon +2064; handler vtable 0x140169C30 | ❓ 0x14009DCA0 |
| NP / StsMsg | ✅ outbound; port ID 2 | NPChannel; NPServiceChannelManager | ✅ start 0x14009B930; manager initialize 0x1400CB380; manager daemon +2000; address/port config +184/+188 | ❓ 0x14009F010; 0x1400CFBA0 |
| DB | ✅ outbound ODBC | Db; LobbyDB | ✅ initialize 0x140097EB0 calls 0x1400E3250; ODBC imports in .idata 0x14011B000 | ❓ 0x14009B300; 0x1400E7DF0 |

✅ AccountInventory and PostOffice startup wait up to 5000 ms for their initial connect event at 0x14009A830.
✅ NP startup also waits 5000 ms at 0x14009B930. The NP manager passes a five-second connect timeout at 0x1400CB380.
✅ NP source-path text names NPChannel2.cpp at 0x14011CAD8. The guide class name is NPChannel, not a proven class named NPChannel2.
❓ Full StsMsg ABI and DLL entry mapping remain unchecked. Do not treat NP as the ordinary packet-command protocol.

## Name-map method and limits

Start with the five named 348 handler arrays. Check target dispatch references and command limits.
✅ The target entry stride is 16 bytes. Offset +0 has the handler pointer. Byte +8 selects channel handling. Byte +9 selects fragment handling.
Sources: 0x140059D60, 0x1400BC750, 0x14002E490, 0x14007F920 and 0x1400075C0.
There is no second handler pointer at +8.

Use command position and independent unique shared string references for exact handler identities.
Use SQL procedure names to propose LobbyDB identities. Only LoadGameAccount has a direct two-build body check in this job.
The map uses `exact` and `likely`. It does not write names into either IDA database.
The JSON stores SHA-256 string anchors. It does not store arbitrary binary strings.
❓ Callee sets, imports and constants did not receive a systematic match pass. The map is not complete.
❓ The guide string-fingerprint pass covered IDA's ASCII string list. Target format and SQL passes also read literals directly.

## Out-of-scope boundaries

| Boundary | Evidence | Scope |
| --- | --- | --- |
| Tencent TSS SDK | ✅ 307 tss_get_sdk 0x140175FC8; tss_sdk 0x140175FD8 | Presence only. No implementation analysis. |
| GameGuard library | ✅ File BnS_2020/LobbyDaemon/bin/ggauth64_27.dll | Directory entry only; binary address not applicable. ❓ EXE reference not established. |
| GameGuard config | ✅ File BnS_2020/LobbyDaemon/bin/csauth3.cfg | Directory entry only; binary address not applicable. File contents not read. |
| BloodCert | ❓ Excluded by the job; no target mapping attempted | No licence-check work. |
| BNSHelper.dll | ✅ File BnS_2020/LobbyDaemon/bin/BNSHelper.dll | Directory entry only. No load, map or call. |

No client executable was read. No network request was made. No binary or IDA database write was requested.
