# 17 — Client.exe 381: coverage audit and open items

1. This chapter audits chapters 00–18 of the 381 client reference. It measures which classes have no chapter, and it collects all open items (❓) in one list.
2. Coverage: 320 classes have 20 or more functions ([`classes.tsv`](classes.tsv)). The text of chapters 00–16 names 165 of them (14,104 functions). 155 large classes (5,591 functions) are only in the TSV files. §2.3 gives a short entry for each.
3. Open items: 509 rows in [`17-open-items.tsv`](17-open-items.tsv) (every ❓ line and every row of each "Open items" section), each with a next action (§3).
4. Chapter 18 (GM debugger, 419 GM) does not exist: slice cx-18 was blocked, because the 419 GM `.i64` did not open (§1). Chapter 13 is a link page to the Datafile book ([13-datafile](13-datafile.md)).
5. Facts of the old notes that no chapter holds are in §4. The old file [`../client-exe-reversing.md`](../client-exe-reversing.md) is now the short overview. The A–Z index is [INDEX.md](INDEX.md).

```
 classes.tsv (3,650 classes) ──► filter functions >= 20 ──► 320 large classes
                                      │
              chapter text 00–16 ◄────┤ whole-word match of the class name
                 165 named            │
                                      ▼
                 155 not named ──► §2.3 short entries (role, 381 ctor, key methods)
                                      │
 every "❓" line + every "Open items" row in 00–16 ──► 17-open-items.tsv (509) ──► §3
 quick-find tables of 00–16 (352 rows) ──► 17-quick-find.tsv ──► INDEX.md (A–Z)
```

Source: the 381 database `ida/lanes/cx/Client.exe.i64` (names matched from the 419 GM PDB, see [00 §3](00-inventory.md)) and the chapter files of this folder on 2026-11-10. The 419 GM database was not opened in this slice. ❓ marks a claim that we did not verify.

## Quick find

The full table (352 rows, all chapters) is [`17-quick-find.tsv`](17-quick-find.tsv). Columns: `address`, `chapter`, `section`, `task`, `function`, `gm_name`, `ida_name_381`, `check`. This table gives the most-used rows.

| Task or question | Function | 381 address | GM name (419 GM) | Section |
| --- | --- | --- | --- | --- |
| Where the game starts | `WinMain` | `0x1190f60` | same | [01 §3.1](01-startup.md) |
| Where the client starts (XML reader DLL load) | `BNSClient::Initialize` | `0x116f550` | same | [00 §1.5](00-inventory.md), [01](01-startup.md) |
| Where the command line is parsed | `BNSClient::DoCommandLine` | `0x113ed60` | same | [01 §1](01-startup.md) |
| Where the engine DLL loads | `LoadEngine` | `0x114c6d0` | same | [15 §1.2](15-engine.md) |
| Where the STS client DLL loads | `NPStsMsgLib::Create` | `0xbc90a0` | same | [03 §1](03-sts-client.md) |
| Envelope length on the wire | `PPUtil::PacketSize` | `0x60f8a0` | same | [02 §2.1](02-transport.md) |
| Plain packet length | `Session::PacketSize` | `0x5d4a20` | `PacketSession::PacketSize` | [02 §2.2](02-transport.md) |
| Packet encryption | `GamePacketProcessor::Encrypt` | `0x608f10` | same | [02 §3](02-transport.md) |
| LobbyGate session object | `LobbyGateSession::LobbyGateSession` | `0xc673e0` | same | [04 §1](04-lobbygate.md) |
| LobbyGate main receive switch | `LobbyGateSession::ProcessLobbyNetObjectEvent` | `0xcb86d0` | same | [04 §2.3](04-lobbygate.md) |
| GameD session object | `GameSession::GameSession` | `0xefbf40` | same | [05 §1](05-game-dispatch.md) |
| Where the client sends a game packet | `BNSClient::SendToGamed` | `0x114ce20` | same | [05 §1.4](05-game-dispatch.md) |
| Where most game packets arrive | `World::ProcessNetObjectEvent` | `0x749550` | same | [05 §2](05-game-dispatch.md) |
| Sync group lock (one pending request, 10 s) | `TaskSession::Sent` | `0x86ed90` | same | [05 §4](05-game-dispatch.md) |
| Where the client sends a move | `MoveListMaker::sendWalkPacket` | `0xcc4ce0` | same | [06 §2.1](06-movement.md) |
| Per-tick move recorder | `MoveListMaker::playerTick` | `0xcc82b0` | same | [06 §2.2](06-movement.md) |
| Where the client starts a skill | `BControlledPlayingActions::UseSkill` | `0x1114f50` | same | [07 §1.1](07-combat.md) |
| Create a creature (spawn) | `World::enterCreature` | `0x74b640` | same | [08 §2.2](08-creature.md) |
| Find a world object by id | `World::find` | `0x72b670` | same | [08 §1.3](08-creature.md) |
| Move an inventory item | `Inventory::RequestMoveItemPosition` | `0xe91e00` | same | [09 §2.2](09-items.md) |
| Where the client applies quest packets | `BQuestJournal::ProcessNetObjectEvent` | `0xa9e5b0` | same | [10 §1.4](10-quests-npc.md) |
| Where the client applies party packets | `BParty::ProcessNetObjectEvent` | `0xc4bac0` | same | [11 §1.3](11-social.md) |
| Where the client sends an InfoGate packet | `BNSClient::SendToInfoGate` | `0x114cdb0` | same | [12 §1.4](12-infogate-arena.md) |
| UI start | `Ui::Initialize` | `0x8135e0` | same | [14 §1.3](14-ui.md) |
| Zone load state machine | `UiTransitControl::PostTick` | `0x7d9680` | `Ui::TransitState` | [15 §1.5](15-engine.md) |
| GameD disconnect reason → quit panel | `GameSession::OnError` | `0xf44150` | same | [16 §1.4](16-strings-errors.md) |
| Datafile load | `Data::DataManager::Load` | `0x5bc500` | same | [13](13-datafile.md) |
| Fishing state change | `PTFishing::ChangeFishingState` | `0xad24e0` | same | §4.6 |
| Large class with no chapter | — | — | — | §2.3, [`17-class-coverage.tsv`](17-class-coverage.tsv) |
| Open item and its next action | — | — | — | §3, [`17-open-items.tsv`](17-open-items.tsv) |

## Files

| File | Rows | Columns | Content |
| --- | ---: | --- | --- |
| [`17-class-coverage.tsv`](17-class-coverage.tsv) | 320 | `class`, `functions`, `area`, `chapter_md`, `chapter_tsv`, `vtable`, `ctor`, `first`, `last`, `key_methods`, `strings`, `cx17_entry` | Every class with 20 or more functions and the chapters that name it. `chapter_md` = chapter text (00–16) that names the class; `chapter_tsv` = chapter TSV files that list it; `cx17_entry` = where this slice added an entry (`17` = §2.3, `13` = [13-datafile](13-datafile.md)). `key_methods` = up to 5 methods named `Send*`, `Req*`, `Request*`, `Process*`, `Handle*`, `Recv*`, `Notify*`, `Set*`, `Update*`, `Open*`, `Close*`, `Show*` with the 381 address. |
| [`17-open-items.tsv`](17-open-items.tsv) | 509 | `id`, `chapter`, `line`, `section`, `kind`, `body`, `next_action`, `address` | All open items of chapters 00–16 (§3). |
| [`17-quick-find.tsv`](17-quick-find.tsv) | 352 | `address`, `chapter`, `section`, `task`, `function`, `gm_name`, `ida_name_381`, `check` | All quick-find rows of chapters 00–16, with the 381 IDA name at the address (§2.4). |

---

## 1. Chapter map (00–18)

| # | Chapter | Status | Large classes named | Open items |
| --- | --- | --- | ---: | ---: |
| 00 | [00-inventory](00-inventory.md): binary map, `classes.tsv`, `functions-1..5.tsv`, name-transfer method | done (cx-00) | — | 29 |
| 01 | [01-startup](01-startup.md): command line, config, main loop | done | 2 / 2 | 40 |
| 02 | [02-transport](02-transport.md): sockets, framing, session crypto | done | 3 / 3 | 32 |
| 03 | [03-sts-client](03-sts-client.md): STS login | done | 1 / 1 | 32 |
| 04 | [04-lobbygate](04-lobbygate.md): LobbyGate session | done | 11 / 15 | 27 |
| 05 | [05-game-dispatch](05-game-dispatch.md): GameSession, dispatch, StructPacket, sync groups | done | (area shared with 07–12) | 33 |
| 06 | [06-movement](06-movement.md): movement and position sync | done | 3 / 4 | 45 |
| 07 | [07-combat](07-combat.md): skills and combat | done | 12 / 32 | 30 |
| 08 | [08-creature](08-creature.md): player, NPC, world objects, summoned cat | done | 7 / 17 | 48 |
| 09 | [09-items](09-items.md): items, inventory, equipment | done | 28 / 60 | 25 |
| 10 | [10-quests-npc](10-quests-npc.md): quests, NPC talk, zones | done | 23 / 44 | 13 |
| 11 | [11-social](11-social.md): party, guild, chat, mail, market | done | 21 / 48 | 30 |
| 12 | [12-infogate-arena](12-infogate-arena.md): InfoGate, ArenaD, other servers | done | 6 / 11 | 49 |
| 13 | [13-datafile](13-datafile.md): link page to the [Datafile book](../../DataFile/book/00-index.md) | link page (this slice) | 3 / 5 (+2 by the link page) | see book |
| 14 | [14-ui](14-ui.md): UI framework, system messages, commands | done | 15 / 41 | 31 |
| 15 | [15-engine](15-engine.md): engine, map load, options, sound | done | (no area tag) | 16 |
| 16 | [16-strings-errors](16-strings-errors.md): error codes, logs, web calls, string anchors | done | 1 / 1 | 29 |
| 17 | this chapter | done | — | — |
| 18 | GM debugger and builder UI (419 GM) | **not written**. Handoff `Docs/Plan/handoffs/cx-18.md`: the 419 GM `.i64` stopped with "internal error 1228". | — | see §3.3 |

"Large classes named" = classes of that `area` (in [`classes.tsv`](classes.tsv)) with 20+ functions that the chapter text names / all large classes of that area. Area `other` has 34 large classes (27 named by some chapter); area `lib-stl` has 2.

---

## 2. Coverage

### 2.1 Method

1. Read [`classes.tsv`](classes.tsv) (3,650 classes, cx-00).
2. For each class, search the class name as a whole word (no letter, digit or `_` before or after it) in each chapter text `00-*.md` … `16-*.md` and in each chapter TSV (`NN-*.tsv`). `functions-*.tsv` and `classes.tsv` do not count.
3. Status: `md` = a chapter text names it; `tsv` = only a chapter TSV lists it; `area` = only the `area` tag of `classes.tsv`; `none` = no chapter at all.

The script runs in the IDA worker (Python) on the 381 database. It also reads the `functions-*.tsv` rows of each class for the key methods, and the strings that the class functions refer to (`idautils.Strings`, `DataRefsFrom`).

### 2.2 Results

All classes:

| Status | Classes | Functions |
| --- | ---: | ---: |
| `md` (chapter text) | 405 | 15,782 |
| `tsv` (chapter TSV only) | 1,911 | 14,140 |
| `area` (tag only) | 408 | 695 |
| `none` (mostly STL templates and small helpers) | 924 | 3,535 |
| name shorter than 3 characters (not searched) | 2 | 36 |

Classes with 20 or more functions (320; 19,695 functions; list in [`17-class-coverage.tsv`](17-class-coverage.tsv)):

| Area (`classes.tsv`) | Large classes | Functions | Named in text | Not named |
| --- | ---: | ---: | ---: | ---: |
| `09-items` | 60 | 3,794 | 28 | 32 |
| `11-social` | 48 | 2,191 | 21 | 27 |
| `10-quests-npc` | 44 | 1,953 | 23 | 21 |
| `14-ui` | 41 | 2,440 | 15 | 26 |
| `other` | 34 | 2,942 | 27 | 7 |
| `07-combat` | 32 | 1,100 | 12 | 20 |
| `08-creature` | 17 | 1,166 | 7 | 10 |
| `04-lobbygate` | 15 | 727 | 11 | 4 |
| `12-infogate-arena` | 11 | 521 | 6 | 5 |
| `13-datafile` | 5 | 145 | 3 | 2 |
| `06-movement` | 4 | 117 | 3 | 1 |
| `02-transport` | 3 | 88 | 3 | 0 |
| `01-startup` | 2 | 110 | 2 | 0 |
| `lib-stl` | 2 | 2,344 | 2 | 0 |
| `03-sts` | 1 | 27 | 1 | 0 |
| `16-strings-errors` | 1 | 30 | 1 | 0 |

Only 1 large class has no chapter at all (text or TSV): `UBlackboardComponent` (25 functions, constructor `0x718440`, first function `0x436230`).
Most of the 155 not-named classes are UI panels and widgets. Chapter 14 describes their common frame (panel vtable, `ProcessNetObjectEvent`, message aliases); each chapter TSV lists their functions. A panel that has a `ProcessNetObjectEvent` method gets game packets from the World fan-out ([05 §2](05-game-dispatch.md)).

### 2.3 Short entries for large classes that no chapter names

One row per class (155). "381 ctor / first" = the constructor address, or the first function address when the constructor is not known. "Key methods" = the first two methods that match the `key_methods` filter (§Files). Roles of UI panels and widgets come from the class name: see [14 §1](14-ui.md) for the panel frame. Roles of the other classes come from their method names (381) ❓ where marked.

#### Area `09-items` (32 classes)

| Class | Functions | 381 ctor / first | Role | Key methods (381) |
| --- | ---: | --- | --- | --- |
| `ItemTooltipPanel` | 116 | `0xd02da0` | UI panel ([14 §1](14-ui.md)) | `ShowPanel` `0xd14ec0`, `ShowTooltipByTransformResult` `0xd2a7e0` |
| `BGoodsStore` | 112 | `0xf4d760` | In-game goods store model (limits, wish list, payment amount). UI: `BGoodsStore*` panels. | `SetGiftUserIdSearch` `0xedfbe0`, `ProcessNetObjectEvent` `0xf4d330` |
| `TencentIngameShopPanel` | 93 | `0x8d53c0` | UI panel ([14 §1](14-ui.md)) | `ProcessNetObjectEvent` `0x8a04e0`, `RequestNotice` `0x8aeab0` |
| `AccountContentsPanelBadge` | 80 | `0x127de90` | UI panel ([14 §1](14-ui.md)) | `SetInventoryFunctionOwner` `0x123fa50`, `UpdateBadgeWidgetTooltip` `0x128c4a0` |
| `NPAuctionPanel` | 80 | `0xc21150` | UI panel ([14 §1](14-ui.md)) | `Show` `0xc20dc0`, `ProcessNetObjectEvent` `0xc231a0` |
| `NewCharEquipWindowPanel` | 80 | `0x11dcf20` | UI panel ([14 §1](14-ui.md)) | `ShowContentsJournalTab` `0x1195bb0`, `ShowRaidDungeonBoundPcTooltip` `0x11af150` |
| `RandomStorePanel` | 80 | `0x9bad10` | UI panel ([14 §1](14-ui.md)) | `ShowError` `0x99ea70`, `ClosePanelProcess` `0x9e0860` |
| `DragonJadeShopPanel` | 65 | `0x1038100` | UI panel ([14 §1](14-ui.md)) | `RequestPurchase` `0x106c2c0`, `ProcessNPEvent` `0x1092bd0` |
| `ItemWarningConfirmPanel` | 60 | `0xcdd200` | UI panel ([14 §1](14-ui.md)) | `Update` `0xd54470`, `Close` `0xd69ae0` |
| `GuildCraftPanel` | 52 | `0xf1ff10` | UI panel ([14 §1](14-ui.md)) | `UpdateSelectLabel` `0xede2d0`, `SendDonateMaterial` `0xf3bb50` |
| `TencentIngameShop` | 44 | `0x125be10` | Tencent in-game shop model (goods, packages, balance, currency lock). China publisher only ❓. | `RequestNotice` `0x8729c0`, `RequestCheckNewGoods` `0x8b0e70` |
| `PartyAuctionPanel` | 40 | `0xc053b0` | UI panel ([14 §1](14-ui.md)) | `SetCustomizeMode` `0xc05200`, `ShowPanel` `0xc05230` |
| `AccountContentsPanelBadgeEquip` | 35 | `0x1253f80` | UI panel ([14 §1](14-ui.md)) | `SetPanelEnable` `0x123f790`, `ProcessNetObjectEvent` `0x12af3c0` |
| `InventoryItemIcon` | 35 | `0x487260` | UI widget | `UpdateRecycleTimeLeftSecond` `0xd0c320`, `UpdateItem` `0xd340b0` |
| `ItemGraphMap` | 32 | `0x4beae0` | Item growth graph UI (paths, scroll, particles). | — |
| `BPTFieldItem` | 31 | `0x4f3a80` | Field item presentation: markers, blink, link to dead body. | `UpdateReserveLinkParticle` `0xabaf00`, `ProcessActionSubEvent` `0xae9030` |
| `DeliveryBox::ReceivedDelivery` | 30 | `0x4638f0` | One received mail; parses auction and market attachment nodes. Mail: [11 §5](11-social.md). | `ReceivedDelivery` `0x4638f0`, `SetSenderName` `0x103f700` |
| `AuctionArticleList` | 29 | `0x11da910` | UI widget | `ShowArticleList` `0x11948b0`, `SetWidget` `0x1194b20` |
| `DeliveryPanel::ReceivedPage` | 29 | `0x534570` | UI panel ([14 §1](14-ui.md)) | `ReceivedPage` `0x534570`, `UpdateListDataAndMaxPageNumber` `0x1077910` |
| `RandomStoreRewardConfirmPanel` | 28 | `0x9b9dd0` | UI panel ([14 §1](14-ui.md)) | `ProcessEvent` `0x9e4200`, `UpdatePanel` `0xa036f0` |
| `InventoryFunctionConfirmPanel` | 27 | `0x4bc740` | UI panel ([14 §1](14-ui.md)) | `Close` `0xdc5400`, `ProcessEvent` `0xe27140` |
| `ItemBarPanel` | 27 | `0xd94230` | UI panel ([14 §1](14-ui.md)) | `ShowPanel` `0xda6160`, `UpdateKeyMappingText` `0xda6450` |
| `ItemStoreBuyWarningPanel` | 27 | `0xcddae0` | UI panel ([14 §1](14-ui.md)) | `Open` `0xd679a0`, `ProcessEvent` `0xd6d560` |
| `ItemStoreInputAmountPanel` | 25 | `0xd250a0` | UI panel ([14 §1](14-ui.md)) | `Open` `0xd2d5f0`, `ProcessEvent` `0xd308a0` |
| `AuctionItemSearchResultList` | 23 | `0x1194490` | UI widget | `SetWidget` `0x11944d0`, `Show` `0x1194600` |
| `GoodsStoreHistoryPanel` | 23 | `0xf340a0` | UI panel ([14 §1](14-ui.md)) | `SetCurrencyGroupId` `0xedf340`, `SetHistoryType` `0xedf350` |
| `ItemIconComp` | 23 | `0x534790` | UI widget | `UpdateTradeItem` `0xd181a0`, `UpdateItem` `0xd185c0` |
| `AuctionMyBidItemList` | 22 | `0x1194ff0` | UI widget | `SetWidget` `0x1195140`, `Update` `0x1209a80` |
| `BGoodsStore::Goods` | 22 | `0x47dda0` | UI or model helper ❓ | — |
| `GoodsIcon::GoodsIconComp` | 22 | `0xee0630` | UI widget | `SetIconStateShow` `0xee0530`, `SetIconStateText` `0xee0580` |
| `SelectItemConfirmPanel` | 22 | `0x938bd0` | UI panel ([14 §1](14-ui.md)) | `Open` `0x9377f0`, `Close` `0x9503e0` |
| `BGoodsStore::GoodsContainer` | 20 | `0xf42260` | UI or model helper ❓ | — |

#### Area `11-social` (27 classes)

| Class | Functions | 381 ctor / first | Role | Key methods (381) |
| --- | ---: | --- | --- | --- |
| `GuildCustomizePanel` | 60 | `0xe77330` | UI panel ([14 §1](14-ui.md)) | `UpdateGuideImg` `0xe33c40`, `UpdatePhotoAlbumMenu` `0xe3b130` |
| `TeamPanel2` | 47 | `0x88ef70` | UI panel ([14 §1](14-ui.md)) | `ProcessChatChannelEvent` `0x874f60`, `UpdateSpecialSkillGauge` `0x88e820` |
| `ChatPanel` | 45 | `0x1125710` | UI panel ([14 §1](14-ui.md)) | `SetCustomizeMode` `0x1125790`, `ProcessEvent` `0x1189a80` |
| `PartyBasicFramePanel` | 45 | `0xbe5620` | UI panel ([14 §1](14-ui.md)) | `ProcessCrossCallbackEvent` `0xbd0170`, `UpdateAggro` `0xbd6110` |
| `GuildDesignPanel` | 41 | `0xe75b90` | UI panel ([14 §1](14-ui.md)) | `Update` `0xe39a70`, `ShowRegistWithDeleteConfirm` `0xe75600` |
| `PartySearchPanel` | 40 | `0xb8e7e0` | UI panel ([14 §1](14-ui.md)) | `UpdateGuildSearchTimer` `0xb67230`, `ProcessPartyChatRecruitEvent` `0xb77ba0` |
| `FriendListTab` | 39 | `0xfab970` | UI widget | `UpdateCompleted` `0xf77ca0`, `Update` `0x1000380` |
| `TeamEditPanel` | 35 | `0x890570` | UI panel ([14 §1](14-ui.md)) | `ProcessCrossCallbackEvent` `0x876470`, `ProcessChatChannelEvent` `0x8765a0` |
| `ChatHistoryOptionPanel` | 33 | `0x115d940` | UI panel ([14 §1](14-ui.md)) | `ProcessEvent` `0x1185a40` |
| `GuildTabPageInformationPanel` | 33 | `0xe9e410` | UI panel ([14 §1](14-ui.md)) | `ShowGuildParticle` `0xe55da0`, `ShowPurchaseConfirm` `0xe6c8f0` |
| `PartyChatRecruit` | 33 | `0xc1b700` | UI widget | `SetAutoSendRecruitmentTime` `0xbd0860`, `UpdateMessageState` `0xbd0910` |
| `GuildChatRecruit` | 32 | `0xefd5f0` | UI widget | `SendSuggestJoin` `0xee2560`, `SetAutoSendRecruitmentTime` `0xef7910` |
| `ChatChannelColumnList` | 30 | `0x1150cc0` | UI widget | `SetGlobalFontsize` `0x112c950`, `ClosePopup` `0x1134380` |
| `PartyMemberPanel2` | 28 | `0xbe29d0` | UI panel ([14 §1](14-ui.md)) | `ProcessCrossCallbackEvent` `0xbcfc40`, `UpdateVoiceBtn` `0xbd53c0` |
| `Guild2DonateConfirmPanel` | 26 | `0xf2f8b0` | UI panel ([14 §1](14-ui.md)) | `Open` `0xf5d290`, `ProcessEvent` `0xf6d890` |
| `FriendPanel` | 25 | `0xfbee60` | UI panel ([14 §1](14-ui.md)) | `ShowAddBlackListConfirm` `0xf7a880`, `ShowDeleteAllSpammerListConfirm` `0xf8f000` |
| `ChatHistory` | 24 | `0x1145750` | Chat history store (rows, unread count, filters). | `SetName` `0x440a70`, `SetChatChannelInfo` `0x115a2e0` |
| `GuildCustomizePurchasePanel` | 24 | `0xe9fce0` | UI panel ([14 §1](14-ui.md)) | `UpdatePurchaseList` `0xe86180`, `ProcessNetObjectEvent` `0xe86270` |
| `GuildMemberColumnList` | 24 | `0xe56c60` | UI widget | `ProcessEvent` `0xedce00` |
| `PartySystemMenuPanel` | 23 | `0xb40510` | UI panel ([14 §1](14-ui.md)) | `ProcessEvent` `0xbbe090` |
| `GuildPageManagementNoticePanel` | 22 | `0xe32e00` | UI panel ([14 §1](14-ui.md)) | `ProcessNetObjectEvent` `0xe6b440`, `ProcessEvent` `0xe9dfd0` |
| `TeamReadyCheck` | 22 | `0x8805e0` | Party ready check (start, tick, replies, result message). | `NotifyWhenTeamLeaderDelegated` `0x86dda0`, `ProcessRecvMsg` `0x8a5360` |
| `ChallengePartyResultPanel` | 21 | `0x11915a0` | UI panel ([14 §1](14-ui.md)) | `ClosePanelProcess` `0x11915c0`, `SetCustomizeMode` `0x11915d0` |
| `Guild2MemberInfoPanel` | 21 | `0xe33170` | UI panel ([14 §1](14-ui.md)) | `ProcessNetObjectEvent` `0xeaf410`, `ProcessEvent` `0xedcbb0` |
| `GuildPageManagementLevelupColumnList` | 20 | `0xe55b70` | UI widget | `Update` `0xe37d90`, `ProcessEvent` `0xe961c0` |
| `PartySearchMsgLabelColumnList` | 20 | `0x4f0b40` | UI widget | `UpdateTimeout` `0xb67f30`, `UpdateSelect` `0xb89710` |
| `VoiceChatHUDPanel` | 20 | `0x751c70` | UI panel ([14 §1](14-ui.md)) | `SetCustomizeMode` `0x751d90`, `UpdateVoiceStatus` `0x75d710` |

#### Area `14-ui` (26 classes)

| Class | Functions | 381 ctor / first | Role | Key methods (381) |
| --- | ---: | --- | --- | --- |
| `PersonalCustomizePanel` | 89 | `0xb3a1b0` | UI panel ([14 §1](14-ui.md)) | `SetMouseGuideImg` `0xb3a230`, `UpdatePhotoAlbumMenu` `0xb3a560` |
| `ProductionPanel2` | 79 | `0xaef300` | UI panel ([14 §1](14-ui.md)) | `ClosePanelProcess` `0xaa86a0`, `ShowCompleteErrorInvenFullConfirm` `0xade540` |
| `ContextMenuPanel` | 69 | `0x10cb600` | UI panel ([14 §1](14-ui.md)) | `SetContextSubMenuPosition` `0x10a7a30`, `Close` `0x10a7c80` |
| `ProductionPanel` | 66 | `0xaa24f0` | UI panel ([14 §1](14-ui.md)) | `RequestRegisterProduct` `0xaaf490`, `ClosePanelProcess` `0xac3d40` |
| `ActionIcon2` | 56 | `0x40b360` | UI widget | `SetParticleScaleRate` `0x12378e0`, `SetKeyMappingText` `0x1237950` |
| `PhotoAlbumPanel` | 47 | `0xb5fbb0` | UI panel ([14 §1](14-ui.md)) | `SetPreviewCreatureProperty` `0xb39b90`, `ProcessEvent` `0xba6130` |
| `DragonJadeRefinementPanel` | 46 | `0x1039270` | UI panel ([14 §1](14-ui.md)) | `ClosePanelProcess` `0x100ba70`, `SetFocus` `0x1010d00` |
| `AutoModeOptionPanel` | 44 | `0x11c61f0` | UI panel ([14 §1](14-ui.md)) | `ShowSkillSelectionPopup` `0x11e75d0`, `ShowPanel` `0x11eed00` |
| `ContextPanel` | 41 | `0x10bac80` | UI panel ([14 §1](14-ui.md)) | `UpdateContextMode` `0x10b2200`, `UpdateKeyMappingText` `0x10cb200` |
| `InGameNoticePanel` | 36 | `0xe31c70` | UI panel ([14 §1](14-ui.md)) | `ClosePanelProcess` `0xe31c70`, `UpdateVip` `0xe93830` |
| `WindowConfirmPanel` | 36 | `0x10d0950` | UI panel ([14 §1](14-ui.md)) | `Close` `0x10df9e0`, `Open` `0x110a660` |
| `AreaInfoPanel` | 32 | `0x1283c70` | UI panel ([14 §1](14-ui.md)) | `SetBackgroundOpacity` `0x12375c0`, `SetDefaultBackgroundOpacity` `0x1237600` |
| `ReplayCommonPanel` | 28 | `0x9b2bc0` | UI panel ([14 §1](14-ui.md)) | `Show` `0x998b90`, `SetActivate` `0x999210` |
| `SelectWorldPanel` | 28 | `0x92e510` | UI panel ([14 §1](14-ui.md)) | `UpdateEndWaitDelPc` `0x91a330`, `OpenSelectWorldPanel` `0x945dd0` |
| `BossListPanel` | 27 | `0x11bb9c0` | UI panel ([14 §1](14-ui.md)) | `ProcessEvent` `0x11bb6f0`, `SetCustomizeMode` `0x11d3ac0` |
| `PersonalCustomizePurchasePanel` | 27 | `0xb8c810` | UI panel ([14 §1](14-ui.md)) | `Show` `0xb72c90`, `ShowUICommand` `0xbb6ca0` |
| `UniverseCombinePCActivationPanel` | 26 | `0x74f000` | UI panel ([14 §1](14-ui.md)) | `Show` `0x776720`, `ProcessLobbyEvent2` `0x776b20` |
| `BCollectingList` | 25 | `0x10da360` | UI widget | `UpdateDoneCollectingIdList` `0x1109020` |
| `QQGroupPanel` | 25 | `0xa54210` | UI panel ([14 §1](14-ui.md)) | `ShowPanel` `0xa5ff50`, `ProcessNetObjectEvent` `0xa6ba60` |
| `SecondPasswordConfigPanel` | 25 | `0x9d0610` | UI panel ([14 §1](14-ui.md)) | `ProcessNPEvent` `0x9ae070`, `ShowPanel` `0x9c27f0` |
| `PlayReportPanel` | 24 | `0xb84400` | UI panel ([14 §1](14-ui.md)) | `Close` `0xb694b0`, `ProcessNetObjectEvent` `0xba8c90` |
| `ProductionNotificationPanel` | 24 | `0x50c370` | UI panel ([14 §1](14-ui.md)) | `Open` `0xb25430`, `ProcessNetObjectEvent` `0xb282d0` |
| `ProductionNotificationPanel2` | 24 | `0xaa2820` | UI panel ([14 §1](14-ui.md)) | `Open` `0xb24d80`, `ProcessNetObjectEvent` `0xb281c0` |
| `GameTipBalloonPanel` | 22 | `0xf2be00` | UI panel ([14 §1](14-ui.md)) | `ProcessEvent` `0xefb670`, `Open` `0xf55590` |
| `QQRecentContactUserPanel` | 21 | `0xa1f170` | UI panel ([14 §1](14-ui.md)) | `UpdateRecentContactUserColumnList` `0xa537f0`, `Update` `0xa6a110` |
| `SurveyPanel` | 21 | `0x91b410` | UI panel ([14 §1](14-ui.md)) | `ClosePanelProcess` `0x8ff740`, `ShowPanel` `0x92f4a0` |

#### Area `10-quests-npc` (21 classes)

| Class | Functions | 381 ctor / first | Role | Key methods (381) |
| --- | ---: | --- | --- | --- |
| `AttendanceCalendarPanel` | 72 | `0x125bb00` | UI panel ([14 §1](14-ui.md)) | `ClosePanelProcess` `0x1235c80`, `ShowPanel` `0x12805b0` |
| `AttendancePanel` | 71 | `0x11b3fd0` | UI panel ([14 §1](14-ui.md)) | `ClosePanelProcess` `0x1195a30`, `ProcessEvent` `0x1235400` |
| `ContentsJournal2Panel` | 66 | `0x74d490` | UI panel ([14 §1](14-ui.md)) | `UpdateCompleted` `0x109e860`, `Update` `0x1102380` |
| `QuestQuickSlotList` | 65 | `0x9c2960` | UI widget | `Show` `0xa1db70`, `UpdateTencentExpBoostGuide` `0xa57e30` |
| `AchievementPanel` | 42 | `0x1286430` | UI panel ([14 §1](14-ui.md)) | `ShowTitleEmptyListGuide` `0x1238370`, `ShowEmptyListGuide` `0x12383b0` |
| `ContentsJournalPanel` | 42 | `0x10cbaa0` | UI panel ([14 §1](14-ui.md)) | `UpdateCompleted` `0x109e5b0`, `Update` `0x10df360` |
| `ChoiceRewardConfirmPanel` | 40 | `0x114eae0` | UI panel ([14 §1](14-ui.md)) | `Update` `0x1173f40`, `Close` `0x1179880` |
| `BPTNpcDeadBody` | 34 | `0x4f2d90` | NPC dead body presentation: loot pouch markers, despawn. | `UpdateNamePlate` `0xaa1820`, `UpdatePhantomMode` `0xaa4620` |
| `AccountContentsPanelCompose` | 29 | `0x1238da0` | UI panel ([14 §1](14-ui.md)) | `ProcessNetObjectEvent` `0x12baf40`, `ShowPanel` `0x12bb450` |
| `BAchievementList` | 28 | `0x1268e60` | UI widget | — |
| `AttendanceTimeTablePanel` | 27 | `0x11dbf30` | UI panel ([14 §1](14-ui.md)) | `ProcessEvent` `0x122c480`, `ShowPanel` `0x122c870` |
| `QuestInfoColumnList` | 25 | `0xa3db10` | UI widget | `UpdateQuest` `0xa3d910`, `ProcessEvent` `0xa9c630` |
| `BTalkAttractionRewardPanel` | 23 | `0xbc9620` | UI panel ([14 §1](14-ui.md)) | `SetWaitingEvent` `0xbc9620`, `ShowQuestFunction` `0xc441f0` |
| `ContentsJournal2` | 23 | `0x10cdc40` | Contents journal model (contents list, recommend items). | `SetNotiType` `0x109eaf0` |
| `DungeoninitializeConfirmPanel` | 23 | `0xfc7f00` | UI panel ([14 §1](14-ui.md)) | `Update` `0xffa060`, `Open` `0xffa120` |
| `TendencyFieldQuestQuickSlotList` | 23 | `0x887370` | UI widget | `UpdateQuestList` `0x887310`, `ProcessEvent` `0x8f6d90` |
| `ProgressQuestList` | 22 | `0x513a60` | UI widget | `UpdateTalkToSelfQuest` `0xa88a80`, `UpdateQuestList` `0xa88c70` |
| `QuestIcon` | 22 | `0x515a70` | UI widget | `ShowToggleState` `0xa1eb10`, `ShowWidget` `0xa279a0` |
| `AccountContentsPanel` | 21 | `0x127e090` | UI panel ([14 §1](14-ui.md)) | `SetTabType` `0x123fa90`, `SetPanelEnable` `0x123fad0` |
| `PTNpcSocial` | 21 | `0x50ded0` | NPC quest / relationship indicator socials (animations). | `SetIndicatorSocial` `0xaa4590` |
| `QuestTableImpl` | 20 | `0x403a90` | Quest table loader ([13](13-datafile.md), [10](10-quests-npc.md)). | `SetTableCacheSize` `0x4f0760` |

#### Area `07-combat` (20 classes)

| Class | Functions | 381 ctor / first | Role | Key methods (381) |
| --- | ---: | --- | --- | --- |
| `DuelTeamMemberPanel` | 54 | `0xfa4f40` | UI panel ([14 §1](14-ui.md)) | `ProcessCrossCallbackEvent` `0xf7d6c0`, `UpdateViewWidget` `0xf81d80` |
| `DuelTeamMemberSlot` | 51 | `0xfa4360` | UI widget | `SetDuelGameFinish` `0xf79a60`, `SetDuelRoundStart` `0xf79a80` |
| `SkillTrainJobStylePanel` | 49 | `0x93e7a0` | UI panel ([14 §1](14-ui.md)) | `SetGuideJobStyleAndTraitIndex` `0x905cd0`, `ShowChangeSkillUIParticleByItemEquip` `0x9118f0` |
| `DuelProgressPanel` | 38 | `0x102e940` | UI panel ([14 §1](14-ui.md)) | `UpdateObserverWinState` `0x102d590`, `UpdateAttrectionWinState` `0x102da10` |
| `DuelResultPanel` | 37 | `0xfbb1f0` | UI panel ([14 §1](14-ui.md)) | `ClosePanelProcess` `0xf7a1f0`, `ProcessNetObjectEvent` `0xfe5460` |
| `DuelObserverTeamMemberSlot` | 35 | `0xfc8790` | UI widget | `ShowTagInterference` `0xf79480`, `UpdateTagInterference` `0xfa0b10` |
| `PlayerCombatStatusPanel` | 30 | `0xb38e20` | UI panel ([14 §1](14-ui.md)) | `ShowGuardGaugeHolder` `0xb4abe0`, `UpdateAutoMode` `0xb4ae70` |
| `SkillTrainingRoomSelectionPanel` | 30 | `0x947d70` | UI panel ([14 §1](14-ui.md)) | `SetPanelEnable` `0x906040`, `ShowPanel` `0x931540` |
| `DuelBotChallengePanel` | 27 | `0x1033cc0` | UI panel ([14 §1](14-ui.md)) | `SetCustomizeMode` `0x100b490`, `ProcessNetObjectEvent` `0x10334c0` |
| `DuelBotTrainingRoomMatchSelectionPanel` | 25 | `0x10307b0` | UI panel ([14 §1](14-ui.md)) | `ShowPanel` `0x104c490`, `ReceiveReward` `0x1064d40` |
| `PTPassiveEffectList` | 23 | `0xab64e0` | Passive effect shows on a creature (buff/duel effects). | `ShowEffectShow` `0xacd650` |
| `DuelBotChallenge` | 22 | `0x100fe60` | Duel bot challenge model: floors, cards, timers from data. | `RequestLeaveDuelBotChallenge` `0x105ac20`, `ProcessNetObjectEvent` `0x109abf0` |
| `DuelBotChallengeCardSelectionPanel` | 22 | `0x1034340` | UI panel ([14 §1](14-ui.md)) | `SetCustomizeMode` `0x100fe10`, `ShowPanel` `0x104d1d0` |
| `DuelProgressState` | 22 | `0x102cdd0` | Duel round state: init, start, finish, timers. | `ProcessActionSubEvent` `0x1049e90` |
| `SkillTrainingRoomControlPanel` | 22 | `0x9010e0` | UI panel ([14 §1](14-ui.md)) | `SetPanelEnable` `0x906900`, `ProcessLeaveBtnClick` `0x932280` |
| `SkillTrainingSequenceManager` | 21 | `0x912bd0` | Skill training room sequence checks ([07](07-combat.md) training room). | `ProcessActionSubEventForOthers` `0x921d40`, `ProcessActionSubEvent` `0x921db0` |
| `DuelBotChallengeResultPanel` | 20 | `0x10321b0` | UI panel ([14 §1](14-ui.md)) | `ClosePanelProcess` `0x100b300`, `SetCustomizeMode` `0x100b350` |
| `DuelSpectateCameraPanel` | 20 | `0xfa6580` | UI panel ([14 §1](14-ui.md)) | `SetCustomizeMode` `0xf7dc60`, `ProcessReleasedKeyEvent` `0xf8b3f0` |
| `SkillComboList` | 20 | `0x534290` | UI widget | `SetSearchStance` `0x917990`, `ProcessEvent` `0x982e20` |
| `SkillTrainComboActionPanel` | 20 | `0x9487a0` | UI panel ([14 §1](14-ui.md)) | `Open` `0x906a30`, `ProcessEvent` `0x90c410` |

#### Area `08-creature` (10 classes)

| Class | Functions | 381 ctor / first | Role | Key methods (381) |
| --- | ---: | --- | --- | --- |
| `CustomizeCharacterPanel` | 77 | `0x1056b50` | UI panel ([14 §1](14-ui.md)) | `UpdateGuideImg` `0x100d470`, `SetEnableCustomize` `0x1013c00` |
| `SelectCharacterPanel` | 73 | `0x9db240` | UI panel ([14 §1](14-ui.md)) | `SetTotalEnterWorldWaitingCount` `0x909a80`, `OpenWaitingEnterWorldConfirmPanel` `0x99be80` |
| `AppearancePictorialBookPanel` | 62 | `0x12842c0` | UI panel ([14 §1](14-ui.md)) | `ClosePanelProcess` `0x1237770`, `SetFocus` `0x123db50` |
| `PetitionPanel` | 41 | `0xb90c80` | UI panel ([14 §1](14-ui.md)) | `UpdateMovementPage` `0xb90740`, `ProcessNetObjectEvent` `0xb97240` |
| `PlayerStatusPanel` | 36 | `0xb56a40` | UI panel ([14 §1](14-ui.md)) | `UpdateSecondaryMoney` `0xb56460`, `UpdateDragonJadeBalance` `0xb565c0` |
| `CreateCharacterPanel` | 35 | `0x10c9a40` | UI panel ([14 §1](14-ui.md)) | `CloseCreationPanel` `0x10a05c0`, `OpenCreationPanel` `0x10f75c0` |
| `PTFishing` | 35 | `0xabab80` | Fishing presentation on the player (rod, fish attach, hookset sequence). See §4.6. | `ProcessFishiManipulateAction` `0xafce30` |
| `PlayerAbilityRemasterPanel` | 33 | `0x4f3220` | UI panel ([14 §1](14-ui.md)) | `Open` `0xb391d0`, `UpdateCompleted` `0xb391f0` |
| `PlayerAbilityPanel` | 30 | `0xb39260` | UI panel ([14 §1](14-ui.md)) | `Open` `0xb39380`, `UpdateCompleted` `0xb393a0` |
| `PlayerExpPanel` | 21 | `0xb38c80` | UI panel ([14 §1](14-ui.md)) | `ProcessNetObjectEvent` `0xb79aa0`, `ProcessEvent` `0xb9cfb0` |

#### Area `other` (7 classes)

| Class | Functions | 381 ctor / first | Role | Key methods (381) |
| --- | ---: | --- | --- | --- |
| `Production` | 44 | `0xaa2b50` | Craft (production) model: recipes, catalog, craft post retrieve. Sync group 14 `G_Craft_SYNC` ([05 §4](05-game-dispatch.md)). | `SetCurCatalogListState` `0xaafa30`, `RequestQuickReceiveProduct` `0xafef80` |
| `DrAliasMapImpl` | 28 | `0x6251c0` | Datafile alias map ([13](13-datafile.md)). | — |
| `namespace'` | 28 | `0x61d660` | Not a class: IDA groups anonymous-namespace helpers here (quest free, BT node, asset path). | — |
| `UBehaviorTreeComponent` | 27 | `0x7188a0` | UE4-style behaviour tree runner (tasks, aux nodes). Used by client-side AI ❓. | `UpdateAbortingTasks` `0x711f60`, `RequestExecution` `0x718660` |
| `UBlackboardComponent` | 25 | `0x718440` | UE4-style blackboard (key values for the behaviour tree). | `SetValue<UBlackboardKeyType_Vector>` `0x436230`, `SetValue<UBlackboardKeyType_Bool>` `0x438300` |
| `Tracker` | 23 | `0x7ff1c0` | Follow / chase helper for a moving object (trail, engine move). | `SetTrailer` `0x7fee00`, `SetValidPos` `0x82ee50` |
| `DrCacheData` | 22 | `0x621100` | Datafile record cache ([13](13-datafile.md)). | `SetTableCacheSize` `0x62fcc0` |

#### Area `12-infogate-arena` (5 classes)

| Class | Functions | 381 ctor / first | Role | Key methods (381) |
| --- | ---: | --- | --- | --- |
| `PartyBattleFieldResultPanel` | 45 | `0xbc7bb0` | UI panel ([14 §1](14-ui.md)) | `ClosePanelProcess` `0xbc7bb0`, `SetCustomizeMode` `0xbc7bd0` |
| `PartyBattleFieldPanel` | 44 | `0xbea6a0` | UI panel ([14 §1](14-ui.md)) | `SetCustomizeMode` `0xbc7c20`, `Show` `0xc12340` |
| `BArenaEnterNotificationPanel` | 35 | `0x1273f80` | UI panel ([14 §1](14-ui.md)) | `SendEnterArenaFinalDecision` `0x1273af0`, `SendEnterArenaPartyAgreement` `0x1273b50` |
| `ArenaDungeonPartyChatRecruit` | 24 | `0x12499a0` | UI widget | `SetUsePromoteBattleRoyalFilter` `0x1236210`, `SetUsePromotePartyBattleFilter` `0x1236220` |
| `BattleRoyalResultPanel` | 21 | `0x11c3700` | UI panel ([14 §1](14-ui.md)) | `ClosePanelProcess` `0x1193410`, `SetCustomizeMode` `0x1193420` |

#### Area `04-lobbygate` (4 classes)

| Class | Functions | 381 ctor / first | Role | Key methods (381) |
| --- | ---: | --- | --- | --- |
| `LobbyServicePanel` | 34 | `0xc86da0` | UI panel ([14 §1](14-ui.md)) | `UpdateCompletedSlotLabelList` `0xc6fff0`, `Show` `0xca6010` |
| `DungeonLobbyControlPanel` | 29 | `0xf9e980` | UI panel ([14 §1](14-ui.md)) | `UpdatePhotoAlbumIconBadge` `0xf7c9e0`, `UpdateSuggestJoinGuildIconBadge` `0xf9e460` |
| `DungeonLobbyPartyBattleInfoPanel` | 24 | `0xf9e340` | UI panel ([14 §1](14-ui.md)) | `UpdateMapSelect` `0xff3a60`, `ProcessEvent` `0xff7cf0` |
| `DuelLobbyControlPanel` | 21 | `0x102fc90` | UI panel ([14 §1](14-ui.md)) | `UpdatePhotoAlbumIconBadge` `0x100f640`, `UpdateSuggestJoinGuildIconBadge` `0x102f5c0` |

#### Area `13-datafile` (2 classes)

| Class | Functions | 381 ctor / first | Role | Key methods (381) |
| --- | ---: | --- | --- | --- |
| `Data::DataManager` | 51 | `0x5b6370` | Datafile table owner ([13](13-datafile.md)). | — |
| `DrDataReaderImpl` | 20 | `0x617480` | Datafile reader ([13](13-datafile.md)). | `Setup` `0x617480` |

#### Area `06-movement` (1 classes)

| Class | Functions | 381 ctor / first | Role | Key methods (381) |
| --- | ---: | --- | --- | --- |
| `JumpingLevel2Panel` | 21 | `0xcf1590` | UI panel ([14 §1](14-ui.md)) | `Close` `0xce0400`, `ProcessNetObjectEvent` `0xd1b8e0` |

### 2.4 Quick-find check

For each quick-find row of chapters 00–16 the script reads the 381 IDA name at the first address of the row ([`17-quick-find.tsv`](17-quick-find.tsv), column `check`):

| `check` | Rows | Meaning |
| --- | ---: | --- |
| `name-ok` | 208 | The last identifier of the function cell is in the 381 IDA name. |
| `func-other-name` | 100 | The address is a function start, but the 381 IDA name is other. Most rows use a role name (`connect switch`) or a `sub_` name, or the 381 IDA name is a known false match (for example `0x5d25d0` "connect by name" has the 381 IDA name `SeasonContentsTimeAttackRecord::~SeasonContentsTimeAttackRecord`). |
| `data` | 26 | The address is a global, a vtable or a string. |
| `inside-func` | 8 | The address is inside a function (a switch or a call site). |
| `no-address` | 10 | The row has no address (a type, or "—"). |

No row has an address outside the 381 segments.

---

## 3. Open items

### 3.1 How the list is made

[`17-open-items.tsv`](17-open-items.tsv) has one row for:

- each line of chapters 00–16 that has ❓ (except the "❓ marks …" legend lines), and
- each list or table row in a section whose title has "Open".

`kind` and `next_action`:

| `kind` | Rows | Rule | Next action |
| --- | ---: | --- | --- |
| `code` | 452 | The row has a 381 address, or no other rule applies | "Decompile `0x…` in 381 and confirm", or "Find a 381 code anchor (string or xref) and confirm" |
| `gm-only` | 15 | The text says the fact comes from 419 GM only | Find the 381 switch, table or string that uses the value |
| `server` | 14 | The text names the server and has no address | Confirm against [`Docs/Server/`](../../Server/) or a server binary |
| `capture` | 10 | The text says capture or live | Confirm with a live capture against the Rust server |
| `data` | 8 | The text names a datafile table or record | Read the table in the [Datafile book](../../DataFile/book/00-index.md) |
| `table` | 10 | The row is in an "Open items" table with a "next step" column | The chapter's own next step |

### 3.2 Count per chapter

| Chapter | Rows | `code` | `gm-only` | `server` | `capture` | `data` | `table` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 00-inventory | 29 | 17 | 0 | 1 | 0 | 1 | 10 |
| 01-startup | 40 | 39 | 0 | 0 | 0 | 1 | 0 |
| 02-transport | 32 | 27 | 2 | 2 | 0 | 1 | 0 |
| 03-sts-client | 32 | 27 | 1 | 4 | 0 | 0 | 0 |
| 04-lobbygate | 27 | 26 | 0 | 0 | 1 | 0 | 0 |
| 05-game-dispatch | 33 | 30 | 2 | 0 | 0 | 1 | 0 |
| 06-movement | 45 | 38 | 3 | 4 | 0 | 0 | 0 |
| 07-combat | 30 | 28 | 2 | 0 | 0 | 0 | 0 |
| 08-creature | 48 | 46 | 1 | 1 | 0 | 0 | 0 |
| 09-items | 25 | 24 | 0 | 0 | 1 | 0 | 0 |
| 10-quests-npc | 13 | 12 | 1 | 0 | 0 | 0 | 0 |
| 11-social | 30 | 26 | 0 | 0 | 4 | 0 | 0 |
| 12-infogate-arena | 49 | 45 | 1 | 1 | 2 | 0 | 0 |
| 14-ui | 31 | 28 | 1 | 0 | 0 | 2 | 0 |
| 15-engine | 16 | 11 | 1 | 1 | 1 | 2 | 0 |
| 16-strings-errors | 29 | 28 | 0 | 0 | 1 | 0 | 0 |
| **Total** | **509** | | | | | | |

### 3.3 Open items that block the server port (first priority)

These rows change what the Rust server must send or accept. Ids are the `id` column of [`17-open-items.tsv`](17-open-items.tsv).

| Id | Chapter | Item | Next action |
| --- | --- | --- | --- |
| Q097 | [02 §5.2](02-transport.md) | Server-ping time-out logic and the send-queue limit | Decompile the ping code from `0x5d28f0` (ping all sessions); find the limit compare in `Network::Send` `0x5d2740`. |
| Q099 | [02 §2.3](02-transport.md) | Meaning of the 4 extra bytes in the next fragments | Read the fragment join in `recvNext` `0x5cfb70`; confirm with a capture. |
| Q187 | [05 §2](05-game-dispatch.md) | Map of each command to its final receiver under `World::ProcessNetObjectEvent` `0x749550` | Script the switch cases of `0x749550` into a TSV (case → callee). |
| Q188 | [05](05-game-dispatch.md) | 129 `own` handlers; only 12 and 17 are read (auth 20/21/24/25, return to lobby 27/28, enter world 32 …) | Decompile the handlers from `game-commands.tsv` column `handler_381`. |
| Q190 | [05 §3](05-game-dispatch.md) | Byte format of `T_PROP_PAIR`, `T_PROP_SYNC`, `T_PACKET_REF` | Read cases 10, 11, 16 of the field reader `0x5d4f20` (also [08](08-creature.md) Q-row for `0x5d4810`). |
| Q192 | [05 §2.3](05-game-dispatch.md) | 28 C2S commands without a builder | Search the command id as an immediate in `push`/`mov` near `SendToGamed` `0x114ce20` callers. |
| Q193 | [05](05-game-dispatch.md) | 381 ping time-out (`_reqPingRemainTime`) and `GameSession::Tick` | Find the `GameSession` tick through its vtable `0x15896b8`. |
| Q263 | [07](07-combat.md) | Sender of `G_SkillTriggerReqCmd` 1231 | Find xrefs to the create function of command 1231 (`game-commands.tsv`, `create_381`). |
| Q266 | [07 §8](07-combat.md) | `PROP_PAIR` / `T_PROP_SYNC` byte format | Same as Q190. |
| Q302 | [08](08-creature.md) | Meaning of `controlId` in cmd 8 / 9; object select in `0x749550` | Read the `controlId` use at the top of `0x749550`. |
| Q310 | [08 §6](08-creature.md) | `G_RespawnFail` handler: decompile failed | Fix the function end at `0xeeeec0`; decompile again. |
| Q338 | [09](09-items.md) | Field layout of the Fail packets (error code offset) | Read `packet-layouts.tsv` rows of `*Fail` commands; confirm the read in the handler. |
| Q341 | [09](09-items.md) | Senders not found: 833, 1325, 1341, 1361 | Xrefs to the create functions (`09-items-commands.tsv`). |
| Q383 | [11](11-social.md) | Builders without address: `LootReq` 1272, `UpdateSalePriceReq` 1536, `RetrievePostCountReq` 368 | Same method as Q192. |
| Q425 | [12 §1.4](12-infogate-arena.md) | Senders of InfoGate commands 7, 49, 55 | Xrefs to `SendToInfoGate` `0x114cdb0` with these ids. |
| Q431 | [12 §2.4](12-infogate-arena.md) | ArenaD connect time-out and reconnect timer | Read the ArenaD path of `GameSession::SendEnterArenaReq` `0xf2bf60`. |
| Q477 | [15](15-engine.md) | Server reply to `G_EnterZoneReqCmd` 42 and the sync option packet | Compare with the GameD spec in [`Docs/Server/`](../../Server/); capture. |
| Q503 | [16 §1.4](16-strings-errors.md) | Writer of `DisconnectGameReason` (`GameSession +0x60`) | Search writes of offset `0x60` through the `GameSession` pointer `BNSClient+0x24`. |

Other blocked work:

| Item | State | Next action |
| --- | --- | --- |
| Chapter 18: GM debugger and builder UI, `gm-commands.tsv` (212 `GameBuilderMainCommand`, 214 `GameDebugMainCommand`), port plan B | Blocked: the 419 GM `.i64` did not open (handoff cx-18). | Repair or replace `Client_gm.exe.i64`, then run cx-18 again. |
| 419 GM cross-checks in chapters 14 and 15 | Names come from the 381 til only (Q464, Q480). | Same as above. |

### 3.4 Old open items (old notes §10): state on 2026-11-10

| # | Old item | State | Where |
| ---: | --- | --- | --- |
| 1 | `sendWalkPacket` `0xcc4ce0` decompile failed | **Closed.** Chapter 06 reads it in full. | [06 §2.1](06-movement.md) |
| 2 | Game packet wire format | **Closed.** Envelope, plain header (`int16 size`, `int16 command`) and field codes are known. | [02 §2](02-transport.md), [05 §3](05-game-dispatch.md) |
| 3 | Real names of the 3 false vehicle matches | Partly closed (§4.5). | §4.5 |
| 4 | 381 MoveType values equal GM values | **Closed** for the wire: 38 makers map to `G_MOVETYPE_*` 0–43. | [06 §1](06-movement.md), [`06-movement-makers.tsv`](06-movement-makers.tsv) |
| 5 | Mount-link use in 381 content | Open. | Read `effect` records with `mount-linker` in the [Datafile book](../../DataFile/book/00-index.md) tables. |
| 6 | Playable jobs in 381 | Open. | Read the `pc` / `job` tables. |
| 7 | Glide check byte @612 == 8 | Open (06 §4 keeps the ❓). | Map the Skill3 action layout ([07 §7](07-combat.md)). |
| 8 | Sync group rule (one pending request per group) | **Closed, and stronger:** in 381 all 18 sync ids map to task group 0, so only one sync request can be pending at a time; time-out 10.0 s. | [05 §4](05-game-dispatch.md) |

Corrections that the chapters make to the old notes:

| Old claim | Correction | Source |
| --- | --- | --- |
| The client sends moves as `GS_ActionsClientMoveCmd` (ActionsCommand 1) | False for the client → GameD path. The client sends `G_MoveReqCmd` 64 (88-byte packet). `GS_Actions*` are ActionsCommand names. | [06 §5](06-movement.md), `0xcc4ce0` |
| `PacketParser::ParseData` `0x5fd770` is the game stream framer | It serves only chat and voice. Game framing is `PPUtil::PacketSize` `0x60f8a0` and `Session::PacketSize` `0x5d4a20`. | [02 §6](02-transport.md) |
| One pending request per sync group | One pending request for all groups (group 0). | [05 §4](05-game-dispatch.md) |

---

## 4. Facts moved from the old notes

The old notes ([`../client-exe-reversing.md`](../client-exe-reversing.md) before 2026-11-10) had these facts. No chapter 00–16 had them. The source is the old notes unless a 381 address is given; "GM TPI" = 419 GM type info.

### 4.1 Symbols and id tables

- Names were demangled with `llvm-undname` (MSVC mangled names in `c20_named.json`, scratch). Method: [00 §3](00-inventory.md).
- The old named database was `t20/pk.i64` (scratch copy). Match confidence: high for the `vtable`, `bytes` and `strings` methods; lower for `fuzzy` and `vtslot` (field `how` per match, not in the TSVs, Q028).
- Old matched-function counts per class (sample): `BPTPlayer` 155, `LobbyGateSession` 55, `PTFishing` 35, `BPTPlayerSummoned` 33, `Fishing` 30, `MoveListMaker` 30, `DrElReaderImpl` 24, `GameSession` 15, `BSummonedBrain` 12, `HyperMove` 12, `GamePacketProcessor` 8, `Glide` 4, `InfoGateSession` 4, `PacketParser` 4, `PTEffectMount` 2. Counts now: [`classes.tsv`](classes.tsv).
- `LobbyGateCommand`: 110 values, the same as `Program/Server/Packet/tables/381.json` (client_max 110, ping 9, ContentsRuleList 106, PcCafe 103). 381 list: [`lobby-commands.tsv`](lobby-commands.tsv).
- Game `MainCommand`: 1,598 names in 381; 1,686 in 419 GM (the GM enum index matches the GM name table for all 1,686).
- GM has 2,017 packet struct types named `G_*`, `GS_*`, `L_*`, `I_*` (`tpi.pkl`, scratch).
- Command id tables that only GM TPI has: `InfoGateCommand` 73, `ActionsCommand` 106 (`GS_ActionsClientMoveCmd` 1, `GS_ActionsClientMoveAndEventJump` 2, `GS_ActionsSkill` 3), `GameDebugMainCommand` 214, `GameBuilderMainCommand` 212, `StsMsgLib::ECommand` 228, auth `Command` (`A_IdAuthReqCmd` …). The 381 InfoGate list is [`12-infogate-commands.tsv`](12-infogate-commands.tsv); STS: [`03-sts-commands.tsv`](03-sts-commands.tsv).
- Game command ids 1–918 are the same in 381 and 419 GM. From 919 the ids shift: 381 #919 = `G_OccupyingRaidDungeonChangeNotifyCmd`, GM #919 = `G_UnbindPersonalRaidDungeonReqCmd`. GM adds 88 names (`G_ActivateVehicle*`, `G_DeactivateVehicle*`, SoulBoost, WorldAccountCard / Museum, TreasureBoard, ImproveItem, PersonalRaidDungeon, PvpResult, FieldBossSpawnWorldAlarm, SyncDynamicData, WebRequest). Rule: use 381 ids ([`game-commands.tsv`](game-commands.tsv)); do not use GM ids above 918.

### 4.2 Characters (GM TPI; 381 use ❓)

`CreatureProperty::Job` (chapter 08 links here):

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

❓ Playable jobs in 381 are not checked. The 381 datafile has `decompose-job-reward-shooter`, so SHOOTER exists in 381.
Races: see [08 §4](08-creature.md) (`race` property).

`Data::ItemWeaponTypeType`:

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

Three gun types exist (PISTOL 5, RIFLE 6, GUN 13). The SHOOTER job uses GUN ❓. A new gun is a data change (item + skill3 + appearance), not a code change ❓.

### 4.3 Network enums (GM TPI)

| Enum | Values |
| --- | --- |
| `PacketEventType` | NONE 0, CONNECT 1, CLOSE 2, DISCONNECT 3, SEND 4, RECEIVE 5 |
| `GamePacketRightType` (flags) | NONE 0, ANY 1, NORMAL_PLAYER 2, OBSERVER 4, MOBILE_APP 8 … |
| `GamePacketSync` 19 | WorldAccount (GM only ❓; 381 has ids 1–18, [05 §4](05-game-dispatch.md)) |

Old in-memory layouts (419 GM) that no chapter repeats: `G_EnterZoneOkPacket` 16 `enterSocial` i64; `GS_ActionsClientMoveSubPacket` (40 B) 8 `creatureId` i64, 16 x i16, 18 y i16, 20 z i16, 22 yaw i16, 24 duration i16, 26 `standingSocial` i8, 28 `moveHistory` (ActionsCommand, not the GameD move path, §3.4); `G_ActivateVehicleReqPacket` (GM only) 16 `equipped_vehicle_id` i32.

### 4.4 Movement extras (GM TPI and 381 tabledefs)

- `EIMoveType` (animation side): THROW 2, ROLLING 3 … SUMMONED_* 22–34. Cat move types: `EI_MOVE_SUMMONED_RUN` … `SUMMONED_DEST` (22–34) and `G_MOVETYPE_SUMMON_COMBAT_*` (37–39).
- Hyper energy data hooks (381 tabledefs): `consume-hyper-energy-point`, `max-hyper-energy-point` (skill3 / action), `hypermove-power-value` (pc, npc, mastery-ability, summonedlevel), `enable-infinite-hyper-energy` (zonerule, battle fields).
- Glide zone hooks: `zoneenv2` airdash / airdash-leave (`airdash-level`), guild battle `airdash-1/2-by-refiner`.
- `MoveListMaker::SendWalkPacketReason` values not in chapter 06: 27 ATTACH_CATCH, 30 DETACH_CATCH, 31 UNLINK_CATCH, 26 ATTACH_INHALATION, 29 DETACH_INHALATION, 28 DETACH_MOUNT, 34 NONE_OWNER_ATTACH_DASH, 35 NONE_OWNER_DETACH_LINK_MOUNT, 36 NONE_OWNER_DETACH_INHALATION, 37 NONE_OWNER_DETACH_THROW, 38/39 ATTACH/DETACH_MOVE_MOUNT, 43 ENTER_BLOCKMOVE, 49/50 START/END_TRADE, 53/54 TALK_START/END, 55/56 PERSONAL_CUSTOMIZE_START/END, 57/58 FISHING_START/END, 59 BIND_SYNC_DUNGEON. Full list (60 values): GM TPI. Chapter 06 §2.4 gives the 381 trigger sites.
- Summoned data (`Data::SummonedRecord`, GM, 272 B): `summoned_type`, `brain_type`, `combat_follow_distance`, `far_event_distance`, `speed`, `max_speed_near_master`, `min/max_distance_to_lerp_speed`, `appearance`, `animset`, `spawn_skill_id`, `lifetime_duration`. 381 tables: `summoned`, `summonedlevel`, `summonedmasterylevel`, `summonedpreset`, `summonedbeautyshop`, `summoneddesignerpreset`. Code: [08 §5](08-creature.md).

### 4.5 Mounts, vehicles and grabs

381 has no vehicles. Check on 2026-11-10: the 381 string list (`idautils.Strings`) has 0 strings with `ehicle`. The old notes found 32 UTF-16 hits in 419 GM, the `vehicle` and `vehicleappearance` tables (`Data::VehicleRecord` 48 B, `Data::VehicleAppearanceRecord` 252 B) and the `G_ActivateVehicle*` / `G_DeactivateVehicle*` commands in GM only.

What 381 has instead ("mount" = carry or grab link):

| Mechanic | 381 data hooks | 381 code hooks |
| --- | --- | --- |
| Mount link | `effect`: `mount-linked`, `mount-linker`; `skillcastcondition3`: `fail-target-mount-linked/linker` | SendWalkPacketReason 28, 35, 38, 39 |
| Catch (grab) | `effect`: `catch-linked/linker`; `effect/link`: `catch-caster-effect`, `catch-target-effect` | 27, 30, 31 |
| Inhalation (pull) | `effect`: `inhalation-linked/linker` | 26, 29 |
| Throw | — | 37, `EI_MOVE_THROW` 2 |
| `PTEffectMount` | — | `Deactivate` `0xaa5510` and `Activate` `0xaa5520` are 1-byte `ret` stubs (checked: byte `0xc3`, size 1). |

False vehicle names in the 381 database (checked 2026-11-10):

| 381 IDA name | 381 address | Finding |
| --- | --- | --- |
| `PTActionRideVehicle::Start` | `0xac32c0` (54 bytes) | Only xref: slot 3 of the vtable at `0x1624dec`. `PTActionPickupItem::Finish` (`0x50c917`, `0xac33e5`) and `sub_ADC920` refer to this vtable. So it is a `PTAction` subclass slot, probably the pickup action ❓. Not a vehicle. |
| `BPTVehicle::DestroyVehicle` | `0xada520` (190 bytes) | Callers: `BPTCreature::OnChangedCreatureProperty` (call `0xb01cda`, only when byte `+0xD4` == 3 and word `+0xD0` > 0) and `sub_AEA9A0`. A creature-property helper; real name unknown ❓. |
| `MakeItemPreviewVehicle` | `0x1027fc0` (806 bytes) | 6 callers, all preview mesh updates: `UniverseCombinePCActivationPanel::updateMesh` `0x7671ea`, `WeaponGemPanel::updateMesh` `0x77b386`, `WardrobePanel::updateMesh` `0x7c2481`, `NativeCharacterInfoPanel::updateMeshWidget` `0xc737e7`, `DungeonLobbyPlayerSlotPanel::ApplyUserInfo` `0xfb1c15`, … So it builds a PC preview with an item (costume) ❓. |

Rule: rename these three to `sub_*` in a later pass.

GM vehicle model (reference for a later feature; GM TPI): item `equipped_vehicle_id` → `G_ActivateVehicleReq` (16: id i32) → `G_ActivateVehicleOk` / `Fail`. Client objects: `BPTVehicle` (32 B: VehicleHandle, RiderHandle, Owner, VehicleAppearance u64, SprintSpeed i32), `PTActionRideVehicle` (40 B: ShowStageHandle, VehicleItemKey u64). Data: `vehicle` {alias, vehicle_speed i16, sprint_speed i32, riding_action_duration i32, appearance, context_script}; `vehicleappearance` {mesh, materials, anim set/tree, physic_asset, ride/getoff/cancel show, `move_rider_*` and `move_vehicle_*` animation sets}.

### 4.6 Fishing

| Item | 381 address / value |
| --- | --- |
| `PTFishing::PTFishing` | `0xabab80` (393 bytes) |
| `PTFishing::ChangeFishingState` | `0xad24e0` (885 bytes) |
| `PTFishing::processFishingHooksetSuccess` | `0xad1940` (2,741 bytes) |
| `Fishing::StopAutomaticFishing` | `0xf7b2c0` (21 bytes) |
| Classes | `PTFishing` 35 functions (first `0xaa4c90`), `Fishing` 29 functions ([`functions-*.tsv`](functions-1.tsv)) |
| Move flush | SendWalkPacketReason FISHING_START 57, FISHING_END 58 |
| 381 tables | `fish`, `fishing-field`, `fishing-paste` |
| Data hooks | `fishing-point`, `grade-a-fish`, `bonus-reduce-waiting-time-amount`, `dispose-fishing-paste-probability`, `accumulate-life-contents-point-by-fishing` |

### 4.7 Ideas: what a skateboard or a horse would touch (documentation only, nothing is built)

| Need | 381 hook | Server side (Rust) |
| --- | --- | --- |
| Faster move speed | `sprint` move types, HyperMove Sprint / Boost, skill3 `fly-speed-*`, effect stats | Accept higher speed in move validation |
| Skate "tricks" | Drift (11–13), EventJump (34–36), WallRun (28–30), HyperMove RocketJump id 7 | Allow these move types out of combat |
| Ride an animal | `mount-linked/linker` effect, ATTACH_MOVE_MOUNT 38 | Make the link, move both creatures as one |
| Board or horse model | Appearance tables and animation sets in the game data | None |
| Start and stop | Item use → skill → effect (mount link or speed buff) | Handle skill and effect packets |

Option A ("fake" mount, data only): sprint / drift / boost skill with a new animation, HyperMove ids, mount-link to an NPC "horse", or the summoned cat as a follower ❓. Option B (GM-style vehicle) needs the vehicle tables and `G_ActivateVehicle*` packets; the 381 exe has neither. Best first test ❓: a costume-like item that casts a sprint-type skill with a custom social animation (datafile edit and server skill support only).

---

## 5. How to run the audit again

Run in the IDA worker (Python) with the 381 database open. Steps (the code is short; write it again from these steps):

1. Read [`classes.tsv`](classes.tsv) and `functions-1..5.tsv` (tab separated, header row).
2. Read the text of each `NN-*.md` and `NN-*.tsv` of this folder, except this chapter and the global TSVs.
3. For each class, match `(?<![A-Za-z0-9_])NAME(?![A-Za-z0-9_])` in each text. Write [`17-class-coverage.tsv`](17-class-coverage.tsv).
4. For each chapter line: if it has ❓ or is a row in an "Open" section, write a row of [`17-open-items.tsv`](17-open-items.tsv) with the rule of §3.1.
5. For each row of each "Quick find" table: take the first `0x` address, read `ida_name.get_name` and `idc.demangle_name`, and write [`17-quick-find.tsv`](17-quick-find.tsv).

Run time: below 30 s.
