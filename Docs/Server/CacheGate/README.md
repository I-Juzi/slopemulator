# CacheGate (build 348)

Source: `CacheGate.exe` + PDB (IDA, names present). Service name `cacheGate`.
Rust port: `Program/Server/CacheGate` (crate `cachegate`).

CacheGate is a relay. Game clients (GameDaemons and other economy users)
connect to the public port. CacheGate connects out to every CacheD in
`cached-list.config.xml`. Relay packets go from a client to a CacheD by
`toWid`, and from a CacheD back to the client whose Hello used that wid.

```
client (wid W) --port 1 (20800)--> CacheGate --port 2 (outbound)--> CacheD (wid C)
        <------ Relay / RelayFail ----        <------ Relay ---------
```

## 1. Ports

| Port id | Direction | Handler | Limit |
| --- | --- | --- | --- |
| 1 | Listen on `public-port` (CACHEGATE_PUBLIC_PORT_VALUE @0x14004611C = 20800) | CacheGateSessionHandler | max-public-session-count + 1 when > 0, else system default |
| 2 | Connect to each CacheD (port number 0) | CacheDChannel | max-cached-session-count |

- Handlers test `HIDWORD(session->_id) == 1` for "sent by a client".
- initNetwork: OpenPort 2 → `CacheDList::MakeSessionAll` → OpenPort 1.
- Reconnect: `Session::tryReconnect`, 5000 ms.
- Timer: interval 1000, max time 20.

## 2. Config

### cachegate.config.xml

`<config module='cacheGate'><option name value/>`. `CacheGateConfig::_Def`
@0x1400596C0, 13 entries, struct size 0x38. Loaded from `initLog`.

| Group | Option | Offset | Kind | Default |
| --- | --- | --- | --- | --- |
| path | root-path | 0x0 | REQ string | — |
| path | log-path | 0x8 | REQ string | — |
| path | config-path | 0x10 | REQ string | — |
| system | worker-thread | 0x20 | OPT int | 0 |
| system | dump-type | 0x24 | OPT int | 1 (DEFAULT_DUMPTYPE @0x140046314; 0 none, 1 minimal, 2 full) |
| system | max-cached-session-count | 0x28 | REQ int | — |
| system | max-public-session-count | 0x2c | REQ int | — |
| network | public-port | 0x30 | OPT | 20800 |
| economy | wid | 0x34 | OPT u16 | 0 |

- `bns.common.config.xml` (module bns-common-config: p-type, np-program-id
  default 303, np-game-code, np-app-id, np-client-app-id) also loads.
  Initialize fails when it fails. The Rust port does not read it (no NP use).
- Samples: `Service_348_8349/Service/CacheGate/bin/cachegate.config.xml`
  (wid 0, counts 0); `BnS_2020/config/CacheGate/bin/cachegate.config.xml`
  (wid 10060, max-cached 1000, max-public 100, dump-type 2).

### cached-list.config.xml (in config-path)

Loader `CacheDList::LoadCachedListConfigFile` @0x140002B20 (XmlReader "15").

```xml
<cached-list>
  <cached wid='1' ip-address='10.10.110.55' internal-port='20200' economy-port='20202'/>
</cached-list>
```

| Attribute | Parse | Bad value |
| --- | --- | --- |
| wid | `_wtoi` | 0 |
| ip-address | `inet_addr` | INADDR_NONE (255.255.255.255) |
| economy-port | `_wtoi` | 0 |
| internal-port | not read | — |

- Every child must be `cached` (case-sensitive). Any other element fails the load.
- Merge: same wid and same addr/port → keep. Changed → replace, connect the
  new one when reloading, StopReconnect + Close the old one. New → add
  (connect only when reloading). Rows missing from the file stay.
- Start: when the load fails, write a sample file (CREATE_NEW) and fail.
  Sample text: `CacheDList::writeSampleConfigFormat` (`config::SAMPLE_CACHED_LIST`).
- `reload`: cached list (reload = true), then bns common config. Returns 1.

## 3. Dispatch

`PacketHandler` @0x140004150 reads `CacheGatePacketServerSideHandlerList[cmd]`
@0x1400592D0 (10 slots) with no bounds check. Names @0x140059288.
Fragment list @0x140059320: only slot 9.

| Cmd | Name | Handler | Result |
| --- | --- | --- | --- |
| 0 | — | null | packet fails, session closes |
| 1 | Hello | 0x140004C00 | section 4 |
| 2 | HelloOk | 0x140004F20 | section 4 |
| 3 | HelloFail | 0x140005040 | log `hE` code, return false |
| 4 | Relay | 0x1400051C0 | section 5 |
| 5 | RelayFail | 0x1400057A0 | log `hhhqhqE`, return false |
| 6 | RelayInvalid | 0x14000CD80 | TheOtherSideListAck: return true |
| 7 | TheOtherSideListReq | 0x14000CD80 | return true |
| 8 | TheOtherSideListAck | 0x14000CD80 | return true |
| 9 | FragmentRelay | 0x140005880 | section 6 |
| ≥ 10 or < 0 | — | out of table | Rust closes the session ❓ (native reads past the table) |

A handler that returns false closes the session.

## 4. Hello and HelloOk

Hello `hhhh`: cmd, packetVersion, economyPacketVersion, wid.

| Check | Reply | Result |
| --- | --- | --- |
| Decompose fails | none | close |
| packetVersion != 4 or economy != 10 | HelloFail 5 | close |
| wid already in ClientManager | HelloFail 7 | close the old session and this one |
| AddSession fails (session id already indexed) | HelloFail 2 | close |
| OK | HelloOk (`h`, 4 bytes) | keep |

- ClientManager: `clientMap` wid → session, `idIndexMap` session → wid.
  `ReleaseSession` on client OnClose.
- Hello from a CacheD session: Rust closes it ❓ (native path not traced).
- Outbound: `CacheDChannel::onConnect` @0x140003C80 runs `CacheD::Verify`
  (`_addr` and `_port` equal the peer) and sends Hello `[1, 4, 10, wid]`.
  The wid is `LOWORD(v8[1].__vftable)` = the CacheD id ❓ (not the gate wid).
- HelloOk needs `session->_channel` (a CacheD), else log "unknow hello ok"
  and return false. Verify == 0 → status = 1. onClose → status = 0.

## 5. Relay

Format `hhhqhq` + payload: cmd, fromWid, toWid, tid u64, hint u16,
reserved u64. The header is 22 bytes; payload starts at frame offset 26.

| From | Lookup | Fail reason |
| --- | --- | --- |
| client | `CacheDList::Find(toWid)`, needs status == 1 | 3 (exists, not ready), 8 (no CacheD) |
| CacheD | `ClientManager::GetSession(toWid)` | 8 |

- Success: forward the frame unchanged (same cmd 4, header and payload).
- Failure: RelayFail (cmd 5, 28 bytes) to the sender, header
  {fromWid = own config wid, toWid = original fromWid, tid, hint, reserved}
  and `h` reason.
- Short header (decompose fails): Rust closes ❓.
- The handler returns true.

## 6. FragmentRelay (cmd 9)

Frame: size i16, cmd i16, mark i32, data. One concatenater per session.

| Mark | Buffer | Result |
| --- | --- | --- |
| > 0 | has data | fail: clear, close |
| > 0 | empty | start; complete when data length == mark |
| ≤ 0 | empty | fail: close |
| ≤ 0 | has data | append; complete when mark == 0 |

- Complete: decompose `hihhqhq` (the relay header is the first 22 data bytes)
  and route as in section 5. Forward the chain with `Session::Send(Packet)`.
  Rust forwards the original frames unchanged ❓.
- Failure: non-fragment RelayFail (28 bytes) with reason 3 or 8. Clear, keep.
- Rust only: a chain over 1 MiB, or data past `mark`, closes the session.

## 7. Rust port map

| Native | Rust |
| --- | --- |
| CacheGateConfig, CacheDList::LoadCachedListConfigFile | `config.rs` |
| Packet formats, reasons | `protocol.rs` |
| ClientManager, CacheDList state, handlers | `gate.rs` (pure) |
| Ports, CacheDChannel, reconnect | `server.rs` (tokio) |
| Tests | `tests/gate.rs`, `tests/network.rs` |
