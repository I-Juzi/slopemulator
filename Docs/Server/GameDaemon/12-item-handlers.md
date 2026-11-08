| Shared helper, build 381.8867 | Purpose | Verified limit |
| --- | --- | --- |
| `0x1405114E0` | Inventory container usable-slot limit | For container 2/3/6/8, use PC signed byte +528/+529/+530/+531. Return the smaller of this value and allocated slot count minus one. Other containers use allocated count minus one. |
| Inline in `0x140515360` | Inventory slot lookup | Require `0 < slot < inventory.s16[80]`. Read `inventory.qword[72] + 8*slot`. Inventory starts at PC +15000. This is a container-2 source lookup, not a universal item lookup. |
| `0x1403E3CA0` | Trade ownership/link check | Require caller, caller.trade == trade, inviter, inviter.trade non-null, and guest. It does not itself require inviter.trade == trade or guest.trade == trade. |
| `0x1405593A0` | Item trade eligibility | Requires item native +16, container 2, virtual +160 ==0, byte +24 !=1, and further config/table/virtual gates. This is not a general ownership check. |
| `0x1403E0A60` | Set offered money | Changes an offer, not the PC balance. Checks sender balance and receiver headroom. Restores old offer on either failure. |
| `0x140267EA0`; `0x140260850` | Change money properties and flush delta | Changes property 109 and, for flag zero, property 110. A second-property failure can follow a first-property mutation. ❓ Setter cap/clamp and persistence remain open. |
| `0x140515360`; item virtual +32 | Move a full stack, or clone a partial stack, into trade escrow | Escrow container 5 has slots 1..8. Partial-stack clone gets the PC owner. ❓ Clone implementations remain open. |
| `0x140313790`; item virtual +0 | Backend clone creation and one deletion path | Creation sends backend command 19, `hqqqcb`. Escrow-return overflow calls deleting virtual +0 with flag 1. ❓ General table-based creation and concrete destructors remain open. |
| `0x140515A80`; `0x140515E60` | Return escrow items and handle overflow | See command 320. No stack-merge rule was proved. |
| `0x140601580` | Operation restriction from table-manager slot 555 | Invite checks operation 4 for each PC. Missing record passes. See command 311. ❓ Table and field names remain open. |
| `0x140254FD0` | Zone task queue wrapper | Requires zone byte +16148, serial queue +16216, and queue DWORD +184. Selects same-thread or cross-thread queue. Calls `0x14025DBF0` on failure. |
| `0x1404C8C10`, `0x1404C8F80`, `0x140314680` | Inventory delta / backend batch construction and flush | Seen in escrow insert/remove. Backend update flush requires command 17. ❓ Named endpoint, client delta IDs and DB procedure remain open. |
| PC virtual +592 → `0x1405F5530` → `0x1406A5E00` | Direct formatted success/failure reply | Destination is that PC's session. Source: `09-parity.md`, checked send endpoint map. |
| `0x1403E4D20` | Trade suggestion task dispatch/failure helper | Sends command 323, `hh`, for task-level failures. |
| `0x14069F410` → `0x1406AA7B0` | Request decoder | Decoder starts at logical packet +2. Callers below do not test its result. ❓ Decoder error semantics need a separate trace. |
| `0x14000B730` | Reference release | Do not interpret this as a socket close or item destruction. |

# 12 — Item and inventory handlers, part 1

## Scope and notation

Static specification only. Source binary: `ida/datafile/gd2020/GameDaemon.exe`, build 381.8867.
The build-348 names below come only from `parity-ledger.tsv`.
A native offset is not a wire offset.
`pc.trade` means native PC +19648. `pc.zone` means +2224.
Object IDs use PC +40. Money uses signed 64-bit PC +312.
A format includes the leading `h` command ID unless stated otherwise.
`h`, `c`, and `q` denote 16-, 8-, and 64-bit values.
Numeric reason codes have no asserted client enum names.
❓ Wire endian and signed conversion details need decoder proof where not stated.

`done` means that all game-action paths in the target were read.
It does not mean that a Rust port or live test passed.
Each section gives acceptance cases for a later port.
No test ran in this job.
A `partial` section must not be used as a complete Rust behavior contract.

## Command 121 — G_AcquireProfessionReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14001AFE0` |
| 348 name | `-` |
| State | **done** |

The entire routed target returns true.
It does not read a packet or PC field.
It has no request decoder or format string.
The action therefore ignores any body delivered by the dispatcher.
It performs no state, ownership, slot, count, money, distance, cooldown or table check.
It makes no state change, reply, broadcast, daemon call, DB call or kick call.
This is not a working profession-acquisition implementation.
Source: full decompilation of `0x14001AFE0`; dispatch map in the ledger.
Acceptance: route command 121 to this target; verify true return and no side effect.
Do not add a decode failure or success packet.

## Command 123 — G_DisposeProfessionReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x14001AFE0` |
| 348 name | `-` |
| State | **done** |

The entire routed target returns true.
It does not read a packet or PC field.
It has no request decoder or format string.
The action therefore ignores any body delivered by the dispatcher.
It performs no state, ownership, slot, count, money, distance, cooldown or table check.
It makes no state change, reply, broadcast, daemon call, DB call or kick call.
This is not a working profession-disposal implementation.
Source: full decompilation of `0x14001AFE0`; dispatch map in the ledger.
Acceptance: route command 123 to this target; verify true return and no side effect.
Do not add a decode failure or success packet.

## Common trade entry gates

Source: receive functions `0x1403947A0` through `0x140396060`.
Apply a gate only where the handler section lists it.

| Gate | Exact predicate | Result |
| --- | --- | --- |
| T1 | Network config byte +237 !=0 AND DWORD +248 !=0; otherwise read PC DWORD +37520 under critical section +37480 and reject values in {2,3,5,6} | Send the handler's failure command, `hh`, reason 5. Return true. |
| T2 | PC byte +13768 ==1 | Send the handler's failure command, `hh`, reason 7. Return true. |
| Zone | PC +2224 is null | Send reason 2. Log/assert through `0x14069CB70`. Return true if the assertion returns. This is not a proved user kick. |
| Queue | `0x140254FD0(zone,task)` returns false | Send reason 2. Release the task. Return true. |

A task stores caller ID at +128 and marks +138 =1.
Its initializer calls `0x14025DB30(task,zone)` and `0x14069BF60(pc,task,...)`.
❓ The common task owner lookup and cancellation lifecycle need a full trace.
The request handlers release their local task reference after posting.
They do not wait for completion.
Do not replace task serialization with an immediate unsynchronized action.

A timed-user gate below uses PC +20696 only when PC byte +20752 ==0.
It rejects when config byte +721 !=0 and user.qword[984] <= `time64()` <= user.qword[992].
Both interval endpoints are inclusive.
❓ The named server policy for this interval is not recovered.

## Command 311 — G_InviteTradeReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403947A0` |
| 348 name | `HandleGame_InviteTradeReq` |
| State | **partial** |
| Request | `hq`: command, guest object ID |
| Failure reply | 312, `hh`: command, reason; requester only |

### Ordered behavior

1. Apply T1, then T2, then decode `hq` through `0x14069F410`. The handler does not test decode success.
2. Require the caller zone. Create a 0x98-byte invite task with vtable `0x140BBBF98`, task kind 46. Store guest ID at task +144. Post the task.
3. Task virtual +72 is `0x1403E4670`. It creates a 0x918-byte trade object with vtable `0x140B98DA0`. It calls `0x1403DE170(trade,caller,guest_id)`.
4. `0x1403DE170` requires caller and caller zone. Null paths log/assert and return false without a direct reply at this level.
5. Require guest ID bits 48..63 ==1. Failure: 907.
6. Resolve the guest in caller.zone +16232 through `0x140259CA0`. Require a non-null guest different from caller. Failure: 907.
7. Require the guest zone to equal caller zone and be non-null. Failure: 904.
8. Require guest byte +13768 !=1. Failure: 8.
9. Apply the timed-user gate to caller, then guest. Fail with 2, then 911, respectively.
10. Require caller.trade ==null. Failure: 905.
11. Require caller byte +888 mask 1 clear and caller byte +578 ==0. Failure: 906.
12. Require guest.trade ==null. Failure: 909.
13. Require guest byte +888 mask 1 clear and guest byte +578 ==0. Failure: 910.
14. Use signed-short server x/y/z at +268/+270/+272. Require squared 3D distance <3969. Equality fails with 908. See `07-server-validation.md`, part 3.
15. Call `0x140601580(caller,4)`, then, only on success, `0x140601580(guest,4)`. Either false result sends reason 2.
16. Attach the same trade object to both PCs at +19648 with reference increments. Store inviter PC/ID/identity at trade +32/+56/+64. Store guest PC/ID/identity at +144/+168/+176. Set both participant states, +40 and +152, to 0.
17. Send command 313, `hqq`, to caller and guest. Body: inviter object ID, guest object ID.
18. When config byte +537 !=0, call audit helper `0x140369F70`. No extra client broadcast appears in this branch.

No inventory or money change appears in the recovered invite path.
The operation restriction helper reads a server table. No cooldown, DB procedure or game-daemon transaction appears in this path.
The audit call is not proved to be a DB transaction.
No direct kick call appears. Do not apply the BoardGacha kick rule from doc 07 part 4.

### Operation restriction check

Source: `0x140601580`, build 381.8867.

1. Look up table-manager qword slot 555 through virtual +168. Use daemon byte +86857 as key when it is 1..15; otherwise use 0. A missing record passes.
2. For operation 4, record byte +36 ==0 passes. Otherwise read required signed short +16 and required integer +20. Both zero passes.
3. A positive level requirement passes when signed PC byte +276 >= that requirement.
4. Otherwise require a positive record +20 value and a matching enabled PC entry. Count is PC.s16[12688]. Entries have stride 32. The active byte is PC +12704+32*i. The matching DWORD is PC +12696+32*i.

No mutation appears in the read helper.

### Gaps and acceptance

- ❓ Restriction table name, field names and virtual lookup implementation remain open.
- ❓ Resolve generic task binding/expiry and decoder failure behavior.
- ❓ Read audit-helper effects if audit parity is required.
- Acceptance: test each ordered failure; test distance 3968 and 3969; verify both recipients get 313 only after both state helpers succeed.

## Command 314 — G_JoinTradeReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140394A90` |
| 348 name | `HandleGame_JoinTradeReq` |
| State | **partial** |
| Request | `hq`: command, inviter object ID |
| Failure reply | 315, `hh`: command, reason; requester only |

### Ordered behavior

1. Apply T1, then T2, then decode `hq`. The handler does not test decode success.
2. Require caller zone. Create a 0xA0-byte join task with vtable `0x140B9A5E0`, kind 47. Store inviter ID at task +152. Post the task.
3. Task virtual +72 is `0x1403E47A0`. Require the task caller zone, else reason 2.
4. Require inviter ID bits 48..63 ==1, else 915. Resolve inviter in caller zone through `0x140259CA0`. Missing inviter: 915. Missing inviter.trade: 916.
5. Call `0x1403DEEA0(inviter.trade,caller)`.
6. Require caller, stored inviter, stored inviter.trade == trade, and stored guest. Missing/link failures log/assert and return false without a direct failure send in this helper.
7. Require stored guest ==caller, else 916.
8. Require the stored inviter and guest to have the same non-null zone, else 912.
9. Apply timed-user gate to inviter, then guest. Either rejection sends reason 2 to caller.
10. Require inviter participant state at trade +40 ==0, else 916. The helper does not separately test guest participant state here.
11. Require guest byte +888 mask 1 clear and guest byte +578 ==0, else 914. No matching inviter dead/state test appears here.
12. Require server squared 3D distance <3969, else 917. Use stored inviter and guest signed-short positions. Equality fails.
13. Set participant states +40 and +152 to 1.
14. Send command 316, `hqq`, to both PCs. Body: inviter ID, guest ID.
15. If config byte +537 !=0, call audit helper `0x14036A3F0`.

This path does not change items or money.
It does not call an item table, cooldown helper or DB procedure directly.
No direct kick call appears. See doc 07 part 4 for why transaction failure is not a generic kick rule.

### Gaps and acceptance

- ❓ Resolve common task binding/expiry, decoder failure, and audit-helper effects.
- Acceptance: test wrong inviter kind, missing trade, wrong guest, cross-zone, each timed gate, inviter state !=0, guest disabled/dead, and strict distance boundary. Verify both states change before both 316 sends.

## Command 317 — G_DeclineTradeReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140394D80` |
| 348 name | `HandleGame_DeclineTradeReq` |
| State | **partial** |
| Request | `hqc`: command, inviter ID, decline byte |
| Failure reply | 318, `hh`: command, reason; requester only |

### Ordered behavior

1. Apply T1 and T2. Decode `hqc` without testing the decoder result.
2. Require caller zone. Create a 0xA8-byte task, kind 48, vtable `0x140BC1400`. Store inviter ID at +152 and decline byte at +160. Post it.
3. Task virtual +72, `0x1403E4980`, requires caller zone, else 2. Require inviter ID bits 48..63 ==1, else 918. Resolve inviter in caller zone, else 918. Require inviter.trade, else 919.
4. Call `0x1403DFA20(trade,caller,decline_byte)` with a held trade reference.
5. Require caller, stored inviter, stored inviter.trade == trade, and stored guest. These invariant failures log/assert and return false without a direct reply in this helper.
6. Require caller == stored guest, then inviter participant state ==0. Either failure sends 318 reason 919.
7. Send command 319, `hqqc`, to inviter and guest. Body: inviter ID, guest ID, supplied decline byte. No decline-byte range check appears.
8. If config byte +537 !=0, call audit helper `0x14036A3F0` with operation 2.
9. On helper success, task calls unlink helper `0x1403E3B40`. It clears both PCs' trade links, releases those trade references, copies participant states to trade +136/+248, sets states +40/+152 to 0, and clears PC pointers +32/+144.

No distance, dead-state, inventory-slot, money, cooldown or item-table check appears after task resolution.
No item transfer, DB procedure or direct kick call appears in this recovered path.
See doc 07 part 4. An assertion call is not evidence of a user kick.

### Gaps and acceptance

- ❓ Common task lifecycle, decoder failures, audit effects, and final trade destructor effects remain open.
- Acceptance: test wrong ID kind, missing inviter, missing trade, wrong guest and inviter state !=0. Verify both 319 sends precede unlink. Verify the request decline byte passes through unchanged.

## Command 320 — G_CancelTradeReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140395080` |
| 348 name | `HandleGame_CancelTradeReq` |
| State | **partial** |
| Request | No body read; no decoder or format string. Effective consumed prefix is command `h`. |
| Failure reply | 321, `hh`: command, reason; requester only |

### Ordered behavior

1. Apply T1 and T2. Require caller zone.
2. Create a 0x90-byte task, kind 49, vtable `0x140B9B250`. Post it.
3. Task virtual +72, `0x1403E4BB0`, holds caller.trade. Missing trade sends 321 reason 920.
4. Call `0x1403E0230(trade,caller,1)`.
5. Require caller, caller.trade non-null, stored inviter, stored inviter.trade == trade, and stored guest. Invariant failures log/assert and return false without a direct reply in this helper.
6. Require caller to be inviter or guest, else 321 reason 920. There is no participant-state or distance gate here.
7. Call `0x140515A80` on inviter inventory, then guest inventory. Apply the escrow-return rules below, including fallback and deletion on failed fallback.
8. Send command 322, literal `hqQ`, to both PCs. Body: cancelling PC object ID and constant 1. Source: literal bytes at `0x14099CEC0`. ❓ Uppercase `Q` wire encoding remains open.
9. If config byte +537 !=0, call audit helper `0x14036BAA0`.
10. On helper success, call unlink helper `0x1403E3B40` as specified for command 317.

The handler reads no client money, slot, count or position.
No cooldown or item-table lookup appears in the top-level path.
Escrow return has inventory and backend effects. Do not treat cancel as memory-only.
No direct kick call appears in the recovered path. See doc 07 part 4.

### Shared escrow-return rules

Sources: `0x140515A80`, `0x140515E60`, build 381.8867.

1. `0x140515A80` creates client/backend delta batches. Visit nine escrow entries, native inventory qword indices 165..173. This includes reserved slot 0, not only slots 1..8.
2. Skip null items. An item with native +16 ==0 logs and keeps its escrow pointer.
3. For a valid item, set owner +8 to the inventory owner. Add removal delta. Call `0x140515E60`, then clear that escrow pointer.
4. `0x140515E60` scans bag slots 1 through `0x1405114E0(inventory,2)`, inclusive. Require an empty pointer and an unset reservation byte in the array at inventory +88. Insert into container 2 through `0x140511560`. Add client/backend updates.
5. If no normal slot is available, call `0x1405110B0(inventory,item,0)`. ❓ Its fallback rules remain unread. Do not call this a proved stack merge.
6. If fallback fails, set item container 4 and slot 0. Queue a backend update. Call deleting virtual +0 with flag 1. ❓ Concrete destructor and persistence effects remain open.
7. Flush nonempty client/backend batches. Backend update flush `0x140314680` requires command 17. ❓ Client delta format and complete endpoint/DB effects remain open.

For command 327, `0x140515770` clears the selected escrow pointer before the return call. For bulk cancel, the loop clears it after that call.

### Gaps and acceptance

- ❓ Read fallback `0x1405110B0`, concrete deletion effects, complete inventory replies and DB procedures.
- ❓ Resolve uppercase `Q` encoding, common task lifecycle, audit and destructor effects.
- Acceptance: test cancel from each participant and from invalid caller. Verify both escrow-return calls precede both 322 sends and unlink.

## Shared suggestion execution

Sources: `0x1403E4D20`, `0x1403E2240`, `0x1403E3CA0`.
Commands 324, 327 and 330 use a 0xD0-byte task, kind 50, vtable `0x140C341E0`.
The task runner is `0x1403E4D20`.
Missing caller.trade sends command 323, `hh`, reason 924.
For each valid operation below, it first calls automatic retract `0x1403E2240(trade,caller)`.
It ignores the retract result, then calls the selected operation.
Do not roll back a completed retract when the selected operation fails.

Automatic retract requires caller.trade == trade, inviter, inviter.trade == trade, guest, caller membership, and equal non-null zones.
Failure logs and returns false without a reply.
It acts only when caller state ==1 and peer state ==2.
It sets both states to 1 and sends command 338, literal format `hqQ`, to both PCs.
Body: caller ID, constant 1 in the `Q` field.
❓ The width/encoding of uppercase `Q` needs decoder/composer proof. It is not lowercase `q`.

The three operation helpers start with `0x1403E3CA0`.
Its invariant failures assert/log and return false without their own packet.
They then require caller membership (reason 924), equal participant zones (923), and caller state ==1 (925), in that order.
All these operation failures send 323, `hh`, to caller only.
The zone test in these three helpers compares pointers; it does not explicitly reject two null zones.
No fresh distance or cooldown check appears here.

## Command 324 — G_AppendTradeSuggestionReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140395340` |
| 348 name | `HandleGame_AppendTradeSuggestionReq` |
| State | **partial** |
| Request | `hhh`: command, source inventory slot, signed stack count |
| Failure reply | 323, `hh`: command, reason |

### Ordered behavior and state changes

1. Apply T1 and T2. Decode `hhh` without testing the result. Require zone.
2. Initialize suggestion task operation +200 =3; source slot at +192, count at +194. Task creation or posting failure sends reason 2.
3. Run shared suggestion execution, then `0x1403E0DC0`.
4. Require receiver free-slot estimate >= caller escrow stack count +1. Failure: 927. This check occurs before source-slot validation.
5. The estimate is `s16(usable_limit - receiver.s8[15096] - receiver.s16[15082])`. `usable_limit` is `0x1405114E0(receiver.inventory,2)`. The subtraction uses 16-bit native arithmetic. Caller escrow count comes from `0x140515DB0`, which counts non-null escrow slots 1..8. Source: assembly around the `0x1405114E0` call in `0x1403E0DC0`.
6. Call `0x140515360(caller.inventory,slot,count,&moved_full_stack)`. Failure maps to 926.
7. That inventory helper first requires signed count >0. It requires `0 < unsigned_slot < inventory.s16[80]`. It then selects the first empty escrow slot in 1..8. Full escrow fails before source-item lookup.
8. Require a non-null source item in the container-2 pointer array at inventory +72. Call item eligibility helper `0x1405593A0(item,0)`. Require item virtual +216 stack count >= requested count.
9. Create inventory client-delta and backend batches through `0x1404C8C10(...,3)` and `0x1404C8F80(...,0)`.
10. For an exact full stack, clear the source slot and decrement inventory short +82. Add removal delta through `0x1404C8E00`. Set item container to 5 through virtual +168. Set escrow slot through virtual +176. Store the item at `inventory +1320 +8*escrow_slot`. Add item delta through `0x1404C8CF0` and backend update through `0x1403141B0`.
11. For a partial stack, decrease source count through virtual +224. Add source delta/backend update. Clone the item through virtual +32. Set clone owner at +8 to the PC. Set clone count, container 5 and escrow slot. Store clone in escrow. Call `0x140313790` for its backend creation. The helper returns a null full-stack output on this branch.
12. Flush nonempty client delta to caller through virtual +584. Flush nonempty backend update through `0x140314680`.
13. Only a non-null full-stack output calls `0x1403E10F0`. It sends caller command 326 with command-only format `h`. It sends peer command 325 using descriptor `hiqSh` and item-specific serialization: caller ID, item type-selected `S` payload, then item count. ❓ Exact nested `S` branch width and descriptor expansion remain open.
14. A partial-stack clone returns success here without that direct 325/326 notification. It has already queued backend creation. ❓ Trace its completion path before adding any substitute reply.

### Item eligibility, tables and failures

`0x1405593A0` requires native item +16 nonzero; virtual +144 ==2; virtual +160 ==0; item byte +24 !=1.
It fetches the item record through virtual +112.
If config +1379 ==1, a non-null record with signed byte +1937 >1 fails.
For argument 0, require record byte +45 ==0 and item virtual +288 !=1.
If daemon config at +2744 byte +909 !=0, require virtual +304 ==0.
For item kind 1, all eight `0x14055EF60(item,index)` results must be zero or in 65001..65011.
Finally require virtual +616 !=1 and virtual +424 !=1.
❓ Record table identity, XML field names, item virtual implementations and `0x14055EF60` remain open. Do not assign invented bind/seal enum names.

No money debit or direct kick appears in the recovered append path.
❓ Inventory batch packet IDs, daemon commands, DB procedures, clone/destruction and asynchronous completion remain open.
Acceptance: test receiver capacity before invalid source; escrow full; count <=0; insufficient count; exact full-stack and partial-stack branches. Verify automatic retract remains applied after a later failure.

## Command 327 — G_RemoveTradeSuggestionReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140395600` |
| 348 name | `HandleGame_RemoveTradeSuggestionReq` |
| State | **partial** |
| Request | `hc`: command, escrow slot byte |
| Failure reply | 323, `hh`: command, reason |

1. Apply T1 and T2. Decode `hc` without testing the result. Require zone.
2. Initialize suggestion operation +200 =4; store slot byte at +196. Creation/post failure sends 2.
3. Run shared suggestion execution, then `0x1403E1500`.
4. Apply the shared invariant, membership, zone and state checks.
5. Call `0x140515770(caller.inventory,slot)`. The native char-to-unsigned-short conversion is preserved. Accept only slots 1..8. Require a non-null escrow item and item native +16 !=0. Any false result maps to reason 931.
6. Clear the escrow pointer. Build inventory delta/backend batches. Add removal delta, then call `0x140515E60` with the shared escrow-return rules in command 320. Flush nonempty client and backend batches.
7. Send caller command 329, command-only format `h`. Send peer command 328, `hqc`: caller ID, original slot byte.

No money, distance or cooldown check appears here.
No direct kick call appears. See doc 07 part 4.
❓ Fallback helper `0x1405110B0`, concrete deletion, full batch packets, backend endpoint/DB procedure and common lifecycle remain open.
Acceptance: test slots 0,1,8,9,128,255; empty slot; missing native +16; and successful return. Verify failed removal does not undo automatic retract.

## Command 330 — G_SuggestTradeMoneyReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403958B0` |
| 348 name | `HandleGame_SuggestTradeMoneyReq` |
| State | **partial** |
| Request | `hq`: command, signed 64-bit offered money |
| Failure reply | 323, `hh`: command, reason |

1. Apply T1 and T2. Decode `hq` without testing the result. Require zone.
2. Initialize suggestion operation +200 =2; store money at +184. Creation/post failure sends 2.
3. Task rejects money <0 or >`qword_140EE58A0.qword[24]` with reason 930 **before** automatic retract.
4. Run automatic retract, then `0x1403E0A60` and its shared invariant, membership, zone and caller-state checks.
5. Repeat the signed money bound. Failure: 930.
6. Save the old caller offer, then store the new offer. The offer is at participant block +16: trade +48 for inviter, +160 for guest.
7. Require caller money >=offer, else 926. Require offer <=`qword_140EE5418.qword[20224] - receiver.money`, else 928. Either failure restores the old offer.
8. On success, send caller command 332, `hq`: new offered money. Send peer command 331, `hqq`: caller ID, new offered money.

This changes no actual balance. It reserves no money at this point.
No item-table, inventory-slot, distance, cooldown, DB procedure or direct kick call appears in this setter.
❓ Names and loading sources for both money maxima remain open. Common lifecycle and decode behavior remain open.
Acceptance: test -1, 0, both maxima, insufficient balance, receiver overflow and restoration. Verify range failure before retract differs from balance failure after retract.

## Command 333 — G_SubmitTradeReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140395B60` |
| 348 name | `HandleGame_SubmitTradeReq` |
| State | **partial** |
| Request | No body read; no decoder or format string. Effective consumed prefix `h`. |
| Failure reply | Usually 334, `hh`; task missing-trade branch uses **321**, `hh`, reason 933 |

1. Apply T1 only, with failure command 334. Do not add T2 to this handler.
2. Require zone. Post a 0x90-byte task, kind 51, vtable `0x140C3F588`. Queue failure uses 334 reason 2.
3. Task `0x1403E4EB0` sends **321 reason 933** when caller.trade is absent. Otherwise call `0x1403E17A0`.
4. Require caller, caller.trade == trade, inviter, inviter.trade == trade, and guest. Invariant failures assert/log and return false without a direct reply from this helper.
5. Require caller membership, else 334 reason 933. Require equal non-null participant zones, else reason 932.
6. Apply timed-user gate to inviter, then guest. Either failure sends 334 reason 2.
7. Require caller participant state ==1, else reason 934.
8. Require caller money >=caller offer, else reason 935.
9. Set caller participant state to 2. Send command 335, `hq`, to both PCs. Body: submitting caller object ID.

This does not debit money or move items.
There is no new distance, cooldown, item-slot or table check in this helper.
No direct daemon/DB or kick call appears in the recovered path.
❓ Common task owner/expiry and final lifecycle remain open.
Acceptance: test missing trade's different reply ID; state 0/1/2/3; exact balance equality; either timed-user gate; and both success recipients.

## Command 336 — G_RetractTradeReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140395DE0` |
| 348 name | `HandleGame_RetractTradeReq` |
| State | **partial** |
| Request | No body read; effective consumed prefix `h`. No request decoder. |
| Failure reply | Usually 337, `hh`; task missing-trade branch uses **321**, `hh`, reason 937 |

1. Apply T1 only, with failure command 337. Require zone.
2. Post a 0x90-byte task, kind 52, vtable `0x140BA0D10`. Queue failure sends 337 reason 2.
3. Task `0x1403E4F30` sends **321 reason 937** when caller.trade is absent. Otherwise call `0x1403E1DD0`.
4. Require caller, caller.trade == trade, inviter, inviter.trade == trade, and guest. Invariant failures assert/log and return false without a direct reply from this helper.
5. Require caller membership, else 337 reason 937. Require equal non-null participant zones, else 936.
6. Require caller state ==2, then peer state !=3. Either failure sends 337 reason 938.
7. Set both participant states to 1. Send command 338, literal format `hqQ`, to both PCs. Body: caller object ID, constant 0 in the `Q` field.

No items or balances change. No timed-user gate, distance, cooldown, table or DB call appears here.
No direct kick call appears. See doc 07 part 4.
❓ Uppercase `Q` encoding and common task lifecycle remain open.
Acceptance: compare explicit retract's `Q=0` with automatic retract's `Q=1`. Test caller state !=2 and peer state 3 separately.

## Command 339 — G_ConfirmTradeReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x140396060` |
| 348 name | `HandleGame_ConfirmTradeReq` |
| State | **partial** |
| Request | No body read; effective consumed prefix `h`. No request decoder. |
| Failure reply | Usually 340, `hh`; task missing-trade branch uses **321**, `hh`, reason 940 |

### Ordered preparation

1. Apply T1 only, with failure command 340. Require zone.
2. Post a 0x90-byte task, kind 53, vtable `0x140BA73B0`. Queue failure sends 340 reason 2.
3. Task `0x1403E4FB0` holds caller.trade. Missing trade sends **321 reason 940**.
4. Call `0x1403E2550(trade,caller,&unlink_flag)`. It initializes the flag to zero.
5. Require caller, caller.trade == trade, inviter, inviter.trade == trade, and guest. Invariant failures assert/log and return false without a direct reply from this helper.
6. Require caller membership, else 340 reason 940. Require equal non-null zones, else 939.
7. Apply timed-user gate to inviter, then guest. Either rejection sends 340 reason 2.
8. Require caller state ==2 and peer state in {2,3}, else 340 reason 941.
9. If peer state ==2, set caller state to 3. Send command 341, `hq`, caller ID, to both PCs. Leave unlink_flag zero.
10. If peer state ==3, set unlink_flag to 1 **before** final transfer checks. Run the checks below. The task unlinks through `0x1403E3B40` whenever this flag is set, even if final checks or a later operation fail.

### Final checks, in order

A final-check failure calls `0x1403E0230(trade,cancel_actor,cancel_byte)`.
It returns both escrow inventories and sends 322 to both PCs as specified for command 320.
It does not send an invented 340 reason for these branches.

| Order | Check | Cancel actor | Cancel byte |
| ---: | --- | --- | ---: |
| 1 | Inviter money >= inviter offer | Inviter | 5 |
| 2 | Guest free-slot estimate >= inviter escrow count | Guest | 9 |
| 3 | Inviter offer <= money maximum minus guest current money | Guest | 10 |
| 4 | Every non-null inviter escrow item in slots 1..8 has native +16 !=0 | Inviter | 5 |
| 5 | Guest money >= guest offer | Guest | 5 |
| 6 | Inviter free-slot estimate >= guest escrow count | Inviter | 6 |
| 7 | Guest offer <= money maximum minus inviter current money | Inviter | 7 |
| 8 | Every non-null guest escrow item in slots 1..8 has native +16 !=0 | Guest | 5 |

The free-slot estimate uses the append formula, but it does not add 1 here.
Instruction inspection in `0x1403E2550` confirms both instances: `s16(usable_limit - receiver.s8[15096] - receiver.s16[15082])`. Preserve 16-bit subtraction and the signed input fields.
The money maximum is `qword_140EE5418.qword[20224]`.
The headroom checks use current money before the two transfers, not a netted final balance.
No fresh distance check appears in this function.

### Transfer and backend prefix

1. Call `0x1403E3E80` for inviter→guest offered money, then guest→inviter offered money.
2. That helper calls money change `0x140267EA0` on sender with negative amount and reason 551, then on receiver with positive amount and reason 151. It flushes each property delta through `0x140260850`. It ignores the money-helper result. Instruction inspection in `0x1403E3E80` shows `mov rdx,rdi; neg rdx`: sender debit uses full 64-bit negation. Do not copy the decompiler's 32-bit cast.
3. `0x140267EA0` returns false for zero delta or null batch. It reads property 109, applies old+delta through `0x140696F20`, and, for flag zero, also updates property 110. A failure on the second property returns after the first property was changed. Changed property 109 queues a delta through `0x140273E10`. ❓ Property cap/clamp and packet/persistence behavior remain open.
4. When the optional audit service at daemon +2728 has a non-null +32, call `0x1403E74A0`.
5. Call `0x140515960` on inviter escrow into trade item array +72, then guest escrow into array +184.
6. If either returned count is positive, register the trade in the daemon map protected by critical section +86640. Allocate a positive ID from daemon +86688, avoiding existing IDs. Store it at trade +24. Hold a trade reference.
7. If config +537 !=0, audit non-null entries in both eight-item arrays through `0x1403E52E0`, operations 2201/2202.
8. Call `0x1403E31B0(trade,trade_id,confirming_caller_id)`.
9. This helper builds backend command **274**, descriptor `hqqqqCcihq;Ccihq;`. Prefix: trade ID, confirming caller ID, inviter identity from PC +20744, guest identity from PC +20744. Each counted group lists the respective eight-item array's non-null entries. Each entry has item-kind selector (kind 1/2/3/4/5 maps to 3/4/5/11/15; other kinds map to 0), item-record DWORD +8, record byte +12, and item native ID +16. ❓ Check descriptor expansion against composer instructions before assigning exact group field widths.
10. It resolves a backend session through daemon +3112 virtual +56 using inviter PC +20664. It sends via `0x1406A5D60` when that session exists. Missing session returns false; this caller ignores the result.
11. The item-bearing branch does not directly send command 341 at this point. If neither escrow produced items, send 341, `hq`, confirming caller ID, to both PCs after the money calls.
12. The task unlinks when unlink_flag is nonzero.

### Gaps and acceptance

- ❓ Read `0x140515960`, backend command-274 response handlers, success/failure item delivery, rollback/timeout and map cleanup. No DB procedure is verified yet.
- ❓ Resolve money property setter, audit effects, item destruction and all inventory/property reply IDs. Sender negation width and both final capacity formulas have instruction proof.
- ❓ Common task lifecycle remains open. No direct kick appears in the read prefix; untraced completion paths remain unknown. Do not copy the BoardGacha kick policy from doc 07 part 4.
- Acceptance: test first and second confirmation separately. Test every final-check cancel actor/byte. Test money-only, item-bearing, missing backend session and backend failure cases. Verify money calls occur before command 274.

## Common account-inventory transaction rules

Sources: `0x1403C0200`, `0x1403C0460`, `0x1403C0780`, transaction runners below.
These handlers do not use the complete trade T1 gate.
First require network config byte +404 !=0, else the handler's failure reply with reason 2.
Then reject config byte +237 !=0 AND DWORD +248 !=0 with reason 5.
They do not read the trade-state DWORD +37520 or T2 byte +13768 here.
Resolve the user from PC +20696 only when PC +20752 ==0.
Missing user logs and returns true **without a failure packet**.
These top-level functions do not require a zone or NPC distance.

`0x14002F440` initializes a user-owned transaction. The user pointer is at transaction +80.
Transaction ID is at +24. State is at +32.
`0x1400D4FE0(daemon+5080,transaction)` registers it.
Registration failure sends reason 2 to requester.
After registration, `0x1400D4A80` starts it with an operation event and optional payload.
This wrapper locks transaction +40. It skips a transaction already at its terminal state.
Otherwise it calls virtual +40, then tests terminal state and calls manager cleanup `0x1400D51C0`.
The default virtual +40 at these vtables is `0x14002F3D0`; the concrete event handler is virtual +48.
❓ The default adapter, registration failure cleanup and timer registration need full traces.

While state ==0, a valid start event sets state 1 and then processes event 0.
An invalid start event sets terminal state 2.
If config +76 !=0, allocate/register a timeout object through `0x1400D4D30`.
Timeout registration or outbound request failure sends the handler failure reply with reason 2 and sets state 2.
Backend requests use `0x140020870(daemon+5152,user.s16[236],format,...)`.
❓ This endpoint's named daemon and routing configuration are not verified. No DB procedure is verified.
Backend reason conversion uses `0x140018AE0`; use the complete map in `07-server-validation.md`, part 4.
This mapper is shared. Its use does not import BoardGacha's kick policy.

## Command 793 — G_GetGoodsCountFromAccountInventoryReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403C0200` |
| 348 name | `HandleGame_GetGoodsCountFromAccountInventoryReq` |
| State | **partial** |
| Request | No body read; no decoder. Effective consumed prefix `h`. |
| Failure reply | 795, `hh`: reason |

1. Apply common account-inventory config and user gates.
2. Create a 0x60-byte transaction, vtable `0x1409E5DE0`. Start it with event 57 and null payload. Top-level handler returns true after releasing local references.
3. Concrete event runner is `0x1401BE670`. Valid start sets state 1. Optional timeout event is 73.
4. On event 0, send backend command **4**, format `hqo`: transaction ID, 16-byte value copied from user +200. ❓ Meaning and exact `o` encoding remain open; do not replace it with text.
5. Event 71 removes the timeout registration through `0x1400D4E90`. Resolve user session through virtual +48. When present, send success command **794**, `hi`, with payload DWORD at event data +0 as the goods count. Then set state 2. Missing session still ends the transaction.
6. Event 72 reads event payload unsigned short, removes timeout registration, maps it through `0x140018AE0`, and sends 795 `hh` when session exists. Then set state 2.
7. Event 73 maps backend reason 3 to client reason **2**, sends 795 `hh` when session exists, and sets state 2. Other events in state 1 return true without a state change.

No inventory item, money, cooldown or game-table lookup appears in this runner.
No direct kick call appears in the recovered path.
❓ Endpoint routing, timeout setup/expiry units, transaction adapter/cleanup and DB procedure remain open.
Acceptance: test disabled feature, restricted config, missing user, registration failure, outbound failure, count success, backend failure and timeout. Verify missing user has no packet; verify success uses 794, not 795.

## Command 796 — G_ListGoodsFromAccountInventoryReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403C0460` |
| 348 name | `HandleGame_ListGoodsFromAccountInventoryReq` |
| State | **partial** |
| Request | `hc`: command, signed selector byte; ❓ semantic name |
| Failure reply | 798, `hh`: reason |

1. Apply the two common config gates.
2. Initialize selector to zero. Decode `hc`. A zero low-16-bit decoder result sends 798 reason 2. Require signed selector >=0, else the same reply. Thus accepted bytes here are 0..127.
3. Resolve user with the common rule. Missing user logs and sends no packet.
4. Create a 0x68-byte transaction, vtable `0x1409E6120`. Start event 58 carries a pointer to the selector byte. Source: assembly at the `0x1400D4A80` call in this handler.
5. Runner `0x1401BE980` copies a non-null start payload byte to transaction +88. Valid event 58 sets state 1. Optional timeout event is 76.
6. Event 0 sends backend command **7**, `hqoc`: transaction ID, 16-byte user +200 value, selector.
7. Event 74 calls success builder `0x1401BEC10`. Apply the success-builder rules below.
8. Event 75 reads an unsigned-short backend reason, removes timeout registration, maps the reason through `0x140018AE0`, sends 798 `hh` to the user session when present, and sets state 2.
9. Event 76 maps backend 3 to client 2, sends 798 `hh` when session exists, and sets state 2. Other events in state 1 do nothing and return true.

### Success event 74

Source: `0x1401BEC10`; descriptor bytes at `0x140A19B80`.

1. Remove timeout registration. Read the encoded backend packet from event data.
2. Build command **797**, exact descriptor `hiCqiQQqqssUhqissbbCiiiibb;;`.
3. Read an outer signed-short count through `0x14069DA00`. Decode 16 fields per outer entry through `0x14069DD00` into a 0x368-byte native temporary. Two blob limits are 32 and 30.
4. Four UTF-16 strings contribute to the size estimate. Start another output packet when the remaining 4000-byte budget is smaller than `signed_blob_lengths + 58 + 2*sum(four_string_lengths)`. Copy fields through `0x140017510`.
5. Read nested counted entries. Each has six decoded fields. Its size threshold uses `signed_blob_lengths + 20`. Maintain counted-group metadata across packet splits.
6. Destroy the backend input packet. Finalize the output packet chain. Resolve user session virtual +48.
7. If a session exists, call `0x1406AB3F0` with the chain and session +264/+288. Lock session +24. Queue the chain through `0x1406A6420(...,1)`. Unlock and release the session. Set transaction state 2 even when no session exists.

No direct inventory or balance mutation appears in this builder.
❓ Semantic field names, complete counted-group/blob encoding, malformed-backend handling and exact packet-split reproduction remain open. The descriptor alone is not a complete wire contract.
❓ Endpoint routing, timeout/transaction cleanup and DB procedure remain open.
No direct kick call appears in the read runner or success builder. Unread dependencies remain unknown.
Acceptance: test selector -128/-1/0/127, parse failure, missing user, registration failure, outbound failure and each event. Do not invent success fields from command names.

## Command 799 — G_TakeGoodsFromAccountInventoryReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403C0780` |
| 348 name | `HandleGame_TakeGoodsFromAccountInventoryReq` |
| State | **partial** |
| Request | `hqc`: command, 64-bit goods selector, selector byte; ❓ exact field names |
| Failure reply | 801, `hh`: reason |

1. Apply common config gates, then resolve user. Missing user logs and sends no packet.
2. Decode `hqc`. The handler does not test the decoder result and does not bound either body field here.
3. Create a 0x70-byte transaction, vtable `0x1409E6D70`. Start it with event 59 and a pointer to the decoded nine-byte pair. Source: handler assembly at the `0x1400D4A80` call.
4. Runner `0x1401BF440` requires start event 59 and non-null payload. Copy qword to transaction +88 and byte to +96. Set state 1. Optional timeout event is 79.
5. Event 0 sends backend command **10**, descriptor `hqoqc`: transaction ID, 16-byte user +200 value, goods selector, selector byte. ❓ The final variadic byte argument is absent in decompiler output; confirm it in assembly.
6. Event 77 calls `0x1401BF770`. ❓ Item grant, success command 800, inventory replies and completion behavior remain untraced at this point.
7. Event 78 reads an unsigned-short backend reason and removes timeout registration. Resolve user PC at user +720 only when user +1017 ==0. When present, send 801 `hh` through its virtual send with mapped backend reason. Set state 2.
8. Event 79 uses the same PC resolution and sends mapped backend reason 3, client reason 2. Set state 2. Unlike count/list, these two paths resolve a PC rather than user session virtual +48.

No ownership, available-goods count, money, slot capacity, cooldown or table check appears in the top-level handler. This does not prove that the untraced grant path omits them.
❓ Success grant/task/transaction paths, sender byte argument, endpoint routing, decoder failure, rollback, timeout and DB procedure remain open.
No direct kick call appears in the read prefix. Unread grant paths remain unknown.
Acceptance: test missing user before decode; malformed request; zero/max selectors; registration/outbound failure; backend success/failure/timeout; and missing PC on callbacks.

## Command 802 — G_RetrieveWarehouseLobbyServiceItemListReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403C0A20` |
| 348 name | `HandleGame_RetrieveWarehouseLobbyServiceItemListReq` |
| State | **partial** |
| Request | Literal `hQq`: command, `Q` selector, 64-bit token; ❓ semantic names and `Q` encoding |
| Failure reply | 804, literal `hEQq`: reason, request selector, request token |

1. Decode the request **before** config or user gates. The handler does not test the decoder result.
2. Require config +404 !=0, else 804 reason 2. Reject config +237 !=0 AND DWORD +248 !=0 with 804 reason 5. Top-level failure replies include the decoded selector and token.
3. Resolve user with the common account-inventory rule. Missing user logs and sends no packet.
4. Create a 0x70-byte transaction, vtable `0x1409E5E18`. Register and start event 60 with the decoded pair. Registration failure sends 804 reason 2 with request selector/token. All top-level paths return true.
5. Runner `0x1401C0140` requires event 60 and non-null payload. Copy payload byte +0 to transaction +96 and qword +8 to transaction +104. Set state 1. Optional timeout event is 82.
6. Event 0 sends backend command **13** through `0x140020870`. ❓ Exact descriptor and trailing variadic arguments need instruction proof. It uses transaction ID, user +200 value, selector and token.
7. Timeout registration or request failure sends 804 reason 2 through user session virtual +48, then sets state 2. ❓ Confirm the echoed selector/token arguments in these runner sends; the decompiler omits them.
8. Event 80 calls `0x1401C0410`. ❓ Success format and list construction remain open.
9. Event 81 removes timeout registration and maps its payload unsigned-short backend reason through `0x140018AE0`. Send 804 when a session exists, then set state 2.
10. Event 82 maps backend 3 to client 2, sends 804 when a session exists, then sets state 2. Other events in state 1 return true without change.

No inventory grant, money, ownership, slot, distance, cooldown or game-table check appears in this prefix.
❓ Success builder, all echoed callback arguments, endpoint routing, decoder behavior, timers, cleanup and DB procedure remain open.
No direct kick appears in the read prefix. Do not infer behavior for unread paths.
Acceptance: verify decode precedes feature rejection. Verify top-level failures echo both request fields. Test missing user, registration/request failure, success, backend failure and timeout.

## Command 805 — G_RetrieveWarehouseItemListReqCmd

| Field | Value |
| --- | --- |
| 381 target | `0x1403C0D70` |
| 348 name | `HandleGame_RetrieveWarehouseItemListReq` |
| State | **partial** |
| Request | Literal `hQqc`: command, `Q` selector, 64-bit token, signed selector byte; ❓ semantic names and `Q` encoding |
| Failure reply | 807, literal `hEQq`: reason, first selector, token |

1. Decode before config or user gates. The handler does not test the decoder result.
2. Require config +404 !=0, else 807 reason 2. Reject config +237 !=0 AND DWORD +248 !=0 with 807 reason 5.
3. Require signed final selector byte >=0, else 807 reason 2. These top-level replies echo the first selector and token, not the final selector byte.
4. Resolve user with the common account-inventory rule. Missing user logs and sends no packet.
5. Create a 0x70-byte transaction, vtable `0x1409E5C80`. Register and start event 61 with the decoded fields. Registration failure sends 807 reason 2 and echoes first selector/token. All top-level paths return true.
6. Runner `0x1401C0840` requires event 61 and non-null payload. Copy first selector byte to transaction +96, token qword to +104, and final selector byte to +97. Set state 1. Optional timeout event is 85.
7. Event 0 sends backend command **16** through `0x140020870`. ❓ Exact descriptor and trailing variadic arguments need instruction proof. It uses transaction ID, user +200 value and all three request fields.
8. Timeout registration/request failure sends 807 reason 2 to the user session and sets state 2. ❓ Echoed callback arguments need instruction proof.
9. Event 83 calls `0x1401C0B20`. ❓ Success format and list construction remain open.
10. Event 84 removes timeout registration, maps unsigned-short payload backend reason and sends 807 when a session exists. Set state 2.
11. Event 85 maps backend 3 to client 2, sends 807 when a session exists, then sets state 2. Other events in state 1 return true without change.

No inventory grant, balance, ownership, slot, distance, cooldown or table check appears in this prefix.
❓ Success builder, echoed callback fields, endpoint, decoder, timers, cleanup and DB procedure remain open.
No direct kick appears in the read prefix. Unread success paths remain unknown.
Acceptance: test final selector -128/-1/0/127, each earlier config failure, missing user, registration/request failure and all callback events. Verify echo excludes the final byte.

## Rust port status (part 1)

Source: `Program/Server/GameDaemon/src/handlers/trade.rs`, tests `tests/gi1_trade.rs`.

| Command | Rust | Boundary that stops the port |
| --- | --- | --- |
| 121, 123 | ported | — |
| 311, 314, 317 | partial | Short-body decode result; audit helpers (+537) not ported |
| 320 | partial | Escrow return `0x140515A80` when either escrow holds an item |
| 324 | partial | Escrow insert `0x140515360` and eligibility `0x1405593A0` after the 927/926 gates |
| 327 | partial | Escrow return after the 931 gate |
| 330, 333, 336 | ported | — |
| 339 | partial | Item transfer `0x140515960` and backend command 274; money-only confirm is ported |
| 793–805 | partial | Transaction runner, backend commands 4/7/10/13/16 and callbacks |

- Uppercase `Q` is one signed byte (BloodLib codec, `0x14069DA00` family). The port encodes 322/338 `Q` and decodes 802/805 `Q` as one byte.
- Request handlers queue a `TradeTask`. `execute_trade_task` runs the task body on adapter-owned `TradeState`.
