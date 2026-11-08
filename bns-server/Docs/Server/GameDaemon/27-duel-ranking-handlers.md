# 27 — Duel and ranking handlers (GameDaemon 381.8867)

Source: `ida/delegate/gamed/GameDaemon.exe.i64`, build 381.8867. Read through
IDA MCP. Handler table `0x1407D9F80`. Wire notation is the same as in doc 25.
Rust receives body bytes only and permits trailing bytes.

Part 1 covers the duel and party-battle zone tasks (277, 280, 283, 289, 292,
1022), the duel-bot zone tasks (968, 973, 977, 1587, 1590, 1594) and the 358
stub. Failure replies are `hh` or `hE`. Both carry a 2-byte reason.

Part 2 (not done): 105, 108, 913, 916, 921, 924, 980, 985, 989, 992, 995, 998,
1001, 1059, 1088, 1134, 1213. ❓

## Shared fields

| Field | Source | Rust |
| --- | --- | --- |
| PC zone | PC qword +2224 | Existing `Pc::zone` |
| PC ID | PC qword +40; word +46 is the ID type (1 = PC) | Existing `Pc::object_id` (`>> 48`) |
| Task attach and queue | `0x14025DB30`, `0x14069BF60`, `0x140254FD0` | `skills::queue` |
| Duel task | `0x14036D860`, init `0x1402B8010(task, zone, pc, mode)` | `DuelTask::Duel { mode }` |
| GM flag | PC signed byte +532 > 0 | `Pc::gm_grade as i8 > 0` |
| Party-battle task | `0x14036D8D0`, init `0x1402B84F0(task, zone, pc, mode)` | `DuelTask::PartyBattle { mode }` |

Rust folds an initialiser failure into the queue result. Both reply reason 2.

## 277 — G_DuelChooseDuelOrderReqCmd — partial

Handler `0x140393380`. Wire `hc` (order, signed).

1. Decode failure → 279 [2].
2. Order ≤ 0 → 279 [898].
3. No zone → 279 [2].
4. Task kind 227, 0x98 bytes, vtable `0x140C3B318`. Order byte at +144.
5. Queue failure → 279 [2]. Success sends no reply here.

## 280 — G_DuelReadyRoundReqCmd — partial

Handler `0x1403935F0`. Wire `h`.

1. No zone → 282 [2].
2. Duel task, init mode 1. Init failure → 282 [2].
3. Queue failure → 282 [2]. Success sends no reply here.

Note: 732 (doc 26) uses the same task with mode 2 and also replies 282.

## 283 — G_DuelTakeTagTokenReqCmd — partial

Handler `0x140393740`. The same flow as 280, with reply 285 and init mode 3.

## 289 — G_PartyBattleReadyReqCmd — partial

Handler `0x140393890`. No decode: the handler does not read the body.

1. No zone → 291 [2].
2. Party-battle task, init mode 0. Init failure → 291 [2].
3. Queue failure → 291 [2]. Success sends no reply here.

## 292 — G_PartyBattleStartReqCmd — partial

Handler `0x1403939E0`. The same flow as 289, with reply 294 and init mode 1.

## 358 — G_DpArenaClientToServerReqCmd — done

Target `0x14001AFE0` is the shared `return true` stub. No decode, no reply, no state.
Rust routes it to `misc::handle_return_true_stub`.

## 968 — G_DuelBotChallengeCombatStartReqCmd — partial

Handler `0x1403CCD10`. Wire `hq` (bot ID). Replies `hE`.

1. GM flag set → 970 [2].
2. Decode failure → 970 [2].
3. No zone → 970 [2].
4. Task `0x140098E90`, init `0x1402BD380(task, zone, pc, bot)`. Init failure → 970 [2].
5. Queue failure → 970 [2]. Success sends no reply here.

## 973 — G_DuelBotChallengePickStrategicToolReqCmd — partial

Handler `0x1403CCEB0`. Wire `hc` (slot). Replies `hE`.

1. Decode failure → 975 [2].
2. No zone → 975 [2].
3. Task `0x14036D930` (0xA8 bytes), init `0x1402BD990(task, zone, pc, 0, 0, slot)`.
4. Init or queue failure → 975 [2]. Success sends no reply here.

## 977 — G_DuelBotChallengeUseStrategicToolReqCmd — partial

Handler `0x1403CD040`. Wire `hq` (tool ID). The same flow as 973, with
reply 979 and init `(task, zone, pc, 1, tool, 0)`.

## 1022 — G_PartyBattleVoteCastReqCmd — partial

Handler `0x140393B40`. Wire `hq` (target PC ID).

1. Decode failure → 1024 [2].
2. No zone → 1024 [2].
3. Task kind 260, 0x98 bytes, vtable `0x140C3A928`.
4. Zone qword +16536 virtual +8 ≠ 6, or caller ID type ≠ 1, or target ID
   type ≠ 1 → log, 1024 [2].
5. Target ID at +144. Queue failure → 1024 [2]. Success sends no reply here.

❓ Rust checks both ID types. The zone-kind check (6 = party battle ❓ name)
is left to the queue adapter, because its failure reply is the same.

## 1587 — G_DuelBotTrainingRoomPickDuelBotReqCmd — partial

Handler `0x1403CC610`. Wire `hic` (bot number, level). Replies `hE`.

1. Decode failure → 1589 [2251].
2. No zone → 1589 [2].
3. Caller ID type ≠ 1, bot not in 1..=14, or level not in 1..=10 → 1589 [2].
4. Task kind 249, 0x98 bytes, vtable `0x140BAA8F0`. Bot DWORD +144, level byte +148.
5. Queue failure → 1589 [2]. Success sends no reply here.

This entry has no GM check.

## 1590 — G_DuelBotTrainingRoomCombatStartReqCmd — partial

Handler `0x1403CC8B0`. Wire `hq` (bot object ID). Replies `hE`.

1. GM flag set → 1592 [2].
2. Decode failure → 1592 [2].
3. No zone → 1592 [2].
4. Caller ID type ≠ 1, or bot ID type ≠ 240 → 1592 [2].
5. Task kind 250, 0x98 bytes, vtable `0x140BC1578`. Bot ID at +144.
6. Queue failure → 1592 [2]. Success sends no reply here.

## 1594 — G_DuelBotTrainingRoomRewardReqCmd — partial

Handler `0x1403CCAF0`. Wire `hccc` (three bytes ❓ meaning). Replies `hE`.

1. Decode failure → 1596 [2].
2. No zone → 1596 [2].
3. Task kind 262, 0x98 bytes, vtable `0x140B1CD90`. Bytes +144, +145, +146.
4. The task is queued with `0x140603BD0(task zone owner, task, 0, 0)`,
   not with `0x140254FD0`. Failure → 1596 [2]. Success sends no reply here.

## Rust port status

| ID | Rust | Tests | Boundary |
| --- | --- | --- | --- |
| 277 | `handle_duel_choose_duel_order` | `ga3_duel_ranking` | task body |
| 280 | `handle_duel_ready_round` | `ga3_duel_ranking` | task init and body |
| 283 | `handle_duel_take_tag_token` | `ga3_duel_ranking` | task init and body |
| 289 | `handle_party_battle_ready` | `ga3_duel_ranking` | task init and body |
| 292 | `handle_party_battle_start` | `ga3_duel_ranking` | task init and body |
| 358 | `handle_return_true_stub` | `ga3_duel_ranking` | none |
| 968 | `handle_duel_bot_challenge_combat_start` | `ga3_duel_ranking` | task init and body |
| 973 | `handle_duel_bot_challenge_pick_strategic_tool` | `ga3_duel_ranking` | task init and body |
| 977 | `handle_duel_bot_challenge_use_strategic_tool` | `ga3_duel_ranking` | task init and body |
| 1022 | `handle_party_battle_vote_cast` | `ga3_duel_ranking` | zone kind, task body |
| 1587 | `handle_duel_bot_training_room_pick_duel_bot` | `ga3_duel_ranking` | task body |
| 1590 | `handle_duel_bot_training_room_combat_start` | `ga3_duel_ranking` | task body |
| 1594 | `handle_duel_bot_training_room_reward` | `ga3_duel_ranking` | zone-owner queue, task body |

❓ The main session wrote this part while both Sol slots were out of quota.
A Sol slot-2 review is still due.
