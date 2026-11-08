# 21 — Quest progress handlers (GameDaemon 381.8867)

Source: `ida/delegate/gamed/GameDaemon.exe.i64` (build 381.8867), read through
IDA MCP. Handler addresses match `parity-ledger.tsv`, column `handler381`.
All addresses in this document apply to this build.

Descriptor sources: `idc.get_strlit_contents`. Descriptor reads use
`ida_nalt.STRTYPE_C`. Wide log reads use `ida_nalt.STRTYPE_C_16`.
`h` is the command header or a two-byte field. `Q` and `c` are one byte.
`i` is four bytes. `q` is eight bytes. `s` is counted UTF-16.
`C` is a two-byte list count. The count reader `0x14069DA00` returns a short.
The callers sign-extend it. Missing counts take the negative-count path.
Reply bodies below exclude the command header. Multi-byte fields use
little-endian order. Brackets show fields, not individual bytes.

Coverage: 1448, 1450, 1452, 1454, 1458, 1465, 1467, 1469, 1472, 1475, 1478.
These are entry-function ports. A queued task is not a completed operation.
The anti-bot helper `0x14060E280` was not read or called.

## Shared fields

| Field | Native source | Rust |
| --- | --- | --- |
| Zone | PC qword +2224 | `Pc::zone` |
| Caster | PC qword +40 | `Pc::object_id` |
| Native kind | PC WORD +46; 1 = PC | `Pc::native_kind` |
| Job | PC byte +100; 6 = summoner | `Pc::job` |
| User available to the name service | PC qword +20696 nonzero and byte +20752 zero | `Pc::user_present` and `Pc::party_user_blocked` |
| Additional-reward feature | rule config byte +2104 | `QuestProgressCheck::AdditionalRewardEnabled` |
| Reward item record | data manager qword slot 1455, virtual +168 | `QuestProgressCheck::RewardItem` |
| Cost group record | data manager qword slot 591, virtual +168 | `QuestProgressCheck::CostGroup` |
| Reset restriction | rule byte +2089 nonzero and PC short +12544 is 77 or 177 | `QuestProgressCheck::ResetRestricted` |
| Summoned-name service | daemon qword +2728 → qword +48 | `QuestProgressCheck::SummonedNameServicePresent` / `SummonedNameAllowed` |
| General name filter | `0x14012F9F0`, daemon +2800, rule +172..+200 | `QuestProgressCheck::SummonedNameFilter` |
| Task state | WORD +136 kind; qword +128 caster; byte +138 attached | `SkillTask`, `SkillTaskKind::QuestProgress` |
| Zone attach / queue | `0x14025DB30` / `0x140254FD0` | `skills::queue` → `Services::queue_skill` |
| Opaque server state | config, record tables, user/name service | `Services::quest_progress`; default `Gate::Unknown`; mock `MemoryServices::quest_progress` |

No entry in this set checks arena form B or PC +13768. Do not add these gates.
No entry looks up a second PC. Only 1469 and 1475 explicitly reject a native
kind other than 1. The common helper `0x14069BF60` records a task reference;
it is not an extra PC-kind gate in these entries.

Position lists use wire `Qh`. Native records use stride 4: byte +0, short +2.
Item orders use wire `Qhh`. Native records use stride 6: byte +0, shorts +2/+4.
Duplicate tests compare only the byte and the first short. They ignore the
last item-order short. The Rust field name `count` for that last short is
❓ a domain interpretation. The port retains its bits.

## 1448 — G_AcquireQuestReqCmd — partial

Handler `0x140382F30`. Descriptor `0x140BD5858`.

Wire: `hhccqhCQh;CQhh;`: quest short, case byte, selector byte, target qword,
value short, brand-position count/list, item-order count/list. Each list has
its own count. ❓ Selector, target and value domain names need task execution.

1. Decode the five fixed fields. Failure → reply 1449 `hh` [955].
2. Read brand count. Negative or > 16 → reply 1449 `hh` [955]. Read each `Qh`. A failed read → the same reply.
3. Duplicate brand position → reply 1449 `hh` [965]. Log source `0x140BD61A0` says "invalid brand pos".
4. Read item count. Negative or > 16 → reply **1451** `hh` [1001]. A failed `Qhh` read → the same reply. This is the native command, not 1449.
5. Duplicate item position → reply **1451** `hh` [1013]. Different final shorts do not make a duplicate position valid.
6. No zone → reply 1449 `hh` [2], then log "cannot find zone" (`0x140BD5EB0`).
7. Allocate kind 93, size 0x188, vtable `0x140C71898`. Attach zone/caster. Set short +144 = quest, byte +146 = case, byte +147 = selector, qword +152 = target, short +160 = value. Set DWORD +168 = item count; copy stride-6 orders to +172. Set DWORD +268 = brand count; copy stride-4 positions to +272. No PC-kind test.
8. Queue failure → reply 1449 `hh` [2]. Queue success → no reply here.

❓ Missing: task execution, quest/item tables, rewards, success reply and
persistence. Zero counts and trailing bytes are accepted by the entry port.

## 1450 — G_ProgressMissionReqCmd — partial

Handler `0x1403836B0`. Descriptor `0x140BD66B0`.

Wire: `hhcccqhCQh;CQh;CQhh;cqq`: quest short, mission byte, case byte,
selector byte, target qword, value short; brand `Qh` list; equip-gem `Qh`
list; item `Qhh` list; reward index byte, reward item key qword, cost group
key qword. Each list starts with a two-byte count.
❓ Selector, target and value domain names need task execution.

All failures below use reply 1451 `hh` with the indicated reason.

1. Decode the six fixed fields. Failure → [1001].
2. Brand count negative or > 16 → [1001]. A failed brand read → [1001].
3. Duplicate brand position → [1013].
4. Equip-gem count is not 0 or 8 → [1001]. A failed equip-gem read → [1001].
5. Duplicate equip-gem position → [1013].
6. Item count negative or > 16 → [1001]. A failed item read → [1001].
7. Duplicate item position → [1013].
8. Decode the reward byte and two keys. Failure → [1001].
9. Feature byte +2104 = 0 and any reward field nonzero → [1001]. If all three are zero, continue.
10. Feature enabled and unsigned reward index > 4 → [1001]. Index 0 skips the key checks, even when keys are nonzero.
11. Feature enabled and index nonzero, with either key zero → [1001].
12. Look up reward item in slot 1455, virtual +168. No record → [1001]. Disassembly at the call near `0x140383E30` keeps the item key in RDX. The decompiler omits that argument because the virtual signature is incomplete.
13. Look up cost group in slot 591, virtual +168, with the cost group key. No record → [1001]. Logs `0x140BD6B00` and `0x140BD6C70` identify the two records.
14. No zone → [2], then log "cannot find zone" (`0x140BD6DE0`). The record checks occur before this gate.
15. Allocate kind 94 through `0x140596970`, size 0x188, vtable `0x140CAF170`. Attach zone/caster. No PC-kind test. Fields: short +144 quest, bytes +146 mission / +147 case / +148 selector, qword +152 target, short +160 value. DWORD +168 and records +172 hold items. DWORD +268 and records +272 hold brands. DWORD +336 and records +340 hold equip-gems. Byte +372 holds reward index. Qwords +376/+384 hold item/cost group keys.
16. Queue failure → [2]. Queue success → no reply here.

❓ Missing: record table contents, task execution, mission rules, success reply,
rewards and persistence. Unknown feature or lookup state returns `Unsupported`.
The port calls each opaque check only when the native branch reaches it.

## 1452 — G_ProgressMissionAllReqCmd — partial

Handler `0x140384210`. Descriptor `0x140BD7620`.

Wire: `hChcccq;`: count, then quest short, mission byte, case byte, selector
byte, target qword for each element. ❓ The last two field names need the task.

1. Count negative or > 8 → reply 1453 `hEhcc` [2, 0, 0, 0].
2. A failed element read → the same reply.
3. No zone → the same reply, then log "cannot find zone" (`0x140BD7770`).
4. Allocate kind 94 through `0x140596970`, size 0x100, vtable `0x140C6EE40`. No PC-kind test. Attach zone/caster. DWORD +144 = count. Copy separate arrays: shorts +148 quest, bytes +164 mission, +172 case, +180 selector, qwords +192 target.
5. Queue failure → reply 1453 `hEhcc` [2, 0, 0, 0]. Raw body: `02 00 00 00 00 00`.
6. Queue success → no reply here. Zero count is valid. The entry does not reject duplicate quests.

❓ Missing: task execution, per-mission checks, success replies and persistence.

## 1454 — G_ProgressMissionByAcquireSummonedReqCmd — partial

Handler `0x1403B6AB0`. Descriptor `0x140C120B8`.
Initializer `0x140597940`. Logs identify `ProgressMissionByAcquireSummonedTask`.

Wire: `hhccqs`: quest short, mission byte, case byte, target qword, summoned
name. The name capacity is 21 UTF-16 units, including the terminal NUL.
The maximum wire count is 20. The name storage starts at zero.
❓ Target field meaning needs task execution.

1. Decode. The entry ignores the result. The fixed locals have no initial values.
2. No zone → log (`0x140C120C0`), reply 1455 `hh` [1].
3. Allocate kind 94 through `0x140596970`, size 0xD0, vtable `0x140BC2AF0`. Constructor clears the name storage at +160..+201.
4. Call initializer `0x140597940`. Null zone, PC or name pointer → crash log, reply 1455 `hh` [2]. These null arguments are not reachable from this entry after its zone gate.
5. PC job byte +100 ≠ 6 → log (`0x140D81860`), reply 1455 `hh` [1060]. No PC-kind test.
6. If daemon +2728 and its qword +48 name service are present, get the user at PC +20696. Null user or PC byte +20752 nonzero → log (`0x140D81790`), reply 1455 `hh` [2]. If the service is absent, skip the user and service checks.
7. With the service present, call its virtual +0 with user +40, user DWORD +104 and name. False → log (`0x140D81300`), reply 1455 `hh` [1062]. Release the user reference.
8. Call `0x14012F9F0` with the name, daemon +2800 and rule fields +172, +176, +180, +192, +193, +194, +200. Nonzero → log, reply 1455 `hh` [1062]. Rust uses `Pass` for an accepted name, not for the native nonzero result.
9. Attach zone/caster. Set short +144 = quest, bytes +146/+147 = mission/case, qword +152 = target. Copy the NUL-prefix name to UTF-16 +160. The copy excludes the NUL; the constructor already cleared storage.
10. Initializer failure sends no second reply. Successful initialization followed by queue failure → log (`0x140C127F0`), reply 1455 `hh` [1]. Queue success → no reply here.

❓ Missing: ignored-decode failure behavior, name service internals, name filter
internals, task execution, success reply and persistence. Rust returns
`Unsupported` when malformed input needs unset fields. The no-zone and job
failure paths remain available on malformed input. Unknown name-service or
filter state is not approval.

## 1458 — G_DisposeQuestReqCmd — partial

Handler `0x140384A70`. Descriptor `0x140BD7984`.

Wire: `hCh;`: count, then quest shorts.

1. Count < 0 or count ≥ 35 → reply 1459 `hh` [2]. The accepted maximum is **34**, not 35.
2. A failed quest read → reply 1459 `hh` [2].
3. No zone → reply 1459 `hh` [2], then log (`0x140BD7AE0`).
4. Allocate kind 92, size 0xE0, vtable `0x140CA98A0`. No PC-kind test. Attach zone/caster. Copy shorts to +144. Set DWORD +216 = count.
5. Queue failure → reply 1459 `hh` [2]. Queue success → no reply here. Zero count and duplicate quest IDs are accepted by the entry.

❓ Missing: task execution, quest disposal rules, success reply and persistence.

## 1465 — G_MissionStepRollbackReqCmd — partial

Handler `0x140384DD0`. Descriptor `0x140BD7810`.

Wire: `hhcc`: quest short, mission byte, step byte. ❓ Byte domain names need
task execution. The decode result is ignored. Locals have no initial values.

1. Decode the three fields.
2. No zone → reply 1466 `hh` [2], then log (`0x140BD7820`).
3. Allocate kind 95, size 0x98, vtable `0x140D16F30`. No PC-kind test. Attach zone/caster. Set short +144 = quest, byte +146 = mission, byte +147 = step.
4. Queue failure → reply 1466 `hh` [2]. Queue success → no reply here.

❓ Missing: unset decode-field behavior, task execution, rollback rules, success
reply and persistence. Rust returns `Unsupported` before it uses unset fields.

## 1467 — G_MissionStepTimeExpiredReqCmd — partial

Handler `0x140384F90`. Descriptor `0x140AEAB1C`.

Wire: `hhc`: quest short, mission byte. The decode result is ignored.
Locals have no initial values.

1. Decode the two fields.
2. No zone → reply 1468 `hh` [2], then log (`0x140BD78D0`).
3. Allocate kind 96, size 0x98, vtable `0x140D12948`. No PC-kind test. Attach zone/caster. Set short +144 = quest, byte +146 = mission.
4. Queue failure → reply 1468 `hh` [2], then log (`0x140BD7E20`). Queue success → no reply here.

❓ Missing: unset decode-field behavior, task execution, timer checks, success
reply and persistence. Rust returns `Unsupported` before it uses unset fields.

## 1469 — G_ResetQuestCompletionTimeReqCmd — partial

Handler `0x140385160`. Descriptor `0x140BCFE58`.

Wire: `hhChh;`: quest short, count, then pairs of shorts.
❓ Pair domain names need task execution.

1. Fixed short decode failure → reply 1471 `hh` [1091].
2. Count < 0 or count ≥ 16 → reply 1471 `hh` [1092]. A missing count uses this path. The accepted maximum is **15**.
3. A failed pair read → reply 1471 `hh` [1091].
4. No zone → log (`0x140BD7B80`), reply 1471 `hh` [2].
5. Rule byte +2089 nonzero and PC short +12544 equal to 77 or 177 → log "not allowed zone attraction" (`0x140BD7C40`), reply 1471 `hh` [1098]. This check precedes the native-kind gate.
6. Allocate kind 134, size 0xD8, vtable `0x140C7D3E8`.
7. Native kind ≠ 1 → log (`0x140D81400`), reply 1471 `hh` [2].
8. Attach zone/caster. Set short +144 = quest, DWORD +148 = count. Copy pairs to +152, stride 4.
9. Queue failure → log (`0x140BD82C0`), reply 1471 `hh` [2]. Queue success → no reply here.

❓ Missing: opaque attraction state and config, task execution, completion-time
rules, success reply and persistence. Unknown restriction state returns
`Unsupported`. Zero count is valid. The entry does not reject duplicate pairs.

## 1472 — G_AcquireRenewalQuestReqCmd — partial

Handler `0x1403855C0`. No request descriptor. The entry reads no body fields.

1. Always reply 1474 `hh` [2].
2. Log "disabled" (`0x140BD83A0`). Return true. No task, zone check or state change.

❓ Missing: logger side effects. All entry branches were read. Rust records a
static log marker. The disabled entry does not acquire a quest.

## 1475 — G_ChallengeListAcquireQuestReqCmd — partial

Handler `0x140385820`. Descriptor `0x140BD81B0`.

Wire: `hch`: sub type byte, quest short. Locals start at −1 and 0.
The decode result is ignored. A decoded sub type is retained when the short
read fails. In that case the quest stays zero.

1. Decode the fields.
2. Quest = 0 → reply 1477 `hh` [1111], then log (`0x140BD81C0`). This check precedes the zone gate.
3. No zone → reply 1477 `hh` [2], then log (`0x140BD8800`).
4. Allocate kind 222, size 0x98, vtable `0x140C61A60`.
5. Native kind ≠ 1 → log (`0x140D81B40`), reply 1477 `hh` [2].
6. Unsigned sub type > 1 → log (`0x140D82430`), reply 1477 `hh` [2].
7. Attach zone/caster. Set byte +144 = sub type, short +146 = quest.
8. Queue failure → reply 1477 `hh` [2], then log (`0x140BD8A10`). Queue success → no reply here.

❓ Missing: task execution, challenge-list checks, success reply and persistence.

## 1478 — G_ProgressSkipMissionStepReqCmd — partial

Handler `0x140385620`. Request descriptor `0x140A3D9D4`.
Reply descriptor `0x1408B88D0`.

Wire: `hhQ`: quest short, skip reason u8. `Q` is **one byte**, not a qword.
The decode result is ignored. Locals have no initial values.

1. Decode the two fields.
2. No zone → call reply 1480 `hEhQ` with reason 2. **The call does not set the quest or skip-reason arguments.** Log (`0x140BD8440`).
3. Allocate kind 180, size 0x98, vtable `0x140CE3258`. No PC-kind test. Attach zone/caster. Set short +144 = quest, byte +146 = skip reason.
4. Queue failure → call reply 1480 `hEhQ` with reason 2. **This call also omits the last two arguments.** Then log (`0x140BD80B0`). Queue success → no reply here.

Disassembly confirms both calls set RCX/RDX/R8/R9 only. The required stack
arguments are not set for the reply. The later log calls set their own
arguments after the reply call. The known body prefix is `02 00`.
❓ The following short and byte depend on stale stack contents. Do not claim
that the body is [2, request quest, request skip reason].

❓ Missing: unset decode fields, exact native failure-body tail, task execution,
skip rules, success reply and persistence. Rust queues valid requests. Rust
returns `Unsupported` on no-zone or queue failure. It emits no fabricated
1480 packet. Queue `Unknown` remains an adapter boundary.

## Rust port status

| Command | Rust | State / boundary |
| --- | --- | --- |
| 1448 | `src/handlers/quest2.rs` | Partial; exact list failures, including reply 1451; task execution boundary |
| 1450 | `src/handlers/quest2.rs` | Partial; ordered list and reward checks; config/table service boundary |
| 1452 | `src/handlers/quest2.rs` | Partial; six-byte failure body; array task boundary |
| 1454 | `src/handlers/quest2.rs` | Partial; job/user/name gates; name services and task execution boundaries |
| 1458 | `src/handlers/quest2.rs` | Partial; maximum 34; task boundary |
| 1465, 1467 | `src/handlers/quest2.rs` | Partial; ignored decode failure is Unsupported; task boundary |
| 1469 | `src/handlers/quest2.rs` | Partial; maximum 15; restriction service and task boundary |
| 1472 | `src/handlers/quest2.rs` | Partial; disabled reply is exact; logger boundary |
| 1475 | `src/handlers/quest2.rs` | Partial; initialized decode defaults; task boundary |
| 1478 | `src/handlers/quest2.rs` | Partial; success queues; unset failure arguments are Unsupported |

`QUEST2_COMMANDS` registers all eleven entries. `Services::quest_progress`
returns `Gate::Unknown` by default. `MemoryQuestProgress` records ordered
checks. Its default does not approve an opaque operation. Queue allocation
and task execution remain adapter boundaries, as in docs 19 and 20.
`Gate::Fail` uses the native failure plan. `Gate::Unknown` returns
`Unsupported`. A successful queue retains the task fields and the original
body through `StateChange::QueuedRequest`.
