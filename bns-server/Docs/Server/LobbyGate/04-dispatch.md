# 04 — Command dispatch and relay

Target build: LobbyGate `0.0.381.572` (2020). Reference build: `0.0.348.527` (2019, with PDB).
Status: **draft 1** — routing rules, tables and both command maps verified ✅ in 348 and 381. Handler bodies are in 05-handlers (not written yet).

`cmd` is the `i16` at packet offset 2 (spec 03 §2).

## 1. Overview

```
            client link                                   LobbyDaemon link
 client ──▶ ClientSessionHandler::PacketHandler           LobbyDSessionHandler::PacketHandler ◀── LobbyD
              │                                              │
              ├─ AnyAuthed table  (cmd ≤ 109)               ├─ cmd in bypass set (54 cmds)
              ├─ not authed → NotAuthed table (≤ 110)       │     → relay to the client (§5)
              └─ authed:                                    └─ else → LobbyD table (+ fragment table)
                   cmd 9  → ping, nothing else                     → own handler (05-handlers)
                   cmd 24, 103 → own handler
                   other cmd in Authed table → relay to LobbyD (§4)
```

## 2. Client link (`ClientSessionHandler::PacketHandler`)

| Step | Rule | 348 | 381 (`0x140006AC0`) |
| --- | --- | --- | --- |
| 1 | `cmd ≤ 109` (`0x6D`) and AnyAuthed entry set → call it, stop | ✅ | ✅ |
| 2 | session has no authed channel (no channel, closing, or channel state ≠ 1) → NotAuthed table if `cmd ≤ 110` and entry set; else log `%I64X[N/A], Unknown packet command %d`, **close** | ✅ | ✅ |
| 3 | authed, `cmd == 9` → log `ping recv ok`, return true | ✅ | ✅ |
| 4 | authed, `cmd > 110` or Authed entry not set → log `Unknown packet command`, **close** | ✅ | ✅ |
| 5 | `cmd == 24` → `GetWorldInfoListReq`; `cmd == 103` → `GetPcCafeMileageFeverTimeScheduleListReq` (inlined in 381) | ✅ | ✅ |
| 6 | other Authed entry → relay to LobbyD (§4) | ✅ | ✅ |

On this client link, a handler that returns false closes the session (spec 03 §1).

### 2.1 Client tables

| Table | 381 | 348 | Commands |
| --- | --- | --- | --- |
| AnyAuthed | `0x140075BE0` | `0x140091B00` | 9 `PingReq`, 106 `ContentsRuleListReq` |
| NotAuthed | `0x140075870` | `0x140091E70` | 1 `HelloReq`, 3 `LoginWithNPTokenReq`, 6 `LoginWithLobbyTokenReq` |
| Authed | `0x140075500` | `0x1400921E0` | 24 `GetWorldInfoListReq`, 103 `GetPcCafeMileageFeverTimeScheduleListReq`, and 28 relay commands (§4) |

Relay entries in the Authed table point to a stub that returns true (348 folds it with `HandleLobbyLobbyGate_GetGoodsCountFail`). The stub only marks the command as allowed. It is never called.

Note: cmd 9 is in AnyAuthed, so step 1 always takes it. Step 3 can only run if the AnyAuthed entry is missing, which does not happen.

## 3. LobbyDaemon link (`LobbyDSessionHandler::PacketHandler`)

| Step | Rule | 348 | 381 (`0x1400090C0`) |
| --- | --- | --- | --- |
| 1 | no table, or no entry for `cmd` → log `handle fail, no such cmd, cmd:%d`, close | ✅ | ✅ |
| 2 | `cmd` in bypass set → relay to the client (§5) | ✅ | ✅ |
| 3 | else → `TFPP_PumpPacket`: when a fragment handler exists for `cmd`, join the fragments first (spec 03 §7), then call the handler | ✅ | ✅ |

The handler runs on a worker thread through the IOCP queue (`ThreadFreePacketPump`). Its `false` result is ignored, so it does not close the link (05 §6.1). Rust: a per-session task is enough. The order of packets from one session must stay the same.

### 3.1 LobbyD tables (381 main `0x1400A4050`, fragment `0x1400A43D0`; identical in 348)

| cmd | Handler | Fragment |
| --- | --- | --- |
| 2 | `HelloOk` | ✅ |
| 3 | `HelloFail` | |
| 4 | `UpdateWorldInfoListNotify` | ✅ |
| 5 | `AnnounceNotify` | |
| 7 / 8 | `InitSessionWithNPTokenOk` / `Fail` | |
| 10 / 11 | `StartSessionWithNPTokenOk` / `Fail` | ✅ (10) |
| 13 / 14 | `InitSessionWithLobbyTokenOk` / `Fail` | |
| 16 / 17 | `StartSessionWithLobbyTokenOk` / `Fail` | ✅ (16) |
| 20 | `CloseAnonymousSessionNotify` | |
| 21 | `CloseSessionNotify` | |
| 22 | `KickNotify` | |
| 25 | `GetEnterWorldTokenOk` | |
| 36 | `GetCharacterInfoListOk` | ✅ |
| 68 | `ListGoodsOk` | ✅ |
| 71 | `TakeGoodsOk` | ✅ |
| 74 | `ListLobbyServiceItemInfoOk` | ✅ |
| 80 | `GetServiceEntryListOk` | ✅ |
| bypass set | stub (relay, §5) | |

Bypass set (54 cmds, `isBypassPacket`, identical in 381): 26 27 29 30 32 33 34 37 39 40 42 43 45 46 48 49 50 52 53 55 56 58 59 61 62 63 65 66 69 72 75 77 78 81 83 84 86 87 89 90 92 93 95 96 98 99 101 102 104 105 107 108 109 110.

## 4. Relay client → LobbyD (`HandleLobbyGate_BypassPacket`)

```
client packet:  [i16 size][i16 cmd_c][body …]
LobbyD packet:  [i16 size][i16 cmd_d][u64 cliKey][body …]      ("hq" + body copied unchanged)
```

- `cmd_d = map_c2d(low byte of cmd_c)` (table below). Not in the table → `cmd_d = 0`.
- `cliKey` = the client's channel id. `cliKey == 0` → log `invalid clikey`, close the client.
- LobbyD session missing or not connected → log `failed to send, not connected`, **close the client**.

| client → LobbyD | | | | | | |
| --- | --- | --- | --- | --- | --- | --- |
| 12→23 | 13→24 | 17→28 | 20→31 | 27→35 | 30→38 | 36→44 |
| 39→47 | 41→41 | 43→51 | 46→54 | 49→57 | 52→60 | 56→64 |
| 59→67 | 62→70 | 65→73 | 68→76 | 71→79 | 74→82 | 77→85 |
| 80→88 | 83→91 | 86→94 | 89→97 | 92→100 | 95→103 | 98→106 |

381: the same 28 pairs, inlined in `0x140015B00` ✅.

Quirks to keep:
- Client cmd **33** is in the Authed table but **not** in the map, so it goes to LobbyD as cmd **0**.
- Map entry 41→41 is unreachable (41 is not in the Authed table).

## 5. Relay LobbyD → client (`HandleLobbyLobbyGate_BypassPacket`)

```
LobbyD packet:  [i16 size][i16 cmd_d][u64 cliKey][body …]
client packet:  [i16 size][i16 cmd_c][body …]                  ("h" + body copied unchanged)
```

- Decompose `"hq"`. Failure → log `decompose fail`, return true, drop the packet. No link closes (05 §8.1).
- Find the client by `cliKey` (`ClientManager::AcquireClient`). Not found → log `failed to find cliKey`, return true, drop the packet. No link closes (05 §8.1).
- `cmd_c = map_d2c(low byte of cmd_d)`. Not in the map → 0.
- The packet is encrypted with the client's session crypt (one frame).

| LobbyD → client | | | | | | |
| --- | --- | --- | --- | --- | --- | --- |
| 26→15 | 27→16 | 29→18 | 30→19 | 32→21 | 33→22 | 34→23 |
| 37→29 | 39→31 | 40→32 | 45→37 | 46→38 | 48→40 | 49→41 |
| 50→42 | 52→44 | 53→45 | 55→47 | 56→48 | 58→50 | 59→51 |
| 61→53 | 62→54 | 63→55 | 65→57 | 66→58 | 69→61 | 72→64 |
| 75→67 | 77→69 | 78→70 | 81→73 | 83→75 | 84→76 | 86→78 |
| 87→79 | 89→81 | 90→82 | 92→84 | 93→85 | 95→87 | 96→88 |
| 98→90 | 99→91 | 101→93 | 102→94 | 104→96 | 105→97 | 107→99 |
| 108→100 | 109→101 | 110→102 | | | | |

52 pairs, identical in 381 (`0x140009390`, 52 `case` labels, main-session check) ✅. Quirk: bypass commands **42** and **43** are not in the map, so they reach the client as cmd **0**.

## 6. Rules for Rust

- Use the low byte of `cmd` for both maps. This only matters for values > 127, and those never pass the table checks.
- Relay bodies byte for byte. Do not decode them.
- Edge case, client → LobbyD only: the new packet has a 12-byte head, so 3,988 bytes are free. If the body is ≥ 3,988 bytes (client packet size ≥ 3,992), the C++ starts a second buffer and the output becomes two packets. The second one has no `cmd` ❓. Rust: log it and close the client. Check this choice against Wine traffic in Phase 4. (LobbyD → client cannot hit this: the body is at most 3,988 bytes and 3,996 are free.)

## 7. Open items

1. Confirm that the client channel state has no value other than 0 and 1 (05 §8.2).
2. Edge case in §6 (oversized relay body).
