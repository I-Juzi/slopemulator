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
| 2 | 486 Edit member permission |
| 3 | 491 Change grade |
| 4 | 489 Change flag bearer |
| 5 | 443 Guild level up |
| 6 | 445 Deposit in guild bank |
| 7 | 447 Contract guild craft |
| 8 | 461 Contract guild uniform |
| 9 | 494 Delegate leader |
| 11 | 511 Leave |
| 12 | 513 Banish |
| 13 | 480 Edit notice |
| 17 | 434 Rename |
| 18 | 557, 563, 567, 571, 575, 579, 584, 588, 592, 596 Join renewal (on the looked-up guild) |

### WSO operations posted here

| Op (+140) | Handler | Extra fields |
| --- | --- | --- |
| 2 | 440 Retrieve info | — |
| 3 | 477 Retrieve members | — |
| 6 | 443 Guild level up | — |
| 13 | 483 Retrieve member permission | — |
| 14 | 486 Edit member permission | +412, +416, +420, +424: the four permission masks |
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

## 443 — G_GuildLevelUpReqCmd — done

- **381:** `0x1403AA270`. **PDB hint:** `HandleGame_GuildLevelUpReq`.
- **Wire:** `h` (no body read).
- **Checks/order (reply 444):**
  1. Arena form B → 5.
  2. Config `qword_140EE58A0` +2107 byte = 1 → 2 (❓ meaning: level-up disabled).
  3. No guild (PC +336 = 0) → 1554.
  4. Guild lookup fails → 2.
  5. Action row 5 not enabled → 2.
  6. WSO alloc fails → 2.
  7. Post op 6 fails → 2.
- **Output:** Success posts op 6 (+144 guild ID, +152 runtime ID). No reply.

## 445 — G_DepositInGuildBankReqCmd — done

- **381:** `0x1403AA5D0`. **PDB hint:** `HandleGame_DepositInGuildBankReq`.
- **Wire:** `hichh`: money (`i`), then a repeat count (`c`, read by `0x14069DA00`), then `count` records of (`h` slot, `h` amount).
- **Checks/order (reply 446):**
  1. Config +2106 byte = 0 (bank off) → 2.
  2. Arena form B → 5.
  3. T2 gate (PC byte +13768 = 1, spec 12) → 7.
  4. Money decode fails → 1553.
  5. Money < 0, or money > config +64 (signed dword) → 33.
  6. No guild → 2. Lookup fails → 2. Action row 6 not enabled → 2.
  7. Count < 0, or count > 5, or money = 0 and count = 0 → 1553 (log 15279).
  8. Money > 0 and count > 0 → 1579 (money and items in one request).
  9. A record decode fails → 1553.
  10. Two records with the same slot → 1553 (log 15311).
  11. No zone (PC +2224) → 2.
  12. Task alloc (`new(0x188)`, `0x140540BF0`) fails → 2.
  13. PC kind (+46) ≠ 1 → assert `0x14069CB70` (15287), then 2.
  14. A record with slot 0 or amount 0 → 2 (log 15309).
- **Output:** Task +144 = 1, +148 money, +152 count, +154 slot words, +174 amount words. Attach to the zone (`0x14025DB30`), +128 runtime ID, PC child link (`0x14069BF60`), +138 = 1. Queue with `0x140603BD0(task+32 → +16, task, 0, 0)`. This queue has no failure branch: no reply on success.

## 483 — G_RetrieveGuildMemberPermissionReqCmd — done

- **381:** `0x1403A3F50`. **PDB hint:** `HandleGame_RetrieveGuildMemberPermissionReq`.
- **Wire:** `h` (no body read).
- **Checks/order (reply 485):** Arena form A → 5. No guild → 1384. Lookup fails → 2. WSO alloc fails → 2. Post op 13 fails → 2.
- **Output:** Success posts op 13. No reply. No action-row check.

## 486 — G_EditGuildMemberPermissionReqCmd — done

- **381:** `0x1403A41F0`. **PDB hint:** `HandleGame_EditGuildMemberPermissionReq`.
- **Wire:** `hiiii`: four permission masks.
- **Checks/order (reply 488):**
  1. Arena form A → 5.
  2. Decode fails → 1395.
  3. No guild → 1398.
  4. Lookup fails → 2. Action row 2 not enabled → 2.
  5. A mask sets a bit outside its limit → 1396. Limits: normal guild (type 1) World +6264, +6268, +6272, +6276; open guild World +6304, +6308, +6312, +6316. The test is `limit < (limit | mask)` (❓ signed compare; equal to a bit-subset test for non-negative values).
  6. WSO alloc fails → 2. Post op 14 fails → 2.
- **Output:** Op 14 with the masks at +412..+424. No reply.

## 489 — G_ChangeGuildFlagBearerReqCmd — partial

- **381:** `0x1403A4690`. **PDB hint:** `HandleGame_ChangeGuildFlagBearerReq`.
- **Wire:** `hcs`: target index, target name (33 units).
- **Checks/order (reply 490):**
  1. Arena form B → 5.
  2. T2 gate → 7.
  3. Decode fails → 1553.
  4. No guild → 1554.
  5. Lookup fails → 2. Action row 4 not enabled → 2.
  6. `0x1400FF0D0(guild, PC, index, name, 33)` returns a nonzero reason → reply that reason.
- **Output:** Helper success (0) → no reply.
- **❓ Missing:** The body of `0x1400FF0D0` (member lookup, broadcasts, DB work).

## Guild craft group (447–459): shared facts

- **User gate (CS):** PC +20696 (ref-counted user object) is non-null and PC byte +20752 = 0. The failure logs "user is not found". Rust field: `Pc::user_present`. 447, 449, 451 and 453 need it.
- **Records:** A `C` (u16) repeat count read by `0x14069DA00`, then `count` records of (`h` slot, `h` amount). The count test is `(u16)count > 5` → reject. Source: format strings `0x140BFFA20` area (`hiQsChh;Chh;`, `hqChh;`, `hqicChh;`); the leading `h` is the opcode.
- **Log names:** The CS failure logs "user is not found". The T2 failure logs "cannot process in trial play".
- **Task queue:** These tasks queue with `0x140603BD0(task+32 → +16, task, 0, 0)`, as in 445. That call has no failure branch.
- **Guild +32488:** leader PC DB ID. **PC +20744:** PC DB ID. **PC +20664:** ❓ unread field passed to the abandon helper.

## 447 — G_ContractGuildCraftReqCmd — partial

- **381:** `0x1403AAF20`. **PDB hint:** `HandleGame_ContractGuildCraftReq`.
- **Wire:** `hiQsChh;Chh;`: `i`, `Q`, `s`, then list A (contract BM items: `C` count, records), then list B (rent BM items: `C` count, records).
- **Checks/order (reply 448):**
  1. Arena form B → 5.
  2. T2 → 7.
  3. CS fails → 1.
  4. No guild → 2. Lookup fails → 2. Action row 7 not enabled → 2.
  5. Header `iqs` decode fails → 1553.
  6. List A count > 5, or a record fails → 1553.
  7. List B count > 5, or a record fails → 1553.
  8. A slot repeats in list A, a list A slot is also in list B, or a slot repeats in list B → 1553.
  9. Both counts are 1 and the two slots are equal → 1553.
  10. No zone → 1553.
  11. Build a 0x158 object (`0x14002F440(obj, CS object)`, vtable `off_1409CEEF8`, id 1843466, +24 = World +5128 call). Try the World +5080 lock (`0x1400D4FE0`). Fail → 1553. Success → `0x1400D4A80`, log, no reply.
- **❓ Missing:** The decompile does not show the decoded fields stored in the object. The object's later work is not read.

## 449 — G_AssentGuildCraftReqCmd — done

- **381:** `0x1403AB970`. **PDB hint:** `HandleGame_AssentGuildCraftReq`.
- **Wire:** `hqc`. Only `q` is stored (❓ the `c` value is not used on any visible path).
- **Checks/order (reply 450):** Arena form B → 5. T2 → 7. CS fails → 1. Decode fails → 1553. No zone → **1553**.
- **Output:** Task `new(0xA8)`, `0x14025D9F0`, vtable `off_140BB8BF0`, +68 = 167, +8 = 10950410. +144 = 1, +152 = q, +160 = 0. Attach and queue (shared). No allocation check, no PC-kind check. No reply.

## 451 — G_DonateGuildCraftNormalMaterialReqCmd — done

- **381:** `0x1403ABD30`. **PDB hint:** `HandleGame_DonateGuildCraftNormalMaterialReq`.
- **Wire:** `hqChh;`: `q`, then records.
- **Checks/order (reply 452):**
  1. Arena form B → 5. T2 → 7. CS fails → 1.
  2. `q` decode fails → 1553.
  3. Count > 5 → 1553. A record fails → 1553.
  4. A slot repeats → **1592**.
  5. No zone → 2.
  6. Task alloc (`new(0x190)`, `0x14053C7D0`) fails → 2.
  7. PC kind ≠ 1 → assert (13440), then 2.
  8. q = 0 or count = 0 → 2.
  9. A record with slot 0 or amount 0 → 2.
- **Output:** Task +144 = 1, +152 q, +160 count, +162 slots, +182 amounts. Queue (shared). No reply.

## 453 — G_DonateGuildCraftBrandMaterialReqCmd — done

- **381:** `0x1403AC500`. **PDB hint:** `HandleGame_DonateGuildCraftBrandMaterialReq`.
- **Wire:** `hqicChh;`: `q`, `i`, `c`, then records.
- **Checks/order (reply 454):** As 451. Header `qic` decode fails → 1553. Task `0x14053D830`.
- **Output:** Task +144 = 1, +152 q, +160 i, +164 c, +166 count, +168 slots, +188 amounts.

## 455 — G_MakeGuildCraftReqCmd — partial

- **381:** `0x1403ACD00`. **PDB hint:** `HandleGame_MakeGuildCraftReq`.
- **Wire:** `hq`.
- **Checks/order (reply 456):**
  1. Arena form B → 5. T2 → 7.
  2. Decode fails → 1553.
  3. CS fails → **2** (not 1).
  4. No guild → 1554. Lookup fails → 2.
  5. `0x1400F6470(guild, PC DB ID, q)` returns nonzero → reply that reason.
- **Output:** Helper success → no reply. No action-row check.
- **❓ Missing:** Body of `0x1400F6470`.

## 457 — G_TakeGuildCraftReqCmd — done

- **381:** `0x1403AD0A0`. **PDB hint:** `HandleGame_TakeGuildCraftReq`.
- **Wire:** `hqh`. Only `q` is stored (❓ the `h` value is not used).
- **Checks/order (reply 458):** Arena form B → 5. T2 → 7. Decode fails → 1553. No zone → 2. Task alloc (`new(0x3400)`, `0x14027F720`) fails → 2. PC kind ≠ 1 → assert (14275), then 2. No CS gate.
- **Output:** Task +144 = 1, +152 q, +160 = 0. Queue (shared). No reply.

## 459 — G_AbandonGuildCraftReqCmd — partial

- **381:** `0x1403AD450`. **PDB hint:** `HandleGame_AbandonGuildCraftReq`.
- **Wire:** `hq`. ❓ The decoded `q` is not used on a visible path.
- **Checks/order (reply 460):**
  1. Arena form B → 5. T2 → 7.
  2. Decode fails → 1553.
  3. No guild → 1554. Lookup fails → 2.
  4. Guild +32488 ≠ PC DB ID (not the leader) → 1597.
  5. No craft object (`0x140105E30` under guild lock +34456) → 1555.
  6. `0x140105BA0(craft, PC +20664, guild +32408, PC DB ID)` nonzero → reply that reason.
- **Output:** Helper success → no reply.
- **❓ Missing:** Body of `0x140105BA0`.

## Guild uniform group (461–475): shared facts

- **Uniform gate (U):** Global byte `qword_140EE58A0 + 152` = 0 → reply 1568. The log says "not use guild uniform". All eight handlers test it first. Rust: `GuildConfig::uniform_enabled`.
- **Then:** Arena form B → 5. T2 → 7. The order is the same in all eight handlers.
- **Guild:** PC +336 = 0 → 1554 ("is not guild member"). Lookup `0x1403FA7D0` fails → 2.
- **Records:** As the craft group: a `C` (u16) count, then (`h` slot, `h` amount) records.
- **Task queue:** 463 and 475 queue with `0x140603BD0`, as 445 and the craft tasks. That call has no failure branch.
- **Helpers:** 465–473 call one opaque guild helper each. A nonzero result is the reply reason. Zero → no reply.

## 461 — G_ContractGuildUniformReqCmd — partial

- **381:** `0x1403AD830`. **PDB hint:** `HandleGame_ContractGuildUniformReq`.
- **Wire:** `hqsCc;Ci;` (`0x140BFFA20`): `q`, `s` uniform name (33 units), then param types (`C` count, `c` each), then recipe IDs (`C` count, `i` each).
- **Checks/order (reply 462):**
  1. U → 1568. Arena form B → 5. T2 → 7.
  2. CS (user gate) fails → 1.
  3. `q` + name decode fails → 1553.
  4. Param type count > 28 → 1553. A param decode fails → 1553. A param is 0 → 1553.
  5. Recipe count > 28 → 1553. A recipe decode fails → 1553.
  6. Name rules: World +2744 config is missing, or `0x14012F9F0(World +2800, name, cfg +588…+608)` is nonzero → 1602.
  7. Name filter (World +2728 → +48, vtable slot 0) exists and (CS object is gone, or the filter rejects the name) → 1602.
  8. PC byte +344 ≠ 0 → 1600 ("invalid activated faction"). ❓ The meaning of +344 is not proven.
  9. No guild → 1554. Lookup fails → 2. Action row 8 not enabled → 2.
  10. `0x1400F73E0(guild, PC, q, params, recipes, outputs…)` nonzero → reply that reason ("fail to EnableContractGuildUniform").
  11. Transaction (`new(0x4F0)`, type `0x24210A`) register fails → 2 ("fail to start tran").
- **Output:** Success → no reply.
- **❓ Missing:** Body of `0x1400F73E0` and the transaction work.

## 463 — G_DonateGuildUniformMaterialReqCmd — done

- **381:** `0x1403AE3D0`. **PDB hint:** `HandleGame_DonateGuildUniformMaterialReq`.
- **Wire:** `hqChh;`: `q` uniform ID, then records.
- **Checks/order (reply 464):**
  1. U → 1568. Arena form B → 5. T2 → 7. No CS gate.
  2. `q` decode fails → 1553.
  3. Count > 5 → 1553. A record decode fails → 1553.
  4. A record has slot 0 or amount ≤ 0 (signed i16) → 1553. This check runs during decode.
  5. A slot repeats → 1592.
  6. No zone (PC +2224) → 2. Task alloc (`new(0x160)`, `0x14053EE50`) fails → 2. PC kind ≠ 1 → assert (14483), then 2.
  7. `q` = 0 or count = 0 → 2.
- **Output:** Task (+144 = 1, +152 `q`, +160 count, records from +162/+172) queues. No reply.

## 465 — G_AbandonGuildUniformReqCmd — partial

- **381:** `0x1403AEBD0`. **Wire:** `hq`.
- **Checks/order (reply 466):** U → 1568. Arena form B → 5. T2 → 7. Decode fails → 1553. No guild → 1554. Lookup fails → 2. `0x1400F7B00(guild, PC +20664, PC DB ID, q)` nonzero → that reason.
- **❓ Missing:** Body of `0x1400F7B00`.

## 467 — G_MakeGuildUniformReqCmd — partial

- **381:** `0x1403AEF30`. **Wire:** `hq`.
- **Checks/order (reply 468):** As 465. Helper `0x1400F7D40(guild, PC, q)`.
- **❓ Missing:** Body of `0x1400F7D40`.

## 469 — G_RegisterGuildUniformReqCmd — partial

- **381:** `0x1403AF230`. **Wire:** `hq`.
- **Checks/order (reply 470):** As 465. Helper `0x1400F8420(guild, PC, q)`.
- **❓ Missing:** Body of `0x1400F8420`.

## 471 — G_PutOnGuildUniformReqCmd — partial

- **381:** `0x1403AF590`. **Wire:** `hq`.
- **Checks/order (reply 472):** As 465. Helper `0x1400F92D0(guild, PC, q)`.
- **❓ Missing:** Body of `0x1400F92D0`.

## 473 — G_RegisterGuildUniformWithDeletionReqCmd — partial

- **381:** `0x1403AF8F0`. **Wire:** `hqq`.
- **Checks/order (reply 474):** As 465, but the decode reads two `q`. Helper `0x1400F8920(guild, PC, q1, q2)`.
- **❓ Missing:** Body of `0x1400F8920`.

## 475 — G_IncreaseUniformRegistrationQuotaReqCmd — done

- **381:** `0x1403AFC90`. **PDB hint:** `HandleGame_IncreaseUniformRegistrationQuotaReq`.
- **Wire:** `hcChh;`: `c` flag, then records.
- **Checks/order (reply 476):**
  1. U → 1568. Arena form B → 5. T2 → 7.
  2. CS fails → 1.
  3. `c` decode fails → 1553. Count > 5 → 1553. A record decode fails → 1553. A slot repeats → **1553** (not 1592).
  4. No zone → 2. Task alloc (`new(0x158)`, `0x14053FE40`) fails → 2. PC kind ≠ 1 → assert (14892), then 2.
  5. Count = 0 → 2. A record with slot 0 or amount 0 → 2.
- **Output:** Task (+144 = 1, +148 flag, +150 count, records from +152/+162) queues. No reply.

## Guild join-renewal group (557–600): shared facts

- **Source:** GameDaemon 381.8867, `ida/delegate/gamed/GameDaemon.exe`. All logs start with `[guild-join-renewal], HandleGame_<Name>Req`.
- **Renewal gate (J):** Config byte `qword_140EE58A0 + 2041` = 0 → reply 2 ("guild join renewal is disabled"). All 13 handlers test it first. Rust: `GuildConfig::join_renewal` (also used by 509).
- **Arena:** 557 and 560 use form A. All others use form B. Both → 5.
- **Cooldown (K):** `GetTickCount64() - base < PC +20880 + 1000 × config dword +2044` → reject ("too frequent request"). 563 → 1624. 579 and 583 → 1644. Only 563 and 579 write PC +20880, and only after the task queues. Rust: `GuildConfig::join_request_cooldown_secs`, `Pc::join_request_tick`, `World::tick`.
- **Row 18:** Action row 18 must be enabled for the type of the looked-up guild (+16 normal, +17 open). Otherwise → 2 ("guild action disabled").
- **Ticket task (T):** `new(0xB0)`, ctor `0x14036D550`, init `0x140297680(task, zone, PC, op, a, b, c)`, queue `0x140603BD0`. Allocation failure, zone (PC +2224) null and init failure all → 2. ❓ The body of `0x140297680` and the task's later replies are not read.
- **Ticket manager:** World +6392, a map under an SRW lock (+48). Key = guild ID (557) or PC DB ID (560). Entry +368 count, +32 ticket pointers, +376 list timestamp, lock +392. Ticket getters `0x140113160` (q), `0x140113240` (q), `0x140113320` (h type), `0x140113400` (h), `0x140113680` (q expiry). 557/560 skip expired tickets and types 4 and 9.

| Cmd | Op | a | b | c | Guild rule | Reply |
| --- | --- | --- | --- | --- | --- | --- |
| 563 Send application | 1 | PC DB ID | PC DB ID | guild found by name | PC has no guild (else 1626) | 565 |
| 567 Cancel application | 2 | PC DB ID | PC DB ID | `q` guild ID | PC has no guild | 569 |
| 571 Reject application | 3 | PC DB ID | `q` PC DB ID | own guild | PC has a guild | 573 |
| 575 Accept application | 4 | PC DB ID | `q` | own guild | PC has a guild | 577 |
| 579 Send invitation | 5 | PC DB ID | `q` | own guild | PC has a guild | 581 |
| 584 Cancel invitation | 6 | PC DB ID | `q` | own guild | PC has a guild | 586 |
| 588 Reject invitation | 7 | PC DB ID | PC DB ID | `q` guild ID | none | 590 |
| 592 Accept invitation | 8 | PC DB ID | PC DB ID | `q` guild ID | PC has no guild | 594 |

## 557 — G_RetrieveGuildJoinTicketListPerGuildReqCmd — partial

- **381:** `0x1403B0450`. **Wire:** `hq` (list timestamp).
- **Checks/order (reply 559):** J → 2. Arena form A → 5. No guild → 2. Decode fails → 2. Lookup `0x1403FA7D0` fails → 2. Row 18 off → 2. Permission `0x1400FCD00(guild, DB ID, 5)` and `(…, 17)` both false → 2.
- **Output:** Command 558, format `hiqCqqhhq;` (`aHiqcqqhhq`). Records = (q, q, h type, h, q expiry). If the request timestamp equals the entry +376, the list is empty. A compose failure → 559 reason 2.
- **❓ Missing:** The leading `i` value (`sub_140017510(&p, 1, stamp, v33)`). Rust stops at a boundary after the gates.

## 560 — G_RetrieveGuildJoinTicketListPerPcReqCmd — partial

- **381:** `0x1403B0DB0`. **Wire:** `h` (no body).
- **Checks/order (reply 562):** J → 2. Arena form A → 5. PC has a guild → 2. PC DB ID 0 → empty list.
- **Output:** Command 561, format `hiCqqhhq;`, keyed by PC DB ID.
- **❓ Missing:** As 557. Rust stops at a boundary.

## 563 — G_SendGuildApplicationReqCmd — partial

- **381:** `0x1403B13F0`. **Wire:** `hs` (guild name).
- **Checks/order (reply 565):**
  1. J → 2. Arena form B → 5.
  2. K → 1624.
  3. PC has a guild → 1626 ("already joined").
  4. Decode fails → 2. Name empty → 2.
  5. Lookup by name `0x1403FA8E0` (World +5824) fails → 2.
  6. Guild +32417 ≠ PC +328 → 1625 ("faction mismatch").
  7. Guild type (+32416) = 2 → 1623 ("guild type is open").
  8. Row 18 off → 2.
  9. T (op 1, c = guild +32408) fails → 2.
- **Output:** Task queues. PC +20880 = now. No reply.

## 567 — G_CancelGuildApplicationReqCmd — partial

- **381:** `0x1403B1990`. **Wire:** `hq`.
- **Checks/order (reply 569):** J → 2. Arena B → 5. PC has a guild → 2. Decode fails → 2. `q` = 0 → 2. Lookup of guild `q` fails → 2. Row 18 off → 2. T (op 2) fails → 2.

## 571 — G_RejectGuildApplicationReqCmd — partial

- **381:** `0x1403B1DE0`. **Wire:** `hq` (applicant DB ID).
- **Checks/order (reply 573):** J → 2. Arena B → 5. No guild → 2. Decode fails → 2. `q` = 0 → 2. Own guild lookup fails → 2. Row 18 off → 2. T (op 3) fails → 2.

## 575 — G_AcceptGuildApplicationReqCmd — partial

- **381:** `0x1403B2240`. As 571, reply 577, op 4.

## 579 — G_SendGuildInvitationReqCmd — partial

- **381:** `0x1403B26A0`. **Wire:** `hq` (target DB ID).
- **Checks/order (reply 581):** J → 2. Arena B → 5. K → 1644. Then as 571 with op 5. Success writes PC +20880.

## 583 — G_SendGuildInvitation2ReqCmd — done

- **381:** `0x1403B2B70`. **Wire:** `hsq`. The reply command is **581**, not 585.
- **Checks/order:** J → 2. Arena B → 5. K → 1644. No guild → 2. Decode fails → 2 ("packet decompose failed"). Decode succeeds → 2 ("invalid parameter").
- **Output:** Build 381 rejects every request. No task.

## 584 — G_CancelGuildInvitationReqCmd — partial

- **381:** `0x1403B31B0`. As 571, reply 586, op 6.

## 588 — G_RejectGuildInvitationReqCmd — partial

- **381:** `0x1403B3610`. As 567 without the guild check, reply 590, op 7.

## 592 — G_AcceptGuildInvitationReqCmd — partial

- **381:** `0x1403B3A40`. As 567, reply 594, op 8.

## 596 — G_UpdateGuildApplicationRefusalReqCmd — done

- **381:** `0x1403B3E90`. **Wire:** `hq` (value; 1 = refuse).
- **Checks/order (reply 598):** J → 2. Arena B → 5. Decode fails → 2. No guild → 2. Lookup fails → 2. Row 18 off → 2. `(q == 1)` = guild byte +32784 → 2 ("already updated"). Zone null → 2. PC kind (+46) ≠ 1 → 2.
- **Output:** Task `new(0xE8)`, +68 = 326, vtable `off_140BB6DA8`: +144 guild ID, +152 DB ID, +160 PC name (PC +194, 33 units), +226 byte `q`. It queues. No reply.

## 600 — G_UpdateGuildInvitationRefusalReqCmd — done

- **381:** `0x1403B4410`. **Wire:** `hq`.
- **Checks/order (reply 602):** J → 2. Arena B → 5. Decode fails → 2. `(PC +890 & 2 ≠ 0)` = `(q == 1)` → 2 ("already set"). Zone null → 2. PC kind ≠ 1 → 2.
- **Output:** Task `new(0xA0)`, +68 = 327, vtable `off_140C408C0`: +144 DB ID, +152 byte `q`. It queues. No reply.

## Status

| Id | State |
| --- | --- |
| 431 | partial |
| 443 | done |
| 445 | done |
| 483 | done |
| 486 | done |
| 489 | partial |
| 447 | partial |
| 449 | done |
| 451 | done |
| 453 | done |
| 455 | partial |
| 457 | done |
| 459 | partial |
| 461 | partial |
| 463 | done |
| 465 | partial |
| 467 | partial |
| 469 | partial |
| 471 | partial |
| 473 | partial |
| 475 | done |
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
| 557 | partial |
| 560 | partial |
| 563 | partial |
| 567 | partial |
| 571 | partial |
| 575 | partial |
| 579 | partial |
| 583 | done |
| 584 | partial |
| 588 | partial |
| 592 | partial |
| 596 | done |
| 600 | done |
