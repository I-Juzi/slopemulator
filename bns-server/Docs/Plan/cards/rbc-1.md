# Card rbc-1: route bus client, spec (IDA)

## Goal
Write the spec of the route bus client: the shared StServers framework code that connects a service to its RouteSrv and registers it. The next slice (Sonnet, no IDA) writes the Rust from your spec only.

## Context
- StServers 1.20.3350 (NCSoft STS/NP platform). No PDB. Each service exe ≈ 2 MB shared framework + a small service part. The route bus client is in the shared framework.
- RouteSrv side is already specified: `Docs/Platform/StServers/13-routesrv.md` (174 lines; read §1 lines 7–21, §3 lines 52–86, §5 lines 99–123).
- DbSrv uses it: `Docs/Platform/StServers/12-dbsrv-contract.md` line 170: "DbSrv.c ~178913 looks up `RouteSrv` / `RouteSrv.<name>` and connects (`Can't find RouteSrv(%s) for %s(%u)`)".
- `Bin/Config/StServer.ini` (in `/home/ubuntu/Documents/Codex/private-project/BNSX-files/D/StServers/Bin/Config/`) has lines like `RouteSrv = 6605, 1` and `DbSrv = 6605, 1, AcctDb`.
- Existing Rust types: `Program/Platform/StServers/StsCore/src/route.rs` (RouteCmd, RouteMessage, Location, Target), `connect.rs` (ConnectInfo). Read only the pub items (`rg -n 'pub ' <file>`).

## Inputs
| What | Path |
| --- | --- |
| IDA copy (open ONLY this) | `/home/ubuntu/Documents/Codex/private-project/ida/lanes/rbc-1/DbSrv.exe.i64` |
| Start point | string `Can't find RouteSrv(%s) for %s(%u)` and its xrefs |

## Work
1. Find how a service selects its RouteSrv: config keys, `RouteSrv` vs `RouteSrv.<name>`, address and port source.
2. Find the connect sequence: TCP connect, `Sts/Connect` identity fields, then each route command in order (RegisterProvider, SubscribeService, CheckRegistration, others). Give fields and values for each.
3. Find replies and events the client handles: ProviderFound, ProviderLost, BlocksFound, ServerShutdown, errors. Give the action for each.
4. Find timers: reconnect delay and backoff, keepalive or CheckRegistration period, request timeout. Give the values in ms.
5. Find shutdown: UnregisterProvider or ShutdownServer, and the order.
6. Find how a request from the service to another protocol goes through the bus (request line, `t` header, `p` header), only the client side.
7. List log strings with their meaning. They are anchors for tests.

## Files
- `Docs/Platform/StServers/14-route-client.md` (new spec)
- `Docs/Plan/cards/rbc-1-rust.md` (Rust card for the next slice; use `Docs/Plan/card-template.md`)

## Rust card requirements (rbc-1-rust)
- Target crate: `sts-core` (path `Program/Platform/StServers/StsCore`), new module `route_client`. Use tokio. Reuse the route.rs and connect.rs types.
- Tests run against the real `routesrv` crate in process (`Program/Platform/StServers/RouteSrv`, a dev-dependency) or a fake listener.
- Give exact spec line ranges, Rust signatures and at least 6 test cases with expected results.
- Max 3,000 tokens.

## Out of scope
- Multi-router, forward scope, TLS (write `keyhere` for any key).
- Rust code. Do not change any file under `Program/`.
