# RankingDaemon (build 348)

Source: `RankingDaemon.exe` + PDB (IDA, names present). Config module `rankingd`.
Rust port: `Program/Server/RankingDaemon` (crate `rankingdaemon`). Phase 1 + InfoGate handlers (phase 2a).

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

- Times are `YYYY-MM-DD H:MM:SS` in server **local time** (`_localtime64_s`
  in loadHiddenFieldFromSeasonContentsSchedule @0x140004480 and
  insertSeasonFromData @0x140029600). The port takes the offset from
  `RANKINGD_UTC_OFFSET_MINUTES` (default 0, KST = 540). A fixed offset has no
  DST ❓ (Korea has none).
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

#### Record layout (field table @0x1400E1D38, in order)

| Offset | Attribute | Kind |
| --- | --- | --- |
| +8 | service-mode | enum |
| +9 | season-contents-type | enum |
| +12 | season-contents-schedule-number | i32 |
| +16 | season-name | string |
| +25 / +26 | player-type / season-type | enum |
| +28 | default-rating-score | i32 |
| +32 | ranking-minimum-score | i32 (convert: types 8, 13, 15 keep it; others `<< 32`) |
| +40 / +48 / +56 | season-start-time / season-end-time / season-reward-expiration-time | time |
| +64 | closing-notice-period | i16 minutes |
| +66 | accumulation-period | i16 minutes |
| +68 / +69 / +70 | enable-weekly-ending-schedule / weekly-ending-day (sun = 0, table @0x1400E1560) / weekly-ending-hour | bool / enum / u8 |
| +72 | reward-group-data | i16 |
| +80 | enable-weekly-ending-schedule-for-qa | bool |
| +88 | weekly-ending-schedule-for-qa-1..20 | time[20] |
| +248 | weekly-ending-schedule-count-for-qa | count ❓ (port: filled entries from qa-1 up) |
| +249 | forced-termination | bool |
| +250 / +256 | season-result-schedule-count / season-result-schedule-1.. | hidden, derived |

#### Result times (loadHiddenField @0x140004480)

Only for records of the config service mode. `wday`, `hour` = local time of
the start. `D`, `H` = weekly-ending-day, weekly-ending-hour.

| Season type | Result times |
| --- | --- |
| normal, weekly flag | first ending: wday == D → start + 3600·(H − hour + 168); wday > D → start + 86400·(D − wday + 7) + 3600·(H − hour); wday < D → start + 3600·(H + 24·(D − wday) − hour), and first > end fails the load (log line 581). Then every 604800 s while < end, then end. |
| normal, no flag | [end] |
| free | [end]; result expiration (+56) becomes end + 3600 |
| other | validation failure, the load fails |

#### Result schedule rows (convertSeasonDataToRankingSeason @0x14002A7E0)

The QA list (+88, count +248) replaces the derived list when +80 is set.
For row i of n with reward time R[i] (acc = accumulation, notice = closing
notice, in seconds):

| Field | Value |
| --- | --- |
| number | i + 1 |
| rankingResultStartTime | i = 0: season start; else R[i − 1] |
| rankingClosingNotiTime | R[i] − acc − notice |
| rankingResultDetermineTime | R[i] − acc (not sent) |
| rankingRewardStartTime | R[i] |
| rankingResultExpirationTime | last: +56; else R[i + 1] − acc (GetWeeklyResultExpirationTime @0x140007AA0) |
| isFinalityResult | i + 1 == n |

- The season "current week" index is the row with start ≤ now < reward start.
- AddResultSchedule @0x140028A80 (the DB load path) uses the previous row's
  determine time as the start. The port uses the data path only.
- forced-termination: modifyActivatedSeason @0x14002B540 and
  modifyFutureSeason @0x14002B8C0 do not insert a forced record (an existing
  one ends with RankingDB::EndRankingSeason). The port drops forced records.
- 2020 table at KST: every service mode derives with no failure
  (`check_seasons <file> 540`; mode 2: 2272 rows, 51 forced).

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

Cmds 5, 8, 11 need a hello. Without a channel the handler returns false (close).
In all three, a decode failure only logs and returns true (no reply, keep).

| Cmd | Handler | Check | Reply |
| --- | --- | --- | --- |
| 5 RetrieveRankingListReq `hcc` | @0x14000E630 | RankingListManager null, or no list for (contentsType, index) in `_currentList` (key = type \| index << 8, @0x1400270A0) | Fail 7 `hccE` (type, index, 2) |
| 5 | | list found | the cached list packet, unchanged |
| 8 RetrieveTimeAttackBestRecordReq `hocciq` (tid, contentsType, job, rankingPerJob, guildMemberPcId) | @0x14000E9A0 | tid == 0, job <= 0, or (rankingPerJob <= 0 and pcId == 0) | Fail 10 `hoE` (tid, 15) |
| 8 | | contentsType != 6 | Fail 10, 15 |
| 8 | | rankingPerJob <= 0: PlayerManager::GetPlayer(1, pcId) missing | Fail 10, 17 |
| 8 | | rankingPerJob > 0: board manager null | Fail 10, 14 (Rust: never null) |
| 8 | | rankingPerJob > 0: GetRankingLogSummary fails | Fail 10, 16 |
| 8 | | OK | cmd 9 `hocciqss` (request fields, rankerName, bestRecord) |
| 11 RetrieveChallengePartyLogSummaryReq `hocq` (tid, contentsType, playerId) | @0x14000F620 | tid == 0, playerId <= 0, or contentsType != 13 | Fail 13 `hoE` (tid, 19) |
| 11 | | GetPlayer(3, playerId) missing | Fail 13, 20 |
| 11 | | OK | cmd 12 `hocqs` (tid, type, playerId, logSummary) |

- GetRankingLogSummary @0x14001AE60: contentsType 6 or 13, job 1..14, rank > 0.
  Board index = convertPcJobToBoardIndex(job) @0x14001BB60 (1→1, 2→2, 3→3,
  4→10, 5..10→job−1, 11..14→job, else 0). The board for (contentsType − 1)
  must exist. SkipListExManager::LookUp @0x140030A10 fails when index > maxIndex.
  A rank past the list size returns OK with a null value → 16.
- Names: wcsncpy 0x21 chars. Log summaries: wcsncpy 0x191 chars
  (Player::GetLogSummary @0x1400173C0; an unset summary stays empty).
  Rust keeps one char less than the buffer ❓.
- Player type 3 = challenge party ❓ (only seen in cmd 11).
- Rust: `Daemon::set_player`, `set_board_list` and `set_ranking_list` take the
  data. Phase 2b fills them from RankingDB and the boards.

## 5. Manage port

| Cmd | Name | Result |
| --- | --- | --- |
| 1 | Hello `hhh` | decompose fail 1, version != 1 → 6; OK `h` cmd 2 |
| 4 | DeleteBlackList `hqqQ` (tid, playerId, playerType) | verify fail 1; missing 7 (Fail 6); OK `hq` cmd 5 |
| 7 | InsertBlackList `hqqQ` | verify fail 1; exists 8 (Fail 9); no active season 2 ❓; OK `hq` cmd 8 |
| other | — | close |

- Verify (shared by COMDAT folding): tid != 0, playerId != 0, 0 < playerType < 3.
- The default failReason is 1.

## 6. Ranking board (SkipListExManager)

| Function | Address | Rule |
| --- | --- | --- |
| Init | 0x14002FDB0 | fail when maxLevel ≥ 16, prob > 100, maxNodeCount ≤ 0, maxIndex outside 1..=14; lists 0..=maxIndex |
| searchKey | 0x14002F7E0 | rank = 1 + nodes with a strictly greater key; missing key → 0 |
| Insert | 0x140030050 | index 1..=maxIndex; list 0 first, then list[index]; a failed second insert rolls back list 0 |
| insertNode | 0x140030E70 | full list: key < tail → rank 0, OK, not stored; key == tail → duplicate error; key > tail → evict tail of that list only |
| SkipListEx::InsertNode | 0x14002E7E0 | duplicate key → OVERLAPPED_KEY, rank 0 |
| Remove | 0x1400302D0 | completeSearchNode (list[index], then list 0); out of both lists; missing key is OK |
| Update | 0x140030410 | take the old node out (or make one), set the new key, insert into list 0 then list[index] |
| Search | 0x140030790 | ranks in list 0 and list[index] |
| LookUp | 0x140030A10 | index 0..=maxIndex, startRank > 0, count > 0; start past the end → OK, empty |

- Key = (key1, key2, key3) u64, compared in that order. A larger key ranks higher.
- The skip-list levels change speed only. The port keeps sorted vectors (`board.rs`).

## 6b. Phase 1 gaps

| Gap | Phase |
| --- | --- |
| RankingDB (insert, load, black list persist) | 2 |
| Ranking boards: RankingBoardManager score insert/update/delete on top of `board.rs`, update period | 2b |
| Result schedules from RankingDB (AddResultSchedule), current week index, season state timers | 2 |
| InfoGate ranking list composition (RankingListMaker, FetchRankingList) | 2b |

## 7. Rust port map

| Native | Rust |
| --- | --- |
| RankingConfig, list loaders, season data | `config.rs` |
| Commands, reasons, decode helpers | `protocol.rs` |
| Session managers, handlers, black list | `service.rs` (pure) |
| SkipListExManager | `board.rs` (pure) |
| Ports | `server.rs` (tokio) |
| Tests | `tests/service.rs`, `tests/network.rs`, `tests/board.rs` |
