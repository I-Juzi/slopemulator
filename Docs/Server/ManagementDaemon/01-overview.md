# ManagementDaemon — overview

Source: `BnS_2020/ManagementDaemon/bin/` (`ManagementDaemon.exe`, `ManagementLib.dll`, `AdminLib.dll`), .NET assemblies, file date 2020-03-20, build path `Management-RNCS-0.0.356`. No PDB. The .NET metadata keeps all names. Decompiled with ILSpy 8.2 (`ilspycmd -p`). The decompiled source stays local (`private-project/decompiled/managementd/`). It is not in this repo.

Mark ❓ = not verified.

## Quick find

| Topic | Section |
| --- | --- |
| What it is | [Role](#role) |
| Ports | [Listeners](#listeners) |
| AdminWeb commands | [AdminWeb interface](#adminweb-interface) |
| Builder level | [Builder level](#builder-level) |
| Other interfaces | [Other interfaces](#other-interfaces) |

## Role

ManagementDaemon is the central operations hub of a server cluster.

```
 AdminWeb (GM web site, host BS-MON-01) ──21101──┐
 Management web (monitoring) ─────────────21100──┤
 Management client tool ──────────────────27501──┤
 Text console ────────────────────────────21105──┤
 External monitoring ─────────────────────21106──┤
 Tencent IDIP ────────────────────────────21103──┤
                                                 ▼
                                       ManagementDaemon ──► cluster DBs (direct writes)
                                                 │
        ┌──────────────┬────────────┬────────────┼──────────────┬──────────────┐
        ▼              ▼            ▼            ▼              ▼              ▼
 ManagementAgent  GameDaemon   LobbyDaemon   Market*Daemon  PostOffice,    NP / STS
 (port 21200,     (Gamed       (LobbyManage  (Dealer,       Ranking,       (port 6601)
 one per host)    packets)     packets)      Manager)       Achievement,
                                                            AccountInv,
                                                            SmartCoupon,
                                                            InfoGate
```

| Part | Function |
| --- | --- |
| ManagementAgent link | Start, stop, reload and cool down daemons. Query processes. Upload, extract and delete files (deploy). Performance counters. Version check. |
| Daemon links | Send operator commands to each server type. |
| Cluster DBs | Read and change game records directly. Example: `CreatureProperty.ModifyProperty`. |

## Listeners

Defaults from `DaemonConfig` constructor (`ManagementDaemon/DaemonConfig.cs`). The config file can change them.

| Name | Default port | Switch |
| --- | --- | --- |
| AgentPort (connect to agents) | 21200 | — |
| ClientPort | 27501 | — |
| ManagementWebPort | 21100 | — |
| AdminWebPort | 21101 | — |
| AlertLogPort | 21102 | `AlertMode` = `db` |
| IDIPListenPort | 21103 | `IDIPOn` = false |
| TextPort | 21105 | `UseText` = false |
| ExternalMonitoringPort | 21106 | `UseExternalMonitoring` = false |
| NPPort (connect) | 6601 | — |

Other defaults: `CertFilePath` `.\cert\management_managementd.cert`, `UseCert` false, `GameDataRootPath` `..\data`, `RepositoryPath` `.\repos-mdaemon`, `AgentFileName` `ManagementAgent.exe`.

## AdminWeb interface

- Session class: `ManagementDaemon.Network/AdminWebSession.cs`. Packets: namespace `ManagementDaemon.Network.AdminWebPacket`, 629 classes, 365 command names.
- Each request carries the operator identity: `AdAuthAccount`, `AdAuthName` (Active Directory account), `GmAddress` (IP). ManagementDaemon writes them to the log. ❓ ManagementDaemon itself does not check them. AdminWeb does the login.
- Command groups (from names): account and character records, character move, rename, race and sex change, items (add, change ability, appearance, gems), quests (add, change), achievements, guilds (banish, rename, leader, bank), auction and market (bid, abandon sale), post box, announce (game and lobby), chat spam filter, ranking and ranking black list, board gacha, Game TV schedule, Builder level, block character.
- AdminWeb `Web.config` repeats game feature flags (`use-builder-level-QA`, `use-white-list`, …). ❓ AdminWeb uses them to show or hide pages.

## Builder level

| Fact | Source | State |
| --- | --- | --- |
| The Builder level is a per-character DB field: `CreatureProperty.Builderright` | `AdminWebSession.handleChangeBuilderReqPacket` | ✅ |
| Request `ChangeBuilderReq`: `AdAuthAccount`, `AdAuthName`, `GmAddress`, `WorldId`, `PcName`, `PcDbId`, `GameAccountNo` (GUID), `BuilderLevel` (int16) | `ChangeBuilderReqPacket` | ✅ |
| Valid levels: -1 to 7. -1 is logged as "QA". Else fail `CHANGEBUILDERFAIL_INVALID_BUILDER_LEVEL`. | handler | ✅ |
| Flow: find world, find cluster, load `CreatureProperty` by `PcDbId`, set `Builderright`, `cluster.ModifyProperty`, send `ChangeBuilderAck(0)` | handler | ✅ |
| `LoadBuilderReq` / `LoadBuilderAck` read the level | packet names | ❓ fields |
| GameDaemon reads this field to allow Builder commands (card gmt-1) | — | ❓ |

## Other interfaces

| Group | Classes | Content |
| --- | --- | --- |
| GamedPacket | 25 | Hello, Announce, Kick, SendHome, CharInfo, guild operations, UnloadCache |
| LobbyManagePacket | 20 | Announce, Kick, CleanupGameAccountIdIndex, CleanupPcidIndex, RetryMileageDeposit |
| AgentPacket | 45 | Start/Stop/Query process, ChangeAndRestart, Reload/Cooldown daemon, file upload and extract, counters |
| ManagementWebPacket | 45 | Machine, cluster and daemon state, counters, alerts, arena info |
| ClientPacket | 50 | Same as ManagementWeb plus file transfer from agents |
| IDIPPacket | 117 | Tencent IDIP operations API: role info, bag, mail, money, kick, mask chat |
| NPPacket | 24 | NP / STS link: slots, external account, `AuthGetUserInfo` |
| TextPacket | 18 | Text console: daemon status, start, stop, reload |
| ExternalMonitoringPacket | 11 | Server base and detail info |
| Market, PostOffice, Ranking, Achievement, AccountInventory, SmartCoupon, InfoGate | 6–95 each | Operator commands per daemon |

`AdminLib.dll` (6 classes): `ExcelManagement.ImportTableFromExcel`, `SeasonContentsTypeData`, `SeasonContentsTypeDataTable`. ❓ Excel import is for operator data such as season tables, not for game design data.
