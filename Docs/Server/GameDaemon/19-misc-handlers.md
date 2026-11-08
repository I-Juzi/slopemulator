# 19 — Misc handlers (GameDaemon 381.8867)

Source: `ida/delegate/gamed/GameDaemon.exe` (build 381.8867), decompiled through
the IDA MCP. Handler table `0x1407D9F80`. Every address below is from this build.

Descriptor letters follow docs 16–18: `h` header or i16, `Q` u8, `q` u64,
`c` i8, `i` i32, `s` UTF-16 string with an i16 count, `E` u16 reason.

Coverage: 165, 356, 359, 360, 362, 407, 419, 820, 1130, 1137, 1188, 1423.
Quest commands are in docs 20 and 21.

## Shared fields

| Field | Source | Rust |
| --- | --- | --- |
| Rate stamp of the last bot report | PC qword +20800 | `Pc::bot_report_tick` |
| Reporter level | byte in the `0x14060D810` PC snapshot (❓ level, log says "current-level") | `Pc::level` |
| Minimum report level | daemon cfg +2744 → DWORD +1160 | `MiscWorld::bot_report_min_level` |
| Trace-route game log | daemon cfg +2736 → byte +537 | `MiscWorld::trace_route_log` |
| Slate amp page count | `qword_140EE58A0` DWORD index 523 | `MiscWorld::slate_amp_pages` |
| Trial play | PC byte +13768 = 1 | `Pc::t2_blocked` |
| Arena state | PC DWORD +37520 under the CS at +37480 | `Pc::arena_state` |
| PC lookup by ID / name | `0x1403F7D20` / `0x1403F7EF0` (daemon +5824, type 6) | `MentoringWorld::players` / `names` |

## 165 — G_StopCinematicReqCmd — partial

Handler `0x14038E560`.

1. Decode `hi`. The decode result is ignored: a short body leaves the i32 at 0.
2. No zone (PC +2224 = 0) → log, reply 167 `hE` [2].
3. Allocate task kind 205 (0x98 bytes, vtable `0x140BC2B48`). The i32 goes to +144. Byte +148 = 0.
4. PC native kind ≠ 1 → crash log, reply 167 [2].
5. Queue failure → reply 167 [2]. Success sends no reply here.

## 356 — G_DpGameClientToServerReqCmd — done

Target `0x14001AFE0` is a shared `return true` stub. No decode, no reply, no state.

## 359 — G_ReloadDaemonReqCmd — done

Same stub `0x14001AFE0`. No action.

## 360 — G_RegisterTraceReqCmd — done

Same stub `0x14001AFE0`. No action.

## 362 — G_UnregisterTraceReqCmd — done

Same stub `0x14001AFE0`. No action.

## 407 — G_WriteClientLogReqCmd — partial

Handler `0x14039C660`.

1. Decode `hs` (capacity 4000).
2. Failure → crash log, reply 409 `hE` [2].
3. Success → log level 10 with the PC name, PC ID, zone ID and the text. Reply 408 with an empty body (descriptor `h`, `asc_1408CFBE8`).

## 419 — G_ReportBotSpammerReqCmd — partial

Handler `0x14039CFC0`. Log tag `[report-bot-spammer]`. This is a player report
to the game log. It does not use the anti-bot helper `0x14060E280`.

1. `now = GetTickCount64() − server start`. If `PC+20800 + 2000 > now` → log "too frequent request". No reply.
2. Decode `hQqs`: type u8, target ID u64, target name (capacity 33). Failure → log, reply 421 `hEQ` [2224, type].
3. Type not 1 or 2 → log "invalid type value", reply 421 [2224, type].
4. Arena state 3 (arena lobby) and (target ID ≠ 0 or empty name) → log, reply 421 [2224, type].
5. Target ID ≠ 0 → look up by ID. Otherwise look up by name.
6. No target and state ≠ 3 → log "target pc not found", reply 421 [2226, type]. In state 3 the handler continues with only the name.
7. Snapshot the reporter (`0x14060D810`). If the configured minimum level > 0 and the reporter level < minimum → log "pc level check failed", reply 421 [2225, type].
8. Write the report game log (`0x14036C6A0`) with both snapshots and the world names.
9. Reply 420 `hQs` [type, target name]. The name is the request name in the lobby, else the target snapshot name.
10. Set PC+20800 = now. Only this path sets the stamp.

## 820 — G_TraceRouteIPReqCmd — partial

Handler `0x1403C1890`. Log tag `[gamelog]`.

1. Decode list `hCi;`: a count, then that many i32 values (❓ exact count framing; Rust uses an i16 count).
2. Count > 5 (unsigned, so negative also fails) → log "too many ip list", return false.
3. An element read fails → log "packet decompose fail", return false.
4. If cfg +2736 byte +537 ≠ 0 and PC native kind = 1 → game log `0x14036CF50` with up to five formatted IPs. Values ≤ 0 stay empty.
5. No reply.

## 1130 — G_ChallengeListsReqCmd — partial

Handler `0x1403D7AD0`. No decode.

1. No zone → log "failed to get zone", reply 1132 `hh` [2].
2. Task kind 333 (0x90 bytes, vtable `0x140C41208`). Queue failure → log "failed to initialize task", reply 1132 [2].
3. Success sends no reply here.

## 1137 — G_SecretTreasureExpirationTimeReqCmd — partial

Handler `0x1403D7E20`.

1. Decode `hh`: request ID i16. Failure → log, reply 1139 `hh` [2].
2. No zone → log, reply 1139 [2].
3. Task kind 320 (0x98 bytes, vtable `0x140B96730`). Request ID at +144.
4. Queue failure → log, reply **1028** `hE` [2]. The native code uses 1028 here, not 1139. Rust keeps it.

## 1188 — G_SetSlateAmpSlotReqCmd — partial

Handler `0x1403DAB70`. Task init `[SetSlateAmpSlotTask]`.

| Step | Check | Reply 1190 `hh` |
| --- | --- | --- |
| 1 | cfg +237 ≠ 0 and +248 ≠ 0, or arena state 2/3/5/6 | 5 |
| 2 | PC byte +13768 = 1 (trial play) | 7 |
| 3 | Slate amp page count ≤ 0 ("not enable") | 2537 |
| 4 | Decode `hcch` fails: page i8, position i8, order i16 | 2539 |
| 5 | No zone | 2 |
| 6 | Page not 1..2, or page > page count | 2539 |
| 7 | Position not 1..6 | 2539 |
| 8 | Order = 0. The DWORD high half is pre-set to 1, so a decoded order is the only variable | 2539 |
| 9 | Task kind 339 (0x98 bytes, vtable `0x140A81828`): page +144, position +145, DWORD +146 = order \| 0x10000. Queue failure | 2 |

Success sends no reply here.

## 1423 — G_ChallengeListTakeRewardReqCmd — partial

Handler `0x140385B20`. Task `[challengelist] ChallengeListTakeRewardTask`.

1. Decode `hc`: sub type i8 (default −1). The decode result is ignored.
2. No zone → reply 1425 `hh` [2], then log.
3. Task kind 223 (0x98 bytes, vtable `0x140CFE4B8`).
4. PC native kind ≠ 1 → log "caster is not PC", reply 1425 [2].
5. Sub type (as u8) > 1 → log "invalid challenge sub type", reply 1425 [2].
6. Sub type at +144. Queue failure → reply 1425 [2], log.
7. Success sends no reply here.

## Rust port status

| Command | Rust | Note |
| --- | --- | --- |
| 165, 407, 419, 820, 1130, 1137, 1188, 1423 | `src/handlers/misc.rs` | Task bodies and game-log writers are boundaries |
| 356, 359, 360, 362 | `src/handlers/misc.rs` | No-op, done |
