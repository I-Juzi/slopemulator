# 10 — Timer bindings and retained refresh, step 7

Build: 0.0.307. This part is partial. No runtime refresh was added.
The local NP service remains deterministic and has no transport.

## World-kick timeout binding

| Source | Checked fact |
| --- | --- |
| `0x140086D50` | GameDaemon Hello passes config +248 as signed i32 to `0x14007A110`. |
| `0x14007A110` | The constructor stores that argument at transaction-channel +40. A positive value registers its manager with lobby tick through `0x1400591A0`. A nonpositive value skips that registration. Registration failure rejects initialization. |
| `0x14007A250` | Earlier checked transaction expiry uses monotonic milliseconds plus signed channel +40. |
| `0x140085B00` | World kick sets transaction context with expiry event 116 through `0x14007A780`. It then sends GameDaemon kick through `0x140080420`. |

This closes the code-field binding for the GameDaemon channel lifetime.
❓ XML descriptor spelling and numeric default remain open.
Do not replace the five-second local world deadline with an unverified value.
The parallel GameDaemon spec also identifies the Hello field at config +248.
Its two files were not edited.

## NP and INIT registration limits

`0x140001FE0` allocates a nonzero transaction ID and records command/context.
`0x140002C30` calls it for tracked request messages before serialization.
Neither inspected function establishes a numeric request deadline.
`0x14003CA50` adds a session to its keyed table and increments the count.
That function does not establish INIT expiry registration.

❓ NP request and INIT timer registration remain unresolved.
The known 30000 NP value remains a keepalive interval, not a request timeout.
Token-desk config +252 and optional IP check +264 retain the earlier evidence.
❓ Their XML descriptor bindings and numeric lifetimes remain open.

## Correction: retained state 1 refreshes grade level

The earlier `04-login.md` table called this a charge refresh. That label was wrong.
`0x1400756D0` sends NP command 1722. Its reply worker `0x140075A00`
calls `0x140051C30` in GradeService. No charge response is claimed here.
The corrected table now names a grade-level refresh.

| Source | Checked rule |
| --- | --- |
| `0x140075500` | Require session `0x50002`. Config +414 selects state 1. Otherwise +413 and +432 together select state 2. Otherwise select state 3. |
| `0x140095510` | Validation identifies +413 as use-grade and +414 as use-grade-level. Grade level requires grade. Exactly one of +416/+420 must be positive and the other must be zero. |
| `0x1400756D0` | If +416 is positive, put it in request integer 0 and clear integers 1/2. Otherwise positive +420 goes in integer 2 and integers 0/1 are zero. If neither applies, fail. Request context stores expiry event 79. |
| `0x140075A00` | Accept status 200, or 400 with case-insensitive Success. Missing response or other status sends D17/2 and closes. Success writes grade level, then selects state 2 when +432 is set, else state 3. |
| `0x140051C30` | Copy five i32 values from response +132/+136/+140/+144/+148 to cache +4/+8/+12/+16/+20. Increment cache generation at +0. Set level-ready byte +56. Log labels identify the first two values as level and score. ❓ The last three meanings remain open. |
| `0x1400762F0` | Grade-list success uses the same status rule. Call `0x140051CE0`, then select state 3. Failure sends D17/2 and closes. |
| `0x140051CE0`, `0x140051AA0` | Clear the prior grade map before adding new rows. Response count is +96; row pointer is +104; stride is 160 bytes. Parse the text key at row +4 as a positive integer. Skip nonpositive keys. Copy row +0/+106/+108/+112/+120 into the local grade record. Increment generation and set list-ready byte +57. |

The validation log names `grade-level-group-id` and `game-grade-key-for-get-level`.
❓ The inspected log does not prove which descriptor binds +416 versus +420.
The sample option `enable-reload-grade-while-return-to-lobby` exists in the prior
names-only inventory. ❓ Its descriptor-to-+432 binding remains unverified.
No config value was read or copied in this pass.

## Step 8 — registration path limits

Build: 0.0.307. No recovered duration replaces a local deadline.

| Checked source | Result |
| --- | --- |
| `0x140001FE0` | Allocate a nonzero NP transaction ID. Store command and context in a keyed map under a lock. The checked body contains no deadline or timer registration. |
| `0x1400CE9B0` → `0x1400CBA00` → `0x140002C30` | Consume-token worker stores reply/expiry context, then registers the tracked request before serialization. Expiry event 35 is not a duration. |
| `0x1400CDA20` → `0x1400BD700` | Expiry callback resumes the saved job with saved state/event. This proves callback routing, not when its timer is registered. |
| `0x140094D50` → `0x140034480` → `0x140038C90` → `0x14003CE40` | INIT installs anonymous gate presence. The final helper inserts a gate-key/owner index under a lock and retains the owner. No timer registration appears in these checked bodies. |
| `0x14003ACD0` | Init manager registers the job through `0x1400BD250`. ❓ Timer effects in the general job framework remain open. |
| `0x1400CB380` | NP channel connect uses a five-second connect timeout. This is not evidence for a per-request deadline. No endpoint value is recorded. |

❓ NP request/INIT registration and numeric lifetimes remain open.
The option-string scan did not establish descriptor bindings. It is not proof
that descriptors are absent. XML names in `01-config.md` remain proposals unless
already checked there. No config values were read in this job.

Grade fields retain the explicit raw contract above: five i32 level-cache values,
level/score labels for the first two, three unknown meanings, and the list-record
source offsets. ❓ Complete grade names, signedness, date units and D16 field
selection remain open. Do not enable retained refresh from this partial contract.

## Runtime boundary and next tests

Optional retained refresh remains disabled under local R13.
No unknown platform response becomes success. D15 still uses the retained cache.
A future implementation must first specify the grade record fields and D16 output.
It must use generated fixtures for success, rejection, missing response, timeout,
cancellation, replacement of prior grades and an empty successful grade list.
It must preserve the no-DB-reload rule for retained start.

The existing retained-session tests still pass. They do not prove a refresh path.
No new timer or refresh runtime test is claimed.
