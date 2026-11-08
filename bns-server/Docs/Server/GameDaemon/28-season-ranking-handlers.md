# 28 — Season and ranking handlers (GameDaemon 381.8867)

Source: `ida/delegate/gamed/GameDaemon.exe.i64`, build 381.8867.
All native addresses below are from this build. Handler table: `0x1407D9F80`.
The table slot for command 1213 is `0x1407DC568`. It points to `0x1403DD290`.
The wide logs at `0x140C3E620` and `0x140C3E690` name `HandleGame_PvpListReq`.

This document specifies entry functions. A queued task is not a completed operation.
The task bodies, persistence, account transactions and remote replies remain partial.
The store handlers (80, 827, 940 and others) are in [doc 22](22-shop-handlers.md) and `shop.rs`.

## Wire rules

| Letter | Width | Meaning |
| --- | --- | --- |
| `h` | 2 bytes | Command header, or short field |
| `Q`, `c` | 1 byte | Byte; use a signed comparison where stated |
| `i` | 4 bytes | Integer |
| `q` | 8 bytes | Integer or ID |
| `E` | 2 bytes | Reason |
| `C...;` | 2-byte count, then rows | Counted list |
| `s` | 2-byte count, then UTF-16 units | Counted string |
| `b` | 2-byte count, then bytes | Counted bytes |

Multi-byte fields use little-endian order. Reply bodies exclude the packet header.
`hh [reason]` and `hE [reason]` both have a two-byte body.
An `h` request has no body field. Dispatch supplies its header.
Trailing body bytes do not cause an entry failure unless stated.

Packet reader `0x14069F4B0` is variadic. Its old fixed-argument IDA type hid output
pointers and real branches. The corrected type and uncached pseudocode agree with
the disassembly. The byte decoder is `0x1406AA7B0`. For `b`, it rejects a negative
count, a count above capacity, or short data before it copies bytes.
Count reader `0x14069DA00` returns `0xFFFFFFFF` on failure. A signed short cast gives -1.

Verified descriptor strings include `hqChh;` at `0x140BCAE34`, `hcic` at
`0x140A66364`, `hhhChh;Chh;` at `0x140C2A650`, `hcQcibChh;` at `0x140C25808`,
`hccQibChh;` at `0x140C272A8`, `hcccChh;` at `0x140C284C8`, and
`hbCibqq;` at `0x140C20938`.

## Shared fields

| Field | Native source | Rust or boundary |
| --- | --- | --- |
| Zone | PC qword +2224 | `Pc::zone` |
| Caster ID | PC qword +40 | `Pc::object_id` |
| Native object kind | PC WORD +46; PC = 1 | `Pc::native_kind` |
| PC database ID | PC qword +20744 | `Pc::db_id` |
| User | PC +20696; unavailable byte +20752; user DWORD +120 | Existing PC fields and doc 22 checks |
| Arena form B | Daemon arena OR PC arena helper | `world.arena || session::arena_blocked(pc)` |
| Global arena only | cfg +2736 byte +237 and DWORD +248 | `World::arena`; no PC arena gate |
| Arena lobby state | PC DWORD +37520, lock +37480 | `Pc::arena_state == 3` |
| Trial play | PC byte +13768 | `Pc::t2_blocked`; not a substitute for WORD +12544 |
| Raid feature | rule cfg +2744 byte +1028 | `SeasonCheck::RaidEnabled` |
| Ranking feature | cfg +2736 byte +476 | `SeasonCheck::RankingEnabled` |
| Season manager | daemon qword +3192 | Explicit unsupported boundary |
| Raid manager | daemon qword +6128, helper `0x1401A51B0` | `SeasonCheck::RaidExists` |
| Raid properties | table `0x1407E5078..0x1407E509A`, reader `0x1406981E0` | `SeasonCheck::RaidProperty` |
| Unbind geo | zone +16224, record WORD +48 | `UnbindGeoPresent` / `UnbindGeoAllowed` |
| Unbind trial rule | global `0x140EE58A0` byte +2089 and PC WORD +12544 | `UnbindTrialAllowed` |
| Ranking interval | global signed DWORD +216 | Ranking cache boundary |
| Own ranking stamps | PC lock +20928; qword +20936 + 8 × type | Ranking cache boundary |
| Reward cache | PC lock +39608, map +39616, stamp +39632 | Ranking reward boundary |
| Guild ID | PC qword +336 | `Pc::guild_id`; `GuildExists` |
| Challenge-party ID | PC qword +800 | `ChallengePartyExists`; no guessed mapping to a team field |
| PC payment byte | PC lock +21072, byte +21080 | Existing `Pc::instant_payment` in doc 22; semantic name is a model label |
| Account inventory | cfg +2736 byte +404 | Existing `World::account_inventory_enabled` |
| PC lookup | helpers `0x1403F7D20` / `0x1403F7EF0`, mask 6 | Existing `world.mentoring` lookup methods; used inside native task bodies |
| Task kind / caster / attached | WORD +136 / qword +128 / byte +138 | `SkillTaskKind`, `SkillTask::caster`; attached = 1 |

Common task setup calls `0x14025DB30` to attach the zone.
Normal task queue: `0x140254FD0`. Action schedule: `0x140603BD0`.
Rust uses `skills::queue` and `Services::queue_skill` for verified task plans.
For command 105, the adapter must use action scheduling, not the normal native queue.
`Gate::Unknown` returns `Unsupported`. It never sends a fabricated failure or success.

## 105 — G_InitializeSeasonContentsInfoReqCmd — partial

Handler `0x140385D80`. Wire `hCc;`: signed i16 count, then one-byte types.

1. Zero a 17-byte local buffer. Read the count. Count ≤ 0 → reply 107 `hh` [3].
2. Read each byte. Ignore each reader return value. Type outside 1–16, or type 2 → 107 [3]. Disassembly confirms the reason argument.
3. Call `0x140612C10`. Under the arena lock, state 3 calls `0x140611EF0` directly.
4. Otherwise, no zone → 107 [2].
5. Allocate task kind 293, size 0x350, constructor `0x1402C0210`, vtable `0x140AE9A20`.
6. Recheck the types. Byte +144 = 1, byte +145 = count, bytes +146 = types. Attach caster and zone. Schedule through `0x140603BD0` with flag 0.
7. Allocation or initialization failure → 107 [2]. No entry success reply.

The entry has no upper count limit before it writes the local buffer. Rust returns
`Unsupported` for counts above 17. It does not reproduce the native overwrite.
The direct lobby helper reads active seasons and rating state. An empty result calls
`0x140611C80`, which builds reply 106 `hCcic;`.

❓ Missing: direct lobby state, rating conversion, DB load completion and full task body.

## 108 — G_SeasonContentsInfoReqCmd — partial

Handler `0x140385FD0`. Wire: no decode, no body fields.

1. No zone → reply 110 `hh` [2].
2. Allocate kind 245, size 0x90, vtable `0x140B19148`. No native-kind check.
3. Queue failure → 110 [2]. Success sends no entry reply.

Task executor `0x1402C04C0` builds reply 109. Its descriptor is
`hCcic;Cciiiiiiiii;Cciqhihis;` at `0x140A80CD0`.
It iterates types 1–16, except 2, then adds combat and time-attack records through
`0x1401B4CF0` and `0x1401B6420`.

❓ Missing: task-owned record lists and their field meanings; separate sync senders 111/112.

## 913 — G_RetrieveBoundRaidDungeonInfoReqCmd — partial

Handler `0x1403C5B10`. Wire `hq`: raid ID, 8 bytes.

1. Arena form B → reply 915 `hh` [5].
2. Raid feature disabled → 915 [2].
3. Decode failure → 915 [2]. No zone → 915 [2].
4. Allocate kind 163, size 0x98, vtable `0x140C3F6D0`.
5. Native kind ≠ 1 → 915 [2]. Copy raid ID to qword +144.
6. Queue failure → 915 [2]. No entry success reply.

Executor `0x1402A4200`: a nonzero raid ID must match a PC raid property, else
915 [2098]. A missing raid record gives 915 [2097]. A zero ID uses `0x140609CD0`
to collect bound raids. Reply 914 uses `hCqicqsii;` at `0x140A58BC8`.
Each row contains record +24 qword, data +8 i32, record +40 byte, record +48 qword,
owner name, record +44 i32, and cfg +1040 i32. Owner lookup uses rights mask 6.

❓ Missing: full task state and owner/team snapshot adapter. Rust does not execute the task body.

## 916 — G_UnbindRaidDungeonReqCmd — partial

Handler `0x1403C5E00`. Wire `hqChh;`: raid qword, signed i16 count, short pairs.

1. Arena form B → reply 918 `hh` [5]. Raid feature disabled → [2].
2. Raid decode failure → [2]. Raid manager record missing → [2099]. This lookup precedes the list count.
3. Count < 0 or ≥ 16 → [2]. Pair decode failure → [2].
4. No zone or no geo record → [2]. Geo WORD +48 = 177 → [2103].
5. Global byte +2089 ≠ 0 and PC WORD +12544 = 77 → [2104].
6. Allocate kind 161, size 0xE0, vtable `0x140B2B080`. Native kind ≠ 1 → [2].
7. Raid qword +144, count DWORD +152, four-byte pairs +156. Queue failure → [2]. No entry success reply.

Executor `0x1402A2990` rechecks the raid and ownership. It checks leave limits,
currency, items and conditional rule 131 state. It can send 918 reasons 2, 2099,
2100, 2101, 2102, 2105, 2106, 2107 or 2108. Its success path removes the binding,
updates properties, calls `0x140609F00`, and sends 917 `h` with an empty body.
These executor reasons are not entry reasons.

❓ Missing: task-owned inventory, limits, rule state and persistence. No task completion in Rust.

## 921 — G_RaidDungeonOnlinePlayerInfoReqCmd — partial

Handler `0x1403C6460`. Wire `hq`: raid ID.

1. Arena form B → reply 923 `hh` [5]. Raid feature disabled → [2].
2. Decode failure → [2].
3. No PC raid property equals the ID → [2110].
4. Raid record missing → [2109]. Otherwise call `0x1401A1D50`. No task.
5. That helper locks record +392 and builds reply 922 `hiCqsccc;` at `0x1409FBEA8`. It serializes nonzero players from the 88-byte rows at record +168. It passes each row ID and row +16 to `0x140017510`.

❓ Missing: leading i32 and all player row meanings. Rust stops before the snapshot.

## 924 — G_RaidDungeonExpirationReqCmd — partial

Handler `0x1403C66F0`. Wire: no decode.

1. Arena form B → log only. Do not send a failure reply.
2. Raid feature disabled → reply 926 `hh` [2]. No zone → [2].
3. Allocate kind 164, size 0x98, vtable `0x140B9BC40`.
4. Initializer `0x1402A4730` requires native kind 1. Set byte +144 = 1.
5. Initialization or queue failure → [2]. No entry success reply.

Executor `0x1402A47F0` calls `0x14060A100`. That helper sends 925 `h` when a bound
record expires or `0x1401A6190` returns 1. It clears PC +38712. It can update other
players through lookups with mask 6 and expiration tasks with mode 0.

❓ Missing: binding expiry state, peer propagation and sender 927.

## 980 — G_ReviveByAttractionRuleReqCmd — partial

Handler `0x1403CD1E0`. Wire `h`: no body fields.

1. Header decode failure → log only. Dispatch supplies the header in Rust.
2. No zone → log only.
3. Allocate kind 251, size 0x90, vtable `0x140B94548`.
4. Native kind ≠ 1, initialization failure or queue failure → log only.
5. No entry reply on any path.

❓ Missing: attraction revive task body and attraction-owned state.

## 985 — G_TakeAttractionRewardReqCmd — partial

Handler `0x1403CDA90`. Wire `hq`: 8-byte value, initialized to zero.

1. Decode failure → reply 987 `hh` [2]. No zone → [2].
2. Allocate kind 241, size 0x90, vtable `0x140BC1F28`.
3. Native kind ≠ 1 → [2]. The entry does not copy the decoded qword into the task.
4. Queue failure → [2]. No entry success reply.

Executor `0x1402BDC10` calls attraction virtual +256 through zone +16536.

❓ Missing: attraction implementation and reply senders 986/988.

## 989 — G_RetrieveSeasonContentsScheduleReqCmd — partial

Handler `0x1403CDCA0`. Wire: no decode.

1. Ranking feature disabled → reply 991 `hh` [2227].
2. Global arena only → [5]. Do not test the PC arena state.
3. Season manager missing → [2].
4. Lock manager +88. If cached packet +96 exists, send its payload through PC virtual +584. If absent, send nothing. No task.

❓ Missing: cached reply 990 construction and state adapter. Rust stops before step 3.
See [RankingDaemon](../RankingDaemon/README.md) for the remote season source.
That document covers build 348. It does not prove a build 381 reply body.

## 992 — G_RetrieveMyRankingInfoReqCmd — partial

Handler `0x1403CE510`. Wire `hc`: contents type byte.

1. Ranking disabled → reply 994 `hh` [2229]. Global arena only → [5].
2. Decode failure → [2]. Type 0 → [2230]. Type outside 1–16 → [2231].
3. Read the PC type timestamp under lock +20928. Require `interval < now - stamp`. Failure → [2231].
4. Find the active season in manager +3192, lock +72, table +80. Type 2 cannot find a season. Missing season → [2229].
5. Recheck the ranking feature. Forward RankingDaemon command 6 `hqQqci`: PC DB ID, player type 1, PC DB ID, contents byte, schedule i32. Send failure or feature disabled → [2229].
6. No task and no entry success reply.

❓ Missing: PC timestamp, active season, remote send result and reply 993.
Rust stops before step 3. See [RankingDaemon game port](../RankingDaemon/README.md#3-game-port).
Do not use that port's phase-1 fallback as a GameDaemon native reason.

## 995 — G_RetrieveGuildRankingInfoReqCmd — partial

Handler `0x1403CDF80`. Wire `hc`: contents type byte.

1. Ranking disabled → reply 997 `hh` [2232]. Global arena only → [5].
2. Decode failure → [2]. Type 0 → [2233].
3. Guild lookup for PC +336 fails → helper `0x1405DD220` sends 997 [1554].
4. Zero a 112-byte snapshot. Type other than 8 → reply 996 `hciqqisq` from this zero snapshot.
5. For type 8, lock guild +35176. Read snapshot +35064 and timestamp +35088. If `interval >= now - stamp`, send 996 from the cache.
6. Otherwise find the active season. Missing season → 997 [2232].
7. Recheck the feature. Forward RankingDaemon command 6 `hqQqci`: PC DB ID, player type 2, guild ID at guild +32408, contents, schedule. Send failure → 997 [2232]. No task.

Reply 996 body: contents byte, snapshot DWORD +8, qwords +16/+24, DWORD +32,
UTF-16 string from +36, qword +104. The zero snapshot body is 35 bytes, including
the contents byte and the empty string count. The descriptor is at `0x140A033F8`.

❓ Missing: guild cache adapter and remote send/completion. Rust supports steps 1–4.
See [RankingDaemon](../RankingDaemon/README.md#3-game-port).

## 998 — G_RetrieveRankingRewardListReqCmd — partial

Handler `0x1403CEA20`. Wire: no decode. Source: full disassembly; Hex-Rays fails.

1. Ranking disabled → reply 1000 `hh` [2229]. Global arena only → [5].
2. Read time and signed cfg DWORD +1572. If PC +39632 ≥ `now - interval`, call `0x1405DCDF0`.
3. Otherwise recheck the ranking feature. Forward RankingDaemon command 9 `hhq`: PC short +96, PC DB ID. Failure or disabled feature → 1000 [2229].
4. No task. The entry does not update the timestamp.

Helper `0x1405DCDF0` sends 999 `hCciciqq;` (string `0x140DA0DA8`).
It includes nonnull PC reward-map records with byte +24 = 0. Each row has the key's
high byte, i32 from key >>24, key byte 2, record DWORD +4, and qwords +8/+16.

❓ Missing: reward cache and remote completion. Rust stops before step 2.
See [RankingDaemon command 9](../RankingDaemon/README.md#3-game-port).

## 1001 — G_TakeRankingRewardReqCmd — partial

Handler `0x1403CEBC0`. Wire `hcic`: contents byte, schedule i32, result byte.

1. Ranking disabled → reply 1003 `hh` [2241]. Arena form B → [5].
2. Decode failure → [2]. Any field zero → [2242].
3. Look up the PC reward map under lock +39608. Key: `(result + ((schedule + ((i64)contents << 32)) << 8)) << 16`. Missing → [2242]. Record byte +24 ≠ 0 → [2244].
4. Season manager missing → [2]. Helper `0x1401AEB30` failure or both output flags false → [2245].
5. DataManager slot 2235 has no reward group for record signed short +0 → [2].
6. Find the rank bracket. Count byte +336; lower bounds DWORD +16 +4×i; upper bounds DWORD +96 +4×i; rank record DWORD +4. No bracket → [2243].
7. Reward qword +176 +8×i = 0 → [2]. No zone → [2].
8. Allocate kind 242, size 0x170, constructor `0x1402AD760`, vtable `0x140AEE2D8`.
9. Native kind ≠ 1, signed contents ≤ 0, or signed low reward DWORD ≤ 0 → [2].
10. Set DWORD +144 = 1; contents byte +148; schedule DWORD +152; result byte +156; group WORD +158; rank DWORD +164; flag B byte +168; bracket DWORD +172; reward DWORD +176; flag A byte +188.
11. Attach and schedule through `0x140603BD0`. Failure helper `0x1405DD290` sends 1003 with the reason. No entry success reply.

❓ Missing: reward records, eligibility, reward groups, task body and replies 1002/1006/1007.
Rust stops before step 3. It does not approve a reward from request data.

## 1059 — G_PublicRaidEventAddScoreReqCmd — partial

Handler `0x1403D20B0`. Wire `h`: no body fields.

1. Header decode failure → reply 1061 `hh` [2370]. Rust dispatch supplies the header.
2. No zone → [2].
3. Allocate kind 289, size 0x90, vtable `0x140BB18F8`. Native kind ≠ 1 → [2].
4. Queue failure → [2]. No entry success reply.

Executor `0x1402C9370` sends 1061 reasons 2371, 2, 2372, 2374, 2373, 2375, 2376
or 2377 for its faction, table, attraction and score-state failures. It sends 1060
`h` with an empty body after a successful score change.

❓ Missing: task-owned score state and attraction virtual operations. Rust queues only.

## 1088 — G_RetrieveChallengePartyRankingInfoReqCmd — partial

Handler `0x1403D3F20`. Wire `hc`: contents byte.

1. Ranking disabled → reply 1090 `hh` [2235]. Global arena only → [5].
2. Decode failure → [2]. Contents ≠ 13 → [2236].
3. Lookup PC qword +800 in daemon manager +6184 with `0x1400673C0`. Missing → [2238].
4. Lock party +24. Snapshot +2264 and timestamp +2288 use the global ranking interval. Fresh cache → reply 1089 `hciqqisq`, with the same field widths as 996.
5. Otherwise find the active season. Missing season or send failure → 1090 [2239].
6. Forward RankingDaemon command 6 `hqQqci`: PC DB ID, player type 3, challenge-party ID, contents 13, schedule. No task.

❓ Missing: party ranking cache, active season, remote send/completion and notify 1091.
Rust stops before step 4. See [RankingDaemon](../RankingDaemon/README.md#3-game-port).

## 1134 — G_SetPvPModeReqCmd — partial

Handler `0x1403D7C20`. Wire `hQ`, descriptor `0x1408E0DA4`: one byte, not a qword.

1. Decode failure → reply 1136 `hh` [2]. No zone → [2].
2. Allocate kind 321, size 0x98, vtable `0x140C3FB08`. No native-kind gate.
3. Byte +144 = request byte ≠ 0. Queue failure → [2]. No entry success reply.

Executor `0x1402D25E0` checks zone permission, current mode and a time rule.
It sends 1136 [2] or [2464] on failure. Success sends 1135 `hQ`, one boolean byte.

❓ Missing: task-owned mode state, permission and time rule.

## 1213 — G_PvpResultListReqCmd — partial

Handler `0x1403DD290`. Wire: no decode.

1. No zone → reply 1215 `hh` [2].
2. Allocate kind 364, size 0x90, vtable `0x140BADAF0`. No native-kind gate.
3. Queue failure → [2]. No entry success reply.

Executor `0x1402DC3D0` calls `0x14000D090` on PC +20640. That helper sends 1214
`hiCqchqcss;` (descriptor `0x1407FBF20`). It serializes nonnull records from a map
of lists through `0x14000CA60`. Each row starts with record qword +16, then six
property fields from record +24. Packet construction can use 4000-byte fragments.

❓ Missing: leading i32, record property meanings and fragment transport adapter.

## Related replies and notifications

This table covers the requested outbound IDs. A descriptor without a verified sender
is not inferred from its command name. `❓` marks a sender or body not yet read.
The new Rust entries do not synthesize task or remote success replies.

| ID | Command | Verified descriptor and body / source |
| --- | --- | --- |
| 106 | G_InitializeSeasonContentsInfoOkCmd | `hCcic;`: list {byte, i32, byte}; `0x140611C80`; schedule/rating state remains partial |
| 107 | G_InitializeSeasonContentsInfoFailCmd | `hh`: reason 3 or 2; entry/helper above |
| 109 | G_SeasonContentsInfoNotifyCmd | `hCcic;Cciiiiiiiii;Cciqhihis;`; `0x1402C04C0`; three counted lists |
| 110 | G_SeasonContentsInfoFailCmd | `hh`: reason 2 |
| 111 | G_SeasonContentsCombatRecordSyncCmd | ❓ Sender and body not read |
| 112 | G_SeasonContentsTimeAttackRecordSyncCmd | ❓ Sender and body not read |
| 914 | G_RetrieveBoundRaidDungeonInfoOkCmd | `hCqicqsii;`; rows and source under 913 |
| 915 | G_RetrieveBoundRaidDungeonInfoFailCmd | `hh`: entry 5/2; task 2098/2097 |
| 917 | G_UnbindRaidDungeonOkCmd | `h`: empty; executor `0x1402A2990` |
| 918 | G_UnbindRaidDungeonFailCmd | `hh`: entry 5/2/2099/2103/2104; task reasons under 916 |
| 919 | G_OccupyingRaidDungeonChangeNotifyCmd | ❓ Sender and body not read |
| 920 | G_UnoccupyingRaidDungeonNotifyCmd | `hq`: raid record +24; `0x140609F00`, when old occupied raid owner has native type 1 |
| 922 | G_RaidDungeonOnlinePlayerInfoOkCmd | `hiCqsccc;`; `0x1401A1D50`; leading i32 and row meanings ❓ |
| 923 | G_RaidDungeonOnlinePlayerInfoFailCmd | `hh`: 5/2/2110/2109 |
| 925 | G_RaidDungeonExpirationOkCmd | `h`: empty; `0x14060A100` |
| 926 | G_RaidDungeonExpirationFailCmd | `hh`: 2; arena entry sends nothing |
| 927 | G_RaidDungeonExceedMaxLeaveCountNotifyCmd | ❓ Sender and body not read |
| 986 | G_TakeAttractionRewardOkCmd | ❓ Attraction virtual sender and body |
| 987 | G_TakeAttractionRewardFailCmd | Entry `hh`: 2; ❓ attraction-specific reasons |
| 988 | G_TakeAttractionRewardNotifyCmd | ❓ Attraction virtual sender and body |
| 990 | G_RetrieveSeasonContentsScheduleOkCmd | Cached packet send at `0x1403CDCA0`; ❓ construction/body |
| 991 | G_RetrieveSeasonContentsScheduleFailCmd | `hh`: 2227/5/2 |
| 993 | G_RetrieveMyRankingInfoOkCmd | ❓ RankingDaemon completion sender/body |
| 994 | G_RetrieveMyRankingInfoFailCmd | `hh`: 2229/5/2/2230/2231 |
| 996 | G_RetrieveGuildRankingInfoOkCmd | `hciqqisq`; body under 995 |
| 997 | G_RetrieveGuildRankingInfoFailCmd | `hh`: 2232/5/2/2233/1554 |
| 999 | G_RetrieveRankingRewardListOkCmd | `hCciciqq;`; body under 998 |
| 1000 | G_RetrieveRankingRewardListFailCmd | `hh`: 2229/5 |
| 1002 | G_TakeRankingRewardOkCmd | ❓ Reward-task sender/body |
| 1003 | G_TakeRankingRewardFailCmd | `hh`: reasons under 1001 |
| 1004 | G_ChangedRankingInfoNotifyCmd | ❓ Ranking completion sender/body |
| 1005 | G_ChangedGuildRankingInfoNotifyCmd | ❓ Guild ranking completion sender/body |
| 1006 | G_TakeRankingRewardForGuildBuffNotifyCmd | ❓ Reward-task sender/body |
| 1007 | G_TakeRankingRewardForGuildBankNotifyCmd | ❓ Reward-task sender/body |
| 1060 | G_PublicRaidEventAddScoreOkCmd | `h`: empty; `0x1402C9370` |
| 1061 | G_PublicRaidEventAddScoreFailCmd | `hh`: entry 2370/2; task reasons under 1059 |
| 1089 | G_RetrieveChallengePartyRankingInfoOkCmd | `hciqqisq`; snapshot send at `0x1403D3F20` |
| 1090 | G_RetrieveChallengePartyRankingInfoFailCmd | `hh`: 2235/5/2/2236/2238/2239 |
| 1091 | G_ChangedChallengePartyRankingInfoNotifyCmd | ❓ Ranking completion sender/body |
| 1135 | G_SetPvPModeOkCmd | `hQ`: boolean byte; `0x1402D25E0` |
| 1136 | G_SetPvPModeFailCmd | `hh`: entry 2; task 2/2464 |
| 1214 | G_PvpResultListOkCmd | `hiCqchqcss;`; `0x14000D090`; record meanings ❓ |
| 1215 | G_PvpResultListFailCmd | `hh`: 2 |

Command 108 is an inbound request, not an extra reply between 106 and 112.
Commands 916/921/924 and 989/992/995/998/1001/1088/1134/1213 have request headings
above. The outbound table does not assign bodies to inbound commands.

## Rust port status

| IDs | State | Rust behavior / remaining boundary |
| --- | --- | --- |
| 105 | partial | Decode, safe count boundary, type gates, normal-world action task; lobby state unsupported |
| 108, 913, 916, 924, 980, 985, 1059, 1134, 1213 | partial | Entry gates and typed `SeasonTask`; no task execution |
| 921 | partial | Entry checks; raid-player snapshot unsupported |
| 989, 992, 998, 1001 | partial | Verified early gates; caches, schedules and remote state unsupported |
| 995 | partial | Early gates, guild lookup, exact zero-snapshot reply; cache and remote state unsupported |
| 1088 | partial | Early gates and party lookup; cache and remote state unsupported |

`SEASON_COMMANDS` registers all 31 IDs before `SHOP_COMMANDS` dispatch.
Each reused store command still calls the same doc 22 function.
`Services::season_check` defaults to Unknown. `MemoryServices::season` records calls
and has an explicit result map. An absent result is Unknown.
Tests: `Program/Server/GameDaemon/tests/gk1_season.rs`.
