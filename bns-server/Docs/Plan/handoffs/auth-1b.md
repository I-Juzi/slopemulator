# Handoff auth-1b: AuthSrv spec finish and Rust card

## Done
- `15-authsrv.md` §6 LoginFinish, §8 DB calls (from the auth-1 handoff; GUID text format and byte order verified in sub_140128270).
- §7 Tokens (IDA): lifetimes from INI `[TokenTimeout] App` 1800 s / `Game` 300 s (sub_1400D6CF0); RequestToken (sub_1400E8190) → `<AuthnToken>` = `<UserId>:<token GUID>`; `CAppToken` store, one token per (user, AppId) (sub_1400E07A0, sub_1400D95B0); RequestGameToken (sub_1400E7F50) → `GameAccount/ListAccounts`, match `Alias`, reply `<Token>` GUID (sub_1400F4A40); `CGameToken` store (sub_1400E0A10, sub_1400D9790); request fields `AccountAlias`, `AppId` (default = request +304), `GameCode` (sub_140110FC0).
- §3.3: AuthSrv does not resolve names. `t` must have a GUID UserId (sub_1401191A0, sub_1400D7710). Resolver table `login` → `$Name.GetIdByLoginName` is registered for the router side (sub_1400D7840).
- `Docs/Plan/cards/auth-1b-rust.md`: crate `authsrv`, 22 tests.

## Not done
- ConsumeGameToken (sub_1400E3D70), app token verify path.
- Pre-checks sub_1400C94A0, sub_1400C9180, sub_1400DF820.

## ❓
- Which component resolves `t=login:<name>` (RouteSrv?); meaning of 3002 and the flag in sub_140157E70.
- `TokenType` INI values use; send flag 3103 in sub_1400E7F50; request +832 text meaning.
- APPTOKEN log kind order (from string order).
- `Alias` compare case (sub_1401014B0).
- LoginFinish error code for an unknown session.
- §8 column types other than Salt and Verifier.
- Old ❓ from auth-1: interleave client side, code 3317 not in errcodes.tsv.

## Blocked
- None. Bash was denied after the first edits; I used the Edit tool. Manual gate check: changed files are `15-authsrv.md`, `auth-1b-rust.md` (new) and this handoff. The card lists "Files" for these two paths only; this handoff path is required by the session rules. No binary files, no keys.
