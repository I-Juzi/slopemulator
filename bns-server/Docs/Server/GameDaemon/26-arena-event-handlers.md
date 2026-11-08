# 26 — Arena event handlers (GameDaemon 381.8867)

Source: `ida/delegate/gamed/GameDaemon.exe.i64`, build 381.8867. Read through
IDA MCP. Handler table `0x1407D9F80`. Addresses below belong to this build.
Wire notation is the same as in doc 25. Rust receives body bytes only and
permits trailing bytes.

This doc covers 22 entry functions: arena entry and return (705–720, 955),
the GM arena-event commands (723–757) and the arena dungeon recruit pair
(959, 963). They stay partial because task bodies, portal records, geo zones,
pending-entry lists and backend sends are boundaries.

Rust ports 20 of them. 705 and 955 have a spec only. ❓

All failure replies are `hE` (2-byte reason). An empty success reply uses
descriptor `h` (`asc_1408CFBE8`).

## Shared fields

| Field | Source | Rust |
| --- | --- | --- |
| Arena feature enabled | daemon cfg +2736 byte +237 | `World::party_config.arena_enabled` |
| Arena server mode | cfg +2736 DWORD +248 | `World::arena` |
| "In arena" test | +237 ≠ 0 and +248 ≠ 0 | `in_arena()` |
| GM grade | PC byte +532; `CheckArenaEventPacketUsable` passes for 1..=7 | `Pc::gm_grade`, `authorized()` |
| Daemon attraction | daemon byte +4152; 1 and 2 are tested ❓ names | `ArenaEventWorld::attraction` |
| Recruit enabled | `qword_140EE58A0` byte +792 | `ArenaEventWorld::recruit_enabled` |
| PC arena state / record | PC DWORD +37520, qword +37528 | `Pc::arena_state`, `Pc::arena_qwords[0]` |
| Lobby route | PC qword +37536 | `Pc::arena_lobby.route` |
| PC name lookup, rights 6 | `0x1403F7EF0` | `world.mentoring.by_name(&name, 6)` |
| Portal record | DataManager lookup by portal key | `ArenaEventOperation::PortalExists` |
| Geo zone | zone geo lookup for 729 | `ArenaEventOperation::GeoZone` |
| Portal type | portal record word +2: 0 duel, 1 and 2 dungeon ❓ names | `ArenaEventOperation::PortalType` |
| Cross option enabled | global byte +819 (`qword_140EE58A0`) | `ArenaEventOperation::CrossOptionEnabled` |
| Guild ID | PC qword +336 | Existing `Pc::guild_id` |
| Backend send | daemon +3840, `0x1400D2EE0`, to PC world (short +96) | `ArenaEventOperation::Send` |

Unknown service results never approve. Rust returns unsupported for them.

## 705 — G_EnterArenaReqCmd — partial

Handler `0x1403BBCB0`. Session-level entry: `a1` is the connection, not a PC.
Wire `hqhq` (key A, world, key B).

1. Decode failure → log only. No reply.
2. Lock the pending-entry list at daemon +2760 (entries at +112, count +124,
   40 bytes each). Find the entry with qword +0 = key A, word +32 = world and
   qword +24 = key B. Remove it (swap with the last entry). Take the PC at +16.
3. No entry → 707 `hh` [1887].
4. The PC has no user (+720 null or byte +1017 set) → 707 [2].
5. Attach the PC to the connection (vtable +32), then `0x1405FAD30`.
6. `0x1403F4E20` (add to name table, daemon +5824) fails → 707 [1].
7. On success: an anti-cheat probe and a login-history write run. Then
   `0x1405F7150(pc)` enters the world. No reply here.

Rust: not ported. ❓ The pending-entry list and connection attach need a
session adapter. The anti-cheat probe is out of scope.

## 708 — G_ReturnToArenaLobbyFromDuelReqCmd — partial

Handler `0x1403BC400`. Wire `hq` (portal key).

1. Arena feature disabled → 710 [1889].
2. Not arena server mode → 710 [1890].
3. Daemon attraction ≠ 1 → 710 [2].
4. Decode failure → 710 [2].
5. No portal record, or portal type ≠ 0 → 710 [1888].
6. No zone → 710 [2].
7. Task kind 125, 0x98 bytes, vtable `0x140BA9F00`. Portal record at +144.
8. Queue failure → 710 [2]. Success sends no reply here.

## 711 — G_ReturnToArenaLobbyFromDungeonReqCmd — partial

Handler `0x1403BC710`. Wire `hqc` (portal key, flag).

1. Arena feature disabled → 713 [1889].
2. Not arena server mode → 713 [1890].
3. Daemon attraction ≠ 2 → 713 [2].
4. Decode failure → 713 [2].
5. No portal record, or portal type not 1 or 2 → 713 [1888].
6. No zone → 713 [2].
7. Task kind 126, 0xA0 bytes, vtable `0x140C36C10`. Portal record at +144.
   Byte +152 = flag ≠ 0.
8. Queue failure → 713 [2]. Success sends no reply here.

## 714 — G_ArenaSpectateGameReqCmd — partial

Handler `0x1403BCA50`. Wire `hqs` (portal key, name cap 33).
All replies are 716 `hEs` [reason, name]. Before decode, the name is empty.
This entry has no GM check and no zone check.

1. Decode failure → 716 [2, empty].
2. No portal record → 716 [2].
3. Cross option disabled → 716 [2].
4. Arena feature disabled → 716 [1895].
5. Arena server mode → 716 [5].
6. PC arena state ≠ 3 → 716 [1896].
7. Portal type 0 → attraction 1. Type 1 or 2 → attraction 2. Other → 716 [2].
8. Send backend 127 `hqqqhQs` [route, DB ID, PC ID, world, attraction, name].
   Arguments were read from the disassembly at `0x1403BCCFD`.
9. Send failure → 716 [1895]. Success sends no reply here.

## 717 — G_ArenaSpectateReturnToArenaLobbyReqCmd — partial

Handler `0x1403BDFD0`. Wire `hq` (portal key).

1. Arena feature disabled → 719 [1907].
2. Not arena server mode → 719 [1908].
3. Decode failure → 719 [2].
4. No portal record → 719 [1906].
5. No zone → 719 [2].
6. Task kind 263, 0x98 bytes, vtable `0x140BAB4E0`. Portal record at +144.
7. Queue failure → 719 [2]. Success sends no reply here.

## 720 — G_ListArenaSpectateGameReqCmd — partial

Handler `0x1403BE2A0`. Wire `hqQc` (portal key, list type, page). Replies `hE`.

1. Decode failure → 722 [2].
2. List type > 1 → 722 [2].
3. Arena feature disabled → 722 [1911].
4. Arena server mode, or PC arena state ≠ 3 → 722 [1912].
5. No user → 722 [2].
6. User slot `+200 → +16 + 8 × list type` plus global DWORD index 207 seconds
   is later than now → 722 [1915]. Otherwise the slot is set to now.
7. No portal record, or portal type ≠ 0 → 722 [1913].
8. Page ≠ 1 → 722 [1914].
9. Send backend 133 `hqqqqQcc` [PC ID, DB ID, route, guild ID, list type, 1, 1 ❓].
   The last byte comes from a register. It is 1 on this path ❓.
10. Send failure → 722 [1911]. Success sends no reply here.

## 723 — G_ArenaEventTeleportToReqCmd — partial

Handler `0x1403BCE20`. Wire `hs` (name, cap 33).

1. Not authorized → 725 [2].
2. Decode failure → 725 [2].
3. Name lookup fails → 725 [1916].
4. No zone → 725 [2].
5. Task kind 158, vtable `0x140BAC8D8`. Target PC ID at +144.
6. Queue failure → 725 [2]. Success sends no reply here.

## 726 — G_ArenaEventRecallReqCmd — partial

Handler `0x1403BD440`. Wire `hs` (name, cap 33).

1. Not in arena → 728 [2].
2. Not authorized → 728 [2].
3. Decode failure → 728 [2].
4. Name lookup fails → 728 [1917].
5. Target zone differs from caller zone → 728 [1918].
6. Task 0xA8 bytes, base `0x14028DF50` type 15, vtable `0x140AE66D8`,
   init `0x140279630(task, 0, zone, pc, target)`.
7. Queue failure → 728 [2]. Success → 727 (empty).

## 729 — G_ArenaEventWarpToPosReqCmd — partial

Handler `0x1403BD140`. Wire `hhhh` ❓ (x, y, z as 2-byte values).

1. Not in arena → 731 [2].
2. Not authorized → 731 [2].
3. Decode failure → 731 [2].
4. No zone → 731 [2].
5. Geo zone is null → log, 731 [2].
6. Task 0xA8 bytes, vtable `0x140A91A48`, init `0x14027FD10`.
7. Queue failure → 731 [2]. Success → 730 (empty).

## 732 — G_ArenaEventDuelGameStartReqCmd — partial

Handler `0x1403BD800`. Wire `h`.

Native quirk: every failure replies 282 (`G_DuelReadyRoundAck`), not 734.

1. Arena feature disabled → 282 [2].
2. Not arena server mode → 282 [2].
3. Daemon attraction ≠ 1 → 282 [2].
4. Not authorized → 282 [2].
5. DuelTask `0x14036D860`, init `0x1402B8010` mode 2.
6. Queue failure → 282 [2]. Success sends no reply here.

## 733 — G_ArenaEventFindArenaSessionReqCmd — partial

Handler `0x1403BDA60`. Wire `hcs` (kind byte, name cap 128).

1. Arena feature disabled → 735 [1919].
2. Arena server mode → 735 [5].
3. Not authorized → 735 [2].
4. Decode failure → 735 [2].
5. Kind 0 → attraction 1. Kind 1 or 2 → attraction 2. Other → 735 [2].
6. Send backend 140 `hqqQs` [DB ID, PC ID, attraction, name].
7. Send failure → 735 [1919]. Success sends no reply here.

## 736 — G_ArenaEventMoveToArenaReqCmd — partial

Handler `0x1403BDCE0`. Wire `hi` (arena ID).

1. Arena feature disabled → 738 [1921].
2. Arena server mode → 738 [5].
3. Not authorized → 738 [2].
4. Decode failure, or arena ID ≤ 0 → 738 [2].
5. PC arena state ≠ 3 → 738 [1922].
6. PC arena record +37528 = 0 → 738 [1923].
7. Send backend 137 `hqqqhiQ` [route, DB ID, PC ID, world, arena ID, 2].
   The trailing `Q` is the constant 2 (`mov [rsp+var_30], 2` at `0x1403BDF25`).
8. Send failure → 738 [1921]. Success sends no reply here.

## 739 — G_ArenaEventReturnToArenaLobbyReqCmd — partial

Handler `0x1403BE780`. Wire `hq` (portal key).

1. Arena feature disabled → 741 [1928].
2. Not arena server mode → 741 [1929].
3. Not authorized → 741 [2].
4. Decode failure → 741 [2].
5. Portal lookup fails → 741 [1927].
6. Task kind 159, vtable `0x140BC2450`. Portal record at +144.
7. Queue failure → 741 [2]. Success sends no reply here.

## 742 — G_ArenaEventReturnToArenaLobbyAllPcReqCmd — partial

Handler `0x1403BEA70`. Wire `hq` (portal key).

1. Not in arena → 744 [2].
2. Not authorized → 744 [2].
3. Decode failure → 744 [2].
4. Portal lookup fails → 744 [2].
5. Task kind 328, vtable `0x140B99D08`. Portal record at +144.
6. Queue failure → 741 [2]. This is a native quirk: not 744.
7. Success → 743 (empty).

## 745 — G_ArenaEventRetreivePcAssetReqCmd — partial

Handler `0x1403BED60`. Wire `hs` (name, cap 33).

1. Not authorized → 747 [2].
2. Decode failure → 747 [2].
3. Name lookup fails → 747 [1932].
4. Target has no zone → 747 [2].
5. Task kind 181, vtable `0x140BB94B8`. The target PC is the caster.
   The requester PC ID is at +144. The task goes to the target zone.
6. Queue failure → 747 [2]. Success sends no reply here.

## 748 — G_ArenaEventWarpToDuelRoundStartPcSpawnReqCmd — partial

Handler `0x1403BF0D0`. Wire `hc` (flag).

1. Arena feature disabled → 750 [1933].
2. Not arena server mode → 750 [1934].
3. Not authorized → 750 [2].
4. Decode failure, or flag > 1 → 750 [2].
5. Task kind 182, vtable `0x140BC1A38`. Flag byte at +144.
6. Queue failure → 750 [2]. Success sends no reply here.

## 751 — G_ArenaEventSetBattleRoyalStateReqCmd — partial

Handler `0x1403BF3C0`. Wire `hs` (state text, cap 128).

1. Not in arena → 753 [2].
2. Not authorized → 753 [2].
3. Decode failure → 753 [2].
4. `wcsicmp` with "standby" → state 1. With "finish" → state 4.
   Other text → 753 [2].
5. Task 0xA0 bytes, vtable `0x140BC1A90`, init `0x14027FB90`. State byte at +152.
6. Queue failure → 753 [2]. Success → 752 (empty).

## 754 — G_ArenaEventInZonePcLocReqCmd — partial

Handler `0x1403BF6A0`. Wire `h`.

1. Not in arena → 756 [2].
2. Not authorized → 756 [2].
3. No zone → 756 [2].
4. Task kind 331, vtable `0x140B96E28`.
5. Queue failure → 756 [2]. Success sends no reply here.

## 757 — G_ArenaEventBattleRoyalScoreReqCmd — partial

Handler `0x1403BF8E0`. Wire `h`.

The same flow as 754, with reply 759 and task kind 338, vtable `0x140BC1E08`.

## 959 — G_PublishRecruitArenaDungeonMemberReqCmd — partial

Handler `0x1403CBF30`. Wire `hq` (portal key).

1. Arena feature disabled → 962 [2214].
2. Not arena server mode → 962 [2215].
3. Recruit disabled → 962 [2213].
4. Daemon attraction ≠ 2 → 962 [2214].
5. Decode failure → 962 [2].
6. Portal lookup fails → 962 [2216].
7. `CheckArenaPortalUsableTask` `0x14036D670`, kind 112, vtable `0x140B9E890`,
   mode 2 (publish).
8. Queue failure → 962 [2]. Success sends no reply here.

## 963 — G_CancelRecruitArenaDungeonMemberReqCmd — partial

Handler `0x1403CC2A0`. The same flow as 959, with reply 966 and task mode 3 (cancel).

## 955 — G_ChoiceReturnToArenaLobbyReqCmd — partial

Handler `0x1403CB520`. Wire `hc` (choice).

1. Arena feature disabled → 958 `hh` [2206].
2. Arena server mode → 958 [5].
3. Decode failure → 958 [2].
4. `0x140044850(manager +4136, DB ID)` true → log and stop. No reply.
5. Choice 0 (stay):
   1. `0x140044D80` returns no session record → 958 [2208].
   2. Send backend 493 `hQqqs` [1, DB ID, record qword +120, PC name]
      to record DWORD +136 via `0x1400D0FC0`.
   3. Send failure → 958 [5].
   4. When cfg byte +537 is set, a telemetry log `0x140204390` runs.
6. Choice ≠ 0 (return):
   1. User missing or user state ≠ 0 → 958 [2209].
   2. `0x140044900` fails → 958 [2208]. It returns the last attraction.
   3. Scan the portal table for the first portal that fits the attraction:
      type 2 when `0x140042750` matches, or type 1 for attraction 1–3 or 4–6.
   4. No portal record → 958 [1687]. No zone → 958 [2].
   5. `CheckArenaPortalUsableTask` `0x14036D670` mode 1.
   6. Queue failure → 958 [2]. Success sends no reply here.

Rust: not ported. ❓ The pending manager, portal table scan and telemetry
need adapters.

## Rust port status

| ID | Rust | Tests | Boundary |
| --- | --- | --- | --- |
| 705 | none | — | pending-entry list, connection attach |
| 708 | `handle_return_to_arena_lobby_from_duel` | `ga2_arena_event` | portal type, task body |
| 711 | `handle_return_to_arena_lobby_from_dungeon` | `ga2_arena_event` | portal type, task body |
| 714 | `handle_arena_spectate_game` | `ga2_arena_event` | cross option, backend send |
| 717 | `handle_arena_spectate_return_to_arena_lobby` | `ga2_arena_event` | portal record, task body |
| 720 | `handle_list_arena_spectate_game` | `ga2_arena_event` | list cooldown, backend send |
| 723 | `handle_arena_event_teleport_to` | `ga2_arena_event` | task body |
| 726 | `handle_arena_event_recall` | `ga2_arena_event` | task body |
| 729 | `handle_arena_event_warp_to_pos` | `ga2_arena_event` | geo zone, task body |
| 732 | `handle_arena_event_duel_game_start` | `ga2_arena_event` | DuelTask body |
| 733 | `handle_arena_event_find_arena_session` | `ga2_arena_event` | backend send |
| 736 | `handle_arena_event_move_to_arena` | `ga2_arena_event` | backend send |
| 739 | `handle_arena_event_return_to_arena_lobby` | `ga2_arena_event` | portal record, task body |
| 742 | `handle_arena_event_return_to_arena_lobby_all_pc` | `ga2_arena_event` | portal record, task body |
| 745 | `handle_arena_event_retrieve_pc_asset` | `ga2_arena_event` | task body |
| 748 | `handle_arena_event_warp_to_duel_round_start` | `ga2_arena_event` | task body |
| 751 | `handle_arena_event_set_battle_royal_state` | `ga2_arena_event` | task body |
| 754 | `handle_arena_event_in_zone_pc_loc` | `ga2_arena_event` | task body |
| 757 | `handle_arena_event_battle_royal_score` | `ga2_arena_event` | task body |
| 955 | none | — | pending manager, portal scan |
| 959 | `handle_publish_recruit_arena_dungeon_member` | `ga2_arena_event` | portal record, task body |
| 963 | `handle_cancel_recruit_arena_dungeon_member` | `ga2_arena_event` | portal record, task body |

❓ This doc was written by the main session while both Sol slots were out of
quota. A Sol slot-2 review is still due.
