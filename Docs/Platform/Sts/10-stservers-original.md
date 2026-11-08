# 10 — Original StServers (NP/STS platform), full inventory

## Evidence and scope

| Item | Value | Source |
| --- | --- | --- |
| Product | NCsoft NP platform "StServers", Chinese repack labelled NPBNS | `BNSX-files/D/StServers/` (extracted from the BNSX VM disk, no VM) |
| Version | 1.20.3350, binaries dated 2016-12-28 | `Bin/Deploy/Deploy_versions.xml`, file dates |
| Programs | 114 `*.exe`, x86-64 PE, no PDB | `Bin/Deploy/` (identical to `Bin/Run/0001`) |
| Config | 70 files in `Bin/Config/` | `Bin/Config/*.ini` |
| Databases | 37 platform databases + 8 game databases, SQL Server | `BNSX-files/D/DB/*.mdf`, attached to `bns-mssql` |
| SQL export | 5,826 modules (procedures, functions), table columns, indexes, table types. No module is encrypted. | `BNSX-files/sql-dump/<Db>/{modules.sql,columns.txt,indexes.txt,tabletypes.txt}` |

Secrets are not copied into this file. `Db.ini` passwords, the `Route.ini [TLS]` key, the 5th column of `Game.ini [Configurations]` and `Certificate/*` stay in the original files. A port writes the placeholder `keyhere` for each.

❓ IDA licence check failed on 2026-11-08 (`Cannot continue without a valid license`). Wire facts below come from configs, SQL, web pages, strings and RTTI, not from decompiled code.

## Topology

```text
 BnS client ──TCP 6600──► CliGate ─┐
 LobbyGate/GameDaemon ─TCP 6601─► GameGate ─┤        ┌─► AuthSrv ─► DbSrv(AcctDb)
 Web/App clients ─6602─► AppGate ─┤        │   SessionGateSrv
 HTTP API ─6607─► ApiGate ────────┼─► RouteSrv (6605) ─┼─► GameAccountSrv ─► DbSrv(GameDb)
 MwGate, WebGate ─6601──────────────┘        │   SlotSrv, WorldSrv, PresenceSrv, NameSrv …
                                              └─► ~100 other services
 SpawnSrv (6606, Windows service) starts and watches every *Srv.exe
```

| Port | Service | Notes | Source |
| --- | --- | --- | --- |
| 6600 | CliGate | Game client gate; system `bnsclient` | `StServer.ini [Systems.Test]` |
| 6601 | GameGate, MwGate, WebGate | Game server gate; system `bnsserver` | same |
| 6602 | AppGate | Web/app API gate (`Web.ini [URIs.AppGate]`) | same |
| 6605 | RouteSrv and every internal service, DbSrv instances 1–33 | Internal bus | same |
| 6606 | SpawnSrv | Process supervisor | same |
| 6607 | ApiGate | REST contexts `/np/v1/*`, `/v1.0/*` (`ApiGate.ini`) | same |
| 6608 | SvcDispatcher instance 2 (Report) | | same |

Start order (`Spawn.ini [Run]`): `RouteSrv = Serial,1,0,30` first, then `Default = Parallel,2,0,30,60,90` for the rest.

## Shared framework (in every program)

Each program is about 2.2–3.1 MB. 160 RTTI classes appear in 100 or more programs. Service code adds 3–265 own classes. A port therefore needs one shared crate and thin service crates.

| Framework part | RTTI / source file | Function |
| --- | --- | --- |
| Socket layer | `ArenaSocket.cpp`, `CArenaSocket`, `CArenaSocketTls`, `CArenaListener` | TCP and TLS sockets |
| STS message | `InetMsg.cpp`, `TXBuilder` (RapidXml), `XParser.cpp`, `StreamJson.cpp` | `POST /<Proto>/<Cmd> STS/1.0` messages, XML/JSON bodies (see 01) |
| Route bus | `StsRoute.cpp`, `Srv2Route.cpp`, `CStsRouteConn`, `CStsRouteLogConn` | Service ↔ RouteSrv |
| Grid | `StsGrid.cpp`, `CStsGridConn`, `StsGridObject` | Keyed shared state (grid_key in `.aver`) |
| Direct conn | `StsDirectConn.cpp`, `CStsDirectConnNotify` | Point-to-point service links |
| Gate | `StsGate.cpp`, `CStsGateSocket`, `CGateTxnFactory<…>` | Connect/Disconnect/GetRights/IpCheck/Inquiry/Report transactions |
| Crypto | `CptRc4.cpp`, `CptAes.cpp`, `CptRsa.cpp`, `SocketCrypt.cpp`, `CCryptFilter`, OpenSSL (`libeay32`) | Client socket encryption |
| Admin HTTP | `StsHttpConn.cpp`, `StsHttpCmd.cpp`, `CAbout*`, `Bin/Web/Sts/*.xsl` | Per-service status pages, command test pages |
| Supervisor link | `CSpawnConn` | Link to SpawnSrv |
| Config | `Config.cpp`, `CConfigServers`, `CConfigWhitelist`, `CChangeMonitor` | INI loading with live reload |
| Timers, logs | `StsTimer.cpp`, `Log.cpp`, `ErrLog.cpp` | |

CliGate source files: `CliConn.cpp`, `CliGateConfig.cpp`, `CliMain.cpp`, `Geolocation.cpp`, `NetAccess.cpp`, `NetAdjust.cpp`, `Tls.cpp`. Source: strings in `CliGate.exe`.

## Gate command whitelists

`Gate.ini` lists which commands each gate forwards. Flags: `FLAG_NONE`, `RANDOM_KEY`, `ALLOW_SELF` and others. The 3rd column is the transaction cost.

| Gate section | Commands |
| --- | --- |
| `[Commands.CliGate]` | 144 |
| `[Commands.GameGate]` | 74 |
| `[Commands.SessionGateSrv]` | 309 |
| `[Commands.MwGate]` | 185 |
| `[Commands.AppGate]` | 734 |

CliGate Auth commands: CreateAccount, Forward, GetMyUserInfo, GetUserInfo, GetUsersInfo, KeyData, ListNetAddressRoles, ListUserRoles, LoginFinish, LoginStart, LoginTokenStart, RequestGameToken, RequestToken, TokenKeyData. Other CliGate commands used by BnS: GameAccount/ListMyAccounts, Slot/ListSlots, Slot/SetClientData, World/ListWorlds, Goods, VirtualCurrency, Presence, Friend.

GameGate (daemon side): Auth/RequestToken, Auth/LoginToken, GameAccount/ListAccounts, Slot/CreateSlots|DeleteSlot|ListSlots|SetData, World/AddWorld|ListWorlds|RemoveWorld|UpdateWorldProfile|UpdateWorldProperty, Presence/Login|UserLogout|SetWatch|UserInfo, Goods/*, VirtualCurrency/Withdraw|Deposit|GetBalance, Charge/Start|Stop, Gate/BulkFind|Find.

`Bin/Products/ApiUpdater/test/fixture/0000.aver` (JSON) holds 154 providers, 1754 commands, 63 services and 1801 URIs with gate exposure, phase, transcost and grid_key. It has no field schemas.

## Applications (`Game.ini`)

| System | GameCode | Id | Program | AppId | Secret |
| --- | --- | --- | --- | --- | --- |
| `bnsserver` | bns | 4000 | 302 | `D5099619-B2F2-26EB-8DDA-DD084E83DB1B` | yes (not copied) |
| `bnsclient` | bns | 1001 | 302 | `B0D42105-0CB6-BC9F-3CB2-BE28A0662340` | empty |

The daemon Connect writes build 4000 (see 01). This matches the `bnsserver` row. ❓ The client AppId value on the wire is not checked against `B0D42105-…`.

## Key service settings

| File | Setting | Value |
| --- | --- | --- |
| `CliGate.ini` | RootKeyBits / ServerKeyBits | 2048 / 1024 |
| `CliGate.ini` | Root / server certificate | `root.ca` / `garena1024.ca` + `garena1024.pr` |
| `Auth.ini` | TokenTimeout App / Game | 1800 s / 300 s |
| `Auth.ini` | NameSession Use | 1 |
| `Session.ini` | Timeout / PersistentSessionTimeout | 3600 s / 2592000 s |
| `Slot.ini` | MaxCreateSlots | 15 |
| `Game.ini` | MaxRequestsToGame | 100 |
| `GameAccount.ini` | `[AccountLocation] bns` | `1, 1, 0` |
| `Name.ini` | Name types | UserName 1, LoginName 2, LoginNameNorm 3 … Garena 20 |
| `Network.ini` | Datacenter | `Test`, id 1, 10.10.66.88 |

Certificates: none of `Certificate/*` contains the client StsMsgCli64 RSA-2048 modulus in plain byte order (checked both orders, three offsets). ❓ The files are not plain DER/PEM; their format needs CliGate code.

## Databases

| Database | Tables | Procs | Schema used | Program |
| --- | --- | --- | --- | --- |
| PlatformAcctDb | 14 | 1429 (≈18 versions) | `v2016102001` | AuthSrv |
| PlatformGameDb | 7 | 120 | `v2016120901` | GameAccountSrv, WorldSrv |
| PlatformSessionDb | 1 | 33 | `v2015102301` | SessionGateSrv |
| PlatformSlotDb | 3 | 20 | `v2016041101` | SlotSrv |
| PlatformMasterDb | 2 | 32 | `v2015102301` | IpCheckSrv |
| GlobalNameDb | 2 | 53 | `v2015062301` | NameSrv |
| AppDb / ApiDb | 19 / 11 | 234 / 145 | `v2016112501` / `v2016122201` | AppAuthSrv / ApiManageSrv |
| Others (Goods, Coupon, Friend, Limit …) | | | `dbo` or per-exe pin | |

Each program pins one schema version string. Sources: strings `v20xxxxxxxx` in each exe (34 programs), schema names in `sql-dump`. Error returns use `RETURN (3000 + n - 100000)`: service base + code − 100000. Source: `v2016102001.p_AcctLoginInfoById`.

Login-path procedures in `v2016102001`: `p_AcctGatewayLogin(@loginName, @netAddress)` → UserId, UserCenter, UserName. `p_AcctLoginInfoById(@userId)` → UserId, UserCenter, UserName, UserStatus, LoginName, Salt, Verifier, PasswordUntil, PasswordUntilType, ValidateType, Validated. `p_AcctCreateSession`, `p_AcctValidateSession`.

Main tables:

| Table | Columns (short) |
| --- | --- |
| `Users` | UserId GUID, UserName, UserStatus, UserDatacenter, LoginName, LoginNameNorm, Salt varchar(1368), Verifier varchar(1368), Created, PasswordUntil, PasswordUntilType, IsLoginNameValidated. Repack adds IpAddress, MachineId, realPassword, safePassword, dynPassword, allInpour, banState, adminLvl, InviteId. |
| `GameAccounts` | GameAccountId varchar(128), AppGroupCode, UserId, Created, IsUserChangeable |
| `Worlds` | WorldId, GameCode, WorldCode, Datacenter, PublicNetAddress, WorldName, IsCharacterCreatable, WorldScore |
| `Slots` | SlotId, UserId, AppGroupId, SlotType, TransactionKey, IsSystemSlot, SlotData, Registered, Changed |
| `PersistentSessions` | SessionId, SessionSecret, UserId, AppId, Registered, EffectiveUntil, AuthProviderCode |

Data present: 4 users, 4 GameAccounts (`bns`), 40 slots, 0 worlds (daemons add worlds at run time with World/AddWorld).

### SRP verifier — checked against original data

The original server stores `Salt` and `Verifier` as Base64. Salt = 8 bytes. Verifier = 128 bytes (1024-bit group, little-endian 32-bit words). For both accounts with a stored test password, `v = 2^x mod N` matched with:

`x = words_reversed(SHA256(salt ‖ SHA256(UTF-8(LoginName ":" password))))`, N = RFC 5054 1024-bit group.

This is the same formula as `Program/Platform/Sts/src/srp6.rs` (`password_x`). Checked 2026-11-08; only match/no-match was printed.

## Admin and test pages

`Bin/Web/<Service>/` holds HTML/XSL admin pages. `Bin/Web/AuthSrv/test.html` lists AuthSrv commands with fields:

| Form action | Fields |
| --- | --- |
| `authenticate_srpstart` | loginName |
| `authenticate_srpfinish` | loginName, srpSession, srpKeyData |
| `authenticate_idpwd` | loginName, password or passwordHash |
| `request_token` | userId, appId, issuerAppId?, issuerIp? |
| `login_token` | appId, authProviderCode, authnToken, tokenInfo |
| `create_account` | loginName, password, userName, userCenter, effectiveUntil, loginNameValidated |
| `get_user_info` | userId or loginName |

## Gap list for the Rust STS (`Program/Platform/Sts`)

Port progress for every original service is in `../StServers/README.md`.

| # | Item | Original evidence | Action | State (8 Nov 2026) |
| --- | --- | --- | --- | --- |
| 1 | `enroll` makes a 32-byte salt | Original salt = 8 bytes | Use 8 bytes; store Base64 like `Users.Salt` | open |
| 2 | Verifier store is a local file | `PlatformAcctDb.dbo.Users` | Port `p_AcctLoginInfoById` semantics through DbSrv | DbSrv ported; AuthSrv not ported yet |
| 3 | No GameGate | 74 daemon commands | Port GameGate for LobbyGate/GameDaemon | not ported yet |
| 4 | No RouteSrv / service split | 49 started services over RouteSrv | Port RouteSrv, then each service | RouteSrv in progress |
| 5 | World list empty | `Worlds` table + World/AddWorld | Port WorldSrv + GameGate World/* | ❓ WorldSrv has no `StServer.ini` line |
| 6 | Slot, GameAccount partial | SlotSrv, GameAccountSrv procs | Port with `v2016041101` / `v2016120901` semantics | not ported yet |
| 7 | AppId values | `Game.ini` AppIds | Use `B0D42105-…` for client, `D5099619-…` for server | open |
| 8 | Error codes | `RETURN (base + n - 100000)` | Map the same numbers | `sts-core` error table + `errcodes.tsv` |
| 9 | Certificates / CliGate TLS | `CliGate.ini`, `Certificate/*` | Key fields → `keyhere` | analysed (`11-cligate-protocol.md`) |
| 10 | Admin web pages | `Bin/Web/*` | Optional; low priority | not ported yet |

## Next steps

1. Done: IDA works; CliGate, AuthSrv, RouteSrv, DbSrv, GameGate, SessionGateSrv, GameAccountSrv, SlotSrv, SpawnSrv and WorldSrv are decompiled.
2. Port the framework crate, then CliGate, GameGate, AuthSrv, SessionGateSrv, GameAccountSrv, SlotSrv, WorldSrv, PresenceSrv, NameSrv.
3. Port the remaining services by the order of the BnS login and in-game flow.
