# Card auth-1: AuthSrv login and token, spec (IDA)

## Goal
Write the AuthSrv spec for the client login path, so that the next slice (Sonnet, no IDA) can write a Rust AuthSrv. With the Rust CliGate (cg-1) it must log in the real client.

## Context
- StServers 1.20.3350. No PDB. The route bus client is in `Docs/Platform/StServers/14-route-client.md` (done in Rust, `sts-core` module `route_client`).
- CliGate side: `Docs/Platform/StServers/11-cligate-protocol.md` §2 (Auth command enum) and §4 (login sequence). Read only those sections (use `tools/spec_index.py`).
- DbSrv contract: `Docs/Platform/StServers/12-dbsrv-contract.md` and `procs.tsv` (stored procedures). Error codes: `errcodes.tsv`.
- `StServer.ini`: `AuthSrv = 6605, ...`. Path: `/home/ubuntu/Documents/Codex/private-project/BNSX-files/D/StServers/Bin/Config/StServer.ini`.

## Inputs
| What | Path |
| --- | --- |
| IDA copy (open ONLY this) | `/home/ubuntu/Documents/Codex/private-project/ida/lanes/auth-1/AuthSrv.exe.i64` |
| Start points | handlers for Auth/LoginStart, Auth/KeyData, Auth/LoginFinish, Auth/RequestToken (find by string) |

## Work (3 topics)
1. SRP login, server side: each step, message bodies, the group (N, g), hash, the verifier source (DB proc), session key use, and each error reply. Write `keyhere` for any fixed key.
2. Tokens: RequestToken / token issue for an AppId: body, lifetime in ms, storage, and how other services validate the token.
3. DB calls: each DbSrv stored procedure that the login and token path call, with parameters and result use. Link to procs.tsv rows.

## Files
- `Docs/Platform/StServers/15-authsrv.md` (new spec)
- `Docs/Plan/cards/auth-1-rust.md` (Rust card for the next slice; use `Docs/Plan/card-template.md`)

## Rust card requirements (auth-1-rust)
- Target: new crate `authsrv` at `Program/Platform/StServers/AuthSrv` (binary + lib). Use tokio and `sts-core`. List `Cargo.toml` (workspace members) in its Files section.
- The Files section must list each path as `- \`path\`` with the path first. The card must contain the line `cargo test -p authsrv`.
- DB access goes through a trait with an in-memory fake for tests. Tests include an SRP round trip against a test vector from the spec.
- Give exact spec line ranges, Rust signatures and at least 6 test cases with expected results. Max 3,000 tokens.

## Out of scope
- ExAuth, OTP, PC-bang and billing paths. Other AuthSrv services.
- Do not change any file under `Program/`.
