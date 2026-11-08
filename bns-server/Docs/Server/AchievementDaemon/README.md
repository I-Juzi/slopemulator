# AchievementDaemon (build 348)

Source binary: `AchievementDaemon.exe`, build path `AchievementDaemon-Common-0.0.348`.
Rust port: `Program/Server/AchievementDaemon` (crate `achievementdaemon`).
Facts marked ❓ are not verified in the binary.

```
GameDaemon ──(game port 22200)──► AchievementDaemon ──► DB (stored procedures)
   ▲  cmd 1 Hello, 4 Ping,            │  Pc map, registers,
   │  6 EventOccuredNotify,           │  completed steps, rewards
   │  8 CompleteFromClientNotify      │
   └── cmd 7 Data, 9/10 TakeReward, 11 CompletedNotify
Manager ──(manage port 32200)──► cmd 1 Hello, 4 Ping, 6 CheckEventProcessingReq
```

## 1. Config

File `config/achievementd.config.xml`, `<config module='achievement-daemon'>`, one `<option name value/>` per option.

| Option | Default | Use in the port |
| --- | --- | --- |
| `internal-port` | 22200 | Game port |
| `manage-port` | 32200 | Manage port |
| `ping-enable` | yes | Parsed. ❓ Ping timeout is not enforced yet. |
| `check-ping-time` | 30000 | Parsed |
| `disconnect-when-ping-timeout` | yes | Parsed |
| `max-pc-count` | 400000 | A new login above this count is dropped |
| `update-cycle` | 3000 ms | Timer that runs updatePc for each Pc |
| `log-register-change` | no | Parsed |
| `data-path` | — | Folder of the rule tables |
| `root-path`, `log-path`, `config-path`, `cert`, `alert-send-address`, `log-codepage`, `dump-type`, `worker-thread`, `event-thread` | — | Kept in `options`, not used |

`config/achievementd-iplist.xml`: `<cluster-list><cluster ip='a.b.c.d' cluster-id='n'/>…`.
An arena cluster id is arena id + 10000.

## 2. Wire

Frames: i16 size (4..=4000), i16 cmd, body. Format codes as in bloodlib.

### Game port (PacketHandler @0x14000C1D0)

- Size must be ≥ 4. The command must be 1..=12.
- Cmd 1 works without a cluster. Other commands without a cluster are logged and ignored.
- The handler always returns 1 (the session stays open).

| Cmd | Handler | Request | Result |
| --- | --- | --- | --- |
| 1 Hello | @0x14000D280 | `hiih` cmd, packetVersion, eventVersion, clusterId | Decode fail → `hh` (3, 2). Versions not 10 / 33 → (3, 3). (IP, clusterId) not listed → (3, 4). Channel fail → (3, 2). OK → `hh` (2, clusterId), then event 1803. |
| 4 Ping | @0x14000D6B0 | `hqh` cmd, count, clusterId | `hqh` (5, count, clusterId) |
| 6 EventOccuredNotify | @0x14000DE90 | Event frame (section 3) | Post. No reply. |
| 8 CompleteFromClientNotify | @0x14000D7E0 | `hqqqqhChh;Chh;` cmd, eventTime, tid, pcdbid, hint, clusterId, rows (register_id, value), rows (achievement_id, step) | Each first-group row → event 1013. Each second-group row → event 1014. |

Port note: "channel fail" is modelled as "cluster id already has a session".

Daemon → GameDaemon:

| Cmd | Format | Fields |
| --- | --- | --- |
| 7 Data | `hiqqqihcChq;Chhq;Chh;` (fragments, 8-byte header) | tid, pcdbid, hint, completedScore, clusterId, changed (0 SendAllData, 1 SendChangedData), registers (id, value), completed (id, step, time), rewards (id, rewardStep) |
| 9 TakeRewardOk | `hqqhh` | transactionId, pcdbid, rewardStep, achievementId |
| 10 TakeRewardFail | `hEqqhh` | reason, transactionId, pcdbid, rewardStep, achievementId |
| 11 CompletedNotify | `hsqqqhhhc` | pcname, pcdbid, pcid, tid, achievementId, oldStep, newStep, isLastStep |

Source of cmd 9/10: `Pc::SendCompleteTakeReward`. The 4th argument is the reward step (the log string names `rewardStep`).

### Manage port (PacketHandler @0x14000E090, cmd 1..=7)

| Cmd | Handler | Request | Result |
| --- | --- | --- | --- |
| 1 Hello | @0x14000E500 | `hi` | Decode fail → (3, 2). Version ≠ 4 → (3, 3). OK → `h` (2). |
| 4 Ping | @0x14000E6E0 | `hqq` | `hqq` (5, counter, timestamp) |
| 6 CheckEventProcessingReq | @0x14000E770 | `hqq` pcdbid, tid | `hqqc` (7, pcdbid, tid, IsLogined) to every manager. ❓ IsLogined = the Pc exists with this tid. |

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
| 40 | fields |

- The body after the size must be ≥ 38 bytes. Else no event.
- Fields follow the static `Fields` array of the event class. Descriptor (32 bytes): name, type, index, offset, countOffset, size, extra.
- Types: 0 u8, 1 u16, 2 u32, 3 u64, 4 16 bytes, 5 wstr (u16 count + UTF-16 units).
- A short field → decompose failure → no event. An unknown id → "invalid event" log.
- All 55 event classes are in `src/schema.rs` (generated from the IDA dump).

| Ids | Events |
| --- | --- |
| 1001–1035 | Gameplay: KillBossNpc, KillNpc, KillPc, EquipItem, UseGrocery, ExpandInventory, Quest*, UseSkillToNpc, DefendNpcSkill, IncrementRegisterFromClient (1013), CompleteRequestFromClient (1014), PcLevelUp, PcExhaustion, PcDead, ExchangeFactionScore, DetachPostAttachment, Teleport, ManipulateEnv2, DuelResult, HelpRestoration, HelpResurrect, GetContributionScore, GiveFactionScoreToRefiner, PartyBattleResult, DuelBotChallengeAttractionResult, GetFieldPlayPoint, SkillTraningSubjectComplete, GetFactionScore, TakeCraft, LeadTheBallGoalIn, AcquireFish |
| 1100–1109 | Item: GrowthItem, AwakeningItem, AttachEquipGemPiece, DecomposeItem, AttachGemToWeapon, TransformItem, ZeroDurability, RepairItem, CompleteTakeReward (1108), DisposeItemBuyPrice |
| 1801–1807 | PcLogin, PcLogout, ClusterConnected, ClusterDisconnected, DaemonStart ❓, DaemonStop ❓, PcIsLogined |
| 1901–1904 | DebugCommand AddRegister, SetRegister, DeleteAchievement, DeleteAll |

### Processing

- `EventManager::Post` maps pcdbid → (thread, refcount). Login of an unknown Pc picks the thread with the fewest Pcs. Other events of an unknown Pc are dropped ("pc not found").
- `processEvent`: with a Pc, the event runs only if the tid matches or the event is 1801 ("invalid tid"). Without a Pc, events with the needs-pc flag are dropped.
- Port: one lock around all Pcs instead of event threads. The order per Pc is the same.

| Event | Action |
| --- | --- |
| 1801 Login | New Pc: `Pc::Load` (LoadRegisters, LoadCompleted; score = sum of step scores). Existing Pc: relogin updates tid, cluster and profile. Then SendAllData. Then updatePcLevel, updateInventory, updateDuelGrade, updatePartyBattle. |
| 1802 Logout | updatePc(pc, 0), then remove the Pc. |
| 1013 | Register must be increment-from-client → AddRegister. |
| 1014 | Record (id, step) must be complete-from-client → checkComplete. |
| 1108 | Look up (id, step, job), then (id, step, 0). Not found → reason 6. Else availableTakeReward. Reply only when transaction_id ≠ 0. |
| 1015 / 1006 / 1023 / 1028 | Update the profile, then the profile registers (as on login). |
| 1803 / 1807 | Logged only. |
| 1804 | Port: flush and drop the Pcs of the cluster. ❓ |
| 1901 / 1902 | AddRegister / SetRegister |
| 1903 | DB DeleteCompleted, remove the reward, recompute score, SendAllData |
| 1904 | DB DeleteCompletedAll + DeleteRegistersAll, SendAllData |
| Other gameplay | Parsed and accepted. **Phase 2:** the register rules per register `type` are not ported. |

## 4. Rules

Tables in `data-path`: `AchievementRegisterData*.xml` and `AchievementData*.xml`. Plain XML or LMXBOSLB (the GameDaemon copies are LMXBOSLB; facts in `notes/client-datafiles.md`).

- `achievement-register` (v0.42, 1067 rows): key (id, job). Used: `id`, `alias`, `type`, `job`, `max-value`, `persistant`, `progress-show` ❓, `increment-from-client` ❓ (the port also sets it for `type="client-only"`).
- `achievement` (v0.19, 1558 rows): key (id, step, job). Used: `current-step-score`, `register-ref-1..5` (register alias), `register-type-N` (`above`, `bits-on`, `below` ❓), `register-value-N`, `deprecated`, `complete-from-client`, `step-complete-reward-*` (has a reward).
- Job names map to the Pc job byte in client enum order (검사 = 1 … 궁사 = 12). ❓
- A register links to at most 8 achievements.

### Register operations

| Operation | Rule |
| --- | --- |
| AddRegister | Row for the Pc job or job-none. Clamp to max-value (when ≠ 0) and to ≥ 0. |
| SetRegister, BitwiseOr | No clamp. |
| Marker | New row → 1 (insert) only if persistant or progress-show, else 0. Changed row → 2 unless it is 1. |
| After | checkComplete for each linked achievement. |

### checkComplete

1. Take the record of the next step (completed step + 1).
2. Stop if it is deprecated.
3. Unless complete-from-client, all conditions must hold. Op 1 value ≥ v. Op 2 value ≤ v. Op 3 low v bits all set.
4. Store the step (time = now), marker 1 or 2, add the step score.
5. Send cmd 11 with isLast = no record for the next step.
6. Port: repeat with the next step. ❓ The binary may stop after one step.
7. ❓ Not ported: reset of flagged registers after completion.

### availableTakeReward

| Reason | Condition (in order) |
| --- | --- |
| 5 | The step has no reward |
| 7 | Deprecated |
| 5 | complete-from-client, or no conditions |
| 9 | Completed step < reward step |
| 8 | Reward step already taken |
| 9 | ❓ Not ported: a previous step has the flag at +146 |
| 0 | OK → store reward step, marker 2 |

## 5. Store (updatePc)

| Call | When |
| --- | --- |
| LoadRegisters, LoadCompleted | Login of a new Pc |
| InsertRegisters / UpdateRegisters | Rows with marker 1 / 2 |
| InsertCompleted / UpdateCompleted | Rows with marker 1 / 2 |
| TakeStepReward | Reward rows with a marker |
| DeleteCompleted, DeleteCompletedAll, DeleteRegistersAll | Debug 1903 / 1904 |

- Markers stay set when a call fails. The next cycle tries again.
- With clientUpdate = 1 (timer), SendChangedData goes to the cluster.
- The port has a `Store` trait and a `MemoryStore`. ❓ Procedure names and arguments are from log strings, not from SQL.

## 6. Status

| Area | State |
| --- | --- |
| Config, iplist | Ported |
| Game port 1/4/6/8, manage port 1/4/6 | Ported |
| Event parser (55 classes) | Ported |
| Login, logout, relogin, profile registers | Ported |
| Register ops, checkComplete, take reward, debug | Ported |
| Store trait, updatePc timer | Ported (memory store only) |
| Gameplay event → register rules (kill, quest, item, …) | Phase 2 |
| Ping timeout, alert mail, dumps | Not ported |
