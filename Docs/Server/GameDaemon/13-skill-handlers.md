| NEW shared helper, build 381 | Purpose |
| --- | --- |
| `0x140498D00` | Initialize a ground-target skill action. See command 1228. |
| `0x140098D90` | Create the cancel-action task used by command 1229. |
| `0x1404A63D0`; `0x1404AE5F0` | Creature-target check and cylinder geometry. Extend doc 07; see command 1226. |
| `0x1404A6BD0` | Ordered cast-filter gates. See command 1226. |
| `0x1404ADF60` | Item, owner and acquired-skill routes. See command 1226. |
| `0x1404AE9C0`; `0x1404AE820` | Ordered resource availability and threshold formulas. See command 1226. |
| `0x140618BE0`; `0x1406185E0`; `0x140618680` | Local category lookup, local readiness and global readiness. See command 1226. |
| `0x1405E8A10` | Skill-skin permission routes by table record kind. See command 1065. |

# 13 — Skill and combat handlers, build 381

## Scope and notation

Source: `ida/datafile/gd2020/GameDaemon.exe`, build 381.8867, without PDB.
Build-348 names come from the requested rows in `parity-ledger.tsv` only.
A native offset is not a wire offset.
A full format includes command `h`. It excludes the outer size.
`c`, `h`, `i`, and `q` are 8-, 16-, 32-, and 64-bit scalar fields.
`Q` is also a one-byte scalar. `u` is the native coordinate-short format.
`C...;` starts with a two-byte repeat count. The semicolon adds no wire byte.
Exact descriptor literals below come from binary string bytes, not IDA symbol spelling.
Keep opaque fields opaque. Do not infer a rule from a command name.
`done` means all game-logic paths were read. `partial` lists unread paths with ❓.
A partial section is not a complete Rust port contract.
No live test ran.

Reference part-1 helpers by their existing names in doc 12: request decoder, reference release, zone task queue wrapper, direct formatted reply, inventory helpers, and money-property helpers.
Common task setup and action end are as specified in doc 11: attach zone with `0x14025DB30`, store caster ID at +128, attach child with `0x14069BF60`, mark +138, then queue with `0x140254FD0`.
Common action initialization is `0x140471800`. Action end is `0x140422850`.
A successful queue is not a successful operation.
Unchecked decode results do not establish a safe malformed-packet result.
All listed wrappers return true after their handled paths.
Unless a section states otherwise, its wrapper has no direct money, stack, cooldown, position, buff, DB, other-daemon, or kick operation.
This statement does not cover an unread task or action.
Do not add a generic kick on rejection. See doc 07, Step 5 part 4.
Stop at vendor anti-cheat and licence boundaries. Do not port their internals.

## 1226 — G_UseSkillReqCmd — partial

- **381:** `0x140373330`. **348:** `HandleGame_UseSkillReq`.
- **Wire:** `hcicqhcCc;Cq;Cc;Cc;`. Body: control selector:c, skill ID:i, variation:c, target object ID:q, request short:h, item/equip selector:c, modifier equip positions:C[c], modifier set-item keys:C[q], item-skill equip positions:C[c], modifier badge equip positions:C[c]. The item/equip selector starts at -1. ❓ Exact semantic names for both selector fields and the request short remain open.
- **Checks/order:** Decode the six scalars. Require signed group counts in 0..8, 0..38, 0..4, and 0..26, in that order. Decode every group element. Selector 1 selects PC summon +36880 only when summon DWORD +6984 is 1 or 2. Every other selector selects the PC. Require selected creature and zone +2224. Form key `(u32(skill ID), u32(variation))`. Create action through `0x1404BE5A0`. Initialize through `0x1404988A0`, mode 1, with target ID and all modifiers. Call `0x14049A550`, log action state through `0x14049A5D0`, then queue.
- **Failure:** Command 1227, `hEcic`: reason, selector, skill ID, variation. Scalar-decode failure uses `(1,0,0,0)` to requester. Group/decode/control failure uses reason 1 and decoded identity to requester. Missing zone, action creation/init failure and queue failure use reason 1 to the selected creature's owner. Init can send a specific failure before the wrapper sends its generic failure. Queue failure also records action state 16.
- **Deferred checks/state:** Extend doc 07 Step 5 part 2. Its dependency slots, ordinary preparation, castability prefix, resource check, target/range/filter/LOS and recycle gates are path-specific. Do not impose them on every action class.
- **Output/backend/kick:** No immediate success, broadcast, DB procedure, other-daemon request or user-manager kick in the wrapper. Reference release is not a kick.
- **Rust comparison:** `validation/skill.rs` implements only the doc-07 subset. It does not implement this complete handler, action lifecycle, modifiers, costs, effect formulas or all virtual classes.
- **❓ Missing:** Full factory branches, modifier data derivation, all action classes and continuations, payment/cooldown mutation order, effect/buff/combat mutation, all broadcasts, persistence and deferred failure paths.

### Ordinary castability extension (`0x1404AB580`)

This is the complete top-level branch order for this one virtual implementation.
It is not the complete set of callee predicates or all action classes.
Let `C` be the server caster, `S=action.qword[288]` the Skill record, and `R=action.qword[296]` the auxiliary rule record.
❓ The exact table name and field names for R remain unresolved.
For ordinary errors below, kinds 1..3 send 1227, `hEcic`, through C virtual +592.
The selector normally comes from C virtual +600. Skill ID/variation come from S +8/+12.
Kinds 240/241 use `0x1400A69F0` with action +280 and the same reason where stated by the shared error branch.
Other kinds can return false without a reply.
The PC-only combo/event/quest failures explicitly use selector zero.
Logging action state 16 is not a user kick.

| Order | Exact top-level gate | Failure |
| ---: | --- | --- |
| 1 | C byte +578 ==1 | 185. This is equality with 1, not every nonzero life state. |
| 2 | R byte +352 !=0 and transformed key from C bytes +3449/+3450 !=0: resolve manager slot 2715; reject selected-record byte +26 !=0 | 185. Missing record logs, then native code still dereferences the null record. Do not invent a safe reject. |
| 3 | Action short +178 >0 and C kind 1: inventory container 2 lookup at that position must exist and item virtual +104 must equal 3 | 1235, `hh`, 216. |
| 4 | Cast-filter `0x1404A6BD0` must pass | Propagated reason. See cast-filter extension below; relation callees remain partial. |
| 5 | Consumable-point check `0x1404AE9C0` must pass | 190. See resource extension below; formula suffix remains partial. |
| 6 | If action byte +628 ==0 and R byte +31 ==0, acquired-skill check `0x1404ADF60` must pass | 188. See acquired-skill extension below; acquisition/item callees remain partial. |
| 7 | For C kind 1, if R byte +31 !=0 and R qword +336 !=0, transformed key must match either nonzero qword R +336 or +344 | 213. A zero first qword bypasses this whole two-entry check. |
| 8 | C flags +888 mask 2 must be clear | 184. |
| 9 | If R byte +30 ==0, action virtual +72 must pass | 184. ❓ Virtual predicate remains unread. |
| 10 | In that same R +30 ==0 branch: signed R byte +29 ==0 requires C byte +378 ==0. Positive R +29 requires equality of R +27/+28/+29 with C +376/+377/+378. If R +27 is outside unsigned 4..7, C qword +368 must also equal action target +192 | 191. Negative R +29 skips these link checks. |
| 11 | Else, for C kind 1 with network config byte +1630 !=0: R +29 ==0 requires C +378 ==0; positive R +29 requires the same three-byte equality | 191. This branch does not apply the preceding target-ID equality. |
| 12 | If C byte +890 mask 1 is clear, target is nonzero and not self, and target is not C's linked companion: resolve target with `0x1404B3DD0`; reject an existing target with byte +1896 !=0 or +1905 !=0 | 182. Companion pointer: C +36880 for kind 1, +6968 for kind 2, +6760 for kind 240, +3480 for kind 241. |
| 13 | C DWORD +2300 ==8 | 185 (swim). |
| 14 | If dash record action +304 exists and its byte +24 !=0, apply dash checks below | 182. |
| 15 | If S byte +546 !=0, apply throw-water check below | 182. |
| 16 | If R byte +353 !=0, require C kind 1; for kind 1 require `0x1405F5280` to pass | Non-PC: 201. PC gauge-helper failure: 180 with selector zero. ❓ Gauge helper remains unread. |
| 17 | For PC, if S byte +260 ==2, require key `S.qword[264]` in server map C +19912 | 192, selector zero. |
| 18 | For PC or kind 240, if R DWORD +248 !=0, apply combo-window check below | 193. |
| 19 | For PC or kind 240, if R byte +312 !=0, apply event-window check below | 193. |
| 20 | PC only: if S qword +192 !=0 and target kind is 1/2/4/240/241 or 32, resolve creature or kind-32 target and call `0x1404B10D0` | 202, selector zero. ❓ Quest-event predicate remains unread. Other target kinds skip this gate. |

**Dash/throw water tests.** Geometry helper `0x1400244E0` queries zone +16224 and returns environment index byte +10 and cell attribute byte +11 in its result. Only a successful query with environment zero, or a positive environment that exists and whose virtual +232 returns zero, enters the water test. Attribute 1 rejects. A failed geometry lookup does not itself reject here. A negative environment index logs and skips the water test.

For dash, action byte +180 ==1 tests action requested XYZ +208/+210/+212. Otherwise, creature target kinds 1/2/4/240/241 resolve the live target, test its server XYZ, then reject if `0x14025FC10(target)` returns nonzero. A missing target does not itself reject in this dash branch. A noncreature target with action byte +180 ==0 rejects with 182; other noncreature modes pass this branch.
For throw, start with action XYZ. If mode +180 ==0 and target has one of those creature kinds, use its server XYZ when resolution succeeds. Apply only the water test. ❓ The dash-availability helper and geometry loader remain unread.

**Combo-window check.** Read C virtual +160 as current combo ID. Let `now=GetTickCount64()-daemon_clock_base[80]`. Scan at most four entries, stopping at the first zero DWORD in R +248 +4*i. Each entry uses selector byte R +264+i, source byte R +268+i, and 16-bit event mask R +272+2*i. At least one entry must match; else reason 193.

- Selector 0 does not match.
- Selector 1, source 0: expiry=C virtual +192; event=C virtual +224. Source 1 uses +208/+240. Source 2 uses expiry from +176 and skips event-mask matching. Require current combo ID == entry DWORD and expiry >=now. For source 0/1, also require a set mask bit for an event index in 1..18 accepted by `0x140495C40(0,event,index)`. ❓ Exact event relation and invalid-source behavior need callee/instruction proof.
- Selector 2 scans indices 1..18. Only indices 1..8 map to event codes 1..8. For a set bit, get expiry with C virtual +272(code). Require current combo ID equality and expiry >=now.
- Other selector values log and do not directly reject; the aggregate match flag controls the final failure.

**Event-window check.** Scan two selector bytes R +312+i and masks R +314+2*i. Scan mask indices 1..18, but only indices 1..8 map to codes. Selector 1 obtains expiry through C virtual +256(code). Selector 2 obtains expiry through +272(code). A selected expiry >=now sets the aggregate match flag. Expiry starts at zero and is retained across iterations. Selector 0 and invalid selectors retain that value; invalid selectors log. A zero first selector bypasses this whole check. Failure of the aggregate match sends 193. Preserve `>=` at the expiry boundary.

Kinds other than PC/240 return success after gate 16. Kind 240 has combo/event gates but no passive-map or quest-event gate. Its failures use `0x1400A69F0`.
No resource deduction, cooldown write, effect application or user-manager kick occurs in this validator.

### Recycle lookup extension

- `0x140618BE0`: local category capacities are `{1:45, 2:15, 3:6, 4:10, 5:25, 6:3}`. A group outside 1..capacity or an unknown category returns expiry zero. A valid group reads the first qword of a 24-byte entry from the category array at recycle-manager +480+8*category.
- `0x1406185E0` requires manager.qword[0] nonzero. It calls `0x140618BE0` and accepts iff signed expiry `< GetTickCount64()-clock_base.qword[80]`. Equality rejects. Disassembly preserves relative ticks in R11 across the leaf lookup. Unlike the global helper, this local helper does not subtract config +700.
- `0x140618680` also requires manager.qword[0] nonzero. For global groups 1..20, read qword at manager +24*group-16. Other groups use zero. Accept only when `expiry - signed_network_config[700] < now`. Equality rejects.
- **Rust delta:** `skill.rs::castability_prefix` misses gate 7 and all gates after the documented prefix. Its dead field is Boolean, but native gate 1 tests byte equality with 1. `global_recycle` matches the valid-manager arithmetic only; it has no manager-present gate. The local category mapping is not implemented there. These APIs do not establish full parity.

### Creature target check (`0x1404A63D0`)

This helper returns a reason; zero means pass. It does not send a packet itself.
Its arguments are action, direction-record pointer, source XYZ, source yaw, check mode, target ID, min/max radius, two vertical limits and an extra range short.
The target resolver `0x1404B3DD0` remains unread.

1. Resolve live target. Require it in zone +16232 and require its zone ID to equal the captured action zone ID. Missing/foreign target returns 182.
2. Read target size: short +2304 for kinds 1/2/4/240/241; virtual +160(0) for kind 32; zero for other kinds. Add size to maximum. Add size to minimum **only if signed minimum >0**; otherwise use zero. This differs from Rust `sized_skill_ranges`, which adds size to negative nonzero minima.
3. Derive two signed direction selectors from the supplied record. Mode 1 uses +24/+25. Mode 2 uses +156/+161 plus signed action byte +633. Mode 3 uses +216/+221 plus that index. Null record/other modes use zero selectors. ❓ Index validity is not established here.
4. If extra range short >0, call `0x1404B2B40` with source XYZ, live target, source yaw, adjusted radii, extra range, vertical limits, and Boolean `((selector1-2)&~2)!=0`. Failure returns 195. ❓ Exact sector/shape predicate remains unread.
5. Otherwise call cylinder helper `0x1404AE5F0` with supplied source and live target XYZ. Failure returns 195. It accepts inclusive radial and vertical bounds. Select first vertical limit when source.z <=target.z; otherwise select second. Success writes full 3D squared distance.
6. For modes 1..3, call `0x1404B14A0` twice: caster selector with caster yaw/server XYZ toward target, then target selector with target yaw/server XYZ toward caster. Failure of either returns 194. These directions use current caster XYZ, not necessarily the supplied range origin.
7. Mode 1 only: inspect up to four nonzero Skill bytes +208..211. For target kind 4, compare against up to eight bytes of target record +3456, offset +76. For kind 2 use up to four bytes of record +6944, offset +56. For each skill byte, scan while it is <=signed target byte; equality returns 182. A larger skill byte stops that inner scan. Other kinds or absent records skip. ❓ Exact category names remain open.
8. Call `0x1404A7730(action,mode,target,&reason)`; false returns its reason.
9. Call `0x1400D6CD0(zone,source_XYZ_with_z+20,target_XYZ_with_z+20,target_short+2304,0)`. False returns 210. The z+20 assignments narrow back to shorts. ❓ Geometry internals remain unread.

The final two checks run for every mode that reaches them.
Release the resolved target on exit. No target mutation or direct backend/kick appears here.

### Cast filter (`0x1404A6BD0`)

Let `R=action.qword[296]`, `S=action.qword[288]`, `C=attached caster`.
The helper sets output reason to zero, then applies these checks in order.
Failures use reason 185 except the two buff gates noted below.
The ordinary castability caller sends the reply.

1. For kinds 1/240, test water at server XYZ using the same geometry/environment rule as the dash check. Successful ordinary-environment query with attribute 1 rejects. Failed lookup does not reject; negative environment logs and skips.
2. If R byte +25 !=0, require caster kind 2 or 241.
3. If R byte +24 !=0, require equality with C byte +364.
4. Require attached caster. R byte +33: 0 imposes no flag test; 1 requires C byte +888 mask 1 set; 2 requires it clear; all other values reject.
5. PC only: if R byte +26 !=0, require inventory item from `0x1404CAD00(C+15000,1,1)`, a nonnull record from item virtual +112, and record byte +2008 equal R +26.
6. PC only: if signed R short +216 >0, require C signed short +546 >=that value.
7. Require `0x140495D90(C,R+34,4,R.byte[42],&count)` true. Then call it on R+44 with selector R +52; reject only if count >0 and result ==1.
8. Set count=0. Require `0x140495EC0(C,R+54,...,R.byte[62],&count)` true. Then call it on R+64 with selector +72; reject only if count >0 and result ==1. ❓ Optimized third arguments and exact relation formulas remain unread.
9. Except kinds 240/241, call `0x140495FD0` on R+104, selector +120; count >0 and true rejects with **186**. Then call it on R+80, selector +96; count >0 and false rejects with **186**. ❓ Exact buff aggregation remains unread.
10. Require attached caster again. For up to four nonzero S bytes +208..211, look up manager slot 2367. Missing record skips. Read at most four nonzero signed shorts from record +16. For values <=64/128/192/256, test the respective caster qword +464/+472/+480/+488 with the native modulo-64 bit index equivalent to value-1. A set bit rejects with 185. Values >256 skip. ❓ Lookup key setup and negative/invalid short behavior require instruction proof; do not add a new bounds gate.

This helper does not deduct resources or change buffs.

### Acquired-skill check (`0x1404ADF60`)

This predicate sends no packet. Its ordinary caller maps false to 188.
The following branch order is verified, but the item and acquisition callees remain partial.

- First try the item-skill route when signed action byte +636 >=0 and caster kind is 1..3. Kind 1 uses self; kind 2 uses owner +6968; kind 3 returns false. Missing owner returns false.
- Fetch item with `0x1404CAD00(owner+15000,1,selector)`. Only item virtual +104 ==1 enters the item route. If rule-config byte +949 ==1 AND daemon byte +4152 ==1, or `0x1405E7DB0(owner)` is 3/4, return false immediately.
- Read two attached item records from native properties `(29,30,31)` and `(32,33,34)` as i/c/i. For each nonzero i/c key, look up manager slot 1455. A record with byte +2008 ==5 passes its qword +2640 to `0x1404ADEB0`; true returns success. Missing/wrong record logs and continues. If neither succeeds, pass base item-record qword +2208 to the same helper; true returns success. ❓ Exact key setup and item-skill matcher remain unread. Native code can dereference a missing base record; do not invent a safe fallback.
- If the item route does not return, dispatch by caster kind.

| Caster | Remaining predicate |
| --- | --- |
| 1 | If action shorts +178 or +232 are positive, or DWORD +256 is 1/2, pass. Otherwise, if first R qword +336 !=0, require transformed key from `0x140264E30` equal a nonzero member of R +336/+344. Zero first entry skips this entire pair check. If first entry is zero, a special room-controller branch can bypass acquisition: controller +16536 virtual +8 ==5, its +8 object virtual +8 ==2, and Skill ID equals rule-config DWORD +892 or +896. Otherwise derive acquisition key as below; if action byte +8360 ==0, return `0x14059AA70(C+19744,key)`. Nonzero +8360 bypasses that lookup. |
| 2 | Derive key; require owner +6968 and `0x14059AA70(owner+19744,key)` true. |
| 240 | If first R +336 !=0, require a matching transformed key in the two entries and return that result. Otherwise derive key; require nonzero key, nonzero low ID and caster +6576 !=0. No actual map membership lookup appears in this branch. |
| 241 | Derive key; require owner +3480, nonzero key, nonzero low ID and owner +6576 !=0. |
| Other | Pass, unless the earlier item-route kind-3 rejection applied. |

**Acquisition key:** Look up manager slot 2463 by Skill ID. If found, use DWORD record +16 with original variation in byte 4. Otherwise use the native qword at Skill +8. ❓ Full key packing, transformed-register recovery, room-controller predicates, permission helper and map membership internals remain unread.

### Resource availability (`0x1404AE9C0`, `0x1404AE820`)

These predicates do not deduct resources. Ordinary castability maps false to 190.
Use the same C/R/S notation as above.

1. If signed R short +218 >0, call resource predicate `0x1404AE820(action,R.byte[220],R.short[218],S.byte[736],0,S.short[734],C.qword[904],C.qword[896]+C.i32[928],C.qword[296])`. False rejects.
2. For i=0,1, if signed R short +222+2*i >0, choose `0x14061D630(C)` if nonnull, otherwise C. Read native qword properties 248+i and 120+i through the part-1 property reader. Call the predicate with R selector +226+i, R threshold +222+2*i, S selector +742+i, index i+2, S short +738+2*i, and current/current/base property values. False rejects. ❓ Owner/substitute helper remains unread.
3. If signed R short +228 <=0, pass. Otherwise require C kind 1 and nonnull summon +36880. Call the predicate with R selector +230, R threshold +228, S selector +746, index 1, S short +744, and summon resources +904/(+896+signed +928)/+296. False rejects.

For resource predicate arguments `(action,mode,threshold,cost_mode,index,cost_value,A,B,current)`, its first remaining-resource result is:

| Mode | First result |
| ---: | --- |
| 0 or 2 | current-threshold |
| 1 | threshold-current |
| 3 | current-(A*threshold)/100 |
| 4 | current-(B*threshold)/100 |
| 5 | Log invalid mode and return false. |
| Other | Zero. No new invalid-mode rejection appears. |

The predicate then calls `0x1404AF380(action,cost_mode,cost_value,A,B,current)` and checks a second remaining-resource value.
For resource indices 2/3 it can add caster signed DWORD +1776+4*index+4 and clamp using record +1824 shorts +24+2*index and +28+2*index.
It accepts only when both remaining values are >=0.
❓ The decompiler loses caster and current-value registers in this suffix. Exact mode-dependent cost formula, signed narrowing, clamp application and arithmetic overflow require instruction proof. Do not replace this with a simple current>=threshold test.

## 1228 — G_UseSkillToGroundReqCmd — partial

- **381:** `0x140373CE0`. **348:** `HandleGame_UseSkillToGroundReq`.
- **Wire:** Native descriptor `hcicuuuhcCc;Cq;Cc;Cc;`. Body: control selector:c, skill ID:i, variation:c, target x/y/z:u, request short:h, item/equip selector:c, then the same four modifier groups as 1226. ❓ Request short and item/equip selector semantic names remain open.
- **Checks/order:** Decode eight fixed fields. Apply the same ordered count bounds and element decodes as 1226. Resolve controlled caster by the same rule. Require caster and zone. Create by skill key through `0x1404BE5A0`. Pack requested XYZ as three native shorts. Initialize through `0x140498D00`, mode 2, with XYZ, request short and modifier groups. Call `0x14049A550`, record state 1, then queue.
- **Failure:** Same command 1227 and `hEcic` contract as 1226. Scalar failure zeros the identity. Later failures preserve decoded identity. Queue failure records action state 16. No wrapper position/range/LOS check appears before action initialization.
- **Initializer `0x140498D00`:** Store mode +168, request parameter +240, clear +280 and store byte parameter +177. Require nonnull zone/caster/key and caster zone equal captured zone. Store key +160, short +178=-1, request short +234, byte +180=1, XYZ +208/+210/+212, item/equip selector +636. Deduplicate each modifier group separately; array/count pairs are +637/+645, +648/+952, +953/+957, +958/+984. Run common initialization, then data initializer `0x1404978D0`; failure calls `0x1404723D0` and returns false. ❓ Typed meaning of request parameters +177/+240 remains open.
- **State/output/calls:** Only action setup occurs here. No immediate success or direct backend call. Ground coordinates are intent, not proof of an accepted destination.
- **Rust comparison:** `validation/skill.rs` does not implement the ground initializer or its position checks.
- **❓ Missing:** Class-specific ground range/geometry checks, full data initialization, all shared skill action checks and effects, output/broadcast framing, DB work and deferred failures.

## 1229 — G_CancelActionReqCmd — partial

- **381:** `0x1403746C0`. **348:** `HandleGame_CancelActionReq`.
- **Wire:** `hcQ`: control selector:c, cancel-option byte:Q. Decode return is ignored. ❓ Exact option meaning remains open.
- **Checks/order:** Select PC or controlled summon by the 1226 rule. Require selected creature and zone. Create task with `0x140098D90`. Use common task setup. Store option as DWORD +144; set qword +152 to zero. Queue.
- **Failure:** 1230, `hEc`, reason 1 and original selector. Caster-resolution failure goes to requester; zone/create/queue failure goes through selected creature's owner.
- **State/output/calls:** No direct action cancellation or success output in wrapper. Task setup only. No direct backend or kick call.
- **Task `0x140291260`:** Resolve attached caster and call its action-manager +2232 virtual +80 with widened option and zero. No other direct operation appears.
- **❓ Missing:** Action-manager virtual +80, option meaning, cancel permission/type checks, interruption state changes, success/broadcasts and deferred failure results.

## 1231 — G_SkillTriggerReqCmd — done

- **381:** `0x140374930`. **348:** `HandleGame_SkillTriggerReq`.
- **Wire:** `hcc`: control selector, trigger byte. Decode return is ignored.
- **Checks/order:** Resolve controlled caster by the 1226 rule. Require caster and zone. Create task with vtable `0x140BC19E0`, kind 13. Use common setup. Store trigger at task +144. Queue.
- **Failure:** 1232, `hEc`, reason 1 and original selector. Missing caster goes to requester. Zone/queue failure goes through selected creature's owner.
- **State/output/calls:** No direct effect, cooldown or success output in wrapper. No direct backend or kick call.
- **Task `0x140291290`:** Resolve attached caster. If caster native kind !=1, log failed task with caster ID. Kind 1 returns without any game operation. The trigger byte is unused. This build has no trigger effect, active-skill validation, success/broadcast or persistence on this path. Preserve the no-op; do not invent a trigger implementation.

## 1270 — G_GatherReqCmd — partial

- **381:** `0x140374FB0`. **348:** `HandleGame_GatherReq`.
- **Wire:** `hq`: target object ID. Decode return is ignored.
- **Checks/order:** Require PC zone. Require user pointer PC +20696, PC byte +20752 ==0, and user DWORD +120 ==0. Otherwise send reason 683. Create action class 15. Require PC native kind +46 ==1. Initialize common action state; store target ID at action +152; queue.
- **Failure:** 1271, `hh`, requester only. User gate uses 683. Zone/create/kind/queue failures use 1. Wrong kind also asserts/logs. No generic kick.
- **State/output/calls:** No direct item count, money or gather reward change in wrapper. No immediate success or backend request.
- **❓ Missing:** GatherAction lifecycle; target ownership, distance, table, tool, resource and cooldown checks; reservation, inventory mutation, success/broadcasts and persistence.

## 77 — G_GatherCancelReqCmd — partial

- **381:** `0x140375290`. **348:** `HandleGame_GatherCancelReq`.
- **Wire:** No body decoder. Body bytes are unused by wrapper.
- **Checks/order:** Require zone. Create task vtable `0x140C39C08`, kind 76. Require PC native kind 1. Use common setup; queue.
- **Failure:** 78, `hh`, reason 1 to requester for zone/kind/queue failure. Wrong kind also asserts/logs.
- **State/output/calls:** No direct cancel or success packet in wrapper. No direct DB/daemon/kick call.
- **Task `0x1405391A0`:** Call PC action-manager +2232 virtual +32 with action class 15. False sends 78/684, `hh`, to requester. True has no direct reply here.
- **❓ Missing:** Action-manager cancellation predicate and GatherAction cancellation mutation/broadcasts.

## 1272 — G_LootReqCmd — partial

- **381:** `0x140374D70`. **348:** `HandleGame_LootReq`.
- **Wire:** `hq`: loot target object ID. Decode return is ignored.
- **Checks/order:** Require zone. Create action class 14. Require PC native kind 1. Require target ID bits 48..63 ==176. Initialize action; store target ID at +152; queue.
- **Failure:** 1273, `hh`, requester only. Zone/create/kind/queue failure uses 1. Invalid ID kind first sends reason 537, then falls through and sends reason 1 too. Preserve both replies and their order. Wrong PC kind asserts/logs, then sends 1.
- **State/output/calls:** No direct inventory, money, ownership or distance check in wrapper beyond ID kind. No immediate success or backend request.
- **❓ Missing:** LootAction checks and lifecycle, target ownership/position, rewards, inventory mutation, success/broadcasts and persistence.

## 71 — G_RecallSummonReqCmd — partial

- **381:** `0x14038A0E0`. **348:** `HandleGame_RecallSummonReq`.
- **Wire:** `huuuh`: requested x/y/z, yaw. Decode return is ignored.
- **Checks/order:** Require PC summon +36880 with DWORD +6984 in {1,2} (reason 2). Require summon native kind 2 (175). If summon record +6960 exists and record byte +56 ==11, reject (179). Require summon zone (2). Create action class 50. Initialize it with summon as caster. Store XYZ at action +152/+154/+156. The wrapper does not copy yaw. Queue in summon zone.
- **Failure/output:** 72, `hh`, reason above to requester. Create/queue failure uses 2. No immediate success or direct backend/kick call.
- **State:** Action setup only. No wrapper distance or ownership check beyond the PC-held controlled summon.
- **❓ Missing:** Recall action, authoritative position/range checks, final position/yaw, broadcasts and deferred failures.

## 74 — G_SpecialSkillGaugeFullReqCmd — partial

- **381:** `0x14038A340`. **348:** `HandleGame_SpecialSkillGaugeFullReq`.
- **Wire:** `h`, decoded command must equal 74.
- **Checks/order:** Decode success and command equality (1548). Require zone (2). Create task vtable `0x140B9A9A8`, kind 272. Require PC native kind 1. Use common setup, then call `0x140603BD0(parent,task,0,0)`. This wrapper does not use the normal queue call directly.
- **Failure/output:** 76 with native descriptor `hEh`; parse/command failure supplies 1548, missing zone supplies 2. ❓ The additional h argument and wrong-kind reason are not recovered from decompiler arguments. Do not fabricate them.
- **State/calls:** No direct gauge assignment, money or cooldown write in wrapper. No direct DB/daemon/kick call.
- **❓ Missing:** Task scheduler, gauge validation and mutation, full reply layout, success/broadcasts and late errors.

## 134 — G_SelectTargetReqCmd — partial

- **381:** `0x14038AF00`. **348:** `HandleGame_SelectTargetReq`.
- **Wire:** `hq`: target object ID. Decode return is ignored.
- **Checks/order:** Require zone. Create task vtable `0x140B96A10`, kind 11. Common setup; store target at +144; queue.
- **Failure/output:** Missing zone or queue failure logs only. No wrapper failure packet, immediate success, backend request or kick.
- **State:** No direct selected-target write in wrapper. Do not add range or life-state rejection here without reading the task.
- **Task `0x1402911E0`:** Zero a 24-byte delta buffer. Call `0x14026B260(PC,requested_target,buffer)`, then the part-1 flush-delta helper `0x140260850(PC,buffer)`.
- **❓ Missing:** Target-change helper lookup/validity/mutation and full flush output/broadcast behavior.

## 135 — G_DieInExhaustionReqCmd — partial

- **381:** `0x14038B1E0`. **348:** `HandleGame_DieInExhaustionReq`.
- **Wire:** No body decoder.
- **Checks/order:** Require zone. Create task vtable `0x140BABB68`, kind 15. Require native PC kind 1. Use common setup; queue. Wrong kind asserts/logs.
- **Failure/output:** 136, `hh`, reason 1 to requester for zone/kind/queue failure. No immediate success or direct backend/kick call.
- **State:** No direct life-state change in wrapper.
- **Task `0x140291BC0`:** Require PC life-state byte +578 ==2; else send 136/275, `hh`. Zero a 24-byte delta buffer. Call PC virtual +128 with first flag 1 and remaining flags/arguments zero, then the part-1 flush-delta helper.
- **❓ Missing:** Death virtual +128, life/resource mutation, broadcasts and persistence.

## 137 — G_RestorationInExhaustionReqCmd — partial

- **381:** `0x14038B3C0`. **348:** `HandleGame_RestorationInExhaustionReq`.
- **Wire:** No body decoder.
- **Checks/order:** Require zone. Create action class 16. Require native PC kind 1. Initialize common action state; queue. Wrong kind asserts/logs.
- **Failure/output:** 138, `hh`, reason 1 to requester for zone/create/kind/queue failure. No immediate success or direct backend/kick call.
- **State:** No direct restoration, resource or cooldown write in wrapper.
- **❓ Missing:** RestorationAction state/time/resource checks, mutation, broadcasts and deferred failures.

## 139 — G_HelpRestorationReqCmd — partial

- **381:** `0x14038B570`. **348:** `HandleGame_HelpRestorationReq`.
- **Wire:** `hcq`: control selector, target object ID. Decode return is ignored.
- **Checks/order:** Select PC or controlled summon by command 1226 rule. Require selected caster and zone. Create action class 17. Require caster kind 1..3. Initialize common action state; store target at +152; queue. Invalid kind asserts/logs.
- **Failure/output:** 141, `hEc`, reason 1 and original selector. Resolution failure goes to requester. Zone/create/kind failure goes through selected caster's owner. Queue failure explicitly uses the original PC sender, not the selected caster pointer. No immediate success or direct backend/kick call.
- **State:** No direct target restoration in wrapper.
- **❓ Missing:** Help action, target relation/life/range checks, timing, costs, final mutation and broadcasts.

## 142 — G_RespawnReqCmd — partial

- **381:** `0x14038B070`. **348:** `HandleGame_RespawnReq`.
- **Wire:** `hQ`: one respawn-option byte. ❓ Exact option meaning remains open.
- **Checks/order:** Require zone. Create task with `0x140212240`. Initialize through `0x140291520(task,zone,PC,request_scalar)`. Queue.
- **Failure/output:** 144, `hh`, reason 1 for zone/create/init/queue failure. No immediate success or direct backend/kick call.
- **State:** No direct respawn or location change in wrapper.
- **❓ Missing:** Initializer and task, option meaning, death-state and respawn-point checks, costs, teleport effects, broadcasts and persistence.

## 821 — G_SaveVitalityReqCmd — done

- **381:** `0x1403C1BD0`. **348:** `HandleGame_SaveVitalityReq`.
- **Wire:** No body decoder. Body bytes are unused.
- **Complete behavior:** Log the deprecated request, then return true. No validation, state change, reply, broadcast, table lookup, DB/daemon request or kick. Do not implement a vitality save from the name.

## 824 — G_RechargeVitalityReqCmd — partial

- **381:** `0x1403C1C00`. **348:** `HandleGame_RechargeVitalityReq`.
- **Wire:** `h` command-only decode. Return is checked; command equality is not checked here.
- **Checks/order:** Capture `time64()` before gates. Require rule config at daemon +2744 byte +1045 !=0. Decode successfully. Require zone. Create task vtable `0x140BAC0C8`, kind 119. Require native PC kind 1. Use common setup; queue.
- **Failure/output:** 826, `hEq`, reason 1 and captured UTC seconds for disabled config, decode/zone/kind/queue failure. No immediate success or direct backend/kick call.
- **State:** No direct vitality/resource/cooldown mutation in wrapper.
- **❓ Missing:** Task recharge eligibility, cap/cost/time rules, success reply, vitality mutation and persistence.

## 113 — G_SkillList2ReqCmd — partial

- **381:** `0x140386160`. **348:** `HandleGame_SkillList2Req`.
- **Wire:** No body decoder.
- **Checks/order:** Require zone. Create task vtable `0x140BC1C58`, kind 82. Common setup; queue. No explicit native-kind gate here.
- **Failure/output:** Missing zone or queue failure sends 115, `hh`, reason 2. Instruction reads confirm the omitted register. No immediate success or direct backend/kick call.
- **State:** No direct skill list change.
- **❓ Missing:** Fetch task, complete list layout, recipients, table/storage dependency and deferred failures.

## 117 — G_SetMasteryStatPointReqCmd — partial

- **381:** `0x140387500`. **348:** `HandleGame_SetMasteryStatPointReq`.
- **Wire:** `hhhccccc`: two shorts and five bytes. ❓ Named allocation fields are not recovered. Decode return is ignored.
- **Checks/order:** Require zone. Create task vtable `0x140C7E0C0`, kind 269. Require PC kind 1. Require each signed short >=0. Attach task. Require each of the five signed bytes >=0, in order. Store shorts +144/+146 and bytes +148..+152; queue.
- **Failure/output:** Zone/kind/value/queue failures send 119, `hh`, reason 2. Instructions confirm the shared failure register. No wrapper total-point, money or table gate. No immediate success or direct backend/kick call.
- **❓ Missing:** Task, available-point totals, cap/property calculations, state mutation and success/persistence.

## 1485 — G_SkillPageTotalReqCmd — partial

- **381:** `0x140386450`. **348:** `HandleGame_SkillPageTotalReq`.
- **Wire:** No body decoder.
- **Checks/order:** Require zone and PC kind 1. Create task vtable `0x140CE2530`, kind 153. Common setup; queue.
- **Failure/output:** 1487, `hh`, reason 2 for zone/kind/queue failure. No immediate success or direct backend/kick call.
- **State:** No direct page mutation.
- **❓ Missing:** Total-page task, list reply, storage source and deferred failures.

## 1488 — G_ChangeSkillPageReqCmd — partial

- **381:** `0x1403862C0`. **348:** `HandleGame_ChangeSkillPageReq`.
- **Wire:** `hc`: page byte. Decode return is ignored.
- **Checks/order:** Require zone and PC kind 1. Create task vtable `0x140D23128`, kind 150. Common setup; copy page byte to +144; queue. No wrapper page-bound, combat or cooldown gate.
- **Failure/output:** 1490, `hh`, reason 2 for zone/kind/queue failure. No immediate success or direct backend/kick call.
- **❓ Missing:** Task page ownership/bounds, combat/cooldown rules, applied skills, replies/broadcasts and persistence.

## 1491 — G_ExpandSkillPageReqCmd — partial

- **381:** `0x1403867A0`. **348:** `HandleGame_ExpandSkillPageReq`.
- **Wire:** `hcsChh;`: page byte:c, page-name:s with decoder capacity 32 UTF-16 units, counted pairs of shorts. The two-short records are copied unchanged with native stride 4. ❓ Exact pair field names remain unresolved.
- **Checks/order:** Decode the two fixed fields. Require signed pair count in 0..15, not 16. Decode every pair. Require zone. Create task vtable `0x140C66B68`, kind 152. Only kind 1 enters initialization/queue; another kind silently skips that work and releases task. Copy pairs to +144, count to +208, page byte to +212, and at most 32 UTF-16 units of name to +214. Queue.
- **Failure/output:** Fixed-decode failure sends **1330**, `hh`, reason 1, then asserts/logs. Count bound or pair-decode failure sends 1493/693. Missing zone or queue failure sends 1493/1. No immediate success or direct backend/kick call. Preserve the different fixed-decode reply command.
- **❓ Missing:** Task, item/count/price checks, name/slot validation, page expansion mutation, replies and persistence.

## 1494 — G_RenameSkillPageReqCmd — partial

- **381:** `0x1403865C0`. **348:** `HandleGame_RenameSkillPageReq`.
- **Wire:** `hcs`: page byte, UTF-16 name. Decoder capacity is 32 units. Decode return is ignored.
- **Checks/order:** Require zone and PC kind 1. Create task vtable `0x140C9A350`, kind 151. Common setup. Store page byte +144 and `wcsncpy(...,32)` name at +146; queue. The copy does not itself prove a terminator for a full-length name.
- **Failure/output:** 1496, `hh`, reason 2 for zone/kind/queue failure. No immediate success or direct backend/kick call.
- **❓ Missing:** Task, page ownership/bounds, name character/length rules, cost, mutation, replies and persistence.

## 1497 — G_TrainTraitReqCmd — partial

- **381:** `0x140386BD0`. **348:** `HandleGame_TrainTraitReq`.
- **Wire:** `hcccccccCic;`: seven fixed byte fields, then counted `(trait ID:i, level:c)` records. The fixed fields are first selector, second selector, and five build-up bytes. A sixth native build-up byte is forced to 1. ❓ Typed selector names remain unresolved.
- **Failure format:** 1499, `hEcc`: reason and both selectors. Predecode failures use selectors -1/-1. No generic kick.
- **Checks/order:**
  1. Require zone (2). Require attached user PC +20696 with PC byte +20752 ==0 (2).
  2. Resolve session through user virtual +48. Missing session skips the rate gate. Existing session requires session +200; missing object sends 2. Let `last=object.qword[8]`, `now=relative ticks`, `interval=rule_config.qword[1264]`. If last !=0 and last+interval >now, reject with 707 and -1/-1. Equality passes. Set last=now before request decoding. Later failure does not roll back this rate timestamp.
  3. Decode seven fixed bytes (2, -1/-1). Require unsigned second selector <=9 (711), then first selector in 1..5 (706).
  4. Create task vtable `0x140C76468`, kind 128. Common setup; store selectors +556/+557. Force native first build-up byte to 1. Require all six native build-up bytes <=3, in order (710). Copy the six bytes to task +144..+149.
  5. Require signed counted-record count in 0..50 (712). Decode each ID/level pair (702). Require ID !=0 (713). Resolve manager slot 2379 by unsigned trait ID (713 if absent). Require signed requested level <= record signed byte +24 (717). There is no wrapper lower-bound check on level. Require stored-record count <50 before append (702); append each pair as native eight-byte record at +152. Duplicates are not removed in this wrapper.
  6. Queue in captured zone (2). Success here has no immediate reply.
- **Table:** Manager slot 2379 is the trait dependency. ❓ Exact table and field names remain unproved; do not name byte +24 from guesswork.
- **State/calls:** Rate timestamp and task state change as above. No direct skill/resource deduction or DB/daemon request in wrapper.
- **❓ Missing:** Task's trait acquisition, totals, money/item/cooldown/combat gates, final skill state, success/broadcasts and persistence.

## 1389 — G_UnlockMasteryLevelReqCmd — partial

- **381:** `0x14037C130`. **348:** `HandleGame_UnlockMasteryLevelReq`.
- **Wire:** `hChh;`: counted short/short pairs, native stride 4. ❓ Pair field names remain open.
- **Checks/order:** Require signed count in 0..15. Decode each pair. Require PC byte +13768 !=1. Require zone. Create action class 29. Only PC kind 1 initializes and queues; another kind asserts/logs, without the wrapper's ordinary failure packet at that branch. Copy pair data to action +152 and count to +216; queue.
- **Failure/output:** Count bound or element-decode failure sends 1391, `hh`, 470. PC +13768 ==1 sends 7. Zone/create/queue failure sends 1. No immediate success or direct backend/kick call.
- **State:** No wrapper unlock, money or item deduction.
- **❓ Missing:** Action lifecycle, mastery table and level gate, inventory/count/cost rules, mutation, success and persistence.

## Common faction gate

For 83/125/127/129 only, apply part-1 T1 before decoding: network config byte +237 and DWORD +248 both nonzero, or locked PC arena session +37520 in {2,3,5,6}, enters failure. Instruction reads confirm reason **5** for all four. Do not add this gate to 131.
For 125/127/129, require global config `qword_140EE58A0` byte +2105 !=0 after T1.
These wrappers do not themselves prove faction ownership or NPC service validity.

## 83 — G_ExchangeFactionScoreReqCmd — partial

- **381:** `0x1403769D0`. **348:** `HandleGame_ExchangeFactionScoreReq`.
- **Wire:** `hqi`: NPC object ID:q, requested amount:i. Decode return is ignored.
- **Checks/order:** Common faction T1. Require attached user PC +20696, PC byte +20752 ==0, user DWORD +120 ==0 (336). Decode. Require zone (1, then assert/log). Create task vtable `0x140BC2808`, kind 60. Require PC kind 1 (1, assert/log). Common setup; store NPC ID +144 and amount +152; queue (1).
- **Failure/output:** 85, `hh`, requester only. T1 uses 5. No immediate success or direct backend/kick call.
- **Deferred subset:** Doc 07 part 3 records `0x140517ED0`: compare signed server PC/NPC XYZ; squared 3D distance <=10000, else 85/334. Equality passes. This is not a wrapper check.
- **❓ Missing:** Task and remaining exchange-helper checks, faction/amount/cap rules, table fields, mutation, success and persistence.

## 125 — G_JoinFactionReqCmd — partial

- **381:** `0x1403877B0`. **348:** `HandleGame_JoinFactionReq`.
- **Wire:** `hcq`: requested faction byte, NPC object ID. Decode return is ignored.
- **Checks/order:** Common T1, then config +2105. Decode. Require zone. Create task vtable `0x140C3CB38`, kind 20. Common setup; store faction +144 and NPC ID +152; queue. No explicit wrapper kind/faction-bound/range gate.
- **Failure/output:** 126, `hh`, requester. T1 uses 5. Disabled config, missing zone and queue failure use 2. Missing zone also asserts/logs. No immediate success or direct backend/kick call.
- **❓ Missing:** Task faction/ownership/NPC/distance/cost/state checks, mutation, success/broadcasts and persistence.

## 127 — G_LeaveFactionReqCmd — partial

- **381:** `0x140387A70`. **348:** `HandleGame_LeaveFactionReq`.
- **Wire:** `hq`: NPC object ID. Decode return is ignored.
- **Checks/order:** Common T1, then config +2105. Decode; require zone. Create task vtable `0x140BC1D60`, kind 21. Common setup; store NPC ID +144; queue. Missing zone asserts/logs.
- **Failure/output:** 128, `hh`, requester. T1 uses 5; disabled config, missing zone and queue failure use 2. No immediate success or direct backend/kick call.
- **❓ Missing:** Task membership/NPC/distance/cost/state checks, mutation, replies/broadcasts and persistence.

## 129 — G_TransferFactionReqCmd — partial

- **381:** `0x140387D20`. **348:** `HandleGame_TransferFactionReq`.
- **Wire:** `hcqc`: faction byte, NPC ID:q, option byte. Decode return is ignored. ❓ Typed meaning of option remains open.
- **Checks/order:** Common T1, then config +2105. Decode; require zone. Create task vtable `0x140C41818`, kind 22. Common setup; store fields at +144/+152/+160; queue. No wrapper range or faction-bound check.
- **Failure/output:** 130, `hh`, requester. T1 uses 5. Config/zone/queue failures use 2. Missing zone also asserts/logs. No immediate success or direct backend/kick call.
- **❓ Missing:** Task transfer/membership/NPC/range/cost gates, mutation, replies/broadcasts and persistence.

## 131 — G_RetrieveFactionStatisticsReqCmd — done

- **381:** `0x140388000`. **348:** `HandleGame_RetrieveFactionStatisticsReq`.
- **Wire:** No body decoder. Body bytes are unused.
- **Complete checks/state:** Read global DWORD `0x140FFDCDC`. If >=100, log and use zero as the prior value. Store prior+1. No faction, zone, life, ownership, range, cost or cooldown validation.
- **Reply:** 132, `hqqq`, requester only. Values are daemon qwords +6032/+6040/+6048, in that order. ❓ Typed statistic names are not established; preserve raw values.
- **Other effects:** No broadcast, table lookup, DB/daemon request or kick.

## 1253 — G_ManipulateReqCmd — partial

- **381:** `0x1403880A0`. **348:** `HandleGame_ManipulateReq`.
- **Wire:** `hqhcCQhh;`: target:q, operation:h, option:c, counted `(container:Q, position:h, count:h)` records. Q is one byte. Native record stride is 6. ❓ Typed operation/option names remain open.
- **Checks/order:** Decode three fixed fields. Require signed count in 0..16 inclusive. Decode all three fields of every record. Compare all pairs: duplicate `(container,position)` rejects, irrespective of requested count. Require zone. Require target ID bits 48..63 ==32. Create action class 19. Common initialize; target +152, operation +160, byte +162=0, option +164, count +168, records +172; queue. No wrapper positive-stack, item-ownership, money or distance gate.
- **Failure/output:** 1254, `hh`, reason 2 to requester for every checked failure. No immediate success or direct backend/kick call.
- **❓ Missing:** ManipulateAction, target/table lookup, item validity/count/ownership/cost/range and cooldown checks, mutation, output and persistence.

## 1255 — G_GiveFactionScoreToRefinerReqCmd — partial

- **381:** `0x140388AC0`. **348:** `HandleGame_GiveFactionScoreToRefinerReq`.
- **Wire:** `hq`: refiner target ID. Decode return is ignored.
- **Checks/order:** Require zone. Create action class 30. Require PC kind 1 and target ID bits 48..63 ==32. Common initialize; target +152; queue.
- **Failure/output:** 1257, `hh`, reason 2 to requester for zone/create/kind/target-kind/queue failure. No immediate success or direct backend/kick call.
- **State:** No direct faction-score deduction.
- **❓ Missing:** Refiner action, ownership/score/amount/range/table/cooldown checks, mutation, success/broadcasts and persistence.

## 1258 — G_TakeRefinerRewardReqCmd — partial

- **381:** `0x140388C00`. **348:** `HandleGame_TakeRefinerRewardReq`.
- **Wire:** `hq`: refiner target ID. Decode return is ignored.
- **Checks/order:** Require zone. Create task vtable `0x140D091E0`, kind 154. Require PC kind 1 and target ID bits 48..63 ==32. Common setup; target +144; queue.
- **Failure/output:** 1259, `hh`, reason 2 to requester for zone/kind/target-kind/queue failure. No immediate success or direct backend/kick call.
- **State:** No direct reward or inventory mutation.
- **❓ Missing:** Task reward eligibility/ownership/count/range/table checks, mutation, output and persistence.

## 1264 — G_NpcManipulateReqCmd — partial

- **381:** `0x140388DA0`. **348:** `HandleGame_NpcManipulateReq`.
- **Wire:** `hqhCQhh;`: NPC target:q, operation:h, counted `(container:Q, position:h, count:h)` records. Q is one byte. Native stride 6. ❓ Operation name remains open.
- **Checks/order:** Before decode, require attached user, PC +20752 ==0, user DWORD +120 ==0 (676). Decode two fixed fields. Require signed count in 0..16 inclusive. Decode all record fields. Reject duplicate `(container,position)` pairs. Require zone. Require target ID bits 48..63 ==4. Create action class 20. Common initialize; store target +152, operation +160, count +168 and records +172; queue.
- **Failure/output:** 1265, `hh`, requester. User gate uses 676. All other checked wrapper failures use 2. No immediate success or direct backend/kick call.
- **❓ Missing:** Action, NPC service/range, inventory count/ownership/cost rules, tables, mutations, output and persistence.

## 1266 — G_FishManipulateReqCmd — partial

- **381:** `0x140388530`. **348:** `HandleGame_FishManipulateReq`.
- **Wire:** `hqhhhchQ`: target:q, x/y/z:h, action:c, operation:h, final option byte:Q. Fixed decoder reads seven fields and tests success. ❓ Exact option meaning remains open.
- **Checks/order:** Require global config +2001 !=0. Decode all fixed fields. Require zone. Action 0 creates class 31; require PC kind 1. Common initialize; store target +152, operation +160, XYZ +412/+414/+416. Zero action fields +162/+164/+168/+400/+408/+449. Queue. Actions 1/2 create task vtable `0x140BC2198`, kind 319; require PC kind 1, then common setup; store action +144 and narrowed final scalar byte +145; queue. Every other action only logs and returns, without a reply.
- **Failure/output:** 1269, `hh`, reason 2 for config/decode/zone/create/kind/queue failure. Wrong kind in task branch asserts/logs. No immediate success or direct backend/kick call.
- **State:** No direct fish reward, item, cooldown or position mutation in wrapper.
- **❓ Missing:** FishAction/task, bait/tool ownership and stack checks, position/target/table/cooldown validation, costs/rewards, broadcasts and persistence.

## 630 — G_ChangeSummonedReqCmd — partial

- **381:** `0x1403B6CB0`. **348:** `HandleGame_ChangeSummonedReq`.
- **Wire:** `hqCc;`: requested summon key:q, counted change bytes. ❓ Exact change-byte meaning remains open.
- **Checks/order:** Decode key. Require signed count in 0..6, not 7. Decode each byte. Require zone. Create task vtable `0x140C39B58`, kind 97. Require PC job byte +100 ==6. Common setup; store key +144, byte array +152, count +160; queue.
- **Failure/output:** Key/element decode sends 632/1331. Invalid count sends **1342/1337**, not 632. Missing zone or queue failure sends 632/1. Wrong job sends 632/1332. All use `hh`, requester only. No immediate success or direct backend/kick call.
- **State:** No direct summon change or cost deduction.
- **❓ Missing:** Task, summon ownership/key/table/change-slot rules, cost, mutation, replies/broadcasts and persistence.

## 633 — G_ChangeSummonedNameReqCmd — partial

- **381:** `0x1403B70A0`. **348:** `HandleGame_ChangeSummonedNameReq`.
- **Wire:** `hsChh;`: UTF-16 summon name with decoder capacity 21 units, counted short/short item orders. Native order stride 4. ❓ Exact pair field names remain open.
- **Checks/order:** Require global config +200 !=0 (1342). Decode name (1342). Require signed count in 0..15 (1346). Decode every pair (1342); require each second short !=0 (1342). Negative second shorts are not rejected here. Require zone (1). Create task vtable `0x140BC18D8`, kind 230. Require PC life-state byte +578 ==0 (2). Require PC job +100 ==6 (1343).
- **Name service:** If daemon +2728 and its +48 service exist, require attached user (PC +20696, PC +20752 ==0; failure 2). Call service virtual +0 with user metadata and requested name. False sends 1345. ❓ This service's identity is unresolved; its internals were not followed. No names or user metadata are copied into this spec.
- **Local name gate:** Call `0x14012F9F0` with name and rule-config fields +172/+176/+180, bytes +192/+193/+194 and qword +200. Nonzero sends 1345. ❓ Full character/length/reserved-name predicate remains unread.
- **Queue/state:** Common setup; copy name to zero-initialized task +208, pair array +144, count +264; queue. Queue failure uses 1. All failures use 635, `hh`, requester only. No immediate success or direct DB command/kick call in wrapper.
- **❓ Missing:** Task, summon ownership, item/count/cost checks, full name predicates, mutation, success/broadcasts and persistence.

## 636 — G_KillSummonedReqCmd — partial

- **381:** `0x14039C990`. **348:** `HandleGame_KillSummonedReq`.
- **Wire:** No body decoder.
- **Checks/order:** Require zone. Create task vtable `0x140BBB6B8`, kind 157. Require PC kind 1. Common setup; queue.
- **Failure/output:** 638, `hh`, reason 2 for zone/kind/queue failure. No immediate success or direct backend/kick call.
- **State:** No direct summon-death mutation.
- **Task `0x1402A0730`:** Require PC summon +36880 and summon DWORD +6984 in {1,2}; else send 638/1351, `hh`, to requester. Zero a 24-byte delta buffer. Call summon virtual +128 with first flag 1, final two flags 1/0 and other arguments zero. Flush through the part-1 delta helper on the summon. No immediate task success send appears.
- **❓ Missing:** Summon death virtual +128, life-state/resource mutation, flush recipients/broadcasts and persistence.

## 788 — G_SkillScoreReqCmd — partial

- **381:** `0x14039C7F0`. **348:** `HandleGame_SkillScoreReq`.
- **Wire:** No body decoder.
- **Checks/order:** Require zone. Create task vtable `0x140C334D8`, kind 155. Require PC kind 1. Common setup; queue.
- **Failure/output:** 790, `hh`, reason 2 for zone/kind/queue failure. No immediate success or direct backend/kick call.
- **State:** No direct score change.
- **Task `0x1402A0710`:** Resolve attached PC and call `0x1405EE2E0(PC)`. No other game operation appears in the task.
- **❓ Missing:** Score helper source/formula, success payload and deferred failures.

## 837 — G_SetEnergyMultipleValueReqCmd — partial

- **381:** `0x1403C6950`. **348:** `-`.
- **Wire:** `hc`: requested multiple byte. Decode return is checked.
- **Checks/order:** Require DataManager byte +82480 !=0 (2614). Decode (2). Require unsigned multiple in 2..63 (2615). Require bit `1u64 << multiple` set in qword +40 of object at DataManager pointer slot 10309 (2615). Require zone (2). Create task vtable `0x140BA0710`, kind 354. Store multiple +144. Common setup; queue (2).
- **Failure/output:** 839, `hh`, reason above, requester only. No immediate success or direct backend/kick call.
- **Table:** ❓ Slot-10309 table/record and bit-field names remain unresolved. These values are runtime data, not code constants for permitted multiples.
- **❓ Missing:** Task current-energy/cost/state rules, mutation, success and persistence.

## 840 — G_OpenEnergyPointRewardReqCmd — partial

- **381:** `0x1403C6BC0`. **348:** `-`.
- **Wire:** `hiQh`: reward ID:i, option byte:Q, requested short:h. Decode return is checked. Reward ID starts at zero.
- **Checks/order:** Require DataManager byte +82480 !=0 (2618). Decode (2). Require option <=8 (2619); no positive lower bound appears. Require signed requested short >=0 (2620). Require zone (2). Create task vtable `0x140BBBCE0`, kind 355. Store ID +144, widened option DWORD +148, short +152. Common setup; queue (2).
- **Failure/output:** Native send is `(format="hEQi",842,1,code,reward_ID)`, with code from the gates above. Preserve that argument order. Do not swap E and Q because their values look unexpected. ❓ Q is one byte in the request decoder, but the output encoder's treatment of 2618/2619/2620 needs a separate check. Do not claim the high bits survive or silently replace the descriptor. No immediate success or direct backend/kick call.
- **❓ Missing:** Task reward/table/ownership/count/cost rules, semantic identity of E/Q fields, inventory/energy mutation, replies and persistence.

## 952 — G_FeedPetFoodReqCmd — partial

- **381:** `0x1403CB270`. **348:** `HandleGame_FeedPetFoodReq`.
- **Wire:** `hQhhc`: selector byte:Q, two shorts, option byte:c. Decode return is checked; all locals start at zero. ❓ Exact item-position/count/selector field names remain unverified.
- **Checks/order:** Decode. Require zone. Create task vtable `0x140B1C680`, kind 234. Require PC kind 1. Common setup; store byte +144, shorts +146/+148, byte +150. Call `0x140603BD0(parent,task,0,0)`. The return is ignored. Wrong kind asserts/logs.
- **Failure/output:** 953, `hh`, reason 2 for decode/zone/kind failure. No immediate success or direct backend/kick call.
- **State:** No wrapper food ownership/stack/count or pet-resource mutation.
- **❓ Missing:** Scheduler/task, pet identity/ownership/equip gates, food validity/count/cost/cooldown rules, mutation, replies/broadcasts and persistence.

## Common training-room wrapper contract

Commands 1026/1029/1032/1035/1038/1053 use checked decoders.
After decode, require zone, create the listed task, require PC kind 1, use common setup, then queue.
A wrong kind asserts/logs and enters the ordinary reason-2 failure.
All wrapper failures go to requester only. No immediate success, broadcast, direct DB/daemon request or kick appears.
The wrappers contain no direct distance, ownership, cost or cooldown test.
Do not treat that absence as a bypass of the unread room task.

## 1026 — G_SkillTrainingRoomCombatStartReqCmd — partial

- **381:** `0x1403D0D40`. **348:** `HandleGame_SkillTrainingRoomCombatStartReq`.
- **Wire:** `hic`: training ID:i, option byte:c. ❓ Exact typed field names remain open.
- **Checks/order/state:** Apply common training-room contract. Task vtable `0x140C3C108`, kind 265. Store sign/widened native ID at +144 and option +152.
- **Failure:** 1028, `hE`; decode failure 2324, zone/kind/queue failure 2.
- **❓ Missing:** Task room/combat/table/target checks, exact ID widening, state changes, success/broadcasts and deferred failures.

## 1029 — G_SkillTrainingRoomCombatStopReqCmd — partial

- **381:** `0x1403D0FB0`. **348:** `HandleGame_SkillTrainingRoomCombatStopReq`.
- **Wire:** `hQ`: stop option byte. ❓ Typed meaning remains open.
- **Checks/order/state:** Apply common training-room contract. Task vtable `0x140C42FA0`, kind 266. Store byte +144.
- **Failure:** 1031, `hE`; decode failure 2329, zone/kind/queue failure 2.
- **Task `0x1402C0AF0`:** Require WORD +48 of the record at `**(zone+16224)` ==216; else 1031/2, `hE`. Require room controller zone +16536; else same failure. Call `0x1401B7C80(controller,PC,stop_byte)`.
- **❓ Missing:** Controller stop routine, room/combat/permission checks, state changes, replies/broadcasts and deferred failures.

## 1032 — G_SkillTrainingRoomCombatRestartReqCmd — partial

- **381:** `0x1403D1250`. **348:** `HandleGame_SkillTrainingRoomCombatRestartReq`.
- **Wire:** `h`: checked command-only decode; no command-equality check here.
- **Checks/order/state:** Apply common training-room contract. Task vtable `0x140C3FAB0`, kind 267. No copied body fields.
- **Failure:** 1034, `hE`; decode failure 2332, zone/kind/queue failure 2.
- **Task `0x1402C0BC0`:** Require WORD +48 of the record at `**(zone+16224)` ==216; else 1034/2, `hE`. Require controller zone +16536; else same failure. Call `0x1401B7F50(controller,PC)`.
- **❓ Missing:** Controller restart routine, room/combat/permission checks, mutations, replies/broadcasts and deferred failures.

## 1035 — G_SkillTrainingRoomInitCreatureStateReqCmd — partial

- **381:** `0x1403D1470`. **348:** `HandleGame_SkillTrainingRoomInitCreatureStateReq`.
- **Wire:** `h`: checked command-only decode; no command-equality check here.
- **Checks/order/state:** Apply common training-room contract. Task vtable `0x140BC2A98`, kind 273. No copied body fields.
- **Failure:** 1037, `hE`; decode failure 2335, zone/kind/queue failure 2.
- **❓ Missing:** Task room/life/resource/permission checks, reset mutation, replies/broadcasts and deferred failures.

## 1038 — G_EnterSkillTrainingRoomReqCmd — partial

- **381:** `0x1403D1690`. **348:** `HandleGame_EnterSkillTrainingRoomReq`.
- **Wire:** `h`: checked command-only decode; no command-equality check here.
- **Checks/order/state:** Apply common training-room contract. Task vtable `0x140C43428`, kind 271. No copied body fields.
- **Failure:** 1040, `hE`; decode failure 2340, zone/kind/queue failure 2.
- **❓ Missing:** Task entry/state/cost/table checks, destination and travel mutation, success/broadcasts and persistence.

## 1053 — G_ResetCombatModeReqCmd — partial

- **381:** `0x1403D1E90`. **348:** `HandleGame_ResetCombatModeReq`.
- **Wire:** `h`: checked command-only decode; no command-equality check here.
- **Checks/order/state:** Apply common training-room wrapper contract, without asserting that this command requires a training room. Task vtable `0x140BB88C8`, kind 288.
- **Failure:** 1055, `hh`; decode failure 2365, zone/kind/queue failure 2.
- **Task `0x1402C8F50`, ordered gates:**
  1. PC life-state byte +578 in {1,2} sends 1055/2366, `hh`.
  2. Compute zone tick as DWORD zone +16144 times DWORD +64 of the object at zone queue +16216, then +24. Missing queue logs and uses tick zero. ❓ Exact overflow/signed conversion needs instruction proof.
  3. If zone +17064 exists, byte +17 !=0 and signed DWORD +20 <= tick-PC.DWORD[20808], take the free reset branch. Call PC virtual +656 with flag 1 and reset-reason descriptor. If PC +888 mask 1 remains set, fail 2. Otherwise write PC qword +20808=tick and send 1054.
  4. Otherwise require DataManager DWORD index 8210 !=0, then manager slot 2067 record for that ID; either missing sends 2.
  5. Require record byte +128 !=0, else 2368. Require PC +12520 nonnull, else 2367. Require its virtual +24(PC,tick) true, else 2369.
  6. Call PC virtual +656 with flag 1 and the reset descriptor. If PC +888 mask 1 remains set, fail 2. Otherwise call `0x1405AF590(9,zone,PC_ID,PC_ID,record+136,4)`, then PC +12520 virtual +16(tick), then send 1054. This branch does not directly write +20808 in the task.
- **Output:** Task failures use 1055, `hh`, requester only. Success uses 1054 and native descriptor at `0x1408CFBE8`. ❓ Exact success literal remains unverified.
- **❓ Missing:** Combat virtual +656, resource/cooldown object virtuals, effect helper, success framing/broadcast recipients and persistence.

## 1062 — G_SkillSkinAcquiredListReqCmd — partial

- **381:** `0x1403D22D0`. **348:** `HandleGame_SkillSkinAcquiredListReq`.
- **Wire:** No body decoder.
- **Checks/order:** Require global config byte +1980 !=0 (2378). Require zone (2). Create task vtable `0x140B9B910`, kind 290. Common setup; queue (2). No explicit wrapper kind gate.
- **Failure/output:** 1064, `hh`, reason above. No immediate success or direct backend/kick call.
- **Task `0x1402C9820`:** Build a packet with command 1063. Give remaining packet capacity to `0x1405FE6F0(PC,destination,capacity)`; append its returned ushort length. Assert/log on insufficient capacity. Send built packet through PC virtual +584, then release it. The native allocation has a 4000-byte payload limit.
- **❓ Missing:** Skin-list serializer/source, complete record format, oversize behavior after assertion and persistence dependency.

## 1065 — G_ApplySkillSkinReqCmd — partial

- **381:** `0x1403D24C0`. **348:** `HandleGame_ApplySkillSkinReq`.
- **Wire:** `hh`: requested skin short. Decode return is checked; local starts at -1.
- **Checks/order:** Require global config +1980 !=0 (2382). Decode (2). Require signed skin >=0 (2). Reject when it equals current PC short +776 (2384). Require zone (2). Create task vtable `0x140AA8730`, kind 291. Repeat skin >=0 check. Common setup; copy skin to +144; queue (2). No wrapper skin-ownership/table/cost check.
- **Failure/output:** 1067, `hh`, requester only. No immediate success or direct backend/kick call.
- **Task `0x1402C99D0`:** Null attached PC logs and returns. Negative skin sends 1067/2. Skin zero bypasses permission helper; nonzero requires `0x1405E8A10(PC)`, else 1067/2383. On pass call `0x14026FFD0(PC,skin,zeroed_24_byte_buffer)`, then `0x140260850(PC,buffer)`; send 1066, `hh`, with skin short. Task failures also use `hh`, requester only.
- **Permission `0x1405E8A10(PC,skin)`:** Negative skin fails; zero passes. Nonzero requires manager-slot-2547 record by skin ID. Dispatch on record byte +24. ❓ The task decompiler omits the second call argument; verify its register before claiming exact argument propagation.
  - Kind 1: index=(skin-1)/8 must be <128. PC object +36512 virtual +48 must exceed 1. Its virtual +40 record must exist with byte +188 in {13,14}. Signed record short +322 must be nonnegative. At that offset in PC +36512, read a short length and following byte array. Length must be <=128 and greater than index. Pass only if bit `(skin-1)%8` is set in that byte.
  - Kind 2: scan equip positions 1..37 but inspect only 1,2,33,34. Position 2 uses item-record qword +2024. Positions 1/33/34 use qword +2440, from the base record or manager-slot-1455 transformed record when item virtual +392 >0 (variation from virtual +408). A nonzero linked key must resolve through slot 2547 to record short +8 equal skin. Any match passes. ❓ Linked-key argument registers and missing linked-record behavior require proof.
  - Kind 3: scan signed inventory byte +1032 entries from inventory +424 with stride 16. Resolve entry key through manager slot 2331. Missing record asserts/logs and continues. For positive entry count, examine up to that count of linked keys from record qwords +984 through +1056 (ten explicit positions). Resolve nonzero keys through slot 2547; matching record short +8 passes. Other indices do not supply a key. ❓ Oversized counts, linked-key registers and exact inventory entry field names remain open.
  - Other kinds log and fail.
- **Mutation helper `0x14026FFD0`:** Require PC kind 1 and current short +776 !=requested skin. Set property 200 through the part-1 property setter. Then append property 200 through `0x140273E10` if a delta buffer exists. Failure can occur after the property write. If daemon config +2736 byte +537 !=0, capture report data through `0x14060D810` and call `0x14020A4C0`. The task ignores this helper's return and still flushes and sends 1066.
- **❓ Missing:** Permission argument recovery and underlying inventory/table access, property setter/delta failure semantics, report helper, full flush recipients/broadcasts and persistence.

## 1208 — G_AutoCombatStartReqCmd — partial

- **381:** `0x1403DCBD0`. **348:** `-`.
- **Wire:** `hc`: start option byte, initialized to zero. Decode return is checked.
- **Checks/order:** Decode; require zone. Create task vtable `0x140BC2558`, kind 365 through `0x14028DF50`. Common setup; option +144. Call `0x140603BD0(parent,task,0,0)` and log if false.
- **Failure/output:** Decode, zone and scheduler failure log only. No wrapper failure/success packet or direct backend/kick call.
- **State:** No direct auto-combat flag change in wrapper.
- **Task `0x1402DC920`:** Capture PC report data through `0x14060D810`; obtain optional attached user and build report text. If signed current mode PC +37456 >0, call `0x1402DC440(...,PC,0)` first. Clear PC qwords +37448/+37464/+37472 and byte +37456. Set mode byte +37456 to requested option and start tick +37448 to relative server ticks. Call report helpers `0x1406A8B50` and `0x140205F00`. The task has no visible option-range/life/zone check or direct client send.
- **❓ Missing:** Scheduler, old-session close helper, report helpers and any indirect output. The task does not prove a broader automatic action/effect loop.

## 1209 — G_AutoCombatStopReqCmd — partial

- **381:** `0x1403DCD80`. **348:** `-`.
- **Wire:** `hcQ`: stop option:c, reason/option byte:Q; both locals start at zero. ❓ Typed names remain open.
- **Checks/order:** Decode; require zone. Create task vtable `0x140C3ED40`, kind 366 through `0x14028DF50`. Common setup; copy bytes +144/+145. Call `0x140603BD0(parent,task,0,0)` and log if false.
- **Failure/output:** Decode, zone and scheduler failure log only. No wrapper failure/success packet or direct backend/kick call.
- **State:** No direct auto-combat flag change in wrapper.
- **Task `0x1402DCC40`:** Map second byte 2/3/4/5/6 to close reason 1/2/3/4/5; all other values map to zero. Call `0x1402DC440(...,PC,mapped_reason)`. Then clear PC qwords +37448/+37464/+37472 and mode byte +37456. The first request byte is unused in this task. No direct client send appears.
- **❓ Missing:** Scheduler, close helper and any indirect report/output.

## Rust port status (part 1)

Source: `Program/Server/GameDaemon/src/handlers/skills.rs`, tests `tests/gs1_skills.rs`.

| Command | Rust state | Boundary |
| --- | --- | --- |
| 1226, 1228 | Full decode, four modifier groups, caster selector, queue; failure 1227/1229 echoes identity | Skill action |
| 1229, 1231 | Selector decode, caster, queue | Action |
| 1270, 77, 1272 | User, zone, PC kind, queue; 1272 sends 537 then 1 for a non-loot target | Gather/loot actions |
| 71 | Summon checks 2, 175, 179, then queue with summon zone | Recall action |
| 74 | Unsupported | Gauge state not modeled |
| 134, 135, 137, 139, 142 | Zone, caster kind, queue | Actions |
| 821 | Log only | — |
| 824 | Feature byte; disabled → 826 reason 1 with UTC | Recharge action |
| 113, 117, 1485, 1488, 1491, 1494 | Decode, range checks, PC kind, queue | Page actions |
| 1497 | Rate gate, all reasons in spec order, trait table through `Services::trait_max_level` | Train action |
| 1389 | 470, T2 → 7, zone → 1, wrong kind logs only | Unlock action |

## Coverage and remaining work

All 51 requested receive wrappers were read.
Only commands 131, 821 and 1231 are done.
The other 48 sections identify unread game paths with ❓.
No handler is not-started.
A missing DB procedure or broadcast name in a partial section means unresolved, not absent.
No anti-cheat or licence implementation was followed.
No source code, ledger, status file, build file or binary changed.
No test ran. This document does not establish complete Rust parity.

## Machine-readable state table

| id | state |
| --- | --- |
| 71 | partial |
| 74 | partial |
| 77 | partial |
| 83 | partial |
| 113 | partial |
| 117 | partial |
| 125 | partial |
| 127 | partial |
| 129 | partial |
| 131 | done |
| 134 | partial |
| 135 | partial |
| 137 | partial |
| 139 | partial |
| 142 | partial |
| 630 | partial |
| 633 | partial |
| 636 | partial |
| 788 | partial |
| 821 | done |
| 824 | partial |
| 837 | partial |
| 840 | partial |
| 952 | partial |
| 1026 | partial |
| 1029 | partial |
| 1032 | partial |
| 1035 | partial |
| 1038 | partial |
| 1053 | partial |
| 1062 | partial |
| 1065 | partial |
| 1208 | partial |
| 1209 | partial |
| 1226 | partial |
| 1228 | partial |
| 1229 | partial |
| 1231 | done |
| 1253 | partial |
| 1255 | partial |
| 1258 | partial |
| 1264 | partial |
| 1266 | partial |
| 1270 | partial |
| 1272 | partial |
| 1389 | partial |
| 1485 | partial |
| 1488 | partial |
| 1491 | partial |
| 1494 | partial |
| 1497 | partial |

