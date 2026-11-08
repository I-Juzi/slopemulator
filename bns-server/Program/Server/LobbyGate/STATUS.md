# Goods handlers and final Phase 3 tests — Report

| Part | State | Test evidence |
| --- | --- | --- |
| D68 → C60 | done | Fake-daemon socket test: nested goods, blobs, NUL handling, fragments, missing client, C59→D67. |
| D71 → C63 | done | Socket test: preserved qword, atomic 29-byte rows, C62→D70. |
| D74 → C66 | done | Socket test: byte/dword header, atomic 29-byte rows, C65→D73. |
| D80 → C72 | done | Socket test: full identifier, atomic 43-byte rows, C71→D79. |
| Goods error edges | done | Shared socket test: signed empty counts, truncated input, trailing bytes, link survival. |
| Idle watcher sockets | done | Disabled/log-only stay open; forced close, partial-receive reset, and late-accept timing pass. |
| Stalled TCP receiver | done | LoginHub/TCP-writer test: stable full queue, FIFO, drop newest, fast peer delivery, recovery. |

## Spec changes
- Spec 05 §§8.6–8.10 records the goods formats, fragments, checks, crypt, and returns. Handler/sender sources: D68 `0x140011490`/`0x140004D20`; D71 `0x1400118C0`/`0x1400050F0`; D74 `0x140011C90`/`0x140005550`; D80 `0x140012050`/`0x1400059B0`.
- D68 string limits and blob handling: `0x140016510`, `0x1400168C0`, `0x140016370`, `0x14001DEB0`. Rust rejects malformed fields; it does not reproduce faults.
- Spec 01 §4 states the late-accept scan rule. Rust no longer consumes a stale startup tick. Sources: `0x140036950`, `0x140036B90`, `0x140018780`.
- Spec 04 already covers all four requests. No relay-map change was needed. Source: `0x140015B00`.

## Spec gaps
- ❓ Original broadcast queue capacity and overload policy remain unknown. Source address is not established. The bounded policy is a Rust policy in spec 05 §3.3.
- ❓ Exact timer-wheel phase remains open: `0x14001F080`, `0x140043B80`.
- ❓ Socket tests do not capture idle log counts. Watcher source: `0x140036950`. Original-client interoperability remains untested.

## Verification
- `cargo test --manifest-path bns-server/Cargo.toml`: **102 passed, 0 failed, 0 ignored**. All 93 baseline tests pass.
- `cargo clippy --manifest-path bns-server/Cargo.toml -p lobbygate-server --all-targets -- -D warnings`: **pass**.
- `cargo fmt --manifest-path bns-server/Cargo.toml --all`: **pass**. The optional repeat/check command did not run because it required approval.
- Initial compile/test issues were corrected: a borrow, new fragment-size arithmetic, and a TCP-ACK timing assumption. No existing test was weakened.
- IDA is closed. Emulator vectors were not edited. No built-in RSA key or licence logic was used.

## Next job
- Add direct idle-log assertions and goods limit/malformed-row tests.
- Extend the stalled-receiver test through the full Server/fake-daemon path.
- Verify original-client interoperability. Resolve the marked queue and timer gaps.

## Previous report

# Phase 3, step 5 — Report

| Part | State | Test evidence |
| --- | --- | --- |
| 1: detached state and close jobs | done | Detached-state unit test; D20/D21, duplicate/missing jobs, malformed input, and zero-timeout socket tests pass. |
| 2: D36 socket edges | done | Boundary, signed-count, malformed-input, embedded-NUL, and multi-fragment socket tests pass. The 68-byte reserve split is checked. |
| 3: lists and broadcasts | done | Hello/D4/C25/C104/announcement/reconnect socket test, FIFO snapshot socket test, and bounded-queue unit test pass. |
| 4: idle watcher sockets | not started | Existing paused-time unit tests pass. No new socket coverage. |

## Verification
- `cargo test --manifest-path bns-server/Cargo.toml`: **93 passed, 0 failed, 0 ignored**. All 85 baseline tests pass.
- `cargo clippy --manifest-path bns-server/Cargo.toml -p lobbygate-server --all-targets -- -D warnings`: **pass**.
- Rust formatting passes. No existing test was weakened. Emulator vectors remain unchanged.
- No IDA database was opened. No built-in RSA key, licence check, or goods handler was added.

## Spec changes and gaps
- Spec 05 §2.2 now states the Rust FIFO Hello snapshot policy. Each D2 event keeps its own body. Sources: Hello `0x14000AB70`; D4 `0x14000B679`.
- Spec 05 §3.3 now states the Rust broadcast policy: 32 queued deliveries per client; drop the newest announcement on a full queue. Other clients continue. Original queue capacity and overload behavior remain ❓; source address not established.
- Detached close behavior follows spec 05 §6.4. A job removes the gate key but keeps a client with no session. Source: `0x140006F60`.
- D36 malformed-input rejection remains an explicit Rust safety difference. Source: decoder wrapper `0x14001DE8A`–`0x14001DE9C`.
- ❓ TODO(spec 01 §4): disabled, log-only, and forced-close socket coverage remains open. Sources: `0x140013AB0`, `0x140036950`.
- ❓ TODO(spec 01 §4): exact original timer-wheel phase remains open. Sources: `0x14001F080`, `0x140043B80`.
- ❓ TODO(spec 05 §3.3): queue saturation is tested at the channel boundary, not with a stalled TCP receiver. Original queue source address is not established.

## Next job
- Add the three idle-watcher socket modes. Check receive reset and late-accept scan timing.
- Add a stalled-receiver TCP test for the bounded broadcast policy.
- Keep goods handlers 68/71/74/80 out of scope.

## Previous report

# Phase 3, step 4 — Report

| Part | State | Test evidence |
| --- | --- | --- |
| 1: idle watcher and timer cadence | partial | Four new paused-time tests pass. All baseline tests pass. |
| 2: detached state and close edges | not started | Existing D22 socket test still passes. No new D20/D21/zero-close test. |
| 3: D36 socket edges | not started | Existing character-list unit test still passes. |
| 4: Hello/lists/broadcast sockets | not started | Existing list unit test still passes. |
| 5: remaining config semantics | not started | TODOs remain. |

## Verification
- `cargo test --manifest-path bns-server/Cargo.toml`: **85 passed, 0 failed, 0 ignored**. All 81 baseline tests pass.
- `cargo clippy --manifest-path bns-server/Cargo.toml -p lobbygate-server --all-targets -- -D warnings`: **pass**.
- No existing test was changed. No emulator vector was changed.
- IDA is closed. No built-in RSA key, licence check, or goods handler was added.

## Spec changes and gaps
- Spec 01 §4 now states seconds, low-16-bit conversion, and 5000 ms scans. Sources: config setup `0x140013AB0`; watcher `0x140036B90`.
- Spec 01 §4 now states receive→watch→candidate→report. It corrects the claim of two full idle periods. Sources: scan `0x140036950`; accept `0x140018780`; receive `0x1400191A0`.
- Spec 01 §4 records the Rust deadline-heap policy and configured cadence. Sources: timer loop `0x140020AA0`; add `0x14001F080`; drain `0x140043B80`.
- ❓ TODO(spec 01 §4): exact wheel phase and short-delay wrap parity remain open. Sources: `0x14001F080`, `0x140043B80`. Rust rejects a zero timer interval.
- ❓ TODO(spec 01 §4): disabled/log-only/forced-close socket coverage remains open. Sources: `0x140013AB0`, `0x140036950`.
- ❓ TODO(spec 05 §6.4): detached close state remains unchanged. Source: close job `0x140006F60`.
- ❓ TODO(spec 05 §7.3): D36 boundary/signed/malformed/multi-fragment socket coverage remains open. Source: handler `0x140010F50`.
- ❓ TODO(spec 05 §§2, 4): reconnect snapshot order and bounded broadcast queue tests remain open. Hello source: `0x14000AB70`; original queue policy is not established.
- ❓ TODO(spec 01 §4): delayed release, alerts, cooldown, and GUID reload remain unimplemented. Alert setup: `0x140013AB0`; release timer call: `0x140020AA0`. Other source addresses were not established in this job.

## Next job
- Finish idle socket coverage and check late-accept scan phase.
- Complete parts 2–4 in order. State the bounded Rust broadcast policy in the spec.
- Then address part 5. Keep goods handlers 68/71/74/80 out of scope.

## Previous job record

# Phase 3, step 3 — Non-goods handlers, relay, and timers

| Part | State | Test evidence |
| --- | --- | --- |
| 1: Hello lists, C25/C104, D4, broadcast | done | Unit test checks world filtering, flags, empty lists, D4 replacement, and code E. |
| 2: C106/C107/C108 | done | Fixture test checks version 74, 16 lines, grace period, 30 s limit, and line bound. TCP test receives C107. |
| 3: close jobs, D25, D36 | partial | Token and character tests pass. TCP test checks D25 and C10 before close. Detached-session quirk needs work. |
| 4: relay sockets | done | Fake LobbyDaemon tests relay bytes in both directions on both login paths. |
| 5: timers | partial | Close deadline queue works in the kick test. Idle watcher and config-driven timer cadence remain TODO. |

## Verification
- `cargo test --manifest-path bns-server/Cargo.toml`: **81 passed, 0 failed, 0 ignored**. All 77 baseline tests pass.
- `cargo clippy --manifest-path bns-server/Cargo.toml -p lobbygate-server --all-targets -- -D warnings`: **pass**.
- No existing test was weakened. Login rejection checks remain on both paths. A third session tests kick and close.
- IDA is closed. No built-in RSA key, licence check, goods handler, or emulator-vector change was added.
- ❓ Original-client compatibility, broadcast socket delivery, D20/D21, timeout zero, and timer ordering lack direct tests.

## Spec changes and gaps
- Close timeout is **milliseconds**. Sources: `AddJobTimer` `0x140013490`, call `0x14001351B`, `TimerQueue::Add` `0x14001F080`, reconnect call `0x140018FB0`. Spec 01 §4 and spec 05 §6.4 now record this.
- Rust rejects malformed D36 fields instead of copying unsafe partial decode results. Spec 05 §7.3.4 records this safety policy. Source: wrapper `0x14001DE8A`–`0x14001DE9C`.
- ❓ Exact timer-wheel phase remains open. Rust uses exact deadlines, not the original wheel. Source: `TimerQueue::Add` `0x14001F080`.
- ❓ Idle units and first-period timing remain open. Source: spec 01 §4; no source address was established in this job.
- ❓ Detached client/session close parity remains incomplete. Source: close job `0x140006F60`, spec 05 §6.4.

## Next job
- Finish `TODO(spec 01 §4)`: two-period idle watcher and timer cadence tests.
- Finish `TODO(spec 05 §6.4)`: detached-session state and close-job edge tests.
- Add D36 boundary, signed-count, malformed-input, and multi-fragment socket tests.
- Add full Hello/D4/C25/C104 and announcement socket tests. Check reconnect snapshot ordering and slow-client broadcast queues.
- Keep goods handlers 68/71/74/80 out of scope.
