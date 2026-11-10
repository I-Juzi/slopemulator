# 00 — StServers shared framework (RE reference)

Summary:
1. All 51 StServers programs link one framework (`Engine\StsApp`, `Services\StsLib`, `Core\Platform`, OpenSSL 1.0.x, RapidXml).
2. HelloSrv is the template: all 5590 functions of `HelloSrv.exe` match `framework-funcs.tsv`. Its own code is at `0x1400C3900`–`0x1400C5000` (`HelloConn.cpp`, `HelloConsole.cpp`).
3. Start: `wmain`-type entry `0x14011C410` parses the command line, selects a run mode, opens a PID file and reads `StServer.ini` `[Logs]`.
4. Service model: a global application object (`CApplication`) plus socket managers, a timer heap (`CStsTimer`) and a cleanup registry with a 7-phase shutdown machine (`0x1400F0300`).
5. Messages, route bus and DbSrv calls reuse the specs in `Sts/` and `StServers/1x`; this chapter adds only new facts.

All addresses are HelloSrv (`HelloSrv.exe.i64`) unless the text gives an other binary. ❓ = not verified in IDA.

## Quick find

| Question | Function | HelloSrv address | Section |
| --- | --- | --- | --- |
| Where does the process start? | `StsCmdMain` | `0x14011C410` | 1.1 |
| Which command line flags exist? | flag table `CCmdParser` | `0x140162400` | 1.1 |
| How does the service install? | `StsInstallService` | `0x14011CAF0` | 1.2 |
| Where is the console/service main loop? | `StsRunApp` | `0x14011C380` | 1.2 |
| Where are log and crash paths set? | `StsInitPaths` | `0x14011BE20` | 1.3 |
| How does a config value get read? | `IniGetStr` / `IniGetInt` | `0x1400D5990` / `0x1400D5A40` | 1.4 |
| Network override lookup | `NetCfgLookup` | `0x1400F79B0` | 1.4 |
| Shutdown phases | `StsShutdownTick` | `0x1400F0300` | 1.6 |
| Framework source file → functions | `source-files.tsv` | — | 1.0 |
| Classes and vtables | `framework-classes.tsv` | — | 2.1 |
| Request → reply wait, timeout | `TxnOnReply` / `StsSocketTxnTimer` | `0x1400EC7F0` / `0x1400EC070` | 2.5 |
| Worker thread pools | `ThreadPoolsStart` | `0x1400C84E0` | 2.6 |
| Idle socket disconnect | `SocketIdleTimer` | `0x1400F9DA0` | 2.7 |
| Live config reload | `ConfigMonitorTimer` | `0x1400F1740` | 2.8 |
| Route register retry | `RouteRegRetry` | `0x140103770` | 2.9 |
| Error record to/from XML | `ErrFromXml` / `ErrToXml` | `0x1400E7580` / `0x1400E7AC0` | 3.1 |
| Send an error reply | `ReplyError` | `0x140104770` | 3.3 |
| Send a request and wait | `SendToProviderTxn` | `0x1401049A0` | 3.3, 4 |
| DbSrv proc call | `DbProcBegin` (AuthSrv) | AuthSrv `0x140112220` | 4 |
| Admin HTTP page list | `StsHttpPagesInit` | `0x14011F3A0` | 5.1 |
| TLS config | `TlsReadCfg` | `0x1400FCA90` | 5.2 |
| AES key config | `CptReadKeyCfg` | `0x1400D2560` | 5.3 |
| Service counters | `StsServiceCounterCfg` | `0x140106170` | 5.4 |
| Command table, procs, keys, timers of a service | `recipe.py` | — | Service recipe |

## 1. Process start and host

### 1.0 Source files

The framework keeps `__FILE__` strings for asserts and logs. `source-files.tsv` lists each source file and the HelloSrv functions that refer to it. Use it to find the part of the framework that a function belongs to. Areas (HelloSrv): Exe platform `0x1400C6000`–`0x1400D0000`, config and crypto `0x1400D2000`–`0x1400D7000`, InetMsg and XML `0x1400DB000`–`0x1400E4000`, HTTP admin `0x1400E4000`–`0x1400E7000`, inet sockets `0x1400EA000`–`0x1400EE000`, StsLib core (timer, main, monitor, server, socket, port, route) `0x1400EF000`–`0x140107000`, Str/Log/File `0x140118000`–`0x14011B000`, StsApp (WinMain, command line, HTTP commands, window) `0x14011B000`–`0x140126000`, Arena sockets, profile, TLS `0x140126000`–`0x140138000`, ErrLog `0x14013B000`–`0x14013D000`, OpenSSL after `0x14013D000`.

### 1.1 Command line (`0x14011C410`)

1. `CCmdParser` (`0x14012A180`, vtable `0x1401625C0`) parses the flags from the table at `0x140162400` (10 rows of 24 bytes: flags, name, id). Flag syntax is `/Name [value]` (usage text in `0x14011CE50`).

| Id | Flag | Value | Effect |
| --- | --- | --- | --- |
| 0 | `?` | none | show usage (`0x14011CE50`), exit code 100 |
| 1 | `AppIndex` | number | instance index, global `0x140217F4C` (default 1). 0 → message, exit 101 |
| 2 | `Email` | text | operator e-mail. Console mode: if empty, read env `UserEmail`; if still empty → exit 102 |
| 3 | `SpawnSrv` | text | run under SpawnSrv (mode 3) |
| 4 | `SpawnPorts` | none | `0x1400EFE40` sets a flag ❓ (use of the ports from SpawnSrv) |
| 5 | `HideWindow` | none | global `0x140217F48` = 1 (no status window) |
| 6 | `RunService` | none | mode 2 (Windows service) |
| 7 | `InstallService` | none | mode 1 (install and exit) |
| 8 | `UserName` | text | service account |
| 9 | `Password` | text | service password (`keyhere`) |

2. Run mode, global `0x140217F44`: 1 = install, 2 = service, 3 = spawned (SpawnSrv given), 4 = console (default).
3. Install mode: `UserName` must have `Password`, except for `LocalService` and `NetworkService` (these get the `NT AUTHORITY\` prefix). Else message, exit 103.
4. Other modes: open the PID file `<ServiceName><AppIndex>.pid` (`0x14011D070`, name in `0x14011CFA0`; content `%S: %u` = name and process id; error log `StsPid, failed to create %s (%s)`). Fail → message box text "Unable to open PID file …", exit 104. This blocks a second process with the same name and `AppIndex`.
5. Service name: `0x14011C9F0` builds `%s%u` (program name + AppIndex) or `%s`. Default e-mail is `%s.%s@ncsoft.com` (service name, computer name).
6. Environment `NpLogPath` and `NpCrashPath` override the log and crash folders (`0x14011C410`, passed to `0x14011BE20`).
7. Mode 1 → `0x14011CAF0`; mode 2 → `StartServiceCtrlDispatcherW` with service main `0x14011CDD0`; modes 3 and 4 → `0x14011C380` directly.

### 1.2 Service and console

- `StsInstallService` `0x14011CAF0`: `OpenSCManager`, `CreateServiceW` with display name `NCsoft.%03d %s` and command `"<exe>" /RunService /AppIndex %u /Email %s`, then `ChangeServiceConfig2` (failure actions and the actions flag). Errors go to `0x14011C160` with the API name.
- Service main `0x14011CDD0`: `RegisterServiceCtrlHandlerW`, `SetServiceStatus`, then `0x14011C380`.
- `StsRunApp` `0x14011C380`: app start `0x14011C040` (module handle, error report `0x14013BDC0`), main loop `0x1400F00F0` (thread pool, `timeGetTime`), stop of the HTTP admin `0x1400E6650`, final cleanup `0x1400EFE10`.
- Status window: `StsWnd.cpp` `0x140125670` (window class `StsWnd`, reads `<exe>.ini`) ❓ details. `/HideWindow` hides it.

### 1.3 Paths, logs and crash dumps (`0x14011BE20`)

1. Find `StServer.ini` in the program folder and up (`0x1400F08A0`, `0x1400F1C90`). Not found → fatal `configuration file not found` (`StsWinMain.cpp` line 125).
2. `StServer.ini [Logs] LogPath` (default `Logs`) → log folder (`0x1400F0C70`). `[Logs] CrashPath` (default `Logs`) → crash folder (`0x1400F0BB0`). The env/argument values from 1.1 step 6 win over the INI.
3. Error log init `0x14013C410` (`ErrLog.cpp`). The app id string is `%s:%s` (service name, version from `0x140132540`), global `0x140217FE0`.
4. Crash handler `0x14011C0F0` is set with `0x140119EF0`. Dump file name: `Crash_%s_%u_%I64u.dmp` (service name, process id, start time) in the crash folder (`0x1400C6CD0`).
5. Fatal errors (`0x1400CF370`, `ExeError.cpp`) append to `NcPlatform.log` (`0x1400CF070`) the text `App: %s\r\nFatal error (%u): %s\r\nWhen: %d/%d/%d %02d:%02d:%02d`.
6. CRT hooks: invalid parameter → log `Invalid parameter in function %S (File: %S, Line:%d) expr=%S` (`0x14011ABD0`); pure virtual call → `A pure virtual function is called.` (`0x14011AC40`).
7. Log file names (table at `0x140187838`, used by log init `0x1401333C0` from `0x1400C3860` with level mask 15 ❓): `Debug%02u%02u%02u_%s_%s.log`, `Info…`, `Error…` (date `yymmdd`, then two names), and the short forms `Debug%02u%02u%02u.log`, `Info…`, `Error…`. Three levels: Debug, Info, Error.
8. Per-socket trace log `StsInetSocket_%u_%u.log`, line format `%u (%u:%u:%u): %s` (`0x1400EA7D0`).
9. Mail of logs (`0x140124CF0`, network lookup first, then `StServer.ini [Logging]`): `EmailLogs` (default 1), `SmtpServer` (default `smtp.ncsoft.com`), `CrashRecipient`, `DebugRecipient`, `ErrorRecipient` (NCsoft addresses). `[Logs] LogAgentAddress`, `[Logs] LogAgentEnable` → `CLogAgentConn` (vtable `0x140164D08`). The port must not send mail.

### 1.4 Config reading

- INI object: `Config.cpp` (`0x1400D5330`–`0x1400D58A0`). Section and key lookups are case-insensitive hashes (`THashKeyStrPtr<wchar_t, THashKeyStrCmpI>`). A key can have more than one value (index argument).
- `IniGetStr(ini, section, key, index, default)` `0x1400D5990`; `IniGetInt(...)` `0x1400D5A40` (`_wtoi`-style); `IniGetValues(ini, section, key, &count)` `0x1400D5640`; `ValueAsInt(values, index, default)` `0x1400D5AF0`; `ValueAsStr` `0x1400D5A20`.
- `NetCfgLookup(ini, scope, key, &count)` `0x1400F79B0`: tries 3 key forms × 5 section forms and returns the first hit. Key forms in order: `<key>.Override`, `<key>.<scope>`, `<key>`. Section forms in order: `Network.<a>.<b>`, `Network.<a>`, `Network.*.<b>`, `Network.*`, `Network` (a and b come from `0x1400F2FF0`, probably datacenter and computer ❓). `NetCfgInt` `0x1400F7980` = lookup + `ValueAsInt`. The file is `Network.ini` (`0x1400F59D0`, `0x1400F5A60`) ❓ (the INI handle is passed by the caller).
- Live reload: `StsConfigMonitor` (vtable `0x140159468`), `CChangeMonitor` (`0x140152F88`), `CStsConfigNotify` (`0x140159650`), `CStsConfigRef` (`0x140159440`). See section 2.
- All config keys that the framework reads, with defaults: `config-keys.tsv`.

### 1.5 Config keys (framework)

Full list: `config-keys.tsv` (47 rows, from `cfgkeys()` in `recipe.py` on HelloSrv; columns getter, call address, function, section, key, default). Section `(arg)` = the caller gives the section. Main rows below.

| File / section | Key | Default | Reader |
| --- | --- | --- | --- |
| `StServer.ini [Logs]` | `LogPath`, `CrashPath` | `Logs` | `0x14011BE20` |
| `StServer.ini [Logs]` | `LogAgentAddress`, `LogAgentEnable` | —, 0 ❓ | `0x140124CF0` |
| `[Logging]` | `EmailLogs` 1, `SmtpServer`, `CrashRecipient`, `DebugRecipient`, `ErrorRecipient` | see 1.3 | `0x140124CF0` |
| Network | `DisableShutdownTimeout` | 0 | `0x1400F02C0` |
| Network | `DisableTransTimeout`, `DisableIdleDisconnect`, `DisableTransTimeoutLogging` | 0 | `0x1400FA020` |
| `Route.ini [General]` | `ShutdownTimeoutMs` 60000, `RegistrationRetry` 3000, `RegistrationRetryLimit` 1200, `CheckRegistrationRetryMs` 5000, `CheckRegistrationRetryLimit` 720 | | `0x140100E50` (see `14-route-client.md` §2.4) |
| `Route.ini [TLS]` | `Certificate`, `KeyBits`, `EncryptedKey`, `Key` (`keyhere`) | | `0x1400D2560` |
| `[Tls]` | `Method`, `Certificate`, `CipherList`, `TmpRsaKeyLength` 1024, `TmpDhKeyLength` 1024 | | `0x1400FCA90` |
| `[Program]` | `Source` (`Hash` default; also `Baked`, `Hash100`) | `Hash` | `0x1400F6700` |
| `[ServiceCounter]` | `UpdateInterval` 1, `Enabled` 0, `Enabled.<name>` | | `0x140106170` |
| `StServer.ini` | `[Datacenters] center`, `[Addresses] Addr`, `[Rights] Right`, `[Systems] Addr`, `[Tcpip] Port` | | `0x1400F5CF0`, `0x1400F6090`, `0x1400F6520`, `0x1400F6C80`, `0x1400FC5D0` |
| `StServer.ini` | `[GlobalPreferredServices] Service`, `[InstantGlobalForwards] Service` | | `0x140100FE0` |

`StServer.ini` with no `[Tcpip]` port → log `Sts: no ports listed in StServer.ini` (`0x1400FBCE0`).

### 1.6 Shutdown machine (`StsShutdownTick` `0x1400F0300`)

The main object holds the current phase function at `+0x20` and an offset at `+0x28`. Each tick calls the phase. A phase sets the next phase, returns 0 (go on) or a wait (10 = poll again ❓ unit ms).

| # | Phase | Function | Action | Next |
| --- | --- | --- | --- | --- |
| 1 | `MainInitializeApp` | `0x1400F05F0` | select profile from the run mode (global `0x140215E90`), load `StServer.ini`, open ports (`0x1400FBCE0`), start monitor (`0x1400F1EB0`) | running |
| 2 | `MainStartCleanup` | `0x1400F0710` | stop monitor (`0x1400F16F0`), close ports (`0x1400FBC30`) | 3 |
| 3 | `ClientStartCleanup` | `0x1400F0530` | call `StartCleanup` (vtable +0x08) of each client cleanup object, last first | 4 |
| 4 | `ClientQueryDestroy` | `0x1400F0490` | call `QueryDestroy` (+0x10) of each object that is not done; 0 → return 10 (wait) | 5 |
| 5 | `ServerStartCleanup` | `0x1400F0810` | `StartCleanup` (+0x18) of server objects | 6 |
| 6 | `ServerQueryDestroy` | `0x1400F0770` | `QueryDestroy` (+0x20); wait as in 4 | 7 |
| 7 | `MainCompleteDestroy` | `0x1400F05B0` | final destroy (`0x1400EFA80`) | exit |

- Cleanup registry: array `0x140215E60` (count in high dword of `0x140215E68`), done flags `0x140215E78`. Cleanup classes: `CStsRouteCleanup`, `CStsHttpConnCleanup`, `CSpawnCleanup`, `CStsDirectConnCleanup`, `CStsServiceCounterCleanup` (6 virtual slots each, see `framework-classes.tsv`).
- Watchdog: when the phase time is over `0x1401C77E8` = 120000 ms and `0x1401C77EC` = 1, it logs `Shutdown timeout (%s), after %u.%03us` with the phase name and calls fatal (`StsMain.cpp` line 282). `DisableShutdownTimeout` = 1 sets `0x1401C77EC` = 0 (`0x1400F02C0`).
- `quirk_shutdown_fatal`: a slow cleanup kills the process after 120 s with a fatal error (no clean exit). Keep it.

## 2. Service model

### 2.1 Classes

`framework-classes.tsv` lists 176 framework RTTI classes with vtable, sub-object offset, slot count and slot addresses (HelloSrv). Important bases: `IStsInetSocketNotify` (`0x140152000`, 8 slots), `CStsInetSocketNotify` (`0x140158928`), `CStsInetSocket` (`0x140158758`, 19 slots), `CStsInetSocketManager` (`0x140158990`), `TStsInetSocketFactory<T>` (2 slots: slot 0 = create T), `CStsAppTimer` (`0x140159250`), `CStsConfigNotify` (`0x140159650`).

### 2.2 Service start (HelloSrv example `CApplication` slot 0 = `0x1400C4C20`)

The framework calls the service init (`CApplication` vtable `0x140152328`, slot 0). HelloSrv init does, in order:
1. `StsSetServerType(115)` `0x1400F0D30` — server type id (115 for HelloSrv ❓ meaning of the number), program name to `0x140215F40`.
2. `unknown_libname_6_0("HelloBassic", 0, &1)` ❓ (not decoded).
3. `StsRouteStart(factory, 0, 0)` `0x140104980` → `StsRouteInit` `0x1401047C0` (`Route.ini`). The factory is `TStsInetSocketFactory<CHelloSocketNotify>` (vtable `0x140152090`, global `0x1401C6000`). Slot 0 (`0x1400C3CA0`) creates one `CHelloSocketNotify` per connection.
4. `StsAppCommonStart(0)` `0x140107A40`: admin page `/sts/counters`, SpawnSrv connection (`L"SpawnSrv"`, `0x1400EE4D0`) when spawned, monitor of `StServer.ini`.
5. `StsConfigMonitorAdd(L"Hello.ini", &CHelloConfig)` `0x1400F1EB0`: watch the service INI. On load and on change the framework calls the notify slot 0 (`0x1400C4C90` reads `[Notification] IntervalMs`). The monitor record is 0x2A0 bytes; the notify object is `StsConfigMonitor` (0x28 bytes).
6. `0x1400C44C0` → `StsHttpAddCommands(table, 4, 1)` `0x1400E6AC0`: admin HTTP test pages `/test/`, `/test/get_echo_test`, `/test/start_notice_test`, `/test/stop_notice_test`.

### 2.3 Message dispatch (per connection)

1. Name → id maps: a static initializer calls `NameMapInit(&hash, count, table)` `0x140107DE0`; the table rows are 16 bytes `{char* name, int64 id}`; `atexit` frees the hash. `NameMapFind` `0x140107EF0` resolves a name.
2. HelloSrv has 8 maps (initializers `0x1400C12D0`–`0x1400C37D0`): Hello (`0x140215970`, 5: Invalid 0, Echo 1, Notice 2, StartNotice 3, StopNotice 4), route bus (`0x140215990`, 23), HTTP methods (`0x140215AE8`, 5), file (`0x1402187A0`, 14), port/SpawnSrv (`0x140218B18`, 9), `0x140218B30` (4), `0x140218E28` (29), service names (`0x1402CF518`, 130, first `AdAuthSrv`).
3. `CHelloSocketNotify` slot 0 `0x1400C4320` = OnMessage: parse with `0x1400D01E0` (gets the command name through message vtable +0x18, looks it up in the map, fills a request record whose first dword is the id), then `switch` on the id: 1 → `0x1400C3CD0` (Echo), 3 → `0x1400C3E40` (StartNotice), 4 → `0x1400C4000` (StopNotice). Reply: `StsRouteReply` `0x140104D60`. Unknown id: no handler ❓ (reply path not decoded).
4. Route bus handlers found by the same switch pattern: ShutdownServer `0x1400FFC00`, SetBlocks `0x1400FEE80`, GetBlocks `0x1400FE460`, RouteXferRequest `0x1400FFFF0`, RouteXferBlock `0x1400FFCD0`, ProviderFound `0x1400FE650`, ProviderLost `0x1400FE960`, RouterConnect `0x1400FEC70`. SpawnSrv map: Shutdown `0x140106E20`, Connect `0x140106BB0`, Found `0x140106CE0`, Lost `0x140106D80`.

### 2.4 Timers (`StsTimer.cpp`)

- `StsTimerSet(notify, ms, flag)` `0x1400EFB50` (thunk `0x1400EFA90`, 44 callers). The notify object has a pointer at +8 to a 0x30-byte `TPriorityTime<CStsTimer>` record (created on first use: +0x08 heap, +0x10 heap index, +0x18 due time, +0x20 notify, +0x28 flags). `ms = -1` with `flag = 0` removes the timer from the heap. Else due = `timeGetTime() + ms` (`StsTimerSchedule` `0x1400EF6E0`). Lock: critical section `0x140215DE0`. If the timer system is not up (`0x140215E2C` = 0): log `StsTimer:%u, notify(%p), timer is not initialized, yet`.
- Negative ms → 120000 ms and one log `StsTimer: negative sleep from %p` (flag bit 2). `quirk_timer_negative_120s`.
- `StsTimerDestroy(notify)` `0x1400EF360` (thunk `0x1400EFA80`): logs `StsTimer:%u, notify is null!`, `StsTimer:%u, notify(%p)'s timer destroyed, already`.
- Timer thread `0x1400EF7D0`: takes due records from the heap `0x1401C77C8`/`0x1401C77D0`, calls notify vtable slot 0 (OnTimer). The return value is the next delay in ms; `-1` = stop. So a periodic timer returns its period (HelloSrv `CNoticeTimerNotify` OnTimer `0x1400C4CD0` sends `Notice` with `StsRouteSend` `0x140104D80` and returns the interval at +724).
- HelloSrv StartNotice (`0x1400C3E40`): create `CNoticeTimerNotify` (`0x1400C4B10`), then `StsTimerSet(obj, interval, 0)`.

### 2.5 Transactions (request → reply, timeout)

- A request that waits for a reply carries a txn object. Txn vtables share one layout (HelloSrv, for example `CStsRouteXferReqTxn` `0x14015ACA8`): slot 0 dtor, slot 1 `TxnOnReply` `0x1400EC7F0`, slot 2 `0x1400C4B00` (calls slot 5), slot 3 = OnSuccess(msg), slot 4 = OnError(err), slot 5 `0x1400C45C0`.
- `TxnOnReply(txn, status, msg)` `0x1400EC7F0`: checks the reply form against the txn flags (dword txn+0x10: bit 0 = message can be absent, bit 4 = pass/fail reply is accepted, bit 8 = many replies allowed). Good form → slot 3. Bad form → slot 4 with error `{code 4 ErrBadData, 9001, line, text}`. Lines/texts: 3590 and 3570 `invalid one/many reply`, 3587 `message required`, 3559 `bad set of replies`, 3562 and 3577 `has many replies`, 3565 `pass/fail required`.
- Error record layout (stack, 0x30 bytes): `{int code, int 9001 ❓(module id), int line, ..., char* text, ..., int 256}`. AuthSrv uses `{3002, 5, 525}` for its own errors, so the 2nd field is a module id ❓.
- Txn timeout: socket timer slot `0x1400EC070` (vtable ref `0x140158B48`). Pending list at notify+280 is sorted by due time (txn record +88 = due `timeGetTime` ms, +104 = txn object, +120 = done flag, +0 = txn number). For each due record: log `StsInetSocket: %S, transaction timeout #%u` (code 42, line 2522; no log when socket flag bit 2 = `DisableTransTimeoutLogging`), then txn slot 1 with status `{42 ErrTimeout, 9001, 2524}` and no message. When the socket is closed (notify−376 ≠ 0), all records of lists +248 and +280 get the same ErrTimeout 2524. Return = ms to the next due record, or −1 when the list is empty or `DisableTransTimeout` is set (bit 0 of notify−372).
- `StsSocketReadDisableCfg` `0x1400FA020`: `NetCfgInt` keys `DisableTransTimeout`, `DisableIdleDisconnect`, `DisableTransTimeoutLogging` (default 0).
- Send with txn: `SendToProviderTxn(msg, txn, ms, flag)` HelloSrv `0x1401049A0` (AuthSrv `0x140157B00`): finds the provider by protocol name in a hash, then calls route manager `qword_140216968` vtable +88. No retry in this path ❓.

### 2.6 Threads (`ExeThread.cpp`)

- `ThreadPoolsStart` `0x1400C84E0` resolves `SetThreadStackGuarantee` and `GetQueuedCompletionStatusEx` with `GetProcAddress`, then makes 3 thread pools (0x1080-byte records, `ThreadPoolInit` `0x1400C7130`): index 0 `Compute`, 1 `Socket`, 2 `File`. Each pool has one IO completion port (`CreateIoCompletionPort(-1, 0, 0, 0)`) and an event `m_wakeAllSignal`. Per-pool constants from tables `0x140152A40`/`0x140152A50`/`0x140152A60`: Compute (9, 10, 13), Socket (15, 16, 19), File (18, 19, 20) ❓ (use not decoded; stored at pool +36, +40, +44).
- Thread count per pool (`0x1400C7130`): override `0x1402153B0[pool]` if not 0; else from CPU count `N` (`0x1400C86E0`: bits in the process affinity mask, else `dwNumberOfProcessors`): Compute = `N`, Socket = `N + 2`, File = `N − 1` but at least 2; minimum 1. Above 64: log `THREAD_POOL_MAX_THREAD_COUNT limitation of %d hit for threadPoolSize (%u)` and use 64.
- `ThreadPoolSetCount(pool, n)` `0x1400C8810` writes the override (log `ThreadPoolSetThreadPoolCount: count (%u) is larger than THREAD_POOL_MAX_THREAD_COUNT (%u)` when n > 64). `WinMain` `0x14011CEE0` calls it for pools 0, 2 and 1 with n = 1. So every StServers process runs **1 Compute, 1 Socket and 1 File thread**. Handlers of one process never run in parallel on the Compute pool (port: one task per pool is enough).
- Worker body `0x1400C8850` → `0x1400C7AF0`: `GetQueuedCompletionStatusEx` (or `GetQueuedCompletionStatus` when absent), then completion dispatch `0x1400C9190(key, overlapped, bytes)`, then `0x1400C79E0`; loops until pool +32 ≠ 0 (stop).
- Extra thread `ThreadPoolComputeWakeUpAndAPC` (body `0x1400C7600`, made by `ThreadCreate` `0x1400C6FE0`): waits on its event with a timeout, records `timeGetTime` at +56, and posts a 0x68-byte wake packet to the Compute port (`PostQueuedCompletionStatus`). Then `SleepEx(1, TRUE)` (alertable, runs APCs).
- `ThreadCreate(obj, fn, stack, name, ...)` `0x1400C6FE0`: `Platform::CThread` object (0xE8 bytes), event `m_pauseSignal`, `_beginthreadex(..., CREATE_SUSPENDED)`. Fail: log `ThreadCreate failed (%d)` and `ExeFatal` (lines 565, 599). Other thread maker: `0x1400C81D0` (used for `ChangeMonitor`, 2.8).
- Main flow `StsMainLoop` `0x1400F00F0`: stores the PID, the program name (`0x140215F40`), log init `0x140130F90`, timer system init `0x1400EF670`, then arms the phase timer `0x1401C77F0` (OnTimer = `StsPhaseMainInitializeApp`) with `StsTimerSet(…, 0, 1)` (run now) and waits on event `0x140215E40` (`0x14011AB10` / `0x14011AAD0`).

### 2.7 Per-connection state and idle disconnect (`StsSocket.cpp`)

- Socket manager `CStsSocketManager` init `0x1400F8630` (reads `StServer.ini`): manager +40 = idle timeout **60000 ms**, +44 = 30000 ❓ (use not decoded), +48 = 0, +36 = disable flags (`StsSocketReadDisableCfg` 2.5), +188 = arg 4, +60 = name (64 wchar). Two lists: +272 (80) and +312 (6) ❓.
- Idle timer slot `0x1400F9DA0` (vtable refs `0x140158838`, `0x140159FA0`; `this` = socket +16). Socket +80 = last activity time (`timeGetTime`). `left = last + idle − now`.
  - `left > 0`: return `left` (next check).
  - `left ≤ 0` and manager +36 bit 1 (`DisableIdleDisconnect`) = 0: if manager +48 ≠ 300, log level 2 `StsSocket::%u, Disconnect - idle timeout for (%s)` (line 718, peer address from vtable +120). Then if socket +132 ≠ 0 call `0x1400CE2E0` (close), call the notify (+104) slot +32 (OnDisconnect), set +96 = 1 (closing). Counter `stssocket disconnect - idle timeout`.
  - `left ≤ 0` and the flag is set: log level 1 `StsSocket::%u, Disconnect skipped by cfg(DisableIdleDisconnect) - idle timeout for (%s)` (line 702); return the full idle time. Counter `stssocket disconnect - idle timeout (skipped)`.
  - Socket +96 ≠ 0 (closing) with no notify and +132 = 0: release the object at +200 (vtable +16) and return −1 (stop timer).
- Other socket logs (HelloSrv): `0x1400F9790` `StsSocket::%u, incomming connection from (%s) is denied by address right(check the whitelist)` (accept filter, admin page `/sts/config-whitelist`); `0x1400FAF00` `Disconnect - previous socket (%s), (%p)`; `0x1400F8DD0` `Disconnect - inactivated socket`; `0x1400F99E0` `StsSocket: reconnecting deactivated socket`; `0x1400FA180` `StsSocket: QueryReconnect, no matching socket`.
- `ExeSocket.cpp` (raw IOCP sockets): `0x1400CDCC0` `ExeSocket::%u, Disconnect - Backlog, q:%u, i:%u` (send queue too large; counter `socket disconnect backlog (total)`); `0x1400CECF0` `ExeSocket::%u, Disconnect - connect timeout for (%s)` (counter `socket disconnect connect timeout (total)`). Limits ❓ (not decoded).
- `StsInetSocket` message checks (each logs and drops): `0x1400EC3E0` `request with reply code`, `reply without transaction code`; `0x1400EC610` `request with transaction response code`; `0x1400EB620` `invalid transaction reply`; `0x1400EB4B0` `unknown internal message, '%s/%s'`; `0x1400ECF50` `dropped overly large reply`, `http response to non-http client`; `0x1400EBCC0` `no registered service, connType %d, program %d, build %d`. Per-socket log file `StsInetSocket_%u_%u.log` (`0x1400EA7D0`), trace line `StsInetSocket: %S, %s (%u head + %u body), %.*s` (`0x1400EADD0`).

### 2.8 Live config reload (`StsMonitor.cpp`, `ExeChange.cpp`)

1. `ChangeMonitorStart` `0x1400C9DE0`: `FindFirstChangeNotificationW(dir, FALSE, 0x11)` (file name and last write, no sub-tree); fail log `Change: %s failed (%#x)`. Thread `ChangeMonitor` (body `0x1400C9F00`).
2. Thread loop: `WaitForMultipleObjects(3, …)`. After a change it calls `FindNextChangeNotification` and waits again with timeout **5000 ms** (collects more changes); on timeout it runs `0x1400C99E0` to check the watched files ❓ (compare method not decoded). Logs: `Change: WaitForMultipleObjects failed %#x`, `Change: FindNextChangeNotification() failed %#x`.
3. `ChangeWatchFile` `0x1400CA230` opens the file (`CreateFileW` read, share read) and adds a 0x238-byte record to the watch hash. `CChangeMonitor` vtable `0x140152F88`.
4. `StsConfigMonitorAdd(file, notify)` `0x1400F1EB0` / `0x1400F1D70`: one 0x250-byte monitor per file name (hash `0x1401C7858`), list of notify objects. It calls the new notify at once and arms the monitor timer with `StsTimerSet(mon+8, 0, 1)`.
5. Monitor OnTimer `0x1400F1740`: `StsConfigLoad(file, mon+152, reload)` (first 0, then 1), then calls slot 0 of every notify with the new config. So a service sees the new values on its next notify call; there is no restart.

### 2.9 Route register retry (`StsRoute.cpp`)

The behaviour is in `../14-route-client.md` §2.4 (keys and defaults) and §4 (registration). HelloSrv agrees with it (same defaults, same 3053 rule, same log strings). HelloSrv addresses for that spec:

| Part | HelloSrv | Note |
| --- | --- | --- |
| `RouteReadGeneralCfg` (`[General]` keys, `[TLS]` with `CptReadKeyCfg`, value `keyhere`) | `0x140100E50` | globals `0x1401C79C0`..`0x1401C79D0` (ShutdownTimeoutMs 60000, RegistrationRetry 3000, RegistrationRetryLimit 1200, CheckRegistrationRetryMs 5000, CheckRegistrationRetryLimit 720) |
| Register reply handler | `0x140103120` | txn type = txn vtable +48: 1 `RequestRegister`, 2 `CheckRegister`; other: log `StsRoute: invalid type of txn, line(%d)` (1322) |
| `Retry(reg, type, …)` | `0x140103770` | count +32 (type 1) or +36 (type 2) ≤ limit → `AddBackLog`, return 1; else log `CStsRouteRegister::Retry: retry limit exceed, type: %s, try count: %u`, return 0; bad type: line 1525 |
| `AddBackLog` | `0x1400FE160` | backlog list at register +120, key `timeGetTime` (`0x140104E90`) |
| `OnBackLogTimeout` | `0x140100D80` | re-parse (`0x140103540`), send again (`0x140103F40`); logs `invaild back log data`, line 1409 |
| `BuildCheckRegistrationMsg` | inside `0x140103120` | fail log line 885 |

Note: the limit test is `limit < count` → fail, so the real number of tries is limit + 1 (`quirk_route_retry_off_by_one`, harmless).

## 3. Messages, replies and errors

Wire format, headers and txn ids: `../../Sts/01-transport.md`. Route bus: `../14-route-client.md`. This section adds the error record and the service reply path.

### 3.1 Error record (`StsError`, 0x30 bytes)

| Offset | Type | Field | XML attribute |
| --- | --- | --- | --- |
| +0 | int | code (`errcodes.tsv`) | `code` |
| +4 | int | server | `server` |
| +8 | int | module | `module` |
| +12 | int | line (source line of the error) | `line` |
| +16 | char* | text (heap; freed by the owner) | `text` |
| +24 | ptr | extra XML (child elements) | children of `<Extra>` |
| +36 | int | extra present flag | — |
| +40 | int | 256 (init value) ❓ | — |

- `ErrInit(err, code)` `0x1400E73F0`: code; server, module, line, text, extra = 0; +40 = 256.
- `ErrFromXml(err, elem)` `0x1400E7580`: root must be `Error`; reads int attributes `code`, `server`, `module`, `line`, string `text`, and copies the child elements into a new `Extra` element.
- `ErrToXml(err, writer)` `0x1400E7AC0`: writes `<Error code server module line text>` and omits each attribute that is 0 or null; then the extra children (parse buffer 50000/50000). This answers the open root-name question of `Sts/02-messages.md` "Response metadata and errors": the root is `Error`.
- `CXNetErrorBuilder` vtable `0x1401585B8` (slots `0x1400E7450`, `0x1400E7490`, `0x1400E7500`, `0x1400E7560`, `0x1400E7570`, `0x1400E7420`).
- Module ids seen: 9001 = StsInetSocket framework errors (txn timeout 42 at line 2524 in `0x1400EC070`; code is written as a 64-bit value, so server = 0); 6000 = route client (`14-route-client.md` §5); 2 = HelloSrv service; 5 = AuthSrv service (`{3002, module 5, line 525}`). So each service has its own module id; later slices must record it.

### 3.2 Service request parse and reply (HelloSrv form)

1. OnMessage `0x1400C4320` calls the service parse `0x1400D01E0(req, msg, err)`. It does `ErrInit(err, 10)` (ErrBadParam), gets the command name (message vtable +24) and looks it up with `NameMapFind` in the service map. Not found: `req.id = 0`, err line 273, return 0. Found: `req.id`, `req.txn` = message vtable +40, then the body parse `0x1400DF6B0(req+8, 'f', msg)`.
2. Parse failed: reply error `{code = err.code (10), module 2, line 389}`.
3. Id with a `case`: call the handler. The handler replies by itself.
4. Id without a `case` (HelloSrv: 0 `Invalid`, 2 `Notice`): reply error `{52 ErrBadCommand, module 2, line 397}`.
5. The error `line` values go into the XML reply. For parity a port sends the same values (`quirk_error_line_numbers`: the client sees source line numbers of the original build).

### 3.3 Reply and send calls (route manager `qword_140216968`)

| Call | HelloSrv | Route manager slot | Use |
| --- | --- | --- | --- |
| `StsRouteReply(txn, err, msg)` | `0x140104D60` | +128 | reply to a request; `err` = record 3.1 or null; `msg` = body or null |
| `StsReplyErrorF(txn, err, fmt, …)` | `0x140104770` | +144 | reply error with a `printf` text |
| `StsRouteSend(msg, …)` | `0x140104D80` | — ❓ | send without reply (Notice) |
| `SendToProviderTxn(msg, txn, ms, flag)` | `0x1401049A0` | +88 | request with reply txn (2.5) |

## 4. DB call layer (AuthSrv; not framework)

- The proc builder is not in `framework-funcs.tsv`: only DB services link it. AuthSrv addresses:
  1. `MsgInit(msg, 0)` `0x140121170` (HelloSrv `0x1400D8160`).
  2. `DbProcBegin(writer, msg, db, schema, proc, flag)` `0x140112220`: `0x1401125E0(msg, flag?2:1, db)` sets protocol `Db.<db>` and command (`Exec` / `ExecBatched`, see `12-dbsrv-contract.md` §2), then writes `<Request><SchemaName/><ProcName/><Params>`.
  3. `ParamAdd` `0x140112570` (29 binaries), `ParamsEnd` `0x1401124D0` (33 binaries).
  4. Send: `SendToProviderTxn(msg, txn, 25000, 0)` `0x140157B00`, or `GridTxnSend(msg, txn, key, 0, 25000, 0)` `0x140135670` → `0x1401319F0` (creates a 0x58-byte `StsGridTxn` that wraps the service txn, then `0x1401582A0`).
  5. `MsgFree` `0x140121270` (HelloSrv `0x1400D8260`).
- Timeout of all AuthSrv DB calls seen: 25000 ms. Reply and timeout go through 2.5.
- AuthSrv: logical DB `AcctDb` (`off_1402322A8[0]`), schema `v2016102001` in all 77 calls.
- Service txn classes: `CAcTxn` vtable `0x1401A6050`, `CAuLoginTxn` `0x1401AA368`, `CLoadTxn` `0x1401AA1C0` (AuthSrv).

## 5. Other shared parts

### 5.1 Admin HTTP pages (`StsHttpCmd.cpp`, `StsHttpConn.cpp`)

- `StsHttpPagesInit` `0x14011F3A0` makes a pool (`0x1400DB470(4096, 0x4000)` → `qword_1402187C0`), then calls `StsHttpAddCommands(table, 20, 1)` `0x1400E6AC0`. Then it adds `off_1401C96C0` (`0x14011BB40`) and starts a message history with the filter `history=1000&exclude[conntypes]=300&exclude[conntypes]=405` (`0x140123870`).
- `StsHttpAddPage` `0x1400E66C0` (path, handler object, method, title, description, a6, a7, a8): allocates one record (`96` + string lengths). Record: +8 handler, +16 method/flag int, +24 title, +32 description, +40 a6, +44 a7, +88 count of path parts, +92 count of `:name` parameters. The path is split on `/`; a part that starts with `:` is a named parameter.
- Page table (the handler is a C++ object; slot 0 of its vtable does the work):

| Path | Handler vtable | Slot 0 | Output |
| --- | --- | --- | --- |
| `/` and `/favicon.ico` | `0x140164168` | `0x140120AB0` | redirect / icon ❓ |
| `/sts/about` (title `About`, "Basic information about the server") | `0x140164240` | `0x140120410` | info page (`conn_count`) |
| `/sts/about-commands`, `/sts/about-connections`, `/sts/about-log` | `0x140164288`, `0x1401642D0`, `0x140164320` | `0x140120410` | same function, other XSL |
| `/sts/commands` | `0x140164368` | `0x140120CB0` | XML list of commands (`all`) |
| `/sts/connections/:remote_addr/:pointer/disconnect` | `0x1401643B8` | `0x140120900` | closes the named connection |
| `/sts/log/:log_addr/:option` | `0x140164428` | `0x140120B50` | changes a log option |
| `/sts/counters` | `0x140164488` | `0x140120EF0` | XML + `../Web/Sts/counters.xsl` |
| `/sts/messages` (title `Messages`) | `0x140164520` | `0x140121110` | XML message history |
| `/sts/messages/trace-enable`, `trace-disable`, `clear` | `0x140164598`, `0x1401645B0`, `0x140164570` | `0x140121920`, `0x140121900`, `0x140121880` | then redirect to `../messages` |
| `/sts/messages/filter` (2 entries: read and set) | `0x1401645C8`, `0x140164618` | `0x140121940`, `0x140122670` | XML / `Bad Request` |
| `/sts/showheap` (title `Heap`) | `0x140164640` | `0x140122790` | heap use; needs `confirm` |
| `/sts/config-whitelist` (title `Config`, "Servers and privileged addresses") | `0x1401646E8` | `0x140120EF0` | XML + `config-whitelist.xsl` |
| `/sts/config-servers` | `0x1401647C8` | `0x140120EF0` | XML |
| `/web` | `0x140164138` | `0x140122990` | static files, `Web/Sts/index.html` |

- HTTP connection: `0x1400E6B70` writes the `Server` header `%S/%u.%u Instance/%u.%u`. `0x1400E6DD0` serves files: `Date`, `Last-Modified`, `if-modified-since` → `Not Modified`; no page → `File Not Found` / "Page not found". Read error log: `StsHttpConn: read error, %u, %S` (`0x1400E5B20`).
- `StsHttpCmd` `0x14011E810`: runs a console command from HTTP and writes `From: `, `To: `, `TraceMessage: %s` ❓ (exact use not checked).
- I did not find an address check on the HTTP pages (whitelist use) ❓. See `Docs/Plan/abuse-notes/st-00.md`.
- `StsIsPing` `0x14011B660`: true when the first 15 bytes of a received buffer are `POST /Sts/Ping ` and the buffer has 16 bytes or more. Callers `0x14011B6F0`, `0x14011B910`.
- Inet socket HTTP errors (`0x1400ECF50`, `0x1400EDA80`, `0x1400EDCE0`): `Out of order http response`, `StsInetSocket: non-http response to http client`, `... http response to non-http client`, `... unrequested response to http client`, `... request to http client`.

### 5.2 TLS (OpenSSL 1.0.x, static)

- `TlsReadCfg` `0x1400FCA90`, section `[Tls]`: `Method` (string; names `SSLv2`, `SSLv3`, `SSLv23`, `TLSv1`, `TLSv1.1`, `TLSv1.2`, `DTLSv1` at `0x14015A500`), `Certificate` (file name under `Certificate\`; value = `keyhere`), `CipherList` (string → `qword_1402168A0`, applied by `0x140127370`), `TmpRsaKeyLength` (default 1024), `TmpDhKeyLength` (default 1024).
- Port option `tls` (port line parse `0x1400FC5D0`, magic `stsporttlspp`): value 0 or 1, else log `StsPort: Tcpip tls on line %d arg %d is not in range [0..1]`.
- Route bus TLS: `StartTls` command (`0x1400D12C0`); logs `StsRoute: CStsRouteConn::DoRouterConnect, TLS negotiation started` (`0x1400FEC70`), `Start Tls handshake with %s` (`0x1401045D0`), `StsRoute; Client Tls handshake finished with %s` (`0x140101EB0`). See `14-route-client.md`.
- `ArenaSocketTls` (`0x140128090`) logs `invalid certificate`, `invalid private key`, `check_private_key failed`. Counter `socket tls error count (total)`.

### 5.3 Crypto helpers (`Services\Crypt`)

- `CptReadKeyCfg(ini, section, out)` `0x1400D2560`: keys `Certificate` (RSA key file in `Certificate\`), `KeyBits`, `EncryptedKey`. When all three are set, it loads the RSA key (`0x1400E40F0`), makes a `CptRsa` (`0x1400D60D0`) and decrypts `EncryptedKey`. Else it reads the plain `Key`. Returns true only when the key length is 32 bytes (AES-256). All values = `keyhere`.
- `CptAes` `0x1400D6370`: allocates the AES context; calls `0x1400D77F0` and `0x1400D7590` (key schedule ❓).
- Random bytes: Windows `CryptAcquireContext` / `CryptGenRandom` (`0x1400C2BA0`, `0x14011B5D0`, release `0x14014FA10`). Logs `Randomness: failed CryptGenRandom, errcode(%d), line(%d)` and three more.

### 5.4 Counters and monitor

- `StsServiceCounterCfg` `0x140106170`, section `[ServiceCounter]`: `UpdateInterval` (default 1, → obj +120), `Enabled` (default 0, → obj +124), plus per-counter keys `Enabled.<name>` (`IniGetValues`). Descriptions `service counter update interval`, `service counter enabled`.
- `StsServiceCounter.cpp`: `0x140105920`, `0x140105BD0`; XML elements `ServiceCounter`, `Counter`, `SuccessLatency` (`0x140106630`, `0x1401066D0`). Page `/sts/counters` (5.1).
- Counter log line `Counter; %s; %S` (`0x1400F7DC0`, called from `0x1400EFE10`).
- Config file monitor `0x1400F1740`: see 2.8.

## Service recipe

All snippets are in `recipe.py` (next to this chapter). Load: `exec(fn_fingerprint.py)`, `apply_names` with `framework-funcs.tsv` and `framework-names.tsv` (set `idc.SN_FORCE = ida_name.SN_FORCE` first on idalib 9.5), then `exec(open(".../RE/recipe.py").read())`.

| Function | Returns | HelloSrv | AuthSrv |
| --- | --- | --- | --- |
| `table()` | (map, id, name, handler) | 219 rows, 8 maps, 15 handlers | 855 rows, 41 maps, 66 handlers |
| `cfgkeys()` | (getter, call, func, args) | 47 calls | 69 calls (IniGetInt 33, IniGetStr 14, IniGetValues 10, NetCfgInt 6, NetCfgLookup 6) |
| `timers()` | (call, func, notify, ms, flag, OnTimer) | 44 calls, 7 with a fixed ms, 7 with a known OnTimer | 52 calls, 7 fixed ms (100, 10000, 15000, 30000), 10 known OnTimer |
| `dbprocs()` | (call, func, [writer, msg, db, schema, proc, flag]) | 0 | 77 calls, 46 proc names, 11 with a computed name |

- `table()` = `commands()` (switch near the `NameMapInit` hash user, HelloSrv form) plus `handlers()` (IDA switch tables whose case set is ≥ 80 % in the map and that are at most 3 calls from the parse function or from sibling vtable slots; AuthSrv form: parse `0x140110FC0` fills global record `0x1402831B0`, switch in `0x1400F3770`, 53 of 110 `Auth` ids). Ids without a handler use a default path ❓.
- `timers()`: ms −1 (0xFFFFFFFF) = remove; ms 0 = run now. OnTimer is found only for a global notify object.
- `cfgkeys()`: `NetCfgInt` args differ (section is computed; key = arg 2).

### Old command table snippet (now in `recipe.py`)

Load names first: `exec(open(WT+"/tools/fn_fingerprint.py").read())`, then `apply_names(WT+"/Docs/Platform/StServers/RE/framework-names.tsv")` (on idalib 9.5 set `idc.SN_FORCE = ida_name.SN_FORCE` first, because `idc` has no `SN_FORCE`).

Command table (`commands()` returns `(map, id, name, handler)`):

```python
def _arg(ea, reg):
    for _ in range(10):
        ea = idc.prev_head(ea)
        if idc.print_insn_mnem(ea) == "call": return None
        if idc.print_insn_mnem(ea) in ("lea", "mov") and idc.print_operand(ea, 0) == reg:
            return idc.get_operand_value(ea, 1)
def commands():
    init = idc.get_name_ea_simple("NameMapInit"); out = []
    for x in idautils.CodeRefsTo(init, 0):
        n, tb, hm = _arg(x, "edx"), _arg(x, "r8"), _arg(x, "rcx")
        if not (n and tb and hm): continue
        ids = {ida_bytes.get_qword(tb+16*i+8): idc.get_strlit_contents(ida_bytes.get_qword(tb+16*i)).decode() for i in range(n)}
        hand = {}
        for p in {ida_funcs.get_func(r).start_ea for r in idautils.DataRefsTo(hm) if ida_funcs.get_func(r)} - {ida_funcs.get_func(x).start_ea}:
            for c in idautils.CodeRefsTo(p, 0):
                try: txt = str(ida_hexrays.decompile(c))
                except Exception: continue
                for cid, h in re.findall(r"case (\d+):\s*\n\s*(?:[^\n]*?)(sub_[0-9A-F]+)\(", txt):
                    hand.setdefault(int(cid), h)
        out += [(hex(hm), i, nm, hand.get(i)) for i, nm in sorted(ids.items())]
    return out
```

Service maps are the maps that are not in the list of 2.3 step 2. This form alone finds only 7 handlers in AuthSrv; use `table()`.
