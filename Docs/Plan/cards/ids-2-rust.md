# Card ids-2-rust: official packet names in GameDaemon

## Goal
GameDaemon uses the `packet-ids` names (from ids-1-rust) for all packet command IDs. No bare number is left for a command ID. Behaviour does not change.

## Context
- The crate `packet-ids` exists. It has `GameCmd`, `LobbyCmd`, `StsCmd`, `InfoGateCmd` (`id()`, `from_id()`, `name()`) and one module of `pub const` per enum. Run `python3 tools/rust_api.py Program/Server/PacketIds` for the interface.
- The main dispatch is a `match` on bare numbers in `Program/Server/GameDaemon/src/handlers/mod.rs` (near line 1450, about 43 arms, for example `11 => handle_hello(ctx, req)`). Other handler files and tests also use bare numbers.

## Work
1. Add `packet-ids` as a dependency of GameDaemon.
2. Replace the bare command numbers in the dispatch `match` with `GameCmd` names. Keep the same handler for each ID.
3. Replace bare command numbers in the other GameDaemon files and tests. Find them with `rg -n "=> *[0-9]+|[0-9]+ *=>|cmd\(|command" Program/Server/GameDaemon`. Do not change numbers that are not command IDs.
4. If the old number and the `packet-ids` value differ, keep the old number, add `// packet-ids says <n>`, and write it in the handoff under ❓.

## Files
- `Program/Server/GameDaemon/`
- `Cargo.lock`

## Acceptance
| Test name | Input | Expected result |
| --- | --- | --- |
| `dispatch_ids_named` | `rg` for a bare number arm in the dispatch `match` | none left (write the command and its output in the handoff) |
| all existing GameDaemon tests | — | pass unchanged |

## Out of scope
- Payload field refactors and any behaviour change.
- Other crates.
