# 03 — GameAccountSrv, SlotSrv, SpawnSrv, WorldSrv (RE reference)

Summary:
1. GameAccountSrv (server type 1030, protocol `GameAccount`, 21 ids, map `0x140239030`) keeps the link user → game account per game code, the last presence, and the account move flow. DB `GameDb` / `v2016120901`.
2. SlotSrv (server type 1022, protocol `Slot`, 13 ids) keeps product slots (create, lock, expire, tag policy). DB `SlotDb` / `v2016041101`. See §2.
3. SpawnSrv (server type 100) starts and watches the other StServers programs. It is the server side of the framework `SpawnSrv` map. It has no DB. See §3.
4. WorldSrv (server type 1033, protocol `World`, 6 ids, map `0x140226620`) keeps the world list in DB `GameDb` (procs `ListWorlds`, `AddWorld`, ...). The world list does not come from an INI file. See §4.
5. Lists: `03-commands.tsv` (all command ids), `03-procs.tsv` (all proc call sites).

All addresses are in the binary of the section. ❓ = not verified in IDA. Framework names (`MsgInit`, `DbProcBegin`, `SendToProviderTxn`, `StsRouteReply`, `ReplyError`, `StsTimerSet`) are from `framework-names.tsv` and `00-framework.md`.

## Quick find

| Question | Function | Binary and address | Section |
| --- | --- | --- | --- |
| GameAccount init | `GaInit` | GameAccountSrv `0x1400CCD30` | 1.1 |
| GameAccount config (`GameAccount.ini`) | `GaReadCfg` | GameAccountSrv `0x1400CCE70` | 1.2 |
| GameAccount message entry | `GaOnMessage` (vtable slot) | GameAccountSrv `0x1400CD4C0` | 1.3 |
| GameAccount pre-load (user game list) | `GaPreload` | GameAccountSrv `0x1400CD760` | 1.3 |
| GameAccount main switch | `GaDispatch` | GameAccountSrv `0x1400C6E80` | 1.3 |
| GameAccount `Exec` sub-switch (ids 15, 16) | `GaExec` | GameAccountSrv `0x1400C47B0` | 1.3 |
| GameAccount txn step switch | `GaTxnStep` | GameAccountSrv `0x1400C9340` | 1.5 |
| Account move token record / timer | — | GameAccountSrv `0x1400CAE30` / `0x1400C6700` | 1.5 |
| GameAccount `Game.ini` `GameDbConfig` | — | GameAccountSrv `0x1400CD140` | 1.2 |
| Slot init | — | SlotSrv `0x1400C7A40` | 2.1 |
| Slot config (`Slot.ini`) | — | SlotSrv `0x1400C7B70` | 2.2 |
| Slot global switch (no user) | — | SlotSrv `0x1400D4A70` | 2.3 |
| Slot user switch | — | SlotSrv `0x1400D88A0` | 2.3 |
| Slot `CreateSlots` checks | — | SlotSrv `0x1400D6020` | 2.4 |
| Slot tag lock worker | `CSlotExtension` | SlotSrv `0x1400D4710` | 2.5 |
| Slot retry timer | — | SlotSrv `0x1400C8260` | 2.5 |
| Spawn init | — | SpawnSrv `0x1400DCA40` | 3.1 |
| Spawn config (`Spawn.ini`) | — | SpawnSrv `0x1400D99C0` | 3.2 |
| Spawn process launch | — | SpawnSrv `0x1400D8880` | 3.3 |
| Spawn child message dispatch | — | SpawnSrv `0x1400D5060` | 3.4 |
| Spawn connect broker | — | SpawnSrv `0x1400D6830` | 3.4 |
| World init | — | WorldSrv `0x1400C5260` | 4.1 |
| World config (`World.ini`) | — | WorldSrv `0x1400C5340` | 4.2 |
| World list load (proc `ListWorlds`) | — | WorldSrv `0x1400C7C60` | 4.1 |
| World command switch | — | WorldSrv `0x1400C8700` | 4.3 |

## 1. GameAccountSrv

### 1.1 Role and init (`0x1400CCD30`)

GameAccountSrv maps a platform user (UserId) to one game account (GameAccountId) for each game code. It also stores the last presence (world, time) of a user per game, and runs the "account move" flow (MoveStart / MoveCheck / MoveEnd) that changes the owner user of a game account. Callers: game servers and gates through the route bus (`$GameAccount.*` resolver names), other GameAccountSrv instances (forward path, `FwdGameAcctTxn.cpp`).

Init order (`0x1400CCD30`):
1. `StsSetServerType(1030, 0)`.
2. `unknown_libname_6_0("GameAccount", 0, &3)` ❓ (same call as HelloSrv; not decoded).
3. If `[UserName] UniqueProperty` of `Auth.ini` is not 0 (`0x1400DE6F0`, default 1): add the resolver `GameAccount` / (user name key) → `$Name.GetIdByUserName` (`0x14011CC10`, error 3002, module 4).
4. Resolvers (`0x14011CC10`): `login` → `$Name.GetIdByLoginName`; `gacct` → `$GameAccount.GetUserIdByAcct`. So a route address `gacct:<code>:<id>` ❓ is resolved by this service with id 15.
5. `StsConfigMonitorAdd(L"GameAccount.ini", &off_1401E81D0)` (notify slot 0 = `0x1400CCE70`, §1.2) and `StsConfigMonitorAdd(L"Game.ini", off_1401E81C8)` (notify slot 0 = `0x1400CD140` ❓ not decoded; game code list ❓).
6. `0x1400CE660`: register txn classes `GameAcctTxn`, `CacheTxn`, `AppGroupCodeTxn` (`0x1400FA900`).
7. `0x1400C6500` ❓ (48 bytes, global init), `0x1400FA1A0(app+16, 0)` (grid start ❓, also adds a config monitor).
8. `0x1400FA390("GameAccount", 86400000)`: grid object timeout 1 day ❓ (same call form as AuthSrv `0x140135160`).
9. `StsAppCommonStart(0)`.

Vtable `0x14016B218` (service socket notify): +0x00 `0x1400CCAF0` ❓, +0x08 `0x1400CD4C0` (OnMessage, §1.3), +0x10 `0x1400CD760` (pre-load, §1.3), +0x18 `0x1400CD400` (run main switch). Vtable `0x14016B200`: `0x1400CD5D0` / `0x1400CD690` count the connections of type 1010 (UpdateSrv ❓): logs `s_updateSrvs(%d) loc(%s)` and `UpdateSrv connection counter reached 0 already` (level 2).

### 1.2 Config keys

| File | Section | Key | Default | Global | Use |
| --- | --- | --- | --- | --- | --- |
| `GameAccount.ini` | `AccountMove` | `ExpirationSec` | 60 | `dword_1401E8110` | life of a move token in s (`0x1400CAE30`: due = now + 1000 × value) |
| `GameAccount.ini` | `AccountMove` | `RetryCount` | 5 | `dword_1401E8114` | retry limit of MoveStart: getter `0x1400CE0E0` (FLIRT misname `GeoIP_num_countries_0`), only reader `0x1400CFC74`. `0x1400CFC40(…, a4)` returns 0 at once when attempt `a4` > `RetryCount`. The getter of `ExpirationSec` is `0x1400CE250` (8 readers). |
| `GameAccount.ini` | (first section, read with `IniValueAsInt` index 0..3) | ❓ | 1, `dword_1401E8120`, `dword_1401E8130`, `dword_1401E8140` | `dword_1401E8280`, `dword_1401E81D8` (`0x140146320`) | ❓ two value sets; not decoded (`0x1400CCE70` head) |
| `GameAccount.ini` ❓ | `Forwards` | `Forward` (many) | — | — | `0x1400CDBD0`: each line = a forward route (`0x14011CC10`, error 3010); bad line: log `invalid config(forward), linenum(%u)` |
| `Auth.ini` | `UserName` | `UniqueProperty` | 1 | — | `0x1400DE6F0`; 0 = no user name resolver (§1.1 step 3) |
| `Game.ini` | `GameDbConfig` | `<game code>` = `<text>` (many lines) | — | hash `dword_1401E8210` (0x50-byte rows) | notify `0x1400CD140`: for each line, row +8 = game code (`0x1400D2E70`), row +56 = value text (max 64). It builds the route name `Game.%s` (max 21 chars) for each game code and registers it (`0x14011D350`). Use of the value text ❓ (DB name per game code, not traced). |

All other keys found by `cfgkeys()` (52 calls) are framework keys (`00-framework.md` §1.4, §5).

### 1.3 Message dispatch

1. `0x1400CD4C0` (OnMessage):
   - Protocol `Game.` → `0x1400D0820` (Game sub-switch: `ListWorlds` → `0x1400D1150`, other → `0x1400D0E80`; else error `{52, module 1?, line 143, "invalid command"}`).
   - Command `Update` (App update notice) → `0x1400CD810` (`OnUpdateApp`: needs `<Update><Object><AppId>`; logs `OnUpdateApp: missing body`, `missing <Update/> in body`, `missing <Object/> in body`; then app cache update `0x1400CC430` ❓).
   - Id 12 `Exec` → `0x1400C47B0`: parse inner message (`0x1400DFD90`); inner id 15 `GetUserIdByAcct` → `0x1400C5000`; inner id 16 `GetAcctByUserId` → `0x1400C4DF0`; other → error `{52, 1, 142, "invalid command"}`.
   - Id 17 `ListUsersAccounts` → `0x1400D0900`.
   - Else parse the request into the global record `0x140237BE0` (`0x1400DFF80`, command id at `0x140237BE8`). Parse fail → `ReplyError(.., "parsing game account message")`.
2. `0x1400CD760` (pre-load, Compute thread ❓): id 10 `CheckTokenSecret` → find the move record (`0x1400CC8E0`, key at `0x140237D00`) and send `Cache.GetValue` (`0x1400CE560`, key `%u.%s`); id 20 `Flush` → nothing; other ids → `0x1400C62B0` (proc `ListUserGames` for the user, to load the user's game accounts), the main switch runs when that reply comes.
3. `0x1400CD400` → main switch `0x1400C6E80(app, err, 0x140237BE0)`. A non-zero result gives `ReplyError(.., "bad server data")`.
4. Main switch `0x1400C6E80` (id at record +8): 1, 3 → `0x1400C70D0`; 2 → `0x1400C7540`; 4 → `0x1400C7AC0`; 5 → `0x1400C7E90`; 6 → `0x1400C7690`; 7 → `0x1400C8490`; 8 → `0x1400C8280`; 9 → `0x1400C80F0`; 11 → `0x1400C8D50`; 13 → `0x1400C8940`; 14 → `0x1400C8C40`; 18 → `0x1400C85B0` (thunk `0x1400FA4D0`); 19 → `0x1400C90C0`; 20 → `0x1400C7020`. Default: error `{52 ErrInvalidCommand ❓, module 3, line 703, "invalid command"}`.

Every id 1..20 has a path. Id 0 `Invalid` has none (default error).

### 1.4 Commands

Error records are `{code, module, line, text}` (`00-framework.md` §3). Code 3002 with module 4..6 is the service "not found" class (3802 = "not changeable" ❓). Request field names come from the strings in the handler; ❓ = not traced to the XML path.

| Id | Command | Handler | What it does |
| --- | --- | --- | --- |
| 1 / 3 | `ListAccounts` / `ListMyAccounts` | `0x1400C70D0` | Needs target code (`0x1400CBA40`, error "missing parameters - no target code", line 134) and UserId ("invalid userid"). Reads the cached user games (`0x1400CC8E0`), reply `<Reply>` with an `array` of `GameAccount` items (`type`, `Alias`). Game that does not use game accounts: error "target game does not use game account". |
| 2 | `ListGames` | `0x1400C7540` | Needs UserId. Sends `GP.ListGames` `<Request><UserId/>` (`0x1400E0550`: name `ListGames`, protocol `GP`) with txn type 2 (`0x1400CE440(.., 2)`) and `0x1400FA8A0` timeout 25000 ms. The service behind `GP` is not in this slice ❓. No UserId → "invalid userid". |
| 4 | `Login` | `0x1400C7AC0` | Finds the user's game account for the game code (`0x1400C6550`). None: create it with proc `AddUserGame` or `AddUserGameForNumericId` (`0x1400C5720`; numeric form when the game uses numeric ids ❓). Then proc `UpdateUserLastPresence` (`0x1400CBAC0`). Reply `<Reply><GameAccountId/>`. Errors: "missing parameters - no inputed gacct", "gacct creation failed", "unexpected kind of game account". |
| 5 | `Logout` | `0x1400C7E90` | Needs a target game ("no target game"). Proc `UpdateUserLastPresence` (call `0x1400C8052`). Reply `<Reply/>`. |
| 6 | `ListUserLastPresences` | `0x1400C7690` | Proc `ListUserLastPresences` (all games) or `GetUserLastPresence` (one game). Reply `array` of presence items. |
| 7 | `MoveStart` | `0x1400C8490` | Retry limit = `[AccountMove] RetryCount` in `0x1400CFC40` (§1.2). Find the game account ("game account not exists", 3002/1829), check that it is changeable ("game account not changeable", 3802/1834), then `0x1400CFC40`: make a move record, key `%s:%s` (`0x1400CC320`), store it in `Cache` (`Cache.GetValue` path `0x1400DEA20`) ❓, timeout 25000 ms. |
| 8 | `MoveEnd` | `0x1400C8280` | Needs the token secret ("missing token secret", line 1862/1872). Reads the move record from `Cache` (`0x1400DEA20`), then `0x1400C6090` sends `Token.Instant` `ValidateToken` (`0x1400E3C70`, `TokenSecret`, client string `;appid=%s;client=%s`, default address `127.0.0.1`). On success the txn steps (§1.5) run `ChangeUserIdByGameCode`. |
| 9 | `MoveCheck` | `0x1400C80F0` | Find the game account (3002/1903). Reply `<Reply><IsUserChangeable/>`. |
| 10 | `CheckTokenSecret` | pre-load `0x1400CD760` → `0x1400CE560` | Read the move record by key from `Cache` (`GetValue`), txn line 798. Reply ❓ (txn step not traced). |
| 11 | `ResetMoveCount` | `0x1400C8D50` | Checks app id / game code ("invalid appId or gameCode"), needs a target buffer, finds the game account; proc `EnableUserChangeable`. |
| 12 | `Exec` | `0x1400C47B0` | Wrapper: runs inner id 15 or 16. |
| 13 | `RemoveAccount` | `0x1400C8940` | Find the game account ("game account given does not exist."); proc `RemoveUserGameByGameAccountId`. |
| 14 | `RemoveAccounts` | `0x1400C8C40` | Proc `RemoveUserGameByUserId` (line 903). |
| 15 | `GetUserIdByAcct` | `0x1400C5000` | Needs `GameCode`, `GameAccountId` ("missing parameters", "invalid parameters"). Proc `GetUserIdByGameAccountId` sent with `SendToProviderTxn` (25000 ms). Also the resolver for `gacct` addresses. |
| 16 | `GetAcctByUserId` | `0x1400C4DF0` | Needs game code ("missing gamecode") and UserId ("invalid parameters(userId)"). Proc `GetUserGame`. |
| 17 | `ListUsersAccounts` | `0x1400D0900` (`GameTxn.cpp`) | Body with `UserId` list and `GameCode` or `AppId` ("has no body.", "has no gameCode or appId.", "no game code or target game does not use game account"). Reply `array`. |
| 18 | `PageAccountMoves` | `0x1400C85B0` | Checks app id / game code, `pageIndex` / `pageSize` ("invalid pageIndex or pageSize"); proc `PageGameAccountUserChanges`. |
| 19 | `UserIdChanged` | `0x1400C90C0` | Needs user id, old and new ("invalid userid or old or new"; "userid is not same the old"). Proc `ChangeUserId`. |
| 20 | `Flush` | `0x1400C7020` | Empty `<Reply/>` (line 5840 ❓ = txn record size, not a source line). |
| — | `Game.ListWorlds` | `0x1400D1150` | Sends proc `ListWorlds` (`0x1400DECE0`) with `SendToProviderTxn` (25000 ms). Reply to the caller. |

### 1.5 State, txns and timers

- User game cache: `0x1400CC8E0` (hash lookup, 288 bytes) holds the per-user game account list loaded by `ListUserGames` (§1.3 step 2). `0x1400C6550` finds one game account in it (by game code `0x1400D2E70` → `0x1400C61F0`).
- Txn step switch `0x1400C9340` (txn type at txn +8): 1 `0x1400CA550` (proc `UpdateUserLastPresence`), 2 `0x1400CA070`, 3 `0x1400C99C0`, 4 `0x1400CA2E0`, 5 `0x1400CB120`, 6 `0x1400CAE30` (move token record), 7 `0x1400CB710` (procs `RemoveUserGameByGameAccountId`, `ChangeUserIdByGameCode`), 8 `0x1400CB2E0`, 9 `0x1400C98C0`, 10 `0x1400C9550`, 11 `0x1400C9790`, 12 `0x1400CA860`, 13 `0x1400C9CF0`, 14 `0x1400CB620`; default error `{52, …, 769, "invalid command"}`. Types 2–5, 8–14 ❓ (not traced).
- Move token (`0x1400CAE30`, txn type 6): needs `TokenId` ("missing token id"); record +76 = due time = `timeGetTime() + 1000 × ExpirationSec`; `StsTimerSet(rec, due − now, 0)`. `0x1400C6700` builds the record from `AppId`, `TokenSecret`, `TokenId`, `TokenDuration` and arms the same timer. Timer action on expiry ❓ (OnTimer not traced; probably delete the record).
- Grid object timeout 86400000 ms (§1.1 step 8) ❓.

### 1.6 Calls out

| Target | Command | Where | Why |
| --- | --- | --- | --- |
| DbSrv `Db.GameDb` | 14 procs (`03-procs.tsv`) | handlers | data |
| `Cache` | `GetValue` (key `%u.%s`) | `0x1400DEA20` from `0x1400CE560`, `0x1400CFC40`, `0x1400C8280` | move records ❓ |
| `Token.Instant` | `ValidateToken` (`TokenSecret`) | `0x1400E3C70` from `0x1400C6090` | MoveEnd |
| `GP` (service name ❓) | `ListGames` (`UserId`) | `0x1400E0550` from `0x1400C7540` | ListGames (id 2) |
| GameAccount (other instance) | `GetUserIdByAcct`, `ListGameAccounts` | `0x1400E0370`, `0x1400E0470` (`FwdGameAcctTxn.cpp`) | forward ❓ |
| `Name` | `GetIdByUserName`, `GetIdByLoginName` | resolvers §1.1 | route address resolve |

### 1.7 Worked example (Login, id 4) ❓ field paths

Request `GameAccount/Login` with UserId `U` and game code `G` (example `G = 1`). Step 1: pre-load sends `ListUserGames(U)` to `Db.GameDb`. Step 2: `0x1400C7AC0` does not find `G` in the list, so it sends `AddUserGame(U, G)`. Step 3: proc `UpdateUserLastPresence(U, G, …)`. Step 4: reply `<Reply><GameAccountId>N</GameAccountId></Reply>` where `N` is the new id from the proc.

## 2. SlotSrv

### 2.1 Role and init (`0x1400C7A40`)

SlotSrv keeps product slots of a user (create, get, list, expire, delete, client data) and tag policies. DB `SlotDb` / `v2016041101` (all 24 call sites). `procs.tsv` names the DB `PlatformSlotDb` (difference, §7).

Init order (`0x1400C7A40`):
1. `StsSetServerType(1022, 0)`; `0x140124A90("Slot", 0, &v)` ❓.
2. `StsConfigMonitorAdd(L"Auth.ini", &off_1401EC0E0)` (`0x1400C80B0`: `[UserName] UniqueProperty`, default 1, global `dword_1401EC0E8`). Log `UserName is %sunique (by Auth.ini), %s forward registration`. If unique: resolver `Slot` → `$Name.GetIdByUserName` (`0x140124930`, error 3002, module 4), log `forward registration with (:%s)`.
3. `0x1400D4B40`: txn class `loadTxn` for `Slot` (`0x140102D90`).
4. `0x140102660(v, 0)` (grid start ❓).
5. `StsConfigMonitorAdd(L"Slot.ini", app+8)` → `0x1400C7B70` (§2.2).
6. `0x1400CD760` ❓ (17 bytes), `0x1400C6C30`: admin pages `/slot/`, `/slot/create_slots`, `/slot/expire_slot`, `/slot/delete_slot`, `/slot/get_slot`, `/slot/list_slots`, `/slot/set_data` (title `Slot Management`).
7. `StsAppCommonStart(0)`.

### 2.2 Config keys (`Slot.ini`, `0x1400C7B70`)

| Section | Key | Default | Global | Use |
| --- | --- | --- | --- | --- |
| `Limits` | `MaxCreateSlots` | 0 | `dword_14023BC70` | read by `0x1400C7420` (getter); limit check ❓ |
| `GridTimeout` | `Timeout` | 3600 s | `dword_14023BC74` | `0x140102850("Slot", 1000 × value)` grid object timeout |
| `RetryTag` | `use` | 0 | `dword_14023BC78` | ≠ 0: `StsTimerSet(obj+16, 60000, 0)` (first retry after 60 s); 0: `StsTimerDestroy` |
| `RetryTag` | `Interval` | 60 s | `dword_14023BC7C`, `dword_14023BC80` = ×1000 | retry timer period (`0x1400C8260` also reads it) |
| `RetryTag` | `MaxRetryTime` | 36000 s | `dword_14023BC84` | getter `0x1400C7440`; retry age limit ❓ |
| section `config` (`0x1400EDEE0`) | slot type list (4 int values per line, `IniValueAsInt` 0..3) | — | `unk_1401EC0A8` | per slot type a `CSlotTypeConfig` (0x70, `0x1400C6F30`); default data file `Data/SlotSrv/%S/%d/default.txt` (`%S` from `0x1400EBC20(1)`, `%d` = type). Read fail (`0x1400C7460`): log `file open (%S) failed (%u)`, `default slot data read failed. its empty.`, `ExeFatal` |

### 2.3 Dispatch

Service socket vtable `0x140171840`: +0x10 `0x1400C7920` → `0x1400D9770` (user path; needs `UserId`, `User.cpp` lines 2038–2054), +0x18 `0x1400C81F0` (parse into global record `0x14023BCA0` with `0x1400EBB40`, map `0x14023CEF0`), +0x20 `0x1400C82C0` (`to` header: user route; bad header → `ReplyError(.., "SlotMain: invalid to header")`; else load the user with proc `ListSlots` `0x1400D9940`), +0x28 `0x1400C80F0` (make `CSlotTxn` 0x38 bytes `0x1400D1AB0`, run global switch).

Global switch `0x1400D4A70` (no user): 2 `DeleteSlot` → `0x1400D1EE0`; 7 `ExpireSlot` → `0x1400D2080` (proc `UpdateSlotExpire`); 8 `PageTagPolicies` → `0x1400D2500` (proc `ListTagPolicies`, `PageIndex`, `PageSize`); 9 `AddTagPolicy` → `0x1400D1BB0` (`TagPolicyId`, `TagPolicyName`, `TagKey`, `TagPolicyData`); 10 `PageSlots` → `0x1400D2280` (proc name chosen by filters: `PageSlots`, `PageSlotsByTagStatus`, `PageSlotsByTagPolicyIdList`, `PageSlotsByTagPolicyIdListAndTagStatus`; request +716, +888, +896 select). Default `{52, 4, 1117}`.

User switch `0x1400D88A0` (user object loaded): 1 `CreateSlots` `0x1400D6020`; 2 `DeleteSlot` `0x1400D6330`; 3 `GetSlot` `0x1400D6750`; 4 `ListSlots` `0x1400D6930`; 5 `ResetData` `0x1400D6B30`; 6 `SetData` and 12 `SetClientData` `0x1400D6D10`; 7 `ExpireSlot` `0x1400D6510`; 11 `Flush` `0x1400D6700` (empty reply). Default `{52, 2, 462}`. Each handler queues a request context through `0x1400CD780(key, type)`: type 1 Create `0x1400C8B30`, 2 List `0x1400C8FE0`, 3 Delete `0x1400C8CC0`, 4 Get `0x1400C8E50`, 5 SetData `0x1400C9300`, 6 ResetData `0x1400C9170` (queues `unk_1401EC128`..`unk_1401EC240`).

Context classes (vtable, run slot +0x18): `CCreateSlotsReqContext` `0x140171ED0` → `0x1400CBB30` (procs `GetTagPolicyData` `0x1400CA4A0`, `CreateSlots` `0x1400CA740`/`0x1400CA590`, `UpdateSlotInstantExpire`, `GetSlot`; fields `AppGroupId`, `TagPolicyData`, `TagLimit`, `SlotId`); `CListSlotsReqContext` `0x140171F28` → `0x1400CCB90` (`CreateSlots` `0x1400CA210`); `CDeleteSlotReqContext` `0x140171F80` → `0x1400CC730` (`DeleteSlot` `0x1400CA8E0`); `CGetSlotReqContext` `0x140171FD8` → `0x1400CCA50` (`GetSlot` `0x1400CA3B0`, `ListSlots` `0x1400D09B0`); `CSetDataReqContext` `0x140172030` → `0x1400CD3D0` (`UpdateSlotData` `0x1400CBA40`); `CResetDataReqContext` `0x140172088` → `0x1400CD060` (`UpdateSlotData` `0x1400CB950`).

Tag lock flow (`CSlotExtension` vtable `0x1401723C8`): `0x1400D4710` reads `ActivateKey`, `DeactivateKey`, `RetryFlag`, `TagStatus` and calls procs `UpdateSlotActivateLock` `0x1400D2830`, `UpdateSlotActivateUnlock` `0x1400D29A0`, `UpdateSlotDeactivateLock` `0x1400D2CF0`, `UpdateSlotDeactivateUnlock` `0x1400D2EE0`, `UpdateSlotRetry` `0x1400D3180`; `0x1400D43F0` → `UpdateSlotDisable` `0x1400D3060`; `CSlotTxn` slot `0x1400D4570` → `0x1400D32A0` (reply); `0x1400D1D50` `UpdateSlotDeactivateLock` (second site). Outgoing `ActivateSlot` (`0x1400E7B40`) and `DeactivateSlot` (`0x1400E7C30`) messages are built for map `0x14023CE90` (goods protocol ❓ target).

### 2.4 Request fields and CreateSlots steps

User-path request parse `0x1400EB640` (global path ids 7..10 use `0x1400EB3A0`; unknown name → `{10, 177, "unknown command"}`, `0x1400EBB40`). Request record offsets:

| Offset | Field | Type |
| --- | --- | --- |
| +716 | `UserId` | GUID (16 bytes) |
| +732 | `SlotId` | GUID ❓ (string parsed at the line before) |
| +748 | `AppGroupId` | int (negative → 0) |
| +752 | `Count` | int (negative → 0) |
| +760 | `TransactionKey` | string |
| +768 | `SlotData` | string |
| +776 | `SlotType` | name → id by map `0x14023CF08`: `Invalid` 0, `char` 1, `activity` 2, `tag` 3, `customize` 4; unknown or empty → 0 |
| +784 | `Tag` | int attribute of `Tag` element |
| +788 | `RequestAppId` | GUID |
| +804..+896 | `ExpireAppId`, `ExpireKey`, `TagProtocol`, `TagKey`, `Expired`, `IsInstantExpire` (+888), `PolicyIdList` (+896), `TagStatus` | — |
| +880 | `PageIndex` | int |
| +900 | `Extension` | — |

`CreateSlots` (id 1, `0x1400D6020`, `SlotUser.cpp` ❓ file):
1. Check `SlotType` ≠ 0, `Count` ≥ 1, `TransactionKey` not empty and ≤ 256 chars. Fail → error `{10, 2, line 525}` (type, count or key bad).
2. If `SlotType` is `activity` (2) or `customize` (4): `SlotData` must be present. Missing → `{10, 2, 525}`; empty string → `{10, 2, 530}`.
3. `SlotData` length > 12288 (0x3000) → error 3411 (line 534).
4. `MaxCreateSlots` (`0x1400C7420`) < `Count` → error 3415 (line 540). Note: with the default `MaxCreateSlots` = 0, every `CreateSlots` fails with 3415 (`quirk_slot_max_create_zero`).
5. Get a request context (`0x1400CD780(user key, 1)`); none → error `{4, 2, 549}`.
6. Fill it (`0x1400D0AA0`: UserId (+272 of the user object), SlotType, SlotData, TransactionKey, Tag, RequestAppId, has-AppGroupId flag, AppGroupId or −1, Count, UserId) and run it (context slot +0x18 → `0x1400CBB30`: procs `GetTagPolicyData`, `CreateSlots`, `UpdateSlotInstantExpire`, `GetSlot`). If it is not done at once, it is queued (`0x1400CD910(1)`, `0x1400CD890`).

### 2.5 Tag lock state machine and timers

`CSlotExtension` (vtable `0x1401723C8`, object 0x38 bytes, ctor `0x1400D1AB0`) is the tag worker. DB result handler `0x1400D4710` (vtable slot 3):
- reply has `ActivateKey` → proc `UpdateSlotActivateUnlock` (`0x1400D29A0`).
- reply has `DeactivateKey` → proc `UpdateSlotDeactivateUnlock` (`0x1400D2EE0`).
- result `Set/Row`: if `RetryFlag` ≠ 0 → proc `UpdateSlotRetry` (`0x1400D3180`, parameter `MaxRetryTime` from `0x1400C7440`). Then by `TagStatus`: 1 → `UpdateSlotActivateLock` (`0x1400D2830`); 2 → `UpdateSlotDeactivateLock` (`0x1400D2CF0`); 3 → send `ActivateSlot` (`0x1400D2640`); 4 → send `DeactivateSlot` (`0x1400D2B20`); other → nothing.
- `0x1400D4A50` (start) → proc `UpdateSlotRetry`.

Timers:

| Timer | Period | OnTimer | Use |
| --- | --- | --- | --- |
| app+24 (`CSlotConfig` +16), vtable `0x140171888` | first 60000 ms, then `RetryTag Interval` × 1000 (return value of OnTimer) | `0x1400C8260` | only when `RetryTag use` ≠ 0. Each tick: make the `CSlotExtension` once (kept at timer +16) and `StsTimerSet(ext, 1)` → retry pass (`UpdateSlotRetry`). `use` = 0 → `StsTimerDestroy`. |
| grid object | `GridTimeout Timeout` × 1000 | framework (`0x140102850`) | user object unload |

### 2.6 Calls out

| Target | Command | Builder | When |
| --- | --- | --- | --- |
| `GameWarehouse.<name>` (map `0x14023CE90`: 1 `NotifyGoodsInfoChanged`, 2 `NotifyGoodsInventoryChanged`, 3 `DeliverIngameCoupon`, 4 `ActivateSlot`, 5 `DeactivateSlot`, 6 `GetBalance`, 7 `IngameGoodsChanged`, 8 `IngameItemChanged`, 9 `IngameCategoryChanged`, 10 `Test`) | `ActivateSlot` | `0x1400E7B40` (`Request/UserId`, `SlotId`, `Tag/TagKey`) | `TagStatus` 3 row; `<name>` from `TagPolicyData` (`0x1400EBC60(1, …)`); `SendToProviderTxn(…, 25000 ms)` |
| same | `DeactivateSlot` | `0x1400E7C30` (`UserId`, `SlotId`) | `TagStatus` 4 row; 25000 ms |
| `Name` | `GetIdByUserName` | resolver | user name routes (§2.1) |

Only ids 4 and 5 of the `GameWarehouse` map are sent. The other ids are not used by SlotSrv.

### 2.7 Worked example: `CreateSlots`

Request `Slot/CreateSlots` to user `U`: `SlotType=char` (1), `Count=2`, `TransactionKey="tk-1"`, no `SlotData`, `AppGroupId=0`. `Slot.ini [Limits] MaxCreateSlots=10`. Steps: type 1 ≠ 0, count 2 ≥ 1, key length 4 → pass; type 1 needs no data; 10 ≥ 2 → pass. Context type 1 runs `GetTagPolicyData` ❓ (only for tag type), then `CreateSlots(U, 1, …, Count 2)` in `SlotDb`, then `GetSlot` per new slot; the reply lists the 2 slots. With `MaxCreateSlots=1` the same request gets error 3415.

<!-- SLOT-END -->

## 3. SpawnSrv

### 3.1 Role and init (`0x1400DCA40`)

SpawnSrv is the process supervisor of one machine. It starts the StServers programs that `Spawn.ini` lists, restarts them when they stop, accepts the `Spawn` protocol from each child (the child side is the framework `SpawnSrv` map, `00-framework.md`), and brokers incoming connections on the managed ports to the correct child listener. It also has an HTTP admin site and registers with `SvcMonSrv` and `DeploySrv`. It has no DB and no proc (recipe: 0 procs).

Init order (`0x1400DCA40`, `SpnMain.cpp`):
1. If the process was started by a SpawnSrv (`0x1401177C0` ≠ 0): `ExeFatal("SpnMain: SpawnSrv recursively spawned", line 180)`.
2. Thread pool name `SpawnApps` (`0x1400F79C0`).
3. `StsSetServerType(100, 1)` (`0x64` = `SpawnSrv` in map `0x14021C730`).
4. `0x1400DB210(dword_14021AD80)`; `StsConfigMonitorAdd(L"StServer.ini", &off_1401CA378)`. The `[Systems]` line `SpawnSrv=<port>` must exist exactly once (`0x1400DCF70`: else log and `ExeFatal("SpnMain: not exactly one system defined with %S=<port>, %u", line 95)`). Datacenter change log `SpnMain: datacenter changed` (`0x1400DCBA0`).
5. `0x1400D33F0`: outgoing link `SvcMonSrv` (`0x1401154A0`, server type 413, timeout 100000 ms), HTTP page `/modules` (`0x1400D3350`: `Modules/Module/name`, link `/sts/counters`).
6. `0x1400D7BF0`: outgoing link `DeploySrv` (type 410, timeout 100000 ms).
7. `0x14010AD00` (framework HTTP client timer 1000 ms), `0x1400D59C0`, `0x1400D7890` (HTTP pages, §3.5), `0x14010D3C0(1, 0)`, `0x1400D6F50` (page `/disconnect`).
8. Config: `0x1400DA100` adds monitors `Spawn.ini` (notify `0x1400D99C0`, §3.2) and `Spawn.deploy` (notify `0x1400D98A0`), then starts the startup timer `StsTimerSet(&off_1401CA298, StartupDelay)` → `0x1400DA940`.

### 3.2 Config keys

`Spawn.ini` (`0x1400D99C0`). The INI file name in the log texts is `spawn.ini`.

| Section | Key | Default | Range | Global | Use |
| --- | --- | --- | --- | --- | --- |
| `Limits` | `ProcessRestart` | 30 s | 5–1800 s (else default) | `dword_14021ABFC` (ms) | delay before restart of a stopped child (timer `0x1400D8880` → `0x1400DA290`) |
| `Limits` | `ProcessStopRetry` | 180 s | 5–1800 s | `dword_14021AC00` (ms) | retry delay of stop/terminate |
| `Limits` | `StartupDelay` | 10 s | 5–1800 s | `dword_14021AC04` (ms) | first start timer (`0x1400DA100`) |
| `Limits` | `MaxConnectMsgSize` | 98304 | 98304–10485760 | `dword_14021AC08` | max size of a forwarded connect message; out of range: log `Too big (or small) connect message size limit(%u), applied to default(%u)` |
| `Limits` | `ShutdownTimeout` | 300 s | clamp in `0x140117630` | framework | app shutdown timeout |
| `Run` | `<program>` = `<type>, a, b, c, d, e` | — | — | list `dword_1401CA328` | one `SpnRunConfig` (0x50 bytes) per line, key = program name (+8) |
| `Deploy` | `Run/*` | — | — | — | deploy file set (`0x1400DA100`) |

Range check form: `(value_ms − 5000) > 1795000` → default. The three seconds values are also copied to `dword_1401CA248`, `dword_1401CA260`, `dword_1401CA278` (seconds, for the HTTP pages).

`[Run]` line fields (`0x1400D99C0`): field 0 = run type: `None` → 0, `Serial` → 1, `Parallel` → 2 (+56) ❓ (order of the nested compares); other text → log `SpnExec: spawn.ini (line %u), invalid run type, %S` and the line is dropped. Fields 1..5 (`0x1400F9A20`) go to +60, +64, +68, +72, +76. Start delay pair (+64, +68): if max > 3600 or min > max → log `invalid start delay` and set 0..300. Second pair (+72, +76): same check → set 600..900. After the loop `0x1400D8210` links each program to its run config by name; a program without a line gets the run config `Default`.

`Spawn.deploy` (`0x1400D98A0`): `[Deploy] BuildId`. `Spawn.deploy` also feeds the deploy state (`BuildId`, `DeployState`, `DeployStateChanged`, `SpawnDeployId`, `LastDeployment`, `ProgramDeployTime`, `LastDeployTime`, `TriggerTime`: `0x1400DC020`, `0x1400DBF50`). Deploy folders `Deploy`, `Web`, `Temp`, `Old` (`0x1400D8790`, `0x1400DC340`).

### 3.3 Process control (`SpnExec`)

| Step | Function | Behaviour |
| --- | --- | --- |
| Launch | `0x1400D8880` | Command line `Run\%04u\%S.exe /AppIndex %u /SpawnSrv %u` + ` /SpawnPorts …` + ` /Email %S`; work folder `Run\%04u` (`%04u` = deploy index). Then `StsTimerSet(&off_1401CA2A8, ProcessRestart)`. |
| Folder prepare | `0x1400D8570`, `0x1400D8B50`, `0x1400D8D00`, `0x1400D8EF0`, `0x1400D9020`, `0x1400D9270` | make `Run\Temp`, copy files (`*`), delete old trees, set file times. Errors are logged (`SpnExec: create dir tree … failed`, `file copy … failed`, `badly placed directory`). |
| Watch / stop | `0x1400DA290` (OnTimer of `off_1401CA2A8`) | state machine per app; `TerminateProcess` (log `SpnExec: TerminateProcess(%u aka %s%.0u) failed`), pid file `Run\%04u/%S%.0u.pid`. |
| Wait | `0x1400DA970`, `0x1400D95B0` | wait for exit (`SpnExec: wait … failed`); `OpenProcess` log; log `launching new instance when s_shutdownDetached(%d)`. |
| Match a new connection | `0x1400DAE80` | find the app by deploy and process id. Fail logs: `could not find appropriate app with [%u/%u]`, `unknown deployment (%u)`, `mismatched processId (%u), expected %u`. |
| Product name | `0x1400DB270` | unknown name → log `SpnExec: unknown product name, %S`; then `StsTimerSet(&off_1401CA2A8, 0)` |

### 3.4 Commands (`Spawn` protocol, map `0x14021C600`, server side)

Message parse `0x1400F3860` (name → id by `NameMapFind` on `0x14021C600`). Request fields: `SockInfo`, `ConnectData`, `Name` (+56, len +68), `Url` (+80, len +92), `Port` (+104), `Tls` (+108), `ConnType` (+112), `Build` (+116), `Program` (+120), `VirtualHost` (+128, len +140), `Location`. Message build for sends: `0x1400F3590`.

Dispatch `0x1400D5060` (per child connection; ignored when conn +104 ≠ 0 ❓ meaning). Parse fail → error reply `SpnApp: msg parse error`; unknown id → `SpnApp: invalid command`.

| Id | Name | Handler | Behaviour |
| --- | --- | --- | --- |
| 1 | `Register` | `0x1400D42F0` | if `Port` ≠ 0: conn +72 = port, reply OK (code line 496); if `Port` = 0: error reply `SpnApp: app register, invalid contact port '0'` (line 494) |
| 2 | `RegisterPort` | `0x1400D42F0` | same as 1 |
| 3 | `RegisterModule` | `0x1400D41F0` | append a module row (320 bytes: name ≤ 64 at +0, url ≤ 256 at +64) to conn +80 (count +92); reply OK |
| 4 | `Listen` | `0x1400D40B0` | for each space-separated host in field +128 and once with no host: `0x1400D3B50` adds a listener route (log `Added Listener as route, from(%s), port(%u), connType(%u), buildId(%u)`); reply OK; then `0x1400D6FE0` retries waiting connects |
| 5 | `Shutdown` | sent | SpawnSrv sends it to an unknown child (§3.4 connect) with reply object `0x1400D35D0`; timeout log `Shutdown request timed out: %s` (`0x1400D49E0`) |
| 6 | `Connect` | sent ❓ | connect broker forward (`0x1400D5280`) |
| 7 | `Found` | sent ❓ | — |
| 8 | `Lost` | sent | `0x1400D4D20`: on child disconnect, sent to every other connected child (list `off_1401CA058`) |
| 9 | `DeployInfo` | sent ❓ | — |

Child connect `0x1400D4A70`: add to list `dword_1401CA048`. If `0x1400DAE80` finds the app: conn +104 = 1, log `System;  new service connected, %s%d, depoly(%u), epoch(%u)`. Else send `Shutdown` (5) to the child. Child disconnect `0x1400D4D20`: log `System;  service disconnected, …`, send `Lost` (8) to the others, remove the routes (`0x1400D5A40`).

Connect broker (managed ports, `ManagedApps`/`ManagedPorts` `0x1400DCE90`): `0x1400D6830` (from `0x1400D6C80`) finds the listener with `0x1400D54A0` (match port, connType, buildId, programId, host; log `SpnAppFindListener, …: from(%s)--> found, dest: (%u,%u,%u,%u)` or `NOT found`). Found → `0x1400D5280` forwards the first message (size > `MaxConnectMsgSize` → log `Incomming too big(%u bytes) request msg from(%s), dropped`). Not found → keep the connection and `StsTimerSet(&off_1401CA178, 20000)`. OnTimer `0x1400D6D20`: log `SpnClient: no registered service, connType %d, program %d, build %d` and send HTTP `GET /disconnect`.

### 3.5 HTTP pages and calls out

Pages (`0x1400D7890`): `/spawned/:0`, `/apps/:0/stop`, `/apps-state`, `/apps-config`, `/apps-listen` (title `Applications`, XSL `Web/SpawnSrv/apps-*.xsl` `0x1400D70E0`), `/heartbeat`, `/modules`, `/disconnect`; proxy `/spawned/%S%.0u.%u.%u` to the child module url (`0x1400D6340`, `0x1400D74C0`, `_rights=`). Unknown host → `Page not found` (`0x1400D7110`).

Calls out: `SvcMonSrv` link (type 413; map `0x14021C638` = `SvcMon` protocol, 15 ids `Register` … `GetStServerStatus`, sent only; `Register` built in `0x1400F3D00`); `DeploySrv` link (type 410); `AggroSrv` request (`0x1400D31D0`, fail log `SpnAggro: Aggro request failed`).

### 3.6 Timers

| Timer object | Period | OnTimer | Use |
| --- | --- | --- | --- |
| `off_1401CA298` | `StartupDelay` | `0x1400DA940` | first start; then `StsTimerSet(off_1401CA2A8, 0)` |
| `off_1401CA2A8` | `ProcessRestart` / 0 | `0x1400DA290` | process state machine (set from `0x1400D8880`, `0x1400D98A0`, `0x1400D99C0`, `0x1400DA1D0`, `0x1400DAE80`, `0x1400DB270`, `0x1400DB650`) |
| `off_1401CA178` | 20000 ms | `0x1400D6D20` | wait for a listener for a brokered connection |

## 4. WorldSrv

### 4.1 Role and init (`0x1400C5260`)

WorldSrv keeps the world (game server group) list of one game code and the user counts of each world. Callers: gates and game services through the route bus (`World/*`), the admin HTTP pages, and GameAccountSrv ❓ (GameAccountSrv reads `ListWorlds` from the DB itself, §1.4).

**World list source (README ❓ closed):** the list comes from the DB. `0x1400C7C60` sends proc `GameDb.ListWorlds` (`v2016120901`, 25000 ms) at start (`0x1400C86A0`) and on every refresh timer. Each result row (`0x1400C76A0`: `WorldId`, `WorldCode`, `UserCounts` = `PlayingUsers`, `WaitingUsers`, `MaxUsers`) becomes a 0x80-byte world object (`0x1400C58E0`, hash `dword_1401D60D0`, key = 16-byte world code). `World.ini` gives only the game code and the cycle times.

Init order (`0x1400C5260`):
1. `StsSetServerType(1033, 0)`.
2. `StsConfigMonitorAdd(L"Game.ini", app+16)` (notify `0x1400C54E0`: `[EventLog] UseTimeout`, default 0 ❓ use).
3. `StsConfigMonitorAdd(L"World.ini", &off_1401D6090)` (notify slot 0 `0x14015E4A8` → `0x1400C5340`, §4.2).
4. `0x1400D99B0(byte_140225748, byte_140225738)` (game code copy ❓), `0x1401128C0` and `0x140112760(.., "GameDb.GetIdByWorldKey", 3800, 4, "v2016120901")`: route resolver for world keys (error 3800, module 4).
5. `0x1400EFB70(app+16, 0)` (grid start ❓).
6. `0x1400C86A0`: register grid call `WdGetWorld` → `0x1400C7850`, txn class `CWdGridTxn` (`off_1401D6108`), `StsTimerSet(&off_1401D6110, refresh, 0)`, then load the list once (`0x1400C7C60`).
7. `0x1400C4D60`: admin HTTP pages (`StsHttpAddCommands`): `/world/`, `/world/add_world`, `/world/update_world_profile`, `/world/update_world_property`, `/world/remove_world`, `/world/list_worlds` (titles `World Management`, `Maintenance Commands`).
8. `StsAppCommonStart(0)`.

### 4.2 Config keys

| File | Section | Key | Value | Use |
| --- | --- | --- | --- | --- |
| `World.ini` | `UpdateCycle` | key = service instance name (`0x1400FCC20` returns `word_140226BB0`) | value 0 = game code (string, max 16, `0x1400C9A90` → `byte_140225738`) | game code of this WorldSrv |
| | | | value 1 = s (0 → 300) | `dword_1401D60A0` = ×1000: `ListWorlds` refresh period (timer `off_1401D6110`, OnTimer `0x1400C79C0` returns this value) |
| | | | value 2 = s (0 → 300) | `dword_1401D60A4` = ×1000: read at `0x1400C5FC9`, `0x1400C5FF6` ❓ (use not decoded) |
| | | | value 3 = s (0 → 300) | `dword_1401D60A8` = ×1000: per-world timer (`0x1400C58E0`), runs `0x1400C8320` (sends a request with `SendToProviderTxn`, 25000 ms, target ❓; user count refresh ❓) |
| `Game.ini` | `EventLog` | `UseTimeout` | 0 | ❓ |

Fatal checks in `0x1400C5340` (`ExeFatal`, `WdMain.cpp`): no `UpdateCycle` line for the instance name → `WdMain: configuration %S is not exist` (line 379); empty game code → `WdMain: configuration %S %s, line %u` with `missing game code` (line 393). `quirk_world_cycle_zero_300`: a cycle value of 0 (or a missing value) gives 300 s, not "off".

### 4.3 Commands (switch `0x1400C8700`)

Map `0x140226620` (6 ids). Default: `ReplyError` with `{51, 2, 1542}` and text `unknown auth command %s` (copy/paste text of AuthSrv; `quirk_world_unknown_cmd_text`).

| Id | Command | Handler | What it does |
| --- | --- | --- | --- |
| 1 | `AddWorld` | `0x1400C64B0` | Proc `AddWorld` (params from request: world code, profile, property ❓ via `0x1400D5B30`, `0x140112400`, `0x1400D5B60`), `SendToProviderTxn` 25000 ms (line 1308). Reply after the DB reply ❓. |
| 2 | `ListWorlds` | `0x1400C6870` | From the cache only (no DB): reply `<Reply>` with `array` of `World` items (`0x1400C61B0`, fields `type`, `UserCounts`, `Property`). |
| 3 | `RemoveWorld` | `0x1400C69F0` | Proc `RemoveWorld` (world id `0x1400D5B10`), line 1358. |
| 4 | `UpdateWorldProfile` | `0x1400C6AF0` | Proc `UpdateWorldProfile` (id + profile), line 1385. |
| 5 | `UpdateWorldProperty` | `0x1400C6C10` | Proc `UpdateWorldProperty` (id + property), line 1411. |
| 6 | `GetWorld` | `0x1400C6630` | Find the world in the cache (`0x1400C6D30`, key = request +8). Found: reply with `Property`, `UserCounts` (`0x1400C6280`). Not found: proc `GetWorld`, 25000 ms. |

Id 0: no map entry. All ids 1..6 have a handler. The map `0x1402265E0` (`LogoutUser`, `GetAccountId`, `MigrateAccountPrepare`, `MigrateAccountCommit`, `GetStatus`, `ListChars`, `ListWorlds`) is the client map of the game server `Game` protocol ❓ (no handler in WorldSrv).

### 4.4 State, timers, calls out

- World cache: hash `dword_1401D60D0`, 0x80-byte objects (§4.1). Each object has its own timer (value 3).
- Timers: list refresh `off_1401D6110` (value 1, OnTimer `0x1400C79C0` → `0x1400C7C60`); per-world `0x1400C58E0` (value 3); `0x1400C5FC9` (value 2) ❓.
- Calls out: `Db.GameDb` (6 procs, `03-procs.tsv`); grid `WdGetWorld` to other WorldSrv ❓; per-world request `0x1400C8320` (target ❓).

### 4.5 Worked example (ListWorlds, id 2)

`World.ini`: `[UpdateCycle]` `<name>=1,60,0,30` gives game code `1`, list refresh 60000 ms, value 2 = 300000 ms (0 → 300), per-world 30000 ms. At start `ListWorlds` loads two rows (WorldId 101 and 102). Request `World/ListWorlds` → `0x1400C6870` replies `<Reply><World .../><World .../></Reply>` with the cached `UserCounts` and `Property` of both worlds; it does not call the DB.

## 5. Quirks

| Name | Address | What the code does | Value it gets | Who can reach it |
| --- | --- | --- | --- | --- |
| `quirk_slot_max_create_zero` | SlotSrv `0x1400D6020` (getter `0x1400C7420`) | `[Limits] MaxCreateSlots` default is 0. The check is `MaxCreateSlots < Count` → error. | With no `MaxCreateSlots` line, every `CreateSlots` (Count ≥ 1) fails with error 3415. | Any `Slot.ini` without `[Limits] MaxCreateSlots`. |
| `quirk_world_cycle_zero_300` | WorldSrv `0x1400C5340` | Each `UpdateCycle` value 1..3 that is 0 or missing is set to 300 s. | 300000 ms; a cycle cannot be turned off. | `World.ini` `[UpdateCycle]` lines with 0 or fewer than 4 values. |
| `quirk_world_unknown_cmd_text` | WorldSrv `0x1400C8700` | Default branch replies `{51, 2, 1542}` with text `unknown auth command %s`. | Error text names "auth", not "world". | Any `World/*` message with an id that has no case (id 0). |

## 6. Framework gaps

Facts that `00-framework.md` does not give (found in these binaries):
- Route resolver add: GameAccountSrv `0x14011CC10`, SlotSrv `0x140124930`, WorldSrv `0x140112760`. Arguments: (prefix, target `$Proto.Command` or `Db.Proc`, not-found error code, module, DB schema ❓). WorldSrv uses it with a DB proc target (`GameDb.GetIdByWorldKey`, `v2016120901`, error 3800).
- Grid object timeout setter: GameAccountSrv `0x1400FA390(name, ms)`, SlotSrv `0x140102850(name, ms)`. SlotSrv sets it from `Slot.ini [GridTimeout] Timeout`; GameAccountSrv uses a fixed 86400000 ms.
- OnTimer return value = the next period in ms (SlotSrv `0x1400C8260` returns `RetryTag Interval` × 1000; WorldSrv `0x1400C79C0` returns the refresh period).
- `StsSetServerType` values: GameAccount 1030, Slot 1022, World 1033, Spawn 100 (with second argument 1 for Spawn only).
- GameAccountSrv `0x1400E0550` message builder `sub_1400E9060(msg, name, protocol, 0)` sets the target protocol by text (here `GP`).
- Spawn: a program started by SpawnSrv that is SpawnSrv itself is fatal (`0x1401177C0` check in `0x1400DCA40`).

## 7. Differences from existing specs

- `Docs/Platform/StServers/procs.tsv` gives the DB names `PlatformGameDb` and `PlatformSlotDb`. The binaries use `SlotDb` (`v2016041101`) for SlotSrv and `GameDb` (`v2016120901`) for GameAccountSrv and WorldSrv (string at each call site, `03-procs.tsv`).
- `Docs/Platform/StServers/README.md` ❓ "where does the world list come from": closed. WorldSrv loads it with proc `GameDb.ListWorlds` (`0x1400C7C60`) at start and on each refresh. No INI file holds the world list (§4.1).
- WorldSrv has no `StServer.ini` line in the inventory. Its game code comes from `World.ini [UpdateCycle] <instance name>` (§4.2).
- GameAccountSrv also answers `Game.ListWorlds` (`0x1400D1150`) from DB `GameDb`, so two services read the same world table.
- SpawnSrv has 0 procs and no DB (matches `procs.tsv`, which has no SpawnSrv rows).


