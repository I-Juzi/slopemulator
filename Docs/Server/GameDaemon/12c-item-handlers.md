| NEW shared helper, build 381.8867 | Purpose | Verified limit |
| --- | --- | --- |
| `0x1403FB2D0` | Submit BoardGacha manager task at daemon +5824 | Entry callers pass final argument 0. False sends the entry failure. Five manager runners below use virtual +48. ❓ Queue internals remain open. |
| `0x1403F7D20` | Resolve manager-task PC by object ID | Manager runners compare returned PC +20744 with task +120. ❓ Lookup internals remain open. |
| `0x1401BB770` | Common slate action gate | Require signed PC bytes +276/+284 >= global DWORDs +2096/+2100, PC byte +578 ==0 and PC byte +888 bit 0 clear. Sources: helper and actions 1194/1197/1200. |
| `0x140329700` | Save slate through daemon +3112 | Requires nonzero PC identity. Builds backend command 17 with one nested operation. ❓ Exact variadic header fields and final transport remain open. |
| `0x140678250` | Initialize the BoardGacha manager task | Task kinds 56..60 appear below. ❓ Full initializer remains open. |
| `0x140055FE0` → `0x140056490` | Open BoardGacha session and send cached board info | See 1160/1165. Same-identity duplicate returns 2513. ❓ Cached packet construction remains open. |

# 12c — Item and inventory handlers, part 3

## Scope and notation

Static specification. Source: `ida/datafile/gd2020/GameDaemon.exe`, build 381.8867.
The build-348 names come from the selected ledger rows. No build-348 binary was opened.
Reuse part-1 request decoding, reference release, direct-PC reply, zone queue, inventory batches and money helpers.
Reuse part-1 timeout registration `0x1400D4D30` and cancellation `0x1400D4E90` at daemon +5080. BoardGacha runners enable timeout records only when cfg +76 !=0. Their records store transaction ID +56, state +64 and timeout event +68. ❓ Timer internals remain open.
Reuse part-2 inventory lookup `0x1404CAD00`, inventory-task factory/binder and alternate craft-task dispatch by name.
Do not apply a shared gate unless a section lists it.
`pc` means requester. `cfg` means daemon +2736. `feature` means daemon +2744.
`tm[n]` means table-manager qword slot n at `qword_140F77978`.
Native offsets are not wire offsets. Formats include the command field. Case is significant.
Where a request row gives a constant address, its literal was checked in build-381 bytes. The handler target is the source for field order and consumed fields. ❓ Codec contracts for `Q`, `E`, `u`, counted groups and nested formats remain open unless stated.
Direct replies use PC virtual +592 and go only to requester unless stated.
A client batch through virtual +584 is not proof of a zone broadcast.
`done` means all action paths were read. `partial` is not a complete Rust port contract.
No runtime test ran. No DB procedure name is verified unless stated.
For kick paths, see `07-server-validation.md`, part 4. A log, assertion or reference release is not a kick.
No direct kick appears in the read paths unless stated. Unread dependencies remain unknown.
No anti-cheat or licence path was followed.

## Shared BoardGacha entry gates

Sources: the eight command targets below, build 381.8867.
All eight require global `qword_140EE58A0` byte +2088 !=0, then cfg byte +404 !=0.
Either failed gate sends the section's failure command, `hh`, reason 2.
None reads a request body or calls a decoder. Effective consumed prefix: `h`.
None applies T1, T2, an inventory-slot check, a money check, a distance check, a cooldown check or a game-table lookup at entry.
All return true. Entry posting does not prove action success.

Manager-task entries allocate 0x80 bytes. Store PC object ID +40 at task +112 and PC identity +20744 at +120.
Submit through `0x1403FB2D0(daemon+5824,task,0)`. False sends the entry failure with reason 2.
Release the local task reference after submission.

User-transaction entries resolve user from PC +20696 only when PC +20752 ==0.
Missing user logs and returns without a packet.
Allocate 0x60 bytes. Initialize with part-1 user-transaction initializer `0x14002F440`.
Set transaction qword +88 to zero. Get transaction ID from daemon +5128.
Register with part-1 `0x1400D4FE0`, then start with `0x1400D4A80` and the event below.
Registration failure sends the entry failure with reason 2. Release local transaction/user references.
Manager runners use virtual +48. They resolve the PC with `0x1403F7D20(daemon+5824,&pc,task.object_id,1)` and compare PC +20744 with task +120.
For 1152/1160/1165/1173, missing PC or identity mismatch logs without a reply. A missing manager at daemon +3680 sends the section's failure command, `hh`, reason 2.
Release the resolved PC reference after use. Command 1162 has the separate offline-close path below.
❓ PC-resolution helper internals, callback ingress, timer/lifecycle internals, cached packet construction and DB procedures remain open.
Do not import the BoardGachaDraw +252 kick rule into these entries without a trace.

## Command 1152 — G_BoardGachaRetrieveSeasonReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D8EB0` |
| 348 name | `HandleGame_BoardGachaRetrieveSeasonReq` |
| State | **partial** |
| Request | No body read; effective `h` |
| Entry failure | 1154, `hh`, reason 2 |

Apply shared BoardGacha gates. Create manager kind 56, vtable `0x1408DD020`. Submit and release as above.
No inventory or balance mutation appears at entry.
Runner `0x140057AF0` checks the common PC identity and manager pointer. Read the manager's first qword as a cache pointer. If non-null, send cache +8 through requester virtual +584. Null cache logs without a failure reply. No inventory/money mutation, zone broadcast, direct backend request or kick appears in this runner.
❓ Cached success command/format, cache construction and manager/PC lookup internals remain open.

## Command 1160 — G_BoardGachaSessionOpenReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D8FF0` |
| 348 name | `HandleGame_BoardGachaSessionOpenReq` |
| State | **partial** |
| Request | No body read; effective `h` |
| Entry failure | 1161, `hh`, reason 2 |

Apply shared BoardGacha gates. Create manager kind 57, vtable `0x1408D30A8`. Submit and release as above.
No session-state mutation appears before manager submission.
Runner `0x140057CD0` applies the common PC-identity/manager checks. Call `0x140055FE0(manager,pc)`. A nonzero returned short sends 1161 `hh` with that reason. Zero adds no runner reply. No direct inventory/money mutation or kick appears in the runner.
Session helper `0x140055FE0`: null PC returns 2. Look up object ID in the manager +24 session map. If absent, insert `(object_id,PC identity)`, then call `0x140056490` and return 0. If present with the same identity, return **2513** without sending cached board info. If present with a different identity, overwrite that identity, send cached board info, and return 0.
The shared board-info sender passes manager qword +8, plus 8, to requester virtual +584 when the cache exists. Missing cache logs and returns success without a packet. This can leave a new session open without a success reply. No 1160-specific success packet appears in these read functions. No inventory/money mutation, direct backend/DB call, broadcast or kick appears.
❓ Cached board-info command/format/construction and generic lookup/lifecycle internals remain open.

## Command 1162 — G_BoardGachaSessionCloseReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D9130` |
| 348 name | `HandleGame_BoardGachaSessionCloseReq` |
| State | **partial** |
| Request | No body read; effective `h` |
| Entry failure | 1164, `hh`, reason 2 |

Apply shared BoardGacha gates. Create manager kind 58, vtable `0x1408DF278`. Submit and release as above.
No session-state mutation appears before submission.
Runner `0x140057E80` resolves the PC if available. A resolved PC with a different identity logs and stops without closing the session or replying. A missing PC does **not** stop session-map cleanup.
Read manager at daemon +3680. Missing manager, zero task object ID, or absent session-map key logs and sends 1164 `hh` reason 2 only if a PC is available.
The ordered map at manager +24 maps object ID to PC identity. If its stored identity equals task +120, erase/free that node. If it differs, log and leave the node intact. **Both** identity-match and stored-identity-mismatch paths send 1163 `h` to the resolved PC, if available.
No inventory/money mutation, zone broadcast, direct backend/DB call or kick appears in this runner. Release PC references. Source for `h`: verified raw bytes at `0x1408CFBE8`.
❓ PC lookup and generic map/reference lifecycle internals remain open. The session action itself is traced.

## Command 1165 — G_BoardGachaBoardInfoReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D9270` |
| 348 name | `HandleGame_BoardGachaBoardInfoReq` |
| State | **partial** |
| Request | No body read; effective `h` |
| Entry failure | 1166, `hh`, reason 2 |

Apply shared BoardGacha gates. Create manager kind 60, vtable `0x1408D4470`. Submit and release as above.
No inventory or money mutation appears at entry.
Runner `0x140058200` applies the common PC-identity/manager checks, then calls `0x140056490(manager,pc)`. No extra runner reply follows that call. No direct inventory/money mutation or kick appears in the runner.
Helper `0x140056490` sends manager qword +8, plus 8, through requester virtual +584. A null cache logs and still returns true. A null PC returns false. This helper makes no session-map check. No direct backend/DB call, broadcast or kick appears.
❓ Cached success command/format and construction remain open.

## Command 1167 — G_BoardGachaGetParticipationCountPerSeasonReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D93B0` |
| 348 name | `HandleGame_BoardGachaGetParticipationCountPerSeasonReq` |
| State | **partial** |
| Request | No body read; effective `h` |
| Entry failure | 1169, `hh`, reason 2 |

Apply shared BoardGacha gates and user-transaction rules.
Use vtable `0x1409E5CD8`, start event 65, null payload. Initial transaction DWORD +8 is 4923658.
No inventory or money mutation appears at entry.
Runner `0x1401C6130`, virtual +48:
1. In state +32 ==0, require start event 65. Otherwise call failure helper `0x1401C6380(reason=2)`. Valid start optionally allocates a 0x48-byte timeout record when cfg +76 !=0, then sets state 1 through `0x1400D47D0`. Reset the local event to 0.
2. In state 1/event 0, register the timeout record with event **129**. Registration failure calls failure helper with 2. Send daemon +5152 command **64**, verified descriptor `hqq` at `0x1408CE234`, transaction ID +24 and user +216; route by signed user short +236. Send failure calls failure helper with 2. ❓ The decompiler adds an uninitialized extra argument; do not port it.
3. Event **127** cancels the timeout. Null payload fails with 2. Otherwise read payload DWORD count. Resolve the current user connection through user virtual +48. If present, send **1168 `hi`**, count, with `0x1406A5D30`, then release the connection. Set state 2 even if no connection exists.
4. Event **128** reads payload u16 reason, cancels timeout, then calls failure helper. Event **129** calls failure helper with **3**. Other events return true without action. No null guard appears before the event-128 reason read.

Failure helper `0x1401C6380` resolves the current user connection through user virtual +48. If present, send **1169 `hh`** with `0x140018AE0(u16(reason))`, then release the connection. If absent, log without a packet. Set state 2 in both cases. Use the existing reason map in doc 07, part 4. This helper does not cancel the timeout itself.
No inventory/money mutation or kick appears in the runner or failure helper. ❓ Timer cancellation after send/registration failure or timeout, state-transition cleanup, daemon reply ingress and DB procedure remain open.

## Command 1170 — G_BoardGachaRetrieveParticipationInfoPerPcReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D95F0` |
| 348 name | `HandleGame_BoardGachaRetrieveParticipationInfoPerPcReq` |
| State | **partial** |
| Request | No body read; effective `h` |
| Entry failure | 1172, `hh`, reason 2 |

Apply shared BoardGacha gates and user-transaction rules.
Use vtable `0x1409E5AD0`, start event 66, null payload. Initial transaction DWORD +8 is 4989194.
No inventory or money mutation appears at entry.
Runner `0x1401C64E0`, virtual +48, uses the same state-0/start/state-1 structure as 1167, but requires start event **66**. Its failure helper is `0x1401C6950`. Optional timeout event is **132**. State 1/event 0 sends daemon +5152 command **67**, verified descriptor `hqq` at `0x1408CE234`, transaction ID +24 and user +216, routed by signed user short +236. Timer registration or send failure uses reason 2.
Event **130** delegates to success helper `0x1401C6690`. Event **131** reads payload u16 reason, cancels timeout and calls the failure helper. Event **132** fails with **3**. Other events return true without action. No null guard appears before the event-131 reason read.
Success helper `0x1401C6690` cancels the timeout before checking payload. Null payload calls failure helper with 2. Native payload: qword +0, signed-short count +8, then two signed bytes per entry from +10 with stride 2. Resolve the current user connection through user virtual +48. If present, build **1171 `hqCcc;`**, raw literal at `0x1408F5020`: payload qword, then the counted byte pairs. Instruction proof confirms both fields of each pair. Serialize only while signed count >0; no list bound appears in this helper. Queue the packet under connection lock +24 through `0x1406A6420`, then release the connection. Set state 2 even if no connection exists. No zone broadcast appears.
Failure helper `0x1401C6950` sends **1172 `hh`** to the current user connection with `0x140018AE0(u16(reason))`, releases it, then sets state 2. Missing connection logs without a packet. Use doc 07, part 4. No timer cancellation occurs inside that failure helper.
No inventory/money mutation or kick appears in these read paths. ❓ Native payload allocation bounds/field meanings, backend reply ingress, timer/state-transition cleanup, serializer failure policy and DB procedure remain open.

## Command 1173 — G_BoardGachaRetrieveParticipationInfoAllReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D9830` |
| 348 name | `HandleGame_BoardGachaRetrieveParticipationInfoAllReq` |
| State | **partial** |
| Request | No body read; effective `h` |
| Entry failure | 1175, `hh`, reason 2 |

Apply shared BoardGacha gates. Create manager kind 59, vtable `0x1408D2290`. Submit and release as above.
No inventory or money mutation appears at entry.
Runner `0x140058390` applies the common PC-identity/manager checks. If manager qword +16 is non-null, pass that pointer **directly** to requester virtual +584. Instruction proof: load `[manager+0x10]` into RDX, then call `[pc.vtable+0x248]`. Unlike the season and board-info senders, this runner adds no +8. Null cache logs without a failure reply. No inventory/money mutation, direct backend/DB call or kick appears in this runner.
❓ Cached success command/format, splitting and cache construction remain open.

## Command 1182 — G_BoardGachaRetrieveParticipationRewardStateReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403DA5D0` |
| 348 name | `HandleGame_BoardGachaRetrieveParticipationRewardStateReq` |
| State | **partial** |
| Request | No body read; effective `h` |
| Entry failure | 1184, `hh`, reason 2 |

Apply shared BoardGacha gates and user-transaction rules.
Use vtable `0x1409E5E50`, start event 70, null payload. Initial transaction DWORD +8 is 5251338.
No reward grant or inventory/money mutation appears at entry.
Runner `0x1401C98F0`, virtual +48, uses the same state-0/start/state-1 structure as 1167, but requires start event **70**. Its failure helper is `0x1401C9E70`. Optional timeout event is **152**. State 1/event 0 sends daemon +5152 command **85**, verified descriptor `hqhq` at `0x1408CFC04`, transaction ID +24, signed user short +236, and user qword +216. Route by the same signed short. Timer registration or send failure uses reason 2.
Event **150** delegates to success helper `0x1401C9AC0`. Event **151** reads payload u16 reason, cancels timeout and calls failure helper. Event **152** fails with **3**. Other events return true without action. No null guard appears before the event-151 reason read.
Success helper `0x1401C9AC0` cancels the timeout first. Native payload has DWORD +0 and a stream pointer at +8. Null payload calls failure helper with 2. Build command **1183**, verified descriptor **`hiiChihh;`** at `0x140A1FF10`. The packet builder reserves one integer for framing/splitting; it writes payload DWORD +0 as the next integer. ❓ Reserved integer semantics remain open.
Read a signed-short group count from the stream. For each positive-count entry, decode four fields into short/integer/short/short destinations, then append them in that order. Instruction proof confirms all four variadic arguments. Start a new packet segment when fewer than ten bytes remain before an entry. Finish the group, invoke the stream virtual cleanup, and finish the packet list. Resolve current user connection through user virtual +48. If present, queue through `0x1406A6420` under connection lock +24 and release it. Set transaction state 2 even if no connection exists. Release packet-builder storage.
Failure helper `0x1401C9E70` instead resolves PC from user +720 only when user byte +1017 ==0. If present, send requester **1184 `hh`**, reason `0x140018AE0(u16(reason))`, then release PC. Missing PC logs without a packet. Set state 2 in both cases. Use doc 07, part 4. The helper does not cancel the timeout itself.
No reward grant, inventory/money mutation, zone broadcast or kick appears in these read paths. ❓ Result field meanings, reserved header/split contract, stream bounds/error handling, backend reply ingress, timer/state-transition cleanup and DB procedure remain open.

## Command 1140 — G_ChangeBadgeEquipPageReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D8040` |
| 348 name | `HandleGame_ChangeBadgeEquipPageReq` |
| State | **partial** |
| Request | `hc`: page byte |
| Failure | 1142, `hh`: reason |

1. Require global bytes +2040 and +2008 at `qword_140EE58A0` nonzero, else reason 2.
2. Decode without testing the result. Require zone, else 2.
3. Allocate 0x98-byte kind-325 task, vtable `0x140BC27B0`. Require PC kind +46 ==1; wrong kind asserts and then sends 2 if it returns.
4. Bind PC/zone. Store page at task +144. Queue with part-1 zone queue. Queue failure sends 2. Release local reference. Return true.

No page-range, item, money, distance, cooldown or table check appears at entry. No direct mutation appears before posting.
### Task action `0x1402D2860`

Apply these checks in order. Each failure sends 1142 `hh`, reason **2468**.
1. Recheck global bytes +2040/+2008. Require signed PC qword +856 plus signed DWORD +824 >0.
2. Read signed PC short +848. Require its unsigned `(value-1)` <=999. Look up the pointer at `tm[3352] + 8*value`. Missing record fails.
3. Require page 1..3 and page <= signed record byte +70.
4. Require page differs from signed PC byte +880.

Call `0x1404CA930(pc+15000)`. Initialize a property-delta batch. Call `0x140271BF0(pc,page,batch)`, then part-1 delta flush `0x140260850`. Call `0x1404CA690(pc+15000)`. Send requester 1141 `hc`, page. No direct kick appears.
❓ Table/field names, page-change helper effects, inventory helper internals, downstream broadcasts/backend/DB work and lifecycle remain open.

## Command 1143 — G_SynthesisBadgeReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D8270` |
| 348 name | `HandleGame_SynthesisBadgeReq` |
| State | **partial** |
| Request | Literal `hCQhh;Chh;`: first group of selector/short/short triples; second group of short pairs |
| Failure | 1145, `hE`: reason |

1. Require the same two global feature bytes as 1140, else reason **1**.
2. Apply part-2 U0 user-state gate, else **2482**. Require zone, else **2**. These checks precede decode.
3. Read first signed-short count. Require 1..8 inclusive. Decode three fields per entry into six native bytes. Count/decode failure sends **2469**.
4. Finish first group. Read second count. Require 0..16 inclusive. Decode two fields per entry into four native bytes. Failure sends **2469**. Finish group.
5. Allocate 0x128-byte kind-324 task, vtable `0x140BC24A8`. Require PC kind 1, else assert then reason 2.
6. Bind. Store PC pointer +144; counts +160/+162; first entries +164; second entries +212. Queue. Failure sends 2. Release local reference. Return true.

No per-item, count, balance, distance, cooldown or table check appears in this entry. No direct inventory mutation appears.
Raw descriptor source: `0x140C39FA8`, verified `hCQhh;Chh;`.
❓ Read task action, field meanings, synthesis tables/probability/cost, ownership, success/broadcasts, backend/DB work and lifecycle.

## Command 1146 — G_SynthesisBadgeRecipeDataReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D8950` |
| 348 name | `HandleGame_SynthesisBadgeRecipeDataReq` |
| State | **partial** |
| Request | No decoder/body read; effective `h` |
| Failure | 1148, `hh`: reason 1 |

Require global bytes +2040 and +2008 nonzero. Failure sends 1148 reason 1.
Success passes table-manager +82176 to PC virtual +584, then logs. Return true.
No inventory, money, ownership, distance, cooldown or per-request table lookup appears in this target.
❓ Read the cached packet construction at table-manager +82176. Success command, format, recipe fields, empty-cache behavior and construction-time table source remain open. No direct daemon/DB call appears in the target.

## Shared slate entry rules

Sources: commands 1191, 1194, 1197 and 1200 below.
Apply part-1 T1 then T2. Fail with reason 5 then 7, using each section's failure command and `hh`.
Require signed global DWORD +2092 at `qword_140EE58A0` >0. Failure uses the section's disabled reason.
These entries return true and use the part-1 zone queue. Release local task references after posting.
No inventory, money, distance, cooldown or table check appears before posting. No direct state mutation appears there.

## Command 1191 — G_DrawSlateNewStoneReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403DB040` |
| 348 name | `HandleGame_DrawSlateNewStoneReq` |
| State | **partial** |
| Request | Verified `hcchChh;ib` at `0x140C3C218`: two bytes, short, counted short pairs, integer, blob |
| Failure | 1193, `hh`: reason |
| Disabled reason | 2542 |

1. Apply shared slate entry gates.
2. Decode three prefix fields into page byte, another byte and a native eight-byte structure. Failure sends **2544**.
3. Read count. Require 0..16. Decode short pairs into four-byte entries. Failure sends 2544. Finish group.
4. Decode two trailing fields into integer/blob storage. Failure sends 2544.
5. The native high DWORD of the eight-byte prefix structure must be <=0 or exactly 8. Otherwise send 2544.
6. Require zone, else 2. Allocate 0xF8-byte task through `0x1402D5E40`.
7. Initialize through `0x1402D5F60` with the decoded prefix, pairs/count and trailing values. False initialization sends 2544. Null task or failed queue sends 2.

The first three destination addresses and the blob capacity/result arguments need instruction proof. The eight-byte native structure starts as `0x800010000`; this does not prove an eight-byte wire field.
❓ Read initializer/action, decoder destination mapping, blob/count interpretation, page checks, consumption, random result, inventory/replies/broadcasts, backend/DB work and lifecycle.

## Command 1194 — G_SetSlateNewStoneReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403DB5F0` |
| 348 name | `HandleGame_SetSlateNewStoneReq` |
| State | **partial** |
| Request | `hc`: page byte |
| Failure | 1196, `hh`: reason |
| Disabled reason | 2547 |

1. Apply shared slate gates. Initialize page to zero and require nonzero low-16-bit decode result, else **2549**.
2. Require zone, else 2. Allocate 0x98-byte kind-341 task, vtable `0x140BBD980`.
3. Require `u8(page-1)<=1` and page <= global DWORD +2092. Failure sends **2**, not 2549.
4. Store page at +144, bind and queue. Queue failure sends 2.

### Task action `0x1402D8390`

1. Require `0x1401BB770(pc+20616)` true, else 1196 `hh` **2548**.
2. Inspect both slate pointers at PC +20624/+20632. At least one must have short +128 !=0 OR byte +130 !=0. If neither does, send **2550**.
3. Require requested page equals PC byte +891, else **2549**.
4. Require page 1..2 and non-null selected pointer at `pc+20616+8*page`. Bad state logs/asserts, then sends **2** if control returns.
5. Snapshot selected byte +130, short +128 and the current value for index 1..9 from shorts +110..+126. Initialize the other result fields to zero.
6. Call `0x1401BA930(slate)`. False sends **2**. True sends requester 1195 `h`, then calls `0x1402D8660(task,snapshot)`.

Apply helper `0x1401BA930` requires both pending short +128 and index byte +130 nonzero. For index 1..9, save the old short at +110..+126 and set property `12+index` to the pending short. An index outside 1..9 logs but still continues with the clear/save path. Clear properties 22 then 23. Call shared slate save `0x140329700`. In a fresh delta batch, remove the old value through `0x1401BB040` only if old value !=0, then apply the pending value through `0x1401BAEB0`. Flush before the 1195 reply. Return true.
Post-success helper `0x1402D8660` runs only when cfg +537 !=0. It collects PC state through `0x14060D810`, looks up old/new stone records in `tm[2691]` by signed stone short, copies record qword +16 and byte +24 to the snapshot, then calls `0x14020D030`. Missing records use an empty-value pointer.
No direct kick appears. ❓ Stat remove/apply helpers, property setter, save transport/DB procedure, optional report codecs and lifecycle remain open.

## Command 1197 — G_GiveUpSlateNewStoneReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403DB9B0` |
| 348 name | `HandleGame_GiveUpSlateNewStoneReq` |
| State | **partial** |
| Request | `hc`: page byte |
| Failure | 1199, `hh`: reason |
| Disabled reason | 2551 |

1. Apply shared slate gates. Initialize page to zero and require decode success, else **2553**.
2. Require zone, else 2. Allocate 0x98-byte kind-342 task, vtable `0x140BB36B8`.
3. Require `u8(page-1)<=1` and page <= global DWORD +2092. Failure sends **2**.
4. Store page +144, bind and queue. Queue failure sends 2.

### Task action `0x1402D88D0`

1. Require `0x1401BB770(pc+20616)` true, else 1199 `hh` **2552**.
2. Inspect both slate pointers at PC +20624/+20632. Require at least one short +128 !=0 OR byte +130 !=0, else **2554**.
3. Require requested page equals PC byte +891, else **2553**.
4. Require page 1..2 and selected slate non-null. Bad state logs/asserts, then sends **2** if control returns.
5. On the selected slate, require **both** short +128 !=0 and byte +130 !=0. Otherwise send **2**.
6. Set property IDs **22**, then **23**, to zero through `0x140696F20(slate+16,id,0,0,0)`. Call `0x140329700(daemon+3112,slate.owner,slate)` to save. Send requester 1198 `h`.

No refund, inventory/money mutation, broadcast or kick appears directly. Shared slate save `0x140329700` uses backend command **17**. It writes a leading zero, starts a nested operation, and uses PC identity and slate page byte +8 in its header. It appends the slate property buffer at slate +24, completes the group, and calls `0x14032AB10`. Identity zero or serialization failure logs without a new client failure reply. The raw save descriptor at `0x140B69EA8` is **`hQCqqchb;`**. ❓ The `Q` header value and omitted header variadic arguments need instruction proof. Property setter, final transport/DB procedure and lifecycle remain open. No refund rule is proved.

## Command 1200 — G_ChangeSlatePageReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403DBD70` |
| 348 name | `HandleGame_ChangeSlatePageReq` |
| State | **partial** |
| Request | `hc`: page byte |
| Failure | 1202, `hh`: reason |
| Disabled reason | 2555 |

1. Apply shared slate gates. Initialize page to zero and require decode success, else **2557**.
2. Require zone, else 2. Allocate 0x98-byte kind-343 task, vtable `0x140BB01B0`.
3. Require `u8(page-1)<=1` and page <= global DWORD +2092. Failure sends **2557**, unlike 1194/1197.
4. Store page +144, bind and queue. Queue failure sends 2.

### Task action `0x1402D8BD0`

1. Require `0x1401BB770(pc+20616)` true, else 1202 `hh` **2556**.
2. Both slate pointers at PC +20624/+20632 must have short +128 ==0 AND byte +130 ==0. Otherwise send **2558**.
3. Require requested page differs from signed PC byte +891, else **2557**.
4. Resolve old and new pages, each 1..2, at `pc+20616+8*page`. A bad page or null pointer logs/asserts, then sends **2** if control returns.
5. Initialize a delta batch. Call `0x1401BADD0(old_slate,batch)`, then `0x1401BACF0(new_slate,batch)`.
6. Set PC property **242** to requested page through `0x140696F20(pc+48,242,page,0,0)`. Append that property with `0x140273E10`. Flush through part-1 `0x140260850`. Send requester 1201 `h`.
7. If cfg byte +537 !=0, collect PC state through `0x14060D810`, then call `0x14020D410`. ❓ Its argument types and outbound purpose remain unverified.

No direct kick appears. ❓ Remove/apply stat effects, property setter, optional post-success path, downstream recipients/backend/DB work and lifecycle remain open.

## Command 830 — G_PassingItemAbilityReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403C2E70` |
| 348 name | `HandleGame_PassingItemAbilityReq` |
| State | **partial** |
| Request | Shared literal `hqChh;Chh;Chh;Chh;Qib` at `0x140C1EE58`. This handler consumes only the qword and first three counted short-pair groups. It does not decode the fourth group or `Qib` suffix. |
| Failure | 832, literal `hEq`: reason, qword echo |

1. Require global byte +1992 !=0, else **2013**, echo 0. Apply U0, else **2030**, echo 0. Require zone, else **2**, echo 0.
2. Decode qword prefix. Failure sends **2013**, echo 0.
3. First count must be 0..1, else **2017**. Second count must be 0..1, else **2021**. Third count must be 0..8, else **2023**. Each entry decodes two shorts. Any entry decode failure sends **2013**. These failures echo the decoded qword.
4. Allocate 0xF8-byte kind-303 task, vtable `0x140BB0F58`. Initialize through `0x140534270` with PC/zone, prefix and three groups. ❓ The decompiler truncates some pointer arguments; use instruction proof before porting.
5. Queue only if initialization succeeds. Failed queue sends reason 2 with the qword echo. Failed initialization has **no additional entry reply**. Release local reference. Return true.

No item/balance mutation, distance, cooldown or table lookup appears in the read entry.
❓ Read initializer/action, exact prefix/group meanings, ownership, transfer cost/results, inventory/replies/broadcasts, backend/DB work and lifecycle.

## Command 833 — G_PickGemCardReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403C35A0` |
| 348 name | `HandleGame_PickGemCardReq` |
| State | **done** |
| Request | No decoder or body read |

The full target logs the requester object ID and returns true.
It performs no state, ownership, slot, count, money, distance, cooldown or table check.
It changes no game state. It sends no reply or broadcast. It calls no daemon transaction, DB procedure or kick helper.
Do not add a gem-card action from its name.

## Command 947 — G_ChangeWeaponAppearanceReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403CAD80` |
| 348 name | `HandleGame_ChangeWeaponAppearanceReq` |
| State | **partial** |
| Request | Verified `hQhhChh;` at `0x140BD20D8`: selector and two shorts, then counted short pairs; ❓ semantic prefix meanings |
| Failure | 949, `hh`: reason |

1. Require global byte +208 !=0, else **2189**. Apply T1, else **5**. No T2 gate appears.
2. Decode three prefix fields into native short-based storage. Read signed-short count; unsigned value must be <=7. Decode two fields per entry. Any decode/count failure sends **2190**.
3. Require zone and non-null 0xB0-byte task from `0x1402B8930`, else **2**.
4. Call validator `0x1402B8C80(pc,decoded_structure)`. False sends 2.
5. Set task DWORD +144 =1. Allocate/copy a 0x24-byte request structure to task pointer +152. Bind and dispatch through part-2 alternate craft-task dispatch. Release reference. Return true.

No direct mutation appears in the entry. ❓ Validator/action, request widths, item/material rules, payment, appearance changes, success/broadcast codecs, tables, backend/DB work and lifecycle remain open.

## Command 1010 — G_ExchangeItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403CFC20` |
| 348 name | `HandleGame_ExchangeItemReq` |
| State | **partial** |
| Request | Verified `hQqChh;` at `0x140BD1E18`: container selector, qword recipe key, counted short pairs |
| Failure | 1012, `hh`: reason; extra native variadic qword is not encoded by this descriptor |

1. Apply U0, else **2302**. Require zone, else **2**.
2. Decode container. Require {2,3,6,8} (mask 332), else **2293**. Decode recipe key, else 2293.
3. Count must be 0..4, else **2296**. Decode each short pair, else 2293.
4. Allocate 0xD0-byte kind-247 task, vtable `0x140C3E0D8`. Require PC kind 1; wrong kind asserts without an explicit reply in that branch.
5. Look up `tm[1479]` virtual +168 with recipe key. Missing record sends **2294**.
6. Bind. Store PC +144, recipe record +152, container +168, count +170, pairs +182. Queue failure sends **1**. Release reference. Return true.

No direct inventory/money mutation appears before posting. No entry distance/cooldown check appears.
❓ Recipe table/field names, action, pair meanings, consumption/output capacity/payment, replies/broadcasts, daemon/DB work and lifecycle remain open.

## Command 1050 — G_MakeCustomDressReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D18B0` |
| 348 name | `HandleGame_MakeCustomDressReq` |
| State | **partial** |
| Request | Verified `hhCcc;Chh;` at `0x140C346E8`: target short, first counted two-byte entries, second counted short pairs |
| Failure | 1052, `hh`: reason |

1. Require global byte +1981 !=0, else **2360**.
2. Decode target short, else **2**. First count must be 0..8, else 2. Decode two fields per first entry into separate byte arrays, else 2.
3. Second count must be 0..16, else **2363**. Decode short pairs, else 2.
4. Require zone, else 2. Allocate 0xF0-byte kind-287 task, vtable `0x140C438A8`. Require PC kind 1; wrong kind asserts then sends 2.
5. Bind. Store PC +144, target +152, first count +154, two byte arrays +156/+164, second count +172 and pairs +174. Queue failure sends 2. Release reference. Return true.

No U0/T1/T2 gate, direct item/money mutation, distance, cooldown or table lookup appears in this entry.
❓ Action, request field meanings, dress/material tables, ownership, payment/output, success/broadcasts, backend/DB work and lifecycle remain open.

## Shared item-use entry rules

Sources: the ten targets in this group. All decode before the zone check and ignore decoder success.
They do not apply T1, T2 or U0. They do not validate client positions at entry.
They bind PC/zone, post through part-1 zone queue, release local references and return true.
No inventory, money, distance, cooldown or table check appears at entry. No direct item mutation appears there.
All action effects, table fields, success replies, broadcasts, backend/DB work and task lifecycle remain ❓ unless the section adds a trace.
Fields without a proved meaning use task offsets below. Do not assign an item/skill/target meaning from width alone.

## Command 1233 — G_UseGroceryReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140377FB0` |
| 348 name | `HandleGame_UseGroceryReq` |
| State | **partial** |
| Request | `hhiqhq`: short, integer, qword, short, qword |
| Failure | 1235, `hh`, reason 1 |

Require zone. Allocate 0xB8-byte task through `0x14036D460`. Missing zone, null task or failed queue sends failure.
Copy body fields in order to task +144, +148, +152, +160 and +168. Bind and post.
❓ Read constructor/action and resolve all five semantic field names and shared item-use gaps.

## Command 1234 — G_UseGroceryToGroundReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140378470` |
| 348 name | `HandleGame_UseGroceryToGroundReq` |
| State | **partial** |
| Request | `hhihhhhq`: short, integer, three coordinate-like shorts, short, qword |
| Failure | 1235, `hh`, reason 1 |

Require zone. Allocate the same 0xB8-byte task through `0x14036D460`.
Copy body fields to +144, +148, +176/+178/+180, +160 and +168. Set task target qword +152 to zero.
Missing zone, null task or failed queue sends failure. Bind and post.
❓ Prove the three-short position meaning, coordinate validation, constructor/action and shared item-use gaps. The binary log uses the non-ground name.

## Command 1236 — G_UseEquippedReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403786A0` |
| 348 name | `HandleGame_UseEquippedReq` |
| State | **partial** |
| Request | `hhqh`: short, qword, short |
| Failure | 1238, `hh`, reason 1 |

Require zone. Allocate 0xA8-byte task through `0x14036DA00`.
Copy body fields to +144, +152 and +160. Missing zone, null task or failed queue sends failure. Bind and post.
❓ Resolve field meanings, constructor/action and shared item-use gaps.

## Command 1237 — G_UseEquippedToGroundReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140378840` |
| 348 name | `HandleGame_UseEquippedToGroundReq` |
| State | **partial** |
| Request | `hhhhhh`: five short body fields |
| Failure | 1238, `hh`, reason 1 |

Require zone. Allocate the same 0xA8-byte task through `0x14036DA00`.
Copy body fields to +144, +162, +164, +166 and +160. Set target qword +152 to zero.
Missing zone, null task or failed queue sends failure. Bind and post.
❓ Prove field/coordinate meanings, constructor/action and shared item-use gaps.

## Command 1242 — G_UseFieldItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140378A30` |
| 348 name | `HandleGame_UseFieldItemReq` |
| State | **partial** |
| Request | `hqhc`: qword, short, byte |
| Failure | 1244, `hh`, reason 1 |

Require zone. Allocate 0xB0-byte task through `0x14036D4C0(task,0)`.
Require task byte +144 ==0; mismatch asserts then sends failure. Copy fields to +152, +166 and +168.
Missing zone, null task or failed queue sends failure. Bind and post.
❓ Resolve target/operand meanings, constructor/action and shared item-use gaps.

## Command 1243 — G_UseFieldItemToGroundReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140378CA0` |
| 348 name | `HandleGame_UseFieldItemToGroundReq` |
| State | **partial** |
| Request | Verified `huuuhc` at `0x140BCCAD0`: three lowercase `u` fields, short, byte |
| Failure | 1244, `hh`, reason **2** |

Initialize all five body destinations to zero. Decode without testing success.
Require zone. Allocate 0xB0-byte task through `0x14036D4C0(task,1)`. Require task byte +144 !=0; mismatch asserts.
Copy first three low-16-bit values to +160/+162/+164; final short/byte to +166/+168. Bind and post.
Missing zone, null task, mode mismatch or failed queue sends reason 2.
❓ Resolve lowercase `u` codec, position validation, constructor/action and shared item-use gaps. Do not copy reason 1 from non-ground use.

## Command 1245 — G_ThrowFieldItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140378EF0` |
| 348 name | `HandleGame_ThrowFieldItemReq` |
| State | **partial** |
| Request | Verified `huuuh` at `0x140BCC998`: three lowercase `u` fields and short; ❓ `u` codec |
| Failure | 1246, `hh`, reason 1 |

Require zone. Allocate 0x98-byte kind-27 task, vtable `0x140B95018`. Require PC kind 1; mismatch asserts then sends failure.
Copy four low-16-bit body values to +144/+146/+148/+150. Bind and post. Missing zone or failed queue sends failure.
❓ Resolve position/operand meanings, task action and shared item-use gaps.

## Command 1247 — G_PickupFieldItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140379170` |
| 348 name | `HandleGame_PickupFieldItemReq` |
| State | **partial** |
| Request | `hq`: object selector qword |
| Failure | 1248, `hh`, reason 1 |

Require zone and inventory factory kind 23. Require PC kind 1; mismatch asserts then sends failure.
Bind with part-2 inventory binder. Store selector at +152. Missing zone, null factory or failed queue sends failure.
❓ Read kind-23 action, object ownership/distance/capacity, pickup mutation and shared item-use gaps.

## Command 1249 — G_PutdownFieldItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140379360` |
| 348 name | `HandleGame_PutdownFieldItemReq` |
| State | **partial** |
| Request | Verified `huuu` at `0x140BCD2B0`: three lowercase `u` fields; ❓ `u` codec |
| Failure | 1250, `hh`, reason 1 |

Require zone and inventory factory kind 24. Require PC kind 1; mismatch asserts then sends failure.
Bind. Store three low-16-bit values at +152/+154/+156 and qword +160 =0.
Missing zone, null factory or failed queue sends failure.
❓ Read kind-24 action, position checks, held-item removal/world creation and shared item-use gaps.

## Command 1251 — G_PickupDeadBodyReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140379590` |
| 348 name | `HandleGame_PickupDeadBodyReq` |
| State | **partial** |
| Request | `hq`: body object selector |
| Failure | 1252, `hh`, reason 1 |

Require zone and inventory factory kind 25. Require PC kind 1; mismatch asserts then sends failure.
Bind. Store selector at +152. Missing zone, null factory or failed queue sends failure.
❓ Read kind-25 action, object/type/state/distance checks, carried-body state and shared item-use gaps.

## Shared pouch entry rules

The seven entries below use the part-1 zone queue and return true.
They do not apply T1, T2 or U0. Release local task references after posting.
No balance/item mutation, distance, cooldown or game-table lookup appears before posting.
❓ Task actions, ownership/loot rights, stack and capacity rules, item delivery, success/broadcast formats, backend/DB work and lifecycle remain open unless stated.

## Command 1296 — G_OpenItemPouchReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037DD60` |
| 348 name | `HandleGame_OpenItemPouchReq` |
| State | **partial** |
| Request | `hq`: pouch object ID; decoder result ignored |
| Failure | 1298, `hh`, reason 2 |

Require zone. Allocate 0xA0-byte task through `0x14036FEC0`.
Call `0x1405188C0(task,zone,pc,pouch_id,0)`. Require initialization and queue success. Any entry failure sends reason 2.
❓ Read initializer/action and shared pouch gaps. Do not copy the decompiler's truncated PC pointer into Rust.

## Command 1299 — G_PrivateItemPouchListReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037DEB0` |
| 348 name | `HandleGame_PrivateItemPouchListReq` |
| State | **partial** |
| Request | `hq`: selector qword; decoder result ignored |
| Failure | 1301, `hh`, reason 2 |

Require zone. Allocate 0x98-byte kind-75 task, vtable `0x140BBC978`.
Require PC kind 1; mismatch asserts then sends failure. Bind. Store selector +144. Queue failure sends failure.
❓ Resolve selector meaning, list builder, success fields/splitting and shared pouch gaps.

## Command 1303 — G_CloseItemPouchReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037E090` |
| 348 name | `HandleGame_CloseItemPouchReq` |
| State | **partial** |
| Request | `hq`: pouch object ID; decoder result ignored |
| Failure | 1305, `hh`, reason 2 |

Require zone. Allocate the same 0xA0-byte task through `0x14036FEC0`.
Call `0x1405188C0(task,zone,pc,pouch_id,1)`. Require initialization and queue success. Any entry failure sends reason 2.
❓ Read initializer/action, close effects and shared pouch gaps.

## Command 1306 — G_PickupItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037E1E0` |
| 348 name | `HandleGame_PickupItemReq` |
| State | **partial** |
| Request | Verified `hqQCc;` at `0x140BD0EA4`: pouch qword, `Q` selector, counted bytes. The selector is stored in a task byte; this does not prove its wire width. |
| Failure | 1309, `hh`: reason |

1. Decode qword/`Q` prefix. Failure sends **505**. Count must be 0..16, else **506**. Decode one byte per entry, else 505. Finish group.
2. Require zone, else **1**. Allocate 0xB0-byte kind-63 task, vtable `0x140C3CB90`. Require PC kind 1; mismatch asserts then sends 1.
3. Require pouch ID bits 48..63 in {176,32,242}. Failure sends **507 then 1**, as two separate 1309 replies.
4. Bind. Store pouch ID +144, selector +152, count byte +153 and full zero-initialized 16-byte list +154. Queue failure sends 1.

❓ Resolve byte meanings, action and shared pouch gaps. Preserve the double failure send on wrong object kind.

## Command 1311 — G_TakeContributionRewardReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037E6F0` |
| 348 name | `HandleGame_TakeContributionRewardReq` |
| State | **partial** |
| Request | `hq`: reward/pouch object selector; decoder result ignored |
| Failure | 1313, `hh`: reason |

Require zone, else **1**. Allocate 0x98-byte kind-79 task, vtable `0x140C3E728`.
Require PC kind 1; mismatch asserts then sends 1. Require selector bits 48..63 ==176; failure sends **515 then 1**.
Bind. Store selector +144. Queue failure sends 1.
❓ Read action, contribution eligibility/reward state and shared pouch gaps. Preserve both wrong-kind replies.

## Command 1316 — G_DistributeItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037E970` |
| 348 name | `HandleGame_DistributeItemReq` |
| State | **partial** |
| Request | Verified `hqsCc;` at `0x140BD1640`: pouch qword, string, counted bytes |
| Failure | Usually 1319, `hh`: reason |

1. Decode prefix with string limit 32 UTF-16 code units. Failure sends **523**. Count must be **0..15**, else **524**. Entry-byte decode failure sends 523.
2. Require zone, else **2**. Allocate 0xF0-byte kind-231 task, vtable `0x140BC14B0`. Require PC kind 1; mismatch asserts then sends 2.
3. Require pouch ID bits 48..63 in {176,32}. Failure sends **1309 `hh` reason 507, then 1319 `hh` reason 2**.
4. Bind. Store pouch ID +144; `wcsncpy(...,32)` to task +152; count byte +216; full 16-byte list +217. Queue failure sends 1319 reason 2.

The request string is not copied into this specification. Do not store account or character names as research examples.
❓ Read action, recipient lookup/rights, assignment/notification and shared pouch gaps. Preserve the unexpected 1309 reply.

## Command 1321 — G_CompleteDistributionReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037EF30` |
| 348 name | `HandleGame_CompleteDistributionReq` |
| State | **partial** |
| Request | `hq`: pouch object ID |
| Failure | 1323, `hh`: reason |

Initialize command short and pouch ID to zero. Require decoder success, else **2**. Require zone, else 2.
Allocate 0x98-byte kind-232 task, vtable `0x140BC1348`. Require PC kind 1; mismatch asserts then sends 2.
Require pouch ID bits 48..63 ==176. Failure sends **536 then 2**, as two 1323 replies.
Bind. Store pouch ID +144. Queue failure sends 2.
❓ Read action, distribution state/rights, completion/remaining-item behavior and shared pouch gaps.

## Shared ability-modification entry boundary

The next entries bind PC/zone, queue through part-1 zone queue, release local references and return true.
No direct inventory/money mutation, distance or cooldown check appears before posting.
Do not infer material/cost rules from the request's short pairs.
❓ Action paths, ownership/slot/count/money checks, table names/fields, mutation, success/broadcasts, backend/DB work and lifecycle remain open unless stated.

## Command 1398 — G_RevealItemAbilityReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037F6D0` |
| 348 name | `HandleGame_RevealItemAbilityReq` |
| State | **partial** |
| Request | Verified `hQhhcChh;` at `0x140BD1D80`: selector, two shorts, byte, counted short pairs |
| Failure | 1399, `hh`: reason |

1. Apply U0, else **575**. Decode four prefix fields, else **565**.
2. Count must be **0..15**, else **569**. Decode short pairs, else 565.
3. Require zone, else **2**. Allocate inventory factory kind 27; null sends **1**.
4. Require PC kind 1; mismatch asserts without an explicit reply in that branch. Bind.
5. Copy four prefix values to task +152/+154/+156/+158; count DWORD +160; pairs +164. Queue failure sends **1**.

❓ Resolve prefix field meanings, factory action and shared ability-modification gaps.

## Command 1400 — G_AttachEquipGemPieceReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037FB90` |
| 348 name | `HandleGame_AttachEquipGemPieceReq` |
| State | **partial** |
| Request | Verified `hQhhChh;` at `0x140BD20D8`: selector, two shorts, counted short pairs |
| Failure | 1401, `hh`: reason |

1. Apply U0, else **585**. Decode three prefix fields, else **576**.
2. Count must be **0..15**, else **580**. Decode short pairs, else 576.
3. Require zone, else **2**. Allocate inventory factory kind 28; null sends **1**.
4. Require PC kind 1; mismatch asserts without an explicit reply in that branch. Bind.
5. Copy prefix values to task +152/+154/+156; count DWORD +160; pairs +164. Queue failure sends **1**.

❓ Resolve target/piece slot meanings, factory action and shared ability-modification gaps.

## Command 1430 — G_R_EnchantToItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140380030` |
| 348 name | `HandleGame_R_EnchantToItemReq` |
| State | **partial** |
| Request | Verified `hQhq` at `0x140BD283C`: selector, short, qword. Selector uses a native byte; ❓ `Q` wire width. |
| Failure | 1432, `hh`: reason |

Require decode success, else **2**. Require short !=0, else **2563**; no positive-only test appears here.
Require zone, else 2. Allocate 0xA0-byte kind-344 task, vtable `0x140C9AE98`.
Require PC kind 1; mismatch asserts then sends 2. Bind. Store short +144, selector +146, qword +152. Queue failure sends 2.
No U0/T1/T2 gate appears. ❓ Action, qword meaning and shared ability-modification gaps remain open.

## Command 1433 — G_R_ResetEnchantItemInfoReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403802D0` |
| 348 name | `HandleGame_R_ResetEnchantItemInfoReq` |
| State | **partial** |
| Request | Verified `hQh` at `0x140BD2C3C`: selector, short. Selector uses a native byte; ❓ `Q` wire width. |
| Failure | 1435, `hh`: reason 2 |

Require decode success and zone. Allocate 0x98-byte kind-345 task, vtable `0x140C6C7B8`.
Require PC kind 1; mismatch asserts then sends failure. Bind. Store selector +144 and short +146. Queue failure sends failure.
No nonzero-slot, U0/T1/T2 gate appears here. ❓ Action and shared ability-modification gaps remain open.

## Command 1442 — G_R_PolishToItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140380A00` |
| 348 name | `HandleGame_R_PolishToItemReq` |
| State | **partial** |
| Request | Verified `hQhCc;qq` at `0x140BD3698`: selector/short, counted bytes, two qwords |
| Failure | 1444, `hh`: reason |

1. Decode two prefix fields, else **2**. Count must be 0..3, else **3**.
2. Decode each byte, else 2. Require byte 1..5. Reject any duplicate of an earlier byte. Either rejection sends **2595**.
3. Finish group. Decode two trailing fields into eight-byte native destinations, else 2. Require zone, else 2.
4. Allocate 0xB0-byte kind-348 task, vtable `0x140CE4D98`. Require PC kind 1; mismatch asserts then sends 2.
5. Bind. Store selector +144, short +146, three-byte list +148, count short +152 and trailing native values +160/+168. Queue failure sends 2.

No U0/T1/T2 gate appears. ❓ Trailing qword meanings, action and shared ability-modification gaps remain open.

## Command 1402 — G_GrowthItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140380F30` |
| 348 name | `HandleGame_GrowthItemReq` |
| State | **partial** |
| Request | Verified `hQhhhChh;` at `0x140BD38C0`: container, target short A, shorts B/C, counted short pairs |
| Failure | 1404, `hh`: reason |

1. Require cfg byte +720 !=0, else **586**. Decode four prefix fields, else **2**.
2. If container ==2, require A differs from B and C. Failure sends 2.
3. Count must be **0..15**, else **598**. Decode short pairs, else 2. For container 2, reject any pair whose first short equals A, reason 2.
4. Require zone, else 2. Allocate 0xE0-byte kind-116 task, vtable `0x140C98840`. Require PC kind 1; mismatch asserts then sends 2.
5. Bind. Store container +144, A/B/C +146/+148/+150, count DWORD +152, pairs +156. Queue failure sends 2.

No U0/T1/T2 gate appears. ❓ Field semantics, action and shared ability-modification gaps remain open. Do not infer that all material references must be unique.

## Command 1405 — G_AwakeningItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403814B0` |
| 348 name | `HandleGame_AwakeningItemReq` |
| State | **partial** |
| Request | Verified `hQhhChh;Chh;` at `0x140BD44B8`: container, shorts A/B, two counted short-pair groups |
| Failure | 1407, `hh`: reason |

1. Require cfg +720 !=0, else **586**. Decode prefix, else **2**. For container 2, A==B sends 2.
2. Each group count must be **0..15**; either invalid count sends **619**. Decode two fields per entry; failure sends 2.
3. In each group, for container 2, reject a pair whose first short equals A, reason 2. No general duplicate check appears.
4. Require zone, else 2. Allocate 0x120-byte kind-117 task, vtable `0x140D00E50`. Require PC kind 1; mismatch asserts then sends 2.
5. Bind. Store container +144, A/B +146/+148, first count DWORD +152 and pairs +156, second count DWORD +220 and pairs +224. Queue failure sends 2.

No U0/T1/T2 gate appears. ❓ Action, group meanings and shared ability-modification gaps remain open.

## Command 1408 — G_FeedingRandomStatEquipGemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140381B70` |
| 348 name | `HandleGame_FeedingRandomStatEquipGemReq` |
| State | **partial** |
| Request | Same descriptor and two-group native layout as 1405 |
| Failure | 1410, `hh`: reason |

Use the ordered entry checks for 1405, but disabled cfg +720 sends **626**, and either bad group count sends **635**.
All decode, target-alias, missing-zone, wrong-PC-kind and queue failures send 2.
Allocate 0x120-byte kind-261 task, vtable `0x140CA94E8`. Bind and copy fields at the same offsets as 1405.
No U0/T1/T2 gate appears. ❓ Feeding action, random-stat rules and shared ability-modification gaps remain open. Similar entry layout does not prove similar game results.

## Command 1445 — G_ProcessItemFusionReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403DC160` |
| 348 name | `HandleGame_ProcessItemFusionReq` |
| State | **partial** |
| Request | Verified `hChh;h` at `0x140C3D9E4`: counted short pairs, trailing short operand |
| Failure | 1447, `hE`: reason |

1. Apply U0, else **2612**. Require zone, else **2**. Both precede decode.
2. Count must be **1..3**. Decode two fields per entry. Finish group. Decode trailing short into a zero-initialized destination. Any count/decode failure sends **2598**.
3. Allocate 0x12A8-byte kind-349 task, vtable `0x140BC2860`. Require PC kind 1; mismatch asserts then sends 2.
4. Bind. Store PC +144, count +160, pairs +162, trailing short +174. Queue failure sends 2. Release reference. Return true.

No direct inventory/money mutation, distance, cooldown or table lookup appears at entry.
❓ Read fusion action, trailing operand meaning, recipes/probability, item cost/output, replies/broadcasts, backend/DB work and lifecycle.

## Command 1411 — G_TakeWarehouseItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403C1100` |
| 348 name | `HandleGame_TakeWarehouseItemReq` |
| State | **partial** |
| Request | Verified `hqqi` at `0x1408CD088`: two qwords and integer; ❓ semantic field meanings |
| Failure | 1413, `hh`: reason |

1. Require cfg +404 !=0, else **2**. Reject cfg +237 !=0 AND DWORD +248 !=0 with **5**. This is the part-1 account-inventory config gate, not full T1.
2. Resolve user from PC +20696 only when +20752 ==0. Missing user sends **1**, unlike the silent missing-user rule for part-1 account list/count. Require zone, else 1.
3. Decode without testing success. Reject `u32(value-1)<=998` with **1964**. Thus 1..999 reject; zero is not rejected by this range test.
4. Look up `tm[1455]` virtual +168 with `u64(value)|0x100000000`. Missing record sends 1964.
5. Create 0x88-byte user transaction, vtable `0x1409E6BB0`, initial DWORD +8 =663818. Register; failure sends **2**.
6. Start event **62** with decoded payload. Allocate 0xA0-byte task through `0x14018DD70`. Initialize through `0x140549BF0(task,zone,pc,1,transaction)`.
7. On task/initialization success, use part-2 alternate craft-task dispatch. On failure, send reason 2, then deliver transaction event **96** with null payload.
8. Release local task/transaction/user references. Return true.

No direct item/money change appears at entry. No NPC distance/cooldown check appears there.
❓ Transaction runner, inventory task, request semantics, table/field names, item reservation/grant/rollback, success/broadcasts, backend commands/DB procedure and callback kick paths remain open.

## Command 1581 — G_TransformItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403C1E70` |
| 348 name | `HandleGame_TransformItemReq` |
| State | **partial** |
| Request | Verified `hqChh;Chh;Chh;Chh;Qib` at `0x140C1EE58`: qword recipe key, four counted short-pair groups, `Q` mode, integer, blob |
| Failure | 1583, literal `hEq`: reason, recipe-key echo |

### Entry decode and common checks

1. Apply U0, else **2012**, echo 0. Require zone, else **2**, echo 0.
2. Decode recipe key, else **1980**, echo 0.
3. Decode four groups in order. Each count must be 0..16. Bad counts send **1987**, **1989**, **1996**, **2002**, respectively. Each entry is a short pair. Any failed entry decode sends **1980**. These failures echo recipe key.
4. Decode three trailing fields into mode byte, integer and bounded blob/native qword storage. Initial blob limit is 8. Failed decode or resulting length >8 sends 1980 with recipe key.
5. If mode !=1, allocate 0x358-byte task through `0x14011D790`. Null sends **2** with recipe key. Initialize through `0x14052CBF0` with PC/zone, key and four groups, final mode argument 0. False initialization has no additional entry reply.
6. Dispatch that task with `0x140603BD0(...,0,1)`. Release local reference. Return true.

### Mode exactly 1

Apply these extra checks in order. Failures echo recipe key except the missing-user branch noted below.

| Check | Failure reason |
| --- | ---: |
| cfg +404 ==0 | 15 |
| cfg +746 !=0 | 15 |
| cfg +237 !=0 AND DWORD +248 !=0 | 5 |
| Either global byte +1952 or +1972 at `qword_140EE58A0` is zero | 15 |
| PC byte +21080 is nonzero, read under shared lock +21072 | 1984 |
| PC qword +2216 is non-null | 1985 |
| User re-resolution fails | 2, **echo 0** |
| `tm[1599]` lookup by recipe key is null | 1981 |
| Recipe record qword +1096 is null | 1982 |
| First, third or fourth group count is positive | 2000 |

The second group may be nonempty at this stage.
Create a 0x60-byte user transaction, vtable `0x1409CEE88`, initial DWORD +8 =2760970.
Attach 0x258-byte operation object, vtable `0x140BA9518`, with recipe key +344.
Register the transaction. Failure sends 1583 reason 2 with recipe key.
Start event **274** with native trailing-value storage. ❓ Full payload layout needs instruction proof.
Allocate the same 0x358-byte task. Initialize `0x14052CBF0` with final mode argument 1.
On success, set task qword +160 =0; store/retain transaction at task +592. Dispatch with `0x140603BD0(...,0,1)`.
A null task sends 1583 reason 2 with recipe key. False initialization has no additional entry reply or explicit event to terminate the already registered transaction.
Release local references. All entry paths return true.

No direct inventory/money mutation, distance or cooldown check appears in this target.
❓ `Q`/blob codec and trailing destination mapping, task initializer/action, transaction/operation state machine, recipe table/field names, consumption/output/rollback, success/broadcasts, daemon command/DB procedure, timeout and callback kick paths remain open. Do not truncate pointers or recipe keys from decompiler casts.

## Coverage limits

All 48 selected entry functions were read. Command 833 is a complete log-only handler.
The other 47 sections remain partial. Most specify entry validation and task setup, not full game actions.
Selected BoardGacha runners/callbacks, the badge-page action and three slate actions have additional traces above.
No DB procedure name was verified. No runtime test or Rust implementation ran.

## Machine-readable states

| id | state |
| --- | --- |
| 830 | partial |
| 833 | done |
| 947 | partial |
| 1010 | partial |
| 1050 | partial |
| 1140 | partial |
| 1143 | partial |
| 1146 | partial |
| 1152 | partial |
| 1160 | partial |
| 1162 | partial |
| 1165 | partial |
| 1167 | partial |
| 1170 | partial |
| 1173 | partial |
| 1182 | partial |
| 1191 | partial |
| 1194 | partial |
| 1197 | partial |
| 1200 | partial |
| 1233 | partial |
| 1234 | partial |
| 1236 | partial |
| 1237 | partial |
| 1242 | partial |
| 1243 | partial |
| 1245 | partial |
| 1247 | partial |
| 1249 | partial |
| 1251 | partial |
| 1296 | partial |
| 1299 | partial |
| 1303 | partial |
| 1306 | partial |
| 1311 | partial |
| 1316 | partial |
| 1321 | partial |
| 1398 | partial |
| 1400 | partial |
| 1402 | partial |
| 1405 | partial |
| 1408 | partial |
| 1411 | partial |
| 1430 | partial |
| 1433 | partial |
| 1442 | partial |
| 1445 | partial |
| 1581 | partial |
