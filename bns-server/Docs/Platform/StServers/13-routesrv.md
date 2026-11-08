# 13 — RouteSrv: registration and routing

Source: `RouteSrv.exe` and the RouteSrv client code inside `AuthSrv.exe`
(build 381 StServers from the BNSX image). Addresses are from those builds.
Facts marked ❓ are not verified.

## 1. Role

```
 service exe ──Sts/Connect──▶ RouteSrv ◀──Sts/Connect── service exe
   (provider of "Auth")          │                 (provider of "Session")
        ▲                        │ route by request-line protocol
        └──────── POST /Auth/LoginStart ◀── CliGate (client of "Auth")
```

- Each service opens one STS connection to its RouteSrv.
- A service is a *provider* of one or more protocols.
- A caller sends a normal STS request (`POST /<Proto>/<Cmd> STS/1.0`).
  RouteSrv picks a provider of `<Proto>` and forwards the request.
- RouteSrv errors use module 2 (CliGate uses module 1).

## 2. Route message body (`<Message>`)

Serializer: AuthSrv `sub_140118530`. Parser: RouteSrv `sub_1400FA0B0`
(`sub_1400FA490` checks the root name `Message`).

| Mask bit | Element | Type / format |
| --- | --- | --- |
| 0x10000 | `SubCmd` | Route command name (only for cmd 20) |
| 0x4 | `Router` | location (see §4) |
| 0x8 | `Provider` | location |
| 0x10 | `Client` | uint, left out when 0 |
| 0x20 | `Block` | uint, left out when 0 |
| 0x40 | `Flags` | uint, left out when 0 |
| 0x80 | `Protocol/Name` | string, left out when empty |
| 0x200 | `Protocol/Build` | uint |
| 0x400 | `Protocol/Topology` | `Invalid`/`Basic`/`Star`/`Grid`/`Extern` (0–4) |
| 0x100 | `Protocol/BlockBits` | uint, left out when 0 |
| 0x2000 | `Protocol/Current` | 2^BlockBits entries, `,`-separated (see below) |
| 0x4000 | `Protocol/Desired` | same format as Current |
| 0x1000 | `Protocol/Forwards` | `type="array"` of `Forward{Scope,NameDb,NotFound?,NoCache?,UseNoName?,UseLocalCenter?,Version?}` |
| 0x800 | `ProtocolTemplate` | `{Name, Build?}` |
| 0x8000 | `Providers`, `Routers` | `type="array"` of `Provider`/`Router` with attributes `code`, `location`, `serverFlags?`, `serviceFlags?` (record 52 bytes) |
| 0x8000 | `Connections` | `type="array"` of `Connection code= epoch= routerCode=` |

`Protocol` is written only when any of the bits 0x7780 is set.

Block entry (Current/Desired), one u32 per block:
- bits 0–23: provider id (written by `sub_14012A820`, text when ≠ 0),
- bits 24–26 and 27–30: two state digits; written as `/<d1><d2>` when either is ≠ 0.

## 3. Commands and their fields

Field mask per command: AuthSrv `sub_140118EF0`. Names: `sub_140118CF0`.

| Id | Command | Mask | Fields |
| --- | --- | --- | --- |
| 1 | CheckRegistration | 0 | — |
| 2 | Snapshot | 0x788 | Provider, Name, BlockBits, Build, Topology |
| 3 | ShutdownServer | 0 | — |
| 4 | RegisterProvider | 0x788 | Provider, Name, BlockBits, Build, Topology |
| 5 | UnregisterProvider | 0x788 | as 4 |
| 6 | RegisterForward | 0x1A80 | Name, Build, ProtocolTemplate, Forwards |
| 7 | SubscribeService | 0x80 | Name |
| 8 | SetBlocks | 0x4780 | Name, BlockBits, Build, Topology, Desired |
| 9 | GetBlocks | 0x180 | Name, BlockBits |
| 10 | RouteXferRequest | 0x7AA | t=`#Block`, Provider, Block, Name, BlockBits, Build, Topology |
| 11 | RouteXferBlock | 0x7A9 | t=`*Provider`, Provider, Block, Name, BlockBits, Build, Topology |
| 12 | BlocksFound | 0x2100 | BlockBits, Current |
| 13 | ProviderFound | 0x88 | Provider, Name |
| 14 | ProviderLost | 0x88 | Provider, Name |
| 15 | ServerShutdown | 0xC | Router, Provider |
| 16 | ServerDisconnect | 0xC | Router, Provider |
| 17 | ProviderChanged | 0x27EC | Router, Provider, Block, Flags, Name, BlockBits, Build, Topology, Current |
| 18 | ProviderShutdown | 0x28C | Router, Provider, Name, Build |
| 19 | ServiceSetForward | 0x1280 | Name, Build, Forwards |
| 20 | MultiRouterChanged | 0x10000 | SubCmd, plus the mask of SubCmd |
| 21 | RouterConnect | 0x204 | Router, Build |
| 22 | ServiceChanged | 0xF780 | Name, BlockBits, Build, Topology, Current, Desired, Providers, Routers, Connections |
| 23 | StartTls | 0 | — |

Request line (`sub_140118B60`):
- mask bits 0x1 or 0x2 set: `POST /<Protocol.Name>/<Cmd>` plus a `t` header
  (0x2 → `t: <center>#<Block hex>`, 0x1 → `t: *<Provider location>`).
- otherwise: `POST /Route/<Cmd>`.

## 4. Location format

Writer `sub_140145320`, parser `sub_14012E370`:

```
<ip[:port]> <u32 a> <u32 b> <u32 c> <u32 d>[ <extra u32>]
```
- The struct is 40 bytes: a socket address (24 bytes) and four u32 at
  offsets 24, 28, 32, 36. ❓ The four u32 match the Connect identity
  (product/index/process/epoch) — not verified.
- When the 5th number is missing, it is 0.

## 5. Target header `t`

Parser `sub_1400F9FC0`. One request may carry several `t` headers
(multicast; RouteSrv groups targets per router, `sub_1400CCD30`).

| Form | Type | Meaning |
| --- | --- | --- |
| `*<location>` | 1 Direct | one exact provider |
| `<c>$<key>` | 2 Key | hash of key → block → provider |
| `<c>@<scope>:<name>[;…]` | 3 Forward | look up scope in the protocol's Forwards; `random` picks a random address |
| `<c>#<hex>` | 4 Block | block number; attribute `protocol` names the protocol |

- `<c>` is a center (datacenter) number, 0–63. 0 = local center.
- Without `t`, the protocol of the request line selects the service
  (`sub_1400CBAD0`, default protocol `Sts`).

Routing errors (RouteSrv `sub_1400CCD30`/`sub_1400CAB70`):

| Code | Line | When |
| --- | --- | --- |
| 10 ErrBadParam | 1115, 1170 | message is a reply and cannot be routed |
| 49 | 1160, 1193, 1213 | no provider accepted the message |
| 48 | 1693 | random address failed |
| 10 | 1573 | invalid direct address |

## 6. Config (`Route.ini`)

| Key | Value in image | Meaning |
| --- | --- | --- |
| ReconnectTimeout | 30000 | wait for a lost provider |
| LeftoverTimeout | 30000 | wait for a lost router |
| `[Network.Test]` BlockMask | 7 | default block bits |
| `[Network.Test.Test]` BlockMask.<Proto> | 3–7 | per protocol block bits |
| `[TLS]` | certificate + EncryptedKey | secret → `keyhere` in the port |

## 7. Port scope

| Part | Port plan |
| --- | --- |
| Connect, RegisterProvider, SubscribeService, ProviderFound/Lost | full |
| Routing by protocol, Direct, Key, Block targets | full |
| Grid blocks (SetBlocks/GetBlocks/BlocksFound/Xfer) | full for one router |
| Multi-router Paxos (257–1030), MultiRouterChanged, RouterConnect | gap: one router only in phase 1 |
| Forward scope (`@`) | needs ForwardSrv/NameSrv; phase 2 |
| TLS between datacenters | gap; key = `keyhere` |

## 8. Forward path (trace in progress, 8 Nov 2026)

Header names in the STS library are one-byte codes, not strings.
Codes that RouteSrv.exe uses: `c` 99, `e` 101, `l` 108, `o` 111, `p` 112, `r` 114, `s` 115, `t` 116, `x` 120.

| Step | Function | Behaviour |
| --- | --- | --- |
| Pick the protocol | `sub_1400CBAD0` | A `t` target of type 4 (Block) with a `protocol` attribute wins. Otherwise the request-line protocol is used. The default is `Sts`. |
| Pick the service | `sub_1400CCD30` → `sub_1400DF980` | Looks up the protocol in the service table (router + 66360). A service with no providers (field +88 = 0) counts as none. |
| Resolve targets | `sub_1400CAB70`, loop on `sub_1400F9CC0` | Each `t` header is resolved. Several targets are grouped per router (multicast). |
| No target found | `sub_1400CCD30` | error 49, module 2, line 1160 |
| Reply frames | `sub_1400CCD30` | a reply cannot be routed: error 10, line 1115 or 1170 |
| No `t` match | `sub_1400CCD30` | line 1105 (code −1) or 1185 when a target list is empty |
| Deliver | `sub_1400CD6C0` | Local provider: the frame is copied (`sub_140102570`, `CInetMsgOut`) and sent on the provider connection (network vtable +32). Each send increments the connection counter at +180. A failed send increments stat 6. |
| Forward scope | `sub_1400F7BC0` / `sub_1400F8220` | Saves `protocol`, `protocolVersion` and `command` in an `r` header, and sets the command to `Forward`. The pop restores the original request line. Used for `@scope` targets through ForwardSrv. |

❓ Reply return: the frame copy keeps the request unchanged. The mapping of the provider's
reply back to the caller is in the STS socket layer (`StsInetSocket`, `sub_1401221A0`,
"invalid transaction reply" → error 4, module 9001, line 3488). Not traced yet.
