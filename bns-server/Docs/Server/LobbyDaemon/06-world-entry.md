# 06 — World-entry research, build 0.0.307

This spec is partial. No real GameDaemon or NP call is permitted in the local server.
❓ Do not implement the unresolved state branches as unconditional success.

## Job and wire boundary

The source path identifies `TryEnterGameWorldJob.cpp` in the checked workers.
Constructor: `0x14007A8C0`. Vtable: `0x140177E30`.
Dispatcher: `0x14007AB10`. Entry event: 13. Job states: 0..8; 8 is terminal in the checked completion workers.

| Direction | Command and format | Evidence |
| --- | --- | --- |
| Gate → LobbyDaemon | D24 `hqo`: cliKey, character-slot GUID | `0x1400BFFE0`, `0x14007AC90` |
| LobbyDaemon → Gate | D25 `hqqh`: cliKey, token key, trailing short | `0x1400B8C60`; ❓ trailing short source/meaning not verified |
| LobbyDaemon → Gate | D26 `hqE`: cliKey, reason | `0x1400B8B90` |
| LobbyDaemon → Gate | D27 `hqiiCch;`: wait data | `0x1400B8D40`; cliKey is written after command 27; ❓ remaining field semantics remain open |
| LobbyDaemon → GameDaemon | Command 26 `hqq`: transaction ID, PCID | `0x140080150` |
| GameDaemon → LobbyDaemon | Command 27 `hq`: transaction ID | handler `0x140089CC0`, table in 02-links.md |
| GameDaemon → LobbyDaemon | Command 28 `hqE`: transaction ID, reason | handler `0x140089E20`, table in 02-links.md |
| GameDaemon → LobbyDaemon | Command 5 `hqqqi`: world entry with token | handler `0x140087D40`; ❓ full consumption job remains deferred |

D24 sends reason 4 for a missing session and reason 6 for job-init failure. The inspected scheduling-failure branch logs without a reply (`0x1400BFFE0`).

## State rules

| Job state | Operation | Source |
| --- | --- | --- |
| 0 | Require session `0x20000`. Resolve character by its full slot GUID. Check playability, service conflict and world channel. Set session `0x30001`. | `0x14007AC90` |
| 1 | Restore the character through GameDaemon when the cached character state needs restoration. | request workers `0x14007B600` / `0x14007B7C0`; reply `0x14007BB40`; expiry `0x14007BF80` |
| 2 | NP status request. A platform result can block entry. | `0x14007C080`, `0x14007C3A0`, `0x14007C8A0` |
| 3 | NP request. ❓ Exact operation and payload remain open. | `0x14007CA40`, `0x14007CD60`, `0x14007D090` |
| 4 | Get NP slot. Verify slot ID and character-slot type. | `0x14007D230`, `0x14007D510`, `0x14007DB00` |
| 5 | Get NP charge info. Apply free/trial policy. | `0x14007DC90`, `0x14007DF70`, `0x14007EA00` |
| 6 | Attach a world-admission object. Immediate admission selects state 7. Queueing sets session `0x30002`, sends D26 reason 107 and D27, then ends the job. | `0x14007EB90` |
| 7 | Create a world token with session, world ID and PCID. Issue through the shared token desk. Set session `0x30003`. Send D25. | `0x14007EF80`, issuer `0x140093D20` |

State 0 uses cached character fields not fully represented by the current Rust `Character`. It accepts original character state 2 with additional flag checks; state 5 selects restoration. ❓ Do not interpret the Rust wire `state` field as this internal state.
The special session flag at +1970 can skip NP states and select admission after optional restoration. Otherwise config +692 selects state 2 or state 4 (`0x14007AC90`). ❓ The flag's complete transport meaning remains open.

## Checked failures

| Condition | D26 reason | Source |
| --- | --- | --- |
| Wrong session state | 5 | `0x14007AC90` |
| No matching character-slot GUID | 101 | `0x14007AC90` |
| Disabled character class 11 | 117 | `0x14007AC90` |
| Character flagged unavailable | 109 | `0x14007AC90` |
| Internal character state 3 | 110 | `0x14007AC90` |
| Other nonplayable character state/flags | 102 | `0x14007AC90` |
| Conflicting service, except permitted kind-2 appearance service | 9 | `0x14007AC90`; config +472 controls the exception |
| Missing world channel; restore send/expiry failure | 104 | `0x14007AC90`, `0x14007B600`–`0x14007BF80` |
| Status request send/expiry failure | 7 | `0x14007C080`, `0x14007C8A0` |
| Admission errors 7/9, 8, 10, 11, 12 | 103, 104, 105, 106, 108 | `0x14007EB90` |
| Other admission error | 2 | `0x14007EB90` |
| Admission queued | 107, then D27 | `0x14007EB90` |
| Token issuance failure | 2; release admission, restore session `0x20000` | `0x14007EF80` |

Admission failure releases the new object and restores session `0x20000`. Token expiry uses the shared desk lifetime. Startup passes config +252 to the desk (`0x140097EB0` → `0x140093B80`). ❓ XML descriptor, numeric default and complete world-token consumption are not verified.

## DB and local implementation boundary

The checked state-0, admission and token workers use cached session data. No direct DB wrapper appears in those three workers. Restoration uses GameDaemon. ❓ Nested DB effects and all NP error maps remain open.

The next implementation must define a `GameDaemon` trait with deterministic restore, admission and release results. It must test failure, expiry, queue and token cleanup. A fixture must explicitly select platform and charge results. It must not bypass unresolved platform checks by returning a token for every request.

## Step 6 — checked platform, restoration and ownership rules

All addresses below are from build 0.0.307. No guide database was needed.

| Source | Checked rule |
| --- | --- |
| `0x14007C080`, `0x14007C3A0` | State 2 sends NP command 1711 with config +696. Success means status 200, or 400 with case-insensitive `Success`. Response +128 bit 0 enables character lock. If absent, select state 4. If present and response +96 is not 1, select state 3. Value 1 rejects with D26 reason 114. Error 3333 selects state 4. Other errors reject with reason 2. |
| `0x14007CA40`, `0x14007CD60` | State 3 sends NP command 2711 with config +696 and a zero field. Success selects state 4. Errors 3333, 3334, 3345, 3346 map to reasons 113, 114, 115, 116. Other platform errors map to 1. Null reply maps to 2. Send/expiry failure maps to 7. ❓ Exact platform field names remain unknown. Do not implement a password algorithm. |
| `0x14007D230`, `0x1400CBBB0`, `0x14007D510` | NP command 1502 takes account GUID at session +216 and the full selected slot GUID. Require successful status, exact returned GUID and type string `char`. Failure maps to 2. Config +400 selects charge state 5; otherwise select admission state 6. |
| `0x14007DC90`, `0x1400CC670`, `0x14007DF70` | NP command 2901 takes account GUID and an IPv4 text conversion. No sample value is needed. Successful nonzero reply +112 selects paid mode 1. With no charge, config +404 value 0 rejects with 111. Value 1 permits trial mode 2 only when no cached character level exceeds config +408; otherwise reject with 111. Value 2 permits free mode 3, except config +412 and cached session +2096/+2104 can reject with 112. ❓ Those cached fields lack verified semantic names. Other +404 values reach admission with mode 0; the local adapter does not assume they are valid. |
| `0x14007BB40`, `0x140031310` | Restore completion removes its transaction. Accepted restore status is 35. Status 33/34 maps to D26 reason 110; other rejected status maps to 1. Find cached PCID again. Nested restore helper requires internal state 5, calls DB wrapper `0x1400A1470`, then changes cached internal state to 2. DB failure maps to D26 reason 2. This is not a character-list reload. |
| `0x140034710`, `0x140034960` | Session +2000 owns admission after successful attach. Replace releases and destroys the prior object. Failed attach leaves the new object with the caller. Release calls the admission detach method, destroys it, and clears +2000. Queueing and D25 do not release admission. Session close and token-issue failure release it. |
| `0x14007EB90`, `0x14007EF80` | Admission carries world, PCID and charge mode. Queued admission has a nonnegative queue marker; immediate admission has marker -1. Queueing emits D26/107 before D27. Issuance creates a kind-1 token carrying session, world and PCID. ❓ D25 trailing short and complete D27 field meanings remain unknown. |
| `0x140087D40`, `0x140093EB0` | Game command 5 decodes transaction, token, cliKey, IPv4. Verify kind 1. Successful verification consumes the token before checking channel world. Missing token gives Game failure 16; wrong fields give 15. Wrong channel world gives 17 and invokes the consumed token failure hook. Correct world starts the enter job with token PCID. Job-init failure gives 4; scheduling failure gives 2; both invoke the failure hook. ❓ Later enter-job effects and exact failure-hook behavior remain deferred to the GameDaemon link work. |

The local Rust boundary can now implement D24 without real platform or world transport.
Require explicit generated character eligibility and NP approval fixtures. Keep them separate from the character-list wire state.
An unknown fixture must reject. A fixture approval is a test input, not proof that a production platform check passed.
Use cancellation-safe asynchronous restore and admission calls. Attach admission only after the last pending await.
Store a single-use local world token with explicit positive lifetime. Keep admission until consume, close, kick or expiry.
The local consumption control checks cliKey, IPv4 and world. It is not a Game command 5 listener.
❓ Full production eligibility flags, platform field names, restore DB bindings, queue promotion and post-consumption world entry remain open.
