# Client.exe reversing notes (build 0.0.381, 2020)

This file collects what we know about the 2020 `Client.exe`.
It is for the Rust server rewrite and for future fun work (new mounts, new movement).
Datafile facts stay in [`client-datafiles.md`](client-datafiles.md).
The full game command list is in [`client-381-game-commands.tsv`](client-381-game-commands.tsv).

Rules for this file:

- Each fact has a source: address, symbol, enum, table or file.
- "381" means the 2020 client (0.0.381.x, x86). "419 GM" means the 2021 GM client with its PDB.
- ❓ marks a fact that we did not verify.
- Sections 1–10: passive reads only. No keys, no crypto internals.
- Section 11 is the exception: GameGuard removal, requested by the operator for
  our private server. Tools live in `~/Desktop/bns-handshake-key/`.

---

## 1. How we got symbols for the 2020 exe

```
 419 GM Client.exe + PDB ──► feature dump (bytes, strings, consts, CFG, calls)
                                         │
 381 Client.exe (no symbols) ──► feature dump
                                         │
                       match.py   : vtables, exact bytes, strings, CFG, consts, call graph
                       match2.py  : ordered anchors + gap alignment + fuzzy call graph
                       datamatch.py: globals and vtables through matched code
                                         │
                       mkpdb.py   ──► v3 PDB for the 381 exe
```

| Item | Value | Source |
| --- | --- | --- |
| Functions matched | 37,543 (83.8 % of the 381 functions) | `c20_named.json` (scratchpad) |
| Name format | MSVC mangled, demangled with `llvm-undname` | `c20_named.json` |
| Named IDA db | `t20/pk.i64` (scratchpad copy) | — |
| Confidence | High for `vtable`, `bytes`, `strings`. Lower for `fuzzy` and `vtslot`. | `how` field per match |

Known false matches: see §6.3 (vehicle names). Treat any name for a feature that has no strings in 381 as ❓.

Matched functions per class (sample):

| Class | Functions | Area |
| --- | ---: | --- |
| BPTPlayer | 155 | player object |
| LobbyGateSession | 55 | lobby protocol |
| PTFishing | 35 | fishing (player side) |
| BPTPlayerSummoned | 33 | summoner cat |
| Fishing | 30 | fishing logic |
| MoveListMaker | 30 | movement packets |
| DrElReaderImpl | 24 | datafile reader |
| GameSession | 15 | game server session |
| BSummonedBrain | 12 | cat AI |
| HyperMove | 12 | stamina moves |
| GamePacketProcessor | 8 | packet framing |
| Glide | 4 | gliding |
| InfoGateSession | 4 | info server |
| PacketParser | 4 | stream to packets |
| PTEffectMount | 2 | mount effect (stubs) |

---

## 2. Networking

### 2.1 Server layout seen by the client

```
             ┌──────────────┐
  client ───►│  LobbyGate   │  L_* packets, LobbyGateCommand 1..110
             └──────┬───────┘
                    │ lobby_token + cli_key
                    ▼
             ┌──────────────┐        ┌──────────┐
  client ───►│  GameD       │◄──────►│ ArenaD   │  PacketDest selects target
             └──────────────┘        └──────────┘
  client ───► InfoGate (I_*), auth (A_*)
```

### 2.2 Command id tables

| Table | 381 | 419 GM | Source |
| --- | --- | --- | --- |
| LobbyGateCommand | 110 values. Same as bns-server `Program/Server/Packet/tables/381.json` (client_max 110, ping 9, ContentsRuleList 106, PcCafe 103). | 110 | GM TPI enum `LobbyGateCommand` |
| Game MainCommand | 1,598 names. Pointer table at `0x1953620` (.data), index 0 = "None", index = command id. | 1,686 | 381 exe; GM enum `MainCommand` (1686/1686 index match) |
| UI button commands | 12 names at `0x1955858` | — | 381 exe |
| InfoGateCommand | — | 73 | GM TPI |
| ActionsCommand | — | 106 (`GS_ActionsClientMoveCmd` 1, `...ClientMoveAndEventJump` 2, `...Skill` 3) | GM TPI |
| GameDebugMainCommand | — | 214 | GM TPI |
| GameBuilderMainCommand | — | 212 | GM TPI |
| StsMsgLib::ECommand | — | 228 | GM TPI |
| Auth `Command` | — | `A_IdAuthReqCmd` ... | GM TPI |

Game command ids, 381 against 419 GM:

```
 id    1 ─────────────── 918 │ 919 ────────────── 1598 (381) / 1686 (GM)
       identical in both     │ ids shift: GM inserts new commands
                             │ 381 #919 = G_OccupyingRaidDungeonChangeNotifyCmd
                             │ GM  #919 = G_UnbindPersonalRaidDungeonReqCmd
```

- Every 381 name exists in GM. GM adds 88 names.
- Added in GM: `G_ActivateVehicle*`, `G_DeactivateVehicle*`, SoulBoost, WorldAccountCard / Museum, TreasureBoard, ImproveItem, PersonalRaidDungeon, PvpResult, FieldBossSpawnWorldAlarm, SyncDynamicData, WebRequest.
- Rule for the server: use the 381 TSV for ids. Do not use GM ids above 918.

### 2.3 Packet description types (StructPacket)

`StructPacket::FieldType` (GM TPI). The packet structs are described by these field kinds:

| Value | Name | Value | Name |
| ---: | --- | ---: | --- |
| 0 | T_INT8 | 9 | T_MSEC |
| 1 | T_INT16 | 10 | T_PROP_PAIR |
| 2 | T_INT32 | 11 | T_PROP_SYNC |
| 3 | T_INT64 | 12 | T_STRUCT |
| 4 | T_INT128 | 13 | T_SUB |
| 5 | T_STRING | 14 | T_REPEAT |
| 6 | T_SBIN | 15 | T_SEQ |
| 7 | T_VBIN | 16 | T_PACKET_REF |
| 8 | T_SU | | |

❓ The wire format probably uses these codes like the LobbyGate format codes in `bns-server/Docs/Server/LobbyGate/03*`. We did not verify it for game packets.

`StructPacket` base (16 bytes, in memory): `_command` short @4, `_dest` @8, `_pending` @12.

### 2.4 Routing, sync groups, events

| Enum | Values | Source |
| --- | --- | --- |
| PacketDest | CURRENT 0, ARENAD 1, GAMED 2, GAMED_ONLY 3, BOTH 4 | GM TPI |
| PacketEventType | NONE 0, CONNECT 1, CLOSE 2, DISCONNECT 3, SEND 4, RECEIVE 5 | GM TPI |
| GamePacketRightType (flags) | NONE 0, ANY 1, NORMAL_PLAYER 2, OBSERVER 4, MOBILE_APP 8 ... | GM TPI |
| PacketProcessor::ErrCode | PP_NO_ERROR 0, INVALID_PARAM 1, NOT_SUPPORT 2, ALLOCATE_FAILED 3, INSUFFICIENT_BUFFER 4 ... | GM TPI |

`GamePacketSync` (one request per group can be pending; the server must answer before the next one ❓):

| Id | Group | Id | Group |
| ---: | --- | ---: | --- |
| 1 | TalkSession | 11 | SkillList |
| 2 | MoveAndWarp | 12 | SkillTrainingList |
| 3 | Action | 13 | PostBox |
| 4 | OpenItemPouch | 14 | Craft |
| 5 | CloseItemPouch | 15 | Market |
| 6 | PickupItem | 16 | Petition |
| 7 | ManufactureItem | 17 | InstantPayment |
| 8 | ExpandInventory | 18 | DuelBotTrainingRoom |
| 9 | Inventory | 19 | WorldAccount (GM only ❓) |
| 10 | Quest | 20 | MAX |

### 2.5 Stream framing (PacketParser::ParseData, 381 `0x5fd770`)

```
 recv bytes ──► ParseData(src, size)
                 │
                 ├─ partial packet pending? ── copy into pending buffer (+16),
                 │                              filled count at +12, needed size at +0
                 │
                 ├─ header complete ──► size callback (vtable at +4) gives packet size
                 │                       size too small → return -1 (drop)
                 └─ full packet ──► PacketPool::AllocPacket ──► packet object at +20 ──► dispatch
```

- Return -1 on null input, size ≤ 0, or a bad size.
- The size callback decides the length from the header. ❓ Header layout is not yet read from code. Use the LobbyGate spec as the model.

### 2.6 Example packet layouts (419 GM, in memory, not wire)

| Struct | Fields (offset: name type) |
| --- | --- |
| G_HelloReqPacket | 16 gamePacketVersion i16, 24 propertyVersion i64, 32 publisher, 40 extraPropertyVersion i64, 48 authData |
| L_HelloReqPacket | 16 packet_version i16, 20 handshake_param |
| L_LoginWithLobbyTokenReqPacket | 16 cli_key i64, 24 lobby_token i64 |
| G_LobbyTokenAuthReqPacket | 16 world_id i16, 24 cli_key, 32 lobby_token |
| G_EnterWorldReqPacket | 16 clientVersion i32, 20 characterId |
| G_EnterZoneOkPacket | 16 enterSocial i64 |
| GS_ActionsClientMoveSubPacket (40 B) | 8 creatureId i64, 16 x i16, 18 y i16, 20 z i16, 22 yaw i16, 24 duration i16, 26 standingSocial i8, 28 moveHistory |
| G_ActivateVehicleReqPacket (GM only) | 16 equipped_vehicle_id i32 |

GM has 2,017 packet struct types named `G_*`, `GS_*`, `L_*`, `I_*` (`tpi.pkl`, scratchpad).

### 2.7 Session classes (381 addresses)

| Function | 381 address |
| --- | --- |
| GameSession::GameSession | `0xefbf40` |
| GameSession::SendEnterArenaReq | `0xf2bf60` |
| LobbyGateSession::SendMakeCharacterRequest | `0xc97480` |
| LobbyGateSession::SendWorldListRequest | `0xc97710` |
| PacketParser::Create | `0x5fd5d0` |
| PacketParser::ParseData | `0x5fd770` |

Lobby enums: `DisconnectLobbyReason` (12 values: NONE 0, DISCONNECTED_GAME 1, DUPLICATE_LOGIN_KICKED_LOBBYGATE 2, ...), `LOBBY_EVENT` (54 values: login, security card, OTP ...). Source: GM TPI.

---

## 3. Characters

### 3.1 Jobs (`CreatureProperty::Job`, GM TPI)

| Id | Job | Id | Job |
| ---: | --- | ---: | --- |
| 1 | BLADE_MASTER | 11 | WARRIOR |
| 2 | KUNG_FU_FIGHTER | 12 | ARCHER |
| 3 | FORCE_MASTER | 13 | SPEAR_MASTER |
| 4 | SHOOTER (gunner) | 14 | THUNDERER |
| 5 | DESTROYER | 15 | DUAL_BLADER |
| 6 | SUMMONER | 16 | PC_MAX |
| 7 | ASSASSIN | 17 | ROOKIE |
| 8 | SWORD_MASTER | 18 | STRIKER |
| 9 | WARLOCK | 19 | DEFENDER |
| 10 | SOUL_FIGHTER | 20 | CONTROLLER |

❓ Which jobs are playable in 381 is not checked. The 381 datafile has `decompose-job-reward-shooter`, so SHOOTER exists in 381.

### 3.2 Races (`Race`, GM TPI)

KUN 1, GON 2, LYN 3, JIN 4, SUMMONED_CAT 10, ADVENTSOUL 11, SWORD_DAEMON 12.

### 3.3 Weapon types (`Data::ItemWeaponTypeType`, GM TPI)

| Id | Type | Id | Type |
| ---: | --- | ---: | --- |
| 1 | BARE_HAND | 10 | DAGGER |
| 2 | SWORD | 11 | PET_1 |
| 3 | GAUNTLET | 12 | PET_2 |
| 4 | AURA_BANGLE | 13 | GUN |
| 5 | PISTOL | 14 | GREAT_SWORD |
| 6 | RIFLE | 15 | LONG_BOW |
| 7 | TWO_HANDED_AXE | 16 | SPEAR |
| 8 | BOW | 17 | ORB |
| 9 | STAFF | 18 | DUAL_BLADE |

Guns: three weapon types exist (PISTOL 5, RIFLE 6, GUN 13). The SHOOTER job uses GUN ❓. A new gun is a data change (item + skill3 + appearance), not a code change ❓.

---

## 4. Movement

### 4.1 Move types (`MoveType`, wire values `G_MOVETYPE_*`, GM TPI)

| Ids | Move |
| --- | --- |
| 0, 1 | WALK, RUN |
| 2–4 | JUMP start / loop / end |
| 5–7 | SPRINT start / loop / end |
| 8–10 | SPRINT_JUMP start / loop / end |
| 11–13 | DRIFT start / loop / end |
| 14 | FALL |
| 15 | SWIM |
| 16–18 | GLIDE |
| 19–21 | NPC_CHASE, NPC_FLEE, NPC_RETREAT |
| 22–24 | BOOST |
| 25–27 | DIVE |
| 28–30 | WALL_RUN |
| 31–33 | WALL_JUMP |
| 34–36 | EVENT_JUMP |
| 37–39 | SUMMON_COMBAT_RUN / SWIM / FALL |
| 40–42 | ATTACK_GLIDE |
| 43 | MAX |

Related enums: `MoveTypeMaker::MoveTypeId` (39 values, same order), `EIMoveType` (animation side: THROW 2, ROLLING 3 ... SUMMONED_* 22–34), `EOWorldInfoMoveType` (16 values for world info: Walk ... EventJump 15).
❓ The 381 values are the same. The order looks stable, but we did not compare against 381 code.

### 4.2 Movement state layouts (GM TPI)

| Struct | Fields |
| --- | --- |
| MoveTypeData (40 B) | 0 moveTypeMakerId i8, 1 moveType i8, 2 startCell i8, 4 startTime f32, 8/12/16 diffX/Y/Z f32, 20/24/28 locationX/Y/Z f32, 32 yaw i32, 36 eventJump i16, 38 isDecelFalling bool |
| HyperMove (88 B) | 0 _state, 4 _max f32, 8 _cur f32, 12 _hyperMove[8], 76 _curHyperMoveType, 80 _block, 84 _isGmMode |
| Glide (3 B) | 0 _state, 1 _isGlide, 2 _countblockGlide |

### 4.3 HyperMove ("Hyper energy", the stamina bar)

`HyperMove::HyperMoveId`: Hyper 0, Sprint 1, Drift 2, Boost 3, Dive 4, WallMove 5, EventJump 6, **RocketJump 7**, Max 8.

| Function | 381 address | What it does |
| --- | --- | --- |
| HyperMove::Start | `0xe93fc0` | |
| HyperMove::Tick | `0xe82b90` | |
| HyperMove::acquireHyperMove | `0xe31ed0` | |
| HyperMove::set | `0xe9ce40` | Gets the player skill book, finds the skill key, calls a UI slot (+100), clears entry `[id]` (8-byte entries at +12: flag, value f32), then calls acquireHyperMove. |

Data hooks: `consume-hyper-energy-point`, `max-hyper-energy-point` (skill3/action), `hypermove-power-value` (pc, npc, mastery-ability, summonedlevel), `enable-infinite-hyper-energy` (zonerule, battle fields). Source: 381 tabledefs.

### 4.4 Glide (`Glide::Set`, 381 `0xf43c60`)

```
 Glide::Set
   ├─ get local player → skill book → skill key
   ├─ look up Skill3 record → action → check byte @612 == 8  (❓ action kind "glide")
   │     yes → _isGlide = 1
   │     no  → _isGlide = 0
   ├─ _isGlide == 1 → decrement _countblockGlide, call UI/actor slot +652
   └─ _isGlide == 0 → Glide::BlockGlide(true)   (0xee0640)
```

Zone hooks: `zoneenv2` airdash / airdash-leave (`airdash-level`), guild battle `airdash-1/2-by-refiner`.

### 4.5 MoveListMaker (sends the client move packets)

| Function | 381 address |
| --- | --- |
| MoveListMaker::MoveListMaker | `0xc61730` |
| playerTick | `0xcc82b0` |
| sendWalkPacket | `0xcc4ce0` (decompile failed, ❓ needs a retry) |
| setPlayerMove | `0xc5b180` |
| playerWarp | `0xc5ae90` |
| SyncServerMove | `0xc5b890` |
| sendCombatHyperMovePacket | `0xc99a00` |
| MoveTypeMaker::SetMoveType | `0xc5ac10` |

The client sends `GS_ActionsClientMoveCmd` (ActionsCommand 1) with `GS_ActionsClientMoveSubPacket` entries (§2.6). Positions are i16 ❓ (relative to a cell, see `startCell`).

`MoveListMaker::SendWalkPacketReason` (why the client flushes moves). The server can log these:

| Id | Reason | Id | Reason |
| ---: | --- | ---: | --- |
| 1 | OVER_TIME | 31 | UNLINK_CATCH |
| 2 | REST | 34 | NONE_OWNER_ATTACH_DASH |
| 3 | NO_MOVE | 35 | NONE_OWNER_DETACH_LINK_MOUNT |
| 4 | LANDING | 36 | NONE_OWNER_DETACH_INHALATION |
| 5 | ENTER_PORTAL | 37 | NONE_OWNER_DETACH_THROW |
| 11 | BEFORE_SKILL | 38 | ATTACH_MOVE_MOUNT |
| 12 | USE_ACTIVE_SKILL | 39 | DETACH_MOVE_MOUNT |
| 13 | AIRDASH_ENV | 43 | ENTER_BLOCKMOVE |
| 15 | DEAD | 49/50 | START/END_TRADE |
| 16 | PLAY_SOCIAL | 53/54 | TALK_START/END |
| 26 | ATTACH_INHALATION | 55/56 | PERSONAL_CUSTOMIZE_START/END |
| 27 | ATTACH_CATCH | 57/58 | FISHING_START/END |
| 28 | DETACH_MOUNT | 59 | BIND_SYNC_DUNGEON |

Full list (60 values) is in GM TPI `MoveListMaker::SendWalkPacketReason`.

---

## 5. Summoner cat (BSummonedBrain, BPTPlayerSummoned)

| Item | Value | Source |
| --- | --- | --- |
| BSummonedBrain::Initialize | `0x90f140` | 381 match |
| BSummonedBrain::GetNextReservedAction | `0x968f30` | 381 match |
| BSummonedAction::EActionType | None 0, Idle 1, Move 2, Recall 3, Skill 4, SequenceSkill 5, SkillCancel 6, HelpRestoration 7 | GM TPI |
| SummonedBrainType | NORMAL 0, COMBAT_FOLLOW 1, FIXED 2 | GM TPI |
| Cat move types | EI_MOVE_SUMMONED_RUN ... SUMMONED_DEST (22–34), G_MOVETYPE_SUMMON_COMBAT_* (37–39) | GM TPI |

`Data::SummonedRecord` (GM, 272 B) key fields: `summoned_type`, `brain_type`, `combat_follow_distance`, `far_event_distance`, `speed`, `max_speed_near_master`, `min/max_distance_to_lerp_speed`, `appearance`, `animset`, `spawn_skill_id`, `lifetime_duration`.
381 tables: `summoned`, `summonedlevel`, `summonedmasterylevel`, `summonedpreset`, `summonedbeautyshop`, `summoneddesignerpreset`.

The cat is a full AI follower with its own move set. It is the closest thing in 381 to a "pet that runs with you".

---

## 6. Mounts, vehicles and grabs

### 6.1 381 has no vehicles

| Check | 381 | 419 GM |
| --- | --- | --- |
| "Vehicle" text (ASCII and UTF-16) | 0 hits | 32 hits (UTF-16) |
| `vehicle` / `vehicleappearance` datafile table | no | yes (`Data::VehicleRecord`, `Data::VehicleAppearanceRecord`) |
| `G_ActivateVehicle*` / `G_DeactivateVehicle*` commands | no | yes |

Conclusion: vehicles came after build 381.

### 6.2 What 381 has instead ("mount" = carry or grab links)

| Mechanic | 381 data hooks | 381 code hooks |
| --- | --- | --- |
| Mount link (one creature rides or holds another) | `effect`: `mount-linked`, `mount-linker`; `skillcastcondition3`: `fail-target-mount-linked/linker` | SendWalkPacketReason DETACH_MOUNT 28, NONE_OWNER_DETACH_LINK_MOUNT 35, ATTACH/DETACH_MOVE_MOUNT 38/39 |
| Catch (grab) | `effect`: `catch-linked/linker`; `effect/link`: `catch-caster-effect`, `catch-target-effect` | ATTACH_CATCH 27, DETACH_CATCH 30, UNLINK_CATCH 31 |
| Inhalation (pull) | `effect`: `inhalation-linked/linker` | ATTACH_INHALATION 26, DETACH_INHALATION 29 |
| Throw | — | NONE_OWNER_DETACH_THROW 37, EI_MOVE_THROW 2 |
| PTEffectMount | — | Activate / Deactivate are 1-byte stubs (`ret`) in 381 |

So 381 can link two creatures and move them together. It uses this for grabs and some boss mechanics ❓. It has no rideable vehicle item.

### 6.3 False vehicle matches in the 381 PDB

| GM name | 381 address | Finding |
| --- | --- | --- |
| PTActionRideVehicle::Start | `0xac32c0` | Generic PTAction start: clears `_isStart`, gets caster, calls player vtable +600 with 0. Not a vehicle. ❓ real name unknown. |
| BPTVehicle::DestroyVehicle | `0xada520` | Decompile failed. Class cannot exist in 381. False. |
| MakeItemPreviewVehicle | `0x1027fc0` | Uses PcAppearance, CreatureAppearance, ItemRecordCostume. Likely a costume preview function ❓. |

Rule: rename these three in the PDB to `sub_*` in a later pass.

### 6.4 How GM vehicles work (reference for a future feature)

```
 item (equipped_vehicle_id) ──► G_ActivateVehicleReq (16: id i32)
                                     │
                    server ──► G_ActivateVehicleOk / Fail
                                     │
       client: BPTVehicle {VehicleHandle, RiderHandle, Owner, VehicleAppearance u64, SprintSpeed i32}
               PTActionRideVehicle {ShowStageHandle, VehicleItemKey u64}
                                     │
       data:   vehicle   {alias, vehicle_speed i16, sprint_speed i32, riding_action_duration i32,
                          appearance → vehicleappearance, context_script}
               vehicleappearance {mesh, materials, anim set/tree, physic_asset,
                          ride/getoff/cancel show,
                          move_rider_*   (idle, walk, run, turn, jump, sprint*, glide*, magnetic_run*)
                          move_vehicle_* (same set for the mount itself)}
```

Source: GM TPI structs `BPTVehicle` (32 B), `PTActionRideVehicle` (40 B), `Data::VehicleRecord` (48 B), `Data::VehicleAppearanceRecord` (252 B), `G_*VehiclePacket`.

---

## 7. Fishing

| Item | 381 address / value |
| --- | --- |
| PTFishing::PTFishing | `0xabab80` |
| PTFishing::ChangeFishingState | `0xad24e0` |
| PTFishing::processFishingHooksetSuccess | `0xad1940` |
| Classes | PTFishing (35 matched), Fishing (30 matched) |
| Move flush | FISHING_START 57, FISHING_END 58 |
| 381 tables | `fish`, `fishing-field`, `fishing-paste` |
| Data hooks | `fishing-point`, `grade-a-fish`, `bonus-reduce-waiting-time-amount`, `dispose-fishing-paste-probability`, `accumulate-life-contents-point-by-fishing` |

---

## 8. Datafile system

See [`client-datafiles.md`](client-datafiles.md). Short version:

- Reader: `DrElReaderImpl` (24 matched functions). `readNative` sets the native size to `2*chars+2`.
- 381 has 277 tables (`bns-server/Docs/DataFile/tabledefs-2020.json`).
- Movement, glide and skill behaviour is mostly data: `skill3`, `skillcastcondition3`, `skillresultcontroll3`, `skill-caster-function-control`, `combat-sequence` (`combat-move/move-type`), `zonepathway` (`move-type`), `zoneenv2`.

---

## 9. Fun stuff: what a skateboard or a horse would touch

Documentation only. Nothing here is built.

```
            ┌────────────── client (381 exe, no code changes possible) ──────────────┐
            │                                                                          │
 Option A   │  reuse existing systems through data only                                │
 "fake"     │   • sprint/drift/boost skill with a new animation (skill3 + effect)      │
 mount      │   • HyperMove ids (Sprint, Drift, Boost, RocketJump) for speed and tricks│
            │   • mount-link effect to attach the player to an NPC "horse"            │
            │   • summoned cat brain as a follower that the player stands on ❓       │
            └──────────────────────────────────────────────────────────────────────────┘
 Option B   GM-style vehicle: needs Vehicle tables + G_ActivateVehicle packets.
 "real"     381 exe has neither → not possible on 381 without client code.
```

| Need | 381 hook | Server side (Rust) |
| --- | --- | --- |
| Faster move speed | `sprint` move types, HyperMove Sprint/Boost, skill3 `fly-speed-*`, effect stats | accept higher speed in move validation |
| Skate "tricks" | Drift (11–13), EventJump (34–36), WallRun (28–30), RocketJump id 7 | allow those move types outside combat |
| Ride an animal | `mount-linked/linker` effect, ATTACH_MOVE_MOUNT 38 | create the link, move both creatures as one |
| Board / horse model | appearance tables + animation sets in the game data | none |
| Start and stop | item use → skill → effect (mount link or speed buff) | handle skill and effect packets |

Best first experiment ❓: a costume-like "skateboard" item that casts a sprint-type skill with a custom social animation. It needs only datafile edits (bnsedit) and server support for the skill.

---

## 10. Open items ❓

| # | Item | Next step |
| ---: | --- | --- |
| 1 | `sendWalkPacket` `0xcc4ce0` decompile failed | redo with a fresh function boundary |
| 2 | Game packet wire format | read the size callback and one encoder; compare with LobbyGate spec 03 |
| 3 | Real names of the 3 false vehicle matches | check callers; rename to `sub_*` |
| 4 | 381 MoveType values equal GM values | find the 381 move-type name table or a switch |
| 5 | Mount-link use in 381 content | search `effect` records with `mount-linker` set |
| 6 | Playable jobs in 381 | read `pc` / `job` tables |
| 7 | Glide check byte @612 == 8 meaning | map Skill3 action layout |
| 8 | Sync group rule (one pending request per group) | confirm on the game server side |

---

## 11. GameGuard removal (protection — active work, not passive)

The passive-only rule above does not apply here. The operator asked to remove
nProtect GameGuard from the client so it runs against our server. The tools live
in `~/Desktop/bns-handshake-key/` (`gg_patch.py`, `gg-unhook-ida.py`,
`remove-gameguard.sh`, `prepare-client.sh`). Source: 2020 `Client.exe`, build
0.0.381 (`ClientGameGuard.cpp`).

### 11.1 How GameGuard is wired in (381 addresses)

```
 WinMain / InfoGate reconnect (sub_115EA90)
     └─ ClientGameSecurity::InitGameGuard            0x10E24C0  (reads INI, builds path)
            └─ ClientGameGuard::InitializeAntiHack   0x115BDF0
                   ├─ PreInitNPGameMonW(path)                  loads GameMon.des
                   ├─ InitNPGameMon()  != 1877 ? → fail        1877 (0x755) = GG "OK"
                   ├─ ok   → byte_1A4C445 = 1 ; return 1
                   └─ fail → byte_1A4C446 = 1 ; return 0
 Per-frame:
     ClientGameGuard::Tick                           0x115BD30  (via thunk 0x10E24B0)
        guard: dword_1A4C45C!=0 && byte_1A4C445!=0 && byte_1A4C446!=1
        every 10 s: CheckNPGameMon()!=1877 → byte_1A4C446 = 1
```

CGameGuard SDK wrapper: `_InitNPGameMon` 0x6F64E0, `_CheckNPGameMon` 0x6F6500,
`_PreInitNPGameMonW` 0x6FAA30, `NPGameMonCallback` 0x115B7C0. The nProtect magic
"all good" return code is **1877 (0x755)**.

### 11.2 The enforcement (what actually kicks)

| Flag | Addr | Meaning | Set by |
| --- | --- | --- | --- |
| `byte_1A4C445` | 0x1A4C445 | GG initialised OK | InitializeAntiHack success |
| `byte_1A4C446` | 0x1A4C446 | **GG failed → quit** | init fail, Tick fail, NPGameMonCallback |
| `dword_1A4C45C` | 0x1A4C45C | GG shared struct ptr | InitializeAntiHack |

Readers of `byte_1A4C446` (the kill switch):

| Site | Addr | Behaviour |
| --- | --- | --- |
| `ClientGameSecurity::AppExitImmediate` | 0x10A03B0 | `return byte_1A4C446;` (whole body) |
| `BNSClient::Tick` | 0x11678CA | `cmp 446,1 ; jnz continue` → else EnterReplay (kick) |
| `WinMain` | 0x1191015 | start-up loop, proceeds to LoadEngine only while 446==0 |

So **while `byte_1A4C446 == 0` the client runs normally.** The caller
`sub_115EA90` also requires `InitGameGuard()` to return true to reach login
(`sub_10DCEF0`); false → replay/error path (system msg 4049).

### 11.3 The patch

Make `ClientGameSecurity::InitGameGuard` return true immediately:

```
 0x10E24C0 (file offset 0xCE18C0)   55 8B EC   →   B0 01 C3   (mov al,1 ; ret)
```

Result: the init body is skipped → GameMon.des never loaded; `445` and `446`
stay 0 → nothing quits; `Tick` is dormant (needs `445!=0`); the caller sees
success and logs in. 3 bytes, size unchanged, no leak (returns before the local
wstring is built). Applied to `Client_patched.exe` (alongside the 128-byte key
patch at 0xFF6380 → 131 bytes changed total).

Locating it across builds (no fixed address): string `"failed to initialize GG"`
(the only user of it) → its single `push` → `InitializeAntiHack` → its single
caller `InitGameGuard` → stub the prologue. Same chain in `gg_patch.py` (bytes)
and `gg-unhook-ida.py` (xrefs).

### 11.4 Other guards in this client (not active self-enforcers)

| Layer | Evidence | Status without a server driving it |
| --- | --- | --- |
| NCGuard (`aegisty.bin`) | `InitializeNCGuardClientW` 0x12C5217, called from `sub_10DCEF0` at start-up | loads `aegisty.bin`; if absent, logs + tries to send `G_NCGuardMessageCmd`, **does not quit**. Return value is ignored by caller. Inert against our server. |
| XignCode | `G_XIGNCode3MessageCmd`, `UI.ExitGame.Kick.XignCode` strings only | protocol stubs; no loader (`x3.xem` etc.) in this build. Inert. |
| GameGuard packets | `G_GameGuardAuthQueryCmd` / `...AnswerReqCmd` / `...AuthFailCmd` | server-driven challenge. Our Rust server never sends them → never triggered. |

Conclusion: GameGuard is the only client-side self-kick. The one 3-byte patch is
sufficient; NCGuard/XignCode need no client change and no server implementation.

### 11.5 Future anti-cheat (EAC / BattlEye) — the seam

`InitGameGuard` → `InitializeAntiHack` is the single client-side AC init point.
We short-circuit it rather than delete it, so it stays as a clean hook site.
EAC and BattlEye integrate differently (own launcher + SDK + their own
client↔server integrity channel, like the `G_*GuardMessageCmd` packets here), so
adding one is a new build/integration step, not a byte patch. Keeping this seam
intact is what makes that future swap straightforward.
