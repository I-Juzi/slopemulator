# Handoff auth-1: AuthSrv login and token spec (IDA)

## Done
- `Docs/Platform/StServers/15-authsrv.md` §1–§5: dispatch and handler ids, `CAuLoginTxn`, login session record, LoginStart (request, DB reply state 7), SRP-6a server (N = RFC 5054 1024-bit, g = 2, SHA-256, little-endian numbers, `hnum`, k, M1, M2, interleave), KeyData.

## Not done
- Spec §6 LoginFinish. Verified, not yet written: handler sub_1400E6380 finds the session by `<Session>`. With no SecondaryAuthSrv (dword_140283180 = 0) and no PC-cafe data it calls sub_1400F4250 and deletes the session. Reply: `LoginName`, `UserId`, `UserName`, `UserCenter`, `UserStatus`, `Roles`, `PcCafe/PcCafeCode` (if set), `PasswordUntil` (if set), `ExternalAccounts`.
- Spec §7 RequestToken (sub_1400E8190), RequestGameToken (sub_1400E7F50): not decoded. Token storage strings: AuToken.cpp, `CAppToken`, `CGameToken`.
- Spec §8 DB calls. Verified, not yet written: `AcctLoginInfoById` on logical DB `AcctDb`, schema `v2016102001`, one param `<G>` = UserId as `%08X-%04X-%04X-%02X%02X-…` (sub_140112570, sub_140128270), procs.tsv row 18. Row columns: `UserId`, `UserCenter`, `UserName` (sub_140128A20), `Salt` and `Verifier` as Base64 (sub_14012CA30), `PasswordUntil`, `ValidateType`, `Validated` (sub_1400F57B0, sub_1400F5630), `Roles/RoleId`.
- `Docs/Plan/cards/auth-1-rust.md` (Rust card) is not written.

## ❓
- How LoginStart gets the UserId in `t`: AuthSrv needs `t` with a UserId (`AuMain: missing userId`, sub_1400D7710). AuthSrv registers key `login` → `$Name.GetIdByLoginName` (sub_1400D7840, sub_140157E70). The resolver path is not decoded.
- Interleave removes non-zero first bytes (sub_14012F030). The client side is not checked.
- Error code 3317 (`no salt verifier`) is not in errcodes.tsv.
- Pre-checks sub_1400C94A0, sub_1400C9180, sub_1400DF820 are not decoded.

## Blocked
- None. The session stopped before the work was complete.
- Gate check: I could not run `tools/slice_runner.py` (Bash permission denied). Manual check of `gate()`: changed files are only `15-authsrv.md` and this handoff (both allowed), no binary files, no keys.
