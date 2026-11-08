# rev-small — native parity review

**Both ports have confirmed behavior differences from build 348.** Neither executable can replace its native daemon without further work.

| Scope | Result |
|---|---|
| AchievementDaemon | Gameplay updates, completion, rewards, rule loading, persistence, and session handling differ. |
| SmartCouponDaemon | The executable has no durable SQL store or NP transport. NP result handling and request decoding also differ. |
| Evidence | Native pseudocode, assembly, static tables, record layouts, and SQL bindings. |
| Tests | Read, but not run. Some tests require behavior that differs from native. |
| IDA | Opened one database at a time. Both databases are closed. |
| Git | No git commands were run. |

**Priority:** P1 = fix before service use. P2 = a confirmed, input-dependent behavior difference.

Paths below are relative to `bns-server/`.

## 1. AchievementDaemon — confirmed divergences

| Priority | Function | Rust file:line | Confirmed difference and native address | Fix |
|---|---|---|---|---|
| P1 | Gameplay event processing | `Program/Server/AchievementDaemon/src/service.rs:411` | Rust accepts normal gameplay events but does not update their registers. Native `KillNpc` updates matching registers at `0x140019C70`. Native `QuestComplete` updates quest-related registers at `0x14001A6E0`. | Implement the native event-to-register rules. Do not treat successful parsing as successful processing. |
| P1 | `main` / store selection | `Program/Server/AchievementDaemon/src/main.rs:57` | The executable uses `MemoryStore`. All state is lost on restart. Native loads and writes SQL state through `0x140003D60`, `0x1400044F0`, and the insert/update procedures. | Add a durable store adapter. Use it in the executable. |
| P1 | `take_reward_reply` | `Program/Server/AchievementDaemon/src/pc.rs:460` | Both reply forms send **step, id**. Native sends **achievementId, rewardStep** at `0x1400183E0`. Caller assembly at `0x140019640` confirms the order. | Swap the final two fields in success and failure replies. Correct the spec and tests. |
| P2 | Client completion / `check_complete` | `Program/Server/AchievementDaemon/src/service.rs:329`; `Program/Server/AchievementDaemon/src/pc.rs:264` | Rust validates the requested record, discards its step, then loops from `completedStep + 1`. Native passes the exact requested record at `0x14001B070`. The record overload at `0x140017DD0` checks only that record. Rust can complete extra client steps or fail to complete a valid later requested step. | Pass the exact record to a single-record completion function. Keep register-link iteration separate. |
| P2 | Completion register reset | `Program/Server/AchievementDaemon/src/pc.rs:293`; `Program/Server/AchievementDaemon/src/rules.rs:100` | Rust does not load or apply `progress-step-register-clear-N`. Native completion calls `SetRegister(..., 0)` for flagged condition registers at `0x140017DD0`. | Load the five reset flags. Apply each reset through the register operation after completion. |
| P2 | `available_take_reward` — first gates | `Program/Server/AchievementDaemon/src/pc.rs:306` | Rust first rejects `!has_reward`. Native first rejects **no completed step and no reward marker**, then checks deprecation and the other gates at `0x140017B80`. For a normal uncompleted record with a reward, Rust returns 9 where native returns 5. | Use the native gate order. Remove the invented initial `has_reward` gate. |
| P2 | `available_take_reward` — earlier rewards | `Program/Server/AchievementDaemon/src/pc.rs:325` | Rust does not scan earlier unclaimed steps. Native checks their derived reward-type byte at record `+146`; a nonzero type returns 9 at `0x140017B80`. Rust can advance the marker past an earlier reward. | Resolve the intervening records and apply the native reward-type check before success. |
| P2 | Reward failure with transaction 0 | `Program/Server/AchievementDaemon/src/service.rs:343` | Rust suppresses every transaction-0 reply. Native sends reason 6 for a missing record even when the transaction is 0 at `0x140019640`. It suppresses transaction-0 replies only on the found-record path. | Send the missing-record failure before the transaction-0 suppression gate. |
| P2 | Register load/version handling | `Program/Server/AchievementDaemon/src/store.rs:13`; `Program/Server/AchievementDaemon/src/pc.rs:116` | Rust has no register version. Native compares stored and current versions at `0x140003D60`. A mismatch resets the value to 0, marks UPDATE, and sends command 12 with the old/new versions and values. | Carry versions through rules and storage. Implement reset, notification, and persistence. |
| P2 | Progress visibility | `Program/Server/AchievementDaemon/src/rules.rs:178`; `Program/Server/AchievementDaemon/src/pc.rs:393` | Rust omits achievement-to-register `progress-show` inheritance. It also sends all register rows, including hidden persistent rows. Native derives visibility at `0x140010600`, includes `for_newbie_care` in row visibility, and filters output at `0x140018520` / `0x140018A10`. | Derive and retain row visibility. Filter both full and changed register output. |
| P2 | Client increment permission | `Program/Server/AchievementDaemon/src/rules.rs:179` | Rust preserves an explicit increment flag on any register type. Native forces it on for subtype 56 (`client-only`) and clears it for other types at `0x140010AF0`. Rust accepts increments that native rejects. | Apply the native subtype normalization after loading. |
| P2 | Persistence normalization | `Program/Server/AchievementDaemon/src/rules.rs:177` | Rust keeps the XML persistence flag unchanged. Native clears it for subtypes 19, 51–55, and 59 at `0x140010AF0`. This changes modified markers and DB writes. | Apply the native persistence normalization before register use. |
| P2 | Duel/party selector loading | `Program/Server/AchievementDaemon/src/rules.rs:180` | Rust reads numeric `sub-type`. Native uses named `duel-type` and `party-battle-type` attributes. Metadata is at `0x140092540` / `0x140093EA0`; consumers are at `0x140019500` / `0x1400195B0`. Rust therefore uses the wrong duel source or skips party-grade updates. | Parse the native attributes and enum names. |
| P2 | Deprecation and score derivation | `Program/Server/AchievementDaemon/src/rules.rs:229`; `Program/Server/AchievementDaemon/src/pc.rs:155` | Rust does not propagate later-step deprecation backward. It sums raw step scores, including deprecated records. Native derives deprecation and cumulative `score_sum` at `0x140010600`, then uses cumulative scores at `0x140016CA0` / `0x140017DD0`. | Add the native preprocessing pass. Use the selected record’s cumulative score. |
| P2 | Register links | `Program/Server/AchievementDaemon/src/rules.rs:234` | Rust stores deduplicated achievement IDs and silently stops at eight IDs. Native stores exact `(id, step, job)` keys and fails loading on more than eight links at `0x140010600`. The register overload consumes those exact keys at `0x140017D60`. | Store full record keys. Reject link overflow instead of truncating it. |
| P2 | Default condition operator | `Program/Server/AchievementDaemon/src/rules.rs:211` | A missing `register-type-N` becomes `Above` in Rust. Native defaults to `none = 0`; that operator fails the condition. Enum metadata is at `0x140094D28` / `0x1400E3550`; evaluation is at `0x140017DD0`. | Represent `None`. Use it as the missing-attribute default. |
| P2 | Job-name lookup | `Program/Server/AchievementDaemon/src/rules.rs:14` | The first 12 names are correct. Native also defines later named values, including `창술사 = 13` and `뇌전술사 = 14`, at `0x1400DDEA0`. Rust rejects those names during rule loading. | Complete the native name mapping. Do not infer the mapping from the first 12 entries only. |
| P2 | Wire event acceptance | `Program/Server/AchievementDaemon/src/event.rs:90` | Rust accepts all 55 metadata classes on command 6. Native accepts 48 at `0x140013770`. It excludes 1013, 1014, and 1803–1807. In Rust, a wire 1804 can reach cluster cleanup. | Separate metadata lookup from the wire allowlist. Keep synthesized/internal events off command 6. |
| P2 | `CheckEventProcessingReq` | `Program/Server/AchievementDaemon/src/service.rs:254` | Rust requires a loaded Pc with the same `tid`. Native `IsLogined` checks only membership in the pcdbid routing map at `0x140009B50`; `tid` is echoed at `0x14000E770`. Rust returns false for an existing Pc with a different supplied `tid`. | Check pcdbid membership. Preserve the supplied `tid` only in the reply. |
| P2 | Manager packet gate | `Program/Server/AchievementDaemon/src/service.rs:237` | Rust answers Ping before Hello. Native gates every non-Hello command on an assigned channel at `0x14000E090`. | Apply the Hello gate before manager command dispatch. |
| P2 | Game Ping reply | `Program/Server/AchievementDaemon/src/service.rs:100` | Rust replies with the authenticated cluster ID. Native echoes the cluster ID from the Ping request at `0x14000D6B0`. | Echo the decoded request field. |
| P2 | Ping timeout | `Program/Server/AchievementDaemon/src/server.rs:32` | Rust starts only the update timer. Native checks last-Ping age at `0x14000CED0`; the timer at `0x140002650` rearms the check after 3000 ms. Expired channels are removed, with socket closure controlled by configuration. | Track last Ping. Add the native expiry and cleanup behavior. |
| P2 | `ClusterDisconnected` | `Program/Server/AchievementDaemon/src/service.rs:286`; `Program/Server/AchievementDaemon/src/service.rs:520` | Rust ignores `disconnected_all` and always selects one cluster. Native branches between one-cluster and all-cluster cleanup at `0x140019990`, using `0x140008F80` / `0x140008D40`. | Pass and process the flag. Keep ordinary socket disconnect on the single-cluster path. |
| P2 | `update_pc` failure handling | `Program/Server/AchievementDaemon/src/service.rs:551` | Rust short-circuits updates when inserts fail. It also retains all group markers when a later group fails. Native calls the DB groups independently and retains failed-row state at `0x140008B20`. | Attempt independent groups independently. Clear only the markers for successful writes. |

### Native reward gate order

`Pc::availableTakeReward` at `0x140017B80` uses this order:

| Order | Native check | Result |
|---|---|---|
| 1 | Completed step is 0 **and** reward marker is absent (`-1`) | 5 |
| 2 | Requested record is deprecated | 7 |
| 3 | Record is client-completable, or its first condition reference is absent | 5 |
| 4 | Completed step is below the requested step | 9 |
| 5 | Reward marker is at or above the requested step | 8 |
| 6 | An intervening earlier record is missing | 5 |
| 7 | An intervening earlier record has nonzero reward type at `+146` | 9 |
| 8 | All checks pass | 0; store the requested reward step with marker 2 |

The byte at `+146` is **reward type**, not a register-reset flag.

## 2. SmartCouponDaemon — confirmed divergences

| Priority | Function | Rust file:line | Confirmed difference and native address | Fix |
|---|---|---|---|---|
| P1 | `main` / publication store | `Program/Server/SmartCouponDaemon/src/main.rs:28`; `Program/Server/SmartCouponDaemon/src/store.rs:193` | The executable creates an empty `MemoryStore`. It never adds coupon rules. Accepted game events therefore produce no publications, and all state is lost on restart. Native registers logs through SQL at `0x140010B00`, obtains publications from SQL at `0x1400117C0` / `0x140012170`, and reloads pending work at `0x140016050` / `0x1400161A0`. | Add a durable SQL adapter. Use the procedure results for publication selection and restart recovery. |
| P1 | `main` / NP selection | `Program/Server/SmartCouponDaemon/src/main.rs:28` | The executable always uses `NoNp`, including when `np-enabled=yes`. Native initializes NP when enabled at `0x1400150E0` and sends publication requests at `0x14000FF10`. Rust cannot publish a coupon through NP. | Select a real NP transport from configuration. Retain `NoNp` for disabled or test use. |
| P2 | Worker / publication dispatch | `Program/Server/SmartCouponDaemon/src/server.rs:40`; `Program/Server/SmartCouponDaemon/src/service.rs:370` | The sole worker awaits each NP result. One pending request blocks later game-event and retry work. Native `SendPublicationEvent` registers an independent job and returns at `0x14000BE30`; sending does not await the response at `0x14000FF10`. | Separate request dispatch from result completion. Keep independent pending publication jobs. |
| P2 | NP error 5809 | `Program/Server/SmartCouponDaemon/src/np.rs:44`; `Program/Server/SmartCouponDaemon/src/np.rs:64` | Rust classifies 5809 as failure with unchanged status and manager reason 6. Native routes 5809 into success handling for commands 2851/2852 at `0x14000A310`. It records publication success and uses manager reason 0. | Route 5809 through success processing. Decode the receipt as on the native success path. |
| P2 | NP status phrase | `Program/Server/SmartCouponDaemon/src/np.rs:56` | Rust requires exact `"Success"` for status 400. Native uses `_wcsicmp` at `0x14000A310`. For example, `"success"` takes the success path natively but the error path in Rust. | Use a case-insensitive comparison for this protocol token. |
| P2 | Partial request decoding | `Program/Server/SmartCouponDaemon/src/service.rs:207`; `Program/Server/SmartCouponDaemon/src/service.rs:237`; `Program/Server/SmartCouponDaemon/src/service.rs:442` | Rust always sends `tid = 0` on decode failure. Native starts at 0 but retains a fully decoded `tid` if a later field is short. This applies to character, account, and manager retry requests at `0x140008360`, `0x140008E80`, and `0x14000F2F0`. | Preserve completed output fields during decode, or recover the complete `tid` prefix for the failure reply. |
| P2 | Missing event attributes | `Program/Server/SmartCouponDaemon/src/config.rs:255` | Rust aborts the complete configuration load. Native logs and skips the malformed entry, then continues at `0x140002C80`. One bad entry prevents valid later entries from loading in Rust. | Skip the bad entry. Continue with the next sibling. |
| P2 | Event child element check | `Program/Server/SmartCouponDaemon/src/config.rs:247` | Rust accepts any child name if the four attributes exist. Native accepts only case-insensitive `event` and skips other names at `0x140002C80`. | Check the child name before reading event attributes. |
| P2 | Event numeric attributes | `Program/Server/SmartCouponDaemon/src/config.rs:263` | Rust requires a complete numeric string. Native uses `_wtoi` for `id` and `type` at `0x140002C80`. A value such as `"10001tail"` converts to 10001 natively but stops Rust loading. | Use the native numeric conversion rules for this loader. |
| P2 | Repeated manager Hello with ID 0 | `Program/Server/SmartCouponDaemon/src/service.rs:425` | Rust rejects every repeated Hello. Native rejects it when the existing channel’s `GetId()` is nonzero at `0x14000EF40`. The first Hello accepts managerWid 0, so a repeated valid Hello on that channel can pass natively. | Use the native channel-ID guard if exact parity is required. Do not add a new zero-ID rejection. |

## 3. Packet ranges, formats, and widths

### Dispatch and request formats

| Daemon / port | Native dispatch | Verified request formats | Result |
|---|---|---|---|
| Achievement game | Commands 1–12, with a nonnull handler. Only Hello is allowed before channel assignment. `0x14000C1D0` | Hello `hiih`; Ping `hqh`; client completion `hqqqqhChh;Chh;` | Rust formats match. Ping reply content differs as listed above. |
| Achievement manager | Commands 1–7, with a nonnull handler. All non-Hello requests require a channel. `0x14000E090` | Hello `hi`; Ping `hqq`; status check `hqq` | Rust formats match. The pre-Hello Ping gate differs. |
| SmartCoupon daemon | Nonnull table entries are **1, 4, 7**. Before authorization, only 1. Table `0x140083660`; handler `0x14000DFE0` | Hello `hhQhh`; character `hqiohq`; account `hqio` | Rust formats and normal command selection match. |
| SmartCoupon manager | Range 1–9. Nonnull entries are **1, 4, 7**. Before Hello, only 1. Table `0x1400836B0`; handler `0x14000E980` | Hello `hhh`; retry `hqoq`; load retry `hq` | Rust formats match. |

The native SmartCoupon daemon dispatcher has no explicit upper-bound check before table indexing. This does **not** establish valid commands above the table. Rust must retain safe bounds checks.

### Field widths

| Code | Width / layout | Review result |
|---|---|---|
| `q` | 8 bytes | Matches |
| `Q` | 1 byte | Matches |
| `c` | 1 byte | Matches |
| `h`, `E` | 2 bytes | Matches |
| `i` | 4 bytes | Matches |
| `o` | 16-byte account ID | Matches |
| `s` | Counted UTF-16 units | Matches for the checked packet layouts |

All **55 Achievement metadata field arrays** match the native names and primitive widths. However, only **48** classes are accepted by the native wire parser.

Two important event widths are correct in the schema:

- `PcLogin.job` is **1 byte**.
- `CompleteTakeReward.job` is **2 bytes** on the wire.

### Reply checks

| Reply | Native layout | Result |
|---|---|---|
| Achievement Data, command 7 | `hiqqqihcChq;Chhq;Chh;` | Format matches. Register visibility differs. |
| Achievement CompletedNotify, command 11 | `hsqqqhhhc` | Field order matches. |
| Achievement reward success, command 9 | `hqqhh`: transaction, pcdbid, **id, step** | Rust swaps the last fields. |
| Achievement reward failure, command 10 | `hEqqhh`: reason, transaction, pcdbid, **id, step** | Rust swaps the last fields. |
| SmartCoupon event success | `hqq`: tid, logId | Matches |
| SmartCoupon request failure | `hqE`: tid, reason | Layout matches. Partial-decode `tid` differs. |
| SmartCoupon retry success | `hqq`: tid, acceptTid | Matches |
| SmartCoupon load-retry success | `hq`: tid | Matches |

SmartCoupon Hello gate order and reasons match native for the normal path: decode **2**, version **9**, service state **8**, missing daemon **2**, IP **11**, duplicate session **13**. The duplicate path closes the old session and refuses the new session.

## 4. AchievementDaemon — resolved ❓ items

| Item | Answer | Native address |
|---|---|---|
| Ping timeout | Native enforces it. It removes expired channels and can close the socket. Rust does neither. | `0x14000CED0`; timer `0x140002650` |
| `IsLogined` | It checks pcdbid routing-map membership. It does not compare the supplied `tid`. | `0x140009B50`, `0x14000E770` |
| Events 1805 / 1806 | They are `DaemonStart` / `DaemonStop`. They are metadata/internal classes, not command-6 wire classes. | `0x140015680`, `0x140013770` |
| Events that need a Pc | Gameplay, item, and debug events need one. Login/logout and internal 1803–1807 do not carry that requirement. Routing can still discard an unknown Pc event before processing. | `0x140015680`, `0x140009960`, `0x140008950` |
| `ClusterDisconnected` | A normal disconnect flushes and removes one cluster’s Pcs. `disconnected_all` selects all Pcs on the target event thread. | `0x140019990`, `0x140008F80`, `0x140008D40` |
| `progress-show` | Achievement visibility propagates to referenced registers. Client output filters hidden rows. `for_newbie_care` also contributes to row visibility. | `0x140010600`, `0x140003D60`, `0x140018520`, `0x140018A10` |
| `increment-from-client` | Native normalizes it to true only for `client-only`, subtype 56. Other subtypes clear it. | `0x140010AF0` |
| Register-type enum | `none=0`, `above=1`, `below=2`, `bits-on=3`. Above means `>=`; below means `<=`; bits-on tests the low specified number of bits. | `0x140094D28`, `0x1400E3550`, `0x140017DD0` |
| Grade selector attribute | Use `duel-type` and `party-battle-type`, not `sub-type`. | `0x140092540`, `0x140093EA0` |
| Single-record completion versus loop | The record overload checks one supplied exact record. The register overload iterates up to eight linked exact records. Thus one register operation can complete several records. | `0x140017DD0`, `0x140017D60` |
| Register reset after completion | Native resets condition registers selected by `progress-step-register-clear-N`. The flags start at record `+101`. | `0x140017DD0`; attribute metadata `0x140094E30` |
| Record byte `+146` | It is derived reward type. Earlier unclaimed reward-bearing records block a later reward with reason 9. | `0x140010600`, `0x140017B80` |
| Modified markers | New tracked row → INSERT. Changed tracked row → UPDATE unless already INSERT. Unchanged values do not gain a marker, but the operation still checks linked completion. “Tracked” means persistence or progress visibility after normalization. | Add `0x140016F70`; Set `0x140017260` |
| SQL names and arguments | Confirmed from prepared SQL and parameter bindings, not only logs. Register writes include **version**. | See below |
| Korean job names | The first 12 Rust mappings match native. The native enum continues beyond 12. | `0x1400DDEA0` |

### Native job values

All values below come from the native sequence at `0x1400DDEA0`.

| Value | Name | Value | Name |
|---:|---|---:|---|
| 0 | `job-none` | 10 | 기권사 |
| 1 | 검사 | 11 | 투사 |
| 2 | 권사 | 12 | 궁사 |
| 3 | 기공사 | 13 | 창술사 |
| 4 | 격사 | 14 | 뇌전술사 |
| 5 | 역사 | 15 | `pc-max` |
| 6 | 소환사 | 16 | 소환수-루키 |
| 7 | 암살자 | 17 | 소환수-striker |
| 8 | 귀검사 | 18 | 소환수-defender |
| 9 | 주술사 | 19 | 소환수-controller |

Selector values are:

- `duel-type`: `none=0`, `solo=1`, `team=2`.
- `party-battle-type`: `none=0`, `occupation-war=1`, `capture-the-flag=2`, `lead-the-ball=3`.

### Confirmed Achievement SQL calls

| Procedure and arguments | Native address |
|---|---|
| `usp_LoadRegisters(pcdbid)` → id, version, value | `0x140003D60` |
| `usp_LoadCompleted(pcdbid)` | `0x1400044F0` |
| `usp_InsertRegister(pcdbid, id, version, value)` | `0x140004D40` |
| `usp_UpdateRegister(pcdbid, id, version, value)` | `0x140005260` |
| `usp_InsertCompleted(pcdbid, id, step, completed)` | `0x140005760` |
| `usp_UpdateCompleted(pcdbid, id, step, completed)` | `0x140005CD0` |
| `usp_DeleteCompleted(pcdbid, id)` | `0x140006240` |
| `usp_UpdateTakeStepReward(pcdbid, id, step)` | `0x140006550` |
| `usp_DeleteRegisterAll(pcdbid)` | `0x1400069A0` |
| `usp_DeleteCompletedAll(pcdbid)` | `0x140006C30` |

Register bindings use **8-byte pcdbid, 2-byte id, 2-byte version, and 8-byte value**.

## 5. SmartCouponDaemon — resolved ❓ items and evidence limit

| Item | Answer | Native address |
|---|---|---|
| Missing event attribute | Log and skip that entry. Continue loading the configuration. Do not fail the entire load. | `0x140002C80` |
| Decode-failure `tid` | Initially 0. A fully decoded `tid` survives failure in a later field. | `0x140008360`, `0x140008E80`, `0x14000F2F0` |
| `enable` / event type gates | The request handlers do not check these fields. They check event existence and daemon type. Rust matches this behavior. | `0x140008360`, `0x140008E80` |
| Session after event-request failure | An authorized request session stays open after the failure reply. Rust matches. | `0x140008360`, `0x140008E80` |
| Manager Hello failure | Sends command 3 and returns false, which closes the session. The repeated-Hello guard uses channel ID, as described above. | `0x14000EF40` |
| Publication selection | Native obtains publication rows from SQL procedure results. The available wrappers do not contain the SQL eligibility rules. The local Rust mapping is not a verified replacement. | `0x1400117C0`, `0x140012170`, `0x14000B6E0` |
| Empty handler slots | Only 1, 4, and 7 are nonnull in each checked table. Treating other in-table slots as failed dispatch matches native. | `0x140083660`, `0x1400836B0` |
| Unknown NP reply command | The outer dispatcher accepts only 2851/2852. Do not infer end-to-end behavior from `eventNotiRes` alone. | Outer dispatch `0x140010700`; job `0x14000A310` |
| NP expiry result | The task-3 expiry handler sets manager reason 3. A fixed publication-response deadline was **not established** in the inspected path. | `0x14000A9D0`, `0x140009F10` |

The fixed NP request timeout remains **❓ unverified**. Two other intervals are confirmed, but neither is that timeout:

| Interval / event | Confirmed native behavior | Address |
|---|---|---|
| 5 seconds | NP connection timeout | `0x140010340` |
| 30,000 ms | Periodic NP Ping | `0x140004D40`, `0x140004490` |
| NP connection closes | Transaction `Expire` resumes stored task 2 with a null response. This is not the task-3 expiry path. An unset manager reason becomes 1 at finalization. | `0x140010560`, `0x14000FE70`, `0x140006C10`, `0x140009CC0` |

### Corrected NP result table

| NP result | Native DB action | Manager reason |
|---|---|---:|
| No channel / send failure | None | 5 |
| Status 200, or status 400 with case-insensitive `Success`; command 2851/2852 | PublicationSuccess | 0 |
| Error **5809**, command 2851/2852 | **PublicationSuccess** | **0** |
| Error 42, 5812, or 30002 | Leave publication status unchanged | 6 |
| Other error | PublicationFailure | 6 |
| Task-3 expiry | None | 3 |

Source: `EventNotiJob::eventNotiRes` at `0x14000A310`, expiry at `0x14000A9D0`, and finalization at `0x140009CC0`.

## 6. Test and spec corrections

The tests were not executed. Source inspection found these incorrect expectations:

| Test or spec area | Required correction |
|---|---|
| Achievement reward tests | Expect `id, step`, not `step, id`. Apply the native first gate and transaction-0 missing-record exception. |
| Achievement completion tests | Test exact requested steps. Test reset flags. Do not assume that one register operation can complete only one record. |
| Achievement condition fixtures | Do not omit `register-type-N` and expect native `above` behavior. |
| SmartCoupon missing-attribute test | Expect the bad entry to be skipped, not a complete load error. |
| SmartCoupon short-request tests | Preserve a complete decoded `tid` when a later field is short. |
| SmartCoupon NP tests | Add 5809 success and case-insensitive `Success` cases. |
| SmartCoupon worker tests | Use a pending NP response. Confirm that later work can continue. |
| Restart tests | A seeded `MemoryStore` does not verify durable restart recovery. |

Rust is **stricter** for several valid native inputs, including SmartCoupon partial-decode replies, event numeric attributes, and later Achievement job names. It is **looser** for other inputs, including internal Achievement events on the wire, non-client-only register increments, and non-`event` SmartCoupon child elements.

The confirmed findings above do not depend on unknown SQL procedure bodies or an assumed NP request timeout.

No files were edited.