# 05 — RouteSrv, ForwardSrv, SvcDispatcher, CacheSrv, DbSrv audit (RE reference)

Summary:
1. RouteSrv keeps the directory of providers per protocol and routes each STS request to a provider by the `t` target (Direct, Key, Forward, Block).
2. ForwardSrv answers the `Forward` protocol (`Forward`, `Find`, `FindBulk`, `Set`, `Flush`, `FlushInternal`) that RouteSrv uses for `@scope` targets.
3. SvcDispatcher bridges STS requests to HTTP web hosts (`Dispatch` raw pass-through, any other command through the API table).
4. CacheSrv serves a key/value cache (`SetValue`, `GetValue`, `DeleteValue`). The caller can give a Db or Mongo data source to load a value.
5. DbSrv is ported; this chapter only audits it against `12-dbsrv-contract.md` (the Storage path is new).

Sources: the `.i64` copies in `ida/lanes/st/` (build 381). All addresses are in the binary of the section. The framework (`00-framework.md`) is not described again. Error records are `{code, module, line}`; code names are from `errcodes.tsv`. Long lists: `05-commands.tsv`, `05-procs.tsv`.

## Quick find

| Question | Function | Binary, address | Section |
| --- | --- | --- | --- |
| RouteSrv start (timers, config monitor) | `RteMainStart` | RouteSrv `0x1400DA2B0` | 1.1 |
| RouteSrv `[General]` keys | `RteReadGeneralCfg` | RouteSrv `0x1400DA490` | 1.1 |
| Route command switch (ids 1–8) | `RteDirManage::OnRequest` | RouteSrv `0x1400D0CD0` | 1.2 |
| Multi-router command switch (ids 15–20) | `RteDirManage::OnPaxosRequest` | RouteSrv `0x1400D01B0` | 1.4 |
| Target resolve (`*`, `$`, `@`, `#`) | `RteDirRouteTarget` | RouteSrv `0x1400CAB70` | 1.3 |
| Key hash of `$key` | `RteKeyHash` | RouteSrv `0x1400E3D50` | 1.3 |
| ForwardSrv command switch | `FwMain::OnRequest` | ForwardSrv `0x1400C6D10` | 2.2 |
| SvcDispatcher command entry | `SdHttpCliConn::OnRequest` | SvcDispatcher `0x1400D7C00` | 3.2 |
| SvcDispatcher STS → HTTP bridge | generic bridge | SvcDispatcher `0x1400D7160` | 3.2 |
| SvcDispatcher `Http.<name>` keys | `CSdHttpConnConfigNotify` | SvcDispatcher `0x1400D6E90` | 3.1 |
| CacheSrv new key / existing entry | `OnRequest` / `OnForward` | CacheSrv `0x1400C78D0` / `0x1400C7520` | 4.2 |
| CacheSrv data source load | — | CacheSrv `0x1400C68D0` | 4.2 |
| DbSrv Storage dispatch (not in `12-dbsrv-contract.md`) | — | DbSrv `0x1400C6480` | 5 |
| DbSrv Storage entry, token cache | — | DbSrv `0x1400C5CE0` | 5 |
| DbSrv `StorageOptions` keys | — | DbSrv `0x1400C5B40` | 5 |

## 1. RouteSrv

### 1.1 Role, start and config

RouteSrv is the hub of the STS route bus. Every StServers process opens one connection to it (framework 2.9 `RouteRegRetry`) and registers as a provider of its protocols. Callers (gates, services) send `POST /<Proto>/<Cmd>`; RouteSrv selects the provider. Source file name in asserts: `..\..\RteMain.cpp` (`0x1400DB7C0`).

Start `RteMainStart` `0x1400DA2B0`:
1. Sets the service name `"Route"` (`qword_1401EC560`).
2. Makes the directory object (`0x218` bytes, `sub_1400D9CE0`) → `qword_14023F338`, and the router object `qword_14023F330` (`sub_1400CDB00`).
3. Makes a 4096-entry table `qword_14023F348` (`sub_140105710(4096)`) ❓ use.
4. `StsConfigMonitorAdd` for `StServer.ini` and `Route.ini` (framework 2.8).
5. Timers (`j_StsTimerSet`): first timer at `StartupDelay*1000` ms, two timers at `StartupDelay*1000 + 1000` ms with OnTimer `0x1400DA9C0` (calls router vtable +0x10) and `0x1400DA9E0` (router vtable +0x8). These start the router after the start-up delay.

Config (`Route.ini`), reader `0x1400DA490` (called on each reload):

| Section | Key | Default | Use |
| --- | --- | --- | --- |
| General | StartupDelay | 0 (s) | `dword_1401EC2A8`; delay before the router starts (timers above) |
| General | LimboTimeout | router default (vtable +0x90 getter) | router vtable +0x20 setter; stored `dword_1401EC308` |
| General | ReconnectTimeout | router default (+0x98) | +0x28 setter; `dword_1401EC2C0` (spec 13: 30000) |
| General | LeftoverTimeout | router default (+0xA0) | +0x30 setter; `dword_1401EC2D8` (spec 13: 30000) |
| General | MultiRouteRequestIntervalMs | router default | +0x38 setter ❓ |
| Program | UseRemoteSourceBaked | 1 | `dword_14023F344`, passed to `sub_1400C6800` (`0x1400DA660`) |
| (network) | TlsEnabled | −1 | per network entry, `NetCfgInt` (`0x1400DAA00`); entries in a hash keyed with the table hash `dword_140176310` |
| TLS | Certificate, Key, EncryptedKey, KeyBits | — | `0x1400DABE0`; secrets → `keyhere` |

### 1.2 Route commands (map `0x140240B20`, ids 1–23)

Handler `RteDirManage::OnRequest` `0x1400D0CD0` (vtable slot +0x30 of the vtable at `0x140173DA8`). It switches on the command id (`msg+16`). Directory object = `this+8262*8`, service table = `this+8295*8`.

Special case before the switch: when `msg+92 == 101` and the id is 4 (RegisterProvider), it calls `RteLocalDirRegister` with the stored result `this+8232*8`, updates the time stamp `sub_1400CE300` (`timeGetTime`) and replies with the stored result ❓ (meaning of field +92 = 101 not verified).

| Id | Command | Handler | Behaviour | Errors (code, module, line) |
| --- | --- | --- | --- | --- |
| 1 | CheckRegistration | `0x1400D5750` | Checks that the caller's Provider, Protocol, Topology and Forwards are registered. Result 0 → error. | 3053 ErrRouteRegisterPending, 3, 799 |
| 2 | Snapshot | `0x1400D8230` | Logs the directory: `RteDirInfo; %s` with `Srv`, `SysCurrent`, `SysDesired`, `Key`, `Blks` | — |
| 3 | ShutdownServer | `0x1400D8050` | Finds the caller's server (`sub_1400D6010`), marks it, unregisters each provider (`sub_1400D72C0`), sends shutdown notices (`sub_1400D6B20`) | 3022 ErrServerNotFound, 5, 541 `RteLocalDir: shutdown, not connected` |
| 4 | RegisterProvider | `0x1400D8A00` | Needs a protocol. The Provider location must match the connection origin (`sub_14012E2B0`) and be in the local datacenter, or the protocol is `Route`. Topology ≠ Grid(3) → registers now (`sub_1400D86F0`). Grid → adds the provider, the block assignment is pending, reply 3053 | 10, 5, 346 `Register must have protocol`; 49, 5, 348 `Registering address inconsistent with origin`; 49, 5, 353 `Provider from remote datacenter`; 3053, ?, 372 |
| 5 | UnregisterProvider | `0x1400D8E20` | Same checks as 4; removes the provider (`sub_1400D61F0` find, `sub_1400D72C0` remove) | 10, 5, 383; 49, 5, 385; 49, 5, 387; 3053, ?, 398 |
| 6 | RegisterForward | `0x1400D09B0` | Empty protocol name (`msg+200 == 0`) → `sub_1400D7F00` (template forward). Else finds the service (`sub_1400DF970`), and for each `Forward` in the list matches the `Scope` name in the service forward list, then stores it (`sub_1400F9B70`) | 3029 ErrServiceNotFound, 3, 849; 3036 ErrFwdNotFound, 3, 873 |
| 7 | SubscribeService | `0x1400D8490` | Adds the caller to the subscriber list of the protocol (`sub_1400D7B10`, `sub_1400D5F20`); then sends `ProviderFound` for each current provider ❓ | 10, 5, 501 `subscribe, missing protocol` |
| 8 | SetBlocks | `0x1400D8C90` | Sets the Desired block map of a Grid protocol (`sub_1400D9220`); also called from `0x1400D6630` | 10, 5, 464 `set blocks, unregistered protocol` |
| 9–14, 21–23 | — | default | Not handled here: `RteDirManage: Unknown command, %s`. GetBlocks/Xfer (9–11) are routed to providers like normal requests (1.3). 12–14 are notices that RouteSrv sends. 21 RouterConnect goes on the router-to-router link ❓. 23 StartTls is framework. | 10, 3, 806 |

After the switch (ids 2–8) the handler logs `"%s; %s; %s"` (command name, peer address, protocol name `msg+120`), records an event (`sub_1400D97B0`), and builds the reply (`sub_1400D2640`).

Worked example (RegisterProvider of AuthSrv): request `POST /Route/RegisterProvider`, body mask 0x788: `Provider = "10.0.0.5:10100 1 2 3 4"`, `Name = Auth`, `BlockBits = 0`, `Build = 381`, `Topology = Basic(1)`. The location matches the socket origin and the center is local → `sub_1400D86F0` adds the provider and the reply is `200` with no error. The same request with a different IP → error 49, module 5, line 348.

### 1.3 Routing (verified against `13-routesrv.md` §5, §8)

`RteDirRouteTarget` `0x1400CAB70` resolves one `t` target (type in `target+0`). All errors are module 2.

| Type | Rule | Errors (code, line) |
| --- | --- | --- |
| 1 Direct `*loc` | Service topology must not be Extern(4) (`svc+88`). Parses the location (`sub_1400FA570`) and finds the provider (`sub_1400DF690`). | 10, 1573 `invalid direct address` |
| 3 Forward `c@scope:name` | Only for Grid/Extern (topology 3 or 4). Splits at `:`; finds the scope in the service forward list (string compare, length first). Scope `*` matches all. Scope `random` (6 chars) → random address (`sub_1400CA260`), then resolves again. A found forward rewrites the target to type 2 (Key) with the name after the scope (to `;`). Extern → `sub_1400CB580` (send to ForwardSrv). | 48, 1693 `error generating random address`; 10, 1723 `forward to unknown scope`; 10, 1650 `forward address to non-grid` |
| 2 Key `c$key`, 4 Block `c#hex` | Only for Grid(3) or Extern(4; → `sub_1400CB580`). Number = `RteTargetNumber` `0x1400F97D0`: Key → `RteKeyHash(key up to ';')`, hash 0 → `0x80000000`; Block → `strtoul(hex, 16)`. Block = number & ((2 << (BlockBits−1)) − 1), 0 when BlockBits = 0. Provider id = low 24 bits of Current[block] (`svc+160`); for command id 10 (RouteXferRequest) Desired[block] (`svc+184`) is used. | 10, 1605 `key or hash to non-grid`; 10, 1557 `unknown address type` |
| other | — | 10, 1533 (initial value of the result) |

`RteKeyHash` `0x1400E3D50` (via thunk `0x1400E48A0`), for the port:
```
h1 = 561029770 (i32), h2 = -490644323 (i32), r = 844963502 (u32)
for each byte b (stop at len or NUL):
    c = (i8)b; if 'a' <= c <= 'z': c -= 32         # case-insensitive
    h2 = c ^ (h2 * 8)
    h1 += T[h2 & 0xF]
    r ^= h1 + h2
return r                                           # len 0 → 844963502
T = 92B9A528 25D4FC88 EDCBEFB8 51063A80 91341C61 0261229D 726F48ED CE1C088C
    76253EB5 31E3A0DE A2AAD215 CA7D6D27 A5F98970 0541C365 3C14FF04 5056AF4F
```
- Quirk `quirk_route_keyhash_signed_byte`: `c` is a signed char, so a byte ≥ 0x80 gives a negative value (sign extension) before the XOR. A UTF-8 key hashes with these negative values.
- Quirk `quirk_route_keyhash_zero`: a key whose hash is 0 is mapped to `0x80000000`, so it falls in block 0 of every mask.

### 1.4 Multi-router (map `0x140240D68`, ids 257–1030, and ids 15–20)

`RteDirManage::OnPaxosRequest` `0x1400D01B0` (vtable `0x140173EA0` slot +0x30, class with `'Paxos'` in slot +0x28 `0x1400D2470`) parses the body (`0x1400FA0B0`), checks the origin server (`RteDirManage: invalid origin server, %s`) and switches:

| Id | Command | Handler (`this+66624*8` = directory) |
| --- | --- | --- |
| 15 | ServerShutdown | `0x1400E16A0` |
| 16 | ServerDisconnect | `0x1400E1330` |
| 17 | ProviderChanged | `0x1400E0C40` |
| 18 | ProviderShutdown | `0x1400E1140` |
| 19 | ServiceSetForward | `0x1400E1830` |
| 20 | MultiRouterChanged | `0x1400CFEB0` (parses the `SubCmd` body again) |
| other | — | `RteDirManage: unknown request, %s` / `invalid request` |

Other slots: `0x1400D1CB0` (`Desired @ New Leader`), `0x1400D18B0` (`Data changed`). The Paxos ids (Alive 257, Suspect 258, Accuse 259, Request 513, Reply 514, Set 515, Electors 516, Prepare 1025, Promise 1026, Accept 1027, Accepted 1028, Sync 1029, Committed 1030) are handled in the shared Paxos code (`0x14010F230` uses the map) ❓ not traced.

### 1.5 State, timers, calls out

| Item | Detail |
| --- | --- |
| Directory | `qword_14023F338`, 0x218 bytes; services at `+66360` (spec 13), providers by 24-bit id (`sub_1400DF6A0`) |
| Router | `qword_14023F330`; timeouts above |
| Timers | `0x1400DA2B0` (start), `0x1400CE300` (`timeGetTime` stamp, 0 ms re-arm), `0x1400CBFC0` (remove, −1), `0x140139690` (remove) |
| Calls out | `ProviderFound`/`ProviderLost`/`BlocksFound` notices to subscribers; `RouteXferRequest`/`RouteXferBlock` to Grid providers; `Forward` to ForwardSrv for Extern services (`sub_1400CB580`, framework `0x1400F7BC0`); ids 15–20 to the other routers |
| DB procs | none |

## 2. ForwardSrv

### 2.1 Role and config

ForwardSrv resolves a name in a "forward scope" to a key (a name → id/datacenter map). RouteSrv sends `@scope:name` targets of Extern services to it (1.3). Source file: `..\..\FwEntry.cpp`. One process serves one forward database: global `qword_1402225B0` (database name), entry table `qword_140222510`, current command id `dword_1402225A8` (set by the parse, `0x1400C7030`). Admin pages `Web/ForwardSrv/tables.xsl` (`0x1400C40C0`), `tables_0.xsl` (`0x1400C40B0`); notify classes `CFwTablesNotify`, `CFwTables0Notify`, `CFwFlushNotify`, `CFwFlushByKeyNotify`, `CFwEntryForwardIni`.

| Section | Key | Default | Use (address) |
| --- | --- | --- | --- |
| Cache | MemoryLimitKB | 102400 | `0x1400C5510`; 0 → `FwEntry: invalid memory limit (0KB)`; too high → `FwEntry: memory limit too high, %I64u` |
| (Forward entry XML) | `Forward` → `Database`, `Key`, `NoCache` | NoCache = off | `0x1400C53A0` reads one entry; `0x1400C7190` writes it |

### 2.2 Commands (map `0x140223C30`)

Dispatch `FwMain::OnRequest` `0x1400C6D10` (vtable data `0x14015C748`). First it parses the body against the database name (`0x1400C7030`); a parse failure → `FwMain: invalid database, %s` (10, 1, 408). Errors are module 1.

| Id | Command | Handler | Behaviour | Errors (code, module, line) |
| --- | --- | --- | --- | --- |
| 1 | Forward | `0x1400C4DC0` / `0x1400C5FE0` | Same as Find. A request that RouteSrv forwarded keeps the original protocol and command in the `r` (refer-to) header (`0x1400C6750` path). | 10, 1, 467 `FwMain: missing refer-to protocol and/or command`; 3010, 1, 472 |
| 2 | Find | `0x1400C4DC0` | Entry in cache → reply (`0x1400C5FE0`, `StsRouteReply`). Miss → a DB call (`DbProcBegin` `0x1400DB2F0`, `Request/SchemaName/ProcName/Params`): params = the name (`*entry`) and, when set, the extra param `table+136`. When the database name starts with `$`, the DB request gets a `t` header `$<name>` (key route). Waits up to 25000 ms (`sub_1400F81C0(..., 0x7FFFFFFF, 0, 25000, 0)`). The new 0x48-byte entry (`0x1400C4650`) is cached unless `table+128` (NoCache) is set. | 3010 ErrKeyNotFound, 1, 417 (no entry) |
| 3 | FindBulk | `0x1400C4FF0` | Body `Name` list. Checks names. | `missing names`, `empty name`, `missing name`, `missing database` (lines 650–687; code 10, module 2) |
| 4 | Set | `0x1400C5EF0` | Body needs `Id` and `Center` (`0x1400C4BB0`: id required, center 0–63). Stores the entry and replies. | 10, 1, 152 `FwMain: set, body missing id and/or datacenter`; `FwEntry: create, missing id` (623), `invalid datacenter` (629) |
| 5 | Flush | inline | Removes the named entry (`0x1400C7E40`), then sends `FlushInternal` (`0x1400DB6E0`) ❓ to the other ForwardSrv processes | — |
| 6 | FlushInternal | inline | Not accepted from callers | 51 ErrNotImplemented, 1, 448 |
| other | — | — | `FwMain: unknown forward command %s` | 51, 1, ? |

Pending finds: when a find is in progress, more requests are queued (`qword_1402224E0`, count doubles `dword_1402224F0 *= 2`); `0x1400C6B00` replies to all queued requests when the DB reply arrives.

Worked example (Find): `POST /Forward/Find`, body `<Name>Alice</Name>` for database `$NameDb`. Cache miss → DB request with `t: $Alice`, param `Alice`; the reply row gives `Id` and `Center`; the reply to the caller carries them. Unknown name → 3010.

### 2.3 DB procs, state, calls out

| Item | Detail |
| --- | --- |
| DB proc | one call site `0x1400C4EAD` (in `0x1400C4DC0`); the proc name is in the entry table at `+144` ❓ (set from config, not traced) |
| Cache | entries 0x48 bytes, limit `MemoryLimitKB` |
| Timer | `0x140140D60` (remove, −1) ❓ |
| Calls out | `Db.<Database>/Exec` to DbSrv; `FlushInternal` to peers ❓ |

## 3. SvcDispatcher

### 3.1 Role and config

SvcDispatcher is an STS → HTTP bridge. It accepts STS requests from other services (any protocol that the config gives it), converts each request to an HTTP request and sends it to one of a list of web hosts (round robin). It converts the HTTP reply back to an STS reply. Source files: `SdHttpCliConn.cpp`, `SdHttpSrvConn.cpp`, `SdApiManage.cpp`. In "API manage" mode it also gets the command → URI table from an `ApiManage` provider.

| Item | Detail (address) |
| --- | --- |
| Service maps | `Dispatch` map `0x14025CD40` (1 row: Dispatch = 1, init `0x1400D0F20`, lookup `0x1400ED880`); SSL method map `0x14025CF70` (SSLv2 0, SSLv3 1, SSLv23 2, TLSv1 3, TLSv1.1 4, TLSv1.2 5, DTLSv1 6) |
| App name | `CStServerConfigNotify::ParseConfigName` `0x1400D4EF0` (`missing app config name`, `app config name loaded as %S`), stored at `off_1401EA148` / `qword_1401EA150` |
| Admin page | `Web/SvcDispatcher/service.xsl` (`0x1400D51E0`); page data `0x1400D7CE0` (`Service`, `ServiceName`, content type `json`/`xml`/`invalid`) |
| Pending-request timer | `0x1400D7160` line 138 `StsTimerSet(off_1401EA230, dword_1401EA214 = 25000 ms, 1)`; OnTimer `0x1400D6790` replies to expired requests (`SdHttpCliConn:: parsing qmsg error`, line 499) |
| Counters | `dword_14024B538` = requests sent (incremented `0x1400D5800`, `0x1400D7160`) |

Config keys. Section `Service.<app>` (`0x1400D4C10`; missing → log `missing section %s`):

| Key | Default | Use |
| --- | --- | --- |
| Mode | — | `0x1400D4AB0` → 0, 1 or 2 (string values not read ❓); stored `a1[2]` |
| Protocol | — | Protocol name that the service registers (copied to `qword_14023B430`) |
| ServicePrefix | 1 | `a1[21]` |
| ApiManage | 0 | `a1[22]`; 1 = get the API table from the `ApiManage` provider (3.3) |

Then `DelegateParsingByProtocol` `0x1400D4B20` (`invalid app ...`) and `0x1400D4FA0` (registers the service). Section `Http.<name>` (`CSdHttpConnConfigNotify` `0x1400D6E90`; missing → `missing configuration section '%s'`):

| Key | Default | Use |
| --- | --- | --- |
| ContentType | — | `json` → `dword_14024B508 = 1` (`Content-Type: application/json`), `xml` → 2 (`application/xml`); other → `invalid content type for '%s'` |
| ForwardFrom | 0 | `dword_14024B50C`; 1 = add header `X-Forwarded-From` with the caller address (`0x1400D5800` line 110) |
| Https | 0 | `dword_14024B514`; passed to the HTTP client open `0x140105280` |
| Proxy | empty | `byte_14024B540` (255 chars) |
| SslMethod | empty | `byte_14024B518` (32 chars), a name of the SSL method map |
| MaxConnections | 16 | `dword_1401EA208` |
| IdleTimeoutSec | 10 | `dword_1401EA20C` |
| RequestTimeoutSec | 25 | `dword_1401EA210` |
| ProgramId, DisableTrace | 0 | read, local use |
| Host | — | `ParseHosts` `0x1400D6950`: host list `qword_14024B4E0` (24-byte rows, count `HIDWORD(qword_14024B4E8)`); none → `ParseHosts, SvcDispatcher.ini: missing host config` (line 401). Separator ❓ |

### 3.2 Commands

Entry `SdHttpCliConn::OnRequest` `0x1400D7C00` (vtable slot; no `table()` row). It reads the command name (`V[a1+8]`):

| Command | Handler | Behaviour |
| --- | --- | --- |
| `Update` | `0x1400D79A0` | Notify from the `ApiManage` provider. Reads `Object`, `ApiMainVersion`, `ApiHotfixVersion`; calls `0x1400D44C0`, which starts a new `ListProviderHttpApis` (3.3) when the versions differ from `dword_14023B404/408`. Replies `Updated`. |
| `Dispatch` (map id 1) | `0x1400D5800` | Raw HTTP pass-through. The request body is an HTTP request message (parse via `qword_14024B4F8`); parse failure → `parse request msg failed: %s` and `SdHttpCliConn: missing or invalid http request msg`. Sets `Host` to the next host; adds `X-Forwarded-From` when ForwardFrom = 1; sends. |
| any other | `0x1400D7160` | Generic bridge (below). |
| empty name | `0x1400D7C00` | `ReplyError(..., "SdHttpCliConn: missing command")`, line 966 (code ❓) |

Generic bridge `0x1400D7160`, step by step:
1. Read protocol (`V[a1+8]`), version (`V[a1+16]`), command (`V[a1+24]`), body (`V[a1+40]`). Protocol or command empty → 10 ErrBadParam, module 5, line 342 `missing protocol or command`.
2. Find the URI: `0x1400D4510` builds the alias key `/%s/%u/%s` (protocol, version, command) or `/%s/%s` when version is 0, and looks it up in the API table (`dword_1401EA0D0` rows). No alias → line 350 `missing protocol or command alias`; empty URI → line 395 `missing uri/query parameter`. With no API table (mode without ApiManage), the URI is `/<protocol>/<command>` (lines 100–113 ❓).
3. Copy STS headers `f`, `r`, `m`, `z`, `v` (byte names 102, 114, 109, 122, 118) to the HTTP request (`0x1400F36B0`).
4. Pick a host: `dword_14024B510 + 1`, wrap to 0 at the count. No host → 3030 ErrBadConfig, module 5, line 385 `host is not configured`.
5. Set `Host`; when there is a body, set `Content-Type` (json or xml) and the body; another mode → 3030, module 5, line 412 `missing or invalid content type`.
6. Open an HTTP client (`0x140105280`, host, Https, Proxy, SslMethod); failure → 3033 ErrConnFailed, module 5, line 429 `get client manager failed`. Send with a 0x1698-byte request record (`0x1400D52B0`); `++dword_14024B538`.
7. When the HTTP reply comes, the record replies to the STS caller (`StsRouteReply`). Timeout: the 25000 ms timer.

Worked example: config `ContentType=json`, `Host=web1`, API row `/Account/1/Get` → `/api/account/get`. A request `Account/Get`, version 1, body `{"id":5}` → `POST /api/account/get` to `web1`, `Content-Type: application/json`, body `{"id":5}`. The HTTP status and body return to the caller.

Inbound HTTP (`SdHttpSrvConn` `0x1400D8900`, `0x1400D83E0`): the HTTP server side accepts `xml` or `json` bodies (`request xml body is empty`, `failed parse json body`, `request json body is empty`, `not supported content type`) and passes them in as STS requests ❓ (target not traced).

### 3.3 API manage, state, calls out

| Item | Detail |
| --- | --- |
| Subscribe | `0x1400D3F70`: when ApiManage = 1 and not subscribed (`dword_14023B40C`), sends `ApiManage/SubscribeTrusted` (`0x1400D3B80`, body `Request/Location/Api`, location GUID `11111111-1111-1111-1111-111111111111`, wait 25000 ms); on stop sends `ApiManage/Unsubscribe` (`0x1400D3CE0`). Failure log `SdApiManage:: %ssubscribe failed` (`0x1400D4090`). |
| Table load | `0x1400D3A60`: `ApiManage/ListProviderHttpApis` (body `Request/ProviderName/ProviderVersion`, wait 25000 ms). Reply parse `0x1400D40C0`: `Apis/Api` rows with `ProviderName`, `ProviderVersion`, `Command`, `TargetUri`; key `/%s/%u/%s`; new 0x88-byte row per API. `dword_14023B410` = load in progress. |
| Host list | round robin index `dword_14024B510` |
| Calls out | HTTP to the configured hosts; `ApiManage` STS calls above |
| DB procs | none |

## 4. CacheSrv

### 4.1 Role and config

CacheSrv keeps values per `group` + `key` in memory. A value comes from the caller (`SetValue`) or from a "data source" (a DbSrv proc or a Mongo command) that the caller describes in the request. Source files: `CacheMain`, `CacheEntry.cpp`, `CacheTxn.cpp`, `CacheConsole.cpp`. Start `0x1400C74C0` (`StsSetServerType`, protocol `Cache`). Admin pages: `Web/CacheSrv/check.xsl` (`0x1400C43C0`), `clients.xsl` (`0x1400C43E0`); console `0x1400C4560` (params `group`, `key`; `cache not found`), console set `0x1400C5120` (params `Key`, `Value`; `missing param`; sends `Cache/SetValue` via `0x1400DC340`).

| Section | Key | Default | Use (address) |
| --- | --- | --- | --- |
| Cache | MemoryLimitKB | 102400 | `0x1400C6CE0`; ×1024 → `qword_1401D4050`; 0 → `CacheEntry: invalid memory limit (0KB)`; above the system limit `qword_1402235D0` → `memory limit too high, %I64u (system allowed %I64u)` |

### 4.2 Commands

Maps: `Cache` map `0x140224C70` (init `0x1400C15B0`): SetValue 1, GetValue 2, DeleteValue 3. Client-side maps (used when CacheSrv calls out): `0x140224C90` Db (Invalid 0, Exec 1, ExecBatched 2); `0x140224CB0` Mongo (Invalid 0, Count 1, Eval 2, EvalJson 3, Find 4, FindJson 5, FindOne 6, FindOneJson 7, Insert 8, InsertBatched 9, MapReduce 10, MapReduceJson 11, Remove 12, Update 13; name lookup `0x1400DC960`).

Handler vtable at `0x14015DAC8`: slot 0 `OnParse` `0x1400C7800`, slot 1 `OnRequest` `0x1400C78D0` (new key), slot 2 `OnForward` `0x1400C7520` (key that has an entry). The labels are mine.

Parse `Srv2Cache::Parse` `0x1400DC480` fills the global request `dword_1402235E0`:

| Offset (from `0x1402235E0`) | Field | Source |
| --- | --- | --- |
| +0 | command id 1–3 | command name |
| +4 | request id | reply-to header |
| +8 (17 chars, `byte_1402235E8`) | group: first the `%u` number from the reply-to info, then body `Group` when present | header / body `Group` |
| +32 | key (`qword_140223600`, the second `%s` in the error texts ❓) | `to` header |
| +40 / +48 | `Value` (pointer, length `dword_140223610`) | body `Value` |
| +48 | `Expiration` | body `Expiration` (int) |
| +56 | `DataSource` element (`qword_140223618`) | body `DataSource` |

Parse errors (text only, then `CacheMain: invalid message, parser line %u`): `invalid to headers`, `invalid reply to info`, `invalid reply to header type`, `invalid reply to header protocol info`, `invalid reply to headers`, `body not exists`.

| Id | Command | New key (`0x1400C78D0`) | Existing entry (`0x1400C7520`) |
| --- | --- | --- | --- |
| 1 | SetValue | Value given → new entry (`0x1400C65B0`: 0xF0 bytes, group, key, value copy, expiration) → reply `Reply/Value` (`0x1400C7BC0`). No value but DataSource → load (below). Neither → 10, 1, 198 `missing value and data source parameter`. | Value given → entry set (`V[entry+48]`); fail → 10, 1, 250 `invalid value`. DataSource → reload; bad → 10, 1, 265. Neither → 10, 1, 280. |
| 2 | GetValue | No DataSource → reply 3010 ErrKeyNotFound. DataSource → new entry (`0x1400C6540`), load from the data source, reply when the load ends. Bad data source → 10, 1, 181 `invalid data source parameter`. | Reply with the cached value. |
| 3 | DeleteValue | Reply empty (nothing to remove). | `0x1400F8CA0` removes the entry, reply empty. |
| other | — | 51 ErrNotImplemented, 1, 232 `unknown cache command %s` | 51, 1, 316 `unknown forward command %s` |

Data source load `0x1400C68D0`. The `DataSource` element (parse `0x1400C6EB0`): `Type` (`Db` or `Mongo`), `DbName`, `ProcName`, `Command`, `FlushOnFailure` (int), `Params` (list of name/value, 48-byte rows). Serialize back: `0x1400C7370` (the framework name `DbProcBegin` on it is wrong in this binary).

| Type | Request | Errors (code 10) |
| --- | --- | --- |
| Db | `0x1400DC870(msg, 2 = ExecBatched, DbName)`, body `Request/ProcName/Params` | line 385 `missing data source procedure name` |
| Mongo | `0x1400DCAE0(msg, <Command id>, DbName)`, body `Request` + params | line 412 `missing data source command`, 418 `invalid data source command` |
| other | — | line 439 `invalid data source type`; no source → line 379 `data source not exits` |

Each load waits up to 25000 ms (`sub_1400F8FF0(..., 0x7FFFFFFF, 0, 25000, 0)`) with a `CacheTxn` (`0x1400C7CA0`, type 0 = Db, 1 = Mongo, FlushOnFailure). The reply handler `0x1400C8010` (`invalid txn type`) stores the value (`0x1400C80E0`, `Reply/Value`); Mongo replies use `_id` (`0x1400C7F70`). With FlushOnFailure = 1 a failed load removes the entry ❓.

Worked example: `Cache/GetValue`, to `PlayerRank`, key `42`, body `<DataSource Type="Db" DbName="GameDb" ProcName="usp_GetRank"><Params>..</Params></DataSource>` (attribute or element form ❓). No entry → `Db.GameDb/ExecBatched usp_GetRank` → the result is cached in a 0xF0-byte entry → reply `Reply/Value`. The next GetValue for the same key replies from memory.

### 4.3 State, procs, calls out

| Item | Detail |
| --- | --- |
| Memory | `qword_1402235C8` = bytes in use; `0x1400C65B0` adds each entry size and calls `0x1400F8E50` (removes the oldest entry) while use > limit; `dword_1401D4070` = use in KB |
| Timers | no service timer (`StsTimerSet` has no caller in the service functions); `Expiration` handling `0x1400C6E20` ❓ |
| DB procs | no fixed proc: the caller gives `DbName` + `ProcName` (call site `0x1400C68D0`) |
| Calls out | `Db.<DbName>/ExecBatched`, `Mongo.<DbName>/<Command>` (target name format ❓), `Cache/SetValue` from the console |

## 5. DbSrv audit

`12-dbsrv-contract.md` covers the `Db.<Logical>` path (`Exec`, `ExecBatched`). It names the `Storage` path (§2a row 2, §5 "StorageSrv separate exe") but does not describe it. In this build the Storage path is inside DbSrv.

| Item | Detail (DbSrv address) |
| --- | --- |
| Storage config | `0x1400C6400`: reads `Storage.ini`, section `Storage.Db` (`StsConfigMonitorAdd`, watched) |
| Storage map | `0x140221128` (init `0x1400C1660`, rows `0x14015BA20`): Create 1, Exec 2, ExecBatched 3, GetInfo 4, Read 5, Write 6 |
| Token map | `0x140221148` (init `0x1400C1750`, rows `0x14015C160`): RequestToken 1, ValidateToken 2, RevokeTokens 3, RevokeTokensInternal 4, ExtendToken 5 (client side: DbSrv calls the `Token` service) |
| Storage dispatch | `0x1400C6480` (callers `0x1400C5CE0`, `0x1400D28B0`): storage disabled → 3030 ErrBadConfig lines 444 / 446 `disabled storage`; id 0 → 10 ErrBadParam line 463 `unknown command (%s)`; line 465 (other unknown) |
| 1 Create | `0x1400C43B0`: builds an id (`%s`, `CoCreateGuid` `0x1400C8850`), calls `Token/RequestToken` (`0x1400D24B0`, `Token.Resource`, fields `id`, `random`); error lines 250, 273 |
| 2, 3 Exec, ExecBatched | `0x1400C45E0`: body `Token` required (line 415, 422); `Token/ValidateToken` (`0x1400D2630`, `Token.Resource`); then the DB exec ❓ |
| 4 GetInfo | `0x1400C49C0`: `Token` check, ValidateToken; reply `Reply/Scope` (entry +72); error lines 281, 288, 306 |
| 5 Read | `0x1400C4E40`: `Token` check (314, 321), ValidateToken, `DbProcBegin(StorageDb, GetStorageByKey, 1 param)`; error line 348 |
| 6 Write | `0x1400C5390`: `Token` + `Data` (361, 368), ValidateToken, `DbProcBegin(StorageDb, CreateStorage, 0)`; error lines 396, 402 |
| Token reply | `0x1400C61F0`: reply `Reply/Token/TokenId` (lines 159–175) |
| DbMain | `0x1400D2740`: `%S has no configuration (database)` → `ExeFatal`; section name `%s.%S` |
| Constant | `844963502` (`0x325D1EAE`) is the hash seed of the Storage token cache (`0x1400C5CE0` line 113, bucket = `qword_1401D1028 + 24 * (hash & dword_1401D1038)`). The same value is set before each ValidateToken (`0x1400C45E0` line 90) |
| Storage options | `0x1400C5B40`: `Storage` key (name from `0x1400EFCA0`, global wide string `word_140221740` ❓) enables storage (flag `dword_1402202B4`); `StorageOptions/ValidateTokenInterval` seconds, default 60, stored ×1000 as ms in `dword_1401D1000`; `StorageOptions/TokenEffectiveUntilSecs`, default 86400 |
| Storage entry | `0x1400C5CE0` (StorageTxn.cpp): no body → error (source line 201); `IssuerAppId` must parse as a GUID (`0x1400DF7B0`) and be non-zero, `ResourceKey` required (line 214). It looks up the token cache by hash; on miss it creates a 0x60-byte entry, parses the token (line 222 on failure), inserts it (`0x14013AD40`) and starts a timer of `ValidateTokenInterval` ms (`StsTimerSet`). Then it calls Storage dispatch `0x1400C6480` |
| Aux error info | `0x1400D2E30`: copies the `<Response type="array">…</Response>` part of an error reply as auxiliary info. Logs `Too big auxillary error info for (%d:%d:%d:%d), ignored` or `Not completed auxillary error info …`. It keeps the text only when it is longer than 0x3B bytes or is not an empty `<Set type="array"></Set>` |
| ODBC timeouts | `0x1400E50F0`: `SQL_ATTR_LOGIN_TIMEOUT` (103) = 10 s, `SQL_ATTR_CONNECTION_TIMEOUT` (113) = 10 s, then `SQLDriverConnect`. `0x1400E5820` sets the query timeout per transaction |
| Named parameters | `0x1400E6720`: `@%s` name set with `SQLSetDescField(1011 = SQL_DESC_NAME)` |
| Conn string check | `0x1400D44C0`: logs `DbTrans: semicolons not allowed in %S string` for `Database`, `Driver`, `Address`, `Mirror` |
| Mongo map | `0x140221110` (init `0x1400C1690`, rows `0x14015B930`): Count 1, Create 2, Eval 3, Find 4, FindOne 5, GetInfo 6, Insert 7, InsertBatched 8, Read 9, Remove 10, Update 11, Write 12 |
| Bucket map | `0x1402210F8` (init `0x1400C16C0`, rows `0x14015BD90`): Invalid 0, AddBucketAclEntity 1, DeleteBucketAclEntity 2, UpdateBucketAclEntity 3, AddBucket 4, DeleteBucket 5, ListBuckets 6, GetBucket 7, UpdateBucket 8, AddObjectAclEntity 9, DeleteObjectAclEntity 10, UpdateObjectAclEntity 11, AddObject 12, DeleteObject 13, ListObjects 14, GetObject 15, UpdateObject 16, GetObjects 17 |
| ACL scope map | `0x1402210D8` (init `0x1400C16F0`, rows `0x14015BBE0`): Invalid 0, AppId 1, UserId 2, GameAccountId 3, AppGroupId 4, Authenticated 5, Everyone 6 |
| ACL role map | `0x1402210C0` (init `0x1400C1720`, rows `0x14015BB10`): Invalid 0, Owner 1, Editor 2, Reader 3 |

The four maps above have only the init and `atexit` xrefs. No DbSrv code reads them. DbSrv does not serve Mongo or Bucket commands.

Audit scope: 191 own functions (`shared_with` = 0). 94 refer to strings; 97 have no strings. Of the 94, the OpenSSL (`crypto\`, `ssl\`), RapidXml (`expected …`) and Windows platform functions are library code. Spec 12 covers DbConn (`0x1400D28B0`), DbTrans (`0x1400D3440`–`0x1400D4E10`) and SqlConn. The rows above are what spec 12 does not cover. The 97 functions without strings are not read one by one ❓.

DB procs of DbSrv itself: `StorageDb.GetStorageByKey` (`0x1400C4E40`), `StorageDb.CreateStorage` (`0x1400C5390`).

Lists: all commands in `05-commands.tsv`, all procs in `05-procs.tsv` (same folder).

## Framework gaps

| Fact | Binary, address |
| --- | --- |
| The framework name `DbProcBegin` is wrong on CacheSrv `0x1400C7370`. That function serializes a `DataSource` element. Do not use the name there. | CacheSrv `0x1400C7370` |
| In DbSrv, Hex-Rays does not show the arguments of `NameMapInit` (map, row count, rows). Read them with `dis(<init>)`: `rcx` = map, `edx` = row count, `r8` = rows (16-byte rows: name pointer, id). | DbSrv `0x1400C1660`–`0x1400C1750` |
| `RteKeyHash` (`0x1400E3D50`) start value `844963502` is also the hash seed of other service hash tables (DbSrv Storage token cache). | RouteSrv `0x1400E3D50`, DbSrv `0x1400C5CE0` |
| The pending-request wait helper takes `(…, 0x7FFFFFFF, 0, 25000, 0)`: 25000 ms timeout in ForwardSrv, CacheSrv and SvcDispatcher. | ForwardSrv `0x1400C4DC0`, CacheSrv `0x1400C68D0`, SvcDispatcher `0x1400D3A60` |
| RouteSrv uses a second request class for router-to-router traffic (`RteDirManage::OnPaxosRequest`, vtable `0x140173EA0`), with the `'Paxos'` name in slot +0x28. | RouteSrv `0x1400D01B0` |

## Differences from existing specs

| Spec | Spec says | Binary shows |
| --- | --- | --- |
| `13-routesrv.md` line 20 | RouteSrv errors use module 2 | Routing errors (`0x1400CAB70`) use module 2. The `RteDirManage` command errors use module 5 (local directory, `0x1400D8A00`, `0x1400D8E20`, `0x1400D8490`, `0x1400D8C90`, `0x1400D8050`) and module 3 (`0x1400D5750`, `0x1400D09B0`, default line 806) (§1.2) |
| `13-routesrv.md`, README | Key hash ❓ | Resolved: `RteKeyHash` `0x1400E3D50`, algorithm and table in §1.3, quirks `quirk_route_keyhash_signed_byte`, `quirk_route_keyhash_zero` |
| `13-routesrv.md` §7 line 142 | Forward scope is a phase-2 gap | Described in §1.3: scope match `*`, `random`, rewrite to Key target, Extern → ForwardSrv (`0x1400CB580`) |
| `13-routesrv.md` §7 line 141 | Multi-router is a phase-1 gap | Ids 15–20 handlers and the Paxos ids 257–1030 (§1.4); Paxos body not traced ❓ |
| `13-routesrv.md` | Does not give the RegisterProvider origin check | The Provider location must match the socket origin and be in the local datacenter (49, module 5, lines 348, 353), except protocol `Route` (`0x1400D8A00`) |
| `12-dbsrv-contract.md` §5 | StorageSrv is a separate exe | DbSrv has the full Storage path (`0x1400C6480`, commands 1–6, `Token` client calls, `StorageDb` procs) (§5) |
| `12-dbsrv-contract.md` | No `StorageOptions` section | `StorageOptions/ValidateTokenInterval` (60 s) and `TokenEffectiveUntilSecs` (86400) (`0x1400C5B40`) |
| `12-dbsrv-contract.md` | No ODBC timeouts | Login and connect timeouts are fixed at 10 s (`0x1400E50F0`) |
| `12-dbsrv-contract.md` lines 103–105 | Does not give a character check for the connection keys | `;` in `Database`, `Driver`, `Address`, `Mirror` is logged as an error (`0x1400D44C0`) |
