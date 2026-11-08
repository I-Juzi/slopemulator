# 08 — DB bindings, build 0.0.307

## State and authority

This part is **partial**. Step 8 checks the login and restoration caller paths.
The remaining caller maps stay open at the eight-turn research limit.
The binary controls the call contract. The supplied schema is a recreated DB.
No SQL connection was made. No original SQL implementation is claimed.

| Check | Result |
| --- | --- |
| Call templates / distinct wrappers | 88 / 87 |
| Present / missing recreated procedures | 86 / 2 |
| Parameter count, SQL type and direction capability | 86 agree; two schema definitions are missing |
| Overall scoped comparison | 82 agree; four result-capacity conflicts; two missing |
| Bound result sets / columns | 17 / 94 |
| Full adapter equivalence | ❓ Not verified |

`db-binding.json` gives each wrapper, SQL literal address, direct call sites,
parameter bind sites, numeric SQL/C types, directions, capacities, result ordinals,
schema parameters, declared error constants and verification limits.
`agree` applies only to the stated comparison scope. It does not certify NULL
handling, ODBC conversion, nested SQL effects or caller-to-wire error maps.
Every entry keeps these open checks. No unknown check becomes success.

## Shared binding rules

| Source | Checked rule |
| --- | --- |
| `0x1400E3AF0` | Argument 2 is the ordinal. Argument 3 is direction. Arguments 7/8 are SQL/C types. Arguments 5/6 are buffer bytes / column size. Scale is zero. ODBC status below 2 succeeds. |
| `0x1400E3970` | Shared column binder. Arguments 2/4/5 are ordinal / buffer bytes / C type. ODBC status below 2 succeeds. Call assembly supplies omitted decompiler arguments. |
| All 87 wrappers | Return placeholder is slot 1, OUT i32, SQL/C 4/-16. Prepare/bind/execute failure gives false. Fetch failure also gives false in loaders. Procedure return must be zero. |
| Direction codes | 1 = IN; 2 = INOUT; 4 = OUT. Schema `is_output` does not distinguish OUT from INOUT. It does not prove use of the incoming value. |
| `0x1400B45C0` | Slots 1/2 bind return/account. One loop binds ten i32 mileage IDs at slots 3..12. Unused IDs bind SQL NULL. Three static sites do not mean three parameters. |
| `0x1400A40F0` | Two branch-selected procedure literals use the same return/PCID bindings. One recreated procedure is missing. |
| `0x1400A39C0` | Result columns 2/3 use shared binder, C i32 (4), zero buffer length. Reject more than one fetched row. |
| `0x1400AA220` | Shared binder supplies PCID unsigned i64 and two UTF-16 name buffers, 66 bytes each. |
| `0x1400AC090` | Shared binder supplies AddedSlotID GUID. |

C and SQL types are separate. A different numeric width is not by itself a
schema conflict. Signedness and conversion ranges still need tests.
A descriptor array can contain compiler-joined labels or a trailing sentinel.
The JSON keeps raw labels. It does not invent a slot name from such an array.

## Four result-capacity conflicts

| Wrapper / result ordinal | Binary capacity | Recreated source capacity |
| --- | --- | --- |
| `0x14009E100` / 13 | 512 bytes | `CharacterData.ShowcaseData`: varbinary(4096) |
| `0x1400A59E0` / 3 | 512 bytes | `CharacterCosmesis.NewAppearanceData`: varbinary(4096) |
| `0x1400A70C0` / 4 | 512 bytes | `ChangeCharacterSex.NewAppearanceData`: varbinary(4096) |
| `0x1400A88D0` / 5 | 512 bytes | `ChangeCharacterRace.NewAppearanceData`: varbinary(4096) |

These procedures select the full source blob. They do not reduce it to 512 bytes.
Small values can fit both contracts. ❓ Behavior for larger values needs an
isolated generated-data ODBC test. Do not assert safe truncation or an over-read.
The binary accepts SQL_SUCCESS_WITH_INFO at bind time. This alone does not
establish fetch truncation behavior. No client-datafile research is implied.

## Results and procedure effects

The call table lists only the result columns that the binary binds.
The JSON pairs them with the recreated SELECT order and C target type.
`usp_LoadMileageFeverTime` returns ten columns from UPDATE OUTPUT, not SELECT.
It deactivates prior rows and activates matching fever rows.
`usp_LoadGameAccountTimeBasedPlayInfo` inserts absent reward rows through MERGE
before its five-column SELECT. Neither call is a read-only operation.
Assignment SELECTs, EXISTS tests and derived-table SELECTs are not extra results.

`usp_FinishCharacterRestoration` takes one positive PCID at `0x1400A1470`.
The recreated body requires one state-5 row. It sets state 2, clears deletion delay
and updates the state timestamp. The binary path is `0x14007BB40` →
`0x140031310` → `0x1400A1470`. Failure prevents successful restoration completion.
❓ The recreated body does not prove the original DB state rules.

## Return codes and caller handling

All wrappers preserve a separate numeric procedure-return channel and a Boolean
execution channel. Nonzero procedure return makes the wrapper fail.
The JSON lists the constants declared by each recreated procedure.
A declaration does not prove reachability. Catch blocks can raise an SQL error
and return an error code. An adapter must not collapse these channels into success.

| Wrapper | Checked caller rule |
| --- | --- |
| LoadGameAccount / RegisterGameAccount | `0x140032C50`: abort login on failure. No account row selects registration. |
| UpdateLastLogin | `0x140032C50`: ignore both return channels. |
| UpdateLastLogout | `0x1400334A0`: ignore both return channels and continue cleanup. |
| LoadCharacterList / LoadIncompleteLobbyService | `0x1400336F0`: stop loading on failure. |
| FinishCharacterRestoration | `0x140031310`: failure blocks completion. |
| Other callers | ❓ Complete job/wire error maps remain open. Direct call addresses are in JSON. |

Schema codes include 0 success and 1 generic failure. The name-check and creation
procedures also declare 2 duplicate name and 3 limited name. Guild rename adds
4 already-in-try-state. Auction try calls declare operation-specific codes.
❓ Do not reuse these codes as wire reasons without checking the caller.

## Step 8 — caller error maps and result rules

Build: 0.0.307. These rules apply to the stated paths only.
The JSON gives each call an explicit checked or open map.

| Calls / path | SQL or execution failure | No row / NULL rule | Source |
| --- | --- | --- | --- |
| LoadGameAccount, RegisterGameAccount, optional LoadJumpingCharacterCreationQuota | Abort login; D11/16; close session | No account row selects registration. Quota accepts zero or one row; more than one fails. | `0x140032C50` → `0x1400D7190`; quota `0x1400A39C0` |
| LoadGameAccountTimeBasedPlayInfo, LoadGameAccountExp, InsertGameAccountExp, LoadCharacterList, LoadIncompleteLobbyService and five service detail loaders | Abort cache load; D11/16; close session | Missing EXP selects insertion. Empty character/service lists succeed. Detail loaders do not enforce one row. ❓ Setup checks for absent detail remain open. | `0x1400336F0` → `0x1400D7190` |
| UpdateLastLogin, UpdateLastLogout | Ignore failure; continue login or cleanup | No result rows | `0x140032C50`, earlier cleanup `0x1400334A0` |
| FinishCharacterRestoration | Wrapper failure becomes character result 6, then D26/2. Invalid cached state becomes result 4, then D26/2. | No result rows; require cached state 5; success sets 2 | `0x140031310` → `0x14007BB40` |
| Retained D15 and cached D35 | No DB call in the checked finish/list workers | Reuse cache; no DB failure map applies | earlier `0x1400768C0`, `0x1400C0A90` |
| Other bound calls or other callers of these calls | ❓ Not verified | ❓ Not verified | Follow each JSON call-site list |

These callers do not select a wire reason from individual nonzero SQL codes.
Execution failure and nonzero procedure return take the same checked failure path.
Do not copy SQL codes 2/3 into wire reasons.

### Fetch and conversion

`0x1400E3C30` accepts SQLExecute statuses 0 and 1.
`0x1400E3CB0` maps SQLFetch -1 to failure, 0/1 to a row, and other statuses
to end-of-results. Status 1 logs diagnostics but still supplies a row.
Thus a driver conversion error reported as SQL_ERROR follows the caller failure map.
❓ Driver-specific overflow, NULL and warning behavior was not run.
Do not claim that every non-success status becomes failure in the original helper.

`0x14009C710` maps NULL GameAccountName to an empty string and NULL
RepresentativePCID to zero. No row leaves the caller's found flag clear.
Other account indicators have no explicit NULL branch in the checked loop.
`0x14009E100` clears each row before fetch. NULL deletion/last-play timestamps
skip conversion. NULL showcase skips allocation and copy.
`0x1400A5090` clears common service fields before fetch and has no explicit
per-column NULL rejection. These observations do not establish all driver writes.

### Blob boundary and detail-output limits

The character loader uses the non-NULL showcase indicator as a signed-16-bit
allocation/copy length. No explicit 512-byte clamp appears in that checked path.
Fetch status 1 can reach it. ❓ Actual over-capacity behavior depends on the driver;
no runtime over-read or safe truncation is claimed.
The appearance detail wrappers clear their 512-byte scratch buffers before fetch.
The checked bodies copy PCID to the caller but do not copy the bound appearance
blob (or sex/race values) to it: `0x1400A59E0`, `0x1400A70C0`, `0x1400A88D0`.
❓ Complete service setup consequences remain open. A full payload in MemoryStore
is a local typed fixture contract, not a claim of original output propagation.
Rust rejects generated blobs above 512 bytes. It does not reproduce unsafe copies.

## Step 9 — optional SQL Server Store

`crates/lobbyd/src/mssql.rs` implements the current 12 Store methods.
They select 16 of the 88 procedure contracts. The other 72 stay outside Store.
No SQL Server connection was made during this job.

| Adapter rule | Scope and source |
| --- | --- |
| Fixed call batches; bound values | Tiberius 0.12 sends an `sp_executesql` RPC. Each batch calls a fixed `dbo.usp_*` name in the binary argument order. No input becomes SQL text. This is not a direct named-procedure TDS RPC. |
| Explicit return check | Each batch captures `EXEC @rc` and selects one private return column. Missing/NULL status, nonzero status and unexpected result sets fail. Source contract: all wrappers, section above. |
| Typed errors | `SqlStoreError::Procedure(i32)` stays inside `io::Error`. Driver errors and timeouts remain separate. Server error text is not exposed. Existing Jobs maps stay unchanged. |
| Connection settings | Only runtime `LOBBYD_MSSQL_URL`, ADO style. An explicit server field is required. No credential or endpoint default is added. The feature-enabled CLI selects SQL when the variable is set. |
| Worker | One dedicated thread owns one connection and one Tokio runtime. Store calls still block synchronously. A ten-second local timeout closes the worker. No automatic retry follows a driver error or timeout. ❓ This does not provide nonblocking lane isolation for SQL latency. |
| GUIDs | Store IDs contain Windows GUID memory bytes. Tiberius uses `Uuid::from_bytes_le` and `to_bytes_le`. An asymmetric generated-byte unit test checks the conversion. |
| Result capacity | All four binary 512-byte limits win over recreated varbinary(4096). Values above 512 return InvalidData. No truncation is used. Sources: `0x14009E100`, `0x1400A59E0`, `0x1400A70C0`, `0x1400A88D0`. |
| No row | Account/EXP/quota return None. Lists return empty vectors. A detail no-row returns NotFound. Single-row APIs reject multiple rows as a local safety policy. |
| NULL | Account name/representative PCID and showcase follow the checked defaults. Optional time-based timestamps preserve components. Required fields reject NULL. These stricter local checks do not prove ODBC equivalence. |
| Numeric conversions | Positive SQL bigint inputs reject zero and overflow. Signed-byte results reject tinyint values above 127. Quota tinyint values widen to the bound i32 fields. |

### Explicit limits

Character deletion and last-play timestamp units remain ❓ in `03-db.md`.
The adapter rejects non-NULL values rather than guessing a wire conversion.
Account representative-world is absent from LoadGameAccount results. The adapter
keeps the local zero value. ❓ Original world derivation remains open.
The local Account omits BotScoreIgnore. A true value now rejects loading.
The local Character omits rights and Blocked. The adapter accepts only right 1
and unblocked rows. These are fail-closed local limits, not recovered rules.
Job/level fields remain outside the local cache.
Appearance detail methods return the typed local payload. They do not prove
original output propagation at `0x1400A59E0`, `0x1400A70C0`, `0x1400A88D0`.
Jobs still does not enable detail-dependent service recovery or optional login data.

### Operator tests

Run only against the recreated, disposable LobbyDB. Set `LOBBYD_MSSQL_URL`
in the operator environment. Do not write its value into this repository.

`cargo test --manifest-path lobbygate-rs/Cargo.toml -p lobbyd --features mssql --test mssql -- --nocapture --test-threads=1`

| SQL test | Expected result |
| --- | --- |
| `sql_account_characters_logout_optional_and_blob_boundaries` | Register/load, empty lists, score 29, EXP and quota, time-based MERGE, and all four blob boundaries. 0/511/512 pass; 513/4096 reject. No deliberate SQL error creates unscoped log rows. |
| `sql_retained_login_world_restoration_and_kick_score` | D12/D15 reuse cached data after SQL changes. D24 restores state 5 to 2. World token consumes once. Kick persists the cached score. |
| `sql_existing_encrypted_gate_flow` | Existing encrypted client helpers drive login, cached characters, retained login, world entry, kick and logout through the unchanged gate. |

The test module also repeats the unchanged memory gate test through its helper import.
The three SQL tests print a clear SKIP message when the variable is unset.
Generated account GUIDs and names isolate fixture rows. Cleanup tracks only
preflight-absent IDs. Drop deletes their dependent test rows and account rows.
No global log deletion or identity reset occurs. ❓ Live SQL execution and cleanup
remain unverified. A killed process cannot run Drop; inspect test rows after a crash.
No new binary addresses were recovered in step 9.

## All 88 call contracts

The return placeholder is omitted from the argument column.
Each argument gives SQL type / direction, in binary slot order.
`I`, `IO`, `O` mean IN, INOUT and OUT. Detailed names and C types are in JSON.

| Procedure | 307 wrapper | Arguments after return | Bound result columns | Scoped state |
| --- | --- | --- | --- | --- |
| `usp_LoadGameAccount` | `0x14009c710` | uniqueidentifier/I | GameAccountID, GameAccountName, GameAccountRight, RepresentativePCID, BotScore, BotScoreIgnore | agree |
| `usp_LoadGameAccountTimeBasedPlayInfo` | `0x14009ccf0` | uniqueidentifier/I | ScheduleNumber, TargetType, ExpireTime, Duration, LastUpdate | agree |
| `usp_UpdateGameAccountTimeBasedPlayInfo` | `0x14009d350` | uniqueidentifier/I, int/I, tinyint/I, datetime/I, int/I, datetime/I | — | agree |
| `usp_RegisterGameAccount` | `0x14009d7c0` | uniqueidentifier/I, nvarchar/I, tinyint/I, nchar/I | — | agree |
| `usp_UpdateLastLogin` | `0x14009dbb0` | uniqueidentifier/I, nchar/I | — | agree |
| `usp_UpdateLastLogout` | `0x14009de90` | uniqueidentifier/I, int/I | — | agree |
| `usp_LoadCharacterList` | `0x14009e100` | uniqueidentifier/I | PCID, GameAccountID, SlotID, WorldID, Job, Level, CharacterName, CharacterRight, CharacterState, DeletionDelayTime, LastPlayEndTime, JumpingCharacterState, ShowcaseData, Blocked | conflict |
| `usp_CheckValidityOfCharacterName` | `0x1400a00f0` | nvarchar/I | — | agree |
| `usp_TryCharacterCreation` | `0x1400a02c0` | uniqueidentifier/I, uniqueidentifier/I, smallint/I, nvarchar/I, tinyint/I, bigint/IO | — | agree |
| `usp_FinishCharacterCreation` | `0x1400a06b0` | bigint/I, smallint/I, smallint/I, tinyint/I, bit/I, varbinary/I | — | agree |
| `usp_CancelCharacterCreation` | `0x1400a0a60` | bigint/I | — | agree |
| `usp_TryCharacterDeletion` | `0x1400a0c30` | bigint/I | — | agree |
| `usp_FinishCharacterDeletion` | `0x1400a0e00` | bigint/I, int/I, datetime/O | — | agree |
| `usp_CancelCharacterDeletion` | `0x1400a10d0` | bigint/I | — | agree |
| `usp_TryCharacterRestoration` | `0x1400a12a0` | bigint/I | — | agree |
| `usp_FinishCharacterRestoration` | `0x1400a1470` | bigint/I | — | agree |
| `usp_RetrieveExpiredDeletionWaitingList` | `0x1400a1640` | int/I | GameAccountID, WorldID, PCID | agree |
| `usp_ExpireCharacterDeletionWaiting` | `0x1400a1b30` | bigint/I | — | agree |
| `usp_UpdateLastPlayStart` | `0x1400a1d00` | bigint/I, nchar/I | — | agree |
| `usp_UpdateLastPlayEnd` | `0x1400a1fa0` | bigint/I | — | agree |
| `usp_UpdateCharacterData` | `0x1400a21c0` | bigint/I, smallint/I, smallint/I, varbinary/I | — | agree |
| `usp_UpdateCharacterLevel` | `0x1400a25b0` | bigint/I, smallint/I | — | agree |
| `usp_TryGuildRegistration` | `0x1400a2890` | smallint/I, smallint/I, tinyint/I, nvarchar/I, bigint/IO | — | agree |
| `usp_FinishGuildRegistration` | `0x1400a2c20` | bigint/I | — | agree |
| `usp_CancelGuildRegistration` | `0x1400a2df0` | bigint/I | — | agree |
| `usp_DeleteGuild` | `0x1400a2fc0` | bigint/I, int/I | — | agree |
| `usp_TryRenameGuild` | `0x1400a3200` | bigint/I, int/I, nvarchar/I, int/I, bigint/IO | — | agree |
| `usp_FinishRenameGuild` | `0x1400a3570` | bigint/I, bigint/I | — | agree |
| `usp_CancelRenameGuild` | `0x1400a37d0` | bigint/I | — | agree |
| `usp_LoadJumpingCharacterCreationQuota` | `0x1400a39c0` | uniqueidentifier/I | GameAccountID, CreationQuota, CreationCount | agree |
| `usp_BindCharacterSlot` | `0x1400a3e80` | bigint/I, uniqueidentifier/I | — | missing |
| `usp_UpdateJumpingCharacterStateCompleteWithoutQuota` | `0x1400a40f0` | bigint/I | — | agree |
| `usp_UpdateJumpingCharacterStateComplete` | `0x1400a40f0` | bigint/I | — | missing |
| `usp_LoadGameAccountExp` | `0x1400a4300` | uniqueidentifier/I | GameAccountID, AccountExp, AccountExpQuotaPerDay, LastUpdateTime | agree |
| `usp_InsertGameAccountExp` | `0x1400a47a0` | uniqueidentifier/I, bigint/I, int/I, bigint/I | — | agree |
| `usp_UpdateGameAccountExp` | `0x1400a4af0` | uniqueidentifier/I, bigint/I, int/I, bigint/I | — | agree |
| `usp_LoadRandomStoreInitializationSchedule` | `0x1400a4e00` | bigint/O, bigint/O, bigint/O | — | agree |
| `usp_LoadIncompleteLobbyService` | `0x1400a5090` | uniqueidentifier/I | ServiceID, GameAccountID, ServiceType, ItemInstanceID, ItemDataID, ServiceState | agree |
| `usp_LoadCharacterCosmesisService` | `0x1400a59e0` | bigint/I | ServiceID, PCID, NewAppearanceData | conflict |
| `usp_TryCharacterCosmesisService` | `0x1400a5e80` | uniqueidentifier/I, bigint/I, bigint/I, int/I, varbinary/I, bigint/IO | — | agree |
| `usp_UpdateLobbyServiceStateToItemConsumptionTryState_CharacterCosmesisService` | `0x1400a65c0` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_UpdateLobbyServiceStateToTaskDoneState_CharacterCosmesisService` | `0x1400a6880` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_UpdateLobbyServiceStateToItemConsumptionFinishState_CharacterCosmesisService` | `0x1400a6b40` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_FinishCharacterCosmesisService` | `0x1400a6e00` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_LoadChangeCharacterSexService` | `0x1400a70c0` | bigint/I | ServiceID, PCID, Sex, NewAppearanceData | conflict |
| `usp_TryChangeCharacterSexService` | `0x1400a75d0` | uniqueidentifier/I, bigint/I, bigint/I, int/I, smallint/I, varbinary/I, bigint/IO | — | agree |
| `usp_UpdateLobbyServiceStateToItemConsumptionTryState_ChangeCharacterSexService` | `0x1400a7dd0` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_UpdateLobbyServiceStateToTaskDoneState_ChangeCharacterSexService` | `0x1400a8090` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_UpdateLobbyServiceStateToItemConsumptionFinishState_ChangeCharacterSexService` | `0x1400a8350` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_FinishChangeCharacterSexService` | `0x1400a8610` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_LoadChangeCharacterRaceService` | `0x1400a88d0` | bigint/I | ServiceID, PCID, Race, Sex, NewAppearanceData | conflict |
| `usp_TryChangeCharacterRaceService` | `0x1400a8e60` | uniqueidentifier/I, bigint/I, bigint/I, int/I, smallint/I, smallint/I, varbinary/I, bigint/IO | — | agree |
| `usp_UpdateLobbyServiceStateToItemConsumptionTryState_ChangeCharacterRaceService` | `0x1400a9720` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_UpdateLobbyServiceStateToTaskDoneState_ChangeCharacterRaceService` | `0x1400a99e0` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_UpdateLobbyServiceStateToItemConsumptionFinishState_ChangeCharacterRaceService` | `0x1400a9ca0` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_FinishChangeCharacterRaceService` | `0x1400a9f60` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_LoadCharacterRenameService` | `0x1400aa220` | bigint/I | ServiceID, PCID, OldCharacterName, NewCharacterName | agree |
| `usp_TryCharacterRenameService` | `0x1400aa680` | uniqueidentifier/I, bigint/I, bigint/I, int/I, nvarchar/I, int/I, bigint/IO | — | agree |
| `usp_UpdateLobbyServiceStateToItemConsumptionTryState_CharacterRenameService` | `0x1400aae40` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_UpdateLobbyServiceStateToTaskDoneState_CharacterRenameService` | `0x1400ab370` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_UpdateLobbyServiceStateToItemConsumptionFinishState_CharacterRenameService` | `0x1400ab8a0` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_FinishCharacterRenameService` | `0x1400abdd0` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_LoadSlotAdditionService` | `0x1400ac090` | bigint/I | ServiceID, AddedSlotID | agree |
| `usp_TrySlotAdditionService` | `0x1400ac440` | uniqueidentifier/I, bigint/I, int/I, bigint/IO | — | agree |
| `usp_UpdateLobbyServiceStateToItemConsumptionTryState_SlotAdditionService` | `0x1400ac9e0` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_UpdateLobbyServiceStateToTaskDoneState_SlotAdditionService` | `0x1400acf10` | bigint/I, uniqueidentifier/I, uniqueidentifier/I | — | agree |
| `usp_UpdateLobbyServiceStateToItemConsumptionFinishState_SlotAdditionService` | `0x1400ad770` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_FinishSlotAdditionService` | `0x1400adca0` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_WriteUserRenameServiceLog` | `0x1400adf60` | uniqueidentifier/I, bigint/I, int/I, nvarchar/I, nvarchar/I | — | agree |
| `usp_TryExtendJumpingCharacterCreationQuota` | `0x1400ae650` | uniqueidentifier/I, bigint/I, int/I, bigint/IO | — | agree |
| `usp_UpdateLobbyServiceStateToItemConsumptionTryState_ExtendJumpingCharacterCreationQuota` | `0x1400aebf0` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_UpdateLobbyServiceStateToTaskDoneState_ExtendJumpingCharacterCreationQuota` | `0x1400af120` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_UpdateLobbyServiceStateToItemConsumptionFinishState_ExtendJumpingCharacterCreationQuota` | `0x1400af650` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_FinishExtendJumpingCharacterCreationQuota` | `0x1400afb80` | bigint/I, uniqueidentifier/I | — | agree |
| `usp_TryAuctionEscrow` | `0x1400afe40` | uniqueidentifier/I, uniqueidentifier/I, smallint/I, bigint/I, bigint/I, bigint/O | — | agree |
| `usp_CancelAuctionEscrow` | `0x1400b0680` | bigint/I, bigint/I | — | agree |
| `usp_FinishAuctionEscrow` | `0x1400b0950` | bigint/I, bigint/I | — | agree |
| `usp_TryAuctionEscrowRollback` | `0x1400b0c20` | uniqueidentifier/I, bigint/O, uniqueidentifier/O, smallint/O, bigint/O, nvarchar/O, bigint/O, bigint/O | — | agree |
| `usp_FinishAuctionEscrowRollback` | `0x1400b1240` | bigint/I, bigint/I, bigint/I | — | agree |
| `usp_TryAuctionResultDelivery` | `0x1400b1560` | nvarchar/I, uniqueidentifier/I, uniqueidentifier/I, smallint/I, bigint/I, bigint/O, nvarchar/O, bigint/O | — | agree |
| `usp_FinishAuctionResultDelivery` | `0x1400b2000` | bigint/I, bigint/I | — | agree |
| `usp_LoadMileagePolicy` | `0x1400b2fc0` | int/I | MileagePolicyID, MileageType, MileagePolicyName, MileageConditionType, MileageConditionValue | agree |
| `usp_LoadMileageFeverTime` | `0x1400b38b0` | int/I, datetime/I | MileageFeverTimeID, MileageID, MileageFeverTimeName, EffectiveStartTime, EffectiveEndTime, FeverVirtualCurrencyAmount, RepeatType, RepeatStartTime, RepeatDuration, DayOfWeek | agree |
| `usp_LoadMileageAccumulatedTimePerDay` | `0x1400b45c0` | uniqueidentifier/I, int/I, int/I, int/I, int/I, int/I, int/I, int/I, int/I, int/I, int/I | GameAccountID, MileageID, EffectiveEndTime, AccumulateMinute | agree |
| `usp_LoadRetryMileageDeposit` | `0x1400b4e40` | int/I, bigint/I, bigint/I, bigint/O | MileageDepositID, GameAccountID, MileageID, AccumulateStartTime, AccumulateEndTime, AccumulateAmount, MileageVirtualCurrencyID, MileageVirtualCurrencyExpireTime | agree |
| `usp_RetrieveMileageDeposit` | `0x1400b5ba0` | bigint/I | MileageDepositID, GameAccountID, MileageID, AccumulateStartTime, AccumulateEndTime, AccumulateAmount, MileageVirtualCurrencyID, MileageVirtualCurrencyExpireTime | agree |
| `usp_TryMileageDeposit` | `0x1400b62d0` | uniqueidentifier/I, int/I, datetime/I, datetime/I, bigint/I, datetime/I, smallint/I, bigint/O | — | agree |
| `usp_FinishMileageDeposit` | `0x1400b6bc0` | bigint/I, bit/I | — | agree |

## Open work

1. Trace the remaining caller return codes to job results and wire reasons.
2. Verify result SQL lineage, NULL defaults and all numeric conversion ranges.
3. Test blob lengths 0, 512, 513 and 4096 with generated data and an isolated adapter.
4. Specify both missing procedures from binary callers. Do not add guessed SQL.
5. Verify nested procedure calls, transaction effects, row-count rules and retries.
6. Keep the local Store separate from the future live SQL adapter.

## Excluded boundaries

TSS strings remain presence-only sources: `tss_get_sdk` at `0x140175FC8` and
`tss_sdk` at `0x140175FD8`. GameGuard sources remain the directory entries
`ggauth64_27.dll` and `csauth3.cfg`. ❓ Their EXE addresses remain unknown.
BloodCert and BNSHelper remain excluded. No excluded component was analysed.
