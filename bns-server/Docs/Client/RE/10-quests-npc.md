# 10 — Quests, NPC talk and zones (Client.exe 0.0.381)

1. The client keeps all quest state in one `BQuestJournal` object (11,664 bytes) at `BPlayer+0xB54`. It holds 94 `QuestObject` slots (120 bytes each). Each slot has a `QuestProperty` sync object (§1.2).
2. Quest packets use sync group 10 (Quest). The client sends `AcquireQuestReq` 1448, `ProgressMissionReq` 1450 and `DisposeQuestReq` 1458. The server answers with `AcquireQuest` 1460, `ChangeQuest` 1461, `CompleteQuest` 1463 or `DisposeQuest` 1464, or with a `…Fail` packet (§1.3).
3. One handler, `BQuestJournal::ProcessNetObjectEvent` `0xa9e5b0`, applies all quest packets. Failure codes 1002–1066 map to UTF-16 message keys `G_PROGRESSMISSIONFAIL_*` (§1.5, `10-quests-errors.tsv`).
4. NPC talk is the UI state `UiStateTalk`. Talk panels (`TalkAcquireQuestPanel`, `TalkProgressMissionPanel`, …) call the `BQuestJournal::Request*` senders. The end of talk sends `TalkSessionCloseReq` 61 (§2).
5. Zone change, teleport, portal, channel, cinematic, raid dungeon and party match use the packets in §3 and §4. The full packet list is `10-quests-commands.tsv`.

```
 NPC click / item / self talk
   v
 BUiWorld::StartNpcTalk 0x7d9980 -> UiStateTalk::StartTalk 0x854550 -> startTalk 0x8543c0
   v  (talk panels)
 TalkAcquireQuestPanel::processButtonEvent 0xc4a6d0 ---> BQuestJournal::RequestAcquireOpenQuest 0xa5c970 --1448-->
 TalkProgressMissionPanel::processButtonEvent 0xc49020 -> BQuestJournal::RequestProgressMission 0xa684a0  --1450-->
 UiStateTalk::EndTalk 0x8443f0 ---------------------------------------------------------- 61 TalkSessionCloseReq -->
                                                      BNSClient::SendToGamed 0x114ce20 ---> GameD
 S2C 1460/1461/1463/1464/1449/1451/... <--------------------------------------------------------+
   v
 World::ProcessNetObjectEvent 0x749550 -> BQuestJournal::ProcessNetObjectEvent 0xa9e5b0
   |-- 1460 setQuest 0xa98d90       |-- 1461 updateQuest 0xa98520  ("Msg.Quest.ProgressMission")
   |-- 1463 completeQuest 0xa9e320  |-- 1464 resetQuest 0xa9d9b0
   v
 UI: QuestJournalPanel 0xa902f0, QuestQuickSlotPanel 0xa9d250, QuestGuideNotificationPanel 0xa8a950,
     UiStateTalk::ProcessNetObjectEvent 0x8530a0, DungeonNotificationPanel 0x10092b0
```

Sources: Client.exe 0.0.381 IDA database (all addresses are 381). Type and member names come from the 381 database types (imported from the 419 GM PDB). Packet ids, sizes and fields come from `game-commands.tsv` and `packet-layouts.tsv` (chapter 05). Datafile schema rows come from `Docs/DataFile/tabledefs-2020.json` (2020 GameDaemon schema, 64-bit offsets, not the 381 client build) — ❓ for exact client offsets.

Related chapters: [05 — game dispatch](05-game-dispatch.md) (framing, `SendToGamed`, sync groups), [06 — movement](06-movement.md) (`MoveListMaker`, portal moves), [08 — creature](08-creature.md) (`T_PROP_SYNC`, NPC objects), [09 — items](09-items.md) (inventory, `InventoryItemListOk` 1332). This chapter does not repeat them.

## Files

| File | Content |
| --- | --- |
| `10-quests-commands.tsv` | 138 commands of this area: id, name, direction, 381 handler, GM handler name, 381 builder, struct size, sync group, fields (offset:name:type/size). |
| `10-quests-errors.tsv` | 40 quest failure codes with UTF-16 message key and key address. |
| `10-quests-tables.tsv` | 2,360 field rows of 80 datafile tables (quest, npc, social, zone*, teleport, dungeon, cinematic, partymatch, raid-dungeon, …). 2020 server schema, 64-bit offsets. |
| `10-quests-enums-gm.tsv` | 307 enums of the area (1,663 rows: enum, member, value). Type names from the 381 database; values are 419 GM. |
| `10-quests-functions.tsv` | 4,023 named functions of the area (quest, talk, zone, dungeon) with class, size, topic and first string. |

## Quick find

| Task | Function | 381 address | GM name | Section |
| --- | --- | --- | --- | --- |
| Where the client applies quest packets | `BQuestJournal::ProcessNetObjectEvent` | `0xa9e5b0` | same | §1.4 |
| Where the client accepts a quest | `BQuestJournal::RequestAcquireOpenQuest` | `0xa5c970` | same | §1.3 |
| Where the client sends mission progress | `BQuestJournal::RequestProgressMission` | `0xa684a0` | same | §1.3 |
| Abandon (dispose) a quest | `BQuestJournal::RequestDisposeOpenQuest` | `0xa5c720` | same | §1.3 |
| Create the quest journal | `BQuestJournal::BQuestJournal` (called from `BPlayer::BPlayer` `0xb8a73c`) | `0xa423f0` | same | §1.1 |
| Add a quest (1460) | `BQuestJournal::setQuest` | `0xa98d90` | same | §1.4 |
| Change a quest (1461) | `BQuestJournal::updateQuest` | `0xa98520` | same | §1.4 |
| Complete a quest (1463) | `BQuestJournal::completeQuest` | `0xa9e320` | same | §1.4 |
| Remove a quest (1464) | `BQuestJournal::resetQuest` | `0xa9d9b0` | same | §1.4 |
| Can the player accept / complete | `BQuestJournal::CanAcquireQuest` / `CanCompleteQuest` | `0xa518b0`, `0xa51d20` / `0xa51530` | same | §1.6 |
| Quest failure message keys | switch in `0xa9e5b0` | `0xa9e9dc` | — | §1.5 |
| Start NPC talk | `BUiWorld::StartNpcTalk` → `UiStateTalk::StartTalk` | `0x7d9980` → `0x854550` | same | §2 |
| End NPC talk (cmd 61) | `UiStateTalk::EndTalk` | `0x8443f0` | same | §2 |

## 1. Quest model

### 1.1 Owner objects

| Object | Where | 381 address | Notes |
| --- | --- | --- | --- |
| `BQuestJournal*` | `BPlayer+0xB54` | ctor `0xa423f0`, stored at `0xb8a74a` in `BPlayer::BPlayer` | Size 11,664 (type `BQuestJournal`). Next to `Inventory` at `BPlayer+0xB50` (chapter 09). |
| quest table | `dword_1A4D5AC` (table manager), virtual slot 23 (`+0x5C`) returns `Data::QuestQuest*` for a quest id | used in `0xa9e5b0` case 1460 and 1470 | Same table manager as other `Data::*` lookups ❓ (slot meaning from use only). |

### 1.2 `BQuestJournal` layout (type `BQuestJournal`, checked in 381 code)

The 381 code agrees with this layout: `setQuest` `0xa98d90` reads `this[2866]` (= `+0x2CC8`, `_requestedToAcquireQuestId`) and indexes slots with a stride of 30 dwords (120 bytes).

| Offset | Member | Type | Meaning |
| --- | --- | --- | --- |
| `0x0000` | `_questList` | `QuestObject[94]` | Open (on-going) quests. One slot per quest. |
| `0x2C10` | `_completeQuestList` | `QuestList` | Completed quest bits (from cmd 90). |
| `0x2C2C` | `_onGoingTutorialQuest` | `vector<OnGoingQuest>` | |
| `0x2C3C` | `_onGoingNormalQuest` | `vector<OnGoingQuest>` | Rebuilt by `UpdateOnGoingQuestCase` `0xa99de0`. |
| `0x2C4C` | `_onAcquiringQuest` | `vector<OnGoingQuest>` | |
| `0x2C5C` | `_delayedProgressTutorialCase` | `vector<enum>` | `AddToDelayedProgressTutorialCase` `0xa27270`. |
| `0x2C70` | `_tutorialCaseTickCheckPeriod` | `PreciseTimer` | |
| `0x2C80` | `_tutorialCaseTickCheckEnabled` | `bool` | |
| `0x2C84` | `_tutorialSkillTargetList` | `vector<u64>` | |
| `0x2C98` | `_journalTickPeriod` | `PreciseTimer` | |
| `0x2CA8` | `_currentEnvResponse` | `i64` | |
| `0x2CB0` | `_currentEnvResponseBGMPlayed` | `bool` | |
| `0x2CB8` | `_currentEnvResponseTimer` | `PreciseTimer` | |
| `0x2CC8` | `_requestedToAcquireQuestId` | `int` | Set before 1448 is sent; cleared in `setQuest` when the id matches. |
| `0x2CD0` | `_talkToSelfAnywhareMapGroupId` | `u64` | |
| `0x2CD8` | `_missedQuestCompleteCaseProcessed` | `bool` | |
| `0x2CDC` | `_requestedDisposeQuestsList` | `vector<short>` | Cleared on 1459 `DisposeQuestFail`. |
| `0x2CEC` | `_currentPortalProcess` | `BQuestJournal::PORTAL_MANIPULATE_*` | §4.2 |
| `0x2CF0` | `_portalDungeonQuestId` | `i16` | |
| `0x2CF8` | `_portalEnvId` | `i64` | |
| `0x2D00` | `_dailyQuestCount` | `int` | `updateDailyWeeklyQuestCount` `0xa22360` |
| `0x2D04` | `_weeklyQuestCount` | `int` | |
| `0x2D08` | `_attractionQuestCount` | `int` | `UpdateAttractionQuestCount` `0xa22310` |
| `0x2D0C` | `_jobQuestCount` | `int` | `UpdateJobQuestCount` `0xa222c0` |
| `0x2D10` | `_currentConvoyFailTalksocialId` | `u64` | |
| `0x2D18` | `_currentConvoyFailTalksocialDelay` | `float` | |
| `0x2D1C` | `_reserveMappingEpicQuestId` | `i16` | |
| `0x2D1E` | `_reserveReplayEpicQuestId` | `i16` | |
| `0x2D20` | `_enteredTencentFatigueTime` | `bool` | |
| `0x2D24` | `_challengeTodayTotalQuestList` | `vector<ChallengeListData>` | |
| `0x2D34` | `_challengeThisWeekTotalQuestList` | `vector<ChallengeListData>` | |
| `0x2D48` | `_challengeWeekStartTime` | `i64` | `SetChallengeWeekStartDateTime` `0xa1e560` |
| `0x2D50` | `_tendencyFieldQuestList` | `vector<TendencyFieldQuest>` | `AddTendencyFieldQuest` `0xa3e150` |
| `0x2D70` | `_mentoringQuestList` | `vector<short>` | |
| `0x2D80` | `_onGoingMentoringQuestId` | `i16` | |
| `0x2D84` | `_dailyRewardIndex` | `int` | |
| `0x2D88` | `_weeklyRewardIndex` | `int` | |

`QuestObject` (120 bytes, one slot):

| Offset | Member | Type |
| --- | --- | --- |
| `0x00` | `_questData` | `Quest*` (440-byte client quest record wrapper) |
| `0x04` | `_questProp` | `QuestProperty` (72 bytes, sync object) |
| `0x50` | `_questMissionStepStartTime` | `PreciseTimer` (reset by `QuestObject::ResetMissionStepTime`) |
| `0x60` | `_lastQuestMissionStep` | `int` |
| `0x64` | `_tutorialSkillSequenceParamList` | `std::map<short,…>` |
| `0x70` | `_partyDungeonStep` | `char` |
| `0x71` | `_timeResetNotification` | `bool` |
| `0x72` | `_tendencyFieldQuestMapUnitEnable` | `bool` |
| `0x74` | `_newbieFlag` | `int` |

`QuestProperty` (72 bytes; the `questProperty` `T_PROP_SYNC` of cmd 1461 and the per-quest body of cmd 87):

| Offset | Field | Type |
| --- | --- | --- |
| `0x00` | `Property` base (vtable) | — |
| `0x04` | `reward_multiplication_factor` | `i8` |
| `0x05` | `current_mission_variation` | `i8` |
| `0x06` | `current_mission_step` | `i8` |
| `0x07 + 4·(n−1)` | `mission_completion_n` (n = 1…16) | `bool` |
| `0x08 + 4·(n−1)` | `mission_achievement_value_n` (n = 1…16) | `i16` |

So a quest has up to 16 missions in the current mission step. Each mission has a done flag and a counter.

### 1.3 C2S quest requests (sync group 10)

All quest requests have `packet_sync` = 10 (`G_QuestSyncGroup_SYNC`). In 381 all sync ids share one lock ([05 §4](05-game-dispatch.md)): the server must answer each request (with an Ok, Fail or a quest packet of group 10), or the client blocks all sync requests for 10 s. Field rows: `10-quests-commands.tsv`.

| Cmd | Name | Size | Sender (381) | Packet ctor (381) | Callers (UI and logic) |
| ---: | --- | ---: | --- | --- | --- |
| 1448 | `AcquireQuestReq` | 64 | `BQuestJournal::RequestAcquireOpenQuest` `0xa5c970`; `BQuestJournal::RequestActionDungeonQuest` `0xa5c8a0` | `0x5116b0` | `TalkAcquireQuestPanel::processButtonEvent` `0xc4a6d0`, `BQuestJournal::ProcessPcSocial` `0xa68fe0`, `BQuestJournal::RequestFieldItemQuest` `0xa5d390`, `BPTNpc::SetQuestApproachMode` `0xb327c0`, `BPlayer::SendProgressQuestAllPacket` `0xb7cfb0`, `ItemPouch::RequestProgressQuestContribution…` `0xd2dbb0`, `0xd261f0`, `PickupItemPanel::requestPickupItemPouchItem` `0xb86560`; dungeon: `MoveListMaker::ReqAcquireQuest` `0xc99a70`, `BQuestJournal::StartDungeonEnterProcessByPortalManipulate` `0xa86100` |
| 1450 | `ProgressMissionReq` | 96 | `BQuestJournal::RequestProgressMission` `0xa684a0` | `0x511790` | 49 callers. Main: `TalkProgressMissionPanel::processButtonEvent` `0xc49020`, `TalkProgressMissionPanel::ProgressComplete` `0xc254c0`, `TalkReactionOnlyMissionPanel::processButtonEvent` `0xc48d50`, `TalkBranchPanel::processBranchEvent` `0xc51ff0`, `QuestMappingPanel::progressAccept` `0xa6f040`, `BQuestJournal::ProcessPcSocial` `0xa68fe0`, `ProcessQuestComplete` `0xa69e10`, `ProcessMissedQuestCompleteOneTime` `0xa69c60`, about 30 `BQuestJournal::ProcessTutorial*` functions `0xa7ce90`–`0xa915d0` |
| 1452 | `ProgressMissionAllReq` | 28 | ctor `0x59b070` ❓ caller | — | — |
| 1454 | `ProgressMissionByAcquireSummonedReq` | 40 | `BQuestJournal::RequestProgressMissionByAcquireSummoned` `0xa68350` | inline | `SummonedBeautyTalkPanel::ProcessEvent` `0x969290`, `JumpingLevelPanel::progressSummoned` `0xd289d0` |
| 1456 | `DungeonProgressToStepReq` | 48 | `BQuestJournal::RequestDungeonProgressToStep` `0xa5c4a0` | `0x511900` | `TalkDungeonProgressPanel::ProgressComplete` `0xc17e00` |
| 1458 | `DisposeQuestReq` | 28 | `BQuestJournal::RequestDisposeOpenQuest` `0xa5c720` | inline | `BUiWorld::DisposeQuestListReq` `0x787d50`, `ClearRetiredDungeonQuest` `0xa5d250`, `isBindByQuestId` `0xa673f0`, `StartDungeonEnterProcessByPortalManipulate` `0xa86100`, `MoveListMaker::ReqTurnBackFromPortalAndDispose…` `0xc99ad0`, `MoveListMaker::playerTick` `0xcc82b0`, `0xc99070` |
| 1465 | `MissionStepRollbackReq` | 20 | ❓ no sender found by an immediate scan | — | — |
| 1467 | `MissionStepTimeExpiredReq` | 20 | `BQuestJournal::RequestMissionStepTimeExpired` `0xa5d1f0` | inline | `QuestObject::UpdateMissionStepTime` `0xa98eb0` (mission step timer) |
| 1469 | `ResetQuestCompletionTimeReq` | 32 | `BQuestJournal::RequestResetQuest` `0xa5c0b0` | inline | `0xfc7d30` |
| 1472 | `AcquireRenewalQuestReq` | 16 | ❓ no sender found by an immediate scan | — | — |
| 1475 | `ChallengeListAcquireQuestReq` | 20 | `BQuestJournal::RequestChallengeListQuest` `0xa5bb70` | inline | `ChallengeTodayPanel::requestAcquireQuest` `0x11df400` |
| 1478 | `ProgressSkipMissionStepReq` | 20 | `BQuestJournal::RequestProgressSkipStep` `0xa5bf60`; `RequestProgressRetiredStep` `0xa5c000` | inline | `SkillTrainingFieldJumpConfirmPanel::onYesClick` `0x9702a0`; `UiStateGame::ProgressComplete` `0x84d950`, `isBindByQuestId` `0xa673f0` |
| 1420 | `ChangeToReplayQuestReq` (sync 9) | 20 | `BQuestJournal::RequestEpicQuestReplay` `0xa5bda0` | inline | — |
| 1423 | `ChallengeListTakeRewardReq` (sync 9) | 20 | `BQuestJournal::RequestChallengeListTalkReward` `0xa5bad0` | inline | `ChallengeTodayPanel::requestTakeReward` `0x11df480` |
| 1130 | `ChallengeListsReq` | 16 | `ChallengeListManager::RequestChallengeLists` `0x11d2f20` | — | `UiStateGame::ProcessEnterWorld` `0x83c3d0`, `UiStateGame::OnDailyQuestReset` `0x85ce60`, `BPTPlayer::OnChangedCreatureProperty` `0xb36270` |
| 89 | `QuestCompleteListReq` | 16 | `0x59f810` ❓ caller | — | — |

"inline" = the sender writes `mov word [pkt+4], cmd` itself (found by an immediate scan of the function).

`AcquireQuestReq` (1448) fields as `RequestAcquireOpenQuest` `0xa5c970` fills them:

| Off (struct) | Field | Type | Value set by the client |
| ---: | --- | --- | --- |
| 16 | `questId` | i16 | argument |
| 18 | `caseIndex` | i8 | argument (the quest `acquisition` case index) |
| 19 | `optionalRewardIndex` | i8 | 0 |
| 24 | `acquireFrom` | i64 | object id (NPC, item or self) that gives the quest |
| 32 | `groceryBagPos` | i16 | 0 or the bag position of the grocery item |
| 36 | `brandItems` | repeat of {`depot` i8, `pos` i16} | items from `Inventory::FindItemListByBrandFromDepot` |
| 48 | `requiredItems` | repeat of {`depot` i8, `pos` i16, `count` i16} | items from `Inventory::FindItemFromDepot` / `GetItemPosition` for the `required-item` fields of the case |

Before it sends, the function calls `BQuestJournal::CanAcquireQuest` and returns false when the check fails. It calls `deleteCompletionTimeGarbage` first.

`ProgressMissionReq` (1450) fields as `RequestProgressMission` `0xa684a0` fills them: `questId` @16, `missionId` @18, `caseIndex` @19, `optionalRewardIndex` @20, `progressFrom` i64 @24, `groceryBagPos` @32, `brandItems` @36, `equipGemItems` @48 (from `Inventory::GetGemSetItemListForQuest`), `requiredItems` @60, `dungeonAdditionalRewardIndex` i8 @72, `dungeonAdditionalRewardItemDataKey` i64 @80, `dungeonAdditionalRewardCostGroupKey` i64 @88. The function first gets the `QuestObject`. It returns when `QuestObject::IsCompleteMission` is true or `QuestObject::CanProgressMission` is false.

### 1.4 S2C quest packets and `BQuestJournal::ProcessNetObjectEvent` `0xa9e5b0`

Route: `World::ProcessNetObjectEvent` `0x749550` → `BQuestJournal::ProcessNetObjectEvent` `0xa9e5b0` (5,364 bytes). The main switch is at `0xa9e9a8` (cases 1332–1470). Other commands use `if` tests.

| Cmd | Name | Size | Fields | Client action (381) |
| ---: | --- | ---: | --- | --- |
| 87 | `QuestListOk` | — | `quest` repeat {`questId` i16, `openQuestList` PROP_SYNC} | `QuestObject::Initialize` on all 94 slots, then `setQuest(id, prop, 0, 0, 0)` for each element. Then `IsQuestExist` checks, `DebugUpdateEpicQuestQuickSlot`, the 4 count updates, `UpdateTendencyFieldQuestList`, `GetQuestListByDungeonType`, `UpdateAndSendOnGoingMentoringQuestId`. |
| 90 | `QuestCompleteListOk` | — | `questCompleteList` PROP_SYNC | `sub_A26E00` (store the completed bits). Then it clears each of 36 quest quick-slot ids that `IsCompleteQuest` reports as complete. Then the count updates. |
| 192 | `PartyDungeonStep` | 20 | `dungeonId` i16 @16, `dungeonStep` i8 @18, `reason` i8 @19 | `getQuestObject(id = i16 @16)`; sets `QuestObject._partyDungeonStep` = byte @18. |
| 310 | `ConvoyNotifyFail` | 20 | i16 @16 | `BQuestJournal::playConvoyFailTalksocial`. |
| 818 | `TencentAASNotify` | 24 | `notifyType` i8 @16, `gameTime` i32 @20 | When `notifyType` = 0 and `gameTime` ≥ 3600 × `Config+5828`, it sets `_enteredTencentFatigueTime` and shows a game message. |
| 1016 | `ClearQuestPackNotify` | 20 | `quest_category` i8 @16, `reason` i8 @17 | `ClearTendencyFieldQuestList(category)` `0xa9e280`. |
| 1260 | `EnvEnteredNotify` | 24 | `envId` i64 @16 | env response (BGM and talk) ❓ detail. |
| 1332 | `InventoryItemListOk` | — | chapter 09 | tutorial item cases (`ProcessTutorialTransformItem`, …). |
| 1421 / 1422 | `ChangeToReplayQuestOk` / `Fail` | 16 / 20 | — | game message. |
| 1424 | `ChallengeListTakeRewardOk` | 20 | `challengeSubType` i8 @16 | `SetChallengeTodayTakeRewarded` `0xa2e830` or `SetChallengeThisWeekTakeRewarded` `0xa4f0a0`. |
| 1449 | `AcquireQuestFail` | 20 | error i16 @16 | Code 979: message when the requested quest exists and `IsRetiredQuest` `0xa21cc0` is false. Code 991: other message ❓ keys. |
| 1451 / 1453 / 1455 | `ProgressMissionFail` / `ProgressMissionAllFail` / `…ByAcquireSummonedFail` | 20 / 24 / 20 | error i16 @16 (1453 adds `questId` @18, `missionId` @20, `caseIndex` @21) | error switch `0xa9e9dc` → message key (§1.5). |
| 1459 | `DisposeQuestFail` | 20 | error i16 @16 | clears `_requestedDisposeQuestsList`. |
| 1460 | `AcquireQuest` | 24 | `questId` i16 @16, `missionStepId` i8 @18, `questCompletionCount` i8 @19, `missionVariationIndex` i8 @20, `rewardMultiplicationFactor` i8 @21 | `setQuest` `0xa98d90` (fill a free slot, reset the step timer, store the 3 bytes, set the completion time, `BPTPlayer::ReserveTalkSocial`); `UpdateOnGoingQuestCase` `0xa99de0`, count updates, `AddTendencyFieldQuest`, `UpdateAndSendOnGoingMentoringQuestId`, `ProcessDungeonEnterByPortalManipulate` `0xa87110`; `BPTPlayer::PlayQuestNavigation`; reward preview with `BQuestRewardInfo::UpdateCompleteQuestBasicReward` and `TencentExpBoost::ComputeAcquirableExp`. |
| 1461 | `ChangeQuest` | 96 | `questId` i16 @16, `missionId` i8 @18, `caseIndex` i8 @19, `newbieState` i8 @20, `questProperty` PROP_SYNC @24 | `resetTeleportCooltime` `0xa3e550`; `updateQuest` `0xa98520`: apply the property, reset the step timer, then show `Msg.Quest.ProgressMission`, `Msg.Quest.CompleteMission`, `Msg.Quest.Dungeon.ProgressMission` or `Msg.Quest.Dungeon.CompleteMission`; mission step reaction talk social; same updates as 1460. |
| 1462 | `UnchangedQuest` | 20 | `questId`, `missionId`, `caseIndex` | not in the switch of `0xa9e5b0` ❓ (panels only). |
| 1463 | `CompleteQuest` | 20 | `questId` i16 @16, `missionId` i8 @18, `caseIndex` i8 @19 | `resetTeleportCooltime`; `completeQuest` `0xa9e320` (reward message, completion talk social, completion time or bit, `resetQuest`); `RemoveQuickSlotItem` (`0xa98020` or `0xa9ac60` ❓ which); `notifySkillTrainingTriggerQuestComplete` `0xa2fa60`; updates; `ProcessQuestComplete` `0xa69e10`; mentoring update when the id is the last mentoring quest. |
| 1464 | `DisposeQuest` | 20 | `questId` i16 @16 | `resetQuest` `0xa9d9b0` (clear the `QuestProperty`), `RemoveQuickSlotItem`, updates, `ProcessDungeonEnterByPortalManipulate`. |
| 1470 | `ResetQuestCompletionTimeOk` | 20 | i16 @16 (quest id) | quest table lookup, `deleteCompletionDay` `0xa26870`. |

Other S2C quest handlers (panels, not the journal): `UiStateTalk::ProcessNetObjectEvent` `0x8530a0` (1449, 1455, 1460), `QuestJournalPanel::ProcessNetObjectEvent` `0xa902f0` (87, 90, 1460, 1461), `QuestGuideNotificationPanel::ProcessNetObjectEvent` `0xa8a950` (87, 90, 1460, 1461), `QuestMappingPanel::ProcessNetObjectEvent` `0xa1dcb0` (1463), `TendencyFieldQuestQuickSlotPanel::ProcessNetObjectEvent` `0x8c79a0` (87, 1460), `DungeonNotificationPanel::ProcessNetObjectEvent` `0x10092b0` (1460, 1461), `GuideMovie::ProcessNetObjectEvent` `0xf76ae0` (1460), `BQuestRewardInfo::updateOptionalReward` `0xa0e6c0` (1460).

### 1.5 Quest failure codes

The error switch at `0xa9e9dc` (in `0xa9e5b0`) maps the error code at packet offset 16 of 1451, 1453 and 1455 to a UTF-16 message key. All 40 rows are in `10-quests-errors.tsv`. Codes 1013, 1021–1024, 1037–1055 and 1060 use the default branch. Examples:

| Code | Key (`G_PROGRESSMISSIONFAIL_` + …) | Key address |
| ---: | --- | --- |
| 1002 | `INVALID_QUEST_ID` | `0x162a7b8` |
| 1008 | `NOT_OPEN_QUEST` | `0x162a9d8` |
| 1009 | `MISSION_ALREADY_COMPLETED` | `0x162aa28` |
| 1010 | `NPC_IS_TOO_FAR` | `0x162aa88` |
| 1016 | `INVALID_ZONE` | `0x162ac00` |
| 1026 | `INVENTORY_FULL` | `0x162ad98` |
| 1035 | `TIME_LIMIT_EXPIRED` | `0x162b0e0` |
| 1056 | `INVALID_DUNGEON_ADDITIONAL_REWARD_REQUEST` | `0x162b4d0` |

### 1.6 Worked example — `AcquireQuestReq` (1448, C2S)

The values are an example (quest 100, acquisition case 0, NPC object id `0x0000000200001234`, no items). The wire has no padding ([05 §3.2](05-game-dispatch.md)).

```
16 00                     size = 22 (header included)
A8 05                     command = 1448 G_AcquireQuestReqCmd
64 00                     questId = 100
00                        caseIndex = 0
00                        optionalRewardIndex = 0
34 12 00 00 02 00 00 00   acquireFrom = 0x0000000200001234
00 00                     groceryBagPos = 0
00 00                     brandItems: 0 elements
00 00                     requiredItems: 0 elements
```

A good server answer is `AcquireQuest` (1460): `questId` 100, `missionStepId`, `questCompletionCount`, `missionVariationIndex`, `rewardMultiplicationFactor`. Then the quest property comes with `ChangeQuest` (1461) when a mission counter changes. Server side: [GameDaemon 20 — quest handlers](../../Server/GameDaemon/20-quest-handlers.md) and [21 — quest progress](../../Server/GameDaemon/21-quest-progress-handlers.md).

### 1.7 Quest datafile tables

The client reads the `quest` table (type id 174) through the table manager `dword_1A4D5AC` (§1.1). The 2020 server schema gives 25 elements: `quest`, `filter`, `reaction`, `filter-set`, `reaction-set`, `acquisition-loss`, `acquisition`, `mission-step`, `case`, `tutorial-case`, `completion-loss`, `giveup-loss`, `basic-reward`, `fixed-reward`, `optional-reward`, `mission`, `mission-step-success`, `mission-step-fail`, `completion`, `next-quest`, `transit`, `destination`, `complete`, `not-acquire`. All fields: `10-quests-tables.tsv`.

`case` subtypes (server schema). The 419 GM enum `Data::QuestCaseSubType` has the same order for 0–24 and adds 25 `ATTRACTION_POPUP`: 0 talk, 1 talk-to-item, 2 talk-to-self, 3 manipulate, 4 npc-manipulate, 5 approach, 6 skill, 7 loot, 8 killed, 9 finish-blow, 10 env-entered, 11 enter-zone, 12 convoy-arrived, 13 convoy-failed, 14 npc-bleeding-occured, 15 enter-portal, 16 acquire-summoned, 17 pc-social, 18 join-faction, 19 duel-finish, 20 party-battle, 21 party-battle-action, 22 complete-quest, 23 pick-up-fielditem, 24 battle-royal.

`tutorial-case` subtypes (server schema): 0 acquire-item … 34 teleport-zone (35 values, in the TSV). Each has a client function `BQuestJournal::ProcessTutorial<Name>` (for example `ProcessTutorialTeleportZone` `0xa850b0`, `ProcessTutorialAirdash` `0xa852f0`) that calls `RequestProgressMission` `0xa684a0`.

Other tables of this chapter in the TSV: `questreward` 175, `questrewardskill3` 176, `skipquest` 220, `skipteleport` 221, `jumpingquest` / `jumpingquest2` 144 / 143, `tencent-quest` 241, `challengelist` 36, `challengelistreward` 37, `attraction-group` 8, `attraction-reward` 9, `contents-reset` 47, `filter` 94, `npc` 161, `social` 225, `cinematic` 39, `teleport` 240, `zone*` 252–277, `dungeon` 77, `raid-dungeon` 177, `interdungeonlist` 118, `cave` / `cave2` 35 / 34, `partymatch` 165, `district` 62, `faction` 88.

## 2. NPC talk, cinematics and zone change

### 2.1 Talk entry points

| Function | 381 address | Talk type (case in `StartTalk`) |
| --- | --- | --- |
| `BUiWorld::StartNpcTalk` | `0x7d9980` | 1 NPC |
| `BUiWorld::StartItemTalk` | `0x7d9960` | 2 item |
| `BUiWorld::StartTalkToSelf` | `0x7d98f0` | 4 self |
| `BUiWorld::StartPlayerSummonedTalk` | `0x7d97f0` | 5 summoned |
| `BUiWorld::RestartTalk` | `0x7cd920` | calls `UiStateTalk::RestartTalk` `0x844920` |

Case 3 exists in the switch of `UiStateTalk::StartTalk` `0x854550`; its caller is not identified ❓. The 419 GM enum `UiStateTalk::TARGET_TYPE` gives ITEM=2, SELF=3, SELF_PROGRESS_MISSION=4, DUNGEON_ADDITIONAL_REWARD=6. These values do not agree with the 381 cases 4 and 5 above. The mapping is ❓.

### 2.2 `UiStateTalk` state

| Function | 381 address | Behaviour |
| --- | --- | --- |
| `UiStateTalk::StartTalk` | `0x854550` | Rejects the talk if the delay at `+0x15A18` (float) is larger than the `PreciseTimer` elapsed time. Then switches on the talk type and shows a fail message if the player cannot talk. |
| `UiStateTalk::startTalk` | `0x8543c0` | Stores the target and opens the talk panels. |
| `UiStateTalk::StartState` | `0x8296e0` | Sends cmd 131 `RetrieveFactionStatisticsReq` (builder `0x7336e0`). |
| `UiStateTalk::LoadUI` | `0x7e0aa0` | Loads the talk panels. |
| `UiStateTalk::EndTalk(reason)` | `0x8443f0` | See §2.3. |
| `UiStateTalk::RestartTalk` | `0x844920` | Restarts the talk with the same target. |
| `UiStateTalk::ProcessEvent` | `0x858760` | UI events. |
| `UiStateTalk::ProcessNetObjectEvent` | `0x8530a0` | See §2.4. |
| `UiStateTalk::EndState` | `0x85ca70` | Leaves the state. |
| `lockCamera` / `unLockCamera` | `0x7ffc60` / `0x7ffc30` | Camera lock during the talk. |
| `PlayDialogSound` | `0x7f8ea0` | Dialog voice. |
| `SetTalkMessage` | `0x854380` | Sets the balloon text. |
| `IsTalkStartDelay` | `0x7e7500` | Delay check. |
| `ShowAcquireQuestFunction` | `0x836570` | Opens `TalkAcquireQuestPanel`. |
| `ShowProgressMissionFunction` | `0x836500` | Opens `TalkProgressMissionPanel`. |
| `ShowDungeonProgressPanel` | `0x800150` | Opens `TalkDungeonProgressPanel`. |

Object fields (381 code):

| Offset | Type | Meaning |
| --- | --- | --- |
| `+0x159E0` (88544) | i32 | target type |
| `+0x159E8` (88552) | i64 | target object id (NPC id for cmd 61) |
| `+0x159F0` (88560) | handle | engine handle of the target |
| `+0x15A18` (88600) | float | talk start delay |

Fail message keys in `StartTalk` `0x854550`:

| Case | Keys |
| --- | --- |
| 1 NPC | `Msg.Npc.TalkFail.OtherAction` |
| 2 item | `Msg.Item.TalkFail.OtherAction` |
| 4 self | `Msg.Player.TalkToSelfFail.Restoration`, `.Exhausition`, `.Dead` |
| 5 summoned | `Msg.Player.TalkToSummoned.Restoration`, `.Exhausition`, `.Dead` |

Talk panel classes (functions in `10-quests-functions.tsv`, topic `talk`): `TalkAcquireQuestPanel`, `TalkBalloonPanel`, `TalkBranchPanel`, `TalkContributeGuildReputationPanel`, `TalkControlPanel`, `TalkDungeonProgressPanel`, `TalkExchangeFactionCoinPanel`, `TalkFactionCoinPanel`, `TalkNotProgressMissionPanel`, `TalkProgressMissionPanel`, `TalkReactionOnlyMissionPanel`, `TalkSocialListPanel`, `TalkSocialQuickSlotPanel`, `TalkStatePanel`.

### 2.3 End of talk and cmd 61

`UiStateTalk::EndTalk(reason)` `0x8443f0`:

1. If the target type is 1 (NPC) and `reason` is not 5, build cmd 61 `TalkSessionCloseReq` (24 B: `npc` i64 @16 = value at `+0x159E8`) and send it.
2. Release the target: `ReleaseCreature` or `ReleaseObjectEngineHandle`.
3. `PresentationWorld`: set balloon, indicator and name plate on again.
4. `TalkBalloonPanel::EndTalk`, `TalkControlPanel::EndTalk`.
5. `BUiWorld::ActivateGame`, `UiStateGame::OpenSavedFloatingPanels`, `UiStateGame::ProcessEndTalk`.

`END_TALK_REASON` (419 GM enum; values seen at 381 call sites in the right column):

| Value | Name | 381 caller |
| --- | --- | --- |
| 0 | TALK_END | normal end ❓ |
| 3 | — | `ProcessActionSubEvent` |
| 5 | CLOSE_TALK_SESSION | BUiWorld net handler `0x7d9a60`, cmd 62 `TalkSessionClosedNotify` (server closed the session, so no cmd 61) |
| 6 | LEAVE_ZONE | `0x7dadb0` (leave zone), `0x8449c3` |
| 0xB | — | `TalkBranchPanel::processBranchEvent` |
| 0xD | FAILED_TO_PLAY_CINEMATIC ❓ (GM says 13) | `TalkBalloonPanel::SetNextPage` `0xc49ffe` |
| 14 | DUNGEON_ADDITIONAL_REWARD | — |

Other 419 GM enums of the talk (values in `10-quests-enums-gm.tsv`): `PLAY_END_TALK_SHOW` 0–4, `QuestStateType`, `QuestIndicatorType` 0–17, `NpcQuestIndicatorType` 0–6.

Talk packets:

| Cmd | Name | Dir | Fields |
| --- | --- | --- | --- |
| 61 | `TalkSessionCloseReq` | C2S | `npc` i64 @16 |
| 62 | `TalkSessionClosedNotify` | S2C | `npc` i64 @16 → `EndTalk(5)` |
| 63 | `NpcDetect` | S2C | `npc` i64 @16 (no action in `0x7d9a60`) |

### 2.4 `UiStateTalk::ProcessNetObjectEvent` `0x8530a0`

| Cmds | Topic |
| --- | --- |
| 1329–1340 | item packets during the talk |
| 1455, 1460, 1461, 1463, 1464 | quest result and journal change (refresh the talk panels) |
| 432, 433, 441 | create guild |
| 540, 541 | guild uniform |
| 631–635 | change summoned (`Msg.ChangeSummonedName.Success`) |

### 2.5 Talk flow

```
click NPC ─► BUiWorld::StartNpcTalk 0x7d9980
              └► UiStateTalk::StartTalk 0x854550 (delay + state check)
                   └► startTalk 0x8543c0 ─► StartState 0x8296e0 ─► cmd 131
player picks quest ─► TalkAcquireQuestPanel ─► cmd 1448 (§1.3)
mission talk       ─► TalkProgressMissionPanel ─► cmd 1450
server 1460/1461/1463 ─► 0x8530a0 refresh
close ─► EndTalk(0) 0x8443f0 ─► cmd 61
server 62 ─► 0x7d9a60 ─► EndTalk(5) (no cmd 61)
```
