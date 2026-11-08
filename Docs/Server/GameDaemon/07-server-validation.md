# GameDaemon 381: server-side validation (port list)

Rule: port the checks that the original GameDaemon does itself. Do not invent new ones.
GameGuard (ncguardserver64.dll, csauth) is out of scope; see 04/06 boundaries.

Source: UTF-16 log strings in `BnS_2020/GameDaemon/bin/GameDaemon.exe` (build 381).
Method: `strings -e l`, then count by log tag. Functions and addresses: ❓ not yet mapped in IDA.

## Evidence by area

| Area | Log tag (count) | Checks seen in log text | Port priority |
| --- | --- | --- | --- |
| Packet parse | `[game packet]` (272), all handlers | "failed to decompose packet" (1201 strings over 374 `HandleGame_*` handlers); count limits ("invalid equip pos count ... max") | 1 — every handler |
| Zone / state | `[game packet]` | "cannot find zone" (280); state checks with `state:%d` (378) | 1 |
| Movement | `[move]` (101) | "fail to validate ... move-state, cell-index, client-req-z, origin-cell, next-cell"; "zone is not equal to caster's zone"; "cell-count / cell-max / duration / max-duration" input limits; "cannot move to start position"; "MoveAction MUST be used only by controlled creature" | 1 — with ZoneCellAttributeManager cell data |
| Skills | `[skill]` (292), `[skill-formula]` (47) | equip-pos and set-item count limits on UseSkillReq; creature state filter; train-req max level / max build-up | 2 |
| Cooldown | various (139) | recycle-time strings ❓ which ones apply to PC skills | 2 ❓ |
| Distance | 26 "too far" strings: `[inventory]` 8, `[move]` 4, `[trade]` 2, `[quest]` 2 | trade invite/join "guest too far"; item and quest interaction range | 2 |
| Trade / money | `[trade]` (144) | "invalid guest"; "invalid suggest money"; "receiver money overflow"; "invalid suggestion" | 2 |
| Inventory / items | `[inventory]` (1456), `[item]`, `[fielditem]`, closet / warehouse / post tran | 178 "overflow" strings; 143 "not enough / insufficient / lack" strings; item moves run as DB transactions (`*-tran`, `*-task`) | 3 — when items exist |
| Kick | `failProcess, kick, state:..., clientReason:%d` | kick on failed state step | 1 |
| Debug / GM | `HandleGameDebug_KickReq`, `Kick_AllReq` | GM kick commands | 3 |

## Design rules for our GameDaemon (from the evidence)

1. Every `HandleGame_*` decodes into a typed packet first. A decode failure logs and drops the packet (original: "failed to decompose packet").
2. Every handler checks the PC session state and zone before it acts.
3. Movement is cell-based: the server checks the requested cell path against zone cell data and move-state, not only a distance. ❓ exact rule; map `[move], fail to validate` in IDA first.
4. Range checks use the server position, never a client value.
5. Money and item changes go through one transaction path with overflow and "not enough" checks.
6. The client never sends a result value (damage, loot, reward). ❓ verify on UseSkillReq body.

## Next IDA work (one Sol job)

Map in GameDaemon.exe: the function that logs `[move], fail to validate` and its rule; UseSkillReq checks in order; the "too far" distance source (which table field); the kick path and `clientReason` values.
