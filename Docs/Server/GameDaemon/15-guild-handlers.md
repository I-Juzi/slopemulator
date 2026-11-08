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
