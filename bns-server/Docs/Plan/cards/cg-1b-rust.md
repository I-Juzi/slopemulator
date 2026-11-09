# Card cg-1b-rust: CliGate crate, phase 1

## Goal
Add crate `cligate` (lib + binary `CliGate`). It accepts game clients, does
the SRP login through AuthSrv on the route bus, keeps the session state, and
does logout, `Cli/Logout` and the idle timeout.

## Context
- Path `Program/Platform/StServers/CliGate`, package `cligate`. Deps: tokio,
  `sts-core` (conn, route_client, error, xml, ini), `sts` (`sts::rc4::Rc4`).
  Dev-dep: `routesrv`.
- Phase 1: no IP check (every socket in state 2, spec §9.2). No TLS, no
  LoginTokenStart certificate (reply ErrBadData 4, line 1466). Presence step 3
  is local (§10.3).
- After the KeyData `Complete` reply is written, all client bytes in both
  directions go through RC4 with the decoded SessionKey (§4, §6).
- CliGate registers provider `Cli` on the bus and receives `Cli/Logout` as
  `RouteEvent::Request`.
- Phase 1: `Cli/Logout` without `<Force>` acts the same as with `<Force>`
  (the client request name is ❓).
- Errors: `StsError::new(code, module::CLI_GATE, line)`. New codes:
  3007 ErrUserNotOnline, 3018 ErrClientNotFound.
- Write `keyhere` for any key. Tests make their own test session key bytes.

## Inputs
| What | Path | Lines or section |
| --- | --- | --- |
| Spec, wire, errors | Docs/Platform/StServers/11-cligate-protocol.md | §1, lines 7–35 |
| Auth ids | same | §2, lines 36–51 |
| Dispatcher | same | §3, lines 52–81 |
| Login, LoginTokenStart, RC4 | same | §4–§6, lines 82–127 |
| Connect identity | same | §7, lines 128–139 |
| Forward, `t`, reply path | same | §9, lines 148–214 |
| LoginFinish chain | same | §10, lines 215–281 |
| RequestToken, gate path | same | §11, lines 282–324 |
| Session lifecycle | same | §12, lines 325–402 |
| Transport | StsCore/src/conn.rs | `FrameReader`, `write_frame`, `request`, `reply_ok`, `reply_error`, `header`, `set_header`, `DEFAULT_TIMEOUT` |
| Bus | StsCore/src/route_client.rs | `RouteClient::start/request`, `RouteEvent`, `RouteClientConfig`, `Registration` |
| Target | StsCore/src/route.rs | `Target`, `Location` |
| Test router | RouteSrv/src/server.rs | `RouteServer::new`, `serve` |

## Work
1. Add `"Program/Platform/StServers/CliGate"` to workspace members.
2. `auth_cmd.rs`: enum `AuthCmd` with the ids of §2 (at least the 16 named ids).
3. `session.rs`: `Session` and the state table of §12.1, `check()` with the
   precheck errors of §3 and §10.1, `reset()` of §12.2.
4. `crypt.rs`: `CryptStream<S>` (AsyncRead + AsyncWrite) that applies RC4 in
   both directions after `enable(key)`. Keep the `FrameReader` leftover bytes
   (`into_parts`) and decrypt them on enable.
5. `conn.rs`: one task per socket. Read client frames, dispatch Auth commands
   (§3), forward to the bus with `t` = `user_target()` (§9.3), reply with the
   client request id (§9.4). LoginStart failures: close at 3 (§9.4).
6. LoginFinish: step 1 Auth/LoginFinish, step 2 Access/GetAppUserRights
   (`t` = `0$<UserId>`), step 3 local; reply body of §10.4.
7. LogoutMyClient (§12.3), idle timeout (§12.5), close → `reset()` log.
8. `gate.rs`: `Clients` map client id → conn handle. `Cli/Logout` (§12.4).
9. `main.rs`: read StServer.ini and Route.ini, start `RouteClient`, listen.
10. Log with `eprintln!` and the strings of §12.7.

## Interfaces
```rust
// auth_cmd.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum AuthCmd { GetMyUserInfo = 18, KeyData = 21, LoginStart = 22, LoginFinish = 23,
    RequestGameToken = 30, RequestToken = 32, CreateAccount = 46, TokenKeyData = 58,
    LoginTokenStart = 96, LogoutMyClient = 97, WaitForAllowedLogin = 99 /* … */ }
impl AuthCmd { pub fn from_name(s: &str) -> Option<Self>; pub fn id(self) -> u32; }

// session.rs
pub const ERR_USER_NOT_ONLINE: u32 = 3007;
pub const ERR_CLIENT_NOT_FOUND: u32 = 3018;
pub const MIN_IDLE_TIMEOUT_MS: u64 = 300_000;
pub fn idle_timeout_ms(seconds: Option<i64>) -> u64;          // §12.5
pub fn user_target(center: u32, user_id: &str) -> String;     // "<center>$<UserId>"
pub fn reset_log_line(logged_in: bool, user_id: &str, addr: &str, login_name: &str) -> String;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionState { Connected, LoginStarted, KeyDataComplete, LoggedIn }
#[derive(Default)]
pub struct Session { pub session: Option<String>, pub user_id: Option<String>,
    pub user_center: u32, pub user_name: String, pub login_name: String,
    pub key_complete: bool, pub login_time: u32, pub login_failures: u32 }
impl Session {
    pub fn state(&self) -> SessionState;
    pub fn check(&self, cmd: AuthCmd) -> Result<(), StsError>;
    pub fn reset(&mut self, addr: &str) -> String; // returns the log line
}

// crypt.rs
pub struct CryptStream<S> { /* inner, Option<(Rc4, Rc4)>, … */ }
impl<S: AsyncRead + AsyncWrite + Unpin> CryptStream<S> {
    pub fn new(inner: S) -> Self; pub fn enable(&mut self, key: &[u8]);
}

// gate.rs
#[derive(Clone)]
pub struct GateConfig { pub server_id: u32, pub center: u32,
    pub idle_timeout: Duration, pub auth_timeout: Duration /* 25 000 ms */ }
pub struct CliGate { /* RouteClient, Clients, GateConfig */ }
impl CliGate {
    pub fn new(cfg: GateConfig, route: RouteClient) -> Arc<Self>;
    pub async fn serve(self: Arc<Self>, listener: TcpListener) -> std::io::Result<()>;
    pub async fn handle_bus(self: Arc<Self>, events: mpsc::UnboundedReceiver<RouteEvent>);
    pub fn client_ids(&self) -> Vec<u32>;
}
```

## Files
- `Cargo.toml`
- `Cargo.lock`
- `Program/Platform/StServers/CliGate/Cargo.toml` (new)
- `Program/Platform/StServers/CliGate/src/lib.rs` (new)
- `Program/Platform/StServers/CliGate/src/main.rs` (new)
- `Program/Platform/StServers/CliGate/src/auth_cmd.rs` (new)
- `Program/Platform/StServers/CliGate/src/session.rs` (new)
- `Program/Platform/StServers/CliGate/src/crypt.rs` (new)
- `Program/Platform/StServers/CliGate/src/conn.rs` (new)
- `Program/Platform/StServers/CliGate/src/gate.rs` (new)
- `Program/Platform/StServers/CliGate/tests/cligate.rs` (new)

## Acceptance
Test setup: in-process `RouteServer` on 127.0.0.1:0. A fake AuthSrv is a
second `RouteClient` that registers `Auth` and `Access` and answers
`RouteEvent::Request`. If key routing of `t` does not reach it, use a fake
listener in place of RouteSrv and write a ❓ in the handoff.

| Test name | Input | Expected result |
| --- | --- | --- |
| auth_cmd_ids | `from_name("LogoutMyClient")`, `("KeyData")` | 97, 21; `from_name("Nope")` = None |
| idle_timeout_min | `idle_timeout_ms(None)`, `(Some(100))`, `(Some(600))` | 300000, 300000, 600000 |
| reset_log_tags | `reset_log_line(true, "U", "1.2.3.4", "n")`, then `false` | starts `LOGOUT, U, 1.2.3.4, n`; then `DISCONNECT, …` |
| user_target_form | `user_target(1, "0A0B…")` | `1$0A0B…` |
| state_checks | `Session::default().check(KeyData)`; state LoginStarted `.check(LoginFinish)` | Err 49; Err 49 line 1681 |
| logout_not_logged_in | client sends `POST /Auth/LogoutMyClient` after connect | status 400, code 3007, line 1736; socket stays open |
| login_flow | LoginStart → KeyData (fake replies `<Complete/><SessionKey>` base64 of 16 test bytes) → LoginFinish (fake `<Licenses/>`, Access `<AccessMask>1</AccessMask>`) | KeyData reply in clear; LoginFinish frame RC4; reply 200 with `<UserId>`, `<UserCenter>`; fake saw `t: <center>$<UserId>` on KeyData |
| login_start_three_fails | fake answers LoginStart with error 3 times | client gets each error; socket EOF after the third |
| logout_after_login | `login_flow`, then LogoutMyClient | 200 OK, then EOF; log line starts `LOGOUT` |
| cli_logout_force | after login, bus `Cli/Logout` `<ClientId>` = `client_ids()[0]`, `<UserId>`, `<Force/>` | requester gets OK; client socket EOF |
| cli_logout_unknown | bus `Cli/Logout` with ClientId 999 | error 3018 |
| idle_close | `idle_timeout` 200 ms in `GateConfig`, client sends nothing | socket EOF in under 1 s |
| auth_timeout | fake never answers LoginStart; `auth_timeout` 300 ms | client error code 42; socket open |

Run `cargo test -p cligate` and `cargo clippy -p cligate -- -D warnings`.

## Out of scope
- AuthSrv logic, IP check, TLS, LoginTokenStart keys, Presence service,
  CreateAccount, secondary auth.
- Changes to `sts-core`, `routesrv`, `sts` or other crates.
- Reading real keys or certificates. Write `keyhere`.
