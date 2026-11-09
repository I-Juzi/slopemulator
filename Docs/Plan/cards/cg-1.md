# Card cg-1: CliGate service part, spec (IDA)

## Goal
Complete the CliGate spec so that the next slice (Sonnet, no IDA) can write a Rust CliGate. The Rust CliGate must replace the old stand-ins (forward.py, lobbyd-sts) on port 6600 for the real client.

## Context
- StServers 1.20.3350. No PDB. The shared framework (route bus client) is specified in `Docs/Platform/StServers/14-route-client.md` and done in Rust (`sts-core` module `route_client`).
- The existing spec is `Docs/Platform/StServers/11-cligate-protocol.md` (146 lines). Read it in full once. Its §8 lists the open items.
- `StServer.ini`: `CliGate = 6600, 1, bnsclient`. `[InstantGlobalForwards]` forwards Auth and ExAuth by userid. Path: `/home/ubuntu/Documents/Codex/private-project/BNSX-files/D/StServers/Bin/Config/StServer.ini`.
- A real client trace (old stack): SRP login on 6600, then RequestToken for 2 AppIds, then the client goes to lobbygate 10900.

## Inputs
| What | Path |
| --- | --- |
| IDA copy (open ONLY this) | `/home/ubuntu/Documents/Codex/private-project/ida/lanes/cg-1/CliGate.exe.i64` |
| Start points | §3 dispatcher and §4 login sequence of 11-cligate-protocol.md |

## Work (3 topics)
1. Forwarding: how CliGate sends each client Auth/* request to AuthSrv over the route bus, and how it returns the reply to the client (target selection, userid forward key, `t` and `p` headers, request id mapping, timeouts, errors to the client).
2. Open items of §8: LoginFinishGateReply full body, and the RequestToken (or GetToken) path end to end. Decode the certificate loader sub_1400D6030 only if the login needs it; else mark it out of scope.
3. Session lifecycle: connect, logout, client disconnect, AuthSrv lost. Give the state table, timers in ms and log strings.

## Files
- `Docs/Platform/StServers/11-cligate-protocol.md` (update; add sections, close §8 items)
- `Docs/Plan/cards/cg-1-rust.md` (Rust card for the next slice; use `Docs/Plan/card-template.md`)

## Rust card requirements (cg-1-rust)
- Target: new crate `cligate` at `Program/Platform/StServers/CliGate` (binary + lib). Use tokio and `sts-core` (route_client, conn). List `Cargo.toml` (workspace members) in its Files section.
- The Files section must list each path as `- \`path\`` with the path first. The card must contain the line `cargo test -p cligate`.
- Tests use a fake AuthSrv on an in-process RouteSrv (`routesrv` crate) or a fake listener.
- Give exact spec line ranges, Rust signatures and at least 6 test cases with expected results. Max 3,000 tokens.

## Out of scope
- AuthSrv logic (card auth-1). SRP math lives in AuthSrv if the spec shows that.
- TLS and keys: write `keyhere`. Do not change any file under `Program/`.
