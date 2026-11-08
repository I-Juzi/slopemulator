# 22 — Shop handlers (GameDaemon 381.8867)

Source: `ida/delegate/gamed/GameDaemon.exe.i64` (build 381.8867), read through
IDA MCP. Handler addresses match `parity-ledger.tsv`, column `handler381`.
All addresses in this document apply to this build.

Descriptor sources: `idc.get_strlit_contents`. Descriptor reads use
`ida_nalt.STRTYPE_C`. Wide log reads use `ida_nalt.STRTYPE_C_16`.
`h` is the command header or a two-byte field. `E` is a two-byte reason.
`Q` and `c` are one byte. `i` is four bytes. `q` is eight bytes.
`C` is a two-byte list count. The count reader `0x14069DA00` returns a short.
Missing counts return −1. Each handler applies the count limits below.
`b` is a two-byte signed length followed by that many bytes. These entries
set its capacity to eight bytes. Decoder `0x1406AA7B0` rejects negative
lengths, lengths above capacity and incomplete data. It does not truncate.
Reply bodies below exclude the command header. Multi-byte fields use
little-endian order. Brackets show fields, not individual bytes.

Coverage: 80, 827, 940, 944, 1008, 1345, 1436, 1439, 1566, 1569, 1572,
1575, 1578, 1584. These are entry-function ports. A queued task is not a
completed purchase. The helper `0x14060E280` was not read or called.

## Shared fields

| Field | Native source | Rust |
| --- | --- | --- |
| Zone | PC qword +2224 | `Pc::zone` |
| Caster | PC qword +40 | `Pc::object_id` |
| Native kind | PC WORD +46; 1 = PC | `Pc::native_kind` |
| Available user | PC qword +20696 nonzero and byte +20752 zero | `Pc::user_present` |
| Fatigue check | available user DWORD +120 = 0 | `ShopCheck::UserNotFatigued` |
| Instant payment in progress | PC byte +21080, read under lock +21072 | `Pc::instant_payment` |
| Leave action absent | PC qword +2216 = 0 | `ShopCheck::NoLeaveAction` |
| Arena configuration | daemon config at global `0x140F76880` +2736; byte +237 nonzero and DWORD +248 nonzero | `World::arena` |
| PC arena state | PC DWORD +37520 under critical section +37480; blocked values 2, 3, 5, 6 | `session::arena_blocked` |
| Live-server gate | daemon config byte +746 = 0 | `ShopCheck::LiveServer` |
| Account inventory enabled | daemon config byte +404 nonzero | `World::account_inventory_enabled` |
| Mileage enabled / limit | rule config at global `0x140EE58A0`; byte +160 / unsigned DWORD +164 | `ShopCheck::MileageEnabled` / `MileageWithinLimit` |
| Collection enabled | rule config byte +2076 nonzero | `ShopCheck::CollectionEnabled` |
| Expansion record | data manager at global `0x140F77978`, qword slot 1023, virtual +168; key bytes [store, slot] | `ShopCheck::ExpandRecord` |
| Expansion payment enabled | expansion record qword +32 nonzero | `ShopCheck::ExpandPaymentEnabled` |
| Collection group | data manager qword slot 507, virtual +168 | `ShopCheck::CollectionGroup` |
| Collection item / charge item | data manager qword slot 1455, virtual +168 | `ShopCheck::CollectionItem` / `CollectionChargeItem` |
| Collection group match | subtype-specific item DWORD equals low DWORD of group key | `ShopCheck::CollectionGroupMatch` |
| Task state | WORD +136 kind; qword +128 caster; byte +138 attached | `SkillTask`, `SkillTaskKind::Shop` |
| Zone attach / queue | `0x14025DB30` / `0x140254FD0` | `skills::queue` → `Services::queue_skill` |
| Async post | `0x140603BD0`; entries do not test its return value | `Unsupported` boundary for 940, payment-4 paths and 1584 |
| Opaque server state | config, record tables and user state | `Services::shop_check`; default `Gate::Unknown`; mock `MemoryShop` |

Only 940 checks both arena configuration and PC arena state. Command 1584
checks arena configuration only. Commands 1569 and 1572 check arena
configuration only on payment 4. No entry in this set checks PC +13768.
Only 80, 940, 1008, 1345, 1436 and 1439 explicitly require native kind 1.
The common helper `0x14069BF60` records a task reference. It is not an extra
PC-kind gate.

Pair lists use wire `hh` and native stride 4. Most entries retain both
shorts without a value check. Command 1569 requires the second short to be
positive as a signed short. ❓ Pair domain names need task execution.
The names store, payment, draw, slot, seller, goods and price below follow
the port and native log text. ❓ Their full domain rules need task execution.

The current IDA signatures for the decode helpers omit variadic output
arguments. This can remove valid branches from decompiled output.
Disassembly confirms the full paths for 1575 and 1578. Disassembly also
confirms reason 1553 on the nonpositive-pair path in 1569.

## 80 — G_RebuyItemListReqCmd — partial

Handler `0x140376820`. No request descriptor. The entry reads no body fields.

1. No zone → reply 82 `hh` [1], then log (`0x140BCB2A0`).
2. Allocate kind 61, size 0x90, vtable `0x140BA7738`.
3. Native kind ≠ 1 → crash log, reply 82 `hh` [1].
4. Attach zone/caster. The task has no request fields after its base state.
5. Queue failure → reply 82 `hh` [1]. Queue success → no reply here.

❓ Missing: task execution, rebuy-list contents, success reply, logger side
effects and persistence. The entry port ignores the request body.

## 827 — G_RestoreVitalityReqCmd — partial

Handler `0x1403C1E40`. No request descriptor. The entry reads no body fields.

1. Log "[vitality], deprecated packet, Pc:0x%I64x" (`0x140C1EE00`).
2. Return true. No reply command, reason or body. No zone check, task kind,
   vtable, task fields or state change.

❓ Missing: logger side effects. All entry branches were read. Rust records
`vitality: deprecated packet`. The entry does not restore vitality.

## 940 — G_DepositMileageReqCmd — partial

Handler `0x1403CA550`. Descriptor `0x140C2A650`.
Constructor `0x1402AEFF0`. Initializer `0x1402AF130`.

Wire: `hhhChh;Chh;`: plan short, count short, first pair count/list, second
pair count/list. Both lists have their own count. The plan and count retain
unsigned 16-bit values. ❓ The two list roles need validator/task analysis.

All failures below use reply 942 `hh` with the indicated reason.

1. Arena configuration active or PC arena state in {2, 3, 5, 6} → [5]. This gate precedes decoding.
2. Rule byte +160 = 0 → [2177].
3. Decode plan/count. Failure, plan = 0 or count = 0 → [2177].
4. Unsigned count > rule unsigned DWORD +164 → [2179].
5. First list count outside 1..5, including a missing count → [2182]. A failed pair read → [2177].
6. Second list count outside 1..5, including a missing count → [2184]. A failed pair read → [2177].
7. No zone → [2]. Allocate kind 224, size 0xB0, vtable `0x140A8F420`.
8. Initializer requires native kind 1 and a nonnull request pointer. Failure → [2]. It then calls validator `0x1402AF440`. Validator failure makes initialization fail; the entry sends [2]. ❓ Validator checks and any additional replies were not read.
9. Successful initialization sets DWORD +144 = 1. Allocate a 0x30-byte request copy at qword +152. Store plan/count at copy +0/+2, first count at +4 and second count at +26. Store the first list's short arrays at +6/+16. Store the second list's short arrays at +28/+38. Attach zone/caster.
10. Post through `0x140603BD0`, with the fourth argument zero. The entry does not test the post result. No success reply here.

❓ Missing: mileage validator, plan/item tables, task execution, async post,
success reply and persistence. Rust implements checks 1..8 through the
native-kind gate. It returns `Unsupported` before the validator. It does
not allocate or post a mileage task.

## 944 — G_RetrieveMysteriousStoreReqCmd — partial

Handler `0x1403CABA0`. Descriptor `0x1408D5024`.
Reply descriptor `0x1408B8898`.

Wire: `hq`: key qword. The key starts at zero. The entry ignores the decode
result. A short read leaves the key zero; it does not retain a partial qword.

1. Decode the key.
2. No zone → log, reply 946 `hEq` [2, key]. The body has ten bytes.
3. Allocate kind 228, size 0x98, vtable `0x140B148E0`. Set qword +144 = key. Attach zone/caster. No PC-kind test.
4. Queue failure → log, reply 946 `hEq` [2, key]. Queue success → no reply here.

❓ Missing: key domain, task execution, store contents, success reply and
persistence. Rust retains the initialized-key behavior on malformed input.

## 1008 — G_SellOldItemReqCmd — partial

Handler `0x1403CF690`. Descriptor `0x140C311EC`.

Wire: `hQChh;`: depot byte, pair count, then pairs of shorts.

All failures below use reply 1009 `hh` with the indicated reason.

1. No available user or user DWORD +120 ≠ 0 → [2291]. This check precedes decoding.
2. Depot decode failure → [2286]. Depot not in {2, 3, 6, 8} → [2286]. The native mask is 332.
3. Count < 0 or count ≥ 16 → [2287]. A missing count uses this path. The accepted maximum is **15**.
4. A failed pair read → [2286]. Zero count is valid. The entry does not check the pair values or reject duplicates.
5. No zone → [2].
6. Allocate kind 246, size 0xD8, vtable `0x140B96C58`. Native kind ≠ 1 → crash log, then [2].
7. Attach zone/caster. Set byte +144 = depot, DWORD +148 = count. Copy pairs to +152, stride 4.
8. Queue failure → [2]. Queue success → no reply here.

❓ Missing: user-state service internals, task execution, depot/item rules,
sale value, success reply and persistence. Unknown fatigue state returns
`Unsupported`. The second pair short retains all its bits.

## 1345 — G_MysteriousBuyReqCmd — partial

Handler `0x1403775C0`. Descriptor `0x140BCBB48`.

Wire: `hqcChh;`: seller qword, index byte, pair count, then pairs of shorts.

All failures below use reply 1347 `hh` with the indicated reason.

1. No available user or user DWORD +120 ≠ 0 → [355]. This check precedes decoding.
2. Seller/index decode failure → [351].
3. Count < 0 or count > 16 → [351]. A missing count uses this path. A failed pair read → [351]. Zero count is valid. The entry does not check seller, index or pair values, or reject duplicates.
4. No zone → [2].
5. Allocate kind 229 through `0x14028DF50`, size 0x110, vtable `0x140BC22F0`. Native kind ≠ 1 → crash log, then [2].
6. Attach zone/caster. Set qword +144 to the `MysteriousBuyTask` label (`0x140D2D398`), qword +152 = seller, byte +160 = index and short +162 = 0. Clear qwords +168/+176/+184 and DWORD +192. Set DWORD +196 = count. Copy pairs to +200, stride 4. Set WORD +264 = 1.
7. Queue failure → [2]. Queue success → no reply here.

❓ Missing: user-state service internals, task execution, seller/store tables,
price checks, success reply and persistence. Unknown fatigue state returns
`Unsupported`. Rust retains the request fields, not the native task label or
constructor bookkeeping.

## 1436 — G_R_RestoreEnchantFailedItemReqCmd — partial

Handler `0x140380770`. Descriptor `0x140BD2DD4`.

Wire: `hQhhq`: depot byte, first short, second short, key qword.
❓ The short and key domain meanings need task execution.

1. Decode failure → log, reply 1438 `hh` [2].
2. No zone → log, reply 1438 `hh` [2].
3. Allocate kind 347, size 0xA0, vtable `0x140CF1800`.
4. Native kind ≠ 1 → crash log, reply 1438 `hh` [2].
5. Attach zone/caster. Set byte +144 = depot, shorts +146/+148 = first/second and qword +152 = key. The entry does not check their values.
6. Queue failure → log, reply 1438 `hh` [2]. Queue success → no reply here.

❓ Missing: task execution, enchant-restore rules, item tables, success reply
and persistence. Rust retains all decoded bits, including an unknown depot.

## 1439 — G_R_ResetEnchantRestoreInfoReqCmd — partial

Handler `0x140380520`. Descriptor `0x140BD2C3C`.

Wire: `hQh`: depot byte, item short. ❓ The short's domain meaning needs task
execution.

1. Decode failure → log, reply 1441 `hh` [2].
2. No zone → log, reply 1441 `hh` [2].
3. Allocate kind 346, size 0x98, vtable `0x140CF1630`.
4. Native kind ≠ 1 → crash log, reply 1441 `hh` [2].
5. Attach zone/caster. Set byte +144 = depot, short +146 = item. The entry does not check their values.
6. Queue failure → log, reply 1441 `hh` [2]. Queue success → no reply here.

❓ Missing: task execution, reset rules, item tables, success reply and
persistence. Rust does not add a depot-range or fatigue gate.

## 1566 — G_RandomStoreInfoReqCmd — partial

Handler `0x1403C6EC0`. No request descriptor. The entry reads no body fields.

1. No zone → log (`0x140C25B70`), reply 1568 `hh` [2].
2. Allocate kind 171, size 0x90, vtable `0x140BC2000`. Attach zone/caster. No PC-kind test. The task has no request fields after its base state.
3. Queue failure → log, reply 1568 `hh` [2]. Queue success → no reply here.

❓ Missing: task execution, random-store state and tables, success reply and
persistence. The entry port ignores the request body.

## 1569 — G_RandomStoreDrawReqCmd — partial

Handler `0x1403C7070`. Descriptor `0x140C25808`.
Reply descriptor `0x1408B8860`. Constructor `0x14011D0C0`.
Normal initializer `0x1402A4CC0`. Payment-4 initializer `0x1402A4F80`.

Wire: `hcQcibChh;`: store byte, payment byte, draws byte, goods DWORD,
price blob, pair count, then pairs of shorts. The three bytes start at zero.
Goods and price storage start at zero. Price capacity is eight bytes.

All failures below use reply 1571 `hEc` [reason, store]. The body has three
bytes. A decoded store survives a later decode failure.

1. Fixed-field or blob decode failure → [2115, store]. Blob length > 8 also uses this reason.
2. Pair count < 0 or count > 16, including a missing count → [2115, store]. Zero count is valid except for payment 2.
3. A failed pair read or signed second short ≤ 0 → [1553, store]. These checks precede store/payment checks. Disassembly `0x1403C7358..0x1403C738A` sets reason 1553; the decompiler omits reply arguments on this path.
4. Store outside 1..2 or payment outside 1..4 → [2116, store]. Payment 2 with zero pairs → [2116, store].
5. Instant payment already in progress → [2126, store].
6. PC qword +2216 nonzero → [2125, store].
7. No available user or user DWORD +120 ≠ 0 → [2118, store].
8. No zone → [2, store]. Draws not 1 or 5 → [2116, store]. No PC-kind test.
9. For payment 1, 2 or 3, allocate kind 172, size 0xF0, vtable `0x1409CEC68`. Attach zone/caster. Set DWORD +144 = 2; bytes +168/+169/+170 = store/payment/draws. Copy pairs to +172, stride 4. Set DWORD +236 = count. The normal task does not retain goods or price. Initializer or queue failure → [2, store]. Queue success → no reply here.
10. For payment 4, daemon config byte +746 nonzero → [15, store]. Then active arena configuration → [5, store]. Do not test PC arena state on this branch.
11. Get the available user again. Failure → [2, store]. Allocate a 0x60-byte transaction, vtable `0x1409CEE88`. Allocate a 0x160-byte operation, vtable `0x140BC20B0`; bytes +344/+345 = store/draws. Store the operation at transaction qword +88. Prepare one goods/price order from the request.
12. Transaction registration through `0x1400D4FE0` fails → [2, store]. After registration, allocate the same kind-172 task. Initialize mode DWORD +144 = 1, transaction qword +152, qword +160 = 0 and bytes +168/+169/+170 = store/4/draws. Initialization failure → [2, store].
13. Post payment-4 work through `0x140603BD0`, with the fourth argument's low byte set to 1. No success reply here. The entry does not test the post result.

❓ Missing: config/user services, task execution, store/payment tables, price
validation, transaction internals, payment-4 async post, success reply and
persistence. Rust queues only payments 1..3. It returns `Unsupported` after
the payment-4 config gates. Unknown leave/fatigue/config state is not approval.

## 1572 — G_ExpandRandomStoreSlotReqCmd — partial

Handler `0x1403C7CC0`. Descriptor `0x140C272A8`.
Reply descriptor `0x1408B88C0`. Constructor `0x14011D1E0`.
Normal initializer `0x1402A7060`. Payment-4 initializer `0x1402A72A0`.

Wire: `hccQibChh;`: store byte, slot byte, payment byte, goods DWORD,
price blob, pair count, then pairs of shorts. Fixed storage starts at zero.
Price capacity is eight bytes. Only payment 2 reads the pair count/list.

All failures below use reply 1574 `hEcc` [reason, store, slot]. The body has
four bytes. The first decode-failure path replaces the slot with 0xFF.

1. Fixed-field or blob decode failure → [2130, store, 0xFF]. A decoded store survives the failure. The decoded slot is not echoed on this path.
2. Store outside 1..2 → [2131, store, slot].
3. Look up slot 1023, virtual +168, with a zero-extended key whose low bytes are [store, slot]. No record → [2139, store, slot]. The entry has no separate slot-range check.
4. No available user → [2, store, slot]. Then no zone → the same reply. These gates precede the payment branch.
5. For payment 4, daemon config byte +746 nonzero → [15, store, slot]. Expansion record qword +32 = 0 → the same reply. Then active arena configuration → [5, store, slot]. No PC-arena-state test.
6. Payment 4 allocates a 0x60-byte transaction, vtable `0x1409CEE88`, and a 0x160-byte operation, vtable `0x140B9C258`. Operation bytes +344/+345 = store/slot. Transaction qword +88 holds the operation. Prepare one goods/price order. Registration failure → [2, store, slot].
7. After registration, allocate kind 173, size 0xF0, vtable `0x1409CE468`. Attach zone/caster. Set mode DWORD +144 = 1, transaction qword +152, qword +160 = 0, bytes +168/+169/+170 = store/slot/4 and DWORD +236 = 0. Initialization failure → [2, store, slot]. Post through `0x140603BD0`, with the fourth argument's low byte set to 1. The entry does not test the post result. No success reply here.
8. For any payment other than 2 or 4 → [2131, store, slot].
9. For payment 2, pair count outside 1..16, including a missing count → [2133, store, slot]. A failed pair read → [2130, store, slot]. The entry does not check pair values or reject duplicates.
10. Allocate the same kind-173 task. Attach zone/caster. Set DWORD +144 = 2, bytes +168/+169/+170 = store/slot/2. Copy pairs to +172, stride 4. Set DWORD +236 = count. The normal task does not retain goods or price.
11. Initializer or queue failure → [2, store, slot]. Queue success → no reply here. No PC-kind, fatigue, leave-action or instant-payment-in-progress gate occurs on the payment-2 entry path.

❓ Missing: expansion records, config/user services, task execution, payment
rules, transaction internals, payment-4 async post, success reply and
persistence. Rust returns `Unsupported` after the payment-4 config gates.
Unknown record or config state is not approval.

## 1575 — G_BuyRandomStoreItemReqCmd — partial

Handler `0x1403C8890`. Descriptor `0x140C284C8`.
Reply descriptor `0x1408B8904`.

Wire: `hcccChh;`: store byte, draw byte, slot byte, pair count, then pairs of
shorts. All three bytes start at zero. Each decoded byte survives a later
decode failure.

All failures below use reply 1577 `hEccc` [reason, store, draw, slot]. The
body has five bytes.

1. Three-byte prefix decode failure → [2145, store, draw, slot].
2. Store outside 1..2 or slot outside 1..8 → [2146, store, draw, slot]. Draw outside 1..5 → the same reply.
3. No available user or user DWORD +120 ≠ 0 → [2148, store, draw, slot]. This gate precedes the pair count.
4. Pair count < 0 or count > 16, including a missing count → [2145, store, draw, slot]. A failed pair read → the same reply. Zero count is valid. The entry does not check pair values or reject duplicates.
5. No zone → [2, store, draw, slot].
6. Allocate kind 174, size 0xD8, vtable `0x140BB5CA8`. The initializer repeats store 1..2 and slot 1..8 checks. No PC-kind test. Attach zone/caster.
7. Set byte +144 = store, byte +145 = draw and byte +146 = slot. Copy pairs to +148, stride 4. Set DWORD +212 = count. Disassembly `0x1403C8CB9..0x1403C8CEA` confirms this field order.
8. Initializer or queue failure → [2, store, draw, slot]. Queue success → no reply here.

The decompiler shows only prefix failure and invalid-parameter replies with
the current decode-helper signature. The stack output pointers at
`0x1403C88FB..0x1403C891E` and the branches after `0x1403C8981` confirm the
full path above. The native handler is not an unconditional failure stub.

❓ Missing: user-state service internals, task execution, store/item tables,
price rules, success reply and persistence.

## 1578 — G_TakeRandomStoreDrawRewardReqCmd — partial

Handler `0x1403C90C0`. Descriptor `0x14096CE28`.
Reply descriptor `0x1408B88C0`.

Wire: `hcc`: store byte, reward byte. Both bytes start at zero. A decoded
store survives a missing reward byte.

All failures below use reply 1580 `hEcc` [reason, store, reward]. The body
has four bytes.

1. Decode failure → [2157, store, reward].
2. Store outside 1..2 or reward negative as a signed byte → [2158, store, reward]. Accepted reward values are 0..127, not only 0 or 1.
3. No zone → [2, store, reward].
4. Allocate kind 312, size 0x98, vtable `0x140BA8F80`. The initializer repeats the store check. No PC-kind test. Attach zone/caster. Set bytes +144/+145 = store/reward.
5. Initializer or queue failure → [2, store, reward]. Queue success → no reply here.

Disassembly at `0x1403C91B9` tests the reward byte's sign. The task writes
at `0x1403C92C4..0x1403C92CB` retain both request bytes. The current
decompiled output omits the second decode output and shows a false zero
reward. Failure replies also retain the decoded reward byte.

❓ Missing: reward-index domain, task execution, reward tables, success reply
and persistence. The entry port adds no fatigue, arena or payment gate.

## 1584 — G_BuyCollectionItemsReqCmd — partial

Handler `0x1403C35D0`. Descriptor `0x140C20938`.
Reply descriptor `0x1408B8838`. Task constructor `0x14011DD50`.
Task initializer `0x1405554D0`.

Wire: `hbCibqq;`: outer price blob, count, then goods DWORD, price blob,
group key qword and item key qword for each element. All blob capacities are
eight bytes. ❓ The outer price's full domain meaning needs transaction
execution. It is not an uncounted eight-byte wire field.

All failures below use reply 1586 `hE` [reason]. The body has two bytes.

1. Active arena configuration → [5]. Do not test PC arena state.
2. Daemon config byte +404 = 0 → [15]. Then config byte +746 nonzero → [15].
3. Rule byte +2076 = 0 → [20].
4. No zone → [2]. Then instant payment already in progress → [29]. Then no available user → [2]. No PC-kind or fatigue test.
5. Outer blob decode failure → [2504].
6. Unsigned count > 10, including a missing count → [2505]. Zero count is valid at the entry.
7. Decode all elements before value or table checks. Any failed goods/blob/key read → [2504]. Each price decoder has capacity 8.
8. After decoding, the native entry checks outer and element price lengths again. A length > 8 → [2507]. Decoder `0x1406AA7B0` already rejects such lengths with decode failure. Normal wire input therefore reaches [2504], not [2507], for an oversized blob.
9. For each element in order, signed goods DWORD ≤ 0 → [2506]. Group key = 0 → [2508]. Item key = 0 → [2509].
10. Look up the group in slot 507, virtual +168. No record, record charge key qword +16 = 0 or record paid key qword +24 ≠ request item key → [2508].
11. Look up the paid item in slot 1455, virtual +168. No record or subtype WORD +2 not in {0, 1, 4} → [2509].
12. Select the paid item's group DWORD: subtype 0 uses +2432; subtype 1 uses +2012; subtype 4 uses +2124. A value unequal to the low DWORD of the request group key → [2510].
13. Look up the charge item in slot 1455 with the group record's qword +16. No record → [2508]. Copy the charge item's DWORD +8 to the native request/order charge-ID fields. Continue with the next element. The entry does not reject duplicate elements.
14. Allocate a 0x60-byte transaction, vtable `0x1409CEE88`, and a 0x268-byte operation, vtable `0x140BC1E60`. Transaction qword +88 holds the operation. Registration through `0x1400D4FE0` fails → [2].
15. Allocate kind 23, size 0x2A0, vtable `0x1409CF020`. Set DWORD +148 = 1. Copy 0x110 bytes of the native collection request to +176. The copy starts with the count short; stride-24 records start at +184 and contain charge ID at record +0, group key at +8 and item key at +16.
16. Initialize through `0x1405554D0`. Attach zone/caster. Set mode DWORD +144 = 4, transaction qword +160 and qword +152 = transaction qword +24. Initializer failure → [2].
17. Post through `0x140603BD0`, with the fourth argument's low byte set to 1. The entry does not test the post result. No success reply here.

❓ Missing: config/user services, collection/item table contents, transaction
internals, charge-ID mapping in Rust, task execution, async post, success
reply and persistence. Rust implements the ordered checks through charge
item presence. It does not copy the charge ID or start a transaction.
It returns `Unsupported` after the checks, including for zero elements.

## Rust port status

| Command | Rust | State / boundary |
| --- | --- | --- |
| 80 | `src/handlers/shop.rs` | Partial; reason 1, zone/kind gates; task execution boundary |
| 827 | `src/handlers/shop.rs` | Partial; deprecated log marker; native logger boundary |
| 940 | `src/handlers/shop.rs` | Partial; arena/config/decode/list/kind gates; validator and async-post boundaries |
| 944 | `src/handlers/shop.rs` | Partial; initialized key and ten-byte failure body; task execution boundary |
| 1008 | `src/handlers/shop.rs` | Partial; fatigue/depot gates, maximum 15; task execution boundary |
| 1345 | `src/handlers/shop.rs` | Partial; fatigue gate, maximum 16; task execution boundary |
| 1436 | `src/handlers/shop.rs` | Partial; exact decode/zone/kind failure reason; task execution boundary |
| 1439 | `src/handlers/shop.rs` | Partial; exact decode/zone/kind failure reason; task execution boundary |
| 1566 | `src/handlers/shop.rs` | Partial; zone gate without a kind gate; task execution boundary |
| 1569 | `src/handlers/shop.rs` | Partial; ordered gates and payment-1..3 task; payment-4 transaction boundary |
| 1572 | `src/handlers/shop.rs` | Partial; record/config gates and payment-2 task; payment-4 transaction boundary |
| 1575 | `src/handlers/shop.rs` | Partial; prefix echo, ranges, fatigue and list gates; task-layout comment differs |
| 1578 | `src/handlers/shop.rs` | Partial; prefix echo and signed reward gate; task execution boundary |
| 1584 | `src/handlers/shop.rs` | Partial; ordered config/decode/table gates; charge-ID, transaction and async-post boundaries |

Rust paths in this table are relative to `Program/Server/GameDaemon`.
`SHOP_COMMANDS` registers all fourteen entries. `Services::shop_check`
returns `Gate::Unknown` by default. `MemoryShop` records ordered checks.
An absent mock result is `Unknown`, not approval. `Gate::Fail` uses the
native failure plan. `Gate::Unknown` returns `Unsupported` without a reply
or a purchase change.

Zone tasks use `skills::queue` and `Services::queue_skill`. A successful
queue retains the decoded task in the memory adapter. Dispatch retains the
original body through `StateChange::QueuedRequest`. Task allocation and
execution remain adapter boundaries. No queued entry claims that a purchase
or reward is complete. Native log/assert side effects are not fully ported.
The entry ports permit trailing bytes.

`tests/gh1_shop.rs` covers registration, ordered failures, initialized prefix
echoes, list bounds, task fields and unknown adapter states. These tests
were read for this document. They were not run in this documentation-only
job.

## Review notes

| Command | Divergence | Native evidence | Rust source |
| --- | --- | --- | --- |
| 1575 | Fixed: the task-layout comment reversed draw and slot. Native byte +145 is draw; byte +146 is slot. | `0x1403C8C59..0x1403C8CCB`; decode outputs at `0x1403C88FB..0x1403C891E` | `Program/Server/GameDaemon/src/handlers/shop.rs:96` now says draw +145 and slot +146. The enum and handler retain the correct named request values. |

This is a comment divergence, not a confirmed runtime field swap. No other
runtime divergence was found in the implemented entry paths checked here.
Unimplemented services, native logs and task/transaction execution remain
the partial-port boundaries stated above. Rust was not changed.
