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
| Algorithm families | RSA handshake → AES-128 session. See the step-4 summary below. ❓ Full native-client acceptance remains unverified. |
| Secret ownership | GameDaemon constructs an RSA private object from static binary data and installs session-secret state. ❓ Full client ownership remains open. No secret material was read or recorded. |
| Vendor acceptance | Static private-object loading is a stop boundary at `0x1406E3100`. ❓ Whether genuine-client acceptance requires that unavailable vendor secret remains open. Do not continue key analysis. |
| Logical packet | Dispatcher reads command at decoded packet +2. Reuse the existing logical size/command codec. Server dispatcher `0x14007BC00`. |
| Outer game frame | Checked size/tag arithmetic matches `BloodLib/src/frames.rs`. See the step-4 comparison below. ❓ Complete processor compatibility remains open. |
| Application fragmentation | EnterWorldOk uses fragment mode 2. Reassemble before application dispatch. See `05-enter-world.md`. |

## Step 4 — Outer-frame comparison

Build: client and GameDaemon 381. This is static evidence, not a connection test.

| Rule | Native evidence | Existing Rust | Result |
| --- | --- | --- | --- |
| Outer header | Two-byte little-endian header; payload starts at +2. Client `0x60F8A0`; server `0x1406ADD00`. | `frames::payload_len` | Same. |
| Size arithmetic | If the top two bits are zero, use low 14 bits as bytes. Otherwise use low 12 bits times four. Same sources. | `frames::payload_len` | Same. Encoded maximum is 16380 payload bytes, not the normal packet-buffer limit. |
| Initial server input | Reject a plain frame. Wait for the complete outer frame before processor checks. Server `0x1406ADD00`. | `frames::decode_client` | Same checked order. ❓ Initial RSA tag validation inside the native processor was not checked in this job. |
| Session input | Require processor state 5 and tags 8 through 11. Server `0x1406ADD00`. | `frames::decode_client` | Same wrapper checks. |
| Logical packet | Signed short size must be at least 2 and must fit the decoded output. Server `0x1406ADD00`. Command follows size. | `frames::decode_client`; Packet layer | Same checked layout. |
| Buffering | Keep incomplete input. Consume complete frames in order. Move remaining bytes to the start. Server `0x1406A5AB0`; receive bound `0x1406A5650`. | Caller must buffer the stream around the frame codec. | Same design. Native receive storage holds at most 4008 bytes in this path. This is not a new payload maximum. |
| Ordinary sends | Packet composition uses a 4000-byte logical buffer. Server `0x1406A5E00`, `0x1406A5FC0`. | BloodLib `MAX_PACKET=4000`; session `MAX_CIPHERTEXT=4000` | Logical bound matches. ❓ Full native session padding and key-slot behavior were not checked. |
| Client receive extensions | Game processor also has broadcast and compressed-packet routes. Client `0x608FB0`; external decompression setup `0x5CF287`. | The current frame API covers ordinary session traffic only. | Do not claim complete game-processor support. ❓ Extension use during first entry remains open. |

Reuse the existing ordinary frame codec. Do not use the LobbyGate command table or version.
Do not treat this comparison as proof of successful native-client acceptance.

### Permitted handshake summary and stop boundary

| Step | Algorithm family / ownership | Evidence |
| --- | --- | --- |
| Initial channel exchange | RSA-family handshake processor. GameDaemon constructs public/private RSA objects. | Factory `0x1406E3100`, selected from receive wrapper `0x1406ADD00`. |
| Hello reply and session setup | AES-128-family session processor. The Hello operation supplies session-secret state on the server side. The client processes the opaque reply. ❓ Full client secret ownership was not traced. | Server `0x1406A65D0` selects factory mode 3, algorithm 1; factory selects `Aes128`. Client callback remains `0xF70280`. |
| Vendor-secret boundary | The server factory loads an RSA private object from static binary data. No key bytes were read or copied. Stop this path. ❓ Native-client acceptance with a replacement key was not established. | `HandshakePacketCrypt` factory branch in `0x1406E3100`. |

Do not implement key extraction or a vendor-secret substitute.
No cryptographic algorithm internals are specified here.

## Restricted boundaries

| Boundary | Address | Action |
| --- | --- | --- |
| Client anti-cheat state in `World::EnterWorld` | `0x741AB2`, reference to `CsAuthServerNumber` | Stop this function path. No anti-cheat work. |
| Client security-version getter on entry-request construction | Calls at `0xF3B00C` and `0xF3B1F6` | Record the boundary only. Do not inspect the getter. |
| Server zone-entry anti-cheat hook | Log reference `0x1405DF767`, containing function `0x1405DE040`; string `0x140DA1BF0` | Stop this function path. No authentication implementation. |
| Server command-18 conditional GameGuard hook | Receive handler `0x140370F10`; hook target `0x14036DE10` | Stop the hook path. Record only the ordinary ping fields. |
| Server dispatch safe-mode filter | Dispatcher `0x14007BC00` | Do not follow the restricted virtual targets. |
| Client command-32 security interface | Callback path at `0xEF68C8` | Stop this call path. |
| Licence code | ❓ No address established in the checked path | No licence analysis. |

No prohibited binary was opened. No binary was patched. No network operation ran.
