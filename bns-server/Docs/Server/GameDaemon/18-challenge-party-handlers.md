# 18 — Challenge-party handlers, build 381

## Scope and notation

Source: `ida/delegate/gamed/GameDaemon.exe.i64`, build 381.8867, without PDB.
Build-348 names come from `parity-ledger.tsv`.
The notation is the notation of doc 17.
`Q` and `c` are one-byte fields. `h`, `i` and `q` are 2-, 4- and 8-byte fields.
The first `h` is the command. Rust handlers receive body bytes only.
`hh` replies contain the command and a 16-bit reason.
`partial` lists unread paths with ❓. No live test ran.

Descriptor strings were read with `idc.get_strlit_contents`.
`0x14069F4B0` calls decoder `0x1406AA7B0`.
For `s`, the decoder reads a signed 16-bit unit count and that many UTF-16 units.
It rejects negative counts and counts that leave no space for the final NUL.
Create and invite pass capacity 33 (`0x21` in the call sites).
They consume all counted units. Name checks and lookup use the NUL-terminated prefix.
Task initializers copy the complete 66-byte name buffer.
A successful queue is not a successful operation.
The anti-bot helper `0x14060E280` is outside this work. It was not read.

Shared fields:

| Field | Source | Use |
| --- | --- | --- |
| Feature switch | Global `qword_140EE58A0`, byte +1982 | All five handlers. Clear → reason 2, except reject, which returns without a reply. |
| Arena form B | cfg (daemon +2736) byte +237 and DWORD +248, or PC DWORD +37520 state 2, 3, 5 or 6 | All five handlers. Reason 5, except reject, which logs only. Rust uses `World::arena` or `session::arena_blocked`. |
| Closed season | daemon +3192 object, DWORD +112 bit 12, under SRW lock +104 | Create, invite, confirm and leave. Set → reason 2. ❓ State producer and season transitions. |
| Name rules | daemon +2744 config; `0x14012F9F0(daemon +2800, name, cfg +1212/+1216/+1220/+1224/+1225/+1226/+1232)` | Create. Missing config or nonzero result → 2405. ❓ Rule implementation. |
| Text filter | daemon +2728, qword +48, virtual +0 | Create, only when the filter exists. False → 2405. ❓ Filter implementation. |
| User presence | PC qword +20696 present and byte +20752 = 0 | Create, only when the filter exists. Missing → reason 2. |
| PC lookup by name | `0x1403F7EF0(daemon +5824, name, 6)` | Invite. Rust uses `World::party.by_name`, with the existing `PartyState` rules. |
| Zone | PC qword +2224 | All tasks attach to the requester's zone. Missing → reason 2, except reject, which logs only. |
| Native kind | PC word +46 = 1 | Initializers. A different value writes a crash log and fails initialization. The Rust port logs and fails. |
| Common task fields | zone attachment `0x14025DB30`; +128 = PC object ID; retain PC with `0x14069BF60`; +138 = 1 | All five tasks. Queue through `0x140254FD0`. |

## 1068 — G_CreateChallengePartyReqCmd — partial

- **381:** `0x1403D2830`. **348:** `HandleGame_CreateChallengePartyReq`.
- **Wire:** `hs` at `0x1409077A8`: challenge-party name (capacity 33).
- **Checks/order:**
  1. Feature switch clear → 1070/2.
  2. Arena form B → 1070/5.
  3. Closed season → 1070/2.
  4. Decode failure → 1070/2404.
  5. Missing rule config or name-rule helper result ≠ 0 → 1070/2405.
  6. Filter exists: missing user → 1070/2; filter false → 1070/2405.
  7. No zone → 1070/2.
  8. Task kind 295, 0xD8 bytes, vtable `0x140BB3458`.
     Initializer `0x1402C9D50` requires native kind 1 and name first unit ≠ 0.
     It copies the name to +144..+209. Initialization failure → 1070/2.
  9. Queue failure → 1070/2. Queue success sends no entry reply.
- **Failure/output:** 1070, `hh` (`0x1408B8848`). The handler returns true on all read entry paths.
- **❓ Missing:** season state producer, name-rule helper, text-filter implementation and task body.

## 1071 — G_InviteChallengePartyReqCmd — partial

- **381:** `0x1403D2E80`. **348:** `HandleGame_InviteChallengePartyReq`.
- **Wire:** `hs` at `0x1409077A8`: target name (capacity 33).
- **Checks/order:**
  1. Feature switch clear → 1074/2.
  2. Arena form B → 1074/5.
  3. Closed season → 1074/2.
  4. Decode failure → 1074/2411.
  5. Name lookup miss → 1074/2413. The entry does not reject self or check the target zone.
  6. No requester zone → 1074/2.
  7. Task kind 296, 0xD8 bytes, vtable `0x140BA9750`.
     Inline initialization requires requester native kind 1 and name first unit ≠ 0.
     It copies the name to +144..+209. Initialization failure → 1074/2.
  8. Queue failure → 1074/2. Queue success sends no entry reply.
- **Failure/output:** 1074, `hh`. The handler returns true on all read entry paths.
- **❓ Missing:** season state producer and task body. The task receives the name, not the lookup result.

## 1075 — G_ConfirmInvitationChallengePartyReqCmd — partial

- **381:** `0x1403D3450`. **348:** `HandleGame_ConfirmInvitationChallengePartyReq`.
- **Wire:** `hq` at `0x1408D5024`: one 64-bit value. ❓ Exact invitation-ID meaning.
- **Checks/order:**
  1. Feature switch clear → 1077/2.
  2. Arena form B → 1077/5.
  3. Closed season → 1077/2.
  4. Decode failure → 1077/2421.
  5. No zone → 1077/2.
  6. Task kind 298, 0x98 bytes, vtable `0x140C414E8`.
     Initializer `0x1402CB300` requires native kind 1 and the wire value ≠ 0.
     It copies the value to +144. Initialization failure → 1077/2.
  7. Queue failure → 1077/2. Queue success sends no entry reply.
- **Failure/output:** 1077, `hh`. The handler returns true on all read entry paths.
- **❓ Missing:** value meaning, season state producer and task body.

## 1078 — G_RejectInvitationChallengePartyReqCmd — partial

- **381:** `0x1403D37F0`. **348:** `HandleGame_RejectInvitationChallengePartyReq`.
- **Wire:** `hqc` at `0x1407FBF3C`: one 64-bit value and one byte. ❓ Exact value meaning.
- **Checks/order:**
  1. Feature switch clear → return without a reply or log.
  2. Arena form B → log only. There is no season check.
  3. Decode failure → log only. The byte must decode, but the entry ignores its value.
  4. No zone → log only.
  5. Task kind 299, 0xA0 bytes, vtable `0x140C3FCB8`.
     Inline initialization requires native kind 1. It accepts a zero 64-bit value.
     It copies the 64-bit value to +144 and forces byte +152 to 0.
     Initialization failure → log only.
  6. Queue failure → log only. Queue success sends no entry reply.
- **Failure/output:** no failure packet. The handler returns true on all read entry paths.
- **❓ Missing:** value meaning and task body.

## 1080 — G_LeaveChallengePartyReqCmd — partial

- **381:** `0x1403D3B40`. **348:** `HandleGame_LeaveChallengePartyReq`.
- **Wire:** `hc` at `0x1408D9508`: one byte (initial value 0xFF). ❓ Option meaning.
- **Checks/order:**
  1. Feature switch clear → 1082/2.
  2. Arena form B → 1082/5.
  3. Closed season → 1082/2.
  4. Decode failure → 1082/2421.
  5. No zone → 1082/2.
  6. Task kind 301, 0xC0 bytes, vtable `0x140C40AF8`.
     Initializer `0x1402CC110` requires native kind 1.
     It copies the 48-byte input record to +144..+191.
     The wire byte goes to +144. Input qwords +8/+24/+32/+40 and short +16 stay zero.
     The entry does not check the byte range. Initialization failure → 1082/2.
  7. Queue failure → 1082/2. Queue success sends no entry reply.
- **Failure/output:** 1082, `hh`. The handler returns true on all read entry paths.
- **❓ Missing:** option meaning, record padding, season state producer and task body.
  Rust retains the byte and the known zero fields by contract. It does not model native padding.

## Rust port status

| Ids | Rust | State |
| --- | --- | --- |
| 1068 | `challenge.rs` | Entry checks and initializer ported; season and name checks use `Services::challenge_party`. |
| 1071 | `challenge.rs` | Entry checks and inline initializer ported; name lookup uses `World::party`. |
| 1075, 1078, 1080 | `challenge.rs` | Entry checks and initializers ported; field meanings remain partial. |
| All five | `skills::queue`, `SkillTaskKind::ChallengeParty(ChallengeTask)` | Tasks queue through `Services::queue_skill`; task bodies are not ported. |
| Opaque state | `ChallengeCheck`, `MemoryServices::challenge` | Gates default to `Gate::Unknown`. Unknown → `Outcome::unsupported("❓ ...")`. |

## Coverage and remaining work

All five client requests in this job have a section.
Commands 1070, 1074, 1077 and 1082 are entry failure replies.
The port does not claim task completion, challenge-party membership changes or persistence.
