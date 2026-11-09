# 12 — Client.exe 381: InfoGate, ArenaD and other servers

Summary:

1. **InfoGate** is a separate TCP server (network slot 9, `InfoGateSession`, vtable `0x15a8608`). The client gets its address, port and a 16-byte token from LobbyGate `L_LoginWithNPTokenOk` and connects in `0xe54080` (§1.2).
2. InfoGate uses its own command set (`InfoGateCommand`, 1–72, `I_*`), its own descriptor table `0x14a0730` and handler table `0x14a0cf8`. Handshake: `I_HelloReq` (1) → `I_HelloOk` (2) → `I_AuthTokenReq` (3) → `I_AuthTokenOk` (4). The client reconnects by itself (30 s, doubled to 300 s, 10 tries) (§1.3–§1.5).
3. InfoGate gives read-only data: season rankings, time-attack and challenge-party records, friend and character search, guild recruit lists, closet, spam filter, GameTV schedule, boast events (§1.6–§1.8). All 72 commands: [`12-infogate-commands.tsv`](12-infogate-commands.tsv); all 224 fields: [`12-infogate-layouts.tsv`](12-infogate-layouts.tsv).
4. **ArenaD** is a second `GameSession` (slot 8) with the same game protocol as GameD. The `PacketDest` field and `BNSClient+0x38` (in-arena flag) select the session (§2).
5. Other servers: web pages through `InGameWeb` / `WebRequester` (HTTP, URLs from config), STS / NP (chapter 03), and no direct CacheGate link (§3).

```
 LobbyGate L_LoginWithNPTokenOk (cli_key, infogate_token, infogate_address, infogate_port)
   └─► ConnectInfoGate 0xe54080 ──► Network connect 0x5d2680 (slot 9, 1 s connect time-out)
         OnConnect 0xe82cb0 ─► I_HelloReq(1) [0xe82c30]
         I_HelloOk(2) ─► SetHandshakeParam ─► I_AuthTokenReq(3) [0xe82b90]
         I_AuthTokenOk(4) ─► SetDaemonBroadcastKey ─► "ready" (+121 = 1)
               └─► first login: friends 0x12890d0, guild info 0xd18b40, I_LoadGameTVScheduleReq(43), I_LoadChatSpamFilterListReq(40)
         game code ─► BNSClient::SendToInfoGate 0x114cdb0 ─► 0x47d530 ─► Network::Send
         server reply ─► onRecv 0xe66070 ─► InfoGateHandler[cmd] 0x14a0cf8 ─► ProcessInfoGateNetObjectEvent 0xea4c90 (fan-out)

 GameD (slot 4) ◄── BNSClient+0x24        ArenaD (slot 8) ◄── BNSClient+0x28
        └──── SendToGamed 0x114ce20 selects by PacketDest + BNSClient+0x38 ────┘
```

Sources: all facts come from the 381 database `ida/lanes/cx/Client.exe.i64`. "419 GM" marks a name or type that comes only from `Client_gm.exe.i64`. ❓ marks a fact that we did not verify. Many 381 IDA names in this area are false PDB matches (for example `HyperMove::Tick` at `0xe82b90` is the `I_AuthTokenReq` builder). The tables give the real role.

Related chapters: transport, crypto, `Session` vtable and connect: [02-transport](02-transport.md). Game dispatch, `PacketDest`, `SendToGamed`: [05-game-dispatch](05-game-dispatch.md). STS / NP: [03-sts-client](03-sts-client.md). Guild and friend InfoGate requests: [11-social §3.2, §4](11-social.md). Old notes: [client-exe-reversing §2](../client-exe-reversing.md). Server side: [GameDaemon arena lobby handlers](../../Server/GameDaemon/25-arena-lobby-handlers.md), [GameDaemon arena event handlers](../../Server/GameDaemon/26-arena-event-handlers.md), [GameDaemon season ranking](../../Server/GameDaemon/28-season-ranking-handlers.md), [RankingDaemon](../../Server/RankingDaemon/README.md), [CacheGate](../../Server/CacheGate/README.md).

Files of this chapter:

| File | Rows | Content |
| --- | ---: | --- |
| [`12-infogate-commands.tsv`](12-infogate-commands.tsv) | 72 | one row per `InfoGateCommand`: direction, handler, create function, vtable, size, `Def` flags, GM struct, 381 sender |
| [`12-infogate-layouts.tsv`](12-infogate-layouts.tsv) | 224 | one row per field (nested), from the 381 `Def` table `0x14a0730`, with 419 GM field names |
| [`12-infogate-arena-enums-gm.tsv`](12-infogate-arena-enums-gm.tsv) | 1,335 | 218 enums of this area (419 GM): arena, duel, battle royal, party battle, season, ranking, `InGameWeb::WebContents` |
| [`12-infogate-arena-functions.tsv`](12-infogate-arena-functions.tsv) | 1,859 | every named 381 function of the area (§4) |

---

## Quick find

| Task or question | Function or object | 381 address | GM name (419 GM) | Section |
| --- | --- | --- | --- | --- |
| Where the client gets the InfoGate address | LobbyGate `HandleLoginWithNPTokenOk` → `ConnectInfoGate` | `0xcc3fa0` → `0xe54080` | `InfoGateSession::ConnectInfoGate` | §1.2 |
| InfoGate session object | `InfoGateSession` ctor | `0xe36920` (vtable `0x15a8608`) | `InfoGateSession::InfoGateSession` | §1.1 |
| InfoGate session pointer | `BNSClient+0x34` | `0x1a4c0f8`→`+0x34` | `BNSClient::_infoGate` ❓ | §1.1 |
| Where the client sends an InfoGate packet | `BNSClient::SendToInfoGate` | `0x114cdb0` | same | §1.4 |
| Where the client gets an InfoGate packet | `InfoGateSession::onRecv` | `0xe66070` | same | §1.4 |
| Handler of InfoGate command N | `InfoGatePacketHandler[N]` | `0x14a0cf8` + 4·N | — | §1.4 |
| Field layout of InfoGate command N | `InfoGatePacket_define[N]` | `0x14a0730` + 20·N | — | §1.4, TSV |
| Hello to InfoGate | `OnConnect` → `SendHelloReq` | `0xe82cb0` → `0xe82c30` | `InfoGateSession::SendHelloReq` | §1.3 |
| Auth token to InfoGate | `SendAuthTokenReq` | `0xe82b90` | same | §1.3 |
| Auth token OK (ready) | `receiveAuthTokenOk` | `0xea4af0` | same | §1.3 |
| Hello / auth reply switch | `onResponse` | `0xea4bd0` | `InfoGateSession::onResponse` | §1.3 |
| Reply fan-out to game objects | `ProcessInfoGateNetObjectEvent` | `0xea4c90` | same | §1.6 |
| Reconnect | `ReconnectInfoGate` (from `BNSClient::Tick`) | `0xe54130` (call `0x1167a30`) | same | §1.5 |
| InfoGate ping | `InfoGateSession::SendPing` → `sendPingRequest` | `0x4a5ef0` → `0xe93d00` | same | §1.5 |
| InfoGate connect error log | `InfoGateSession::OnError` | `0xea11f0` | same | §1.5 |
| Season ranking request | `RetrieveCurrentRankingListSummaryReq` / `…ListReq` | `0x9d1360` / `0x9d12d0` | `BSeason::*` ❓ | §1.7 |
| Guild member ranking request | `BSeason::RequestRetireveCurGuildRanker` | `0x9d1240` | same | §1.7 |
| GameD or ArenaD? | `BNSClient::isInArena` | `0x401690` | same | §2.1 |
| Hello OK on ArenaD → enter arena | `HandleHelloOkPacket` → `GameSession::SendEnterArenaReq` | `0xf70280` → `0xf2bf60` | same | §2.2 |
| Arena world object | `ArenaWorld` ctor | `0x123a180` | same | §2.3 |
| Arena lobby object | `ArenaLobby` ctor | `0x123ade0` | same | §2.3 |
| In-game web URLs | `InGameWeb::GetDomainUrl` | `0xe31ae0` | same | §3.1 |
| HTTP request thread | `WebRequester::RequestUri` | `0x7373c0` | same | §3.2 |

Search anchors: string `InfoGateSession Connect Error, ip:%s, port:%d, errorCode:%d, networkErrorCode:%d, infogatePacketVersion:%d, gamePacketVersion:%d` (`0x159ba78`); string `InfoGateSession set info gate broadcast key failed, session: %s(%d)` (`0x159bc70`); `InfoGateSession.cpp` source path (lines 493 and the `OnError` log); constant `73` (`cmdmax`, `mov [reg+2Ch], 49h` in `0xe36920`); `push 1Ch` + `mov word [reg+4], 1` (hello builder); `mov word [reg+10h], 19h` (InfoGate version 25).

---

## 1. InfoGate

### 1.1 `InfoGateSession` object (ctor `0xe36920`, 128 bytes)

Made once by `BNSClient::Initialize` (call at `0x116f7cc`). `BNSClient+0x34` holds it: `SendToInfoGate` `0x114cdb0`, `HandleAuthTokenOkPacket` `0xea5b80` and all 72 handler stubs read `[0x1a4c0f8]+0x34`. (`BNSClient::GetInfoGateSession` `0x4404a0` returns `+0x64`; that 381 name is false ❓.)

| Off | Name (419 GM) | Type | Initial value (ctor `0xe36920`) | Written by (381) |
| ---: | --- | --- | --- | --- |
| 0 | vftable | | `0x15a8608` | ctor |
| 44 | `cmdmax` (`PacketSession`) | int | 73 | ctor |
| 48 | `define` (`PacketSession`) | `Def*` | `0x14a0730` | ctor |
| 56 | `_npCliKey` | i64 | 0 | `ConnectInfoGate` `0xe54080` |
| 64 | `_infoGateToken` | id128 (16 B) | 0 | `ConnectInfoGate` |
| 80 | `_isReconnect` | bool | 0 | 0 in `ConnectInfoGate`; 1 in `ReconnectInfoGate` `0xe54130`; 0 after `receiveAuthTokenOk` |
| 84 | `_reconnect._reconnectInterval` | int (s) | 30 | `receiveAuthTokenOk`, `OnClose` |
| 88 | `_reconnect._maxReconnectInterval` | int (s) | 300 | ctor |
| 92 | `_reconnect._reconnectCount` | int | 10 | `receiveAuthTokenOk` (= max) |
| 96 | `_reconnect._maxReconnectCount` | int | 10 | ctor |
| 100 | `_reconnect._connected` | bool | 0 | 1 in `receiveAuthTokenOk`; 0 in `OnClose`, `OnError`, auth fail |
| 104 | `_reconnect._lastReconnectTime` | u32 (`GetTickCount`) | 0 | `OnClose`, `OnError`, auth fail |
| 108 | `_infoGateAddress` | u32 (IPv4) | 0 | `ConnectInfoGate` |
| 112 | `_infoGatePort` | i16 | 0 | `ConnectInfoGate` |
| 116 | `_reqPingRemainTime` | float | — | ❓ not set in ctor |
| 120 | `_handshakeFinish` | bool | 0 | 1 after `I_HelloOk` (`0xea4bd0`); 0 in connect, reconnect, `SendAuthTokenReq` reads it ❓ |
| 121 | `_authTokenFinish` | bool | 0 | 1 in `receiveAuthTokenOk` `0xea4af0`; 0 in `SendAuthTokenReq` `0xe82b90`, connect, reconnect |

Vtable slots: see [02 §1.2](02-transport.md) (`Id` = 9, `Version` = 25 at `0x4a5ee0`, `CryptAlgorithm` = 0, `onRecv` `0xe66070`, `OnConnect` `0xe82cb0`, `OnClose` `0xe31cc0`, `OnError` `0xea11f0`, `SendPing` `0x4a5ef0`).

419 GM `ClientRuleConfig._useInfogate` (offset 16, bool): `ConnectInfoGate` `0xe54080` returns 0 at once when the byte `[GRuleConfig 0x194f658]+16` is 0. So the region config switches InfoGate off.

### 1.2 Address source (`ConnectInfoGate`, `0xe54080`)

```
 LobbyGate L_LoginWithNPTokenOk handler 0xcc3fa0
   ├─ BAccount (BNSClient+0x54) event, LobbyGateSession::ProcessLobbyNetObjectEvent
   └─ if BNSClient+0x34 ≠ 0: ConnectInfoGate(session, cli_key, infogate_token, infogate_address, infogate_port)
        0xe54080: if GRuleConfig._useInfogate == 0 → return 0
                  if override wstring 0x1a82348 not empty and override port dword 0x1955824 > 0:
                       address = Network::Address(override)  (0xe540b4), port = 0x1955824
                  else address = infogate_address, port = infogate_port
                  store cli_key → +56, token → +64..+79, +80 = 0, +120 = 0, +121 = 0
                  connect request 0x5d2680(session, address, port)
```

Packet fields that the client copies (419 GM `L_LoginWithNPTokenOkPacket`, offsets in memory from the push list at `0xcc3fa0`):

| Mem off | Field (419 GM) | Type | To session |
| ---: | --- | --- | --- |
| 16 | `cli_key` | i64 | `+56` |
| 28 | `infogate_token` | id128 | `+64` |
| 44 | `infogate_address` | int (IPv4, network order ❓) | `+108` |
| 48 | `infogate_port` | i16 | `+112` |

The override values come from `InitializeTCLS` `0x114c110` (419 GM name; Tencent launcher). It splits a string on `;` (`0x14d3f18`) and `:` (`0x14b33e4`) and writes the port with `_wtoi` at `0x114c45a`. ❓ The exact part of the Tencent string is not traced. For the server port: send a real `infogate_address` / `infogate_port` in `L_LoginWithNPTokenOk`; the server side is in [LobbyGate handlers](../../Server/LobbyGate/05-handlers.md).

### 1.3 Handshake

```
 connect OK ─► OnConnect 0xe82cb0 ─► SendHelloReq 0xe82c30
                  new I_HelloReq (cmd 1, 28 B, vtable 0x1589734), packet_version (+16) = 25
                  +120 = 0; if connected: Network::Send 0x5d2740 (direct), else delete
 I_HelloOk (2) ─► stub 0xea5c10 ─► ProcessInfoGateNetObjectEvent 0xea4c90 ─► onResponse 0xea4bd0 case 2
                  if SetHandshakeParam (vtable +52)(handshake_param) is true:
                       +120 = 1; SendAuthTokenReq 0xe82b90
 SendAuthTokenReq 0xe82b90: new I_AuthTokenReq (cmd 3, 40 B)
                  cli_key (+16) = session +56 (8 B); token (+24) = session +64 (16 B); +121 = 0; send direct
 I_AuthTokenOk (4) ─► HandleAuthTokenOkPacket 0xea5b80
                  SetDaemonBroadcastKey(broadcast_key); on failure: ClientLog line 493
                        "InfoGateSession set info gate broadcast key failed, session: %s(%d)" (0x159bc70), return 0
                  else ProcessInfoGateNetObjectEvent ─► onResponse case 4 ─► receiveAuthTokenOk 0xea4af0
 I_AuthTokenFail (5) ─► stub 0xea5b60 ─► onResponse case 5:
                  Session close 0x5d49d0 (reason 130), +100 = 0, +104 = GetTickCount  (reconnect timer runs)
```

`receiveAuthTokenOk` `0xea4af0` (419 GM `InfoGateSession::receiveAuthTokenOk`):

1. `ReconnectTimer::Connected` inline: `+92` = `+96`, `+100` = 1, `+104` = 0, `+84` = min(`+88`, 30).
2. `+121` = 1 (`_authTokenFinish`).
3. First connect (`+80` = 0): `0x12890d0` on `BNSClient+0x54` (419 GM `BAccount::OnConnectInfoGate`; it calls `0xfbd910` → `NPSession::SendMsgRequest`, friend list ❓). If `BNSClient+0x48` exists and its byte `+118` = 1: `0xd18b40` (419 GM `Lobby::RequestPcGuildInfoAll`; sends `I_RetrieveCharacterGuildInfoListReq` 19 through `0xd18a30`), then clear `+118`.
4. Reconnect (`+80` = 1): `0x128a7a0` (419 GM `BAccount::OnReconnectInfoGate`; calls `Friends::updateInvalidFriendsList` `0xfd25c0` and `0xfd2330`, which sends 10 through `0xfbdb30`), the same guild step, then `ClosetGroup::loadCharacterClosetInfoListReq` `0x10e5660` when the closet group (`[BNSClient+0x4c]+164`) is not enabled. Then `+80` = 0.
5. Both paths end with `I_LoadGameTVScheduleReq` (43, `0xe82ab0`) and `I_LoadChatSpamFilterListReq` (40, `0xe82b20`). Both are 16-byte packets with no field.

The packet version 25 is the same value as vtable slot `Version` (`0x4a5ee0`). `handshake_param` in `I_HelloReq` stays empty in 381 (`+20` and `+24` = 0) ❓ (the `AuthPacketProcessor` may fill it, see [02 §3](02-transport.md)).

### 1.4 Send and receive

| Step | 381 | Behaviour |
| --- | --- | --- |
| Send | `BNSClient::SendToInfoGate` `0x114cdb0` | Calls packet vtable `+12` (`Command()`). If `BNSClient+0x34` = 0: delete the packet, return 0. Else send with `0x47d530` (connected check, `Network::Send` `0x5d2740`). There is **no** check of `_authTokenFinish` here; 419 GM `InfoGateSession::CheckReadyInfogate` (`_useInfogate` and connected and `+121`) has no 381 caller found ❓. No `PacketSync` lock (419 GM `InfoGatePacketSync`: only `I_SYNC_NONE` 0, `I_SYNC_MAX` 1). |
| Receive | `InfoGateSession::onRecv` `0xe66070` (slot 23) | If `InfoGatePacketHandler[cmd]` (`0x14a0cf8`) ≠ 0: call it. Else, if connected, close the session with reason 105 (`0x5d26e0`). |
| Handler stubs | `0xea55e0`–`0xea5c10` | 47 non-null entries (one per S2C id). 46 stubs are about 10 instructions: if `BNSClient` and `BNSClient+0x34` ≠ 0, call `ProcessInfoGateNetObjectEvent` `0xea4c90` with the packet. Entry 4 is `HandleAuthTokenOkPacket` `0xea5b80` (§1.3). |
| Descriptors | `0x14a0730` | `StructPacket::Def` (20 B) × 74 (index 0 empty; index 73 is not a command). Same format as `GamePacket_define`: [05 §3](05-game-dispatch.md). |

Direction rule (all 72 ids): `Def.sendOnly` = 1 ⇔ handler = 0 ⇔ C2S (25 ids). S2C = 47 ids. Only `I_RetrieveFriendListOk` (8) has `fragment` = 1; only `I_AuthTokenOk` (4) has `keyField` = 0 (the others -1).

C2S commands and their 381 builders (all in [`12-infogate-commands.tsv`](12-infogate-commands.tsv)):

| Id | Name | Body fields (offset in memory, 419 GM name) | 381 builder | Values the builder writes |
| ---: | --- | --- | --- | --- |
| 1 | `I_HelloReq` | 16 `packet_version` i16, 20 `handshake_param` vbin | `0xe82c30` | 25 |
| 3 | `I_AuthTokenReq` | 16 `cli_key` i64, 24 `token` id128 | `0xe82b90` | from session `+56`, `+64` |
| 6 | `I_PingReq` | — | `0xe93d00` | only when `+120` = 1 |
| 7 | `I_RetrieveFriendListReq` | 16 `user_id_list` repeat of id128 | ❓ `0xfbdbe0` ([11 §4](11-social.md)) | |
| 10 | `I_RetrieveUserIdByCharacterNameReq` | 16 `character_name` string | `0xf38f70`, `0xfbdb30` | `0xf38f70` sets `+740` = 1 on its owner |
| 13 | `I_RetrieveCharacterDetailReq` | 16 `pc_id` i64, 24 `character_name` string | `0xc98a60` ❓ | |
| 16 | `I_SearchCharacterListReq` | see TSV | `0xfbd020` | |
| 19 | `I_RetrieveCharacterGuildInfoListReq` | 16 `search_key` repeat {8 `pc_id` i64, 16 `world_id` i16} | `0xd18a30` | one element per character |
| 22 | `I_LoadCharacterClosetInfoListReq` | 16 `pc_id` i64, 24 `world_id` i16 | `0x10e5660` | |
| 25 | `I_RetrieveCurrentRankingListSummaryReq` | 16 `season_contents_type` i8 | `0x9d1360` | arg |
| 28 | `I_RetrieveCurrentRankingListReq` | 16 `season_contents_type` i8, 17 `job` i8, 18 `page_number` i8 | `0x9d12d0` | args |
| 31 | `I_RetrieveGuildMemberRankingInfoListReq` | 16 `season_contents_type` i8, 24 `guild_id` i64 | `0x9d1240` | guild id from the player guild object `+8` |
| 34 | `I_RetrieveTimeAttackBestRecordReq` | 16 `season_contents_type` i8, 17 `job` i8, 20 `ranking_per_job` i32, 24 `guild_member_pc_id` i64 | `0x105a1a0` | `season_contents_type` = 6 (INFINITE_TOWER_BEST_RECORD) |
| 37 | `I_RetrieveChallengePartyLogSummaryReq` | 16 `season_contents_type` i8, 24 `player_id` i64 | `0x9d5ac0` (false 381 name `BSeason::GetActivatedRankingSeasonState`) | `season_contents_type` = 13 (CHALLENGE_PARTY_BEST_RECORD); keeps the id at owner `+42984` |
| 40 | `I_LoadChatSpamFilterListReq` | — | `0xe82b20` | after auth |
| 43 | `I_LoadGameTVScheduleReq` | — | `0xe82ab0` | after auth |
| 46 | `I_RetrieveGuildJoinRenewalPcInfoReq` | see TSV | `0xf304b0` | [11 §3.2](11-social.md) |
| 49 | `I_RetrieveGuildJoinRenewalPcListReq` | see TSV | ❓ none found | |
| 52 | `I_RetrieveGuildJoinRenewalGuildInfoReq` | see TSV | `0xf30750` | |
| 55 | `I_RetrieveGuildJoinRenewalGuildListReq` | see TSV | ❓ none found | |
| 58 | `I_RetrieveGuildPointInfoListReq` | see TSV | `0xf30180` | |
| 65 | `I_RetrieveOpenGuildRecommendListReq` | 16 `world_id` i16, 18 `faction` i8 | `0x9d51a0` | from player `+36` and `+208`; called by `World::EnterWorld` `0x742865` and `BPlayer` property change `0xbc4ce2` |
| 68 | `I_EnterWorldNotifyReq` | 16 `world_cluster_id` i16 | `World::SendInfoGateEnterWorldNotifyReq` `0x733c30` | cluster id |
| 69 | `I_ReturnToLobbyNotifyReq` | — | `0x733bc0` (419 GM `World::SendInfoGateReturnToLobbyNotifyReq`) | |
| 70 | `I_RetrievePcNameListReq` | 16 `world_cluster_id` i16, 20 `pcdbid_list` repeat i64 | `BoardGachaManager::RequestPcNameList` `0x11d4510` | |

Wire form: plain header (`size` i16, `command` i16, [02 §2.2](02-transport.md)), then the fields in `Def` order with no padding (types in [05 §3.2](05-game-dispatch.md)). The offsets in the tables are C++ struct offsets, not wire offsets.

### 1.5 Reconnect, ping, close and error

| Event | 381 | Behaviour |
| --- | --- | --- |
| Tick | `BNSClient::Tick` `0x1167a30` → `0xe54130` | If not connected and `0x5d5450` (419 GM `ReconnectTimer::CanReconnect` ❓ by position) is true: `+80` = 1, `+120` = 0, `+121` = 0, connect again to `+108` / `+112`. |
| `CanReconnect` (419 GM `0x7bf2a0`) | — | false if `_connected`, or `_reconnectCount` ≤ 0, or `_lastReconnectTime` = 0, or less than `1000 × _reconnectInterval` ms since `_lastReconnectTime`. Else: `_lastReconnectTime` = 0, `_reconnectCount` −1 (unless max = -1), `_reconnectInterval` = min(2 × interval, max). |
| Close | `OnClose` `0xe31cc0` | `+100` = 0, `+104` = `GetTickCount`. If reason = 100 (419 GM `SendLogoutRequest` uses `Network::Close(…, 100)`): `+104` = 0 and `+84` = min(`+88`, 30): no auto reconnect after a logout. |
| Error | `OnError` `0xea11f0` | For event 2 or 3: `+100` = 0, `+104` = `GetTickCount`. For event 2 when `+92` = 0 (no tries left): ClientLog with format `0x159ba78` (`ip`, `port`, `errorCode`, `networkErrorCode`, `infogatePacketVersion`, `gamePacketVersion`). |
| Ping | `SendPing` `0x4a5ef0` → `0xe93d00` | `I_PingReq` (6, 16 B) only after `I_HelloOk` (`+120` = 1). Period: the network keepalive, 30 s ([02 §5.2](02-transport.md)) ❓. There is no `I_PingOk`. |

Reconnect time line with the default values: first retry 30 s after the drop, then 60, 120, 240, 300, 300 … s; at most 10 tries. A good `I_AuthTokenOk` resets the counter to 10 and the interval to 30 s.

### 1.6 Receivers (`ProcessInfoGateNetObjectEvent`, `0xea4c90`)

The function first calls `onResponse` `0xea4bd0` (§1.3), then gives the packet to these `BNSClient` members (in call order). Each one is a virtual call with the packet.

| Order | `BNSClient` member | Vtable slot | Object (by offset, ❓ from 05 / 419 GM) |
| ---: | --- | --- | --- |
| 1 | `+0x54` (+12) | +4 | `BAccount` |
| 2 | `+0x48` | +8 | `Lobby` ❓ |
| 3 | `+0x234` | +4 | ❓ |
| 4 | `+0x23c` | +4 | ❓ |
| 5 | `+0x4c` (+20) | +4 | `World` |
| 6 | `+0x44` → `+24` | +84 | UI state ❓ |
| 7–9 | `+0x44` → `+28`, `+32`, `+40` (+28 or +32 base) | +4 | UI panels ❓ |

381 functions named `*::ProcessInfoGateNetObjectEvent` and the command ids that they compare (from `cmp`/`sub` immediates and `case` labels; small ids can be false ❓):

| 381 | Name | Command ids found |
| --- | --- | --- |
| `0x9c2a20` | `BSeason` | 7, 8, 13–17, 19, 20, 26, 29, 32 |
| `0xa00c40` | `RankingPanel` | 26, 27, 29, 30, 32, 33, 38, 39 |
| `0xb8a570` | `BPlayer` | 26–34, 47–60 |
| `0x1065a50` | `DuelBotChallengeVirtualRacePanel` | 26, 27, 29, 30, 32, 33, 35 |
| `0x837e80` | `UiStateGame` | 11–72 (many) |
| `0xf3df60` | `Guild` | ([11 §3.2](11-social.md)) |
| `0xe96fb0` | `GuildPanel2` | 56, 59, 66 |
| `0x100a180` | `Friends` | 8, 11 |
| `0xfaab50` | `FriendSearchPanel` | 17 |
| `0xc73a00` / `0xc675c0` | `NativeCharacterInfoPanel` (+ `SetNativeCharacterInfoFromInfoGatePacket`) | 14 |
| `0x811d30` / `0xc5f9f0` | `BUiLobby` / `Lobby` | 20 |
| `0x1112e70` / `0x77dfc0` | `ClosetGroup` / `WardrobePanel` | 23 |
| `0x910870` | `SpamFilter` | 41, 61, 62 ❓ (21 and 72 seen) |
| `0x122c110` | `BnsTv` | 44 (GameTV schedule) |
| `0x1197b10` / `0x119c6c0` | `BoastChatShowPanel` / `BoastChatPanel` | 64 |
| `0x1217c80` | `BoardGachaPanel` | 71 |
| `0x9770d0` | `SuggestJoinGuildNotificationPanel` | ❓ |
| `0x752e10`, `0x7dd4e0`, `0x7e4690`, `0x1239230` | `BUiWorld`, `UiStateGuildCustomize`, `BUiGuildCustomize`, `BAccount` | forwarders |

### 1.7 Rankings (what the client gets from InfoGate)

```
 RankingPanel::updateProcess 0xa10a43 ─► 0x9d1360 I_RetrieveCurrentRankingListSummaryReq(25, season_contents_type)
 RankingPanel::requestRankerListPage 0x9d5bd7 ─► 0x9d12d0 I_RetrieveCurrentRankingListReq(28, type, job, page)
 guild tab ─► BSeason::RequestRetireveCurGuildRanker 0x9d1240 ─► 31 (type, guild_id)
 DuelBotChallengeVirtualRacePanel (0x105a2ef, 0x105a38d, 0x106558f) ─► 25, 28, 34
 InfoGate replies 26 / 29 / 32 / 35 / 38 ─► BSeason 0x9c2a20, RankingPanel 0xa00c40, BPlayer 0xb8a570
```

Reply layouts (419 GM names; full list in [`12-infogate-layouts.tsv`](12-infogate-layouts.tsv)):

| Id | Name | Fields (memory offset: name type) |
| ---: | --- | --- |
| 26 | `I_RetrieveCurrentRankingListSummaryOk` | 16 `season_contents_type` i8; 24 `make_time` i64; 32 `ranking_list_summary_list` repeat {8 `job` i8, 9 `total_page_number` i8} |
| 29 | `I_RetrieveCurrentRankingListOk` | 16 `season_contents_type` i8; 17 `job` i8; 18 `page_number` i8; 24 `page_make_time` i64; 32 `ranking_info_list` repeat {8 `ranking` i32, 12 `job` i8, 16 `pc_name` string, 24 `world_id` i16, 28 `guild_name` string, 40 `challenge_party_id` i64, 48 `primary_score` i64, 56 `last_modification_time` i64} |
| 32 | `I_RetrieveGuildMemberRankingInfoListOk` | 16 `season_contents_type` i8; 24 `list_make_time` i64; 32 `guild_member_ranking_info` repeat {8 `pc_name` string, 16 `ranking_per_job` i32, 24 `primary_score` i64, 32 `last_modification_time` i64} |
| 35 | `I_RetrieveTimeAttackBestRecordOk` | 16 `season_contents_type` i8; 17 `job` i8; 20 `ranking_per_job` i32; 24 `guild_member_pc_id` i64; 32 `ranker_name` string; 40 `best_record` string |
| 38 | `I_RetrieveChallengePartyLogSummaryOk` | 16 `season_contents_type` i8; 24 `player_id` i64; 32 `log_summary` string |
| 27, 30, 33, 36, 39 | `…Fail` | 16 `reason` i16 (`FailPacket`) |

`season_contents_type` = 419 GM `Data::SeasonContentsSeasonContentsTypeType` (0–17; see [`12-infogate-arena-enums-gm.tsv`](12-infogate-arena-enums-gm.tsv)): 1 DUEL_DEATH_MATCH_1VS1, 2 DUEL_DEATH_MATCH_3VS3, 3 DUEL_TAG_MATCH_3VS3, 4 PARTY_BATTLE_OCCUPATION_WAR, 5 SOLO_DUNGEON_1_BEST_RECORD, 6 INFINITE_TOWER_BEST_RECORD, 7 PARTY_BATTLE_CAPTURE_THE_FLAG, 8 GUILD_COMBAT, 9 SOLO_DUNGEON_2_BEST_RECORD, 10/11 INGAME_CHAMPIONSHIP_*, 12 PARTY_BATTLE_LEAD_THE_BALL, 13 CHALLENGE_PARTY_BEST_RECORD, 14/15 BATTLE_ROYAL_FREE_FOR_ALL(_PARTY), 16 MASTERY_EXP_BEST_RECORD. The table data is in `SeasonContentsScheduleData` ([client-datafiles](../client-datafiles.md)). Server data source: [RankingDaemon](../../Server/RankingDaemon/README.md) ❓ (InfoGate reads the ranking DB).

Season reward requests (`BSeason::RequestTakeRankingReward` `0x9d1550`) go to **GameD**, not InfoGate ❓ (no `SendToInfoGate` call): see [GameDaemon season ranking](../../Server/GameDaemon/28-season-ranking-handlers.md).

### 1.8 Other InfoGate data

| Ids | Data | Client use |
| --- | --- | --- |
| 7–9 | friend list by user id (`I_RetrieveFriendListOk` is a fragment packet) | `Friends` `0x100a180` ([11 §4](11-social.md)) |
| 10–15 | user id by name, character detail (level, `showcase_data` vbin) | friends, `NativeCharacterInfoPanel` `0xc73a00` |
| 16–21 | character search, character guild info | `FriendSearchPanel`, `Lobby` |
| 22–24 | closet (wardrobe) list | `ClosetGroup` `0x1112e70` |
| 40–42, 61, 62 | chat spam filter list, add, delete (`{i32 id, string word}`) | `SpamFilter` `0x910870` |
| 43–45, 63 | GameTV schedule | `BnsTv` `0x122c110` |
| 46–60, 65–67 | guild recruit (join renewal), guild points, open guild list | `Guild`, `GuildPanel2` ([11 §3.2](11-social.md)) |
| 64 | `I_BoastEventInWorldNotify` (server push) | `BoastChatPanel`, `BoastChatShowPanel` |
| 68, 69 | enter world / return to lobby notify (C2S only, no reply) | presence for InfoGate |
| 70–72 | name list by pc id | `BoardGachaManager`, `BoardGachaPanel` `0x1217c80` |

### 1.9 Worked example — `I_AuthTokenReq` (cmd 3, C2S)

Plain packet before AES (field values are examples; the token is `keyhere`):

```
 1C 00                              size = 28 (4 header + 8 + 16)
 03 00                              command = 3 (I_AuthTokenReq)
 xx xx xx xx xx xx xx xx            cli_key  i64 LE   (from L_LoginWithNPTokenOk.cli_key)
 keyhere (16 bytes)                 token    id128    (from L_LoginWithNPTokenOk.infogate_token)
```

The `Def` field types for id 3 are `T_INT64` then `T_INT128` ([`12-infogate-layouts.tsv`](12-infogate-layouts.tsv)), so the body is 24 bytes. `T_INT128` is a 16-byte copy ([05 §3.2](05-game-dispatch.md)). The second example: `I_HelloReq` = `size` i16, `command` 1, `packet_version` `19 00` (25), `handshake_param` vbin `00 00` (empty) ❓ (the `AuthPacketProcessor` handshake may add data, [02 §3](02-transport.md)).

---

## 2. ArenaD

### 2.1 Session selection

ArenaD is a second `GameSession` object at `BNSClient+0x28` (network slot 8). It uses the game command set and the game `Def` table of chapter 05. There is no separate ArenaD command table.

| Item | 381 address | Behaviour |
| --- | --- | --- |
| `BNSClient::isInArena` | `0x401690` | Returns the byte at `BNSClient+0x38` (`_enterArena`, 419 GM name ❓). It returns 0 when the global `0x1a4c0f8` is null. |
| `BNSClient::GetArenaSession` | `0x1122f00` | Returns the ArenaD session (`+0x28`) ❓ (name only). |
| `BNSClient::IsConnectedArena` | `0x11230a0` | Connected test of the ArenaD session ❓ (name only). |
| `SendToGamed` | `0x114ce20` | Selects GameD or ArenaD from `PacketDest` and `+0x38`. See [05 §1.3–§1.4](05-game-dispatch.md). |

`PacketDest` values (column `dest` of [`game-commands.tsv`](game-commands.tsv)): `CURRENT` 0, `ARENAD` 1, `GAMED` 2, `GAMED_ONLY` 3, `BOTH` 4. 35 commands have `ARENAD`. The arena lobby commands (639–704, 760–784) have `GAMED`. Commands 723–747 (`ArenaEvent*`) have 0 or 3. 784 and 954 have `BOTH`.

### 2.2 Enter-arena flow

```
 GameD                                   Client                                       ArenaD (slot 8)
   │ G_PrepareEnterArena 783 (GAMED) ──► 0xf1ec10
   │   arenaIp, arenaPort, arenaOtpKey      └─ 0x1137560(ip, port, otp)
   │                                           ├─ fail if GameD not connected or ip/port = 0
   │                                           ├─ ArenaD session +56/+60 = arenaOtpKey
   │                                           └─ BNSClient+0x38 = connect 0x5d2680(slot 8) ───► TCP connect
   │                                        World+0x10 NetArenaEventHandler vtable +0x1c (UI)
   │                                                                     ◄── HelloOk (05) 0xf70280
   │                                        GameSession::SendEnterArenaReq 0xf2bf60
   │                                           G_EnterArenaReq 705 (ARENAD) ─────────────────►
   │                                                                     ◄── G_EnterArenaOk 706 / Fail 707
   │                                        0xf70220: SetDaemonBroadcastKey(key @0x4F0) (vtable +0x38)
   │                                           └─ World::EnterArena 0x7415a0
   │                                                BPlayer::Initialize, skill timers, presence cookie
```

| Step | Command | Dest | 381 function | Behaviour |
| --- | --- | --- | --- | --- |
| 1 | 639 `G_ReserveArenaLobbyReq` (C2S, 24 B) | GAMED | `BPlayer::RequestEnterArena` `0xb7d7f0` | Body: `arenaPortalKey` i64. |
| 2 | 640 / 641 `ReserveArenaLobbyOk/Fail` | GAMED | `0xef3d50` / `0xef3d10` | Fan-out to `World::ProcessNetObjectEvent`. |
| 3 | 784 `G_PrepareEnterArenaLobby` (S2C, 88 B) | BOTH | `0xf1ec60` | Sets byte `0x1a4c312` = 0. If ArenaD is connected: `+0x38` = 0, ArenaD `+0x60` (disconnect reason) = 12, close `0x5d26e0` with reason 100. Then `NetArenaEventHandler` slot `+4`. Body: `creatureId`, `roomInfo` (`roomId`, `password` = `keyhere`, `gameMode`, `checkDuplicatedJob`, `allowSpectate`, `detailRule`), `chattingIp/Port/ServerId/UserId`, `mapOption`, `teamList`. |
| 4 | 642 `G_EnterArenaLobbyReq` (C2S, 24 B) | GAMED | `0x1278d20`, `0x12a58b0` | Body: `flag` vbin. |
| 5 | 643 / 644 `EnterArenaLobbyOk/Fail` | GAMED | `0xef3cd0` → `World::EnterArenaLobby` `0x732200` / `0xef3c90` | Lobby UI starts. |
| 6 | 783 `G_PrepareEnterArena` (S2C, 32 B) | GAMED | `0xf1ec10` → `0x1137560` | Body: `arenaIp` i32 @16, `arenaPort` i16 @20, `arenaOtpKey` i64 @24 (`keyhere`). |
| 7 | Hello on slot 8 | — | `HandleHelloOkPacket` `0xf70280` | Same hello as GameD ([05](05-game-dispatch.md)). Then step 8. |
| 8 | 705 `G_EnterArenaReq` (C2S, 40 B) | ARENAD | `GameSession::SendEnterArenaReq` `0xf2bf60` | Needs `World` (`BNSClient+0x4c`) with `+128` ≠ 0. `arenaOtpKey` i64 @16 = ArenaD session `+56`/`+60`; `worldId` i16 @24 = `[World+76]+36`; `worldPcId` i64 @32 = `[[World+76]+128]+8/+12`. Sent with `0x47d530`. |
| 9 | 706 `G_EnterArenaOk` (S2C, 1304 B) | ARENAD | `0xf70220` → `World::EnterArena` `0x7415a0` | Sets the broadcast key from `key` vbin @0x4F0 (1264). On fail: returns 0. If the Tencent flag is set: `GetTencentClientSecurity` and `0x86cad0`. |
| 10 | 707 `G_EnterArenaFail` | ARENAD | `0xef3250` | Fan-out to `World::ProcessNetObjectEvent`. |

Note on step 8: the `Def` layout gives `arenaOtpKey` i64 @16, `worldId` i16 @24, `worldPcId` i64 @32 (struct offsets with the 16-byte in-memory header). The code writes `+24` (word) and `+32`/`+36` (dwords) in the lines that we read; the `+16` write is before them ❓.

`G_EnterArenaOk` 706 and `G_ReturnToGameOk` 649 use the same 1304-byte layout: `creatureId`, `chattingIp/Port/ServerId`, `debugPort`, `gamePacketRight`, `creatureInfo` (PROP_SYNC), `publisherCreatureInfo` vbin @1256, `key` vbin @1264, `recycleTime`, `clientOptionList`, `rule_client_option_1..4`, `pc_cafe_publisher_code`. Full rows: [`packet-layouts.tsv`](packet-layouts.tsv).

### 2.3 Return to game, lobby leave, reconnect

| Command | Dest | 381 function | Behaviour |
| --- | --- | --- | --- |
| 645 `G_LeaveArenaLobbyReq` (16 B, no body) | GAMED | `ArenaCommonLobby::LeaveLobby` `0x1278830` | Leaves the lobby. |
| 646 / 647 `LeaveArenaLobbyOk/Fail` | GAMED | `0xef3c50` → `World::LeaveArenaLobby` `0x7321c0` / `0xef3c10` | |
| 648 `G_ReturnToGameReq` (16 B, no body) | GAMED | `ArenaCommonLobby::SendLeaveArenaLobby` `0x1278c60` | |
| 649 `G_ReturnToGameOk` (1304 B) | GAMED | `0xf701f0` → `World::ReturnToGame` `0x7412e0` | Reads `World` from `BNSClient+0x4c`. Returns 1 if the packet is null or the call succeeds. |
| 650 `G_ReturnToGameFail` | GAMED | `0xef2c50` | Fan-out. |
| 954 `G_ExistDisconnectedArenaSessionNotify` (32 B) | BOTH | `0xeea140` | Body: `expire_time` i64, `gameMode` i8. Fan-out to `World::ProcessNetObjectEvent`. |
| `EnterArenaLobbyNotify` / `LeaveArenaLobbyNotify` | GAMED | `World::EnterArenaLobbyNotify` `0x71f5d0`, `World::LeaveArenaLobbyNotify` `0x71f5a0` | Other players in the lobby. |

ARENAD commands with a 381 C2S builder (all other ARENAD rows are S2C):

| Cmd | Name (GM) | Builder(s) 381 |
| --- | --- | --- |
| 358 | `DpArenaClientToServerReq` (Tencent DP data) | `0x8b3ca0` |
| 705 | `EnterArenaReq` | `0xf2bf60` |
| 708–713 | `ReturnToArenaLobbyFromDuel` / `…FromDungeon` | `0x12781c0`, `0x12c3370`, `0x12c34b0` |
| 717–719 | `ArenaSpectateReturnToArenaLobby` | `0x1277e10` |
| 729–731 | `ArenaEventWarpToPos` | `0x11d61c0` |
| 739–744 | `ArenaEventReturnToArenaLobby(AllPc)` | `0x1277f80`, `0x11d6350` |
| 748–750 | `WarpToDuelRoundStartPcSpawn` | `0x1277d80` |
| 751–753 | `SetBattleRoyalState` | `0x11d6250`, `0x11d62d0` |
| 754–756 | `InZonePcLoc` | `0x11d7380` |
| 757–759 | `BattleRoyalScore` | `0x11d6060` |

357 `G_DpArenaServerToClient` (S2C) has the handler `0xefb900`.

Arena objects (381):

| Object | 381 address | Note |
| --- | --- | --- |
| `ArenaWorld` ctor | `0x123a180` | vtable functions `0x1235e20`–`0x1236070` |
| `ArenaLobby` ctor | `0x123ade0` | functions `0x12361d0`–`0x12a89e0`; `SendFindUserGM` `0x12740f0`, `SendMoveToArenaGM` `0x1274180` (GM tools) |
| `Ui::ExchangeArenaLobbyToArenaWorld` | `0x83d3c0` | UI switch lobby → world |
| `Ui::ExchangeArenaWorldToArenaLobby` | `0x843610` | UI switch world → lobby |
| `BNSClient::LoadBattleRoyalExtraMode` | `0x1129ef0` | |
| `BNSClient::LoadBattleRoyalPlayerMode` | `0x112a180` | |
| `BNSClient::IsBattleRoyalObserverMode` | `0x1131390` | |

State enums (419 GM, full lists in [`12-infogate-arena-enums-gm.tsv`](12-infogate-arena-enums-gm.tsv)):

| Enum | Values |
| --- | --- |
| `ArenaSessionState` | 0 INVALID, 1 IN_WORLD, 2 IN_ARENA_LOBBY, 3 IN_ARENA |
| `EArenaGameType` | 1 DUEL, 2 DUNGEON, 3 PARTY_BATTLE, 4 PUBLIC_RAID_1, 5 PUBLIC_RAID_2, 6 PUBLIC_RAID_2_PARTY_MODE, 7 BATTLE_ROYAL, 8 PUBLIC_RAID_3, 9 PUBLIC_RAID_4 (0 and 10: see TSV) |
| `EArenaType` | 0–3 (see TSV) |
| `EnterArenaCheckErr` | 1–10 (see TSV) |
| `ArenaLobbyLeaveReason` | 0–3 (see TSV) |

Disconnect reason 12 (`DISCONNECTED_ARENA`) and close reason 100 come from `0xf1ec60`. ArenaD reconnect uses the `Session` reconnect logic of [02](02-transport.md) ❓ (we did not trace a separate ArenaD timer).

### 2.4 Timers

| Timer | Value | Source |
| --- | --- | --- |
| InfoGate reconnect | 30 s start, doubled, max 300 s, 10 tries | §1.5 |
| ArenaD connect | uses `0x5d2680` (same connect as InfoGate) ❓ time-out not read | `0x1137560` |
| Disconnected arena session | `expire_time` from packet 954 | `0xeea140` |

---

## 3. Other servers

### 3.1 In-game web URLs (`InGameWeb`)

- `InGameWeb::GetDomainUrl` `0xe31ae0` takes an index 1–75 (`InGameWeb::WebContents`, 419 GM, 77 names in the enums TSV). It returns a copy of a `std::wstring` from a table of 0x68-byte entries. Table entry 1 is at `0x1a82f54 + 0x68` (inside the large global `0x1a4dbb0`). For an index out of range it returns an empty string.
- `InGameWeb::SetDomainUrl` `0xe31a80` writes one entry. Only 2 callers: `BnsTv::initGameTVSchedule` `0x436800` (URL from InfoGate `I_UpdateGameTVScheduleNotify`) and `GuideMovie::SetGuideMovieUrl` `0xedf1d0`. The loader of the other entries is not found ❓ (no direct xrefs to entries; probably config text).
- 64 functions call `GetDomainUrl`: web panels (`DigitalRiverPanel` `0x1010e70`, `BoastPanel` `0x1192890`, `BigSnsPanel` `0x11930e0`, `AuctionPanel2` `0x1193fa0`, `CharacterInfoPanel` `0x1196790`, …).
- `InGameWeb::MakeDuelRankingUri` `0xe36300` (callers `DuelLobbyRoomInfoPanel::showRankView` `0x1014930`, `DuelStatusPanel::showDuelRankView` `0xf8b1f0`) and `MakeSnsDuelRankingUri` `0xe363b0` build duel ranking URLs.
- The 381 binary has no `http://` or `https://` C-string. URLs are data, not code.

### 3.2 HTTP client (`WebRequester`, `HTTPClient`)

```
 game code (guild bank, donate, guild craft …) ─► WebRequester::RequestUri 0x7373c0 (queue)
 WebRequester::InternalThreadProc 0x73aa90 ─► processWebRequest 0x739840
      ─► HTTPClient::Request 0xea5320 ─► connect 0xea2130 (InternetOpenW, InternetConnectW)
                                     └─► 0xea2030 ─► requestGet 0xea1e30 / requestPost 0xea1440 / requestPostMultiPartsFormData 0xea19c0
                                                     (HttpOpenRequestW, HttpSendRequestW)
```

- Transport is WinINet (`HttpOpenRequestW` import `0x139a9e0`, `HttpSendRequestW` `0x139a9d4`, `InternetConnectW` `0x139a9d0`, `InternetOpenW` `0x139a9f0`).
- `processWebRequest` `0x739840`: user agent from `InGameWeb::GetAgentName`; it splits the response status line with `" \r\n\t"` and tests for `"200"`; it writes errors to `ClientLog` (source `WebRequester.cpp`); it calls `InGameWeb::IsWebEnabled`.
- Callers of `RequestUri` `0x7373c0`: `0xf44360`, `0xf45540`, `Guild::receiveDepositInGuildBankNotify` `0xf46db0`, `Guild::recevieDonateUniformMaterial` `0xf47150`, `GuildCraft::onReceivedAssentGuildCraft` `0xf4f970`. These send a web notice after a game packet ❓ (URL kind not read).
- Other `HTTPClient::Request` callers: `ClientUserLog::SendPostArguments` `0x10e8df0`, `ClientExecutionLog::SendPostArguments` `0x1167300` (URLs from `ClientRuleConfig` `_userlogUrl` @32, `_excutionlogUrl` @20 / `_excutionlogUrl2` @24, 419 GM), `WebUtil::DownloadFile` `0x738330`, `0x738760`, `0x738bf0`, `0x738f80`.
- Helpers: `BNSClient::GetWebRequester` `0x47c690`, `WebRequester` ctor `0x7897e0`, `startThread` `0x73ab10`, `UrlEncodeA/W` `0x726da0` / `0x726be0`, `UrlDecodeA/W` `0x72c3e0` / `0x72c300`, `WebUtil::OpenUrl` `0x737e10`, `GetJpegFileNameFromUrl` `0x7215d0`.

### 3.3 Other links

| Server | Client link | Address source |
| --- | --- | --- |
| STS / NP login | `NPSession::SendWebLoginRequest` `0xc220a0` → connect `0x5d2680` | chapter [03](03-sts-client.md) |
| LobbyGate | `BNSClient::EnterGameForNPWithLobbyGate` `0x1157ee0`, `0xc881d0` ❓ → `0x5d2680` | chapter 03 / 04 |
| Other `0x5d2680` callers | `0xbdd570`, `0xc73350` ❓ (role not read) | — |
| Chat server | `chattingIp/Port/ServerId` in 706, 649, 784 | [11-social](11-social.md) |
| CacheGate | no client link. No 381 function name and no C-string contains `CacheGate` (checked). A hidden link through another name is not excluded ❓ | server only, [CacheGate](../../Server/CacheGate/README.md) |
| Shop | web panels through `GetDomainUrl` (`DigitalRiverPanel` `0x1010e70`) ❓; in-game shop packets are game commands (chapter 05) | `InGameWeb` table |

---

## 4. Function list

[`12-infogate-arena-functions.tsv`](12-infogate-arena-functions.tsv) has 1,859 rows. Columns: `addr_381`, `name_381` (demangled), `class`, `topic` (`infogate` 203, `arena` 1,418, `web` 238), `size` (bytes).

- Selection: a 381 function name matches `InfoGate|Arena|BattleField|BattleRoyal|Ranking|Tournament|^I_|CacheGate|WebRequest|WebBrowser|Http|Url|URL|InGameWeb|HTTPClient`, and does not start with `std::`.
- The names come from the 381 database. Some are false PDB matches (see the note at the top). The unnamed functions of this chapter (`sub_*`, for example `0x1137560`, `0xf1ec10`, `0xf1ec60`, `0xf70220`) are only in the tables of §1–§3 and in the `handler_381` / `sender_381` columns of the command TSVs.
- Topic is a name rule: `InfoGate`, `I_`, `Ranking` → `infogate`; `Arena`, `BattleRoyal`, `BattleField`, `Tournament` → `arena`; the rest → `web`.

---

## 5. Open items (❓)

1. Senders of InfoGate commands 7, 49 and 55 are not found (§1.4).
2. Receivers through `BNSClient+0x234` / `+0x23c` are not traced (§1.6).
3. The exact field meaning in `CanReconnect` `0x5d5450` (§1.5).
4. The Tencent override of the InfoGate address string (§1.2).
5. The loader of the `InGameWeb` URL table at `0x1a82f54` (§3.1).
6. The write of `arenaOtpKey` @16 in `0xf2bf60` (§2.2, step 8).
7. ArenaD connect time-out and reconnect timer (§2.4).
8. Roles of the `0x5d2680` callers `0xbdd570`, `0xc73350`, `0xc881d0` (§3.3).
9. URL kind of the `WebRequester::RequestUri` guild callers (§3.2).

---
