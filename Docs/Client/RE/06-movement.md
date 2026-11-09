# 06 — Client.exe 381: movement and position sync

Summary:

1. The local player and the summoned cat each have one `MoveListMaker` (14,920 bytes). It records the engine positions (`EOWorldInfo`, 68 bytes) every tick, and it flushes them to GameD as `G_MoveReqCmd` (command 64, 88-byte packet) in `MoveListMaker::sendWalkPacket` (`0xcc4ce0`).
2. The send interval (`_moveIntervalTime`) is 0.5 s. When the option byte `Config+16347` is set, it is 0.25 s (`MoveListMaker::MoveListMaker` `0xc61730`). Other events flush the list early (`SendWalkPacketReason`, §2.2).
3. Position compression: `EngineUnitToServerUnit` (`0x4014b0`) is `floor(engine / 4)`. The packet has the start and end position two times: as i16 server units and as i32 engine units (truncated). The move type list has i8 deltas in server units and a start time in 5 ms steps (max 110).
4. The 38 move type makers (`MoveTypeMaker::MoveTypeId`, vtables `0x15e97b4` … `0x15e98dc`) convert engine moves into wire move types (`G_MOVETYPE_*`, 0–43). A packet has max 12 move types and max 32 cells.
5. The server answers only on error: `G_MoveFailCmd` (65) and `G_ChangeMoveKeyCmd` (73). The `moveKey` (i16) is an epoch counter: the client puts it in every move, and the server changes it when it forces a position (§3).

```
 input / engine tick                                  network
 BPTPlayer::setMoveByKeyboard 0xb2a2f0 ─┐
 BPTPlayer::setMoveByAuto     0xaff4e0 ─┤
                                         ▼
 MoveListMaker::Tick 0xcc93d0 ─► playerTick 0xcc82b0 / summonedTick 0xcc7f30
        │ checkMove 0xcb26d0, checkRotate 0xc5b3e0
        │ makeCell 0xc74470 ─► _cellList[32]
        │ _moveTypeMaker[id]->Get (vtable slot 1) ─► MoveTypeMaker::SetMoveType 0xc5ac10 ─► _moveTypeList[12]
        │ _accumTime > _moveIntervalTime (0.5 s / 0.25 s) or SendWalkPacketReason ≠ 0
        ▼
 MoveListMaker::sendWalkPacket 0xcc4ce0 ── G_MoveReqPacket (cmd 64, vtable 0x15e9ae4)
        ▼
 BNSClient::SendToGamed 0x114ce20 (ch. 05 §1.4) ─► GameD
 GameD ─► G_MoveFailCmd 65 → HandleMoveFailPacket 0xef1870
       ─► G_ChangeMoveKeyCmd 73 → HandleChangeMoveKeyPacket 0xef1770
```

Sources: all facts come from the 381 database `ida/lanes/cx/Client.exe.i64`, unless the text says "419 GM". The 381 database has the struct types `MoveListMaker`, `EOWorldInfo`, `MoveTypeData`, `PTCell`, `PortalDesc`, `HyperMove`, `Glide`. We checked their offsets against the 381 code (for example `sendWalkPacket` reads `_controlId` at +14800 and `_moveKey` at +14810). ❓ marks a fact that we did not verify.

Related chapters: command table and packet field table: [05-game-dispatch](05-game-dispatch.md), [`game-commands.tsv`](game-commands.tsv), [`packet-layouts.tsv`](packet-layouts.tsv). Old notes: [client-exe-reversing §4](../client-exe-reversing.md) (this chapter replaces §4.1, §4.5 and updates §4.3, §4.4).

Files of this chapter:

| File | Content |
| --- | --- |
| [`06-movement-enums-gm.tsv`](06-movement-enums-gm.tsv) | every value of the movement enums and of the `G_MOVEFAIL`/`TURNFAIL`/`AIRDASHFAIL`/`WARPTOPOS` errors (419 GM names); `addr_381` = a 381 function that uses the enum |
| [`06-movement-functions.tsv`](06-movement-functions.tsv) | named functions, vtables and globals of this area, with 381 addresses |
| [`06-movement-makers.tsv`](06-movement-makers.tsv) | the 38 move type makers: id, vtable, Get function, wire move types that it writes |

---

## Quick find

| Task | Function | 381 addr | 419 GM name | Section |
| --- | --- | --- | --- | --- |
| Find the move list layout | `MoveListMaker` type | — | `MoveListMaker` | §1.1 |
| Convert engine units to server units | `EngineUnitToServerUnit` | `0x4014b0` | same | §1.5 |
| Find the per-tick move recorder | `MoveListMaker::Tick` / `playerTick` | `0xcc93d0` / `0xcc82b0` | same | §2.2 |
| Build and send `G_MoveReqCmd` 64 | `MoveListMaker::sendWalkPacket` | `0xcc4ce0` | same | §2.1 |
| Flush with a reason | `sub_CC55B0` | `0xcc55b0` | ❓ | §2.2 |
| Make cells | `MoveListMaker::makeCell` / `SetArea` | `0xc74470` / `0xc5b8f0` | same | §2.3 |
| Map engine moves to wire move types | maker vtables, slot 1 `Get` | `0x15e97b4` + 8·i | `MoveTypeMaker::MoveTypeId` | §1.4 |
| Send a turn (`G_TurnReqCmd` 66) | `MoveListMaker::checkRotate` | `0xc5b3e0` | same | §2.4 |
| Handle `G_MoveFailCmd` 65 | `walkFail` | `0xaa33a0` | `BPTPlayer::walkFail` `0xc5e210` | §3.5 |
| Handle `G_ChangeMoveKeyCmd` 73 | `BPTPlayer::ProcessNetObjectEvent` case 73 | `0xb15dd0` | same | §3.4 |
| Snap the player to the server position | `SetWorldInfo` (vslot 81) | `0xafb560` | `BPTPlayer::SetWorldInfo` | §3.2 |
| Reset the move list to a position | `MoveListMaker::SyncServerMove` / `setPlayerMove` | `0xc5b890` / `0xc5b180` | same | §3.3 |
| Server-driven dash, knockback, mount | `BPTPlayer` vslots 73–76 | `0xaaa440`, `0xaaa3a0`, `0xaaa2c0`, `0xaaa270` | `MoveKnockBack`, `MoveDash`, `MoveMount`, `MoveTurnBack` | §3.6 |
| Sync after a rule move | `BPTPlayer::SyncRuleMoveLocation` | `0xaaa110` | same | §3.6 |
| Sprint energy drain and refill | `HyperMove` tick | `0xe94110` | `HyperMove::Tick` | §4.1 |
| Start / end sprint | `HyperMove::Start` / `End` | `0xe93fc0` / `0xe93ee0` | same | §4.1 |
| Block sprint | `HyperMove::Block` | `0xe9d1c0` | same | §4.1 |
| Combat sprint request (1053) | `BPTPlayer::RequestCombatHyperMove` | `0xaf2510` | same | §4.2 |
| Block glide | `Glide::BlockGlide` | `0xee0640` | same | §4.3 |
| Fail reason names | G_MOVEFAIL enum (381 = GM − 5) | `0xaa347c` (381 table) | `$18A4E27B71DB8B3296A36A5F753469C9` | §3.5 |

Search anchors (381):

- `G_MoveReqPacket` vtable `0x15e9ae4`; `G_MoveReqPacket::FieldDefine` Def `0x19dbfb0`.
- Move key fields: search the byte patterns `[esi+3D99h]` and `[esi+3D9Ah]` (player `_isValidMoveKey` / `_moveKey`).
- The 850.0 trace range: `flt_1871B60` (`0x1871b60`).
- The sprint refill constant 500.0 in `0xe94110` (`a2 * 500.0`).
- The switch `add eax, 0FFFFFF9Ch` (−100) at `0xaa33a0` is the move fail switch.
- Strings: `Msg.Teleport.Fail.Sprint` `0x14e4fa0`, `Msg.EnterZone.Sprint.Infinity` `0x16b03ac`.

---

## 1. Data structures

### 1.1 `MoveListMaker` (14,920 bytes, 381 type, offsets checked in `0xcc4ce0`, `0xc61730`)

| Offset | Size | Field | Type | Meaning |
| ---: | ---: | --- | --- | --- |
| 0 | 13600 | `_moveHistoryList` | `EOWorldInfo[200]` | Positions of this send interval. Entry `[_moveHistoryCount-1]` is the end position of the packet. |
| 13600 | 4 | `_moveHistoryCount` | int | Count. `playerTick` adds an entry only when the count is < 200 (`0xcc82b0`). |
| 13604 | 12 | `_curPos` | Vector | |
| 13616 | 68 | `_rotate` | EOWorldInfo | Last turn state (for `G_TurnReqCmd`). |
| 13684 | 4 | `_yaw` | int | Engine yaw (0–65535). `sendWalkPacket` converts it to degrees. |
| 13688 | 12 | `_nextCell` | PTCell | |
| 13700 | 384 | `_cellList` | `PTCell[32]` | Cells that the path goes through. PTCell = 3 × i32 (`_x`, `_y`, `_z`). |
| 14084 | 4 | `_cellListCount` | int | |
| 14088 | 480 | `_moveTypeList` | `MoveTypeData[12]` | Move type changes (§1.3). |
| 14568 | 4 | `_moveTypeCount` | int | Max 12 (`MoveTypeMaker::SetMoveType` `0xc5ac10`). |
| 14572 | 4 | `_accumTime` | float | Seconds since the last send. |
| 14576 | 4 | `_duration` | float | Seconds that the packet covers. Sent as ms. |
| 14580 | 4 | `_moveIntervalTime` | float | 0.5 s, or 0.25 s when `Config+16347` ≠ 0 (`0xc61730`). Also the upper limit of `startTime`. |
| 14584 | 1 | `_ruleMove` | bool | Server-driven "rule move" (no input). |
| 14585 | 1 | `_playerMoveState` | bool | |
| 14586 | 1 | `_playerRotateState` | bool | |
| 14588 | 4 | `_lastYaw` | int | |
| 14592 | 1 | `_summonedMoveState` | bool | |
| 14596 | 4 | `_fallingStartZ` | float | |
| 14600 | 4 | `_prevMoveZ` | float | |
| 14604 | 4 | `_prevMoveType` | int | |
| 14608 | 1 | `_sendWalkPacketReason` | char | `MoveListMaker::SendWalkPacketReason` (§2.4). |
| 14609 | 1 | `_inPortal` | bool | |
| 14612 | 8 | `_originX`, `_originY` | int | Area origin (`SetArea` `0xc5b8f0`). |
| 14620 | 24 | `_areaMin`, `_areaMax` | Vector | Valid area box (`isValidArea` `0xc57220`). |
| 14644 | 4 | `_rotateSendTime` | float | |
| 14648 | 152 | `_moveTypeMaker` | `MoveTypeMaker*[38]` | One maker per `MoveTypeMaker::MoveTypeId` (§1.4). |
| 14800 | 1 | `_controlId` | char | 0 = player, 1 = summoned (cat). Goes to the packet byte +16. |
| 14804 | 4 | `_handle` | `EInterface::Handle` | Engine actor handle (`SetHandle` `0x50cf00`). |
| 14808 | 1 | `_broadCastMove` | bool | Toggles each packet when `Config+16347` ≠ 0. |
| 14809 | 1 | `_isValidMoveKey` | bool | |
| 14810 | 2 | `_moveKey` | i16 | Move epoch from the server (§3). |
| 14812 | 12 | `_turnBackPos` | Vector | Position to go back to from a portal. |
| 14824 | 4 | `_turnBackYaw` | int | |
| 14828 | 1 | `_isLink` | bool | |
| 14832 | 32 | `_portalDesc` | PortalDesc | `reason` +0 i32, `acquireQuestId` +4 i16, `disposeQuestId` +6 i16, `caseIdx` +8 i8, `transitZoneId` +16 u64, `envDataId` +24 i32. |
| 14864 | 1 | `_enterPortal` | bool | When set, `playerTick` does not record moves. |
| 14865 | 1 | `_enterArenaLobby` | bool | Same. |
| 14868 | 16 | `_moveDebugInfo` | std::vector | |
| 14884 | 4 | `_deltaTime` | float | |
| 14888 | 16 | `_moveDebugInfo2` | std::vector | |
| 14904 | 16 | `_moveInfoTimer` | PreciseTimer | |

### 1.2 `EOWorldInfo` (68 bytes, one engine position sample)

| Offset | Field | Type | Meaning |
| ---: | --- | --- | --- |
| 0 | `MoveType` | int | `EOWorldInfoMoveType` (0 Walk … 15 EventJump, 16 Num). |
| 4, 8, 12 | `LocationX/Y/Z` | float | Engine units (UU). |
| 16, 20, 24 | `ScaleX/Y/Z` | float | |
| 28, 32, 36 | `RotationYaw/Pitch/Roll` | int | Engine angle, 65536 = 360°. |
| 40 | `fTimeStamp` | float | |
| 44 | `InEnvVolume` | int | Event-jump id when `MoveType` = 15 (`0xc5ac10`). |
| 48 | `ZoneID` | int | |
| 52 | `EnvID` | int | |
| 56 | `bSprintJtoJDeceleration` | bool | Copied to `MoveTypeData.isDecelFalling`. |
| 60 | `GroundSpeed` | float | |
| 64 | `DistanceForMoveHistory` | float | |

### 1.3 `MoveTypeData` (40 bytes, one entry of `_moveTypeList`)

Writer: `MoveTypeMaker::SetMoveType` (`0xc5ac10`). It writes entry `[n]` and the diff of entry `[n-1]`, then increments `n`. It does nothing when `n` ≥ 12.

| Offset | Field | Type | Meaning |
| ---: | --- | --- | --- |
| 0 | `moveTypeMakerId` | i8 | `MoveTypeMaker::MoveTypeId` (§1.4). Not sent. |
| 1 | `moveType` | i8 | Wire `G_MOVETYPE_*` value. |
| 2 | `startCell` | i8 | Index into `_cellList` (`_cellListCount - 1` at the time of the change, `0xcc82b0`). |
| 4 | `startTime` | float | Seconds since the packet start. |
| 8, 12, 16 | `diffX/Y/Z` | float | Engine delta from this entry to the next entry (the last entry: to the end position, set in `sendWalkPacket`). |
| 20, 24, 28 | `locationX/Y/Z` | float | Engine position at the change. |
| 32 | `yaw` | int | `abs(yaw) & 0xffff`. |
| 36 | `eventJump` | i16 | Event-jump id. Only when `EOWorldInfo.MoveType` = 15, else 0. |
| 38 | `isDecelFalling` | bool | From `EOWorldInfo.bSprintJtoJDeceleration`. |

### 1.4 Move type makers (`MoveTypeMaker`, 8 bytes: vtable +0, `_controlId` +4)

`MoveListMaker::MoveListMaker` (`0xc61730`) allocates 38 makers (8 bytes each) and stores them in `_moveTypeMaker[0..37]` in vtable order. Each vtable has 2 slots: slot 0 = destructor (`0x4d6c20` or `0x5ac520`), slot 1 = `Get` (the state function). The state function looks at the engine sample and the old maker, and calls `SetMoveType` with the new maker id and the wire move type. Full list: [`06-movement-makers.tsv`](06-movement-makers.tsv).

Wire move type from maker id (checked from the immediate operands at every `SetMoveType` call in the 38 `Get` functions):

| Maker id (`MoveTypeId`, 419 GM name) | Wire `moveType` (`G_MOVETYPE_*`) |
| --- | --- |
| 0 Walk … 18 GlideEnd | same value (0–18) |
| 19 BoostStart … 33 EventJumpEnd | maker id + 3 (22–36). Wire 19–21 (`NPC_CHASE`, `NPC_FLEE`, `NPC_RETREAT`) are for NPCs only. |
| 34 CombatRun, 35 CombatSwim, 36 CombatFall | 37, 38, 39 (`SUMMON_COMBAT_*`, the cat) |
| 37 NotMove | no `SetMoveType` call (`Get` `0xca7460` only calls the others) ❓ |

Wire 40–42 (`ATTACK_GLIDE_*`) have no maker in 381. The client does not send them ❓ (no `push 40..42` before a `SetMoveType` call).

381 IDA names of some maker vtables are false matches: the Get function of maker 5 (`0xcb1130`) has the 381 name `MoveSprintJumpEnd::Get`, but its 419 GM `MoveTypeId` name is `SprintStart`. Use the maker index (the position in the vtable list), not the 381 IDA name. The "next_makers" column of the TSV comes from a push-pattern heuristic and has noise (for example `7:0`).

### 1.5 Units and angles

| Item | 381 source | Rule |
| --- | --- | --- |
| Server unit | `EngineUnitToServerUnit` `0x4014b0` | `su = floor(uu / 4)`. Uses `__ftol2` (truncate) and then corrects negative values down. 1 su = 4 engine units. |
| Packet `*_uu` fields | `0xcc4ce0` | `(int)uu` (truncate to zero). |
| Yaw on the wire | `0xcc4ce0`, `YAW_TO_DEGREE_CONVERTER` `0x197bb18` (f32 = 360/65536) | `deg = (i16)(int)((yaw & 0xffff) · 360/65536 + 0.5) % 360`. 0–359. |
| Yaw from the wire | `DEGREE_TO_YAW_CONVERTER` `0x197bb1c` (f32 = 65536/360) | `yaw = deg · 182.0417`. |
| `duration` | `0xcc4ce0` | `(int)(_duration · 1000)` ms, i16. |
| `startTime` | `0xcc4ce0` | `clamp(startTime, 0, _moveIntervalTime)`, then `(int)(t · 200)`, clamp to 0…110. Unit = 5 ms. |
| `diffx/y/z` (move type list) | `0xcc4ce0` | `EngineUnitToServerUnit(diff)` stored as i8 (low byte). |

---

## 2. Client → server: `G_MoveReqCmd` (64) and `G_TurnReqCmd` (66)

### 2.1 `G_MoveReqPacket` (88 bytes in memory, vtable `0x15e9ae4`, create `0x57e410`, Def `0x19dbfb0`, dest CURRENT, sync group 0)

Builder: `MoveListMaker::sendWalkPacket` (`0xcc4ce0`). The second builder in [`game-commands.tsv`](game-commands.tsv) (`0x4d5c40`) is the destructor. The wire form is the fields in this order, with no padding (ch. 05 §3.2: `T_SU` = 2 bytes LE, `T_REPEAT` = `i16 count` + elements).

| # | Mem off | Wire size | Field (419 GM) | Type | Value that the client writes (`0xcc4ce0`) |
| ---: | ---: | ---: | --- | --- | --- |
| 0 | 16 | 1 | `controlId` | i8 | `_controlId`: 0 player, 1 summoned cat |
| 1–3 | 18, 20, 22 | 2 each | `startx/y/z` | T_SU (i16) | `EngineUnitToServerUnit(_moveHistoryList[0].Location)` |
| 4–6 | 24, 26, 28 | 2 each | `endx/y/z` | T_SU (i16) | `EngineUnitToServerUnit(_moveHistoryList[count-1].Location)` |
| 7–9 | 32, 36, 40 | 4 each | `startx_uu/y/z` | i32 | `(int)` start engine position |
| 10–12 | 44, 48, 52 | 4 each | `endx_uu/y/z` | i32 | `(int)` end engine position |
| 13 | 56 | 2 | `duration` | i16 | `(int)(_duration · 1000)` ms |
| 14 | 58 | 2 | `yaw` | i16 | degrees 0–359 from `_yaw` (§1.5) |
| 15 | 60 | 2 | `moveKey` | i16 | `_moveKey` (§3.2) |
| 16 | 62 | 1 | `broadcast` | i8 `a3` low byte ≠ 0 (forced, `sub_CC55B0`) → 1, and `_broadCastMove = 0`. Else, when `Config+16347` ≠ 0 → the old `_broadCastMove`, then it toggles. Else → 1. So in the 0.25 s mode, every second packet has `broadcast` = 0 ❓ (server meaning: do not relay to others) |
| 17 | 64 | 2 + 9·n | `moveTypeList` | T_REPEAT of `GR_MoveTypeListRptBody` (sub-def `0x1495910`, element create `0x577cc0`, element size 20 in memory) | one element per `_moveTypeList[0.._moveTypeCount-1]`, n ≤ 12 |
| 17/0 | +8 | 1 | `moveType` | i8 | `MoveTypeData.moveType` (`G_MOVETYPE_*`) |
| 17/1 | +9 | 1 | `sprintJumpToDeceleration` | i8 | `isDecelFalling ≠ 0` |
| 17/2 | +10 | 1 | `startTime` | i8 | 5 ms units, 0–110 (§1.5) |
| 17/3 | +11 | 1 | `startCell` | i8 | `MoveTypeData.startCell` |
| 17/4–6 | +12…+14 | 1 each | `diffx/y/z` | i8 | `EngineUnitToServerUnit(diff)` low byte |
| 17/7 | +16 | 2 | `eventJump` | i16 | `MoveTypeData.eventJump` |
| 18 | 76 | 2 + 6·m | `cellCount` | T_REPEAT of `GR_CellListRptBody` (sub-def `0x1495640`, element create `0x578d80`, 16 bytes in memory) | one element per `_cellList[0.._cellListCount-1]`, m ≤ 32 |
| 18/0–2 | +8, +10, +12 | 2 each | `cellX`, `cellY`, `z` | i16 | low 16 bits of `PTCell._x/_y/_z` (§2.3) |

Other behaviour of `sendWalkPacket` (`0xcc4ce0`):

- When `_controlId` = 1 and the summoned actor state byte (+38) = 11, it clears the list and sends nothing ❓ (meaning of state 11).
- The player needs the local player actor (`[world+76]+128`); the cat needs `[world+76]+132`. When it is absent, the packet is deleted and not sent.
- Before the send, it sets the diff of the last move type entry to `end − entry.location`.
- The packet header byte +12 is the high dword low byte of the `a3` argument (`sub_CC55B0` passes `a3`; the tick passes 0). Ch. 05 describes the header.
- After the send, it calls `ClearMoveList` (`0xc5b580`): the last history sample becomes sample 0, the last cell becomes cell 0, the last move type becomes the first entry (with the old move type at +1), `_accumTime = _duration = 0`.
- Debug: with the summoned cat, it calls `0xb9cc30` with the start and end positions (draw helper) ❓.

### 2.2 When the client sends (`MoveListMaker::playerTick` `0xcc82b0`)

`MoveListMaker::Tick` (`0xcc93d0`) calls `playerTick` for the player and `summonedTick` (`0xcc7f30`) for the cat. Each tick:

1. No record while `_enterPortal` or `_enterArenaLobby` is set.
2. `checkMove` (`0xcb26d0`) and `checkRotate` (`0xc5b3e0`) update `_playerMoveState` / `_playerRotateState` from the engine.
3. For each new engine sample (engine interface slot +564), the tick validates the area (`isValidArea` `0xc57220`, else engine slot +492 resets the actor ❓), calls `makeCell` (`0xc74470`), and calls the `Get` of the current maker. Max 200 samples (`_moveHistoryCount` < 200; the list grows only up to 199).
4. Flush reasons that the tick sets (only these immediate values exist in the 381 code, scan of all `mov byte [reg+3910h], imm`):

| Reason | Where | Condition |
| ---: | --- | --- |
| 1 OVER_TIME | `0xcc82b0` line "moveIntervalTime < accumTime" | `_accumTime > _moveIntervalTime` (0.5 s or 0.25 s) |
| 2 REST | `0xcc82b0` | no new sample, `_duration > 0`, interval passed, and the `NotMove` maker (`_moveTypeMaker[37]`) returns ≠ 0. Also ends the hyper move (`BPTPlayer::EndHyperMove` `0x4d6c90`) when a player flag is set. |
| 3 NO_MOVE | `0xcc82b0` | no new sample and the `NotMove` maker returns 0, or the last sample is `Run` with `fTimeStamp` < 0.0001 (then `_accumTime = _duration = 0.001`) |
| 4 LANDING | `0xcc82b0` | a maker `Get` returns 2 (`MoveTypeMaker::Result_Landing`) |
| 5–8 | `MoveListMaker::getPortalDesc` `0xc99130` writes `_portalDesc.reason` (5, 6, 7, 8) and the tick copies it | the move enters a portal / env volume: 5 ENTER_PORTAL, 6 WAIT_BIND_DUNGEON_PORTAL, 7 WAIT_UNBIND_DUNGEON_PORTAL, 8 TRANSIT_FAILED (419 GM names) |
| any | `sub_CC55B0` (`0xcc55b0`, flush with a reason, ❓ GM name) | callers: `0xa92490` (cat, reason 1), `0x1113950` called from `BControlledPlayingActions::useSkillByPlayer` `0x1114de0` (reason 12 USE_ACTIVE_SKILL, header byte 1), `BControlledPlayingActions::UseAction` `0x1118750` (reason 13 AIRDASH_ENV), `BPTPlayer` vtable slot 77 `0xb16830` (reason from its argument, §3.4) |

The other reasons of the enum (59 values, [`06-movement-enums-gm.tsv`](06-movement-enums-gm.tsv)) have no writer in 381 code that we found. The reason is not sent on the wire: it only selects the action after the flush (portal request, UI). ❓ Some can come through `0xb16830`.

5. Switch on the reason after the flush (`0xcc82b0`):
   - 5 and 8: when the destination zone resource is not loaded (`World::IsDestZoneResource`), the UI shows a loading wait (`0x752330(6 or 5, 0)`) and the tick returns. Else the flush is sent and `MoveListMaker::ReqEnterPortal` (`0xcc5510`, `G_EnterPortalReqCmd` 296) follows.
   - 6, 7: same resource check, then flush.
   - other: flush, then `ReqEnterPortal` when `_inPortal`.
6. With no reason: when `Config+16347` = 0 and `_moveTypeCount` > 1, the tick also flushes at once (a move type change is sent without wait).

### 2.3 Cells (`MoveListMaker::makeCell` `0xc74470`, `SetArea` `0xc5b8f0`)

- `SetArea(origin_x, origin_y, count_x, count_y)` (i16 args): `_originX = origin_x · 256`, `_originY = origin_y · 256` (engine units). `_areaMin = (originX, originY, 0)`, `_areaMax = _areaMin + (count_x · 256, count_y · 256, 0)`.
- Cell index: `cellX = (int)(x − _originX) / 32`, `cellY = (int)(y − _originY) / 32` (C division, truncate). Cell z: `EngineUnitToServerUnit(z)`, the maximum z seen in the cell.
- A new cell is appended when the index changes. When the move goes diagonally to a cell that is not a side neighbour, `makeCell` projects the path (`GetProjectedPosition`) and inserts the cell in between. Max 32 cells.
- One cell = 32 engine units = 8 server units. One area unit = 256 engine units.

### 2.4 Turn: `G_TurnReqCmd` (66, 20 bytes, create `0x57e4c0`, Def `0x19e47b8`)

| Mem off | Field | Type | Value |
| ---: | --- | --- | --- |
| 16 | `controlId` | i8 | `_controlId` |
| 18 | `yaw` | i16 | `GetDegreeFromYaw` (`0x449b20`) of `_rotate.RotationYaw`, 0–359 |

Rules (`playerTick` `0xcc82b0`, standing still only):

- Angle change from the last sent yaw > 25° → send at once.
- Else `_rotateSendTime += dt`; when it is > 0.25 s and the change is > 2° → send. Then `_rotateSendTime = 0`.
- `sub_CC55B0` (`0xcc55b0`) also sends a turn when there is no move to flush (`_moveHistoryCount` ≤ 1 and not `_broadCastMove`) and the yaw in degrees changed. Header byte +12 = `a3`.
- Reply on error: `G_TurnFailCmd` (67) → `HandleTurnFailPacket` `0xee10a0` (route own).

### 2.5 Worked example (body bytes, header from ch. 05)

Player (`controlId` 0) walks from engine (1000.0, −2000.5, 300.0) to (1020.0, −2000.5, 300.0) in 0.5 s, yaw 16384 (90°), `moveKey` = 7, one move type `RUN` at time 0, cell 0. Area origin (0, 0): `cellX = 1000/32 = 1020/32 = 31`, `cellY = −2000/32 = −62` (truncate), `z = 75`. So there is one cell.

| Field | Value | Bytes (LE) |
| --- | --- | --- |
| controlId | 0 | `00` |
| startx, starty, startz | 250, −501, 75 (floor(−2000.5/4) = −501) | `fa 00` `0b fe` `4b 00` |
| endx, endy, endz | 255, −501, 75 | `ff 00` `0b fe` `4b 00` |
| startx_uu … startz_uu | 1000, −2000, 300 | `e8 03 00 00` `30 f8 ff ff` `2c 01 00 00` |
| endx_uu … endz_uu | 1020, −2000, 300 | `fc 03 00 00` `30 f8 ff ff` `2c 01 00 00` |
| duration | 500 | `f4 01` |
| yaw | 90 | `5a 00` |
| moveKey | 7 | `07 00` |
| broadcast | 1 | `01` |
| moveTypeList count | 1 | `01 00` |
| element: moveType, decel, startTime, startCell, diffx, diffy, diffz, eventJump | 1, 0, 0, 0, 5, 0, 0, 0 (diff 20 UU → 5 su) | `01 00 00 00 05 00 00 00 00` |
| cellCount | 1 | `01 00` |
| element: cellX, cellY, z | 31, −62, 75 | `1f 00` `c2 ff` `4b 00` |

Body length: 1 + 12 + 24 + 6 + 1 + 2 + 9 + 2 + 6 = 63 bytes.

---

## 3. Server → client: corrections

The server does not send an "OK" for a move. It answers only when it rejects a move or forces a position. The client never moves the local player to the position in the error packet. It always moves to the position of its own net object (§3.1).

```
 G_MoveFailCmd 65 ─► 0xef1870 ─► World 0x749550 ─► BPTPlayer::ProcessNetObjectEvent 0xb15dd0
                                                     ├ case 65 ─► walkFail 0xaa33a0 ─► OnLand 0xaa1010
                                                     │                              └► SetWorldInfo 0xafb560 (vslot 81)
                                                     └ case 73 ─► inline ground trace ─► SyncServerMove 0xc5b890
 G_ChangeMoveKeyCmd 73 ─► 0xef1770 ─► (same route)                                     └► setPlayerMove 0xc5b180
 SetWorldInfo ─► HyperMove::End 0xe93ee0 ─► ground trace (850 / 64) ─► SyncServerMove(force 1)
```

Worked example: the client sends a move with `moveKey` 7. The server finds it too fast. It sends `G_MoveFail` with 381 `failReason` 110 (GM `TOO_FAST` 115) and `moveKey` 8. The client (§3.5 first row) calls `OnLand`, snaps to the net object position (for example su (250, −501, 75) → engine (1000, −2004, 300), then the ground trace), sets `_isValidMoveKey` = 1 and `_moveKey` = 8. The next `G_MoveReqCmd` has `moveKey` 8, start = the snap position, one move type (RUN, FALL or SWIM).

### 3.1 Net object position (the snap target)

`BPTPlayer` +4 is the net object link. Its vtable slot 10 (+40) returns the net object of the player (spawn and update data, ch. 05). The snap code reads these net object fields (381 `0xafb560`, `0xaaa110`, `0xb15e89`):

| Net object offset | Type | Meaning |
| --- | --- | --- |
| +148 / +150 / +152 | i16 | server position x / y / z (su). Engine = su × 4.0 |
| +154 | i16 | yaw in degrees. Engine yaw = deg × 182.0417 + 0.5 (`DEGREE_TO_YAW_CONVERTER` `0x197bb1c`) |
| +458 | u8 | 1 → the client ignores `G_MoveFail` (case 65 of `0xb15dd0`). Name ❓ |
| +776 | u8 | 1 = combat mode ❓ (`SyncRuleMoveLocation` `0xaaa110` uses it for the speed; `RequestCombatHyperMove` `0xaf2510` and `SetCanCombatHyperMove` `0xaa2d70` test it == 1) |
| +840 | i16 | move speed in su. Engine = su × 4.0 (× 0.8 in one branch of `0xaaa110`) |

### 3.2 Snap: `BPTPlayer::SetWorldInfo` (`0xafb560`, vtable `0x16241a0` slot 81; 419 GM `?SetWorldInfo@BPTPlayer@@UAEXXZ`, GM slot 86)

Order of operations:

1. Get the net object (slot 10). If it is null, stop.
2. Call two `EInterface` slots (+628, +532) ❓ (we think that they stop the current engine move). Then call `HyperMove::End` (sprint stops).
3. Make an `EOWorldInfo`. Position = net object su × 4.0.
4. Ground trace 1: `EInterface` trace from z + 32.0, range 850.0 (`flt_1871B60`). It returns x, y, z and a "hit" byte.
5. If trace 1 did not hit: ground trace 2 with extent 64.0 and direction z −1.0, from z + 32.0.
6. If the server z is higher than the trace z, the client uses the server x, y, z. Else it uses the trace result.
7. Yaw = deg × `DEGREE_TO_YAW_CONVERTER` + 0.5.
8. `MoveListMaker::SyncServerMove(force = 1, x, y, z)` on `this + 960`.

Case 73 (`G_ChangeMoveKeyCmd`) of `BPTPlayer::ProcessNetObjectEvent` (`0xb15dd0`, code at `0xb15e89`) has the same steps 3–7 inline. Then it calls `SyncServerMove(force = 0, …)` and updates the move key (§3.4).

### 3.3 `MoveListMaker::SyncServerMove` (`0xc5b890`) and `MoveListMaker::Warp` (`0xc5b840`)

- Both do nothing when `_isValidMoveKey` (+14809) is 0.
- If `_controlId` (+14800) ≠ 0 (summoned): `setSummonedMove` `0xc5aff0` / `summonedWarp`. Else: `setPlayerMove` `0xc5b180` / `playerWarp` `0xc5ae90`.

`setPlayerMove` (`0xc5b180`) resets the move list to one sample at the server position. The game code has no smoothing for the local player:

| Field (381 offset) | New value |
| --- | --- |
| `_moveHistoryList[0]` x, y, z (+4, +8, +12), yaw (+28) | server position, yaw |
| `_moveHistoryList[0]` +40 (time) | 0.0 |
| `_moveHistoryCount` (+13600) | 1 |
| `EInterface` slot +568 | called ❓ (engine position set) |
| if `force` = 1: `EInterface` slot +492 (this) | called ❓ |
| `_curPos` (+13604) | x, y, 0.0 |
| `_yaw` (+13684) | yaw |
| `_cellList[0]` (+13700) | `((x − _originX)/32, (y − _originY)/32, floor(z/4))` (`sar 5` with sign fix) |
| `_cellListCount` (+14084) | 1 |
| `_accumTime` (+14572), `_duration` (+14576) | 0.0 |
| `_fallingStartZ` (+14596), `_prevMoveZ` (+14600) | z + 10.0 |
| `_moveTypeList[0]` (+14088) | type 14 `FALL` if `EInterface` slot +644 returns 1; else 15 `SWIM` if slot +540 ≠ 0; else 1 `RUN`. Start time 0, start cell 0, position = server position |
| `_moveTypeCount` (+14568) | 1 |

Thus the next `G_MoveReqCmd` starts at the server position.

### 3.4 Move key (`G_ChangeMoveKeyCmd` 73)

Packet (381 reads): `controlId` u8 +16, `moveKey` i16 +18, `reason` u8 +20 (`MoveKeyReason`: 0 NONE, 1 INITIALIZE, 2 SERVER_MOVE; 419 GM enum).

Route: `HandleChangeMoveKeyPacket` `0xef1770` → `World::ProcessNetObjectEvent` `0x749550` (case group 0x49) → `BPTPlayer::ProcessNetObjectEvent` `0xb15dd0` case 73.

Steps (case 73):

1. Snap (§3.2) with `SyncServerMove(force = 0)`. It acts only when the old key is valid.
2. `_moveKey` (+14810) = packet `moveKey`.
3. `_isValidMoveKey` (+14809) = (reason == 1 INITIALIZE).

Thus the first key after spawn (INITIALIZE) enables sending. A SERVER_MOVE key moves the player and then clears `_isValidMoveKey`. ❓ We did not find which event sets it to 1 again after SERVER_MOVE. Known setters: `walkFail` (§3.5) and `SyncRuleMoveLocation` (§3.6).

### 3.5 Move fail (`G_MoveFailCmd` 65): `BPTPlayer::walkFail` (381 `sub_AA33A0` `0xaa33a0`; 419 GM `?walkFail@BPTPlayer@@AAEXPBVG_MoveFailPacket@@@Z` `0xc5e210`)

Packet (ch. 05): `failReason` i16 +16, `controlId` u8 +18, x/y/z i16 +20/+22/+24, `moveKey` i16 +26. The 381 `walkFail` reads only +16 and +26. It does not use x/y/z (419 GM uses them only in a log line "%s : x[%d],y[%d],z[%d],movekey[%d]").

Gate (case 65 of `0xb15dd0`, code `0xb15e5c`): the client calls `walkFail` only when (the net object is null or net object +458 ≠ 1) and `controlId` == 0. The summoned cat has its own `walkFail` (419 GM `0xc63ab0`; 381 address ❓).

**Numbering.** The 381 switch is on `failReason − 100`, 62 cases (jump table `0xaa347c`, index bytes `0xaa3494`). The 419 GM switch (`0xc5e22a`) is on `failReason − 105`, 62 cases, with the same default set moved by 5. Thus **381 value = 419 GM `G_MOVEFAIL_*` value − 5**. The names below use this rule.

Helpers:

- `OnLand` = 381 `sub_AA1010` (`0xaa1010`; 419 GM `?OnLand@BPTPlayer@@QAEXXZ` `0xc57140`). If `_isLanding` (381 `this + 0x3DDA`) is 1: call vtable slot 150 `BlockRotate(0)` (`0xaa10f0`), clear `_isLanding`, call slot 161 `SetIdleAnim(0, 1, 0)` (`0xafb350`). 419 GM log: "move stop-false : PTPlayer::OnLand".
- `SetWorldInfo` = slot 81 (§3.2). 419 GM log: "walkFail-SetWorldInfo".

| 381 failReason | 419 GM name (value + 5) | Client action (381 address) |
| --- | --- | --- |
| 100, 105, 107–112, 117, 118, 124, 125, 139–146, 151–154, 161 | INTERNAL, NOT_MOVABLE, START_POSITION_IS_NOT_IN_START_CELL, END_POSITION_IS_NOT_IN_END_CELL, START_POSITION_IS_TOO_FAR, TOO_FAST, INVALID_CELL, CANNOT_MOVE_BETWEEN_CELLS, CELL_IS_NOT_ADJACENT, CELL_INDEX_IS_NEGATIVE_NUMBER, INVALID_SPRINT_START, INVALID_SPRINT, CANNOT_GLIDE_START … TOO_HIGH_RISE_IN_GLIDE (GM 144–151), INVALID_BOOST_START, INVALID_DIVE_START, RISE_IN_BOOST, RISE_IN_DIVE, GO_TO_PORTAL_WHEN_LINK | `0xaa33c1`: `OnLand`; `SetWorldInfo` (snap); `_isValidMoveKey` = 1; `_moveKey` = packet `moveKey` |
| 104 | INVALID_MOVE_KEY | `0xaa3420`: if `_moveKey` == packet `moveKey`, ignore (419 GM log "Ignore : G_MOVEFAIL_INVALID_MOVE_KEY"). Else `_isValidMoveKey` = 1, `_moveKey` = packet key, `OnLand`, `SetWorldInfo` |
| 126 | INVALID_SPRINT_DURATION | `0xaa3411`: `OnLand`; `HyperMove._cur` (381 `this + 0x36C`) = 0.0; then the same as the first row |
| 148 | CANNOT_WALK_IN_RULE_MOVE | `0xaa33fd`: `_isValidMoveKey` = 1, `_moveKey` = packet key. No snap |
| 149 | CANNOT_REQUEST_TWO_SEQUENTIAL_NONE_BROADCAST_REQ | `0xaa3450`: `OnLand`, `SetWorldInfo`, `_isValidMoveKey` = 1, `_moveKey` = packet key, `_broadCastMove` (+14808) = 1 (the next move is a broadcast move) |
| 101–103, 106, 113–116, 119–123, 127–138, 147, 150, 155–160 | INVALID_CELL_COUNT, TOO_MANY_CELL_COUNT, TOO_MANY_MOVE_TYPE, TOO_SHORT_MOVE_REQUEST, TOO_HIGH_JUMP, TOO_SLOW_FALLING, TOO_FAST_FALLING, CLIMB_IN_FLOAT, INVALID_MOVE_TYPE … INVALID_MOVE_TYPE_START_TIME_ORDER (GM 124–128), INVALID_SPRINT_DURATION + 1 … CANNOT_SWIM_OR_DRIFT_NOT_IN_WATER (GM 132–143), CANNOT_WALK_IN_COMBAT_MODE, MISMATCH_LAST_MOVE_TYPE_AND_REQ_INIT_MOVE_TYPE, INVALID_WALL_RUN_STATE … ZONE_TRANSIT (GM 160–165) | default `0xaa347a`: do nothing |

Offsets: `_broadCastMove`/`_isValidMoveKey`/`_moveKey` are `MoveListMaker` +14808/+14809/+14810, at `BPTPlayer` +960. In 381 code they are `this + 0x3D98`/`0x3D99`/`0x3D9A`. In 419 GM they are `this + 0x4128`/`0x4129`/`0x412A` (`_moveListMaker` at GM +1872).

Rule for a server: after a `G_MoveFail` of the first row, the client sends its next moves from the net object position, with the key from the fail packet. Thus the server must set the net object position before it sends the fail. It must accept the `moveKey` that it put in the fail packet.

### 3.6 Server-driven moves (`BPTPlayer` vtable `0x16241a0`; 381 slot = 419 GM slot − 5)

| 381 slot | 381 addr | 419 GM name | Behaviour |
| --- | --- | --- | --- |
| 70 | `0xafb780` | `Move` | normal move of a net creature. Calls `BPTCreature::IsSpecialEffectMoveType` two times. Details ❓ |
| 71 | `0xaa2e20` | `ServerSetYaw` | one engine call (62 bytes) |
| 72 | `0xaa0e50` | `ClientSetYaw` | ❓ |
| 73 | `0xaaa440` | `MoveKnockBack` | `EInterface` slot +512 with mode 3 (flag = 1) or mode 2, position su × 4, yaw deg → yaw. Then `SyncRuleMoveLocation` |
| 74 | `0xaaa3a0` | `MoveDash` | `EInterface` slot +512 mode 2, position su × 4, yaw, time = arg × 0.1 (100 ms units → s). Then `SyncRuleMoveLocation` |
| 75 | `0xaaa2c0` | `MoveMount` | two engine paths. Each path ends in `SyncRuleMoveLocation` |
| 76 | `0xaaa270` | `MoveTurnBack` | one engine call, then `SyncRuleMoveLocation` |
| 77 | `0xb16830` | `MoveStop(bool,int,__int64)` (419 GM slot 82) ❓ | block counter at +540. The first block flushes with `sub_CC55B0(this + 960, reason)` and calls slot 149 (+596) |
| 78 | `0xac9d20` | `AirDash` | ❓ (the 381 IDA name is a false match) |
| 79 | `0xab23c0` | `Warp` | makes an `EOWorldInfo` and an engine show handle. Details ❓ |

`BPTPlayer::SyncRuleMoveLocation` (`0xaaa110`): sets `_ruleMove` = 1. Sets `_isValidMoveKey` = 1 if it is 0. Then calls `SyncServerMove(force = 0)` to the net object position (+148 … +154, no ground trace). If net object +776 ≠ 0, it sets the engine move speed (`EInterface` slot +932) to net object +840 × 4.0 (× 0.8 when `Config` is valid).

Other creatures (NPCs, other players) do not use `MoveListMaker`. Their moves arrive as actions (`G_BroadcastAction` 6/7 → `World::processAction`, ch. 05) and go to slot 70 `Move`. ❓ The engine (`EInterface`) does the interpolation. We did not find smoothing in the 381 game code.

### 3.7 What the server checks (from the fail list)

The client does no speed check of its own. The `G_MOVEFAIL_*` names (419 GM enum `$18A4E27B71DB8B3296A36A5F753469C9`) show what the original server checks:

- cell count (≤ 32) and move type count (≤ 12)
- move key
- start and end position in the start and end cell; start not too far from the last end
- speed ("TOO_FAST"); cells adjacent; cell index not negative
- jump height, fall speed, climb in float
- move type order and start time order
- sprint start, sprint duration, sprint skills; glide start and glide rise; swim vs land
- walk in combat mode and in rule move; boost and dive rise; wall run and wall jump state; event jump; zone transit
- `UU_AND_SU_IS_NOT_MATCH` (GM 167 = 381 162): the engine-unit and server-unit copies in `G_MoveReqCmd` must agree (`floor(uu/4)` = su).

❓ The limits (speed in su/s, jump height) are not in the client.

---

## 4. Special moves: HyperMove (sprint energy), Glide, swim, climb, dash, knockback

### 4.1 `HyperMove` (88 bytes, at `BPTPlayer` + 0x364 in 381)

| Offset | Field | Meaning |
| --- | --- | --- |
| +0 | `_state` u8 | `HyperMoveState` (419 GM): 0 NONE, 1 START, 2 END, 3 READY |
| +4 | `_max` f32 | max energy |
| +8 | `_cur` f32 | current energy (381 `BPTPlayer + 0x36C`) |
| +12 + 8·i | `_hyperMove[i].acquired` u8 | i = `HyperMoveId`, 8 entries |
| +16 + 8·i | `_hyperMove[i].consumePoint` f32 | energy per second while state is START |
| +76 | `_curHyperMoveType` i32 | the id that runs now |
| +80 | `_block` i32 | block counter |
| +84 | `_isGmMode` u8 | ❓ not read in the code that we checked |

State machine (all times in seconds; `dt` from `BPTPlayer::Tick` `0xb36040`):

```
        Start(id) 0xe93fc0                 _cur ≤ 0 (Tick) → End(false)
 READY ───────────────────────► START ─────────────────────────────► END
   ▲   (needs _block == 0                 End(bool) 0xe93ee0          │
   │    and _cur > 0)                     (also from Block, SetWorldInfo,
   │                                       StartCinema, ChangeMoveKey)│
   └────────────── _cur ≥ _max (Tick: _cur = _max) ◄──── _cur += 500·dt
```

| Function | 381 addr | Behaviour |
| --- | --- | --- |
| Tick (real) | `sub_E94110` `0xe94110` | Only caller `BPTPlayer::Tick` `0xb36040`. In replay mode (global object +2084 == 1): a state other than START becomes READY. START: `_cur −= consumePoint[_curHyperMoveType] × dt`; if `_cur` < 0 → `_cur` = 0, `End(false)`. END: `_cur += 500.0 × dt`; if `_cur` > `_max` → `_cur` = `_max`, state READY |
| `HyperMove::Start(char id)` | `0xe93fc0` | Acts only when `_block` == 0 and `_cur` > 0. Sets `_curHyperMoveType` = id and `_state` = 1. UI `BUiWorld::SprintStart(_cur)` and `ReplayRecorder::SprintStart` (when not replay) |
| `HyperMove::End(bool force)` | `0xe93ee0` | Acts when `_state` == 1 or `force`. `_state` = 2. `SprintPanel::SprintEnd(_cur)`, `ReplayRecorder::addSprintCommand`, then 5 `EInterface` calls (+584, +612, +596, +600, …) ❓ (stop sprint animation and speed) |
| `HyperMove::Ready` | `0xe32210` | `_state` = 3. No code caller (only in a table ❓) |
| `HyperMove::Block(bool)` | `0xe9d1c0` | true: `++_block`; on the first block `End(false)` and 7 `EInterface` "block" calls (+648, +656, +664, +668, +660, +676, +672, one for each move kind ❓). false: `--_block` (min 0). If net object +458 == 2, `_block` stays 1 ❓ (dead state). At 0 the engine flags are set again from `_hyperMove[i].acquired`. Callers: `BPTPlayer::resetMoveState` `0xaff1c0`, `0xaff570` |
| `HyperMove::UpdateMax` | `0xea1270` | Only when `GRuleConfig + 1693` ≠ 0. `_max` = player ability value (`BAbilityBase::GetPlayerAbilityValue`; ability id ❓). If old `_max` and `_cur` are 0: `_cur` = `_max`. UI `SprintInitialize`. If `_cur` ≥ `_max` (or replay): `set(…, 242, false)` ❓. Else START → UI `SprintStart`, other → `_state` = 2 and UI `SprintEnd`. Caller: `BPTPlayer::OnChangedCreatureProperty` `0xb36270` |
| `HyperMove::set` | `0xe9ce40` | finds the skill key in the skill book, calls a UI slot (+100), clears `_hyperMove[id]`, calls `acquireHyperMove(id)` |
| `HyperMove::acquireHyperMove(char)` | `0xe31ed0` | switch on id 0–7; each case enables one `EInterface` move kind (case 0 → +648) |
| `HyperMove::Set(bool)` | `0xe9d410` | callers `resetMoveState` `0xaff1c0`, `updatePredefinedSkillList` `0xaff460` |
| `HyperMove::UpdateSkill(int,bool)` | `0xe9d520` | callers `0xaff400`, `0xaff430` |
| `HyperMove::updateSprintBlockState` | `0xe9cd90` | ❓ |
| `BPTPlayer::StartHyperMove(int)` | `0x4d6c70` | wrapper of `Start`. Callers: 30 move type maker `Get` functions (`0xca77a0` … `0xcb2480`), thus the sprint starts when the engine reports a sprint move type |
| `BPTPlayer::EndHyperMove` | `0x4d6c90` | wrapper of `End` |

Energy value 500 per second for the refill is a constant in the 381 code (`0xe94110`). The drain rate comes from data (`consume-hyper-energy-point`, see old notes §4.3). `_max` comes from the ability `max-hyper-energy-point` ❓. The server checks the sprint duration (`G_MOVEFAIL` 381 126 = GM `INVALID_SPRINT_DURATION`): then the client sets `_cur` = 0 (§3.5).

### 4.2 Combat sprint (`G_ResetCombatModeReq` 1053)

| Function | 381 addr | Behaviour |
| --- | --- | --- |
| `BPTPlayer::SetCanCombatHyperMove(bool)` | `0xaa2d70` | sets `BPTPlayer + 15844` (`0x3DE4`) = 1; if net object +776 == 1 sets a UI flag |
| `BPTPlayer::RequestCombatHyperMove` | `0xaf2510` | if net object +776 == 1 and +15844 == 1: `setCombatHyperMoveCoolTimeEvent`. Sends `G_ResetCombatModeReq` (command 1053, 16-byte packet, no body) with `BNSClient::SendToGamed` |
| `BPTPlayer::setCombatHyperMoveCoolTimeEvent` | `0xab0950` | registers a timer (`Timer::RegisterTimerEvent`) of `BPTPlayer + 15840` ms / 1000 s; sets `+15845` = 1 (timer runs) |
| `SetCombatHyperMoveCoolTimeTimer` (vfunc) | `0xae1cf0` | timer callback ❓ |
| `MoveListMaker::sendCombatHyperMovePacket` | `0xc99a00` | ❓ |
| `SprintPanel::StartCombatSprintCountDown` / `StopCombatSprintCountDown` | `0x904ec0` / `0x904e80` | UI count down |

### 4.3 Glide (3 bytes)

| Offset | Field |
| --- | --- |
| +0 | `_state` |
| +1 | `_isGlide` (glide acquired) |
| +2 | `_countblockGlide` i8 |

| Function | 381 addr | Behaviour |
| --- | --- | --- |
| `Glide::Glide` | `0xee0700` | constructor |
| `Glide::Set` | `0xf43c60` | `_countblockGlide` = 0. Finds the glide skill in the skill book (player +128 → +2904), checks the Skill3 action (old notes §4.4: byte @612 == 8 ❓), sets `_isGlide` |
| `Glide::BlockGlide(bool)` | `0xee0640` | true: `++_countblockGlide`; on the first block `EInterface` +652(true) (block glide), +628, and if +640 returns 1 (now gliding) +636 (stop glide). false: `--_countblockGlide`; when < 1 and `_isGlide` == 1, +652(false) |
| `BPTPlayer::StopGlide` | `0xaa0f30` | vfunc, 29 bytes ❓ |
| `BPTCreature::IsGlide` / `BPTPlayerSummoned::IsGlide` | `0xaa1aa0` / `0xa20e50` | vfuncs |

The server checks glide start, glide skill, glide in block glide and rise in glide (GM `G_MOVEFAIL` 144–151 = 381 139–146). The client answer is a snap (§3.5 first row).

### 4.4 Swim, climb, fall

There is no special client state object for swim or climb. They are move types (`G_MOVETYPE_SWIM` 15, `FALL` 14, climb types in [`06-movement-enums-gm.tsv`](06-movement-enums-gm.tsv)) that the move type makers make from the engine state (§1.4). After a snap, `setPlayerMove` picks FALL, SWIM or RUN from the engine (§3.3). Server checks: `CANNOT_RUN_IN_WATER`, `CANNOT_SWIM_IN_LAND`, `CLIMB_IN_FLOAT`, `TOO_SLOW_FALLING`, `TOO_FAST_FALLING`, `TOO_HIGH_JUMP` (§3.7). `_fallingStartZ` (+14596) in `MoveListMaker` keeps the fall start ❓ (used by the fall move types).

### 4.5 Dash, knockback, mount, turn back, air dash

These are server-driven moves (§3.6). The client does not send its own move while the engine plays them ❓; `SyncRuleMoveLocation` sets `_ruleMove` = 1 and puts the move list at the server end position. The dash time on the wire is in 100 ms units (`MoveDash` `0xaaa3a0` × 0.1). Air dash: client sends `G_AirDashReq` 1219 (`airDashId` i64) from `UseAction` `0x1118750`; the error is `G_AirDashFail` 1220 → `0xef16b0` (GM names `G_AIRDASHFAIL_*` 174–180). Warp to a position: `G_WarpToPosReq` 1221 (zoneDataId i32, x/y/z i16); error `G_WarpToPosFail` 1222 → `0xee1090`. Teleport errors use the strings `Msg.Teleport.Fail.Sprint` `0x14e4fa0`, `Msg.Teleport.Fail.Swim` `0x14e4fd4`, `Msg.Teleport.Fail.Move` `0x14e5004`. Infinite energy zone message: `Msg.EnterZone.Sprint.Infinity` `0x16b03ac`.

### 4.6 Changes to the old notes (client-exe-reversing §4)

| Old note | 381 fact |
| --- | --- |
| §4.3 `HyperMove::Tick` = `0xe82b90` | False name match. `0xe82b90` builds a packet with the `L_GetPcCafeMileageFeverTimeScheduleListReqPacket` vtable. The real tick is `sub_E94110` `0xe94110` (§4.1) |
| §4.3 HyperMoveId 0–8 (Hyper, Sprint, Drift, Boost, Dive, WallMove, EventJump, RocketJump, Max) | 419 GM enum has only Hyper 0, WallMove 5, RocketJump 7. The other names match the `G_MOVEFAIL` skill names (SPRINT, EVENT_JUMP, DRIFT, BOOST, DIVE, WALL_MOVE) ❓ |
| §4.5 client move is `GS_ActionsClientMoveCmd` | False for the client → GameD path. The client sends `G_MoveReqCmd` 64 (§2). `GS_Actions*` are ActionsCommand names (strings `0x16152d4` …, used by `0xb3bd80`) |
| §4.4 Glide fields | Confirmed: `_isGlide` +1, `_countblockGlide` +2 (`0xee0640`) |

---

## 5. Open items (❓)

| Item | Where to look |
| --- | --- |
| Names of the `EInterface` slots (+492, +512, +532, +568, +584, +628, +640, +644, +648 … +676, +932) | engine DLL interface; compare with 419 GM call sites |
| Which event sets `_isValidMoveKey` = 1 after a SERVER_MOVE key | other writers of `this + 0x3D99` in 381 |
| Summoned `walkFail` in 381 (419 GM `0xc63ab0`, 65 cases) | case 65 of the summoned `ProcessNetObjectEvent` |
| `BPTPlayer` vslots 70, 72, 77, 78, 79 in detail | `0xafb780`, `0xaa0e50`, `0xb16830`, `0xac9d20`, `0xab23c0` |
| `HyperMove::updateSprintBlockState`, `Set`, `UpdateSkill`, the `set(…, 242, …)` call | `0xe9cd90`, `0xe9d410`, `0xe9d520`, `0xea1270` |
| Ability id for `_max` | `0xea1270` (`off_1574ACC`) |
| Net object fields +458, +776 names | 419 GM net object type |
| Interpolation of other creatures | engine; `World::processAction` (ch. 05) |
