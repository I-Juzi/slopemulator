# 15 — Engine: base, map loading, options, sound and video

Client.exe 0.0.381 (x86). Source: the 381 IDB. The 419 GM IDB did not open in this session (IDA internal error 1228). All "419 GM" names come from the 419 GM type library (til) that is in the 381 IDB.

**Summary**

1. Client.exe has the game logic only. The engine is in `bsengine_Shipping.dll`. `LoadEngine` `0x114c6d0` loads it, and the exe calls it through one interface object, `EInterface` (version "812", see [01 — startup](01-startup.md)).
2. The engine is a UE4 build ("BNSR"). The build paths contain `\Program\ue4\BNSR\Source\ClientEXE\`, and the maps are `.umap` files (§1.1).
3. `EInterface` has about 886 vtable slots. The 381 table has some slots that are not in the 419 GM table, so 381 offset + shift = 419 offset (§1.3, [15-engine-einterface.tsv](15-engine-einterface.tsv)).
4. Zone load: the zone id gives the Zone record, then the Terrain record, then the umap name. `UiTransitControl` runs the load state machine. When the map is ready, it sends `G_EnterZoneReqCmd` (42) to GameD (§1.5–§1.7).
5. View distance and cull options go only to the engine. No option path sends a packet to the server (§2). The exe has no sound or video library version strings. Audio and Bink video are inside the engine DLL (§3).

```
 WinMain → MainFramework (EI call at 0x1190edc)
   │ LoadEngine 0x114c6d0 ── bsengine_Shipping.dll ── EInterfaceGetInstance (ptr 0x1a4be5c)
   │ EI+0x8 EnterEngineFramework(Tick, PostTick, ReplayTick, ...)
   ▼
 BNSClient::Tick 0x1167630 ── EI+0xd50 GetEEventQueue ── EngineEvent (type 1..55)
   │                                         │
   │                                         ▼
   │                          UiTransitControl::ProcessEvent 0x7c1950
   ▼                                         │ 4 MAPLOADINGCOMPLETE, 27 ASYNC..., 34 KISMET...
 UiTransitControl::PostTick 0x7d9680 ── state 1..15 ── enterWorld 0x7d8e70 / changeZone 0x7d8bf0
   │                                         │
   │  loadZoneMap 0x78fed0: Zone → Terrain → umap → EI+0x14 MapFunc_StartMap
   │  poll EI+0x24 MapFunc_QueryAsyncLoadComplete + _transitMinTime
   ▼
 sub_78E8F0 → BUiWorld::RequestEnterZone 0x7815c0 → BNSClient::SendToGamed 0x114ce20 (cmd 42) → GameD
```

Related chapters: [00 — inventory](00-inventory.md), [01 — startup](01-startup.md) (LoadEngine, version check 812, error 2002), [05 — game dispatch](05-game-dispatch.md) (framing, `SendToGamed`), [06 — movement](06-movement.md) (engine unit ↔ server unit), [10 — quests and NPC](10-quests-npc.md) (`G_EnterZoneReqCmd` layout), [14 — UI](14-ui.md) (`UIEngineInterface`, option table). This chapter does not repeat them.

## Quick find

| Task | Function / item | 381 address | 419 GM name | Section |
| --- | --- | --- | --- | --- |
| Load engine DLL | `LoadEngine` | `0x114c6d0` | — | §1.2 |
| Get engine interface | `EInterfaceGetInstance` (import pointer) | `0x1a4be5c` | `EInterface` | §1.3 |
| Start engine main loop | EI slot 0x8 call in `MainFramework` | `0x1190edc` | `EnterEngineFramework` | §1.4 |
| Read engine events | `BNSClient::Tick` | `0x1167630` | `GetEEventQueue` (EI+0xd50) | §1.4 |
| Zone load state machine | `UiTransitControl::PostTick` | `0x7d9680` | `Ui::TransitState` | §1.5 |
| Set transit state | `UiTransitControl::SetTransitState` | `0x754610` | — | §1.5 |
| Engine event to transit | `UiTransitControl::ProcessEvent` | `0x7c1950` | `EngineEvent::type` | §1.5 |
| Zone id → umap name | `UiTransitControl::loadZoneMap` | `0x78fed0` | `MapFunc_StartMap` (EI+0x14) | §1.6 |
| Tell GameD map is ready | `BUiWorld::RequestEnterZone` | `0x7815c0` | `G_EnterZoneReqCmd` 42 | §1.7 |
| Sub-level streaming | `EventSubLevel::set` | `0xf95091` | `LoadSubLevel` (EI+0x38) | §1.8 |
| Partial client check | `BNSClient::IsFullClient` | — ❓ | `GetLevelDownloadInfo` (EI+0x118) | §1.8 |
| Data directory | `BNSClient::GetDataDirectoryPath` | ref `0x1129d45` | — | §1.9 |
| Apply options to engine | `Ui::ApplyEngineConfiguration` | `0x82bbe0` | `OptFunc_*` (EI+0x7f0..0x888) | §2 |
| Per-level particle/LOD | `ApplyTestOptimizeOption` | `0x1120370` | EI+0x168..0x1a4 | §2 |
| Raid NPC cull distance | `BPTNpc::updatePublicRaidFieldCullDistance` | `0xacf8b0` | `ObjFunc_SetFixedCullDistance` | §2 |
| Audio device list | `AudioDeviceComboBox::RefreshAudioDeviceList` | `0xbf21d0` | EI+0xbe4.. | §3 |
| Sound handle | `CreateSoundEngineHandle` | `0xf7bff0` | `PlaySoundReturnHandle_2` (EI+0xc34) | §3 |
| Windows mixer | `MixerLib::Control::SetDetails` | `0x6005f0` | — | §3 |

## 1. Engine base

### 1.1 Engine generation

| Fact | Evidence (381) |
| --- | --- |
| UE4-based build, project name BNSR | Build path strings `E:\DailyBuild02\Client-Common-0.0.381\depot\Branch\381NC\Program\ue4\BNSR\Source\ClientEXE\Private\*.cpp`, for example `0x14fd7b8` (Client.cpp). About 25 such strings, `0x14c5f60`..`0x159a270`. |
| Maps are `.umap` | `Start_001.umap` `0x1697de0` (no xref). Log `"update data - no umap name, zone id : %d"` in `loadZoneMap` (`0x790312`). |
| No package file names | The exe has no `.pak`, `.upk` or `.uasset` strings. Package I/O is inside the engine DLL ❓. |
| Compiler | Imports `MSVCR80` / `MSVCP80` (Visual C++ 2005). |
| Asset reference format | `<8-digit package>.<object path>`, for example `"00007975.PC.LinF"` (EI+0x36c/0x510 `SetVoiceSet`), `"00010057.Effect_Waypoint"` (EI+0x9a8 `ParticleFunc_SetTemplate`), `"00011949.PostProcess_Slow"` (EI+0xc50). UI packages: see [14 — UI](14-ui.md) §1.3. |

An older, UE3-style path list exists in `ClientExpireCheck::fileList` `0x1955840`: `../contents/data/xml.dat` `0x14f3f9c`, `../contents/data/*.*` `0x14f3f84`, `../contents/bns/CookedPC/*.*`, `../contents/bns/Datasheet/*.*`, `../contents/Engine/Shaders/*.*`, `../bin/*.dll` (strings near `0x14f3f20`), `../bin/*.exe` `0x14f3f10`. Only `Wiper::Execute` `0x5b03e0` (client expiry clean-up, writes `bsw.dat` `0x179ddac`) uses this list. ❓ Why it keeps UE3 paths. This chapter gives no more detail about this feature.

Imports of Client.exe (module, count): ADVAPI32 41, CRYPT32 11, GDI32 2, KERNEL32 152, MSVCP80 179, MSVCR80 205, MSWSOCK 1, NETAPI32 1, RPCRT4 7, SHELL32 5, SHLWAPI 1, USER32 7, VERSION 3, WININET 14, WINMM 13, WS2_32 43, `game_presence-32-vc_v80-MD` 2, IPHLPAPI 2, ole32 5. No Direct3D, DirectSound or video library is imported. The engine DLL owns all of them.

### 1.2 Engine DLL

`LoadEngine` `0x114c6d0` loads `bsengine_Shipping.dll` and gets the engine exports (`EInterfaceGetInstance`, `EInterfaceFree` `0x14fe1d4`, `EInterfaceSetClientInfo`, `UIEngineInterfaceGetInstance`). The interface version check ("812") and error 2002 are in [01 — startup](01-startup.md). The UI side (`UIEngineInterface`) is in [14 — UI](14-ui.md) §1.2.

### 1.3 `EInterface` vtable

- Call pattern (4876 xrefs to `0x1a4be5c`): `call EInterfaceGetInstance` → `mov reg,[eax]` → `call [reg+offset]`.
- The 419 GM til type `EInterface_vtbl` is 3580 bytes (895 slots). The 381 table ends near 0xdd4 (0x24 shorter).
- To get the 419 GM name of a 381 slot, add the shift. The shift was found with a fit of the pushed argument bytes against the til signatures. 553 of 576 used slots pass the check.

| 381 offset range | Shift (381 → 419) |
| --- | --- |
| 0x000–0x388 | 0 |
| 0x390–0x5b0 | +0xc |
| 0x5b8–0x680 | +0x18 |
| 0x690–0x778 | +0x1c |
| 0x77c–0x784 | +0x20 |
| 0x788–0xbf8 | +0x24 |
| 0xc00–0xc64 | +0x28 |
| 0xc78–0xdd4 | +0x24 |

Gaps (no fit): 0x38c, 0x5b4, 0x684–0x68c, 0xbfc, 0xc68–0xc74. The 0xc68–0xc74 range is possibly a 381-only slot group ❓.

Full list: [15-engine-einterface.tsv](15-engine-einterface.tsv). Columns: `offset_381_hex`, `slot_381`, `shift_to_419`, `name_419gm`, `check` (args_ok / args_differ / gap / blank), `call_sites_381`, `first_call_addr`, `first_caller_func`, `callers_sample`, `string_args_seen`, `type_419gm_til`.

Slots used in this chapter (381 offset; the name is from 419 GM after the shift):

| 381 offset | 419 GM name | 381 caller / evidence |
| --- | --- | --- |
| 0x8 | `EnterEngineFramework` | `0x1190edc` in `MainFramework` |
| 0x14 | `MapFunc_StartMap` | `loadLobbyMap` `0x7545d8`, `loadArenaLobbyMap`, `loadZoneMap` |
| 0x1c / 0x20 | `WorldFlagInit` / `ClearLevelStreaming` | transit functions |
| 0x24 | `MapFunc_QueryAsyncLoadComplete` | 12 sites, `zoneCinema` `0x759f9c` |
| 0x28 / 0x2c | `DynamicLoadLevel` / `DynamicUnloadLevel` | `loadDynamicSubLevel` `0x7509a0`, `unLoadDynamicSubLevel` `0x750970` |
| 0x38 / 0x3c | `LoadSubLevel` / `UnLoadSubLevel` | `EventSubLevel::set` `0xf95091` / `clear` |
| 0x44 | `SystemFunc_Exit` | `loadZoneMap` `0x790426` |
| 0x48 | `SystemFunc_ExecCommands` | string `"SHOW SCREENDEBUGMSGS OFF"` |
| 0x118 | `GetLevelDownloadInfo` | `BNSClient::IsFullClient` |
| 0x124 | `SystemFunc_ExitMiniLauncher` | `loadZoneMap` `0x790414` |
| 0x130 | `Restrict15ClientMode` | `MainFramework` |
| 0x14c | `UseUITextureDelayedLoading` | `MainFramework` |
| 0x168–0x1a4 | emitter visibility, particle spawn rates, `ApplyCharLod`, `UpdateCountForHiddenObject` | §2 |
| 0x1ac | `RemoveTextureTopMipMap` | low-memory path |
| 0x1ec | `PlayerFunc_SetWorldInfo` (`EOWorldInfo`) | `loadLobbyMap` `0x7545c1`, `loadZoneMap` |
| 0x23c | `GetMoveHistoryCount` | `MoveListMaker::checkMove` |
| 0x410 (+0xc) | `PlayerFunc_SetTickPauseFlag` | `BUiWorld::RequestEnterZone` `0x7815fa` |
| 0x7e0 (+0x24) | `ObjFunc_SetFixedCullDistance` | `BPTNpc::updatePublicRaidFieldCullDistance` `0xacf8b0` |
| 0x7f0–0x888 (+0x24) | `OptFunc_*` | `Ui::ApplyEngineConfiguration` `0x82bbe0`, OptionPanel `0xc523f0` |
| 0x98c (+0x24) | `ShowFunc_SetParticleCullDistance` | — |
| 0xbc8 (+0x24) | `PathFunc_EnablePathEngine` | `loadZoneMap` |
| 0xbe4–0xbf4 (+0x24) | audio device list, `AudioMuteFade`, `SetAudioReverb` | §3 |
| 0xc00–0xc10 (+0x28) | `SetDefaultSoundMode`, `PlayDynamicBGM`, `StopDynamicBGM`, `SetSoundGroupMode`, `ModifyMusicPitch` | §3 |
| 0xc1c (+0x28) | `Play` | `UiCommonContents::PlayGameMessageSound` |
| 0xc34 (+0x28) | `PlaySoundReturnHandle_2` | `CreateSoundEngineHandle` `0xf7bff8` |
| 0xc3c / 0xc48 | `PlayKismetSequence` / `StopKismetSequence` | 55 sites, string `"Music_Stop"` |
| 0xc50 | `PostProcessFunc_Lock` | `"00011949.PostProcess_Slow"` |
| 0xc78 (+0x24) | `ReplayFunc_Play` | `loadReplayMap` `0x78c9d0` |
| 0xd18 | `MiscFunc_GetPositionByTag` | `"CharPreview_Pos_1"` |
| 0xd50 | `GetEEventQueue` | `BNSClient::Tick` `0x116768c` |
| 0xd54 | `IsValidHandle` | 559 sites |

### 1.4 Main loop and engine events

`MainFramework` calls EI+0x8 `EnterEngineFramework` at `0x1190edc`. The arguments are: hInstance, hPrevInstance, nCmdShow, `BNSClient::Tick`, `BNSClient::PostTick`, `BNSClient::ReplayTick`, publisher, language, window title, `true`, and two flags. After this call, the engine owns the main loop and calls the three tick callbacks.

Before this call, `MainFramework` calls these EI slots (381 offsets): 0x138, 0x130, 0xce8, 0x48c, 0x4c, 0x88, 0xca8, 0x50, 0x108, 0x10c, 0x14c, 0x154, 0x150. Names: see the TSV.

`BNSClient::Tick` `0x1167630`:
1. Gets the engine event queue (EI+0xd50) and reads all events.
2. Event type 55 `EE_CRITICALLOG` writes the log `"[Engine] Critical Log: %s"`.
3. Sends other events through `ClientEventQueue::Pop` → `EventTransmitter<…>::Transmit` to the handlers (for example `UiTransitControl::ProcessEvent`).
4. Also calls `DelayedCall::Tick`, `NPSession::Tick`, `saveLocalOption` `0x1137100`, `saveSyncOption` `0x115b6b0`, `InfoGateSession::ReconnectInfoGate`. Log `"exit by security. (%s)"`.

All 55 `EngineEvent::type` values (419 GM) and `EnginePlayType` are in [15-engine-enums-gm.tsv](15-engine-enums-gm.tsv).

### 1.5 `UiTransitControl` state machine

Object layout (419 GM, size 2104):

| Offset | Field | Type |
| --- | --- | --- |
| +0 | vtable (`EngineEventHandler`) | ptr |
| +4 | `_state` | `Ui::TransitState` (u8) |
| +5 | `_mapLoadingComplete` | bool |
| +6 | `_asyncLoadComplete` | bool |
| +7 | `_kismetMapLoadingComplete` | bool |
| +8 | `_transitStep` | `UiTransitControl::TransitStep` |
| +16 | `_transitMinTime` | `PreciseTimer` |
| +32 | `_isIntro` | bool |
| +40 | `_key` | i64 |
| +48 | `_address` | int (IPv4) |
| +52 | `_port` | i16 |
| +54 | `_replayFilePath` | wchar[1024] |

`PostTick` `0x7d9680` does work only when `_transitStep` is not 0 and not 5. It selects the handler by `_state`:

| State | Handler | 381 address |
| --- | --- | --- |
| 1 | `enterLobby` | `0x7cf890` |
| 2 | `enterWorld` | `0x7d8e70` |
| 3, 4 | `changeZone` | `0x7d8bf0` |
| 5 | `enterCinema` | `0x78f340` |
| 6 | `zoneCinema` | `0x759f70` |
| 7 | `worldToLobby` | `0x7cf790` |
| 8 | `worldToLogin` | `0x7cf680` |
| 9, 13 | `enterArenaLobby` | `0x7d8a50` |
| 10 | `enterArena` | `0x7d93d0` |
| 11 | `enterGuildCustomize` | `0x790660` |
| 12 | `leaveGuildCustomize` | `0x7d8970` |
| 14 | `enterReplay` | `0x7905a0` |
| 15 | `leaveReplay` | `0x7cf5c0` |
| 17 | DISCONNECT, special case in `SetTransitState` `0x754610` | — |

All `Ui::TransitState` (17 values) and `TransitStep` (5 values) names: [15-engine-enums-gm.tsv](15-engine-enums-gm.tsv).

Other functions: ctor `0x754710`, `loadLobbyMap` `0x754570`, `loadArenaLobbyMap` `0x759b70`, `loadReplayMap` `0x78c9d0`, `loadZoneMap` `0x78fed0`, `ShowLoading` `0x7c16a0`.

Engine events in `ProcessEvent` `0x7c1950`:

| Type | 419 GM name | Action |
| --- | --- | --- |
| 1 | `EE_UI` | UI sub-type 74 → `ShowLoading` |
| 4 | `MAPLOADINGCOMPLETE` | set flag, `PresentationWorld::UpdateEventSubLevel` |
| 17 | `ENGINE_INITIALIZED` | set mouse values 1.0 / 0.7; `loadLobbyMap` if `byte_1A4C43B` ≠ 1 |
| 27 | `ASYNCMAPLOADQUERYRETURN` | set `_asyncLoadComplete` |
| 32 | `FINISHCHARACTERLOAD` | `sub_759E60` (gets World and Ui) |
| 34 | `KISMETMAPLOADINGCOMPLETE` | set `_kismetMapLoadingComplete` |
| 48 | `NONEXISTENTMAPLOAD` | checks GRuleConfig +314/+313 and `IsFullClient` (map not downloaded) |
| 50 | `DYNAMICLEVELSTREAM` | dynamic sub-level result |
| 51–53 | replay events | replay load / end ❓ exact meaning |

Flow of `enterWorld` and `changeZone`:

```
 delete old world UI
   → World::IsDestZoneResource
   → loadZoneMap 0x78fed0           (EI+0x14 StartMap)
   → Ui::CreateWorld
   → each PostTick: EI+0x24 QueryAsyncLoadComplete
                    + PreciseTimer::IsOverLimit(_transitMinTime)
   → all done → sub_78E8F0 0x78e8f0
        → UiStateCinema::PlayCinema (if a cinema is set)
        → PresentationWorld::UpdateSceneEffect
        → BUiWorld::RequestEnterZone 0x7815c0  (§1.7)
```

### 1.6 Zone id → umap name

`loadZoneMap` `0x78fed0`:

1. Zone id → Zone record (Zone table, vfunc +132).
2. Zone record +82 (i16) = terrain id → Terrain record.
3. Terrain record +20 = umap name (wchar*). The name goes to EI+0x14 `MapFunc_StartMap`.
4. Terrain record +24/+26/+28/+30 = four i16 values. They are copied into `PresentationWorld`. ❓ meaning (possibly map bounds).
5. Calls EI+0x1ec `PlayerFunc_SetWorldInfo` with an `EOWorldInfo`, and EI+0xbc8 `PathFunc_EnablePathEngine`.

Errors:
- No zone record → SystemText 5002.
- No terrain record → SystemText 5001.
- Empty umap name → log `"update data - no umap name, zone id : %d"` `0x790312`.
- After an error: `Ui::ExitGame`, or EI+0x124 `SystemFunc_ExitMiniLauncher` and EI+0x44 `SystemFunc_Exit`.

Server note: the server must send a zone id that has a Zone record and a Terrain record with a umap name in the client datafile. If not, the client exits.

### 1.7 Map ready → server (worked example)

`BUiWorld::RequestEnterZone` `0x7815c0`:
1. Calls `World::RePrepareEnterZone`.
2. Calls EI+0x410 `PlayerFunc_SetTickPauseFlag` (pauses the player tick).
3. Gets `PresentationWorld::GetSummonedEnterZoneInfo` (summoned x, y, z, yaw).
4. Allocates the packet, sets command id 42, and sends it with `BNSClient::SendToGamed` `0x114ce20`.

Packet `G_EnterZoneReqCmd` (42, C2S, 28 bytes). The layout is from [10-quests-commands.tsv](10-quests-commands.tsv). Header bytes 0–15: see [05 — game dispatch](05-game-dispatch.md).

| Offset | Field | Type | Example |
| --- | --- | --- | --- |
| 16 | `spawnActionKey` | i32 | 0 |
| 20 | `summonedX` | i16 (server unit) | 0 |
| 22 | `summonedY` | i16 | 0 |
| 24 | `summonedZ` | i16 | 0 |
| 26 | `summonedYaw` | i16 | 0 |

Example: a normal zone entry with no summon has all body fields 0 ❓ (not checked in a capture). ❓ The answer command is not traced in this chapter; see [05 — game dispatch](05-game-dispatch.md) (`G_PrepareEnterZoneCmd` 41).

### 1.8 Level streaming and partial client

- Sub-levels: `EventSubLevel::set` `0xf95091` calls EI+0x38 `LoadSubLevel`; `clear` calls EI+0x3c `UnLoadSubLevel`. `PresentationWorld::UpdateEventSubLevel` runs on event 4.
- Dynamic levels: `UiTransitControl::loadDynamicSubLevel` `0x7509a0` (EI+0x28), `unLoadDynamicSubLevel` `0x750970` (EI+0x2c). Result comes back as event 50.
- Partial client: `BNSClient::IsFullClient` uses EI+0x118 `GetLevelDownloadInfo`. When a map is missing, the engine sends event 48. `LevelDownloadConfirmType` values: see the enum TSV.

### 1.9 Data paths

| String | Address | User |
| --- | --- | --- |
| `..\contents\` | `0x14fd8dc` | `BNSClient::GetDataDirectoryPath` (ref `0x1129d45`), `GetWebDataDirectoryPath` (ref `0x1129c37`) |
| `\contents\local\` | `0x151fe3c` | `CommonWebBrowserCallback::getContentsPublisherPath` |
| `datafile.bin` | `0x13f3b7c` | `BNSClient::LoadData` (ref `0x114c03a`) |
| `..\bin\Version.ini` | `0x14fd9c4` | ProductVersion read |
| `Splash.bmp` | `0x14fda1c` | start splash |
| `..\bin\SystemText.ini` | `0x14fdfac` | system text |

`FileModifyChecker::InternalThreadProc` also calls `GetDataDirectoryPath` (`0xf849c3`). The datafile format is in `Docs/Client/client-datafiles.md`.

## 2. Options that touch the engine

Option names and defaults are in [14-ui-options.tsv](14-ui-options.tsv). The rows that matter here:

| Option | Entry address | Default |
| --- | --- | --- |
| `effect-view-distance` | `0x1955e50` | see TSV |
| `character-view-distance` | `0x1955e88` | see TSV |
| `bg-view-distance` | `0x1955ec0` | see TSV |
| `character-view-count` | `0x1955f68` | see TSV |
| `effect-view-spawn` | `0x195ad80` | see TSV |
| `effect-view-spawn-signi` | `0x195c440` | see TSV |
| `foreground-max-framerate` | `0x195aae0` | 60 |
| `background-max-framerate` | `0x195ab18` | 20 |
| `frame-auto-control` | `0x195f1c0` | see TSV |

Apply functions:

| Function | 381 address | Engine slots |
| --- | --- | --- |
| `Ui::ApplyEngineConfiguration` | `0x82bbe0` | EI+0x7f0..0x888 `OptFunc_*` |
| OptionPanel apply | `0xc523f0` | EI+0x7f0..0x888, EI+0x168..0x178 |
| `ApplyTestOptimizeOption` | `0x1120370` | EI+0x168..0x1a4. One value per option level. Uses `Config` floats +21384..+21432. |
| `ApplyLowPerformanceGraphicOption` | `0x111db10` | EI+0x180..0x198 |
| `ApplyFrameAutoControl` | `0x111da20` | — |
| `ApplyOptimizeBattlePresetOption` | `0x111da50` | — |
| `LoadOption` | `0x115e820` | — |
| `PTTakgi::CreateTakgi` | `0xa2a8b0` | — |
| `BPTNpc::updateNPCPortal` | `0xacf7f0` | — |
| `BPTNpc::updatePublicRaidFieldCullDistance` | `0xacf8b0` | EI+0x7e0 `ObjFunc_SetFixedCullDistance`, value 10000000.0 (no cull for raid field NPCs) |

Server notes:
- View distance, view count and cull options change only engine drawing. No option apply path calls `SendToGamed` ❓ (checked in the functions above, not in all option code).
- The server sight range (which objects the server sends) is a server rule. The client option does not change it. To show a far object, the server must send it; the client draws it only if it is in the option distance.
- `IntegerSyncOptionType` (27 values) is possibly the option set that `saveSyncOption` `0x115b6b0` stores on the server ❓. Values: [15-engine-enums-gm.tsv](15-engine-enums-gm.tsv). ❓ The packet for sync options is not traced here.
- Unit conversion (engine = server × 4): see [06 — movement](06-movement.md).

## 3. Sound and video

The exe has no version strings for audio or video libraries. It imports no audio or video DLL (§1.1). Audio and Bink video are in the engine DLL ❓ (library versions must come from `bsengine_Shipping.dll` or the `bin` folder).

| Item | 381 address | Note |
| --- | --- | --- |
| `AudioDevices::Clear` | `0x4499a0` | device list holder |
| `AudioDeviceComboBox::RefreshAudioDeviceList` | `0xbf21d0` | EI+0xbe4.. device list; option `audio-device` `0x19565f8`, default `"DirectSound Default"` |
| `MixerLib::Control` | `0x6002a0`..`0x6006f0` (`SetDetails` `0x6005f0`/`0x6006f0`, `GetDetails` `0x600680`) | Windows `winmm` mixer (WINMM import) |
| `MixerLib::MixerBase` | `0x5e6e80` (ctor), `SetVolume` `0x5e75f0` | system volume |
| `CreateSoundEngineHandle` | `0xf7bff0` | EI+0xc34 `PlaySoundReturnHandle_2` |
| `ApplySoundMuteOption` | `0x1121e20` | option `sound-mute`; `sound-level` `0x1956208` default 100 |
| `SkillComboVideoPanel` | ctor `0x929d30`, `videoLoad` `0x901d00`, `videoPlay` `0x901c90`, `videoPause` `0x901c40`, `videoStop` `0x901c00` | skill video |
| `GuideMovie` / `GuideMoviePanel` | `0xef9130` / `0xf27500` | web video (in-game browser), not Bink |
| `WardrobePanel::pauseSkillSkinBink` | — ❓ | Bink widget control |
| Bink widget names | `CreateCharacterInfomationHolder_Bink` `0x152ac78`, `lobby-play-bink` `0x17ffd5c` | see [00 — inventory](00-inventory.md) |

Sound enums (`EIAudioTrackType`, `EIAudioSoundModeType`, `SoundPresetOptionType`, `Video_Stauts`): [15-engine-enums-gm.tsv](15-engine-enums-gm.tsv).

## Open items (❓)

- Terrain record +24..+30 meaning.
- Server reply to `G_EnterZoneReqCmd` 42 and the sync option packet.
- Library versions (Bink, audio) in `bsengine_Shipping.dll`.
- EI slot gaps 0x38c, 0x5b4, 0x684–0x68c, 0xbfc, 0xc68–0xc74.
- 419 GM IDB did not open: names come from the til only, not from 419 code.
