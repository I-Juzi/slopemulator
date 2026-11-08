# 16 — Chat, talk-session, petition and QQ handlers, build 381

## Scope and notation

Source: `ida/delegate/gamed/GameDaemon.exe`, build 381.8867, without PDB.
Build-348 names come from `parity-ledger.tsv`.
The notation is the notation of doc 13.
A full format includes command `h`. It excludes the outer size.
`c`, `h`, `i` and `q` are 8-, 16-, 32- and 64-bit scalar fields.
`Q` is a one-byte scalar. `s` is a UTF-16 string with a capacity.
`E` in a reply is the 16-bit reason, as in doc 13 command 1497.
`done` means all game-logic paths of the wrapper were read.
`partial` lists unread paths with ❓.
No live test ran.

Common task setup is the setup of doc 11 and doc 13:
allocate 0x98 bytes, initialize with `0x14025D9F0`, set the kind word at +136,
attach the zone with `0x14025DB30`, store the PC object ID at +128,
attach the child with `0x14069BF60`, set byte +138, then queue with `0x140254FD0`.
A successful queue is not a successful operation.

Shared fields:

| Field | Source | Use |
| --- | --- | --- |
| Chat state | PC DWORD +12560 | 1 = normal. Other values block chat. |
| Chat rule | rule config (daemon +2744) DWORD +216 ≠ 0 | 51 passes without chat state 1. |
| Item message rule | rule config byte +1021 ≠ 0 | 55 runs only when set. |
| Arena config | cfg +237 ≠ 0 and DWORD +248 ≠ 0 | 55, 1557, 1560, 1563 → reason 5. |
| Text filter service | daemon +2728, qword +48 ≠ 0 | 51, 55 call the filter instead of the chat routine. |
| User | PC +20696 when byte +20752 = 0 | Filter and petition paths. |
| Trial jumping | PC byte +13740 = 1 | 1185 → reason 2. |

## Handler summary

| Id | Name | Wire | Output | Status |
| --- | --- | --- | --- | --- |
| 50 | G_GodSayCmd | `h` | log only | done |
| 51 | G_MessageReqCmd | `hcsiss` | 54 `hh` | partial |
| 55 | G_MessageByItemReqCmd | `hsch` | 57 `hh` | partial |
| 58 | G_MessageByDiceReqCmd | `hhh` | 60 `hh` | partial |
| 61 | G_TalkSessionCloseReqCmd | `hq` | none | partial |
| 1216 | G_TalkSessionOpenReqCmd | `hq` | 1218 `hh` | partial |
| 68 | G_SetPcSocialReqCmd | `hh` | 70 `hh` | partial |
| 342 | G_JoinChattingChannelReqCmd | `hc` | 344 `hh` | partial |
| 345 | G_LeaveChattingChannelReqCmd | `h` | 347 `hh` | partial |
| 387 | G_ChattingServerLoginSignatureReqCmd | `hQ` | 388 / 389 | partial |
| 390 | G_CurrentPetitionReqCmd | `h` | backend 173 | partial |
| 393 | G_EndPetitionNotifyReqCmd | `h` | none | partial |
| 395 | G_SendPetitionChatNotifyReqCmd | `h` | log only | done |
| 1557 | G_SubmitPetitionReqCmd | `hiss` | 1559 `hEcc` | partial |
| 1560 | G_CancelPetitionReqCmd | `h` | 1562 `hEcc`, backend 154 | partial |
| 1563 | G_FeedbackPetitionReqCmd | `hccS…` | 1565 `hh` | partial |
| 1185 | G_BoardGachaTakeParticipationRewardReqCmd | `hh` | 1187 `hh` | partial |
| 844 … 893 | QQ group (15 commands) | – | as 887 | partial |

## 50 — G_GodSayCmd — done

- **381:** `0x140372280`. **348:** `HandleGame_GodSay`.
- **Wire:** `h` (`asc_1408CFBE8`).
- **Checks/order:** Decode. A decode failure logs "[pc], god say req; packet decompose fail" and returns false.
- **Failure/output:** On success log "pcId, GodSay(%d)" and return true. No reply, task or state change.

## 51 — G_MessageReqCmd — partial

- **381:** `0x14038A580`. **348:** `HandleGame_MessageReq`.
- **Wire:** `hcsiss`: type c, target name s (capacity 32), i, message s (capacity 101), extra s (capacity 1000).
  The decode result is ignored. The handler writes NUL at index 32 and index 1000.
- **Checks/order:**
  1. Decode (unchecked).
  2. Pass when the chat rule is set or chat state = 1. Otherwise reply 54/90 and log "slience state".
  3. When the text filter service exists: no user → 54/90 and log "cannot find user".
     Resolve the target name through daemon +5824 `0x1403F7EF0(…, 6)`. Call the service vfunc +8.
     A false result logs "[tencent_uic] fail checkTextFilter" only.
  4. Otherwise call the chat routine `0x1405F89E0(pc, type, to, i, msg, extra)`.
- **Failure/output:** 54, `hh`, reason 90.
- **❓ Missing:** the chat routine `0x1405F89E0` and the text filter service callback.

## 55 — G_MessageByItemReqCmd — partial

- **381:** `0x14038A8A0`. **348:** `HandleGame_MessageByItemReq`.
- **Wire:** `hsch`: message s (capacity 400), c, h.
- **Checks/order:**
  1. Item message rule clear → log only. No reply.
  2. Arena config → 57/5.
  3. Decode failure → 57/96.
  4. Chat state ≠ 1 → 57/93.
  5. Text filter service: no user → 57/93; filter false → 57/93.
  6. Otherwise call `0x1405F97B0(pc, msg, c, h)`.
- **Failure/output:** 57, `hh`.
- **❓ Missing:** `0x1405F97B0` (item consumption and broadcast) and the filter callback.

## 58 — G_MessageByDiceReqCmd — partial

- **381:** `0x14038AC00`. **348:** `HandleGame_MessageByDiceReq`.
- **Wire:** `hhh`: min (signed), max (signed).
- **Checks/order:**
  1. Decode failure → 60/2.
  2. Chat state ≠ 1 → 60/97.
  3. min < 0, max < 0, min > max or max > 10000 → 60/98.
  4. No zone (PC +2224) → 60/2.
  5. Task kind 268, vtable `0x140AEDEE8`, min at +144, max at +146. Queue failure → 60/2.
  There is no PC-kind check.
- **Failure/output:** 60, `hh`.
- **❓ Missing:** the dice task body (random roll and broadcast).

## 61 — G_TalkSessionCloseReqCmd — partial

- **381:** `0x140373210`. **348:** `HandleGame_TalkSessionCloseReq`.
- **Wire:** `hq`. The decode result is ignored. The field is not zero-initialized.
- **Checks/order:** No zone → nothing. With a zone: task kind 9, vtable `0x140BBA628`, q at +144. The queue result is ignored.
- **Failure/output:** none.
- **❓ Missing:** the task body.

## 1216 — G_TalkSessionOpenReqCmd — partial

- **381:** `0x140373080`. **348:** `HandleGame_TalkSessionOpenReq`.
- **Wire:** `hq`. The decode result is ignored.
- **Checks/order:** No zone → 1218/2 and log "cannot find zone". With a zone: task kind 8, vtable `0x140C382D0`, q at +144. Queue failure → 1218/2.
- **Failure/output:** 1218, `hh`, reason 2. Doc 07 lists the sends.
- **❓ Missing:** the task body.

## 68 — G_SetPcSocialReqCmd — partial

- **381:** `0x14038BC50`. **348:** `HandleGame_SetPcSocialReq`.
- **Wire:** `hh`: social (signed short). The decode result is ignored.
- **Checks/order:** No zone → 70/2 and log "cannot find zone". With a zone: task kind 19, vtable `0x140BC1B28`, social at +144. Queue failure → log "ExecuteSerializedJob Fail" and 70/2.
- **Failure/output:** 70, `hh`, reason 2 (r9 = rbp+2, rbp = 0).
- **❓ Missing:** the task body.

## 342 — G_JoinChattingChannelReqCmd — partial

- **381:** `0x1403962E0`. **348:** `HandleGame_JoinChattingChannelReq`.
- **Wire:** `hc`: channel. The decode result is ignored.
- **Checks/order:** Build a 0x88-byte manager task, mode 2, vtable `0x140D02470`: PC object ID at +120, DWORD +112 = 0, channel at +128. Submit to daemon +5824 `0x1403FB2D0`. Failure → 344/2.
- **❓ Missing:** the manager task body.

## 345 — G_LeaveChattingChannelReqCmd — partial

- **381:** `0x1403963C0`. **348:** `HandleGame_LeaveChattingChannelReq`.
- **Wire:** `h`. There is no decoder.
- **Checks/order:** The task of 342 with DWORD +112 = 1 and channel byte 0xFF. Failure → 347/2.
- **❓ Missing:** the manager task body.

## 387 — G_ChattingServerLoginSignatureReqCmd — partial

- **381:** `0x1403A1580`. **348:** `HandleGame_ChattingServerLoginSignatureReq`.
- **Wire:** `hQ`: server type (zero-initialized).
- **Checks/order:**
  1. Decode failure → log "WriteLatencyLogReq decompose failed", return true. No reply.
  2. Type 1: id = PC object ID. Type 2: id = u16(PC +40) | (short_96 << 32). Other → 389/2.
  3. `0x14033ED20(type)` null → 389/2 ("arena lobby disconnected").
  4. BlowFish key failure → 389/2.
  5. Build a 64-byte signature with `0x1403A0800` from cfg strings +328/+320, DWORD +312 and time32. Reply 388 (`hqqb` builder) through vtable +584.
- **❓ Missing:** the signature internals (a boundary) and the 388 body layout.

## 390 — G_CurrentPetitionReqCmd — partial

- **381:** `0x14039B200`. **348:** `HandleGame_CurrentPetitionReq`.
- **Checks/order:** Petition manager (daemon +4160) `0x140189FB0(mgr, short_96, "u1i8", 173, db_id)`. The result is ignored.
- **❓ Missing:** the petition daemon channel.

## 393 — G_EndPetitionNotifyReqCmd — partial

- **381:** `0x14039BD60`. **348:** `HandleGame_EndPetitionNotifyReq`.
- **Checks/order:** Look up the petition map (daemon +2832) by db_id under its critical section. Found: `0x14018CB90(mgr, db_id)` then `0x14018CA00(mgr, value)`. Not found: log only.
- **❓ Missing:** the petition map and both manager calls.

## 395 — G_SendPetitionChatNotifyReqCmd — done

- **381:** `0x14039BD30`.
- **Checks/order:** Log "[petition], HandleGame_FeedbackPetitionReq not supported". Return true. No reply.

## 1557 — G_SubmitPetitionReqCmd — partial

- **381:** `0x14039B240`. **348:** `HandleGame_SubmitPetitionReq`.
- **Wire:** `hiss`: i, s (capacity 801), s (capacity 801). The buffers are zeroed. The decode result is ignored.
- **Checks/order:**
  1. Arena config → 1559/5 and log "cannot process in arena".
  2. No user → log "user is null" and 1559/2.
  3. Petition channel `0x14018A1B0` missing → log "not connected" and 1559/2.
  4. Channel session state ≠ 5 or session id -1 → 1559/2.
  5. Create a petition transaction (0x110 bytes) and add it to the petition map (daemon +2832). Failure → 1559/2.
  6. Send backend request 169 (`u1i4i4s2gs2i8s…`) with the transaction id. No reply here.
- **Failure/output:** 1559, `hEcc` (reason, 0, 0).
- **❓ Missing:** the petition transaction, the map and the channel.

## 1560 — G_CancelPetitionReqCmd — partial

- **381:** `0x14039B6D0`. **348:** `HandleGame_CancelPetitionReq`.
- **Checks/order:** Arena config → 1562/5 and log. Otherwise backend request 154 (`u1s2i8s2i8`) with PC +194, to world short_96. The result is ignored.
- **❓ Missing:** the petition daemon channel.

## 1563 — G_FeedbackPetitionReqCmd — partial

- **381:** `0x14039B7F0`. **348:** `HandleGame_FeedbackPetitionReq`.
- **Wire:** a nested score list, then a string (forwarded raw).
- **Checks/order:**
  1. Arena config → 1565/5.
  2. Petition map lookup by db_id. Not found → log and 1565/2.
  3. Build forward packet (`hccS`, opcode 0xAA, petition id). Score count > 255 → log.
  4. Send it on the petition channel. Final reply 1565 with a computed reason.
- **❓ Missing:** the score list codec, the channel send and the final reason.

## 1185 — G_BoardGachaTakeParticipationRewardReqCmd — partial

- **381:** `0x1403DA810`. **348:** `HandleGame_BoardGachaTakeParticipationRewardReq`.
- **Wire:** `hh`: count (signed, zero-initialized).
- **Checks/order:**
  1. BoardGacha disabled (doc 12c gate) → 1187/2.
  2. No user or user DWORD +120 ≠ 0 (fatigue) → 1187/2.
  3. Trial jumping → 1187/2.
  4. Decode failure → 1187/2.
  5. count ≤ 0 → 1187/3.
  6. No user → log "user is null" only.
  7. User transaction (0x68 bytes, DWORD +8 = 0x4F210A, qword +96 = 0, ID from daemon +5128). Register with `0x1400D4FE0`; failure → 1187/2. Start with event 69 (0x45) and the count as payload (`0x1400D4A80`).
- **❓ Missing:** the transaction runner.

## 844 … 893 — QQ group — partial

- **381:** all 15 commands (844, 849, 852, 855, 857, 860, 863, 866, 869, 873, 877, 881, 884, 890, 893) share target `0x1400DE550` with 887 (doc 10).
- **Checks/order:** The same as 887: no QQ interface → native false; no user → nothing; otherwise the QQ interface (`0x140371430`).
- **❓ Missing:** the QQ interface identity and effects.

## Rust port status (part 1)

| Ids | Rust | State |
| --- | --- | --- |
| 50, 395 | `chat.rs` | done: log only |
| 51, 55 | `chat.rs` | gates ported; chat routine and filter → unsupported |
| 58, 61, 1216, 68 | `chat.rs` | queue through `Services::queue_skill` (`SkillTaskKind::Chat`) |
| 342, 345, 390, 1560 | `chat.rs` | `Services::chat` boundary |
| 387, 393, 1557, 1563 | `chat.rs` | gates ported; petition and signature internals → unsupported |
| 1185 | `chat.rs` | gates ported; transaction through `Services::board_gacha` |
| QQ group | `session.rs` `handle_qq_logout` | shared with 887 |

## Coverage and remaining work

Other social handlers (achievement, mentoring: 896, 898, 901, 904, 1041, 1098–1115) are not read yet.
