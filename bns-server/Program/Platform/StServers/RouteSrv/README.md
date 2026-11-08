# RouteSrv, phase 1

Source: `Docs/Platform/StServers/13-routesrv.md` and the existing Rust crates.
This crate does not use binaries or decompiled source.

```sh
cargo run -p routesrv --bin RouteSrv -- --config Route.ini --listen 127.0.0.1:6605 --server-id 2
```

Cargo writes build output to `Program/output/`.

## Supported paths

| Path | Phase 1 behavior |
| --- | --- |
| `Sts/Connect` | Accept the shared header and XML identity forms. Remember the location. Return plain OK, as DbSrv does. Close an invalid or repeated Connect. |
| CheckRegistration | Return plain OK. |
| RegisterProvider / UnregisterProvider | Update the service table. Send ProviderFound / ProviderLost to subscribers. |
| SubscribeService | Subscribe once. Send the current providers after OK. |
| Snapshot | Return one service's metadata with the Snapshot mask. Use Protocol/Name to select the service. |
| SetBlocks | Validate Desired. Apply Desired to Current at once. Send BlocksFound to subscribers. |
| GetBlocks | Return a BlocksFound body with Current. |
| ShutdownServer | Remove the sender's providers. Keep the router running. |
| No target | Select a provider by round robin. |
| Direct / Key / Block | Resolve each local target. Send once per provider connection. A type-4 protocol attribute wins. |
| Reply relay | Preserve or set `p`. Require a matching Direct `t` on replies. Restore the caller id on every partial and final frame. Keep reply bodies and sequence numbers unchanged. |
| Cleanup | Remove mappings on final reply, caller loss, provider loss, or a 25000 ms deadline. Provider loss gives each pending caller 49/2/1160. |
| Config | Read timeout values and Network.* block-bit defaults and overrides. Do not use TLS or password settings. |

## Limits and unverified rules

These rules are local choices where the spec gives no algorithm or response shape.
They do not establish compatibility with the original server.

| Item | Rule |
| --- | --- |
| ❓ Key hash | FNV-1a over UTF-8 bytes. Use the low BlockBits bits. |
| ❓ Provider code | Use the local connection id, starting at 1. Codes fit in 24 bits. The local `RouteServer::snapshot` API exposes codes. The wire spec gives no code-assignment rule. |
| ❓ Automatic blocks | Assign blocks in provider-code order until SetBlocks supplies explicit ownership. Clear lost explicit owners to zero. |
| ❓ Command responses | CheckRegistration returns OK. Snapshot returns one service. GetBlocks returns BlocksFound. ShutdownServer removes the sender's providers. The spec lists fields but does not define these response shapes. |
| ❓ Command validation errors | Use code 10, module 2. Omit the source line because it is not in the spec. |
| ❓ Pending provider loss | Use error 49, module 2, line 1160. This path is not traced. |
| Allocation limit | Allow 0–16 block bits. This is a local memory limit. |
| Block transfers | Apply ownership in one router. No RouteXfer handshake or state migration. |
| Multi-router / remote center | Not supported. Return 49, module 2, line 1160. |
| Forward scope / RegisterForward | Phase 2. Return 49, module 2, line 1160. |
| TLS / StartTls | Not supported. Use `keyhere` for any future key or password example. |
| ReconnectTimeout / LeftoverTimeout | Parse and retain these settings. Phase 1 removes lost providers at once. It does not retain routers or reconnect state. |
| Transaction timeout | Drop the entry after 25000 ms. Do not accept late replies. No timeout reply is specified. |

Use a new RouteServer instance after shutdown. The server does not persist registrations.
