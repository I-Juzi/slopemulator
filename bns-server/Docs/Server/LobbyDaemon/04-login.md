# 04 — Login path, build 0.0.307

✅ Target facts have a 307 address. ❓ Marks a proposed name or an unchecked detail. The guide database was not opened in this pass. No platform request was sent.

## Command entry points

| Input | Checked local work | Failure and output | 307 source |
| --- | --- | --- | --- |
| ✅ D6 InitSessionWithNPToken | Decode gateKey, NP token, portal GUID, raw IPv4. Call session manager `0x14003ACD0`. It requires nonzero gateKey and both GUIDs. It allocates a session and a job. | ✅ Handler maps manager error 3 to reason 6; other errors to 2. D8 uses gateKey. ❓ Init job state steps and D7 timing remain. | ✅ `0x1400BE4C0`, `0x14003ACD0`, sender `0x1400B7680` |
| ✅ D9 StartSessionWithNPToken | Decode cliKey and two GUIDs. Manager looks up cliKey and starts the job constructed at `0x1400CDB70`. | ✅ Decode failure gives D11 reason 2. Manager error 3 maps to 6; error 5 maps to 4; others map to 2. | ✅ `0x1400BEAB0`, `0x14003B310` |
| ✅ D12 InitSessionWithLobbyToken | Decode gateKey, cliKey, token qword and raw IPv4. Verify token with type 2. Find retained session. Initialize and enqueue an init job. | ✅ D14 reason 19 for expired token; 20 for verification failure; 2 for missing retained session or job failure. Token verifier rules appear in the step-5 table below. ❓ Later reason 21 remains open. | ✅ `0x1400BEFC0`, verifier `0x140093EB0`, job init `0x140053440` |
| ✅ D15 StartSessionWithLobbyToken | Look up cliKey. Initialize a job and execute it with event 15. | ✅ D17 reason 4 if session is absent; 2 for init or start failure. Job states and the no-DB-reload finish worker appear in the step-5 table below. | ✅ `0x1400BF5A0`, job init `0x140075330` |
| ✅ D35 GetCharacterInfoList | Read the current session cache. Compose D36 with cliKey, representative PCID, world ID and character rows. No direct DB call in this handler. | ✅ D37 reason 4 for absent session; cache error 3 maps to 5; other cache errors map to 2. | ✅ `0x1400C0A90`, cache writer `0x140036330` |
| ✅ D24 GetEnterWorldToken | Find session. Construct and initialize enter-world job. Execute with event 13. | ✅ D26 reason 4 for absent session; 6 for job initialization failure. ❓ Job scheduling failure only logs in the inspected handler. | ✅ `0x1400BFFE0`, ctor `0x14007A8C0`, init `0x14007A9E0` |

✅ Full receive and reply literals are in `02-links.md`, section “LobbyGate link, full”.

## NP start job

❓ Proposed class name: `StartCliSessionWnpTokenJob`. The step 1 map names its presence response at `0x1400D2310`. ✅ Constructor `0x1400CDB70` installs vtable `0x1401D1740`. ✅ Dispatcher `0x1400CDD00` reads state at job +32. Its entry event is 8. State 0 calls `0x1400CE010`.

✅ The table lists dispatcher states, not a claim that every login visits every state. Event 0 starts each listed operation. Reply events and timeout events are separate. Source: `0x1400CDD00`.

| State | Operation / request worker | Reply worker | Expiry worker | NP boundary |
| --- | --- | --- | --- | --- |
| ✅ 1 | ✅ Consume game token `0x1400CE9B0`; send `0x1400CBA00` | ✅ `0x1400CECD0`, event 34 | ✅ `0x1400CF390`, event 35 | ✅ NP answer accepts or rejects token |
| ✅ 3 | ✅ List last presences `0x1400CF4B0`; send `0x1400CCB70` | ✅ `0x1400CF820`, event 72 | ✅ `0x1400CFF00`, event 73 | ✅ NP presence data selects activity path |
| ✅ 4 | ✅ List slots `0x1400D0020` | ✅ `0x1400D03D0`, event 39 | ✅ `0x1400D09D0`, event 40 | ✅ Existing activity slot selects set-data or create path |
| ✅ 5 | ✅ Create slots `0x1400D0AF0`; send `0x1400CBC30` | ✅ `0x1400D0FB0`, event 41 | ✅ `0x1400D15F0`, event 42 | ✅ NP result and slot ID decide continuation |
| ✅ 6 | ✅ Set slot data `0x1400D1710`; send `0x1400CBD40` | ✅ `0x1400D1AF0`, event 43 | ✅ `0x1400D1EF0`, event 44 | ✅ NP result decides continuation |
| ✅ 7 | ✅ Presence login `0x1400D2010`; send `0x1400CBE30` | ✅ `0x1400D2310`, event 45 | ✅ `0x1400D2CE0`, event 46 | ✅ NP answer decides login and later optional states |
| ✅ 8 | ✅ Get profile `0x1400D2E00`; send `0x1400CD4C0` | ✅ `0x1400D3170`, event 93 | ✅ `0x1400D3700`, event 94 | ✅ NP profile result decides continuation |
| ✅ 9 | ✅ Get game time `0x1400D3820`; send `0x1400CD540` | ✅ `0x1400D3BA0`, event 95 | ✅ `0x1400D4350`, event 96 | ✅ NP schedule data decides continuation |
| ✅ 10 | ❓ Semantic operation not resolved; `0x1400D4470` | ✅ `0x1400D47A0`, event 78 | ✅ `0x1400D4C60`, event 79 | ❓ Keep opaque; do not assume a safe default |
| ✅ 11 | ✅ List all grades `0x1400D4D80`; send `0x1400CCC00` | ✅ `0x1400D50F0`, event 76 | ✅ `0x1400D55E0`, event 77 | ✅ NP grade data feeds cache |
| ✅ 12 | ❓ Semantic operation not resolved; `0x1400D5700` | ✅ `0x1400D59E0`, event 87 | ✅ `0x1400D5EC0`, event 88 | ❓ Keep opaque |
| ✅ 13 | ✅ RMS inquiry `0x1400D5FE0`; send `0x1400CD3C0` | ✅ `0x1400D62D0`, event 89 | ✅ `0x1400D67A0`, event 90 | ✅ NP result feeds session data; ❓ full semantics |
| ✅ 14 | ✅ Promotion status `0x1400D68C0`; send `0x1400CD770` | ✅ `0x1400D6C40`, event 101 | ✅ `0x1400D7070`, event 102 | ✅ NP promotion result feeds cache |
| ✅ 15 | ✅ Finish login and load DB `0x1400D7190` | ✅ D10 sender `0x1400B78F0` | ❓ No separate expiry checked | ✅ DB failure prevents D10 |

✅ Consume-token reply checks platform status 200, or status 400 with case-insensitive `Success`. It requires non-null login-name and user-name fields. A rejection gives D11 reason 15. Missing required data gives reason 2. Success selects state 3 when config +700 is on, else state 7. Source: `0x1400CECD0`.

✅ Presence-login response checks platform status. Checked platform errors 3352 and 3716 select D11 reasons 18 and 16. ❓ Other platform error mappings remain. Source: `0x1400D2310`.

✅ Checked NP expiry workers above send D11 reason 2 and move to terminal state 16. Invalid INIT.verify session state gives reason 5 in checked reply workers. Sources: `0x1400CF390`, `0x1400D2CE0`, `0x1400D3170` and listed workers. ❓ Numeric timeout durations and timer units remain unchecked. Do not treat event IDs as durations.

✅ Finish worker calls login `0x140032C50`, handles duplicate-account kick scheduling, calls DB loader `0x1400336F0`, then sends D10. Login failure gives reason 16. Step 8 confirms that cache-loader failure also sends D11 reason 16 and closes the session. Source: `0x1400D7190`.

## Step 3 — init and kick checks

✅ Build 0.0.307. NP init vtable `0x14014BD30` selects dispatcher `0x140094D30`. State 0 accepts event 7 only. Worker `0x140094D50` requires a gate channel, nonzero gateKey and nonzero NP token. The manager also requires a nonzero portal GUID (`0x14003ACD0`). It sets session state `0x10001`, stores raw IPv4 and registers the session with `0x14003CA50`. Registration failure sends D8 reason 6. Success installs the gate binding, sends D7 and ends the job in state 1. D7 echoes both GUIDs. No NP token consumption occurs in this init worker. Token consumption belongs to D9.

✅ Retained-session init vtable `0x140159040` selects dispatcher `0x140053580`. Event 14 runs `0x1400535C0`. It requires session state `0x50001`; otherwise it sends D14 reason 5. Job state 1 with event 0 runs `0x140053720`. It releases the prior binding, tries the new gate binding with `0x140034480`, releases world admission with `0x140034960`, sets session state `0x50002`, sends D13 and ends at job state 2. Binding failure closes the session and sends D14 reason 2. ❓ Token lifetime and the reason-21 path remain open. Step 5 below adds a local same-link retained-session milestone.

✅ Kick constructor `0x140084D70` installs vtable `0x14017B588`. Dispatcher `0x140084E70` accepts entry event 1. State 0 runs `0x140084F50`; state 1/event 0 runs `0x1400850D0`; state 2 dispatches events 114/115/116 to `0x140085DD0`/`0x140085F10`/`0x140086070` and has worker `0x140085B00`. State 3/event 0 closes the session with `0x140032A10`. State 4/event 0 calls the completion object and ends at state 5. Management caller `0x140057610` constructs this job with reason 7. ❓ Full worker branches, D22 reason mapping and kick timeout duration remain open. The local milestone exposes an explicit kick operation; it does not claim full management/GameDaemon behavior.

✅ D23 sets the logout flag to 1 (`0x1400BFD80`). D18/D19 set it to 0. In lobby state `0x20000`, logout worker `0x1400948C0` closes the session. It sends D21 only when the flag is 1 and a gate channel exists. In INIT state `0x10001`, it closes without D21. L2W_ok `0x30003` absorbs logout and can send D21 when the flag is 1. ❓ Other state paths and presence cleanup remain incomplete.

### Timer boundary

✅ NP token and presence request workers set expiry event IDs 35 and 46, respectively (`0x1400CE9B0`, `0x1400D2010`). They are not durations. NP send wrapper `0x140002C30` registers a transaction through `0x140001FE0`. ❓ Numeric NP/init/kick deadlines, units and configuration defaults were not resolved within the seven-turn spec budget. Rust uses an explicit local `np_timeout` setting. Its default is a local policy, not a recovered server value. Tests must use paused Tokio time.

## Step 4 — checked kick effects and timer limits

Build: 0.0.307. These checks do not resolve all deadlines.

| Source | Checked effect |
| --- | --- |
| `0x140084F50` | Save the prior session state. Set FIN (`0x60000`). An already-FIN session completes with result 3. |
| `0x1400850D0` | Prior states `0x10002`, `0x10003`, `0x20000`, `0x30001`, `0x50003` kick the lobby presence. State `0x30002` also releases world admission. Result 1 then closes the session. |
| `0x1400850D0` | Prior states `0x10001`, `0x30003`, `0x50001`, `0x50002` close the presence, rather than kick it. Prior `0x40000` cancels the pending world object and enters world-kick state 2. Unknown states use result 2 and still close. |
| `0x140038DB0` / `0x140038DD0` | Anonymous close sends D20 with gateKey. Anonymous kick is forbidden. Thus an INIT kick must not emit D22. |
| `0x140085B00` | World kick requires world presence. Release the attached object. Resolve the world channel. If absent, use result 1 and close. Otherwise send GameDaemon command 14 through `0x140080420`, with expiry event 116. |
| `0x14007A250` | Transaction expiry is monotonic elapsed milliseconds plus signed channel field +40. ❓ Field initialization and config association remain open. |
| `0x140085DD0` / `0x140085F10` / `0x140086070` | World success removes the transaction and sets result 1. Failure also removes it; reason 24 gives result 1, other reasons give 2. Expiry gives result 2. All move to close state 3. |
| `0x140032A10` | Close releases the attached object and world admission. Remove and destroy the presence. Run account cleanup `0x1400334A0`. Remove the session-table entry. |
| `0x140093450` | Reason conversion preserves 1..12. All other values become 0. Checked use is the GameDaemon kick sender. The different D22 conversion appears in step 5 below. |
| `0x1400025D0`, `0x140001DE0`, `0x140003290` | NP global 30000 controls keepalive command 1002 and timer re-registration. It is not evidence of a request deadline. |
| `0x140001FE0`, `0x1400CDA20` | NP registration stores command/context. Expiry callback dispatches the saved job state and event. ❓ Timer registration and request duration remain open. |
| `0x140093D20` | Lobby-token expiry is elapsed `GetTickCount64` plus desk field +56, in milliseconds. Step 5 below binds desk initialization to config +252. ❓ Numeric lifetime and XML name remain open. |

❓ NP request, anonymous/init and numeric world-kick deadlines remain unverified.
No recovered duration replaces the local NP deadline in this part.
The guide was not opened. No database save was requested.

## Step 5 — login leftovers

Build: 0.0.307. These facts replace the earlier open D22 conversion question.

| Source | Checked rule |
| --- | --- |
| `0x140039380` → `0x1400933D0` → `0x1400B81E0` | D22 preserves reasons 1..8. Reasons 10, 11 and 12 become 9, 10 and 11. All other reasons become 0. The GameDaemon conversion at `0x140093450` is different. |
| `0x1400336F0` | Nonempty incomplete services are restored into the session cache. Kinds 1, 2, 3, 7 and 8 require detail loaders. Kind 6 needs only the common row. Unknown kinds fail. A load/setup/add failure blocks login. |
| `0x14006DC10`, `0x140035D20` | Kind 6 copies the common row and adds it to the session. This does not complete, consume or delete the service. |
| `0x140093B80`, `0x140093D20` | Token-desk constructor argument becomes signed field +56. Issuance adds this field to monotonic milliseconds. Startup `0x140097EB0` passes config +252 to this constructor. ❓ XML name and numeric default remain open. |
| `0x140093EB0`, `0x140093780` | Successful verification removes the token from both indexes. Verify kind, token key, cliKey and optionally IPv4 (config +264). Missing token sets the absent flag. Wrong fields do not consume it. |
| `0x140094140`, `0x140093A50` | Expiry sweep removes tokens at their deadline and calls their expiry hook. The checked hook schedules kick reason 8. |

❓ NP request and INIT timer registration, world-kick config binding, token-lifetime config binding and numeric defaults remain unresolved at the seven-turn cutoff. Keep local policies. Do not infer durations from event IDs.

Local recovery scope: restore kind 6 only. Reject other nonempty kinds explicitly until their detail payloads are specified. Keep pending rows in the session. Do not write service completion to the Store.

## Step 5 — retained-session login

Build: 0.0.307. D12/D15 keep the same cliKey and account cache.

| Source | Checked rule |
| --- | --- |
| `0x140086980` | World-return job issues a kind-2 lobby token, sets `0x50001`, and replies to GameDaemon. This is not a fresh NP login. ❓ Real world-return transport remains deferred. |
| `0x1400535C0`, `0x140053720` | D12 requires `0x50001`; wrong state gives D14 reason 5. It replaces the prior binding, releases admission, sets `0x50002`, and sends D13. Binding failure closes the session and sends reason 2. |
| `0x140075440`, `0x140075500` | D15 event 15 requires `0x50002`; wrong state gives D17 reason 5. Config +414 selects job state 1. Otherwise +413 and +432 select state 2; otherwise state 3. |
| `0x1400756D0`, `0x140075A00`, `0x140075EE0` | State 1 requests NP command 1722 for grade level, not charge data. Reply/expiry events: 78/79. Config +416/+420 select request fields. Success calls grade-level cache writer `0x140051C30` and selects state 2 if +432, else state 3. Failure/expiry sends D17 reason 2 and closes the session. Step 7 records raw grade fields in `10-timers-refresh.md`. |
| `0x140076000`, `0x1400762F0`, `0x1400767A0` | State 2 lists all NP grades. Reply/expiry events: 76/77; request context stores 75. Success refreshes cache and selects state 3. Failure/expiry sends D17 reason 2 and closes. |
| `0x1400768C0` | State 3 installs lobby presence and sets `0x20000`. Send D16 from cached charge/EXP data. No account, character or incomplete-service DB reload occurs in this worker. Optional friend notification uses NP command 2208. Failure sends D17 reason 2 and closes. |

Token verification and expiry rules are in the leftovers table. D12 reports absent tokens as reason 19 and wrong fields as reason 20 (`0x1400BEFC0`). Verification consumes a valid token before the state job runs. ❓ Reason 21 is not established. Do not implement a guessed branch.

Local R13: provide an explicit generated world-return control, not a network GameDaemon implementation. Require an explicit positive token lifetime. Keep the retained session on its existing gate link. D19 from the old client must not close `0x50001`; D23 can send D21 without destroying that retained session (`0x1400948C0`). Link loss still closes sessions under R9. D12 consumes one kind-2 token, rebinds the gateKey and enters retained-init. D15 returns the cached account data as D16. Optional NP charge/grade refresh remains disabled in this local milestone. Reject expired tokens at the exact deadline, even before a sweep. A sweep closes the retained session with kick reason 8. A direct Jobs D12 call can instead remove the expired session and return reason 19. The Server runs expiry before packet dispatch. D12 starts the opt-in INIT deadline when configured. ❓ Original retained-init timeout remains open.

## Enter-world job

Steps 5–6 extend this inventory in `06-world-entry.md`. The production spec remains partial. Step 6 adds a local D24 implementation with explicit fixtures and no GameDaemon transport.

✅ Constructor `0x14007A8C0` installs vtable `0x140177E30`. Dispatcher `0x14007AB10` uses entry event 13 and states 0..7.

| State | Checked operation | Source / reply / expiry |
| --- | --- | --- |
| ✅ 0 | Require lobby session; find selected character; check playability and service conflicts; acquire game channel | ✅ `0x14007AC90` |
| ✅ 1 | Restore character through GameDaemon when needed | ✅ `0x14007B600` / `0x14007B7C0`, `0x14007BB40` / `0x14007BF80` |
| ✅ 2 | Ask platform for status; platform response can block entry | ✅ `0x14007C080` / `0x14007C3A0` / `0x14007C8A0` |
| ✅ 3 | Platform request; ❓ exact operation unresolved | ✅ `0x14007CA40` / `0x14007CD60` / `0x14007D090` |
| ✅ 4 | Get slot; verify slot ID and character-slot type | ✅ `0x14007D230` / `0x14007D510` / `0x14007DB00` |
| ✅ 5 | Get charge info; platform result and free/trial policy decide entry | ✅ `0x14007DC90` / `0x14007DF70` / `0x14007EA00` |
| ✅ 6 | Admit or enqueue; send D27 wait data when queued | ✅ `0x14007EB90` |
| ✅ 7 | Issue token; send D25 `hqqh` | ✅ `0x14007EF80`; issuer `0x140093D20`; sender `0x1400B8C60` |

✅ Restore send/expiry failure gives D26 reason 104. Status request send/expiry failure gives reason 7. Invalid session state gives reason 5. Token issue failure gives reason 2. Sources: workers above. ❓ Full error maps, token lifetime, numeric timeout durations and nested DB effects remain.

## Close and kick

✅ D18 looks up by gate identity plus gateKey. D19 and D23 look up by cliKey. All create a logout job and execute event 9. D18/D19 set its logout flag to 0. ❓ D23 flag meaning needs a full check. Sources: `0x1400BF8B0`, `0x1400BFB20`, `0x1400BFD80`, constructor `0x140094680`.

✅ Logout dispatcher `0x140094740` calls init at `0x140094780`, then process at `0x1400948C0`. The process uses current session state. It can release world admission, close presence, close the session or absorb logout during L2W_ok. It can send D21. ❓ Exact session-state numbers and DB cleanup paths remain.

✅ D20 uses gateKey only; D21 uses cliKey only; D22 uses cliKey plus reason byte. Sources: `0x1400B8060`, `0x1400B8120`, `0x1400B81E0`. Step 4 checks the kick dispatcher and worker branches above. Step 5 checks the D22 reason conversion. ❓ Numeric deadlines and remaining config bindings remain open.

## Login DB dependency boundary

✅ `0x1400336F0` always loads characters with `0x14009E100` and incomplete services with `0x1400A5090`. Feature flag global +20 adds time-based play info (`0x14009CCF0`). Flag +22 adds EXP load (`0x1400A4300`) and inserts missing EXP (`0x1400A47A0`). Any checked wrapper failure stops the loader.

✅ Existing incomplete rows can also load slot addition (`0x1400AC090`), cosmesis (`0x1400A59E0`), rename (`0x1400AA220`), sex (`0x1400A70C0`) and race (`0x1400A88D0`). Source: `0x1400336F0`. ❓ Their full bindings remain open. An empty service fixture avoids these branches only in the proposed test milestone.

## Proposed local milestone — not original-server behavior

❓ Use a local NP trait. Do not call the original platform. The stub must return explicit token accept/reject, stable synthetic account and slot IDs, presence ownership, profile, charge eligibility and grade data. Expose timeout and platform-error fixtures. Do not silently make every platform response succeed.

✅ Step 3 implements the local Hello, NP Init/Start, cached character list, close/logout and notification controls in `Program/Server/LobbyDaemon`. Local rules and limits are in `05-rust-milestone.md`. Step 5 adds same-link retained-session login. Step 6 adds fixture-driven D24 and local world-token controls. Real world transport remains deferred. ❓ A live SQL adapter still needs schema NULL rules, optional-loader bindings and complete return-code semantics.
