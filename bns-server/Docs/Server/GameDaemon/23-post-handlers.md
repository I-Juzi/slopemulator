# 23 — Post handlers (GameDaemon 381.8867)

Source: `ida/delegate/gamed/GameDaemon.exe.i64` (build 381.8867), read through
the IDA MCP. The addresses agree with `parity-ledger.tsv`, column `handler381`.
Every address below belongs to this build.

Read descriptors with `idc.get_strlit_contents`. Read wide log strings with
`ida_nalt.STRTYPE_C_16`. Check the assembly when the decompiler omits variadic
arguments. Do not infer letter case from an IDA symbol name.

Wire: `h` = header or 2-byte short, `E` = 2-byte reason, `Q` / `c` = 1 byte,
`i` = 4 bytes, `q` = 8 bytes, `s` = a signed 2-byte count and UTF-16 units.
A string count must be less than its destination capacity. `C...;` = a 2-byte
count and repeated records. `b` = a 2-byte byte count and raw bytes.
The Rust dispatcher receives the body only. It supplies the command identity.
The reader permits trailing bytes, as the native entry functions do.

Coverage: 368, 371, 374, 785, 1500, 1503, 1506, 1509.
All eight remain partial. Cached reply handlers and account transaction bodies
are not covered. The anti-bot helper `0x14060E280` was not read or called.

## Shared fields

| Field | Source | Rust |
| --- | --- | --- |
| Arena form B | cfg at daemon +2736: byte +237 and DWORD +248 both nonzero, or PC DWORD +37520 in 2/3/5/6 under CS +37480 | `ctx.world.arena || session::arena_blocked(ctx.pc())` |
| Trial play | PC byte +13768 = 1 | `Pc::t2_blocked` |
| User present | PC qword +20696 is nonzero and PC byte +20752 is zero | `Pc::user_present && !Pc::party_user_blocked` |
| PC object ID | PC qword +40 | `Pc::object_id` |
| PC DB ID | PC qword +20744 | `Pc::db_id` |
| Cached routing value | PC qword +20664 | `Pc::routing` |
| User DB ID for post requests | user qword +216; zero prevents the cached send | `Services::post`, not `Pc::db_id` |
| Cached post channel | daemon +3112, send `0x1406A7400` | `PostOperation::Cached` |
| Account transaction manager | daemon +5080; register `0x1400D4FE0`, deliver register `0x1400D49E0` | `PostOperation::Register` |
| Money limit | `qword_140EE58A0`, signed qword index 5 | `PostOperation::MoneyLimit` |
| Descriptor decode | `0x14069F410` → `0x1406AA7B0`; fields before a failed read remain written | `Reader` and the explicit partial decode in 1506 |
| Repeated count | `0x14069DA00`; reads 2 bytes, returns −1 on a short count | Rust maps a missing count to the native rejection |

These entries do not look up another PC. They do not read the mailbox in the
entry function. Do not add a lookup or a mailbox rule. The shared PC registry
stays in `world.mentoring.players/names` for handlers that need it.

These entries do not create a zone task. They do not call the zone queue.
The Rust port reserves `SkillTaskKind::Post(PostTask)`, as the handler family
interface requires. It does not send an account transaction through
`skills::queue`. That would change the native destination.

## 368 — G_RetrievePostCountReqCmd — partial

Handler `0x140396690`.

Wire: no decode. The request has no fields used by this handler.

1. Arena form B → reply 370 `hh` [5]. Log the arena rejection.
2. No user → log `HandleGame_RetrievePostCountReq, PC(%I64X) has no user`
   (`0x140BE9230`). No reply.
3. User +216 = 0 → log `SendPostPacket failed, pcdbid is invalid`
   (`0x140AF6D20`). No reply and no cached request.
4. Otherwise send cached command 69 `hqq` [PC object ID, user DB ID], with
   routing PC +20664. The send result does not change the entry result.
5. Release the user reference. Return true. No success reply here.

Task: none. No task kind, vtable or task fields. Cached descriptor `hqq` is at
`0x1408CE234`. Failure descriptor `hh` is at `0x1408B8848`.

❓ Missing: cached command 69 processing and client reply 369. The adapter must
supply the cached channel and the user DB ID. It must not invent a mailbox count.

## 371 — G_RetrievePostBriefListReqCmd — partial

Handler `0x140396860`.

Wire: no decode. The request has no fields used by this handler.

1. Arena form B → reply 373 `hh` [5].
2. No user → log the missing user. No reply.
3. User +216 = 0 → log the invalid DB ID. No cached request.
4. Otherwise send cached command 72 `hqq` [PC object ID, user DB ID], with
   routing PC +20664. Ignore the send result.
5. With a user, log `HandleGame_RetrievePostBriefListReq, pc(%I64X)`
   (`0x140BE9830`), even if the user DB ID is zero.
6. Release the user reference. Return true. No success reply here.

Task: none. No task kind, vtable or task fields.

❓ Missing: cached command 72 processing, brief record format and client reply 372.

## 374 — G_RetrievePostReqCmd — partial

Handler `0x140396A50`.

Wire: `hQq` at `0x1408D90EC`: post kind u8, post ID u64.

1. Arena form B → call reply 376 `hEQq` with reason 5. This occurs before
   decoding. The call at `0x140396C1C` sets no arguments for `Qq`.
   ❓ The reply tail comes from unset stack slots. Rust returns `Unsupported`
   and sends no incomplete reply. It does not use zeroes or echo the request.
2. Decode `hQq`. Ignore the decode result. The destination locals have no
   initial values. A short body can leave fields unset.
3. No user → log the missing user. No reply.
4. User +216 = 0 → log the invalid DB ID. No cached request.
5. Otherwise send cached command 75 `hqQqq` [PC object ID, post kind,
   user DB ID, post ID]. Route with PC +20664. Descriptor `0x140AA1ED8`.
   The assembly at `0x140396B88` confirms all four fields.
6. With a user, write the request log (`0x140BE9570`). Release the reference.
   Return true. No success reply here.

Task: none. No task kind, vtable or task fields.

❓ Missing: unset error-tail values, unchecked short-body behavior, cached command
75 processing, post content format and client reply 375. Rust stops before the
cached send if a request field is unset.

## 785 — G_RetrievePcDbIdReqCmd — partial

Handler `0x1403BFC80`.

Wire: no request decode. Reply 786 uses `hb`, not `hq`.

1. Create an output buffer. Write reply command 786 (`0x312`, assembly
   `0x1403BFD30`) with descriptor `hb` at `0x140935448`.
2. Append a 2-byte byte count of 8. Append PC qword +20744 as eight raw bytes
   (`0x1403BFD5C`). Body: `[8:u16, PC DB ID:u64]`, little-endian.
3. Check buffer capacity for the ten appended bytes. Failure enters the native
   crash-assert path. A normal buffer has capacity 4000.
4. Send through PC virtual +584. Release the buffer. Return true.

No arena gate, trial gate, user lookup or zone check. Task: none. No task kind
or task fields. Descriptor object vtable: `0x1407FB520`, not a zone task vtable.

❓ Missing: native buffer allocation failure and send implementation. Rust returns
an inert reply plan. It does not reproduce a process crash or socket write.

## 1500 — G_ArchivePostReqCmd — partial

Handler `0x140396C80`.

Wire: `hQq`: post kind u8, post ID u64.

1. Arena form B → call reply 1502 `hEQq` [5, unset kind, unset ID].
   ❓ The native call does not set the tail. Rust returns `Unsupported`.
2. Trial play → call reply 1502 `hEQq` [7, unset kind, unset ID].
   ❓ This call also does not set the tail. Rust returns `Unsupported`.
3. Decode `hQq`. Ignore the decode result. The locals are not initialized.
4. No user → log the missing user. No reply.
5. User +216 = 0 → log the invalid DB ID. No cached request.
6. Otherwise send cached command 93 `hqQqq` [PC object ID, post kind,
   user DB ID, post ID]. Route with PC +20664. Ignore the send result.
7. With a user, write the archive request log (`0x140BE9B70`). Release the
   reference. Return true. No success reply here.

Task: none. No task kind, vtable or task fields.

❓ Missing: unset reply tails, unchecked short-body behavior, cached command 93
processing, archive state changes and client reply 1501.

## 1503 — G_MarkExpressPostReqCmd — partial

Handler `0x140397C20`.

Wire: `hQq`: post kind u8, post ID u64. Failure descriptor `hEQq` is at
`0x1408B8868`. Every complete failure body is `[reason:u16, kind:u8, ID:u64]`.

1. Decode first. Ignore the decode result. The locals have no initial values.
2. Arena form B → reply 1505 `hEQq` [5, request kind, request ID].
3. Trial play → reply 1505 [7, kind, ID].
4. No user → log `user is not found` (`0x140BEA5B0`). No reply.
5. Allocate account transaction, size 0x70, vtable `0x1409E63D8`.
   Native type tag DWORD +8 = `0xB210A` (729354). Byte +88 = 0xFF.
   Qwords +96/+104 = 0. Base user reference comes from `0x14002F440`.
   Assign +24 from the daemon callback at +5128.
6. Register with `0x1400D4FE0`. Failure → reply 1505 [2, kind, ID], then log
   `cannot start tran` (`0x140BEA690`). Success calls `0x1400D4A80`.
7. Release the transaction and user references. Return true. No success reply here.

Task kind: account transaction, not a zone task. ❓ A separate zone kind is not
present. The entry does **not** copy request kind or ID into +88/+96/+104.
Rust retains the same default transaction fields through `PostTask::MarkExpress`.

❓ Missing: transaction vtable methods, mailbox rules, backend requests and reply
1504. Short requests can leave reply fields unset. Rust returns `Unsupported`
before registration on that path. A missing user still has the known log-only path.

## 1506 — G_DetachPostAttachmentReqCmd — partial

Handler `0x1403977F0`.

Wire: `hQqqq` at `0x140BE9E9C`: kind u8, post ID u64, attachment instance
ID u64 (log says `instid`), final u64 (❓ meaning). The two 128-bit decode
buffers start at zero. Earlier decoded fields survive a later failed read.

1. Decode fails or decoded header is not 1506 → reply 1508 `hEQq`
   [2, kind, post ID]. Use the partial decoded fields or their zero defaults.
   The Rust dispatcher has already selected command 1506 from the header.
2. Arena form B → reply 1508 [5, kind, post ID].
3. Trial play → reply 1508 [7, kind, post ID].
4. No user → log `user is not found` (`0x140BEA940`). No reply.
5. Allocate account transaction, size 0x90, vtable `0x1409E6BE8`.
   Native type tag DWORD +8 = `0xC210A` (794890). Qword +88 = 0.
   Byte +96 = 0xFF. Qwords +104/+112/+120/+128 = 0.
   DWORD +136 = `0x20000`. Assign +24 from daemon callback +5128.
6. Register with `0x1400D4FE0`. Failure → reply 1508 [2, kind, post ID], then
   log `cannot start tran` (`0x140BEAA60`). Success calls `0x1400D4A80`.
7. Release the transaction and user references. Return true. No success reply here.

Task kind: account transaction, not a zone task. The entry does **not** copy the
four request fields into the transaction. Rust uses `PostTask::DetachAttachment`.

❓ Missing: transaction methods, attachment validation, final request qword meaning,
backend requests and client reply 1507. Do not infer attachment ownership here.

## 1509 — G_DeliverPostReqCmd — partial

Handler `0x140396EF0`.

Wire: `hQqhsssqChh;Chh;` at `0x140BE99D8`.
The prefix has kind u8, qword (❓ meaning), short (❓ meaning), receiver string
(capacity 33), subject string (65), content string (1025), signed money i64. ❓ The labels receiver, subject and content are inferred from the string capacities.
Then read attached-item pairs of shorts. Then read charge-item pairs of shorts.
Each list has a 2-byte count. Only each pair's first short takes part in the
entry's duplicate-position check. ❓ The second short's meaning is not proven.

1. Arena form B → reply 1511 `hh` [5].
2. Trial play → reply 1511 [7].
3. No user → log the missing user (`0x140BE9D40`). No reply.
4. Allocate account transaction, size 0x9A8, vtable `0x1409E6810`.
   Native type tag DWORD +8 = `0x3C210A` (3940618).
5. Decode the seven prefix fields, starting after the header. Failure → log
   `invalid packet` (`0x140BE99F0`). No reply.
6. Read attached count as a signed short, then compare as unsigned. Count not
   0..3, including a missing count (decoder −1), → reply 1511 [1287].
7. Decode that many `hh` pairs. Failure → log `invalid packet`. No reply.
8. Duplicate attached position → reply 1511 [1292]. This check comes before
   reading the charge count. Log source `0x140BEA100`.
9. Read charge count. Count not 0..6, including a missing count, → reply 1511
   [1292]. Log source `0x140BEA210`.
10. Decode that many `hh` pairs. Failure → log `invalid packet`. No reply.
11. Duplicate charge position → reply 1511 [1292]. Log source `0x140BEA2F0`.
    The entry does not compare positions between the two lists.
12. Money < 0 or money > signed global limit → reply 1511 [33]. Log source
    `0x140BEA400`. A negative amount does not read the global limit.
13. Register with `0x1400D49E0`. Failure → reply 1511 [2]. Success sends no
    reply here. Release the transaction and user references. Return true.

| Transaction field | Offset | Rust |
| --- | --- | --- |
| Kind | byte +88 | `PostDelivery::kind` |
| Qword, ❓ meaning | +96 | `reference` |
| Short, ❓ meaning | +104 | `value` |
| Receiver / subject / content | UTF-16 +106 / +172 / +302 | `receiver` / `subject` / `content` |
| Money | signed qword +2352 | `money` |
| Attached count, positions, second shorts | DWORD +2360, short arrays +2364 / +2370 | `attached` pairs |
| Charge count, positions, second shorts | DWORD +2376, short arrays +2380 / +2392 | `charged` pairs |
| Initial continuation state | qword +2408 = 0; word +2416 = 0x100; qwords +2424/+2432/+2440 = 0; DWORD +2448 = 0; DWORD +2456 = 0; qword +2464 = 0 | Adapter-owned defaults |

Task kind: account transaction, not a zone task. Rust uses
`PostTask::Deliver(Box<PostDelivery>)`. It retains both shorts in every pair.
It does not complete delivery when the adapter accepts registration.

❓ Missing: receiver rules, subject/content filtering, inventory rules, attachment
ownership, fees, backend requests, transaction methods and client reply 1510.
The entry has no such checks. Do not add speculative checks to this port.

## Rust port status

| Command | Rust | State and boundary |
| --- | --- | --- |
| 368, 371 | `src/handlers/post.rs` | Partial. Arena reply and user gate ported. Cached channel and user DB ID use `Services::post`. |
| 374, 1500 | `src/handlers/post.rs` | Partial. Gate order and valid cached requests ported. Unset failure tails and short request fields return `Unsupported`. |
| 785 | `src/handlers/post.rs` | Partial. Exact reply 786 counted blob ported. Native allocation and send are boundaries. |
| 1503 | `src/handlers/post.rs` | Partial. Complete-request replies 5/7/2 and transaction registration ported. Unset short-request fields return `Unsupported`. |
| 1506 | `src/handlers/post.rs` | Partial. Partial decode, replies 2/5/7 and registration ported. Transaction execution is a boundary. |
| 1509 | `src/handlers/post.rs` | Partial. Decode, count order, duplicate checks, money checks and replies 5/7/1287/1292/33/2 ported. Transaction execution is a boundary. |

`Services::post` defaults to `PostResult::Unknown`. `MemoryServices::post` holds
`MemoryPost`, with independent cached/registration results, an optional money
limit and recorded operations. An unknown result does not send a packet and
does not approve a transaction. No mailbox, attachment or backend reply is
fabricated. Tests are in `tests/gh1_post.rs`.
