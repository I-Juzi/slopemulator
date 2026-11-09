# 02 — Client.exe 381: network transport, framing and session crypto

Summary:

1. All game-server connections (AuthD, LobbyGate, GameD, ArenaD, InfoGate, Proxy) use one network layer: the global `Network` object (`0x1a4da00`), one `Connection` per slot (`Connection.cpp`, `0x5cf200`–`0x5d0080`), and overlapped WinSock I/O.
2. On the wire, each packet is in an envelope: `u16 EncHeader` (crypt code in bits 15..12, length in 4-byte blocks in bits 11..0) and the encrypted payload (§2.1). Inside is the plain packet: `int16 size` (header included), `int16 command`, body (§2.2). The `Session` virtual functions `PacketSize` (`0x5d4a20`) and `IsCompleted` (`0x5d4a40`) cut plain packets and join fragments.
3. A `PacketProcessor` on the connection encrypts and decrypts. LobbyGate, AuthD, Proxy and InfoGate use `AuthPacketProcessor`. GameD and ArenaD use `GamePacketProcessor` with ZSTD decompression.
4. The session key comes from a key exchange (`Dh`, `0x60ebd0`) or from a handshake parameter. The cipher is `Aes128` (Crypto type 3) by default. We write `keyhere` for all key values.
5. `PacketParser::ParseData` (`0x5fd770`) is **not** the game framing. Only the channel chat and NCTalk voice code use it (§6). The old notes (`client-exe-reversing.md` §2.5) are corrected here.

```
 Game code (GameSession / LobbyGateSession / InfoGateSession)
   │ SendXxx → packet object (StructPacket)
   ▼
 Network send 0x5d2740 ──► queue per Connection ──► Session::ToStream 0x5d4d20 (struct → bytes)
   │                                                 PacketProcessor::Encrypt
   ▼                                                 WSASend 0x5cfa40
 socket (TCP, non-blocking, overlapped)
   ▲                                                 WSARecv 0x5cfad0
   │ recvNext 0x5cfb70: PacketProcessor::Decrypt → Session::PacketSize / IsCompleted
   │                    → Session::ToPacket 0x5d4c10 (bytes → struct) → Session::OnRecv
```

Sources: all facts come from the 381 database `ida/lanes/cx/Client.exe.i64`. "419 GM" marks a fact (name, type, enum) that comes only from `Client_gm.exe.i64`. ❓ marks a fact that we did not verify. Function lists: [`02-transport-functions.tsv`](02-transport-functions.tsv) (88 rows, made by one IDA Python script; every `func` row is a function start in 381). Its `ida_name_381` column is the name in the 381 database. Some of these names are false PDB matches; use `doc_name`.

---

## Quick find

| Task | Function | 381 address | GM name (419 GM) | Section |
| --- | --- | --- | --- | --- |
| Find the network object | global pointer | `0x1a4da00` | `Network` | §1.1 |
| Slot → processor kind, max size, timeout | connect switch | `0x5d3030` (jump table `0x5d3122`) | `Network::Connect` ❓ | §1.1 |
| Open a socket | `Connection::connect` | `0x5cf400` | `Connection::connect` | §4.3 |
| Request a connect | connect request | `0x5d2680` | — | §4.1 |
| Resolve a host name | connect by name | `0x5d25d0` | — | §4.1 |
| Network thread loop | thread loop | `0x5d5ab0` | — | §4.2 |
| Keepalive ping (30 s) | ping all sessions | `0x5d28f0` | — | §4.2 |
| Send a packet | `Network::Send` | `0x5d2740` | — | §4.4 |
| Receive and cut packets | `recvNext` | `0x5cfb70` | `Connection::recvNext` | §4.4 |
| Dispatch received packets | dispatch | `0x5d0080` | — | §4.4 |
| Map errors to reasons | connection event | `0x5d2d80` | — | §5.1 |
| Disconnect | `Network::Disconnect` | `0x5d33c0` | `Network::Disconnect` ❓ | §5.1 |
| Envelope length | `PPUtil::PacketSize` | `0x60f8a0` | `PPUtil::PacketSize` | §2.1 |
| Plain packet length | `Session::PacketSize` | `0x5d4a20` | `PacketSession::PacketSize` | §2.2 |
| Fragments | `Session::IsCompleted` | `0x5d4a40` | `PacketSession::IsCompleted` | §2.3 |
| Bytes → struct | `ToPacket` | `0x5d4c10` | `PacketSession::ToPacket` | §2.4 |
| Struct → bytes | `ToStream` | `0x5d4d20` | `PacketSession::ToStream` | §2.4 |
| Make the processor | `PacketProcessor::Create` | `0x6076b0` | same | §3.1 |
| Encrypt / decrypt (GameD) | `GamePacketProcessor::Encrypt` / `Decrypt` | `0x608f10` / `0x608fb0` | same | §3.3 |
| Encrypt / decrypt (LobbyGate, InfoGate) | `AuthPacketProcessor::Encrypt` / `Decrypt` | `0x6088e0` / `0x6089a0` | same | §3.3 |
| Session key rotation | `SetSessionKey` | `0x608d10` (Game), `0x608740` (Auth) | same | §3.1 |
| Handshake packet | `PPUtil::EncryptHandshake` | `0x60fb10` | same | §3.4 |
| Session key from DH | `PPUtil::SetupSessionCrypto` | `0x60f8f0` | same | §3.4 |
| AES-128 | `Aes128::Encrypt` / `Decrypt` | `0x611750` / `0x6117c0` | same | §3.2 |
| ZSTD broadcast | `ZSTD_decompress_usingDict` | `0x5ced10` | same | §3.5 |
| Chat / voice parser | `PacketParser::ParseData` | `0x5fd770` | same | §6 |
| Byte example | — | — | — | §7 |

Search anchors (strings): `Connection.cpp` path `0x186c338`; `.\ClientPacketProcessor.cpp`; `%d; %s; %s` (`Network::SocketError`); `InfoGateSession Connect Error, ip:%s, port:%d, errorCode:%d,` `0x159ba78`; slot name table `0x19499b8`; `%s/dict` `0x5ceba0`.

---

## 1. Objects and slots

### 1.1 `Network` connection slots

`Network` is a global object. The pointer is at `0x1a4da00` (381). Layout (419 GM type `Network`, size 88): `printNetworkLog` bool @0, `connection[10]` @4, `connectionVersion[10]` @44, `securityManager` @84.

Each session object returns its slot id from virtual slot 1 (`Session::Id`). The network code checks `0 <= id < 10` and then uses `connection[id]` (`0x5d2840`, `0x5d2740`, `0x5d4b10`).

Slot ids (419 GM enum `Network::<unnamed_tag>`). The per-slot connection settings come from the switch in the connect function `0x5d3030` (381, jump table at `0x5d3122`):

| Id | Name (GM) | Session class (381 vtable) | Processor kind passed to `Connection` ctor `0x5cf200` | Packet processor | `packetMaxSize` | `useTimeout` |
| ---: | --- | --- | ---: | --- | ---: | --- |
| 0 | INVALID | — | — | — | — | — |
| 1 | AUTHD | ❓ no 381 vtable found | 1 | `AuthPacketProcessor` | 4096 | 0 |
| 2 | NEXTPLATFORM | ❓ (STS uses `StsMsgCli32.dll`, see [03-sts-client](03-sts-client.md)) | 0 | none | 65000 (`0xFDE8`) | 0 |
| 3 | LOBBYGATE | `LobbyGateSession` (`0x15e9a80`) | 1 | `AuthPacketProcessor` | 4096 | 0 |
| 4 | GAMED | `GameSession` (`0x15896b8`) | 2 | `GamePacketProcessor` + ZSTD | 4096 | 0 |
| 5 | PROXY | ❓ | 3 | `AuthPacketProcessor` | 4096 | 0 |
| 6 | GAMED_DEBUG | ❓ | 0 | none | 4096 | 0 |
| 7 | BUILDER | ❓ | 0 | none (same code as 6) | 4096 | 0 |
| 8 | ARENA | `GameSession` (`0x15896b8`) | 2 | `GamePacketProcessor` + ZSTD (same code as 4) | 4096 | 0 |
| 9 | INFOGATE | `InfoGateSession` (`0x15a8608`) | 1 | `AuthPacketProcessor` | 4096 | 1 |

- Slots 4 and 8 also pass the security manager object (`Network+84`, virtual call) as `Connection.security3rd` (`0x5d31a0`).
- `GameSession::Id` (`0x47d780`) returns the signed byte at `GameSession+0x34`. So one `GameSession` class serves slot 4 (GameD) and slot 8 (ArenaD). ❓ The code that writes `+0x34` is not read yet.

### 1.2 `Session` virtual table

All three 381 session classes derive from `PacketSession` (419 GM). Slot names come from 419 GM `PacketSession_vtbl`. The 381 functions are the entries in the three vtables.

| Slot | Offset | Name (GM) | `GameSession` `0x15896b8` | `LobbyGateSession` `0x15e9a80` | `InfoGateSession` `0x15a8608` | Meaning |
| ---: | ---: | --- | --- | --- | --- | --- |
| 0 | 0 | dtor | `0x47d7b0` | `0x4d61b0` | `0x5d4990` | destructor |
| 1 | 4 | Id | `0x47d780` (byte `+0x34`) | `0x46a4e0` (3) | `0x59a880` (9) | slot id, §1.1 |
| 2 | 8 | Version | `0x47d790` (`0x4FE` = 1278) | `0x4d6190` (`0x3A` = 58) | `0x4a5ee0` (`0x19` = 25) | protocol version that the session sends in its hello ❓ |
| 3 | 12 | CryptAlgorithm | `0x403650` (1) | `0x403a90` (0) | `0x403a90` (0) | `PacketProcessor::Algorithm`, §3.2 |
| 4 | 16 | OnError | `0xf44150` | `0xca7080` | `0xea11f0` | |
| 6 | 24 | OnConnect | `0xf2c0c0` | `0xc979f0` | `0xe82cb0` | |
| 8 | 32 | OnClose | `0xefbd80` | `0xc67290` | `0xe31cc0` | |
| 10 | 40 | OnRecv | `0x5d4a10` | `0x5d4a10` | `0x5d4a10` | calls slot 23 |
| 12 | 48 | SendPing | `0x47d7a0` | `0x4d61a0` | `0x4a5ef0` | |
| 13 | 52 | SetHandshakeParam | `0x5d4b00` | `0x5d4b00` | `0x5d4b00` | → `PacketProcessor::SetHandshakeParameter`, §3 |
| 14 | 56 | SetDaemonBroadcastKey | `0x5d4b10` | `0x5d4b10` | `0x5d4b10` | → `PacketProcessor::SetGlobalBroadcastKey` |
| 15 | 60 | SetZoneBroadcastKey | `0x5d4b90` | `0x5d4b90` | `0x5d4b90` | → `PacketProcessor::SetLocalBroadcastKey` |
| 16 | 64 | IsUrgent | `0xee1130` | `0x4f11c0` | `0x4f11c0` | |
| 17 | 68 | PacketSize | `0x5d4a20` | `0x5d4a20` | `0x5d4a20` | §2.1 |
| 18 | 72 | IsCompleted | `0x5d4a40` | `0x5d4a40` | `0x5d4a40` | §2.2 |
| 19 | 76 | ToPacket | `0x5d4c10` | `0x5d4c10` | `0x5d4c10` | bytes → packet struct, §2.3 |
| 20 | 80 | ToStream | `0x5d4d20` | `0x5d4d20` | `0x5d4d20` | packet struct → bytes, §2.4 |
| 21 | 84 | PropertyPairFieldMax | `0xee11a0` | — | — | |
| 22 | 88 | PropertyPairFieldInfo | `0xee1170` | — | — | |
| 23 | 92 | onRecv | `0xf44030` | `0xc88240` | `0xe66070` | command dispatch, see [05-game-dispatch](05-game-dispatch.md), [04-lobbygate](04-lobbygate.md), [12-infogate-arena](12-infogate-arena.md) |
| 24 | 96 | onStreamError | `0xf4b550` | `0x4f0090` | `0x4f0090` | |

Note: some 381 names of these slots are false PDB matches (for example `DrNullDataTable::createInnerIter` at `0x47d120`). Use the GM slot names in this table.

`Session` data (419 GM, 40 bytes): `nagle` bool @4, `printNetworkLog` @5, `logModuleId` @8, `gamePacketSkillCmd` @12, `gamePacketMoveCmd` @16, `gamePacketBroadCastActionCmd` @20, `lastBroadCastActiontime` SYSTEMTIME @24. `PacketSession` adds (52 bytes): `fragsize` int @40, `cmdmax` int @44, `define` (`StructPacket::Def` table) @48.

### 1.3 `Connection` object

Size `0x60` (96) bytes, made by `operator new(0x60)` in `0x5d3030`, constructor `0x5cf200`. Field names: 419 GM type `Connection`. Offsets match the 381 code.

| Off | Name (GM) | Type | Set or used in 381 |
| ---: | --- | --- | --- |
| 0 | vftable | | `0x5cf200` |
| 4 | session | `Session*` | ctor arg; recv code calls its slots 17/18 (`0x5cfb70`) |
| 8 | useTimeout | bool | ctor arg (`cl`); 1 only for slot 9 |
| 12 | version | int | ctor arg (`Network.connectionVersion[id]`) |
| 16 | socket | SOCKET | -1 when closed |
| 20 | address | IPv4 | |
| 24 | port | u16 | |
| 28 | socketEvent[2] | WSAEVENT | `WSACreateEvent` in `0x5cf400` |
| 36 | recvBuffer | `RecvBuffer*` | raw bytes from `WSARecv` |
| 40 | pktBuffer | `RecvBuffer*` | decrypted bytes, cut into frames |
| 44 | sendBuffer | `SendBuffer*` | |
| 48 | packetListSize | int | |
| 52 | packetCount | int | |
| 56 | pendingCount | int | |
| 60 | packetList | `Packet**` | send queue |
| 64 | packetProcessor | `PacketProcessor*` | §3; null for kinds 0 |
| 68 | security3rd | `Security3rdParty*` | slots 4 and 8 |
| 72 | sendTotalSize | u32 | |
| 76 | recvTotalSize | u32 | `+=` frame size in `0x5cfb70` |
| 80 | recorder | `PacketRecorder*` | |
| 84 | usePacketRecorder | bool | |
| 88 | packetMaxSize | int | ctor arg, §1.1 |
| 92 | JustDisconnected | bool | set to 1 on every I/O error |

Buffers (419 GM): `ConnectionBuffer` = vftable @0, `OVERLAPPED` @4, `WSABUF` @24. `RecvBuffer` (44 B) adds `buffer` @32, `size` @36, `count` @40. `SendBuffer` (48 B) adds `sending` @32, `size` @36, `count` @40, `buffer` @44.

---

## 2. Framing

There are two layers. Connections with a packet processor (slots 1, 3, 4, 5, 8, 9) put each plain packet into an encrypted envelope. Connections without a processor (slots 2, 6, 7) send the plain packet as it is.

```
 wire (processor slots):   [EncHeader u16][payload: len bytes, encrypted]   ──Decrypt──►  plain packet
 wire (no processor):      plain packet
 plain packet:             [size i16][command i16][body …]
```

### 2.1 Envelope: `EncHeader` (`PPUtil::PacketSize`, `0x60f8a0`)

| Bits of the u16 LE | Name | Meaning |
| --- | --- | --- |
| 15..14 = 0 | plain | `len = hdr & 0x3FFF`; the payload is the plain packet, copied as it is |
| 15..12 = code (4, 5, 8–14) | enc code | §3.3 |
| 11..0 | blocks | `len = 4 × (hdr & 0x0FFF)` |
| codes 6, 7 or > 14 | — | `PacketSize` returns 0 |

- Frame size on the wire = `len + 2`.
- Send side: `PPUtil::Encrypt` (`0x60fd40`) writes `hdr = (code << 12) ^ ((cipherLen >> 2) & 0xFFF)` and returns `cipherLen + 2`. The cipher length is a multiple of 16 (AES), so it is always a multiple of 4.
- Receive side: `AuthPacketProcessor::Decrypt` (`0x6089a0`) and `GamePacketProcessor::Decrypt` (`0x608fb0`) read `hdr`. If fewer than 2 bytes or fewer than `len + 2` bytes are present, they set error 10 `PP_MORE_PACKET` and the connection waits. If `len` is larger than the output buffer, they set error 4 `PP_INSUFFICIENT_BUFFER`.
- Max payload: `4 × 0xFFF` = 16,380 bytes.
- The server reads the same envelope: [LobbyGate framing §4](../../Server/LobbyGate/03-framing.md).

### 2.2 Plain packet header

| Off | Size | Type | Name | Meaning |
| ---: | ---: | --- | --- | --- |
| 0 | 2 | int16 LE | size | whole plain packet length in bytes, header included |
| 2 | 2 | int16 LE | command | command id (LobbyGateCommand, MainCommand, InfoGateCommand …) |
| 4 | … | | body | fields of the `StructPacket` for this command |

- `Session::PacketSize` (`0x5d4a20`): if 2 or more bytes are present, return the `int16` at offset 0. Else return 0 (wait for more bytes).
- The receive loop (`0x5cfb70`) takes a packet only when `0 < size <= bytes present`.
- After AES decryption the buffer can be longer than `size` (zero padding). The client uses `size`, not `len`.
- Same layout on the server: [LobbyGate framing §2](../../Server/LobbyGate/03-framing.md).

### 2.3 Large packets: fragments (`IsCompleted`, `0x5d4a40`)

`IsCompleted(frame, len, &cutOffset, &cutSize, &isFragment)` decides if a frame is a whole packet. It also tells the caller which header bytes to cut out.

| Condition | Result |
| --- | --- |
| `len == 2` | complete, nothing cut |
| `len < 4` | not complete (error) |
| `frame.length > len` | not complete |
| no `define` table, or the command's def entry has byte `+5` = 0 | complete, nothing cut |
| command is a fragment command, first fragment (`this+40` = 0) | `remaining = int32 at frame+4 + 4`; cut 4 bytes at offset 4; `frame.length -= 4` |
| command is a fragment command, next fragment (`this+40` ≠ 0) | cut 8 bytes at offset 0 (the next fragment's header and its 4 extra bytes) |
| after the cut | `remaining -= (frame.length − header cut)`; complete when `remaining <= 0` |

`remaining` is `PacketSession.fragsize` (`this+40`, 419 GM name). "Frame" in this table means one plain packet (§2.2).

So a large packet goes as: first plain packet `[size16][cmd16][total32][body…]`, then more plain packets `[size16][cmd16][4 bytes][body…]`. Each is in its own envelope. The receive loop appends each body to `pktBuffer` and parses the packet when `IsCompleted` returns true. ❓ The meaning of the 4 bytes in the next fragments is not verified.

`define` table entry (20 bytes per command, index = command id, `this+48`; count `cmdmax` at `this+44`): +0 `StructPacket::Def*`, +4 byte "forbidden" (ToPacket raises an exception, `0x5d4c10`), +5 byte "fragment command".

### 2.4 Bytes to packet and packet to bytes

| Step | Function (381) | What it does |
| --- | --- | --- |
| ToPacket | `0x5d4c10` (slot 19) | Gets `size` from slot 17. Reads `command` at +2. If `command < cmdmax` and a def exists: `PacketSession::stream2structure` (`0x5d4ed0`) → `PacketSession::setField` (`0x5d4f20`) fills the struct, then sets `_command`. Else `0x5d4d90` copies the raw bytes into a generic packet (`_aligned_malloc`). |
| ToStream | `0x5d4d20` (slot 20) | Reads the command from the packet (packet vtable +12). If it has a def: `0x5d5260` → `0x5d52c0` (recursive field writer). Else `0x5d4e30` (raw `memcpy`). |
| Field setters | `PfString::Set` `0x5d35d0`/`0x5d3690`, `PfVarBin::Set` `0x5d37a0`/`0x5d3800`, `PropertySync::Set` `0x5d4600`, `Property::SetField` `0x5d3f30` | field kinds of `StructPacket::FieldType`, see old notes `client-exe-reversing.md` §2.3 |

The field encoding (format codes) is the same as the server `PacketBuffer::Compose`: [LobbyGate framing §5](../../Server/LobbyGate/03-framing.md). ❓ We did not compare each field kind byte by byte.

---

## 3. Session crypto

The server side of the same handshake is in [LobbyGate handshake](../../Server/LobbyGate/02-handshake.md) (RSA OAEP variant, DH byte order, key derivation). This section gives the client addresses and the client steps. All key values: `keyhere`.

### 3.1 Packet processors

`PacketProcessor::Create` (`0x6076b0`, file string `.\ClientPacketProcessor.cpp`) makes the processor for a connection. `PacketProcessor::Type` (419 GM): `PP_AUTHD` 0, `PP_GAMED` 1.

| Item | `AuthPacketProcessor` | `GamePacketProcessor` |
| --- | --- | --- |
| vtable (381) | `0x1870c4c` | `0x1870c74` |
| size | 40 B | 10,304 B |
| `+4` | error code (`PacketProcessor::ErrCode`) | same |
| `+8` | handshake crypto (RsaPublic, built-in server key) | same |
| `+12` | private crypto (RsaPrivate, client temp key) | same |
| `+16` | key exchanger (`Dh`) | same |
| `+20` | global (daemon) broadcast crypto | global broadcast crypto |
| `+24` | crypto type = 3 | zone broadcast crypto |
| `+28` | key slot index; -1 = handshake | crypto type (from algorithm) |
| `+32` | session crypto slot 0 | key slot index; -1 = handshake |
| `+36` | session crypto slot 1 | session crypto slot 0 |
| `+40` | — | session crypto slot 1 |
| `+44` | — | byte: compression kind of zone broadcast (last byte of the zone key) |
| `+45` | — | 10,240-byte buffer for compressed broadcast payloads |
| `+10288` | — | decompressed size |
| `+10292`/`+10296`/`+10300` | — | external uncompress function and its 2 arguments (`SetExternalUncompressFunc` `0x609240`) |

Virtual functions (419 GM `PacketProcessor_vtbl`):

| Slot | Name | Auth (381) | Game (381) | Behaviour |
| ---: | --- | --- | --- | --- |
| 0 | dtor | `0x6085f0` | `0x608b70` | |
| 1 | GetType | `0x6085e0` | `0x68f150` | |
| 2 | SetSessionKey | `0x608740` | `0x608d10` | new crypto in slot `(idx+1)%2`; error 7 if that slot is in use; then `idx = new slot` (key rotation) |
| 3 | SetGlobalBroadcastKey | `0x6087f0` | `0x608dc0` | makes or re-keys the global broadcast crypto |
| 4 | SetLocalBroadcastKey | `0x608850` (error 2 `PP_NOT_SUPPORT`) | `0x608e20` | Game: the last key byte → `+44` compression kind; the other bytes are the AES key |
| 5 | SetHandshakeParameter | `0x608860` | `0x608e90` | only when `idx < 0` and a key exchanger exists (else error 6 or 7); `PPUtil::SetupSessionCrypto` → slot 0; `idx = 0` |
| 6 | Encrypt | `0x6088e0` | `0x608f10` | §3.3 |
| 7 | Decrypt | `0x6089a0` | `0x608fb0` | §3.3 |
| 8 | SetExternalUncompressFunc | `__purecall` | `0x609240` | Game only |

`PacketProcessor::Algorithm` (419 GM) → crypto type in `PacketProcessor::Create` (`0x6076b0`): DEFAULT 0 and TYPE1 1 → 3; TYPE2 2 → 4; TYPE3 3 → 5; TYPE4 4 → 6. `Crypto::InstanceSize` (`0x60af60`) returns -1 for types 4, 5, 6, so only types 0 and 1 work. `GameSession::CryptAlgorithm` returns 1 (`0x403650` in the vtable): GameD and ArenaD use AES-128.

### 3.2 Crypto classes (`Crypto::Create` `0x60b010`, `Crypto::InstanceSize` `0x60af60`)

| Type | Class | vtable (381) | Instance size | Used for |
| ---: | --- | --- | ---: | --- |
| 1 | `SimpleBlock` | (set in `0x60b010`) | 80 | not used by the game transport ❓ |
| 3 | `Aes128` | `0x1870cf4` (Encrypt at `+28` = `0x1870d10`) | 368 | session and broadcast crypto |
| 33 (`0x21`) | `SimpleStream` | (set in `0x60b010`) | 24 | not used by the game transport ❓ |
| 65 (`0x41`) | `RsaPublic` | (set in `0x60b010`) | 1,404 | handshake crypto (built-in server key) |
| 81 (`0x51`) | `RsaPrivate` | `0x1870d70` | 1,924 | client temporary key |

Crypto virtual slots used by `PPUtil`: `+4` Setup(key, len, bits), `+24` EncSize(len), `+28` Encrypt(in, len, out, &outLen), `+32` Decrypt(in, len, out, &outLen).

`Aes128` (381):

| Function | Address | Behaviour |
| --- | --- | --- |
| `Aes128::Setup` | `0x611730` | calls the pointer at `0x1a3dbfc` = `Aes128Table_Setup` (`0x615820`). Needs a key of 16 bytes or more (else error 5); uses the first 16 bytes. |
| `Aes128::EncSize` | `0x6116a0` | `(len + 15) & ~15` |
| `Aes128::Encrypt` | `0x611750` | pointer `0x1a3dc00` = `Aes128Table_Encrypt` (`0x615860`): each 16-byte block alone (ECB), last block zero-padded |
| `Aes128::Decrypt` | `0x6117c0` | pointer `0x1a3dc04` = `Aes128Table_Decrypt` (`0x615960`) |
| `CryptoKey::Initialize` | `0x607ff0` | writes the 3 pointers above (`0x60802f`, `0x608039`, `0x608043`) |
| tables | `0x13f3d30` … `0x13f4d30` | standard AES T-tables (Te0 first word `0xa56363c6` LE) |
| key masking in memory | `Crypto::EncodeKey` `0x60b2e0`, `Crypto::DecodeKey` `0x60b460` | if `Crypto+9` is set, the round keys are masked in memory and unmasked for each call. No effect on the wire. |

`Dh` (381, vtable `0x1870c9c`, made by `PPUtil::AllocateKeyExchanger` `0x60f7d0`, 2,096 bytes):

| Slot | Function | Address | Behaviour |
| ---: | --- | --- | --- |
| 1 | `Dh::Setup(bits)` | `0x60ece0` | accepts 32..512 bits; the client uses 128 |
| 2 | `Dh::Cleanup` | `0x60ed10` | |
| 3 | `Dh::Start(params, len, out, &outLen)` | `0x60ed90` | params = `[u32 n][g][u32 n][p]`; if no params, makes a random `g` and prime `p`; writes `g`, `p`, and the public value `A` |
| 4 | `Dh::Process(peer, len)` | `0x60efb0` | takes the server value `B` |
| 5 | `Dh::GetKey(out, &len)` | `0x60f030` | `S = ExpMod(…)`; output `(bits+7)/8` = 16 bytes; repeats bytes of S if S is shorter |

### 3.3 Encryption codes (`hdr >> 12`)

| Code | Direction | Key (Game processor offset / Auth processor offset) | Who makes it (381) |
| ---: | --- | --- | --- |
| 0 (bits 15..14 = 0) | both | none (plain) | `Decrypt` copies |
| 4 | C→S | server RSA public key (`+8` / `+8`) | `PPUtil::EncryptHandshake` `0x60fb10` writes `0x4000 \| blocks` |
| 4 | S→C | `+8` | server; after this packet the client frees both RSA cryptos |
| 5 | S→C | client RSA private key (`+12` / `+12`) | server; then the client frees both RSA cryptos |
| 8 | both | session slot 0 (`+36` / `+32`) | `PPUtil::SessionEncCode` `0x60f870` (idx 0) |
| 9 | both | session slot 1 (`+40` / `+36`) | `SessionEncCode` (idx 1) |
| 10 | both | session slot 0; flag = 1 | Game only; `SessionEncCode(0, true)` |
| 11 | both | session slot 1; flag = 1 | Game only |
| 12 | S→C | zone broadcast (`+24`) | Game only |
| 13 | S→C | zone broadcast (`+24`), then decompress | Game only; kind `+44`: 1 = zlib `uncompress`, 2 = external function (ZSTD, §3.5); other = error |
| 14 | S→C | global broadcast (`+20` / `+20`) | |
| other | — | error 8 `PP_INVALID_ENC_CODE` | |

- `SessionEncCode(idx, flag)` (`0x60f870`): idx 0 → 8 (+2 if flag); idx 1 → 9 (+2 if flag); else `0xFF`.
- The Auth processor accepts only 4, 5, 8, 9, 14 (`0x608a3c`–`0x608ab4`). Its `Encrypt` uses code 8 or 9 only.
- The Game decrypt flag (codes 10, 11) goes to the security third-party object `Connection+68` (vtable `+4`, called from `0x5cfb70`). The server spec says 10/11 mean "AES + 3rd-party step". ❓ We did not read the 3rd-party step in the client.

### 3.4 Handshake steps (client side)

```
 client                                                         server
 SendHello (L_HelloReq / G_HelloReq), last field = empty VarBin (u16 len = 0)
   │ Encrypt with idx = -1 → PPUtil::EncryptHandshake 0x60fb10
   │   1 alloc RsaPublic (type 0x41) + RsaPrivate (0x51) + Dh       (0x60fb85, 0x60fb8e, 0x60fb98)
   │   2 last u16 of the plain packet must be 0, else error 11 PP_INVALID_FIRST_PACKET
   │   3 RsaPublic.Setup(built-in key blob 0x13f6f80, 132 bytes, 1024 bits) = keyhere
   │   4 setupPrivateCrypto 0x60fa60: CryptoKey::generateRsaKey(512) 0x6081d0,
   │     append [u32 n][modulus] to the packet
   │   5 Dh.Setup(128); Dh.Start(param blob 0x13f7004, 32 bytes) → append [g][p][A]
   │   6 add the appended length to the trailing u16 and to the packet size
   │   7 RsaPublic.Encrypt(whole packet) → hdr = 0x4000 | blocks           ───────►
   │                                                                         HelloAck (code 5 or 4)
   │ ◄─────── Decrypt (code 5: RsaPrivate +12; code 4: RsaPublic +8), free both RSA objects
   │ HandleHelloOk: session slot 13 SetHandshakeParam(ack field at packet+0x10)
   │   → 0x5d27c0 → processor SetHandshakeParameter
   │   → PPUtil::SetupSessionCrypto 0x60f8f0: Dh.Process(B), Dh.GetKey → 16 bytes
   │     → Aes128.Setup(key) in session slot 0; idx = 0
   │ all later packets: code 8 (AES-128-ECB, slot 0)                       ◄──────►
```

| Step | 381 address | Notes |
| --- | --- | --- |
| LobbyGate HelloOk handler | `LobbyGateSession::ReceiveHelloOk` `0xcb8660` | on success sets `LobbyGateSession+0xB8` = 1, calls `0xc967d0` (next lobby step ❓); on failure logs and calls `BUiLobby::ExitGameReq` (`0xcb86c6`) |
| GameD / ArenaD HelloOk handler | `HandleHelloOkPacket` `0xf70280` | on success: if `Id == 4` (GameD) calls `0xf2bef0` and sets `GameSession+0x64` and `+0x5C` = 1; else (ArenaD) calls `GameSession::SendEnterArenaReq` `0xf2bf60` and sets `+0x5C` = 1. On failure logs and calls `BUiWorld::ExitGame` |
| `GameSession::SetHandshakeFinish` | `0xee11d0` | sets byte `+0x5C` = 1 |
| `GameSession::SendPing` | `0x47d7a0` → `0xf3b2a0` | sends only when `+0x5C` (handshake done) is set |
| Session `SetHandshakeParam` | `0x5d4b00` → `0x5d27c0` | passes `(VarBin.data, VarBin.len16)` to processor slot 5 |
| Session `SetDaemonBroadcastKey` | `0x5d4b10` | processor slot 3 |
| Session `SetZoneBroadcastKey` | `0x5d4b90` | processor slot 4 |

The handshake key blob (`0x13f6f80`, 132 bytes) and the DH parameter blob (`0x13f7004`, 32 bytes) are data in `.rdata`. We do not copy them: `keyhere`. The server spec gives the public key format: [LobbyGate handshake §7](../../Server/LobbyGate/02-handshake.md).

### 3.5 Compression

- Connection kind 2 (slots 4 and 8) calls `SetExternalUncompressFunc(ZSTD_decompress_usingDict 0x5ced10, 0, 0)` in the `Connection` ctor (`0x5cf200`).
- Only code 13 (zone broadcast) packets are compressed. The compression kind is the last byte of the zone broadcast key (`SetLocalBroadcastKey` `0x608e20`): 1 = zlib, 2 = ZSTD. Decompress output goes to the normal output buffer; on error the processor sets 16 `PP_NOT_SUPPORT_COM` or 13.
- ZSTD code in 381: `ZSTD_decompressFrame` `0x5d8790`, `ZSTD_decompressMultiFrame` `0x5d89d0`, `ZSTD_decodeFrameHeader` `0x5d67e0`. `CompressUtil::CompressConfigration` `0x5ceb00` uses the string `%s/dict` (`0x5ceba0`) ❓ (a ZSTD dictionary file).
- Client packets to the server are not compressed (the Encrypt functions have no compress step).

---

## 4. Socket layer

### 4.1 Connect requests (game thread)

| Step | 381 address | Behaviour |
| --- | --- | --- |
| Connect by host name | `0x5d25d0` (false PDB name `~SeasonContentsTimeAttackRecord`) | If the first character is a letter (`iswalpha`), converts the wide host name (`WideCharToMultiByte`) and calls `gethostbyname`. Then calls `0x5d2680`. |
| Connect request | `0x5d2680(session, addr, port)` | Posts a command (event `0x5d5fc0`) to the network thread only when the slot of `session.Id` is free. |
| Slot check | `0x5d2840(slotId)` | Returns the connection in the slot (used before each connect and send). |
| Callers of `0x5d2680` | `0x5d2661` (by host name), `0xe54068` (no function in IDA), `sub_E54080` `0xe54121`, `sub_E54130` `0xe5415f`, `sub_1137560` `0x113759c`, `BNSClient::EnterGameForNPWithLobbyGate` `0x1157fbd` | `sub_E54080` clears bytes `+0x50`, `+0x78`, `+0x79` before the call. `sub_E54130` sets `+0x50` = 1. ❓ The session class for each caller is not verified. |
| Slot names (for logs) | table `0x19499b8` (11 pointers, UTF-16) | `INVALID`, `AUTHD`, `NEXTPLATFORM`, `LOBBYGATE`, `GAMED`, `PROXY` (`0x186c088`), `GAME_DEBUG`, `GAME_BUILDER`, `ARENA`, `INFOGATE`, `none`. Used by `HandleAuthTokenOkPacket` (`0xea5bac`). |

`/ProxyIP`: ❓ **not found**. No ASCII or UTF-16 string `ProxyIP`, `ProxyPort` or `LobbyIP` exists in 381. The only "proxy" strings are the slot name `PROXY` (`0x186c088`), OpenSSL certificate strings, `TenProxy.dll` (`0x1663178`) and the `[proxys]` section of `InitializeClientModuleNames` (`0x112319c`). The command-line parser is chapter 01 ❓. Slot 5 (`PROXY`) has kind 3 (§1.1); we did not find a caller that connects slot 5 ❓.

### 4.2 Network thread

| Item | 381 address | Value / behaviour |
| --- | --- | --- |
| Thread start | `0x5d54b0` | `_beginthreadex`, entry `0x5d5a80` |
| Loop | `0x5d5ab0` | `WSAWaitForMultipleEvents`, timeout 3,000 ms (`0xBB8`) |
| Event index 10 | | quit the loop |
| Event index 11 | `0x5d0910` | command queue signal |
| Other event index n | `0x5d2d80(n)` | I/O on connection slot n (§4.4) |
| Keepalive | `0x5d28f0` | every 30.0 s: for each connected session, call `SendPing` (Session vtable `+48`, slot 12) |
| Loop timing statistics | `0x1a4da0c` last, `0x1949a00` min, `0x1a4da10` max | |
| Command queue | `0x5d5d30` → `0x5d2cf0` | lock `0x5d61d0` (critical section); signal `0x5d5820` / `0x5d5850` |

Commands in the queue (`0x5d2cf0`):

| Type | Action | 381 address |
| ---: | --- | --- |
| 2 | connect (network, session, addr, port) | `0x5d3030` |
| 3 | disconnect (reason, 0) | `0x5d33c0` |
| 4 | ❓ (send flush) | `0x5d3440` |

Event objects posted by the game thread (allocation helpers): `0x5d5f70` (cleanup, from `BNSClient::Cleanup` via `0x5d24c0`), `0x5d5fc0` (connect), `0x5d6030` (posted by `0x5d26e0`), `0x5d60a0` (send), `0x5d6110` (received packet, network → game thread). ❓ The exact type value of each event object is not verified.

### 4.3 Connect (`Connection::connect`, `0x5cf400`)

1. `WSASocketW(AF_INET, SOCK_STREAM, IPPROTO_TCP, overlapped)`.
2. If `useTimeout` (slot 9 InfoGate only, §1.1): `ioctlsocket(FIONBIO = 0x8004667E, 1)`, `WSAConnect`. On `WSAEWOULDBLOCK` (10035) `select` with a 1-second timeout. On timeout the error is `WSAETIMEDOUT` (10060). Then the socket goes back to blocking mode.
3. Else a blocking `WSAConnect`.
4. `setsockopt SO_RCVBUF (0x1002) = 0` and `SO_SNDBUF (0x1001) = 0` (no kernel buffers; I/O goes direct to the overlapped buffers).
5. `TCP_NODELAY = 1`, unless `Session.nagle` (`+4`) is set.
6. Creates 2 `WSAEvent` objects (recv and send), allocates the buffers (§1.3: recv 0x2000, pkt 0x2000, send 0x8000), then posts the first `WSARecv` (`postRecv` `0x5cfad0`).
7. Errors are logged by `Network::SocketError` (`0x5d2930`, `FormatMessageW`, format `%d; %s; %s`). InfoGate logs `InfoGateSession Connect Error, ip:%s, port:%d, errorCode:%d,` (`0x159ba78`, `InfoGateSession::OnError` `0xea123c`).

### 4.4 Receive and send

| Path | 381 address | Behaviour |
| --- | --- | --- |
| Post receive | `postRecv` `0x5cfad0` | `WSARecv` into the free part of the recv buffer |
| Receive complete | `0x5d2d80` → `recvNext` `0x5cfb70` | Decrypt each envelope (§2.1), cut plain packets (§2.2, §2.3), ToPacket (§2.4) |
| Dispatch | `0x5d0080` | For each decoded packet: if `IsUrgent` (vtable slot 16) returns 1, call `OnRecv_2` (slot 11) on the network thread. Else post event `0x5d6110` to the game thread. Then post `WSARecv` again. |
| Send request | `Network::Send` `0x5d2740` (from `0x47d530` via `0x5d2840`) | Posts send event `0x5d60a0` and signals `0x5d5820` |
| Send next | `Connection::sendNext` `0x5cff70` | ToStream (§2.4) + Encrypt (§3.3) into the 0x8000 send buffer |
| Post send | `postSend` `0x5cfa40` | `WSASend` |

❓ The send-queue limit and the code that gives `DisconnectGameReason` 10 `SEND_BUFFER_OVERFLOW` are not found.

---

## 5. Disconnect, timeouts and reconnect

### 5.1 Error → reason (`0x5d2d80`, `Network::Disconnect` `0x5d33c0(reason, wsaerr)`)

| Source | Value | Reason (`Network::DisconnectReason`, 419 GM) |
| --- | --- | --- |
| Socket error `WSAECONNRESET` | 10054 | 104 `CONN_RESET` |
| Other socket error | | 103 |
| `Connection::RecvCompleteErr` 1..6 | | 131..136 (`RECV_FAIL_*`) |
| other receive error | | 130 |
| `Connection::SendErr` 1..4 | | 151..154 (`SEND_FAIL_*`) |
| other send error | | 150 |

`BNSClient::GetDisconnectWSALastError` (`0x520c60`) returns the stored WSA error for the UI.

### 5.2 Timers

| Timer | Value | 381 address |
| --- | --- | --- |
| Connect timeout (InfoGate only) | 1 s (`select`) | `0x5cf400` |
| Network wait | 3,000 ms | `0x5d5ab0` |
| Keepalive ping | 30.0 s | `0x5d28f0` |
| Server ping check | ❓ `GameSession::ReceivedServerPing` vector (`0x488160` is `vector::resize`); `DisconnectGameReason` 8 `NO_RESPONSE_SERVER_PING` | not found |

### 5.3 Reconnect

| Item | 381 address | Behaviour |
| --- | --- | --- |
| `InfoGateSession::ReconnectInfoGate` | `0x1131520` | Writes a log, allocates a task object (`_aligned_malloc`, field `+0x0C` = 0). ❓ The task queue is not traced. |
| `BAccount::OnReconnectInfoGate` | `0x127f130` | Writes a log, allocates a task (`+0x0C` = 0), calls 2 virtual functions, then `0x47d530` (send) |
| `NPSession::ReconnectionStep` (419 GM) | | `INVALID` 0, `DETECTED_DISCONNECTION` 1, `RECONNECTING` 2, `COUNT` 3 — the STS / NP session. See chapter 03 ❓. |

The client has no automatic reconnect for LobbyGate, GameD or ArenaD in the network layer: a disconnect goes to the session `OnDisconnect` and to the UI ❓ (no reconnect call found in `0x5d33c0`).

---

## 6. `PacketParser` (`0x5fd770`): chat and voice only

`PacketParser::Create` (`0x5fd5d0`) has 2 callers: `ChannelChattingClient::Initialize` (`0x5e9bb1`) and `VC::NCTalkIpcServer::Init` (`0x5f0fd6`). The game connections do not use it.

| Offset | Field |
| ---: | --- |
| +0 | header size |
| +4 | listener (vtable `[0]` GetPacketSize, `[1]` OnPacket) |
| +12 | filled byte count |
| +16 | pending buffer |
| +20 | current packet |

`ParseData` asks the listener for the packet size after `header size` bytes, collects the bytes, then calls `OnPacket`.

Related functions (381): `PacketPool::Init` `0x5ed350`, `Create` `0x5ed6f0`, `Destroy` `0x5ed7c0`, `AllocPacket` `0x5ed850`, `FreePacket` `0x5ed980`, `PopFreePacket` `0x5eda10`; `TcpSession::SendPacket` `0x5e4bb0`, `Disconnect` `0x5e4b70`; `_SendDataTo` `0x5f0050`, `_ProcessNetwork` `0x5f04e0`, `_CreateNode` `0x5f07a0`, `_ConnectSession` `0x5f0960`, `_DisconnectSession` `0x5f0ba0`, `TcpNode` `0x5fd980`, `IoThreadPool` `0x5fe200`, `_SelectSocket` `0x5febe0`; `ChannelChattingClient::SendKeepAlive` `0x5ea720`, `StartKeepAlive` `0x5ea7e0`, `StopKeepAlive` `0x5ea900`.

---

## 7. Worked byte example (code 8 envelope)

A plain packet with `size` = 6, `command` = `0x0010`, body = 2 bytes `AA BB` (example values):

```
plain:      06 00 10 00 AA BB
AES input:  06 00 10 00 AA BB 00 00 00 00 00 00 00 00 00 00     (zero pad to 16, Aes128::EncSize 0x6116a0)
cipherLen:  16  → blocks = 16 >> 2 = 4
hdr:        (8 << 12) ^ 4 = 0x8004  → bytes 04 80
wire:       04 80 <16 bytes AES-128-ECB(keyhere) output>          frame size = 18
```

Receive: `hdr >> 12` = 8 → session slot 0; `len = 4 × (0x8004 & 0xFFF)` = 16; decrypt 16 bytes; `size` = 6 → the packet is the first 6 bytes; the 10 pad bytes are ignored (§2.2).

The same packet on a slot without a processor (slots 2, 6, 7): `06 00 10 00 AA BB`.

---

## 8. Enums (419 GM names; values used in 381 code)

| Enum | Values |
| --- | --- |
| `Network` slot | 0 INVALID, 1 AUTHD, 2 NEXTPLATFORM, 3 LOBBYGATE, 4 GAMED, 5 PROXY, 6 GAMED_DEBUG, 7 BUILDER, 8 ARENA, 9 INFOGATE |
| `Network::DisconnectReason` | 100 NORMAL, 101 TIMEOUT, 102 NET_ERROR, 103 INVALID_SESSION, 104 CONN_RESET, 105 DATA_ERROR, 130–136 RECV_FAIL_*, 150–154 SEND_FAIL_* |
| `Network::Error` | 0–6 |
| `Network::BroadcastKeyType` | 0 DAEMON, 1 ZONE |
| `Connection::RecvCompleteErr` | 0 NONE, 1 DISCONNECTED, 2 OVERLAPPEDFAIL, 3 RESULT_ZERO, 4 INVALID_PACKET, 5 INVALID_PACKET2, 6 DECRYPT_FAIL |
| `Connection::SendErr` | 0–4 |
| `PacketProcessor::Type` | 0 PP_AUTHD, 1 PP_GAMED |
| `PacketProcessor::Algorithm` | 0 DEFAULT, 1 TYPE1, 2 TYPE2, 3 TYPE3, 4 TYPE4 |
| `PacketProcessor::ErrCode` | 0 NO_ERROR, 1 INVALID_PARAM, 2 NOT_SUPPORT, 3 ALLOCATE_FAILED, 4 INSUFFICIENT_BUFFER, 5 KEY_SETUP_FAILED, 6 NOT_READY_CRYPTO, 7 ALREADY_SETUP_SESSION_CRYPTO, 8 INVALID_ENC_CODE, 9 INVALID_FIRST_CRYPTO, 10 MORE_PACKET, 11 INVALID_FIRST_PACKET, 12 PRIVATE_KEY_FAILED, 13 KEY_EXCHANGE_FAILED, 14 ENCRYPT_FAILED, 15 DECRYPT_FAILED, 16 NOT_SUPPORT_COM, 17 FAILED_SET_EXT_COMP_FUNC, 18 FAILED_UNCOMP |
| `DataProcessor::ErrCode` | 0–11 |
| `DisconnectGameReason` | 0–19 (8 NO_RESPONSE_SERVER_PING, 10 SEND_BUFFER_OVERFLOW, 11 TOO_MANY_RECEIVED_PACKET, 15 RETURN_TO_LOBBY …) |
| `DisconnectLobbyReason` | 0–11 |
| `ArenaSessionState` | 0–3 |
| `NPSession::ReconnectionStep` | 0 INVALID, 1 DETECTED_DISCONNECTION, 2 RECONNECTING, 3 COUNT |

❓ The full name lists of `Network::Error`, `Connection::SendErr`, `DataProcessor::ErrCode`, `DisconnectGameReason` (all 20) and `DisconnectLobbyReason` are in 419 GM; we give only the ranges here.

---

## 9. Open items (❓)

1. `/ProxyIP` source and the user of slot 5 (§4.1).
2. Server-ping timeout logic (§5.2) and the send-queue limit (§4.4).
3. Event object type values (§4.2) and command type 4 (`0x5d3440`).
4. Meaning of the 4 extra bytes in the next fragments (§2.3).
5. The client 3rd-party step for codes 10 and 11 (§3.3). Out of scope: no anti-cheat details.
6. Reconnect task flow for InfoGate (§5.3).

