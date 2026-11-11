# Backlog

Items for later. They are not in the slice queue.

| Id | Item | Inputs | Notes |
| --- | --- | --- | --- |
| gmweb | Own GM web tool in Rust (axum), in place of NCSoft AdminWeb | `Docs/Server/ManagementDaemon/01-overview.md`; local decompile `private-project/decompiled/managementd/` (`AdminWebSession.cs`, 365 commands) | Speak the ManagementDaemon AdminWeb protocol (port 21101), or call our server directly. Start after the ManagementDaemon port. Scope: all 365 AdminWeb commands, plus extra features (search, audit log, Builder level page, live status). |

### gmweb: bans

| Feature | In AdminWeb | Source |
| --- | --- | --- |
| Block / unblock character | yes | `BlockCharacter`, `UnblockCharacter` |
| Kick (game, lobby) | yes | `Kick`, `KickToLobby` |
| Chat spam filter | yes | `ChatSpamFilter*` |
| Ranking black list | yes | `RankingBlackList` |
| White list (login) | yes | `WhiteList`, `InsertWhiteList` |
| Lock / unlock item | yes | `LockItem`, `UnlockItem` |
| Account ban | no ❓ (probably in NC account system / STS) | — |
| IP ban | no | — |
| Hardware ban | no (`pcid` = character id, not hardware) | — |
| Timed bans, ban reasons, ban history | no ❓ | — |

Add the missing rows in our tool. Account and IP bans go in our login server (StServers port).

