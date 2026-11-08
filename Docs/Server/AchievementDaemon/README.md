# AchievementDaemon (build 348)

Source binary: `AchievementDaemon.exe`, build path `AchievementDaemon-Common-0.0.348`.
Rust port: `Program/Server/AchievementDaemon` (crate `achievementdaemon`).
Facts marked ❓ are not verified in the binary.

| Connection | Port | Main commands |
| --- | --- | --- |
| GameDaemon → AchievementDaemon | 22200 | Hello 1, Ping 4, Event 6, client completion 8 |
| AchievementDaemon → GameDaemon | 22200 | Data 7, reward 9/10, completion 11, register reset 12 |
| Manager → AchievementDaemon | 32200 | Hello 1, Ping 4, status check 6 |

## 1. Config and Ping timer

File: `config/achievementd.config.xml`. Root: `<config module='achievement-daemon'>`.
Each option uses `<option name='...' value='...'/>`.

| Option | Default | Use in the port |
| --- | --- | --- |
| `internal-port` | 22200 | Game port |
| `manage-port` | 32200 | Manage port |
| `ping-enable` | yes | Enable the independent Ping timer |
| `check-ping-time` | 30000 ms | Maximum last-Ping age |
| `disconnect-when-ping-timeout` | yes | Close the expired channel's socket |
| `max-pc-count` | 400000 | Drop a new login above this count |
| `update-cycle` | 3000 ms | Run updatePc for each Pc |
| `log-register-change` | no | Parsed |
| `data-path` | — | Rule table folder |
| `root-path`, `log-path`, `config-path`, `cert`, `alert-send-address`, `log-codepage`, `dump-type`, `worker-thread`, `event-thread` | — | Keep in `options`; not used |

`config/achievementd-iplist.xml` uses `<cluster-list><cluster ip='a.b.c.d' cluster-id='n'/></cluster-list>`.
An arena cluster id is arena id + 10000.

Native `CheckPing` at `0x14000CED0` removes channels when last-Ping age is **greater than** the maximum interval.
It removes at most 100 channels per check.
It removes the channel from routing even when socket closure is disabled.
It closes the socket only when `disconnect-when-ping-timeout` is true.
Socket closure runs the ordinary single-cluster disconnect path.
Without socket closure, the session stays open and its Pcs stay loaded.
A valid Ping updates the channel's last-Ping time (`0x14000D6B0`). Other traffic does not.
The port uses a monotonic millisecond clock, separate from event time.
`CheckPingTimer::HandleEvent` rearms **3000 ms after each check** (`0x140002650`).

## 2. Wire

Frames: i16 size (4..=4000), i16 cmd, body. Format codes follow bloodlib.

### Game port (`0x14000C1D0`)

The command must be 1..=12 and have a handler.
Only Hello can run before channel assignment.
The handler keeps the session open after an invalid request.

| Cmd | Handler | Request | Result |
| --- | --- | --- | --- |
| 1 Hello | `0x14000D280` | `hiih`: cmd, packetVersion, eventVersion, clusterId | Decode failure → `hh` (3, 2). Versions not 10 / 33 → (3, 3). Unlisted IP/cluster → (3, 4). Channel failure → (3, 2). Success → (2, clusterId), then internal event 1803. |
| 4 Ping | `0x14000D6B0` | `hqh`: cmd, count, clusterId | `hqh` (5, count, **request clusterId**). Refresh last Ping only after successful decode. |
| 6 EventOccuredNotify | `0x14000DE90` | Event frame (section 3) | Post. No reply. |
| 8 CompleteFromClientNotify | `0x14000D7E0` | `hqqqqhChh;Chh;`: cmd, eventTime, tid, pcdbid, hint, clusterId, registers (id, value), achievements (id, step) | First group → internal 1013. Second group → internal 1014. |

The port models channel failure as an already assigned session or cluster.

| Reply | Format | Fields |
| --- | --- | --- |
| 7 Data | `hiqqqihcChq;Chhq;Chh;` (fragments, 8-byte header) | tid, pcdbid, hint, completedScore, clusterId, changed, registers (id, value), completed (id, step, time), rewards (id, rewardStep) |
| 9 TakeRewardOk | `hqqhh` | transactionId, pcdbid, **achievementId, rewardStep** |
| 10 TakeRewardFail | `hEqqhh` | reason, transactionId, pcdbid, **achievementId, rewardStep** |
| 11 CompletedNotify | `hsqqqhhhc` | pcname, pcdbid, pcid, tid, achievementId, oldStep, newStep, isLastStep |
| 12 RegisterResetNotify | `hsqqqhhhqq` | pcname, pcdbid, pcid, tid, registerId, oldVersion, newVersion, oldValue, newValue (0) |

Reward reply source: `0x1400183E0`. Caller assembly at `0x140019640` confirms `id, step`.
A missing record sends reason 6 even with transactionId 0.
A found record suppresses the reply when transactionId is 0 (`0x140019640`).
Register reset source: `LoadRegisters` at `0x140003D60`.

SendAllData (changed=0) filters hidden register rows (`0x140018520`).
SendChangedData (changed=1) filters hidden rows from both register insert and update lists (`0x140018A10`).
If all visible changed lists are empty, SendChangedData sends nothing.
Hidden rows still keep their DB markers and can be written to the store.
Row visibility is `progress-show || for-newbie-care` (`0x140003D60`, `0x140016F70`, `0x140017260`).
Only persistence or progress-show creates a modified marker. for-newbie-care alone does not.

### Manage port (`0x14000E090`, commands 1..=7)

**All non-Hello commands require an assigned channel. This includes Ping.**

| Cmd | Handler | Request | Result |
| --- | --- | --- | --- |
| 1 Hello | `0x14000E500` | `hi` | Decode failure → (3, 2). Version not 4 → (3, 3). Success → `h` (2). |
| 4 Ping | `0x14000E6E0` | `hqq` | `hqq` (5, counter, timestamp) |
| 6 CheckEventProcessingReq | `0x14000E770` | `hqq`: pcdbid, tid | Broadcast `hqqc` (7, pcdbid, **supplied tid**, IsLogined) to managers. |

`IsLogined` checks pcdbid **routing-map membership only** (`0x140009B50`).
It does not compare tid or require a successful Pc load.
The port keeps routing membership separate from loaded Pcs.

## 3. Events (cmd 6)

| Offset | Field |
| --- | --- |
| 0 | size |
| 2 | cmd |
| 4 | q time |
| 12 | q tid |
| 20 | q pcdbid |
| 28 | q hint |
| 36 | u16 eventId (≤ 0x770) |
| 38 | u16 clusterId |
| 40 | Class fields |

The body after size must be at least 38 bytes.
Fields follow the native static Fields array.
A descriptor has 32 bytes: name, type, index, offset, countOffset, size, extra.
Types: 0 u8, 1 u16, 2 u32, 3 u64, 4 16 bytes, 5 counted UTF-16.
Short fields fail decoding. Unknown or internal-only IDs fail wire parsing.

All **55 metadata classes** remain in `src/schema.rs`.
Native command 6 accepts **48 classes** (`0x140013770`).
It excludes **1013, 1014, 1803–1807**.
Internal code can still construct those events without the wire parser.
1805 is DaemonStart. 1806 is DaemonStop (`0x140015680`).
PcLogin.job is u8. CompleteTakeReward.job is u16 on the wire.

### Processing

Native Post maps pcdbid to an event thread and reference count (`0x140009960`).
Gameplay, item, and debug events need a Pc. Login/logout and 1803–1807 do not (`0x140015680`).
Routing can discard an unknown-Pc event before processing (`0x140008950`).
With a Pc, tid must match, except for login.
The port uses one logical event thread with one state lock.

| Event | Action |
| --- | --- |
| 1801 Login | Load registers/completed state. Send version-reset notices, then SendAllData. Update profile registers. Relogin keeps rows and refreshes the profile/session. |
| 1802 Logout | updatePc(pc, 0), then remove the Pc. |
| 1013 (internal) | Require normalized increment-from-client, then AddRegister. |
| 1014 (internal) | Resolve requested (id, step, Pc job), then job-none. Require complete-from-client. Check that exact record (`0x14001B070`). |
| 1108 | Resolve (id, step, request job), then job-none. Apply the reward gates below. Missing-record replies bypass transaction-0 suppression (`0x140019640`). |
| 1015 / 1006 / 1023 / 1028 | Update profile and profile registers. Grade selectors use the named attributes below. |
| 1803 / 1807 | No state change. |
| 1804 (internal) | disconnected_all=false: flush/remove one cluster's Pcs. true: flush/remove all Pcs on the target thread (`0x140019990`, `0x140008F80`, `0x140008D40`). The port has one logical thread. Ordinary socket closure uses false. |
| 1901 / 1902 | AddRegister / SetRegister. |
| 1903 | DeleteCompleted, remove reward, recompute score, SendAllData. |
| 1904 | DeleteCompletedAll + DeleteRegistersAll, SendAllData. |
| Other gameplay | Parsed. **Phase 2:** event-to-register rules are not yet ported. |

## 4. Rules

Tables in data-path: `AchievementRegisterData*.xml` and `AchievementData*.xml`.
Plain XML and LMXBOSLB are supported. Related client-file research stays in `notes/client-datafiles.md`.
Register key: (id, job). Achievement key: (id, step, job).

### Native normalization

| Rule | Native source |
| --- | --- |
| increment-from-client is true only for client-only (subtype 56). Clear explicit flags on other types. | `0x140010AF0` |
| Clear persistence for duel-grade (19), inventory-size through pc-mastery-level (51–55), and party-battle-grade (59). | `0x140010AF0` |
| A deprecated later step deprecates earlier exact-job steps. Skip deprecated records when deriving links and scores. | `0x140010600` |
| Inherit each active achievement's progress-show on its referenced registers. | `0x140010600` |
| Store exact (id, step, job) links on the referenced register row. More than eight links fails loading. | `0x140010600` |
| score_sum is the cumulative score of nondeprecated steps of the same id/job. Native stores a signed 16-bit sum. | `0x140010600` |
| Missing register-type-N defaults to none=0, which fails the condition. above=1 means ≥. below=2 means ≤. bits-on=3 tests low N bits. | Metadata `0x140094D28`, enum `0x1400E3550`, evaluator `0x140017DD0` |
| Load progress-step-register-clear-1..5. The five flags start at record +101. | Metadata `0x140094E30`, completion `0x140017DD0` |
| Reward type at +146 is nonzero for an item reward or positive game-cash reward. It is not a reset flag. | `0x140010600` |
| Register version defaults to 1. | Metadata `0x140091378` |

| Selector | Native enum names | Sources |
| --- | --- | --- |
| duel-type | none=0, solo=1, team=2 | Metadata `0x140092540`, enum `0x1400E3418`, consumer `0x140019500` |
| party-battle-type | none=0, occupation-war=1, capture-the-flag=2, lead-the-ball=3 | Metadata `0x140093EA0`, enum `0x1400E34F8`, consumer `0x1400195B0` |

Do not use the old numeric sub-type attribute for these selectors.

The full job-name table comes from `0x1400DDEA0`:

| Value | Name | Value | Name |
| ---: | --- | ---: | --- |
| 0 | job-none | 10 | 기권사 |
| 1 | 검사 | 11 | 투사 |
| 2 | 권사 | 12 | 궁사 |
| 3 | 기공사 | 13 | 창술사 |
| 4 | 격사 | 14 | 뇌전술사 |
| 5 | 역사 | 15 | pc-max |
| 6 | 소환사 | 16 | 소환수-루키 |
| 7 | 암살자 | 17 | 소환수-striker |
| 8 | 귀검사 | 18 | 소환수-defender |
| 9 | 주술사 | 19 | 소환수-controller |

### Register operations and completion

AddRegister selects the Pc-job row, then job-none.
It clamps to max-value when nonzero, then to at least 0 (`0x140016F70`).
SetRegister and BitwiseOr do not clamp (`0x140017260`).
A new tracked row gets INSERT=1. A changed tracked row gets UPDATE=2 unless already INSERT.
Tracked means persistence or inherited progress-show. Unchanged values keep their marker.
Every operation checks linked completion, even if the value did not change.

The register completion overload visits each exact linked key (`0x140017D60`).
The record overload checks **one supplied record**, not a next-step loop (`0x140017DD0`):

1. Stop for a deprecated record or a step not above the stored step.
2. Unless complete-from-client, require all conditions to hold.
3. Store the requested step and current time. Keep INSERT, or mark UPDATE.
4. Replace the old cumulative score with the selected record's score_sum.
5. For each flagged condition, call SetRegister(0). This can check linked completion again.
6. Send cmd 11. isLast tests the next exact record with the same job.

One register operation can complete several linked records.
Client completion can complete a later requested step without completing earlier steps.
Pc load also uses each selected completed record's score_sum (`0x140016CA0`).

### availableTakeReward (`0x140017B80`)

| Order | Check | Reason |
| --- | --- | --- |
| 1 | Completed step is 0 and reward marker is absent (-1) | 5 |
| 2 | Requested record is deprecated | 7 |
| 3 | complete-from-client, or first condition reference absent | 5 |
| 4 | Completed step below requested step | 9 |
| 5 | Reward marker at or above requested step | 8 |
| 6 | An earlier unclaimed record is missing | 5 |
| 7 | An earlier unclaimed record has nonzero reward type at +146 | 9 |
| 8 | All checks pass; store requested reward step with marker 2 | 0 |

Scan earlier steps backward from requested step - 1 to the reward marker, excluding step 0.
Resolve each step with the request job, then job-none.
At each step, check record existence before reward type.
The requested record does not need a reward.

## 5. Store and updatePc

The Store trait and MemoryStore carry register id, version, and value.
On load, resolve the register with the Pc job, then job-none. Skip unknown records.
A version mismatch resets value to 0, marks UPDATE, and sends cmd 12 with old/new versions and values (`0x140003D60`).
A matching version keeps the value and starts CLEAN.

Native updatePc attempts each DB group independently (`0x140008B20`).
Only successful writes clear their group's markers. Failed groups retry on the next cycle.
Insert failure must not suppress update calls.
A later failure must not restore markers for an earlier successful group.
The periodic path also sends visible changed data.

| Native SQL procedure | Arguments / result | Address |
| --- | --- | --- |
| usp_LoadRegisters | pcdbid → id, version, value | `0x140003D60` |
| usp_LoadCompleted | pcdbid | `0x1400044F0` |
| usp_InsertRegister | pcdbid, id, version, value | `0x140004D40` |
| usp_UpdateRegister | pcdbid, id, version, value | `0x140005260` |
| usp_InsertCompleted | pcdbid, id, step, completed | `0x140005760` |
| usp_UpdateCompleted | pcdbid, id, step, completed | `0x140005CD0` |
| usp_DeleteCompleted | pcdbid, id | `0x140006240` |
| usp_UpdateTakeStepReward | pcdbid, id, step | `0x140006550` |
| usp_DeleteRegisterAll | pcdbid | `0x1400069A0` |
| usp_DeleteCompletedAll | pcdbid | `0x140006C30` |

These names and arguments come from prepared SQL and bindings, not only logs.
Register bindings: pcdbid 8 bytes, id 2 bytes, version 2 bytes, value 8 bytes.

## 6. Scope and open items

| Area | State |
| --- | --- |
| Config, IP list, game/manager dispatch | Implemented |
| Metadata / wire events | 55 metadata classes; 48 command-6 classes |
| Client completion, register reset, rewards, rule normalization | Implemented for the reviewed build-348 paths |
| Register visibility, version reset, independent DB groups | Implemented |
| Ping timeout and cluster disconnect branches | Implemented; one logical event thread |
| Store | MemoryStore only. Durable SQL adapter is a separate job. |
| Gameplay event → register rules (kill, quest, item, …) | Phase 2; not part of this fix job |
| Alert mail, dumps, native event-thread distribution | Not ported |
| ❓ Stored procedure bodies | Calls and bindings are verified. SQL implementation and durable recovery remain unverified here. |

The review resolved all former ❓ items for the requested fix paths.
