# Handoff: ids-1-rust

## Done
- New crate `Program/Server/PacketIds` (`packet-ids`): generator `generate()`, bin `gen-ids`, checked-in `src/generated/commands.rs` and `enums.rs`.
- `GameCmd`, `LobbyCmd`, `StsCmd`, `InfoGateCmd` (`#[repr(i16)]`) with `id()`, `from_id()`, `name()`. One module per enum with `pub const NAME: i64`.
- packet, lobbygate and sts use the official names for client-space command IDs. Behaviour is the same.
- Tests in `tests/ids.rs`: `generated_is_current`, `game_hello_ids`, `lobby_hello_id`, `sts_name_kept`, `infogate_hello_id`, `names_are_cleaned`, `generator_stops_on_duplicate_id`, `enum_const`.
- All existing tests of packet, lobbygate, sts, sts-core, dbsrv, cligate and routesrv pass. fmt and clippy (`-D warnings`) are clean.
- `cargo fmt` also changed the layout of 5 Sts files (`examples/enroll.rs`, `src/listener.rs`, `tests/client_xml.rs`, `tests/codec.rs`, `tests/wire.rs`) and of the DbSrv files (`src/*.rs`). Only formatting. Gate fix: `cargo fmt -p dbsrv` after the `fmt --check` failure in `DbSrv/src/main.rs`.
- Checks run again for all 8 crates after the fix: `fmt --check`, `test`, `clippy -D warnings` all pass.
- Abuse notes: none found (`Docs/Plan/abuse-notes/ids-1-rust.md`).

## Not done
- sts-core, dbsrv, cligate, routesrv: no bare packet command IDs exist there. No code change (dbsrv has format changes only).
- LobbyDaemon (D-space) IDs stay as numbers. No TSV exists for them.

## ❓
- Backend IDs 17 and 271 in `Packet/src/game_requests.rs` and lobby command 5 in `game_session.rs` have no TSV name. Kept as numbers.
- STS reply IDs 3957, 3970 and 3981 are not in `03-sts-commands.tsv`. Kept as numbers.
- LobbyGate `server.rs` uses ID 19 for the world-list failure reply. The TSV names 19 `L_GetWaitInfoFail`. `L_GetWorldInfoListFail` is 26. Number kept (`// TSV says 26`).
- 2891 enum rows repeat in more than one TSV with the same value (for example `Data::SkillMoveTypeType`). The generator keeps the first row and lists the repeats in the file header. No repeat has a different value.
- Enum values above `i64::MAX` are stored as `i64` with the same bits (18446744073709551615 becomes -1).

## Blocked
- Nothing.
