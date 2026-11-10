# 01 — Gate library, CliGate, SessionGateSrv (RE reference)

Summary:
1. All 5 gates (CliGate, SessionGateSrv, GameGate, AppGate, MwGate) link one gate library: 663 functions of `CliGate.exe` are in all 5 function lists (733 are also in SessionGateSrv). Source files: `StsGate.cpp`, `NetAccess.cpp`, `NetAdjust.cpp`, `Geolocation.cpp`, `StreamJson.cpp`, GeoIP C library 1.6.4.
2. The gate library starts with `StsGateStart` (CliGate `0x140119A90`): it watches `Gate.ini`, registers 11 gate txn kinds and starts the route client for the `Gate` provider name.
3. CliGate (server type 1001) is the client-facing gate. `CCliConn` OnMessage `0x1400D4AD0` runs the local Auth login state machine and sends all other client requests through the gate library to the route bus.
4. SessionGateSrv (server type 1014) keeps the login sessions of the gates. It handles 17 commands (`Auth/*Login*`, `MobileAuth/Login*`, `Session/*`) through a name → handler-object table (`0x1400D1280`), keeps session objects, stores persistent sessions in `SessionDb` (6 procs) and sends login steps to the Auth service through the gate library.
5. The CliGate spec `../11-cligate-protocol.md` agrees with this binary (same build, same addresses). §5 lists the new facts and the differences.

All addresses are in the binary of the section. CliGate = `CliGate.exe.i64`, SessionGateSrv = `SessionGateSrv.exe.i64`. ❓ = not verified in IDA. Lists: `01-commands.tsv`, `01-procs.tsv`.

## Quick find

| Question | Function | Binary, address | Section |
| --- | --- | --- | --- |
| Where does the gate library start? | `StsGateStart` | CliGate `0x140119A90` | 1.2 |
| Gate txn kinds and their vtables | `CStsGate*Txn` | CliGate `0x1401B1030`–`0x1401B1650` | 1.3 |
| Client request → route bus (txn) | `StsGateSendTxn` | CliGate `0x14011A3A0` → `0x140113F90` | 1.4 |
| Client request → route bus (no reply) | `StsGateSendOneWay` | CliGate `0x14011A630` | 1.4 |
| Gate send with IP check state | `StsGateSend` | CliGate `0x140114A10` | 1.4 |
| `Gate.ini` rules (commands, rights, risk rules, keys) | | CliGate `0x140111B20`, `0x140111DB0`, `0x140111F50`, `0x140112460`, `0x140112AB0` | 1.5 |
| NetAdjust (local replies, target rewrite) | | CliGate `0x14017BEC0`–`0x140180020` | 1.6 |
| CliGate start | `CliMain` | CliGate `0x1400D6720` | 2.1 |
| CliGate config (`CliGate.ini`) | `CCliGateConfig` slot 0 | CliGate `0x1400D6030` | 2.1 |
| Client message dispatch | `CCliConn` slot 0 | CliGate `0x1400D4AD0` | 2.2 |
| Bus messages to CliGate (`Cli/Logout`, ApiManage) | bus OnMessage | CliGate `0x1400D6A00` | 2.3 |
| Client reply for forwarded calls | `ClientReply` | CliGate `0x1400CED50` | 2.2 |
| SessionGateSrv start and `Session.ini` keys | start, INI reader | SessionGateSrv `0x1400D2A40`, `0x1400D2B20` | 3.1 |
| SessionGateSrv command table | builder | SessionGateSrv `0x1400D1280` | 3.2 |
| SessionGateSrv handlers step by step | `COn*` slot 1 | SessionGateSrv `0x1400CE6A0`–`0x1400D0AB0` | 3.5 |
| Persistent session lookup in DB | `GetPersistentSession` | SessionGateSrv `0x1400CE2B0` | 3.5 |

## 1. Gate library (CliGate addresses)

### 1.1 Parts

Function sets: `inn(h)` = gate binaries whose function list has hash `h`. In `CliGate.tsv`: 663 functions in all 5 gates, 44 in 4, 49 in 3, 44 in 2, 217 only in CliGate. Address clusters of the 663 shared functions:

| Range | Count | Part |
| --- | --- | --- |
| `0x140001000`–`0x140007950` | 89 | GeoIP C library 1.6.4 (`GeoIP.dat`, `GeoIPRegion.dat`, `GeoIPCity.dat`, `GeoIPISP.dat`) |
| `0x1400C7F50`–`0x1400CD810` | 59 | containers, allocator (`Collections/Allocator.h`) |
| `0x1400E35A0`–`0x1400EEC60` | 31 | protocol name maps and request builders (`ApiManage/*`, `Auth.ini [UserName] UniqueProperty`), `Srv2IpAuthz` body parsers `0x1400E6B00`–`0x1400E74E0` |
| `0x1400F8510`–`0x1400FF250` | 49 | `NetAccess.cpp` (header `f`/`t` build and parse, `Evidence`) |
| `0x1401026B0`–`0x14011C8C0` | 273 | `StsGate.cpp` core (gate sockets, user conns, txns, `Gate.ini`, ApiManage client) |
| `0x14014E7B0`–`0x140153C4C` | 13 | JSON helpers (`portal_name`) |
| `0x14016F570`–`0x140172110` | 32 | request builders: `AppAuth/Authenticate` `0x14016F630`, `AppAuth/AuthenticateBearerToken` `0x14016F730`, `AppAuth/GetAppGroupCode` `0x14016F7B0`, `AppAuth/GetAppRights` `0x14016F830`, `RiskMgmt/Inquiry` `0x140170E70`/`0x140170F50`/`0x140171060`, `RiskMgmt/Report` `0x140171150`/`0x140171240`, `RiskMgmt/SetUserData` `0x140171340`, `World` key `%s.%s` `0x140172110` |
| `0x14017AE60`–`0x140180140` | 54 | `NetAdjust.cpp`, `Geolocation.cpp` |
| `0x1401833C0`–`0x14018A2DC` | 29 | `StreamJson.cpp` |

Note: the recipe `table()` gives wrong handlers for gate binaries. The gate links about 150 protocol name maps (the gate must know every command name for the rights check). The switch tables that `table()` finds are message-name builders (for example `0x14016F570`, `0x14016F5A0`), not command handlers. Use the dispatch functions in this chapter.

### 1.2 Start (`StsGateStart` `0x140119A90`)

Called by the service init (CliGate `CliMain` `0x1400D6720`) with the gate name, the gate config and flags. In order:
1. Copies a 16-byte id to `0x1402AF268` and stores a hash of it at `0x1402AF264` ❓ (use not decoded).
2. Makes two 4096-entry tables `0x1402AF280`, `0x1402AF288` (`0x1400F4620`) ❓ (client id tables).
3. `0x14012F960(11, …)`, `0x14017E940(a4)` (NetAdjust init), `0x1400F9C80`, `0x14011C8C0`, `0x14017B2E0` (Geolocation init: `StsMonitorGeoIP` vtable `0x1401DD9E8`).
4. `StsConfigMonitorAdd(L"Gate.ini", CStsGateConfigNotify)` (notify vtable `0x1401B1688`, slot 0 `0x14010E510`).
5. `0x1401442D0("Gate", 0, …)`: route provider name `Gate`.
6. Without the "no route" flag: `0x140121A10` (route start with `Route.ini`) and registers 11 gate txn factories with `0x140121F50(name, factory)`: `inquiry`, `report`, `ipCheck`, `connectUser`, `connectApp`, `connectAppUser`, `disconnect`, `getRights`, `getAppGroupCode`, `getStsApi`, `idConversion`. With the flag: `StsRouteStart(factory 0x140248FA8, 0, …)`.
7. `0x1401446B0("ApiManage", …)`: ApiManage client (§1.5). Then `0x14012EC30(cleanup 0x140248FA0)` (`CStsGateCleanup`).

### 1.3 Gate txn classes

All gate txns derive from `CGateTxn` (vtable `0x1401B28F0`). Slot layout: 0 dtor, 1 = name (returns the factory name of §1.2), 2 `0x1400CDE0`, 3 `0x14011B360`, 4 = OnReply (success), 5 = OnError, 6 = release. So the gate txn layout differs from the framework txn (`00-framework.md` §2.5: slot 3/4).

| Class | Vtable | OnReply | OnError | Log in OnReply |
| --- | --- | --- | --- | --- |
| `CStsGateConnectAppTxn` | `0x1401B1230` | `0x14010F390` | `0x14010E7E0` | `StsGate: clientId(%u) does not exists`; reads `AppGroupCode` |
| `CStsGateConnectAppUserTxn` | `0x1401B12B0` | `0x14010F810` | `0x14010E7E0` | same |
| `CStsGateConnectUserTxn` | `0x1401B1330` | `0x14010FA00` | `0x14010E810` | |
| `CStsGateDisconnectTxn` | `0x1401B1030` | — (slot 4 is a no-op) | `0x14010E9A0` | logs feed type `user`/`channel`/`(unknown)` |
| `CStsGateGetAppGroupCodeTxn` | `0x1401B1430` | `0x14010FC30` | `0x14010EA90` | |
| `CStsGateGetRightsTxn` | `0x1401B13B0` | `0x14010FD00` | `0x14010EB10` | |
| `CStsGateGetStsApiTxn` | `0x1401B1530` | `0x1401100B0` | `0x14010EBB0` | reads `ProviderName`, `Command`, `Right`; error log `Gate: can't added default sts api info (%s, %s)` |
| `CStsGateIdConversionTxn` | `0x1401B14B0` | `0x1401104A0` | `0x14010E7E0` | |
| `CStsGateInquiryTxn` | `0x1401B10B0` | `0x1401105F0` | `0x14010E7E0` | `StsGate: clientId(%u) does not exist`; reads `Result`, `ReportId` |
| `CStsGateIpCheckTxn` | `0x1401B11B0` | `0x1401109B0` | `0x14010EDA0` | |
| `CStsGateReportTxn` | `0x1401B1130` | `0x140111350` | `0x14010F170` | `StsGate: clientId(%u), RiskMgmt rule is null` |
| `CStsGateListAdjustingFlagsTxn` | `0x1401B15E0` | `0x140110B70` | `0x14010EFF0` | `ListAdjustingFlags : no flags`; error `…:retry(%u),error(%u),product(%u),module(%u),line(%u)` |
| `CStsGateListProvidersTxn` | `0x1401B1570` | `0x140110CE0` | `0x14010F050` | `ListProviders : no providers` |
| `CStsGateListReservedProtocolsTxn` | `0x1401B1618` | `0x140110FE0` | `0x14010F0B0` | `ListReservedProtocols : no reserved protocols` |
| `CStsGateListRightsTxn` | `0x1401B15A8` | `0x1401111E0` | `0x14010F110` | `ListRights : no rights` |
| `CStsGateSubscribeTxn` | `0x1401B1650` | `0x1401115E0` | `0x14010F350` | |
| `CWatchTxn` (NetAdjust) | `0x1401DDBF8` | `0x14017F190` | `0x14010E7E0` | `NetAdjust: %s watch, bad reply, %u` |

Other gate classes: `CStsGateSocket` (`0x1401B0FA8`, factory `TStsInetSocketFactory<CStsGateSocket>`), conn records `StsGateConn` (`0x1401B0F08`), `StsGateSocketConn` (`0x1401B0F20`), `StsGateUserConn` (`0x1401B0F48`, 0x608 bytes, `../11-cligate-protocol.md` §10.3), `CStsDirectClientFactory` (`0x1401DE740`), Geolocation `CGeoIPRef` (`0x1401DD9C0`), `City`, `Coordinates`, `Geoloc`, `IPv4`, `Iso3166alpha2`.

Request targets of the gate txns (from the builders of §1.1). The pair of txn → builder is ❓ (matched by name only):

| Txn | Request | Builder |
| --- | --- | --- |
| inquiry | `RiskMgmt/Inquiry` (`Event`, target `@%s:%s`) | `0x140170E70`, `0x140170F50`, `0x140171060` |
| report | `RiskMgmt/Report` (`ReportId`) | `0x140171150`, `0x140171240` |
| ipCheck | `IpAuthz/CheckIp` ❓ (map `0x14029E798` has only `CheckIp`) | — |
| getAppGroupCode | `AppAuth/GetAppGroupCode` (`AppId`) | `0x14016F7B0` |
| getRights | `AppAuth/GetAppRights` (`AppId`) ❓ | `0x14016F830` |
| connectApp | `AppAuth/Authenticate` (`AppId`, `AppSecret`) ❓ | `0x14016F630` |
| connectUser | `Presence/Login` ❓ (`../11-cligate-protocol.md` §10.3) | `0x1400E7600` |
| getStsApi | `ApiManage/GetStsApi` (`GateName`) | `0x1400E4960` |
| ApiManage lists | `ApiManage/ListAdjustingFlags`, `ListProviders` (`PublisherId`), `ListReservedProtocols`, `ListRights`, `Subscribe` (`Scope`) | `0x1400E4A30`, `0x1400E4A80`, `0x1400E4B00`, `0x1400E4B50`, `0x1400E4BA0` |

### 1.4 Send path

| Function | Address | Fact |
| --- | --- | --- |
| `StsGateSendTxn(clientId, msg, txn)` | `0x14011A3A0` → `0x140113F90` | generic user path; checks and header changes in `../11-cligate-protocol.md` §11.1 (lines 2560–2711) |
| `StsGateSendOneWay(clientId, msg)` | `0x14011A630` | request without a request id: no txn |
| `StsGateSend` | `0x140114A10` | IP check state machine (socket conn+1108, states 0–4), `../11-cligate-protocol.md` §9.2 |
| `StsGateRefreshAppInfo` ❓ | `0x1401178E0` | called with arg 130 before each forward |
| `StsGateBindApp` ❓ | `0x14011A690` | binds the client app info (fail → CliGate `ErrBadConfig` 3030) |
| `StsGateRequestDisconnect` | `0x140113C60` | feed type `Channel` or `Presence`; bad type: log `StsGate::RequestDisconnect: invalid feed type: %d` |
| `0x140115160`, `0x140115500`, `0x140115DB0` | | errors `StsGate: missing app id at from header or conn`, `StsGate: missing app group code`, `StsGate: bad server data` |
| `0x140115AC0` | | `StsGate: invalid net adjust data type` |
| `0x14011A3D0` | | `StsGate: clientId not found, %u` |

Default timeout: 25000 ms (`0x61A8`) when the caller gives 0 (`0x140114A10`).

### 1.5 `Gate.ini` and ApiManage

| Key | Reader | Default | Use |
| --- | --- | --- | --- |
| `[ApiConfig] Source` | `0x14010E510` | `file` | `file` = rules from `Gate.ini`; `server` = rules from ApiManage (`GetStsApi`); other: log `invalid api config source` |
| `[Permission] AppPermissionFailCountLimit` | `0x14010E510` (`0x1400EEC60`) | 30 | global `0x1402AF2B8` ❓ (count of app permission failures before ...) |
| `[Commands.<protocol>] Command` (values) | `0x140111B20` | — | per-command flags (`FLAG_NONE`, `NO_REQUIRED`, …) |
| `[ApiRight] Right` (values) | `0x140111DB0` | — | rights (`RIGHT_UNKNOWN` = not known) |
| `[RiskMgmtRule.<name>] Rule` (values) | `0x140111F50` | — | RiskMgmt inquiry/report rules |
| `[Key] Key`, `[Name] Name` (values) | `0x140112460` | — | grid key rules. Logs `Gate: missing protocol on line %d`, `Gate: missing keyName on line %d`, `Gate: invalid key configuration %d` |
| `<section> Scope` (values) | `0x140112AB0` | — | ApiManage subscribe scope ❓ |
| `Auth.ini [UserName] UniqueProperty` | `0x1400E47B0` | 1 | user name is unique ❓ |

Forward rules (`0x140113720`, `0x1401139B0`): JSON rules per protocol (`s_gridTargets`). Logs `Gate: protocol(%S) is not defined in the s_gridTargets`, `json parse fail. (forwardRule:%s)`, `json parse fail. (gridKey:%s)`.
ApiManage subscribe: `SendSubscribeMsg` `0x140116670` (log `SendSubscribeMsg: not exist ApiManageSrvs`); spawn notice URL `http://%s/spawned/%s%d.%d.%d` (`0x14011BD90`, `0x14011C760`, log `invalid subscription`).

### 1.6 NetAdjust and Srv2IpAuthz

NetAdjust (`NetAdjust.cpp`) rewrites or answers requests before they go to the bus. Each function reads one body field and fails with the given text:

| Function | Reads | Error texts |
| --- | --- | --- |
| `0x14017BEC0` | `GameCode` | `NetAdjust: missing body`, `NetAdjust: no app id info at from header or conn` |
| `0x14017C1C0` | `GameCode` | `NetAdjust: invalid GameCode` |
| `0x14017C720` | `GameAccountId`, `GameCode` | missing body |
| `0x14017CB60` | `UserId` | no app id info |
| `0x14017CEA0` | `UserCenter` | missing body |
| `0x14017D0C0` | `AppGroupCode` | no app id info |
| `0x14017D410`, `0x14017FE60` | header `f` | `NetAdjust: missing from header` |
| `0x14017D500` | target `@login:` | `NetAdjust: msg is null`, `NetAdjust: target searching failed` |
| `0x14017DC60` | `Version`, `Values` | `NetAdjust: bulk find, missing transaction` |
| `0x14017DF90` | `Name` | `NetAdjust: find, missing transaction`, `NetAdjust: find, missing search name` |
| `0x14017E170` | `Location` | `NetAdjust: watch, no request body`, `NetAdjust: watch, has location element` |
| `0x14017E380` | `Database` | `NetAdjust: find, no request body`, `NetAdjust: find, missing database` |
| `0x14017E990` | protocols `Channel`, `Group`, `Presence` | `Auth` id 1 local reply `NetAdjust: access allowed` (`../11-cligate-protocol.md` §9.1) |
| `0x14017F5F0` / `0x14017F7F0` | `UserId` + `FolloweeId` / `FriendId` | missing body |
| `0x14017F9F0` | | `NetAdjust: missing user conn`, `NetAdjust: multiple to header values` |
| `0x14017FBF0` | `AppId` | missing body |
| `0x14017FD50` | target `@userid:%s` | `NetAdjust: user is not loggedIn` |
| `0x140180020` | `GameCode` | `NetAdjust: world protocol, no request body`, `NetAdjust: world protocol, get protocol failed` |

Srv2IpAuthz body parsers (`0x1400E6B00`–`0x1400E74E0`): fields `AppId`, `Description`, `IpCheck` (`true`), `IpToken`, `NetAddress` (`%03u.%03u.%03u.%03u`), `PageIndex`, to-header `UserId` (`Srv2IpAuthz: Zero UserId`). Each logs `Srv2IpAuthz: Empty body` or `Srv2IpAuthz: Not found <field>`.

## 2. CliGate

### 2.1 Role, start and config

- Role: TLS/RC4 front door for the game client (`../11-cligate-protocol.md`). Callers: game client (Auth and service protocols), backend services by bus (`Cli/Logout`, `Presence` command 14 `UserLogout`, `ApiManage` notices).
- `CliMain` `0x1400D6720`: `StsSetServerType(1001)`. If the game code config is empty: `ExeFatal("CliMain: no configuration (game code)")` (`CliMain.cpp` line 102). `StsConfigMonitorAdd(L"Game.ini")`, `0x1400D5F80` (`StsConfigMonitorAdd(L"CliGate.ini", CCliGateConfig)`). Only when the config is loaded (`0x14029D6C0` ≠ 0): `0x1401442D0("Cli", …)` (provider name `Cli`), `StsGateStart(…)`, `0x1400CEA70(0x14029D6C4)` (client listen ❓). Then `StsAppCommonStart(0)`.

| File, key | Reader | Default | Use |
| --- | --- | --- | --- |
| `CliGate.ini [CliGateOptions] RootKeyBits` | `0x1400D6030` | 2048 | root key size |
| `[CliGateOptions] ServerKeyBits` | `0x1400D6030` | 2048 | server key size |
| `[CliGateOptions] RootCaName` | `0x1400D6030` | `root.ca` | file name, prefix `off_1402471A0` (cert folder) |
| `[CliGateOptions] ServerCaName` | `0x1400D6030` | `plaync.ca` | server certificate file (content `keyhere`) |
| `[CliGateOptions] ServerPrName` | `0x1400D6030` | `plaync.pr` | server private key file (content `keyhere`) |
| `[IdleTimeout]` (values per game code) | `0x1400D6030` → `0x1400EE6B0` | 300 s, minimum 300 s | `GameCodeClientIdleTimeout` (vtable `0x1401A1800`); `../11-cligate-protocol.md` §12.5 |
| `Game.ini [Configurations]` (values) | `0x1400D67F0` | — | AppId per config name (`../11-cligate-protocol.md` §10.2) |

Logs: `Certificate file read success`, `Certificate file read fail` (`0x1400D6030`).
Timers: no service timer has a fixed period. One `StsTimerSet(…, -1, 0)` (remove) in `0x140149D80`. The client idle timeout uses the framework socket idle timer (`00-framework.md` §2.7).

### 2.2 Client dispatch (`CCliConn` slot 0 `0x1400D4AD0`)

`CCliConn` vtable `0x1401A0E28` (slot 0 OnMessage `0x1400D4AD0`, 1 `0x1400D48B0`, 2 `0x1400D4820`, 4 `0x1400D4730`). `CCliConnTxn` vtable `0x1401A0E70` (slot 1 `0x1400D53C0` reply dispatch).

Order of checks for each client request:
1. Protocol `Auth` and id 22 or 96: Reset (`0x1400D4390`).
2. Bind the client app info: `0x14011A690`. Fail → `ErrBadConfig` 3030. Then `0x1400FD4D0` (fail: `CliConn: invalid client app info`) and `0x1400FCA80` (fail: `CliConn: missing client info`).
3. `GameAccount/ListMyAccounts` (3) → `0x1400D0500`: forward with `StsGateSendTxn`, reply callback `0x1400D3340` (no body: `ErrBadParam` line 1160).
4. `Auth` switch (`0x1400E4780`, map `0x14029E4E0`): table below.
5. `SecondaryAuth/RegisterSecondPassword` (19) → `0x1400D16C0`.
6. `IpAuthz` (map `0x14029E7C0`): `AddIp` (1) when logged in → set `f` (`0x1400FD400`) and go on. `CheckIp` (5) and `RequestIpToken` (9): not logged in → `0x1400CFFE0` (session path); logged in → set `f` and go on.
7. Session path commands (allowed before login, use the Session of conn+328). If the conn is logged in, the command goes on to step 9:

| Command | Handler | Fail (no session / not allowed) |
| --- | --- | --- |
| `Profile/GetUserContact` (7) | `0x1400CFE90` | `ErrPermission` 49, module 1, line 2445 |
| `Mail/Mail` (1) | `0x1400D1260` | 49, line 2465 |
| `Gate/LookupGeolocation` | `0x1400CF780` | 49, line 2481 |
| `Slot/ListSlots` (4) | `0x1400D0620` | 49, line 2296 |
| `Purchase/Purchase` (1) | `0x1400D1530` | 49, line 2504 |
| `Purchase/PagePurchaseHistory` (3) | `0x1400D13A0` | 49, line 2515 |
| `PurchaseLimit/ListGoodsLimits` (23) | `0x1400D0370` | 49, line 2532 |
| `VirtualCurrency/GetBalance` (3) | `0x1400CF8C0` | 49, line 2549 |

   The session-path handlers remove the client header `p` (112), build a new frame with the Session (and for `Profile` the UserId conn+248, `0x1400D29E0`), and send it with `0x1400CEF30` (25000 ms, callback `ClientReply` `0x1400CED50`). The `CF8C0` form handlers forward with `StsGateSendTxn` when logged in.
8. `Auction` (map `0x14029E6E0`): `Sell` 1, `CancelSale` 2, `Bid` 4, `RestractBid` 5, `PageMySales` 8, `PageMyBids` 9, `PageMySaleHistory` 10, `PageMyBuyHistory` 11: not logged in and the request has an id → `ErrPermission` 49, module 1, line 902. Other Auction commands go on.
9. Generic forward: if not `IpAuthz/SendIpAuthCode` (11): `0x1401178E0(clientId, 1, 0, 0, 0, 130)`. Request with an id → `CCliConnTxn` (0x50 bytes) with callback `ClientReply` `0x1400CED50` → `StsGateSendTxn`. Without an id → `StsGateSendOneWay`. `IpAuthz/SendIpAuthCode` goes to `0x1400D1CE0` (reads `Locale`, `Method`; no body → `ErrBadParam` line 2567).

Auth switch (step 4). Lines are module 1.

| Id | Command | Handler | Reply callback | Errors (code, line) | Timeout |
| --- | --- | --- | --- | --- | --- |
| 18, 20 | GetMyUserInfo, GetUserInfo | `0x1400CFA50` | `ClientReply` | 49 line 2331 | 25000 |
| 21 | KeyData | `0x1400D0130` | `0x1400D2E60` | 10 line 1333/1336, 49 line 1338, 10 line 1342 | 25000 |
| 22 | LoginStart | `0x1400D09E0` | `0x1400D4030` | 10 line 1264/1268 | 25000 |
| 23 | LoginFinish | `0x1400D07B0` | `0x1400D3B20` → `0x1400D2A90` → `0x1400D35C0` | 49 line 1681/1686, 10 line 1691 | 25000 |
| 30 | RequestGameToken | `0x1400D1940` | `ClientReply` | — | 25000 |
| 32 | RequestToken | `0x1400D1B10` | `ClientReply` | — | 25000 |
| 35, 36, 38 | VerifyMobiOtp, VerifySecondPwd, VerifySecondaryAuth | `0x1400D2380` | | 49 line 1982, 52 line 1988, 10 line 1993, 52 line 2018, 10 line 2025; texts `CliConn: verify secondary auth, missing comma` | 25000 |
| 37 | GetSecurityCardCodeIndex | `0x1400CFBA0` | | 49 line 1931, 52 line 1937, 10 line 1942/1950; `CliConn: invalid auth type` | 25000 |
| 46 | CreateAccount | `0x1400CF3E0` | `0x1400CF120` | 10 lines 1196, 1200, 1203, 1206, 1211; reply 10 line 1240/1244/1253 | 25000 |
| 58 | TokenKeyData | `0x1400D1ED0` | `0x1400D55A0` | 10 line 1482/1485, 49 line 1487, lines 1497/1507/1510; reply 4 line 1555/1630/1637, 10 line 1566 | 25000 |
| 96 | LoginTokenStart | `0x1400D0B60` | local | 10 lines 1451/1456/1460, 4 line 1466, line 1475 | — |
| 97 | LogoutMyClient | `0x1400D1150` | local | 3007 line 1736; OK line 1733 | — |
| 99 | WaitForAllowedLogin | `0x1400D27A0` | `ClientReply` | 49 line 2269 (no session) | **600000** (`0x927C0`) |
| other | | generic forward (step 9) | `ClientReply` | | 25000 |

`ClientReply` `0x1400CED50`: timeout 42 → report with tag `Cli` (module 1, line 981 ❓ for the report record).

### 2.3 Bus messages to CliGate (`0x1400D6A00`)

| Protocol / command | Handler | Fact |
| --- | --- | --- |
| `ApiManage` `Updated` (36), `Flush` (37), `FlushDefinitions` (38) (map `0x14029E6C0`) | `0x1400D6A00` | reload the gate API rules (`ApiMainVersion`, `ApiHotfixVersion`; `0x140117320`, `0x1401172F0`, `0x1401173C0`); reply `<Reply>` |
| `Cli/Logout` (1) (map `0x14029E718`) | `0x1400CEB30` → `0x1400D0EE0` | `../11-cligate-protocol.md` §12.4; error 10 line 2600 |
| `Presence/UserLogout` (14) (map `0x14029E940`) | `0x1400CEB30` | same path as `Cli/Logout` (closes `../11-cligate-protocol.md` §12.4 ❓ row) |

Other CliGate own maps (no handler, used for body enums): `0x14029E6F8` Auction sort order (15: `UnitPriceAsc` … `BidPriceDesc`), `0x14029E738` (`LogLines` 1, `LogEvent` 2), `0x14029E758` (`Forward` 1, `Find` 2, `FindBulk` 3, `Set` 4, `Flush` 5, `FlushInternal` 6), `0x14029EA48` (`char`, `activity`, `tag`, `customize`).

### 2.4 DB procs

CliGate calls no DB proc (recipe `dbprocs()` ❓ not run; no `DbProcBegin` form found in the dispatch).

### 2.5 Worked example: Auth/LoginStart

1. Client: `POST /Auth/LoginStart STS/1.0`, `s: 1`, body `<Request><LoginName>user@example</LoginName></Request>`.
2. `0x1400D4AD0`: protocol `Auth`, id 22 → Reset `0x1400D4390`, bind app info, `0x1400D09E0`.
3. `0x1400D09E0`: builds `Auth/LoginStart` with `<LoginName>` and `<NetAddress>`; `0x1400CEF30(conn, msg, txn{cb 0x1400D4030, id 1}, 25000)`.
4. AuthSrv reply: `<Reply><UserId>…</UserId><UserCenter>1</UserCenter><Session>…</Session><KeyData>…</KeyData></Reply>`.
5. `0x1400D4030`: conn+248 = UserId, conn+240 = 1, conn+328 = Session; client gets `STS/1.0 200 OK`, `s: 1R`, `<Reply><KeyData>…</KeyData></Reply>`.

## 3. SessionGateSrv (SessionGateSrv addresses)

Function list: 1364 service functions, 443 not in `CliGate.tsv`.

### 3.1 Config

- Service start `0x1400D2A40`: `StsSetServerType(0x3F6 = 1014)`, `StsConfigMonitorAdd(L"Session.ini")`, builds the command table (`0x1400D1280`, §3.2), then `StsAppCommonStart`. It also names the protocols `Session` and `Access` (bus subscriptions ❓).
- `Session.ini` reader `0x1400D2B20` (runs at start and on each file change):

| Section | Key | Default | Use | Global |
| --- | --- | --- | --- | --- |
| `Session` | `Timeout` | 3600 (s) | session idle timeout; stored ×1000 as ms | `dword_1402AA47C` |
| `Session` | `PersistentSessionTimeout` | 2592000 (s, 30 days) | expiry of persistent sessions; stored ×1000 as ms; read by `0x1400D3700` | `dword_1402AA480` |
| `OnOff` | `ValidateHasAccess` | 0 | 1 → flag = 1; read by `0x1400D36F0`; given as arg 2 to the gate client-state setter `0x14011F700` | `dword_1402AA484` |
| `OnOff` | `EnableTimeoutLog` | 0 | 1 → flag = 1 | `dword_1402AA488` |
| `LocalLog` | `Use` | 0 | not 0 → open the local log `qword_1402534D0` (`0x1400D2780`) | |
| `LocalLog` | `InfixFilename` | empty | local log file name infix | |
| `LocalLog` | `ExtName` | `log` | local log file extension | |
| `LocalLog` | `Period` | 0 | log file period; ≥ 4 → log `Session.ini: invalid log period (period : %d)` (value is still used) | |
| `LocalLog` | `Activate` | 0 | local log on/off | |

  At the end: if `0x1401386F0()` is not 0 (storage cache present ❓), call `0x14012AC00("Session", 24 * TimeoutMs)` (cache entry lifetime = 24 × timeout ❓).
- Expiry time (`0x1400D8040`): `now_seconds + timeout_ms / 1000` (`0x1400E1420`/`0x1400E13B0` = current time in seconds ❓).
- Local log `0x1400D2940` (only if `qword_1402534D0` is set, test `0x1400D2920`): for example `SESSION_CREATE,UserId=%s,SessId=%s,UserName=%s,addr=%s,status=%d`.

### 3.2 Command handler objects

The service commands are handled by static handler objects (`COn*`). Each object is made by a static initializer (`0x1400C7810`–`0x1400C7A10`). Slot 1 of each vtable is the handler.

Command table builder `0x1400D1280` (called from service start): for each row it makes the key `"%s/%s"` (protocol, command name), stores the object and the command id (+56), and inserts it in the map `dword_140253450` (`0x140111A60`). Lookup: `0x1400C7740` (vtable slot of the conn class at `0x1401A7800` ❓). The name functions are `0x1400F2A80` (Auth, 110 ids), `0x1400F53E0` (MobileAuth, 37 ids), `0x1400FA1A0` (Session, 13 ids: 1 `CheckSession`, 2 `CreateSession`, 3 `Event`, 4 `Login`, 5 `LoginFinish`, 6 `Logout`, 7 `VerifySecondaryAuth`, 8 `RevokePersistentSession`, 9 `Validate`, 10 `UpdateTimeout`, 11 `Relogin`, 12 `Forward`, 13 `Invalid`).

| Command | Id | Object | Class | Handler |
| --- | --- | --- | --- | --- |
| `Auth/LoginWebStart` | 26 | `0x140253010` | `COnAuthLogin` | `0x1400CE6A0` |
| `Auth/LoginTrusted` | 25 | `0x140253050` | `COnAuthLogin` | `0x1400CE6A0` |
| `Auth/LoginToken` | 53 | `0x140253090` | `COnAuthLogin` | `0x1400CE6A0` |
| `Auth/LoginWebTokenStart` | 54 | `0x1402530D0` | `COnAuthLogin` | `0x1400CE6A0` |
| `MobileAuth/Login` | 1 | `0x140253110` | `COnAuthLogin` | `0x1400CE6A0` |
| `MobileAuth/LoginTrusted` | 2 | `0x140253150` | `COnAuthLogin` | `0x1400CE6A0` |
| `Session/RevokePersistentSession` | 8 | `0x1402533D0` | `COnSessionRevoke` | `0x1400D00F0` |
| `Session/CreateSession` | 2 | `0x140253290` | `COnSessionCreate` | `0x1400CF0A0` |
| `Session/Logout` | 6 | `0x140253190` | `COnSessionLogout` | `0x1400CF6A0` |
| `Session/LoginFinish` | 5 | `0x1402531D0` | `COnSessionLoginFinish` | `0x1400CF410` |
| `Session/VerifySecondaryAuth` | 7 | `0x140253210` | `COnSessionVerifySecondaryAuth` | `0x1400D0AB0` |
| `Session/Validate` | 9 | `0x140253250` | `COnSessionValidate` | `0x1400D05D0` |
| `Session/CheckSession` | 1 | `0x1402532D0` | `COnSessionCheck` | `0x1400CEC60` |
| `Session/Event` | 3 | `0x140253390` | `COnEventRequest` | `0x1400CE910` |
| `Session/UpdateTimeout` | 10 | `0x140253310` | `COnSessionUpdateTimeout` | `0x1400D0470` |
| `Session/Relogin` | 11 | `0x140253350` | `COnSessionRelogin` | `0x1400CFAF0` |
| `Session/Forward` | 12 | `0x140253410` | `COnForward` | `0x1400CEA40` |

`Session/Login` (4) and `Session/Invalid` (13) have no row: no handler.

Handler call form: `handler(this, msg, ?, conn)`. `conn` (SgConn) fields used: +24 client address text, +652 session id (16 bytes), +676 authenticated flag, +680 a second flag ❓. Errors use `ReplyError(msg, err, text)` with `err = {code, module 1 | line << 32}`.

Session object (`SgSession.cpp`, 0xD98 bytes): `0x1400D7FC0(conn, conn+652, Location)` allocates it, runs the constructor `0x1400D37F0`, sets `Location` (vtable +464) when not empty, and registers it (`0x14012B050`). `0x1400D10D0(body, session)`: reads `Persistent` (int) → vtable +80; if the session is persistent (+72) → +120; then sets the persistent expiry (+88) = expiry(`PersistentSessionTimeout`). `0x1400D7310(session, body)` copies `ExternalAccount`, `AuthProviderCode`, `ExternalAccountKey`, `ExternalAccountName`, `ExternalAccountData`, `ExternalUserAppKey`, `CreateStatus`.

| Class | Vtable | Handler (slot 1) | Error texts in the handler |
| --- | --- | --- | --- |
| `COnAuthLogin` | `0x1401A8CB8` | `0x1400CE6A0` | |
| `COnSessionCheck` | `0x1401A8E90` | `0x1400CEC60` | `SgConn: no body` |
| `COnSessionCreate` | `0x1401A8DC8` | `0x1400CF0A0` | `SgConn: no body`, `SgConn: session create, null userId`, `SgConn: session create, missing id, center, or name` |
| `COnSessionLoginFinish` | `0x1401A8CE8` | `0x1400CF410` | `SgConn: no session`, `SgConn: no body` |
| `COnSessionLogout` | `0x1401A8CD0` | `0x1400CF6A0` | |
| `COnSessionRelogin` | `0x1401A8EF8` | `0x1400CFAF0` | `SgConn: no body`, `SgConn: no session secret`; calls `GetPersistentSession` |
| `COnSessionRevoke` | `0x1401A8F50` | `0x1400D00F0` | `SgConn: no body`, `SgConn: no valid appId`, `SgConn: no userId of invalid value` |
| `COnSessionUpdateTimeout` | `0x1401A8EA8` | `0x1400D0470` | `SgConn: no body`, `SgConn: missing or invalid parameter` |
| `COnSessionValidate` | `0x1401A8DB0` | `0x1400D05D0` | |
| `COnSessionVerifySecondaryAuth` | `0x1401A8D28` | `0x1400D0AB0` | `SgConn: session not found`, `SgConn: body not found`, `SgConn: invalid auth type` |

Other `SgConn` checks: `0x1400CE910` (`SgConn: no body`), `0x1400D1140` (`SgConn: wrong datacenter`, `SgConn: missing session ID`, `SgConn: invalid to header`), `0x1400D1640` (`SgConn: Invalid msg, %s/%s`, `SgConn: Invalid from header (app info required), %s/%s`). `SgMain` `0x1400D3370`: `SgMain: Invalid from header, appId, appSecret`. `0x1400D6540` (`CWebSession`, vtable `0x1401A9D20`): `WebSrv: invalid user ref in row element`.

### 3.3 Internal txn map `SgTxn` (`0x1402AA498`, 18 ids)

0 `SgTxn`, 1 `CheckPersSession`, 2 `ChangeMyUserName`, 3 `ChangeUserStatus`, 4 `AssociateMyExternalAccount`, 5 `PresNotify`, 6 `AccountLogin`, 7 `AccountLoginInternal`, 8 `AccessGetPermission`, 9 `LoginFinish`, 10 `VerifySecondaryAuth`, 11 `LoginCancel`, 12 `CreateSession`, 13 `CreatePersistentSession`, 14 `RemovePersistentSession`, 15 `RevokePersistentSession`, 16 `Forward`, 17 `GetUserInfo`. Parse `0x1400DD080` (callers `0x1400D2900`, `0x1400DA580`). The `table()` handlers for this map (`0x1400D99B0`…`0x1400DA4F0`) are not checked ❓ (in CliGate the same form gave name builders).

Txn classes (base `CSgBaseTxn` `0x1401A8A08`; slot 4 = OnReply, slot 5 = OnError):

| Class | Vtable | OnReply | OnError | Texts |
| --- | --- | --- | --- | --- |
| `CSgTxn` | `0x1401AA5A0` | `0x1400DB670` | `0x1400DA7D0` | |
| `CSgTxnAccsGetPermission` | `0x1401A99D8` | `0x1400DB770` | `0x1400DA820` | `SgTxn: missing body`, `SgTxn: missing accessMask`, `SgTxn: missing grid data` |
| `CSgTxnAcctLogin` | `0x1401AA7F8` | `0x1400DB870` | `0x1400DA870` | `SgTxn: missing body` |
| `CSgTxnAcctLoginCancel` | `0x1401A9A50` | `0x1400DB670` | `0x1400DA8C0` | `Login cancel failed - session: %s, user: %s, userId: %s` |
| `CSgTxnAcctLoginFinish` | `0x1401A8A80` | `0x1400DB9C0` | `0x1400DA9A0` | `SgTxn: missing body` |
| `CSgTxnAcctLoginInternal` | `0x1401AA870` | `0x1400DBB40` | `0x1400DAA30` | |
| `CSgTxnAcctVerifySecondaryAuth` | `0x1401A8AF8` | `0x1400DC280` | `0x1400DAAC0` | |
| `CSgTxnAssociateMyExternalAccount` | `0x1401AA780` | `0x1400DC350` | `0x1400DAB50` | `SgTxn: missing winnerId`, `SgTxn: missing connect status` |
| `CSgTxnChangeMyUserName` | `0x1401AA690` | `0x1400DC5B0` | `0x1400DABB0` | |
| `CSgTxnChangeUserStatus` | `0x1401AA708` | `0x1400DC760` | `0x1400DAC10` | |
| `CSgTxnCheckPersSession` | `0x1401A8BE8` | `0x1400DC860` | `0x1400DAC70` | |
| `CSgTxnCreatePersistentSession` | `0x1401A9AC8` | `0x1400DCCD0` | `0x1400DACC0` | |
| `CSgTxnForward` | `0x1401AAA78` | `0x1400DCE70` | `0x1400DAD30` | |
| `CSgTxnPresNotify` | `0x1401AA618` | `0x1400DCEA0` | `0x1400DAD80` | |
| `CSgTxnRevokePersistentSession` | `0x1401AA8E8` | `0x1400DB670` | `0x1400DADF0` | |
| `CSgTxnSessionCreate` | `0x1401A8B70` | `0x1400DCF00` | `0x1400DAE40` | |
| `CCheckPersSessionTxn` | `0x1401AA978` | `0x1400DA640` | `0x1400CE680` | |

OnReply steps (`a3` = session, `a4` = reply msg; "Err c/3/L" = error code `c`, module 3, line `L`; "pass through" = remove header 116 (To) from the reply (msg vtable +216), copy it (`0x140106A30`) and `StsRouteReply(requester, err 0, copy)`):

| Txn | OnReply | Steps |
| --- | --- | --- |
| `AcctLogin` | `0x1400DB870` | 1. Session vtable +480 (`Create`: requester, reply, txn+24). 2. True → local log level 2 `AUTH_LOGIN,UserId=%s,SessId=%s,UserName=%s` (UserId = session +288, SessId = +272, name = +304). 3. False → `0x14012AEA0` (release held object, §3.7), then `ReplyError` Err 4/3/634 `SgTxn: missing body` (`0x1400D94D0`). |
| `AcctLoginFinish` | `0x1400DB9C0` | 1. No reply body → Err 4/3/794 `SgTxn: missing body`. 2. Body has `AuthType` (Auth asks for a secondary auth) → pass through (line 800). 3. Else → session vtable +496 (`0x1400D6290`, login finish, see below). |
| `AcctVerifySecondaryAuth` | `0x1400DC280` | Pass through (line 856). No other step. |
| `CheckPersSession` (relogin) | `0x1400DC860` | Default result 0/3/1742. 1. No session → go to step 6 with error 0. 2. No reply body (`0x1400F3BF0`) → Err 4/3/1745. 3. `EffectiveUntil` (time text, `0x140150F80`) < now (`0x1400E1420`) → Err 3102/3/1751 (`ErrSessionNotFound`). 4. Session vtable +136 (check `SessionSecret`) fails → Err 3101/3/1757 (`ErrBadSession`) and proc `RemovePersistentSession(session +272)` (`0x1400D83D0`). 5. `UserId` missing or not a GUID → Err 3101/3/1763. Else build `Auth/LoginInternal` (`0x1400F2690`: `Request{AuthProviderCode}`, header 116 = UserId, header 102 = request address of the session) and send it (`0x14012B0A0`, 25000 ms) with `CSgTxnAcctLoginInternal` (`0x1400D90E0`, 0xB0 bytes). 6. Error ≠ 0 → session vtable +512 (`0x1400D6870`, end session); `0x14012AEA0`; `StsRouteReply(requester, err, no body)`. Whether the success path also reaches step 6 is not checked ❓. |
| `AcctLoginInternal` | `0x1400DBB40` | 1. Read `UserStatus`, `UserCenter` (int), list `Roles/RoleId`, `PcCafeCode`, `LoginName`, `UserName`. 2. `UserName` not empty → session vtable +360 (set name). 3. New session id (`CoCreateGuid`, `0x1400E15A0`); copy the external account (`0x1400D7310`). 4. Build `Session/CreateSession` (`0x1400F9DA0`: `Request{UserId, UserCenter, UserStatus, UserName, LoginName, ExternalAccount{…}}`). Session vtable +304 (name) not empty → send it direct (`0x14012B0A0`, 25000 ms, txn 0x68 bytes). Else → `0x1400CE390`: wrap it with a storage refer-to `AcctDb.GetNameById` (key `$<new session id>`, columns `UserId`, `UserCenter`, `UserName`; `0x1400F4040`), To = new session id text, txn `CSgTxnSessionCreate` (id 16/12) ❓ (exact use of the key not checked). 5. Local log `AUTH_LOGIN_INTERNAL,UserId=%s,OldSessId=%s,NewSessId=%s,UserName=%s,UserCenter=%d`. |
| `SessionCreate` | `0x1400DCF00` | Pass through (line 1158); session vtable +24 (set state 8); +512 (end session); `0x14012AEA0`. |
| `CreatePersistentSession` | `0x1400DCCD0` | Default Err 4/3/1180. 1. Session, reply body and `SessionId` (GUID) needed; `SessionId` must equal session +272 (`0x1400D41E0`). 2. `AuthProviderCode` present → session vtable +216. 3. Result 0/3/1205; then session vtable +504 (AccessGetPermission send). 4. Error ≠ 0 → session vtable +512. `StsRouteReply(requester, err, no body)`. |
| `AccsGetPermission` | `0x1400DB770` | 1. No body → `0x14012AEA0`, Err 4/3/698 `SgTxn: missing body`. 2. No `AccessMask` → `0x14012AEA0`, Err 4/3/709 `SgTxn: missing accessMask`. 3. Session present → `0x1400D95D0` (send `PresNotify`); no session → Err 4/3/713 `SgTxn: missing grid data`. |

Login chain after `Session/LoginFinish` (all `CWebSession` vtable `0x1401A9B08` slots):
- +496 `0x1400D6290` (login finish): reads the Auth reply `UserStatus` (→ session +144), `Roles/RoleId` (appended to the u32 array at +344, count +356), `PcCafe/PcCafeCode` (≤ 32 chars → +368). Not persistent (vtable +72 = 0) → +504. Persistent → state 3 (vtable +24) and proc `CreatePersistentSession` (`0x1400D8160`); its reply (`0x1400DCCD0`) then calls +504.
- +504 `0x1400D5E10` (AccessGetPermission send): txn `CSgTxnAccsGetPermission` (0x68, `0x1400D8F60(…, −1, 8, …)`), request address; if session byte +545 ≠ 0 → gate library `0x14011F960(gateClient, txn, session+280, …)` ❓ (args not read).
- `0x1400D95D0` (PresNotify send): txn `CSgTxnPresNotify` (0x68); gate library `0x14011FD60(…)` with the session id text and the address text (54 chars); local log level 1 `PRESENCE_LOGIN,UserId=%s,SessId=%s,UserName=%s,addr=%s`.
- +512 `0x1400D6870` (end session), by state (session +148): state ≥ 8 and persistent → proc `RemovePersistentSession(+272)`. State 2 (login pending) → `Auth/LoginCancel` (`0x1400F2520`: `Request{Session, Reason = 0, AppIds}`, header 116 = UserId) with `CSgTxnAcctLoginCancel` (gate send, 25000 ms). State 3 or 4 → `RemovePersistentSession(+272)`. State ≥ 5 → `0x1401218C0(gateClient)` (gate library), clear the session id (+220) and +280/+288, vtable +24 (state 0). Then local log `SESSION_LOGOUT,UserId=%s,SessId=%s,UserName=%s,addr=%s,state=%d`.

`0x1400DB3A0`: `SgTxn: missing body`, `SgTxn: invalid data`. Storage entities `Srv2Storage::App`, `AppGroupCode`, `Entity`, `EntityEx`, `Owner`, `Role`, `TEntity` (vtables `0x140222DF0`–`0x140222E70`). `Srv2Forward::Parse` `0x1400F4980` (`refer-to missing database`, `updates to non-XML message`, `missing body`).

### 3.4 DB procs

Recipe `dbprocs()`: 7 call sites, 6 procs, all in DB `SessionDb`, schema `v2015102301`, flag 0. Expected 6 (`../procs.tsv`, `PlatformSessionDb`); found 6. List: `01-procs.tsv`.

| Proc | Builder | Call site |
| --- | --- | --- |
| `CreatePersistentSession` | `0x1400D8160` | `0x1400D825D` |
| `ChangeUserId` | `0x1400D85E0` | `0x1400D863C` |
| `RevokePersistentSession` | `0x1400D8480` | `0x1400D84E4` |
| `RevokePersistentAppSession` | `0x1400D8480` | `0x1400D855B` |
| `GetPersistentSession` | `0x1400CE2B0` | `0x1400CE313` |
| `GetPersistentSession` | `0x1400CFAF0` (`COnSessionRelogin`) | `0x1400CFE58` |
| `RemovePersistentSession` | `0x1400D83D0` | `0x1400D842C` |

### 3.5 Handlers step by step

Common terms:
- `body` = request body (msg vtable +96). Body field read: text (+192), int (+208).
- "Err c/L" = error code `c` (`errcodes.tsv`: 4 `ErrBadData`, 10 `ErrBadParam`, 3308 `ErrLoginNotPending`), module 1, line `L`. `ReplyError` sends it with a text; `StsRouteReply(…, err, 0)` sends it without a text.
- "Refresh gate client": `0x14011FB40(gateClient, conn)` and `0x14011F700(gateClient, ValidateHasAccess, 0, 0, conn+676, conn+680)` (gate library; CliGate `0x140117D20`, `0x1401178E0`). `gateClient` = session vtable +280.
- "Gate send": `0x140121E60(gateClient, msg, txn, 25000)` → gate library `0x14011C700` (CliGate `0x140114980`; log `StsGate: send adjusted, invalid out msg`). `0x140121E30(gateClient, msg, txn)` and `0x140121E40(…, 25000)` are the same family ❓.
- "Persistent lookup" `0x1400CE2B0(reqId, conn, _RequestAddr)`: proc `SessionDb.GetPersistentSession(conn+652)`; the reply goes to `CCheckPersSessionTxn` (0x300 bytes, `0x1400D8D60`) through `0x1400D0F00`. On error code 42 it logs `SessionDb`/`GetPersistentSession` (`0x1400D35B0`) and sends the error to the requester (`0x1400DA640`). What happens on success (rebuild of the session, new run of the command) is not read ❓.
- `sub_140109DC0(x) != 0`: the byte at `x`+79 is not 0 (verified, 13-byte function). The meaning "the session has a remote owner (another SessionGateSrv instance)" is not verified ❓. Then the handler forwards the request to the owner with a `CSgTxn` (`0x1400D8FA0`, 0xF0 bytes, name from `0x1400FA1A0`) or, with no session, with a `CCheckedTxn` (`0x1400D8D90`, 0x300 bytes) through `0x1400D0F00`.

| Command | Handler | Steps |
| --- | --- | --- |
| Auth/MobileAuth login (6 rows) | `0x1400CE6A0` | 1. No body → Err 4/271 (no text). 2. Read `_RequestAddr`, `Location`. 3. Make a session (`0x1400D7FC0`). 4. Write `TokenInfo` into the body ❓ (source value not read). 5. Refresh gate client. 6. `Persistent` (`0x1400D10D0`). 7. Txn `CSgTxnAcctLogin` (`0x1400D9040`, 0xF0 bytes; gets the request id, source and command of the msg); set request address (`0x1400DCFE0`). 8. Copy the msg (`0x140106A30`) and gate send it (25000 ms) to Auth. The Auth reply goes to `CSgTxnAcctLogin` OnReply `0x1400DB870`. |
| `Session/CheckSession` (1) | `0x1400CEC60` | 1. No body → Err 10/673 `SgConn: no body`. 2. No session → persistent lookup. 3. `Location` not empty → session vtable +464. 4. Remote owner → refresh gate client, forward `Session/CheckSession` (`CSgTxn`). 5. Else reply `Reply{UserId, UserCenter, UserName, UserStatus, [ExternalAccount{AuthProviderCode, ExternalAccountKey, ExternalAccountName, ExternalUserAppKey}] (only if session +144), LoginName}`; `LoginName` = session +312, or empty. Error 0. |
| `Session/CreateSession` (2) | `0x1400CF0A0` | 1. No body → Err 10/594 `SgConn: no body`. 2. Read `UserId` (text → GUID `0x1401097E0`), `UserName`, `UserCenter` (int), `UserStatus` (int), `Location`. 3. Bad or missing `UserId`, empty `UserName`, or `UserCenter` = 0 → Err 10/610 `SgConn: session create, missing id, center, or name`. 4. All-zero GUID → Err 10/613 `SgConn: session create, null userId`. 5. Remove one leading `:` from `UserName`. 6. Make a session; bind it to the conn (`0x1400D1080`); refresh gate client with authenticated = conn+676 or body `IsAuthenticated`. 7. `Persistent`; copy external account fields (`0x1400D7310`). 8. Session vtable +480 (`Create`: requester, msg, `_RequestAddr`) sends the reply ❓. 9. Local log `SESSION_CREATE,…`. |
| `Session/Event` (3) | `0x1400CE910` | 1. No body → Err 10/890. 2. No session → persistent lookup. 3. Read `After` (int). Session vtable +408 ≠ 0 → vtable +416(requester, `After`); else +432(requester, `After`). With no session the handler reads `_RequestAddr` for the lookup. |
| `Session/LoginFinish` (5) | `0x1400CF410` | 1. No session → Err 10/397 `SgConn: no session`. 2. Session not in login state → Err 3308/400 (`ErrLoginNotPending`). 3. No body → Err 10/406. 4. Read `RegisterVerifiedIp`; txn `CSgTxnAcctLoginFinish` (`0x1400CDEB0`, 0x68 bytes); request address. 5. Build `Auth/LoginFinish` (`0x1400F2770`) and gate send it (25000 ms). |
| `Session/Logout` (6) | `0x1400CF6A0` | 1. No body → Err 10/322. 2. Remote owner → forward `Session/Logout` (`CSgTxn`) or, with no session, `CCheckedTxn`. 3. Session → session vtable +456 (logout: conn+208, owner, authenticated); true → reply `Reply` (empty), then `0x14012AEA0` (release held object, §3.7); false → Err 10/379. 4. No session → read the msg header 116 (`t`, the To header); remove a leading `$`; text → GUID; proc `RemovePersistentSession(GUID)` (`0x1400D83D0`); reply error 0 (line 371). No To header → Err 4/359. |
| `Session/VerifySecondaryAuth` (7) | `0x1400D0AB0` | 1. No session → Err 10/434 `SgConn: session not found`. 2. Session not in login state (vtable +344 = 0) → Err 3308/437. 3. Read `AuthType` (int, not 0), `Value` (text, not empty), `Force` (int). Missing → Err 10/454 (`SgConn: body not found` / `SgConn: invalid auth type`). 4. Txn `CSgTxnAcctVerifySecondaryAuth` (`0x1400CDEF0`, 0x68 bytes). 5. Build `Auth/VerifySecondaryAuth` (`0x1400F2880`: session id text (37 chars), `AuthType`, `Value`, `Force`, a 54-byte value from `0x1400DFA90` ❓) and gate send it (25000 ms). |
| `Session/RevokePersistentSession` (8) | `0x1400D00F0` | 1. No body → Err 10/916. 2. `UserId` text → GUID; bad → Err 10/921 `SgConn: no userId of invalid value`. 3. Read the list `AppIds/AppId` (u32 each) into an array. A bad `AppId` → Err 10/938 `SgConn: no valid appId`. 4. Proc call `0x1400D8480(UserId, apps)`: no apps → `RevokePersistentSession`, else `RevokePersistentAppSession`. 5. Reply `Reply` (empty), error 0. |
| `Session/Validate` (9) | `0x1400D05D0` | 1. `0x140103E00(msg, 1)` (inner request present ❓) else end. 2. No session → persistent lookup. 3. Parse the inner request (`0x1400FA2A0`, 0x98-byte header). Refresh gate client. 4. By inner `Auth` command: `ChangeMyUserName` (5) needs `UserNameNew` (else Err 10/537) → `CSgTxnChangeMyUserName` (`0x1400D91D0`, 0xB0); `AssociateMyExternalAccount` (63) → `CSgTxnAssociateMyExternalAccount` (`0x1400D9180`, 0x68); `ActivateAccount` (2) / `DeactivateAccount` (13) → `CSgTxnChangeUserStatus` (`0x1400D9240`, 0x70; status 1 / 2, verified); `RequestToken` (32) with `AuthProviderCode` and session +144 (has external account) → `CSgTxn` for the inner command, else Err 10/515. 5. Send with `0x140121E10(…, msg, txn)`. Other inner commands → `StsRouteReply(requester, Err 4/1/579, no body)` (`ErrBadData`, no text). |
| `Session/UpdateTimeout` (10) | `0x1400D0470` | 1. No body → Err 10/744. 2. No session → persistent lookup. 3. `TimeoutMs` (int, default −1). < 0 → Err 10/757 `SgConn: missing or invalid parameter`. 0 → 0xFFFFFFFF (`quirk_update_timeout_zero`). 4. Session vtable +56 (set timeout). Reply error 0. |
| `Session/Relogin` (11) | `0x1400CFAF0` | 1. No body → Err 10/775. 2. Read `_RequestAddr`, `Location`, `SessionSecret`; no secret → Err 10/781 `SgConn: no session secret`. 3. Remote owner → forward `Session/Relogin` (`CSgTxn`) or `CCheckedTxn`. 4. No session → make one, bind it, refresh gate client, proc `GetPersistentSession(conn+652)` (call site `0x1400CFE58`); the reply checks the secret ❓. |
| `Session/Forward` (12) | `0x1400CEA40` | 1. `0x140103E00(msg, 1)` else end. 2. Make a session object (`Location` = none). 3. Parse the inner request (`0x1400FA2A0`); fail → Err 10/967. 4. Refresh gate client; `CSgTxnForward` (`0x1400D9290`, 0xF0; inner protocol + command); request address; `0x140121E40(…, msg, txn, 25000)`. |

Worked example (`Session/CreateSession`, `0x1400CF0A0`; values are samples):
- Request body: `UserId` = `5f1c…-…` (GUID text), `UserName` = `:player01`, `UserCenter` = 1, `UserStatus` = 0, `Location` = `kr`, `IsAuthenticated` = 1, `Persistent` = 0.
- Checks pass (GUID is not zero, name has ≥ 2 bytes, center ≠ 0). Name becomes `player01`. A session (0xD98 bytes) is made for conn+652, `Location` = `kr`. The gate client is set with authenticated = 1. Persistent expiry = now + 2592000 s (defaults).
- Local log (if `LocalLog/Use` = 1): `SESSION_CREATE,UserId=5f1c…,SessId=<conn+652 as text>,UserName=player01,addr=<conn+24>,status=0`.
- With `UserCenter` = 0: Err 10 (`ErrBadParam`), module 1, line 610, text `SgConn: session create, missing id, center, or name`.

Quirk rows:

| Name | Address | Behaviour |
| --- | --- | --- |
| `quirk_update_timeout_zero` | `0x1400D0470` | `TimeoutMs` = 0 is stored as 0xFFFFFFFF (the largest u32), not as 0. The session then has the longest timeout. |
| `quirk_create_name_colon` | `0x1400CF0A0` | The name check tests the second byte (`UserName[1] != 0`), before one leading `:` is removed. So a 1-character name is rejected, and `":x"` is accepted and becomes `x` ❓ (string form of the field not checked). |
| `quirk_invalid_period_used` | `0x1400D2B20` | `LocalLog/Period` ≥ 4 logs an error but the value is still given to the log open. |

### 3.6 Calls out

| Target | Command | When | Address |
| --- | --- | --- | --- |
| Auth (gate send) | the client login request (copy) | `Auth/*Login*`, `MobileAuth/Login*` | `0x1400CE6A0` |
| Auth | `LoginFinish` | `Session/LoginFinish` | `0x1400CF410` |
| Auth | `VerifySecondaryAuth` | `Session/VerifySecondaryAuth` | `0x1400D0AB0` |
| Auth | inner `ChangeMyUserName`, `AssociateMyExternalAccount`, `ActivateAccount`, `DeactivateAccount`, `RequestToken` | `Session/Validate` | `0x1400D05D0` |
| any (inner target) | inner request | `Session/Forward` | `0x1400CEA40` |
| SessionGateSrv (owner) | `Session/CheckSession`, `Logout`, `Relogin` | the session has a remote owner | `0x1400CEC60`, `0x1400CF6A0`, `0x1400CFAF0` |
| SessionDb | 6 procs | §3.4 | |
| Auth | `LoginInternal` | relogin: persistent session found, secret good | `0x1400DC860` |
| Auth | `LoginCancel` | end session in state 2 (login pending) | `0x1400D6870` |
| Access (gate library send) | AccessGetPermission | login finish, not persistent; or after `CreatePersistentSession` | `0x1400D5E10` |
| Presence (gate library send) | PresNotify | AccessGetPermission reply with `AccessMask` | `0x1400D95D0` |
| SessionGateSrv (new session) | `Session/CreateSession` (direct, or with storage refer-to `AcctDb.GetNameById`) | `Auth/LoginInternal` reply | `0x1400DBB40`, `0x1400CE390` |

### 3.7 State and timers

- Correction (verified): `0x14012B050(obj)` and `0x14012AEA0()` are not a session cache. They hold one global object pointer (`xmmword_1402BC230`): `0x14012B050` releases the old object (its slot 0 with argument 1 = delete) and stores `obj`, flag `DWORD2(xmmword_1402BC220)` = 0; `0x14012AEA0` releases the held object, stores null, flag = 1. Framework helper ❓ (meaning of the flag not read).
- Named timer: `0x14012AC00(name, ms)` looks up `name` in a string hash map (`dword_140255808`) and calls `0x14012A990(entry, ms)`, which calls `StsTimerSet` (`0x140143990`). Both helpers have `shared_with` = 19 (common code, not gate code). SessionGateSrv sets timer `"Session"` with period 24 × `dword_1402AA47C` ms (callers `0x1400D2A40` and the config load `0x1400D2B20`; default 24 × 3600 = 86400 ms if `dword_1402AA47C` = `Session/Timeout` ❓). The OnTimer of the `"Session"` entry is not read ❓.
- Session states (session +148, set by vtable +24): 0 none, 2 login pending, 3 persistent create pending, 8 set after `SessionCreate` reply; the end-session steps per state are in §3.3 (`0x1400D6870`). States 1, 4–7 are not named ❓.
- Session object 0xD98 bytes; fields read through vtable slots: +56 set timeout, +72 is persistent, +80 set persistent, +88 set persistent expiry, +120 persistent start ❓, +144 has external account, +160/+168/+176/+192 external account texts, +280 gate client, +288 UserId, +304 UserName, +312 LoginName, +320 UserCenter, +328 UserStatus, +344 login pending, +416/+432 event, +456 logout, +464 set Location, +480 create reply.
- Timers: `StsTimerSet` has 45 call sites; 7 are below `0x140130000`. Of these, only `0x14012A990` (named timer above) is reached from service code. The others (`0x1400F13A0` 100 ms, `0x1400F1520` remove, `0x1400F1940` 15000 ms, `0x1400F1980`, `0x14012B320`, `0x14012DF20`) are in framework code. The recipe `timers()` fails on this binary (`IndexError` in `recipe.py` line 154: a call with fewer than 3 arguments).

## 4. Framework gaps

- Config section readers not in `00-framework.md`: `CfgSection(cfg, name)` CliGate `0x1400EE900`, `CfgSectionInt(sec, key, 0, default)` `0x1400EEC60`, `CfgSectionValues(cfg, name, …)` `0x1400EE6B0`. The recipe `cfgkeys()` does not find keys read through them.
- Gate txns use slot 4 = OnReply and slot 5 = OnError (§1.3); the framework txn uses slots 3/4.
- `table()` is not usable for gate binaries (§1.1 note).
- SessionGateSrv does not use a command-id switch: it uses a `"protocol/command"` → handler-object map (`0x1400D1280`). The command names come from the per-protocol name functions (`0x1400F2A80` Auth, `0x1400F53E0` MobileAuth, `0x1400FA1A0` Session). The "Service recipe" does not find this map.
- `apply_names` with `framework-names.tsv` gave 0 new names in SessionGateSrv, but `fingerprint()` from `tools/fn_fingerprint.py` matches 86 of the 87 hashes (for example `StsTimerSet` `0x140143990`, `MsgInit` `0x140106C70`, `IniGetInt` `0x1400FEE50`); a direct `idc.set_name` loop works. `DbProcBegin` has no match. The gate-library helpers (`0x14011C700`, `0x14011F700`, `0x14011FB40`) were mapped to CliGate by the `hash` column of `funcs/*.tsv`.

## 5. Differences from existing specs

| Spec (`../11-cligate-protocol.md`) | Binary | Address |
| --- | --- | --- |
| §3: "All forwarded calls use … 25 000 ms" | `WaitForAllowedLogin` (99) uses 600000 ms | `0x1400D27A0` |
| §3 dispatcher list | adds IpAuthz split (AddIp, CheckIp, RequestIpToken, SendIpAuthCode `0x1400D1CE0`), session-path handlers (§2.2 step 7), Auction login check line 902 | `0x1400D4AD0` |
| §12.4 "Presence command 14 … name not decoded" | name is `UserLogout` | map `0x14029E940` |
| — (not in spec) | bus `ApiManage` `Updated`/`Flush`/`FlushDefinitions` reload the gate API rules | `0x1400D6A00` |
| — (not in spec) | `CliGate.ini [CliGateOptions]` keys, `Gate.ini` keys | `0x1400D6030`, §1.5 |
| §1 server type | 1001 | `0x1400D6720` |
