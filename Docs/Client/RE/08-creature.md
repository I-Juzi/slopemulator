# 08 — Client.exe 381: player, NPC and world objects

Summary:

1. Every world object has a 64-bit object id. Bits 48–63 are the object type: PC 1, summoned 2, NPC 4, duel bot 0xF0, duel bot summoned 0xF1, dropped pouch 0xF2 (§1.1). `World::enterCreature` (`0x74b640`) selects the C++ class from this type.
2. The server spawns and despawns creatures with commands 1–5 (`G_EnterLeaveBroadcast`, `G_EncounterOutOfSightBroadcast`, `G_InZone`, `G_InSightCube`, `G_OutSightCube`). The client logic layer is `World` (NetWorld vtable `0x173fbb4`), the 3D layer is `PresentationWorld`, the UI layer is `BUiWorld` (§2).
3. Creature data is a property table. `CreatureProperty::FIELD_INFO` (`0x19f6b48`) has 422 fields (ids 1–422). The server sends a full or partial copy with a `T_PROP_SYNC` field (a fixed field list, no ids on the wire) and changes with a `T_PROP_PAIR` field (a list of `field id + value`) (§3).
4. All 454 property fields of 4 tables are in [`08-creature-properties.tsv`](08-creature-properties.tsv). All 26 sync field lists (1,043 rows) are in [`08-creature-sync-tables.tsv`](08-creature-sync-tables.tsv) (§4).
5. The summoner cat is a creature of type 2 with an own `SummonedProperty` table. The own cat is `BPlayerSummoned` (`World+0x84`), other cats are `BSummoned`. Death is CreatureProperty field 163 `dead`; respawn is `G_RespawnReq` (cmd 142) (§5, §6).

```
 SERVER                         CLIENT (381)
 cmd 1..5 ─► GamePacketHandler[cmd] 0xef6b60.. ─► World (BNSClient+0x4C) NetWorld vtable 0x173fbb4
                                                   ├─ EnterLeaveObject 0x74c370 ─► enterCreature 0x74b640 / leaveCreature 0x74b140
                                                   ├─ EncounterOutOfSight 0x73f710 ─► encounter 0x72bc00 / outOfSightCreature 0x73f3c0
                                                   ├─ InZoneObject 0x74c0c0 ─► enterCreature
                                                   ├─ InSightObject 0x740cd0 ─► encounter 0x72bc00 ─► BCreature EncounterSync
                                                   └─ OutSightObject 0x73f4d0 ─► outOfSightCreature / outOfSightObject 0x73f230
                                                        │ listeners
                                                        ▼
                                       PresentationWorld (3D: 0xb93260, 0xb93500, 0xb411c0) and BUiWorld (UI: 0x7c3cc0, 0x7c3c20)
 cmd 8, 9 ─► World::ProcessNetObjectEvent 0x749550 ─► creature SetPair 0x1049760 ─► TracableProperty::SetPair 0x465d40
                                                   ─► listener OnChangedCreatureProperty(fieldId, new, old) (§3.5)
```

Sources: all facts come from the 381 database `ida/lanes/cx/Client.exe.i64`. "419 GM" marks a name or type that comes only from `Client_gm.exe.i64`. ❓ marks a fact that we did not verify. Many 381 IDA names are false matches from the 419 PDB (old notes §6.3). Where this chapter gives a role that differs from the IDA name, the role is correct and the IDA name is false. Examples: `0xb56290` has the IDA name `PassiveEffectList::GetTargetIndicatorEffectList`, but it is the `BPlayerSummoned` constructor; `0x91cc00` has the name `SkillToolTip::SkillToolTip`, but it is the `BSummoned` constructor ❓; `0x43fdf0` `CreaturePropertySync::FieldCount` and `0x43fe70` `CreaturePropertySync::FieldInfo` are base-class constructors, not the field functions.

Related chapters: command table and packet field descriptors: [05-game-dispatch](05-game-dispatch.md) and its [`packet-layouts.tsv`](packet-layouts.tsv). Movement of creatures: [06-movement](06-movement.md). Old notes: [client-exe-reversing §3 and §5](../client-exe-reversing.md). Server side: [GameDaemon enter world](../../Server/GameDaemon/05-enter-world.md).

Files of this chapter:

| File | Rows | Content |
| --- | ---: | --- |
| [`08-creature-functions.tsv`](08-creature-functions.tsv) | 1,859 | every named 381 function of this area (name filter: creature, player, NPC, summoned, sight, spawn, dead, revive, property sync), with size and first string |
| [`08-creature-properties.tsv`](08-creature-properties.tsv) | 454 | every field of `CreatureProperty` (422), `ExtraCreatureProperty` (9), `EnergyProperty` (13), `SummonedProperty` (10): id, name, type, size, offset, order in the sync lists |
| [`08-creature-sync-tables.tsv`](08-creature-sync-tables.tsv) | 1,043 | every field of the 26 `*Sync` field lists, in wire order |
| [`08-creature-enums-gm.tsv`](08-creature-enums-gm.tsv) | 321 | 419 GM enums of this area (reasons, race, job, stance, dead, summoned brain, and more) |

---

## Quick find

| Task or question | Function or object | 381 address | GM name (419 GM) | Section |
| --- | --- | --- | --- | --- |
| Object type of an id | `id >> 48` (switch in `World::enterCreature`) | `0x74b6cd` | `BGameObject::TYPE` | §1.1 |
| Make an object id from zone and index | `ComposeObjectId` | `0x71f570` | same | §1.1 |
| Find a world object by id | `World::find` | `0x72b670` | same | §1.3 |
| Create a creature (spawn) | `World::enterCreature` | `0x74b640` | same | §2.2 |
| Remove a creature (despawn) | `World::leaveCreature` | `0x74b140` | same | §2.3 |
| Handle cmd 1 (enter / leave) | `World::EnterLeaveObject` | `0x74c370` | same | §2.1 |
| Handle cmd 2 (encounter / out of sight) | NetWorld slot 7 | `0x73f710` | `World::EncounterOutOfSightObject` | §2.1 |
| Handle cmd 3 (zone snapshot) | NetWorld slot 8 | `0x74c0c0` | `World::InZoneObject` | §2.1 |
| Handle cmd 4 (in sight) | NetWorld slot 9 | `0x740cd0` | `World::InSightObject` | §2.1 |
| Handle cmd 5 (out of sight) | `World::OutSightObject` | `0x73f4d0` | same | §2.1 |
| Show a creature (3D) | `PresentationWorld::encounterCreature` | `0xb53270` | same | §2.4 |
| Create the own player | `World::createPlayer` | `0x737d30` | same | §1.2 |
| Create the own cat | `World::createPlayerSummoned` | `0x7325b0` | same | §5.2 |
| Read a `T_PROP_SYNC` field | `PropertySync::Set` | `0x5d4600` | same | §3.2 |
| Read a `T_PROP_PAIR` field | prop-pair reader | `0x5d4810` | `PropertyPair::Set` ❓ | §3.3 |
| Read one pair value | value reader | `0x5d4410` | `VariantField::Set` ❓ | §3.3 |
| Pair table id → field table | `GameSession::PropertyPairFieldInfo` | `0xee1170` | same | §3.3 |
| Pair table id → field count + 1 | `GameSession::PropertyPairFieldMax` | `0xee11a0` | same | §3.3 |
| Apply a pair to a creature | creature `SetPair` | `0x1049760` | `BCreature::SetPair` ❓ | §3.5 |
| Apply changes and call listeners | `TracableProperty<CreatureProperty>::SetPair` | `0x465d40` | same | §3.5 |
| Copy a sync into a property | `Property::SetSync` | `0x5d3d70` | same | §3.4 |
| Field list of CreatureProperty | `CreatureProperty::FIELD_INFO` | `0x19f6b48` | same | §4 |
| Field id of "hp", "level", "dead" | 107, 103, 163 | — | — | §4.2 |
| Respawn request | `BControlledPlayingActions::RequestRespawn` | `0x10d3ad0` | same | §6.2 |
| Cat recall request (cmd 71) | `BPTPlayerSummoned::Recall` | `0xa64640` | same | §5.4 |
| Cat despawn when too far (cmd 636) | builder | `0xb78b00` | ❓ | §5.4 |
| Worked example | `G_EnterLeaveBroadcast` enterSummoned | — | — | §7 |

Search anchors: string `enter Creature failed., same id, different npc. id: %I64X` `0x16b0518` (and 8 more at `0x16b0590`–`0x16b0928`, all in `0x74b640`); `OutOfSight object., object exist., id:%I64X` `0x16b0ae0` (in `0x73f3c0`); `Msg.NewbieCare.Complete` `0x16b0b38` (in `0x74b140`); `Msg.Party.Summoned.Spawn` (in `0x952650`); field name strings `world_id` `0x17c1928` (first entry of `0x19f6b48`); constant `0x200000` (type 0x20 << 16) in `0x71f570`; constants 423, 10, 14 in `0xee11a0`; command ids 142 (`0x8e`) in `0x10d3ad0`, 636 (`0x27c`) in `0xb78b00`, 71 (`0x47`) in `0x59f350`, 980 (`0x3d4`) in `0xb7d5b0`.

---

## 1. Object model

### 1.1 Object id

| Bits | Meaning | Source |
| --- | --- | --- |
| 48–63 | object type (`BGameObject::TYPE`) | `World::enterCreature` `0x74b640`: `switch (HIWORD(HIDWORD(id)))` at `0x74b6cd` and `0x74b947` |
| 32–47 | zone part. For a summoned (type 2): the low 16 bits of the owner PC id (§5.1) | `0x74b640` (summoned branch); `ComposeObjectId` `0x71f570` |
| 0–31 | object index | `ComposeObjectId` `0x71f570` |

`ComposeObjectId` (`0x71f570`, one caller `0x111c687`) returns `((zone & 0xFFFF) + 0x200000) << 32 | index`. The constant `0x200000` is type `0x20` in bits 48–63 (the type argument is folded into the only call) ❓ type 0x20 name.

Type values (381 switch cases in `0x74b640`; names from 419 GM and the log strings):

| Type | Object | 381 class created by `enterCreature` | `new` size | Constructor (381) |
| ---: | --- | --- | ---: | --- |
| 1 | PC (other player) | `BPc` | `0xae0` (aligned 16) | `0xb64490` `BPc::BPc` |
| 1 | own player | `BPlayer` (made by `World::createPlayer` `0x737d30`, not by `enterCreature`) | `0xe58` | `0xb8a610` `BPlayer::BPlayer` |
| 2 | summoned, owner is the own player | `BPlayerSummoned` (kept in `World+0x84`) | `0xe38` | `0xb56290` (false name, see top) |
| 2 | summoned, other owner | `BSummoned` ❓ class name | `0xb00` | `0x91cc00` |
| 4 | NPC | `BNpc` ❓ class name | `0xb00` | `0xbf8830` |
| 0xF0 (240) | duel bot | `BDuelBot` | `0xae8` | `0x105b340` |
| 0xF1 (241) | duel bot summoned | `BDuelBotSummoned` | `0xb00` | `0x1030d30` |
| 0xF2 (242) | dropped pouch | not a creature (cmd 4 `DroppedPouch` list) | — | `BGameObject::TYPE::GO_DROPPED_POUCH` (419 GM) |
| 3, 5–239 | — | `enterCreature` logs nothing and creates nothing (default case `0x74b921`) | — | — |

Env objects, field items, gather sources, NPC dead bodies and campfires have own id lists in cmd 4 and 5 (§2.1). Their id type bits are not checked ❓.

### 1.2 Layers and owners

| Object | 381 address | Role |
| --- | --- | --- |
| `BNSClient` pointer | `0x1a4c0f8` | `+0x4C` = `World` ([05 §1](05-game-dispatch.md)) |
| `World` | `*(0x1a4c0f8)+0x4C` | logic layer. `+0x4` = net-event interface (slot 3 `+0xC` = `ProcessNetObjectEvent`); `+0x8` = NetWorld interface (vtable `0x173fbb4`); `+0x80` = own `BPlayer`; `+0x84` = own `BPlayerSummoned` |
| NetWorld vtable | `0x173fbb4` | slot 1 `EnterWorld` `0x741a50`, 4 `EnterZone` `0x74c850`, 5 `LeaveZone` `0x744bf0`, 6 `EnterLeaveObject` `0x74c370`, 7 `0x73f710`, 8 `0x74c0c0`, 9 `0x740cd0`, 10 `OutSightObject` `0x73f4d0`, 11 `GodSays` `0x747b60`; slots 2 `0x73f910` and 3 `0x73e6c0` ❓ |
| `PresentationWorld` | — | 3D layer: `addPTPlayer` `0xb68b10`, `encounterCreature` `0xb53270`, `InSightObject` `0xb93260`, `InZoneObject` `0xb93500`, `OutSightObject` `0xb411c0`, `remove` `0xb40d30` |
| `BUiWorld` | — | UI layer: `InSightObject` `0x7c3cc0`, `OutSightObject` `0x7c3c20`, `SpawnNpc` `0x7d99c0`, `SpawnPlayerSummoned` `0x7b8b50`, `OnChangedCreatureProperty` `0x7c1220` / `0x78c930`, `OnChangedPcProperty` `0x7ce070` |

The handlers of cmd 1–5 (`0xef6b60`, `0xef6b20`, `0xef6ae0`, `0xef6aa0`, `0xef6a60`) all do the same: read `World` from `BNSClient+0x4C`, take the NetWorld interface at `World+8`, call vtable slot `+0x18`, `+0x1C`, `+0x20`, `+0x24`, `+0x28`. The handlers of cmd 8 and 9 (`0xef69a0`, `0xee6680`) call `World+4` slot `+0xC` (`World::ProcessNetObjectEvent` `0x749550`).

Creature classes (381 vtables `{for BGameObject}`): `BPc` `0x161850c`−0x12C, `BControlledCreature` `0x152c98c`−0x12C, `BDuelBot` `0x154ed24`−0x12C, `BDuelBotSummoned` `0x154ee6c`−0x12C. (The address is the slot that holds `0x1049760`, slot 75.) Class tree (419 GM): `BGameObject` → `BCreature` → `BPlayer` / `BPc` / `BNpc` / `BSummoned` → `BPlayerSummoned`. Each `BCreature` has the presentation peer `BPTCreature` (`BPTPlayer`, `BPTPc`, `BPTNpc`, `BPTSummoned`, `BPTPlayerSummoned`).

### 1.3 Object store

`World::find(id)` (`0x72b670`) returns the `BGameObject*` for an id or null. Other lookups: `World::GetPTPlayer` `0x722d70`, `World::GetPTPlayerSummoned` `0x722d50`, `World::GetItemPouch` `0x732140`, `World::cleanUpObjects` `0x72e9f0` (called at the end of cmd 1 and cmd 5).

---

## 2. Spawn and despawn packets

### 2.1 Commands

All fields are in [`packet-layouts.tsv`](packet-layouts.tsv) (cmd 1–5). Summary:

| Cmd | Name | Lists (element fields) | World function | What the client does |
| ---: | --- | --- | --- | --- |
| 1 | `G_EnterLeaveBroadcast` | `zoneId` i64; `leave` (creatureId i64, leaveReason i8, leaveSocial i64); `enter` (creatureId i64, enterReason i8, `CreaturePropertyEnterSync`); `enterSummoned` (creatureId i64, enterReason i8, `SummonedPropertyEnterSync`) | `0x74c370` | for each `leave`: `leaveCreature` (`0x74c3c9`); for each `enter` and `enterSummoned`: `enterCreature` (`0x74c3f5`, `0x74c41d`); then listeners (3 virtual calls), `SkillTrainingRoom::EnterLeaveObject` `0x9237f0`, `TendencyField` `0x8de390`, `PartyBattleField` `0xc044f0`, `BattleRoyalField` `0x11a9160`; then `cleanUpObjects` |
| 2 | `G_EncounterOutOfSightBroadcast` | `outOfSight` (creatureId, outOfSightReason i8); `encounter` (creatureId, encounterReason i8, enterSocial i64, `CreaturePropertyEncounterSync`, `effects` list of `EffectPropertySync`); `encounterHoldingFieldItem` | `0x73f710` | `outOfSightCreature` `0x73f3c0` per id; encounter `0x72bc00` per creature; `BParty::EncounterOutOfSightObject` `0xc13e90` |
| 3 | `G_InZone` | zone state, `creature` (creatureId, `CreaturePropertyEnterSync`), `summonedCreature` (creatureId, `SummonedPropertyEnterSync`), boss HP, refiners, squads | `0x74c0c0` | `enterCreature` per creature (`0x74c122`, `0x74c14a`), item pouches, then field handlers (`GuildBattleField` `0xf3c750`, `PartyBattleField` `0xbecb70`, `PublicRaidField` `0xa8f8e0`, `DuelBotTrainingRoom` `0x1030a50`, `SkillTrainingRoom` `0x992b80`) |
| 4 | `G_InSightCube` | `creature` (creatureId, `CreaturePropertyEncounterSync`, `effects`); `EnvObject`; `FieldItem`; `GatherSource`; `NpcDeadBody`; `HoldingFieldItem`; `Campfire`; `DroppedPouch` | `0x740cd0` | encounter `0x72bc00` per creature, `World::encounterNpcDeadBody` `0x740a50`, item pouches, `BParty::InSightObject` `0xc13dd0` |
| 5 | `G_OutSightCube` | id lists: `creature`, `envObject`, `fieldItem`, `gatherSource`, `npcDeadBody`, `campfire`, `droppedPouch` | `0x73f4d0` | `outOfSightCreature` `0x73f3c0` per creature; `outOfSightObject` `0x73f230` for the 6 other lists; `BParty::OutSightObject` `0xc13ce0`; `BTeam::OutSightObject` `0x86e840`; `cleanUpObjects` |

Two steps per creature: "enter" creates the object with its appearance (`EnterSync`: world_id, race, sex, job, appearance1–92, name, faction); "encounter" (in sight) fills position, HP, level and state (`EncounterSync`, 119 fields). A creature can be in the zone (entered) but out of sight.

Reason enums (419 GM, [`08-creature-enums-gm.tsv`](08-creature-enums-gm.tsv)):

| Enum | Values |
| --- | --- |
| `EnterReason` | 0 ENTER_WORLD, 1 ZONE_TRANSIT, 2 REVIVE, 3 NPC_SPAWN, 4 RETURN, 5 SUMMONED_SPAWN, 6 SUMMONED_RESPAWN, 7 MOVE_TO_ARENA, 8 RETURN_TO_GAME, 9 DUEL_BOT_SPAWN, 10 ENTER_ATTRACTION_SHORTCUT |
| `LeaveReason` | 0 RELOAD, 1 LEAVE_WORLD, 2 PORTAL, 3 DECOMPOSE, 4 RETURN, 5 TELEPORT, 6 SCRIPT, 7 CHANGE_CHANNEL, 8 DISMISS_PARTY, 9 SUMMONED_RESPAWN, 10 ENTER_ARENA_LOBBY, 11 RETURN_TO_ARENA_LOBBY, 12 AIRDASH, 13 ENTERED_FATIQUE_TIME, 14 DUEL_ROUND_START, 15 ZONE_CLOSE, 16 WARP_TO_GUILD_MEMBER, 17 WARP_TO_GUILD_MEETING_PLACE, 18 PARTY_BATTLE_START, 19 RETURN_TO_ARENA_DUNGEON, 20 ENTER_ATTRACTION_SHORTCUT, 21 LEAVE_ATTRACTION_SHORTCUT, 22 NPC_PORTAL, 23 PVP_RESULT_WARP_TO_OPPONENT, 24 ENTER_ATTRACTION_SHORTCUT_NO_RETURN |
| `EncounterReason` | 0 ZONE_IN, 1 WALK_IN, 2 WARP_IN |
| `OutOfSightReason` | 0 ZONE_OUT, 1 WALK_OUT, 2 WARP_OUT |

Client use of the reasons (381): encounter reason 2 (WARP_IN) sets a flag byte in the creature object (`0x72bc00`, `if (a5 == 2)`) ❓ flag meaning. `enterCreature` stores the enter reason in the own cat at `BPlayerSummoned+0xAE0` (2784) (`0x74b640`).

### 2.2 `World::enterCreature` (`0x74b640`)

Arguments: id (u64), sync (`PropertySync*`), enter reason (i8).

1. `World::find(id)` (`0x74b69f`). If an object exists, check that it is the same kind (switch on type at `0x74b6cd`):

   | Type | Check | Log string on error (ClientLog, `World.cpp`) |
   | ---: | --- | --- |
   | 4 | object is NPC and `BNpc::GetNpcDataID` (`0xbcf160`) equals the new data id | `enter Creature failed., same id, different npc. id: %I64X` `0x16b0518`; `... existing creature is not npc ...` `0x16b0590` |
   | 2 | object is summoned and same data | `... same id, different summoned ...` `0x16b0610`; `... existing creature is not summoned ...` `0x16b0690` |
   | 1 | object is PC and same data | `... same id, different pc ...` `0x16b0720`; `... existing creature is not pc ...` `0x16b0798` |
   | 0xF0 | `BDuelBot::GetDuelBotDataID` (`0x10103e0`) equal | `... same id, different duel bot ...` `0x16b0818`; `... not duel bot ...` `0x16b0898` |
   | 0xF1 | object is duel bot summoned | `... existing creature is not duel bot summoned ...` `0x16b0928` |

   On a mismatch the client logs and calls `leaveCreature(id, reason, 3)` (`0x74b759`, `0x74b91c`), then makes a new object.
2. Create the object (second switch `0x74b947`, table in §1.1). For type 2 the client compares the low 16 bits of the own player id (`World+0x80` → `+8`) with bits 32–47 of the new id. Equal → own cat: reuse or create `BPlayerSummoned` at `World+0x84`, call `BPlayerSummoned::Enter` (`0xb55920`). Not equal → `BSummoned`.
3. Initialize the object from the sync (virtual `Initialize(PropertySync*)`: `BPlayer::Initialize` `0xbb5260`, `BNpc::Initialize` `0xc1f3f0`, `BSummoned::Initialize` `0x92fff0`). These call `TracableProperty<CreatureProperty>::SetSync` `0x465c40` → `Property::SetSync` `0x5d3d70`.
4. Add to the world maps (`TMap<u64,int>::Add` `0x72e890`), friend check (`BAccount::IsFriend` `0x1246bc0`), UI: `BUiWorld::SpawnPlayerSummoned` `0x7b8b50`, `BUiWorld::SpawnNpc` `0x7d99c0`, `GuildBattleField::SpawnNpc` `0xefda00`, `PublicRaidField::SpawnNpc` `0xa34ae0`, minimap `MinimapPanel::IsPlayerSummonedMapUnitEnable` `0x72aad0`.

### 2.3 `World::leaveCreature` (`0x74b140`) and out of sight

`leaveCreature(id, reason, social)`: removes object events (`BPlayer::RemoveObjectEvent` `0xb85310`), removes the target (`BPTPlayer::RemoveTarget` `0xb11750`), calls listeners, `World::find`; for the own cat `BPlayerSummoned::LeaveZone` (`0xb558b0`) and `BUiWorld::SpawnPlayerSummoned(false)`; NPC UI clean-up (`BUiWorld::SpawnNpc` `0x7d99c0`, `GuildBattleField::SpawnNpc`); duel observer camera (`ArenaDuel::UpdateObserverCameraState` `0x125ee90`); party newbie message `Msg.NewbieCare.Complete` (`0x16b0b38`, via `World::InvokeGameMessage` `0x401ea0`).

`World::outOfSightCreature(id, reason)` (`0x73f3c0`): hides the creature, it stays in the world. Log `OutOfSight object., object exist., id:%I64X` (`0x16b0ae0`) ❓ condition. Then `PresentationWorld::outOfSightCreature` `0xaa2b70`.

### 2.4 Encounter (`0x72bc00`, GM name `World::encounterCreature` ❓)

1. If the id is the own player (`World+0x80`, id at `+8`): if not in sight, call `InSight`, then vtable `+264` (EncounterSync) with the sync.
2. Else `World::find(id)`. Unknown id → return 0 (no object is made; "encounter" needs a prior "enter"). If not in sight: `InSight`, then `EncounterSync(sync, 0, effects)`.
3. If the reason is 2 (WARP_IN), set a flag in the object.

The 3D side: `PresentationWorld::InSightObject` (`0xb93260`) → `PresentationWorld::encounterCreature` (`0xb53270`) → `BPTCreature::EncounterSync` (`0xad5c30`); also `addPTFieldItem` `0xb48480`, `encounterHoldingFieldItem` `0xb530a0`, `encounterGathersource` `0xb53020`, `addPTCampFire` `0xb48390`, `encounterNpcDeadBody` `0xb52f30`, `addPTDroppedPouch` `0xb48520`, env object state `BEnvObject::ChangeState` `0xfdd1a0`. Logic side: `BCreature::EncounterSync` (caller of `0x465c40`).

---

## 3. Property sync

### 3.1 Types

`Property::FInfo` (20 B, 381 data matches the 419 GM type):

| Off | Name | Type | Notes |
| ---: | --- | --- | --- |
| 0 | `id` | i16 (stored as 32 bit) | field id, 1-based. In a `*Sync` list: the id of the field in the base property table |
| 4 | `type` | `Property::FType` | 1 F_INT8, 2 F_INT16, 3 F_INT32, 4 F_INT64, 5 F_BOOL, 6 F_STRING, 7 F_BINARY, 8 F_VARBINARY (0 F_UNKNOWN, 9 F_TYPE_MAX) |
| 8 | `size` | i16 | byte size; 0 for F_STRING and F_VARBINARY |
| 12 | `name` | `const wchar_t*` | field name, for example `world_id` |
| 16 | `offset` | u32 | offset in the C++ property object (vtable at 0, first field at 4) |

A property class has a vtable with (slot 1) table id and (slot 2) field count (419 GM; `CreatureProperty` table id 2). A sync class (`*Sync`) has a vtable with 3 slots: 0 destructor, 1 `FieldCount` (`mov eax, N`), 2 `FieldInfo` (`mov eax, offset table`). The 381 sync vtables are at `0x179c2d4`–`0x179c400` (12 B each); [`08-creature-sync-tables.tsv`](08-creature-sync-tables.tsv) gives all.

### 3.2 `T_PROP_SYNC` wire format (`PropertySync::Set`, `0x5d4600`)

Packet field type 11 ([05 §3.2](05-game-dispatch.md)). The packet struct holds a sync object (with its vtable) at the field offset. `PacketSession::setField` (`0x5d4f20`, case 11) calls `PropertySync::Set(object, stream)`; the return value is the bytes used (0 = error → setField returns -1).

There is no length and no field id on the wire. The reader walks `FieldInfo()[0 .. FieldCount()-1]` in order:

| Field type | Wire form |
| --- | --- |
| F_BOOL (5) | packed: one byte holds up to 8 consecutive F_BOOL fields. Bit `i` (mask `STRUCT_BIT_MASK[i]` `0x19499f8` = 1,2,4,…,128) → field `i` of the run. A new byte starts after 8 bools or at the first non-bool field |
| F_STRING (6) | `i16 n` + `n` UTF-16LE code units (`PropString::Set`) |
| F_VARBINARY (8) | `i16 n` + `n` bytes (`PropVarBin::Set`) |
| other with `size > 0` (F_INT8/16/32/64, F_BINARY) | `size` bytes, little-endian, copied to `object + offset`. If `size` > bytes left → error (return 0) |
| other with `size <= 0` | error (return 0) |

Then `Property::SetSync` (`0x5d3d70`) copies each sync field into the base property: for sync entry `k`, base entry = `base.FIELD_INFO[id-1]` (`Property::CopyField`).

### 3.3 `T_PROP_PAIR` wire format (field type 10, changes)

Reader in `PacketSession::setField` `0x5d4f20` case 10, then `0x5d4810`, then `0x5d4410` per value.

| Part | Size | Meaning |
| --- | ---: | --- |
| `table` | i8 | property table id: 2 `CreatureProperty`, 17 `ExtraCreatureProperty`, 40 `EnergyProperty` (`GameSession::PropertyPairFieldInfo` `0xee1170`). Other ids → no field table |
| `n` | i16 | must be > 0 ❓ meaning (pair count?). The reader only checks it |
| `len` | i16 | byte length of the pair list |
| pairs | `len` | repeated: `i16 fieldId` + value. `fieldId` must be ≥ 1 and < max (`GameSession::PropertyPairFieldMax` `0xee11a0`: table 2 → 423, 17 → 10, 40 → 14) and the field entry must have `id ≠ 0` |

If `table`, `n` or `len` is ≤ 0, the field is only the 5-byte head (no pairs). Value form per field type (`0x5d4410`):

| Field type | Value bytes |
| --- | --- |
| F_INT8 | 1 |
| F_INT16 | 2 |
| F_INT32 | 4 |
| F_INT64 | 8 |
| F_BOOL | 1 (≠ 0 = true) |
| F_STRING | `i16 n` + `2·n` bytes UTF-16LE (the client adds a 0 terminator) |
| F_BINARY | `size` bytes (fixed) |
| F_VARBINARY | `i16 n` + `n` bytes |

An error in a value stops the list and writes a `LT_ERROR` log entry (`0x5d4810`). Each pair becomes a 16-byte `VariantField` node (`next`, `fieldInfo`, `value`; 419 GM) in a linked list in `PropertyPair` (`head` at +4, `_typeId` at +8).

Commands with `T_PROP_PAIR`: 8 `G_ChangedPrivateProperty` (controlId i8, list `propertys`), 9 `G_ChangedOtherProperty` (controlId i8, list `properties`), 190 `G_BroadcastPartyProperty`, 269 `G_BroadcastTeamPartyMemberProperty` ([`packet-layouts.tsv`](packet-layouts.tsv)). ❓ meaning of `controlId` (in 06 "control id" is the controlled creature slot).

### 3.4 Where each sync class is used

| Sync class (381 vtable) | Fields | Base table | Packets (cmd: field) |
| --- | ---: | --- | --- |
| `CreaturePropertySync` `0x179c2e0` | 422 (all) | CreatureProperty | 32 `G_EnterWorldOk.creatureInfo`; 627, 629 `creatureProp`; 628 `creatureProp`; 649 `G_ReturnToGameOk`; 706 `G_EnterArenaOk` |
| `CreaturePropertyEnterSync` `0x179c2ec` | 98 | CreatureProperty | 1 `enter`; 3 `creature` |
| `CreaturePropertyEncounterSync` `0x179c2f8` | 119 | CreatureProperty | 2 `encounter`; 4 `creature` |
| `SummonedPropertySync` `0x179c304` | 10 | SummonedProperty | 627, 628, 629 `summonedProp` |
| `SummonedPropertyEnterSync` `0x179c310` | 10 | SummonedProperty | 1 `enterSummoned`; 3 `summonedCreature` |
| `EffectPropertySync` `0x179c358` | 6 | EffectProperty | 2, 4 `effects` |
| `EnergyPropertySync` `0x179c3f4` | 13 | EnergyProperty | 836 `G_EnergyPropertySync` |
| others (21) | — | — | see [`08-creature-sync-tables.tsv`](08-creature-sync-tables.tsv) and the `T_PROP_SYNC` rows of [`packet-layouts.tsv`](packet-layouts.tsv) |

The 381 class names come from a name match against the 419 GM sync tables (Jaccard ≥ 0.97 for all except `PvpResultDeadPropertySync` 0.75 and the 3-field table `0x179c2d4` with `optionType`, `optionData`, no match ❓). `0x179c304` and `0x179c310` have the same 10 fields; the names come from the packet that uses them.

### 3.5 Change flow (cmd 8 / 9)

```
HandleChangedPrivatePropertyPacket 0xef69a0 / HandleChangedOtherPropertyPacket 0xee6680
  → World::ProcessNetObjectEvent 0x749550 → object (by id / control id ❓) net-event
    → creature SetPair 0x1049760 (all creature vtables, slot 75 of the BGameObject vtable)
         pair._typeId == 2  → TracableProperty<CreatureProperty>::SetPair 0x465d40 (property at creature+0x20)
         pair._typeId == 17 → TracableProperty<ExtraCreatureProperty>::SetPair 0x465e20 (at creature+0x9A8)
       (one class, vtable 0x16180e0 ❓: 0xb6f850 → SetPair(pair, short) 0x5017d0)
    → for each pair: copy current field to the "old" copy, set the field;
      if the value changed and a listener is set, call listener slot 1 (fieldId, new, old)
      → BCreature::ChangedPropertyCallback 0x10717f0 / BPlayer 0xbc4390 / BNpc 0xc39f20 / BSummoned 0x952650
      → UI listeners: BUiWorld::OnChangedCreatureProperty 0x7c1220, OnChangedPcProperty 0x7ce070, …
```

`TracableProperty<CreatureProperty>` (381 layout from `0x465d40`): current property at +0, old copy at +0x4BC (1212), listener pointer at +0x978 (2424). Setters used by client code: `Set<char>` `0x510420`, `Set<short>` `0x465f00`, `Set<int>` `0x465fe0`, `Set<__int64>` `0x5019b0`, `Set<wchar_t const*>` `0x546670`.

There are 120 functions named `*::OnChangedCreatureProperty` or `*::OnChangedPcProperty` in 381 (panels, quest journal, party, minimap, duel). They are in [`08-creature-functions.tsv`](08-creature-functions.tsv) (grep `OnChanged`).

---

## 4. Creature properties

### 4.1 Tables

| Table | Pair id | 381 `FIELD_INFO` | Fields (381) | Fields (419 GM) | Fields only in 419 GM |
| --- | ---: | --- | ---: | ---: | --- |
| `CreatureProperty` | 2 | `0x19f6b48` | 422 | 433 | activated_vehicle_id, vehicle_speed, variation_id, login_reset_time, defend_difficulty_type_damage_reduce_percent, personal_raid_dungeon_1..3_expiration_time, personal_raid_dungeon_1..3_progress_step |
| `ExtraCreatureProperty` | 17 | `0x19f9ac8` | 9 | 10 | acquired |
| `EnergyProperty` | 40 | `0x19fd958` | 13 | 14 | (one, name not read) ❓ |
| `SummonedProperty` | — (no pair) | `0x19fe3c0` (unnamed in 381) | 10 | 11 | variation_id |

All 381 field names exist in 419 GM. The 381 IDA name `SummonedProperty::FIELD_INFO` (`0x186c788`) is false.

### 4.2 Main CreatureProperty fields

Full list: [`08-creature-properties.tsv`](08-creature-properties.tsv). Type: i8 = F_INT8, and so on. "Enter" / "Enc" = the field is in `CreaturePropertyEnterSync` / `CreaturePropertyEncounterSync`.

| Id | Name | Type | Off | Enter | Enc | Values |
| ---: | --- | --- | ---: | :-: | :-: | --- |
| 1 | world_id | i16 | 4 | yes | | |
| 2 | race | i8 | 6 | yes | | `CreatureProperty::Race`: 1 KUN, 2 GON, 3 LYN, 4 JIN, 5–9 TESTMOB1–5, 10 SUMMONED_CAT, 11 ADVENTSOUL, 12 SWORD_DAEMON |
| 3 | sex | i8 | 7 | yes | | 1 MALE, 2 FEMAIL (GM spelling), 3 NEUTER |
| 4 | job | i8 | 8 | yes | | `CreatureProperty::Job` 1–20 ([old notes §3.1](../client-exe-reversing.md)) |
| 5–96 | appearance1–92 | i8 | 9–100 | yes | | face and body sliders |
| 97 | name | string | 104 | yes | | |
| 98 | geo_zone | i32 | | | yes | |
| 99–101 | x, y, z | i16 | | | yes | position, unit ❓ (see [06](06-movement.md)) |
| 102 | yaw | i16 | | | yes | |
| 103 | level | i8 | 124 | | yes | |
| 104 | exp | i32 | 128 | | yes | |
| 105 | mastery_level | i8 | | | yes | |
| 107 | hp | i64 | 144 | | yes | |
| 109 | money | i64 | 160 | | | |
| 111 | faction | i8 | | yes | yes | |
| 113 | guild_id | i64 | | | yes | |
| 120, 121 | sp, sp2 | i16 | | | yes | |
| 122 | stance | i8 | | | yes | `CreatureProperty::Stance` (GM TSV) |
| 127 | target | i64 | | | yes | |
| 128 | action_type | i8 | | | yes | `CreatureProperty::ActionType`: 1 HELP_RESTORATION, 2 PICKING, 3 FIRST_ATTACKER, 4 PC_SOCIAL, 5 MANIPULATE, 6 FISH_MANIPULATE |
| 142 | summoned | i64 | 352 | | yes | id of the own summoned (cat). `BSummoned::ChangedPropertyCallback` `0x952650` tests field 142 |
| 163 | dead | i8 | 426 | | yes | `CreatureProperty::Dead`: 0 NOT_DEAD, 1 DEAD, 2 EXHAUSTION, 3 RESTORATION |
| 164 | gm_mode | i8 | | | yes | |
| 202 | ghost_type | i8 | | | yes | 0 NONE, 1 BASIC, 2 BEYOND, 3 PHASE |
| 212 | account_level | i16 | 704 | | yes | |
| 218–227 | combat_mode, falling, convoy, attackable, casting, immune_breaker_disable, npc_interactive, npc_talkable, special_skill_gauge_full, pvp_mode | bool | | | yes | 10 bools = 2 packed bytes in the encounter sync |
| 244 | max_hp | i64 | 776 | | yes | |
| 248, 249 | max_sp, max_sp2 | i16 | | | yes | |
| 250 | speed | i16 | | | yes | |
| 284, 285 | attack_power_creature_min / max | i32 | 892 | | | |
| 288 | defend_power_creature_value | i32 | 904 | | | |
| 344 | invisible | bool | | | yes | |
| 368–401 | equip_hand … equip_badge_appearance_item | i32 / i8 / i64 | | | yes (31 of them) | visible equipment |
| 422 | hypermove_power_equip_value | i16 | 1212 | | | last field |

`CreaturePropertyEncounterSync` type mix: 38 i8, 16 i16, 20 i32, 26 i64, 19 bool. `CreaturePropertyEnterSync`: 96 i8, 1 i16, 1 string.

---

## 5. Summoner cat (summoned)

### 5.1 Objects

| Object | 381 address | Role |
| --- | --- | --- |
| Object id | type 2 | bits 32–47 = low 16 bits of the owner PC id (§2.2 step 2) |
| `BPlayerSummoned` | ctor `0xb56290`, size `0xe38` | the own cat, at `World+0x84`. `Enter` `0xb55920`, `LeaveZone` `0xb558b0`, `RequestRecall` `0xb78ad0`, `RequestHelpRestoration` `0xb84620` |
| `BSummoned` ❓ name | ctor `0x91cc00`, size `0xb00` | a cat of another player. `Initialize` `0x92fff0`, `ChangedPropertyCallback` `0x952650` |
| `World::createPlayerSummoned` | `0x7325b0` | makes `BPlayerSummoned` (`0xe38`) when cmd 627 or 629 comes and `World+0x84` is empty (`World::ProcessNetObjectEvent` `0x749550`, cases `0x273` and `0x275`) |
| `BSummonedBrain` | `Initialize` `0x90f140` (size `0x194`), `GetNextReservedAction` `0x968f30` (size `0x352`) | client AI of the own cat. `Initialize` gets the owner id, the cat id, the `SummonedRecord` and a `BSummonedSensor` (from the 419 symbol signature). `GetNextReservedAction` has only one string, the source path (`0x165a9f8`, an assert) |
| `SummonedProperty` | `0x19fe3c0` (10 fields) | own property table of the cat (§4.1). Sync lists `0x179c304` and `0x179c310` |

`BSummoned::Initialize` (`0x92fff0`) sets CreatureProperty field 142 (`summoned`), runs `SetSync` of the `SummonedProperty`, then sets race, job and name. It computes a lifetime from a record field (`record+144` × 0.001, seconds) ❓. `BSummoned::ChangedPropertyCallback` (`0x952650`) tests field 142 and shows `Msg.Party.Summoned.Spawn` (`0x1659c94`).

### 5.2 Brain and actions (419 GM enums)

The 381 brain was not decompiled in full ❓. The GM enums give the states ([`08-creature-enums-gm.tsv`](08-creature-enums-gm.tsv)):

| Enum (419 GM) | Values |
| --- | --- |
| `BSummonedAction::EActionType` | 0 None, 1 Idle, 2 Move, 3 Recall, 4 Skill, 5 SequenceSkill, 6 SkillCancel, 7 HelpRestoration |
| `BSummonedBrain::EUseActionErrorReason` | 0 None, 1 Distance, 2 Properties |
| `BPTPlayerSummoned::MOVE_STATE` | 0 NONE, 1 FOLLOW, 2 CHASE |
| `BPTPlayerSummoned::FOLLOW_STATE` | 0 FOLLOW_STOP, 1 FOLLOW_MOVE |
| `BPTPlayerSummoned::RUN_DEST` | 0 BACK_LEFT, 1 BACK_RIGHT, 2 FRONT_RIGHT, 3 FRONT_LEFT |
| `BPlayerSummoned::ESkillFinishedReason` | 0 Normal, 1 ServerFail, 2 ClientFail, 3 Cancel |
| `Data::SummonedBrainTypeType` | 0 NORMAL, 1 COMBAT_FOLLOW, 2 FIXED |
| `Data::SummonedEventActionTypeType` | 0 NONE, 1 RECALL, 2 DESPAWN |

Timers: the brain timers were not found ❓. The only verified distance rule is the despawn rule in §5.4.

### 5.3 Packets

Fields from [`packet-layouts.tsv`](packet-layouts.tsv); handlers and builders from [`game-commands.tsv`](game-commands.tsv). Wire id = command id unless the table shows a different wire id.

| Cmd | Name | Dir | Fields | Handler or builder (381) |
| ---: | --- | --- | --- | --- |
| 71 | `G_RecallSummonReq` | C2S | x, y, z (`T_SU`), yaw | `0x59f350` (factory), `BPTPlayerSummoned::Recall` `0xa64640` (picks a spot near the player), builder `0xd0d140` |
| 72 | `G_RecallSummonFail` | S2C | error i16 | `0xef17b0` → `ProcessNetObjectEvent` |
| 627 | `G_SummonedEnterWorldOk` | S2C | creatureId i64, `summonedProp` (`SummonedPropertySync`), `creatureProp` (`CreaturePropertySync`) | `0xef3f90` → `ProcessNetObjectEvent` case `0x273` |
| 628 | `G_SummonedPropertyChangedNotify` | S2C | `summonedProp`, `creatureProp` | `0xef3f50` |
| 629 | `G_MakeSummonedOk` | S2C | same as 627 | `0xef3f10` → case `0x275` |
| 630 | `G_ChangeSummonedReq` | C2S | npcId, item list | `SummonedBeautyShopPanel::requestBuyBeautyItem` `0x938d40` |
| 631 / 632 | `G_ChangeSummonedOk` / `Fail` | S2C | — / error i16 | `0xef3ed0` / `0xef3e90` |
| 633 | `G_ChangeSummonedNameReq` | C2S | summonedName string, consumeItems | vtable builders `0x41bfb0`, `0x41bfe0`, `0x587570` |
| 634 / 635 | `G_ChangeSummonedNameOk` / `Fail` | S2C | — / error i16 | `0xef3e50` / `0xef3e10` |
| 636 | `G_KillSummonedReq` | C2S | (empty) | `0xb78b00` |
| 637 / 638 | `G_KillSummonedOk` / `Fail` | S2C | — / error i16 | `0xef3dd0` / `0xef3d90` |

The cat appears to other players with cmd 1 `enterSummoned` / cmd 3 `summonedCreature` (`SummonedPropertyEnterSync`), and with cmd 2 / 4 as a normal creature (`CreaturePropertyEncounterSync`).

### 5.4 Cat flow

```
server cmd 627/629 ─► ProcessNetObjectEvent 0x749550 ─► (World+0x84 empty) createPlayerSummoned 0x7325b0
                                                     ─► SetSync summonedProp + creatureProp
server cmd 1 enterSummoned ─► enterCreature 0x74b640 ─► own? BPlayerSummoned::Enter 0xb55920 : new BSummoned 0x91cc00
                                                     ─► BUiWorld::SpawnPlayerSummoned 0x7b8b50
own cat AI (BSummonedBrain 0x90f140 / 0x968f30) ─► cat too far from player ─► G_KillSummonedReq 636 (0xb78b00)
player "recall" ─► BPlayerSummoned::RequestRecall 0xb78ad0 ─► BPTPlayerSummoned::Recall 0xa64640 ─► G_RecallSummonReq 71
server cmd 1 leave ─► leaveCreature 0x74b140 ─► BPlayerSummoned::LeaveZone 0xb558b0, SpawnPlayerSummoned(false)
```

`0xb78b00` sends cmd 636 when the distance from the player to the cat is greater than a float in the client config at `+1272` (1272 = `0x4F8`) ❓ config field name and default value.

---

## 6. Death and revive

### 6.1 State

Death is CreatureProperty field 163 `dead` (i8, offset 426). It is in `CreaturePropertyEncounterSync`, so other clients get it with cmd 2 / 4, and changes come with cmd 8 / 9 (`T_PROP_PAIR`, table 2, field 163). Values (`CreatureProperty::Dead`, 419 GM):

| Value | Name | Meaning ❓ (from names) |
| ---: | --- | --- |
| 0 | NOT_DEAD | alive |
| 1 | DEAD | dead, can respawn |
| 2 | EXHAUSTION | down, an ally can help (`action_type` 1 HELP_RESTORATION) |
| 3 | RESTORATION | an ally restores the creature now |

The client reacts in the property listeners (§3.5). Skills can test the state (`Data::SkillCastCondition3DeadStateType` 0–5, 419 GM). The cat can help a down player (`BSummonedAction` type 7 HelpRestoration; `BPlayerSummoned::RequestHelpRestoration` `0xb84620`).

### 6.2 Packets

| Cmd (wire) | Name | Dir | Fields | 381 function |
| --- | --- | --- | --- | --- |
| 142 | `G_RespawnReq` | C2S | respawnType i8 at +16: 0 NORMAL, 1 UNEQUIP_FACTION_COSTUME (`RespawnType`, 419 GM) | `BControlledPlayingActions::RequestRespawn` `0x10d3ad0` |
| 143 | `G_RespawnOk` | S2C | — | `0xef10d0` → `ProcessNetObjectEvent` |
| 144 | `G_RespawnFail` | S2C | error i16 | `0xeeeec0` (own handler, not through `ProcessNetObjectEvent`; not decompiled ❓) |
| 980 (wire 986) | `G_ReviveByAttractionRuleReq` | C2S | (empty) | builder `0xb7d5b0` |
| 1251 (wire 1310) | `G_PickupDeadBodyReq` | C2S | one field ❓ (see [`packet-layouts.tsv`](packet-layouts.tsv)) | `0x10d25c0`, `0x10d4360` |
| 1252 (wire 1311) | `G_PickupDeadBodyFail` | S2C | error i16 | `0xef1470` |

`RequestRespawn` (`0x10d3ad0`) sends cmd 142 only when: the own player exists; no request is pending (flag at `+12` of the actions object); the player byte at `+458` is not 0 ❓ meaning. After the send it sets two `World` flags ❓.

After a respawn the server moves the player: cmd 1 `leave` (reason ❓) and `enter` with `EnterReason` 2 REVIVE, or a property change of `dead` to 0 ❓ (server spec: [GameDaemon](../../Server/GameDaemon/10-session-handlers.md)).

### 6.3 Dead UI

`DeadCommandPanel::Start` (`0x1060730`, size `0x103`) uses the UI names `DeadCommand_WarptoNearpoint` (`0x1533ccc`) and `DeadCommand_Cancel` (`0x1533d04`). The other 381 `DeadCommandPanel::*` names are very small functions and are probably false ❓. NPC dead bodies come in cmd 4 `NpcDeadBody` and go in cmd 5 `npcDeadBody` (`World::encounterNpcDeadBody` `0x740a50`, `PresentationWorld::encounterNpcDeadBody` `0xb52f30`).

---

## 7. Worked example

### 7.1 Another player's cat enters (cmd 1, `enterSummoned` element)

The server sends one element of the `enterSummoned` list. The bytes are in wire order, little-endian. The id and the values are an example.

| Bytes | Value | Field | Reader |
| --- | --- | --- | --- |
| `05 00 00 00 07 00 02 00` | `0x0002000700000005` | creatureId: type 2 (bits 48–63), owner PC id low bits `0x0007`, index 5 | packet list reader |
| `05` | 5 | enterReason = SUMMONED_SPAWN | i8 |
| `0A` | 10 | race (SummonedProperty, i8 @4); value 10 = `Race::SUMMONED_CAT` in CreatureProperty ❓ same enum in SummonedProperty | `PropertySync::Set` `0x5d4600`, fixed size 1 |
| `01` | 1 | job (i8 @5) | fixed size 1 |
| `03 00` `4D 00 69 00 6F 00` | "Mio" | name (F_STRING @8): i16 n = 3, then 3 UTF-16 units | `PropString::Set` |
| `01 02 03 04 05 06 07` | — | appearance1–7 (i8 @16–22) | fixed size 1 each |

Client steps: `World::EnterLeaveObject` `0x74c370` → `enterCreature(0x0002000700000005, sync, 5)` `0x74b640` → type 2 → owner bits `0x0007` ≠ own player low bits → `new BSummoned` (`0xb00`, `0x91cc00`) → `BSummoned::Initialize` `0x92fff0` → `BUiWorld::SpawnPlayerSummoned` is not called (only for the own cat).

### 7.2 HP change (cmd 8, `T_PROP_PAIR`)

| Bytes | Field |
| --- | --- |
| `00` | controlId i8 ❓ |
| `01 00` | list count of `propertys` = 1 (packet list) |
| `02` | table = 2 (`CreatureProperty`) |
| `01 00` | n = 1 (must be > 0) ❓ |
| `0A 00` | len = 10 |
| `6B 00` | fieldId = 107 (`hp`, F_INT64) |
| `40 0D 03 00 00 00 00 00` | hp = 200,000 |

The client reads the pair with `0x5d4810` / `0x5d4410`, then `SetPair` `0x1049760` → `TracableProperty<CreatureProperty>::SetPair` `0x465d40` → listener `(107, new, old)`.

---

## 8. Open items (❓)

| Item | 381 address |
| --- | --- |
| Meaning of the `i16 n` in the `T_PROP_PAIR` head (pair count?) | `0x5d4f20` case 10, `0x5d4810` |
| Meaning of `controlId` in cmd 8 / 9 and how `ProcessNetObjectEvent` selects the object | `0x749550` |
| NetWorld slots 2 and 3 | `0x73f910`, `0x73e6c0` |
| Class names of the NPC and other-cat constructors (`BNpc`, `BSummoned`) | `0xbf8830`, `0x91cc00` |
| Name of object type `0x20` made by `ComposeObjectId` | `0x71f570`, caller `0x111c687` |
| Id type bits of env objects, field items, pouches | `0x740cd0`, `0x73f230` |
| Flag that encounter reason 2 sets | `0x72bc00` |
| Config float at `+1272` (cat despawn distance) | `0xb78b00` |
| Player byte `+458` and the two `World` flags in `RequestRespawn` | `0x10d3ad0` |
| `G_RespawnFail` handler (decompile failed) | `0xeeeec0` |
| Brain states, timers and the action queue of `BSummonedBrain` | `0x90f140`, `0x968f30` |
| Lifetime of a `BSummoned` (`record+144`) | `0x92fff0` |
| One `EnergyProperty` field that is only in 419 GM | `0x19fd958` |
| 381 values of the enums that come only from 419 GM | [`08-creature-enums-gm.tsv`](08-creature-enums-gm.tsv) |
| Table `0x179c2d4` (3 fields `optionType`, `optionData`, …) class name | `0x179c2d4` |
| Listener TSV for the 120 `OnChanged*Property` functions (now only in the functions TSV) | — |
