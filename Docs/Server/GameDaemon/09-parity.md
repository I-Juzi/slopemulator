# 09 — GameDaemon 381 parity ledger

## State

Static specification only. No Rust change. No test or client connection ran.
The ledger has one row for each supplied build-381 command name.
The ledger records dispatch membership, checked sends and existing spec coverage.
It has 1598 rows. It has 41 partial specs and no complete behavior specs.
An unknown direction is a research gap. It is not an unused command.

## Dispatch evidence

| Build | Dispatcher | Table role | Table base | Entries | Entry width |
| --- | --- | --- | --- | ---: | ---: |
| 348.8349 | `0x14007DA70` | Any authentication state | `0x140E833C0` | 1544 | 8 bytes |
| 348.8349 | `0x14007DA70` | No authenticated user | `0x140E86400` | 1544 | 8 bytes |
| 348.8349 | `0x14007DA70` | Authenticated PC | `0x140E89440` | 1544 | 8 bytes |
| 381.8867 | `0x14007BC00` | Any authentication state | `0x1407DD180` | 1599 | 8 bytes |
| 381.8867 | `0x14007BC00` | No authenticated user | `0x1407D6D50` | 1599 | 8 bytes |
| 381.8867 | `0x14007BC00` | Authenticated PC | `0x1407D9F80` | 1599 | 8 bytes |

Build 348 PDB names identify `ClientSessionHandler::PacketHandler` and all three tables.
The dispatcher reads the command short at logical packet header +2.
It first checks the any-state table. It then selects the unauthenticated or PC table.
Each lookup uses `table + command * 8`. A null pointer does not dispatch.
Build 348 checks command < `0x608`. Build 381 checks command <= `0x63E`.

The build-381 structure matches the build-348 structure.
Both paths use the session-user link, then the user-PC link.
Both paths check packet rights and product type before the PC call.
Both paths have unknown-command and missing-PC log branches.
The source-path reference at build-381 `0x1409087E0` names `ClientSessionHandler.cpp` and build 381.
The safe-mode virtual calls are stop boundaries. They were not followed.

The supplied source is `notes/client-381-game-commands.tsv`.
Its client name table starts at `0x1953620`. Its index is the command ID.
The source contains IDs 1–1598. It omits sentinel 0.
Do not add sentinel 0 as a ledger row.
Do not copy a build-348 numeric ID into build 381.
Match historical handler names by command name or explicit handler-log evidence.

## Columns

| Column | Meaning |
| --- | --- |
| `id`, `name` | Exact ID and name from the supplied build-381 list. |
| `dir` | `c2s`: a non-null GameDaemon receive slot; `s2c`: a checked GameDaemon send; `other-daemon`: checked ownership elsewhere; `unused`: checked absence of use; `unknown`: no direction proof. |
| `handler381` | Build-381 receive target, or `-`. A send address belongs in `notes`, not this column. |
| `name348` | PDB handler or sender name from build 348, or `-`. This is historical name evidence, not proof of unchanged behavior. |
| `subsystem` | Port-planning group. `out-of-scope` excludes anti-cheat and licence work. |
| `spec` | `none`, `partial: <doc files>`, or `done: <doc files>`. The state covers behavior, not only a packet codec. |
| `rust`, `tests` | `none` for this specification job. These columns do not audit existing code or tests. |
| `notes` | Table slots, log references, send evidence, boundaries and gaps. `❓` marks an unresolved fact or a name-based planning guess. |

## Evidence files

| File | Content |
| --- | --- |
| `parity-ledger.tsv` | Command-level parity ledger. |
| `parity-evidence-348.json` | Non-null build-348 slots, targets and PDB names. |
| `parity-evidence-381.json` | Non-null build-381 slots and targets. |
| `parity-handler-evidence.json` | Build-381 handler-log tags and their instruction/string references. No full log text or runtime values. |

## Limits

A receive slot proves routing. It does not prove a complete handler implementation.
A shared return-true target is still a routed receive command.
A null receive slot does not prove a server send, another daemon, or an unused ID.
A command suffix is not direction evidence.
All existing behavior specifications remain partial until their gaps are closed.
Do not derive a working entry sequence from this ledger alone.

## Extraction and send proof

The build-348 tables have 2 any-state, 6 unauthenticated and 477 PC entries.
The build-381 tables have 2 any-state, 6 unauthenticated and 494 PC entries.
All counts above include only non-null entries.
The build-381 union contains 502 command IDs. None overlap between tables.
Fourteen IDs route to the shared return-true stub at `0x14001AFE0`.
Sixteen QQ IDs share target `0x1400DE550`. No function was recovered there.
These IDs stay `c2s`. Do not claim that their named behavior works.

The name pass matched 454 rows to build-348 receive-handler names.
It matched exact command stems, not numeric IDs.
The command-32 row also names PDB sender `Pc::SendEnterWorldOk` at `0x1405EB540`.
Thus 455 rows have a historical name. Other rows keep `-`.
The build-381 log pass found 367 exact command-stem matches.
Fourteen other rows have different handler-log labels. Their notes retain the difference.
For example, command 64 names `MoveReq`, but its build-381 log names `HandleGame_WalkReq`.
The dispatch slot, not the log spelling, proves the target.

The send pass used these checked endpoints:

| Endpoint | Evidence |
| --- | --- |
| Session format send | `0x1406A5D30` passes variadic arguments to `0x1406A5E00`. That helper composes a logical packet and queues it on the session. |
| PC format send | `0x1405F5530` obtains the user session and calls `0x1406A5E00`. PC vtable base `0x140C9AAC8`, slot +592, points to it. |
| Summoned-creature format send | `0x140619A40` obtains the owning PC's user session and calls `0x1406A5E00`. Vtable base `0x140C9BFD8`, slot +592, points to it. |
| Build-348 name guide | PDB `Pc::Send` at `0x1405EB350` has the same user-session and format-send structure. |

The pass examined direct format sends in permitted receive handlers.
It also examined PC/summon virtual format sends at slot +592 in PC handlers.
It required a constant command ID and a static format that starts with `h`.
It recorded the host, call address, format address and format in `parity-send-evidence.json`.
The pass found 296 send sites for 137 distinct IDs. No ID overlaps a receive slot.
It added 15 send IDs from the address-backed evidence in docs 04–07.
It did not infer a success reply from an adjacent failure ID.
It did not treat packet composition alone as a proved send.

Command 24 stays `unknown` in this stricter ledger.
Docs 04–06 identify a client receive callback, but they do not identify its GameDaemon send site.
Its partial spec remains linked. Its complete body remains open.

The send pass did not decompile the ordinary ping handler `0x140370F10`.
Doc 06 supplies its ordinary behavior and its restricted-hook boundary.
One other function, `0x1403CEA20`, did not decompile. Its receive slot still proves `c2s`.
The send pass did not follow helper call graphs or inspect every server sender.
Thus the `s2c` count is a lower bound, not the complete server command count.

## Direction counts

| Direction | Rows |
| --- | ---: |
| c2s | 502 |
| s2c | 152 |
| other-daemon | 0 |
| unused | 0 |
| unknown | 944 |
| Total | 1598 |

No row has enough evidence for `other-daemon` or `unused`.
Of the 944 unknown rows, 940 are in scope and 4 are out of scope.
Only one unknown row has `Req` in its name. That name does not prove its direction.

## Subsystem counts

Subsystem groups support port planning. They do not prove native subsystem ownership.
The groups use command-name terms. Each in-scope row marks this limit with `❓`.
For example, guild names map to `guild`; arena/duel names map to `pvp-arena`.
Item, inventory, equipment and crafting names map to `item-inventory`.
Buy, sell and store names map to `shop`. Post and market names map to `mail-market`.
Team names map to `party`. Mentoring and QQ names map to `social`.
Entry and channel names map to `session`. Unclear names remain `misc`.
NPC manipulation requests do not prove an NPC AI implementation.

| Subsystem | Total | c2s | s2c | unknown |
| --- | ---: | ---: | ---: | ---: |
| session | 92 | 30 | 17 | 45 |
| movement | 30 | 13 | 6 | 11 |
| skill-combat | 151 | 51 | 16 | 84 |
| npc-ai | 2 | 0 | 0 | 2 |
| item-inventory | 306 | 106 | 21 | 179 |
| shop | 49 | 18 | 7 | 24 |
| quest | 66 | 22 | 16 | 28 |
| party | 156 | 40 | 16 | 100 |
| chat | 46 | 16 | 3 | 27 |
| guild | 213 | 64 | 15 | 134 |
| social | 87 | 28 | 1 | 58 |
| pvp-arena | 274 | 75 | 29 | 170 |
| mail-market | 62 | 21 | 0 | 41 |
| misc | 55 | 13 | 5 | 37 |
| out-of-scope | 9 | 5 | 0 | 4 |
| Total | 1598 | 502 | 152 | 944 |

The 9 out-of-scope rows are a subset of the direction totals.
Do not add 9 to 1598. There are 1589 in-scope rows.

## Restricted rows and addresses

| IDs | Boundary address in build 381 | Rule |
| --- | --- | --- |
| 348–350 | Receive target for 349: `0x140396480` | GameGuard. Stop. Sender addresses for 348/350 are not established. |
| 351 | `0x1403BFE10` | NCGuard. Stop. |
| 352 | `0x1403C0000` | XIGNCode3. Stop. |
| 353–354 | Receive target for 354: `0x1403C16B0` | ❓ Anti-bot scope follows the command names. Stop. Sender address for 353 is not established. |
| 791–792 | Receive target for 792: `0x1403C46F0` | ❓ Test-bot query/answer scope follows the command names. Stop. Sender address for 791 is not established. |

A paired receive boundary is not a recovered sender address.
No BloodCert or BNSHelper row was invented. The supplied names do not identify one.
Ordinary session rows stay in scope when they also contain a restricted hook.
Their notes retain the hook boundary. Do not follow that hook.

## Existing spec coverage

There are 41 `partial` rows, 1557 `none` rows and no `done` rows.
Docs 04–06 cover Hello, admission, entry, ordinary ping and the command-929 observation.
Doc 07 covers command 64 movement, command 1226 skill use, selected interactions and failure replies.
Its BoardGacha links cover shared failure handling only. They do not specify complete transactions.
No complete behavior claim follows from a known packet format.
The `rust` and `tests` columns remain `none` in all 1598 rows, as this job requires.
Existing codec code and unrun tests in STATUS are not erased or reclassified by this ledger.

## Proposed port order

Start with session and movement. Then use descending in-scope c2s row counts.
A row count measures command coverage, not engineering effort.
Resolve table and transaction dependencies before each implementation step.

| Order | Subsystem | c2s rows | First gate |
| ---: | --- | ---: | --- |
| 1 | session | 30 | Close command 24/32, standing-entry and ordinary liveness gaps in docs 04–06. |
| 2 | movement | 13 | Close the remaining cell-loading, transition and wire-field gaps in doc 07. |
| 3 | item-inventory | 106 | Specify server inventory ownership, transactions and failure paths. |
| 4 | pvp-arena | 75 | Separate local game rules from arena-daemon coordination. |
| 5 | guild | 64 | Specify guild ownership and backend transactions. |
| 6 | skill-combat | 51 | Extend the checked UseSkill subset to all action classes. |
| 7 | party | 40 | Specify membership state, votes and item distribution. |
| 8 | social | 28 | Separate local behavior from external-service routes. |
| 9 | quest | 22 | Specify mission state and exact interaction predicates. |
| 10 | mail-market | 21 | Specify daemon links, attachment ownership and transaction completion. |
| 11 | shop | 18 | Preserve NPC range boundaries and non-NPC branches. |
| 12 | chat | 16 | Specify routing and channel ownership. |
| 13 | misc | 13 | Resolve each command's owner before implementation. |
| 14 | npc-ai | 0 | Build a separate server-initiated AI ledger. Client commands cannot measure AI parity alone. |

## Validation and open work

The validation checks all 1598 persisted rows against the supplied ID/name pairs.
It checks unique IDs, exact dispatch membership and handler addresses.
It checks send evidence, allowed column values and unchanged `none` progress fields.
It checks that notes contain no field-breaking tabs or newlines.
It also checks for endpoint literals. No runtime account data or key material was exported.
No Rust tests or live-client tests ran.

- ❓ Recover send sites for the remaining 940 in-scope unknown rows. Do not assign direction by suffix.
- ❓ Prove unused IDs or other-daemon ownership before using either label.
- ❓ Resolve command 24 and the complete command-32 payload outside restricted paths.
- ❓ Recover command-929 cached success construction. The handler name is now known; the success body is not.
- ❓ Replace name-based subsystem guesses with handler and dependency evidence.
- ❓ Add server timers, NPC AI, transactions and world rules that have no client-command row.
- ❓ Define acceptance tests before marking any behavior spec `done`.

Only the two authorized GameDaemon databases were opened.
No network operation, git command, binary patch or database-save request was used.
