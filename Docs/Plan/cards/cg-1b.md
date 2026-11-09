# Card cg-1b: CliGate session lifecycle and Rust card (no new IDA work)

## Goal
Finish cg-1. Write §12 Session lifecycle into the CliGate spec. Write the Rust card for the CliGate crate.

## Inputs
| What | Path |
| --- | --- |
| Handoff with verified facts | `Docs/Plan/handoffs/cg-1.md` ("Not done" list) |
| Spec | `Docs/Platform/StServers/11-cligate-protocol.md` (run `python3 tools/spec_index.py` first) |
| Template | `Docs/Plan/card-template.md` |
| IDA copy (only to check one fact, max 3 calls) | `/home/ubuntu/Documents/Codex/private-project/ida/lanes/cg-1/CliGate.exe.i64` |

## Work
1. Write §12 Session lifecycle from the handoff facts: state table, Reset, LogoutMyClient (97), bus `Cli/Logout`, idle timeout 300000 ms, AuthSrv lost (keep ❓). Write the log strings.
2. Write `Docs/Plan/cards/cg-1b-rust.md`:
   - Target: new crate `cligate` at `Program/Platform/StServers/CliGate` (binary + lib). Use tokio and `sts-core` (route_client, conn). Phase 1 skips the IP check.
   - The `## Files` section lists each path as a line that starts with `` - `path` ``. Include `Cargo.toml` (workspace members).
   - The card contains the line `cargo test -p cligate`.
   - Tests use a fake AuthSrv on an in-process RouteSrv (`routesrv` crate) or a fake listener.
   - Give exact spec line ranges, Rust signatures and at least 6 test cases with expected results. Max 3,000 tokens.

## Files
- `Docs/Platform/StServers/11-cligate-protocol.md`
- `Docs/Plan/cards/cg-1b-rust.md` (new)

## Out of scope
- AuthSrv logic. Do not change any file under `Program/`. Write `keyhere` for any key.
