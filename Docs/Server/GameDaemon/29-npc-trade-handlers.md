# 29 — NPC trade, board gacha and dungeon-step handlers (GameDaemon 381.8867)

Source: `ida/delegate/gamed/GameDaemon.exe.i64`, build 381.8867. Read through
IDA MCP. Entry addresses match `parity-ledger.tsv`, column `handler381`.

Read this doc with [07 — Server validation](07-server-validation.md#step-5-part-3--interaction-distance-partial).
Doc 07 keeps the earlier task-distance and gacha-transaction evidence. This doc
specifies the receive entries. It does not move task checks into the entries.
Other store commands belong to [doc 22](22-shop-handlers.md).

Descriptor letters: `h` header or 2-byte short, `Q` / `c` 1-byte value,
`i` 4-byte integer, `q` 8-byte integer, `s` counted UTF-16. `C` is a signed
2-byte list count. `b` is a signed 2-byte byte-count, then that many bytes.
All multi-byte wire fields use little-endian order. Reply bodies exclude the
command header. Brackets list fields, not raw bytes. Rust permits trailing bytes.

Descriptor sources: `idc.get_strlit_contents`, type `STRTYPE_C`.
Wide-log sources use `ida_nalt.STRTYPE_C_16`. Decoder `0x1406AA7B0`, case `b`,
rejects negative lengths and lengths above the supplied capacity.
The count reader `0x14069DA00` returns −1 on failure.

Coverage: 1341, 1343, 1348, 1350, 1352, 1176, 1179, 1456.
All entries stay partial. A queued task does not complete a transaction.
The anti-bot helper `0x14060E280` was not read or called.

## Shared fields

| Field | Native source | Rust |
| --- | --- | --- |
| User present / available | PC qword +20696 non-null and byte +20752 = 0 | `Pc::user_present`, `!Pc::party_user_blocked` |
| User state | user DWORD +120; these entries require 0 | `Pc::party_user_state` |
| Zone | PC qword +2224 | `Pc::zone` |
| PC kind | PC WORD +46; 1 = PC | `Pc::native_kind` |
| Task owner | PC qword +40 → task qword +128 | `SkillTask::caster` |
| Task attachment | `0x14025DB30`; task byte +138 = 1 | `SkillTask::zone` |
| Arena form B | cfg +237 and +248 nonzero, or arena state 2/3/5/6 | Existing `world.arena || session::arena_blocked(pc)`; these eight entries do not test it |
| T2 / trial play | PC byte +13768 | Existing `Pc::t2_blocked`; these eight entries do not test it |
| Trial jumping | PC byte +13740 = 1 | Existing `Pc::trial_jumping`; 1179 only |
| PC lookups | daemon +5824, rights mask 6 | Existing `world.mentoring.players` / `names`; no entry here looks up another PC |
| Gacha enabled | global `qword_140EE58A0` byte +2088 AND network cfg (daemon +2736) byte +404 | `NpcTradeCheck::BoardGachaEnabled` |
| Gacha cost record | `qword_140F77978` qword slot 10305 | `NpcTradeCheck::BoardCostExists` |
| Instant-payment mode | cost record byte +42 | `NpcTradeCheck::InstantPaymentEnabled` |
| Instant-payment item | manager slot 1455, key = cost record qword +48; item DWORD +8 | `NpcTradeCheck::InstantPaymentItemExists`; returned item value stays at the transaction boundary |
| Server-data boundary | configuration and table predicates above | `Services::npc_trade`; default `Gate::Unknown`; mock `MemoryServices::npc_trade` |
| Zone task queue | `0x140254FD0` | `skills::queue` → `Services::queue_skill`; `SkillTaskKind::NpcTrade(NpcTradeTask)` |

The native logs call the nonzero user state “entered fatique time”.
Source: buy log `0x140BCA5C0`. ❓ A complete user-state enum is not verified.
No entry below checks balances, NPC distance or item ownership before it queues.
Those checks belong to later task or transaction paths.

## 1341 — G_BuyReqCmd — partial

Handler `0x140375DE0`. Descriptor `hqCch;` at `0x140BCAA10`.
Wire: NPC qword, signed i16 count, then `[index byte, count short]` per record.
❓ Record field names need the complete task. The native record stride is 4;
the wire record is 3 bytes. There is no wire padding byte.

1. User missing, unavailable or state ≠ 0 → reply 1342 `hh` [314]. This check precedes decode.
2. NPC decode failure → reply 1342 `hh` [303].
3. List count < 0 or ≥ 16, including a missing count → reply 1342 `hh` [304]. Zero is valid.
4. Any record decode failure → reply 1342 `hh` [303].
5. No zone → reply 1342 `hh` [2].
6. Allocate 0xE0 bytes, task kind 68, vtable `0x140BB66B8`.
7. Native kind ≠ 1 → crash log, reply 1342 `hh` [2].
8. Attach zone and caster. Set NPC qword +144, count DWORD +152. Copy the native records to +156, stride 4.
9. Queue failure → reply 1342 `hh` [2]. Queue success → no reply here.

❓ Missing: task execution, NPC store tables, item/money checks, transaction
commit/rollback and success reply. Doc 07 maps task `0x140520250`: NPC distance
squared > 10000 sends 1342 [306]. Rust does not run that task body.

## 1343 — G_Buy2ReqCmd — partial

Handler `0x140377060`. Descriptor `hqcchChh;` at `0x140BCB5B0`.
Wire: NPC qword, index byte, selector byte, store short, signed i16 count,
then `[short, short]` per record. ❓ Selector/store semantic names are provisional.

1. User missing, unavailable or state ≠ 0 → reply 1344 `hh` [350].
2. Scalar decode failure → reply 1344 `hh` [348].
3. Unsigned index ≥ 120 → reply 1344 `hh` [348]. This also rejects negative signed-byte encodings.
4. List count < 0 or ≥ 16, including a missing count → reply 1344 `hh` [349]. Zero is valid.
5. Any pair decode failure → reply 1344 `hh` [348].
6. No zone → reply 1344 `hh` [2].
7. Allocate 0x118 bytes. Base constructor `0x14028DF50` uses type 122. Vtable `0x140BA72B0`.
8. Native kind ≠ 1 → crash log, reply 1344 `hh` [2].
9. Attach zone and caster. Set NPC qword +152, index byte +160, store WORD +162, count DWORD +196 and pairs +200 (stride 4). Set selector byte +272 and store WORD +264. Clear qwords +168/+176/+184 and DWORD +192. Qword +144 is the `Buy2Task` log label.
10. Queue failure → reply 1344 `hh` [2]. Queue success → no reply here.

❓ Missing: task execution, alternate store selection, item/money transactions
and success reply. Doc 07 maps task `0x140526F10`: the NPC branch sends 1344
[35] when distance squared > 10000. NPC ID zero selects a separate store path.
Do not add an entry-level nonzero-NPC check.

## 1348 — G_SellReqCmd — partial

Handler `0x140376300`. Descriptor `hqChh;` at `0x140BCAE34`.
Wire: NPC qword, signed i16 count, then `[short, short]` per record.
❓ Complete item selector/count semantics need the task.

1. User missing, unavailable or state ≠ 0 → reply 1349 `hh` [322].
2. NPC decode failure → reply 1349 `hh` [315].
3. List count < 0 or ≥ 16, including a missing count → reply 1349 `hh` [316]. Zero is valid.
4. Any pair decode failure → reply 1349 `hh` [315].
5. No zone → reply 1349 `hh` [2].
6. Allocate 0xE0 bytes, task kind 69, vtable `0x140BC26F8`.
7. Native kind ≠ 1 → crash log, reply 1349 `hh` [2].
8. Attach zone and caster. Set NPC qword +144, count DWORD +152 and pairs +156 (stride 4).
9. Queue failure → reply 1349 `hh` [2]. Queue success → no reply here.

❓ Missing: task execution, ownership, sale price, money overflow, persistence
and success reply. Doc 07 maps task `0x140520E90`: distance squared > 10000
sends 1349 [318].

## 1350 — G_RebuyReqCmd — partial

Handler `0x140376D90`. Descriptor `hqc` at `0x1407FBF3C`.
Wire: NPC qword, rebuy-slot byte. The handler ignores the decode result.
Neither local has an initial value.

1. User missing, unavailable or state ≠ 0 → reply 1351 `hh` [332].
2. Decode NPC and slot without a failure branch.
3. No zone → reply 1351 `hh` [1], then log.
4. Allocate 0xA0 bytes, task kind 70, vtable `0x140BA7C30`.
5. Native kind ≠ 1 → crash log, reply 1351 `hh` [1].
6. Attach zone and caster. Set NPC qword +144 and slot byte +152.
7. Queue failure → reply 1351 `hh` [1]. Queue success → no reply here.

❓ Missing: values of unset decode locals, task execution, rebuy history,
item/money transactions and success reply. Rust returns `Unsupported` if it
needs an unset local. The user, no-zone and wrong-kind replies remain available.
Doc 07 maps distance reason 326 in task `0x140522710`; its outer reply remains
unverified in that earlier pass.

## 1352 — G_TradeWithNpcReqCmd — partial

Handler `0x140377AC0`. Descriptor `hqchChh;` at `0x140BCBFC8`.
Wire: NPC qword, selector byte, store short, signed i16 count, then
`[short, short]` per record. ❓ Scalar semantic names are provisional.

1. User missing, unavailable or state ≠ 0 → reply 1353 `hh` [347].
2. Scalar decode failure → reply 1353 `hh` [337].
3. List count < 0 or ≥ 16, including a missing count → reply 1353 `hh` [338]. Zero is valid.
4. Any pair decode failure → reply 1353 `hh` [337].
5. No zone → reply 1353 `hh` [2].
6. Allocate 0xE0 bytes, task kind 71, vtable `0x140BABD80`.
7. Native kind ≠ 1 → crash log, reply 1353 `hh` [2].
8. Attach zone and caster. Set NPC qword +144, selector byte +152, store WORD +154, count DWORD +156 and pairs +160 (stride 4).
9. Queue failure → reply 1353 `hh` [2]. Queue success → no reply here.

❓ Missing: task execution, store exchange records, ownership/cost checks,
transactions and success reply. Doc 07 maps distance reason 341 in task
`0x140522A00`; the outer reply remains unverified in that earlier pass.

## 1176 — G_BoardGachaDrawByItemReqCmd — partial

Handler `0x1403D9970`. Descriptor `hqcChh;` at `0x140BCBB48`.
Wide tag `HandleGame_BoardGachaDrawByItemReq` at `0x140C3AD80`.
Wire: board qword, draw byte, signed i16 count, then `[short, short]` per item.
❓ Complete board/draw and item-field meanings need the transaction.

1. Global byte +2088 = 0 or network cfg byte +404 = 0 → reply 1178 `hh` [2].
2. No cost record at DataManager slot 10305 → reply 1178 `hh` [2].
3. Cost record byte +42 ≠ 0 → reply 1178 `hh` [2518].
4. User missing, unavailable or state ≠ 0 → reply 1178 `hh` [2520].
5. Board/draw decode failure → reply 1178 `hh` [2519].
6. List count ≤ 0 or > 10, including a missing count → reply 1178 `hh` [2519].
7. Any item-pair decode failure → reply 1178 `hh` [2519].
8. Reacquire the user. Missing/unavailable → reply 1178 `hh` [2]. The pure Rust PC snapshot cannot change during this call.
9. No zone → reply 1178 `hh` [2]. Rust stops at the transaction boundary after this gate.
10. Allocate a 0x1A8-byte transaction, vtable `0x1409E6410`. Set transaction ID +24 from daemon callback +5128. Registration `0x1400D4FE0` under daemon +5080 fails → reply 1178 `hh` [2].
11. Allocate a 0x98-byte task, base kind 334, vtable `0x140BC2750`. Initializer `0x14054A250` needs non-null zone, PC and transaction. It sets transaction qword +144, attaches zone and caster, and sets byte +138 = 1. No PC-kind test occurs here. Failure → transaction-manager cleanup `0x1400D4A80`, reply 1178 `hh` [2].
12. Submit through `0x140603BD0`, arguments `[task, 0, 0]`, using task +32 → +16. This is not the ordinary `0x140254FD0` queue. No submit-result failure branch exists. No reply here on success.

❓ Missing: decoded-request transfer into the transaction (not shown in the
checked constructor/initializer), transaction state machine, exact manager
cleanup, special scheduler execution, board data, costs, rewards and persistence.
Rust does not claim registration or enqueue at this boundary. Doc 07 part 4
keeps the downstream 1178 failure reasons and conditional kick evidence.
No kick implementation belongs to this receive entry.

## 1179 — G_BoardGachaDrawByInstantPaymentReqCmd — partial

Handler `0x1403DA120`. Descriptor `hqcib` at `0x140C3B18C`.
Wide tag `HandleGame_BoardGachaDrawByInstantPaymentReq` at `0x140C3B370`.
Wire: board qword, draw byte, i32 value, counted byte blob (capacity 8).
Zero through eight blob bytes are valid. The zeroed local buffer supplies the
remaining bytes. ❓ The i32 and blob domain names are not verified.

1. Global byte +2088 = 0 or network cfg byte +404 = 0 → reply 1181 `hh` [2].
2. No cost record at DataManager slot 10305 → reply 1181 `hh` [2].
3. Cost record byte +42 = 0 → reply 1181 `hh` [2]. This is the opposite mode gate to 1176.
4. User missing, unavailable or state ≠ 0 → reply 1181 `hh` [2520].
5. PC byte +13740 = 1 (trial jumping, log `0x140C3B130`) → reply 1181 `hh` [32]. Do not use the +13768 T2 field.
6. Decode failure, including a negative, oversized or short blob → reply 1181 `hh` [2].
7. Set local WORD +32 = 1. Copy the first eight blob-buffer bytes (local +48) to local +16.
8. Resolve slot 1455 by cost record qword +48. No item record → reply 1181 `hh` [2]. Copy item DWORD +8 to local DWORD +40.
9. Reacquire the user. Missing/unavailable → log only, no reply. Rust stops at the transaction boundary on the available-user path. No zone or PC-kind check occurs.
10. Allocate 0x178 bytes, transaction vtable `0x1409E6158`. Set transaction ID +24 from daemon callback +5128. Register with `0x1400D4FE0` under daemon +5080. Failure → reply 1181 `hh` [2]. Success → no reply here. No zone task is allocated by this entry.

❓ Missing: decoded-request transfer into the transaction, payment request,
backend completion, failure mapping, board/reward state and persistence.
Rust validates the blob but does not register a payment transaction.
The second user read cannot differ from the first in the pure snapshot model.

## 1456 — G_DungeonProgressToStepReqCmd — partial

Handler `0x140384630`. Descriptor `hhhcqChh;` at `0x140BD7428`.
Wire: dungeon short, step short, selector byte, NPC qword, signed i16 count,
then `[short, short]` per record. ❓ Domain names need the complete task.

1. Scalar decode failure → reply 1457 `hh` [1067].
2. List count < 0 or ≥ 16, including a missing count → reply 1457 `hh` [1068]. Zero is valid.
3. Any pair decode failure → reply 1457 `hh` [1067].
4. No zone → reply 1457 `hh` [2].
5. Allocate 0xE8 bytes, task kind 121, vtable `0x140CAFE18`.
6. Native kind ≠ 1 → log, reply **1451** `hh` [2].
7. Attach zone and caster. Set dungeon WORD +144, step WORD +146, selector byte +148, NPC qword +152, count DWORD +160 and pairs +164 (stride 4).
8. Queue failure → reply **1451** `hh` [2], not 1457. Queue success → no reply here.

❓ Missing: dungeon-step state, quest conditions, cost transactions, task
execution and success reply. Doc 07 maps downstream helper `0x14058EA90`:
quest-distance helper uses constant 125, mode 1, and maps a nonzero result to
961. Its full outer reply remains open. Rust does not add this task gate to
the receive entry.

## Rust port status

| IDs | Rust | State / boundary |
| --- | --- | --- |
| 1341, 1343, 1348, 1352 | `src/handlers/npc_trade.rs` | Partial; exact entry gates and task fields; store and transaction task bodies stay opaque |
| 1350 | `src/handlers/npc_trade.rs` | Partial; unchecked decode is unsupported when unset fields are needed; task body stays opaque |
| 1176 | `src/handlers/npc_trade.rs` | Partial; config, user, wire and zone gates; transaction registration and special scheduler are unsupported |
| 1179 | `src/handlers/npc_trade.rs` | Partial; config, user, trial, wire and item-record gates; payment transaction is unsupported |
| 1456 | `src/handlers/npc_trade.rs` | Partial; exact decode and task gates; keeps failure command 1451; task body stays opaque |

Tests: `tests/gn1_npc_trade.rs`. Tests use synthetic fields, not game records.
No client data file was read. This pass found no new client-data-file fact.
