# 05 — Client.exe 381: game session and command dispatch

Summary:

1. One class, `GameSession` (vtable `0x15896b8`), serves GameD (network slot 4) and ArenaD (slot 8). The `BNSClient` object (pointer at `0x1a4c0f8`) holds the GameD session at `+0x24`, the ArenaD session at `+0x28` and the "in arena" flag at `+0x38`.
2. Receive: `GameSession::onRecv` (`Session` vtable slot 23, `0xf44030`) filters the packet by its `PacketDest`, then calls `GamePacketHandler[cmd]` (`0x149ec40`, 1,599 entries). An unknown command closes the session with reason 105. 925 handlers forward to `World::ProcessNetObjectEvent` (`0x749550`).
3. Send: `BNSClient::SendToGamed` (`0x114ce20`) selects GameD or ArenaD from `PacketDest` and the arena flag. It blocks a packet when its sync group (`PacketSync`, 1–18) has a request that is not answered yet (10 s time-out).
4. All 1,598 MainCommands are in [`game-commands.tsv`](game-commands.tsv): 1,099 server → client (have a handler), 499 client → server (`sendOnly` = 1). There is no overlap and no gap.
5. The packet field descriptors (`StructPacket::Def`, `0x1496f40`) are present in 381. [`packet-layouts.tsv`](packet-layouts.tsv) has every field of every command (4,521 rows, nested to depth 3), with the 419 GM field names.

```
 SERVER → CLIENT                                         CLIENT → SERVER
 socket → Connection::recvNext 0x5cfb70 (ch. 02)         UI / game code: builder (game-commands.tsv builder_381)
   → ToPacket 0x5d4c10 → setField 0x5d4f20 (§3.3)           │ new packet: create fn from Def table (§3)
   → Session::OnRecv 0x5d4a10                               ▼
   → GameSession::onRecv 0xf44030 ── PacketDest filter   BNSClient::SendToGamed 0x114ce20
        │                           (§1.3)                 │ PacketDest → GameD (+0x24) or ArenaD (+0x28) (§1.4)
        ▼                                                  │ TaskSession lock check (§4)
   GamePacketHandler[cmd] 0x149ec40 (§2)                   ▼
        ├─ World::ProcessNetObjectEvent 0x749550 (925)   send helper 0x47d530 → Network::Send 0x5d2740
        ├─ BAccount::ProcessNetObjectEvent 0x128c650 (32)  → ToStream 0x5d4d20 → field writer 0x5d52c0 (§3.3)
        ├─ World::<NetWorld/NetArena event> (13)           → TaskSession::Sent (§4)
        └─ own code (129: hello, auth, ping, enter world …)
```

Sources: all facts come from the 381 database `ida/lanes/cx/Client.exe.i64`. "419 GM" marks a name or type that comes only from `Client_gm.exe.i64`. ❓ marks a fact that we did not verify. Many 381 IDA names of the handler functions are false matches (for example `0xef6b60` has the 381 name `HandleImproveItemOkPacket`, but it is the handler of command 1). Use the `handler_gm_name` column, not the 381 IDA name.

Related chapters: transport, framing, crypto and the `Session` vtable: [02-transport](02-transport.md). STS login: [03-sts-client](03-sts-client.md). Old notes: [client-exe-reversing §2](../client-exe-reversing.md). Server side: [GameDaemon session handlers](../../Server/GameDaemon/10-session-handlers.md), [GameDaemon handshake key](../../Server/GameDaemon/08-client-handshake-key.md), [GameDaemon enter world](../../Server/GameDaemon/05-enter-world.md), [LobbyGate framing](../../Server/LobbyGate/03-framing.md).

Files of this chapter:

| File | Rows | Content |
| --- | ---: | --- |
| [`game-commands.tsv`](game-commands.tsv) | 1,598 | one row per MainCommand: direction, handler, route, Def entry, size, dest, sync, builder |
| [`packet-layouts.tsv`](packet-layouts.tsv) | 4,521 | one row per field (nested), from the 381 Def table, with 419 GM names |
| [`05-game-dispatch-functions.tsv`](05-game-dispatch-functions.tsv) | 335 | named functions and globals of this area (63 core rows + 272 `ProcessNetObjectEvent` receivers) |

---

## Quick find

| Task or question | Function or object | 381 address | GM name (419 GM) | Section |
| --- | --- | --- | --- | --- |
| Where the client gets a game packet | `GameSession::onRecv` | `0xf44030` | `GameSession::onRecv` | §1.3 |
| Handler of command N | `GamePacketHandler[N]` | `0x149ec40` + 4·N | `GamePacketHandler` | §2 |
| Name of command N | name table | `0x1953620` + 4·N | enum `MainCommand` | §2 |
| Field layout of command N | `GamePacket_define[N]` | `0x1496f40` + 20·N | `GamePacket_define` | §3 |
| Where the client sends a game packet | `BNSClient::SendToGamed` | `0x114ce20` | same | §1.4 |
| GameD or ArenaD? | `BNSClient::isInArena` | `0x401690` | same | §1.2 |
| Hello to GameD / ArenaD | `GameSession::OnConnect` | `0xf2c0c0` | same | §1.5 |
| Hello reply | `HandleHelloOkPacket` | `0xf70280` | same | §1.5 |
| Server ping reply | `HandleServerPingNotifyPacket` → `SendServerPingReply` | `0xf4b430` → `0xf3b230` | same | §1.5 |
| Disconnect reason → UI event | `GameSession::OnClose` | `0xefbd80` | same | §1.6 |
| Bad stream hex dump | `GameSession::onStreamError` | `0xf4b550` | same | §1.6 |
| Second-level dispatch (world objects, UI) | `World::ProcessNetObjectEvent` | `0x749550` | same | §2.2 |
| "Request still pending" lock | `TaskSession::IsLocked` / `Sent` / `Received` | `0x86ed10` / `0x86ed90` / `0x880fa0` | same | §4 |
| Sync group names | `GetPacketSyncEnumString` | `0x867870` | same | §4 |
| Field wire encoding (read) | `PacketSession::setField` | `0x5d4f20` | same | §3.3 |
| Field wire encoding (write) | field writer | `0x5d52c0` | `PacketSession::structure2stream` ❓ | §3.3 |
| Byte example | G_HelloReq | — | — | §5 |

Search anchors: string `Source\ClientEXE\Private\GameSession.cpp` `0x15762b0` (ClientLog file path); `ReceiveHelloOk. failed to send G_ContentsRuleListReqPacket` `0x1576440`; `received server ping but couldn't reply. session connected:%d, blocked:%d, pingCount:%I64d, pingServerTime:%I64d` `0x15764b8`; `G_TalkSessionSyncGroup_SYNC` `0x1662ab4`; `G_CMD_MAX` `0x1698ec0`; constant `1599` (`0x63f`) in `0xf44030` and `0xefbf40`; constant `1278` (`0x4fe`, game packet version) in `0xf2c0c0` and `0x47d790`.

---

## 1. GameSession

### 1.1 Object (`GameSession::GameSession`, `0xefbf40`)

Size 120 bytes (419 GM type `GameSession`; the 381 constructor writes the same offsets). Base: `PacketSession` (52 bytes, see [02 §1.2](02-transport.md)).

| Off | Name (419 GM) | Type | 381 value or use |
| ---: | --- | --- | --- |
| 0 | vftable | | `0x15896b8` |
| 4 | `nagle` | bool | ctor: 1 |
| 5 | `printNetworkLog` | bool | ctor: 0 |
| 40 | `fragsize` | int | ctor: 0; fragment rest, [02 §2.3](02-transport.md) |
| 44 | `cmdmax` | int | ctor: **1599** |
| 48 | `define` | `StructPacket::Def*` | ctor: **`0x1496f40`** (§3) |
| 52 | `_sessionId` | char | network slot: 4 GameD, 8 ArenaD; `Id()` `0x47d780` returns it |
| 56 | `_otp` | int64 | ctor: 0 |
| 64 | `_npCliKey` | int64 | ctor: 0 |
| 72 | `_worldId` | int16 | ctor: 0 |
| 76 | `_address` | u32 | ctor: 0 |
| 80 | `_accountIP` | `wchar_t*` | `GameSession::MakeAccountIp` `0xefbbf0` |
| 84 | `_macAddress` | `wchar_t*` | `GameSession::MakeMacAddress` `0xefba60` |
| 88 | `_reqPingRemainTime` | float | ❓ not read in 381 |
| 92 | `_handshakeFinish` | bool | 0 in ctor, OnConnect, OnClose, OnError; `SetHandshakeFinish` `0xee11d0` |
| 93 | `_isReceivedlobbyTokenAuthOk` | bool | same resets |
| 96 | `_disconnectGameReason` | `DisconnectGameReason` | read by OnClose / OnError (§1.6) |
| 100 | `_proceedEnterWorldWhenReceivingContentsRuleList` | bool | ctor: 0 |
| 104 | `_receivedServerPing` | `std::vector<ReceivedServerPing>` | cleared in OnConnect, OnClose, OnError |

The vtable slots are in [02 §1.2](02-transport.md). This chapter adds: `IsUrgent` `0xee1130` returns true only for command 17 (`G_ServerPingNotifyCmd`). `SendPing` `0x47d7a0` calls the `G_PingReq` (16) builder `0xf3b2a0`. `Version` `0x47d790` returns 1278.

### 1.2 Two sessions in `BNSClient`

`BNSClient` instance pointer: `0x1a4c0f8` (381; IDA shows a false name `EInterfaceSetClientInfo+1`). Offsets are from 381 code. Names are from 419 GM; in GM the offsets are `+0x30` higher because GM has more fields before them.

| 381 off | Name (419 GM) | Used by |
| ---: | --- | --- |
| `+0x24` | `_game` (GameD `GameSession*`) | `IsConnectedGame` `0x11230b0`, `SendToGamed` |
| `+0x28` | `_arena` (ArenaD `GameSession*`) | `IsConnectedArena` `0x11230a0`, `SendToGamed` |
| `+0x38` | `_enterArena` (bool) | `isInArena` `0x401690`; `onRecv` `0xf44030` |
| `+0x4c` | `_world` (`World*`) | 938 handlers (§2.2) |
| `+0x54` | `_account` (`BAccount*`) ❓ (by offset shift) | 32 handlers (§2.2) |

`IsConnected(session)` = `0x5d2840`: the global `Network` exists, session ≠ null, `0 ≤ Id() < 10`, and `Network.connection[Id()]` ≠ null.

### 1.3 Receive filter (`GameSession::onRecv`, `0xf44030`)

`dest` = packet vtable `+16` (`Destination()`, value of `StructPacket._dest`). `id` = this session's `Id()`. `arena` = `BNSClient+0x38` (false when `BNSClient` is null).

| `dest` (419 GM `PacketDest`) | Packet dropped when | Else |
| --- | --- | --- |
| 0 `CURRENT` | `arena` and `id == 4`; or not `arena` and `id == 8` | dispatch |
| 1 `ARENAD` | `id == 4` | dispatch |
| 2 `GAMED` | `id == 8` | dispatch |
| 3 `GAMED_ONLY` | `id == 8`, or `arena` | dispatch |
| 4 `BOTH` and other values | never | dispatch |

Dispatch:

1. If `0 ≤ cmd < 1599` and `GamePacketHandler[cmd]` ≠ null: when the Tencent safe mode flag is set (byte at `0x1a82429`), `BNSClient::GetTencentSafeMode()` ≠ null and `0x865e90(cmd)` is true (byte table `0x1952fd8`, 1,599 entries), call `BNSClient::HandleSafeMode` first. Then call `GamePacketHandler[cmd](session, packet)`.
2. Else, if the session is connected, call `Session::Close(session, 105)` (`0x5d26e0` → close event `0x5d6030`). The server must not send a command that has no client handler: the client drops the connection.

419 GM writes a log line for each drop and each dispatch. The 381 build has no such strings.

### 1.4 Send routing (`BNSClient::SendToGamed`, `0x114ce20`)

Arguments: packet, `noSync` (bool). Return: true when the packet went to a connection. When the packet is not sent, the function deletes it (vtable slot 0 with flag 1).

1. If the byte `0x1a4c312` = 1: delete, return false. ❓ (the server-ping handler also reads this byte as "blocked").
2. `sync` = packet vtable `+20` (`PacketSync()`), `cmd` = vtable `+12`, `dest` = vtable `+16`.
3. Select the session:

| `dest` | Target session | Condition |
| --- | --- | --- |
| 0 `CURRENT`, 4 `BOTH` | ArenaD (`+0x28`) when `arena`; else GameD (`+0x24`) | target connected |
| 1 `ARENAD` | ArenaD | `arena` and ArenaD connected |
| 2 `GAMED` | GameD | GameD connected |
| 3 `GAMED_ONLY` | GameD | not `arena` and GameD connected |
| other | none | — |

Note: `BOTH` (4) does **not** send to two sessions in 381. It acts as `CURRENT`.

4. If `noSync` = false, `sync` ≠ 0 and the TaskSession exists: call `TaskSession::Update` (time-outs), then `TaskSession::IsLocked(sync)`. If locked: delete, return false (§4).
5. Send with `0x47d530` (connected check, then `Network::Send` `0x5d2740`; else delete).
6. If sent and `noSync` = false and `sync` ≠ 0: `TaskSession::Sent(sync, cmd, extra)`. `extra` = the dword at packet `+20` for cmd 1226 (`G_UseSkillReqCmd`) and 1228 (`G_UseSkillToGroundReqCmd`), else 0.
7. If sent and `cmd` is 414 (`G_CheckRoundTripTimeReqCmd`) or 415: `TaskSession::SentPing`.

The `dest` of each command is a constant that the create function writes (column `dest` in [`game-commands.tsv`](game-commands.tsv)). Counts: C2S `CURRENT` 280, `GAMED` 128, `GAMED_ONLY` 77, `ARENAD` 12, `BOTH` 2. S2C `CURRENT` 604, `GAMED` 311, `GAMED_ONLY` 150, `ARENAD` 23, `BOTH` 11. The server must write the same `dest` only in memory: `_dest` is not a wire field (§3.2).

### 1.5 Connect, hello and ping

```
 Network connect OK ──► GameSession::OnConnect 0xf2c0c0
                          clear ping vector, _handshakeFinish=0, _isReceivedlobbyTokenAuthOk=0, _disconnectGameReason=0
                          new G_HelloReqPacket (cmd 11, 56 B) ──► 0x5d2740 (direct, not SendToGamed)
 G_HelloOk (12) ──► HandleHelloOkPacket 0xf70280
                          Id()==4 (GameD):  SendContentsRuleListReq 0xf2bef0 (cmd 929, 16 B, dest GAMED)
                                            fail → ClientLog "ReceiveHelloOk. failed to send G_ContentsRuleListReqPacket" (line 903), BUiWorld::ExitGame
                          Id()==8 (ArenaD): GameSession::SendEnterArenaReq 0xf2bf60
 G_ServerPingNotify (17) ──► 0xf4b430
                          if byte 0x1a4c312 == 0: SendServerPingReply 0xf3b230 → G_ServerPingReq (18, 32 B, dest 4 BOTH) → 0x47d530 (direct)
                          else / not connected: ClientLog "received server ping but couldn't reply…" (line 956)
```

`G_HelloReqPacket` values set by `OnConnect` (381):

| Field (419 GM) | Off | 381 value |
| --- | ---: | --- |
| `gamePacketVersion` i16 | 16 | 1278 (`0x4fe`) |
| `propertyVersion` i64 | 24 | `0x00012D26_1D120403` (high dword 77094, low dword 487719939) |
| `publisher` string | 32 | `PUBLISHER_NCSOFT` → `"default"` (`0x14fb4d4`); else the name from table `0x19498ec` (`ncsoft`, `tencent`, `innova`, `ncjapan`, `garena`, `nctaiwan`, `ncwest`, `rncs`…); out of range → empty |
| `extraPropertyVersion` i64 | 40 | `1024 << 32` for TENCENT, INNOVA, NCJAPAN, GARENA, NCTAIWAN, NCWEST and the six `R*` publishers; 0 for the others |
| `authData` vbin | 48 | not set (empty) ❓ |

`G_ServerPingReq` (18) builder `0xf3b230(session?, pingCount lo, hi, serverTime …)`: writes 4 dwords at `+16..+28` = `pingCount` i64, `serverTime` i64 (copied from the notify), dest value 4 (`BOTH`), and sends with `0x47d530`. The ping vector and its time-out are in 419 GM `GameSession::Tick` and `sendPingRequest`; ❓ no 381 match.

### 1.6 Close and error (`OnClose` `0xefbd80`, `OnError` `0xf44150`)

Both clear the ping vector and the two handshake flags, then switch on `_disconnectGameReason` (`+96`). They call `BNSClient::DisconnectProcess` (`0x1129a90`, 419 GM signature `(int, int, int, int)`) with `(code, 0, 0, event)`. ❓ `event` is probably a `LOBBY_EVENT` value (see [03 §5](03-sts-client.md)).

| `+96` (419 GM `DisconnectGameReason`) | OnClose call | OnError `code` |
| ---: | --- | --- |
| 0 NONE | `World::LeaveWorld` (NetWorldEventHandler) | — |
| 3 KICK_DUPLICATE_LOGIN | `(0, 0, 0, 12)` | 0 |
| 4 KICK_CREDENTIAL_CHANGED | `(0, 0, 0, 13)` | 0 |
| 5 KICK_BY_GM | `(0, 0, 0, 16)` | 0 |
| 6 KICK_BY_SYSTEM | `(1001, 0, 0, 8)` | 0 |
| 7 KICK_BY_XIGNCODE | `(0, 0, 0, 23)` | 0 |
| 8 NO_RESPONSE_SERVER_PING | `(1003, 0, 0, 17)` | 0 |
| 9 INVALID_RESPONSE_SERVER_PING | `(1004, 0, 0, 18)` | 0 |
| 10 SEND_BUFFER_OVERFLOW | `(1005, 0, 0, 19)` | 0 |
| 11 TOO_MANY_RECEIVED_PACKET | `(1006, 0, 0, 20)` | 0 |
| 13 MOVEFAIL_ARENA | `(0, 0, 0, 9)` | ❓ |
| 14 MOVEFAIL_DUEL | `(0, 0, 0, 10)` | ❓ |
| 16 EXCEED_SECOND_PASSWORD_TRY_LIMIT | `(0, 0, 0, 21)` | ❓ |
| 17 KICK_BY_SHUTDOWN | `(0, 0, 0, 22)` | ❓ |
| 19 MOVEFAIL_BATTLEROYAL | `(0, 0, 0, 43)` | ❓ |
| 1, 2, 12, 15, 18 | no call | — |

OnError skips the switch when `_sessionId == 8` and the reason is 12 (DISCONNECTED_ARENA). ❓ The enum values are from 419 GM; the 381 switch has the same case set, so we think they match. ❓ The 381 code that writes `+96` is not found yet (419 GM: `GameSession::Disconnect(DisconnectGameReason)` `0x117cca0` GM address).

`onStreamError` `0xf4b550`: makes a hex string (`%02x` per byte) of the first 900 bytes at most and writes it to ClientLog (`GameSession.cpp` line 569).

---

## 2. Command dispatch and [`game-commands.tsv`](game-commands.tsv)

### 2.1 Tables

| Table | 381 address | Entries | Entry |
| --- | --- | ---: | --- |
| Command names | `0x1953620` | 1,600 | `wchar_t*`; 0 = `None`, 1599 = `G_CMD_MAX` |
| `GamePacketHandler` | `0x149ec40` (.rdata) | 1,599 | `bool (*)(const GameSession*, const StructPacket*)`; 1,099 non-null |
| `GamePacket_define` | `0x1496f40` (.rdata) | 1,599 | `StructPacket::Def`, 20 B (§3.1) |
| Safe-mode flags | `0x1952fd8` | 1,599 | byte |

Direction rule (exact for all 1,598 ids): handler ≠ null ⇔ `Def.sendOnly` = 0 ⇔ S2C (1,099). Handler = null ⇔ `sendOnly` = 1 ⇔ C2S (499).

419 GM rule (1,163 of 1,163 checked): the handler of `G_<X>Cmd` is `Handle<X>Packet`, and the struct is `G_<X>Packet`. 381 ids 1–918 are equal to GM ids. For 919–1598 use the `gm_id` column (match by name). 381 has no command that GM does not have.

### 2.2 Handler routes (column `handler_route`)

Most handlers are 10-instruction forwarders: load `BNSClient`, load an object field, add a base-class offset, call a vtable slot with the packet. The return value is the handler result.

| Route | Count | Object (381) | Interface slot (419 GM) |
| --- | ---: | --- | --- |
| `World::ProcessNetObjectEvent` | 925 | `BNSClient+0x4c` +4 | `NetObjectEventHandler` slot 3 → `0x749550` |
| `BAccount::ProcessNetObjectEvent` | 32 | `BNSClient+0x54` +8 | `NetObjectEventHandler` slot 3 → `0x128c650` ❓ |
| `World::EnterLeaveObject`, `EncounterOutOfSightObject`, `InZoneObject`, `InSightObject`, `OutSightObject`, `EnterZone`, `LeaveZone`, `GodSays` | 8 | `BNSClient+0x4c` +8 | `NetWorldEventHandler` slots |
| `World::EnterArenaLobby`, `LeaveArenaLobby`, `CreateArenaLobbyRoomNotify`, `EnterArenaLobbyNotify`, `LeaveArenaLobbyNotify` | 5 | `BNSClient+0x4c` +16 | `NetArenaEventHandler` slots ❓ (GM slot order) |
| `own` | 129 | — | handler has its own code (hello, auth, ping, enter world, ok/fail stubs …) |

`World::ProcessNetObjectEvent` (`0x749550`, 2,011 pseudocode lines) is a second switch on the command. It calls `World::processAction` for 6/7, gives property and object packets to the objects, and passes packets to the UI and subsystem receivers. 272 functions in 381 have a `ProcessNetObjectEvent` name (list in [`05-game-dispatch-functions.tsv`](05-game-dispatch-functions.tsv); these names are PDB matches, verify each one). ❓ The full map command → final receiver is not made yet.

### 2.3 Columns of `game-commands.tsv`

| Column | Meaning |
| --- | --- |
| `id`, `name` | MainCommand id and name (381 name table) |
| `direction` | `S2C` or `C2S` (§2.1 rule) |
| `gm_id` | id of the same name in 419 GM |
| `handler_381` | entry of `GamePacketHandler` (S2C only) |
| `handler_gm_name` | `Handle<X>Packet` (419 GM rule) |
| `handler_route` | §2.2 |
| `handler_shared` | blank (no two commands share a handler address in 381) |
| `gm_has_handler` | `yes` when GM has a handler for this name |
| `create_381` | `Def.create` (makes the packet object) |
| `vtable_381` | vtable that `create` writes. Packets with identical code share one folded vtable (906 rows use `0x179cdf0`, 211 of them with no field; its 381 name is false). |
| `struct_gm_name` | `G_<X>Packet` |
| `struct_size` | bytes that `create` allocates (in memory, not wire) |
| `send_only`, `fragment`, `key_field`, `field_count`, `fielddef_381` | `StructPacket::Def` fields (§3.1) |
| `dest`, `dest_name` | `_dest` that `create` writes (§1.4) |
| `packet_sync` | return value of vtable slot 5 `PacketSync()` (§4) |
| `builder_count`, `builder_381`, `builder_how`, `builder_ida_name_381` | C2S only: functions that make this packet. `vtable`: functions that use the packet vtable. `id-filter`: of the functions that use a shared vtable, the ones that store this command id at `+4`. 471 of 499 C2S rows have a builder; 28 have none (ids 38, 77, 121, 127, 224, 226, 230, 307, 354, 359, 360, 362, 368, 395, 404, 788, 820, 855, 887, 1044, 1137, 1205, 1231, 1270, 1272, 1341, 1361, 1536) ❓. |

Other facts from the table:

- 63 commands have `fragment` = 1 (large S2C packets; framing in [02 §2.3](02-transport.md)): 1–9, 32, 81, 189, 267, 325, 353, 355, 357, 366, 372, 375, 441, 478, 516, 519, 556, 558, 561, 643, 649, 706, 715, 746, 755, 765, 766, 770, 784, 797, 800, 803, 806, 808, 850, 897, 907, 911, 922, 930, 943, 990, 1083, 1084, 1110, 1122, 1126, 1147, 1153, 1157, 1159, 1183, 1214, 1332, 1552.
- `key_field` ≠ -1 only for 32 `G_EnterWorldOkCmd` (11), 41 `G_PrepareEnterZoneCmd` (6), 649 `G_ReturnToGameOkCmd` (8), 706 `G_EnterArenaOkCmd` (8). ❓ meaning (419 GM name `keyField`).
- 244 commands have no field (16-byte packet: header only).
- Largest structs: 32 `G_EnterWorldOkCmd` 1,384 B; 649 `G_ReturnToGameOkCmd` and 706 `G_EnterArenaOkCmd` 1,304 B.

Generator: one IDA Python run in this slice (cx-05) on the 381 database, with two JSON side files from 419 GM (enum `MainCommand` and the GM handler table `0x193d488` GM address). Steps: read the three tables of §2.1; for each `create` function, follow at most one tail `jmp` and read the first `push imm` before `call eax` (size), `mov word [reg+4], imm` (command id; 1,597/1,597 match, id 494 needed `create_insn` at `0x586bf0`), `mov [reg+8], imm|zero-reg` (dest), `mov [reg], offset` (vtable).

---

## 3. StructPacket descriptors and [`packet-layouts.tsv`](packet-layouts.tsv)

Answer to the card question: **yes**, the 381 exe has all field descriptors. The table is `GamePacket_define` (`0x1496f40`). `PacketSession.define` points to it (§1.1).

### 3.1 Types (419 GM, 381 data matches)

`StructPacket::Def` (20 B):

| Off | Name | Type | 381 data |
| ---: | --- | --- | --- |
| 0 | `create` | `void* (*)()` | allocs `struct_size` (aligned 16), sets `_command`, `_dest`, `_pending`=0, vtable |
| 4 | `sendOnly` | bool | 1 for all C2S; `ToPacket` raises an exception for it ([02 §2.4](02-transport.md)) |
| 5 | `fragment` | bool | 1 = large packet, [02 §2.3](02-transport.md) |
| 8 | `keyField` | int | -1 or a field index (§2.3) |
| 12 | `fieldCount` | int | |
| 16 | `field` | `FieldDef*` | array in `.data` |

`StructPacket::FieldDef` (16 B): `type` int @0 (`StructPacket::FieldType`), `size` i16 @4, `offset` u32 @8 (offset in the struct, top level starts at 16), `def` `Def*` @12 (sub-struct, element or variant table).

`StructPacket` (16 B, in memory): vftable @0, `_command` i16 @4, `_dest` `PacketDest` @8, `_pending` bool @12. Packet vtable slots used in this chapter: +0 delete, +12 `Command()`, +16 `Destination()`, +20 `PacketSync()`.

Sub-structs (`def` ≠ 0) are `Def` entries too (create fn = element factory). 223 distinct sub-defs in 381. In 419 GM they have names `GR_<Name>RptBody_define` (for `T_REPEAT`) and similar; the `gm_subdef_name` column gives them.

### 3.2 Field types in 381 data

| Code | Name | Rows in TSV | Wire form (from the reader and writer, §3.3) |
| ---: | --- | ---: | --- |
| 0 | T_INT8 | 1,188 | `size` bytes, copied (1) |
| 1 | T_INT16 | 1,152 | 2 bytes LE |
| 2 | T_INT32 | 516 | 4 bytes LE |
| 3 | T_INT64 | 842 | 8 bytes LE |
| 4 | T_INT128 | 5 | 16 bytes |
| 5 | T_STRING | 244 | `i16 n` + `n` UTF-16LE code units (no terminator) |
| 6 | T_SBIN | 0 | `size` bytes, copied |
| 7 | T_VBIN | 66 | `i16 n` + `n` bytes |
| 8 | T_SU | 45 | `size` bytes, copied |
| 9 | T_MSEC | 0 | `size` bytes, copied |
| 10 | T_PROP_PAIR | 4 | read: 5-byte head `i8 table, i16 a, i16 len`; if all > 0, `len` bytes decoded by `0x5d4810` with `PropertyPairFieldInfo/Max` (GameSession slots 21/22). Write: nothing, skip `size` ❓ |
| 11 | T_PROP_SYNC | 42 | read: `PropertySync::Set` `0x5d4600` returns the length ❓ format |
| 12 | T_STRUCT | 54 | sub-struct fields in line (recursive). Write: the writer does not write it ❓ |
| 13 | T_SUB | 12 | read only: `u8 len` (whole, 2 head bytes included) + `i8 variant` + body; `variant` selects `def[variant]` (20 B steps); check `len == body + 2`. Write: returns -1 (not supported) |
| 14 | T_REPEAT | 351 | `i16 count` + `count` elements (recursive). In memory: linked list, first at field `+4`, next at element `+4` |
| 15 | T_SEQ | 0 | — |
| 16 | T_PACKET_REF | 0 | read: skip 1 byte (3 if the ref def has `fragment`), then recursive ❓ |

There is no alignment or padding on the wire: fields follow each other in `field` order. The `offset` column is only the place in the C++ struct. The `size` is 0 for variable types.

### 3.3 Reader and writer

| Step | 381 | Behaviour |
| --- | --- | --- |
| bytes → struct | `PacketSession::setField` `0x5d4f20(stream, len, def, struct)` | loop over `def.field`; returns bytes used, -1 on error (unknown type with a sub-def, failed sub-read, variant without a def, end of buffer before the last field) |
| struct → bytes | `0x5d52c0(def, struct, out, room)` | loop over `def.field`; returns bytes written, -1 on error or no room |
| helpers | `PfString::Set` `0x5d35d0`, `PfVarBin::Set` `0x5d37a0`, list append `0x5d3960`, prop-pair `0x5d4810` | |

### 3.4 Columns of `packet-layouts.tsv`

`cmd_id`, `cmd_name`, `direction`, `path` (field index path, `1/2` = field 2 of the element of field 1), `depth`, `type`, `type_code`, `size`, `offset_381`, `subdef_381`, `subdef_field_count`, `subdef_create_381`, `gm_field_name`, `gm_field_type`, `gm_subdef_name`, `gm_path_found`.

- Names come from 419 GM: the same command by name, the same field index path, and the GM struct member at the GM offset. All 4,521 paths exist in GM. 363 rows have no GM member name (the GM type has no member at that offset ❓).
- Only 3 rows have a type that does not match the GM member size (code 2 against GM `__int64`) ❓.

Manual method (for a command that is not in the TSV, or to check a row): in IDA go to `0x1496f40 + 20·id`, read `field` and `fieldCount`, then read 16-byte entries. For a sub-def, repeat at `def`. Names: 419 GM `G_<X>Packet` type and `GR_*_define` symbols.

---

## 4. Sync groups (`TaskSession`)

`TaskSession` instance pointer: `0x1a4c0fc`. Constructor `0x881340`, `Initialize` `0x86ee10` (called from `BNSClient::Initialize` `0x116f550`).

`PacketSync` ids (381 strings, `GetPacketSyncEnumString` `0x867870`):

| Id | String (381 address) | Id | String (381 address) |
| ---: | --- | ---: | --- |
| 1 | `G_TalkSessionSyncGroup_SYNC` `0x1662ab4` | 10 | `G_QuestSyncGroup_SYNC` `0x1662cbc` |
| 2 | `G_MoveAndWarpSyncGroup_SYNC` `0x1662aec` | 11 | `G_SkillListSyncGroup_SYNC` `0x1662ce8` |
| 3 | `G_ActionSyncGroup_SYNC` `0x1662b24` | 12 | `G_SkillTrainingListSyncGroup_SYNC` `0x1662d20` |
| 4 | `G_OpenItemPouchSyncGroup_SYNC` `0x1662b54` | 13 | `G_PostBoxSyncGroup_SYNC` `0x1662d64` |
| 5 | `G_CloseItemPouchSyncGroup_SYNC` `0x1662b90` | 14 | `G_Craft_SYNC` `0x1662d94` |
| 6 | `G_PickupItemSyncGroup_SYNC` `0x1662bd0` | 15 | `G_Market_SYNC` `0x1662db0` |
| 7 | `G_ManufactureItemSyncGroup_SYNC` `0x1662c08` | 16 | `G_PetitionSyncGroup_SYNC` `0x1662dcc` |
| 8 | `G_ExpandInventorySyncGroup_SYNC` `0x1662c48` | 17 | `G_InstantPaymentSyncGroup_SYNC` `0x1662e00` |
| 9 | `G_InventorySyncGroup_SYNC` `0x1662c88` | 18 | `G_DuelBotTrainingRoomSyncGroup_SYNC` `0x1662e40` |

Valid ids: 1–18 (all functions test `(id - 1) <= 17`). Per-command ids: column `packet_sync` (C2S: 136 rows ≠ 0; S2C: 247 rows ≠ 0).

State machine (one slot per task group):

- `packetSessionToTaskGroupTable` (`0x1a16cf8`, filled by `0x867740`) maps every sync id 1–18 to **group 0** (`TASK_GROUP_GAME`). So in 381 **all 18 sync ids share one lock**: only one sync request can be pending at a time.
- Slot fields (419 GM names): `_packetSyncId[group]`, `_taskGroup[group]` (= command id), time limit (float, seconds), start time (QPC).

| Event | Function | Action |
| --- | --- | --- |
| send check | `IsLocked(sync)` `0x86ed10` | locked when `_packetSyncId[group(sync)]` ≠ 0 → `SendToGamed` drops the packet |
| sent | `Sent(sync, cmd, extra)` `0x86ed90` | if locked return false; else store `sync`, `cmd`, limit **10.0 s**, QPC start; `extra` kept for 1226/1228 |
| time-out | `Update()` `0x876e50` (called in `SendToGamed`) | for each of 4 slots: when elapsed > limit, clear the slot |
| reply | `Received(sync)` `0x880fa0` | only if the slot holds the same `sync`: add a latency history row, clear the slot |
| reply entry | `BNSClient::ReceivePacket(sync, cmd)` `0x115eb30` | cmd 414/415 → `ReceivedPing`; else `Received(sync)` |

Callers of `Received`: `World::processSkillActionToOwner` `0x737a60`, `World::processActionToOwner` `0x744a20`, `SkillTrainingRoom::ProcessNetObjectEvent` `0x994a40`, `0xb9c910`, `0xbbf390`, `BPlayer::ProcessNetObjectEvent` `0xbbfe10`, `DuelBotTrainingRoom::ProcessNetObjectEvent` `0x1081b70`, `BNSClient::ReceivePacket` `0x115eb30` (called by `World::ProcessNetObjectEvent` `0x749550`).

Server rule: answer each C2S request that has `packet_sync` ≠ 0 with a S2C packet of the same `packet_sync`. If not, the client blocks all other sync requests for 10 s.

`TaskSession::Tick` `0x8d69d0` (from `World::Tick` `0x74ce80`) sends `G_CheckRoundTripTimeReq` (414) by timer and writes the network latency log. Timer values from rule config `+764` / `+768` (defaults 60 s and 86,400 s) ❓ field names.

---

## 5. Worked example — `G_HelloReq` (cmd 11, C2S)

Layout rows (from `packet-layouts.tsv`): `0 T_INT16 gamePacketVersion`, `1 T_INT64 propertyVersion`, `2 T_STRING publisher`, `3 T_INT64 extraPropertyVersion`, `4 T_VBIN authData`. Values from §1.5 with publisher `ncwest` (index 6).

Plain packet (before the envelope of [02 §2.1](02-transport.md)):

```
26 00                    size = 38 (header included)
0B 00                    command = 11 G_HelloReqCmd
FE 04                    gamePacketVersion = 1278
03 04 12 1D 26 2D 01 00  propertyVersion = 0x00012D261D120403
06 00                    publisher: 6 UTF-16 units
6E 00 63 00 77 00 65 00 73 00 74 00   "ncwest"
00 00 00 00 00 04 00 00  extraPropertyVersion = 1024 << 32
00 00                    authData: 0 bytes ❓ (not set in OnConnect)
```

The server reply is `G_HelloOk` (12, S2C, handler `0xf70280`); then the client sends `G_ContentsRuleListReq` (929, no fields). Compare with the server side: [GameDaemon session handlers](../../Server/GameDaemon/10-session-handlers.md).

---

## 6. Open items (❓)

- Full map of each command to its final receiver under `World::ProcessNetObjectEvent` `0x749550` (2,011 lines).
- The 129 `own` handlers: only 12, 17 are read. Others (auth 20/21/24/25, return to lobby 27/28, enter world 32 …) are not read.
- 381 writer of `GameSession+96` (`_disconnectGameReason`) and of `+52` (`_sessionId`).
- `T_PROP_PAIR`, `T_PROP_SYNC`, `T_PACKET_REF` byte format; writer behaviour for `T_STRUCT`.
- `keyField` meaning; byte `0x1a4c312` meaning.
- 28 C2S commands without a builder (§2.3).
- 381 ping time-out (`_reqPingRemainTime`) and `GameSession::Tick`.
