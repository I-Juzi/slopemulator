# 15 — Guild receive handlers, build 381.8867 (core group)

This file covers the 17 core guild commands: create, rename, destroy, info, members, notice, grade, leader, invite, join, leave, banish and query. The craft, uniform, bank, level-up, permission and flag-bearer commands (443–475, 483, 486, 489) are not in this file yet. They are the next batch.

## New shared helpers

| Address / field | Purpose | Detail |
| --- | --- | --- |
| World +2736 → +237 byte, +248 dword | Arena config gate ("form A") | Both nonzero: the handler is "in arena". |
| PC +37520 dword under lock PC +37480 | Arena session gate ("form B") | Form B = form A, **or** the PC state is 2, 3, 5 or 6. Used by 431, 434, 437, 503, 509. |
| PC +336 (qword, `a1[42]`) | Current guild ID | 0 = no guild. `> 0` (signed) = in a guild. |
| PC +20760 (qword, `a1[2595]`) | Pending invitation guild ID | Set by the invite path (not read here). Cleared by 506/508. |
| PC +40 / +20744 | Runtime ID / PC DB ID | As in `10-session-handlers.md`. |
| `0x1403FA7D0(World+5824, &tmp, guildId)` | Guild lookup | Returns a ref slot. The caller add-refs (`0x140012090`) and releases (`0x14000B730`). |
| Guild +32416 byte | Guild type | 1 = normal, 2 = open. |
| Guild +32368 | Guild lock | Critical section. |
| Guild +32418 | Guild name | UTF-16, 33 units. |
| Guild +32771 byte | Rename in progress | 1 = yes. |
| `qword_140F77978` +10008, virtual +168(row) | Guild action table | Row object or null. Type 1 reads row +16 byte; type 2 reads row +17 byte. Null row, other type, or 0 byte = "guild action not enabled". |
| `0x14067A710` on `new(0x288)` | Guild work object (WSO) | +140 dword op, +144 guild ID, +152 requester runtime ID; other fields per op below. |
| `0x1403FB2D0(World+5824, wso, 0)` | Post WSO to the guild manager | Returns bool. On success the handler sends **no** immediate reply; the manager answers later. |
| `0x1400F9B80(guild, pcDbId)` | Remove a pending invitee from the guild | Called under the guild lock. |
| `0x1400F37C0(guild, runtimeId, 2)` | Guild leader-vote cleanup | Called when a 499/501 post fails. ❓ exact effect. |

### Guild action rows used here

| Row | Handler |
| --- | --- |
| 1 | 509 Join |
| 3 | 491 Change grade |
| 9 | 494 Delegate leader |
| 11 | 511 Leave |
| 12 | 513 Banish |
| 13 | 480 Edit notice |
| 17 | 434 Rename |

### WSO operations posted here

| Op (+140) | Handler | Extra fields |
| --- | --- | --- |
| 2 | 440 Retrieve info | — |
| 3 | 477 Retrieve members | — |
| 11 | 480 Edit notice | +170 notice text, `wcsncpy` 121 units |
| 17 | 506 Confirm invitation | +160 PC DB ID |
| 18 | 508 Reject invitation | +152 = PC DB ID (not runtime ID) |
| 22 | 491 Change grade | +440 byte target index, +456 byte new grade, +458 target name (33 units) |
| 25 / 29 | 494 Delegate leader | 29 if config `qword_140EE58A0`+776 byte = 1, else 25. +456 byte target index, +458 target name |
| 26 | 501 Reject leader | — |
| 27 | 499 Confirm leader | — |
| join (size 0x90, `0x140678250(obj,42)`, vtable `off_140C84668`) | 509 Join | +120 guild ID, +112 dword 0, +128 runtime ID, +136 PC DB ID |

## Scope and notation

Use the notation and shared helpers in `10-session-handlers.md`. Use the dispatch rules in `09-parity.md`. The dispatcher is `0x14007BC00`. The authenticated-PC table is `0x1407D9F80`. Build 348 PDB hints supply names only. All behavior and addresses come from build 381. No live test ran.

`done` means all game-logic paths in the handler were read. `partial` lists unread paths with ❓. A posted WSO or queued task is not a completed operation. All replies use `hh` (command, reason) and go to self. Every handler returns true. No handler here kicks. Offsets are decimal unless marked `0x`.

Shared task attachment (431, 434) uses `0x14025DB30` (zone), `0x14069BF60` (PC child link), and queue `0x140254FD0`, as in spec 14.

## 431 — G_CreateGuildReqCmd — partial

- **381:** `0x1403A1D10`. **PDB hint:** `HandleGame_CreateGuildReq`.
- **Wire:** `hQsq` (3 fields): guild type (u8), guild name (string, ≤ 33 units), q (❓ meaning; checked nonzero by the task init).
- **Checks/order (reply 433):**
  1. Arena form B → 5.
  2. Decode fails → 1352.
  3. Type 0 → 1352.
  4. Type 2 and config +1616 byte set and World +6208 dword ≥ config +1620 dword (open-guild limit) → 1352.
  5. UIC user lookup fails → 2.
  6. Forbidden name (name filter at PC-world +2728, virtual +48) → 1356.
  7. PC +336 > 0 (already in a guild) → 1358.
  8. Config `qword_140EE58A0`+2105 byte set and PC +328 (faction, signed byte) ≤ 0 → 2.
  9. No zone → 2.
  10. Register-guild task (vtable `off_140CF12B8`, task +68 = 101): init needs type, name and q nonzero, caster kind 1 → else 2. Queue failure → 2.
- **State:** Task only. No guild record is made in the handler.
- **Output:** Success sends nothing now. ❓ The task path (432 Ok / 433 Fail, DB procedure) is not read.

## 434 — G_RenameGuildReqCmd — partial

- **381:** `0x1403A2310`. **PDB hint:** `HandleGame_RenameGuildReq`.
- **Wire:** `hQsChh;`: rename type (u8), new name (string, 33 units), then a counted list (≤ 16) of dispose-item records `hh` (slot, count).
- **Checks/order (reply 436, default reason 2):**
  1. Arena form B → 5.
  2. Config +244 byte = 0 (rename disabled) → 1363.
  3. PC +336 = 0 → 2.
  4. Guild lookup fails → 2.
  5. Guild action row 17 not enabled → 2.
  6. Guild +32771 = 1 (rename in progress) → 2.
  7. Decode fails → 2. List count < 0 or > 16 → 2. Any record decode fails or has count 0 → 2.
  8. Type 1 (by guild-name prefix): config +246 prefix empty → 1363. Item count ≠ 0 → 2. Old name lacks the prefix → 1368.
  9. Type 2 (by inventory item): config +768 dword = 0 → 1363. Item count ≠ 1 → 2. Rename time limit not passed → 1367.
  10. Other type → 2.
  11. UIC user lookup fails → 2. Forbidden new name → 1371.
  12. No zone → 2. Task alloc (`0x140543E50`) fails → 2. Task init (type not 1/2, caster not PC) fails → 2. Queue fails → 2.
- **State:** Task only.
- **Output:** Success sends nothing now. ❓ Rename task, 435 notify and DB work not read.

## 437 — G_DestroyGuildReqCmd — done

- **381:** `0x1403A3420`. **PDB hint:** `HandleGame_DestroyGuildReq`.
- **Wire:** `hsqq` (3 fields): guild name, guild ID, NPC ID.
- **Checks/order (reply 439):** Arena form B → 5. Otherwise decode, log (decode fail or "invalid packet parameter"), and **always** reply 1374.
- **State:** None. In 381 the destroy path is disabled at the handler.

## 440 — G_RetrieveGuildInfoReqCmd — done

- **381:** `0x1403A37B0`. **PDB hint:** `HandleGame_RetrieveGuildInfoReq`.
- **Wire:** No body decoder.
- **Checks/order (reply 442):** Arena form A → 5. PC +336 = 0 → 1384. WSO alloc fails → 2. Post op 2 fails → 2.
- **Output:** Success: no immediate reply (the manager sends 441).

## 477 — G_RetrieveGuildMembersReqCmd — done

- **381:** `0x1403A3960`. **PDB hint:** `HandleGame_RetrieveGuildMembersReq`.
- **Wire:** No body decoder.
- **Checks/order (reply 479):** Arena form A → 5. PC +336 = 0 → 1385. WSO alloc fails → 2. Post op 3 fails → 2.

## 480 — G_EditGuildNotificationMessageReqCmd — done

- **381:** `0x1403A3B20`. **PDB hint:** `HandleGame_EditGuildNotificationMessageReq`.
- **Wire:** `hs`: notice text.
- **Checks/order (reply 481):** Arena form A → 5. Decode fails → 1386. Empty text → 1386. PC +336 = 0 → 1389. Guild lookup fails → 2. Action row 13 not enabled → 2. WSO alloc fails → 2. Post op 11 fails → 2.
- **Output:** Success: no immediate reply; log "#begin edit guild notification message".

## 491 — G_ChangeGuildMemberGradeReqCmd — done

- **381:** `0x1403A49D0`. **PDB hint:** `HandleGame_ChangeGuildMemberGradeReq`.
- **Wire:** `hcsc` (3 fields): target index (i8), target name, new grade (i8).
- **Checks/order (reply 492):** Arena form A → 5. Decode fails → 1403. PC +336 = 0 → 1405. Guild lookup fails → 2. Action row 3 not enabled → 2. WSO alloc fails → 2. Empty target name → 2 (no post). Post op 22 fails → 2.

## 494 — G_DelegateGuildLeaderReqCmd — done

- **381:** `0x1403A4E20`. **PDB hint:** `HandleGame_DelegateGuildLeaderReq`.
- **Wire:** `hcs` (2 fields): target index (i8), target name.
- **Checks/order (reply 495):** Arena form A → 5. Decode fails → 1409. PC +336 = 0 → 1410. Guild lookup fails → 2. Action row 9 not enabled → 2. WSO alloc fails → 2. Empty name → 2. Post op 25/29 fails → 2.

## 499 — G_ConfirmGuildLeaderReqCmd — done

- **381:** `0x1403A5220`. **PDB hint:** `HandleGame_ConfirmGuildLeaderReq`.
- **Wire:** No body decoder.
- **Checks/order (reply 500):** Arena form A → 5. PC +336 = 0 → 1410. WSO alloc or post op 27 fails → look up the guild and call `0x1400F37C0(guild, runtimeId, 2)`, then reply 2.

## 501 — G_RejectGuildLeaderReqCmd — done

- **381:** `0x1403A5450`. **PDB hint:** `HandleGame_RejectGuildLeaderReq`.
- Same as 499, with reply 502 and WSO op 26.

## 503 — G_InviteGuildReqCmd — done

- **381:** `0x1403A5680`. **PDB hint:** `HandleGame_InviteGuildReq`.
- **Wire:** `hsQ` (2 fields): target name, invitation type (u8).
- **Checks/order (reply 504):** Arena form B → 5. Otherwise decode, log, and **always** reply 1427.
- **State:** None. Invite by this command is disabled in 381. ❓ Which path sets PC +20760 (pending guild) is not found yet.

## 506 — G_ConfirmInvitationGuildReqCmd — done

- **381:** `0x1403A5A70`. **PDB hint:** `HandleGame_ConfirmInvitationGuildReq`.
- **Wire:** No body decoder.
- **Checks/order (reply 507):** Arena form A → 5. PC +20760 = 0 → 1440. Guild lookup fails → 1439. WSO alloc fails, or post op 17 fails → remove the invitee (`0x1400F9B80` under guild lock), clear PC +20760, **no reply**.
- **Output:** Success: no immediate reply. PC +20760 stays set.

## 508 — G_RejectInvitationGuildReqCmd — done

- **381:** `0x1403A5DA0`. **PDB hint:** `HandleGame_RejectInvitationGuildReq`.
- **Wire:** No body decoder.
- **Checks/order:** Arena form A → log and return, **no reply** (unlike the other handlers). PC +20760 = 0 → clear, log. Guild lookup fails → clear, log. WSO alloc or post op 18 fails → remove the invitee under guild lock, clear, log. Post succeeds → clear PC +20760.
- **Output:** This handler never sends a reply.

## 509 — G_JoinGuildReqCmd — done

- **381:** `0x1403A6050`. **PDB hint:** `HandleGame_JoinGuildReq`.
- **Wire:** `hq`: guild ID.
- **Checks/order (reply 510):**
  1. Arena form B → 5.
  2. Decode fails → 1446.
  3. Guild ID 0 → 1447.
  4. PC +336 > 0 → 1451.
  5. Guild lookup fails → 1448.
  6. Guild type ≠ 2 (not open) → 1449.
  7. Action row 1 not enabled → 2.
  8. Config +2041 byte = 1 (join-renewal on): look up the PC's join tickets in the ticket manager (World +6392, ordered map by PC DB ID, SRW lock +48). Any live ticket (count > 0 after expiry pruning, `0x140113160`…`0x140113680`) → 1457.
  9. Post the join object fails → 2.
- **Output:** Success: no immediate reply.

## 511 — G_LeaveGuildReqCmd — partial

- **381:** `0x1403A6760`. **PDB hint:** `HandleGame_LeaveGuildReq`.
- **Wire:** No body decoder.
- **Checks/order (reply 512):** Arena form A → 5. PC +336 = 0 → 1459. Guild lookup fails → 2. Action row 11 not enabled → 2. `0x1400E6970(guild, runtimeId, PC+194 name, &reason)` fails → 2 (the helper's reason output is logged only).
- **Output:** Success: no reply here. ❓ `0x1400E6970` (member removal, broadcasts, DB) not read.

## 513 — G_BanishGuildMemberReqCmd — partial

- **381:** `0x1403A6A40`. **PDB hint:** `HandleGame_BanishGuildMemberReq`.
- **Wire:** `hcs` (2 fields): target index (i8), target name (33 units).
- **Checks/order (reply 514):** Arena form A → 5. Decode fails → 1420. Index < 0 → 1420. PC +336 = 0 → 1422. Guild lookup fails → 2. Action row 12 not enabled → 2. `0x1400E5890(guild, runtimeId, index, name, &reason, 33)` fails → reply 514 with the helper's `reason` output.
- **Output:** Success: no reply. ❓ `0x1400E5890` (removal, broadcasts, DB) not read.

## 515 — G_QueryGuildInfoReqCmd — partial

- **381:** `0x1403A6DE0`. **PDB hint:** `HandleGame_QueryGuildInfoReq`.
- **Wire:** `hCq;`: a counted list of guild IDs.
- **Checks/order (reply 517 on error):** Count > 50 → 2. Any ID decode fails → 2. Any ID = 0 → 2. No arena check.
- **Output:** Reply 516 `hiCqQsCc;;`: per guild, ID, type, name (from guild +32418 or a cache), and a 28-entry uniform list. Compose fail → 517 reason 2.
- **❓ Missing:** The cache branch (`0x1400176E0`, ordered map, entries ≥ 8 bytes) and the exact field meaning of `i`, `C` and the uniform records.

## Status

| Id | State |
| --- | --- |
| 431 | partial |
| 434 | partial |
| 437 | done |
| 440 | done |
| 477 | done |
| 480 | done |
| 491 | done |
| 494 | done |
| 499 | done |
| 501 | done |
| 503 | done |
| 506 | done |
| 508 | done |
| 509 | done |
| 511 | partial |
| 513 | partial |
| 515 | partial |
