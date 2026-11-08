# RankingDaemon (build 348)

Source: `RankingDaemon.exe` + PDB (IDA, names present). Config module `rankingd`.
Rust port: `Program/Server/RankingDaemon` (crate `rankingdaemon`). Phase 1.

RankingDaemon keeps the season list and the ranking boards. GameDaemons
send scores; InfoGates read ranking lists; an operator tool manages the
black list.

```
GameDaemon  --world port (12510)----> RankingDaemon <---manage port (32510)-- operator tool
InfoGate    --infogate port (12520)-->      |
                                        RankingDB (phase 2)
```

## 1. Ports

| Port | Option | Default | Peer | Packet version |
| --- | --- | --- | --- | --- |
| Game | world-listen-port | 12510 | GameDaemon (game-list) | 10 |
| InfoGate | infogate-listen-port | 12520 | InfoGate (infogate-list) | 6 |
| Manage | manage-listen-port | 32510 | operator tool | 1 |

- Bind order: manage, infogate, game.
- Frames: size i16 (4..=4000), cmd i16, body. Large replies use fragments
  (size, cmd, mark i32, data).

## 2. Config

### rankingd.config.xml

`RankingConfig::_Def` @0x1400E2410. `<config module='rankingd'><option name value/>`.

| Option | Kind | Default |
| --- | --- | --- |
| root-path, data-path, log-path, config-path, transaction-log-path, cert-file | REQ string | — |
| worker-thread | OPT | 0 |
| max-world | OPT | 100 |
| dump-type | OPT | 1 |
| timer-interval / timer-max-time | OPT | 1000 / 30 |
| service-mode | OPT, 1..11 | 1 |
| world-listen-port / infogate-listen-port / manage-listen-port | OPT | 12510 / 12520 / 32510 |
| service-world-id | REQ | — |
| max-ranking-list-count-per-contents | OPT | 10 |
| max-ranking-count | OPT | 100000 |
| max-search-count-one-time | OPT | 1000 |
| pc-list-hash-count | REQ | — |
| ranking-list-update-period | REQ | — |
| accumulate-season-interim-score-period | OPT | 600 |
| skip-list-ex-max-level / -prob / -max-node-count | OPT | 15 / 36 / 100000 |

### game-list.config.xml and infogate-list.config.xml

```xml
<game-list><game service-type='1' id='1' ip-address='10.0.0.5' internal-port='20100'/></game-list>
<infogate-list><infogate wid='1' ip-address='10.0.0.7' internal-port='22100'/></infogate-list>
```

- Every child must be `game` / `infogate`. Any other element fails the load.
- service-type: 1 world, 2 arena. Arena ids start at 10000 (ARENA_ID_BASE).
- A bad ip becomes 255.255.255.255 (inet_addr).

### SeasonContentsScheduleData.xml

Table `release-module="ItemData" release-side="ranking" type="season-contents-schedule" version="0.4"`.
Each `record` gives one season. Only records whose `service-mode` equals the
config service-mode stay.

| Enum | Values |
| --- | --- |
| season-contents-type | 0 invalid, 1 duel-death-match-1vs1, 2 dd-3vs3, 3 tag-3vs3, 4 occupation-war, 5 solo-dungeon-1, 6 infinite-tower, 7 capture-the-flag, 8 guild-combat, 9 solo-dungeon-2, 10/11 ingame-championship, 12 lead-the-ball, 13 challenge-party-best-record, 14/15 battle-royal |
| season-type | none, free, normal |
| player-type | none, pc, guild, challenge-party |
| service-mode | 0 none … 6 live-rc … 11 addition-2 |

- Times are `YYYY-MM-DD H:MM:SS`. The port reads them as UTC ❓.
- The exporter leaves out attributes at their default (`reward-group-data`
  is missing in 579 of 3012 records; ArenaLobby leaves out
  `season-reward-expiration-time`). The port reads a missing number or time
  as 0 and a missing name as index 0 ❓.
- The 348 enum @0x1400E15C0 has 16 contents types. The 2020 table also has
  `mastery-exp-best-record` (8 records). The port skips records with
  unknown names ❓ (native DrDataReader path not traced).
- Data size limit 64 MiB (the 2020 table is 1.9 MB). Check a file with
  `cargo run -p rankingdaemon --example check_seasons -- <file>`:
  RankingDaemon 2020 table → 3004 seasons; ArenaLobby → 2832.
- `convertSeasonContentsTypeToMaxRankingListIndex` @0x14001BAE0: types 1,
  3–7, 9–12, 14, 15 → 14; 8 → 2; 13 → 1; else 0.
- Result schedules come from `loadHiddenField` (phase 2; the list is empty now).

## 3. Game port

| Cmd | Name | Phase 1 result |
| --- | --- | --- |
| 1 | HelloReq `hhh` (cmd, version, clusterId) | see below |
| 4 | PingReq `hqh` | PingAck cmd 5, same body |
| 6 | RetrievePlayerRankingInfoReq | Fail cmd 8, reason 2 ❓ |
| 9 | RetrievePcRankingResultListReq | verify (worldId, playerId != 0) → Fail 11 reason 1, else 4 ❓ |
| 12 | LoadGuildRankingResultListReq | verify (clusterId != 0) → Fail 14 reason 1, else 4 ❓ |
| 15 | LoadPlayerSeasonResultReq | verify (clusterId != 0) → Fail 17 reason 1, else 4 ❓ |
| 18 | InsertPlayerRatingScoreReq | verify fail 6; unknown season 2; else 4 ❓ (no DB). Fail `hqQqhQciE` cmd 20 |
| 21 | InsertRankingScoreReq `hqqhQqqqciscqss` | same reasons. Fail `hciqqhQE` cmd 23 |
| 24 | DeletePlayerNotify `hQq` | add (playerType 1..2, playerId) to the black list; no reply |
| other | — | close |

Hello (gate: only Hello before auth; Hello after auth closes):

| Check | Reason (HelloFail cmd 3, `hh`) |
| --- | --- |
| Decompose fails, service unavailable, cluster not in game-list | 2 |
| version != 10 | 7 |
| peer ip != list ip | 9 |
| (type, id) already has a session | 11; the old session closes too |
| FetchSeasonList fails | 5 |
| OK | HelloAck as fragments, cmd 2 |

HelloAck body `CcischqqqhiCcqqqqQ;;CcischqqqhiCcqqqqQ;;`: active seasons,
then seasons in the result period. Season fields: type, schedule, name,
seasonType, playerType, start, end, rewardExpiration, maxIndex, defaultRating,
results `Cc qqqq Q;`.

## 4. InfoGate port

| Check | Reason |
| --- | --- |
| Decompose fails, service unavailable, wid not in list | 2 |
| version != 6 | 7 |
| ip mismatch | 9 |
| port mismatch | 10 |
| duplicate wid | 11 (old session closes) |
| OK | HelloAck cmd 2 `hCcc;`: active seasons {type, maxIndex + 1} |

Cmds 5, 8, 11 need a hello. Phase 1 replies Fail (7, 10, 13) with reason 2 ❓.

## 5. Manage port

| Cmd | Name | Result |
| --- | --- | --- |
| 1 | Hello `hhh` | decompose fail 1, version != 1 → 6; OK `h` cmd 2 |
| 4 | DeleteBlackList `hqqQ` (tid, playerId, playerType) | verify fail 1; missing 7 (Fail 6); OK `hq` cmd 5 |
| 7 | InsertBlackList `hqqQ` | verify fail 1; exists 8 (Fail 9); no active season 2 ❓; OK `hq` cmd 8 |
| other | — | close |

- Verify (shared by COMDAT folding): tid != 0, playerId != 0, 0 < playerType < 3.
- The default failReason is 1.

## 6. Phase 1 gaps

| Gap | Phase |
| --- | --- |
| RankingDB (insert, load, black list persist) | 2 |
| Ranking boards, skip list, update period | 2 |
| Result schedules (`loadHiddenField`) | 2 |
| InfoGate ranking list replies | 2 |

## 7. Rust port map

| Native | Rust |
| --- | --- |
| RankingConfig, list loaders, season data | `config.rs` |
| Commands, reasons, decode helpers | `protocol.rs` |
| Session managers, handlers, black list | `service.rs` (pure) |
| Ports | `server.rs` (tokio) |
| Tests | `tests/service.rs`, `tests/network.rs` |
