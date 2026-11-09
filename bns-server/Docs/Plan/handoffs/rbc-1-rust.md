# Handoff rbc-1-rust

## Done
Module `route_client` in `sts-core`, with tests in `StsCore/tests/route_client.rs`.
All 15 acceptance tests pass: select_default_router, select_named_router,
select_missing_own, select_bad_port, route_ini_defaults, connect_body,
register_with_routesrv, subscribe_gets_found, retry_on_pending, retry_limit,
unknown_route_cmd, watchdog, request_adds_p, request_not_connected, shutdown_sends.
`cargo fmt -p sts-core`, `cargo test -p sts-core` and
`cargo clippy -p sts-core --all-targets -- -D warnings` all pass (one run).
I did not repeat the tests to check for flaky runs (the shell denied the loop).

## Not done
- `cargo fmt` also changed seven other files in `StsCore/src` (conn, connect, error,
  ini, route, secrets, xml). Only the layout changed. The card does not list them.
  The gate can keep or revert these.
- `Cargo.lock` has one new line (dev-dependency `routesrv`).
- Out of scope parts are not built: multi-router, forward scope, TLS, block transfer, reload.

## ❓
- Ini reader has no line numbers. "line N" in errors is the entry position in its section.
- An Addr line with an unknown type, or a bad column, returns an error. The spec only says "log".
- IPv6 locality compares the first 8 bytes. The spec has no mask.
- No reply to RouterConnect (§3.3 item 5 has none). The router may wait for one.
- ShutdownServer from the router: reply OK, clear registered, send `Unregistered`.
  The socket stays open (§5.1 item 2 "inactive" is unclear).
- After `RegistrationFailed` the client stays connected and does not register again
  until the next reconnect.
- Replies of the service to `RouteEvent::Request` need a `t` header equal to the `p`
  header, or routesrv rejects them (relay.rs). `Incoming::reply` does not set `t`.
  The service must set it. Maybe add a helper in `conn.rs` later.
- Shutdown log numbers `[%u:%u:%u]` are written as code, module, line.

## Blocked
None.
