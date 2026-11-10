# 02 — AuthSrv, AppAuthSrv (RE reference)

Summary:
1. AuthSrv (`AuthSrv.exe.i64`, lane `st`) serves protocol `Auth` (110 command ids, map `0x140294C30`). Server type 1012 (`StsSetServerType(1012, 0)` in init `0x1400D6C50`).
2. Every `Auth` id has a known path: main switch `0x1400F3770`, three pre-check switches (`0x1400C94A0`, `0x1400C9180`, `0x1400DF820`) or a fixed error. See `02-commands.tsv`.
3. DB: logical DB `AcctDb`, schema `v2016102001`. All 68 procs of `procs.tsv` are found (77 call sites plus 4 test names). See `02-procs.tsv`.
4. Config file `Auth.ini` (`StsConfigMonitorAdd` in `0x1400D6C50`, notify slot `0x1400D6CF0`). Two service timers: name session cleanup and tentative cleanup.
5. AppAuthSrv (`AppAuthSrv.exe.i64`, server type 1039) serves protocol `AppAuth` (49 ids, map `0x14023D1A0`), DB `AppDb` / `v2016112501` (50 procs), an app cache, an OAuth-type token flow with the Token service, and one timer (expired token delete). See §2.

All addresses are AuthSrv unless the text gives another binary. ❓ = not verified in IDA.

## Quick find

| Question | Function | Binary and address | Section |
| --- | --- | --- | --- |
| Service init | `AuInit` | AuthSrv `0x1400D6C50` | 1.1 |
| Config keys | `AuReadCfg` | AuthSrv `0x1400D6CF0` | 1.2 |
| Message entry | `AuOnMessage` | AuthSrv `0x1400D7040` | 1.3 |
| `t` header / UserId parse | `AuParseTo` | AuthSrv `0x1400D7710` | 1.3 |
| Pre-check switches | — | AuthSrv `0x1400C94A0`, `0x1400C9180`, `0x1400DF820` | 1.4 |
| Local or remote user | `AuDispatchGrid` | AuthSrv `0x1400D68F0` | 1.3 |
| Main command switch | `AuDispatch` | AuthSrv `0x1400F3770` | 1.4 |
| ConsumeGameToken | handler | AuthSrv `0x1400E3D70` | 1.6 |
| `CAuTxn` OK reply switch | slot +32 | AuthSrv `0x1400F22C0` | 1.7 |
| Provider lost counters | `AuOnProviderLost` | AuthSrv `0x1400D7320` | 1.8 |
| Outgoing message builders | `MsgSetCmd` callers | AuthSrv `0x1401214E0` | 1.9 |
| Test proc switch | `AuSetDbProcs` | AuthSrv `0x1400D10E0` | 1.5 |
| Timers | `AuTimersStart` / `AuCfgTimers` | AuthSrv `0x1400D89D0` / `0x1400D94B0` | 1.10 |
| AppAuth init | `AaInit` | AppAuthSrv `0x1400D4DA0` | 2.1 |
| AppAuth config keys | `AaReadCfg` | AppAuthSrv `0x1400D4E40` | 2.2 |
| AppAuth entry / parse | `AaOnMessage` / `AaParse` | AppAuthSrv `0x1400C8290` / `0x1400F3150` | 2.3 |
| Local / app command switch | `AaLocalDispatch` / `AaAppSwitch` | AppAuthSrv `0x1400C5440` / `0x1400CEA30` | 2.3 |
| App cache (find, load, touch) | `AppCacheFind` / `AppCacheLoad` / `AppTouch` | AppAuthSrv `0x1400CD630` / `0x1400CD010` / `0x1400D4210` | 2.4 |
| App txn OK / error switch | slot +24 / +32 | AppAuthSrv `0x1400D6F70` / `0x1400D7930` | 2.6 |
| Expired token timer | `AaDelExpTokensTimer` | AppAuthSrv `0x1400D7A70` | 2.7 |
| Authenticate / Authorize / Token | handlers | AppAuthSrv `0x1400CF340` / `0x1400D02B0` / `0x1400D2280` | 2.8 |
| Make access token | `AddAccessToken` | AppAuthSrv `0x1400D7280` | 2.8 |

## 1. AuthSrv

### 1.1 Role and init (`0x1400D6C50`)

AuthSrv owns the account login (SRP), app and game tokens, name changes, password changes and the account life cycle. Callers: CliGate and SessionGateSrv (see `01-gate-cligate-sessiongate.md`), other services through the route bus, and other AuthSrv instances (grid).

Init order (`0x1400D6C50`):
1. `StsSetServerType(1012, 0)`.
2. `unknown_libname_6_0("Auth", 1, &3)` ❓ (HelloSrv has the same call with other values; not decoded).
3. `0x1400D7930(app)` ❓, `0x1400D7840()` (resolver table, `15-authsrv.md` §3.3; it also writes `dword_140283880`).
4. `0x140134F70(app+24, 0)` (grid/route start ❓; it adds a config monitor).
5. `StsConfigMonitorAdd(L"Auth.ini", app+16)` → notify slot 0 = `0x1400D6CF0` (vtable `0x1401A7B58` slot 6).
6. `0x1400DEFF0`, `0x1400DB200`, `0x1400D9D70`, `0x1400D13D0` ❓ (not decoded).
7. `StsAppCommonStart(0)`, then `0x1400D89D0` (timers, 1.10).

Service vtable `0x1401A7B58` slots (part): +0x18 `0x1400D7290` ❓ (provider found ❓), +0x20 `0x1400D7320` (provider lost, 1.8), +0x30 `0x1400D6CF0` (config), +0x58 `0x1400D7130` (parse error text, `15-authsrv.md` §1), +0x68 `0x1400D7040` (OnMessage).

### 1.2 Config keys (`Auth.ini`, `0x1400D6CF0`)

`IniGetInt(ini, section, key, index, default)` (`00-framework.md` §1.4). Getter functions were misnamed `GeoIP_num_countries_N` by FLIRT; they only return the global.

| Section | Key | Default | Global (getter) | Use |
| --- | --- | --- | --- | --- |
| `Common` | `GridObjectTimeoutMs` | see quirk | `dword_1402831AC` | `0x140135160("Auth", value)` grid object timeout ❓ |
| `TokenTimeout` | `App` | 1800 s | `dword_140283870` = ×1000 (`0x1400D68D0`) | app token life (`15-authsrv.md` §7) |
| `TokenTimeout` | `Game` | 300 s | `dword_140283874` = ×1000 (`0x1400D68E0`) | game token life |
| `UserName` | `NotAvaliableSecs` | 7776000 | `dword_140283878` | no reader (1 xref, the write) |
| `UserName` | `NotChangeableSecs` | 7776000 | `dword_14028387C` (`0x1400D6AA0`) | user name change rule ❓ |
| `LoginName` | `NotChangeableSecs` | 7776000 | `dword_140283888` (`0x1400D6A10`) | login name change rule ❓ |
| `Password` | `ExpirationSecs` | 0 | `dword_14028388C` (`0x1400D6A60`) | ❓ |
| `TokenType` | `App` | 0 | `dword_140283890` (`0x1400D6AC0`) | 1 = app token must be used from the issue address (1.6 step 5) |
| `TokenType` | `Game` | 0 | `dword_140283894` (`0x1400D6AD0`) | same for game tokens |
| `PasswordBlacklist` | `StorePasswordBlacklist` | 0 | `dword_140283898` (`0x1400D6B80`) | ❓ |
| `PasswordBlacklist` | `CheckPasswordBlacklistLevel` | 0 | `dword_14028389C` (`0x1400D69F0`) | ❓ |
| `NameSession` | `Use` | 1 | `dword_1402838A8` (`0x1400D6B40`) | name session timer on/off; test procs (1.5) |
| `NameSession` | `MaxTryCount` | 5 | `dword_1402838AC` (`0x1400D6A30`) | param 1 of `GetNameSessionInfoByTerm` |
| `NameSession` | `SessionFromAfterSecs` | 300 | `dword_1402838B0` (`0x1400D6A20`) | param 2 of `GetNameSessionInfoByTerm` |
| `NameSession` | `TimerSleepSecs` | 900 | `dword_1402838B4` (`0x1400D6A50`) | timer period (1.10) |
| `NameSession` | `SkipSyncByIdForTest` | 0 | `dword_1402838B8` (`0x1400D6A40`) | ❓ |
| `EventLog` | `UseTimeout` | 0 | `dword_140283884` (= value ≠ 0) | ❓ |
| `SecondaryAuthSkipRule` | `RoleId` | 0 | `dword_1402838A0` (`0x1400D6A70`) | ❓ |
| `TentativeCleanupTimer` | `Interval` | 0 | `dword_1402831A4` (`0x1400D6A90`) | timer period in s (1.10) |
| `TentativeCleanupTimer` | `DeleteCount` | 20 | `dword_1402831A8` (`0x1400D6A80`) | ❓ (rows per run) |
| `UserName` | `UniqueProperty` | 1 | read at `0x140111E81` | selects the `…NonUniqueUserName…` proc variants ❓ (1.5) |

The function ends with a call to `0x1400D94B0` (timers re-armed on every config load).

- `quirk_grid_timeout_index`: `0x1400D6CF0` reads `GridObjectTimeoutMs` with index = 3600000 and default = 0, so the key value is not used (index out of range ❓ → 0). It calls `0x140135160("Auth", that value)`, and later in the same function `0x140135160("Auth", 3600000)` unconditionally. The effective value is always 3600000 ms.
- `quirk_username_notavailable_unused`: `UserName/NotAvaliableSecs` (spelling as in the binary) is stored but never read.

### 1.3 Message entry (`0x1400D7040`)

1. `0x1400D7710` parses the `t` header into a local record (16-byte UserId, +16 int center/grid key ❓, +24 text). Errors: `{10, module 5, line 166} "AuMain: invalid to header"`, `{10, 5, 170} "AuMain: missing userId"` (as `15-authsrv.md` §3.3). The request body was parsed before into the global request record `dword_1402831B0` (`0x140110FC0` ❓ caller not traced).
2. If record +16 = 0 and the grid object exists: +16 = `OwnDatacenter()` (`0x140157AF0`, `dword_1402965F8`).
3. Pre-check 1 `0x1400C94A0`, then pre-check 2 `0x1400C9180`, then pre-check 3 `0x1400DF820`. A pre-check that returns not 0 handled the message.
4. Else `0x1400D68F0(app, to, req, msg, grid)`:
   - grid object null → reply `{4, module 5, line 545} "no grid data"`.
   - `req+8 & 0x1F` = 0, or `0x1400ECC30(grid, key)` true (user is local) → main switch `0x1400F3770`.
   - else `0x1400ECF00`: build `Auth/LoadUserInfo` (`0x14010F930`), `GridTxnSend(msg, txn(0x20), req+8 & 0x1F, 0, 25000, 0)`, then `0x1401352A0(msg)` (keep the message until the user object is loaded ❓).

### 1.4 Command dispatch, all ids

- Main switch `0x1400F3770` (jump table at `0x1400F37AD`, ids 2..98). Default and ids 1, 99..110 that reach it: `ReplyError({52, module 6, line 6158}, "unknown auth command %u")`.
- Pre-check 1 `0x1400C94A0`: ids 1 `AccessTest`, 96 `LoginTokenStart`, 97 `LogoutMyClient`, 99 `WaitForAllowedLogin` → `StsRouteReply({51, module 1, line 2669}, no body)`. It also handles 42, 43–90 (part) and 94, 95 (table below).
- Pre-check 2 `0x1400C9180`: ids 16, 46, 48, 50, 53, 54, 58, 64, 76–80, 82, 85, 91–93, 100–104. Id 100 `SetDbProcForTest`: `0x1400D10E0(req[310] = req+1240)`, then reply `0x1400D0E10`.
- Pre-check 3 `0x1400DF820`: ids 105–110 (`Notify*`, `Flush`).
- So no `Auth` id falls to the 6158 error in normal flow, except ids that a pre-check does not take and the main switch does not have. Check: every id 1..110 has a path (list in `02-commands.tsv`, column `path`).

Handler addresses by id (H = handler; same handler = shared code):

| Ids | Handler | Path |
| --- | --- | --- |
| 2 ActivateAccount | `0x1400E0F20` | main |
| 3 AuthenticateToken | `0x1400E1150` | main |
| 4 ChangeExpiredPassword, 7 ChangeMyPassword | `0x1400E14A0` (needs current password, `"missing current password"`, then `0x1400E32B0`) | main |
| 5 / 6 ChangeMyUserName(Incomplete) | `0x1400E2680` / `0x1400E28E0` | main |
| 8 ChangeLoginName, 9 …Start, 87 …Finish, 88 …Cancel | `0x1400E1510`, `0x1400E1C10`, `0x1400E1A50`, `0x1400E1960` | main |
| 10 ChangePassword, 11 ChangeUserName | `0x1400E32B0`, `0x1400E3AF0` | main |
| 12 CreateLoginSession, 13 DeactivateAccount | `0x1400E41E0`, `0x1400E43E0` | main |
| 14 DeleteUser, 33 RemoveAccount, 34 RemoveMyAccount | `0x1400E7AD0` | main |
| 15 DeleteLocalUser | `0x1400E4600` | main |
| 17 ExpirePassword | `0x1400E4730` | main |
| 18 GetMyUserInfo, 20 GetUserInfo | `0x1400E5090` | main |
| 19 GetUserEmail | `0x1400E4F50` | main |
| 21 KeyData, 22 LoginStart, 23/27 LoginFinish/LoginWebFinish | `0x1400E5B30`, `0x1400E6910`, `0x1400E6380` | main |
| 24 LoginCancel, 25/26 LoginTrusted/LoginWebStart, 28 LoginInternal | `0x1400E6010`, `0x1400E6B50`, `0x1400E67E0` | main |
| 29 RevokeAuthnToken, 30 RequestGameToken, 31 ConsumeGameToken, 32 RequestToken | `0x1400E8460`, `0x1400E7F50`, `0x1400E3D70`, `0x1400E8190` | main |
| 35 VerifyMobiOtp, 36 VerifySecondPwd | `0x1400E8A90`, `0x1400E8AD0` (both only call `0x1400E8B10`) | main |
| 37 GetSecurityCardCodeIndex, 38 VerifySecondaryAuth | `0x1400E4C50`, `0x1400E8B10` | main |
| 39 Forward, 40 MergeInternal, 41 Authenticate | `0x1400E49B0`, `0x1400E6EC0`, `0x1400E1010` | main |
| 65 DeactivateUser, 81 ReactivateUser, 83 ReissueEmailToken, 86 ValidateEmailToken, 98 RequestCryptKey | `0x1400E44D0`, `0x1400E7880`, `0x1400E79B0`, `0x1400E8880`, `0x1400E7D10` | main |
| 69, 70, 71 Is…Changeable | `0x1400E5860`, `0x1400E58E0`, `0x1400E5A10` | main |
| 72/73, 74, 75 Get…Changeable | `0x1400E4BB0`, `0x1400E5610`, `0x1400E5740` | main |
| 42 AddBanWord, 55 RemoveBanWord | `0x1400CAC90`, `0x1400CDF60` (→ `Name/AddBanWord`, `Name/RemoveBanWord`) | pre1 |
| 43, 47, 49, 51, 52, 56, 61, 68, 84 (role commands) | `0x1400CEED0`: needs `Conn1121() ≠ 0` (server type 1121 connected), then forward to protocol `Role` (`0x1400F9590`, command from `0x140117A40`) | pre1 |
| 44 CheckLoginName, 45 CheckUserName | `0x1400CB490`, `0x1400CB930` (→ `Name/CheckNames`) | pre1 |
| 57 RequestGuestToken | `0x1400CE500`: needs ExAuthSrv connected, forward to `ExAuth`; else error text `exauthsrv…` | pre1 |
| 59, 60, 62, 63, 66, 67, 89, 90 (external accounts) | `0x1400CBE20`, `0x1400CD800`, `0x1400CB090`, `0x1400CB290`, `0x1400CBD00`, `0x1400CBB60`, `0x1400CE860`, `0x1400CE360` (→ `ExAuth/…`; 62/63 text `exauthsrv not found`) | pre1 |
| 94 PageVerifiedIps, 95 UnregisterVerifiedIp | `0x1400CDD00`, `0x1400CE620` (→ `IpVerify/PageIps`, `IpVerify/UnregisterIp`) | pre1 |
| 16, 46, 48, 82, 85, 104 | `0x1400C8590`, `0x1400C8490`, `0x1400C8510`, `0x1400C8690`, `0x1400C8790`, `0x1400C8610`: each calls a check (`0x1400C5800`, `0x1400C54B0`, `0x1400C5610`, `0x1400C5900`, `0x1400C59D0`, `0x1400C58A0`); check error → reply it; else the work function (`0x1400C8140`, `0x1400C7D20`, `0x1400C7F10`, `0x1400C8260`, `0x1400C8330`, `0x1400C81D0`) | pre2 |
| 50 GetUsersInfo | `0x1400CC000` (→ `Auth/GetUserInfo` per user; `"AuConn: has no body."`) | pre2 |
| 53, 54, 58 LoginToken, LoginWebTokenStart, TokenKeyData | `0x1400CCBB0` (`"AuConn: missing parameter"`; → `Auth/Authenticate`, `Auth/AuthenticateToken`, `ExAuth/AuthenticateToken`) | pre2 |
| 64 CheckPassword | `0x1400CB6C0` (`"AuConn: missing login name or password"`) | pre2 |
| 76–79 Page…Changes/Owners | `0x1400CD260` (`"AuConn: no pageIndex"`, `"AuConn: unsupported command"`; proc by id `0x1400CEA90`: 76 `ListLoginNameChanges`, 77 `ListLoginNameChangesByLoginName`, 78 `ListPasswordChanges`, 79 `ListUserNameChanges`) | pre2 |
| 80 PageMergedUsers | `0x1400CD950` (`AcctListMergedUsers`) | pre2 |
| 91, 92, 93 blocked password | `0x1400CC590`, `0x1400CAE80`, `0x1400CE150` | pre2 |
| 101, 102, 103 Load… | `0x1400CC8E0`, `0x1400CCA50` (`AcctInfo`), `0x1400CC780` (`AcctLoginInfoById`) | pre2 |
| 105–109 Notify…, 110 Flush | `0x1400E7330`, `0x1400E7580`, `0x1400E74D0`, `0x1400E7250`, `0x1400E7700`, `0x1400E4960` | pre3 |

Note: the main switch also has cases for ids that pre-checks take first (for example 65, 69–75, 81, 83, 86–88 are in the main switch only; 43 etc. only in pre1). Order is pre1 → pre2 → pre3 → main, so the pre-check wins ❓ (no overlap was found).

Request and reply fields per command: not decoded in this stage, except the ids in `15-authsrv.md` and 1.6. `02-commands.tsv` has the error tuples and texts that a regex found in each handler (not complete) and the outgoing commands and procs reachable within 3 calls.

### 1.5 DB procs

- All calls: `DbProcBegin(writer, msg, "AcctDb", "v2016102001", proc, flag)` (`00-framework.md` §4). 77 call sites; list in `02-procs.tsv` (function and handler ids that reach it within 3 calls).
- Computed names: one call site selects one of two names.
  - `0x1400DF700`: `AcctChange`, or `AcctChangeExpired` when arg 2 = 4 (ChangeExpiredPassword).
  - `0x1400FD700`: `CreateNameSessionForANCA` / `CreateNameSessionNonUniqueUserNameForANCA`, and `…ForCSVE` / `…NonUniqueUserNameForCSVE`.
  - `0x1400FD250`, `0x1400FD8B0`: `AcctCreate(Tentative)ForNameSrv` / `…NonUniqueUserNameForNameSrv`.
  - `0x1400E7AD0`: `CreateNameSessionForRNRA` / `…NonUniqueUserNameForRNRA`. `0x1400FD510`: `CreateNameSessionForCNLN` / `…ForRCLN`.
  - `0x1400E5610`, `0x1400FDE10`, `0x1400E2680`, `0x1400E58E0`: `Get/Is…Changeable` / `…Incomplete`.
  - The NonUnique selector is `UserName/UniqueProperty` ❓ (not traced to each site).
- `AuSetDbProcs` `0x1400D10E0(flag)` (id 100 `SetDbProcForTest`): when flag ≠ 0 and `NameSession/Use` ≠ 0, the globals `off_1402322B0`..`off_1402322C8` get `FailAcctCreateForNameSrv`, `FailAcctValidateTentative`, `FailAcctChangeLoginNameById`, `FailAcctChangeUserNameById`; else the normal names `AcctCreateForNameSrv`, `AcctValidateTentative`, `AcctChangeLoginNameById`, `AcctChangeUserNameById`. It always sets `off_1402322D0` = `AcctDeleteUser` (used by `0x1400DE690`).
- Count: 68 of 68 expected names of `procs.tsv` found.

### 1.6 ConsumeGameToken (id 31, `0x1400E3D70`)

Args: user grid object (a1), UserId (a2), request record (a3), message (a4).

| Step | Fact |
| --- | --- |
| 1 | No body (msg vtable +96 = 0) → `ReplyError({10, 6, 7933}, "missing body")`. |
| 2 | Token (request +1248, 16 bytes) all zero → `{10, 6, 7940} "missing token"`. |
| 3 | `GameCode` (request +984, pointer) null or empty → `{10, 6, 7947} "missing game code"`. |
| 4 | Find: `0x1400E9000(user, token, gameCode)` walks the game token list of the user (first element pointer at user +272, link offset at user +256, bit 0 = end marker). Match = token GUID (record +32) equal and GameCode equal to record +48 (`0x1401014B0`, byte compare). Not found → code 3010, sub 1. |
| 5 | Location check `0x1400F63B0(rec+104 issue address, req+120 caller address, UserId, "game")`: equal → OK. Else log `consume %s token from wrong location[u:%s,rip:%s,cip:%s]`, and fail only when `TokenType/Game` ≠ 0 (for kind `"app"`: `TokenType/App`). Fail → code 3303, sub 3. |
| 6 | Fail (step 4 or 5): log level 2 `failed to authenticate game token - user:%s, from:%s, error:%u(%u), app:%s, game:%s, token:%s`; `StsRouteReply({code, 6, 8038}, no body)`. |
| 7 | OK: log level 0 `game token is authenticated - user:%s, from:%s, app:%s, game:%s, token:%s`. |
| 8 | If request +304 (16 bytes, AppId ❓) ≠ 0: build `Role/Inquiry` (`0x140117980`: header `t` = UserId, `f` = request +96, body `<Request><UserId>`), `CAuTxn` (`0x1400DD6A0`, 0x160 bytes, state 20, txn +312 = record +168 text), `0x1400F5560(txn, req.id, rec+64 GameAccountId, req+304, req+120)`, `GridTxnSend(msg, txn, 3103, 0, 25000, 0)`. |
| 9 | Else reply now: `0x1400F3FB0(replyId, rec+64, user, rec+168, null)`. |
| 10 | Always when the record was found (also after step 5 fail): delete it, `0x1400F3EE0(user, rec)`. A game token is single use. |

State 20 reply (`CAuTxn` slot +32 `0x1400F22C0`): no body → code 4 `consume game token reply, missing data`; else `0x1400F3FB0(txn+32, txn+64, user, txn+312, roleReply)`.

Reply `0x1400F3FB0` (code 0, module 6, line 444): `<Reply>[<GameAccountId>]<LoginName>(user+32)<UserId>(user+12)<UserName>(user+97)<UserStatus>(user+164 if user+8 bit 3, else 0)<UserCenter>(user+28)<Roles type="array"><RoleId>…</Roles>[<TokenInfo><AuthProviderCode>text</AuthProviderCode></TokenInfo>]</Reply>`. RoleIds come from the `Roles/RoleId` children of the Role reply; none when no Role call.

Worked example (TokenType/Game = 0): request `t` = UserId `03020100-0504-0706-0809-0A0B0C0D0E0F`, body `<Request><Token>T</Token><GameCode>bns</GameCode></Request>` from address A2; the token was made for A1 with GameAccountId `G1`, request AppId zero → log wrong location, reply code 0 with `GameAccountId G1`, `Roles` empty; token deleted. A second call with T → 3010.

No expiry check here: an expired token is removed by its timer (`15-authsrv.md` §7.5) ❓.

- `quirk_token_wrong_location_allowed`: with default `TokenType` = 0 a token from another address is accepted (only a log line). Abuse note `st-02`.
- `quirk_consume_deletes_on_location_fail`: with `TokenType/Game` = 1 a wrong-location try deletes the token too.

### 1.7 `CAuTxn` (vtable `0x1401AA238`)

Slots: `0x1400DE100`, `0x1400F5CC0`, `0x1401804E0`, `0x1400F6DD0`, +32 OK `0x1400F22C0`, +40 error `0x1400F0DC0`, `0x1400F0800`.
OK switch on state (0..37; > 37 = `abort`): 0 → `0x1400E8F10`; 1–12, 15–17, 21–29, 33, 34 → `0x1400E9830`; 13 → `0x1400E9190`; 14 → `0x1400E9B90`; 18 → `0x1400F3D30`; 19 → `0x1400F4A40` (RequestGameToken); 20 → ConsumeGameToken (1.6); 30 → `0x1400DE690` (AcctDeleteUser); 31 → `0x1400DEA70`; 32 → `0x1400DE8D0`; 35 → `0x1400E0AF0`; 36 → `0x1400E0330`; 37 → `0x1400E0D80`. The meaning of each state is not decoded ❓.

### 1.8 Provider counters (`0x1400D7320`, provider lost)

On a lost provider of server type T, decrement the counter; if it is already 0, log level 2 `AuthSrv%d: (%s/%u), <name>'s connection counter reached 0 already` (file `..\..\AuMain.cpp`, line L). `0x1400D6BD0` sets all to 0. The getters gate features (1.4).

| T | Counter (getter) | Name in log | L |
| --- | --- | --- | --- |
| 113 | `dword_1402838A4` (`0x1400D6B50`) | NameSrv | 404 |
| 1010 | `dword_140283184` (`0x1400D6B90`) | UpdateSrv | 378 |
| 1036 | `dword_140283194` (`0x1400D6AE0`) | ExAuthSrv | 443 |
| 1041 | `dword_140283180` (`0x1400D6B70`) | SecondaryAuthSrv | 365 |
| 1049 | `dword_140283188` (`0x1400D6B00`) | ICafeSrv | 391 |
| 1081 | `dword_14028318C` (`0x1400D6AF0`) | GameTimeSrv | 417 |
| 1084 | `dword_140283190` (`0x1400D6B10`) | IpVerifySrv | 430 |
| 1085 | `dword_140283198` (`0x1400D6B20`) | MobileAuthSrv | 456 |
| 1087 | `dword_14028319C` (`0x1400D6B30`) | MobileFriendSrv | 469 |
| 1121 | `dword_1402831A0` (`0x1400D6B60`) | MobileFriendSrv (copy text) | 482 |

`quirk_rolesrv_log_text`: type 1121 gates the `Role` forward (`0x1400CEED0`), so it is RoleSrv ❓, but the log text says MobileFriendSrv.

### 1.9 Calls out (message builders, `MsgSetCmd` `0x1401214E0`, 121 callers)

Targets seen: `Auth/*` (own protocol, grid: LoadUserInfo, LoadLoginInfoOnLocal, GetUserInfo, Authenticate…), `Access/Kick`, `AppAuth/ListAppGroups` (`0x140112180`), `EventLog/LogEvent` (5 builders), `ExAuth/*` (11), `Follow/RemoveUser`, `Friend/RemoveUser`, `GameAccount/ListAccounts` (`0x140114360`), `GameAccount/RemoveAccounts`, `GameTime/GetGameTime`, `ICafe/CheckIp`, `IpAuthz/RemoveUser`, `IpVerify/DeleteUser|PageIps|RegisterIp|UnregisterIp|VerifyIp`, `MobileAuth/RemoveMobileInfo`, `MobileFriend/UserRemoved`, `MobilePush/RemoveUser`, `Name/*` (9), `Presence/RemoveUser`, `Privacy/RemoveUser`, `Role/DeleteUser`, `Role/Inquiry` (`0x140117980`), `SecondaryAuth/*`, `Update/Update`. Per command use: column `calls_out` of `02-commands.tsv`.

### 1.10 Timers

| Timer | Period | OnTimer | Fact |
| --- | --- | --- | --- |
| Name session (`off_1402323B0`) | `TimerSleepSecs`×1000 when `NameSession/Use` ≠ 0 and `TimerSleepSecs` ≠ 0, else −1 (off) | `0x1400D8D20` → `0x1400D92E0` | proc `GetNameSessionInfoByTerm(MaxTryCount, SessionFromAfterSecs)`; returns the same period |
| Tentative cleanup (`qword_140283900`) | `Interval`×1000 when `Interval` ≠ 0, else −1 | ❓ | `DeleteCount` copied to `dword_140232328`; proc `AcctDeleteTentatives` (`0x1400FB11D`) ❓ link |

Armed in `0x1400D89D0` (start) and `0x1400D94B0` (every config load). `0x1400D89D0` also calls `0x140144560(5, &unk_140232300)` ❓.

## 2. AppAuthSrv

All addresses in this section are AppAuthSrv (`AppAuthSrv.exe.i64`). The recipe ran: `table()` 296 rows, 13 maps. The only service map that the service dispatches on is `AppAuth` (`0x14023D1A0`, 49 rows, ids 0–48). The other non-framework maps (`Exec` `0x14023D1C0`, `Session` `0x14023D200`, `Token` `0x14023D220`, `Presence`-type `0x14023D240`, `0x140240760`) have no user except their static initializer and `atexit` (`fn` refs only `0x1400C2B80`…`0x140169F60`), so they are client maps that the program does not use.

### 2.1 Role and init

- Init `AaInit` `0x1400D4DA0`: `StsSetServerType(1039, 0)`, `unknown_libname_6_0("AppAuth", …)`, `StsRouteStart(factory 0x1400C8240)`, provider name `AppAuth` (`0x14012E6B0`), `qword_1402396A8 = 0x14010D1B0(&off_1401EA300)` ❓, `StsConfigMonitorAdd(L"AppAuth.ini", …)`, `StsAppCommonStart(0)`, then `0x1400C8480` ❓.
- The service owns the app registry in DB `AppDb` (schema `v2016112501`): apps, secrets, app members, app groups, permissions, external apps (OAuth-type key/secret per provider), app service codes and regions, access/refresh tokens and user app agreements.
- Callers: other services and web front ends through the route bus (protocol `AppAuth`). AuthSrv calls `AppAuth/ListAppGroups` (AuthSrv `0x140112180`, 1.9). There is no client gate check in the entry: any route peer can send any command. `AaOnMessage` calls only `AaParse`, the dispatchers and `StsRouteReply` / `ReplyError` (abuse note `exploit_appauth_export_secrets`).

### 2.2 Config keys (`AppAuth.ini`, notify `AaReadCfg` `0x1400D4E40`)

| Section / key | Default | Global, getter | Use |
| --- | --- | --- | --- |
| `CacheObjTimeout/timeout` (s) | 86400 | `dword_140239688` (×1000; −1 stays −1), `CfgCacheObjTimeoutMs` `0x1400D4CD0` | sliding idle timeout of a cached app (2.4) |
| `LifeTime/short-term` (s) | 3600 | `dword_14023968C`, `0x1400D4D30` | used by `0x1400D2280` (id 8 `Token`) |
| `LifeTime/long-term` (s) | 2592000 | `dword_140239690`, `0x1400D4D10` | used by `0x1400D2280`, `0x1400DBB50`, `0x1400DDA50` |
| `LifeTime/refresh-term` (s) | 2592000 | `dword_140239694`, `0x1400D4D20` | used by `AddAccessToken` `0x1400D7280`, `RefreshAccessToken` `0x1400D7560` |
| `DeleteExpiredTokens/interval` (ms) | 60000 | `dword_140239698`, `0x1400D4CF0` | timer period (2.5) |
| `DeleteExpiredTokens/counts` | 100 | `dword_14023969C`, `0x1400D4CE0` | parameter of `DeleteAccessTokenByEffectiveUntil` |
| `JwtExpiration/use` | 1 | `dword_1402396A0`, `0x1400D4D40` | used by `0x1400CF660` (id 24 `AuthenticateJwtToken`) |
| `JwtExpiration/timeout` (s) | 3600 | `dword_1402396A4`, `0x1400D4D00` | used by `0x1400CF660` |

- `quirk_cache_timeout_overflow`: `AaReadCfg` stores `1000 * timeout` in a 32-bit int. A value above 2147483 s overflows (wraps). A negative result other than −1 makes every cached app expire at once on next use (`AppTouch` compare is signed).
- `AaReadCfg` calls `StsTimerSet(&off_1401EA2E8, interval, 0)` on every config load, so a new `interval` is used at once.
- Framework keys (Logs, Tls, Route `General`, Logging, …) are the same as in `00-framework.md`.

### 2.3 Entry and dispatch

1. `AaOnMessage` `0x1400C8290` (connection slot 0). `AaParse` `0x1400F3150` fills a request record (layout 2.3.1). Parse fail → `ReplyError(msg, err)` with text `parsing appauth message`.
2. `AaParse`: command name → `NameMapFind(0x14023D1A0)`. Unknown name → id 49 and error code 10, line 1036. No body element → line 1045. The error starts as `ErrInit(err, 10)`.
3. `switch(id)`:
   - Local ids 1, 20–23, 25–30, 33–39, 44–48 → `AaLocalDispatch` `0x1400C5440` (switch, one handler per id).
   - App ids 2–19, 24, 31, 32, 41–43 → `AaAppDispatch` `0x1400D1CE0` on the app cache singleton `AaAppCache` `0x1400CE920` (`qword_140239668`, 0x70 bytes, made at first use by `0x1400CC1E0`).
   - Id 40 `GetExternalAppInfo`: `AppId` (record +16, 16 bytes) not zero → app path, else local path.
   - Other ids (0 `Invalid`, 49) → `StsRouteReply` error `{52, module 5, line 169, "Invalid Command"}`.
4. `AaAppDispatch`: id 41 `AppDataUpdated` → `AaAppDataUpdated` `0x1400D3CC0`; id 42 `GetAppData` → `AaGetAppData` `0x1400CDEA0`; id 43 `CleanAppData` → `AaCleanAppData` `0x1400CCBE0`. Other ids:
   1. `AppCacheFind(cache, AppId)` `0x1400CD630`. Not found → `AppCacheLoad` `0x1400CD010` (new app object, queue the message, DB `GetAppInfo`), return.
   2. `AppTouch` `0x1400D4210` returns 0 (idle too long) → `AppCacheEvict` `0x1400D3B80`, then `AppCacheLoad`.
   3. App state (`app+160`) < 2 (load not complete) → `AppCacheQueue` `0x1400CD200` (the message waits for the load).
   4. Else `AaAppSwitch` `0x1400CEA30`. If it returns 0, `StsRouteReply(msg route id, err)`. A handler that returns non-zero replies by itself or starts a DB transaction.
5. `AaAppSwitch` default (ids that are not app ids): error `{52, module 2, line 1149, "invalid command"}`. Only reachable for id 1 and 41–48 if the table changes; `AaOnMessage` does not send them here.

#### 2.3.1 Request record (filled by `AaParse`)

| Offset | Field (XML name) | Type |
| --- | --- | --- |
| +0 | message pointer | ptr |
| +8 | command id | int |
| +12 | reply route id (message vtable +40) | int |
| +16 | `AppId` | GUID (zero GUID `xmmword_14018D860` when absent) |
| +32 | `AppName` | str |
| +40 | `SecureApp` | int |
| +48 | `AppSecret` | str |
| +56 | `RedirectUri` | str |
| +64 | `RightName` | str |
| +72 | `Permissions` | str |
| +80 | `AppType` | int |
| +84 | `AppStatus` | int |
| +88 | `IsExternalApp` | int |
| +92 | `AppGroupId` | uint (`strtoul` base 10 of a string; absent → 0) |
| +96 | `AppGroupDesc` | str |
| +104 | `AppGroupType` | int |
| +112 | `AppGroupName` | str |
| +120 | `AppGroupCode` | str |
| +128 | `Apps` element present | bool |
| +132 | `UseKeyFile` | bool (`0x1400F3090`) |
| +136 | `TokenRequired` | bool (`0x1400F3090`) |
| +140 | `PermissionId` | int |
| +144 | `PermissionIds` | element |
| +152 | `PermissionName` | str |
| +160 | `AccessMask` | uint64 (`strtoui64` base 10; absent → 0) |
| +176 | `SecureLevel` | int |
| +180 | `SecretType` | int |
| +184 | `GrantType` | int |
| +192 / +200 / +208 / +216 / +224 | `ProjectName` / `ProjectDesc` / `ProjectIcon` / `PolicyUri` / `AllowedDomain` | str |
| +232 | `UserId` | GUID |
| +248 | `Scopes` | int |
| +256 | `FromUri` | str |
| +264 | `AuthzCode` | str |
| +272 | `KeyId` | GUID |
| +288 | `Token` | str |
| +296 | `LoginName` | str |
| +304 | `Creater` (spelling of the original) | int |
| +308 | `MemberType` | int |
| +312 | `PublicKey` | str |
| +344 | `RefreshToken` | str |
| +352 / +360 / +368 | `AuthProviderCode` / `ExternalAppKey` / `ExternalAppSecret` | str |
| +376 / +384 | `ExternalScopeData` / `RedirectUriData` | str |
| +392 | `TokenDataType` | int |
| +396 | `Scope` | value of vtable +208 (enum or list ❓) |
| +400 / +408 | `AppServiceCode` / `RegionCode` | str |

All fields are optional in the parse; each handler checks the fields that it needs. Int fields use message vtable +216, string fields +200, GUID +192, element +168.

### 2.4 App cache

- `AppTouch` `0x1400D4210`: `now = timeGetTime()`. If `CacheObjTimeoutMs == -1` or `timeout + app.last(+328) - now >= 0` → keep (1), else expired (0). In both cases `app+328 = now` (sliding timeout).
- `quirk_touch_resets_on_expiry`: the expired path also writes `now`; the caller evicts the object at once, so no effect is visible ❓.
- DB load: `AppDbGetAppInfo` `0x1400D3E10` writes proc `GetAppInfo` with a separate writer `0x1400F3A90` (not `DbProcBegin`; `dbprocs()` misses it) and a 0x230-byte transaction object (`0x1400D6010`).
- Load chain (txn types, 2.6): type 0 reply `0x1400D8D00` reads `AppId`, `AppName`, `AppType`, `AppStatus`, `IsExternalApp`, `SecureLevel`, `Changed`, `AccessMask`, `Registered`, then sends proc `GetSecrets` (`0x1400D9021`). Type 1 reply `0x1400D8B50` stores the secrets. Type 13 reply `0x1400DA450` (proc `GetAppGroupByAppId`, `0x1400DA686`) reads `Set/Row`. Errors: `bad app info reply`, `on transaction: invaild data`, `missing data`.
- Load fail (txn OnError `0x1400D7930`, types 0, 1, 13): `0x1400D39D0` removes the app, `0x1400CD090` replies the error to all queued messages. Special: type 1 with error 3734 and type 13 with error 3752 are treated as OK with an empty result (`ErrInit(3734)` → `0x1400D8B50`; `ErrInit(3752)` → `0x1400DA450`) ❓ meaning (no secrets / no group).

### 2.5 Commands (`AppAuth`, ids 0–48)

Paths: L = `AaLocalDispatch` case, A = `AaAppSwitch` case (app must be in cache), C = answered from the cached app (no DB). TX n = DB transaction type n (2.6). Per-row details also in `02-commands.tsv`. Every DB call uses `SendToProviderTxn(msg, txn, 25000, 0)` (timeout 25000 ms). Request errors are `{10, module 5, line, text}` unless the table gives other values.

| Id | Name | Handler | Path | Request fields used | DB (TX → reply builder) | Reply / errors |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | RegisterApp | `0x1400C7B90` | L | AppId (zero → new GUID `0x1400EA6A0`), AppName, AppType, AppStatus, IsExternalApp, AccessMask | `RegisterApp`(g,s,i,i,i,i,i); TX 8 → `0x1400DC670` | `AppId`; `missing parameters` (no AppName or AccessMask 0); `bad register app reply` |
| 2 | UnregisterApp | `0x1400D27F0` | A | AppId | `UnregisterApp`(g); TX 10 → `0x1400DD4A0` (evicts cache) | empty `Reply` |
| 3 | ChangeApp | `0x1400D0490` | A | AppId, AppName, AppType, AppStatus, AccessMask | `ChangeApp`; TX 9 (`0x1400D6790`) → `0x1400D9130` | `Reply`; also sends `Update` (`StsRouteSend`) with AppName, AppType, AppStatus, AccessMask ❓ target; `invalid parameter` |
| 4 | GetAppRights | `0x1400D1170` | A, C | AppId | — | `AccessMask` |
| 5 | GetAppInfo | `0x1400D0F60` | A, C | AppId | — | `AppName`, `AppType`, `AppStatus` |
| 6 | Authenticate | `0x1400CF340` | A, C | AppId, AppSecret, RedirectUri | — | `AccessMask`, `AppGroupCode`, `AppServiceCode`; errors `{?, 2, 1261}`, `{?, 2, 1269}` `app authenticate failed`, `invalid redirect uri`, `missing appsecret` |
| 7 | Authorize | `0x1400D02B0` | A | AppId, UserId, Scopes (2.8) | TX 16 (`0x1400D5EC0`) → `0x1400DD150` → proc `AddUserAppAgreement` → TX 18 `0x1400D84E0` | `AuthzCode`, `TokenId`, `EffectiveUntil`; `invalid parameter`, `invaild data from token server` |
| 8 | Token | `0x1400D2280` | A | GrantType, AuthzCode, RefreshToken, AppSecret | uses `LifeTime` short/long; TX 17 `0x1400DDA50` (`GetUserAppAgreement`), TX 19 `0x1400DBB50`, TX 27/28 `0x1400D7B30`/`0x1400DC3C0` (`AddAccessToken` `0x1400D7280`, `RefreshAccessToken` `0x1400D7560`); order per grant in 2.8 | `AccessToken`, `Expiration`, `RefreshToken`, `RefreshTokenExpiration`; errors line 1529, 1541; texts `invalid grantType`, `missing parameter:authz_code`, `refreshToken required, but refreshToken is null`, `external app should not use 'grant_credentials' grantType.`, `parsing token failed`, `app authenticate failed`; string `127.0.0.1` ❓ |
| 9 | ListAppGroups | `0x1400D1B10` | A, C | AppId, AppGroupType | — | array of `AppGroupId`; `invalid group type` |
| 10 | GetAppGroupCode | `0x1400D0DC0` | A, C | AppId | — | `AppGroupCode`; `has no app group code` |
| 11 | AddSecret | `0x1400CEE00` | A | AppId, SecretType, RedirectUri | `AddServerSecret` or `AddNativeAppSecret`; TX 11 (`0x1400D6900`) → `0x1400D7DE0` | `KeyId`, `AppSecret` or `PublicKey`/`PrivateKey`; `already exist server secret`, `already exist native app secret`, `invalid secret type.` |
| 12 | GetSecrets | `0x1400D1820` | A, C | AppId | — | `Keys` array of `Key{KeyId, SecretType, AppSecret, RedirectUri}` or `{KeyId, SecretType, PublicKey, PrivateKey}` |
| 13 | RemoveSecret | `0x1400D2180` | A | KeyId, SecretType | `RemoveSecret`(g,i); TX 15 → `0x1400DCF90` | `Reply`; `invalid secret type` |
| 14 | ChangeServerSecret | `0x1400D0710` | A | KeyId, RedirectUri | `ChangeServerSecret`(g,s); TX 14 → `0x1400D9730` | empty `Reply` |
| 15 | ExportSecrets | `0x1400D09C0` | A, C | AppId | — | AppId, AppName, AppType, AppStatus, IsExternalApp, AccessMask, `Keys` (incl. `PrivateKey`), `AppGroups` |
| 16 | AddAppMember | `0x1400CEC30` | A | AppId, UserId, MemberType, LoginName | `AddAppMember`; TX 23 → `0x1400DCEE0` | empty `Reply`; `missing userid or memberType is zero` |
| 17 | RemoveAppMember | `0x1400D1FF0` | A | AppId, UserId | `RemoveAppMember`; TX 24 | `missing userid` |
| 18 | CheckAppMember | `0x1400D0810` | A | AppId, UserId, MemberType | `CheckAppMember`; TX 25 | `missing userid or memberType is zero` |
| 19 | GetAppMembers | `0x1400D1080` | A | AppId | `GetAppMembers`; TX 26 → `0x1400DA720` | array `App{UserId, MemberType, LoginName}` |
| 20 | GetMemberApps | `0x1400C73A0` | L | UserId | `GetMemberApps`; TX 20 → `0x1400DB630` | array `App{AppId, AppName}`; `missing userid` |
| 21 | GetMemberAppGroups | `0x1400C7230` | L | UserId | `GetMemberAppGroups`; TX 21 → `0x1400DB450` | array `AppGroup`; `missing userid` |
| 22 | AddMemberAppGroup | `0x1400C5820` | L | AppGroupId, UserId, MemberType, LoginName | `AddMemberAppGroup`; TX 22 → `0x1400DCEE0` | `missing userid / appGroupId / memberType` |
| 23 | AuthenticateBearerToken | `0x1400C5BC0` | L | Token (parsed; type 1–3 with a GUID) | `GetAccessToken`(0, Token); TX 12 (`0x1400D6630`) → `0x1400D9EF0` | `AccessTokenType`, `AppId`, `UserId`, `PermissionMask`, `EffectiveUntil`…; `{10,5,435} invalid token`, `{4,5,461} parsing token failed`, `access token expired` |
| 24 | AuthenticateJwtToken | `0x1400CF660` | A | Token (JWT), TokenRequired | uses `JwtExpiration/use`, `/timeout`; TX 29 (`0x1400D6A40`) → `0x1400D97E0` | `AppId`, `AccessMask`, `TokenType`, `AppGroupCode`, `AppServiceCode`; errors lines 1309, 1329, 1336, 1376; algs `RS256`, `HS256`; texts `invalid jwt format`, `expired jwt`, `invalid iat`, `failed verify signatrue`, `server app secret not found`, `native app secret not found`, `no exist jwt` |
| 25 | GetAccessToken | `0x1400C6AF0` | L | Token or RefreshToken, TokenDataType (0/1) | `GetAccessToken`; TX 12 → `0x1400D9EF0` | `invalid token data` |
| 26 | CreateGroup | `0x1400C64F0` | L | AppGroupName, AppGroupDesc, AppGroupType, AppGroupCode | `CreateAppGroup`; TX 36 → `0x1400D9930` | `AppGroupId`; `missing or invalid parameters` |
| 27 | DeleteGroup | `0x1400C6840` | L | AppGroupId | `DeleteAppGroup`(i); TX 37 (`0x1400D6140`) → `0x1400D9CA0` | `AppId` list; `{10,5,279} invalid groupId` |
| 28 | GetGroupInfo | `0x1400C70D0` | L | AppGroupId | `GetAppGroupInfo`; TX 39 → `0x1400DAF00` → proc `ListAppGroupApps` → TX 40 `0x1400DB220` | AppGroupName, AppGroupDesc, AppGroupType, AppGroupCode, `Apps` array `App{AppType…}`; `invalid app group Id` |
| 29 | ListGroups | `0x1400C78A0` | L | AppGroupType | `ListAppGroups`; TX 43 → `0x1400DBF60` | array `AppGroup{AppGroupId, AppGroupName, AppGroupDesc, AppGroupCode}`; `invalid group type` |
| 30 | ListGroupApps | `0x1400C7760` | L | AppGroupId | `ListAppGroupApps`; TX 44 → `0x1400DBDC0` | array `App`; `invalid group id` |
| 31 | RegisterGroupApp | `0x1400D1E30` | A | AppId, AppGroupName, AppGroupType | `RegisterAppGroupApp`; TX 41 → `0x1400DCB50` | `invalid appid or appgroupid`, `bad data` |
| 32 | UnregisterGroupApp | `0x1400D28F0` | A | AppGroupId, AppId | `UnregisterAppGroupApp`; TX 42 → `0x1400DD810` | `invalid appid or appgroupid` |
| 33 | GetPermissions | `0x1400C7510` | L | PermissionIds | `GetPermissions`; TX 7 → `0x1400DB8D0` | `Permissions` array; `missing permissionIds` |
| 34 | CreatePermission | `0x1400C6690` | L | PermissionName, AccessMask, SecureLevel | `CreatePermission`; TX 4 → `0x1400D9A90` | `PermissionId`; `missing accessmask`, `missing permission name` |
| 35 | ChangePermission | `0x1400C61F0` | L | PermissionId, AccessMask, SecureLevel | `ChangePermission`; TX 5 → `0x1400D9730` | empty; `missing permission id` |
| 36 | DeletePermission | `0x1400C69A0` | L | PermissionId | `DeletePermission`; TX 6 → `0x1400D9730` | empty |
| 37 | RegisterExternalApp | `0x1400C7DB0` | L | AppGroupCode, AuthProviderCode, ExternalAppKey, ExternalAppSecret, RedirectUriData, ExternalScopeData | `AddExternalSecret`; TX 45 (`0x1400D6290`) → `0x1400DC860` | `missing …` per field |
| 38 | UnregisterExternalApp | `0x1400C8070` | L | AppGroupCode, AuthProviderCode | `RemoveExternalSecret`; TX 46 (`0x1400D64B0`) → `0x1400DD5A0` | `missing authProviderCode`, `missing appGroupCode` |
| 39 | ChangeExternalAppInfo | `0x1400C5F30` | L | same as 37 | `ChangeExternalSecret`; TX 47 → `0x1400D9410` ❓ | `missing …` per field |
| 40 | GetExternalAppInfo | `0x1400C6E80` (L) / `0x1400D1250` (A) | L or A | AppId or AppGroupCode, AuthProviderCode | `GetExternalSecret` or `ListExternalSecrets`; TX 48 → `0x1400DAC00` | `ExternalApps` array; `missing appId and appGroupCode` |
| 41 | AppDataUpdated | `0x1400D3CC0` | grid | AppId | — | evicts cached app ❓ |
| 42 | GetAppData | `0x1400CDEA0` | grid | AppId | — | `Reply` (cache dump ❓) |
| 43 | CleanAppData | `0x1400CCBE0` | grid | ❓ | — | `Reply` |
| 44 | CreateAppServiceCode | `0x1400C6390` | L | AppServiceCode | `CreateAppServiceCode`; TX 49 → `0x1400D9730` | `missing appServiceCode` |
| 45 | AddAppServiceCodeRegion | `0x1400C5660` | L | AppServiceCode, RegionCode | `AddAppServiceCodeRegion`; TX 50 | `missing regionCode`, `missing appServiceCode` |
| 46 | AddRegionalAppGroup | `0x1400C59E0` | L | AppGroupId, AppServiceCode, RegionCode | `AddRegionalAppGroup`; TX 51 → `0x1400D81C0` | `missing appGroupId` … |
| 47 | GetAppServiceCode | `0x1400C6C80` | L | AppGroupCode+AppGroupType or AppId | `GetAppServiceCodeByAppGroupCode` / `…ByAppId`; TX 52 → `0x1400DAA10` | `AppServiceCode`, `RegionCode`, `AppGroupCode` |
| 48 | ListRegionalAppGroups | `0x1400C79F0` | L | AppServiceCode, AppGroupType (optional) | `ListRegionalAppGroupsByAppServiceCodeAndAppGroupType` / `…ByAppServiceCode`; TX 53 → `0x1400DC1D0` | `RegionalAppGroups` array `{RegionCode, AppGroupId, AppGroupType}` |

### 2.6 DB transaction object

- Ctor `0x1400D5AD0` (variants `0x1400D6010`, `0x1400D6140`, `0x1400D6290`, `0x1400D64B0`, `0x1400D6630`, `0x1400D6790`, `0x1400D6900`, `0x1400D6A40`, `0x1400D5EC0`; all call base init `0x1400D77A0`), 0x230 bytes, vtable `0x140171CB0`. Type at +120, reply route id at +32, AppId at +96.
- Slot +8 `TxnOnReply` (framework), slot +24 OK `0x1400D7A10` → switch `0x1400D6F70` on type (types 3, 30–35, 38 have no builder), slot +32 error `0x1400D7930` (2.4; other types → `StsRouteReply(route id, err)`).

### 2.7 Timer and procs

- Expired token delete: `StsTimerSet(&off_1401EA2E8, DeleteExpiredTokens/interval, 0)` in `AaReadCfg`. OnTimer `AaDelExpTokensTimer` `0x1400D7A70`: proc `DeleteAccessTokenByEffectiveUntil(counts)` by `StsRouteSend` (no reply wait); returns `interval` (re-arm).
- Procs: 50 (49 from `dbprocs()` plus `GetAppInfo`), all `AppDb` / `v2016112501`. List in `02-procs.tsv`. `procs.tsv` has no AppAuthSrv rows, so there is no expected count.

### 2.8 Handler steps (OAuth-type flow)

Cached app object fields used here: +56 `IsExternalApp`, +60 `SecureApp` (secret check on), +132..+144 `AppId` (4 dwords), +176 `AccessMask`, +184 / +200 app group list (link offset / head; group +176 type, +180 code), +296 secret object (vtable +16 returns the secret string). Secret compare `0x1400DE620` is `strncmp(a, b, -1)` (byte compare to the first NUL; no NULL check).

**Id 6 Authenticate `0x1400CF340`** (no DB):
1. `IsExternalApp` (+56) ≠ 0 and no `RedirectUri` → `{3731, module 2, line 1251, "invalid redirect uri"}`.
2. `SecureApp` (+60) = 0 → go to step 4 (no secret check).
3. If the secret object exists, its string is not empty, and the request has no `AppSecret` → `{3730, 2, 1261, "missing appsecret"}`. Else, if the secret object exists, compare it with the request `AppSecret`; not equal → `{3731, 2, 1269, "app authenticate failed"}`.
4. Find the first app group with type 6 (group +176) in the app group list; take its code (+180) as `AppGroupCode` and the service code ❓ field. Reply `Reply{AccessMask = app+176, AppGroupCode, AppServiceCode}` by `StsRouteReply`.
- `quirk_authn_no_secret_obj`: with `SecureApp` ≠ 0 and no secret object (+296 = 0, app has no server secret), step 3 does no compare and the reply is OK without a secret.
- `quirk_authn_empty_secret_null`: with a stored secret that is an empty string and no request `AppSecret`, step 3 calls the compare with a NULL second pointer. The compare reads address 0 (access violation) ❓ (not run).

**Id 7 Authorize `0x1400D02B0`**:
1. `AppId` (+16) and `UserId` (+232) must not be zero, else `{10, module 2 ❓, line 1482 ❓, "invalid parameter"}`.
2. `UserId` → text (`0x140105ED0`, 38 bytes). Txn type 16 (`0x1400D5EC0`: route id, AppId, UserId, `Scopes` +248, app `IsExternalApp`).
3. Send `Token.Instant` `RequestToken` (`0x1400F6DE0`, attribute `id="random"`): `AppId`, `EffectiveUntilSecs` = 10, `TokenData` = UserId text, `TokenSecretType` = 1, `TokenSecret` = none. `SendToProviderTxn(…, 25000)`.
4. Reply (TX 16 → `0x1400DD150`): reads `TokenId` and `EffectiveUntil`; missing → `"invaild data from token server"`. If the app is external (txn +28 ≠ 0): proc `AddUserAppAgreement` (TX 18, `0x1400D5BE0`), and the reply comes from TX 18 `0x1400D84E0`. Else reply `Reply{AuthzCode = TokenId}` at once.

**Id 8 Token `0x1400D2280`**, switch on `GrantType` (+184):
- 1 (authorization code):
  1. Request `AppId` must be non-zero and equal to app +132..+144, else `{3731, 2, 1523, "app authenticate failed"}`.
  2. No `AuthzCode` (+264) → `{10, 2, 1529, "missing parameter:authz_code"}`.
  3. Parse `AuthzCode` as a GUID (`0x140105DF0`, 37-byte buffer); fail → `{4, 2, 1541, "parsing token failed"}`.
  4. Txn type 17 (`0x1400D5D20`: route id, AppId, code GUID, `IsExternalApp`, `AccessMask`, `AppSecret`, `RedirectUri`). Send `Token.Instant` `ValidateToken` (`0x1400F6F30`, `TokenSecret` = code GUID), route text `;appid=<AppId>;client=127.0.0.1` (`0x1400D40C0`).
  5. TX 17 reply `0x1400DDA50`: reads `TokenData` (the UserId); bad → `"invaild data from token server"` (lines 1837, 1843). External app (txn +28 ≠ 0) → proc `GetUserAppAgreement` (TX 19; reply `0x1400DBB50` reads `AppId`, `UserId`, `AgreementMask`; errors lines 1533, 1541, 1548, 1552 `… (get app agreement)`) and then `AddAccessToken(type 2, …)`. Else `AddAccessToken(route, 2, AppId, UserId, refresh 1, mask = txn +56, LifeTime/long-term)` at once.
- 2 (client credentials):
  1. `IsExternalApp` ≠ 0 → `{10, 2, 1572, "external app should not use 'grant_credentials' grantType."}`.
  2. No `AppSecret` → lifetime `LifeTime/short-term`, no refresh token.
  3. With `AppSecret`: secret object missing or compare not equal → `{3731, 2, 1585, "app authenticate failed"}`. Equal → lifetime `LifeTime/long-term`, refresh flag 1.
  4. `AddAccessToken(route, 1, AppId, zero UserId, refresh, AccessMask, lifetime)`.
- 3 (refresh): `RefreshToken` (+344) missing or empty → `{10, 2, 1610, "refreshToken required, but refreshToken is null"}`. Else `RefreshAccessToken(route, RefreshToken, LifeTime/long-term)`.
- Other → `{10, 2, 1628, "invalid grantType"}`.
- `quirk_token_cc_no_secret`: grant 2 without `AppSecret` gives an access token (short term) with no secret check, also for an app with `SecureApp` set. Only the AppId is needed.

**`AddAccessToken` `0x1400D7280`** (route, token type, AppId, UserId, refresh flag, mask, lifetime): makes a new GUID; `AccessToken` text = `"%d:%s"` (1, GUID text). Proc `AddAccessToken` (TX 27) with AppId, UserId (only if non-zero and type 2 ❓ condition reads "UserId non-zero and type 2"), refresh flag, mask, lifetime, and for type 2 with refresh flag a second lifetime `lifetime + refresh-term` ❓. **`RefreshAccessToken` `0x1400D7560`**: proc `RefreshAccessToken` (TX 28) with the token text and lifetime.

### 2.9 Calls out

| Target | Command | Call site | When |
| --- | --- | --- | --- |
| Token (`Token.Instant`) | `RequestToken` (`0x1400F6DE0`) | Authorize `0x1400D02B0` | make an authz code (10 s) |
| Token (`Token.Instant`) | `ValidateToken` (`0x1400F6F30`) | Token `0x1400D2280` grant 1 | check an authz code |
| ❓ (`StsRouteSend`) | `Update` | ChangeApp reply `0x1400D9130` | after `ChangeApp` |
| AppDb | `DeleteAccessTokenByEffectiveUntil` | `AaDelExpTokensTimer` `0x1400D7A70` | timer |

### 2.10 Worked example (Token, grant 2)

Request `AppAuth/Token`: `AppId = {A1…}`, `GrantType = 2`, `AppSecret = keyhere`. App in cache: `IsExternalApp = 0`, `SecureApp = 1`, secret = `keyhere`, `AccessMask = 7`. Steps: grant 2 → not external → secret equal → lifetime 2592000 (long-term), refresh 1 → `AddAccessToken(route, 1, {A1…}, zero, 1, 7, 2592000)` → TX 27 reply `0x1400D7B30` → reply `AccessToken = "1:<new GUID>"`, `Expiration` ❓ (reply field values not traced in `0x1400D7B30`). `AddAccessToken` makes a refresh token only for token type 2 (both checks at `0x1400D7280` need `type == 2`), so grant 2 (type 1) never gets a `RefreshToken`; the refresh flag has no effect. Same request without `AppSecret`: lifetime 3600.

## 3. Framework gaps

- `0x1400D7040` pattern: AuthSrv runs three service pre-check switches before the main switch; `table()` sees only the main switch. Ids in the pre-checks have no `table()` handler.
- FLIRT names `GeoIP_num_countries_N` and some `unknown_libname_N` in AuthSrv are one-line global getters (`mov eax, global; retn`), not library code.
- `IniGetInt` index argument: a non-zero index with default 0 (`GridObjectTimeoutMs`) makes the key useless (1.2).
- AppAuthSrv `GetAppInfo` uses a second DB proc writer `0x1400F3A90` (not `DbProcBegin`). The recipe `dbprocs()` does not see it; add it to the recipe.
- AppAuthSrv string compare `0x1400DE620` is `strncmp` with count −1 and no NULL check.
- `Token.Instant` client builders in AppAuthSrv: `RequestToken` `0x1400F6DE0`, `ValidateToken` `0x1400F6F30`.
- Error module ids: AppAuthSrv uses module 5 (parse, dispatch) and 2 (app handlers). AuthSrv uses module 5 (`AuMain` parse, grid) and 6 (handlers), and module 1 for the not-supported reply (51).

## 4. Differences from existing specs

- `15-authsrv.md` §1 "Pre-checks ❓": now decoded (1.3, 1.4). §7.5 "ConsumeGameToken not decoded": now 1.6.
- `15-authsrv.md` §7.5 says game token list head at user +264; this chapter reads the first element at user +272 with link offset +256 (`0x1400E9000`). +264 may be the list object start ❓.
- `15-authsrv.md` §1 "forwards the request to the owner": `0x1400ECF00` sends `Auth/LoadUserInfo` to the grid owner and keeps the message; it does not forward the original request ❓.
- `15-authsrv.md` §7.1 `TokenType` "use not decoded": used by the location check `0x1400F63B0` (1.6).
- Not yet compared: the rest of `15-authsrv.md` (SRP, KeyData, LoginFinish) was not checked again in this stage.
