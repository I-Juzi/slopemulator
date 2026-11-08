# 25 — Arena lobby handlers (GameDaemon 381.8867)

Source: `ida/delegate/gamed/GameDaemon.exe.i64`, build 381.8867. Read through
IDA MCP. Handler table `0x1407D9F80`. Addresses below belong to this build.
Descriptor strings were read with `idc.get_strlit_contents`. Wide log strings
were read with `ida_nalt.STRTYPE_C_16`. Do not infer widths from IDA symbol names.

`h` is the command header or a 2-byte field. `E` is a 2-byte reason.
`Q` and `c` are 1 byte. `i` is 4 bytes. `q` is 8 bytes.
`s` is counted UTF-16. `b` is a counted byte block.
`C...;` has a 2-byte count (`0x14069DA00`), then the repeated fields.
Rust receives body bytes only. It permits trailing bytes, as in doc 19.

All 23 entry functions were read. They remain partial because task bodies,
server-owned tables, packet builders, and backend completions are boundaries.
No listed target is the shared `0x14001AFE0` stub.

## Shared fields

| Field | Source | Rust |
| --- | --- | --- |
| Arena feature enabled | daemon cfg +2736 byte +237 | `World::party_config.arena_enabled` |
| Arena server mode | cfg +2736 DWORD +248, after feature check | `World::arena` |
| Arena form B, where used elsewhere | server mode OR PC state 2/3/5/6 | Existing `session::arena_blocked`; these entries check server mode only |
| PC arena state | PC DWORD +37520, lock +37480 | Existing `Pc::arena_state` |
| Arena record/key | PC qword +37528 | Existing `Pc::arena_qwords[0]` |
| Arena context key | PC qword +37544; cleared with +37528 by 648 | Existing `Pc::arena_qwords[1]` |
| Lobby route | PC qword +37536 | `Pc::arena_lobby.route` |
| Lobby kind | PC byte +37552 | `Pc::arena_lobby.kind` |
| Current attraction | PC byte +38485 | `Pc::arena_lobby.attraction` |
| Waiting ID | PC qword +37608 | `Pc::arena_lobby.waiting_id` |
| Last waiting-number tick | PC qword +37624 | `Pc::arena_lobby.waiting_tick` |
| User and fatigue state | PC +20696, byte +20752, user DWORD +120 | Existing `user_present`, `party_user_blocked`, `party_user_state` |
| Trial play | PC byte +13768 | Existing `Pc::t2_blocked`; these entries do not test it |
| PC ID, DB ID, world, name | PC +40, +20744, signed short +96, +194 | Existing `object_id`, `db_id`, `short_96`, `name` |
| Local world list | daemon shorts +86748; count +86848 | `ArenaLobbyOperation::LocalWorld` |
| PC name lookup, rights 6 | `0x1403F7EF0`, daemon +5824 | Existing `world.mentoring.players/names` |
| Cross-option enabled | global config byte +819 | `ArenaLobbyOperation::CrossOptionEnabled` |
| Game time valid | `0x1406117A0`: user +712 <= 0 OR expiry − now > 60 × global DWORD index 494; missing user fails | `ArenaLobbyOperation::GameTimeValid` |
| Special matching enabled | rule cfg +2744 byte +652 and global bytes +800/+801/+802 for attractions 4/5/6 | `ArenaLobbyOperation::SpecialMatchEnabled` |

### Shared boundaries

`Services::arena_lobby` defaults to `Unknown`. `MemoryServices::arena_lobby`
records calls and supplies explicit test results. Unknown never means approval.

Backend sends use `0x1400D2EE0`, daemon +3840, routed by signed PC short +96.
For the usual `hqqqh...` body, the prefix after the command is
[lobby route, PC DB ID, PC object ID, signed PC short +96]. The operation retains
these fields and the decoded suffix. Command 89 has a different prefix order.
A successful send is not a completed lobby operation. Rust returns Unsupported
at that completion boundary. Failed sends use the exact entry failure reply.
There is no zone task on these backend-send paths.

`0x140605F20(pc, reason)` sends 641 `hE` [reason]. Its builder was read.

`0x140606B50` sends backend 36 `hqqqhsQ`, leave reason 3, only when state = 3
and route != 0. It returns false if the route is missing or the send fails.
It does not require +37528. Rust uses this helper plan for the fatigue paths.

The failure call in 675 uses `hEq` (`0x1408B8898`). It does not explicitly set
the fifth variadic argument. Disassembly at `0x1403BA521` reads an outgoing
stack slot left by earlier calls. ❓ The stable value of the trailing qword is
not a verified contract. Rust requests an explicit 8-byte `ReplyTail` result.
It does not emit a short `hE` body or invent a zero qword.

## 639 — G_ReserveArenaLobbyReqCmd — partial

Handler `0x1403B77F0`. Wire `hq` (`0x1408D5024`): portal key u64.

1. Arena feature disabled → 641 `hE` [1691].
2. Arena server mode → 641 [5].
3. Decode failure → 641 [2].
4. No zone → 641 [2].
5. Missing/blocked user, or user state != 0 → 641 [1712].
6. `0x140044850(manager +4136, DB ID)` true → log and stop. No reply.
7. `0x140044D80` returns a session record → send backend 493 `hQqqs`
   [2, DB ID, record qword +120, PC name] to record DWORD +136, via `0x1400D0FC0`.
   This send precedes the portal lookup. The entry does not reject its result.
8. DataManager slot 87 virtual +168 lookup by portal key fails → 641 [1687].
9. Task kind 112, 0xA0 bytes, vtable `0x140B9E890`, constructor `0x14036D670`.
   Qword +144 = portal record pointer; DWORD +152 = 0. Attach to PC and zone.
10. Allocation/queue failure → 641 [2]. Success sends no reply here.

❓ Missing: manager state, optional session-timer send, portal records, and task body.
Rust exposes ordered pending/timer/portal service calls. The task carries the
verified portal key. The adapter must resolve the record pointer; it must not
use the key as a pointer.

## 642 — G_EnterArenaLobbyReqCmd — partial

Handler `0x1403B8360`. Wire `hb` (`0x140935448`): byte block, capacity 150.

1. Arena feature disabled → 644 `hh` [1723].
2. Arena server mode → 644 [5].
3. Decode failure → **641** `hE` [2], not 644.
4. Call `0x140606280(pc, bytes, length)`. Its initial state check requires
   state = 2 and +37528 != 0. Failure returns 1724 → 644 [1724].
5. The helper snapshots lobby fields, user data, inventory, and skill state.
   It builds backend enter 27 or return-enter 30. Send failure returns 1723.
   Other nonzero helper results → 644 [result]. Zero sends no reply here.
6. Only helper result 1723 → first send 644 [1723], then set arena state = 4;
   zero bytes +37528..+37559 and +37584..+37663; then send 646 `hQ` [3].
   The feature-disabled 1723 path does not do this reset.

No zone task. ❓ Missing: exact enter/return-enter packet builders, side effects
in `0x140606280`, and backend replies. Rust retains the byte block at this boundary.
It clears all modeled fields in the native reset ranges. Attraction +38485 stays unchanged.

## 645 — G_LeaveArenaLobbyReqCmd — partial

Handler `0x1403B8620`. Wire: no decode.

1. Feature disabled → 647 `hh` [1725].
2. Arena server mode → 647 [5].
3. State != 3 → 647 [1726]. This check does not require +37528.
4. Backend 36 `hqqqhsQ` [route, DB ID, PC ID, world, name, 1].
   Send failure → 647 [1725]. Success sends no reply here.

No task. ❓ Missing: backend completion.

## 651 — G_ExitFromArenaLobbyRoomReqCmd — partial

Handler `0x1403B8840`. Wire: no decode.

1. Feature disabled → 653 `hh` [1731].
2. Arena server mode → 653 [5].
3. State != 3 → 653 [1732].
4. +37528 = 0 → 653 [1733].
5. Backend 36 `hqqqhsQ` [route, DB ID, PC ID, world, name, 2].
   Send failure → 653 [1731]. Success sends no reply here.

No task. ❓ Missing: backend completion.

## 654 — G_SetArenaLobbyRoomMatchOptionReqCmd — partial

Handler `0x1403B8A40`. Wire `hcccQCh;` (`0x140C149D0`): three bytes,
one u8, then counted i16 map options.

1. Feature disabled → 656 `hh` [1736].
2. Arena server mode → 656 [5].
3. First four fields fail to decode → 656 [2].
4. Global byte +819 = 0 → force the third byte to 0.
5. Read the signed 2-byte map count with `0x14069DA00`.
   Missing count or count outside 0..3 → 656 [1742].
6. Map read failure → 656 [1742]. The entry attempts to omit duplicates.
7. Call `0x140606CE0`. State != 3 → 656 [1737]; +37528 = 0 → 656 [1738].
8. The helper validates attraction with `0x140042650` (false → 1740).
   Invalid fourth-byte selector → 1747. Selectors 2/4/7 require lobby kind 2
   and attraction 4..6 (else 1747). Selectors 5/6 require kind 1 (else 1747).
9. For special attractions, disabled rule +652 → 2; map selection disabled
   (+800/+801/+802) with maps → 1740; more than one map → 1742.
   Map outside 1..299 → 1742. Missing map record (slot 1419), missing referenced
   record, invalid record category, or disabled feature → 2.
10. Kind 1 cannot select maps (2). Attraction 2 requires rule +881 (2).
    Selector 6 requires global +896 (2). Other table paths remain at the helper boundary.
11. The helper builds backend 40 (`hiqqqhcccQCh;`, descriptor read from its builder).
    Send failure → 656 [1736]. Other nonzero helper result → 656 [result].
    Zero sends no reply here.

No task. ❓ Missing: the precise duplicate-map behavior, the full record rules,
the builder fields, and backend completion. Rust retains the raw map list;
it stops at the helper boundary before it must reproduce duplicate handling.

## 657 — G_DelegateArenaLobbyRoomTeamLeaderReqCmd — partial

Handler `0x1403B8E90`. Wire `hhq` (`0x14093B660`): world i16, target u64.

1. Feature disabled → 659 `hh` [1748].
2. Arena server mode → **656** `hh` [5], not 659.
3. Decode failure → 659 [2].
4. State != 3 → 659 [1749].
5. +37528 = 0 → 659 [1750].
6. Backend 44 `hqqqhqh`: usual prefix, then [target, request world].
   Send failure → 659 [1748]. Success sends no reply here.

No task. ❓ Missing: backend completion.

## 660 — G_SetArenaLobbyRoomPasswordReqCmd — partial

Handler `0x1403B9140`. Wire `hh` (`0x1408B8848`): password i16.

1. Feature disabled → 662 `hh` [1755].
2. Arena server mode → 662 [5].
3. Decode failure → 662 [2].
4. State != 3 → 662 [1756].
5. +37528 = 0 → 662 [1757].
6. Backend 48 `hqqqhh`: usual prefix, then password.
   Send failure → 662 [1755]. Success sends no reply here.

No task. ❓ Missing: backend completion.

## 663 — G_InviteArenaLobbyRoomReqCmd — partial

Handler `0x1403B93C0`. Wire `hhsb` (`0x140C14EA4`): world i16,
target name (capacity 33), byte block (capacity 512).

1. Feature disabled → 665 `hh` [1759].
2. Arena server mode → 665 [5].
3. Decode failure → 665 [2].
4. If request world > 0 and it is in the daemon local-world list,
   look up the name with rights 6. No target → 665 [1760].
5. Local target → send target 769 `hb` [request block]. Then send requester
   664 `hQ` [target category from `0x140608A60`]: state 1 → 1;
   states 2/3/4/5 → 2; state 6 → 3; other → 0. No local zone check.
6. Other world → `0x1400D3320` with request world, sender world/name,
   target name, sender PC ID, and a message wrapper containing the block.
   Failure → 665 [1759]. Success sends no reply here.

No task. ❓ Missing: local delivery adapter, cross-world wrapper, and backend completion.
Rust keeps the ordered local send and requester reply behind one service call.

## 666 — G_BroadcastArenaLobbyRoomUnratedInvitationReqCmd — partial

Handler `0x1403B9840`. Wire `hcqsb` (`0x140C15258`): attraction i8,
room u64, name (capacity 33), byte block (capacity 512).

1. Feature disabled → 668 `hh` [1759].
2. Arena server mode → 668 [5].
3. Decode failure → 668 [2].
4. Attraction 0 → backend category 1. Attraction 1 or 2 → category 2.
   All other values → 668 [2].
5. Backend 18 `hQQqqqqsb` (`0x140C159D8`):
   [category, 1, route, DB ID, PC ID, request room, name, block].
   No arena-state or +37528 check precedes this send.
6. Send failure → 668 [1759]. Success sends no reply here.

No task. ❓ Missing: backend completion.

## 669 — G_JoinArenaLobbyRoomReqCmd — partial

Handler `0x1403B9B30`. Wire `hqshcQ` (`0x140C15780`): room u64,
name (capacity 33), password i16, cross byte, u8 option.

1. Feature disabled → 671 `hh` [1765].
2. Arena server mode → 671 [5].
3. Decode failure → 671 [2].
4. Global +819 = 0 → force cross byte to 0.
5. Capture state error: state != 3 → 1766; else +37528 = 0 → 1767.
6. Call game-time check `0x1406117A0` even if step 5 failed.
   False → 671 [1780]. This result overrides the captured state error.
7. If time is valid, captured state error → 671 [error].
8. Backend 52 `hqqqhqshcQ`: usual prefix, then
   [room, name, password, effective cross byte, option 0/1/2 unchanged, other→0].
   Send failure → 671 [1765]. Success sends no reply here.

No task. ❓ Missing: server-owned time/config state and backend completion.
Disassembly confirms the full room qword is forwarded; an IDA int temporary is misleading.

## 672 — G_BanishFromArenaLobbyReqCmd — partial

Handler `0x1403B9EE0`. Wire `hhq` (`0x14093B660`): world i16, target u64.

1. Feature disabled → 674 `hh` [1781].
2. Arena server mode → 674 [5].
3. Decode failure → 674 [2].
4. Request world <= 0 or target = 0 → 674 [2].
5. State != 3 → 674 [1782].
6. +37528 = 0 → 674 [1783].
7. Backend 55 `hqqqhqh`: usual prefix, then [target, request world].
   Send failure → 674 [1781]. Success sends no reply here.

No task. ❓ Missing: backend completion. Disassembly confirms a full target qword.

## 675 — G_ReadyToMatchArenaLobbyRoomReqCmd — partial

Handler `0x1403BA1F0`. Wire: no decode. All failure replies use 677 `hEq`
[reason, ❓ trailing qword; see shared boundary].

1. Feature disabled → reason 1788.
2. Arena server mode → reason 5.
3. Missing/blocked user or user state != 0 → call `0x140606B50`.
   Leave send succeeds → reason 1793. Otherwise → reason 2.
4. State != 3 → reason 1789.
5. +37528 = 0 → reason 1790.
6. Game-time check false → reason 1799.
7. Backend 58 `hqqqhq`: usual prefix, then guild ID (PC +336).
   Send failure → reason 1788. Success sends no reply here.

No task. ❓ Missing: failure qword, game-time state, and backend completion.

## 678 — G_UnReadyToMatchArenaLobbyRoomReqCmd — partial

Handler `0x1403BA560`. Wire: no decode.

1. Feature disabled → 680 `hh` [1805].
2. Arena server mode → 680 [5].
3. State != 3 → 680 [1806].
4. +37528 = 0 → 680 [1807].
5. Backend 62 `hqqqh`: usual prefix only.
   Send failure → 680 [1805]. Success sends no reply here.

No task. ❓ Missing: backend completion.

## 681 — G_MatchArenaLobbyRoomReqCmd — partial

Handler `0x1403BA740`. Wire `hc` (`0x1408D9508`): apply/recruit flag i8.

1. Feature disabled → 683 `hh` [1810].
2. Arena server mode → 683 [5].
3. Decode failure → 683 [2].
4. Missing/blocked user or user state != 0 → call `0x140606B50`.
   Leave send succeeds → 683 [1820]. Otherwise → 683 [2].
5. State != 3 → 683 [1811].
6. +37528 = 0 → 683 [1812].
7. Lobby kind = 2 and attraction 4..6 → require rule +652 and the corresponding
   global +800/+801/+802. Failure → 683 [1816]. Force the flag to 0 on this path.
8. Backend 66 `hqqqhcq`: usual prefix, then [effective flag, guild ID].
   Send failure → 683 [1810]. Success logs and sends no reply here.

No task. ❓ Missing: server-owned rule state and backend completion.

## 684 — G_SkipArenaLobbyMatchReqCmd — partial

Handler `0x1403BABC0`. Wire `h` (`0x1408CFBE8`): empty body.

1. Feature disabled → 686 `hh` [1834].
2. Arena server mode → 686 [5].
3. Native header decode failure → 686 [2]. The body-only Rust interface has
   no header failure input. It accepts an empty body.
4. Missing/blocked user or user state != 0 → call `0x140606B50`.
   Leave send succeeds → 686 [1845]. Otherwise → 686 [2].
5. State != 3 → 686 [1835].
6. +37528 = 0 → 686 [1836].
7. Backend 74 `hqqqh`: usual prefix only.
   Send failure → 686 [1834]. Success sends no reply here.

No task. ❓ Missing: framing failure input and backend completion.

## 687 — G_CancelArenaLobbyMatchingReqCmd — partial

Handler `0x1403BAF50`. Wire `hc` (`0x1408D9508`): cancel flag i8.

1. Feature disabled → 689 `hh` [1849].
2. Arena server mode → 689 [5].
3. Decode failure → 689 [2].
4. State != 3 → 689 [1850].
5. +37528 = 0 → 689 [1851].
6. Backend 78 `hqqqhc`: usual prefix, then cancel flag.
   Send failure → 689 [1849]. Success sends no reply here.

No task. ❓ Missing: backend completion.

## 690 — G_CancelArenaLobbyMatchReqCmd — partial

Handler `0x1403BB1D0`. Wire: no decode.

1. Feature disabled → 692 `hh` [1854].
2. Arena server mode → 692 [5].
3. State != 3 → 692 [1855].
4. +37528 = 0 → 692 [1856].
5. Backend 82 `hqqqh`: usual prefix only.
   Send failure → 692 [1854]. Success sends no reply here.

No task. ❓ Missing: backend completion.

## 693 — G_ReadyToGoToArenaReqCmd — partial

Handler `0x1403BB3B0`. Wire `hs` (`0x1409077A8`): name (capacity 33).

1. Feature disabled → 695 `hh` [1859].
2. Arena server mode → 695 [5].
3. Decode failure → 695 [2].
4. State != 3 → 695 [1860].
5. +37528 = 0 → 695 [1862].
6. Backend 85 `hqqqhs`: usual prefix, then name.
   Send failure → 695 [1859]. Success sends no reply here.

No task. ❓ Missing: backend completion.

## 696 — G_DecideToGoToArenaReqCmd — partial

Handler `0x1403BB670`. Wire **`hQ`** (`0x1408E0DA4`): one decision byte.
The symbol `aHq` is not a reliable width description.

1. Feature disabled → 698 `hh` [1863].
2. Arena server mode → 698 [5].
3. Decode failure → 698 [2].
4. State != 3 → 698 [1864].
5. +37528 = 0 → 698 [1866].
6. Backend 89 `hqqqhQ` (`0x140DAF4CC`):
   **[PC ID, DB ID, route, world, decision byte]**, not the usual prefix order.
   Send failure → 698 [1863]. Success sends no reply here.

No task. ❓ Missing: backend completion. The native temporary is an uninitialized
DWORD, but only its decoded low byte is serialized. Rust retains that byte only.

## 699 — G_RetrieveArenaLobbyWaitingNumberReqCmd — partial

Handler `0x1403BBAB0`. Wire: no decode. No reply on any entry path.

1. Feature disabled → log and stop.
2. Arena server mode → log and stop.
3. Read server tick (`GetTickCount64` minus server start).
4. State != 5 → log and stop.
5. Last tick != 0 and signed(now − last) < 30000 → log and stop.
6. Snapshot route and waiting ID. Set +37624 = now **before** the send check.
7. If both IDs != 0, backend 94 `hqqqhq`: usual prefix, then waiting ID.
   The entry ignores the send result. Zero IDs still retain the new tick.

No task. ❓ Missing: backend completion. Rust uses wrapping subtraction then a
signed comparison. Unknown send state preserves the tick update.

## 702 — G_CancelArenaLobbyWaitingReqCmd — partial

Handler `0x1403BB8F0`. Wire: no decode.

1. Feature disabled → log and stop. No reply.
2. Arena server mode → log and stop. No reply.
3. State != 5 → 704 `hh` [1880].
4. +37528 = 0 → 704 [1882].
5. Backend 97 `hqqqhq`: usual prefix, then waiting ID.
   Send failure → 704 [1879]. Success sends no reply here.

No task. ❓ Missing: backend completion.

## 761 — G_ReserveArenaLobbyPartyAgreementReqCmd — partial

Handler `0x1403B7D50`. Wire `hc` (`0x1408D9508`): agreement byte.

1. Feature disabled → 641 `hE` [1691].
2. Arena server mode → 641 [5].
3. Decode failure → 641 [2].
4. No zone → 641 [2].
5. Missing/blocked user or user state != 0 → 641 [1712].
6. Task kind 41, 0x98 bytes, vtable `0x140B95C00`:
   normalized boolean (wire byte != 0) at +144. Attach to PC and zone; byte +138 = 1.
7. Queue failure → 641 [2]. Success sends no reply here.

❓ Missing: task body. The entry does not test trial play or PC native kind.

## 763 — G_ReserveArenaLobbyPartyFinalDecisionReqCmd — partial

Handler `0x1403B80C0`. Wire `hc` (`0x1408D9508`): decision byte.

1. Feature disabled → 641 `hE` [1691].
2. Arena server mode → 641 [5].
3. Decode failure → 641 [2].
4. No zone → 641 [2].
5. Task kind 41, 0x98 bytes, vtable `0x140C3E5C0`:
   normalized boolean (wire byte != 0) at +144. Attach to PC and zone; byte +138 = 1.
6. Queue failure → 641 [2]. Success sends no reply here.

❓ Missing: task body. Unlike 761, this entry has no user/fatigue check.

## Rust port status

| Command | Rust | State / boundary |
| --- | --- | --- |
| 639 | `src/handlers/arena_lobby.rs` | Partial: ordered manager calls, portal lookup, task 112 queue |
| 642 | Same | Partial: decode, state gate, helper result, reset and 646 |
| 645, 651, 657, 660, 666, 672, 678, 687, 690, 693, 696, 702 | Same | Partial: entry gates and typed backend send plans |
| 654 | Same | Partial: fixed decode, config mask, list checks, helper boundary |
| 663 | Same | Partial: local-world gate, name lookup, local/cross-world delivery boundaries |
| 669 | Same | Partial: time check overrides captured state error |
| 675 | Same | Partial: fatigue/time/send gates; explicit 677 tail boundary |
| 681, 684 | Same | Partial: fatigue leave-send order, special matching gates |
| 699 | Same | Partial: signed rate gate, timestamp update, backend send boundary |
| 761, 763 | Same | Partial: distinct task vtables through `skills::queue` |

All IDs have a dispatcher entry and a parity tag. No entry is marked done.
Task bodies and backend replies are not implemented. No client-datafile fact
was found in this handler read. The anti-bot helper `0x14060E280` was not read.
