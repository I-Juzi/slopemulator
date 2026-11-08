GP1: `parity-ledger.tsv` and `09-parity.md` record 1598 commands: 502 c2s, 152 s2c, 944 unknown; 9 rows are out of scope within those totals. 41 specs are partial. Specs only. No live test.
Step 5: movement, skills, distance and failure/kick rules in `07-server-validation.md` are partial and address-backed. Specs only. No live test.

## Rust gr1 — 2026-11-08

New crate: `Program/Server/GameDaemon` (`gamedaemon`). It has no dependencies or sockets. Each check cites Step 5 in `07-server-validation.md` and a build-381 address. Geometry and skill data use traits. Tests use generated fixtures only.

| Rule | State | Spec section |
| --- | --- | --- |
| Walk2 handler/action limits, cell records, endpoint layers and start tolerance | Ported subset | Step 5 part 1, Input limits / Cell pseudo-steps |
| Speed sources, squared limits and optional duration-history comparison | Ported subset | Step 5 part 1, Speed and duration sources |
| Server height/state checks and correction plans | Partial | Step 5 part 1, Height/state / Failure result |
| UseSkill group limits, deduplication, data dependencies and ordered gates | Partial | Step 5 part 2 |
| Strict global recycle boundary | Ported | Step 5 part 2, Verified check order |
| Trade/NPC/quest distance and stated bypass helpers | Ported subset | Step 5 part 3 |
| Skill cylinder and ordinary target-check order | Partial | Step 5 part 2, Target/range |
| Backend reason conversion and ordered BoardGachaDraw failure plan | Ported subset | Step 5 part 4 |

**Checks:** 55 new tests passed; 0 failed; 0 ignored. Strict Clippy passed after one needless-borrow fix. Formatting passed. The plain commands `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` ran in the dependency-free check harness at `Program/Server/GameDaemon/target/standalone`. It points to the actual new source and tests. These are **not full source-workspace results**. The source-workspace baseline of 308 passed / 6 ignored was not rerun. Existing tests were not changed.

The full-workspace copy commands required approval and did not run. `tools/check_workspace.py` provides an offline copy procedure for an approved run. The root lock file was not edited. Cargo inherited the existing root target directory and wrote build artifacts to `Program/output`. No network or git command ran. The parity ledger and `09-parity.md` were not changed.

**TODO ❓:** Complete movement transitions (`0x14048C0B0`), geometry loading, duration-ring updates, jump tracking, rising-fall predicate (`0x14048BF70`), summoned rules and unknown correction reasons. Pending height branches do not approve movement. State-only results do not invent transition side effects.

**TODO ❓:** Full skill parsing/dispatch, table-field mappings, server formulas, cast/resource/acquired predicates, later castability checks and local recycle expression. Trait gates preserve the verified order, but do not implement unknown predicates. A passed castability prefix does not prove complete castability. Full skill integration remains blocked.

**TODO ❓:** Quest configuration/caller mapping, unknown outer replies, target-filter/LOS internals, kick identity/reason, transaction +252 provenance and final close. No generic validation kick was added. Restricted hooks remain comments only. No live acceptance test ran.

# GameDaemon world entry: step 4 report

Specs: `bns-server/Docs/Server/GameDaemon/`.

| Part | State | Evidence |
| --- | --- | --- |
| Frame comparison | done | `04-world-connect.md`: ordinary size/tag rules match BloodLib. Server `0x1406ADD00`; client `0x60F8A0`. ❓ Compression/broadcast compatibility remains open. |
| Payload gaps | partial | `05-enter-world.md`: command-32 groups; native K at `0x1406AA7B0`; command-42 correlation/summoned fields at `0x1402912F0`; command 43 is `hq` at `0x1402304A0`. |
| Rust game layer | partial | Separate `game381.json`; ten structural codecs in `game.rs`. Unknown bodies return unsupported errors with raw bytes. ❓ Compilation unverified. |
| Checks | not started | Test, Clippy and formatting requests did not run. Tool output: `This command requires approval`. |

Code: `bns-server/Program/Server/Packet/src/game.rs:1`.
Table: `bns-server/Program/Server/Packet/tables/game381.json:1`.
Tests: `bns-server/Program/Server/Packet/tests/game381.rs:1`.

**Tests:** 16 new generated tests written; none run. Ten round-trip tests each use 256 generated fixtures.
**Totals:** no new results. Task baseline only: 292 passed, 6 ignored. Existing tests were not changed.
**Clippy:** not run; no pass result. Formatting also did not run.

**Boundaries:** RSA handshake → AES-128 session families only. Static RSA private-object loading at `0x1406E3100` is a stop boundary. No secret bytes were read. Restricted hooks were not followed.
**❓ Open:** command 24; full command 32 and native fragment metadata; command-43 q meaning; appearance IDs/bodies/order; DB mappings; coordinate units; command 929; full client secret ownership and acceptance.
Appearance queue `0x14024B0C0` runs before command 43. This does not establish wire delivery order.
`06-minimum-session.md` records the updated standing-session limits. No successful client entry or standing test ran.

**Next build step:** obtain tool approval, then run these checks without network access:
```sh
CARGO_NET_OFFLINE=true cargo test --manifest-path bns-server/Cargo.toml
CARGO_NET_OFFLINE=true cargo clippy --manifest-path bns-server/Cargo.toml --workspace --all-targets -- -D warnings
```
Resolve command 24 and the full command-32 contract before server integration. Keep unknown bodies unsupported.
Both IDA leases are closed. No network, git, binary patches or explicit database saves were used. `Program/Tools/Emu/` is unchanged.

## Step 3 report

# GameDaemon world entry: step 3 report

Specs: `bns-server/Docs/Server/GameDaemon/`.

| Part | State | Evidence |
| --- | --- | --- |
| Connection | partial | `04-world-connect.md`: separate game connection; Hello 11/12 and token request 23. Server `0x1403703D0`, `0x140371170`. |
| Enter-world sequence | partial | `05-enter-world.md`: 31/32 → 41/42/43. Prepare sender `0x1404BEED0`; client zone request `0x7815C0`. |
| Minimum session | partial | `06-minimum-session.md`: ping 16 and echo 17/18; timer `0x1406832D0`. ❓ Complete standing packet set remains unverified. |

**Verified:** game protocol 1278; fragmented PC initialization; zone preparation; client load/enter request; ordinary ping echo.
**Boundaries:** client `0x741AB2`, `0xF3B00C`/`0xF3B1F6`, `0xEF68C8`; server zone hook `0x1405DF767`, ping hook `0x14036DE10`. No restricted hook internals were inspected.
**❓ Connection gaps:** outer framing; handshake family sequence and secret ownership; command 929; command-24 success body. No vendor-secret dependency was established.
**❓ Entry gaps:** full command-32 payload and DB mapping; command-42 field meanings; command-43 body; spawn/appear packets; final load-screen exit.
**❓ Session gaps:** safe omission of inventory, skills, quests and social data; complete timeout rules. No standing-client test ran.
**First Rust step:** add a separate game dispatch schema and synthetic codec tests in the existing Packet/BloodLib layers. Keep unknown bodies unsupported. Do not claim successful entry.
**Scope:** specifications only. No Rust changes, network use, git commands, binary patches or explicit database saves. Both IDA leases are closed.

## Previous report

# GameDaemon data: step 2 report

| Part | State | Evidence |
| --- | --- | --- |
| Survey | done | `02-tables.md`: 252 types, 21,062 table files, 1,018,183 direct elements; attribute types and reference candidates. |
| Minimum-world chain | partial | `03-minimum-world.md`: spatial chain specified. All 30 level-one settings resolve to zone 1101 and terrain 1001. ❓ Full gameplay dependencies remain open. |
| Typed loader | done | `Program/Server/Data/GameData`: 12 types; id/alias indexes; initial-setting keys; unknown attributes and children retained; reference diagnostics. |
| Tests | done | 14 generated tests; separate real-folder test passes; unset `BNS_GAMEDATA` prints `SKIP`. |
| Workspace checks | done | 290 passed, 0 failed, 6 ignored. Clippy passes with `--workspace --all-targets -- -D warnings`. |

Opt-in run: **12 table types, 10,608 files, 138,440 indexed records**.
Skipped: 10,454 other table files, 2 other binary roots, 1 plain XML file.

| Loaded table | Records |
| --- | ---: |
| PcInitialSetting | 93 |
| zone | 1,533 |
| Terrain | 164 |
| ZonePcSpawn | 2,951 |
| ZoneRespawn | 1,652 |
| ZoneArea | 76,936 |
| ZoneEnv | 98 |
| ZoneEnv2 | 11,626 |
| ZoneEnv2Place | 11,682 |
| ZoneEnv2Spawn | 11,832 |
| script | 17,263 |
| quest | 2,610 |

Reference checks report **8,500 unresolved candidate links**: 6,217 script, 1,974 placement, 175 environment-template, 83 zone, 33 player-spawn, 16 phase-zone, and 2 respawn links.
Three extra script-table elements remain in file metadata. The survey counts them; the script index does not.
❓ Open questions: missing-link causes, placement scope, initial spawn selection, zero defaults, coordinate units, and terrain geometry.
Next job: resolve reference naming and scope from bytes. Extend dependency checks. Specify initial spawn selection before server integration.
No game records were added. No IDA or network tools were used. Downloads and `Program/Tools/Emu/` were not changed.

## Step 1 evidence

# GameDaemon data: step 1

| Part | State | Evidence |
| --- | --- | --- |
| Header | done | 21,064 v3 files; exact size fields; 64 zero reserved bytes. See bns-server/Docs/Server/GameDaemon/01-lmxboslb.md:1. |
| Body | done | 1,786,132 elements; 2,202,819 text nodes; maximum depth 7. Strings use UTF-16LE and a repeating 16-byte XOR. |
| Decoder and encoder | done | New dependency-free `lmxb` crate. Bounds checks, metadata retention, XML escaping, and malformed-input tests pass. |
| Round trip | done | Generated data and all 21,064 binary files pass byte-exact round trips. |
| XML dump | done | `lmxb-dump <file>` prints XML. Generated CLI tests pass. |
| Workspace checks | done | 275 passed, 0 failed, 6 ignored. Workspace Clippy passes with `--all-targets -- -D warnings`. |

## Opt-in results

- Found 21,065 XML paths. Decoded 21,064 LMXBOSLB files.
- Read 1,674,199,807 binary bytes. Counted 1,786,132 elements.
- Decode failures: **0**. Byte-exact round-trip failures: **0**. XML render failures: **0**.
- Skipped `accountleveldata.xml`: verified plain UTF-8 XML with a BOM.
- Binary groups: root 1,098; quest 2,610; script 401; social 88; zone 16,867.
- Without `BNS_GAMEDATA`, the opt-in test prints `SKIP`. The normal test total includes that test.

## Open questions and limits

- ❓ The meanings of the opaque flags and reserved header fields remain unknown.
- ❓ The source-line field meaning is inferred. No source XML pair was available.
- ❓ Other format versions and node kinds were not verified.
- The decoder accepts version 3 and at most 128 node levels. Encoded documents must fit a u32 file size.
- No game data was added to the repository. `bns-server/Program/Tools/Emu/` was not changed.

## Next job

Write a data-loader specification from decoded table metadata. Start with zone and quest tables. Define typed Rust records and reference checks. Keep plain XML handling separate from this binary codec.

## Rust gr2 — 2026-11-08

Build: GameDaemon 381.8867. Sources: docs 04–07, 09–11. Code: `Program/Server/GameDaemon/src/handlers/`. Each of the 43 handler functions has a `parity:` comment. The dispatcher checks route membership and PC presence. Unknown commands log only. Unmapped packet-right and product-type checks return `Unsupported`. The caller must resolve these checks from server state.

| IDs | State |
| --- | --- |
| 13, 16, 19, 22, 29, 34, 38, 307, 414, 416, 422, 812, 1221 | Ported receive logic |
| 11, 18, 23, 26, 31, 42, 64, 66, 99, 102, 145, 174, 262, 296, 298, 300, 303, 305, 411, 648, 809, 887, 929, 981, 984, 1149, 1210, 1219, 1223, 1239 | Partial; explicit `Unsupported` boundaries |

`Session`, `Pc` and `World` hold server state. `Outcome` lists replies, broadcast plans, state changes, daemon requests and kick actions. No handler adds a generic kick. Native false returns do not close a connection. The queue and transaction adapter uses `Services`. `MemoryServices` is an in-memory fake. Unknown adapter results do not approve a task. The dispatcher retains raw input for queued continuations. A queued task does not prove completed travel or a completed save.

Command 64 reuses `validation::movement` for control selection, creature selection, handler counts and initializer limits. It stops before incomplete duration-history and action-start integration. It does not replace speed, height or transition checks with a distance check. Switch detection includes the verified table and squared-radius failure checks. Air dash includes the verified prepare-check order. Channel change preserves signed channels and the cooldown boundary. Portal state 2 stops before its unmapped mutation.

New codecs: `Program/Server/Packet/src/game_requests.rs`. This separate API covers all 43 requests. It uses the full verified descriptors and the existing BloodLib format codec. It keeps unused bodies opaque. It permits trailing input in handler-prefix decoding. It preserves the third qword in command 1149. Existing `game::GamePacket` behavior and table rows are unchanged. Backend codec plans cover Lobby command 17 (`hqQQ`), cached command 271 (`hQqq`) and the one-record client-option command 17 (`hQCqqchb;`). No backend completion is claimed.

**Tests written:** 80 new tests: 43 per-handler tests, 22 shared handler tests, 8 edge tests and 7 codec tests. Each handler test includes a malformed or unused-body request. Other tests cover ordered failure checks, counts, state latches, signed fields, pending backend work and codec round trips. Existing tests were not changed. Deferred paths behind `Unsupported` do not have full success/failure coverage.

**Execution:** Workspace tests and strict Clippy did not run. The tool returned `This command requires approval`. Requested commands included `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`, with offline mode and a target directory under GameDaemon. Compilation and formatting are not verified. The supplied baseline remains 363 passed / 6 ignored; it is not a new result. No new pass total is claimed.

**TODO ❓:** Full handshake acceptance and command 24; complete command 32; world-entry destination and failure cleanup; all EnterZoneTask effects; movement duration history, speed/height/transition integration and broadcasts; turn bounds and unknown wrapper failure arguments; activated-property metadata and mutation; base-camp helper; party manager predicates and notifications; channel restriction branches and LeaveAction routing; cached option persistence; auto-logout report destination; unchecked malformed echo bits; QQ interface effects; contents-cache construction; direct-zone tasks; jumping-ticket transaction; ClientRun reports; air-dash start/effect helpers; teleport event/table/cost/travel chain; cave-escape reservation/disposal and destination chain. These paths remain unsupported. No incomplete command 32, arbitrary-coordinate warp or synthetic broadcast command was added.

Restricted hooks remain address comments only: `0x1403710A5` / `0x14036DE10`, `0x140371AC8`, `0x1403721D5`, `0x1405DF767`, `0x140394181`, `0x140394427` / `0x14060E280`, `0x1403895C7` / `0x1403EAB50`. The private-object boundary `0x1406E3100` remains excluded. No anti-cheat, licence, socket, network or Git operation ran. No other Docs file was changed. No live-client acceptance test ran.
