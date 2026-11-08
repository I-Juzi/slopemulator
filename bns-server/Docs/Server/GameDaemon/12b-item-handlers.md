| NEW shared helper, build 381.8867 | Purpose | Verified limit |
| --- | --- | --- |
| `0x140472550` | Inventory-task factory | Kind 7: 0x108-byte equip task, vtable `0x140CDF218`. Kind 8: 0xE8-byte unequip task, vtable `0x140D21980`. Other selected kinds appear below. |
| `0x140471800` | Bind an inventory task to zone and PC | Used before the part-1 zone task queue. ❓ Full initializer and inventory-task execution protocol remain open. |
| `0x140603BD0` | Alternate craft-task dispatch | Receives task and the pointer at task.+32→+16, or null. ❓ Queue, retry and failure behavior remain open. Do not substitute the part-1 queue without proof. |
| `0x14060EC20` | Build the caller's craft list | Shared lock at PC +39752; vector at +39760/+39768; success 1513. See command 1512. |
| `0x1405DD300` | Register-craft failure reply | Requester only; 1517, `hEi`: reason, recipe ID. |
| `0x1405DD380` | Take-craft failure reply | Requester only; 1520, `hEq`: reason, craft ID. |
| `0x1405DD410` | Abort-craft failure reply | Requester only; 1523, `hEq`: constant reason 3, craft ID. |
| `0x1404CAD00` | Resolve caller inventory container/slot | Container 1..12; nonzero zero-extended slot <= sign-extended usable limit. See shared lookup rules. |
| `0x1404F4460` / `0x1404F4630` / `0x1404F47C0` | Lock, unlock, set lock expiry | Distinct eligibility checks; client batch reasons 44/45/46 and backend item updates. |
| `0x1404F4B30` / `0x1404F4960` | Check used-item eligibility / set used-item flag | Different final flag tests. Set helper flushes per item; no rollback across a list. |
| `0x1405296E0` / `0x14050BC90` | Check NPC repair cost / apply durability repair | Eligibility and mutation differ. Mutation can skip items, but caller still debits full prior cost. |

# 12b — Item and inventory handlers, part 2

## Scope and notation

Static specification. Source: `ida/datafile/gd2020/GameDaemon.exe`, build 381.8867.
The build-348 names come from the selected rows in `parity-ledger.tsv`.
No build-348 binary was opened.
Reuse the notation, request decoder, reference release, direct-PC send endpoint, inventory batches and zone queue from part 1.
Reuse part-1 T1, T2 or account-inventory rules only when a section says so.
A native offset is not a wire offset.
`pc` means the requesting PC. `cfg` means daemon +2736.
`tm[n]` means table-manager qword slot n at `qword_140F77978`.
A format includes its command field. Case is significant.
`Q`, `E`, `C`, `;` and nested formats must not be replaced by lowercase scalar guesses.
❓ Their full codec contracts remain open unless a section gives additional proof.
A field with no recovered semantic name keeps its native role or offset.

`done` means all action paths were read. It does not mean that a port or test passed.
`partial` means the section is not a complete Rust behavior contract.
Each partial section lists its missing paths.
No live test ran. No binary or IDB was changed or saved.
No anti-cheat or licence path was intentionally followed.
For kick behavior, use `07-server-validation.md`, part 4.
An assertion or reference release is not evidence of a session kick.
No direct kick appears in the read entry or action paths below unless stated.
Unread helper, action, cleanup or callback paths can still have kick behavior.
All listed direct failure/success sends use requester virtual +592, unless a section gives another endpoint.
A delta sent through virtual +584 is not proof of a zone broadcast.
Descriptor `0x1408CFBE8` contains exactly `h` (verified bytes). Every command-only use of this descriptor below has format `h`.
No timeout or disconnect rule is inferred from failure reasons. Use doc 07 part 4 for that boundary.
No DB procedure name is verified in this file unless stated.
Do not infer a DB procedure from a client command name or backend command ID.

## Command 1325 — G_ManufactureItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14001AFE0` |
| 348 name | `-` |
| State | **done** |
| Request | No decoder. No body field is read. |

The entire routed target returns true.
It makes no state, ownership, slot, count, money, distance, cooldown or table check.
It makes no state change, reply, broadcast, daemon call, DB call or kick call.
Do not implement manufacture from this command name.
Source: full target decompilation and ledger dispatch row.

## Equipment entry rules

The four equipment entry functions below always return true.
They do not apply T1 or T2 at entry.
They use part-1 request decoding and queue `0x140254FD0`.
Successful posting is not proof of a completed equipment change.
A missing zone, null factory result or failed queue sends the listed failure command with `hh`, reason 1, to the requester.
A PC kind other than 1 at native +46 logs/asserts after task allocation; this branch has no explicit failure send.
Local task references are released after posting.
❓ Task execution, ownership, item compatibility, capacity, money, durability, cooldown, tables, success deltas, broadcasts, daemon/DB work and asynchronous failure paths remain open for all four sections.

## Command 1354 — G_EquipItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140379780` |
| 348 name | `HandleGame_EquipItemReq` |
| State | **partial** |
| Request | Literal `hQhQh`: source container selector, source slot, destination container selector, destination slot |
| Failure | 1355, `hh`: reason |

1. Decode without testing decoder success.
2. Require destination selector in {1,10,11,12}. Otherwise send reason 1.
3. Require source selector in {1,2,9,10,11,12}. Otherwise send reason 1.
4. Apply the equipment entry rules. Allocate factory kind 7.
5. Bind the task. Copy source selector/slot to task +249/+250. Copy destination selector/slot to +252/+254.
6. Set task DWORD +256 to 1 for destination 10..12, else 0. Queue the task.

No slot-range or item check appears in this entry function.
❓ Read the kind-7 inventory action protocol behind vtable `0x140CDF218`. Do not treat its virtual +72 return-true stub as the action itself.

## Command 1356 — G_EquipItemListReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140379A70` |
| 348 name | `HandleGame_EquipItemListReq` |
| State | **partial** |
| Request | Literal `hCQhh;`: counted entries, each selector plus two short fields |
| Failure | 1355, `hh`: reason |

1. Initialize the counted parser. Read its signed-short count.
2. Convert that count to an unsigned comparison. Reject values >=16 with reason 2. Thus negative counts also reject. Zero is accepted here.
3. Decode three fields per entry into six native bytes. A zero low-16-bit decode result sends reason 2.
4. Finish parser bookkeeping. Apply the equipment entry rules. Allocate kind 7.
5. Copy `6 * u8(count)` bytes to task +152. Store count at byte +248. Set DWORD +256 to 2. Queue.

The entry function does not repeat the single-item selector checks.
❓ Full counted codec, semantic names of both shorts, per-entry validation, order/atomicity and kind-7 execution remain open.

## Command 1359 — G_UnequipItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140379E30` |
| 348 name | `HandleGame_UnequipItemReq` |
| State | **partial** |
| Request | Literal `hQhQh`: source selector/slot, destination selector/slot |
| Failure | 1360, `hh`: reason |

1. Decode without testing decoder success.
2. Require source selector in {1,10,11,12}, else reason 1.
3. Require destination selector in {2,9}, else reason 1.
4. Apply the equipment entry rules. Allocate kind 8.
5. Store source selector/slot at task +152/+154. Store destination selector/slot at +156/+158.
6. Set DWORD +228 to 1 for source 10..12, else 0. Queue.

No slot-range or item check appears at entry.
❓ Kind-8 inventory action protocol behind vtable `0x140D21980` remains open.

## Command 1361 — G_UnequipItemListReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037A120` |
| 348 name | `HandleGame_UnequipItemListReq` |
| State | **partial** |
| Request | Literal `hChh;`: counted entries of two short fields |
| Failure | 1360, `hh`: reason |

1. Read signed-short parser count. Unsigned count >=16 sends reason 2. Zero passes this entry gate.
2. Decode two fields per entry into four native bytes. A zero low-16-bit decode result sends reason 2.
3. Finish parser bookkeeping. Apply the equipment entry rules. Allocate kind 8.
4. Copy `4 * u8(count)` bytes to task +160. Store count at byte +224. Set DWORD +228 to 2. Queue.

The entry function does not perform the single-item container checks.
❓ Counted codec, field meanings, mutation helpers and pending completion remain open. See the shared equipment execution below.

### Shared equipment execution — commands 1354, 1356, 1359 and 1361

Source: full actions `0x140529B10` (equip virtual +96) and `0x14052A250` (unequip virtual +96), build 381.8867.
These are **not** the shared return-true stub at virtual +72.
Resolve caller from task +32→+16.

| Ordered initial condition | Equip | Unequip |
| --- | --- | --- |
| Lookup `tm[2751]` virtual +168 by signed PC byte +364. Record exists and byte +56 !=0 | Fail 1355 `hh`, 356 | Fail 1360 `hh`, 365 |
| PC byte +464 mask 0x10 set | Same failure | Same failure |
| PC byte +888 mask 1 set | No corresponding test here | Same failure |
| PC byte +578 is 1 or 2 | Same failure | Same failure |

Any such failure calls `0x140422850` after the reply.
Do not add unequip's +888 test to equip.

**List mode (operation DWORD ==2):**

- Equip uses unsigned byte count +248. Each six-byte entry gives source container at +152, source slot at +154 and destination equipment slot at +156, with the entry stride added. Destination slot 2 or 4 sends **1358 `hEc`, reason 358, zero-based index**. Other values call `0x1404F92E0(inventory,source_container,source_slot)`. A nonzero helper reason sends 1358 with that reason and index.
- Unequip uses unsigned byte count +224. Each four-byte entry gives equipped slot at +160 and destination operand at +162. Equipped slot 2 or 4 sends **1363 `hEc`, reason 366, zero-based index**. Other values call `0x1404FD760(inventory,equipped_slot,destination_operand,0)`. A nonzero helper reason sends 1363 with that reason and index.
- Both loops continue after a failed entry. They do not roll back prior helper calls. Count zero performs no loop iterations.
- After the loop, call task virtual +208. ❓ Its completion response and cleanup are not yet read.

**Normal single equip (operation ==0):**

1. Destination slot other than 2 or 4 skips the special timing branch and calls virtual +208.
2. For slot 2 or 4, resolve the requested source container/slot. Missing item sends **1355 reason 357** and finishes.
3. Read item record qword +512, or zero if no record. Let `new_value` be that value. Let `current_value = signed PC byte +328` and `active_slot = PC WORD +3376`.
4. With task zone +112 non-null, if new_value !=0 and zone +17064 points to a record with byte +26 !=0, fail **365**.
5. Define `Z = (feature byte +932 !=0 && (zone base WORD +48 ==87 || (base WORD +48 ==77 && tm[927](base qword +56) exists && record byte +752 !=0))) || (zone +16536 non-null && its virtual +368 returns 0)`. Here zone base is `**(qword**)(zone+16224)`.
6. If Z and requested slot ==active_slot and current_value !=new_value, fail **370**.
7. Set notification parameter from PC DWORD +1940 for slot 2 or DWORD +1964 for slot 4. Default zero.
8. If requested slot ==active_slot and existing equipped item in container 1 at that slot has nonzero record qword +512, load signed feature-config DWORD +284 as delay. A positive delay starts `0x140674A00`, calls `0x14023A290(zone,&server_position,pc_id,0,0,parameter,delay/100)` and returns without immediate virtual +208.
9. Otherwise call the same notification helper with the computed delay/100, then virtual +208. With no zone, skip zone checks/notification and call virtual +208.

**Normal single unequip (operation ==0):**

1. Resolve container 1 at task source slot +154. Missing item sends **1360 reason 366** and finishes.
2. For source slot 2 or 4 and a non-null task zone, compute Z as above. Source slot ==active_slot and Z fails **370**. This branch has no new/old-value comparison.
3. Use the same per-slot notification parameter. A nonzero item-record qword +512 loads delay from feature DWORD +284. Positive delay starts `0x140674A00`, calls `0x14023A290(zone,&server_position,pc_id,1,0,parameter,delay/100)` and returns.
4. Otherwise call that notification helper, then virtual +208. Non-2/4 slots or missing zone go to virtual +208 without this notification branch.

Operation ==1, used by selectors 10..12, goes directly to virtual +208 after the initial checks.
Other nonzero operation values also reach that virtual without the operation-2 loop.
❓ Read the two virtual +208 completions, `0x1404F92E0`, `0x1404FD760`, timing/cancel handling and `0x14023A290`. Table names, final item mutation, success formats/recipients, broadcasts and daemon/DB work remain open. Notifications are not yet proved to be broadcasts.

## Command 1512 — G_RetrieveCraftListReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140397FA0` |
| 348 name | `HandleGame_RetrieveCraftListReq` |
| State | **partial** |
| Request | No decoder or body read at entry |
| Success | 1513; literal `hCqicqqQCihi;;`; requester only |

The entry calls `0x14060EC20` and returns true.
The builder creates a packet chain and uses a shared lock at PC +39752.
It visits the pointer vector at PC +39760..+39768 in vector order.
It skips null records.
It starts another packet when the current packet is absent or has fewer than 216 bytes left in the 4000-byte budget.
It emits six fields per record, starting with craft ID at record +24.
It reads record byte +144 as the state.
Only state 4 emits the nested reward entries.
It visits at most five 12-byte entries, starting at record +148.
It stops at the first entry whose DWORD +0, WORD +4 and DWORD +8 are all zero.
It finishes both counted groups and sends the chain through PC virtual +584.
A failed field composition releases the lock and destroys the chain without a direct failure reply.
No inventory or money mutation appears in this builder.
No distance, ownership, cooldown, table, daemon or DB call appears in the read builder.
❓ The six record fields after the first ID, all variadic values, nested reward wire widths and complete split bookkeeping need instruction proof. No success-field semantics are inferred from the format alone.

## Command 1515 — G_RegisterCraftReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140397FB0` |
| 348 name | `HandleGame_RegisterCraftReq` |
| State | **partial** |
| Request | `hiChh;`, confirmed in `0x1402C3660` instructions: recipe integer followed by counted short pairs |
| Entry failure | 1517, `hEi`: reason 2, recipe ID **0** |

1. Reject PC byte +13768 ==1 with the entry failure. This is not T1.
2. Resolve user from PC +20696 only when PC +20752 ==0. Require a user and user DWORD +120 ==0. Otherwise send the same entry failure.
3. Require zone. Allocate 0x3498-byte task through `0x1402C3540`, kind 274, vtable `0x140AA5F78`.
4. Initialize through `0x1402C3660`. Null zone/PC/request or PC kind !=1 logs/asserts and fails initialization.
5. Decode the first field into task DWORD +164. Require decode success and a nonzero value.
6. Read signed-short count to task +170. Reject count >9. The initializer does not explicitly reject a negative count here.
7. For positive counts, decode two fields per entry into four native bytes at task +172. Any failed decode fails initialization.
8. Finish the parser. Require recipe DWORD +164 >0.
9. Look up `tm[603]` through virtual +168. Require a non-null record. Store it at task +152.
10. Set DWORD +160 to 1. Bind zone/PC and dispatch through `0x140603BD0`. Release the local task reference.

Every entry allocation/initialization failure sends reason 2 with recipe ID 0, even if a nonzero recipe was decoded.
No item or money change occurs before dispatch in this read prefix.
❓ Read task runner `0x1402C4F20`, recipe table/field names, consumption, cost, deadlines, completion, rollback, success reply and backend/DB path. Confirm request literal and signed-count handling in the codec.

## Command 1518 — G_TakeCraftReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140398230` |
| 348 name | `HandleGame_TakeCraftReq` |
| State | **partial** |
| Request | `hq`: craft ID |
| Entry failure | 1520, `hEq`: reason 2, requested craft ID |

1. Initialize craft ID to zero. Decode without testing decoder success.
2. Require zone. Allocate 0x34F8-byte task through `0x1402C5BD0`, kind 275, vtable `0x140B24A90`.
3. Require craft ID !=0. Require PC kind +46 ==1; wrong kind logs/asserts and fails initialization.
4. Store DWORD +152 =1 and craft ID at +160. Bind PC/zone. Dispatch through `0x140603BD0`.
5. Release local task reference. All entry paths return true.

No T1, T2, user-state, ownership, deadline or capacity check appears in this entry.
❓ Runner `0x1402C5CF0`, craft lookup/state checks, item grant, success 1519, inventory replies, rollback and daemon/DB work remain unread.

## Command 1521 — G_AbortCraftReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403984A0` |
| 348 name | `HandleGame_AbortCraftReq` |
| State | **partial** |
| Request | `hq`: craft ID |
| Immediate failure | 1523, `hEq`: reason **3**, requested craft ID |
| Backend request | 367, `hqiiqqq` |

1. Initialize ID to zero. Decode without testing decoder success. Zero ID sends immediate failure.
2. Acquire exclusive lock at PC +39752. Read `time64()` once.
3. Scan the craft pointer vector at +39760..+39768. Skip null records and nonmatching IDs at record +24.
4. For a matching record, state byte +144 !=3 stops the scan with failure.
5. A matching state-3 record passes only when `record.qword[56] <= now < record.qword[64]`. If its time test fails, the scan continues.
6. On the first passing record, set state byte +144 to **6** before the backend request. Release the lock.
7. Build backend command 367. Fields: PC +20664, signed short PC +96 promoted to integer, server restriction value, PC object ID +40, PC identity +20744, craft ID.
8. The restriction value is cfg DWORD +248 only when cfg byte +237 !=0 and that DWORD is nonzero; otherwise 0.
9. Only successful packet composition calls `0x14032AB10(daemon+3112,pc,packet)`. Destroy the local packet afterward.
10. If no record passed, send immediate failure. Return true.

There is no rollback of state 6 in this function when composition or sending fails.
No T1, T2, slot, money, distance or table gate appears here.
❓ Backend response handler, success 1522, later state repair, endpoint routing, DB procedure and send-failure behavior remain open. Do not add a refund or immediate success packet.

## Command 1524 — G_JoinCraftUnionReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140398760` |
| 348 name | `HandleGame_JoinCraftUnionReq` |
| State | **partial** |
| Request | `hcq`: signed union selector, signed 64-bit target ID |
| Entry failure | 1526, `hEc`: reason, union selector |

1. Initialize selector and ID to zero. PC byte +13768 ==1 sends reason 7 with selector 0 **before decode**.
2. Decode without testing success. Require zone, else assert/log and send reason 2 with the decoded selector.
3. Allocate 0xA8-byte task through `0x1402C6D90`, kind 276, vtable `0x140B20730`.
4. Require signed selector >0 and target ID >0.
5. Reject selectors 7..14 by `u8(selector-7) <=7`.
6. Require PC kind +46 ==1. Wrong kind logs/asserts. Any allocation/initialization failure sends reason 2.
7. Store task DWORD +152 =1, selector byte +156 and target qword +160. Bind zone/PC. Dispatch through `0x140603BD0` and release local reference.

No union membership, distance, money, cooldown or table check appears in this entry.
❓ Runner `0x1402C6E40`, membership limits, state changes, success 1525, broadcasts, backend request and DB procedure remain unread.

## Command 1527 — G_LeaveCraftUnionReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140398AD0` |
| 348 name | `HandleGame_LeaveCraftUnionReq` |
| State | **partial** |
| Request | `hc`: signed union selector |
| Entry failure | 1529, `hEc`: reason, union selector |

1. Initialize selector to zero. PC byte +13768 ==1 sends reason 7 with selector 0 before decode.
2. Decode without testing success. Require zone, else assert/log and send reason 2.
3. Allocate 0xA0-byte task through `0x1402C7C50`, kind 277, vtable `0x140AAA7B8`.
4. Require signed selector >0 and PC kind +46 ==1. Wrong kind logs/asserts. Allocation/initialization failure sends reason 2.
5. Unlike join, this entry has no explicit 7..14 exclusion.
6. Store task DWORD +152 =1 and selector byte +156. Bind and dispatch through `0x140603BD0`. Release local task reference. Return true.

No membership, active-craft, money, distance, cooldown or table check appears in this entry.
❓ Runner `0x1402C7D00`, removal restrictions, state changes, success 1528, broadcasts and daemon/DB paths remain unread.

## Shared inventory lookup and lock updates

NEW helper `0x1404CAD00` resolves a container/slot in the caller's inventory.
It accepts container bytes 1..12 and rejects slot 0.
Instructions zero-extend the wire/native low-16-bit slot into a 32-bit temporary.
They call part-1 usable-slot limit `0x1405114E0`, sign-extend its low-16-bit result and make a signed 32-bit `jg` comparison.
The exact slot test is `u16(slot) !=0 && int(u16(slot)) <= int(i16(limit))`.
High-bit slot values become 32768..65535. They are not negative array indices.
It reads `*(qword*)(*(qword*)(inventory +32*container +8) +8*u16(slot))`.
Source: full `0x1404CAD00` instructions, build 381.8867.
This lookup is not proof of a universal ownership rule outside the caller's container arrays.

Lock/update helpers use the part-1 inventory delta and backend batch functions.
They send a nonempty client batch through owner virtual +584.
They flush a nonempty backend batch through `0x140314680`, backend command 17 as specified in part 1.
The batch reasons below are **not** proved client command IDs.
❓ Client delta command/format, concrete item virtual setters, table names/fields and DB procedure remain open.
No money, stack-count, distance or cooldown check appears in these read lock helpers.

## Command 1414 — G_LockItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403C94A0` |
| 348 name | `HandleGame_LockItemReq` |
| State | **partial** |
| Request | `hQh`: container selector, slot |
| Failure | 1415, `hh`: reason; requester only |
| Task | Kind 177; 0x98 bytes; vtable `0x140D12DD8`; runner `0x14053BC80` |

1. Require daemon +2744 byte +909 !=0. Failure sends reason 2 before decode.
2. Require nonzero low-16-bit decoder result. Failure sends reason 2.
3. Require zone and PC kind +46 ==1. Wrong kind logs/asserts. Missing zone, wrong kind or failed part-1 queue sends reason 1.
4. Bind task. Store container byte +144 and slot short +146. Post; release local reference; return true.
5. Runner checks PC byte +888 mask 1. If set, send reason 2040. Then require PC byte +578 ==0, else 2041.
6. Call `0x1404F4460(inventory,container,slot)`. A false result sends reason 2.
7. That helper requires container in {1,2}. Resolve item through `0x1404CAD00`.
8. Require item non-null, item byte +24 ==0, item virtual +104 kind !=3, item-record byte +1936 !=0, and item virtual +304 ==0, in this order.
9. Call item virtual +312. Queue item delta with batch reason **44** and backend item update. Flush both nonempty batches.

No separate command-only success appears in the runner/helper.
❓ Read concrete virtual +112/+304/+312, the record table and field name, generic task lifecycle, lookup comparison and delta codec. No DB procedure or broadcast audience beyond the owner is proved.

## Command 1416 — G_UnlockItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403C9780` |
| 348 name | `HandleGame_UnlockItemReq` |
| State | **partial** |
| Request | `hQh`: container selector, slot |
| Failure | 1417, `hEi`: reason, remaining-time integer; requester only |
| Task | Kind 178; 0x98 bytes; vtable `0x140C649D8`; runner `0x14053BD10` |

1. Require the same feature byte as lock. Require successful decode. Either failure sends reason 2 and remaining time 0.
2. Require zone and PC kind 1. Wrong kind logs/asserts. Missing zone, wrong kind or failed queue sends reason 2 and time 0.
3. Store container at task +144 and slot at +146. Bind and queue. Release local reference. Return true.
4. Runner resolves the item through `0x1404CAD00`; missing item sends reason 2, time 0.
5. Read `now = time64()`, then item virtual +328 as expiry.
6. If `now < expiry`, send reason **2043** and `i32(expiry)-i32(now)`. Equality permits the next step.
7. Call `0x1404F4630`. Require non-null item, item byte +24 ==0 and item virtual +304 !=0. False sends reason 2, time 0.
8. Call item virtual +320. Queue item delta with batch reason **45** and backend update. Flush nonempty batches.

Unlike lock, this runner does not test PC +888 or +578.
Unlike the lock helper, this helper does not explicitly restrict the container to {1,2}.
No separate direct success packet appears.
❓ Concrete expiry/lock getters and clear setter, slot comparison, feature policy, delta codec, task lifecycle and DB procedure remain open. Preserve the 32-bit reply subtraction rather than inventing a clamp.

## Command 1418 — G_SetItemLockExpirationTimeReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403C9A80` |
| 348 name | `HandleGame_SetItemLockExpirationTimeReq` |
| State | **partial** |
| Request | `hQh`: container selector, slot; **no client expiry field** |
| Failure | 1419, `hh`: reason; requester only |
| Task | Kind 179; 0x98 bytes; vtable `0x140D07430`; runner `0x14053BE30` |

1. Require the same feature byte and successful decode as lock. Failure sends reason 2.
2. Require zone and PC kind 1. Wrong kind logs/asserts. Missing zone, wrong kind or failed queue sends reason 1.
3. Bind and queue container/slot at task +144/+146. Release local reference. Return true.
4. Runner requires PC +888 mask 1 clear, else 2044. Require PC +578 ==0, else 2045.
5. Require container in {1,2}, else reason 2. Resolve item; missing item sends reason 2.
6. Compute expiry as `time64() + signed_i32(daemon.+2744→+912)`.
7. Call `0x1404F47C0`. It repeats container {1,2}, item presence, byte +24 ==0 and virtual +304 !=0.
8. Call item virtual +336 with the computed 64-bit expiry. Queue item delta with batch reason **46** and backend update; flush.
9. A false helper result sends reason 2. No direct success appears.

Do not accept a client-selected expiry or add a nonnegative-duration check.
❓ Concrete setter, configuration field name/units, slot comparison, batch codec, task lifecycle and DB procedure remain open.

## Command 1426 — G_SetItemUsedReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D7050` |
| 348 name | `HandleGame_SetItemUsedReq` |
| State | **partial** |
| Request | `hCQh;`: counted container/slot pairs |
| Failure | 1427, `hh`: reason; requester only |
| Task | Kind 315; 0xD8 bytes; vtable `0x140D0DAB0`; runner `0x14053BF30` |

1. Read a signed-short count. Require 1..16, inclusive. Decode two fields per entry into four native bytes. Count/decode failure sends reason 2.
2. Require zone and PC kind 1. Wrong kind logs/asserts. Missing zone, wrong kind or failed queue sends reason 2.
3. Store count at task DWORD +144. Copy the full 64-byte initialized entry array to +148. Bind and queue. Release reference. Return true.
4. Runner checks PC +888 mask 1, failure 2046; then PC +578, failure 2047.
5. First pass: call `0x1404F4B30` for every pair, in request order. Stop at first false result and send 2048. No setter has run yet.
6. Eligibility helper accepts container set **{1,2,3,6,8}**, from bit mask 334. It requires a non-null item and non-null item record, byte +24 ==0, virtual +104 kind !=3, record byte +52 !=0, and virtual +288 !=1.
7. Second pass: call `0x1404F4960` in request order. It repeats eligibility and lookup. It further requires record byte +52 !=0 and virtual +288 ==0.
8. On success for that item, call virtual +296 with byte 1. Queue item delta with batch reason **69** and backend update. Flush each item's batches before the next item.
9. A second-pass false result stops the loop and sends 2048. Earlier successful mutations and flushes are **not rolled back** here.

There is no uniqueness check. Repeated references can pass the first pass and then fail after the first mutation.
No separate success command appears in the read runner/helper.
❓ Concrete virtual +288/+296, record table/field name, batch codec, task lifecycle and DB procedure remain open. Shared lookup instructions resolve slot extension as stated above. Test duplicate pairs and later-entry failure before asserting atomicity.

## Command 934 — G_PutItemIntoClosetReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403C9F00` |
| 348 name | `HandleGame_PutItemIntoClosetReq` |
| State | **partial** |
| Request | `hh`: inventory slot, decoded as a short |
| Entry failure | 936, `hh`: reason |
| Task | Kind 189; 0xC0 bytes; constructor `0x1402AB200`; vtable `0x140AA9180` |

1. Initialize slot to zero. Decode through the stateful parser. A zero low-16-bit result sends reason **2168**.
2. Require zone, else reason 2.
3. Allocate task. Require PC kind 1 and decoded slot >0. Wrong kind logs/asserts. Allocation/initialization failure sends reason 2.
4. Store DWORD +144 =1 and slot short +148. Bind zone/PC. Dispatch through `0x140603BD0`. Release local reference. Return true.

No T1, T2, item, closet capacity, money, distance, cooldown or table check appears in this entry.
❓ Closet action runner, ownership and item eligibility, removal/collection rules, success 935, broadcasts, inventory updates and daemon/DB work remain unread.

## Command 937 — G_TakeItemFromClosetReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403CA200` |
| 348 name | `HandleGame_TakeItemFromClosetReq` |
| State | **partial** |
| Request | `hih`: signed closet-item selector, short destination slot |
| Entry failure | 939, `hh`: reason |
| Task | Kind 190; 0xD8 bytes; constructor `0x1402AC7B0`; vtable `0x140B0B650` |

1. Initialize both body fields to zero. Decode through the stateful parser. Failure sends **2173**.
2. Require zone, else 2. Allocate task.
3. Require PC kind 1, signed selector >0 and destination slot >=1. Wrong kind logs/asserts. Allocation/initialization failure sends 2.
4. Set task DWORD +144 =1, selector DWORD +148, destination short +152. Bind and dispatch through `0x140603BD0`. Release reference. Return true.

No closet membership, count, free-slot, money, distance, cooldown or table check appears at entry.
❓ Action runner, destination container, item creation/removal, success 938, inventory deltas, broadcasts, backend commands and DB procedure remain unread.

## Command 1118 — G_SetClosetItemUsedReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D74F0` |
| 348 name | `HandleGame_SetClosetItemUsedReq` |
| State | **partial** |
| Request | `hCi;`: counted signed 32-bit closet selectors |
| Entry failure | 1120, `hh`: reason 2 |
| Task initializer | `0x1402D1CB0`; 0xD8-byte task from `0x1402D1C20` |

1. Require zone. Allocate the task. Initialize it with zone, PC and request.
2. Initializer requires non-null zone/PC/request and PC kind 1.
3. Read count to task short +148. Require 1..16 inclusive.
4. Decode one integer per entry to task +152 with four-byte stride. Failed decode rejects initialization.
5. Finish parser bookkeeping. Set task DWORD +144 =1. Bind zone/PC.
6. Dispatch through `0x140603BD0`. Release reference. Return true.
7. Missing zone, null task or failed initialization sends 1120 reason 2.

This prefix does not test each selector's sign, closet membership or duplicate status.
❓ Task action, per-selector checks, mutation, success 1119, broadcasts, tables and daemon/DB work remain unread.

## Command 1121 — G_CollectingListReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D7650` |
| 348 name | `HandleGame_CollectingListReq` |
| State | **partial** |
| Request | No decoder or body read |
| Failure | 1123, `hh`: reason 2 |
| Success | 1122, literal `hiCiq;`; requester only |
| Task | Kind 313; 0x90 bytes; vtable `0x140BC2058`; runner `0x1402D1470` |

1. Require zone, else failure. Bind the task to zone/PC. Queue through part-1 queue. Failed queue sends failure.
2. Release local reference and return true.
3. Runner calls `0x14007FD60(pc+20496)`.
4. Builder traverses the first pointer vector in that collection object, skipping null records. It uses `0x14007F250` to encode each record.
5. Start another packet when less than 12 bytes remain in the 4000-byte budget. Maintain the counted-group state and send the chain through PC virtual +584.

No collection mutation, money, distance, cooldown, table or backend/DB call appears in this read builder.
❓ Top-level integer, per-record integer/qword meanings, serializer `0x14007F250`, split metadata and generic task lifetime remain open.

## Command 1125 — G_FishingListReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D7940` |
| 348 name | `HandleGame_FishingListReq` |
| State | **partial** |
| Request | No decoder or body read |
| Failure | 1127, `hh`: reason 2 |
| Success | 1126, literal `hiChiiq;`; requester only |
| Task | Kind 318; 0x90 bytes; vtable `0x140BC1988`; runner `0x1402D2390` |

1. Require zone, else failure. Bind the task. Queue through part-1 queue. Queue failure sends failure.
2. Release local reference and return true.
3. Runner calls `0x1400B98C0(pc+20568)`.
4. Builder traverses that object's ordered tree in key order. Record pointer is node qword[5]. Skip null records.
5. Encode each record through `0x1400B8E90`. Split when fewer than 18 bytes remain in the 4000-byte budget. Send the final chain through PC virtual +584.

No list mutation, money, distance, cooldown, table or backend/DB call appears in the read builder.
❓ Top-level integer, per-entry field meanings, serializer `0x1400B8E90`, exact split bookkeeping and generic task lifetime remain open.

## Command 1428 — G_CompleteClosetCollectingReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403D77E0` |
| 348 name | `HandleGame_CompleteClosetCollectingReq` |
| State | **partial** |
| Request | Literal `hiCQQhQi;`: collection integer and counted five-field entries |
| Entry failure | 1429, `hEi`: reason 2; ❓ trailing integer argument at entry failure sites is not recovered |
| Task initializer | `0x1402D1560`; 0x3540-byte task from `0x1402D14A0` |

1. Require zone. Allocate task. Initializer requires non-null zone/PC/request and PC kind 1. Invalid invariants log/assert.
2. Decode collection integer to task +148. Require decode success and signed value >0.
3. Decode signed-short count to task +152. Unsigned comparison rejects values >8, including negative counts. Zero passes.
4. Decode five fields per entry to task +156 with 12-byte native stride. Failed decode rejects initialization.
5. Finish parser bookkeeping. Set task DWORD +144 =1. Bind zone/PC. Dispatch through `0x140603BD0`.
6. Release local reference. Return true. Missing zone, allocation or initialization failure uses the entry failure.

❓ Five entry meanings, task action, completion ownership/requirements, item consumption/rewards, full success/failure packets, broadcasts, table fields and daemon/DB work remain unread. Do not infer that the entry failure's absent variadic integer is zero.

## Command 1331 — G_InventoryItemListReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140374BA0` |
| 348 name | `HandleGame_InventoryItemListReq` |
| State | **partial** |
| Request | No decoder or body read |
| Entry failure | 1333, `hh`: reason 1 |
| Task | Kind 59; 0x90 bytes; vtable `0x140BC1880`; runner `0x140517D40` |

1. Require zone, else failure. Require PC kind 1 after allocation. Wrong kind logs/asserts without an explicit reply in that branch.
2. Bind and queue through part-1 queue. Failed queue sends failure. Release local reference. Return true.
3. Runner creates an inventory client batch with reason **0**.
4. Visit containers 1 through 12 in ascending order. For each container, visit slots 1 through part-1 usable-slot limit, inclusive. Skip null pointers.
5. Append each item through part-1 `0x1404C8CF0`. Finalize the batch. Send only when the batch entry count is positive, through owner virtual +584.

The runner performs no inventory mutation, money, distance, cooldown or table check. It makes no backend update.
❓ Client batch command, item-kind serialization, empty-list behavior inside batch construction, packet splits and generic task lifecycle remain open. Do not assume a direct 1332 command from the request name alone.

## Command 1334 — G_DisposeItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140375490` |
| 348 name | `HandleGame_DisposeItemReq` |
| State | **partial** |
| Request | `hhh`: bag slot, requested short count |
| Failure | 1335, `hEq`: reason, item-record identity or zero |
| Task | Kind 65; 0x98 bytes; vtable `0x140A97EA8`; initializer `0x14051D3A0`; runner `0x14051D470` |

1. Decode without testing success. Require zone and task allocation. Missing zone/allocation or failed queue sends reason 1 and qword 0.
2. Initializer requires PC kind 1, else logs/asserts and returns false. The entry sends no extra failure when initialization alone returns false.
3. Store slot/count at task shorts +144/+146. Bind and queue. Release local reference. Return true.
4. Runner rejects PC byte +578 in {1,2} with reason **288**, qword 0.
5. Resolve slot in **container 2** through `0x1404CAD00`. Missing item or item record sends **289**, qword 0.
6. Item byte +24 ==1 sends 289, qword 0.
7. For item kind virtual +104 ==1, inspect eight `0x14055EF60(item,index)` results. Each must be zero or 65001..65011. Failure sends **290**, qword 0.
8. Read record qword +8 for later failure identity. Require native item ID +16 !=0, record byte +43 ==0, virtual +144 container ==2, and virtual +160 ==0. Otherwise send **288** with that record qword.
9. Capture item metadata for audit. This includes virtual +192/+208/+432, additional kind-specific fields and config +1379 lifetime metadata. These reads precede disposal.
10. Call `0x1404D84E0(inventory,1,&slot,2)`. False sends **287** and the record qword.
11. Call `0x1404D8660(inventory,1,&slot,2,0,&output)`. False logs/asserts, then sends reason **2** and the record qword if the assertion returns.
12. Success can call audit `0x14043D590` when cfg +537 !=0 and PC kind 1. An optional service at daemon +2728→+32 also receives an audit call.

The requested count is stored and used by diagnostic/audit code. The two action calls above receive a slot-array pointer and no separately recovered count argument. Do not invent partial-stack disposal behavior.
❓ Read `0x1404D84E0` and `0x1404D8660` before asserting full-stack/count rules, removal, deletion, replies or backend effects. Read concrete item virtuals, record table/field names, audit effects and DB procedure. No direct success command is proved by the read runner.

## Command 1336 — G_MoveItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140375620` |
| 348 name | `HandleGame_MoveItemReq` |
| State | **partial** |
| Request | `hQhhQhq`: source selector, source slot, count short, destination selector, destination slot, qword token |
| Failure | 1337, `hh`: reason |
| Task | Kind 66; 0xC0 bytes; vtable `0x1409CEDA0`; runner `0x14051E330` |

1. Decode without testing success. Require zone and allocated task. Missing zone, null task or failed queue sends reason 1.
2. Require PC kind 1; wrong kind logs/asserts without an explicit reply here.
3. Bind task. Set operation DWORD +144 =0. Store source selector/slot +168/+170, destination selector/slot +172/+174, count +176 and token +184. Queue and release reference. Return true.
4. Runner first selects cfg byte +1534.
5. With that byte zero: call validator `0x14051E810`. Nonzero short result is a 1337 failure reason. Otherwise call `0x1404F55C0(inventory,source,slot,destination,destination_slot,count)`. A nonzero short result also sends 1337 with that reason.
6. With cfg +1534 !=0: run the same validator, then operation 0 calls `0x14051EC20`. Later operation states 1..4 process an asynchronous path.
7. Later states can restore task fields from PC +13776, clear a 0x4B0-byte PC area at +13776, call `0x14051EB20` and complete through `0x14051EBA0`.
8. Visible completion code distinguishes status shorts 67,68,69,71. Status 69 maps to reason 11; status 71 maps to 12; unexpected status maps to 2. Invalid operation maps to 1.
9. One state-1 branch calls `0x14014C670(daemon+4928,pc.s16[96],0,pc.identity,cfg.byte[1536])`. Its false result clears the pending area and sends reason 2.

❓ Validators, direct move/merge/count semantics, asynchronous setup/callbacks, token ownership, status meanings, config field names, success inventory replies, broadcasts, daemon routing and DB procedure remain unread. The state-machine details above are not a complete transaction contract.

## Command 1338 — G_MoveItemListReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403758B0` |
| 348 name | `HandleGame_MoveItemListReq` |
| State | **partial** |
| Request | `hqQQChhh;Chhh;`: qword token, two container selectors, two counted groups of short triples |
| Entry failure | 1337, `hh`: reason |
| Action failure | 1340, `hEc`: reason, entry index |
| Task | Kind 67; 0x540 bytes; vtable `0x140CA2DA8`; runner `0x14051F100` |

1. Initialize the three prefix fields to zero. Decode them. Failure sends **292**.
2. Read first signed-short count; unsigned comparison requires <=96. Invalid count, including negative, sends **293**. Zero passes.
3. Decode three fields per first-group entry to six native bytes. Failure sends 292. Finish that counted group.
4. Repeat for second count/group, with the same bounds and failure reasons.
5. Require zone. Allocate task and require PC kind 1. Wrong kind logs/asserts. Missing zone, allocation, wrong kind or failed queue sends reason 2.
6. Store token +1328; selector bytes +1336/+1337; first list +168 with short count +744; second list +746 with count +1322. Set operation DWORD +144 =0. Bind, queue and release reference. Return true.
7. Runner selects cfg +1534, as for single move. It calls validator `0x14051F6A0`; nonzero result sends 1340 with that reason and index 0.
8. With cfg +1534 zero, process the first group in order through `0x1404F55C0`. Each triple supplies source slot, count and destination slot, in that native order. The two prefix selectors apply to both groups.
9. A first-group move failure sends 1340 with the failing zero-based index and stops. Earlier moves are not rolled back here.
10. Then process the second group. A failure sends 1340 with that group's zero-based index but **continues** to later entries.
11. After the second-group loop, send command **1339** with a command-only descriptor at `0x1408CFBE8`. This send also occurs after second-group failures.
12. With cfg +1534 nonzero, operation 0 calls `0x14051FFA0`. Later states use saved PC list fields, clear the 0x4B0-byte pending area, call `0x14051FC90`, and can complete through `0x14051FE10`. Visible status mappings include 69→11 and 71→12.

❓ Read both validators, all move helpers, full asynchronous branch, token checks, item/money rules, success deltas, callbacks, daemon routing and DB procedure. The command-only descriptor is verified as `h` in the scope notes. Do not impose atomic all-or-nothing behavior on the observed direct branch.

## Command 1328 — G_ExpandInventoryReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037F2A0` |
| 348 name | `HandleGame_ExpandInventoryReq` |
| State | **partial** |
| Request | `hQqChh;`: container selector, NPC object ID when required by the action branch, counted short pairs |
| Usual entry failure | 1330, `hh`: reason 1 |
| Count/entry decode failure | **1344**, `hh`: 349 for invalid count; 348 for entry decode |
| Task | Inventory factory kind 26; 0xF0 bytes; vtable `0x140CDC530` |

1. Initialize prefix values and 16-pair storage to zero. Decode two prefix fields. Failure sends 1330 reason 1 and logs/asserts.
2. Read count. Require 0..15. Negative or >=16 sends **1344 reason 349**, not 1330.
3. Decode two fields per pair into four native bytes. Failure sends **1344 reason 348**. Finish parser bookkeeping.
4. Require zone and non-null kind-26 task. Missing zone, null task or failed queue sends 1330 reason 1.
5. Require PC kind 1; wrong kind logs/asserts without a direct failure in that branch.
6. Bind task. Store selector byte +152, token qword +160, pairs +168 and count DWORD +232. Queue and release reference. Return true.

### Inventory action `0x14053B0A0`, virtual +96

1. Require container in {2,3,6,8}, else 1330 reason **1**.
2. Container 8 also requires `0x1405EC080(pc,58)>0`, else **564**. ❓ Property/skill 58 meaning.
3. For containers {3,6,8}, if daemon byte +86857 is 2 or 10 and cfg byte +237 and DWORD +248 are both nonzero, send reason **5**. This is a local condition, not the complete part-1 T1 check.
4. For those same containers, `0x1405EC080(pc,25)>0` or `0x1405ED890(pc)!=0` bypasses the NPC requirement.
5. Otherwise require the request qword to identify a zone object of kind 4. Require its template pointer +3456 and template byte +235 !=0. Require squared signed-short server 3D distance <=10000. Any failure sends **558**.
6. Container 2 bypasses this NPC branch.
7. Map container 2→expansion index 0; 3→1; 6→2; 8→3.
8. Call `0x1404E41C0(inventory,index,1,count,pairs)`. A nonzero result sends 1330 `hh` with that reason. Zero sends **1329 `hc`**, expansion index, to requester.
9. Finish the inventory task through `0x140422850`, including the reject paths.

The request qword is an NPC object ID in the branch above, not a proved payment token.
❓ Read `0x1404E41C0`, helpers for selectors 25/58 and bypass policy, material pair meanings, expansion maxima, payment items/money, data tables/field names, inventory deltas/broadcasts, backend commands, DB procedure and task cleanup. Preserve the two unexpected entry failure command IDs.

## Command 1385 — G_RepairReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037CA50` |
| 348 name | `HandleGame_RepairReq` |
| State | **partial** |
| Request | `hqCQh;`: NPC object ID, counted container/slot pairs |
| Failure | 1386, `hh`: reason; requester only |
| Task | Kind 72; 0x1C0 bytes; vtable `0x140B948F8`; runner `0x140528D80` |

### Entry and ordered validation

1. Resolve user only when PC +20752 ==0. Require non-null user and user DWORD +120 ==0. Otherwise send **455** before decode.
2. Decode NPC ID. Failure sends **448**. Read count; require 0..96 inclusive, else **449**.
3. Decode each container byte and slot short into separate arrays. Any decode failure sends 448.
4. Require zone, else 2. Allocate task. Require PC kind 1; wrong kind logs/asserts without an explicit reply in that branch.
5. Store NPC ID +144, count DWORD +152, container bytes +156 and slot shorts +252. Bind and queue. Failed queue sends 2. Release reference. Return true.
6. Runner resolves NPC ID in the caller's zone map. Require object kind +46 ==4 and its template pointer +3456→byte +224 !=0. Failure sends **450**.
7. Use signed-short server x/y/z. Require squared 3D distance **<=10000**, else **451**. Equality passes. This differs from trade's strict boundary.
8. Run cost/eligibility helper `0x1405296E0`. Failure sends **453**.
9. Require accumulated signed 64-bit cost <= PC money +312. Failure sends **454**.

### Cost/eligibility helper

Visit input pairs in request order. Zero count passes with cost zero.
For each entry:

1. Require container in {1,2}. Resolve item through `0x1404CAD00`. Require non-null item record; missing record logs/asserts and fails.
2. If cfg byte +1379 !=0 and signed record byte +1937 >1: require virtual +360 ==1 **or** `now < virtual +376`. Otherwise fail.
3. Require current durability virtual +240 < maximum virtual +232. Full durability rejects the whole request.
4. For kind virtual +104 ==1, require record byte +2195 !=0.
5. For kind 1, reject record byte +2008 in {11,12}.
6. Add `(maximum - current) * signed_i32(record+144)` to the 64-bit total. Store the pair in the output arrays. No duplicate check appears.

❓ Record table and named cost/repairability fields remain unknown. Preserve the native signed conversions; the exact virtual return types need concrete implementation proof.

### Mutation and notifications

1. Runner snapshots old durability for audit, then calls `0x14050BC90` on the accepted pairs.
2. That helper creates client batch reason **15** and backend batch with flag **1**.
3. It processes every pair. Missing item, virtual +160 !=0, item byte +24 ==1, or kind-1 record +2008 in {11,12} logs and **skips that item**, rather than returning a failure to the runner.
4. Otherwise call item virtual +248 with its virtual +232 maximum. Queue item delta/backend update. Continue to the next pair.
5. Flush nonempty client/backend batches. Audit under cfg +537 through `0x140440E20`. Optional cfg +436 calls `0x140477F40` with old/new durability.
6. After the repair helper returns, the runner debits the **precomputed full cost** through part-1 money helper `0x140267EA0`, reason **521**, flag 0, and flushes the property delta. It does not recompute cost for skipped items.
7. Further cfg +537 audit uses `0x140441200`; optional daemon +2728→+32 audit uses `0x140479120`.

No separate direct success command appears in the read runner.
❓ Concrete durability setters, property/batch packets, overflow behavior, audit effects, generic task lifecycle and DB procedure remain open. The checks above do not establish atomic repair-and-payment semantics. See doc 07 for the earlier NPC validation findings and part-4 kick boundary.

## Command 1387 — G_RepairWithCampfireReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037CEE0` |
| 348 name | `HandleGame_RepairWithCampfireReq` |
| State | **partial** |
| Request | `hqh`: qword campfire selector, short item slot |
| Entry failure | 1388, `hh`: reason |
| Task | Inventory factory kind 9; 0xC8 bytes; vtable `0x140CB1740` |

1. Apply the repair user-state gate. Failure sends **464** before decode.
2. Decode without testing success. Require zone, else reason 2.
3. Allocate kind-9 task and require PC kind 1. Wrong kind logs/asserts. Allocation, kind or queue failure sends reason 2.
4. Bind task. Copy qword to +152 and slot short to +160. Zero short +162, qwords +176/+184 and bytes +192..+197 as shown in the initializer.
5. Queue through part-1 queue. Release reference. Return true.

### Start action `0x14052B130`, virtual +96

The client short is a **material slot in container 2**.
The repair target is **equipped container 1, slot 1**.
All failures below send 1388 `hh` to requester and finish through `0x140422850`.

1. PC byte +1898 !=0 sends **456**.
2. Require target item and kind virtual +104 ==1, else **457**. Require weapon-record byte +2195 !=0, else **465**.
3. If cfg byte +1379 !=0 and signed record byte +1937 >1, require virtual +360 ==1 or `time64() < virtual +376`, else 457.
4. Reject weapon-record byte +2008 in {11,12} with 465.
5. Require maximum durability virtual +232 >0 and deficit `(maximum - virtual +240)>0`, else **460**.
6. Resolve material in container 2 at task short +160. Require kind virtual +104 ==3, item byte +24 !=1 and material-record byte +2008 ==1, else **458**.
7. Let L = weapon-record WORD +2196. If material-record WORD +2110 is positive, require minimum <=L. If material-record WORD +2112 is positive, require L<=maximum. Failure sends 457.
8. Required material count starts at weapon-record WORD +2198, stored in task short +162. For a positive count, let reduction = `0x1405EC080(pc,35)`. When reduction >0, subtract `(reduction * required +99)/100`. If the result <=0, set required to 1.
9. Require signed-short required count <= material virtual +216 signed-short count, else 457.
10. If material-record byte +2120 !=0, resolve request campfire ID from the zone object list at task +112→+16392. Missing object or false `0x140262770(pc,campfire,...)` sends **459**. The helper's distance/type policy is not yet read.
11. Set material byte +24 =1. This reserves the material before delayed completion.
12. Compute and store duration DWORD +176. The decompiler gives `100 * ((int(feature_cfg_i32[+552] + (maximum + material_record_i32[+2116] * deficit -1) / maximum) +99) /100)`. ❓ Confirm native arithmetic widths, signed division and units in instructions before porting this formula.
13. Store `GetTickCount64() - global_timebase_qword[+80]` at task qword +168.
14. Call `0x14023C280(zone,&server_position,pc_id,campfire_id,material_slot,duration,0)`. ❓ Notification codec and recipients are inside that helper.
15. Duration <=0 calls task virtual +208 immediately. Positive duration calls pending-action starter `0x140674A00(task)`.

❓ Read completion virtual +208, timer/cancel paths, reservation release, actual material consumption/durability change, helper-35 meaning, duration arithmetic, campfire validation, notification/success codecs and recipients, task cleanup and backend/DB work. Do not copy NPC repair money cost or distance rules into this action.

## Command 1395 — G_DecomposeItem2ReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037D5E0` |
| 348 name | `HandleGame_DecomposeItem2Req` |
| State | **partial** |
| Request | Literal `hQhcChh;QQCcq;` |
| Failure | 1397, `hh`: reason |
| Task | Kind 139; 0x130 bytes; vtable `0x140B9D1E8`; runner `0x14052C5D0` |

1. Apply the same user-state gate as repair. Failure sends **489** before decode.
2. Decode the three prefix fields: container selector, slot short and mode byte. Decode failure sends **480**.
3. Require container in **{2,3,6,8}**, from bit mask 332. Require mode <=6 as an unsigned byte. Either failure sends **482**.
4. Read first count. Require 0..16 inclusive, else 482. Decode short pairs into four native bytes each. Decode failure sends 480. Finish the group.
5. Decode the two `Q` fields. Failure sends 480.
6. Read second count. Require 0..8 inclusive, else **483**. Decode each byte/qword pair into separate initialized arrays. Decode failure sends 483. Finish group.
7. Require zone, else **1**. Allocate task and require PC kind 1. Wrong kind logs/asserts without a direct reply in that branch.
8. Store PC pointer +144, container +152, slot +154, mode +156, short-pair array +158 and count +222. Store the two `Q` values at +224/+225.
9. For positive second count, store count +226, bytes +228 and qwords +240. Queue failure sends 1. Release reference. Return true.
10. Runner calls `0x14050E7C0(inventory,container,slot,mode,pairs,count,Q1,Q2,auxiliary_group)`.
11. A nonzero short return sends 1397 with that signed-short reason. Zero returns without a separate direct success at this runner level.

❓ Read `0x14050E7C0`, request field meanings, item/table eligibility, stack/material count rules, output probability, costs, output capacity, inventory updates, broadcasts, backend/DB path and auxiliary native argument layout. No decompose rule is inferred from this wrapper alone.

## Gem and seal entry rules

The nine entry functions below return true.
The **U0 gate** resolves user from PC +20696 only when PC +20752 ==0.
U0 requires a non-null user and user DWORD +120 ==0.
Failure uses the per-handler reason below. U0 does not apply T1 or T2.
When listed, U0 occurs before request decode.
These functions bind zone/PC and use part-1 queue `0x140254FD0`.
They release the local task reference after posting.
Except where a section says otherwise, wrong PC kind +46 logs/asserts without an explicit entry failure send in that branch.
No item mutation, money debit, distance, cooldown or table lookup occurs in these **entry functions**.
The action helpers can perform those operations. Their unread paths remain open.

## Command 1364 — G_AttachGemToWeaponReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037A4C0` |
| 348 name | `HandleGame_AttachGemToWeaponReq` |
| State | **partial** |
| Request | `hCQhhcQ;`: counted entries: weapon container, weapon slot, gem slot, signed socket index, socket-kind selector |
| Failure | 1366, `hEc`: reason, zero-based entry index |
| Task | Kind 73; 0xB8 bytes; vtable `0x140CFC808`; runner `0x14052AA90` |

1. Apply U0; failure reason **379**.
2. Read count. Require 0..3 inclusive. Negative or >=4 sends **373**.
3. Decode five fields per entry to eight native bytes. Decode failure sends **372**.
4. For each entry, let limit =8 when socket-kind selector ==0, else 2. Require signed socket index >=0 and <limit. Failure sends **378**.
5. Finish parser bookkeeping. Require zone, else reason 1. Allocate task and require PC kind 1.
6. Copy entries to task +144, eight-byte stride. Store short count +176. Queue failure sends reason 1. The early `hEc` failure sites do not supply a recovered explicit index argument; ❓ confirm the actual variadic value in instructions before defining it.
7. Runner rechecks PC kind 1. Visit entries in order. Socket-kind zero calls `0x1404FFE10(inventory,container,weapon_slot,gem_slot,index)`; nonzero calls `0x140501240` with the same arguments.
8. A nonzero helper result sends 1366 with that reason and the current zero-based index, then stops. Earlier successful entries are not rolled back by this runner.
9. After all entries, including count zero, send **1365**, command-only descriptor at `0x1408CFBE8`, to requester.

❓ Read both attach helpers, ownership/container meaning of gem slot, socket tables, compatibility, occupied-socket replacement, item consumption, deltas/broadcasts, backend/DB work and malformed early failure index. The success descriptor is verified as `h` in the scope notes. No atomicity is proved beyond sequential execution.

## Command 1367 — G_DetachGemFromWeaponReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037AA40` |
| 348 name | `HandleGame_DetachGemFromWeaponReq` |
| State | **partial** |
| Request | `hQhcQ`: weapon container, weapon slot, signed socket index, socket-kind selector |
| Failure | 1368, `hh`: reason |
| Task | Kind 74; 0x98 bytes; vtable `0x140C3B430`; runner `0x14052ABD0` |

1. Apply U0; failure **385**.
2. Initialize socket-kind to zero. Decode without testing success.
3. Require signed socket index 0..7 when socket-kind ==0; otherwise 0..1. Failure **382**.
4. Require zone, else 1. Allocate task and require PC kind 1.
5. Store container +144, slot +146, socket index +148, socket-kind +149. Queue failure sends 1. Release reference.
6. Runner rechecks PC kind 1. Socket-kind zero calls `0x140502290`; nonzero calls `0x1405031C0`. Arguments: caller inventory, container, slot, index.
7. Nonzero short return sends 1368 with that signed-short reason. Zero returns without another direct success at this runner level.

❓ Read both detach helpers for output capacity, destruction/return behavior, costs, table fields, success packets, inventory changes/broadcasts and backend/DB work.

## Command 1369 — G_OpenWeaponGemSlotReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037B9D0` |
| 348 name | `HandleGame_OpenWeaponGemSlotReq` |
| State | **partial** |
| Request | `hQhChh;`: weapon container/slot, counted short material pairs |
| Failure | 1371, `hh`: reason |
| Success | 1370, `hc`: helper output byte; requester only |
| Task | Kind 141; 0xE0 bytes; vtable `0x140B9D4B8`; runner `0x14052AFC0` |

1. Apply U0; failure **392**.
2. Decode container/slot. Failure **386**.
3. Read count; require 0..16 inclusive, else **387**. Decode short pairs, else 386. Finish parser.
4. Require zone, else 1. Allocate task; require PC kind 1.
5. Store PC +144, container +152, slot +154, pairs +156 and short count +220. Queue failure sends 1.
6. Runner requires a resolved PC and PC kind 1. It acts only when daemon +2744 byte +933 !=0. With that byte zero, it returns without a direct reply or action.
7. Call `0x140506EB0(inventory,container,slot,pairs,count,&output_byte)`, with output initialized to zero.
8. Nonzero return sends 1371 with that reason. Zero sends 1370 with the output byte.

❓ Material pair meanings, helper validation/mutation, socket output meaning, probability/cost tables, inventory deltas, broadcasts and backend/DB work remain unread.

## Command 1372 — G_ExpandWeaponGemSlotReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037ADF0` |
| 348 name | `HandleGame_ExpandWeaponGemSlotReq` |
| State | **partial** |
| Request | `hQhh`: weapon container, weapon slot, short operand |
| Failure | 1374, `hh`: reason |
| Success | 1373, `hci`: helper byte and integer outputs |
| Task | Kind 127; 0x98 bytes; vtable `0x140B95528`; runner `0x14052ACF0` |

1. Apply U0; failure **400**.
2. Decode without testing success. Require zone, else 1. Allocate task; require PC kind 1.
3. Store container +144, slot +146, operand short +148. Queue failure sends 1.
4. Runner does nothing when daemon +2744 byte +933 is **exactly 1**. Other values, including 0 and 2, continue.
5. Require PC kind 1. Initialize byte/integer outputs to zero. Call `0x1405040E0(inventory,container,slot,operand,&byte,&integer)`.
6. Nonzero return sends 1374 with that reason. Zero sends 1373 with both outputs to requester.

❓ Short operand meaning, helper checks, slot cap, item/money cost, table fields, mutation, deltas/broadcasts and daemon/DB path remain unread. Preserve the exact equality-to-1 feature test.

## Command 1375 — G_AddWeaponGemSlotReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037B120` |
| 348 name | `HandleGame_AddWeaponGemSlotReq` |
| State | **partial** |
| Request | `hQhh`: weapon container, weapon slot, short operand |
| Failure | 1377, `hh`: reason |
| Success | 1376, `hci`: helper byte and integer outputs |
| Task | Kind 140; 0xA0 bytes; vtable `0x140B95FD8`; runner `0x14052AE30` |

1. Apply U0; failure **406**.
2. Decode without testing success. Require zone, else 1. Allocate task; require PC kind 1.
3. Store PC pointer +144, container +152, slot +154 and operand short +156. Queue failure sends 1.
4. Runner returns without action/reply when daemon +2744 byte +933 ==1. Otherwise initialize byte/integer outputs to zero.
5. Call `0x140504F20(inventory,container,slot,operand,&byte,&integer)`.
6. Nonzero return sends 1377 with that reason. Zero sends 1376 with the outputs to requester.

❓ Operand meaning, helper validation, slot/count limits, item/money costs, tables, mutation, deltas/broadcasts and backend/DB path remain unread.

## Command 1378 — G_DeleteWeaponGemSlotReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037B440` |
| 348 name | `HandleGame_DeleteWeaponGemSlotReq` |
| State | **partial** |
| Request | `hQhccChh;`: container, weapon slot, byte A, unsigned byte B, counted short pairs |
| Failure | 1380, `hh`: reason |
| Success | 1379, command-only descriptor at `0x1408CFBE8` |
| Task | Kind **141**, as for open, but vtable `0x140B9CCC0`; runner `0x14052AF00` |

1. Apply U0; failure **414**.
2. Decode four prefix fields; failure **408**. Require byte B <=3, else **411**. Byte A has no bound in this entry.
3. Read count; require 0..16 inclusive, else 411. Decode short pairs; failure 408. Finish parser.
4. Require zone, else 1. Allocate 0xE0-byte task and require PC kind 1.
5. Store PC +144, container +152, slot +154, byte A +156, byte B +157, pairs +158, count short +222. Queue failure sends 1.
6. Runner does nothing when daemon +2744 byte +933 ==1. Otherwise call `0x140505DA0(inventory,container,slot,A,B,pairs,count)`.
7. Nonzero return sends 1380 with that reason. Zero sends 1379 to requester.

❓ Prefix byte meanings, concrete pointer-width correction at the action call, helper checks, occupied-slot behavior, material cost, table fields, mutation, inventory replies/broadcasts and backend/DB work remain unread. Shared task kind 141 does not imply shared action behavior.

## Command 1381 — G_IdentifyItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037BEF0` |
| 348 name | `HandleGame_IdentifyItemReq` |
| State | **partial** |
| Request | `hhh`: two short operands; ❓ exact item/material slot meanings |
| Failure | 1382, `hh`: reason |
| Task | Inventory factory kind 11; 0xA0 bytes; vtable `0x140D1CE20`; action virtual +96 = `0x14052C410` |

1. Decode without testing success **before** PC-state checks.
2. PC byte +13768 ==1 sends **7**.
3. Require zone and task allocation, else reason 1. Require PC kind 1.
4. Bind and copy shorts to task +152/+154. Queue failure sends 1. Release reference. Return true.
5. Action calls `0x140509EB0(inventory,u16(first),u16(second))`. Nonzero return sends 1382 with that reason.
6. Action then removes/completes the inventory task through PC +2232 virtual +48 and reference handling, even after a helper failure.

No U0 gate appears here. Do not add one from the nearby gem handlers.
❓ Identify helper, target/material lookup, random result, money/items, table fields, item mutations, success deltas/broadcasts, cleanup protocol and daemon/DB work remain unread.

## Command 1383 — G_UnsealItem2ReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037C520` |
| 348 name | `HandleGame_UnsealItem2Req` |
| State | **partial** |
| Request | `hhcChh;`: target short, unsigned mode byte, counted short pairs |
| Failure | 1384, `hh`: reason |
| Task | Inventory factory kind 12; 0xE8 bytes; vtable `0x140C9DAA0`; action `0x14052C520` |

1. If daemon +2744 byte +935 ==0, log and return true **without decode or reply**.
2. Apply U0; failure **446**.
3. Decode target/mode. Failure **441**. Require mode <=3, else **443**.
4. Read count; require 0..16 inclusive, else 443. Decode short pairs; failure 441. Finish parser.
5. Require zone and allocated task, else 1. Require PC kind 1; wrong kind logs/asserts without an explicit reply in that branch.
6. Bind task. Store PC +152, target short +160, mode +162, pairs +166 and count short +230. Queue failure sends 1.
7. Action calls `0x14050ABE0(inventory,target,mode,pairs,count)`. A nonzero return sends 1384 with that reason. Then call task completion `0x140422850`.

❓ Concrete pointer-width correction at the helper call, target/material semantics, seal state checks, item/money cost, tables, result mutation, success packets/broadcasts, cleanup and daemon/DB work remain unread.

## Command 1392 — G_SealItemReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14037D130` |
| 348 name | `HandleGame_SealItemReq` |
| State | **partial** |
| Request | `hhChh;`: target short and counted short pairs |
| Failure | 1394, `hh`: reason |
| Task | Inventory factory kind 10; 0xE0 bytes; vtable `0x140CDA240`; action `0x14052C2F0` |

1. Apply U0; failure **478**.
2. Decode target short. Failure **473**.
3. Read count; require **1..16**, inclusive. Zero, negative or >16 sends **475**.
4. Decode each short pair; failure 473. Finish parser.
5. Require zone and allocated task, else 1. Require PC kind 1.
6. Bind task. Store target short +152, pairs +154 and count short +218. Queue failure sends 1.
7. Action calls `0x14050D540(inventory,target,pairs,count)`. Nonzero return sends 1394 with that reason.
8. Remove/complete the inventory task through PC +2232 virtual +48 and reference handling, even after helper failure.

❓ Target/material lookup, seal eligibility, seal count, item/money cost, table fields, mutation, success deltas/broadcasts, generic cleanup and daemon/DB path remain unread.

## Rust port status (part 1)

Source: `Program/Server/GameDaemon/src/handlers/items.rs`, tests `tests/gi2_items.rs`.

| Command | Rust | Boundary that stops the port |
| --- | --- | --- |
| 1325 | ported | — |
| 1354, 1356, 1359, 1361 | partial | Entry ported. Equip/unequip execution (`0x140529B10`, `0x14052A250`) and virtual +208 completions are not ported. |
| 1512 | partial | Builder `0x14060EC20` record fields unread |
| 1515 | partial | Entry ported. Runner `0x1402C4F20` not ported. |
| 1518, 1524, 1527 | partial | Entry ported. Runners not ported. |
| 1521 | ported | Backend 367 response handler is outside this entry |
| 1414, 1416, 1418, 1426 | partial | Entry ported. Runners need item virtual getters/setters. |

- Short-body decode for 1518/1521/1524/1527 uses the zero-initialized value, as the entry functions do.
- 1354/1359 do not initialize the decoded fields, so a short body stops at a boundary.

## Rust port status (part 2)

Source: `Program/Server/GameDaemon/src/handlers/items2.rs`, tests `tests/gi3_inventory.rs`.
All 23 entries queue an `ItemTaskKind::Inventory` task. No runner is ported.

| Command | Rust | Boundary that stops the port |
| --- | --- | --- |
| 934, 937, 1118 | partial | Entry ported (dispatch `0x140603BD0`). Closet runners unread. |
| 1121, 1125 | partial | Entry ported. Serializers `0x14007F250` and `0x1400B8E90` unread. |
| 1428 | partial | Success path queues. Every entry failure is a boundary: the 1429 trailing integer is not recovered. |
| 1331 | partial | Entry ported. Batch item serialization unread. |
| 1334, 1336, 1338 | partial | Entry ported. Inventory helpers `0x1404D84E0`, `0x1404D8660`, `0x1404F55C0` and validators unread. |
| 1328 | partial | Entry ported, with the 1344 failures. Action `0x14053B0A0` needs NPC and `0x1404E41C0` adapters. |
| 1385, 1387 | partial | Entry ported. Runners need item record and durability adapters. |
| 1395 | partial | Entry ported. Helper `0x14050E7C0` unread. |
| 1364 | partial | Success path queues. Every entry failure is a boundary: the `hEc` index is not recovered. |
| 1367, 1369, 1372, 1375, 1378 | partial | Entry ported. Gem helpers unread. The daemon +2744 byte +933 test is in the runners. |
| 1381, 1383, 1392 | partial | Entry ported. Identify, seal and unseal helpers unread. |

- 1334, 1336, 1372, 1375, 1381 and 1387 do not zero-initialize their fields, so a short body stops at a boundary.
- 1367 zero-initializes only the socket kind. A body without it decodes as kind 0.

## Coverage limit

This file covers 38 of the 89 selected commands.
Only the return-true manufacture target is done.
The other 37 sections have explicit gaps. Do not treat them as complete Rust port contracts.
The other 51 selected commands were not started in this job. They have no state row below.
No DB procedure was verified. Backend commands 17 and 367 are not procedure names.
No runtime parity test ran. This is a static binary trace only.

## Machine-readable handler states

| id | state |
| --- | --- |
| 934 | partial |
| 937 | partial |
| 1118 | partial |
| 1121 | partial |
| 1125 | partial |
| 1325 | done |
| 1328 | partial |
| 1331 | partial |
| 1334 | partial |
| 1336 | partial |
| 1338 | partial |
| 1354 | partial |
| 1356 | partial |
| 1359 | partial |
| 1361 | partial |
| 1364 | partial |
| 1367 | partial |
| 1369 | partial |
| 1372 | partial |
| 1375 | partial |
| 1378 | partial |
| 1381 | partial |
| 1383 | partial |
| 1385 | partial |
| 1387 | partial |
| 1392 | partial |
| 1395 | partial |
| 1414 | partial |
| 1416 | partial |
| 1418 | partial |
| 1426 | partial |
| 1428 | partial |
| 1512 | partial |
| 1515 | partial |
| 1518 | partial |
| 1521 | partial |
| 1524 | partial |
| 1527 | partial |
