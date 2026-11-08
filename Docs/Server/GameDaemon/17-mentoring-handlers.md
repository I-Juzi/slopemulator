# 17 — Mentoring handlers, build 381

## Scope and notation

Source: `ida/delegate/gamed/GameDaemon.exe`, build 381.8867, without PDB.
Build-348 names come from `parity-ledger.tsv`.
The notation is the notation of doc 16.
`c` and `Q` are one-byte fields. `h`, `i` and `q` are 16-, 32- and 64-bit fields.
`E` in a reply is the 16-bit reason.
`partial` lists unread paths with ❓. No live test ran.

Common task setup is the setup of doc 16.
A successful queue is not a successful operation.

Shared fields:

| Field | Source | Use |
| --- | --- | --- |
| Mentoring rule | DataManager `qword_140F77978` signed short +32896 > 0 | All handlers except 1103. |
| Mentoring record | DataManager qword +32888: record, short +8 > 0, byte +25 maximum mentee count | 1103. |
| Arena form B | cfg +237 and +248, or PC +37520 state 2, 3, 5 or 6 | All handlers (reason 5). |
| Target arena form C | cfg +237 ≠ 0 and +248 = 0, and target PC +37520 state 2, 3, 5 or 6 | 1100 (2448), 1111 and 1115 (6). The same form as doc 14 `target_arena_blocked`. |
| Mentoring type | PC byte +36746 | 1098, 1103. |
| PC lookup by name | `0x1403F7EF0(daemon +5824, name, 6)` | 1100. Use the nonempty prefix before the first NUL. Require target rights word +20672 & 6 ≠ 0. |
| PC lookup by object ID | `0x1403F7D20(daemon +5824, q, 6)` | 1103, 1111, 1115. Require `q >> 48 == 1`. Resolve the configured slot. Require target rights word +20672 & 6 ≠ 0. |
| Native kind | PC word +46 = 1 | A different value writes a crash log and fails the queue path. |
| Mentoring registry | daemon +6200, SRW lock, `0x140165AD0` and `0x140164490` | 1103. ❓ |

### Shared PC lookup

Source: `0x1403F7D20` and `0x1403F7EF0`, build 381.8867.
The ID helper reads the configuration pointer at lookup object +152.
It computes `(id & config.qword_24) >> config.dword_40`.
It uses the low 32 bits as the slot index.
The index must be less than 20000.
The slot must contain a PC.
The helper does not compare the full wire ID with the PC object ID.
The rights check runs before the later target checks in each handler.
The name helper uses a direct name entry and the same rights check.
It does not apply the ID type or slot checks to the name entry.
The decoder consumes all counted name units before lookup.

The Rust model uses `MentoringWorld::by_id` and `by_name` for these checks.
The `players` map keeps one current object ID per occupied slot.
The `names` map uses NUL-prefix keys.
`Pc::rights` models word +20672.
Its model default is 6, so existing fixtures pass mask 6.
The slot model defaults are mask `0xffff` and shift 0.
These are fixture defaults, not verified native configuration values.
The adapter must set `slot_mask` and `slot_shift` from the native configuration.
❓ The runtime configuration values remain unverified.

## 1098 — G_ChangeMentoringTypeReqCmd — partial

- **381:** `0x1403D4770`. **348:** `HandleGame_ChangeMentoringTypeReq`.
- **Wire:** `hQ`: type (initial value 0xFF).
- **Checks/order:**
  1. Mentoring rule clear → 1099/2436.
  2. Arena form B → 1099/5.
  3. Decode failure → 1099/2.
  4. Type > 2 → 1099/2435.
  5. Type = PC mentoring type → 1099/2434.
  6. No zone → 1099/2.
  7. Task kind 304, 0x98 bytes, vtable `0x140B9A840`, type at +144. Queue failure → 1099/2.
- **Failure/output:** 1099, `hh`.
- **❓ Missing:** the task body.

## 1100 — G_InviteMentoringReqCmd — partial

- **381:** `0x1403D4B10`. **348:** `HandleGame_InviteMentoringReq`.
- **Wire:** `hQs`: type, target name (capacity 33).
- **Checks/order:**
  1. Mentoring rule clear → 1102/2446.
  2. Arena form B → 1102/5.
  3. Decode failure → 1102/2.
  4. Type not 1 or 2 → 1102/3.
  5. Name lookup miss or target rights & 6 = 0 → 1102/2447. Use the nonempty NUL prefix. Target object ID or DB ID equal to the PC → 1102/2447.
  6. Target arena form C → 1102/2448.
  7. Target has no zone → 1102/2449.
  8. PC has no zone → 1102/2.
  9. Task kind 305, 0xA8 bytes, vtable `0x140C39BB0`, attached to the target zone and target object ID.
     Fields: +144 PC DB ID, +152 PC object ID, +160 type.
     Target native kind ≠ 1 → 1102/2.
     Queue on the PC zone fails → 1102/2.
- **Failure/output:** 1102, `hh`.
- **❓ Missing:** the task body.

## 1103 — G_ConfirmInvitationMentoringReqCmd — partial

- **381:** `0x1403D51C0`. **348:** `HandleGame_ConfirmInvitationMentoringReq`.
- **Wire:** `hqq`: inviter object ID, inviter DB ID.
- **Checks/order:**
  1. No mentoring record, or record short +8 ≤ 0 → 1104/2454.
  2. Arena form B → 1104/5.
  3. Decode failure → 1104/2.
  4. Object-ID lookup miss, rights failure, DB ID mismatch, or inviter equal to the PC (object ID or DB ID) → 1104/2. Use the shared slot lookup.
  5. PC mentoring type 2: mentor = PC, mentee = inviter. Type 1: mentor = inviter, mentee = PC. Other → 1104/2.
  6. Copy the task payload before the registry checks. Sign-extend each job byte (+100) to a signed short. Copy each name (+194) with `wcsncpy`, count 33. Pad after a NUL with zero units. A full 33-unit name has no added NUL.
  7. Registry: the mentee already has a mentoring → 1104/2452. ❓ exact key.
  8. Registry: the mentor count ≥ record byte +25 → 1104/2453. ❓ exact key.
  9. Mentee has no zone → 1104/2.
  10. Task kind 309, 0x148 bytes, vtable `0x140C413F8`. Attach and queue it on the mentee zone. Set task owner +128 to the actual mentee object ID, not the requester ID or a wire alias.
      Mentee native kind ≠ 1 or inviter DB ID 0 → 1104/2. Queue failure → 1104/2.

  | Task offset | Captured value |
  | --- | --- |
  | +144 | Requester PC object ID |
  | +152 | Original wire inviter object ID |
  | +160 | Wire inviter DB ID |
  | +168 | Mentor DB ID |
  | +176 | Mentee DB ID |
  | +184 | Maximum mentee count, signed byte |
  | +186, +188 | Mentor job, mentee job, signed shorts |
  | +190, +256 | Mentor name, mentee name, 33 UTF-16 units each |
- **Failure/output:** 1104, `hh`.
- **❓ Missing:** the registry lookups and the task body.

## 1105 — G_RejectInvitationMentoringReqCmd — partial

- **381:** `0x1403D5AE0`. **348:** `HandleGame_RejectInvitationMentoringReq`.
- **Wire:** `hqq`: two 64-bit IDs (task +144, +152).
- **Checks/order:**
  1. Mentoring rule clear → log only.
  2. Arena form B → **1104**/5 (the confirm failure command).
  3. Decode failure → log only.
  4. No zone → log only.
  5. Task kind 307, 0xA8 bytes, vtable `0x140BC1930`, +160 = 0. Native kind ≠ 1 → log only. Queue failure → log only.
- **❓ Missing:** the task body.

## 1107 — G_LeaveMentoringReqCmd — partial

- **381:** `0x1403D5ED0`. **348:** `HandleGame_LeaveMentoringReq`.
- **Wire:** `hQcq` through a descriptor object (`0x14069F4B0`). ❓ field names.
- **Checks/order:**
  1. Mentoring rule clear → 1109/2455.
  2. Arena form B → 1109/5.
  3. Decode failure → 1109/3.
  4. No zone → 1109/2.
  5. Task kind 308 (`0x14028DF50`), 0xB0 bytes, vtable `0x140BC13A0`. Initializer vfunc +88 false → 1109/2. Queue failure → 1109/2.
- **Failure/output:** 1109, `hh`.
- **❓ Missing:** the field mapping, the initializer and the task body.

## 1111 — G_CallMentoringMemberReqCmd — partial

- **381:** `0x1403D6280`. **348:** `HandleGame_CallMentoringMemberReq`.
- **Wire:** `hqh`: target object ID, one short.
- **Checks/order:**
  1. Decode failure → 1113/3.
  2. Mentoring rule clear → 1113/2455.
  3. Arena form B → 1113/5.
  4. Target 0 or shared slot lookup failure (type, bound, empty slot or rights) → 1113/2458.
  5. Target arena form C → 1113/6.
  6. No zone → 1113/2.
  7. Task kind 310, 0xA0 bytes, vtable `0x140BC29E8`, target +144, short +152 = 0. Native kind ≠ 1 → 1113/2. Queue failure → 1113/2.
- **Failure/output:** 1113, `hEqh` (reason, target, 0).
- **❓ Missing:** the task body.

## 1115 — G_WarpToMentoringMemberReqCmd — partial

- **381:** `0x1403D6880`. **348:** `HandleGame_WarpToMentoringMemberReq`.
- **Wire:** `hqh`: target object ID, one short.
- **Checks/order:**
  1. Decode failure → 1117/3.
  2. Mentoring rule clear → 1117/2455.
  3. Arena form B → 1117/5.
  4. PC byte +21096 or byte +13752 set ("taking achievement") → 1117/2.
  5. PC byte +21080 set ("try instant party") → 1117/2.
  6. Target 0 or shared slot lookup failure (type, bound, empty slot or rights) → 1117/2458.
  7. Target arena form C → 1117/6.
  8. No zone → 1117/2.
  9. Task kind 311, 0xA0 bytes, vtable `0x140B98E58`, target +144, short +152 = 0. Native kind ≠ 1 → 1117/2. Queue failure → 1117/2.
- **Failure/output:** 1117, `hEqh` (reason, target, 0).
- **❓ Missing:** the task body.

## Rust port status

| Ids | Rust | State |
| --- | --- | --- |
| 1098, 1100, 1105, 1111, 1115 | `mentoring.rs` | gates ported; tasks queued through `Services::queue_skill` |
| 1103 | `mentoring.rs` | gates ported; registry through `Services::mentoring_registry` |
| 1107 | `mentoring.rs` | gates ported; initializer → unsupported |

## Coverage and remaining work

All seven client requests of the mentoring group have a section.
Notify and fail commands (1099, 1101, 1102, 1104, 1106, 1108–1110, 1112–1114, 1116, 1117) are server replies.
