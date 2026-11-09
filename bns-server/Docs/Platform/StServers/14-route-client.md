# 14 Route bus client (service side)

Source: `DbSrv.exe` (StServers shared framework, lane copy `rbc-1`).
Router side: see `13-routesrv.md`. Shared types: StsCore `route.rs`, `connect.rs`.
Mark ❓ = not verified in IDA.

## 1 Role

1. Each service (DbSrv, CacheSrv, and so on) has one route bus client.
   The client connects the service to RouteSrv and registers the
   service protocols (`sub_1401042F0` init).
2. The client has these parts:
   - socket manager `Srv2Route` (`sub_1400EDB50` call in `sub_1401042F0`).
   - connection class `CStsRouteConn` (ctor `sub_1400FCEA0`, vtable `0x140163948`).
   - registration engine `CStsRouteRegister` (`sub_1401032E0`, `sub_140102B80`).
   - config readers `CStsRouteServerIni` (`sub_140100A40`) and
     `CStsRouteRouteIni` (`sub_1401008B0`).
3. This spec covers one router only (phase 1). Multi-router, forward
   scope and TLS are out of scope.

## 2 RouteSrv selection

### 2.1 StServer.ini format (`sub_1400FA420`, `sub_1400FA190`, `sub_1400F7530`)

1. Section `[Systems]`: key `Addr`, value
   `<IpAddress>, <Type>[, <RemoteIpAddress>]` (`sub_1400FA420`).
   - Bad address: log `StsServer: invalid address on line %d`.
   - No type: log `StsServer: missing system type on line %d`.
2. Section `[Systems.<Type>]` (name made with `Systems.%s`, `sub_1400FA190`):
   key `<ServerName>`, value columns:
   | Col | Field | Note |
   |---|---|---|
   | 0 | Port | 1..65535, else log `invalid port number on line %d` |
   | 1 | AppIndex | number |
   | 2 | ConfigName | max 63 chars, else log `config name on line %d was more than %u characters` |
   | 3 | RouteConfigName | max 31 chars |
   | 4 | UseTls | 0/1 |
   | 5 | UseRemoteIp | 0/1 |
   Missing columns are empty or 0.
3. Unknown type: log `undefined system type, %S` (`sub_1400FA190`).
4. One entry = one address of `[Systems]` with that type, plus the columns
   (`sub_1400F7530`). The reader sorts the list and removes duplicates
   (`sub_1400FA660`).

### 2.2 Own entry (`sub_140100A40`)

1. The client finds its own entry: ServerName = own program name, AppIndex =
   own AppIndex (`sub_1400FB0E0` call in `sub_140100A40`).
2. No own entry: log (wide) `Remove %s(%u) in stserver.ini` and stop. No
   connection.
3. From the own entry the client reads RouteConfigName, UseTls and
   UseRemoteIp. If one of them changes on reload, the client reconnects (§8).

### 2.3 Router name and lookup

1. RouteConfigName empty: router name = `RouteSrv`.
2. RouteConfigName not empty: router name = `RouteSrv.<RouteConfigName>`
   (`sub_140100A40`).
3. Name rule `X.Y` (`sub_1400F64C0`, `sub_1400F6090`): split at the first `.`.
   ServerName must be `X`. ConfigName (col 2) must be `Y`. Name without `.`:
   no ConfigName filter.
4. The lookup is not filtered by AppIndex for the router (`sub_140100A40`).
5. Address = `<IpAddress of [Systems] line>:<Port>`. With UseRemoteIp the
   client uses the remote address ❓ (flag 2 path in `sub_1400F1FA0`).
6. Locality (`sub_1400F64C0`, flag `0x20000000`): the client tries local
   entries first (same /24 network as own address, entry flag +408). If no
   local entry: it uses remote entries and logs
   `%S: %s (%S)` with `no local servers found in config file; using remote servers`.
   No entry at all: log `no servers found in config file` and
   log level 2 (wide) `Can't find RouteSrv(%s) for %s(%u)` (string `0x140163ea0`,
   ref `0x140101109`). ❓ exact mask bits.
7. Many addresses: the manager removes duplicates and shuffles them. With
   flag 0x1 the manager keeps one connection only (`sub_1400F5AC0`).
   Phase 1 Rust: use the first address.
8. Explicit address argument (`sub_1401042F0`, manager vt+72): if the service
   gets a router address directly, it uses that address and does not read
   StServer.ini for the router.

Example: `[Systems.Test]` line `RouteSrv = 6605, 1` and `[Systems]` line
`Addr = 10.10.66.88, Test` give router `10.10.66.88:6605`.

### 2.4 Route.ini `[General]` (`sub_1401008B0`)

| Key | Default | Unit | Use |
|---|---|---|---|
| ShutdownTimeoutMs | 60000 | ms | ShutdownServer txn timeout (§7) |
| RegistrationRetry | 3000 | ms | request backlog period (§4.4) |
| RegistrationRetryLimit | 1200 | count | max request retries |
| CheckRegistrationRetryMs | 5000 | ms | check backlog period after 10 s (§4.4) |
| CheckRegistrationRetryLimit | 720 | count | max check retries |

Section `[TLS]` holds TLS files. Write `keyhere`. Out of scope.

## 3 Connect sequence

### 3.1 TCP and Sts/Connect (`sub_1400EDB50`, `sub_1400E9010`)

1. The client opens TCP to the router address.
2. The first frame is `POST /Sts/Connect STS/1.0` with an XML body
   `<Connect>` (`sub_1400E9010`). Elements in this order:
   | Element | Value | Rule |
   |---|---|---|
   | ConnType | 404 | always |
   | ConnProductType, ConnAppIndex, ConnDeployment, ConnEpoch | target | only if not zero; 0 for the router |
   | Address | own `ip:port` | always |
   | ProductType | own product (`sub_1400EF840`) | always |
   | AppIndex | own AppIndex (`sub_1400EF7C0`) | always |
   | Deployment | own deployment (`sub_1400EFE70`) | only if not zero |
   | Epoch | own epoch (`sub_1400EF850`) | always |
   | Program | own program id (`sub_140136BD0`) | always |
   | Build | own build (`sub_140136B90`) | always |
   | Process | own process id (`sub_1400EF820`) | always |
   | NotifyFlags, VersionFlags | 0 | only if not zero; omitted |
3. The router can also read this data from header `X-Sts-Connect`
   (`sub_1400EDCC0`). The client sends the body form.
4. Manager flags `0xD01` (`sub_1401042F0`): 0x1 one connection,
   0x100 add `p` header (§6), 0x400 queue requests while not connected,
   0x800 ❓.

### 3.2 Watchdog (`sub_140102220`, `sub_140102AA0`)

1. On TCP connect: log
   `StsRoute: CStsRouteConn::OnSocketConnect (%s) --> (%s)` (own, router).
2. The client starts a 10000 ms watchdog.
3. If the watchdog expires before RouterConnect: log
   `StsRoute: CStsRouteConn::OnTimer(), disconnect!` and close the socket.

### 3.3 RouterConnect (`sub_1400FE790`)

1. The router sends `POST /Route/RouterConnect` (cmd 21) to the client.
   Field `Router` = router location.
2. The client stops the watchdog, keeps the router location, and sets
   state 1.
3. UseTls = 1: StartTls (cmd 23), 25000 ms. Out of scope.
4. UseTls = 0: log
   `StsRoute: CStsRouteConn::DoRouterConnect, registration started`, then
   start registration (§4).
5. Reply to RouterConnect: ❓ (no explicit reply found).

Note: the Rust `routesrv` crate does not send RouterConnect. It replies OK
to Sts/Connect. The Rust client must also start registration on the
Sts/Connect OK reply when the router does not send RouterConnect ❓
(port decision, not from IDA).

## 4 Registration (`CStsRouteRegister`)

### 4.1 Requests (`sub_1401032E0`)

The client sends all requests at the same time. Each has timeout 25000 ms.
A pending counter counts them.

1. For each registered protocol, in registration order:
   `POST /Route/RegisterProvider` (cmd 4), fields:
   | Field | Value |
   |---|---|
   | Provider | own location (`sub_1400ED880` → `sub_1400EA0F0`), `<ip:port> p i d e` |
   | Name | protocol name |
   | Build | protocol build; 0 for Grid |
   | Topology | protocol topology (1 Basic, 2 Star, 3 Grid) |
   | BlockBits | from protocol handler, Grid only |
2. For each forward: RegisterForward (cmd 6). Out of scope.
3. For each subscription: `POST /Route/SubscribeService` (cmd 7),
   field `Name` = protocol name.
4. Body = route Message, field mask of the command (`13-routesrv.md` §3).

### 4.2 Results (`sub_140102B80`)

| Result | Action |
|---|---|
| 0 OK | Request done. Pending − 1. |
| 3053 ErrRouteRegisterPending on a request | Resend the same body as `POST /Route/CheckRegistration` (cmd 1, `sub_140129670`). Put it in the check backlog. |
| 3053 on a check | Check count + 1. Count > CheckRegistrationRetryLimit: fail, log `StsRoute: CStsRouteRegister::Retry: retry limit exceed, type: %s, try count: %u`. Else put it in the check backlog again. |
| other error | Log `StsRoute: txn(%s) err(%d)%s` (`%s` = `request` or `check`; suffix `, when s_shutdown(true)` during shutdown). Retry (`sub_1401031D0`): request count + 1; count > RegistrationRetryLimit: fail; else request backlog. |

When pending = 0 and both backlogs are empty: finish with success.
On fail: finish with failure (`sub_140100260`).

### 4.3 Finish (`sub_140102190`)

1. Log `StsRoute: CStsRouteConn::OnRouteRegisterFinished, registration finished(%s)`
   with `success` or `failure`.
2. Success: set registered, `s_registered` + 1, perf counter
   `route registered (0: false, 1: true)` = 1, call service callback
   "registered".
3. Failure: call service callback "unregistered".

### 4.4 Timers (`sub_140104D30`, `sub_140102AE0`, `sub_1401007E0`, `sub_1401039A0`)

| Timer | Value | Action |
|---|---|---|
| register timer | 10000 ms after RouterConnect | check backlog period becomes CheckRegistrationRetryMs; check retries start to count (`sub_140102AE0`) |
| check backlog period | 500 ms first, then 5000 ms | resend one check (`sub_1401007E0` → `sub_1401039A0`) |
| request backlog period | RegistrationRetry 3000 ms | resend one request |
| txn timeout | 25000 ms | error 42 → "other error" row |

## 5 Messages from the router (`sub_1401026E0`)

1. Cmd 10 and 11 always go to protocol Route. Other protocols go to the
   service handler (§6.3).
2. Route commands:
   | Cmd | Action |
   |---|---|
   | 3 ShutdownServer | §5.1 |
   | 8 SetBlocks, 9 GetBlocks | block handler (Grid). Out of scope ❓ |
   | 10 XferRequest, 11 XferBlock | block transfer. Out of scope |
   | 13 ProviderFound | §5.2 |
   | 14 ProviderLost | §5.2 |
   | 21 RouterConnect | §3.3 |
   | other | error reply 10 ErrBadParam, module 6000; log `StsRoute: unknown route command, %s` |

### 5.1 ShutdownServer from router (`sub_1400FF720`)

1. Reply OK.
2. Mark the socket inactive ❓ (socket vt+48).
3. Clear registered; call service callback "unregistered".

### 5.2 ProviderFound / ProviderLost (`sub_1400FE170`, `sub_1400FE480`)

1. Name empty: error reply 10, module 6000; log
   `StsRoute: provider found, invalid notice` or
   `StsRoute: provider lost, invalid notice`.
2. Else: find subscriptions with that Name. Call the subscriber
   "found" or "lost" callback with Provider location and Name.
3. A subscription without a callback is deleted.
4. Reply OK ❓ (not explicit in IDA). Rust: reply OK.

### 5.3 Disconnect (`sub_140102480`)

1. Log `StsRoute: CStsRouteConn::OnSocketDisconnect (%s) --x (%s)`.
2. Clear the router location. If registered: `s_registered` − 1, call
   "unregistered".
3. Reset registration state. The manager reconnects. Log
   `StsSocket: reconnecting deactivated socket` ❓ on reconnect.
4. Reconnect delay: not found ❓. Rust: 3000 ms.

## 6 Requests through the bus (`sub_1401044D0`, `sub_1400ECFA0`)

### 6.1 Send

1. Request line and `t` rule: `13-routesrv.md` §3, §5.
2. Forward scope rewrite (`$key` → `@scope:`, `sub_1400DEF60`): out of scope.
3. Connected (`sub_1400ED360`): the client sets the txn id. Flag 0x100 and
   no `p` header: add `p: *<own location>` (`sub_1400EA0F0`, format
   `<ip:port> p i d e[ x]` from `sub_1400F1FA0`).
4. Not connected, flag 0x400: queue (`sub_1400EC100`, max 10485760 bytes;
   oldest dropped). The queue is sent after connect ❓.
5. Not connected, no queue, or dropped: fail after 3000 ms with error 42.

### 6.2 Reply

The router returns the reply on the same TCP with the same txn id
(`13-routesrv.md` §8.1). Timeout default 25000 ms, error 42 module 9001.

### 6.3 Incoming request

A request for a service protocol arrives on the router TCP with `p` set.
The service handler replies on the same TCP with the same txn id
(`sub_1401026E0`, error helper `sub_1401042A0`).

## 7 Shutdown (`sub_140100510`, `sub_140101300`)

1. Connected: send `POST /Route/ShutdownServer` (cmd 3, empty body mask),
   timeout ShutdownTimeoutMs.
   - Takes ≥ 10 s: log `shutdown txn takes over %u sec`.
   - Error 42/9001: log `StsRoute: shutdown server txn timedout(%u), [%u:%u:%u]`.
   - Other error: log `StsRoute: Shutdown transaction`.
   - Then close the manager, set `s_shutdown`.
2. Not connected: if `s_registered` ≠ 0 log
   `StsRoute: s_registered(%d) still not 0 in RouterStartCleanup()`;
   set `s_shutdown`; close the manager.
3. No UnregisterProvider is sent ❓ (not found).

## 8 Config reload (`sub_140100A40`, `sub_1400E8C30`)

1. StServer.ini changes: the client reads it again.
2. Address list changes: set new list (`sub_1400F5AC0`).
3. RouteConfigName/UseTls/UseRemoteIp changes: reconnect (`sub_1400E8C30`).
   Queued requests fail with error 42, module 9001.
4. Log `StsRoute: Datacenter changed` ❓ (exact trigger).

## 9 Other log strings (test anchors)

| String | Meaning |
|---|---|
| `StsRoute: BuildCheckRegistrationMsg, parse msg data failed, line(%d)` | cannot make check body |
| `StsRoute: ParseMsgData parse failed, line(%d)` | bad stored body |
| `StsRoute: invalid type of txn, line(%d)` | bad txn type |
| `StsRoute: CStsRouteRegister::OnBackLogTimeout, invaild back log data` | bad backlog entry |
| `StsRoute: CStsRouteRegister::SendRequest invalid register job type, line(%d)` | bad job type |
| `/providers` page | debug page "List of registered providers" (`sub_1401042F0`) |
