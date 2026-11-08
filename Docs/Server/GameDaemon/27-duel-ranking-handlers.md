# 27 — Duel and ranking handlers (GameDaemon 381.8867)

Source: `ida/delegate/gamed/GameDaemon.exe.i64`, build 381.8867. Read through
IDA MCP. Handler table `0x1407D9F80`. Wire notation is the same as in doc 25.
Rust receives body bytes only and permits trailing bytes.

Part 1 covers the duel and party-battle zone tasks (277, 280, 283, 289, 292,
1022) and the 358 stub. All failure replies are `hh` (2-byte reason).

Part 2 (not done): 105, 108, 913, 916, 921, 924, 968, 973, 977, 980, 985, 989,
992, 995, 998, 1001, 1059, 1088, 1134, 1213, 1587, 1590, 1594. ❓

## Shared fields

| Field | Source | Rust |
| --- | --- | --- |
| PC zone | PC qword +2224 | Existing `Pc::zone` |
| PC ID | PC qword +40; word +46 is the ID type (1 = PC) | Existing `Pc::object_id` (`>> 48`) |
| Task attach and queue | `0x14025DB30`, `0x14069BF60`, `0x140254FD0` | `skills::queue` |
| Duel task | `0x14036D860`, init `0x1402B8010(task, zone, pc, mode)` | `DuelTask::Duel { mode }` |
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

## Rust port status

| ID | Rust | Tests | Boundary |
| --- | --- | --- | --- |
| 277 | `handle_duel_choose_duel_order` | `ga3_duel_ranking` | task body |
| 280 | `handle_duel_ready_round` | `ga3_duel_ranking` | task init and body |
| 283 | `handle_duel_take_tag_token` | `ga3_duel_ranking` | task init and body |
| 289 | `handle_party_battle_ready` | `ga3_duel_ranking` | task init and body |
| 292 | `handle_party_battle_start` | `ga3_duel_ranking` | task init and body |
| 358 | `handle_return_true_stub` | `ga3_duel_ranking` | none |
| 1022 | `handle_party_battle_vote_cast` | `ga3_duel_ranking` | zone kind, task body |

❓ The main session wrote this part while both Sol slots were out of quota.
A Sol slot-2 review is still due.
