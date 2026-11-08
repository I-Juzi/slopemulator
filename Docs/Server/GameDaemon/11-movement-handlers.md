# 11 — Movement receive handlers, build 381.8867

## Scope and shared rules

This file extends doc 07, Step 5 part 1. It does not replace that exact subset.
Build 348.8349 supplies names. Build 381 supplies IDs and behavior.
This is a static spec. No Rust change or client test ran.
`done` means that all game-logic paths were read. `partial` lists unread paths.
Full formats include command `h`. They exclude the outer size field.
See doc 10 for scalar widths and dispatch rules.
All handlers here use authenticated-PC dispatch.
No body decoder means that body bytes are unused. It does not prove exact-length validation.
Unchecked decode returns do not establish a safe malformed-packet result.
No direct user-manager kick appears in the wrappers read here.
Do not reuse the BoardGacha kick condition. See doc 07 part 4.
No direct DB procedure appears in these wrappers. Deferred work can have other effects.
Do not infer cooldown, terrain, range or ownership rules from a task name.

### Common task setup

`0x14025DB30` attaches a zone to a task.
The wrapper stores caster object ID at task +128.
`0x14069BF60` adds the task as a caster child. Task +138 records that relation.
`0x140254FD0` queues the object on the zone (`Zone::PostZSO` in 348).
`0x14000B730` releases a reference. It is not a socket close.
`0x140472550` creates an action by class. `0x140471800` initializes its zone/caster.
Task creation and queue success do not prove operation success.

Common end helper `0x140422850` removes the action from the caster action manager through virtual +48, unlinks the child through `0x14069C1D0`, and releases the reference when required. It does not close the session.

Teleport-action broadcast helper `0x14023C550` and cave-escape helper `0x1402411F0` require a nonempty zone broadcast structure at +16232. They append an inner `cqQ` record: event type byte 48 (teleport) or 51 (escape), object ID:q, state byte:Q. States passed by these actions are 0=start, 1=accepted finish, 2=cancel/failure. These are inner event types, not proved top-level network command IDs. Both helpers obtain a spatial cell from x/y/z through `0x140212720` and queue through `0x140248A50`. Outer format literal is `hiCS;` at `0x140B674C8`. ❓ Full outer command/fields, grouping and recipient filter remain unread. Do not send command 48 or 51 from these constants alone.

## 66 — G_TurnReqCmd — partial

- **381:** `0x14038B990`. **348:** `HandleGame_TurnReq`, `0x14038AF50`.
- **Wire:** `hch`: command, control selector, yaw. Decode return is ignored.
- **Checks/order:** Selector 1 selects PC summon +36880 only if summon state +6984 is 1 or 2. Any other selector selects the PC. Require selected creature and its zone +2224. Allocate TurnTask. Require unsigned yaw <=359. Attach task and queue.
- **State:** Store yaw at task +144. No direct yaw update in the wrapper. Task vtable `0x140BB54D0`.
- **Failure:** Send 67, `hEc`, to the selected creature's owner. Missing zone explicitly sends reason 2 and request selector. Failed creature resolution sends through the PC. ❓ Other reason/selector arguments are omitted by the decompiler.
- **Output/calls:** No immediate success reply, broadcast or daemon call. Return true.
- **Task `0x140291D00`:** Clamp requested yaw to signed global bounds at `qword_140EE5418` +18928/+18936. Set caster property 102 through `0x140696F20`; if changed, add property change through `0x140273E10`. Call property broadcast `0x140260850`. Queue movement broadcast through `0x140236820`, centered at current position, with authoritative object ID/pose/yaw, duration zero, standing-social zero and one MoveType record. Its type is 1; startTime/startCell/diffx/diffy/diffz/eventJump are zero. Failure for caster native kinds 1..3 sends 67, `hEc`, with selector from caster virtual +600.
- **Deferred failure:** Task failure sends reason 168, verified at `0x140291E33`. The selector comes from caster virtual +600, not necessarily the original request byte.
- **❓ Missing:** Movement broadcast format/recipients, property broadcast details, exact global bound names and omitted wrapper failure arguments. No table check in wrapper.

## 99 — G_ActivatedTeleportListReqCmd — partial

- **381:** `0x140382820`. **348:** `HandleGame_ActivatedTeleportListReq`, `0x1403813A0`.
- **Wire:** No body decoder.
- **Checks/order:** Require zone +2224. Create ActivatedTeleportTask, vtable `0x140BB06D8`. Use common task setup and queue.
- **State/output:** The wrapper changes only task state and child relation. Missing zone or queue failure sends 101, `hh`, reason 2 to requester. Both reason arguments are verified in instructions. Return true. No immediate success or broadcast.
- **Task `0x1402962B0`:** Send 100, `hb`, to requester through PC packet-send virtual +584. Command immediate 100 is verified at `0x140296375`. `Pc::FetchActivatedTeleport`, `0x1405FE680`, obtains packed property field 1 from ActivatedTeleport at PC +29496 through `0x140697E50`. It returns the packed byte count, or zero on helper failure. The returned block is appended directly. Do not insert a second length prefix without the packed-property codec.
- **Packed body:** For the byte-array metadata types 13/14, shared reader `0x140697E50` writes signed-16 byte length followed by the raw flag bytes. Thus command 100 has one b block, not two. Bit mapping is in command 102. On metadata/capacity failure, FetchActivatedTeleport returns zero and the task still sends its assembled packet. Queue failure reason is 2, verified at `0x14038293B`.
- **❓ Missing:** ActivatedTeleport field-definition source and malformed/unavailable property contract. No task broadcast or direct backend request. Storage is `_activatedTeleportList` in 348.

## 102 — G_DetectTeleportSwitchReqCmd — partial

- **381:** `0x140382980`. **348:** `HandleGame_DetectTeleportSwitchReq`, `0x140381540`.
- **Wire:** `hh`: command, teleportSwitchId. Decode return is ignored.
- **Checks/order:** Require zone. Allocate DetectTeleportSwitchTask, vtable `0x140B97BE8`. Require native creature kind at PC +46 to be 1. Attach task, copy switch ID to +144, then queue.
- **Failure/output:** Missing zone sends 104, `hh`, reason 2. Wrong kind asserts/logs and enters failure reply. Initialize/queue failure sends 104, reason 2, verified in instructions. Return true. No immediate success, broadcast or daemon call.
- **Task `0x140296460`:** Look up ZoneTeleportSwitch at DataManager pointer slot 3303 by `(unsigned16(switchId)<<32) | geoZoneId`. Missing switch sends 104/1128. Compute 3D squared distance from PC signed x/y/z +268/+270/+272 to record signed x/y/z +24/+26/+28. Require distance squared <= signed radius at +30 squared; else 104/1129. Look up Teleport through DataManager slot 2883 using switch-record qword +32. Missing record or Teleport byte +24 nonzero sends 104/1128. Set activated bit with `0x1405FEA50(PC, Teleport record ID at +8, 1, true)`; false sends 104/2. Errors use `hh`, requester only.
- **Mutation helper:** Require signed teleportId>0 and valid property field 1 metadata of type 13 or 14. Current flag block must be <=256 bytes. Index `(teleportId-1)/8`, bit `(teleportId-1)%8`; extend with zeros if needed; set bit. `Property::Set` at `0x1406975B0` changes ActivatedTeleport at PC +29496. With notify=true, send 103, `hhc`: teleportId, activation byte 1. Then call cached persistence helper `0x140316F40`. The helper's return is not checked here. No direct broadcast or kick.
- **❓ Missing:** Exact record field names, table constraints that bound bit index to the 256-byte temporary array, Property::Set side effects, backend command/procedure and late failure behavior. Initialize/queue failure reason is 2, verified at `0x140382B0F`.

## 145 — G_ReturnToBaseCampReqCmd — partial

- **381:** `0x14038B800`. **348:** `HandleGame_ReturnToBaseCampReq`, `0x14038AD80`.
- **Wire:** No body decoder.
- **Checks/order:** Require zone. Create ReturnToBaseCampTask, vtable `0x140BC1778`. Attach and queue through common setup.
- **State/output:** No direct location change. Missing zone or failed queue sends 146, `hh`, reason 2 to requester. Return true. No immediate success or broadcast.
- **Task `0x140291C80`:** For parent native kind 1, call `Pc::ReturnToBaseCamp` at `0x1405E6EE0` with second argument zero. If it returns false, send 146, `hh`, reason 284. Other parent kinds do nothing in the task.
- **❓ Missing:** ReturnToBaseCamp helper, camp eligibility and destination selection, data tables, spawn chain, success outputs and DB/daemon work.

## 296 — G_EnterPortalReqCmd — partial

- **381:** `0x140393D70`. **348:** `HandleGame_EnterPortalReq`, `0x140393DD0`.
- **Wire:** No body decoder. No portal ID comes from this wrapper.
- **Checks/order:** Require zone. Create EnterPortalTask, vtable `0x140BC1458`. Require PC native kind +46 equal to 1. Attach and queue.
- **State/output:** No direct position change. Missing zone, wrong kind or queue failure sends 297, `hh`, reason 2 to requester. Wrong kind also asserts/logs. Return true.
- **Task `0x140296620`:** Use signed environment index at PC +2250. Require index>0, an entry in zone environment array +16336, and record subtype short at entry-record +2 equal to 7. Else send 297/1152. PC life-state byte +578: state 0 continues; states 1/3 send 297/1155; state 2 first calls creature virtual +128 and processes changed properties through `0x140260850`, then still sends 297/1155; other nonzero states assert/log and send 297/2. For valid environment and state 0, call `0x1405A96F0(environment, PC)`. Nonzero returned reason passes to `0x1406035B0`. Direct task errors use `hh`, requester only.
- **❓ Missing:** Environment record field names, virtual +128 exact state mutation, portal helper and reason converter, range/ownership/cooldown rules, spawn effects, success/broadcasts and DB/daemon calls.

## 298 — G_TurnBackFromPortalReqCmd — partial

- **381:** `0x140393F80`. **348:** `HandleGame_TurnBackFromPortalReq`, `0x140393FD0`.
- **Wire:** `huuuh`: command, x/y/z shorts, yaw:h. Decode return is ignored.
- **Checks/order:** Require zone. Create action class 33 (`ACTION_WARP` in 348). Initialize zone/caster. Store pose at action +152/+154/+156/+158, set +160 to 1 and short +164 to zero. Queue.
- **Failure/output:** Missing zone, action creation failure or queue failure sends 299, `hh`, reason 2. Return true. No immediate success or broadcast.
- **❓ Missing:** WarpAction start/continuations, terrain/range validation, authoritative state updates, success/broadcasts and DB/daemon effects. The wrapper alone does not authorize arbitrary coordinates.

## 648 — G_ReturnToGameReqCmd — partial

- **381:** `0x1403BFB20`. **348:** `HandleGame_ReturnToGameReq`, `0x1403BAA30`.
- **Wire:** No body decoder.
- **Checks/order:** Arena config +237 and +248 rejects with 650, `hh`, reason 5. Otherwise lock PC +37480. Require arena-session +37520 equal to 4. On match, set it to 1 and clear qwords +37544 and +37528. Unlock before backend send.
- **Daemon call:** On match, call cached-channel send `0x1406A7400` using routing value PC +20664. Backend command is 271, exact format `hQqq` at `0x1409C9E38`. Arguments are constant byte 2, PC DB ID +20744:q and PC runtime object ID +40:q. The call at `0x1403BFC17` verifies all arguments. No local rollback occurs in this wrapper.
- **Failure/output:** Wrong state sends 650, `hh`, reason 2. Return true. No immediate success or broadcast.
- **❓ Missing:** Backend response route, success/failure packets, DB procedure and late rollback.

## 1210 — G_ClientRunReqCmd — partial

- **381:** `0x1403DCF40`. **348 name:** `-`.
- **Wire:** `hcc`: command, byte A, byte B. Both bytes initialize to zero before decode.
- **Checks/order:** Decode success only. No movement action, coordinate input or rate check appears in this wrapper.
- **State/calls:** Build a server-side report through `0x14060D810`. Obtain optional user metadata. Pass report fields and both request bytes to `0x140368150`. Release temporary storage and user reference.
- **Failure/output:** Decode failure logs only. Return true. No direct client reply, broadcast or game-position mutation.
- **❓ Missing:** Meaning of the bytes, report-helper side effects and final logging/backend call. Do not treat the command name as proof that it starts running.

## 1221 — G_WarpToPosReqCmd — done

- **381:** `0x14001AFE0`. **348 name:** `-`.
- **Wire:** No decoder. Body bytes are unused.
- **Checks/state:** No checks after PC dispatch. Return true. No warp or other state change.
- **Output/dependencies:** No reply, broadcast, failure, kick, table lookup, daemon request or DB call.

## 1239 — G_UseCaveEscapeReqCmd — partial

- **381:** `0x140378190`. **348:** `HandleGame_UseCaveEscapeReq`, `0x140376580`.
- **Wire:** `hh`: command, grocery inventory position. Decode return is ignored.
- **Checks/order:** Require zone; read `_isTakingAchievementReward` at +21096 under lock +21088; read `_isUsingJumpingCharacter2Ticket` at +13752 under lock +13744 if needed; reject either flag with 254. Read trying-instant-payment flag +21080 under lock +21072; reject with 255. Create action class 52 (`ACTION_ESCAPE_FROM_CAVE`). Recheck current PC zone equals captured zone. Require native kind +46 equal to 1. Initialize zone/caster and grocery position at action +152. Queue.
- **Failure/output:** Send 1241, `hh`, to requester. Missing zone or action create/initialize/queue failure uses 1. Flag failures use 254/255 as above. Return true. No direct location or inventory mutation in wrapper.
- **Start `0x140494F60`:** If PC byte +13740 is nonzero and zone helper `0x140254F60` rejects the current job/state, send 1241/252 and end. Resolve inventory object with `0x1404CAD00(PC+15000,2)`. Require object, virtual +104 result 3, nonnull record from virtual +112, record byte +2008 equal to 4, `0x140565FC0(item)` success, and item byte +24 clear. Failure sends 1241/253 and ends through `0x140422850`. Set item byte +24 to 1 before any success output.
- **Accepted start:** Queue escape broadcast `0x1402411F0` with current pose, PC object ID and state byte 0. Let duration be rule config +80. Send 1240, `hh`, with signed-short duration/100. Positive duration schedules `0x140674A00`; zero calls action virtual +208; negative does not schedule here.
- **Cancellation/failure:** `0x1404954F0` queues broadcast state byte 2 only for nonzero reason, re-resolves the inventory object and clears byte +24 if present, then removes attached action via `0x140676850`. `0x140495590` sends 1241, `hh`, reason 4.
- **Finish `0x1404951F0`:** Permit current zone subtype 34, or category byte +40 equal to 3 and subtype in {26,34,38,77,177}. Also permit captured zone category 2/subtype 77. Else broadcast state 2 and end, without direct error reply. Obtain destination key through `0x1403FBB60(capturedGeoZone, geoZoneId, PC)` and resolve DataManager slot 3243. Missing record or false `0x1404955C0(action)` also broadcasts state 2 and ends. The latter helper runs BEFORE LeaveAction allocation. Create LeaveAction class 3 and initialize via `0x1404C2920`, leave mode 3, destination record i32 +8 / short +12, movement reason 8, other options zero. Successful init posts through `0x1404C3090` and broadcasts state 1. Create/init failure broadcasts state 2. End through `0x140422850` and release references.
- **Consumption `0x1404955C0`:** Re-resolve grocery item through `0x1404CAD00`. This helper takes depot byte 2 and a position short. The 348 type guide calls it `Inventory::GetItem`; ❓ the optimized build-381 call's omitted position register needs a separate instruction check. If the item exists, clear reserved byte +24 and build one dispose order: action grocery position, count 1. Call `Inventory::CheckDispose` at `0x1404D84E0` with one order and reason 18, then `Inventory::Dispose` at `0x1404D8660` with the same arguments. False returns stop finish before travel. Successful disposal can write a GameLog::UseItem report through `0x140209970`. This precedes LeaveAction allocation. If the item lookup fails, native order locals can be uninitialized; do not invent a safe malformed-state result.
- **❓ Missing:** Omitted caller position register, inventory dispose checks/mutations, record fields and CheckUsableZone helper, destination selector, cooldown, LeaveAction spawn, complete broadcast framing/recipients, DB work and late failures. No restoration after failed LeaveAction allocation appears in this finish function. Do not omit the item busy flag or assume the start packet consumes the item.

## 64 — G_MoveReqCmd — partial

- **381:** `0x1403892E0`. **348:** `HandleGame_MoveReq`, `0x1403887C0`. Build-381 logs call it WalkReq. Dispatch proves the MoveReq name.
- **Wire:** Literal at `0x140BDB7B0`: `hcuuuuuuiiiiiihhhcCQcccccch;Chhh;`. Fixed body: control selector:c; start x/y/z and end x/y/z:u; start x/y/z and end x/y/z in UU:i; duration:h; yaw:h; moveKey:h; broadcast:c. First counted group: type:Q, sprintJumpToDeceletion:c, startTime:c, startCell:c, diffx:c, diffy:c, diffz:c, eventJump:h. Normalize the sprint byte to Boolean. Second counted group: cell cx:h, cy:h, z:h.
- **STOP:** After fixed decode, the user movement-count block matches 348 `User::CheckBotMovePacketReceived` / `CheckBotSummonedMovePacketReceived`. It is inlined in 381 before controlled-creature resolution. Call instruction `0x1403895C7` targets `0x1403EAB50`. This anti-bot path is excluded. Do not port its internals from this spec.
- **Ordered ordinary checks:** Fixed decode (reason 2); selector 1 resolves summon +36880 only in state 1 or 2, all other selectors resolve PC (2); summon native kind 2 with entry +6960 and race byte +56 equal to 11 rejects (166); unsigned move-type count <=12 (103); decode each move-type entry (2); unsigned cell count <=36 (2); decode cells (2), but incremented parsed count >=36 rejects (102); actual cell count must equal declared count (102); selected creature must have zone (2).
- **Action routing:** Native kind 1 creates class 34 and calls PC initializer `0x14048A1E0`. Kind 2 creates class 51 and calls summon initializer `0x140490430`. Other kinds return true without action. Create/initialize/queue failure sends corrective reason 2. PC initialization gets all fixed fields, both arrays and broadcast. Summon initialization omits UU coordinates. Queue through `0x140254FD0`.
- **State/checks:** Extend doc 07 Step 5 part 1 for duration <=550, type transitions, speed bounds, height validation and duration history. Its PC start `0x14048A890`, transition `0x14048C0B0`, height `0x14048CDB0`, summon start `0x140490A80` and duration-history helper `0x1405D87D0` remain the evidence source. Do not replace those checks with distance alone.
- **Failure/output:** `0x14038BE00` sends 65, `hEchhhh`, reason, controlled selector and authoritative x/y/z/yaw to selected creature's owner. Fixed-decode or creature-resolution failure targets PC. Return true after each handled path. No direct DB/other-daemon request or user-manager kick in ordinary wrapper.
- **❓ Missing:** All action continuations and interruption branches, complete cell loader and transition predicates, successful movement broadcast ID/body/recipients, and backend effects. This is not a complete movement implementation spec.

## 1219 — G_AirDashReqCmd — partial

- **381:** `0x140389F40`. **348:** `HandleGame_AirDashReq`, `0x140389400`.
- **Wire:** `hq`: command, airDashId:q. Decode return is ignored.
- **Checks/order:** Require PC zone; create action class 36 (`ACTION_AIRDASH` in 348); recheck current zone equals captured zone; require PC native kind 1; initialize common action state; store airDashId at action +152; queue.
- **State/output:** No direct position change. Every wrapper failure sends 1220, `hh`, reason 2 to requester. Return true. No immediate success, broadcast, backend request or kick.
- **Prepare `0x140480630`:** Interpret low 32 bits of airDashId as signed environment index. Require 1..255, nonnull zone environment entry (array +16336), nonnull entry record +48, and record subtype +2 equal to 11 or 12. Failure sends 1220/169 and prepare returns false. Subtype 11 calls `AirDash::CheckAvailableAirDash`, `0x1405AB690`; zero succeeds, else send returned reason. Subtype 12 requires environment-record pointer +56 short +12 to equal signed PC environment index +2250 (169), env byte +120 nonzero (171), helper `0x14059A9C0(PC+19744,65002)` nonzero (170), and env qword +136 either zero or equal to signed PC byte +344 (174). Replies use `hh`.
- **Start `0x1404808A0`:** Recheck environment/index/subtype. Invalid sends 1220/169, then ends through `0x140422850`. Subtype 12 calls `0x1405ABF10`; false sends 1220/171, then ends. Subtype 11 calls `0x1405AB8B0(env, PC, &duration)`; false sends 1220/171, then ends. If duration<=0 call virtual +208. If positive, invoke effect helper `0x1405BB850` and schedule `0x140674A00(duration)`. Summon state 1/2 and job byte +100 equal to 6 runs the dash helper for the summon too. Job 9 calls summon virtual +128 and sends changed properties if it reports a change.
- **Subtype-11 availability `0x1405AB690`:** Apply subtype-12 environment index/enable/helper/faction checks with the same reasons. Then require signed environment i32 +92 <=0 or PC i32 +552 >= that value (172). If zone +16816 exists, require `0x1401035B0` to accept env/PC (173). Otherwise return zero. ❓ The typed field meanings and the latter helper remain unread.
- **Finish `0x140480C10`:** In daemon mode 2 and caster native kind 1 or 2, store current UTC seconds at caster +3400, then call `0x14025F0C0(caster,1,0)`. End through `0x140422850` on every path.
- **❓ Missing:** Dash/effect/helper internals, final common cleanup, environment/table field names, range/state/cooldown rules, broadcast bodies/recipients and backend effects. Unchecked decode does not define the q on malformed input. Do not copy build-348 effect IDs or failure reasons.

## 1223 — G_UseTeleportReqCmd — partial

- **381:** `0x140382B50`. **348:** `HandleGame_UseTeleportReq`, `0x140381700`.
- **Wire:** `hhh`: command, startTeleportId:h, destTeleportId:h. Decode return is ignored.
- **Checks/order:** Reject arena config or PC arena session 2/3/5/6 (reason 5); decode; require zone (2); achievement-reward flag +21096 or jumping-ticket flag +13752, under their locks, rejects (1144); trying-instant-payment +21080 under its lock rejects (1145); create action class 49 (`ACTION_TELEPORT`); recheck current zone equals captured zone; require native PC kind 1; initialize common action state; copy start/destination IDs to action +152/+154; queue. Action create/init/queue failure uses 2.
- **Failure/output:** Send 1225, `hEq`, reason and q zero to requester. Return true. No direct teleport-bit/location update, broadcast, DB request, daemon request or kick in wrapper.
- **Start:** `0x140488C70`, action vtable `0x140CAA328`. All early failures send 1225, `hEq`, then end through `0x140422850`. Unless stated, q is zero. Instructions verify reason 1132 for the first combat/state checks and reason 1148 for unavailable type-242 event.

| Order | Build-381 check or action | Failure reason |
| ---: | --- | ---: |
| 1 | Daemon mode +2752 equal to 2: require PC +1898 clear. Other modes: require PC +888 bit 0 clear | 1132 |
| 2 | Require PC +1897 clear | 1132 |
| 3 | Recheck achievement/jumping flags, then instant-payment flag | 1144 / 1145 |
| 4 | Destination activation helper `0x1405FE950` accepts destination ID | 1136 |
| 5 | Daemon-relative tick >= PC deadline +20832 | 1139; q = UTC now + floor(remaining ticks/1000) |
| 6 | Start Teleport record exists in DataManager slot 2883; byte +24 is zero | 1135 |
| 7 | Destination Teleport record exists; byte +24 is zero | 1136 |
| 8 | Destination record qword +32 resolves through DataManager slot 3291 | 1134 |
| 9 | PC byte +13740 permits target geo-zone under current-zone helper `0x140254F60` | 1131 |
| 10 | `0x14060DB50(PC, destination geo-zone)` returns zero | Return 2→1141, 3→1143, 8→1142, other nonzero→2 |
| 11 | Resolve destination zone record via DataManager slot 3343. Apply type-93/event/time branch and type-242/event/time branch | 1148 / 1147 with absolute UTC q |
| 12 | Destination Teleport qword +48 is zero or equals selected PC faction byte (+344, or +328 after event branch) | 1134 |
| 13 | Positive signed-byte requirements +57 and +58 do not exceed PC level +276; requirement +59 does not exceed `0x140260E70(PC+348)` | 1134 |
| 14 | Start-position helper `0x140489960(action, &startRecord)` succeeds | 1133 |
| 15 | PC currency qword +312 >= calculated cost | 1137 |

- **Cost:** Let P be `qword_140EE58A0`. Native signed arithmetic computes `P.i32[22] * (start.i32@44 + dest.i32@44) * (P.i32[21] + abs64(8 * (start.i16@40 - dest.i16@40))) / 10000`. Store cost at action +160 before currency comparison. Do not add a minimum fee.
- **Accepted start:** Queue teleport-action broadcast `0x14023C550` at current PC position with object ID and state byte 0. Send 1224, `hhhh`: start ID, destination ID, signed-short `P.i32[20]/100`. If duration P.i32[20]>0 schedule timer; if zero call virtual +208; if negative do not schedule here. This is a start notification, not proof of completed travel.
- **Cancellation/failure:** `0x14048A0D0` queues broadcast state byte 2 only when its reason argument is nonzero, then removes the action through `0x140676850` if attached. `0x14048A150` sends 1225, `hEq`, reason 4, q zero. `0x14048A190` is the cancellation-availability predicate: values 0..2 pass; 3 passes outside daemon mode 2; 4 passes only in mode 2; other values fail.
- **Start-position helper `0x140489960`:** Resolve requested start's location record through DataManager slot 3291. Compute distance helper `0x140489810` from PC geo-zone and unsigned x/y. Iterate all enabled Teleport records (byte +24 zero), except the requested start. If another resolved location has strictly smaller computed distance, fail. Equal distance is allowed. This is a nearest-teleport rule, not a fixed radius. Distance helper `0x140489810` resolves both geo-zones through DataManager slot 3343. Missing either logs and returns zero, not infinity. Resolve each zone's region key through slot 2919. If present, add region i32 +24/+28 to signed-short x/y. Return 64-bit squared XY distance; ignore Z. The call signatures and signed-short conversion control coordinate sign.
- **Finish `0x140489BA0`:** Re-resolve destination Teleport and its location. Missing record/location, or currency now below stored cost, queues broadcast state 2 and ends without a direct failure reply. Recheck type-242 event eligibility; rejection sends 1225 with reason 1148, then ends. The reason is verified at `0x140489D56`. Optional game-log branch calls `0x1406A6E10` (❓ exact logging packet layout). Create LeaveAction class 3; set destination teleport short at LeaveAction +54412. Resolve named Teleport enter/leave table values. Initialize through `0x1404C2920`, leave mode 3, target geo-zone, movement reason 5 and those table values. On success, attach stored cost/object ID to LeaveAction +208/+216 and enable cost processing through `0x1404C20D0(action,1)`. Post through `0x1404C3090`. Then set PC teleport deadline +20832 to daemon-relative tick + P.i32[23] - 1000*`0x1405EC080(PC,49)`. Queue broadcast state 1. Create/init failure queues state 2 instead. End and release references on both branches. Cost is deferred to LeaveAction, not directly deducted here.
- **Activation reader `0x1405FE950`:** Read field 1 in ActivatedTeleport at PC +29496. Require field count>1, field metadata type 13/14, nonnegative property offset and flag length<=256. For valid positive destination IDs, return whether length exceeds `(id-1)/8` and the bit `(id-1)%8` is set. Unlike the setter, it has no initial signed-id>0 guard. ❓ Negative/nonpositive IDs can reach native array/shift behavior; no safe invented result is specified.
- **❓ Missing:** Exact event/zone-type rules and table field names, malformed activation IDs, LeaveAction spawn, currency consumption and rollback, complete broadcast framing/recipients and backend procedures.

