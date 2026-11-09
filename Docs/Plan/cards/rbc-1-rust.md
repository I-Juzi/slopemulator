# Card rbc-1-rust: Route bus client in sts-core

## Goal
Add module `route_client` to crate `sts-core`. A service uses it to connect
to one RouteSrv, register its protocols, get provider events, and send
requests through the bus.

## Context
- Crate: `Program/Platform/StServers/StsCore` (package `sts-core`), tokio.
- The client is phase 1: one router, no TLS, no forward scope.
- Rust `routesrv` replies OK to `Sts/Connect` and does not send
  RouterConnect (cmd 21). It never returns 3053. So: start registration on
  RouterConnect, or on the Sts/Connect OK reply if no RouterConnect comes
  in 200 ms (spec §3.3 note). Use the first that occurs. Do not register two times.
- `routesrv` depends on `sts-core`. A dev-dependency back on `routesrv` is
  a cycle that cargo accepts. Tests talk to it over TCP only, so the two
  copies of sts-core types do not mix.
- TLS files: write `keyhere`. Do not read real config files.

## Inputs
| What | Path | Lines or section |
| --- | --- | --- |
| Spec, selection | Docs/Platform/StServers/14-route-client.md | §2, lines 21–93 |
| Spec, connect | same | §3, lines 94–144 |
| Spec, registration | same | §4, lines 145–195 |
| Spec, router messages | same | §5, lines 196–235 |
| Spec, bus requests | same | §6, lines 236–259 |
| Spec, shutdown | same | §7, lines 260–272 |
| Route command table, `t` rule | Docs/Platform/StServers/13-routesrv.md | §3 lines 52–86, §5 lines 99–123 |
| Types to reuse | StsCore/src/route.rs | `RouteMessage`, `RouteCmd`, `Location`, `Target`, `Topology` |
| Types to reuse | StsCore/src/connect.rs | `ConnectInfo`, `CONN_*` |
| Transport | StsCore/src/conn.rs | `Peer::spawn/call/send`, `Incoming`, `request`, `reply_ok`, `reply_error`, `DEFAULT_TIMEOUT` |
| Ini reader | StsCore/src/ini.rs | `Ini`, `Section` |
| Test router | StServers/RouteSrv/src/server.rs | `RouteServer::new`, `serve` |

## Work
1. Add `pub mod route_client;` to `StsCore/src/lib.rs`.
2. Parse StServer.ini (§2.1) and select the router (§2.2, §2.3). Use the
   first matching address. Errors use the log text of the spec as message.
3. Parse Route.ini `[General]` with the defaults of §2.4.
4. Connect: TCP, then `POST /Sts/Connect` with body in §3.1 order,
   ConnType 404. Watchdog 10000 ms (§3.2).
5. Registration (§4): send all RegisterProvider, then SubscribeService,
   at the same time, 25000 ms each. Do the result table of §4.2 and the
   timers of §4.4. Send `RouteEvent::Registered` or `RegistrationFailed`.
6. Router messages (§5): RouterConnect, ShutdownServer, ProviderFound,
   ProviderLost, unknown cmd → error 10. Reply OK where the spec says so.
7. Disconnect (§5.3): send `RouteEvent::Unregistered` if registered,
   reconnect after `reconnect_delay` (default 3000 ms), register again.
8. `request()` (§6): add `p: *<own location>` if absent; not connected →
   queue up to 10485760 bytes, fail with error 42 after 3000 ms.
9. Incoming requests for own protocols go to `RouteEvent::Request`.
10. `shutdown()` (§7): send ShutdownServer with ShutdownTimeoutMs, then close.
11. Print the log strings of the spec with `eprintln!` (no new crate).

## Interfaces
```rust
pub struct SystemEntry { pub server_name: String, pub addr: SocketAddr,
    pub remote_addr: Option<SocketAddr>, pub app_index: u32,
    pub config_name: String, pub route_config_name: String,
    pub use_tls: bool, pub use_remote_ip: bool, pub system_type: String }
pub fn parse_systems(ini: &Ini) -> Result<Vec<SystemEntry>, StsError>;
pub fn router_name(own: &SystemEntry) -> String; // "RouteSrv" or "RouteSrv.<x>"
pub fn select_router(entries: &[SystemEntry], program: &str, app_index: u32)
    -> Result<SocketAddr, String>;

#[derive(Clone, Debug)]
pub struct RouteIni { pub shutdown_timeout: Duration, pub registration_retry: Duration,
    pub registration_retry_limit: u32, pub check_registration_retry: Duration,
    pub check_registration_retry_limit: u32 }
impl Default for RouteIni { /* §2.4 values */ }
impl RouteIni { pub fn from_ini(ini: &Ini) -> Self; }

pub struct Registration { pub name: String, pub build: u32,
    pub topology: Topology, pub block_bits: u32 }

pub struct RouteClientConfig { pub router: SocketAddr, pub identity: ConnectInfo,
    pub own: Location, pub route_ini: RouteIni,
    pub providers: Vec<Registration>, pub subscriptions: Vec<String>,
    pub reconnect_delay: Duration, pub watchdog: Duration }

#[derive(Debug)]
pub enum RouteEvent { Registered, RegistrationFailed, Unregistered,
    ProviderFound { name: String, provider: Location },
    ProviderLost { name: String, provider: Location },
    Request(Incoming) }

#[derive(Clone)]
pub struct RouteClient { /* Arc inner */ }
impl RouteClient {
    pub fn start(cfg: RouteClientConfig) -> (Self, mpsc::UnboundedReceiver<RouteEvent>);
    pub fn is_registered(&self) -> bool;
    pub async fn request(&self, f: Frame, timeout: Duration) -> Result<Frame, StsError>;
    pub async fn shutdown(&self);
}
```

## Files
- `Program/Platform/StServers/StsCore/src/route_client.rs` (new)
- `Program/Platform/StServers/StsCore/src/lib.rs`
- `Program/Platform/StServers/StsCore/tests/route_client.rs` (new)
- `Program/Platform/StServers/StsCore/Cargo.toml` (dev-dep `routesrv`, tokio test features)

## Acceptance
| Test name | Input | Expected result |
| --- | --- | --- |
| select_default_router | ini: `[Systems] Addr = 10.10.66.88, Test`; `[Systems.Test] RouteSrv = 6605, 1` and `DbSrv = 6605, 1, AcctDb` | `select_router(.., "DbSrv", 1)` = `10.10.66.88:6605` |
| select_named_router | own entry col 3 = `R2`; two RouteSrv lines, ConfigName `R2` on port 6606 | router_name = `RouteSrv.R2`; address port 6606 |
| select_missing_own | no DbSrv line | Err text contains `in stserver.ini` |
| select_bad_port | `RouteSrv = 70000, 1` | Err text contains `invalid port number on line` |
| route_ini_defaults | empty ini | 60000, 3000, 1200, 5000, 720 |
| connect_body | fake listener reads first frame | `POST /Sts/Connect`; body ConnType 404, Address, ProductType, AppIndex, Epoch, Program, Build, Process; no NotifyFlags |
| register_with_routesrv | in-process `RouteServer` on 127.0.0.1:0; client registers `Echo` Basic | `Registered` event; `snapshot("Echo")` has 1 provider = own location |
| subscribe_gets_found | client A registers `Echo`; client B subscribes `Echo` | B gets `ProviderFound{name:"Echo", provider: A.own}` |
| retry_on_pending | fake router: 3053 to RegisterProvider, OK to CheckRegistration | client sends `POST /Route/CheckRegistration` with same body; then `Registered` |
| retry_limit | fake router: always 3053; check limit 2, `start_paused` | `RegistrationFailed` |
| unknown_route_cmd | fake router sends `POST /Route/GetMembership` | reply error 10 |
| watchdog | fake router accepts, never replies; watchdog 10000 ms, paused time | socket closed after 10000 ms, reconnect after reconnect_delay |
| request_adds_p | registered client sends `POST /Echo/Ping` with `t: $1` to fake router | frame has `p: *127.0.0.1:<port> ...` |
| request_not_connected | no router; `request()` | Err code 42 after 3000 ms (paused time) |
| shutdown_sends | `shutdown()` with fake router | router gets `POST /Route/ShutdownServer`; client closes |

Run `cargo test -p sts-core` and `cargo clippy -p sts-core -- -D warnings`.

## Out of scope
- Multi-router, forward scope (`@scope:`, RegisterForward), TLS, block
  transfer (cmd 8–11), StServer.ini reload (§8).
- Changes to `routesrv` or other crates.
- Reading real keys or certificates. Write `keyhere`.
