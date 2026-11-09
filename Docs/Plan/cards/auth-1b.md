# Card auth-1b: AuthSrv finish spec and Rust card

## Goal
Finish auth-1. Write the verified facts into the AuthSrv spec. Decode the token path. Write the Rust card for the AuthSrv crate.

## Inputs
| What | Path |
| --- | --- |
| Handoff with verified facts | `Docs/Plan/handoffs/auth-1.md` ("Not done" and ❓ lists) |
| Spec | `Docs/Platform/StServers/15-authsrv.md` (run `python3 tools/spec_index.py` first) |
| Template | `Docs/Plan/card-template.md` |
| IDA copy (open ONLY this) | `/home/ubuntu/Documents/Codex/private-project/ida/lanes/auth-1/AuthSrv.exe.i64` |

## Work (in this order; write each part into the file at once)
1. Write §6 LoginFinish and §8 DB calls from the handoff facts. No IDA needed.
2. Write `Docs/Plan/cards/auth-1b-rust.md` now, before step 3:
   - Target: new crate `authsrv` at `Program/Platform/StServers/AuthSrv` (binary + lib). Use tokio and `sts-core`.
   - The `## Files` section lists each path as a line that starts with `` - `path` ``. Include `Cargo.toml` (workspace members).
   - The card contains the line `cargo test -p authsrv`.
   - DB access goes through a trait with an in-memory fake. Tests include an SRP round trip.
   - Give exact spec line ranges, Rust signatures and at least 6 test cases with expected results. Max 3,000 tokens.
3. IDA: decode §7 RequestToken (sub_1400E8190) and RequestGameToken (sub_1400E7F50): body, token format, lifetime in ms, storage (CAppToken, CGameToken). Then the ❓ item "how LoginStart gets the UserId in `t`". Update the Rust card with the token part.

## Files
- `Docs/Platform/StServers/15-authsrv.md`
- `Docs/Plan/cards/auth-1b-rust.md` (new)

## Out of scope
- ExAuth, OTP, PC-cafe, billing. Do not change any file under `Program/`. Write `keyhere` for any key.
