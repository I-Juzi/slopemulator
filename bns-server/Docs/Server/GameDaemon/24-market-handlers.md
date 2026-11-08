# 24 — Market handlers (GameDaemon 381.8867)

Source: `ida/delegate/gamed/GameDaemon.exe.i64` (build 381.8867), read through
the IDA MCP. Entry addresses match `parity-ledger.tsv`, column `handler381`.
All addresses below refer to this build. No anti-bot helper was read or ported.

Descriptor strings were read with `idc.get_strlit_contents`. Wide log strings
were read with `ida_nalt.STRTYPE_C_16`. `Q` and `c` use 1 byte. `h`, `C` and
`E` use 2 bytes. `i` uses 4 bytes. `q` uses 8 bytes. `o` uses 16 bytes.
`s` is counted UTF-16. The request bodies below exclude the command header `h`.
`0x1406AA7B0` verifies the field widths and stops at the first short field.
It leaves later destination fields unchanged. `0x14069DA00` verifies that
`C` uses an unsigned 2-byte count.

Coverage: 378, 381, 384, 1013, 1530, 1533, 1536, 1539, 1542, 1545, 1548,
1551, 1554. All entries remain partial. Task bodies, transactions and backend
replies are not complete market operations.

## Shared fields

| Field | Native source | Rust |
| --- | --- | --- |
| Arena form B | cfg at daemon +2736: byte +237 ≠ 0 and DWORD +248 ≠ 0, or PC DWORD +37520 in 2/3/5/6 under CS +37480 | `world.arena || session::arena_blocked(pc)` |
| Arena form A | Same cfg test only; 1551 and 1554 do not read the PC arena state | `world.arena` |
| Trial play | PC byte +13768 = 1 | `Pc::t2_blocked` |
| Register-sale switch | Global `qword_140EE58A0` byte +220 | `MarketCheck::RegisterSaleEnabled` |
| Sale/purchase price maximum | Global signed qword +8 | `MarketCheck::SalePrice`; mock `MemoryMarket::sale_price_limit` |
| Bid price maximum | Global signed qword +16 | `MarketCheck::BidPrice`; mock `MemoryMarket::bid_price_limit` |
| Auction escrow service type | Global DWORD +156; zero disables 1013 | `MarketCheck::AuctionEscrowEnabled` |
| User lookup | PC qword +20696 must be nonzero, PC byte +20752 must be zero | `Pc::user_present && !Pc::party_user_blocked` |
| PC ID / DB ID / world route | PC qword +40 / qword +20744 / short +96 | `object_id` / `db_id` / `short_96 as u16` |
| Zone | PC qword +2224 | `Pc::zone` |
| Native PC kind | PC word +46 = 1; checked by the 1530 initializer | `Pc::native_kind` |
| Market dealer | Daemon +4616; send `0x14015CA00` | `Services::market(MarketCheck::Submit(..))` |
| Market reader | Daemon +4768; send `0x14015FC00` | Same boundary |
| Search user value | User 16 bytes +200, sent as `o` | Adapter-owned; not a client value |
| Account transaction manager | Daemon +5080; register `0x1400D4FE0`, activate `0x1400D4A80` | Same boundary |
| PC lookup, if needed by a continuation | Shared daemon +5824 ID/name registry | Existing `world.mentoring.players/names`; these entries do not look up another PC |

A missing user does not send a failure in 378, 381, 384, 1533, 1536, 1539,
1542, 1545 or 1548. Entries 1539, 1545 and 1548 log that condition.
A dealer send can return false. These callers ignore its result.
Reader entries 1551 and 1554 check their send result.

`MemoryMarket` records all checks and submissions. Its configuration values
start at `None`. Its submission result starts at `Gate::Unknown`. Unknown
state returns `Outcome::unsupported("❓ ...")`. A submitted request is not a
backend reply or a completed purchase. Rust retains decoded transaction
inputs for inspection only. It does not infer native transaction fields from them.

## 378 — G_RetrieveSaleListReqCmd — partial

Handler `0x1403995C0`.

Wire: `hQQqc` at `0x140BEC33C`: first u8, second u8, cursor u64, direction i8.

1. Arena form B → log, reply 380 `hEQQqc` at `0x1408B8870`, body `[5:u16, 0:u8, 0:u8, 0:u64, 0:i8]`.
2. Decode. The entry ignores the result. It does not initialize the four fields.
3. Get the user. No user → no reply and no send.
4. Send dealer command 4 `hqqQQqc` at `0x1409BB160`: `[PC ID, DB ID, first, second, cursor, direction]`, route PC short +96. Ignore the send result. No local success reply.

Task: none. No task kind, vtable or fields.

❓ Missing: values after a short unchecked decode; Rust stops at that boundary.
Dealer response and listing state are not read.

## 381 — G_RetrieveBidListReqCmd — partial

Handler `0x14039A220`.

Wire: no decode. The entry ignores the body.

1. Arena form B → log, reply 383 `hh` at `0x1408B8848`, body `[5:u16]`.
2. Get the user. No user → no reply and no send.
3. Send dealer command 7 `hqq` at `0x1408CE234`: `[PC ID, DB ID]`, route PC short +96. Ignore the send result. No local success reply.

Task: none. No task kind, vtable or fields.

❓ Missing: dealer response and bid state.

## 384 — G_RetrieveBidRankStatusReqCmd — partial

Handler `0x14039A460`.

Wire: `hCq;` at `0x140BED548`: unsigned u16 count, then that many u64 sale IDs.

1. Arena form B → log, reply 386 `hh`, body `[5:u16]`.
2. Read the count with `0x14069DA00`. A short count returns −1. Cast to the signed short, then compare as unsigned to 32. Invalid count → log, reply 386 `[2:u16]`.
3. Allocate `8 * count` bytes. Read each ID with `0x14069F4B0`. A failed element read → log, reply 386 `[2:u16]`.
4. Call `0x14015DE90`. A null array → assert log and no reply. Get the user. No user → no reply.
5. Build dealer command 3 `hqqCq;` at `0x1409BB2DC`: `[PC ID, DB ID, count, IDs...]`. A compose failure logs and sends no reply. Ignore the send result. Free the array.

Task: none. No task kind, vtable or fields.

❓ Missing: allocation failure and zero-size allocation details, backend response,
and packet compose failures. Rust supplies the decoded vector to the boundary.

## 1013 — G_NPAuctionBidReqCmd — partial

Handler `0x1403D02E0`. Log tag `[auction-escrow-bid]`.

Wire: `hohq` at `0x140C326A4`: auction ID (16 raw bytes), currency group i16,
bid price i64. Failure descriptor `hE` is at `0x1408B8838`.

1. Arena form B → log, reply 1015 `hE`, body `[5:u16]`.
2. Decode failure → log, reply 1015 `[2:u16]`.
3. Currency group ≤ 0 or price ≤ 0 → log, reply 1015 `[2303:u16]`.
4. Global DWORD +156 = 0 → log, reply 1015 `[2304:u16]`.
5. No zone → log, reply 1015 `[2:u16]`.
6. Allocate task kind 280 (0xC0 bytes). Base vtable `0x140AA2A00`; final vtable `0x140AEBB00`. Initializer requires signed PC short +96 > 0 and PC DB ID ≠ 0. Failure → log, reply 1015 `[2:u16]`. It repeats the currency and price checks.
7. Attach to zone. PC object ID → +128; byte +138 = 1; world i16 → +144; DB ID → +152; auction bytes → +160; currency i16 → +176; price i64 → +184.
8. Queue `0x140254FD0`. Failure → log, reply 1015 `[2:u16]`. Success logs and sends no reply here. The entry does not apply the trial-play gate or a PC-kind gate.

❓ Missing: task body, escrow state and backend response.

## 1530 — G_RegisterSaleReqCmd — partial

Handler `0x140398D80`.

Wire: `hqQqihh` at `0x140BEB9F0`: item u64, sale type u8, price i64,
count i32, first i16, second i16. ❓ Names of the final two shorts are not verified.
Failure descriptor is `hh`.

1. Global byte +220 = 0 → log, reply 1532, body `[1223:u16]`.
2. Arena form B → log, reply 1532 `[5:u16]`.
3. Trial play → log, reply 1532 `[7:u16]`.
4. Decode failure → log, reply 1532 `[1213:u16]`.
5. No zone → log, reply 1532 `[1:u16]`.
6. Action factory `0x140472550`, class 18. Allocation failure → reply 1532 `[1:u16]`. Size 0x208; vtable `0x140D05EB0`; class at +146 = 18. This is an action, not a numbered task kind.
7. Initialize `0x140547320`. PC kind ≠ 1 → assert log, reply 1532 `[1:u16]`. Otherwise attach to zone through `0x140471800`: PC ID +128, zone ID +136, byte +149 = 1. Copy item +176, count +184, sale type +188, price +192, first +152, second +154. Set byte +516 = 0.
8. Queue `0x140254FD0`. Failure → log, reply 1532 `[1:u16]`. Success logs and sends no reply here. This entry does not check count, sale type or price limits.

❓ Missing: action body, item validation, listing creation and backend response.

## 1533 — G_AbandonSaleReqCmd — partial

Handler `0x140399440`; dealer wrapper `0x14015D150`.

Wire: `hq` at `0x1408D5024`: sale ID u64. Failure descriptor is `hh`.

1. Arena form B → log, reply 1535, body `[5:u16]`.
2. Trial play → log, reply 1535 `[7:u16]`.
3. Decode. Ignore the result. The sale ID has no initial value.
4. Wrapper gets the user. No user → no reply.
5. Send dealer command 25 `hqqq` at `0x1408CB7FC`: `[PC ID, sale ID, DB ID]`. Ignore the send result. The wrapper can write a game log. No local success reply.

Task: none. No task kind, vtable or fields.

❓ Missing: short unchecked decode value, game-log writer, dealer response and
listing state.

## 1536 — G_UpdateSalePriceReqCmd — partial

Handler `0x140399110`.

Wire: `hqq` at `0x1408CE234`: sale ID u64, new price i64. Failure descriptor is `hh`.

1. Arena form B → log, reply 1538, body `[5:u16]`.
2. Trial play → log, reply 1538 `[7:u16]`.
3. Set price = 0. Decode and ignore the result. Sale ID has no initial value. A missing price leaves price at 0.
4. Price < 0 or price > signed global qword +8 → log, reply 1538 `[33:u16]`.
5. Get the user. No user → no reply.
6. Send dealer command 22 `hqqqq` at `0x1409BADE4`: `[PC ID, sale ID, DB ID, price]`. Ignore the send result. No local success reply.

Task: none. No task kind, vtable or fields.

❓ Missing: short decode without a complete sale ID, dealer response and listing
state. Rust preserves the initialized price default.

## 1539 — G_RegisterBidReqCmd — partial

Handler `0x1403998A0`.

Wire: `hqq`: sale ID u64, bid price i64. Failure descriptor is `hh`.

1. Arena form B → log, reply 1541, body `[5:u16]`.
2. Trial play → log, reply 1541 `[7:u16]`.
3. Initialize both qwords to 0. Decode and ignore the result. Keep fields read before a short field. Leave later fields at 0.
4. Price < 0 or price > signed global qword +16 → log, reply 1541 `[33:u16]`. Disassembly verifies that the signed test uses the price, not the sale ID.
5. Get the user. No user → log and no reply.
6. Allocate account transaction (0x80 bytes, vtable `0x1409CE540`, tag DWORD +8 = `0xE210A`, type 14). User constructor `0x14002F440`. Qwords +88/+96/+104/+112 = 0; word +120 = 2. Set transaction ID +24 from daemon virtual +5128. The entry does **not** copy the decoded sale ID or price into these fields. Disassembly verifies this detail.
7. Register `0x1400D4FE0`. Failure → log, reply 1541 `[1:u16]`. Success → activate `0x1400D4A80`, log, no local reply.

Zone task: none. The object is an account transaction.

❓ Missing: transaction body, meaning of zero fields, listing/bid state and
backend response. The service receives the decoded values only as input context.
It must not treat them as verified transaction field assignments.

## 1542 — G_AbandonBidReqCmd — partial

Handler `0x14039A090`; dealer wrapper `0x14015D690`.

Wire: `hqq`: sale ID u64, bid ID u64. Failure descriptor is `hh`.

1. Arena form B → log, reply 1544, body `[5:u16]`.
2. Trial play → log, reply 1544 `[7:u16]`.
3. Decode and ignore the result. Neither qword has an initial value.
4. Wrapper gets the user. No user → no reply.
5. Send dealer command 40 `hqqqq`: `[PC ID, sale ID, bid ID, DB ID]`. Ignore the send result. The wrapper can write game logs. No local success reply.

Task: none. No task kind, vtable or fields.

❓ Missing: short unchecked decode values, game-log writers, backend response and
bid state.

## 1545 — G_UpdateBidPriceReqCmd — partial

Handler `0x140399CA0`.

Wire: `hqqq`: sale ID u64, bid ID u64, new price i64. Failure descriptor is `hh`.

1. Arena form B → log, reply 1547, body `[5:u16]`.
2. Trial play → log, reply 1547 `[7:u16]`.
3. Initialize all three qwords to 0. Decode and ignore the result. Preserve the decoded prefix and zero defaults.
4. Get the user. No user → log and no reply. This check is **before** the price check.
5. Price < 0 or price > signed global qword +16 → log, reply 1547 `[33:u16]`.
6. Allocate account transaction (0x78 bytes, vtable `0x1409CE100`, tag DWORD +8 = `0xF210A`, type 15). Qwords +88/+96/+104 = 0; word +112 = 2. User constructor `0x14002F440`; transaction ID at +24. No decoded field is copied into this object before registration. Disassembly verifies this detail.
7. Register failure → log, reply 1547 `[1:u16]`. Success → activate, no local reply.

Zone task: none. The object is an account transaction.

❓ Missing: transaction body, zero-field meanings, bid state and backend response.
Decoded values at the service boundary are input context, not native field mappings.

## 1548 — G_RegisterPurchaseReqCmd — partial

Handler `0x14039A7A0`.

Wire: `hqq`: sale ID u64, price i64. Failure descriptor is `hh`.

1. Arena form B → log, reply 1550, body `[5:u16]`.
2. Trial play → log, reply 1550 `[7:u16]`.
3. Initialize both qwords to 0. Decode and ignore the result. Preserve the decoded prefix and zero defaults.
4. Price < 0 or price > signed global qword +8 → log, reply 1550 `[33:u16]`. Disassembly verifies that the signed test uses the price.
5. Get the user. No user → log and no reply.
6. Allocate account transaction (0x80 bytes, vtable `0x1409CDD00`, tag DWORD +8 = `0xD210A`, type 13). Qwords +88/+96/+104/+112 = 0; word +120 = 2. User constructor `0x14002F440`; transaction ID +24. No decoded field is copied into the object before registration. Disassembly verifies this detail.
7. Register failure → log, reply 1550 `[2:u16]`, not reason 1. Success → activate, no local reply.

Zone task: none. The object is an account transaction.

❓ Missing: transaction body, zero-field meanings, purchase state and backend
response. Decoded values at the service boundary are input context only.

## 1551 — G_MarketSearchItemReqCmd — partial

Handler `0x14039AB50`.

Wire: `hQQQQiqqq` at `0x140BEDBF0`: search type u8, order type u8, price type
u8, sale type u8, item i32, first u64, second u64, third u64. ❓ The meanings of
the three qwords are not verified. Failure descriptor is `hE`.

1. Arena form A → log, reply 1553, body `[5:u16]`. Do not apply PC arena state or trial-play gates.
2. Decode failure → log, reply 1553 `[2:u16]`.
3. Search type not in 1..3 → log, reply 1553 `[1955:u16]`.
4. Order type not in 1..2 → log, reply 1553 `[1955:u16]`.
5. Price type not in 1..2 → log, reply 1553 `[1955:u16]`.
6. Sale type not in 1..2 → log, reply 1553 `[1955:u16]`.
7. Get the user. No user → log, reply 1553 `[2:u16]`.
8. Send reader command 52 `hoqQQQQiqqq` at `0x1409BD010`: `[user bytes +200, DB ID, search type, order type, price type, sale type, item, first, second, third]`. Route PC short +96. Send failure → log, reply 1553 `[2:u16]`. Success sends no reply here.

Task: none. No task kind, vtable or fields.

❓ Missing: adapter user value, search implementation and backend response.

## 1554 — G_RetrieveInstantSaleItemLowestPriceReqCmd — partial

Handler `0x14039AF90`.

Wire: `hi` at `0x140907178`: item i32. Failure descriptor `hEi` is at
`0x1408B88A4`.

1. Arena form A → log, call reply 1556 `hEi` with reason 5.
2. Decode failure → log, call reply 1556 `hEi` with reason 2.
3. Get the user. No user → log, call reply 1556 `hEi` with reason 2.
4. Send reader command 61 `hqi` at `0x1408C7F48`: `[PC ID, item]`, route PC short +96. Send failure → log, call reply 1556 `hEi` with reason 2. Success sends no reply here.
5. Each failure call sets only the reason argument in R9. The call does not set the final i32 argument on the stack. Disassembly verifies the missing assignment. The known body prefix is `[reason:u16]`; ❓ the final four bytes depend on native stack contents. Do not invent a zero or echo the item. Rust returns `Unsupported` at each such reply boundary and sends no malformed reply.

Task: none. No task kind, vtable or fields.

❓ Missing: final i32 failure-body value, price lookup and backend response.

## Rust port status

| Command | Rust | State / boundary |
| --- | --- | --- |
| 378 | `src/handlers/market.rs` | Partial; unchecked short decode and dealer response |
| 381 | `src/handlers/market.rs` | Partial; dealer response |
| 384 | `src/handlers/market.rs` | Partial; array allocation, compose and dealer response |
| 1013 | `src/handlers/market.rs` | Partial; kind 280 queued through `skills::queue`; task body unread |
| 1530 | `src/handlers/market.rs` | Partial; action class 18 queued through `skills::queue`; action body unread |
| 1533, 1536, 1542 | `src/handlers/market.rs` | Partial; unchecked uninitialized fields and dealer response |
| 1539, 1545, 1548 | `src/handlers/market.rs` | Partial; native gates and transaction register result; transaction body unread |
| 1551 | `src/handlers/market.rs` | Partial; reader submission and backend response |
| 1554 | `src/handlers/market.rs` | Partial; reader submission; undefined reply tail stays unsupported |

`Services::market` defaults to `Gate::Unknown`. `MemoryServices::market` holds
the mock configuration, submission result and calls. The two zone objects use
`SkillTaskKind::Market(MarketTask)`. The dispatch retains raw queued requests.
Tests are in `tests/gh1_market.rs`.
