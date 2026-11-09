# Handoff rbc-1: route bus client spec

## Done
- Spec `Docs/Platform/StServers/14-route-client.md` from `DbSrv.exe` lane copy rbc-1:
  RouteSrv selection, Route.ini defaults, Sts/Connect body, watchdog,
  RouterConnect, registration and retry, router messages, bus requests,
  shutdown, log anchors. Each claim cites a function address.
- Rust card `Docs/Plan/cards/rbc-1-rust.md` (15 tests).

## Not done
- TLS (StartTls cmd 23), forward scope, block transfer, multi-router (out of scope).
- StServer.ini reload: described in §8, not in the Rust card.

## ❓
- Reply to RouterConnect and to ProviderFound/Lost: not explicit in IDA.
- Reconnect delay of the socket manager: not found. Card uses 3000 ms.
- Locality mask bits (local before remote) and UseRemoteIp address use.
- Manager flag 0x800 meaning; queue flush after connect.
- No UnregisterProvider on shutdown found.
- Rust `routesrv` does not send RouterConnect: card starts registration
  on Sts/Connect OK after 200 ms (port decision, not IDA).
- Dev-dependency cycle sts-core ↔ routesrv: cargo accepts it; check on build.

## Blocked
- None.
