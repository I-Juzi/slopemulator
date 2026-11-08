# 20 — Quest, discovery and survey handlers (GameDaemon 381.8867)

Source: `ida/delegate/gamed/GameDaemon.exe.i64` (build 381.8867), read through
IDA MCP. Handler table `0x1407D9F80`. Addresses match column `handler381` in
`parity-ledger.tsv`. All addresses below apply to this build.

Descriptor letters: `h` header or i16, `Q` u8, `c` i8, `i` i32, `q` u64,
`s` counted UTF-16, `E` u16 reason. `C` is a two-byte list count.
Reply bodies below exclude the command header. Numbers in brackets are fields,
not raw bytes. Multi-byte fields use little-endian order.

Descriptor sources: `idc.get_strlit_contents`, with `STRTYPE_C` for descriptors
and `ida_nalt.STRTYPE_C_16` for wide logs. Shared descriptors: `hh` at
`0x1408B8848`, `hEc` at `0x1408B8860`, `h` at `0x1408CFBE8`.
The count reader `0x14069DA00` reads a WORD. The caller casts it to signed i16.
Decoder `0x1406AA7B0`, case `s`, requires `count + 1 <= capacity`.

Coverage: 86, 89, 93, 96, 398, 401, 404, 1017, 1205, 1420.
These are entry-function ports. A queued task is not a completed operation.
The excluded anti-bot helper `0x14060E280` was not read or called.

## Shared fields

| Field | Source | Rust |
| --- | --- | --- |
| Zone | PC qword +2224 | `Pc::zone` |
| PC object ID | PC qword +40 | `Pc::object_id` |
| Native PC kind | PC WORD +46; 1 = PC | `Pc::native_kind` |
| Arena form B | cfg (daemon +2736) byte +237 and DWORD +248 both nonzero, or PC DWORD +37520 in 2/3/5/6 under CS +37480 | `world.arena || session::arena_blocked(pc)` |
| T2 / trial gate | PC byte +13768 | Existing `Pc::t2_blocked`; these ten entries do not test it |
| Daemon mode (❓ domain name "region") | daemon WORD +2752; 1205 requires 2 | `QuestRules::region` |
| Refresh feature | rule cfg (daemon +2744) byte +1196 | `QuestRules::refresh_enabled` |
| Skip maximum level | rule cfg signed DWORD +40 | `QuestRules::skip_max_level` |
| Skip mastery gate | rule cfg byte +44 | `QuestRules::skip_mastery_enabled` |
| Skip maximum mastery level | rule cfg signed DWORD +48 | `QuestRules::skip_max_mastery` |
| Task common state | kind WORD +136; zone attach `0x14025DB30`; caster qword +128; attached byte +138 = 1 | `SkillTask`, `SkillTaskKind::Quest(QuestTask)` |
| Configuration boundary | server-owned daemon/rule config | `Services::quest_config`; default `QuestConfig::Unknown`; mock `MemoryServices::quest` |
| Queue | `0x140254FD0(zone, task)` | `skills::queue` → `Services::queue_skill` |

The three list tasks, refresh task and replay task have no native-kind gate.
Do not add one. No entry in this set looks up a second PC.

## 86 — G_QuestListReqCmd — partial

Handler `0x140382230`. Log `0x140BD5100`.

Wire: no decode. The entry reads no request fields.

1. No zone → reply 88 `hh` [2], then log "cannot find zone".
2. Allocate task kind 100 (0x90 bytes, vtable `0x140C35A78`). Attach the zone and caster. No request fields.
3. Queue failure → reply 88 `hh` [2]. Disassembly confirms R9D = 2.
4. Queue success → no reply here.

❓ Missing: task execution, list data, success reply and persistence.

## 89 — G_QuestCompleteListReqCmd — partial

Handler `0x140382390`. Log `0x140BD51A0`.

Wire: no decode. The entry reads no request fields.

1. No zone → reply 91 `hh` [2], then log "cannot find zone".
2. Allocate task kind 99 (0x90 bytes, vtable `0x140BC2500`). Attach the zone and caster. No request fields.
3. Queue failure → reply 91 `hh` [2]. Disassembly confirms R9D = 2.
4. Queue success → no reply here.

❓ Missing: task execution, completion list data, success reply and persistence.

## 93 — G_DiscoveryAreaListReqCmd — partial

Handler `0x1403824F0`. Log `0x140BD5250`.

Wire: no decode. The entry reads no request fields.

1. No zone → reply 95 `hh` [2], then log "cannot find zone".
2. Allocate task kind 29 (0x90 bytes, vtable `0x140C3FC10`). Attach the zone and caster. No request fields.
3. Queue failure → reply 95 `hh` [2]. Disassembly confirms R9D = 2.
4. Queue success → no reply here.

❓ Missing: task execution, discovery list data, success reply and persistence.

## 96 — G_AddDiscoveryAreaReqCmd — partial

Handler `0x140382650`. Log `0x140BD5300`.

Wire: `hh`: area i16. The decode result is ignored. The local short has no
initial value. Rust does not invent a value after a short-body decode.

1. Decode the area.
2. No zone → reply 98 `hh` [2], then log "cannot find zone".
3. Allocate task kind 30 (0x98 bytes, vtable `0x140C3D180`).
4. Native kind ≠ 1 → crash log, reply 98 `hh` [2].
5. Attach the zone and caster. Set short +144 = area.
6. Queue failure → reply 98 `hh` [2]. Disassembly confirms R9D = 2.
7. Queue success → no reply here.

❓ Missing: unset-short behavior on decode failure, task execution, area checks,
rewards and persistence. Rust returns `Unsupported` when it needs the unset short.
The no-zone and native-kind failure replies remain available on that input.

## 398 — G_RegisterSurveyReqCmd — partial

Handler `0x14039BE70`. Descriptor `hCc;` at `0x140A80578`.
Logs `0x140BEE730`, `0x140BEE7F0`, `0x140BEE890`.

Wire: `hCc;`: signed i16 count, then that many i8 order values.
The entry checks only the count. It does not check each order value.

1. Read the count through `0x14069DA00`. Count < 0 or count ≥ 10 → log "invalid order", reply 400 `hEc` [2, −1]. A missing count returns −1 and takes this path.
2. Read each byte through `0x14069F4B0`. A failed read → log "packet decompose fail", reply 400 `hEc` [2, −1]. Zero count is valid.
3. No zone → reply 400 `hEc` [2, −1], then log "cannot find zone".
4. Allocate 0xF80 bytes. Constructor `0x14029B820` sets kind 39 and vtable `0x140B20018`. Allocation/init failure → reply 400 `hEc` [2, −1].
5. Native kind ≠ 1 → log "[survey-task], invalid pc", reply 400 `hEc` [2, −1].
6. Attach the zone and caster. Set DWORD +144 = 0. Copy count bytes to +3952. Set WORD +3962 = count. Constructor clears WORD +148, text storage at +150 and WORD +3962.
7. Queue failure → reply 400 `hEc` [2, −1]. Raw body: `02 00 FF`.
8. Queue success → no reply here.

❓ Missing: survey registry, order validation in the task, task execution,
success reply and persistence. The queue adapter owns allocation/init failure.

## 401 — G_AnswerSurveyReqCmd — partial

Handler `0x14039C200`. Descriptor `hccs` at `0x140BEE924`.
Logs `0x140BEE3F0`, `0x140BEE4C0`.

Wire: `hccs`: survey i8, answer i8, counted UTF-16 text. Capacity 1900 code
units includes the terminal NUL. The maximum wire count is 1899.
The byte field names follow request order. ❓ Their domain meaning needs the task.

1. Decode failure → reply 403 `hh` [2], then crash log "decompose failed".
2. No zone → reply 403 `hh` [2], then log "cannot find zone".
3. Allocate the same 0xF80-byte survey task as 398: kind 39, vtable `0x140B20018`. Allocation/init failure → reply 403 `hh` [2].
4. Native kind ≠ 1 → log "[survey-task], invalid pc", reply 403 `hh` [2].
5. Attach the zone and caster. Set DWORD +144 = 1, byte +148 = first byte, byte +149 = second byte. Copy text to +150 with `wcscpy_s`, capacity 1901. The copy stops at the first NUL, including an embedded NUL from the wire.
6. Queue failure → reply 403 `hh` [2].
7. Queue success → no reply here.

❓ Missing: survey data, answer validation, task execution, success reply and
persistence. Rust retains byte bit patterns. The queue adapter owns allocation.

## 404 — G_AnswerExtraSurveyReqCmd — partial

Handler `0x14039C510`. Descriptor `hcs` at `0x14096B104`.
Failure log `0x140BEE580`.

Wire: `hcs`: one i8, then counted UTF-16 text. Capacity 1900 includes the
terminal NUL. The maximum count is 1899. The entry does not use the byte.

1. Decode failure → crash log, reply 406 `hh` [2]. The native call supplies another DB-ID argument, but `hh` does not serialize it.
2. Decode success → log the text at level 8 with format `%s`, then reply 405 `h` with an empty body.
3. No zone check, task allocation, queue or PC-kind check.

❓ Missing: logger side effects. Rust records a static log marker, not client text.

## 1017 — G_RefreshSimpleQuestPackReqCmd — partial

Handler `0x1403D0A70`. Logs `0x140C32F20`–`0x140C330A0`.

Wire: `h` at `0x1408CFBE8`. No body field. The extra pointer argument to the
decoder does not add a wire byte.

1. Arena form B → log "cannot process in arena", reply 1019 `hh` [5].
2. Rule cfg byte +1196 = 0 → log "tendency field config off", reply 1019 `hh` [2].
3. Header decode failure → log "fail to decompose packet", reply 1019 `hh` [2]. Rust dispatch already supplies the command header. An empty body is valid.
4. Read rule cfg byte +1196 again. Zero → log "rule config tendency filed enable off", reply 1019 `hh` [2]. Rust calls the configuration service twice in this order.
5. No zone → log "fail to get zone", reply 1019 `hh` [2].
6. Allocate task kind 257 (0x98 bytes, vtable `0x140A9CEC8`). Byte +144 = 1 at construction and after attachment. Attach the zone and caster. No PC-kind check.
7. Queue failure → log "RefreshSimpleQuestPackTask execute failed", reply 1019 `hh` [2].
8. Queue success → no reply here.

❓ Missing: malformed outer-header handling, task execution, quest-pack data,
success reply and persistence. Unknown configuration returns `Unsupported`.

## 1205 — G_R_SkipQuestTeleportReqCmd — partial

Handler `0x1403DC760`. Descriptor `hcc` at `0x14096CE28`.
Log `0x140C3DEE0`. Level/mastery logs confirm the signed byte fields.

Wire: `hcc`: level i8, mastery level i8. Both locals start at zero.

1. Daemon WORD +2752 ≠ 2 → log, reply 1207 `hh` [2627].
2. Decode failure → log, reply 1207 `hh` [2627].
3. No zone → log "pc has no zone", reply 1207 `hh` [2].
4. Level ≤ 0 or level > rule cfg signed DWORD +40 → reply 1207 `hh` [2630], then log the requested level and maximum.
5. Rule cfg byte +44 ≠ 0, and (mastery < 0 or mastery > signed DWORD +48) → reply 1207 `hh` [2630], then log the requested mastery and maximum. When byte +44 = 0, skip this check.
6. Allocate task kind 360 (0x98 bytes, vtable `0x140AF0170`). Constructor clears DWORD +144.
7. Native kind ≠ 1 → crash log, reply 1207 `hh` [2].
8. Unsigned byte `(level − 1)` > 69 → crash log "invalid level", reply 1207 `hh` [2]. Thus the task accepts levels 1..70 even if the rule maximum is higher.
9. Unsigned mastery byte > 50 → crash log "invalid mastery level", reply 1207 `hh` [2]. Thus the task rejects negative mastery even when the configuration check is off.
10. Set byte +144 = level, byte +145 = mastery, WORD +146 = 0. Attach the zone and caster.
11. Queue failure → log, reply 1207 `hh` [2]. Queue success → no reply here.

❓ Missing: task execution, quest skip rules, travel, success reply and persistence.
Unknown configuration returns `Unsupported`. The port does not complete travel.

## 1420 — G_ChangeToReplayQuestReqCmd — partial

Handler `0x1403C9D50`. Logs `0x140C29730`, `0x140C297D0`.

Wire: `hh`: quest i16. The decode result is ignored. The local short has no
initial value.

1. Decode the short.
2. No zone → reply 1422 `hh` [1], then log "cannot find zone".
3. Allocate task kind 185 (0x98 bytes, vtable `0x140BC2288`). Attach the zone and caster. Set short +144 = quest. No PC-kind check.
4. Queue failure → reply 1422 `hh` [1], then log "failed to ExecuteSerializedJob".
5. Queue success → no reply here.

❓ Missing: unset-short behavior on decode failure, task execution, replay quest
checks, success reply and persistence. Rust returns `Unsupported` when it needs
the unset short. The no-zone failure still uses reason 1, not 2.

## Rust port status

| Command | Rust | State / boundary |
| --- | --- | --- |
| 86, 89, 93 | `src/handlers/quest.rs` | Partial; list tasks queue through `skills::queue` |
| 96, 1420 | `src/handlers/quest.rs` | Partial; unset decode fields are `Unsupported`; task bodies are boundaries |
| 398, 401 | `src/handlers/quest.rs` | Partial; survey task fields retained; survey data and execution are boundaries |
| 404 | `src/handlers/quest.rs` | Partial; exact success/failure bodies; logger is a boundary |
| 1017, 1205 | `src/handlers/quest.rs` | Partial; configuration uses `Services::quest_config`; task bodies are boundaries |

`QUEST_COMMANDS` registers all ten entries. `MemoryServices::quest` supplies
configuration for tests. Its default is `Unknown`, not a successful lookup.
`Gate::Unknown` from the queue returns `Unsupported`. `Gate::Fail` emits the
native failure body. A successful queue retains the original request through
`StateChange::QueuedRequest`.
