# 04 — GameGate, AppGate, MwGate, WebGate (RE reference)

Summary:
1. The four programs are gates. They link the gate library (`01-gate-cligate-sessiongate.md` §1) and the framework (`00-framework.md`). This chapter gives only the service code.
2. GameGate (`GgMain.cpp`, `GgCli.cpp`, `GgSrv.cpp`) connects LobbyGate and GameDaemon (TCP 6601) to the route bus. It handles 18 commands locally. It sends all other commands through `StsGateSendTxn` after the `Gate.ini [Commands.GameGate]` check.
3. GameGate also has a bus side (`GgSrv`): the daemon registers custom protocols (`Srv2World`, `Srv2Game*`), and the gate sends bus requests for these protocols to the daemon connection.
4. AppGate (HTTP 6602) and WebGate (HTTP) change HTTP requests into bus requests. AppGate and MwGate also accept extern services (GP proxies) that register `Srv2Gp*` protocols.
5. Command rows: `04-commands.tsv`. Proc rows: `04-procs.tsv` (the gates call no DbSrv proc directly).

## Quick find

| Question | Function | Binary, address | Section |
| --- | --- | --- | --- |
| GameGate start | `GgMain` start | GameGate `0x1400D7E20` | 1.1 |
| GameGate `Game.ini [Configurations]` | notify | GameGate `0x1400D7F30` | 1.1 |
| Daemon request dispatch | `CGgCliConn` slot 0 | GameGate `0x1400D5F90` | 1.2 |
| Daemon login (`Presence/Login`) | handler | GameGate `0x1400CFA20` | 1.3 |
| Login chain (IpCheck, Access, Presence) | callbacks | GameGate `0x1400D3920` … `0x1400D2600` | 1.3 |
| Default forward reply to daemon | `CGgCliDataTxn<void>` callback | GameGate `0x1400CEE20` | 1.2 |
| Bus → GameGate dispatch | bus OnMessage | GameGate `0x1400D8170` | 1.4 |
| Custom protocol route (Srv2World …) | parser | GameGate `0x1400D8850` | 1.4 |
| Bus → daemon forward, `UserIdByName` | `GgSrv` exec | GameGate `0x1400D8E30` | 1.4 |
| `MaxRequestsToGame` limit | config notify | GameGate `0x1400D9C30` | 1.5 |
| AppGate start, `AppGate.ini` keys | start, INI reader | AppGate `0x1400CF900`, `0x1400CF990` | 2.1 |
| HTTP request → bus | `AgHttpConn` | AppGate `0x1400CE600` | 2.2 |
| Extern service register (GP) | `CAgStsConn` slot 0 | AppGate `0x1400D2F20` | 2.3 |
| MwGate start, `Gate.ini [ExternService.*]` | start, reader | MwGate `0x1400CEAD0`, `0x1400CEB60` | 3.1 |
| Middleware request dispatch | `CMwgCliConn` slot 0 | MwGate `0x1400CE5B0` | 3.2 |
| WebGate start, `WebGate.ini [Timeout]` | start, reader | WebGate `0x1400C62B0`, `0x1400C6D80` | 4.1 |
| WebGate request path | `CWebSocket` OnMessage | WebGate `0x1400C5C10` | 4.2 |

## 1. GameGate (GameGate addresses)

### 1.1 Role, start and config

- Role: daemon gate. Callers: LobbyGate and GameDaemon (TCP 6601, system `bnsserver`; `../../Sts/10-stservers-original.md`). Target: route bus providers (Auth, Presence, Charge, World, Slot, GameAccount, Goods, VirtualCurrency …). Source files: `GgMain.cpp`, `GgCli.cpp`, `GgSrv.cpp` (strings at `0x1401A68B8`, `0x1401A5E88`, `0x1401A6F10`).
- Service code range: `0x1400CEFD0`–`0x1400DAC60` (112 functions). The other 224 functions that only GameGate has are OpenSSL and GeoIP library code.
- Classes (vtable, slots): `CGgCliConn` `0x1401A59E0` (slot 0 OnMessage `0x1400D5F90`, 1 `0x1400D5BB0`, 2 `0x1400D5B90`, 4 `0x1400D5B30`); `CGgCliTxn` `0x1401A5A28`; `CGgCliDataTxn<void>` `0x1401A5A68`; `CGgCliStartChargeTxn` `0x1401A5AA8`; `CGgCliDataTxn<LoginInfo>` `0x1401A5B00`; `CGgCliDataTxn<PortalId>` `0x1401A5B60`; `CGgCliLoginTxn<unsigned>` `0x1401A5BB8`; `CGgCliDataTxn<RegisterInfo>` `0x1401A5C48`; `CGgCliDataTxn<KickUserInfo>` `0x1401A5CB0`; `CGgSrvTxn` `0x1401A6DF0` (OnReply `0x1400DA1D0`, OnError `0x1400D9E40`); `CGgSrvUserLogoutTxn` `0x1401A6E28` (`0x1400DA280`, `0x1400D9F10`); `CGgSrvGetAccountIdTxn` `0x1401A6E98` (`0x1400DA000`, `0x1400D9CD0`); `GameConnList` (made in `0x1400DA660`).
- All `CGgCli*Txn` use the gate txn layout (slot 6 = callback: `0x1400D5A30`, `0x1400D5A60`, `0x1400D5A80`, `0x1400D5AC0`). The txn object stores the handler callback at +184 (qword index 23). Txn constructor `0x1400CE1B0(txn, conn, txnId, protocol, command, n)`; `n` = 20 for the default forward, 33 for logins ❓ (use not decoded).
- Start `0x1400D7E20` (`GgMain.cpp`): if the game code is empty: `ExeFatal("GgMain: no configuration (game code)")` (line 109). `StsConfigMonitorAdd(L"Game.ini", …)`; provider name `Game` (`0x14013F8B0("Game", 0, …)`); gate start `0x140114400(…, 1)` (= `StsGateStart`, §1.2 of chapter 01 ❓ matched by call shape); client listen `0x1400D30B0` ❓; then `0x1400D8CF0` (GgSrv init).
- GgSrv init `0x1400D8CF0(gameCode)`: two 4096-entry hashes `qword_1402A3988`, `qword_1402A3A28` (`0x1400F8780`); stores the game code at `0x1402A3978` (16 bytes); builds the names `Db.Game.<code>` (`0x1402A3990`), `Game.<code>.UserIdByName` (`qword_1402A3960`), `GameWarehouse.<code>`, `GameAuction.<code>`, `GameAdmin.<code>`; registers a `Presence` route for `Game.<code>.UserIdByName` with value 3002 ❓ (`0x14013F750`); `0x14012C1F0(6, unk_14024D120)` ❓; a second `StsConfigMonitorAdd(L"Game.ini", off_14024D1B0)` (call `0x1400D8E19`; this notify reads `[Limits] MaxRequestsToGame`, §1.1 table); then `0x140129E20(off_14024D228)` ❓.

| File, key | Reader | Default | Use |
| --- | --- | --- | --- |
| `Game.ini [Configurations]` (values) | `0x1400D7F30` (`IniGetValues`) | — | AppName, game code, portal build id, portal program id, AppId per config. Logs: `GgMain: Game.ini config skipped, because buildId is already …`, `missing AppName at Game.ini which used ConfigName at StServer.ini`, `missing game code`, `missing game to portal buildId`, `missing game to portal programId`, `appId is zero` |
| `Game.ini [Limits] MaxRequestsToGame` | `0x1400D9C30` (`IniGetInt`, second `Game.ini` monitor of `0x1400D8CF0`) | 100 | Stored in `dword_14024D148`. Limit of open bus→daemon requests ❓. Out of range: log `MaxRequestsToGame must be between 1 and 100000` and keep the old value. Then `0x1400DA4F0` applies it |

The recipe `cfgkeys()` finds only these 2 service calls. `timers()` finds no service timer with a fixed ms; the only service timer is §1.5.

### 1.2 Daemon request dispatch (`CGgCliConn` slot 0 `0x1400D5F90`)

Order of checks for each request from the daemon:
1. Client info: `0x1400FD980`/`0x1400FD7C0` (CliGate `0x1400FD340`/`0x1400FD180`) read the header. If conn+180 (client id) ≠ 0, look up the IP-check entry (`0x140135A10` on `0x14024D078`) and apply it (`0x140114740` = CliGate `0x140119F70`).
2. `0x140114DD0` (CliGate `0x14011A690`, bind client app info). Fail → `ErrBadConfig` 3030, text `missing client info`. `0x1400FDA40` (CliGate `0x1400FD4D0`) fail → `invalid client info`. `0x1400FD0C0(0x66 'f', …)` fail → `missing client info`. Then `0x140112650` (CliGate `0x140117D20`).
3. Protocol `Presence` (map `0x1402B4CF8`, 27 names): `Login` (3) → `0x1400CFA20`; `Logout` (4) → `0x1400D0820`.
4. Protocol `Auth` (map `0x1402B49E0`, 110 names): `LoginTrusted` (25) → `0x1400D04A0`; `ConsumeGameToken` (31) → `0x1400CF690`; `RequestToken` (32) → `0x1400D1460`; `VerifyMobiOtp` (35) and `VerifySecondPwd` (36) → `0x1400D24B0` (always rejected); `LoginToken` (53) → `0x1400D0070`; other Auth ids → step 7.
5. Protocol `Gate` (map `0x1402B4BC0`, 8 names: BulkFind 1, Find 2, StreamConnect 3, StreamDisconnect 4, RegisterService 5, UnregisterService 6, RegisterForward 7, LookupGeolocation 8): 3 → `0x1400D1FA0`; 4 → `0x1400D20C0`; 5 → `0x1400D2240`; 6 → `0x1400D9470` (called on conn+8, the GgSrv side); 7 → `0x1400D1140`; 1, 2, 8 → step 7.
6. Protocol `Charge` (map `0x1402B4C18`, 23 names): `Start` (2) → `0x1400D1640`; `Stop` (3) → `0x1400D1B50`; others → step 7.
7. Default forward (label at `0x1400D5F90` line 249): `0x1400D69F0` reads the target client id from the request. If it gives an id and the id is not in the conn table (conn+96): reply `ErrConnNotFound` 3032 (only if the request has a txn id). Else `0x140112210(clientId, 1, 0, 0, 0, 0x82)` (CliGate `0x1401178E0`); then if the request has a txn id: new `CGgCliDataTxn<void>` (0xC0 bytes, callback `0x1400CEE20`) and `StsGateSendTxn` (`0x140114AF0` = CliGate `0x14011A3A0`); else `StsGateSendOneWay` (`0x140114D70` = CliGate `0x14011A630`). The `Gate.ini [Commands.GameGate]` check (74 commands) is in the gate library (chapter 01 §1.5).
- Default reply `0x1400CEE20`: sends the bus reply back to the daemon. On error it logs `Error(%d): #%u /%s/%s: server=%d module=%d line=%d` (line 1244).

### 1.3 Local handlers

All error replies below use module 1 ❓ and the `GgCli.cpp` line. Send timeout of every `SendToProviderTxn` here is 25000 ms (`0x61A8`).

| Command | Handler | Steps | Errors (code, line, text) |
| --- | --- | --- | --- |
| `Presence/Login` (3) | `0x1400CFA20` | Parse body (`0x1400EBF60`). Check the login name (needs `:` ❓, `0x1400DC280(name, 58)`), alias chars, client net address. Build client info (`0x1400FC990`, `0x140114740`), header `f` (`0x1400FE1C0(msg, 102, …)`). Build `Auth/LoginInternal` (id 28, `0x1400E80D0(28)`). New `CGgCliDataTxn<LoginInfo>` (0xA38 bytes, protocol `Auth`, callback `0x1400D3920`). `SendToProviderTxn(…, 25000)`. No txn: `StsRouteSend`. | 10 ErrBadParam: 1755 `invalid chat message, parser line %u`, 1759 `missing login name`, 1762 `missing client net address`, 1766 `invalid login name`, 1777 `alias has invalid chars`, 1790 `invalid client` |
| Login reply chain | `0x1400D3920` → `0x1400D3390` → `0x1400D2560` → `0x1400D6A70` → `0x1400D2600` | See "Login chain" below the table. | `invalid client`; framework error reply on each failed step |
| `Presence/Logout` (4) | `0x1400D0820` | Read `UserId`. Find the login (`0x1400D75B0`). Check `f` and `t` headers and the permission. If the user has a charge session: log `logout user without stop charge. user(%s)` and stop it (`0x1400D77F0`). Forward the logout. | 10/2249 `logout, no body element`; 3007 ErrUserNotOnline/2266 `logout, login not found. user(%s)`; 10/2286 `logout, invalid from header`; 49 ErrPermission/2298 `logout, permission denied`; 3007/2309; `logout, invalid to header` |
| `Auth/LoginTrusted` (25) | `0x1400D04A0` | Parse; find client info; new `CGgCliDataTxn<LoginInfo>`; `SendToProviderTxn` (25000). | 10/1595 `invalid auth client message, parser line %u`; 10/1604 `not found client info, parser line %u` |
| `Auth/ConsumeGameToken` (31) | `0x1400CF690` | Parse; set field `TokenInfo` (msg vtable +288); header `f`; `CGgCliDataTxn<void>`; `SendToProviderTxn` (25000). | 10/1346 `invalid auth client message, parser line %u`; 10/1351 `invalid client` |
| `Auth/RequestToken` (32) | `0x1400D1460` | Find the target conn (header `m` = 109 ❓); `CGgCliDataTxn<void>`; forward. | 3032 ErrConnNotFound |
| `Auth/VerifyMobiOtp` (35), `Auth/VerifySecondPwd` (36) | `0x1400D24B0` | Always reply error. | 49 ErrPermission/1742 `invalid cmd` |
| `Auth/LoginToken` (53) | `0x1400D0070` | Parse; find client info; set `TokenInfo`; read `id`, set `random`; `CGgCliDataTxn<LoginInfo>`; `SendToProviderTxn` (25000). | 10/1426, 10/1435 (same texts as LoginTrusted) |
| `Gate/StreamConnect` (3) | `0x1400D1FA0` | Reply with element `Reply` / `Stream` (stream id) at once. | line 1266 |
| `Gate/StreamDisconnect` (4) | `0x1400D20C0` | Find the stream (header `m`); remove it. | 3032/1279, 1275, 1281 |
| `Gate/RegisterService` (5) | `0x1400D2240` | Read `Protocol`. If `Srv2World`: read `WorldCode`, name `Db.World.<code>.<world>`. Register the custom protocol (`0x1400D8850`, `0x1400DA660`: log `custom protocol is registered (%s)`). Send a register request (`SendToProviderTxn`, 25000). | 10: 2356 `missing body`, 2360 `missing protocol`, 2366 `missing worldCode` |
| `Gate/UnregisterService` (6) | `0x1400D9470` (GgSrv) | Parse the protocol (`0x1400D8850`); remove the conn from the `GameConnList`. | 3029/854, 857 |
| `Gate/RegisterForward` (7) | `0x1400D1140` | Read `Scope`, `Version`, `ProtocolTemplate`, `ScopeTemplate`; `CGgCliDataTxn<void>`; `SendToProviderTxn` (25000). | 10/1297 `register fwd, missing protocol template` |
| `Charge/Start` (2) | `0x1400D1640` | Read `UserId`, `ClientNetAddress`, `Location`, `LocationId`. Find the login. `CGgCliStartChargeTxn` (0x190 bytes); forward. | `start charge, no body element`; `start charge, no userId`; `start charge, invalid userId`; 3007/2444 `start charge, login not found.`; 10/2436 `start charge, no client address` |
| `Charge/Stop` (3) | `0x1400D1B50` | Read `UserId`, `Location`, `LocationId`. Find the login. `CGgCliDataTxn<PortalId>`; forward. | 2478 `stop charge, no body`; 2487 `stop charge, no userId`; 3007/2497 `stop charge, login not found.` |

Login chain (after `Auth/LoginInternal`, all txns are `CGgCliDataTxn<LoginInfo>`, 0xA38 bytes, timeout 25000 ms):
1. `0x1400D3920` (LoginInternal reply). Fail: log `Error(%d). fail to logininternal name(%s)` (name at txn+156). Success: read `UserId`, `UserCenter`, `UserName`, `Roles`/`RoleId`, `PcCafe`, `PcCafeCode`, `UserStatus`; copy them into the login info. Build a request with header `t` (116) = client info and command id 1 of protocol `IpCheck` (`0x1400EB340(1)`). Txn ctor last arg 33, callback `0x1400D3390`, `SendToProviderTxn` (`0x61A8`). If the daemon request has no txn id: `StsRouteSend` (one way).
2. `0x1400D3390` (IpCheck reply). Error: log `Error(%d): #%u /%s/%s: …`. Success: apply the client info (`0x140114740`), then protocol `Access` command id 5 (`0x1400E9720(5)`), ctor last arg 20, callback `0x1400D2560`, sent with `0x140112210(clientId, 1, txn, &done, 0, 0x82)`. If `done` ≠ 0 (the gate library did not send it), free the txn and go to step 4 at once.
3. `0x1400D2560`: error code ≠ 0 → log and error reply; 0 → step 4.
4. `0x1400D6A70`: protocol `Access` command id 6 (`0x1400E9720(6)`), ctor last arg 20, callback `0x1400D2600`, sent with `0x140112470(clientId, txn, req+132, req+368, byte_1402A38B0, …)`.
5. `0x1400D2600`: read `AccessMask`, AND it with the client mask (`0x1401137A0(clientId)`). Make a login record (0xE8 bytes), add it to the user table `dword_14024D078` and to the conn list conn+40. Then send a `Presence` request (ctor last arg 33, 0xA40 bytes, callback `0x1400D4220`); a second `Presence` path (0xE0 bytes, callback `0x1400D35D0`) or `StsRouteSend` when no txn ❓ (condition not decoded). The names of `Access` ids 5, 6 and the `Presence` ids are ❓ (map not printed).

The txn ctor last arg is 20 for `Access` and for the default forward, 33 for `IpCheck`, `Presence` and the logins. Its use is ❓.

Internal stop charge `0x1400D77F0`: logs `System error(%d): fail to stop charge. invalid arguments. us…`.

### 1.4 Bus side (`GgSrv`)

Bus OnMessage `0x1400D8170`:
1. `ApiManage` notices (names from map ids 36, 37, 38; `ApiMainVersion`, `ApiHotfixVersion`, `Scope`) → gate library ApiManage update (`0x140111C50`, `0x140111C20`, `0x140111CF0`), reply `Reply`.
2. `Charge` ids 8 `NotifyRemainTime` → `0x1400D2F70`; 9 `NotifyApplyNewChargeItem` → `0x1400D2E80` (reads `ChargeKey`); 10 `RequestStop` → `0x1400D3010` (verified: `id - 8` = 0, 1, 2 at `0x1400D8170`; other Charge ids → `0x1400D8E30`). Each finds the user's daemon conn and sends the message to it (`0x1400D1090`).
3. `Presence/UserLogout` (14) → `0x1400D32A0`: find the user (`0x140135A10`), send it to the daemon (`0x1400D0FF0`).
4. All other bus requests → `0x1400D8E30`.

`0x1400D8E30(msg, protocol)`:
- Find the custom protocol (`0x1400D8850`). Unknown: `ReplyError` 940 `GgSrv: protocol %s, unknown command`.
- Protocol `Db.Game.<code>` ❓ exec path: no body → 946 `GgSrv: exec, null body`; no txn id → 950 `GgSrv: exec, no transaction id`; read `ProcName`, `Params`. Only `UserIdByName` is known; other → 960 `GgSrv: exec %s, unknown procName`; no char name → 964 `GgSrv: exec %s, missing character name`. Builds `Request`/`CharName` and a `CGgSrvGetAccountIdTxn` to the daemon. Reply `0x1400DA000`: `GgSrv: exec %s, null body`, `GgSrv: exec %s, missing game account`.
- Other protocols: new `CGgSrvTxn` (header `p` = 112 ❓) and send to a daemon conn of the `GameConnList` (`0x1400DA840`, list names `World`, `GameWarehouse`, `GameAdmin`, `GameAuction`, `GameGroup`, `MobileGameWarehouse`, `Game`).

Custom protocol parser `0x1400D8850` (log lines 999–1155): `Protocol` required (`GgSrv: missing protocol`), max length (`GgSrv: protocol too long, %s`). Name map: `Srv2Game` → `Game.<code>`; `Srv2World` + `WorldCode` → `Db.World.<code>.<world>` (`GgSrv: missing worldcode`); `Srv2GameWarehouse` → `GameWarehouse.<code>[.<x>]`; `Srv2GameAuction` → `GameAuction.<code>[.<x>]`; `Srv2GameGroup`; `Srv2GameAdmin` → `GameAdmin.<code>[.<x>]`; `Srv2MobileGameWarehouse` → `MobileGameWarehouse.<code>[.<x>]`; other names must start with `Game.%s` (`GgSrv: custom protocol missing 'Game.%s' prefix, %s`, line 1155).

### 1.5 State and timers

- Conn table conn+96 (client id → conn), user table (`0x140135A10` lookups), custom protocol hashes `qword_1402A3988`, `qword_1402A3A28` (4096 entries each).
- Request queue timer: global notify `0x14024D208` (vtable `0x1401A6F00`, OnTimer `0x1400DA430`). Armed in `0x1400DA840` with `StsTimerSet(0x14024D208, dword_14024D220, 0)`; `dword_14024D220` = 25000 ms at load. OnTimer returns −1 when the queue is empty, else the period at +28 (after `0x1400DA340` drops expired entries ❓).
- `MaxRequestsToGame` (default 100, range 1–100000) limits the open bus → daemon requests (`0x1400DA4F0`) ❓.

### 1.6 Calls out

| Target | Command | When | Call site |
| --- | --- | --- | --- |
| AuthSrv | `Auth/LoginInternal` | daemon `Presence/Login` | `0x1400CFA20` |
| AuthSrv | `Auth/LoginTrusted`, `ConsumeGameToken`, `LoginToken` | same daemon command | `0x1400D04A0`, `0x1400CF690`, `0x1400D0070` |
| IpCheck provider | `IpCheck` id 1 | after `LoginInternal` success | `0x1400D3920` |
| Access provider | `Access` id 5, then id 6 | after `IpCheck` success | `0x1400D3390`, `0x1400D6A70` |
| PresenceSrv | `Presence` login ❓ (id not printed) | after `Access` id 6 success | `0x1400D2600` |
| Presence / Charge | `Presence/Logout`, `Charge/Start`, `Charge/Stop` | same daemon command | `0x1400D0820`, `0x1400D1640`, `0x1400D1B50` |
| route bus | register custom protocol, `RegisterForward` | daemon `Gate/RegisterService`, `Gate/RegisterForward` | `0x1400D2240`, `0x1400D1140` |
| any provider | daemon command (after `Gate.ini` check) | default | `0x1400D5F90` → `StsGateSendTxn` |
| daemon conn | bus request for a custom protocol, `UserLogout`, charge notices | bus message | `0x1400D8E30`, `0x1400D32A0`, `0x1400D2F70` |

### 1.7 Worked example

See §5.

## 2. AppGate (AppGate addresses)

### 2.1 Role, start and config

- Role: HTTP/JSON gate for web and app clients (TCP 6602, `Web.ini [URIs.AppGate]`). It changes each HTTP request into a route bus request (the URI → command table and the 734 `[Commands.AppGate]` rules are in the gate library). It also accepts "extern service" connections (STS socket, `CAgStsConn`) from outside servers (GP proxies) that register their own protocols.
- Source files: `AgHttpConn.cpp` (`0x1400CE600`), `AgSrv.cpp` (15 refs, `0x1400CF990`–`0x1400D2A60`), `AgStsConn.cpp` (`0x1400D2F20`). Service range `0x1400CE100`–`0x1400D2F20` (91 functions). The other functions that only AppGate has are library code.
- Classes: `CAgSrvExternService` `0x1401A2FA8` (slot 0 dtor `0x1400D0450`, slot 4 `0x1400D11F0`, slot 5 OnError `0x1400D0FF0`); `CAgSrvTxn` `0x1401A2FB8`; `CStsTimerList<AgSrvBacklogMsg>` `0x1401A3178` (OnTimer `0x1400D1370`); `CAgStsConn` `0x1401A3328` (slot 0 OnMessage `0x1400D2F20`, slot 1 OnDisconnect `0x1400D2E70`); `ReservedProtocol`.
- Start `0x1400CF900`: `StsConfigMonitorAdd(L"AppGate.ini", …)`; `StsGateStart` (`0x1401013B0` = CliGate `0x140119A90`); `0x1400CE1F0` (reply buffer pool `qword_1402A0948`, 4096 × 0x4000 ❓; HTTP listen `0x140116710(off_14024A060, …, 1)` ❓); `0x1400D2740` (extern service listener: `0x14011EC40(…, 414, 1000, 0, 0, 4000, …)` ❓ meaning of the numbers); `0x1400D0610` (`0x140122790(4, …)`, then `0x1400D1870` = extern protocol name table).
- `AppGate.ini` reader `0x1400CF990` (INI getters `0x1400E24E0` section, `0x1400E2400` values, `0x1400E2840` bool; not the framework `IniGetInt` — so `cfgkeys()` finds no AppGate key):

| Key | Default | Use (global) |
| --- | --- | --- |
| `[ExternServices] service` (many values) | none | list of extern service names that may register |
| `[Rights] CheckRightOnHttp` | 0 | `dword_1402A0980`: check rights for HTTP requests |
| `[Rights] CheckRightOnSts` | 0 | `dword_1402A0984`: check rights for extern STS requests |
| `[Log] EnableErrorReplyLog` | 0 | `dword_1402A0988`: log error replies (`Error(%d): #%u /%s/%s: server=%d module=%d line=%d`) |
| `[Log] EnableEventLog` | 0 | `dword_1402A098C` |
| `[Log] ExcludeErrorCode` (many values) | none | error codes that are not logged (`qword_1402A0970`, count `dword_1402A0978`) |

### 2.2 HTTP request path

`0x1400CE600` (vtable `0x1401A28D0` slot 1; `AgHttpConn.cpp`): builds client info from the HTTP request (`0x1400EA870`, remote address, headers at +24, +208, +295, +311), gets a gate request (`0x1401015E0` = CliGate `0x140119DC0`), sets the client info (`0x1400FFF00`), sets flags `0x1400FFA80(…, 130)` (= CliGate `0x1401178E0`, same `0x82` as GameGate), makes an HTTP txn (`0x1400CDEE0`, 0x20E8 bytes) and sends with `0x140101A50(req, http, txn, 25000)` (timeout 25000 ms).

Reply writers:
- `0x1400CE240`: JSON error body with `error`, `product`, `module`, `line`, `text`; `Content-Type: application/json` (`0x1400CE8D0`).
- `0x1400CEA40`: big reply; logs `dropped overly large reply`, `Srv2SvcDispatcher::GetResultMsg failed.`, `… (large msg`.
- `0x1400CEEB0`: XML reply (`application/xml`); `Invalid content type for reply`; error log when `EnableErrorReplyLog`.
- `0x1400CF170`: chunked reply; `parse, chunked message parse failed`.
- `0x1400CF650`: reads `UserName`, `PortalName` (`type`, `array`) ❓ (use not decoded).

### 2.3 Extern service connection (`CAgStsConn` slot 0 `0x1400D2F20`)

Accepts only protocol `Gate`; other: error 200 ❓/`%s is unknown command`.

| Command | Handler | Steps | Errors |
| --- | --- | --- | --- |
| `Gate/RegisterService` | `0x1400D2F20` → `0x1400D0B20`, `0x1400D0660` | Log `request register service (%u)`. Read `Protocol` (must be in the extern table `0x1400D1870`, check `0x1400D1430`), `ServiceCode`, `WorldCode`; route name `<proto>.<service>` or `<proto>`. Register (`0x1400D1670`: `extern protocol is registered (%s)` / `extern protocol is already registered (%s)`). | 261 `missing body`; 268 `missing protocol`; 279 `custom protocol is wrong`; 297; `fail to register service` |
| `Gate/UnregisterService` | `0x1400D2F20` → `0x1400D08E0`, `0x1400D2380`, `0x1400D2290` | Log `request unregister service (%u)`. Remove the protocol when no conn uses it. | `fail to unregister service`; `fail to unregister service, because of no register`; `extern protocol is already unregistered (%s)`; `fail to unregister. connections are remaining yet …` |

- Extern protocol table `0x1400D1870` (pairs name → bus protocol): `Gp`→`Srv2Gp`, `GPProxy`→`Srv2GpProxy`, `GpIpCharge`→`Srv2GpIpCharge`, `GpVmCharge`→`Srv2GpVmCharge`, `GpMtx`→`Srv2GpMtx`, `GpVmBilling`→`Srv2GpVmBilling`, `GpAllPoint`→`Srv2GpAllPoint`, `GpBilling`→`Srv2GpBilling`, `GpNctasCm`→`Srv2GpNctasCm`, `GpPurchaseNoti`→`Srv2GpPurchaseNoti`.
- Bus → extern service: `0x1400D2120` finds the extern protocol (`can not find extern protocol (%s)`), then `CAgSrvExternService` txn sends to the extern conn. Reply errors are logged by `0x1400D0FF0` / `0x1400D2A60`.
- OnDisconnect `0x1400D2E70`: `0x1400D27A0` removes the conn; log `extern service socket disconnect (%u). address: %s`.
- ApiManage notices `0x1400CFD50` (`Scope`, `ApiMainVersion`, `ApiHotfixVersion`, reply `Reply`): same as GameGate §1.4 step 1.

### 2.4 State and timers

- Backlog: `CStsTimerList<AgSrvBacklogMsg>` (OnTimer `0x1400D1370`: −1 when empty, else period at +28 after `0x1400D12B0`). Parse fail log `fail to parse backlog message` (`0x1400D1530`). Period ❓ (no `StsTimerSet` call with a fixed ms in the service range).
- No DB proc, no fixed-period timer (`timers()` finds none in the service range).

### 2.5 Calls out

| Target | Command | When | Call site |
| --- | --- | --- | --- |
| any provider | HTTP URI → command (gate library) | each HTTP request | `0x1400CE600` |
| extern GP service | bus request on `Srv2Gp*` | bus message for a registered extern protocol | `0x1400D2120` |

### 2.6 Worked example: HTTP request

`POST` to an AppGate URI with JSON body. `0x1400CE600` builds the client info (remote IP, headers), the gate library maps the URI to a command and checks `[Commands.AppGate]`, and the request goes to the bus with 25000 ms timeout. The reply comes back as JSON (`0x1400CE240` on error: `{"error":…, "product":…, "module":…, "line":…, "text":…}`).

## 3. MwGate (MwGate addresses)

### 3.1 Role, start and config

- Role: middleware gate (TCP 6601). Middleware servers (GP proxy, billing, charge) connect with STS sockets (`CMwgCliConn`), send bus requests through the gate (`[Commands.MwGate]`, 185 commands, gate library check), and register extern protocols so that bus requests for them come back to their connection. Same model as the AppGate extern services (§2.3), but on the gate port and with `Gate.ini`.
- Source files: `MwgCli.cpp`, `MwgSrv.cpp`. Service range `0x1400CDF10`–`0x1400D17A0` (148 functions in `0x1400CD000`–`0x1400D2000`). The RiskMgmt rule classes (`CRiskMgmtRule` `0x1401B6168` and its fields) are gate library code.
- Classes: `CMwgCliConn` `0x140195850` (slot 0 OnMessage `0x1400CE5B0`, slot 1 OnDisconnect `0x1400CE490`); `CMwgCliTxn` `0x140195898` (callback `0x1400CE030`; reply writer `0x1400CE820`: `Response`, `Reply`); `CMwgSrvTxn` `0x140195CB8` (slot 3 `0x1400D0F40`, slot 4 `0x1400D0F10`); `CMwExternProtocol` `0x140195CF0`; `CMwExternService` `0x140195D00`; `CStsTimerList<CMwgSrvBacklogMsg>` `0x140195FB0` (OnTimer `0x1400D1090`).
- Start `0x1400CEAD0`: `StsGateStart` (`0x1400FD250` = CliGate `0x140119A90`); `0x1400CE1E0(1000, 0)` (client listener: `0x140110C10(…, 409, 1000, 0, 0, 4000, …)` ❓ meaning of 409/1000/4000); `0x1400D03F0` (extern protocol table); `StsConfigMonitorAdd(L"Gate.ini", …)`.

| Key | Reader | Default | Use |
| --- | --- | --- | --- |
| `Gate.ini [ExternService.<name>] service` (many values) | `0x1400CEB60` (`IniGetValues`) | none | services that may register extern protocol `<name>`; bit mask built per value (`i *= 2`) |

Extern protocol table `0x1400D03F0`: same 10 pairs as AppGate (`Gp`/`Srv2Gp` … `GpPurchaseNoti`/`Srv2GpPurchaseNoti`) ❓ (last pairs not printed; first 8 are the same).

### 3.2 Client dispatch (`CMwgCliConn` slot 0 `0x1400CE5B0`)

1. Client info (`0x1400E7390`, `0x1400E7E40(conn+48)`), header `f` (`0x1400E7870(102, …)`), `0x1400FBD50` (= CliGate `0x140117D20`).
2. Protocol `Gate` (map `0x1402913E0`: RegisterService 1, UnregisterService 2, LookupGeolocation 3):
   - 1 → `0x1400D0B90`: parse protocol (`0x1400D01A0`: `null body`, `missing protocol`, `protocol is too long (%s)`) and `ServiceCode`/`WorldCode` (`0x1400D00C0`); find the group (`0x1400D15F0`: `<proto>.<service>`, `reserved protocol, but can not find protocol group`, `can not find protocol group (%s)`); register (`0x1400D1430`: `custom protocol is registered (%s)`).
   - 2 → log `MwGate request unregister service (%u)`, `0x1400D0CB0` (`0x1400D1EC0`: `try to unregister service, but no registered servi…`; `0x1400D1C90`: `custom protocol is unregistered (%s). address: %s`).
   - 3 → forward (step 3).
   - other Gate id: error `%s is unknown command` ❓ (path decoded only by string).
3. Other protocols: `0x1400FBAF0(clientId, 0,0,0,0, 130)` (= CliGate `0x1401178E0`); with txn id: `CMwgCliTxn` (0xB8 bytes, `0x1400CDD20`) and `0x1400FD7C0(clientId, msg, txn, 25000)` (gate send txn, 25000 ms); else `0x1400FDA40` (one way).
- OnDisconnect `0x1400CE490`: log `MwGate client socket disconnect (%u). address: %s`; removes the conn's protocols (`0x1400CFD90`: `clear custom protocol (%s)`; `0x1400D1AB0`: `service(%s)'ve lost all connection -- and should'v…`, `can not find registered protocol (%s)`).
- Bus → middleware: `0x1400D1DA0` (`can not find registered service protocol (%s)`) and `CMwgSrvTxn`. ApiManage notices `0x1400CED70` (same as GameGate §1.4).

### 3.3 State and timers

- Backlog timer list `CStsTimerList<CMwgSrvBacklogMsg>`: arm in `0x1400D1C00` = `StsTimerSet(list, list+24, 0)` (period from the record ❓); OnTimer `0x1400D1090` → `0x1400D0FF0`; `0x1400D1310` log `process backlog, fail to queued msg`.
- No DB proc. `cfgkeys()` finds 1 call, `timers()` 1 call (no fixed ms).

### 3.4 Calls out

| Target | Command | When | Call site |
| --- | --- | --- | --- |
| any provider | middleware request (after `Gate.ini` check) | each request | `0x1400CE5B0` → `0x1400FD7C0` |
| middleware conn | bus request on a registered extern protocol | bus message | `0x1400D1DA0` |

Worked example: GP proxy connects, sends `Gate/RegisterService` with `Protocol=Gp`, `ServiceCode=…`. `0x1400D0B90` checks that `Gate.ini [ExternService.Gp] service` allows it, registers `Srv2Gp.<service>` and logs `custom protocol is registered (Srv2Gp…)`. Then a bus request to `Srv2Gp` goes to this socket; if the socket closes, `0x1400CE490` clears the protocol.

## 4. WebGate (WebGate addresses)

### 4.1 Role, start and config

- Role: small HTTP gate for web pages (it does not link the full gate library: 350 service functions). It sends each HTTP request to the bus as a route request with the client address, and it has a local `Session` protocol path (web session in SessionGateSrv ❓ target). Callers: web front ends. Listen ports: TCP 6601 group in `../../Sts/10-stservers-original.md` ❓.
- Source files: `WgMain.cpp`, `WgConn.cpp`. Service range `0x1400C47B0`–`0x1400C6C70` (59 functions in `0x1400C4000`–`0x1400C7000`).
- Class: `CWebSocket` `0x140165258` (OnMessage `0x1400C5C10`).
- Start `0x1400C62B0`: two reply buffer pools (4096 × 0x4000, `qword_14022E8F8` and one more); provider name `WebGate` (`0x140111D30`); two listeners `0x1400FADF0(…, 402, 1000, 0, 0, 4000, …)` and `0x1400FADF0(…, 87, 1000, 0, 0, 4000, …)` ❓ (meaning of 402/87; the same shape as AppGate 414 and MwGate 409); config `WebGate.ini` (`0x1400C6D20`).

| `WebGate.ini` key | Reader | Default | Use |
| --- | --- | --- | --- |
| `[Timeout] Target` | `0x1400C6D80` (`IniGetInt`) | 0 | timeout log target; bad value: log `WebGate.ini: invalid log target (target : %d)` |
| `[Timeout] InfixFilename` | `0x1400C6D80` (`IniGetStr`) | `Timeout` | timeout log file name part |
| `[Timeout] ExtName` | `0x1400C6D80` (`IniGetStr`) | `log` | timeout log file extension |
| `[Timeout] Period` | `0x1400C6D80` (`IniGetInt`) | 0 | log file period; bad value: `WebGate.ini: invalid log period (period: %d)` |

Timeout log writer `0x1400C6F20`: one CSV line `%s,%s,%s,%s,%s` per timed-out request ❓ (field order not decoded); bad input: `TimeoutLogWrite : Invalid Param.`

### 4.2 Request path (`CWebSocket` OnMessage `0x1400C5C10`)

1. Stream and from header check `0x1400C60F0`: errors `Missing From Header`, `Invalid Stream Header`, `Missing or Invalid Stream Header`; reads `expire`.
2. Protocol `Session`: a `t` (to) header is not allowed → error line 946 `session command with 'to' header`. JSON body parse `0x1400C49E0` (`Request`, `Persistent`, `Expire`); fail → line 966 `Invalid json message body`. Then a txn (0x118 bytes) is sent ❓ (target).
3. Other protocols: header `m` (109) ❓, element `Request`, client address `0x1400C6A00` (`X-Forwarded-For` → `_RequestAddr`), txn `0x1400C3F30` (0x118 bytes), `SendToProviderTxn(…, 25000, 0)`.
- JSON body helpers: `0x1400C6700` (`_Origin`, `Method`, `Client`, `AppId`, `Params`), `0x1400C6500` (`UserName`, `PortalName`).
- Replies: `0x1400C4340` JSON error (`error`, `product`, `module`, `line`, `text`); `0x1400C4D00` big reply (`dropped overly large reply`, `Srv2SvcDispatcher::GetResultMsg failed.`); `0x1400C5170`/`0x1400C53F0`/`0x1400C5690` XML (`Invalid content type for reply`).
- Bus messages `0x1400C5930`: ApiManage notices (same as GameGate §1.4); other protocol → log `WebGate: unknown protocol %s`.

### 4.3 State, timers, calls out

- No DB proc, no service timer (`timers()` finds none), no command table: WebGate forwards all non-`Session` requests. Calls out: any provider (`SendToProviderTxn`, 25000 ms) at `0x1400C5C10`.

## 5. GameGate worked example: `Presence/Login`

Daemon sends `Presence/Login` with txn id 7, body login name `bns:player1` ❓, client net address `10.0.0.5`. `0x1400CFA20` checks the name and address, builds `Auth/LoginInternal` with header `f` = client info, makes `CGgCliDataTxn<LoginInfo>` (callback `0x1400D3920`) and sends it to AuthSrv with 25000 ms timeout. AuthSrv reply (`UserId`, `UserName`, `Roles` …) → `0x1400D3920` sends `IpCheck` id 1 → `0x1400D3390` sends `Access` id 5 → `0x1400D6A70` sends `Access` id 6 → `0x1400D2600` reads `AccessMask`, adds the user to `dword_14024D078` and conn+40, and sends the `Presence` login. The final callback (`0x1400D4220` ❓) replies to txn 7. No AuthSrv reply in 25000 ms → framework `ErrTimeout` 42 → error reply to the daemon.

## 6. Quirks

| Name | Address | What the code does | Value | Who can reach it |
| --- | --- | --- | --- | --- |
| `quirk_gg_verify_cmds_rejected` | GameGate `0x1400D24B0` | `Auth/VerifyMobiOtp` and `Auth/VerifySecondPwd` from the daemon always fail. | `ErrPermission` 49, line 1742, text `invalid cmd` | any daemon that sends them |
| `quirk_wg_xff_trusted` | WebGate `0x1400C6A00` | The client address comes from the `X-Forwarded-For` header when it is present. | header value, no proxy check ❓ | any HTTP client |
| `quirk_ag_rights_off` | AppGate `0x1400CF990` | Rights checks for HTTP and extern STS requests are off unless the INI sets them. | 0 (no check) | any AppGate client |

## 7. Framework gaps

- Gate binaries use INI getters that are not in `framework-names.tsv` (AppGate `0x1400E24E0` section, `0x1400E2400` values, `0x1400E2840` bool). `cfgkeys()` does not see them.
- The listener maker has the shape `(factory, mgr, N, 1000, 0, 0, 4000, 0, notify)` with N = 414 (AppGate `0x14011EC40`), 409 (MwGate `0x140110C10`), 402 and 87 (WebGate `0x1400FADF0`). The meaning of N, 1000 and 4000 is ❓ (connection type id, backlog, buffer size?).
- `CStsTimerList<T>` (AppGate `0x1401A3178`, MwGate `0x140195FB0`): OnTimer returns −1 when the list is empty, else the period stored at +28 (AppGate) or arms with +24 (MwGate `0x1400D1C00`).
- Gate library functions have no names in these binaries (`gate-names.tsv` does not exist). This chapter gives the CliGate address of the same function (match by hash in `funcs/*.tsv`).

## 8. Differences from existing specs

- `../../Sts/10-stservers-original.md` puts WebGate on port 6601 with GameGate and MwGate. WebGate is an HTTP gate (like AppGate), not an STS gate ❓ (port not read from the binary).
- The GameGate list in `10-stservers-original.md` names `Auth/RequestToken` and `Auth/LoginToken` as plain forwarded commands. GameGate handles them locally (§1.3) and changes the request (`TokenInfo`, `random`, header `f`).
- GameGate `Presence/Login` is not forwarded to PresenceSrv as is: the gate sends `Auth/LoginInternal` to AuthSrv (§1.3).
