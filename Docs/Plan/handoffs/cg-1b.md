# Handoff cg-1b

## Done
- 11-cligate-protocol.md §12 Session lifecycle (lines 325–402): state table, Reset,
  LogoutMyClient (97), bus `Cli/Logout`, idle timeout, AuthSrv lost, log strings.
- IDA check (2 calls): idle timeout log is in sub_1400CF050 (called from sub_1400D6030).
  Also log `Old ClientManager`. `Force` is read in sub_1400D0EE0 (called from sub_1400CEB30).
  Socket idle close log in sub_140138A90, with cfg `DisableIdleDisconnect`.
- Rust card `Docs/Plan/cards/cg-1b-rust.md`: crate `cligate`, 13 tests.

## Not done
- No Rust code (next slice cg-1b-rust).

## ❓
- Socket close call in LogoutMyClient (vtable+72).
- New client id after logout in Reset: exact call not decoded.
- `Cli/Logout` with `<Force>`: reply OK to requester not verified.
- Client request name for `Cli/Logout` without `<Force>`. Rust phase 1 treats it as Force.
- Presence command 14 name. Source of cfg `DisableIdleDisconnect`.
- Config section of `IdleTimeout`.
- AuthSrv lost: no special code found.
- Rust `routesrv` key routing of `t` = `<center>$<UserId>` to a fake AuthSrv: not checked.

## Blocked
- Bash was denied after the first calls, so I did not run `tools/slice_runner.py` checks.
