# 04 — Client.exe 381: LobbyGate session

Summary:

1. `LobbyGateSession` (vtable `0x15e9a80`, ctor `0xc673e0`, object at `BNSClient+0x30`) is network slot 3 (`LOBBYGATE`). It uses `AuthPacketProcessor` (see [02-transport §1.1](02-transport.md)).
2. Each `LobbyGateCommand` (1..109, `L_CMD_MAX` = 110) has one `StructPacket::Def` row in the table `0x14a0f98` (110 rows × 20 bytes). The row gives the create function, the send-only flag, the fragment flag and the field list. All 110 rows are in [`lobby-commands.tsv`](lobby-commands.tsv).
3. Receive: `onRecv` `0xc88240` (vtable slot 23) calls the handler table `0x14a1830[cmd]`. Most handlers call `LobbyGateSession::ProcessLobbyNetObjectEvent` `0xcb86d0`. Account and goods packets go to `BAccount::ProcessLobbyNetObjectEvent` `0x128a7f0` (`BAccount` at `BNSClient+0x54`) and to `Lobby` (`BNSClient+0x48`).
4. Session start: `OnConnect` `0xc979f0` → `L_HelloReq` (1) → `L_HelloOk` (2) → `L_ContentsRuleListReq` (106) → `L_ContentsRuleListOk` (107) → `L_LoginWithNPTokenReq` (3) or, on reconnect, `L_LoginWithLobbyTokenReq` (6).
5. 381 has no LobbyGate timer of its own. `LobbyGateSession::Tick` `0xc59720` is a 1-byte `retn`. The keepalive ping (cmd 9) comes from the network layer every 30 s (`0x5d28f0`, [02-transport](02-transport.md)).

```
 Network connect (slot 3)
   └─► OnConnect 0xc979f0: +0x8C=0, +0x90=0, clear 0xc5f870, send L_HelloReq (0xc978b0, cmd 1)
 L_HelloOk (2) ─► ReceiveHelloOk 0xcb8660
   ├─ SetHandshakeParam (vt+0x34) fails ─► return (no action)
   └─ +0xB8=1 ─► send L_ContentsRuleListReq (0xc967d0, cmd 106)
        ├─ sent ─► +0xBB=1
        └─ not sent ─► log "ReceiveHelloOk. failed to send L_ContentsRuleListReqPacket" ─► BUiLobby::ExitGameReq
 L_ContentsRuleListOk (107) ─► 0xcb8390: parse rules ─► GLobbygateContentsRuleConfig
   └─ parsed and +0xBB==1 ─► +0xBB=0 ─► +0x54==0 ? L_LoginWithNPTokenReq 0xc97810 : L_LoginWithLobbyTokenReq 0xc97780
 L_LoginWithNPTokenOk (4) ─► RecieveLoginWithNPTokenOk 0xc732b0  (and BAccount 0x12468b0)
 L_LoginWithLobbyTokenOk (7) ─► stub 0xcc3f20: SetDaemonBroadcastKey(pkt+0x58) ─► BAccount 0x123fe30
 UI ─► Lobby::Request* (0xd0d0a0..0xd0d7c0) ─► LobbyGateSession::Send* (0xc96730..0xc978b0) ─► 0x47d530 / 0x5d2740 send
```

Sources: all facts come from the 381 database `ida/lanes/cx/Client.exe.i64`. "419 GM" marks a fact (name, type, enum) that comes only from `Client_gm.exe.i64`. ❓ marks a fact that we did not verify. Many 381 PDB names near this code are false matches (identical code folding). Example: the sender of cmd 1 has the name `Lobby::OnEnterWorldFail` (`0xc978b0`), and the sender of cmd 9 has the name `LobbyGateSession::ReceiveKickNotify` (`0xca1ac0`). Use the addresses and the command ids, not the names. The call `Log::Write` at `0x5d2150` in each sender returns the allocator interface; it is not a log call.

Server side: [LobbyGate handlers](../../Server/LobbyGate/05-handlers.md), [framing](../../Server/LobbyGate/03-framing.md), [handshake](../../Server/LobbyGate/02-handshake.md), [`Program/Server/Packet/tables/381.json`](../../../Program/Server/Packet/tables/381.json).

Files next to this chapter:

| File | Content |
| --- | --- |
| [`lobby-commands.tsv`](lobby-commands.tsv) | All 110 `LobbyGateCommand` ids: name, direction, Def flags, create function, sender, receive stub, switch case, handler (session and `BAccount`), GM handler name, server table role, every field (offset, name, type) |
| [`04-lobby-enums-gm.tsv`](04-lobby-enums-gm.tsv) | `DisconnectLobbyReason` (12), `LOBBY_EVENT` (54), `LobbyGatePacketSync` (2) from 419 GM |

## Quick find

| Task or question | Function (381 name or role) | 381 address | GM name (419) | Section |
| --- | --- | --- | --- | --- |
| Where the client receives a LobbyGate packet | `onRecv` | `0xc88240` | `LobbyGateSession::onRecv` | §2.1 |
| Handler table (index = cmd) | data | `0x14a1830` | `0x19401c8` (cmd 0) | §2.1 |
| Packet Def table (fields per cmd) | data `LobbyGatePacket_define` | `0x14a0f98` | `0x193f930` | §2.2 |
| Main receive switch | `LobbyGateSession::ProcessLobbyNetObjectEvent` | `0xcb86d0` (switch `0xcb870a`) | same | §2.3 |
| Account / goods receive switch | `BAccount::ProcessLobbyNetObjectEvent` | `0x128a7f0` | same | §2.4 |
| What happens on connect | `OnConnect` (vt slot 6) | `0xc979f0` | `LobbyGateSession::OnConnect` | §1.2 |
| Where the client sends Hello | sender cmd 1 | `0xc978b0` | `SendHelloReq` | §1.2 |
| Handshake reply | `ReceiveHelloOk` | `0xcb8660` | same | §1.3 |
| Contents rule list request / reply | sender cmd 106 / handler cmd 107 | `0xc967d0` / `0xcb8390` | `SendContentsRuleListRequest` / `ReceiveContentsRuleListOk` | §1.4 |
| Login with NP token / lobby token | senders cmd 3 / cmd 6 | `0xc97810` / `0xc97780` | `SendLoginWithNPTokenReq` / `SendLoginWithLobbyTokenReq` | §1.5 |
| Login fail to UI | `ProcessSystemLoginFail` | `0xc72b30` | same | §1.5 |
| Disconnect reason to message | `OnClose` (vt slot 8) | `0xc67290` | `LobbyGateSession::OnClose` | §1.6 |
| Network error | `OnError` (vt slot 4) | `0xca7080` | `LobbyGateSession::OnError` | §1.6 |
| Keepalive ping (cmd 9) | `SendPing` slot 12 → sender | `0x4d61a0` → `0xca1ac0` | `SendPing` / `sendPingRequest` | §1.7 |
| Reset session data | `clear` | `0xc5f870` | `LobbyGateSession::clear` | §1.1 |
| World list request | `SendWorldListRequest` | `0xc97710` | same | §3.1 |
| Character list request | `SendCharacterListRequest` | `0xc976a0` | same | §3.2 |
| Create character | `SendMakeCharacterRequest` | `0xc97480` | same | §3.3 |
| Delete / cancel / recover | senders cmd 39 / 43 / 46 | `0xc973c0` / `0xc97300` / `0xc97240` | `SendRemoveCharacterRequest` / `SendCancelRemoveCharacterRequest` / `SendRecoverCharacterRequest` | §3.4 |
| Select character (enter-world token) | sender cmd 13 | `0xc970a0` | `SendEnterWorldTokenRequest` | §3.5 |
| Enter-world token reply | handler cmd 14 | `0xc71970` | `ReceiveEnterWorldTokenOk` | §3.5 |
| Connect to GameD with the lobby token | `BNSClient::EnterGameForNPWithLobbyGate` | `0x1157fbd` (call site) | same | §3.5 |

## 1. Session state machine

### 1.1 Object and fields

`LobbyGateSession` is 192 bytes in 419 GM. The field names come from the GM type. The 381 column gives the 381 code that uses the same offset (verified offsets only).

| Offset | GM name | Type | 381 use |
| --- | --- | --- | --- |
| `+0x00` | `PacketSession` base | — | vtable `0x15e9a80` (`0xc67430`) |
| `+0x30` | Def table pointer (in `PacketSession`) | `const Def *` | set to `0x14a0f98` in ctor (`0xc674a7`) |
| `+0x34` | `_userGuid` | GUID | ❓ |
| `+0x44` | `_gameToken` | GUID | ❓ |
| `+0x54` | `_isReconnect` | bool | `0xcb8390` selects cmd 3 (0) or cmd 6 (1); cases 5, 26, 29 of `0xcb86d0` skip the UI when set |
| `+0x58` | `_lobbyToken` | i64 | ❓ (used by cmd 6) |
| `+0x60` | `_npCliKey` | i64 | ❓ (used by cmd 6) |
| `+0x68` | `_enterWorldId` | int | ❓ |
| `+0x6C` | `_charSlotBodyList` | `vector<SlotBody>` | erased by `clear` `0xc5f870` |
| `+0x7C` | `_worldInfoList` | `vector<WorldInfo>` | erased by `clear` `0xc5f870` |
| `+0x8C` | `_disconnectLobbyReason` | `DisconnectLobbyReason` | set to 0 in `OnConnect`; read in `OnClose` |
| `+0x90` | `_lobbyTokenFailReason` | int | set to 0 in `OnConnect`; case 8 writes the `L_LoginWithLobbyTokenFail` error code (`0xcb88cd`) |
| `+0x94` | `_lobbyGateAddress` | wstring | ❓ |
| `+0xB0` | `_lobbyGatePort` | u16 | ❓ |
| `+0xB4` | `_reqPingRemainTime` | float | ❓ not used in 381 (Tick is empty) |
| `+0xB8` | `_handshakeFinish` | bool | 1 in `ReceiveHelloOk`; 0 in `clear` |
| `+0xB9` | `_isReceivedWorldList` | bool | 0 in `clear`; case 26 tests it |
| `+0xBA` | `_isReceivedLobbyServiceEntrySlotList` | bool | 0 in `clear` |
| `+0xBB` | `_proceedLoginWhenReceivingContentsRuleList` | bool | 1 after cmd 106 is sent; 0 in `0xcb8390` |

### 1.2 Vtable `0x15e9a80`

| Slot | Byte offset | 381 address | GM name | Note |
| --- | --- | --- | --- | --- |
| 4 | `+0x10` | `0xca7080` | `OnError` | §1.6 |
| 6 | `+0x18` | `0xc979f0` | `OnConnect` | sets `+0x8C` = 0, `+0x90` = 0, calls `clear`, then sends cmd 1 (`0xc978b0`) |
| 8 | `+0x20` | `0xc67290` | `OnClose` | §1.6 |
| 10 | `+0x28` | `0x5d4a10` | `PacketSession::OnRecv` | |
| 12 | `+0x30` | `0x4d61a0` | `SendPing` | calls `0xca1ac0` (cmd 9) |
| 13 | `+0x34` | `0x5d4b00` | `SetHandshakeParam` | called by `ReceiveHelloOk` |
| 14 | `+0x38` | `0x5d4b10` | `SetDaemonBroadcastKey` | called by stub cmd 7 with `packet+0x58` |
| 15 | `+0x3C` | `0x5d4b90` | `SetZoneBroadcastKey` | |
| 23 | `+0x5C` | `0xc88240` | `onRecv` | §2.1 |

### 1.3 States

381 has no state variable. The state is the set of flags in §1.1. We name the states here for the reader:

| State | Flags | Enter by | Leave by |
| --- | --- | --- | --- |
| S0 Closed | all 0 after `clear` | `OnClose`, `OnError` | connect request (slot 3) |
| S1 Connected | `+0xB8` = 0 | `OnConnect` `0xc979f0` (cmd 1 sent) | cmd 2 |
| S2 Handshake done | `+0xB8` = 1, `+0xBB` = 1 | `ReceiveHelloOk` `0xcb8660` (cmd 106 sent) | cmd 107 or cmd 108 |
| S3 Login sent | `+0xBB` = 0 | `0xcb8390` (cmd 3 or cmd 6 sent) | cmd 4/5 or cmd 7/8 |
| S4 Logged in | — | cmd 4 (`0xc732b0`) or cmd 7 (stub `0xcc3f20`) | world/character requests (§3), close |

Transitions and failures:

| From | Event | Action | 381 address |
| --- | --- | --- | --- |
| S1 | cmd 2 `L_HelloOk`, `SetHandshakeParam` returns false | return; no other action | `0xcb8674` |
| S1 | cmd 2, key set | `+0xB8` = 1; send cmd 106 | `0xcb8678` |
| S1 | cmd 106 send fails | log (`LobbyGateSession.cpp` line 1334, category `0x201`) `ReceiveHelloOk. failed to send L_ContentsRuleListReqPacket` (`0x15d9720`); `BUiLobby::ExitGameReq` when `Ui(+0x44)+0x18` ≠ 0 | `0xcb8690` |
| S2 | cmd 107 data valid, `+0xBB` = 1 | `+0xBB` = 0; `+0x54` = 0 → cmd 3; else cmd 6 | `0xcb85bc`, `0xcb85cb` |
| S2 | cmd 107 data not valid | log `L_ContentsRuleListOkPacket invalid data. version(%d.%d), reloadCount(%d), dataCount(%d)` (`0x15d9898`, line 3094) | `0xcb85f1` |
| S2 | cmd 107 null packet | log `received L_ContentsRuleListFail. null packet` (`0x15d9948`) | `0xcb8613` |
| S2 | cmd 108 | log `received L_ContentsRuleListFail. reason(%d)` (`0x15d99a8`, line 3121, category `0x202`); `BUiLobby::ExitGameReq` | `0xcb8340` |
| any | cmd 109 `L_ContentsRuleReloadNotify` | send cmd 106 again (`0xc967d0`) | case `0xcb8aca` |
| S3 | cmd 5 `L_LoginWithNPTokenFail` | not reconnect: error 11 → `ProcessSystemLoginFail(1, 204)`; error 13 → `(13, 0)`; other → `(1, 200)` | `0xcb8740` → `0xc72b30` |
| S3 | cmd 8 `L_LoginWithLobbyTokenFail` | `+0x90` = error code (packet `+0x10`) | `0xcb88cd` |
| S4 | cmd 10 `L_KickNotify` | `0xca18d0` (GM `ReceiveKickNotify`) | `0xcb8731` |

### 1.4 Contents rules (cmd 106 → 107/108/109)

- Sender `0xc967d0`: 16-byte packet, cmd 106, no fields (Def row 106 has 0 fields). Callers: `ReceiveHelloOk` `0xcb867f`, case 109 `0xcb8acc`, `0xca18c0` ❓.
- Handler `0xcb8390`: the packet has `major_version` i16 `+16`, `minor_version` i16 `+18`, `reload_count` i32 `+20`, `contents_rule_data` repeat `+24` (each element a vbin). The handler adds the lengths of all elements, joins the bytes into one buffer, adds a 0 byte, and calls `0x10a3aa0(…, 0x19f1ff8, 14, 44, buffer, size)` ❓ (config parser). On success the result replaces the global `GLobbygateContentsRuleConfig` (default object `0x1492f3c`).
- Server side: [LobbyGate handlers §5](../../Server/LobbyGate/05-handlers.md).

### 1.5 Login

| Cmd | Sender / handler | What the client does |
| --- | --- | --- |
| 3 `L_LoginWithNPTokenReq` | `0xc97810` (48-byte packet) | fields `np_game_token` i128 `+16`, `portal_account_id` i128 `+32` (419 GM names, 381 offsets) |
| 4 `L_LoginWithNPTokenOk` | `0xc732b0` `RecieveLoginWithNPTokenOk`; also `BAccount` `0x12468b0` | ❓ body not read |
| 5 `L_LoginWithNPTokenFail` | case `0xcb8740` | §1.3 |
| 6 `L_LoginWithLobbyTokenReq` | `0xc97780` (32-byte packet) | `cli_key` i64 `+16`, `lobby_token` i64 `+24` |
| 7 `L_LoginWithLobbyTokenOk` | stub `0xcc3f20` | `SetDaemonBroadcastKey(&packet.broadcast_key)` (`+0x58`); if false, return false. Then `BAccount` (`0x123fe30`, GM `BAccount::ReceiveLoginWithLobbyTokenOk`). The session switch has no case 7. |
| 8 `L_LoginWithLobbyTokenFail` | case `0xcb88cd` | stores the error code in `+0x90` |

`ProcessSystemLoginFail` (`0xc72b30`, GM name) takes (kind, code). We did not read its body ❓.

### 1.6 Close and error

`OnClose` `0xc67290` switches on `_disconnectLobbyReason` (`+0x8C`, values in [`04-lobby-enums-gm.tsv`](04-lobby-enums-gm.tsv)) and then calls `clear`:

| Reason | Name (419 GM) | Call |
| --- | --- | --- |
| 2 | `DUPLICATE_LOGIN_KICKED_LOBBYGATE` | `0x1129a90(0, 0, 0, 12)` |
| 3 | `CREDENTIAL_CHANGED_KICKED_LOBBYGATE` | `0x1129a90(0, 0, 0, 13)` |
| 4 | `EXCEED_SECOND_PASSWORD_TRY_LIMIT` | `0x1129a90(0, 0, 0, 21)` |
| 5 | `SYSTEM_KICKED_LOBBYGATE` | `0x1129a90(2001, 0, 0, 8)`, then `0x1129a90(0, 0, 0, 22)` |
| 11 | `KICK_BY_SHUTDOWN` | `0x1129a90(0, 0, 0, 22)` |
| other | — | `clear` only |

`0x1129a90` shows a system message in the UI ❓ (first argument: message id; fourth: type).

`OnError(kind, a, b)` `0xca7080`: when `kind` = 2 and `BNSClient+0x2C` ≠ 0, it calls `NPSession::SendLogoutRequest(0)` on that object (381 PDB name; address not recorded ❓). Then it calls `0x1157e30(a, b, 0)` ❓ and `clear`.

### 1.7 Timers

| Timer | Value | Source |
| --- | --- | --- |
| Keepalive `L_PingReq` (cmd 9) | 30.0 s, from the network layer | `0x5d28f0` → slot 12 `0x4d61a0` → `0xca1ac0` |
| `LobbyGateSession::Tick` | none (1-byte `retn`) | `0xc59720` |
| Ping reply timeout | none found in 381 ❓ | — |

## 2. Commands

### 2.1 Receive path

`onRecv` `0xc88240`: read the command (packet vt+12). If `0x14a1830[cmd]` ≠ 0, call it with (session, packet). Else, if the slot is connected (`0x5d2840`), call `0x5d26e0(session, 105)` ❓ (disconnect with code 105).

Handler stubs (in `0xc5f280`..`0xcc4050`) have four forms:

| Form | Commands | Action |
| --- | --- | --- |
| A | 2, 4, 5, 8, 10, 14..54 (S>C), 72..94 (S>C), 107..109 | `BNSClient+0x30` → `ProcessLobbyNetObjectEvent` `0xcb86d0` (some first call `BAccount`) |
| B | 55, 57, 58, 60, 61, 63, 64, 66, 67, 69, 70, 99..102, 104, 105 | `BAccount` (`BNSClient+0x54`, sub-object `+4`, vt+4), then `Lobby` (`BNSClient+0x48`, sub-object `+4`, vt+4) |
| C | 96, 97 | `Lobby` only |
| D | 11 `L_AnnounceNotify` | `0xc595d0` (GM `LobbyNotificationPanel::Show`) |
| E | 7 | §1.5 |

The exact form and target of each command are in [`lobby-commands.tsv`](lobby-commands.tsv) (columns `recv_stub_381`, `recv_case_381`, `handler_381`, `baccount_381`).

### 2.2 Def table `0x14a0f98`

Row = 20 bytes (419 GM type `StructPacket::Def`): `create` (fn), `sendOnly` (u8, 1 = C>S), `fragment` (u8), `keyField` (i32, −1 = none), `fieldCount` (i32), `field` (pointer). Field = 16 bytes (`StructPacket::FieldDef`): `type` (i32), `size` (i16, filled at run time from `StructPacket::FieldSize`), `offset` (u32), `def` (sub-Def pointer for struct/repeat).

`StructPacket::FieldType` (419 GM): 0 `T_INT8`, 1 `T_INT16`, 2 `T_INT32`, 3 `T_INT64`, 4 `T_INT128`, 5 `T_STRING`, 6 `T_SBIN`, 7 `T_VBIN`, 8 `T_SU`, 9 `T_MSEC`, 10 `T_PROP_PAIR`, 11 `T_PROP_SYNC`, 12 `T_STRUCT`, 13 `T_SUB`, 14 `T_REPEAT`, 15 `T_SEQ`, 16 `T_PACKET_REF`, 17 `T_FIELD_MAX`. The TSV uses `i8 i16 i32 i64 i128 str sbin vbin su msec prop_pair prop_sync struct sub repeat seq pref`.

381 vs 419 GM: the field types and offsets of all 110 rows are the same, and the flags are the same. Only exception: the GM field array of the `…Fail` packets shows offset 0; 381 shows `16:i16` (the error code). The TSV uses the 381 values.

Only `L_LoginWithLobbyTokenOk` (7) has a key field (5 = `broadcast_key`). Fragment = 1 for cmds 7, 25, 28, 60, 63, 66, 72, 104, 107 (see TSV).

### 2.3 Session switch `0xcb86d0`

Jump table at `0xcb870a`; default target `0xcb8aef` (returns false, no action). Cases with a body (381 handler → GM name):

| Cmd | 381 handler | GM name |
| --- | --- | --- |
| 2 | `0xcb8660` | `ReceiveHelloOk` |
| 4 | `0xc732b0` | `RecieveLoginWithNPTokenOk` |
| 5 | inline → `0xc72b30` | `ProcessSystemLoginFail` |
| 8 | inline | (stores reason) |
| 10 | `0xca18d0` | `ReceiveKickNotify` |
| 14 / 15 | `0xc71970` / `0xc718b0` | `ReceiveEnterWorldTokenOk` / `Fail` |
| 16 / 18 | `0xca6fd0` / `0xca6f30` | `ReceiveStartWaitNotify` / `ReceiveGetWaitInfoOk` |
| 21 | `0xc66e30` (only if `BNSClient+0x48` ≠ 0) | `ReceiveGiveupWaitOk` |
| 23 | `0xc71890` | `ReceiveEndWaitNotify` |
| 25 / 26 | `0xc72b70` / inline | `ReceiveWorldListOk` / fail: if `+0xB9` set, `Lobby::ProcessLobbyEvent2` event 14 (`REFRESH_WORLDLIST_FAILURE`); else (not reconnect) `0x1129a90(403,0,0,8)` and `ProcessSystemLoginFail(1, 201)` |
| 28 / 29 | `0xc87b70` / inline | `ReceiveCharacterListOk` / fail: (not reconnect) `0x1129a90(404,0,0,8)` and `ProcessSystemLoginFail(1, 202)` |
| 31 | `0xc67080` | `ReceiveGetCharacterInfoOk` |
| 37 / 38 | `0xc72600` / `0xc725a0` | `ReceiveMakeCharacterOk` / `Fail` |
| 40 / 41 / 42 | `0xc721a0` / `0xc71ec0` / `0xc719a0` | `ReceiveRemoveCharacterOk` / `Fail` / `ReceiveEndWaitRemoveCharacterNotify` |
| 44 / 45 | `0xc66e60` / `0xc71ca0` | `ReceiveCancelRemoveCharacterOk` / `Fail` |
| 47 / 48 | `0xc66cd0` / `0xc71670` | `ReceiveRecoverCharacterOk` / `Fail` |
| 50 / 51 | `0xc66980` / `0xc714e0` | `ReceiveBindCharacterSlotOk` / `Fail` |
| 53 / 54 | `0xc66900` / `0xc71480` | `ReceiveGetChargeInfoOk` / `Fail` |
| 72 / 73 | `0xc66200` / `0xc87b00` | `ReceiveGetLobbyServiceEntryListOk` / `Fail` |
| 75..94 | see TSV | `ReceiveChangePc*`, `ReceiveExpandSlot*`, `ReceiveChangeUserName*`, `ReceiveExtendJumpingCharacterQuota*` |
| 107 / 108 / 109 | `0xcb8390` / `0xcb8340` / `0xc967d0` | `ReceiveContentsRuleListOk` / `Fail` / `ReceiveContentsRuleReloadNotify` |

Commands 7, 19, 22, 32 and all C>S ids go to the default case.

### 2.4 `BAccount` switch `0x128a7f0`

`BAccount` (ctor `0x128e100`, 0x288 bytes, stored at `BNSClient+0x54` by `BNSClient::Initialize` `0x116fb65`). Packet interface vtable `0x14d1030`, slot 1 = `0x128a7f0`.

| Cmd | 381 target | GM name |
| --- | --- | --- |
| 4, 5, 8, 99..102 | `0x12468b0` | `BAccount::ReceiveLoginWithNpTokenOk` ❓ (folded name; it forwards to the object at `this+0x14C`, vt+4) |
| 7 | `0x123fe30` | `BAccount::ReceiveLoginWithLobbyTokenOk` |
| 28 | `0x1254b60` | `BAccount::ReceiveCharacterListOk` |
| 37 | `0x1254a80` | `BAccount::ReceiveCreateCharacterOk` |
| 40 / 42 / 44 | `0x1246b00` / `0x1246a60` / `0x123ff20` | `ReceiveDeleteCharacterOk` / `ReceiveEndWaitCharacterDeletionNotify` / `ReceiveCancelWaitCharacterDeletionOk` |
| 84 | `0x123fe70` | `BAccount::ReceiveRenameCharacterOk` |
| 93 | stores packet `+0x20` at `this+0x200`, then goods | `ReceiveExtendJumpingCharacterCreationQuotaOk` |
| 104 | `0x127ef40` | `BAccount::receivePCCafeMileageFeverTimeInfo` |
| 55..91 (goods, S>C) | `0xf438c0` | `GoodsInventory::ProcessLobbyNetObjectEvent` |

### 2.5 Senders

Each sender allocates the packet through the allocator interface (`0x5d2150`) or `_aligned_malloc`, writes the vtable and the command (`word [p+4]`), fills the fields, checks the slot (`0x5d2840`) and sends with `0x5d2740` (or `0x47d530(session, packet)`, which does the same check). When the slot is not connected, the sender deletes the packet and returns false.

| Cmd | Sender | Caller (UI / Lobby) |
| --- | --- | --- |
| 1 | `0xc978b0` | `OnConnect` `0xc979f0` |
| 3 / 6 | `0xc97810` / `0xc97780` | `0xcb8390` |
| 9 | `0xca1ac0` | `SendPing` `0x4d61a0` |
| 13 | `0xc970a0` | `UiLobbyStateSelect::RequestEnterWorld` `0x810919`, `0xd0d64e` |
| 13 | `0xc98a60` ❓ (second builder of cmd 13) | `BUiWorld::ShowCharacterInfo` `0x78e826`, `0x815665`, `0x81568b` |
| 17 / 20 | `0xc97030` / `0xc96fc0` | ❓ |
| 24 | `0xc97710` | `RequestSelectWorldRefreshWorldList` `0x82dfd2`, `ReturnToLobby` `0xc979e0`, `0xd0d780` |
| 27 | `0xc976a0` | `ReceiveNPCharacterListSuccessNotify` `0xca1a91` |
| 30 | `0xc975e0` | `0xd0d160` |
| 36 | `0xc97480` | `Lobby::RequestCreateCharacter` `0xd0d6d0` |
| 39 / 43 / 46 | `0xc973c0` / `0xc97300` / `0xc97240` | `0xd0d6b0` / `0xd0d690` / `0xd0d670` |
| 49 | `0xc97170` | `Lobby::RequestBindCharacterSlot` `0xd0d5c0` |
| 52 | `0xc96f50` | ❓ |
| 56 / 59 / 62 | `0xf39d50` / `0xf39c00` / `0xf39990` | `GoodsInventory` (sends on GameD `BNSClient+0x24` when connected, else on LobbyGate) |
| 65 / 68 | `0xf438c0` ❓ / `0xf396b0` | goods |
| 71 | `0xd0d140` | Lobby |
| 74 / 77 / 80 | `0xc96df0` / `0xc96b10` / `0xc96c80` | `0xd0d480` / `0xd0d340` / `0xd0d3e0` (use `ComposeFsdMakePc` `0xce37f0`) |
| 83 | `0xc96a10` | `Lobby::RequestChangePcName` `0xd0d2a0`, `RequestExclusiveChangePcName` `0xd0d210` |
| 86 | `0xc96970` | `Lobby::RequestExpandSlot` `0xd0d180` |
| 89 | `0xc968b0` | `0xd0d0a0` |
| 92 | `0xc96730` | ❓ |
| 95 | `0xd18d20` | `Lobby::RequestCreateUserName` |
| 103 | `0x127f130` (381 name `BAccount::OnReconnectInfoGate` ❓) | — |
| 106 | `0xc967d0` | §1.4 |
| 12, 33, 98 | no sender found in 381 ❓ | — |

## 3. Lobby flows

### 3.1 World list

Cmd 24 (no fields) → cmd 25 `L_GetWorldInfoListOk` (fragment) → `ReceiveWorldListOk` `0xc72b70`. Element `LD_WorldInfo` (419 GM names, 381 offsets): `world_id` i16 `+4`, `world_cluster_id` i16 `+6`, `world_name` str `+8`, `world_type` i8 `+16`, `play_type` i8 `+17`, `public_address` i32 `+20`, `public_port` i16 `+24`, `world_score` i32 `+28`, `world_live` i8 `+32`, `entrance` i8 `+33`, `character_creation` i8 `+34`, `character_creation_limited_on_new_account` i8 `+35`, `recommend` i8 `+36`, `brandnew` i8 `+37`, `crowdness` i8 `+38`. ❓ How the handler fills `+0x7C`.

### 3.2 Character list

Cmd 27 (no fields) → cmd 28 `L_GetCharacterInfoListOk`: `time` i64 `+16`, `time_zone_bias` i16 `+24`, `character_info_list` repeat `+28` of `LD_CharacterInfo`: `pc_id` i64 `+8`, `slot_id` i128 `+16`, `world_id` i16 `+32`, `character_name` str `+36`, `showcase_data` vbin `+44`, `last_play_end_date` i64 `+56`, `state` i8 `+64`, `jumping_character_state` i8 `+65`, `delete_remain_time` i32 `+68`. Handlers: `ReceiveCharacterListOk` `0xc87b70` and `BAccount` `0x1254b60`. Slot fill: `fillPcSlotWithCharacterInfo` `0xc70230` ❓.

### 3.3 Create

Cmd 36 `L_CreateCharacterReq` (56-byte packet, ctor `0x4d5d70`): `slot_id` i128 `+16`, `world_id` i16 `+32`, `character_name` str `+36`, `character_making_data` vbin `+44` (made by `ComposeFsdMakePc` `0xce37f0`), `jumping_character` i8 `+52`. Reply cmd 37: `LD_CharacterInfo` struct `+16`, i32 `+88`; cmd 38: error code.

### 3.4 Delete, cancel, recover, bind

Cmds 39, 43, 46 (32-byte packets): `slot_id` i128 `+16`. Cmd 49: 40-byte packet (fields in TSV). Replies: §2.3 and §2.4.

### 3.5 Select and hand-over to GameD

Cmd 13 `L_GetEnterWorldTokenReq` (32-byte packet): `slot_id` i128 `+16`. Server side: [handlers §7](../../Server/LobbyGate/05-handlers.md). Reply cmd 14 `L_GetEnterWorldTokenOk`: `enter_world_token` i64 `+16` → `0xc71970` (GM `ReceiveEnterWorldTokenOk`; GM also has `processReciveEnterWorldToken`). Cmd 15: error code → `0xc718b0`.

The GameD connect with the token is in `BNSClient::EnterGameForNPWithLobbyGate` (call to connect `0x5d2680` at `0x1157fbd`, see [02-transport](02-transport.md)). ❓ We did not read `0xc71970` and `0x1157fbd`: the source of the GameD address (world `public_address`/`public_port`?) and the fields of `G_LobbyTokenAuthReq` are not verified here. Old notes give `G_LobbyTokenAuthReqPacket`: `world_id` i16 `+16`, `cli_key` `+24`, `lobby_token` `+32` (`../client-exe-reversing.md` §2).

### 3.6 Worked example (field values)

`L_GetEnterWorldTokenReq` built by `0xc970a0`: packet object 32 bytes; `+4` command = 13 (`0x000d`); `+16` `slot_id` = the 16-byte slot id of the selected `SlotBody` (read through `vector<SlotBody>::operator[]` `0x4d8380`). The Def row 13 says send-only 1, fragment 0, 1 field (`16:i128`). The wire header and field coding are in [02-transport](02-transport.md) and [LobbyGate framing](../../Server/LobbyGate/03-framing.md).

## 4. Search anchors

| Anchor | Address | Leads to |
| --- | --- | --- |
| `LobbyGateSession.cpp` path string | `0x15d9798` | log calls in `0xcb8390`, `0xcb8340`, `0xcb8660` |
| `ReceiveHelloOk. failed to send L_ContentsRuleListReqPacket` | `0x15d9720` | `0xcb8660` |
| `received L_ContentsRuleListFail. reason(%d)` | `0x15d99a8` | `0xcb8340` |
| `received L_ContentsRuleListFail. null packet` | `0x15d9948` | `0xcb8390` |
| `L_ContentsRuleListOkPacket invalid data. …` | `0x15d9898` | `0xcb8390` |
| `GLobbygateContentsRuleConfig` default | `0x1492f3c` | `0xcb8390` |
| `mov word ptr [reg+4], <cmd>` in `0xc96730..0xc97930` | — | senders (§2.5) |
