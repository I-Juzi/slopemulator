# Handoff cg-1b-rust

## Done
- Crate `cligate` (lib + bin `CliGate`) in the workspace. fmt, test (22) and clippy `-D warnings` pass.
- Unit tests: `auth_cmd_ids`, `idle_timeout_min`, `reset_log_tags`, `user_target_form`, `state_checks`, `reset_clears_and_logs`, 3 `crypt` tests.
- Integration tests (RouteServer + fake Auth/Access): `login_flow`, `login_start_three_fails`, `login_start_bad_userref`, `key_data_before_login_start`, `logout_not_logged_in`, `logout_after_login`, `cli_logout_force`, `cli_logout_without_force_acts_the_same`, `cli_logout_unknown`, `idle_close`, `auth_timeout`, `not_logged_in_other_protocol_is_refused`, `login_token_start_without_certificate`.
- Key routing of `t` works through RouteServer. No fake listener needed.
- Added `CliGate::log_lines()` (not in the card) so tests can read the `LOGOUT` line.
- Non-Auth protocol, not logged in: error 49, module 8000, line 2603.

## Not done
- Generic gate path for logged-in clients (§11.1): reply is error 10 line 2405.
- Auth commands other than the 7 handled: error 10 line 2405.
- `f` header on RequestToken/RequestGameToken. `AppId` of GetAppUserRights is empty.
- IP check, TLS, LoginTokenStart keys, Presence, CreateAccount.

## ❓
- `Incoming::reply` in sts-core sets no `t`. The router drops such replies. CliGate and the test fake set `t` = `p` by hand. Fix in sts-core?
- Line of the KeyData precheck error 49 (spec has none): 0 used.
- Body root names (`Request`, `Reply`), `NetAddress` = ip only, `Complete` copied to the client, KeyData error clears the session.
- Local errors (bad LoginName, no body) use line 0 and do not count as LoginStart failures.
- `IdleTimeout` section in StServer.ini (any section is searched). ConnType 404, program 101, build 381 in main.rs are test values.
- A bus request such as `Cli/Logout` waits while the client task waits for AuthSrv (up to 25 s).

## Blocked
- None.
