# 04 — World connection, build 381

## State and evidence

Partial static specification. No connection test ran.
Client source identity: `Client-Common-0.0.381` in the allowed `Client_copy.i64`.
Server source identity: `GameDaemon-Common-0.0.381` in the allowed `GameDaemon.exe`.
Addresses below use each database's loaded image base.

The client database has inconsistent symbols. For example, `HandleEnterZoneOkPacket` at `0xF3A860` contains UI-panel code. Some packet constructors share a misleading `L_GetPcCafeMileageFeverTimeScheduleListReqPacket` vtable name. Do not use these names alone to identify a command. The tables below separate symbol evidence from instruction evidence.

## Connection ownership

| Step | Verified action | Evidence |
| --- | --- | --- |
| Lobby approval | The existing lobby path issues the world token. D25 is an internal daemon-to-gate command, not a game-client command. | [Lobby entry](../LobbyDaemon/06-world-entry.md); [link](../LobbyDaemon/07-gamed-link.md) |
| Client handoff | Lobby event 26 passes an address, port, two 64-bit values and a short to `BNSClient::EnterGameForNPWithLobbyGate`. | `Lobby::ProcessLobbyEvent2`, `0xD34230`; call `0xD346B3` |
| Separate session | The entry function stores the two 64-bit values at game session +0x38/+0x40 and the short at +0x48. It queues a connection with the supplied address and port. | `BNSClient::EnterGameForNPWithLobbyGate`, `0x1157EE0`; `0x5D2680`; connection task constructor `0x5D5FC0` |
| Server listener | GameDaemon opens client port type 1. The TCP port comes from configuration object +144. | `0x14033CBF0`, call to `0x1406A8360` |
| No lobby relay | The client uses a separate game session. The game listener handles Hello and world-token authentication. This is not ordinary game traffic through LobbyGate. | Client `0x1157EE0`, `0xF2C0C0`; GameDaemon `0x1403703D0`, `0x140371170` |

❓ An external deployment can put a proxy in front of this listener. These binaries do not establish that deployment topology.
No endpoint value is recorded. The client gets the endpoint through the lobby event. It does not use a fixed port in the checked entry function.
❓ The complete gate-to-client reply layout that populates event 26 was not traced. Do not mistake internal D25 for that complete endpoint reply.

## Initial game commands

Full formats include command `h`, but not the outer size field.
Use the notation in [Lobby link, G4](../LobbyDaemon/07-gamed-link.md#field-notation).
`h/E/C=2`, `i=4`, `q=8`, `Q/c=1` bytes. Scalars use little-endian order.
`s` is counted UTF-16LE. `b` is a counted byte string.

| Command | Direction | Name / full format | Ordered application fields | Evidence |
| --- | --- | --- | --- | --- |
| 11 | Client → GameDaemon | `G_HelloReqPacket`; `hhqsqb` | protocol version:h, property version:q, publisher:s, capability/version bits:q, handshake blob:b | Client game-open callback `0xF2C0C0`; server `0x1403703D0` |
| 12 | GameDaemon → client | HelloOk; `hb` | handshake blob:b | Server `0x1403703D0`; client `HandleHelloOkPacket`, `0xF70280` |
| 23 | Client → GameDaemon | LobbyTokenAuthReq; `hhqq` | world_id:h, client_key:q, lobby_token:q | Server `0x140371170`; unauthenticated table `0x1407D6D50`, slot 23 |
| 24 | GameDaemon → client | ❓ LobbyTokenAuthOk | ❓ Complete body not recovered. Client tests a byte at packet-object +0x10. This is not a wire offset. | Client callback `0xF3AF90`; receive table `0x149EC40`, slot 24. Dispatcher `0xF44030` uses command × 4. |
| 25 | GameDaemon → client | LobbyTokenAuth failure; `hh` | reason:h | Server `0x140371170` |

The command-12 client callback also calls constructor `0xF2BEF0`.
That constructor writes command 929 (`0x3A1`) with no visible application fields.
❓ Its purpose and its position relative to token authentication remain open.
Do not assume that HelloOk directly sends command 23.

The game protocol version is **1278**, not LobbyGate version 58.
Client `GameSession::Version`, `0x47D790`, returns 1278.
The server checks both the protocol version and a fixed property-version identifier.
The client Hello sender sets the same property-version identifier.
Sources: client `0xF2C17E`–`0xF2C18B`; server `0x1403703D0`.
The publisher string and capability bits must also pass server checks.
No publisher or capability sample is needed in the repository.

The token request checks a configured world list and a nonzero client key.
It starts `LobbyTokenAuthTran` at `0x14014C7B0`.
The transaction contacts LobbyDaemon to consume the token.
The existing LobbyDaemon link specification gives command 5 as `hqqqi` with transaction, client key, token and observed client address.
❓ This job did not close every build-381 transaction branch or its command-24 success body.
The token is an opaque single-use admission value. It is not a packet cipher key.
The client sends world ID, client key and token. GameDaemon obtains the client address from the accepted session. It is not a fourth field of command 23.

## Handshake and framing boundary

GameDaemon has its own Hello exchange. Do not send LobbyGate commands 1/2 or version 58 on this connection.
The Hello handler passes the opaque blob to a handshake-parameter operation. It replies with an opaque blob and changes packet-processor state.
Source: `0x1403703D0`; operation call `0x1406A65D0`.
No cryptographic internals or key bytes were inspected for this specification.

| Topic | State |
| --- | --- |
| Algorithm families present in client packet library | RSA and AES; symbols `CryptoKey::generateRsaKey` at `0x6081D0`, `Aes128::GetType` at `0x611680`. ❓ Exact game-handshake family chain was not verified. |
| Secret ownership | ❓ Exact game-channel ownership was not verified. Do not infer it from LobbyGate. No secret material was read or recorded. |
| Vendor acceptance | ❓ Whether genuine-client acceptance requires an unavailable vendor secret remains open. Stop that path if such a dependency is found. |
| Logical packet | Dispatcher reads command at decoded packet +2. Reuse the existing logical size/command codec. Server dispatcher `0x14007BC00`. |
| Outer game frame | ❓ Full comparison with LobbyGate frame flags, cipher selection and size limits remains open. Reuse of the existing transport needs a focused compatibility check. |
| Application fragmentation | EnterWorldOk uses fragment mode 2. Reassemble before application dispatch. See `05-enter-world.md`. |

## Restricted boundaries

| Boundary | Address | Action |
| --- | --- | --- |
| Client anti-cheat state in `World::EnterWorld` | `0x741AB2`, reference to `CsAuthServerNumber` | Stop this function path. No anti-cheat work. |
| Client security-version getter on entry-request construction | Calls at `0xF3B00C` and `0xF3B1F6` | Record the boundary only. Do not inspect the getter. |
| Server zone-entry anti-cheat hook | Log reference `0x1405DF767`, containing function `0x1405DE040`; string `0x140DA1BF0` | Stop this function path. No authentication implementation. |
| Server command-18 conditional GameGuard hook | Receive handler `0x140370F10`; hook target `0x14036DE10` | Stop the hook path. Record only the ordinary ping fields. |
| Client command-32 security interface | Callback path at `0xEF68C8` | Stop this call path. |
| Licence code | ❓ No address established in the checked path | No licence analysis. |

No prohibited binary was opened. No binary was patched. No network operation ran.
