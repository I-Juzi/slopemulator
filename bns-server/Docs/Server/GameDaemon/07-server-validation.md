# GameDaemon 381: server-side validation (port list)

Rule: port the checks that the original GameDaemon does itself. Do not invent new ones.
GameGuard (ncguardserver64.dll, csauth) is out of scope; see 04/06 boundaries.

Source: UTF-16 log strings in `BnS_2020/GameDaemon/bin/GameDaemon.exe` (build 381).
Initial method: `strings -e l`, then count by log tag.
Step 5 extends this evidence with UTF-16 string xrefs and static decompilation in the allowed GameDaemon 381 database.
All addresses below are in GameDaemon 381. Native object offsets are not wire offsets.
No live acceptance test ran.

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
| Kick | `failProcess, kick, state:..., clientReason:%d` | BoardGachaDraw transaction failure; conditional user-manager kick path. See part 4. Not a generic movement rejection rule. | 1 |
| Debug / GM | `HandleGameDebug_KickReq`, `Kick_AllReq` | GM kick commands | 3 |

## Design rules for our GameDaemon (from the evidence)

These were initial port hypotheses, not universal verified rules.

1. Decode each checked request before use. Failure handling is handler-specific. Walk2 sends a failure reply; it does not only drop the request.
2. Check state and zone as specified for each recovered path. ❓ A universal rule for every handler is not verified.
3. Movement checks cell data and server move state. The exact recovered subset is below.
4. ❓ Verify each range check's position source. Do not apply a universal claim from log text alone.
5. ❓ Inventory transaction, overflow and balance rules remain outside this pass's proof.
6. ❓ A universal claim that clients never send results is not proved. Check UseSkillReq separately below.

## Step 5, part 1 — Movement (partial)

### Entry points and inputs

| Role | Address | Verified use |
| --- | --- | --- |
| Walk2 request handler | `0x1403892E0` | Parses the request. Resolves the PC or controlled summoned creature. Requires a zone. |
| PC move initializer | `0x14048A1E0` | Copies decoded fields into a move action. |
| PC move action start | `0x14048A890` | Contains log xrefs `0x14048B922` and `0x14048BE83` to UTF-16 `[move], fail to validate` at `0x140CDAED0`. |
| Move-state transition helper | `0x14048C0B0` | Runs before height validation at each checked cell step. ❓ Full transition matrix not mapped. |
| Height/state validator | `0x14048CDB0` | Uses server creature move state +2300. Returns a failure reason or zero. |
| Summoned counterpart | `0x140490430`, `0x140490A80` | Same log family. ❓ Full summoned rules not mapped. |

Decoded inputs passed by Walk2 include: controlled-creature selector; start and end x/y/z shorts; two integer triples; duration; yaw; transition counter; move-type records; cell x/y/z records; and a flag.
The initializer stores start at action +152/+154/+156 and end at +158/+160/+162.
It stores integer triples at +164..+184, duration at +536, yaw at +188 and counter at +190.
Move-type records have a 10-byte native stride. Cell records have a 6-byte native stride.
❓ Full wire format, integer-triple units and final flag meaning remain open.
The handler selects the summoned object only when the selector is 1 and its control state +6984 is 1 or 2.
A selected creature of native kind 2 with action byte +56 equal to 11 gets failure reason 166.

### Input limits and early checks

1. Require the supplied zone to equal creature +2224.
2. Require creature native kind +46 in 1..3 in the PC initializer.
3. Require cell count in 1..36 and a non-null cell pointer.
4. Require unsigned duration <=550. Zero is allowed here.
5. Require move-type count in 1..12.
6. For a PC, call duration-history check `0x1405D87D0` before action initialization.
7. Queue the initialized action through `0x140254FD0`.

All count and duration maxima above are code constants, not table fields.
Walk2 first rejects decoded move-type count >12 and declared cell count >36.
Its cell loop also rejects when the incremented parsed count reaches 36.
Thus the checked handler admits at most 35 parsed cell records, although its declared-count bound and initializer allow 36.
It then requires parsed count == declared count.
Do not silently replace these distinct native bounds with one inferred limit.
Parse/create/initialize/queue failures use `0x14038BE00`: command 65, format `hEchhhh`, reason 2, and zero coordinate/counter fields.
Too many move types uses reason 103. Cell-count failures use reason 102.

### Cell and start-position pseudo-steps

Source: `0x14048A890`, `0x14048F620`, `0x14048F810`, `0x14048DE30`.

1. Require successive cell indices to be cardinal neighbours: `(abs(dx),abs(dy))` is `(1,0)` or `(0,1)`. Duplicates and diagonals fail with reason 117.
2. Require every cell x and y index >=0. Otherwise return 118.
3. Accept move-type codes only in 0..18, 22..36 or 40..42. Otherwise return 119.
4. Require each move-type start-cell byte in 0..cell-count, inclusive. Otherwise return 120.
5. Require start-time byte >=0 and `5 * start-time <= duration`. Otherwise return 121.
6. Require nondecreasing start-cell bytes and start-time bytes. Otherwise return 122 or 123.
7. Require the request transition counter to equal creature +3480. Otherwise send reason 104 without incrementing that counter.
8. Check the start position against the first cell and the end position against the last cell with `0x14048DE30`.
9. Compute x/y cell indices as signed native division: `(position - geometry-origin) / 8`. Require equality with the requested cell x/y.
10. Resolve the requested cell layer at `[cell-z, cell-z+1)`. Require its environment to be zero or a valid, enabled zone environment.
11. If the exact layer fails, try `[cell-z-10, cell-z+10)`. The helper can adjust the cell z and a matching position z to the returned floor. Re-query the adjusted layer.
12. Require position z in the returned cell's inclusive floor/ceiling range.
13. Compare request start with server creature position +268/+270/+272. Require each x/y delta <= creature signed-short +2304 plus 5. Require absolute z delta <=10.
14. Run the speed check below.
15. If duration is zero, require identical start and end x/y/z.
16. Resolve the first cell through the zone geometry object. For each later cell, resolve its layer and environment. Reject overlapping consecutive vertical intervals when `origin.floor >= next.ceiling && origin.ceiling <= next.floor`.
17. Run move-type transitions, then the height/state validator, per step. For one cell, validate start z to end z in that cell. For multiple cells, validate the initial state, then each origin/next pair.
18. Continue the move action only after these checks pass.

Cell source: zone +16224 is the geometry-cell object. `0x1400DD7D0` checks x/y bounds and calls `0x1400D9E80` through geometry +432.
Geometry +440 adds per-cell attribute byte +10 when present; absent attributes yield zero.
Zone environments use zone +16336. The move validator uses returned attribute byte +11 for ground/water state.
`ZoneCellAttributeManager` is named by UTF-16 source-path/log strings at `0x140B6E4F0` and `0x140B6E8B0`.
❓ The manager-to-geometry loading chain, file format and exact attribute-field names were not traced. Do not substitute a flat distance check or invented cell flags.

### Speed and duration sources

Source: `0x14048E6C0`.
Let `D` be duration, `dx=max(abs(end.x-start.x)-1,0)` and `dy=max(abs(end.y-start.y)-1,0)`.
The helper selects a speed per move-type record. It uses the maximum selected speed, then the prior server speed at creature +3904 if larger.
It clears +3904 when its stored age exceeds 500, but still uses the previously read value for this call.

| Move types | Selected speed source |
| --- | --- |
| 0, 1 | Minimum of creature signed-short +1008 and network-config +548. A nonzero creature +3449/+3450 selector can replace this with selected-record signed-short +24. ❓ Table name open. |
| 2..4; type 14 with flag byte zero | Creature signed-short +1008 |
| 5..10, 11..13, 34..36; type 14 with flag byte nonzero | Constant 109 |
| 15 | `50 * creature.speed / 100` |
| 16..18 | Constant 79 |
| 22..24 | Constant 100 |
| 25..27 | `150 * creature.speed / 100` |
| 28..33, 40..42 | Constant 94 |

Add network-config +556 if any type 0/1 occurs; otherwise add +552. Call the sum `S`.
For `D>0`, reject when `(S*D)^2 < 1000000*(dx^2+dy^2)`.
If config byte +1497 is enabled and end.z > start.z, reject when `(config[1500]*D)^2 < 1000000*(end.z-start.z)^2`.
The separate integer-triple speed comparison against `4 * selected-speed` records telemetry; it does not reject in this helper.
❓ Exact configuration field names, selected-record table name and property derivation remain open. The values above are offsets or code constants, not copied game records.

Duration-history check `0x1405D87D0` uses an optional server ring at PC +3776.
It adds request duration to the rolling duration total and compares it with elapsed server ticks plus network-config +560.
An excess returns false only when config byte +564 is enabled; otherwise it records the excess and continues.
❓ Ring initialization and configured units remain open. No vendor anti-cheat call was followed.

### Height/state rules ready to port

Source: `0x14048CDB0`. Require native creature kind 1; otherwise return 100.
Let `F0=origin.floor`, `F1=next.floor`, `Z0=client-origin-z`, `Z1=client-next-z`, `H=Z1-F1`, and `delta=F1-F0`.
These are native signed-short heights. Move state comes from server creature +2300, not directly from a packet result.

| Server state | Exact checked rule / action |
| --- | --- |
| 0 (run from log) | If H>11, change to state 2 at Z0. Else require delta<=20 (reason 112), delta>=-24 (100), and next attribute +11 ==0 (144). |
| 1 (jump) | If Z1<=creature.s16[2256]+global `0x140E7BCF8`, update jump tracking or end-jump state. Else, if H<=11, arrive on ground (state 0) or water when attribute==1 (state 8). Otherwise return 113. ❓ Global initialization/value open. |
| 2 (fall) | Move type 14 bypasses these height tests. Otherwise require Z1-Z0>=-1000 (115). If rising and H>16, helper `0x14048BF70` must accept; success changes to state 3, failure returns 114. If H<10, arrive on floor, state 0 or 8 by nonzero water attribute. ❓ Fallback helper's exact rule open. |
| 3 | If H<11, change to state 0 at F1. |
| 4 (sprint) | If H<=22, require delta in [-24,20] and attribute +11 ==0; reasons 100/112/144. If H>22, require Z1<=Z0 (100). |
| 5 (sprint jump) | Require Z1<=creature.s16[2256]+135; else 113. |
| 6, 8 (drift/swim from log) | If H>11, change to state 2 at Z0. Else require delta in [-15,15] (100/112) and attribute +11 ==1 (138). |
| 7 (glide) | Require Z1<=creature.i32[2288]+25; else 146. |
| 9 (boost) | Require Z1<=creature.i32[2292]+5; else 153. The log prints 25, but code compares +5. |
| 10 (dive) | Require Z1<=creature.i32[2296]; else 154. |
| 11, 12 | Return success in this validator. This does not bypass other helpers. |
| Other states | Assert/log, then return zero in the checked helper. Do not invent a reject here. |

### Failure result and remaining movement gaps

`0x14048DC90` sends command 65, full format `hEchhhh`: reason, server movement byte, server x/y/z and transition counter.
Its position fields come from creature +268/+270/+272, not request coordinates.
The checked validation callers request counter increment, then end the action through `0x140422850`.
Start/end cell mismatch, start-distance failure, speed failure and zero-duration displacement also send correction from server position.
No direct session kick was found in these checked movement failure branches.
❓ Complete move-type transition rules (`0x14048C0B0` and its callees), delayed action checks, all wire fields, summoned rules and client reason names remain open.
This is an exact subset, not a complete movement implementation contract.

## Step 5, part 2 — UseSkillReq (partial)

### Request parsing and action creation

UTF-16 handler logs at `0x140BC7180`..`0x140BC8680` map to `HandleGame_UseSkillReq`, `0x140373330`.
The checked full descriptor at `0x140BC7818` is `hcicqhcCc;Cq;Cc;Cc;`.
The leading `h` is the command. `C...;` denotes a counted group in this native descriptor.

| Body order | Type | Use |
| --- | --- | --- |
| 1 | c | Controlled-creature selector |
| 2 | i | Skill ID |
| 3 | c | Variation ID |
| 4 | q | Requested target object ID; copied to action +192 |
| 5 | h | Request short; copied to action +234. ❓ Semantic name not verified. |
| 6 | c | Item/equip selector; copied to action +636. Initial local value is -1. ❓ Complete meaning open. |
| 7 | C[c] | Equip positions for skill modification; count 0..8 |
| 8 | C[q] | Set-item keys for skill modification; count 0..38 |
| 9 | C[c] | Item-skill equip positions; count 0..4 |
| 10 | C[c] | Badge equip positions for skill modification; count 0..26 |

All four maxima are code constants. Negative counts fail too.
The handler checks the groups in table order and decodes every element.
This request sends intent and item-modifier selectors, not a damage amount, hit result, loot result or reward amount.
This conclusion applies to this descriptor only. ❓ It does not establish the contents of all client requests.
The q set-item keys are identifiers, not computed combat results.

Pseudo-steps in handler order:

1. Decode the six scalar fields. On failure, send command 1227 with reason 1 and zero selector/skill/variation.
2. Decode and bound the four counted groups in the order above.
3. Select the PC, or its controlled summoned creature for selector 1. The summoned control state +6984 must be 1 or 2.
4. Require a non-null creature and zone at creature +2224.
5. Form the skill key from skill ID and variation.
6. Call factory `0x1404BE5A0`. It resolves the skill table and chooses the action class from record byte +214. It refuses record kind +2 in 1..2 and unsupported action classes.
7. Initialize the action with `0x1404988A0`. Require non-null zone/caster/key and equality with caster +2224.
8. Deduplicate each supplied modifier group inside the action. Repeat its maximum-count checks.
9. Run data initialization `0x1404978D0`.
10. Run helper `0x14049A550` and queue through `0x140254FD0`.

Handler group/control/zone/create/initialize/queue failures send command 1227, format `hEcic`: reason 1, selector, skill ID and variation.
Data initialization can also send a more specific failure before the handler sends its generic failure.
No direct kick is shown in these branches.

### Data dependencies

Source: `0x1404978D0`. The table-manager slot numbers below are qword indices from `qword_140F77978`.
They identify the exact binary dependency without inventing XML field names.

| Action field | Lookup source | Verified purpose |
| --- | --- | --- |
| +288 | Manager slot 2355, requested skill key | Skill record |
| +296 | Slot 2403, skill qword +400 | ❓ Auxiliary skill-rule table; exact table/field name open |
| +304 | Slot 2427, skill qword +224 | Dash attribute, from failure log at `0x140CE1960` |
| +312 | Slot 2535, skill qword +336 | Result-control record, from log at `0x140CE2090` |
| +320 | Slot 2451, skill qword +248 | Gather-range record, from log at `0x140CE1FF0` |
| +328 | Slot 2571, skill qword +240 | Target-filter record; missing record logs but does not return failure at this site |
| +352 | Slot 2415, skill qword +832 | ❓ Table name/meaning open |
| State filter | Slot 1131, filter key | Creature-state filter; `0x140496090` resolves its evaluator through manager slot 3344 |

Missing skill, auxiliary rule, dash, result-control or gather-range record fails initialization.
For PC-like kinds, the checked missing-data reply is command 1227 with reason 180.
Skill byte +575 ==1 also requires action byte +628 !=0.
❓ Exact table names, XML field names and the full modifier derivation are not verified.
Do not treat opaque offsets as invented typed table fields.

### Verified check order and limits

The factory selects different virtual validation paths.
Action type 0 uses vtable `0x140C78848`; virtual +288 resolves to `0x1404AB580`.
Type 4 uses vtable `0x140C7FC68`; the same slot resolves to `0x1404BA030`.
The following order is the checked common preparation path and the recovered prefix of `0x1404AB580`, not all skill classes.

`0x14049A7A0` (prepareAction):

1. Copy server caster position +268/+270/+272 into action +242/+244/+246.
2. Apply skill-record target-redirection mode +809 when present.
3. Save the original target. Self-type action classes can replace the active target with the caster.
4. Calculate server action/formula values through `0x1404B4840`, `0x14049B9F0` and `0x14049C500`.
5. For the ordinary branch, call virtual +288 (castability). Special skill/mode branches can return earlier. ❓ Those branches are not fully specified.
6. For PC/summon owners, check global recycle group through `0x140618680`, with group from skill byte +711. Failure sends command 1227 reason 196.
7. Check recycle group through `0x1406185E0`, with skill bytes +716/+717. Failure sends reason 196.
8. If item-skill equip-position count is positive, call `0x14049C8B0`. Failure sends reason 2.
9. Continue preparation only after the selected checks pass.

Thus the checked castability phase precedes these two recycle checks. Do not reorder checks to match the initial log-based hypothesis.
`0x140618680` accepts only when `stored_expiry - network_config.i32[700] < server_now`.
Its valid group indices are 1..20; an out-of-range group selects expiry zero.
`server_now` uses the daemon's `GetTickCount64`-based relative time.
`0x1406185E0` calls expiry lookup `0x140618BE0` and compares expiry with server time. ❓ Register-derived time expression and exact recycle-group lookup rules remain open.

Recovered prefix of `0x1404AB580` (castability), in order:

| Order | Gate | Failure |
| ---: | --- | --- |
| 1 | Server dead flag at caster +578 | Reason 185 |
| 2 | Conditional transformed/soulmask restriction: rule-record +352, caster selector +3449/+3450, selected-record +26 | Reason 185 |
| 3 | Conditional grocery/item presence check when action short +178 >0 | Item reply command 1235, reason 216 |
| 4 | `0x1404A6BD0`, cast filter | Propagated reason |
| 5 | `0x1404AE9C0`, consumable points | Reason 190 |
| 6 | If action +628 ==0 and rule byte +31 ==0, `0x1404ADF60`, acquired/usable skill | Reason 188 |
| 7 | Server falling/disabled bit at caster +888, mask 2 | Reason 184 |
| 8 | Conditional virtual action-availability check; link-stage rules | Reason 184 or 191 |
| Later | Swim, dash, water, throw and special-skill checks | ❓ Full predicates and complete order not mapped |

Cost check `0x1404AE9C0` reads positive required-point shorts at rule-record +218, +222/+224 and +228.
It passes type bytes at +220, +226/+227 and +230 to `0x1404AE820`.
It uses server caster resources +904, +896/+928 and +296, plus property IDs 248/249 and 120/121.
It can resolve an owning creature through `0x14061D630` and can check the controlled summon.
Skill-record +734..+746 supplies additional selector/value inputs.
❓ Exact resource formula, percentage modes and later deduction timing remain open. This pass proves a server cost gate, not a complete payment implementation.

### Target, range and line of sight

UTF-16 `checkCreature, !checkLineOfSight` at `0x140CE7BF0` maps to `0x1404A63D0`.
This helper resolves the target object, then checks range, direction, target attributes/filter and line of sight in that order on its ordinary target path.
❓ All virtual dispatch routes from UseSkillReq to this helper are not mapped. Do not state that every skill runs this exact route.

- Ordinary range uses cylinder helper `0x1404AE5F0`.
- It requires `min_radius^2 <= dx^2+dy^2 <= max_radius^2`.
- It bounds `dz^2` by the selected vertical height/depth squared. Equality passes.
- `0x1404A63D0` adds a server target-size value to nonzero minimum range and to maximum range.
- Target position comes from target object +268/+270/+272. Direction uses server object yaw +274.
- Range failure returns reason 195. Direction failure returns 194.
- Target-filter helper is `0x1404A7730`; its reason propagates.
- Line-of-sight helper is `0x1400D6CD0`. It uses zone geometry, source position z+20, target server position z+20 and target short +2304. Failure returns 210.
- Skill line of sight is therefore verified in this server helper. It is not only a log-based guess.

❓ Gather-range field mapping, all target-filter predicates, cast-filter predicates, exact per-class range origins, effect formulas and complete use/cooldown/cost ordering remain open.
Keep this part partial. Do not invent a single universal skill-validation pipeline.

## Step 5, part 3 — Interaction distance (partial)

See [29 — NPC trade, board gacha and dungeon-step handlers](29-npc-trade-handlers.md)
for receive entries 1341, 1343, 1348, 1350, 1352 and 1456.
The task-distance evidence below stays in this doc.

### Common position and distance rule

The checked trade and NPC inventory paths read signed-short x/y/z from live server objects at +268/+270/+272.
They calculate squared Euclidean distance inline: `d2=dx*dx+dy*dy+dz*dz`.
They do not call a square-root distance function.
They do not read request coordinate fields for these checks.
These statements describe the checked interaction paths. They do not prove a universal rule for every request.

| Path | Function / log xref | Accept range | Limit source | Distance failure |
| --- | --- | --- | --- | --- |
| Trade invite | `0x1403DE170` / `0x1403DE638` | `d2 <3969` | Code constant, radius 63 with strict boundary | Command 312, format `hh`, reason 908 |
| Trade join | `0x1403DEEA0` / `0x1403DF2E9` | `d2 <3969` | Code constant | Command 315, `hh`, reason 917 |
| Exchange faction score | `0x140517ED0` / `0x14051807A` | `d2 <=10000` | Code constant, radius 100 with inclusive boundary | Command 85, `hh`, reason 334 |
| Move depot, two task paths | `0x14051E910`, `0x14051F7A0` / `0x14051EA7F`, `0x14051F90F` | `d2 <=10000` | Code constant | Helper returns 301. ❓ Outer reply not traced. |
| Buy | `0x140520250` / `0x140520440` | `d2 <=10000` | Code constant | Command 1342, `hh`, reason 306 |
| Sell | `0x140520E90` / `0x140521036` | `d2 <=10000` | Code constant | Command 1349, `hh`, reason 318 |
| Rebuy | `0x140522710` / `0x14052284B` | `d2 <=10000` | Code constant | Reason 326. ❓ Final reply command not recovered in this pass. |
| TradeWithNpcTask | `0x140522A00` / `0x140522CA4` | `d2 <=10000` | Code constant | Reason 341. ❓ Outer reply contract remains open. |
| Buy2Task, NPC branch | `0x140526F10` / `0x14052716B` | `d2 <=10000` | Code constant | Command 1344, `hh`, reason 35 |
| Repair | `0x140528D80` / `0x140528ED6` | `d2 <=10000` | Code constant | Command 1386, `hh`, reason 451 |

Trade invite compares the resolved inviter and guest server objects.
Trade join compares the stored inviter at trade +32 with the resolved guest.
The inventory paths compare the task's server PC with the zone-resolved NPC.
NPC kind +46 must be 4 on the checked branches.
Other NPC/service/store checks run too. The table above specifies only distance.

Conditional distance bypasses exist:

- Depot helpers return success before NPC lookup when `0x1405EC080(pc,25)>0` or `0x1405ED890(pc)` is true. ❓ Exact property/permission meaning open.
- Buy2Task has a non-NPC branch when its object ID is zero. It uses a separate selected store path. The NPC distance rule does not apply to that branch.
- ❓ Other conditional non-NPC branches are not fully mapped. Do not impose radius 100 on every inventory operation.

### Quest distance helper

UTF-16 log `0x140D69210` → xref `0x14057404B` → helper `0x140573EA0`.
The helper receives two server object pointers, a signed integer maximum distance and an operation mode.
It does not receive a client position vector.

Pseudo-steps:

1. Require non-null source and target objects.
2. Require source kind in {1,2,4,240,241} and target kind in {1,2,4,32,240,241}.
3. If mode is 2, skip the distance calculation after those pointer/kind checks.
4. For ordinary creatures, read target x/y/z at +268/+270/+272.
5. For kind 32, call target virtual +152 for its server position. Failure returns 1014.
6. Read source position at +268/+270/+272.
7. Compute the same 3D squared distance inline.
8. Return reason 1010 if `maximum*maximum < d2`. Equality passes.
9. Otherwise return success.

❓ Invalid pointer/kind return code was not recovered in this pass.
The mode-2 bypass is verified. Its complete semantic name and all callers remain open.

| Caller / call site | Maximum-distance source |
| --- | --- |
| Quest case dispatcher `0x140568010`, calls `0x140568EC0`, `0x1405691A3`, `0x14056A221` | Code constant 125 on the checked cases |
| Same dispatcher, `0x140569DF0` | Server configuration object at daemon +2744, integer field +36 |
| Same dispatcher, `0x140569F86` | Same configuration object, integer field +452 |
| Quest helper `0x140572790`, call `0x140572C0E` | Selects constant 125 or configuration +452 by case |
| Quest helper `0x140577F90`, call `0x14057824D` | Configuration +36 |
| DungeonProgressToStepN `0x14058EA90`, call `0x14058EF62`, too-far log xref `0x14058EF84` | Constant 125; mode 1 |

The dungeon-progress caller converts a nonzero distance-helper result to reason 961 in its checked branch.
❓ Its final wire reply and all quest failure propagation remain open.
No checked trade/NPC range above comes from a game-table distance field.
Quest range can come from server configuration, not only a constant.
❓ Configuration option names and complete quest case-to-field mapping remain open.
This part maps the requested log families. It does not cover all 26 initial too-far strings.

## Step 5, part 4 — Failure and kick path (partial)

### Log scope and ordered failure actions

UTF-16 `failProcess, kick, state:...clientReason` at `0x140A1FE70` has xrefs at `0x1401C8934` and `0x1401C896B` in `0x1401C8820`.
The log prefix is `BoardGachaDraw`.
It belongs to that transaction's failure path, not a universal movement or skill validation function.
Do not turn this log into a rule to kick on each rejected game request.

Checked failure steps in `0x1401C8820`:

1. Read the owning user object from transaction +80.
2. Get its session through virtual +48.
3. If a session exists, send command 1178, full format `hh`, with the signed-short `clientReason` argument. Release the session reference through `0x14000B730`.
4. Otherwise, log the missing session. Continue failure processing.
5. If transaction byte +252 is nonzero, log the kick and call user-manager helper `0x1403EF8E0`.
6. Log the transaction failure.
7. Call `0x1400D47D0(transaction,5)` to change transaction state.

The log reads current state from transaction +32 and transaction ID from +24.
Those fields describe server transaction state. They are not client movement state.
`0x14000B730` is a reference-release call here, not evidence of a socket close.
❓ The code that sets byte +252 was not recovered. A failed transaction alone does not prove that this byte is set.

### Direct failure reasons and triggers

These are numeric reasons and descriptions from server calls and logs.
They are not verified client enum names.

| clientReason | Checked trigger / source | Evidence |
| ---: | --- | --- |
| 1 | Abort helper has no supplied reason pointer | `0x1401C84F0` |
| 2 | Finish-ticket request failure or later task initialization failure | `0x1401C6AC0`, `0x1401C8050`, `0x1401C82D0` |
| 2530 | Try-draw-ticket request fails, or its task creation/initialization path fails | `0x1401C6AC0`, `0x1401C7A80` |
| 2531 | Condition check fails; consume-items preparation/request fails; or a checked consume-state/expiry path fails | `0x1401C76B0`, `0x1401C7D80`, `0x1401C8050`, `0x1401C6AC0` |
| Supplied short | Abort helper forwards the supplied reason | `0x1401C84F0` |
| Supplied short | Expiry helper forwards its argument; one checked state-dispatch caller supplies 2531 | `0x1401C8720`, caller `0x1401C6AC0` |
| Mapped backend reason | Failure callback converts its backend code through `0x140018AE0` | `0x1401C8600` |

Condition helper `0x1401C72D0` can fail for a missing PC, missing/invalid cost configuration, missing/invalid item, invalid requested count, insufficient item count, wrong item-data identity, wrong total count, or failed disposal check.
It reads server inventory and required-cost data.
The caller converts failure to 2531.
❓ This is not a complete specification for BoardGachaDraw item consumption or rollback.

### Backend reason conversion

The complete switch in `0x140018AE0` has these numeric mappings:

| Backend code | clientReason |
| --- | --- |
| 0 | 0 |
| 2..8, 21..23, 45 | 2 |
| 15..19 | 1959..1963, in the same order |
| 25..35 | 16..26, in the same order |
| 41 | 27 |
| 46, 47, 48 | 2515, 2516, 2517 |
| 49..55 | 2523..2529, in the same order |
| 56, 57 | 2532 |
| 59..62 | 2533..2536, in the same order |
| All other codes | 1 |

This is the mapper's output set, not proof that each input reaches this transaction.
❓ Reachable backend codes and the complete set of supplied abort/expiry reasons remain open.
The client database was not opened in this pass. No client reason names are claimed.

### User-manager kick boundary

`0x1403EF8E0` takes a manager, an identity argument and a separate short reason.
It reads network configuration through daemon +2736.
When config byte +237 and integer +248 are both nonzero, it uses a remote manager path and composes command 481 with format `hqh`.
Its other branch resolves a local user and creates a task with vtable `0x1408D5BB0` for transaction-manager processing.
It also contains conditional integration calls. Those callees were not followed.

❓ The failProcess call's identity and separate kick-reason register arguments were not recovered.
Do not assume the separate kick reason equals the command-1178 `clientReason`.
❓ The final socket-close action, queue completion, cleanup, policy option names and byte +252 provenance remain open.
The checked code proves entry into a conditional user-manager kick path, not an immediate socket close on every failure.

## Step 5 closure and port limits

| Part | Result | Port boundary |
| --- | --- | --- |
| Movement | Partial | Port the stated predicates and correction fields. Resolve cell loading, transitions and remaining packet fields before full integration. |
| UseSkillReq | Partial | Port the checked parser bounds and path-specific gates. Do not invent a universal action-class pipeline or cost formula. |
| Interaction distance | Partial | Preserve strict trade and inclusive NPC/quest boundaries. Preserve the stated bypasses. |
| Failure/kick | Partial | Preserve transaction scope, reply reason conversion and the +252 condition. Do not add a generic validation kick. |

This pass changed specifications only. No Rust code or binary was changed.
No test or live acceptance check ran.
Only the allowed GameDaemon database was opened. No client binary was read.
No network, SSH, git command, key material or game-record value was used.
The anti-cheat hooks at `0x1405DF767` and `0x14036DE10` remain stop boundaries.
No anti-cheat or licence implementation was inspected or specified.
No explicit database save was requested.
