# 07 — Client.exe 381: skills and combat

Summary:

1. The player uses a skill through `BControlledPlayingActions::UseSkill` (`0x1114f50`). The client checks the skill (recycle time, points, conditions, range). Then `requestSkill` (`0x10d26e0`) sends `G_UseSkillReqCmd` 1226 (or `requestGroundSkill` `0x10d2ec0` sends `G_UseSkillToGroundReqCmd` 1228). Both use sync group 3 (Skill).
2. The server does not send one "damage packet". All skill, hit, effect and buff results come as **actions** in `G_BroadcastActionCmd` 6 and `G_BroadcastPrivateActionCmd` 7. Each action is a `T_SUB` variant of the `GS_Actions` table (`0x1496720`, variants 1–103). Skill is 3, SkillCancel 4, SkillHit 5, Effect 6, PassiveEffect 7.
3. An Effect action (6) carries a second variant (`GS_Effects`, `0x14962e8`, 1–15): damage, heal/drain, knockback, dash, property diff, recycle-time changes. A PassiveEffect action (7) carries `GS_PassiveEffectInfo` (`0x14964a0`, 1–4): buff/debuff full sync and detach.
4. Failure replies (`G_UseSkillFailCmd` 1227 and others) and recycle-time notifies (1261–1263) go to `BControlledPlayingActions::ProcessNetObjectEvent` (`0x10fd7f0`). Cool-downs live in `SkillTimer` (global cool time + recycle groups).
5. Every action layout is in [`07-combat-action-layouts.tsv`](07-combat-action-layouts.tsv) (446 rows). The full game error-code name table (2,631 codes) is in [`07-combat-error-codes.tsv`](07-combat-error-codes.tsv).

```
 INPUT                                     NETWORK                                SCREEN
 key / mouse / skill bar                                                         
   └► BPlayer::UseSkill 0xbbeac0                                                 
       └► BControlledPlayingActions::UseSkill 0x1114f50                          
           ├► useSkillByPlayer 0x1114de0 ─► useActiveSkill? 0x1113950           
           └► useSkillByEgo 0x10fceb0                                            
                 │ isSkillAvailableToUse 0x10e9140 (SKILL_USE_ERROR, §1.3)       
                 │   fail ─► OutputSkillErrorGameMessage 0x10b90d0 ──────────────► "Msg.Skill.CastFail.*"
                 ▼                                                               
           requestSkill 0x10d26e0 ── G_UseSkillReqCmd 1226 ──► GameD             
           requestGroundSkill 0x10d2ec0 ─ G_UseSkillToGroundReqCmd 1228 ─►       
                                                                                 
 GameD ── G_BroadcastActionCmd 6 / 7 ──► World::processAction 0x748b00            
          (actions: Skill 3, SkillCancel 4, SkillHit 5, Effect 6, PassiveEffect 7)
             ├► World::processSkillActionToOwner 0x737a60 (3, 4) ─► creature ProcessActionSubEvent
             └► World::processActionToOwner 0x744a20 (5, 6, 7, …) ─► BPTCreature::presentationActions 0xb1a590 ─► damage text, buff icons
 GameD ── G_UseSkillFailCmd 1227, G_SetRecycleTime*Notify 1261–1263 ──► BControlledPlayingActions::ProcessNetObjectEvent 0x10fd7f0 ─► SkillTimer
```

Sources: all facts come from the 381 database `ida/lanes/cx/Client.exe.i64`. "419 GM" marks a name or type that comes only from `Client_gm.exe.i64`. ❓ marks a fact that we did not verify. Some 381 IDA names are false PDB matches; where a 381 function has no name, the table gives the 419 GM name with ❓ if we matched it only by its callers.

Related chapters: command table, `PacketSync` lock and `StructPacket::Def` wire format: [05-game-dispatch](05-game-dispatch.md). Own-player movement and `G_MoveReqCmd`: [06-movement](06-movement.md). Old notes: [client-exe-reversing §2](../client-exe-reversing.md), datafile tables: [client-datafiles](../client-datafiles.md).

Files of this chapter:

| File | Rows | Content |
| --- | ---: | --- |
| [`07-combat-action-layouts.tsv`](07-combat-action-layouts.tsv) | 446 | every field of every action variant (`GS_Actions` 1–103) and of the nested variant tables `GS_SkillTarget`, `GS_Effects`, `GS_PassiveEffectInfo`, with 419 GM field names |
| [`07-combat-enums-gm.tsv`](07-combat-enums-gm.tsv) | 2,285 | 149 combat enums from 419 GM (skill flow, hit code, use errors, data-table enums) |
| [`07-combat-error-codes.tsv`](07-combat-error-codes.tsv) | 2,631 | game error code → name, from the 381 table `0x194fc30`; column `combat` = 1 for skill/effect/target codes |
| [`07-combat-functions.tsv`](07-combat-functions.tsv) | see §9 | named functions and globals of this area |

---

## Quick find

| Task or question | Function or object | 381 address | GM name (419 GM) | Section |
| --- | --- | --- | --- | --- |
| Where the client starts a skill | `BControlledPlayingActions::UseSkill` | `0x1114f50` | same | §1.1 |
| Where the client sends a skill | `requestSkill` / `requestGroundSkill` | `0x10d26e0` / `0x10d2ec0` | same ❓ | §1.4 |
| `G_UseSkillReq` packet constructor | `G_UseSkillReqPacket::G_UseSkillReqPacket` | `0x449fd0` | same | §1.4 |
| Client-side skill check | `isSkillAvailableToUse` | `0x10e9140` | same | §1.3 |
| Cast-fail message to the player | `OutputSkillErrorGameMessage` | `0x10b90d0` | same | §1.3 |
| "One skill request in flight" lock | `IsRequestedSkillAnswered` | `0x10b0090` | same | §1.5 |
| Cancel a skill | `RequestCancelAction` | `0x10d3ba0` | same | §1.6 |
| Server skill/hit/effect stream | `World::processAction` | `0x748b00` | same | §2.1 |
| Skill action of a creature | `World::processSkillActionToOwner` | `0x737a60` | same | §2.2 |
| Own skill result (success/cancel) | `BControlledPlayingActions::processSkill` | `0x111d2d0` | same | §2.3 |
| Damage numbers, effect show | `BPTCreature::presentationActions` | `0xb1a590` | same | §3.3 |
| Battle message ("X hits Y") | `BPTCreature::invokeBattleMessage` | `0xb319d0` | same | §3.3 |
| Action variant table | `GS_Actions_define` | `0x1496720` | same name (GM address ❓) | §2.1 |
| Effect variant table | `GS_Effects_define` | `0x14962e8` | same name (GM address ❓) | §3.1 |
| Buff/debuff variant table | `GS_PassiveEffectInfo_define` | `0x14964a0` | same name (GM address ❓) | §3.2 |
| Skill target variant table | `GS_SkillTarget_define` | `0x1496428` | same name (GM address ❓) | §2.4 |
| Skill fail / recycle replies | `BControlledPlayingActions::ProcessNetObjectEvent` | `0x10fd7f0` | same ❓ | §4 |
| Global cool-down | `SkillTimer::SetGlobalCoolTime_renewal` | `0x916030` | same | §5 |
| Recycle (cool-down) group | `SkillTimer::SetSkillRecycleTime_renewal` | `0x974fc0` | same | §5 |
| Select a target | `BPTPlayer::RequestSetTarget` | `0xaf30a0` | same | §6 |
| Error code → name | `L_ERROR_CODE_TO_STRING` | `0x712bc0` (table `0x194fc30`) | same | §4.2 |
| Byte examples | `G_UseSkillReq`, Effect action | — | — | §8 |

---

## 1. Skill use flow (client → server)

### 1.1 Input to `UseSkill`

| Step | Function | 381 | Behaviour |
| --- | --- | --- | --- |
| 1 | key / skill bar | `UiStateGame::ProcessPressedKeyEvent` `0x859e90`, `SkillBarPanel::ProcessEvent` `0x995950` | input event → context process |
| 2 | context | `BContextProcess::ProcessContextInput` `0x111ab90`, `processSkillbarPanelInput` `0x111a1a0`, `ProcessStanceInput` `0x1119fe0` → `UseSkillByContext` `0x1119f50` | selects the skill of the slot (context-sensitive keys) |
| 3 | player | `BPlayer::UseSkill(uint key, bool)` `0xbbeac0` | also called by `UpdateAutoEgoSkillProcess` `0x11160c0` (from `Tick` `0x1116460`) and by AI tasks (`UBTTask_BnsUseApproachSkillTo::TickTask` `0x1234810`) |
| 4 | actions | `BControlledPlayingActions::UseSkill` `0x1114f50` | returns false in observer mode (`byte_1A4C32F` and `BNSClient::IsObserverMode`). Branches to `0x10fcad0` ❓ (item/equip skill), `useSkillByEgo` `0x10fceb0`, or `useSkillByPlayer` `0x1114de0` |
| 5 | player skill | `useSkillByPlayer` `0x1114de0` → `0x1113950` (`useActiveSkill`, 419 GM name ❓) | gets the skill key (`BSkillBook::GetSkillKey` `0x94afc0`) |

### 1.2 `useActiveSkill` steps (`0x1113950`)

Calls in this order (call list of `0x1113950`): `SkillTracer::StartSkill` `0x933350` → `BPTPlayer::IsPlayingKismet` `0xaa1180` → `IsRequestedSkillAnswered` `0x10b0090` (§1.5) → `BPTPlayer::MoveCancel` `0xb2b080` → `isSkillAvailableToUse` `0x10e9140` (§1.3) → on error `OutputSkillErrorGameMessage` `0x10b90d0` and `SkillTracer::FailedByClient` `0x924310` → `CancelSocialMode` `0x10a0a10`, `CancelAction` `0x10dd540`, `CancelSkill` `0x10dd2a0` (stop the running action) → field-item skills: `GetFieldItemSkillTypeAndIndex` `0x10b8ee0`, `0x10d36c0` (`G_UseFieldItemReq` builder) → target skill: `requestSkill` `0x10d26e0`; ground skill: `requestGroundSkill` `0x10d2ec0`.

### 1.3 Client checks and `SKILL_USE_ERROR`

| Check | 381 | Notes |
| --- | --- | --- |
| all checks | `isSkillAvailableToUse` `0x10e9140` | calls the checks below |
| basic (data, caster, weapon, state) | `canUseSkillByBasicCheckInternal` `0x10afdf0` → `ControlledPlayingActionsBase::canUseSkillByBasicCheckInternal` `0x10a5c10` | |
| points and recycle | `canUseSkillByPointCheckInternal` `0x10e57f0` | calls `GetSkillTimer` `0x10a0c30`, `PlayerCommon::CheckClientSideRecycleTime` `0xb8b7c0` |
| SP / HP cost | `ControlledPlayingActionsBase::isConsumablePointEnough` `0x10e01d0` | log strings `0x15298d0`…`0x152a4c0` "[SkillRecycleTime Debug] : …isConsumablePointEnough() - CONSUME_POINT_…" |
| condition status (stance, buffs, move state) | `canUseSkillByConditionStatusCheckInternal` `0x10e5c20` | log `0x152a5b0` "…find fail SoulMaskSetRecord key(%d, %d)" (`0x10e30e0`) |

The enum `ControlledPlayingActionsBase::SKILL_USE_ERROR` (419 GM, 70 values) and `CONSUME_POINT_ERROR` (11 values) are in [`07-combat-enums-gm.tsv`](07-combat-enums-gm.tsv). **381 values are not the same as 419 GM after 37.** `OutputSkillErrorGameMessage` `0x10b90d0` gives these 381 values (case → string):

| 381 value | Message key (string address) | 419 GM name of the same value |
| ---: | --- | --- |
| 9 | `Msg.Skill.CastFail.StillOnRecycling` (`0x1529110`) | STILL_ON_RECYCLING |
| 11, 13 | `Msg.Skill.CastFail.TooFar` (`0x1529158`) | TOO_FAR, TOO_FAR_LASER |
| 12 | `Msg.Skill.CastFail.TooClose` (`0x152918c`) | TOO_CLOSE |
| 14 | `Msg.Skill.CastFail.NoTarget` (`0x15291c4`) | NO_TARGET |
| 18 | `Msg.Skill.CastFail.Block` (`0x15291fc`) | SKILL_BLOCKED |
| 23 | `Msg.Skill.CastFail.NotEnoughConsumePointSp` (`0x1529230`) | NOT_ENOUGH_CONSUME_POINT_SP |
| 29 | `Msg.Skill.CastFail.NotEnoughConsumePointHP` (`0x1529288`) | NOT_ENOUGH_CONSUME_POINT_HP |
| 30 | `Msg.Skill.CastFail.NotEnoughConsumePointSummonedHP` (`0x15292e0`) | NOT_ENOUGH_CONSUME_POINT_SUMMONED_HP |
| 37 | `Msg.Skill.CastFail.NeedWeapon` (`0x1529348`) | NEED_WEAPON |
| 45 | `Msg.Skill.CastFail.AttributeBlocked` (`0x1529388`) | GM 47 ATTRIBUTE_BLOCKED |
| 46 | `Msg.Skill.CastFail.TargetAttributeBlocked` (`0x15293d0`) | GM 48 NPC_TARGET_ATTRIBUTE_BLOCKED |
| 56 | `Msg.Skill.CastFail.BlockedByDuel` (`0x1529428`) | GM 59 BLOCKED_BY_CONTENTS_DUEL |

Value 8 (global cool-down) shows no message. In a duel (player vtable `+304` ≠ 0) only value 56 shows a message.

### 1.4 `G_UseSkillReqCmd` 1226 (C2S, 96 B struct, sync 3 Skill)

Builder: `requestSkill` `0x10d26e0` (args: `controlId`, target id, …, yaw, `BPlayer*`, skill record). It allocates 0x60 bytes, calls the constructor `0x449fd0`, sets the fields, then `BNSClient::SendToGamed` `0x114ce20`. On success `SkillTracer::SentRequest`; on fail `SkillTracer::FailedByClient` `0x924310`. Second builder: `DuelBotBrain::RequestSkill` `0x105ae70`.

Wire order = field order, no padding ([05 §3.2](05-game-dispatch.md)):

| # | Field (419 GM) | Type | Struct off | Value set by `requestSkill` |
| ---: | --- | --- | ---: | --- |
| 0 | `controlId` | i8 | 16 | argument (0 = player; 1 is used by other control paths ❓) |
| 1 | `skillId` | i32 | 20 | skill record `+8` |
| 2 | `skillVariationId` | i8 | 24 | skill record `+12` |
| 3 | `targetId` | i64 | 32 | target object id (0 = no target ❓) |
| 4 | `yaw` | i16 | 40 | `(int)(yaw16 × YAW_TO_DEGREE_CONVERTER + 0.5) % 360` → **degrees 0–359** |
| 5 | `equipPos` | i8 | 42 | -1, or the item equip position for an item skill |
| 6 | `modifyEquipPositions` | repeat of i8 `pos` | 44 | `Inventory::GetEquipItemPosListThatSkillBuildUpLevel` |
| 7 | `modifySetItemKeys` | repeat of i64 `setItemKey` | 56 | set-item keys that change the skill |
| 8 | `itemSkillEquipPosList` | repeat of i8 `pos` | 68 | item skill positions |
| 9 | `modifyBadgeEquipPositions` | repeat of i8 `pos` | 80 | badge positions (`Inventory::GetEquipItemPosListThatModifySkill`) |

`G_UseSkillToGroundReqCmd` 1228 (84 B, builder `requestGroundSkill` `0x10d2ec0`, ctor `0x44a0d0`, also `DuelBotBrain::RequestGroundSkill` `0x105b040`): `controlId` i8, `skillId` i32, `skillVariationId` i8, `x`,`y`,`z` (T_SU, 2 B each), `yaw` i16, `equipPos` i8, then the same 4 repeats.

`G_SkillTriggerReqCmd` 1231 (`controlId` i8, `skillInstanceId` i8): no builder found in 381 (game-commands.tsv `builder_count` 0) ❓.

### 1.5 Request lock (one skill request at a time)

- `BControlledPlayingActions +0x1250` holds 2 pending slots of 8 B (index = the bool argument). `requestSkill` and `requestGroundSkill` write it; `VerifyRequestSkillId` `0x10b9670` and `CancelSkill` `0x10dd2a0` clear it (writer list from a displacement scan of `0x10a0000`–`0x1120000`).
- `IsRequestedSkillAnswered` `0x10b0090`: slot 0 → true. Slot set and elapsed ≤ **0.5 s** → false. Elapsed > 0.5 s → writes a ping-history record (sync 3, command 1226, elapsed ms) through `0x880e50`, clears the slot, returns false (the next try passes).
- This lock is on top of the `PacketSync` group 3 lock of [05 §4](05-game-dispatch.md).

### 1.6 Cancel: `G_CancelActionReqCmd` 1229 (C2S, 20 B)

`RequestCancelAction` `0x10d3ba0(controlId, reason, pending)`: makes the packet in line (vtable `0x179caf0`, command 1229), `+16 controlId`, `+17 reason`, `+12 _pending`. After the send it sets `+0x12b0` = 1 and the time `+0x12a0` (QPC). `IsRequestedCancelActionAnswered` `0x10a0a90`: flag 0 → true; flag 1 and > **0.5 s** → clears the flag. `G_CancelActionFailCmd` 1230 clears the flag (§4).

| Caller | 381 | controlId | reason |
| --- | --- | ---: | ---: |
| `CancelSkill` | `0x10dd2a0` | 0 | argument |
| `CancelAction` | `0x10dd540` | 0 | argument |
| `BPlayingActions::ProcessSkillResult` | `0xbc25d0` | 0 | 3 |
| `BPlayer::ChangedPropertyCallback` | `0xbc4390` | 0 | 3 |
| `0xb845a0`, `0xb84660` | — | 1 | variable |

`CancelActionReason` (419 GM): 0 USER_REQUEST, 1 MOVE, 2 OVERRIDE_SKILL, 3 ATTACKED, 4 ABNORMAL_STATE. ❓ 381 uses the same values.

---

## 2. Server action stream

### 2.1 `G_BroadcastActionCmd` 6 and `G_BroadcastPrivateActionCmd` 7 (S2C, 28 B struct)

One field `actions`: `T_REPEAT` (i16 count) of `T_SUB` elements. Each element on the wire: `u8 len` (includes the 2 head bytes) + `i8 variant` + body. The variant selects `GS_Actions_define[variant]` (`0x1496720`, 20 B per entry). 381 has variants 1–103; entry 104 is empty. Variants 1–103 have the same field types as 419 GM (all 104 entries compared); GM adds 104 `PvpResultWarpToOpponentAction` and 105 `ActivateVehicleAction`. The enum `ActionsCommand` is in [`07-combat-enums-gm.tsv`](07-combat-enums-gm.tsv).

`World::processAction` `0x748b00` walks the list. The variant is the byte at element `+5`, the body pointer is at element `+8`. Jump index table `0x7494f0` (one byte per variant 1–96; 97–103 go to the default). Routing:

| Variants | Route |
| --- | --- |
| 1, 2, 87, 88 (moves, warp-to-guild-member) | move code (ch. 06) |
| 3 Skill, 4 SkillCancel | `World::processSkillActionToOwner` `0x737a60` (§2.2) |
| 5 SkillHit | `World::processActionToOwner` `0x744a20(body, caster +8, target +16)`; first `World::SetBulletEffect` `0x72d770` when bullet effect directing is on (`World::UseBulletEffectDirecting` `0x729780`, `Config+21909`, `hitCode` (+32) = 1) |
| 6 Effect | `processActionToOwner` with caster +16, target +24, `state` |
| 7 PassiveEffect | `processActionToOwner` with caster, target +20 |
| 21–28, 33–37 | field items, dead bodies, pouches (`World::encounterNpcDeadBody`, `World::outOfSightObject`) |

A filter skips an element when the world flag `+0x880` = 1, the list is from the packet, and `0x722c00(variant)` = 1 ❓ (replay/observer).

### 2.2 `World::processSkillActionToOwner` `0x737a60`

1. `World::find(casterId)`; no caster → return 0. `targetId` > 0 → find the target.
2. When the caster is the own player (creature vtable `+48` or `+120`): `TaskSession::Received` releases the sync lock ([05 §4](05-game-dispatch.md)).
3. Variant 3: `BCreature::GetInterpolatedSkillStep` `0x102a8f0` (skill step from `flowStep`/`duration`). Variant 4: `World::setSkillRepeatByServerToOwner` `0x72eab0`.
4. `caster->ProcessActionSubEvent(variant, body, target)`; then the zone and geo-zone handlers (`vtable +16`); then `target->ProcessActionSubEvent` when asked.

### 2.3 Own skill result: `BControlledPlayingActions::processSkill` `0x111d2d0`

- Skill flow code (`SkillFlowCode`, 419 GM: 0 CAST, 1 FIRE, 2 FIRE_FAIL, 3 FIRE_MISS, 4 EXEC, 5 EXEC_INTERVAL, 6 EXEC_FAIL, 7 RETURN, 8 FINISH, 9 CANCEL, 10 TRIGGERED). Code 9 and skill byte `+28` = 0 → `SkillTracer::Canceled` `0x924190`; else `SkillTracer::Succeeded` `0x924220`.
- Then `SkillTimer::UpdateRecycle_renewal` `0x9967e0` and `VerifyRequestSkillId` `0x10b9670` (releases the §1.5 lock).
- Codes 2, 8, 9 compare the requested skill id; main-slot skills go to `processSkillForCancelAction` `0x10dd680`, `getTargetObjectOrTargetGroundPosition` `0x10fe6e0`, `processSkillForSkillTrace` `0x10ca440`, `GetSkillUseTarget` `0x10b9a70`, `processSkillForReplayRecorder` `0x10ef010`.

### 2.4 Skill action fields (wire order; full list in the TSV)

| Variant | Fields (type) |
| --- | --- |
| 3 Skill | `casterId` i64, `skillId` i32, `skillVariationId` i8, `skillBuildUpLevel` i8, `skillInstanceId` i8, `yaw` i16, `next_yaw` i16, `relative_skill_yaw` i16, `increasing_flow_step` i16, `flowStep` i8, `state` i8, `duration` i16, `skillTarget` SUB |
| 4 SkillCancel | `casterId`, `skillId`, `skillVariationId`, `skillBuildUpLevel`, `skillInstanceId`, `yaw`, `increasing_flow_step`, `flowStep`, `duration` i16, `cancelTime` i8, `remain_global_recycle_time` i32, `remain_recycle_time` i32, `skill_result` i8, `skillTarget` SUB |
| 5 SkillHit | `casterId` i64, `targetId` i64, `skillId` i32, `skillVariationId` i8, `skillBuildUpLevel` i8, `skillInstanceId` i8, `skillState` i8, `hitCode` i8 (`SkillHitCode`: 0 HIT, 1 MISS, 2 DODGE, 3 PARRY, 4 PERFECT_PARRY, 5 BOUNCE, 6 COUNTER, 7 CRITICAL_HIT, 419 GM) |

`skillTarget` (`GS_SkillTarget_define` `0x1496428`, same as GM): 1 TargetData (`targetId` i64), 2 PositionData (x,y,z T_SU), 3 ModifiedPositionData (x,y,z), 4 PositionListData (repeat of x,y,z), 5 LinkLaserData (`target_from_id` i64, `target_to_id` i64).

---

## 3. Effects, damage, buffs

### 3.1 Effect action (variant 6) and `GS_Effects` (`0x14962e8`)

Effect body: `EffectId` i32, `EffectSlotIndex` i16, `casterId` i64, `targetId` i64, `state` i8 (`ActionState` ❓: 0 START, 1 FINISH, 2 CANCEL), `skillInstanceId` i8, `prop` PROP_PAIR, `extraProp` PROP_PAIR, `effectData` SUB.

**381 differs from 419 GM:** 381 has 15 effect variants; GM 15 `SetRecycleTimeEffect` is missing, so 381 15 = GM 16 (matched by field types).

| 381 | Name (419 GM) | Fields |
| ---: | --- | --- |
| 1 | NoneEffect | — |
| 2 | PassiveEffectDuration | `duration` i16 |
| 3 | AttackEffect | `damage` i32, `hitCode` i8 |
| 4 | AttackPassiveEffect | `damage` i32, `hitCode` i8, `duration` i16 |
| 5 | UnlinkEffect | `linkType` i8, `linkerId` i64, `linkeeId` i64 |
| 6 | DashEffect | x, y, z (T_SU), `yaw` i16, `duration` i16 |
| 7 | KnockbackEffect | x, y, z, `yaw` i16, `duration` i16, `damage` i64, `wallDamage` i64 |
| 8 | DiffPropertyEffect | `type` i8, `diff` i64 (resource change, HP/SP ❓) |
| 9 | DetachInfo | `detachReason` i8 |
| 10 | ReplacePositionEffect | caster x,y,z, target x,y,z |
| 11 | DrainEffect | `caster_heal_hp` i64, `caster_heal_sp` i16, `target_damage` i64 |
| 12 | PassiveEffectAttachFail | `failCode` i8, `damage` i64 |
| 13 | InitializeRecycleTimeEffect | `recycle_group_1`, `recycle_group_id_1`, `recycle_group_2`, `recycle_group_id_2` (i8 each) |
| 14 | ModifyRecycleTimeEffect | `recycle_group` i8, `recycle_group_id` i8, `recycle_group_modify_time` i32 |
| 15 | ShooterLockOnEffect (GM 16) | `lock_on_target_id` i64, `duration` i16 |

### 3.2 PassiveEffect action (variant 7): buffs, debuffs, CC

Body: `casterId` i64, `targetId` i64, `passiveEffect` SUB (`GS_PassiveEffectInfo_define` `0x14964a0`; 381 has 1–4, same as GM):

| Variant | Name | Fields |
| ---: | --- | --- |
| 1 | PassiveEffectFullSync | `passiveEffect` T_PROP_SYNC (`EffectPropertySync`) ❓ format |
| 2 | PassiveEffectFullSyncForEnterZone | `curTime` i64, `passiveEffect` T_PROP_SYNC |
| 3 | PassiveEffectDetachCountSync | `EffectSlotIndex` i16, `detachCount` i8 |
| 4 | PassiveEffectDetach | `EffectSlotIndex` i16 |

Receivers (`ProcessPassiveEffectEvent`): `SkillBarPanel` `0x944d60`, `PlayerEffectStatusPanel` `0xba3890`, `PartyToolTipPanel` `0xba1bd0`, `NpcTooltipPanel` `0xc33b30`, `TargetIndicator` `0x8e8270`, `EffectLinkCastPanel` `0x966880`. CC and stance live in the effect record data (`Data::EffectAttributeType`, `Data::EffectFlagType`, `CreatureProperty::Stance` in the enum TSV).

### 3.3 Presentation (screen)

| Function | 381 | Does |
| --- | --- | --- |
| `BPTCreature::presentationActions` | `0xb1a590` | Effect action → `presentationActionsEffectInstance` `0xb17ef0`, `presentationActionsEffectInterval` `0xb172a0`, `actionsEffectAttachFailPlayShow` `0xb05e30` |
| `presentationActionsEffectInstance` | `0xb17ef0` | `casterPlayerFindSkill` `0xac27a0`, `getSkillEffectHitcode` `0xaa7ce0`, `GetAndSetDamageSignalType` `0xaf8230`, `actionsEffectPlayShow` `0xb03120`, `actionsFireEffectPlayShow` `0xabc210`, `addEffectDamageMeterHitInfo` `0xb168a0` |
| `invokeBattleMessage` | `0xb319d0` | `effectMessageType` `0xaa1bc0`, `damageEffectMessageType` `0xaa1bf0`, `battleMessageSkillResultType` `0xaa7e80` → `World::InvokeGameMessage` `0x50b620` |
| `presentationActionsEffectAttachFialImmuned` | `0xafd300` | immune text (`BPTCreature::DAMAGE_TYPE` 5 IMMUNED, 419 GM) |
| damage meter | `addSkillDamageMeterHitInfo` `0xb16b60`, `addDamageMeterInfo` `0xb16e00`, `World::createDamageAccum` `0x789640` | |
| `TargetStatusPanel2::ProcessEffectEvent` | `0x8dcf60` | target frame |
| `SkillCastPanel::ProcessSkillHitEvent` | `0x96eff0` | cast bar |

`BPTCreature::DAMAGE_TYPE` (419 GM): 0 NONE, 1 DAMAGE, 2 HEAL, 3 DAMAGE_SP, 4 HEAL_SP, 5 IMMUNED.

---

## 4. Server replies to the own player

### 4.1 `BControlledPlayingActions::ProcessNetObjectEvent` `0x10fd7f0` (419 GM name ❓)

Switch on the packet command (`packet vtable +12`):

| Command | Action |
| --- | --- |
| 138, 141, 1225, 1241, 1248, 1250, 1252, 1254, 1257, 1265, 1269, 1298, 1355, 1360, 1388 (Fail packets of actions) | `0x10a0ad0(cmd)` (end the pending client action ❓) |
| 299 | `CancelSkill` |
| 365, 1297 | player vtable `+304` |
| 1227 `G_UseSkillFailCmd` | see below |
| 1230 `G_CancelActionFailCmd` | clears the cancel flag `+0x12b0` |
| 1240 `G_UseCaveEscapeOkCmd` | `+0x12b4` = field +16 × 0.1 (s) |
| 1261, 1262 | `GetSkillTimer` only (no change here ❓) |
| 1263 `G_SetRecycleTime2NotifyCmd` | `SetGlobalCoolTime_renewal(global_time, global_group)`, `SetSkillRecycleTime_renewal(recycle_group, recycle_group_id, …)` |
| 1271 `G_GatherFailCmd` | code 683 (`G_GATHERFAIL_ENTERED_FATIQUE_TIME`) → message |

`G_UseSkillFailCmd` 1227 (S2C, 28 B): `failReason` i16 (+16), `controlId` i8 (+18), `skillId` i32 (+20), `skillVariationId` i8 (+24). Steps:

1. `controlId` = current control → `addSkillFailTrace` `0x10e0900` (usage map `+4928`), `SkillTracer::FailedByServer` `0x9242a0`.
2. `VerifyRequestSkillId` (releases the §1.5 lock).
3. `controlId` 0: `failReason` 193 (`G_USESKILLFAIL_INVALID_COMBO`) → `CancelCombo` `0x10ca370`. 194, 195, 210 (WRONG_DIRECTION, WRONG_RANGE, FAILED_TO_CHECK_LINE_OF_SIGHT) → `Msg.Skill.CastFail.ByServer` (`0x1528f14`). 212 (EMPTY_SIM_SLOT_IS_NOT_EXIST) → `ClientLog::Write` (ControlledPlayingActions.cpp line 2866).

### 4.2 Error codes

`L_ERROR_CODE_TO_STRING` `0x712bc0`: index into `0x194fc30` (4 B pointers), codes 0–0xA46. Skill codes 180–215: 180 INVALID_SKILL_DATA … 213 INVALID_SOULMASK_ID, 214 `G_CANCELACTIONFAIL_CANNOT_CANCEL`, 215 `G_SKILLTRIGGERFAIL_NOT_EXECUTE`. All in [`07-combat-error-codes.tsv`](07-combat-error-codes.tsv).

### 4.3 Recycle-time notifies (S2C, sync 3)

| Cmd | Fields (wire order) |
| --- | --- |
| 1261 `G_SetRecycleTimeNotify` | `group1` i8, `group1_time` i32, `group2` i8, `group2_time` i32 |
| 1262 `G_ModifyRecycleTimeNotify` | `group1` i8, `group1_modify_time` i32, `group2` i8, `group2_modify_time` i32 |
| 1263 `G_SetRecycleTime2Notify` | `global_group` i8, `global_time` i32, `recycle_group` i8, `recycle_group_id` i8, `recycle_group_time` i32 |

Other skill commands: 74/75/76 `SpecialSkillGaugeFull` (Req builder `BPlayer::RequestSpecialSkillGaugeFull` `0xb7c6f0`; Fail: `failReason` i16, `curGaugeAmount` i16), 113/114 `SkillList2` (Ok: repeat of `page` i8, `job_style` i8, repeat of `skill_id` i32, `skill_variation_id` i8, `skill_build_up_level` i8), 787 `SendHateListNotify` (`npcId` i64, repeat of `creatureId` i64, `masterId` i64, `hate` i32), 1053 `ResetCombatModeReq` (builder `0xaf2510`), 1481–1496 skill acquire/page commands (sync 11). Full list: [game-commands.tsv](game-commands.tsv).

---

## 5. Cool-downs (`SkillTimer`)

- `SKILL_RECYCLE_STATUS` (419 GM): 0 NONE, 1 ON_GLOBAL_COOLTIME, 2 ON_RECYCLING, 3 OTHER_SKILL_CASTING, 4 ON_EFFECT.
- `SetGlobalCoolTime_renewal` `0x916030`: time in ms → s (÷ 1000), minus `GetCoolTimeBias` `0x906e30`, clamp to ≥ 0. Stores `IdAndTimer` {limit s, start QPC} in `std::map<int, IdAndTimer>` keyed by group.
- Other methods: `SetSkillRecycleTime_renewal` `0x974fc0`, `ResetSkillRecycling_renewal` `0x94f7f0` (log `0x164d3f8` "SkillTimer ResetSkillRecycling of %d group. timer: %f limit, …"), `IsSkillRecycled_renewal` `0x94f940`, `IsGlobalCoolTimeRecycled_renewal` `0x94f740`, `UpdateRecycle_renewal` `0x9967e0`, `UpdateGlobalRecycle_renewal` `0x9966c0`, `ModifySkillRecycleTimeInGroupIndex` `0x90cde0`, `InitPlayerSkillCoolTime` `0x90cc50`, `SetEffectTime` `0x916160`. Access: `BControlledPlayingActions::GetSkillTimer` `0x10a0c30`, `GetGlobalCoolTime_renewal` `0x10a6780`, `IsSkillRecycled_renewal` `0x10e16b0`, `UpdateSkillRecycleTimer` `0x111d290`.
- Server sources of recycle data: SkillCancel action (`remain_global_recycle_time`, `remain_recycle_time`), Effects 13/14, commands 1261–1263.
- `Data::SkillRecycleGroupType` (419 GM, 8 values) is in the enum TSV.

---

## 6. Target selection

- `BPTPlayer::RequestSetTarget(u64)` `0xaf30a0` sends `G_SelectTargetReqCmd` 134 (`targetId` i64) with `SendToGamed`. Caller: `BPTPlayer::SetPlayerAttackTarget` `0xb0a4e0`. Second builder `0x5a0830` (no caller found ❓).
- Skill target: `BControlledPlayingActions::GetSkillUseTarget` `0x10b9a70`, `BPlayingActions::GetSkillUseTarget` `0x44b1e0`, `getTargetObjectOrTargetGroundPosition` `0x10fe6e0`, `World::FindAttackAbleTargetInPosition` `0x733850`.
- Enums (419 GM): `BPTPlayer::TARGET_ZONE`, `TARGET_MARK_TYPE`, `BPTControlledCreature::TARGET_SUB_TYPE`, `Data::SkillCastingTargetType`, `Data::SkillTargetStateType`.

## 7. Combo and datafile tables

- Combo: `comboStart` `0x10b9b30`, `processSkillForCombo` `0x10cb860`, `CancelCombo` `0x10ca370`, `ResetComboTimer` `0x10a68a0`, `OnChangeComboDuration` `0x10a6830`, `SetLastComboResult` `0x10b9490`, `UpdateComboResult` `0x10b95c0`, `UpdateHitResult` `0x10b9520`, `Pause/ResumeLastComboResult` `0x10a0950`/`0x10a0980`. ❓ Combo timer value not read.
- Tables used by the code above (record pointer types in the calls): `Skill3` (`Data::Skill3RecordPtr`, `Skill3RecordActiveSkillPtr`, `Skill3RecordActionPtr`), `SkillCastCondition3`, `SkillGatherRange3`, `Effect` (`Data::EffectRecordPtr`), `SoulMaskSet`. Table enums: `Data::Skill*`, `Data::Effect*`, `Data::Stance*` in [`07-combat-enums-gm.tsv`](07-combat-enums-gm.tsv). Table format: [client-datafiles](../client-datafiles.md).

---

## 8. Worked examples (body bytes; packet head as in [05 §5](05-game-dispatch.md))

`G_UseSkillReqCmd` 1226, example values: `controlId` 0, `skillId` 0x00011170, variation 0, `targetId` 0x0000000100000ABC, yaw 90°, `equipPos` -1, four empty lists:

```
00                       controlId
70 11 01 00              skillId
00                       skillVariationId
BC 0A 00 00 01 00 00 00  targetId
5A 00                    yaw (degrees)
FF                       equipPos
00 00 00 00 00 00 00 00  4 × repeat count 0
```

One Effect action element with AttackEffect, damage 1234, hitCode 7 (critical), both property pairs empty (5-byte head of zeros ❓):

```
2B 06                    len 43 (with the 2 head bytes), variant 6 Effect
xx xx xx xx              EffectId
xx xx                    EffectSlotIndex
<8 B> <8 B>              casterId, targetId
00 xx                    state, skillInstanceId
00 00 00 00 00           prop (empty)
00 00 00 00 00           extraProp (empty)
07 03 D2 04 00 00 07     effectData: len 7, variant 3 AttackEffect, damage 1234, hitCode 7
```

## 9. Functions file

[`07-combat-functions.tsv`](07-combat-functions.tsv): 1,062 rows. 7 rows found in this slice (unnamed in 381) and 1,055 PDB-match names of the classes `BControlledPlayingActions`, `BPlayingActions`, `BSkillBook`, `PTActiveSkill2`, `SkillTimer`, `SkillTracer`, effect lists, skill panels, and the combat methods of `World`, `BPTCreature`, `BPTPlayer`, `BCreature`, `BPlayer`. PDB-match names can be false (old notes §6.3).

## 10. Open items ❓

- Names of `0x1113950` (useActiveSkill) and `0x10fcad0` (item/equip skill path).
- Action filter `0x722c00` (which variants it skips, and when).
- Handling of 1261 / 1262 (only `GetSkillTimer` seen in `0x10fd7f0`).
- Sender of `G_SkillTriggerReqCmd` 1231.
- Full 381 ↔ 419 GM map of `SKILL_USE_ERROR` (only the values in §1.3 are known).
- `BPTCreature::ProcessActionSubEvent` per variant, and `processActionToOwner` `0x744a20` details.
- `PROP_PAIR` and `T_PROP_SYNC` byte format (§8 uses an empty 5-byte head).
- Skill3 / SkillCastCondition3 / SkillGatherRange3 / Effect field use by the checks.
- Combo timer values.
