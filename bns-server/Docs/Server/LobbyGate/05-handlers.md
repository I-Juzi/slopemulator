# 05 — Handlers

Target build: LobbyGate `0.0.381.572` (2020). Reference build: `0.0.348.527` (2019, with PDB).
Status: **draft 6** — §1 login flow, §2 LobbyD link start, §3 broadcast crypt, §4 world list and PC-cafe schedule, §5 contents rules, §6 close and kick notifies verified ✅ in 348 and 381. §7 and §8 drafted and reviewed by Sol 6.1; §7 in both databases; the main session did not re-read it. Other handlers: not written yet.

Notation: `"hqE" {a, b}` = `Packet::Compose`/`Decompose` format (spec 03 §5) and the field values after `cmd`. "C" = client link. "D" = LobbyDaemon link. `gateKey` and `cliKey` are u64.

## 1. Login flow

There are two login paths. Both use the same three steps: **Login** (client) → **InitSession** (LobbyD) → **StartSession** (LobbyD).

| Path | Client request | Client reply cmd (Ok / Fail) |
| --- | --- | --- |
| NP token (first login, from the launcher) | cmd 3 `LoginWithNPTokenReq` | 4 / 5 |
| Lobby token (re-login, e.g. back from a world) | cmd 6 `LoginWithLobbyTokenReq` | 7 / 8 |

### 1.1 Sequence (NP token path; the lobby-token path has the same shape)

```
client                      LobbyGate                                        LobbyD
  │ C3 "hoo"{npTok,portal}    │                                                │
  │──────────────────────────▶│ gateKey = MakeGateKey(session)                 │
  │                           │ D6 "hqooi"{gateKey,npTok,portal,ip}            │
  │                           │───────────────────────────────────────────────▶│
  │                           │            D7 "hqqoo"{gateKey,cliKey,npTok,portal}
  │                           │◀───────────────────────────────────────────────│
  │                           │ AddClient(session, gateKey, cliKey)            │
  │                           │ D9 "hqoo"{cliKey,npTok,portal}                 │
  │                           │───────────────────────────────────────────────▶│
  │                           │            D10 fragment "hiqQoih C…; C…; Qqiqiq"
  │                           │◀───────────────────────────────────────────────│
  │                           │ client.state = 1 (authed)                      │
  │ C4 fragment (D10 fields + K broadcast key)                                 │
  │◀──────────────────────────│ client.broadcast = true                        │
  │ C109 "hi"{reloadCount}    │ (only when the condition in §1.6 is true)      │
  │◀──────────────────────────│                                                │
```

### 1.2 Client requests (NotAuthed table, spec 04 §2)

| cmd | Name | Format | Then |
| --- | --- | --- | --- |
| 3 | `LoginWithNPTokenReq` | `"hoo"` {npToken o, portalAccountId o} | `MakeGateKey`; D6 `InitSessionWithNPTokenReq` `"hqooi"` {gateKey, npToken, portalAccountId, ip} |
| 6 | `LoginWithLobbyTokenReq` | `"hqq"` {cliKey, lobbyToken q} | `MakeGateKey`; D12 `InitSessionWithLobbyTokenReq` `"hqqqi"` {gateKey, cliKey, lobbyToken, ip} |

- `ip` is the raw IPv4 `in_addr` value from `Connection::_ip`. Its memory bytes, and the four bytes in the LobbyD packet, are the network-order IPv4 octets. Rust uses `u32::from_le_bytes(ip.octets())` for the little-endian `i` writer. Verified in 381: NP compose argument write `0x1400099E2`, compose call `0x140009A16`; lobby-token argument write `0x14000A211`, compose call `0x14000A232`. Neither path swaps the value. Both use the same value for `Network::GetAddressStringW` (`0x140021AB0`), which passes it directly to `inet_ntoa`.
- Decompose fails → return false → **close** the client.
- `MakeGateKey` returns 0 → C `"hh"` {5 or 8, **2**}, return true (the socket stays open).
- D send fails → C `"hh"` {5 or 8, **2}**, return true. ❓ the gate key stays in the map (no `RemoveGateKey` on this path).
- Nothing stops a second login request before the first one ends. Each request makes a new gate key.

### 1.3 LobbyD replies

All LobbyD handlers first check the LobbyD session. If it has no channel or is closing → log `== CRASH == … not authed session` and return false. A decompose failure also returns false. The normal queued path ignores this result and keeps the LobbyD link open (§6.1).

| D cmd | Name | Format | Action | Client result |
| --- | --- | --- | --- | --- |
| 7 | `InitSessionWithNPTokenOk` | `"hqqoo"` {gateKey, cliKey, npToken, portal} | find gate key entry (missing entry or missing session → log, stop); `AddClient(session, gateKey, cliKey)`; `entry.cliKey = cliKey`; D9 `StartSessionWithNPTokenReq` `"hqoo"` {cliKey, npToken, portal} | AddClient fails or send fails → C `"hh"` {5, 2} |
| 8 | `InitSessionWithNPTokenFail` | `"hqE"` {gateKey, reason} | `RemoveGateKey(gateKey)` | C `"hh"` {5, reason} |
| 10 | `StartSessionWithNPTokenOk` | fragment, §1.4 | §1.4 | C4 fragment |
| 11 | `StartSessionWithNPTokenFail` | `"hqE"` {cliKey, reason} | gateKey = client's gate key (0 if no client); `RemoveClient(cliKey)`; `RemoveGateKey(gateKey)` | C `"hh"` {5, **map_np(reason)**} |
| 13 | `InitSessionWithLobbyTokenOk` | `"hqq"` {gateKey, cliKey} | as cmd 7; then D15 `StartSessionWithLobbyTokenReq` `"hq"` {cliKey} | failures → C `"hh"` {8, 2} |
| 14 | `InitSessionWithLobbyTokenFail` | `"hqE"` {gateKey, reason} | `RemoveGateKey(gateKey)` | C `"hh"` {8, **map_lt(reason)**} |
| 16 | `StartSessionWithLobbyTokenOk` | fragment, §1.5 | §1.5 | C7 fragment |
| 17 | `StartSessionWithLobbyTokenFail` | `"hqE"` {cliKey, reason} | as cmd 11 | C `"hh"` {8, reason} |

- The client reply goes to `entry.session` of the removed gate key entry. No entry (or no session) → no reply.
- All the handlers in this table return true after the checks above. The client socket stays open after a Fail reply. The client closes it, or the session lifecycle does (later section).
- The D9 / D15 compose call also gets the IP, but the format has no field for it. The IP is **not** sent.

Fail-reason maps (identical in 348 and 381 ✅). Other values pass unchanged.

| map | LobbyD reason → client reason |
| --- | --- |
| `map_np` (D11 only) | 15→10, 16→11, 17→12, 18→13 |
| `map_lt` (D14 only) | 19→14, 20→15, 21→16 |

Quirk to keep: D8 and D17 pass the reason **without** a map.

### 1.4 D10 `StartSessionWithNPTokenOk` → C4 `LoginWithNPTokenOk`

Input (fragment, D10): `"hiqQoihCiiiii;CiiQiqq;Qqiqiq"`.
Output (fragment, C4): `"hiqQoihCiiiii;CiiQiqq;QqiqiqK"` — the **same fields in the same order**, plus `K` at the end.

| # | Code | Field (348 names) |
| --- | --- | --- |
| 1 | `q` | cliKey |
| 2 | `Q` | createInitialUserName (0/1; output is written as `value == 1`) |
| 3 | `o` | infoGateToken |
| 4 | `i` | infoGatePubAddr |
| 5 | `h` | infoGatePubPort |
| 6 | `C iiiii ;` | repeat: 5 × i32 per item (20 bytes) ❓ meaning |
| 7 | `C iiQiqq ;` | repeat: `GradeInfo` {gameGradeKey i, gradeId i, activation Q, gradeCount i, totalEffectiveFrom q, totalEffectiveTo q} |
| 8 | `Q` | teen (output is written as `teen != 0`) |
| 9 | `q` | closestEndTime |
| 10 | `i` | remainJumpingCharacterCreationQuota |
| 11 | `q` | accountExpInfo.exp |
| 12 | `i` | accountExpInfo.expQuotaPerDay |
| 13 | `q` | accountExpInfo.lastUpdateTime |
| 14 | `K` | broadcast crypt key (`LobbyGate::_broadcastCrypt.key`, `size` bytes) — output only |

Steps:
1. Decode all fields. A failure → log, stop (return true; nothing sent).
2. `AcquireClient(cliKey)`. Missing → stop.
3. `client.state = 1` (authed; spec 04 §2 step 2).
4. Compose C4 with the fragment rules (spec 03 §7). **Rust:** compose it again. Do not copy the LobbyD fragments, because `K` changes where the fragments split.
5. Send. Then `client.broadcast = true` (the client now gets broadcast packets).
6. §1.6.

### 1.5 D16 `StartSessionWithLobbyTokenOk` → C7 `LoginWithLobbyTokenOk`

Input: `"hiqCiiiii;CiiQiqq;iqiq"`. Output: `"hiqCiiiii;CiiQiqq;iqiqK"`.

Fields: 1 `q` cliKey; 6 and 7 as in §1.4; 10–13 as in §1.4; 14 `K` broadcast key. Fields 2–5, 8 and 9 are not in this path.

Steps: the same as §1.4.

### 1.6 Contents-rule reload notify (C109)

After the C4 / C7 send:

```
if session.extraData != null and 0 < extraData.lastRuleTime < ContentsRuleConfig::ConfigLastReloadTime:
    send C109 "hi" {ContentsRuleConfig::ConfigReloadCount}
    extraData.lastRuleTime = 0
```

381: the same send (`r8d = 0x6D`, value from `dword_1400A6128`) in both Ok handlers ✅. `extraData` and the two globals are explained in §5. **Result:** both globals are always 0, so C109 is never sent (§5.4).

### 1.7 381 sources

| Item | 381 |
| --- | --- |
| `"hoo"` | `0x14008EF34` ← `HandleLobbyGate_LoginWithNPTokenReq` |
| `"hqq"` | `0x140088368` ← `LoginWithLobbyTokenReq`, `InitSessionWithLobbyTokenOk` |
| `"hqooi"` / `"hqqqi"` / `"hqoo"` / `"hq"` | `0x140084280` / `0x14008460C` / `0x140084480` / `0x1400846F0` |
| `"hqqoo"` | `0x140085194` ← `InitSessionWithNPTokenOk` |
| `"hqE"` | `0x140085CC8` ← the four Fail handlers |
| D10 in / C4 out formats | `0x140085FE0` / `0x1400873F8` |
| D16 in / C7 out formats | `0x140088CF8` / `0x1400898B0` |
| reason 2 (`mov r9d, 2`; `lea r8d, [r9+3]` = 5, `[r9+6]` = 8) | `0x14000C49D`, `0x14000C689`, `0x14000EF48`, `0x14000EFD4`, `0x140015910`, `0x140015AD2` |
| `map_np` / `map_lt` | `0x14000E9C0` / `0x14000F100` |

### 1.8 Open items

1. IP byte order is resolved in §1.2 (381 compose and `inet_ntoa` evidence).
2. Gate key cleanup when the D6 / D12 send fails, and gate key timeout (if any).
3. Meaning of the `C iiiii` items (the 2019 code has no field names for them).

## 2. LobbyDaemon link start

LobbyGate **connects to** LobbyD (`lobby-daemon-addr`, `lobby-daemon-port`). LobbyD never connects to LobbyGate: `LobbyDSessionHandler::OnAccept` logs `== CRASH == … accept, invalid call` and returns false.

### 2.1 Start order (`LobbyGate::Initialize`)

| Step | Action | On failure |
| --- | --- | --- |
| 1 | `Network::Initialize(workerThreadCount)` | log `init network fail`; start fails |
| 2 | `BroadcastCrypt::Initialize` (§3) | log `init broadcast crypt fail`; start fails |
| 3 | `LobbyD::Hello`: make the LobbyD session, then **wait without a time limit** for HelloOk or HelloFail | log `init lobbyd channel fail`; start fails |
| 4 | open the client port (`client-port`, `max-client-session`) | log `Cannot open Client port, %d`; start fails |

So the client port opens only after LobbyD accepts the Hello. Rust: keep this order. Scheduled reconnect retries use 5,000 ms, and the first reconnect can be immediate (§8.4).

Rust safety policy (not a fact about 381): the trusted receive path requires a command, so frame size is 4..=4000. Joined Hello/login messages have a 1 MiB limit. A full 64-entry inbound queue causes a reconnect, not an unbounded allocation. Outgoing requests carry a connection generation; a request from a closed connection fails instead of being sent after reconnect.

### 2.2 Hello exchange

```
LobbyGate                                         LobbyD
   │ (connect or reconnect)                          │
   │ D1 HelloReq "hhhh" {predefinedWorldId, 58, internalPort}
   │────────────────────────────────────────────────▶│
   │ D2 HelloOk (fragment)          or  D3 HelloFail "hh" {reason}
   │◀────────────────────────────────────────────────│
```

- D1 is sent on **every** connect and reconnect (`OnConnect` → `hello`). The values come from config (`predefined-world-id`, `internal-port`; spec 01). `58` is the LobbyD↔LobbyGate protocol version (the same number as the client `HelloReq` version). 381: `0x140008EC0`, inlined in `OnConnect`; same config offsets `+0x3C` / `+0x44` ✅.
- **HelloOk (D2):** LobbyGate sets the LobbyD state = 1 ("enable service") and wakes the start thread **before** it decodes the body. Then it decodes:

```
"hiC hhsQQiihhiQQQQQQQ ; QQq C qqiQiic ;"
     └ world list ─────┘   │  └ fever-time schedule ┘
                           enablePcCafeMileage, enablePcCafeMileageDeposit, seasonExpireTime
```

| World field | Code | | Schedule field | Code |
| --- | --- | --- | --- | --- |
| worldId | h | | startTime | q |
| worldClusterId | h | | endTime | q |
| worldName (≤ 16 UTF-16 units) | s | | feverPoints | i |
| worldType | Q | | repeatType | Q |
| playType | Q | | repeatStartTime | i |
| publicAddress | i | | repeatDuration | i |
| internalAddress | i | | daysOfWeek | c |
| publicPort | h | | | |
| mobilePort | h | | | |
| worldScore | i | | | |
| worldLive, entrance, characterCreation, characterCreationLimitedOnNewAccount, recommend, brandnew, crowdness | 7 × Q | | | |

  - Decode fails → log, return false. Normal path: the packet is dropped and the LobbyD link stays open (§6.1). HelloOk is a fragment command; the same rule applies (§7.4).
  - Then `WorldListManager::SetWorldList` and `SetPcCafeMileageFeverTimeScheduleList` store the lists. These build the cached client packets (§3.3). A store failure is only logged. Return true.
- **HelloFail (D3):** log `hello fail, reason:%d`, wake the start thread, return true. The state stays 0, so step 3 fails: `service not available` → start fails.
- **OnClose:** LobbyD state = 0 (`disable service`); the session layer reconnects (§8.4). Connected clients stay open. A client relay fails only when the LobbyD transport is unavailable, and that closes that client (spec 04 §4).

Rust event policy: pair each queued D2 event with its own complete Hello body. Apply that snapshot in event order, including after reconnect. Do not read the latest global Hello body for an older event. The pending snapshot queue shares the 64-event bound, plus one body during enqueue. This preserves the Hello/D4 update order. Source: Hello handler `0x14000AB70`; D4 handler `0x14000B679`. This is a Rust transport policy, not a claim about the original worker scheduling.

381: HelloOk format string at `0x140084998` (used at `0x14000AC67`) ✅. `"hh"` HelloFail ✅ (spec 04 xref list).

## 3. Broadcast crypt

One server-wide AES key for packets that LobbyGate builds **once** and sends to many clients. The client learns the key from the login reply (`K`, §1.4 field 14).

### 3.1 Key (`BroadcastCrypt::Initialize`, `BroadcastPacketCrypt::GenerateKey`)

| Item | Value | 348 | 381 |
| --- | --- | --- | --- |
| Cipher | AES-128 (same `Crypto` type 2 as the session crypt) | ✅ | ✅ |
| Mode | `PARAM_BROADCAST_GLOBAL` (3) | ✅ | ✅ |
| Key | 16 bytes from MSVC `rand()` (low byte of each call), made once at start | ✅ | ✅ `0x14005F6C0` |
| `K` blob sent to the client | **17 bytes** = key[16] + compress-type byte **1** | ✅ | ✅ (`LOBYTE = 1`, `0x1400334E0`) |

The blob buffer is 64 bytes. `GenerateKey` writes 16 bytes and then appends the compress-type byte. **Rust:** use a CSPRNG for the 16 bytes. The value is random, so wire compatibility stays.

### 3.2 Encrypt (`BroadcastPacketCrypt::Encrypt`)

Frame header `hdr = (code << 12) | ((cipher_len >> 2) & 0xFFF)`, the same layout as spec 03 §4.

| Param | Code | Body |
| --- | --- | --- |
| `BROADCAST_LOCAL` (2) | `0xC` | AES only |
| `BROADCAST_GLOBAL` (3) — **the one LobbyGate uses** | `0xE` | AES only |
| `BROADCAST_LOCAL_COMPRESS` (4) | `0xD` (or `0xC` when compression fails or does not help) | compress (type 1 = zlib `compress2`, type 2 = zstd), then AES |

So LobbyGate sends broadcast frames with code **0xE**: AES-128-ECB with the broadcast key, zero padding, no compression. `Decrypt` always fails (error 7): LobbyGate never receives these frames. Codes `0xC`/`0xD` are not used by LobbyGate.

### 3.3 Packets that use the broadcast crypt

| Packet | Built by | When it is sent |
| --- | --- | --- |
| C11 `AnnounceNotify` `"hs"` {message} | D5 `AnnounceNotify` `"hs"` → `ClientManager::BroadcastAnnounceNotify` | at once, to every client with `broadcast == true` (set after the login reply, §1.4 step 5) |
| world list (fragment) | `WorldList::Set` (HelloOk, D4 `UpdateWorldInfoListNotify`) | cached; sent on client cmd 24 (later section) |
| PC-cafe fever-time schedule list (fragment) | `PcCafeMileageFeverTimeScheduleList::Set` (HelloOk) | cached; sent on client cmd 103 (later section) |

Each fragment is encrypted once, on its own (spec 03 §7). Rust: encrypt the cached frames once per list change and keep the bytes.

**Rust queue policy (not a verified 381 queue policy):** Each client has a bounded queue of 32 deliveries. One delivery holds one complete announcement or reply fragment list. Broadcast uses a nonblocking send. If the queue is full, Rust drops the newest announcement for that client. Rust keeps queued deliveries in order. Other clients continue. Rust does not close the slow client for this condition. Source for announcement eligibility: §3.3; the original queue capacity and overload behavior remain ❓ TODO(spec 05 §3.3).

TCP test coverage: `login::tests::stalled_tcp_receiver_keeps_fifo_drops_newest_and_recovers` uses real loopback sockets with small socket buffers. It checks a stable full queue, FIFO delivery, newest-drop behavior, fast-peer delivery, and recovery. It exercises `LoginHub` and the wire writer, not the full Server/fake-daemon path. This does not establish the original queue policy or a source address for it.

## 4. World list and PC-cafe fever-time schedule

LobbyGate keeps one cached client packet for each list. LobbyD sends the data (§2.2 HelloOk, and D4). The client asks for it (C24, C103).

```
LobbyD ── D2 HelloOk ─────────────┬─▶ WorldList::Set ──────────▶ cached C25 (fragment, broadcast crypt)
       ── D4 UpdateWorldInfoList ─┘                                   │
       ── D2 HelloOk ───────────────▶ ScheduleList::Set ─────────▶ cached C104 (fragment, broadcast crypt)
                                                                      │
client ── C24 GetWorldInfoListReq ────────────────────────────────────┤ send cached frames
       ── C103 GetPcCafeMileageFeverTimeScheduleListReq ──────────────┘
```

### 4.1 D4 `UpdateWorldInfoListNotify` (fragment)

- Format `"hiChhsQQiihhiQQQQQQQ;"`: only the world list part of HelloOk (§2.2 table).
- Decode → `WorldListManager::SetWorldList`. The new cached packet replaces the old one under a lock.
- Decode or store failure → log; the old list stays ❓ (return value not checked).
- LobbyGate does **not** push the new list to clients. A client gets it on its next C24.

### 4.2 Cached client packets

| Packet | cmd | Format | Fields |
| --- | --- | --- | --- |
| `GetWorldInfoListOk` | **25** | `"hiChhsQQihiQQQQQQQ;"` | repeat { worldId h, worldClusterId h, worldName s, worldType Q, playType Q, publicAddress i, publicPort h, worldScore i, worldLive Q, entrance Q, characterCreation Q, characterCreationLimitedOnNewAccount Q, recommend Q, brandnew Q, crowdness Q } |
| `GetPcCafeMileageFeverTimeScheduleListOk` | **104** | `"hiQQqCqqiQiic;"` | enablePcCafeMileage Q, enablePcCafeMileageDeposit Q, seasonExpireTime q, repeat { startTime q, endTime q, feverPoints i, repeatType Q, repeatStartTime i, repeatDuration i, daysOfWeek c } |

- The client world entry has **no** `internalAddress` and **no** `mobilePort`. LobbyGate drops them. Keep this.
- `enable…` flags are written as `value == 1`.
- An empty list is valid: count = 0 (spec 03 §6).
- Both packets are fragment packets (spec 03 §7). Each fragment is encrypted **once** with the broadcast crypt (code `0xE`, §3) when the list is stored. The same bytes go to every client.

### 4.3 Client requests (Authed table)

| Step | C24 `GetWorldInfoListReq` | C103 `GetPcCafeMileageFeverTimeScheduleListReq` |
| --- | --- | --- |
| cliKey = 0 | log `invalid clikey`; return false (close) | same |
| LobbyD missing, or LobbyD state ≠ 1 | C `"hh"` {**19**, **7**}; return true | C `"hh"` {**105**, **7**}; return true |
| no cached list | C `"hh"` {19, **2**}; return true | C `"hh"` {105, **2**}; return true |
| else | send the cached C25 frames | send the cached C104 frames |

The body of the request is not read. cmd 19 = `GetWorldInfoListFail`, cmd 105 = `GetPcCafeMileageFeverTimeScheduleListFail` (names from the log strings ❓).

### 4.4 381 sources

| Item | 381 |
| --- | --- |
| C24 / C103 handlers | inlined in `ClientSessionHandler::PacketHandler` `0x140006AC0`: reason 7 at `0x140006C53` / `0x140006DAB`, reason 2 at `0x140006CE0` / `0x140006E4C`, cmd `0x13` / `0x69` ✅ |
| C25 format, cmd `0x19` | `0x140081808`, used in `SetWorldList` `0x140017506` ✅ |
| C104 format, cmd `0x68` | `0x140090198`, used in `HelloOk` `0x14000B2F8` (inlined) ✅ |
| D4 format | `0x140084F68`, used in `UpdateWorldInfoListNotify` `0x14000B679` ✅ |

348: `WorldList::Set` `0x140016280`, `PcCafeMileageFeverTimeScheduleList::Set` `0x140015E80`, client handlers `0x140014F40` / `0x1400150F0` (reasons 7 and 2 read from the disassembly; Hex-Rays drops the 7).

## 5. Contents rules (client cmd 106 → reply 107 / 108)

The server sends the contents-rule config (spec 01, `ContentsRule.lobbygate.config.xml`) to the client as XML text lines. The client can ask before login: cmd 106 is in the AnyAuthed table (spec 04 §2.1).

### 5.1 Per-session state (`session.extraData`)

| Item | Value | Source |
| --- | --- | --- |
| Type | one `i64`: `lastListTick` = ms since daemon start of the last cmd 107 send | 348 `ClientSessionHandler::OnAccept` `0x140006820` |
| Start value | `0` (set in `OnAccept`) | ✅ 348 |
| Freed | `OnClose` | ✅ 348 |
| Clock | `GetTickCount64() − Daemon.StartSystemTick64` (ms, monotonic) | ✅ 348, 381 |

### 5.2 C106 `ContentsRuleListReq` (no body is read)

```
now  = ms since daemon start
last = session.lastListTick                      (0 if no extraData)
if last <= 0 or last <= ConfigLastReloadTime or now <= 30000 or now - last >= 30000:
    if ListPacket exists: send a copy of ListPacket (session crypt, one frame per fragment)
    session.lastListTick = now
else:
    send C108 "hE" {108, 234}                    (ContentsRuleListFail, "too soon")
return true                                      (the session stays open in both cases)
```

| Rule | Value | 348 | 381 (`0x140015040`) |
| --- | --- | --- | --- |
| Rate limit | one list per **30,000 ms** per session (`0x7530`) | ✅ | ✅ |
| Grace period | no limit in the first 30,000 ms after daemon start | ✅ | ✅ |
| Fail packet | `"hE"` {108, **234**} (`r9d = 0xEA`, `r8d = r9 − 0x7E`; Hex-Rays drops the 234) | ✅ `0x140014365` | ✅ `0x1400150BD` |
| Crypt | the session crypt, **not** the broadcast crypt | ✅ | ✅ |
| `ListPacket` missing | nothing is sent, but `lastListTick` is updated and the log says "sent" | ✅ | ✅ |

Rust: the list is built at start (§5.3), so it is never missing. Keep the 30 s rule and the 30 s grace period.

### 5.3 C107 `ContentsRuleListOk` (fragment message, `ContentsRuleConfig::MakeListPacket`)

Built **once** in `LoadContentsRuleConfig` and cached (`ListPacket`). Format `"hihhiCb;"`:

| Field | Code | 348 | 381 |
| --- | --- | --- | --- |
| cmd | `h` | 107 | 107 (`ebp = 0x6B`) |
| fragment mark | `i` | spec 03 §7 | same |
| major version | `h` | **65** | **74** (`r8d = r15 + 0x4A`) |
| minor version | `h` | 0 | 0 |
| reload count | `i` | `ConfigReloadCount` (always 0, §5.4) | same (`dword_1400A6128`) |
| line count | `C` | 2 + rule count = **15** | 2 + rule count = **16** |
| line | `b` | UTF-8 text, no NUL | same |

Lines, in this order (the text is UTF-16 first, then `WideCharToMultiByte(CP_UTF8)`):

| # | Text |
| --- | --- |
| 1 | `<?xml version="1.0" encoding="utf-8"?><config major-version="M" minor-version="0" module="contents-rule-config" release-module="LocalizationData" release-side="client">` (M = 65 / 74, the same value as the `h` field) |
| 2 … n+1 | one line per rule in table order: `<option name='NAME' value='VALUE'/>` (single quotes) |
| last | `</config>` |

Value text by rule type (rule table `_Def`, 381 `0x1400A4AB0`, 56-byte entries: `+0` field offset, `+8` name, `+16` type):

| Type | XML type | Value text | Rules in 381 |
| --- | --- | --- | --- |
| 1 | INT | `%d` | 1, 6–12 |
| 2 | INT64 | `%I64d` | none |
| 3 | FLOAT | `%f` (6 decimals) | none |
| 4 | BOOL | `yes` / `no` | 2–5, 13 |
| 6 | STR | `%s`; null → empty | 14 `enable-job` |
| other | — | the rule is skipped | none |

Rule order and types in 381 = the spec 01 table (14 rules, `enable-job` new). The loop bound is 13 in 348 and 14 in 381 (`cmp r15d, 0Eh` at `0x140008887`) ✅.

Fragment rules: each line is one `b` item. A line is never split. A new fragment starts when the next line does not fit (spec 03 §7).

Buffer limits: the format buffer is 260 UTF-16 units and the UTF-8 buffer is 260 bytes. ❓ A longer line (a long `enable-job` value) makes `swprintf_s` fail; the C++ result is not defined here. Rust: refuse a config whose option line is longer than 259 UTF-8 bytes, and log the reason.

### 5.4 Reload state (`ConfigReloadCount`, `ConfigLastReloadTime`)

| Fact | Evidence | Status |
| --- | --- | --- |
| Both globals are in `.data` past the raw data, so they start at 0 | 381 `0x1400A6128` / `0x1400A6150` not loaded (zero fill) | ✅ |
| No code writes either global | xrefs in 348 (`0x140093F90` / `0x140093F98`) and 381: reads only (Dump, MakeListPacket, both login Ok handlers, ContentsRuleListReq, `reload`) | ✅ |
| `LobbyGate::reload` does not reload the contents rules | it reloads BnsCommonConfig only, then logs `contents rule config reload complete(reload-count:%d), not changed` | ✅ 348 `0x140013770`; 381 caller check: `LoadContentsRuleConfig` is only called from `LobbyGate::initialize` `0x140013734` |

Consequences for Rust (wire parity):

1. The `i` reload count in C107 is always 0.
2. C109 (§1.6) is never sent: the test `0 < last < 0` is always false.
3. In §5.2 the test `last <= ConfigLastReloadTime` is the same as `last <= 0`.
4. Contents rules change only on a restart. A console `reload` must not change C107.

Rust can keep the C109 path behind a feature that is off by default. With the feature off, the wire output is the same as 381.

### 5.5 381 sources

| Item | Address |
| --- | --- |
| `HandleLobbyGate_ContentsRuleListReq` | `0x140015040` (`"hE"` `0x14008EC54`) |
| `MakeListPacket` | `0x140008470` (`"hihhiCb;"` `0x140083698`, XML head `0x1400836B0`, option formats `0x140083808` / `…850` / `…898` / `…8D8`, `</config>` `0x140083918`) |
| `LoadContentsRuleConfig` | `0x140007EB0` (`xchg ListPacket` `0x1400081CD`) |
| `ListPacket` / `ConfigReloadCount` / `ConfigLastReloadTime` | `0x1400A6140` / `0x1400A6128` / `0x1400A6150` |

348: `MakeListPacket` `0x140008800`, `ContentsRuleListReq` `0x1400142E0`, `LobbyGate::reload` `0x140013770`.

## 6. Close and kick notifies

Drafted by Sol 6.1 from the 348 pseudocode, then reviewed by Sol 6.1 on slot 2. The 381 column and the §6.1 pump were read in IDA in the main session.

### 6.1 What a `false` result means on the LobbyD link

| Path | Result of `false` | 348 | 381 |
| --- | --- | --- | --- |
| Normal: packet posted to the worker queue | Ignored. The packet is dropped and the link stays open. `HandleEvent` always returns true. | ✅ `0x140001680` | ✅ `0x140001790` |
| Queue post fails (`PostQueuedCompletionStatus`) | The handler runs at once, and its result goes to the receive path. `false` closes the link. | ✅ `0x140001980` | ✅ `0x140001990` |
| Pump event setup fails | The pump returns false, and the link closes. | ✅ | ✅ |
| Fragment commands | Same rule for the handler result. Fragment event setup failure returns true, so the link stays open (§7.4). | ✅ | ✅ |

**Rule for Rust:** on the LobbyD link, a handler failure logs and drops the packet. It does not close the link. Rust has no matching queue-post failure.

### 6.2 Common checks (D20, D21, D22)

| Condition | Result |
| --- | --- |
| LobbyD session has no channel or is closing | Log and alert `not authed session`, return false, do not decode |
| Channel exists | Decode. The channel state value is **not** tested. |
| Decode fails | Log `decompose fail`, send nothing, change nothing, return false |

### 6.3 Handlers

| Packet | Format | Action on success | Client packet | Close job reason |
| --- | --- | --- | --- | --- |
| D20 `CloseAnonymousSessionNotify` | `"hq"` {gateKey} | **Log only.** No lookup, no cleanup, no close. | none | none |
| D21 `CloseSessionNotify` | `"hq"` {cliKey} | Find the client, then add a close job with the client's gateKey | none | 3 |
| D22 `KickNotify` | `"hqQ"` {cliKey, reason u8} | Find the client and its session, send C10, then add a close job | C10 `"hQ"` {10, reason} | 1 |

| Failure | D21 | D22 | Return |
| --- | --- | --- | --- |
| Client not found | Log, drop | Log, drop | true |
| Client session missing or closing | not tested (session not read) | Log, no C10, no close job | true |
| C10 compose fails | — | Log, no C10, no close job | true |
| Encrypt or send fails | — | Result not tested: the close job is still added | true |

C10 notes:
- The `reason` value passes unchanged. There is no reason map.
- The plain packet is 5 bytes. It uses the session crypt.
- 381 call: `r8d = rax + 8` (= 10), `r9d = movsx reason`, at `0x140010D97`–`0x140010DAB` ✅. The decompiled call also shows 10.

No handler closes the client at once, removes a manager entry, or replies to LobbyD.

### 6.4 Close job (`LobbyGate::AddJobTimer`, `ClientSessionTimerJob`)

The job reason is internal. It never goes on the wire.

381 timer-unit check (2026-11-07): `AddJobTimer` at `0x140013490` passes the config value unchanged to `TimerQueue::Add` (call `0x14001351B`). `TimerQueue::Add` at `0x14001F080` divides the delay by the queue interval (the interval is in ms, spec 01 §4). The same queue receives 5000 for reconnect at `0x140018FB0`; its caller at `0x140018CC0` uses `GetTickCount64` for elapsed time. The close timeout is in **milliseconds**, not seconds. The queue wheel can quantize the due time. Rust uses exact monotonic deadlines; original wheel phase parity remains ❓.

```
AddJobTimer(gateKey, reason):
    if client-close-timeout <= 0: do nothing          (no close ever comes from D21 or D22)
    else: run the job after client-close-timeout milliseconds

job(gateKey, reason):
    if gateKey == 0 or reason not in 1..3: log, stop
    entry = RemoveGateKey(gateKey)
    if no manager or no entry: log, stop
    if entry.cliKey != 0:
        client = AcquireClient(entry.cliKey)
        if client exists and client has a session:
            RemoveClient(entry.cliKey); close that session (not forced)
        (client missing, or no session: the client entry stays; no close)
    else if entry.session exists:
        close entry.session (not forced)
```

| Rule | Status |
| --- | --- |
| The job sends no packet | ✅ 348, 381 |
| A non-forced close waits for pending sends, so C10 can reach the client first | ✅ 348 `0x140018350`; ❓ 381 helper not read |
| A session already in force-closing state closes at once, even with a pending send (reviewer finding, 348) | ✅ 348 `LobbyGate.c:17595`; ❓ 381 |
| 381 label `Session::tryReconnect` in the job is a wrong port label. It has the position of 348 `Session::Close(false)`. | ❓ confirm when session lifecycle is written |

Quirk to keep: when the client exists but its session is gone, the gate key is removed and the client entry stays.

Allocation failures reach a crash path in the C++. They have no wire behaviour to copy. Rust returns an error and logs it.

### 6.5 Sources

| Item | 348 | 381 |
| --- | --- | --- |
| D20 handler | `0x1400102B0` | `0x140010750` (table slot `0x1400A40F0`) ✅ |
| D21 handler | `0x140010440` | `0x1400108D0` ✅ |
| D22 handler | `0x140010660` | `0x140010AE0` (`"hqQ"` `0x14008A550`, `"hQ"` `0x14008A834`) ✅ |
| `AddJobTimer` | `0x140012BF0` | `0x140013490` ✅ |
| Job `HandleEvent` | `0x140006FA0` | `0x140006F60` ✅ |
| Pump | `0x140001980`, `0x140001680` | `0x140001990`, `0x140001790` ✅ |

No behaviour difference between 348 and 381 was found in §6.

## 7. Enter-world token and character list

Target: LobbyGate `0.0.381.572` (2020). Reference: `0.0.348.527` (2019).

**Evidence notation:** ✅ 348 = read in the supplied 348 pseudocode. ✅ 381 = read in the 381 database. Register evidence is from 381 only. ❓ = not established.

### 7.1 Routing and common checks

| Input | Receive path | Client output | Evidence |
| --- | --- | --- | --- |
| D25 `GetEnterWorldTokenOk` | Normal packet handler | C14, normal packet | ✅ 348, 381 |
| D36 `GetCharacterInfoListOk` | Fragment handler, after concatenation | C28, fragment message | ✅ 348, 381 |

✅ **348, 381:** Both handlers first acquire the LobbyD session's channel.

| Condition | Action | Handler return |
| --- | --- | --- |
| No channel, or session marked for release | Log and alert `not authed session`. Do not decode. | false |
| Channel exists and session is not marked for release | Continue. The channel state value is not tested. | See below |

✅ **348, 381:** Neither handler changes the client's authentication state. Neither handler removes a client or gate-key entry. Neither handler schedules a close job or sends a reply to LobbyD.

A handler return value is not a delivery result. Section 7.4 defines its effect on the LobbyD link.

### 7.2 D25 `GetEnterWorldTokenOk` → C14

#### 7.2.1 Input

✅ **348, 381:** Input format: `"hqqh"`.

| Order | Code | Field | Use |
| --- | --- | --- | --- |
| 1 | `h` | cmd = 25 | Normal packet command |
| 2 | `q` | cliKey | Select the client |
| 3 | `q` | lobbyToken | Send this value to the client |
| 4 | `h` | worldId | Log only |

✅ **348, 381:** The handler initializes `cliKey`, `lobbyToken`, and `worldId` to zero before decoding. It tests the decoder's result. It does not require the decoded fields to consume the whole packet.

#### 7.2.2 Processing and output

✅ **348, 381:** The handler performs these steps:

1. Apply the common channel check.
2. Decode `"hqqh"`.
3. Find the client by `cliKey`.
4. Send C14 to that client's session.
5. Log `send to client ok`.
6. Return true.

✅ **348, 381:** The client packet contains only the token.

| Property | Value |
| --- | --- |
| Client command | **14** |
| Format | `"hq"` |
| Values | `{14, lobbyToken}` |
| Plain packet size | 12 bytes, including `size` |
| Fragment message | No |
| Crypt | Client's current session crypt; not broadcast crypt |
| Additional client packets | None |

✅ **381 — send-call evidence:** At `0x1400125E9`–`0x140012601`, the call has `rdx = "hq"`, `r8d = 0x0E`, and `r9 = lobbyToken`.

#### 7.2.3 Failure paths

✅ **348, 381:**

| Failure | Wire result | Handler return |
| --- | --- | --- |
| Common channel check fails | No client packet | false |
| Decode fails | Log `decompose fail`. No client packet. | **true** |
| Client not found | Log `failed to find cliKey`. No client packet. | true |
| Client's `GetSession` returns no session | `Channel::Send` returns false. The handler ignores it. No client packet. | true |
| Encryption returns false | The send path does not test the result. It still attempts the send. No Fail reply is generated. | true |
| Session send returns false | The send path does not pass this failure back to the handler. No Fail reply is generated. | true |
| Normal send path completes | C14 is submitted for sending | true |

✅ **348, 381:** `Channel::Send(format, …)` returns true when it obtains a session. It calls the void `Session::SendV` routine. It does not confirm encryption or socket-send success.

✅ **348, 381:** D25 is an exception to the general decode-failure summary in §1.3. Its decode failure returns **true**, not false.

#### 7.2.4 Quirks

✅ **348, 381:**

- `worldId` does not select an address or change server state.
- The output omits both `cliKey` and `worldId`.
- The token passes unchanged. LobbyGate does not generate a replacement token.
- The handler does not test the client's authentication state.
- Extra bytes after the required input fields are ignored.
- `send to client ok` does not prove that C14 reached the client.
- No failure path sends a client Fail packet or retries the token reply.

### 7.3 D36 `GetCharacterInfoListOk` → C28

#### 7.3.1 Input fields

✅ **348, 381:** Input format: `"hiqqhCqohsbqQQi;"`.

The fragment pump removes the fragment headers before the handler reads the message data. The decoder starts at format index 2, after `hi`.

| Order | Code | Field | Evidence |
| --- | --- | --- | --- |
| 1 | `h` | cmd = 36 | ✅ 348, 381 |
| 2 | `i` | Fragment mark, spec 03 §7 | ✅ 348, 381 |
| 3 | `q` | cliKey | ✅ Wire position in both; name from 348 |
| 4 | `q` | now | ✅ Wire position in both; name from 348 |
| 5 | `h` | timeZoneBias | ✅ Wire position in both; name from 348 |
| 6 | `C` | Character count, followed by the entry fields below | ✅ 348, 381 |

The labels `entryQ0`, `entryO1`, `text`, `entryQ5`, `byte6`, and `byte7` below identify wire fields. They do not assign game meanings.

| Entry order | Code | Field label | Wire value and decoder capacity | Meaning evidence |
| --- | --- | --- | --- | --- |
| 1 | `q` | entryQ0 | 8 bytes | ❓ Meaning |
| 2 | `o` | entryO1 | 16 bytes | ❓ Meaning |
| 3 | `h` | worldId | 2 bytes | ✅ Name from 348 |
| 4 | `s` | text | Length-prefixed UTF-16LE; destination capacity = 33 code units, including the local terminator | ❓ Meaning |
| 5 | `b` | characterData | Length-prefixed bytes; destination capacity = 1024 bytes | ✅ Name from 348 |
| 6 | `q` | entryQ5 | 8 bytes | ❓ Meaning |
| 7 | `Q` | byte6 | 1 byte | ❓ Meaning |
| 8 | `Q` | byte7 | 1 byte | ❓ Meaning |
| 9 | `i` | deleteRemainTime | 4 bytes | ✅ Name from 348 |

✅ **348, 381:** The entry format, field order, capacities, and matching output positions are the same in both builds.

✅ **381:** The low-level decoder applies these checks:

| Field | Low-level check |
| --- | --- |
| Fixed-width field | All field bytes must be present |
| `s` | Length must be nonnegative. Length must be at most 32 code units. All encoded code units must be present. |
| `b` | Length must be nonnegative and at most 1024 bytes. All blob bytes must be present. |

**These checks do not guarantee a dropped D36 message.** The fragment decode wrapper discards the low-level failure result. Section 7.3.4 defines this quirk.

❓ The units and epoch of `now`, `timeZoneBias`, and `deleteRemainTime` are not established here.

#### 7.3.2 Processing

✅ **348, 381:** For a valid input, the server performs these steps:

1. Join the D36 fragments.
2. Apply the common LobbyD channel check.
3. Decode `cliKey`, `now`, and `timeZoneBias`.
4. Read the repeat count.
5. Interpret that count as a **signed 16-bit** value in this handler.
6. For each positive-count entry, decode the nine entry fields in order.
7. Close the repeat group.
8. Find the client by `cliKey`.
9. Obtain the client's session.
10. Require session state **5** (`CONNECTED` in 348) and a socket other than `-1`.
11. Build a new C28 fragment message.
12. Finalize its fragment marks.
13. Encrypt each output fragment with the client's current session crypt.
14. Submit the buffer list to the session send path.
15. Release the temporary decoded data.
16. Return true.

✅ **348, 381:** The server builds the client message again. It does not relay the original D36 fragments.

#### 7.3.3 Client message and fragment boundaries

✅ **348, 381:** Output format: `"hiqhCqohsbqQQi;"`.

| Property | Value |
| --- | --- |
| Client command | **28** |
| Leading data fields | `{now q, timeZoneBias h}` |
| Repeat group | `{entryQ0 q, entryO1 o, worldId h, text s, characterData b, entryQ5 q, byte6 Q, byte7 Q, deleteRemainTime i}` |
| cliKey | Omitted |
| Entry order | Preserved |
| Crypt | Client's current session crypt; not broadcast crypt |
| Encryption unit | One client frame per fragment |
| Other client packets | None |

✅ **348, 381:** Fixed-width values pass unchanged. The two `Q` fields retain their low eight bits. The sender does not convert them to `value != 0` or `value == 1`. A valid blob uses the decoded length and bytes.

✅ **348, 381:** An empty list is valid. The output still contains `now`, `timeZoneBias`, and a zero repeat count. Its plain fragment is 20 bytes. Its single-fragment mark is 12.

✅ **348, 381:** The sender uses the fragment rules in spec 03 §7:

- Each fragment has command 28.
- Each fragment is at most 4000 plain bytes.
- A field is never split.
- An entry can continue in the next fragment.
- The first mark gives the total data length.
- Middle marks are `-1`.
- The last mark is `0` when there is more than one fragment.
- The repeat count counts complete output entries across the message.

✅ **348, 381 — boundary quirk:** Before writing `text`, the sender requires **68 free bytes** (`0x44`). It starts a new fragment when fewer bytes remain, even if the actual string is short enough to fit. Other entry fields use their fixed width, or `blob length + 2` for `b`.

✅ **381 — string quirk:** The sender measures `text` up to its first local UTF-16 NUL. An embedded NUL in the input string therefore shortens the output string. This path is not a byte-for-byte string relay.

❓ The 348 low-level string writer was not read for the embedded-NUL edge case in this job.

**381 compose and send evidence**

✅ **381:** The output command is stored as `0x1C` at `0x14000482E`–`0x140004859`. The format starts at index 2.

| Call address | Operation | Register values |
| --- | --- | --- |
| `0x140004879` | Compose leading fields | `edx = 2`, `r8 = now`, `r9d = sign-extended timeZoneBias` |
| `0x140004904` | Compose entryQ0 | `edx = 1`, `r8 = entryQ0` |
| `0x140004950` | Compose entryO1 | `edx = 1`, `r8 = address of copied 16-byte value` |
| `0x140004992` | Compose worldId | `edx = 1`, `r8d = sign-extended worldId` |
| `0x1400049D4` | Compose text | `edx = 1`, `r8 = text buffer` |
| `0x140004A29` | Compose characterData | `edx = 1`, `r8 = blob buffer`, `r9 = blob length` |
| `0x140004A6B` | Compose entryQ5 | `edx = 1`, `r8 = entryQ5` |
| `0x140004AAE` | Compose byte6 | `edx = 1`, `r8d = sign-extended byte6`; `Q` writes one byte |
| `0x140004AF1` | Compose byte7 | `edx = 1`, `r8d = sign-extended byte7`; `Q` writes one byte |
| `0x140004B33` | Compose deleteRemainTime | `edx = 1`, `r8d = deleteRemainTime` |
| `0x140004BC2` | Encrypt buffer list | `rcx = list head`, `rdx = session.currentCrypt`, `r8 = session.security3rd` |
| `0x140004BE3` | Session send | `rcx = client session`, `rdx = address of buffer-list pointer`; command 28 is already in the fragments |

#### 7.3.4 Decode-result quirk

✅ **348, 381:** `PacketConcatenateBuffer::Decompose(int, …)` does **not** return the low-level decoder's success or failure.

| Low-level result | Wrapper action |
| --- | --- |
| Positive consumed-byte count | Advance the format index and data offset. Return true. |
| Zero, including a decode error | Advance the format index by the requested field count. Add zero to the data offset. Return **true**. |
| No buffer, or data offset already at or beyond the buffer end | Enter the assertion/crash path. There is no normal false return from this path. |

✅ **381 — disassembly evidence:** After the low-level decode call at `0x14001DE8A`, the wrapper advances the format index, adds the returned byte count to the offset, and executes `mov al, 1` at `0x14001DE9C`. It does not test for zero.

✅ **348, 381:** The D36 handler contains two nominal decode-failure branches:

| Nominal failure branch | Branch result | Reachability through this wrapper |
| --- | --- | --- |
| Decode the three leading fields returns false | Log, clear the joined packet, return false | Not reached when the wrapper returns normally |
| Decode one entry returns false | Log, free temporary data, clear the joined packet, return false | Not reached when the wrapper returns normally |

✅ **348, 381:** A low-level decode error can leave fields written before the error intact. Other entry fields retain their zero-initialized values. The blob capacity starts at 1024. The handler then continues unless a later operation enters a failure or crash path.

✅ **348, 381:** Do not describe an invalid string length, invalid blob length, or truncated entry as an unconditional D36 drop. The low-level error is swallowed. The handler can continue to client lookup and output composition.

**Count and trailing-data quirks**

✅ **348, 381:**

| Input condition | Behaviour |
| --- | --- |
| Count = 0 | Send an empty list if client lookup and sender checks succeed |
| Count in `0x8000`…`0xFFFF` | The handler sign-extends it to a negative value. It skips entry decoding. The client sender writes an empty repeat group. |
| Repeat-count reader reports `-1` | No separate rejection branch exists. The handler follows the nonpositive-count path and attempts to close the repeat group. |
| Bytes remain after the declared entries | No final full-consumption check rejects them |
| Repeat-group pop reports a remaining count | The handler does not test that result |

❓ A malformed count or a swallowed decode error has no single guaranteed client result. Later assertions and the retained field values affect the result. This section does not assign a clean Fail reply to those cases.

Rust safety policy: reject malformed D36 fields. Do not reproduce swallowed decoder errors or assertion paths. This differs from the wrapper at `0x14001DE8A`–`0x14001DE9C`. Signed nonpositive counts still produce an empty list. No Fail packet is generated.

#### 7.3.5 Handler and sender failure paths

✅ **348, 381:**

| Failure | Client result | Handler return |
| --- | --- | --- |
| Common LobbyD channel check fails | Nothing sent | false |
| Nominal leading-field decode-failure branch | Nothing sent; joined packet cleared | false; see §7.3.4 |
| Nominal entry decode-failure branch | Nothing sent; temporary data released; joined packet cleared | false; see §7.3.4 |
| Client not found | Log `failed to acquire client`. Nothing sent. | false |
| Client's session is missing or unavailable | Sender returns false. Handler logs send failure. Nothing sent. | false |
| Client session is not connected, or socket = `-1` | Sender returns false. Handler logs send failure. Nothing sent. | false |
| Any checked, one-field entry compose call returns zero | Discard the unsent output buffer list. Sender returns false. Handler logs send failure. | false |
| Encryption returns false | Result ignored. The sender still calls the session send path. No Fail reply is generated. | true |
| Session send returns false | Result ignored. No Fail reply is generated. | true |
| Send submission completes | C28 fragments submitted for sending | true |

✅ **348, 381:** The initial two-field compose result is not tested. The fragment-finalize result is not tested. The checked compose failures are the nine one-field entry calls.

✅ **348, 381:** No fragment is submitted by this sender until the message has been composed. An entry-compose failure therefore discards the whole unsent message, not just its last entry.

✅ **381:** Temporary entry and blob allocations have no clean allocation-failure return path in this handler. Assertion and allocation faults do not generate a protocol Fail packet.

❓ No normal handler return value is assigned here to a process fault.

### 7.4 Meaning of `false` for fragment commands

This resolves the fragment-pump open item in §6.1.

✅ **348, 381:** `TFPP_PumpPacket` selects the fragment handler when that command's fragment-table entry is nonzero. In 381, its call to `__pump_fragment_packet` is at `0x140002111`.

✅ **348, 381:** The fragment pump acts as follows:

| Condition | Pump action | Return to receive path | Link result |
| --- | --- | --- | --- |
| Fragment accepted, message incomplete | Keep the partial message. Do not call the handler. | true | LobbyD link stays open |
| Concatenation fails | Log. Clear the session's concatenater. Do not call the handler. | true | Drop the partial message; link stays open |
| Message complete, queue post succeeds | Copy the complete message. Clear the session's concatenater. Queue the handler event. | true | Link stays open |
| Queued fragment handler returns false | Ignore that result. Release the event. `ThreadFreeFragmentPacketPump::HandleEvent` returns true. | No handler false reaches the receive path | Link stays open |
| Queue post fails | Call the fragment handler at once on the receive path. Return its result. | Handler result | **false closes the LobbyD link** |
| Fragment pump event initialization fails | Log. Release the copied message and event. Do not call the handler. | **true** | Drop the message; link stays open |

✅ **348, 381:** The event-initialization result differs between the two pump types:

| Pump | Initialization failure return |
| --- | --- |
| Normal `__pump_packet` | false |
| Fragment `__pump_fragment_packet` | **true** |

✅ **348, 381:** The invalid-parameter check in `TFPP_PumpPacket` is outside the fragment helper. It returns false before either helper runs.

✅ **348, 381:** A fragment handler's false result does not cause an automatic client Fail reply or retry. On the normal queued path, it does not close either socket. On queue-post failure, it can close the **LobbyD** socket through the receive path.

✅ **348:** The fragment pump also contains a copy-failure branch. It logs, clears the session concatenater, and returns true. The supplied copy implementation returns true after copying, unless it enters a fault path.

✅ **381:** The copy operation is inline. It has assertion checks and the copy itself. It has no separate copy-failure false-result branch.

### 7.5 Build comparison

| Area | Comparison | Evidence |
| --- | --- | --- |
| D25 input and C14 output | Same commands, formats, token value, omitted fields, and handler returns | ✅ 348, 381 |
| D36 input and C28 output | Same formats, field order, capacities, client checks, and checked compose-failure returns | ✅ 348, 381 |
| C28 fragment boundaries | Same 4000-byte limit and 68-byte precheck for `text` | ✅ 348, 381 |
| Send-result handling | Both paths ignore encryption and session-send failure results | ✅ 348, 381 |
| Fragment decode wrapper | Both builds return true after a low-level zero result | ✅ 348, 381 |
| Signed character count | Same nonpositive-count behaviour | ✅ 348, 381 |
| Queued and inline fragment-handler results | Same behaviour | ✅ 348, 381 |
| Fragment event initialization failure | Both return true, unlike the normal pump | ✅ 348, 381 |
| Complete-message copy | 348 calls the copy helper and tests its result. 381 has the copy inline, without a recoverable false-result branch. | ✅ 348, 381 |
| Temporary entry/blob storage | 348 uses memory-manager `AllocZero`/`Free` calls. 381 uses `malloc`, conditional zeroing, and `free`. | ✅ 348, 381 |
| Embedded-NUL string output | Verified in 381. The matching 348 low-level writer was not read for this edge case. | ✅ 381; ❓ 348 |

✅ **348, 381:** No wire-behaviour difference was found in the compared paths. The copy and allocation changes above change code organization. They do not change the valid packet formats.

### 7.6 Sources

| Item | 348 address | 381 address |
| --- | --- | --- |
| D25 handler | ✅ `0x140011EE0` | ✅ `0x1400123C0`; main-table slot `0x1400A4118` |
| D25 client send | ✅ In D25 handler: `Channel::Send("hq", 14, lobbyToken)` | ✅ `0x140012601`; format at `0x1400846F0`; input format at `0x14008C048` |
| D36 handler | ✅ `0x140010AB0` | ✅ `0x140010F50`; fragment-table slot `0x1400A44F0`; input format at `0x14008A9B0` |
| C28 sender | ✅ `0x140004500` | ✅ `0x1400046D0`; output format at `0x140081900` |
| `Channel::Send(format, …)` | ✅ `0x1400216C0` | ✅ `0x140031C40` |
| Fragment decode wrapper | ✅ `0x14001BC10` | ✅ `0x14001DDF0` |
| Repeat-count reader | ✅ `0x14001B8B0` | ✅ `0x14001DAF0` |
| Fragment concatenater `Add` | ✅ `0x1400025D0` | ✅ `0x140002810` |
| Complete-message copy | ✅ `0x140002770`, with base copy at `0x14001BE20` | ✅ Inline at `0x140001D7E`–`0x140001EFE` |
| Fragment splitter `Finalize` | ✅ `0x1400023A0` | ✅ `0x140002520` |
| `TFPP_PumpPacket` | ✅ `0x140001FC0` | ✅ `0x1400020A0`; fragment call at `0x140002111` |
| `__pump_fragment_packet` | ✅ `0x140001BC0` | ✅ `0x140001B90` |
| Fragment pump `HandleEvent` | ✅ `0x1400018C0` | ✅ `0x1400018D0`; handler call at `0x140001925`; true return at `0x14000197A` |
| Normal pump, for comparison | ✅ `0x140001980` | ✅ `0x140001990` |
| Normal pump `HandleEvent` | ✅ `0x140001680` | ✅ `0x140001790` |

## 8. Session lifecycle and goods handlers

Status: Sol drafted this section and Sol reviewed the risky claims in IDA (381). The main session did not re-read it in IDA.

### 8.1 LobbyD → client relay failures

✅ Both failures **drop the packet and return true**. Neither failure closes a link or sends a reply.

| Failure | Exact log string | LobbyD link | Client link |
|---|---|---|---|
| Cannot decode `"hq"` | `[lobbyd], 0x%.16I64X, %I64d, decompose fail, cmd:%d, cliKey:0x%I64X` | Stays open | No change |
| Cannot find `cliKey` | `[lobbyd], 0x%.16I64X, %I64d, failed to find cliKey, cmd:%d, cliKey:0x%I64X` | Stays open | No client found; other clients stay open |

**Sources:** handler `0x1400126B0`; log strings `0x14008C410` and `0x14008C520`.

### 8.2 Client channel states

✅ The client channel state is a 32-bit field at **channel +0x28**. It is separate from the socket state and the crypt mode.

| Value | Meaning | What sets it | 381 source |
|---|---|---|---|
| **0** | Not authenticated | `AddClient` creates the channel after InitSessionOk | `0x140005FF0` |
| **1** | Authenticated | D10 or D16 StartSessionOk finds the client and sets its state before sending the client login reply | `0x14000CBC0`; `0x14000F410` |

✅ Client dispatch accepts a channel as authenticated only when this field equals **1** (`0x140006AC0`). A completed crypt handshake does not establish authentication.

❓ No other client channel value was found. A complete audit of indirect and register-valued stores remains open. Check all writes to client **+0x28** to prove that the table is exhaustive.

### 8.3 Client session end

✅ `OnClose` selects the notification by the available client channel or gate-key entry. It does **not** test the client channel state.

| Condition | Packet to LobbyD | Fields |
|---|---|---|
| Client channel is available | **D19 `LostConnectionNotify`**, `"hq"` | `{19, channel.cliKey}`; applies to state **0 and 1** |
| No available channel; session lookup gives a positive gate key | **D18 `LostAnonymousConnectionNotify`**, `"hq"` | `{18, gateKey}` |
| Neither condition applies | None | — |

✅ Each notification is one **12-byte plain packet**:

| Offset | Bytes | Value |
|---|---|---|
| 0 | u16 LE | 12 |
| 2 | u16 LE | 19 or 18 |
| 4 | u64 LE | `cliKey` or `gateKey` |

There is no reason field. The gate key does not appear in D19.

✅ The sender requires a connected LobbyD transport and a valid socket. It does not require LobbyD channel state 1. If that check fails, it logs the failure and sends nothing. `OnClose` does not retain the notification for retry.

**Sources:** `OnClose` `0x140006790`; D18 sender `0x14000A650`; D19 sender `0x14000A890`.

### 8.4 Timers and LobbyD recovery

| Timer | Value and source | Check or action | Evidence |
|---|---|---|---|
| ✅ Client idle timeout | Config **`max-idle-time`**, in seconds. Startup uses its low 16 bits and multiplies by 1,000. **`max-idle-time-enable`** controls installation. | Watcher scans every **5,000 ms**. Expiry requires `last receive + timeout < now`. It marks the candidate on one scan, then checks it again on the next scan. | Setup `0x140013AB0`; scan `0x140036950`; interval `0x140036B90` |
| ✅ Idle expiry result | Config **`disconnect-after-idle-time`** | True: request a forced close. False: log only. Accept sets the initial receive time. Receive processing updates it. | `0x140036950`; `0x1400216D0`; accept `0x140018780`; receive `0x1400191A0` |
| ❓ Numeric idle-time default | Config definition points to **`0x1400761C0`**; its value was not read. | Read that default and the deployed config next. | Definition entry `0x1400A5388` |
| ❓ Client handshake timeout | No separate value, interval, or expiry action was established. | Check session setup `0x140018220`, the client crypt path, and timer creation next. Do not treat the outgoing TCP connect deadline as a client handshake timeout. | Accepted-client path `0x140018780` |
| ✅ LobbyD reconnect | Scheduled retries use constant **5,000 ms**. The close path can attempt the first reconnect immediately. | A failed reconnect schedules another attempt. | `0x140018CC0`; `0x140018B00` |

✅ **LobbyD drops:** `OnClose` sets LobbyD channel state to **0**. It does not close connected clients or change their authentication state. A later client relay fails and closes that client if the LobbyD transport is unavailable.  
Sources: `0x140008FA0`; relay `0x140015B00`.

✅ **LobbyD returns:** `OnConnect` sends D1 HelloReq again. D2 HelloOk sets LobbyD channel state to **1**. These paths do not replay client logins or lost-connection notifications. Existing client channels remain unchanged.  
Sources: `0x140008DA0`; `0x14000AB70`.

### 8.5 Port crypt mode and internal port

| Fact | 381 evidence |
|---|---|
| ✅ Client port uses port kind **0**. `Port::Initialize` stores **2** at **Port +0x14**. | Caller `0x140013AB0`; store **`0x140035CD4`** |
| ✅ Other supported port kinds **1, 2, 3** use crypt mode **1**. LobbyD uses kind **2**. | Stores `0x140035C70`, `0x140035CCA`; LobbyD setup `0x140013AB0` |
| ✅ Startup opens no separate `internal-port` listener. It opens the client listener and an outgoing LobbyD port with local listen port 0. `internal-port` is advertised in D1. | `0x140013AB0`; D1 send `0x140008DA0` |

❓ No wire difference from build 348 was established in this pass. The 348 file supplied symbol names only.

### 8.6 Goods reply common rules (381)

✅ The main table at `0x1400A4050` and fragment table at `0x1400A43D0` both select these handlers:

| D command | Handler | Main slot | Fragment slot |
| --- | --- | --- | --- |
| 68 | `0x140011490` | `0x1400A4270` | `0x1400A45F0` |
| 71 | `0x1400118C0` | `0x1400A4288` | `0x1400A4608` |
| 74 | `0x140011C90` | `0x1400A42A0` | `0x1400A4620` |
| 80 | `0x140012050` | `0x1400A42D0` | `0x1400A4650` |

✅ Each handler requires an available LobbyD channel (`session +192`, unavailable flag at `+208`). Failure returns false. The handlers then decode the joined body and look up `cliKey`. A missing client or a failed client sender only logs and drops the reply. These paths return **true**, not false. The joined packet is cleared. Sources: the four handler addresses above.

✅ Fragment-handler false follows §7.4. Queued false is ignored. Inline false after queue-post failure can close LobbyD. No automatic client Fail is sent. Sources: `0x1400018D0`, `0x140001B90`.

✅ Each client sender requires an available session, session state 5, and a socket other than -1. It uses `session.currentCrypt` and `security3rd`, not the broadcast crypt. It finalizes fragments before submission. Checked compose failures discard the unsent list. Encryption and send return values are ignored. Sources: senders in §§8.7–8.10.

✅ Repeat counts are signed 16-bit values in these handlers. Nonpositive counts skip the corresponding rows and emit an empty repeat group. Trailing input has no final consumption check. The fixed-field wrapper can swallow decoder errors as in §7.3.4. Sources: handlers above; wrapper `0x14001DDF0`; repeat reader `0x14001DAF0`.

Rust safety policy: reject malformed fields and oversized output fields. Do not reproduce unsafe allocations or decoder faults. No Fail reply is added. ❓ Malformed original input has no single guaranteed result.

### 8.7 D68 ListGoodsOk → C60

✅ Input format is `hiqCqiQQqqssissbbCiiibb;;` at `0x14008ADE0`. `hi` is the fragment header. The body starts with `cliKey`, then goods rows. Handler: `0x140011490`.

✅ Output format is `hiCqiQQqqssissbbCiiibb;;` at `0x140081910`. The sender removes only `cliKey`. Sender: `0x140004D20`.

✅ Each goods row has `q i Q Q q q s s i s s b b`, then nested item rows `i i i b b`. The four string capacities, including local NUL, are 51, 81, 81, and 201 UTF-16 units. Strings are emitted only up to the first NUL. Blob bytes are preserved. The variable-data decoder rejects a negative length or a blob that exceeds the remaining input. Source: `0x14001DEB0`. No payment or entitlement decision occurs here. Sources: setup `0x140016510`; fetch `0x1400168C0`; item fetch `0x140016370`.

✅ C60 splits at individual field boundaries, including nested count fields. Strings and blobs use their actual encoded length. Maximum plain fragment size is 4000 bytes. Sources: `0x1400168C0`, `0x140016370`, `0x140004D20`.

✅ C59 → D67 already relays the request body unchanged under spec 04 §4. Source: `0x140015B00`.

### 8.8 D71 TakeGoodsOk → C63

✅ Input `hiqqCqqqiQ;` at `0x14008B1B8`: `cliKey`, one qword, then rows `q q q i Q`. Output `hiqCqqqiQ;` at `0x1400819B8` removes only `cliKey`. Sources: handler `0x1400118C0`; sender `0x1400050F0`.

✅ The output qword precedes the repeat count. Each 29-byte row stays in one fragment. The sender reserves 29 bytes before composing all five row fields. Source: `0x1400050F0`.

✅ C62 → D70 already relays the request. Source: `0x140015B00`.

### 8.9 D74 ListLobbyServiceItemInfoOk → C66

✅ Input `hiqQiCqqqiQ;` at `0x14008B628`: `cliKey`, one byte, one dword, then the same rows as D71. Output `hiQiCqqqiQ;` at `0x140081AE8` removes only `cliKey`. Sources: handler `0x140011C90`; sender `0x140005550`.

✅ Each 29-byte row stays in one fragment. The byte and dword precede the repeat count. Source: `0x140005550`.

✅ C65 → D73 already relays the request. Source: `0x140015B00`.

### 8.10 D80 GetServiceEntryListOk → C72

✅ Input `hiqCqQqohq;` at `0x14008BBD8`: `cliKey`, then rows `q Q q o h q`. Output `hiCqQqohq;` at `0x140081B68` removes only `cliKey`. Sources: handler `0x140012050`; sender `0x1400059B0`.

✅ Each 43-byte row stays in one fragment. The identifier keeps all 16 bytes. Source: `0x1400059B0`.

✅ C71 → D79 already relays the request. Source: `0x140015B00`.
