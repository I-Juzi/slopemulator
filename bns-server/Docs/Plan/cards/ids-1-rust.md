# Card ids-1-rust: official packet and enum names instead of bare numbers

## Goal
A new crate `packet-ids` holds the official packet command IDs and enum values from the client RE tables. The crates `packet`, `lobbygate`, `sts`, `sts-core`, `dbsrv`, `cligate` and `routesrv` use these names. No bare number is left for a packet command ID. Behaviour does not change.

## Context
- The client RE made tables with the official names (from the 419 GM PDB) and the values (from 381). Example: `G_HelloReqCmd` = 11.
- Today the code matches commands as bare numbers, for example `Self::Hello(_) => 11` in `Program/Server/Packet/src/game.rs`. That is hard to check against the docs and IDA.
- The generated code is checked in. A test makes sure that it is the same as the generator output, so a TSV change cannot be forgotten.
- Keep the official names exactly. That makes a name in the code, the docs and IDA the same text. Use `#[allow(non_camel_case_types)]`.
- Do not touch `Program/Server/GameDaemon/`. It has other work in progress. Card ids-2-rust does GameDaemon later.

## Inputs
| What | Path | Columns used |
| --- | --- | --- |
| Game commands (master list, 1,598 rows) | `Docs/Client/RE/game-commands.tsv` | `id`, `name`, `direction` |
| Lobby commands | `Docs/Client/RE/lobby-commands.tsv` | `id`, `name`, `direction` |
| STS commands | `Docs/Client/RE/03-sts-commands.tsv` | `id`, `name`, `direction` |
| InfoGate commands | `Docs/Client/RE/12-infogate-commands.tsv` | `id`, `name`, `direction` |
| Enums (all files) | `Docs/Client/RE/*-enums*.tsv` | `enum`, `value`, `name` |

Read only the header line and a few rows of each TSV. The generator reads the full files, not you.

## Work
1. Make the crate `Program/Server/PacketIds` (package `packet-ids`). Add it to the workspace `members` in `Cargo.toml`.
2. Add a bin `gen-ids` (`src/bin/gen-ids.rs`). It reads the TSVs above (paths relative to the workspace root) and writes `src/generated/commands.rs` and `src/generated/enums.rs`. Put the generator logic in the library (`pub fn generate(root: &Path) -> Result<(String, String), GenError>`), so that a test can call it.
3. Commands: make one enum per TSV: `GameCmd`, `LobbyCmd`, `StsCmd`, `InfoGateCmd`. Use `#[repr(i16)]` (use `i32` for `StsCmd` if a value does not fit `i16`). Each enum gets `pub const fn id(self)`, `pub fn from_id(id) -> Option<Self>` and `pub fn name(self) -> &'static str`.
4. Name rule: keep the TSV name. Replace characters that are not allowed in a Rust identifier (for example `/` in `Sts/Connect`) with `_`. `name()` returns the original TSV text.
5. Enums: the values can repeat in one enum (aliases). Do not make Rust enums. Make one module per enum with `pub const` items of type `i64`: `pub mod AcquireSkillBuildUpPointReason { pub const G_ACQUIRESKILLBUILDUPPOINTREASON_DEBUG: i64 = 0; }`. Skip rows with a name that is not a valid identifier after rule 4, and write a count of skipped rows in a comment at the top of the file.
6. The generator stops with an error on a duplicate name in one enum or table, or on a duplicate command ID in one table. If the TSVs have such duplicates, keep the first, write each case in a comment, and write the list in the handoff under ❓.
7. Run `cargo run -q -p packet-ids --bin gen-ids`. Check in the output.
8. In the crates `packet`, `lobbygate`, `sts`, `sts-core`, `dbsrv`, `cligate` and `routesrv`, replace each bare packet command number with the enum, for example `GameCmd::G_HelloReqCmd.id()`. In a `match` on a command, use `match GameCmd::from_id(cmd)` or consts. Use `rg -n "=> *[0-9]+|[0-9]+ *=>"` to find them. Do not change numbers that are not command IDs (lengths, offsets, field values).
9. If the old number and the TSV value differ, keep the old number, add a comment `// TSV says <n>`, and write it in the handoff under ❓. Do not change behaviour.

## Interfaces
```rust
// packet-ids
#[allow(non_camel_case_types)]
#[repr(i16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameCmd { G_EnterLeaveBroadcastCmd = 1, /* ... */ }
impl GameCmd {
    pub const fn id(self) -> i16;
    pub fn from_id(id: i16) -> Option<Self>;
    pub fn name(self) -> &'static str;
}
pub fn generate(root: &std::path::Path) -> Result<(String, String), GenError>;
```

## Files
- `Program/Server/PacketIds/`
- `Cargo.toml`, `Cargo.lock`
- `Program/Server/Packet/`, `Program/Server/LobbyGate/`, `Program/Platform/`

## Acceptance
| Test name | Input | Expected result |
| --- | --- | --- |
| `generated_is_current` | `generate(workspace root)` | equal to the checked-in `commands.rs` and `enums.rs` |
| `game_hello_ids` | `GameCmd::G_HelloReqCmd` | `id()` = 11; `from_id(11)` gives it back; `name()` = `"G_HelloReqCmd"` |
| `lobby_hello_id` | `LobbyCmd::from_id(1)` | `L_HelloReq` |
| `sts_name_kept` | `StsCmd::from_id(1001)` | `name()` = `"Sts/Connect"` |
| `enum_const` | `AcquireSkillBuildUpPointReason::G_ACQUIRESKILLBUILDUPPOINTREASON_DEBUG` | 0 |
| all existing tests | the 7 crates | pass unchanged |

## Out of scope
- `Program/Server/GameDaemon/` (card ids-2-rust).
- Field enums in packet payloads. Use the new consts only where a number is clearly one of them. Do not refactor payload code.
- Any behaviour change.
