# SmartCouponDaemon (build 348)

Source: `SmartCouponDaemon.exe` + PDB (GUID F07533E7-94BA-4ECE-89D4-868CCB48CA42),
build path `E:\DailyBuild02\SmartCouponDaemon-Common-0.0.348`. Service name `smartcoupond`.
The 348 log at `Service_348_8349/Service/SmartCouponDaemon` shows only a failed start (`cannot execute, error:10`).
Rust port: `Program/Server/SmartCouponDaemon` (crate `smartcoupondaemon`).

```
GameDaemon --(22000: hello, game event)--> smartcoupond --(event noti)--> NP (6601)
                                               ^   |
manager    --(32000: retry, load retry)--------+   +-- store (usp_* procedures)
```

## 1. Config

| File | Loader | Content |
| --- | --- | --- |
| smartcoupond.config.xml | Daemon options | `<config module='smartcoupond'><option name value/>` |
| game-event.config.xml | GameEventConfig::SetupGameEventListFromXml @0x140002C80 | root → daemon-type groups → `<event id name type enable>` |
| gamed-list.config.xml | DChannelMgr::SetupDaemonListFromXml @0x14000CAC0 | `<gamed wid ip-address internal-port>` |

Options in the shipped 348 file: root-path, log-path, config-path, cert-file, worker-thread 0,
game-event-worker-thread 0, max-world 100, dump-type 2, init-job-count 4096,
wait-time-for-reuse-pool-object 0, job-hash-count 0, np-enabled yes, np-address 10.10.110.55,
np-port 6601, manage-listen-port 32000, daemon-listen-port 22000, service-world-id 1,
np-reward-changeover-enable yes.

- SmartCouponDaemonTypeDesc @0x1400835F8 = `none`, `gamed`, then `bns-common-config` (next string, not a type). Group names match case-insensitively; index 0 (`none`) is skipped.
- Event: all four attributes are required. `enable` is "true" (case-insensitive). Name max 260. Sample comment: type 1 = account, 2 = character; gamed event ids use the 10000 range.
- gamed-list: element name must be `gamed`, otherwise log "unknown element" and skip. Empty wid/ip/port → entry skipped. XmlReader_sv64.dll interface version must be "15".
- Port: missing-attribute event → load error ❓ (binary path not traced).

## 2. Wire

Trusted frame `i16 size, i16 cmd, body`. `Q` = 1 byte, `o` = 16-byte account id.

### Daemon port (DChannelMgr::PacketHandler @0x14000DFE0, table @0x140083660, 10 entries)

cmd must be ≥ 0. Before hello only cmd 1 is accepted.

| Cmd | Dir | Format | Fields |
| --- | --- | --- | --- |
| 1 | gamed→ | `hhQhh` | cmd, packetVersion (=1), daemonType, worldId, internalPort |
| 2 | →gamed | `h` | hello ack |
| 3 | →gamed | `hh` | hello fail, reason |
| 4 | gamed→ | `hqiohq` | cmd, tid, gameEventId, account, worldId, pcdbid |
| 5 | →gamed | `hqq` | tid, gameEventLogId |
| 6 | →gamed | `hqE` | tid, reason |
| 7 | gamed→ | `hqio` | cmd, tid, gameEventId, account |
| 8 | →gamed | `hqq` | tid, gameEventLogId |
| 9 | →gamed | `hqE` | tid, reason |

### Manager port (ManagerSessionHandler::PacketHandler @0x14000E980, table @0x1400836B0)

cmd must be 1..9.

| Cmd | Dir | Format | Fields |
| --- | --- | --- | --- |
| 1 | mgr→ | `hhh` | cmd, version (=2), managerWid |
| 2 / 3 | →mgr | `h` / `hh` | hello ack / fail reason |
| 4 | mgr→ | `hqoq` | cmd, tid, account, publicationId |
| 5 | →mgr | `hqq` | tid, acceptTid (NP) |
| 6 | →mgr | `hqE` | tid, reason |
| 7 | mgr→ | `hq` | cmd, tid |
| 8 / 9 | →mgr | `hq` / `hqE` | ack tid / fail tid, reason |

## 3. Handlers

### Hello (HandleSmartCoupon_HelloReq @0x140007DC0)

Reasons read from the disassembly (r9 of each `Session::Send("hh", 3, r)`):

| Order | Check | Reply | Session |
| --- | --- | --- | --- |
| 1 | session already authed | none | close |
| 2 | decode fail (0x140007EC1) | 3, reason 2 | close |
| 3 | version ≠ 1 (0x140007F22) | 3, reason 9 | close |
| 4 | service not available (0x140007F9F) | 3, reason 8 | close |
| 5 | {type, wid} not in gamed-list (0x14000803D) | 3, reason 2 | close |
| 6 | remote IP ≠ list IP (0x1400080D7) | 3, reason 11 | close |
| 7 | world already has a session (0x140008252) | 3, reason 13; old channel closed and replaced | close |
| 8 | ok | 2 | keep |

### GameEventByCharacterReq (@0x140008360) / ByAccountReq (@0x140008E80)

| Order | Check | Reason |
| --- | --- | --- |
| 1 | decode fail | 2 (tid 0 ❓) |
| 2 | char: account 0, world ≤ 0, world ≠ channel wid, pcdbid ≤ 0, event ≤ 0; account: account 0, event ≤ 0 | 7 |
| 3 | event not in game-event config | 7 |
| 4 | event daemonType ≠ channel daemon type | 7 |
| 5 | usp_RegisterGameEventLog fails | 6 |
| 6 | ok: reply 5/8 [tid, logId], post GameEventLogEvent (target 2 char / 1 account) | — |

Account requests bind world and pcdbid as NULL. `enable` and eventType are not checked here ❓.
After a failure reply the session stays open ❓.

### Manager

| Handler | Order | Result |
| --- | --- | --- |
| HelloReq @0x14000EF40 | already hello → 8; decode → 2; version ≠ 2 → 9 | cmd 3 + close ❓; ok → cmd 2 |
| RetryCouponPublicationReq @0x14000F2F0 | decode or no IOCP → 2; tid ≤ 0, account 0, publication ≤ 0 → 7 | fail cmd 6; ok → post RetryCouponPublicationEvent (no direct reply) |
| LoadRetryReadyGameEventLogListReq @0x14000F7F0 | decode → 2; tid ≤ 0 → 7; DB → 4; post → 2 | fail cmd 9; ok → cmd 8 [tid], post up to 128 logs |

## 4. Database (procedures not in the dumps)

| Procedure | Use |
| --- | --- |
| usp_RegisterGameEventLog(acc, wid/NULL, pcdbid/NULL, eventId, OUT logId) | handler 4/7 |
| usp_ProcessAccountGameEventLog(logId, acc, eventId) / usp_ProcessCharacterGameEventLog(logId, acc, wid, pcdbid, eventId) | returns ≤128 publications |
| usp_UpdateStatusToProcessFailure | process failed |
| usp_UpdateStatusToPublicationSuccess(pub, acc, couponEventId, acceptTid) | NP ok |
| usp_UpdateStatusToPublicationFailure(pub, acc, couponEventId) | NP rejected |
| usp_LoadGameEventLogList / usp_LoadCouponPublicationList | start-up reload |
| usp_LoadSingleCouponPublication(pub, acc) → couponEventId | manager retry |
| usp_LoadRetryReadyGameEventLogList | manager cmd 7 |

The Rust `MemoryStore` uses local rules (event id → coupon event ids). The real selection logic is inside the SQL ❓.

## 5. Events and EventNotiJob

| Event | Behaviour |
| --- | --- |
| GameEventLogEvent @0x14000B6E0 | process procedure; fail → UpdateStatusToProcessFailure; each publication → SendPublicationEvent (managementTid 0) |
| RetryCouponPublicationEvent @0x14000C2F0 | LoadSingleCouponPublication: fail → reply 6 reason 4; couponEventId 0 → reason 10; else SendPublicationEvent (managementTid = tid) |
| SendPublicationEvent @0x14000BE30 | EventNotiJob init needs account, couponEventId, publicationId ≠ 0; fail + tid > 0 → reason 2 |

EventNotiJob → NP (NPServiceChannel::SendRequestEventNoti @0x14000FF10):

- changeover on: COMMAND_SRV_REWARD_REQUEST_REWARD_FOR_EVENT, nRequestFromCode 9, userId = account, pszEventId = "%d" couponEventId, pszRequestKey = "%I64d" publicationId.
- changeover off: command 2851 with userId, eventId, publicationId.

| NP result | DB | managementReason |
| --- | --- | --- |
| no channel / send fail | — | 5 |
| status 200 (or 400 + "Success"), cmd 2851/2852, acceptTid = `_wtoi64(field 12)` | PublicationSuccess | 0 |
| error code 42, 5809, 5812, 30002 | none | 6 |
| other error code | PublicationFailure | 6 |
| expire (timeout value not read ❓) | — | 3 |

EventNotiJob::finalize @0x140009CC0: only manager-started jobs reply. Reason −1 → 1. Reason 0 →
SendRetryCouponPublicationOkEvent (cmd 5 [tid, acceptTid]); reason > 0 → SendRetryCouponPublicationFailEvent (cmd 6 [tid, reason]).

## 6. Reasons

| Value | Use |
| --- | --- |
| 1 | finalize, reason never set |
| 2 | decode fail, world not listed, post fail, job init fail |
| 3 | NP expired |
| 4 | DB load fail |
| 5 | NP not connected / send fail |
| 6 | register fail; NP rejected |
| 7 | invalid parameter |
| 8 | service not available; manager hello repeated |
| 9 | packet version |
| 10 | publication has no coupon event |
| 11 | IP validation |
| 13 | duplicate world session |

## 7. Status

| Part | State |
| --- | --- |
| Config parsers (3 files) | Ported |
| Daemon port handlers 1/4/7 | Ported |
| Manager port handlers 1/4/7 | Ported |
| Event pipeline + EventNotiJob outcomes | Ported (NP transport is a trait; `NoNp` = reason 5) |
| Start-up reload | Ported |
| MSSQL store | Not started (procedure bodies unknown) |
| NP transport (StsMsg/NP protocol) | Not started |
| Sol review | Pending (Sol quota) |

Tests: 22 (`tests/config.rs`, `handlers.rs`, `events.rs`, `sockets.rs`). Fixtures are synthetic.
