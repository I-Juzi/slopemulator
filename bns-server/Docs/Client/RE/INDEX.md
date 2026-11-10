# INDEX — Client.exe 381 reverse-engineering reference

1. This page is the entry point to [`Docs/Client/RE/`](.). Use it instead of IDA, or to jump in IDA to the right address at once.
2. Part 1 is the chapter list. Part 2 is the "how to find anything" guide (grep the TSV files, then jump in IDA).
3. Part 3 is the A–Z keyword index: function, class, packet or feature → 381 address → chapter section. It has the quick-find rows of all chapters (source: [`17-quick-find.tsv`](17-quick-find.tsv)).
4. All addresses are 381 virtual addresses (image base `0x400000`), written `0x` + lower-case hex, so `grep` finds them.
5. "419 GM" marks a name or type that comes only from the 2021 GM client PDB. ❓ marks a claim that we did not verify.

## 1. Chapters

| # | Chapter | Main TSV files |
| --- | --- | --- |
| 00 | [Binary inventory and method](00-inventory.md) | [`classes.tsv`](classes.tsv), `functions-1.tsv` … `functions-5.tsv` |
| 01 | [Start, command line, config, main loop](01-startup.md) | `01-startup-*.tsv` |
| 02 | [Transport, framing, session crypto](02-transport.md) | [`02-transport-functions.tsv`](02-transport-functions.tsv) |
| 03 | [STS client (login)](03-sts-client.md) | [`03-sts-commands.tsv`](03-sts-commands.tsv), `03-sts-*.tsv` |
| 04 | [LobbyGate session](04-lobbygate.md) | [`lobby-commands.tsv`](lobby-commands.tsv), [`04-lobby-enums-gm.tsv`](04-lobby-enums-gm.tsv) |
| 05 | [Game session and command dispatch](05-game-dispatch.md) | [`game-commands.tsv`](game-commands.tsv), [`packet-layouts.tsv`](packet-layouts.tsv) |
| 06 | [Movement](06-movement.md) | `06-movement-*.tsv` |
| 07 | [Skills and combat](07-combat.md) | `07-combat-*.tsv` |
| 08 | [Player, NPC, world objects](08-creature.md) | `08-creature-*.tsv` |
| 09 | [Items, inventory, equipment](09-items.md) | `09-items-*.tsv` |
| 10 | [Quests, NPC talk, zones](10-quests-npc.md) | `10-quests-*.tsv` |
| 11 | [Party, guild, chat, mail, market](11-social.md) | `11-social-*.tsv` |
| 12 | [InfoGate, ArenaD, other servers](12-infogate-arena.md) | `12-infogate-*.tsv` |
| 13 | [Datafile (link page to the Datafile book)](13-datafile.md) | [`../../DataFile/book/tables.tsv`](../../DataFile/book/tables.tsv) |
| 14 | [UI framework, system messages, commands](14-ui.md) | `14-ui-*.tsv` |
| 15 | [Engine, map load, options, sound](15-engine.md) | [`15-engine-einterface.tsv`](15-engine-einterface.tsv) |
| 16 | [Error codes, logs, web calls, string anchors](16-strings-errors.md) | [`16-string-anchors.tsv`](16-string-anchors.tsv), `16-*.tsv` |
| 17 | [Coverage audit and open items](17-open-items.md) | `17-*.tsv` |
| 18 | GM debugger and builder UI (419 GM): not written, blocked ([17 §3.3](17-open-items.md)) | — |

Overview and old notes: [`../client-exe-reversing.md`](../client-exe-reversing.md).

## 2. How to find anything

### 2.1 Which TSV answers which question

| Question | File | Key column(s) |
| --- | --- | --- |
| What is at address X? | `functions-1.tsv` … `functions-5.tsv` | `address` (one row per function: size, name, class, area, string anchor) |
| Which functions does class C have? | `functions-*.tsv`, [`classes.tsv`](classes.tsv) | `class`; `ctor`, `vtable`, `first`, `last` |
| Which chapter covers class C? | [`17-class-coverage.tsv`](17-class-coverage.tsv) | `class` → `chapter_md`, `chapter_tsv` |
| Game command id, name, direction, handler, builder | [`game-commands.tsv`](game-commands.tsv) | `id`, `name`, `handler_381`, `create_381` |
| Game packet fields (wire order, type, offset) | [`packet-layouts.tsv`](packet-layouts.tsv) | `cmd_id`, `cmd_name`, `path` |
| LobbyGate command | [`lobby-commands.tsv`](lobby-commands.tsv) | `id`, `name`, `sender_381`, `handler_381` |
| InfoGate command and fields | [`12-infogate-commands.tsv`](12-infogate-commands.tsv), [`12-infogate-layouts.tsv`](12-infogate-layouts.tsv) | `id`, `cmd_id` |
| STS command | [`03-sts-commands.tsv`](03-sts-commands.tsv) | `id`, `sender_381`, `reply_handler_381` |
| Item / quest / social packet detail | `09-items-commands.tsv`, `10-quests-commands.tsv`, `11-social-commands.tsv` | `id`, `name`, `fields` |
| A text string seen in the game or a log | [`16-string-anchors.tsv`](16-string-anchors.tsv), [`03-sts-strings.tsv`](03-sts-strings.tsv) | `text` → `func_addr_381` |
| A UI message alias (`Msg.*`) | [`14-ui-msg-aliases.tsv`](14-ui-msg-aliases.tsv), [`14-ui-msg-cases.tsv`](14-ui-msg-cases.tsv) | `alias` → `func_addresses` |
| An error code | `16-errors-enums-gm.tsv`, `07-combat-error-codes.tsv`, `01-startup-error-codes.tsv` | `value` / `code` |
| An enum value (419 GM name) | `NN-*-enums-gm.tsv`, [`14-ui-enums.tsv`](14-ui-enums.tsv) | `enum`, `value` |
| An engine interface slot (`EInterface + off`) | [`15-engine-einterface.tsv`](15-engine-einterface.tsv) | `offset_381_hex` |
| A rule-config or client option | [`16-ruleconfig-options.tsv`](16-ruleconfig-options.tsv), [`14-ui-options.tsv`](14-ui-options.tsv) | `key`, `name` |
| A creature or item property field | [`08-creature-properties.tsv`](08-creature-properties.tsv), [`09-items-properties.tsv`](09-items-properties.tsv) | `name`, `field_id` |
| An open item (❓) | [`17-open-items.tsv`](17-open-items.tsv) | `address`, `chapter` |

### 2.2 Grep recipes (run from the repository root)

```sh
# 1. A command by name or id -> handler and builder addresses
grep -P '\tG_MoveReq' Docs/Client/RE/game-commands.tsv
awk -F'\t' '$1==64' Docs/Client/RE/game-commands.tsv

# 2. All fields of one packet, in wire order
awk -F'\t' '$2=="G_MoveReqCmd"' Docs/Client/RE/packet-layouts.tsv

# 3. What is at an address (function rows start with the address)
grep -h '^0xcc4ce0' Docs/Client/RE/functions-*.tsv

# 4. Every place that names an address (chapters and TSVs)
grep -rn '0xcc4ce0' Docs/Client/RE/

# 5. A string seen in the client -> function
grep -i 'disconnect' Docs/Client/RE/16-string-anchors.tsv | cut -f2-6 | head

# 6. All functions of one class
awk -F'\t' '$4=="MoveListMaker"' Docs/Client/RE/functions-*.tsv | cut -f1,5

# 7. Open items of one chapter
awk -F'\t' '$2=="06-movement.md"' Docs/Client/RE/17-open-items.tsv | cut -f1,6,7
```

`functions-*.tsv` columns: `address`, `size`, `name`, `class`, `method`, `source`, `confidence`, `area`, `anchor`. All TSV files have a header row and tab separators.

### 2.3 Jump in IDA

Open the 381 database (`ida/lanes/cx/Client.exe.i64`), then in the IDA Python console:

```python
import idc
idc.jumpto(0xcc4ce0)            # MoveListMaker::sendWalkPacket
# Jump to every address of a TSV column (example: handlers of game commands 60-70)
import csv
for r in csv.DictReader(open("Docs/Client/RE/game-commands.tsv"), delimiter="\t"):
    if 60 <= int(r["id"]) <= 70 and r["handler_381"].startswith("0x"):
        print(r["id"], r["name"], r["handler_381"])
```

Find by string: search the text in [`16-string-anchors.tsv`](16-string-anchors.tsv), then `idc.jumpto(<func_addr_381>)`. If the text is not there, in IDA use Search → Text, or `ida_bytes.find_bytes` on the UTF-16 form (the client uses UTF-16 for most UI and log text).

Find by command id: game packet objects have a create function and a vtable per command ([05 §3](05-game-dispatch.md)). Take `create_381` or `vtable_381` from [`game-commands.tsv`](game-commands.tsv) and list the xrefs (`idautils.XrefsTo`).

### 2.4 Rules

- Names come from the 419 GM PDB by matching ([00 §3](00-inventory.md)). A name for a feature with no 381 strings can be false (example: the vehicle names, [17 §4.5](17-open-items.md)). Check the callers before you trust a name.
- Game command ids: use the 381 ids. GM ids above 918 are other commands ([17 §4.1](17-open-items.md)).
- No keys, passwords or certificates are in these files. Where a chapter needs one, it writes `keyhere`.

## 3. A–Z keyword index

The "381" column gives the first address of the quick-find row; "§" gives the chapter number (links in §1) and the section. Rows without an address are types or flows.

### #

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `(no name) Inventory event handler` | Item change packet handler | `0xed3820` | 09 §2.1 |
| `1332 packet decoder` |  | `0xef1430` | 09 §2.1 |

### A

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `Accept or reject a party invite` |  | `0xc14a70` | 11 §1.2 |
| `Aes128::Encrypt / Decrypt` | AES-128 | `0x611750` | 02 §3.2 |
| `Any class, its vtable and constructor` |  | — | 00 §2 |
| `Any function by name or address` |  | — | 00 §4 |
| `Apply a pair to a creature` |  | `0x1049760` | 08 §3.5 |
| `Apply mouse button options` |  | `0x1121e80` | 14 §3.1 |
| `ApplyTestOptimizeOption` | Per-level particle/LOD | `0x1120370` | 15 §2 |
| `Arena lobby object` |  | `0x123ade0` | 12 §2.3 |
| `Arena world object` |  | `0x123a180` | 12 §2.3 |
| `AuctionHouse::ProcessNetObjectEvent` | Market packets | `0x1201b20` | 11 §6 |
| `AudioDeviceComboBox::RefreshAudioDeviceList` | Audio device list | `0xbf21d0` | 15 §3 |
| `AuthPacketProcessor::Encrypt / Decrypt` | Encrypt / decrypt (LobbyGate, InfoGate) | `0x6088e0` | 02 §3.3 |

### B

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `BAccount::ProcessLobbyNetObjectEvent` | Account / goods receive switch | `0x128a7f0` | 04 §2.4 |
| `BControlledPlayingActions::ProcessNetObjectEvent` | Skill fail / recycle replies | `0x10fd7f0` | 07 §4 |
| `BControlledPlayingActions::processSkill` | Own skill result (success/cancel) | `0x111d2d0` | 07 §2.3 |
| `BControlledPlayingActions::RequestEquipPacket` | Equip | `0x10d4070` | 09 §2.3 |
| `BControlledPlayingActions::RequestRespawn` | Respawn request | `0x10d3ad0` | 08 §6.2 |
| `BControlledPlayingActions::RequestUnequipPacket` | Unequip | `0x10d3ec0` | 09 §2.3 |
| `BControlledPlayingActions::UseSkill` | Where the client starts a skill | `0x1114f50` | 07 §1.1 |
| `BNSClient+0x34` | InfoGate session pointer | `0x1a4c0f8` | 12 §1.1 |
| `BNSClient::Cleanup` | Shutdown | `0x116dc90` | 01 §3.6 |
| `BNSClient::DoCommandLine` | Where the command line is parsed | `0x113ed60` | 01 §1 |
| `BNSClient::EnterGameForNPWithLobbyGate` | Connect to GameD with the lobby token | `0x1157fbd` | 04 §3.5 |
| `BNSClient::GetDataDirectoryPath` | Data folder path | `0x1129d30` | 01 §2.2 |
| `BNSClient::GetDataDirectoryPath` | Data directory | `0x1129d45` | 15 §1.9 |
| `BNSClient::GetDisconnectWSALastError` | WSA error shown on the panel | `0x520c60` | 16 §1.5 |
| `BNSClient::GetUi` | Root UI object | `0x401600` | 14 §1.3 |
| `BNSClient::Initialize` | Client start, loads the XML reader DLL | `0x116f550` | 00 §1.5, 01-startup |
| `BNSClient::IsFullClient` | Partial client check | — | 15 §1.8 |
| `BNSClient::isInArena` | GameD or ArenaD? | `0x401690` | 05 §1.2 |
| `BNSClient::LoadLocaleInfo` | Where `Local.ini` is read | `0x11297c0` | 01 §2.1 |
| `BNSClient::LoadSystemText` | Where `SystemText.ini` is read | `0x114cbe0` | 01 §2.2 |
| `BNSClient::MainFramework` | Main start sequence | `0x11904c0` | 01 §3.3 |
| `BNSClient::SaveLocaleInfo` | Where `Local.ini` is written | `0x11313b0` | 01 §2.1 |
| `BNSClient::SendToGamed` | Where the client sends a game packet | `0x114ce20` | 05 §1.4 |
| `BNSClient::SendToInfoGate` | Where the client sends an InfoGate packet | `0x114cdb0` | 12 §1.4 |
| `BNSClient::Tick` | Per-frame client tick | `0x1167630` | 01 §3.5 |
| `BNSClient object pointer` |  | `0x1a4c0f8` | 01 §3.4 |
| `BParty::InvokePartyErrorMessage` | Party error code → text | `0xc4b380` | 14 §2.4 |
| `BParty::ProcessNetObjectEvent` | Where the client applies party packets | `0xc4bac0` | 11 §1.3 |
| `BParty::RequestChangePickupOption` | Change loot (pickup) rule | `0xc14000` | 11 §1.4 |
| `BParty::RequestInvite / RequestInviteByName` | Invite a player to the party | `0xc14d60` | 11 §1.2 |
| `BParty::RequestLeave` | Leave the party | `0xc147b0` | 11 §1.2 |
| `BParty::Tick` | Party timers (logout check, member sync) | `0xc1cb10` | 11 §1.5 |
| `BPlayer::InvokeFailMessage` | Item / grocery error code → text | `0xb5b840` | 14 §2.4 |
| `BPTCreature::invokeBattleMessage` | Battle message ("X hits Y") | `0xb319d0` | 07 §3.3 |
| `BPTCreature::presentationActions` | Damage numbers, effect show | `0xb1a590` | 07 §3.3 |
| `BPTNpc::updatePublicRaidFieldCullDistance` | Raid NPC cull distance | `0xacf8b0` | 15 §2 |
| `BPTPlayer::BPTPlayer / BPTPlayer::Tick (vtable 0x16187a0)` | Player object (constructor, tick) | `0xae3140` | 00 08-creature |
| `BPTPlayer::ProcessNetObjectEvent case 73` | Handle `G_ChangeMoveKeyCmd` 73 | `0xb15dd0` | 06 §3.4 |
| `BPTPlayer::RequestCombatHyperMove` | Combat sprint request (1053) | `0xaf2510` | 06 §4.2 |
| `BPTPlayer::RequestSetTarget` | Select a target | `0xaf30a0` | 07 §6 |
| `BPTPlayer::SyncRuleMoveLocation` | Sync after a rule move | `0xaaa110` | 06 §3.6 |
| `BPTPlayerSummoned::Recall` | Cat recall request (cmd 71) | `0xa64640` | 08 §5.4 |
| `BQuestJournal::BQuestJournal (called from BPlayer::BPlayer 0xb8a73c)` | Create the quest journal | `0xa423f0` | 10 §1.1 |
| `BQuestJournal::CanAcquireQuest / CanCompleteQuest` | Can the player accept / complete | `0xa518b0` | 10 §1.6 |
| `BQuestJournal::completeQuest` | Complete a quest (1463) | `0xa9e320` | 10 §1.4 |
| `BQuestJournal::ProcessNetObjectEvent` | Where the client applies quest packets | `0xa9e5b0` | 10 §1.4 |
| `BQuestJournal::RequestAcquireOpenQuest` | Where the client accepts a quest | `0xa5c970` | 10 §1.3 |
| `BQuestJournal::RequestDisposeOpenQuest` | Abandon (dispose) a quest | `0xa5c720` | 10 §1.3 |
| `BQuestJournal::RequestProgressMission` | Where the client sends mission progress | `0xa684a0` | 10 §1.3 |
| `BQuestJournal::resetQuest` | Remove a quest (1464) | `0xa9d9b0` | 10 §1.4 |
| `BQuestJournal::setQuest` | Add a quest (1460) | `0xa98d90` | 10 §1.4 |
| `BQuestJournal::updateQuest` | Change a quest (1461) | `0xa98520` | 10 §1.4 |
| `BSeason::RequestRetireveCurGuildRanker` | Guild member ranking request | `0x9d1240` | 12 §1.7 |
| `BTeam::InvokeTeamErrorMessage` | Team error code → text | `0x8a5710` | 14 §2.4 |
| `BTeam::ProcessNetObjectEvent` | Raid (team) packets | `0x8f9c90` | 11 §1.6 |
| `Build a request` |  | `0xbce0f0` | 03 §2.2 |
| `builder` | Cat despawn when too far (cmd 636) | `0xb78b00` | 08 §5.4 |
| `BUIProxy::getMainSceneName` | Main scene name per UI type | `0x7de210` | 14 §1.3 |
| `BUIProxy::getPackageName` | UE package id per UI type | `0x7de270` | 14 §1.3 |
| `BUIProxy::LoadByPackageName` | Load a UI package / scene | `0x7f3010` | 14 §1.3 |
| `BUiWorld::RequestEnterZone` | Tell GameD map is ready | `0x7815c0` | 15 §1.7 |
| `BUiWorld::StartNpcTalk → UiStateTalk::StartTalk` | Start NPC talk | `0x7d9980` | 10 §2 |
| `Byte example` |  | — | 02 §7 |
| `Byte examples` |  | — | 07 §8 |

### C

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `ChannelChattingClient::PacketHandler` | Chat server packet switch | `0x5eaff0` | 11 §2.2 |
| `ChatChannel::requestLoginSignature (387)` | Chat server login signature | `0x11564e0` | 11 §2.2 |
| `ChatChannel::SendChatMessageReq` | Where the client sends a chat line | `0x116f260` | 11 §2.1 |
| `ChatInputPanel::findUserCommandType` | Find the command word in the table | `0x116d580` | 14 §3.2 |
| `ChatInputPanel::parseUserCommand` | Split a typed command | `0x116e8f0` | 14 §3.2 |
| `ChatInputPanel::processChatFunctionWorldByEnterKey` | Chat enter key (world) | `0x118ce60` | 14 §3.2 |
| `ChatInputPanel::processDiceUserCommand` | `/dice` command | `0x1150260` | 14 §3.3 |
| `ChatInputPanel::sendChatMessage / sendChatMessageWorld` | Send typed chat | `0x1179170` | 14 §3.2, 11-social  |
| `clear` | Reset session data | `0xc5f870` | 04 §1.1 |
| `ClientExecutionLog::SendPostArguments` | Execution log POST | `0x1167300` | 16 §2.4 |
| `ClientGameGuard::GetGameGuardInitializeError` | Error code → GameGuard result (4047) | `0x114b560` | 01 §3.7 |
| `ClientLog::WriteV → ClientLog::Flush` | Client log line → GameD | `0x10dcd50` | 16 §2.3 |
| `ClientSystemText` | Start error text (`SystemText.ini`) | — | 14 §2.6 |
| `ClientSystemText::SetAdditionalMsg` | Start error text (set) | `0x10a0110` | 01 §3.7 |
| `ClientUserLog::SendPostArguments` | User log POST | `0x10e8df0` | 16 §2.4 |
| `ClosetGroup::MoveClosetToInventoryReq` | Take from wardrobe | `0x10db6a0` | 09 §5 |
| `ClosetGroup::MoveInventoryToClostReq` | Put into wardrobe | `0x10dbb10` | 09 §5 |
| `ComposeObjectId` | Make an object id from zone and index | `0x71f570` | 08 §1.1 |
| `Connection::connect` | Open a socket | `0x5cf400` | 02 §4.3 |
| `Contents rule list request / reply` |  | `0xc967d0` | 04 §1.4 |
| `Coverage audit` | Large classes with no chapter | — | 17 §2 |
| `Crash filter` |  | `0x119104f` | 16 §2.6 |
| `CreateSoundEngineHandle` | Sound handle | `0xf7bff0` | 15 §3 |
| `CreateTclsProxy` | Tencent TCLS proxy load | `0x54ca60` | 00 §1.5 |
| `CreatureProperty::FIELD_INFO` | Field list of CreatureProperty | `0x19f6b48` | 08 §4 |

### D

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `data` | Handler table (index = cmd) | `0x14a1830` | 04 §2.1 |
| `Datafile book` | Datafile chapters | — | 13 — |
| `Delete / cancel / recover` |  | `0xc973c0` | 04 §3.4 |
| `DeliveryBox::RequestDeliverPost` | Send a mail | `0x106d530` | 11 §5 |
| `DeliveryBox::RequestRetrievePost` | Read a mail | `0x1060290` | 11 §5 |
| `dispatch` | Dispatch received packets | `0x5d0080` | 02 §4.4 |
| `DrElReaderImpl::readNative` | Datafile native string read | `0x61cb90` | 00 13-datafile |

### E

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `EInterfaceGetInstance (import pointer)` | Get engine interface | `0x1a4be5c` | 15 §1.3 |
| `Engine exports` |  | `0x1a4be5c` | 01 §3.2 |
| `EngineUnitToServerUnit` | Convert engine units to server units | `0x4014b0` | 06 §1.5 |
| `Enter-world token reply` |  | `0xc71970` | 04 §3.5 |
| `Event entry slot (calls Transmit)` |  | `0x7f3220` | 14 §1.5 |
| `EventSubLevel::set` | Sub-level streaming | `0xf95091` | 15 §1.8 |
| `EventTransmitter<…>::AddReceiver` | Add a widget receiver | `0x55fca0` | 14 §1.5 |
| `EventTransmitter<…EUiEvent…>::Transmit` | Engine → client widget event | `0x55fbe0` | 14 §1.5 |
| `ExpandInvenSlotConfirmPanel::ProgressComplete` | Expand inventory | `0xfdb7e0` | 09 §1.2 |
| `Export names that LoadEngine resolves` |  | `0x14fe05c` | 14 §1.2 |

### F

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `Fail reason names` |  | `0xaa347c` | 06 §3.5 |
| `Field id of "hp", "level", "dead"` |  | — | 08 §4.2 |
| `Field wire encoding (write)` |  | `0x5d52c0` | 05 §3.3 |
| `Find the move list layout` |  | — | 06 §1.1 |
| `Find the network object` |  | `0x1a4da00` | 02 §1.1 |
| `Fishing` | PTFishing, Fishing | `0xabab80` | 17 §4.6 |
| `Fishing::StopAutomaticFishing` | Auto fishing stop | `0xf7b2c0` | 00 10-quests-npc ❓ |
| `Flush with a reason` |  | `0xcc55b0` | 06 §2.2 |
| `Friends::ProcessNPEvent` | Friend NP replies | `0x100a3f0` | 11 §4 |

### G

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `G_HelloReq` | Byte example | — | 05 §5 |
| `G_UseSkillReqPacket::G_UseSkillReqPacket` | `G_UseSkillReq` packet constructor | `0x449fd0` | 07 §1.4 |
| `Game token` |  | `0xc063c0` | 03 §3.5 |
| `GameMessageHandler::ProcessNetObjectEvent (vtable 0x1574230)` | Central net → message handler | `0xfe7300` | 14 §2.3 |
| `GamePacket_define[N]` | Field layout of command N | `0x1496f40` | 05 §3 |
| `GamePacketHandler[N]` | Handler of command N | `0x149ec40` | 05 §2 |
| `GamePacketProcessor::Encrypt / Decrypt` | Packet encryption entry (no key data here) | `0x608f10` | 00 02-transport |
| `GamePacketProcessor::SetSessionKey` | Session key set | `0x608d10` | 00 02-transport |
| `GameSession::GameSession (vtable 0x15896b8)` | Game session object | `0xefbf40` | 00 05-game-dispatch |
| `GameSession::OnClose` | Disconnect reason → UI event | `0xefbd80` | 05 §1.6 |
| `GameSession::OnConnect` | Hello to GameD / ArenaD | `0xf2c0c0` | 05 §1.5 |
| `GameSession::OnError` | Where GameD disconnect reason picks the qu… | `0xf44150` | 16 §1.4 |
| `GameSession::onRecv` | Where the client gets a game packet | `0xf44030` | 05 §1.3 |
| `GameSession::onStreamError` | Bad stream hex dump | `0xf4b550` | 05 §1.6 |
| `GameSession::PropertyPairFieldInfo` | Pair table id → field table | `0xee1170` | 08 §3.3 |
| `GameSession::PropertyPairFieldMax` | Pair table id → field count + 1 | `0xee11a0` | 08 §3.3 |
| `GameSession::SendPing` | Game ping send | `0x47d7a0` | 00 05-game-dispatch |
| `getLocalOptionFilePath` | User option file path | `0x1130100` | 01 §2.1 |
| `GetPacketSyncEnumString` | Sync group names | `0x867870` | 05 §4 |
| `Glide::BlockGlide` | Block glide | `0xee0640` | 06 §4.3 |
| `GoodsInventory::requestTakeWarehouseGoodsItemInQueue` | Goods warehouse take | `0xf39550` | 09 §6 |
| `GS_Actions_define` | Action variant table | `0x1496720` | 07 §2.1 |
| `GS_Effects_define` | Effect variant table | `0x14962e8` | 07 §3.1 |
| `GS_PassiveEffectInfo_define` | Buff/debuff variant table | `0x14964a0` | 07 §3.2 |
| `GS_SkillTarget_define` | Skill target variant table | `0x1496428` | 07 §2.4 |
| `Guild::ProcessInfoGateNetObjectEvent` | Guild InfoGate replies | `0xf3df60` | 11 §3.2 |
| `Guild::ProcessNetObjectEvent` | Guild packets | `0xf6de30` | 11 §3 |

### H

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `Handle cmd 2 (encounter / out of sight)` |  | `0x73f710` | 08 §2.1 |
| `Handle cmd 3 (zone snapshot)` |  | `0x74c0c0` | 08 §2.1 |
| `Handle cmd 4 (in sight)` |  | `0x740cd0` | 08 §2.1 |
| `HandleHelloOkPacket` | Hello reply | `0xf70280` | 05 §1.5 |
| `HandleHelloOkPacket → GameSession::SendEnterArenaReq` | Hello OK on ArenaD → enter arena | `0xf70280` | 12 §2.2 |
| `Hello to InfoGate` |  | `0xe82cb0` | 12 §1.3 |
| `HTTPClient::connect` | WinINet wrapper | `0xea2148` | 16 §2.5 |
| `HyperMove::Block` | Block sprint | `0xe9d1c0` | 06 §4.1 |
| `HyperMove::Start / End` | Start / end sprint | `0xe93fc0` | 06 §4.1 |
| `HyperMove::Tick` | Sprint, drift, boost tick | `0xe82b90` | 00 06-movement |

### I

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `id >> 48 (switch in World::enterCreature)` | Object type of an id | `0x74b6cd` | 08 §1.1 |
| `InfoGate session object` |  | `0xe36920` | 12 §1.1 |
| `InfoGatePacket_define[N]` | Field layout of InfoGate command N | `0x14a0730` | 12 §1.4, TSV |
| `InfoGatePacketHandler[N]` | Handler of InfoGate command N | `0x14a0cf8` | 12 §1.4 |
| `InfoGateSession::OnError` | InfoGate connect error log | `0xea11f0` | 12 §1.5 |
| `InfoGateSession::onRecv` | Where the client gets an InfoGate packet | `0xe66070` | 12 §1.4 |
| `InfoGateSession::ReconnectInfoGate` | Info server reconnect | `0x1131520` | 00 12-infogate-aren |
| `InfoGateSession::SendPing → sendPingRequest` | InfoGate ping | `0x4a5ef0` | 12 §1.5 |
| `InGameWeb::GetDomainUrl` | In-game web URLs | `0xe31ae0` | 12 §3.1 |
| `IntensionPanel::RequestPacket` | Enchant | `0xe9c170` | 09 §3.4 |
| `Inventory::GetItem` | Get item at depot/pos | `0xe3dd40` | 09 §1.4 |
| `Inventory::RequestAttachGem` | Attach gem | `0xe91a30` | 09 §3.3 |
| `Inventory::RequestDisposeItem` | Dispose item | `0xe91c50` | 09 §2.2 |
| `Inventory::RequestMoveItemPosition` | Move item | `0xe91e00` | 09 §2.2 |
| `Inventory::requestRearrange` | Sort / stack (move list) | `0xea9d10` | 09 §2.4 |
| `Inventory::RequestUseGroceryItem` | Use grocery | `0xeda780` | 09 §3.1 |
| `Is this publisher + language pair valid` |  | `0x5d05b0` | 01 §1.4 |
| `IsRequestedSkillAnswered` | "One skill request in flight" lock | `0x10b0090` | 07 §1.5 |
| `isSkillAvailableToUse` | Client-side skill check | `0x10e9140` | 07 §1.3 |
| `ItemStorePanel::RequestBuyItem` | Buy at NPC | `0xd23960` | 09 §4 |
| `ItemStorePanel::RequestSellItem` | Sell at NPC | `0xd23270` | 09 §4 |

### J

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `Jobs (CreatureProperty::Job)` | Job ids 1–20 | — | 17 §4.2 |

### K

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `Keepalive ping (30 s)` |  | `0x5d28f0` | 02 §4.2 |
| `Keepalive ping (cmd 9)` |  | `0x4d61a0` | 04 §1.7 |

### L

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `L_ERROR_CODE_TO_STRING` | Error code → name | `0x712bc0` | 07 §4.2 |
| `L_ERROR_CODE_TO_STRING (table 0x194fc30)` | Error code → name text | `0x712bc0` | 16 §1.2 |
| `Language name → enum` |  | `0x5d04d0` | 01 §1.3 |
| `LoadEngine` | Where the engine DLL loads | `0x114c6d0` | 00 §1.5, 15-engine |
| `LobbyGateSession::LobbyGateSession (vtable 0x15e9a80)` | Lobby session object | `0xc673e0` | 00 04-lobbygate |
| `LobbyGateSession::ProcessLobbyNetObjectEvent` | Main receive switch | `0xcb86d0` | 04 §2.3 |
| `LogFile ctor (IDA name PvpResultDeadProperty::PvpResultDeadProperty...` | Generic log file object | `0x5d0da0` | 16 §2.2 |
| `Login with NP token / lobby token` |  | `0xc97810` | 04 §1.5 |
| `LogManager::CreateLogFile` | Client log file | `0x5e6630` | 00 16-strings-error |

### M

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `Map engine moves to wire move types` |  | `0x15e97b4` | 06 §1.4 |
| `Map errors to reasons` |  | `0x5d2d80` | 02 §5.1 |
| `Market search` |  | `0x11db510` | 11 §6 |
| `MixerLib::Control::SetDetails` | Windows mixer | `0x6005f0` | 15 §3 |
| `Mouse button command names` |  | `0x1955858` | 14 §3.1 |
| `MoveListMaker::checkRotate` | Send a turn (`G_TurnReqCmd` 66) | `0xc5b3e0` | 06 §2.4 |
| `MoveListMaker::makeCell / SetArea` | Make cells | `0xc74470` | 06 §2.3 |
| `MoveListMaker::sendWalkPacket` | Where the client sends a move | `0xcc4ce0` | 00 06-movement |
| `MoveListMaker::SyncServerMove / setPlayerMove` | Reset the move list to a position | `0xc5b890` | 06 §3.3 |
| `MoveListMaker::Tick` | Move list tick | `0xcc93d0` | 00 06-movement |
| `MoveListMaker::Tick / playerTick` | Find the per-tick move recorder | `0xcc93d0` | 06 §2.2 |

### N

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `Name of command N` |  | `0x1953620` | 05 §2 |
| `Network thread loop` |  | `0x5d5ab0` | 02 §4.2 |
| `Network::Disconnect` | Disconnect | `0x5d33c0` | 02 §5.1 |
| `Network::Send` | Send a packet | `0x5d2740` | 02 §4.4 |
| `NPAuction::RequestBid` | NP auction bid | `0xc98750` | 11 §6.3 |
| `NPSession::OnConnect` | Start STS after connect | `0xc22150` | 03 §3.1 |
| `NPSession::OnError` | Link error | `0xc223a0` | 03 §6 |
| `NPSession::OnRecv` | Route by message type | `0xc253d0` | 03 §2.3 |
| `NPSession::ReceiveLoginFailure` | Login failure → UI | `0xc21c10` | 03 §5 |
| `NPSession::ReceiveLoginFinishSuccess` | Login success | `0xc24d80` | 03 §3.3 |
| `NPSession::RecordPacket` | Remember txn → command | `0xbdd680` | 03 §2.2 |
| `NPSession::SendLoginRequest` | Start the login | `0xbd40c0` | 03 §3.1 |
| `NPSession::SendLoginStartMessage` | Send LoginStart | `0xc076d0` | 03 §3.2 |
| `NPSession::SendLogoutRequest` | Logout | `0xc075b0` | 03 §7.3 |
| `NPSession::SendTokenRequest` | Request a token | `0xc24710` | 03 §3.4 |
| `NPSession::Tick` | Reconnect timers | `0xc24c50` | 03 §6 |
| `NPStsMsgLib::Create` | Where the STS client DLL loads | `0xbc90a0` | 00 03-sts-client |
| `NPStsMsgLib::GetTransactionId / global` | Next transaction id | `0xbc9050` | 03 §2.2 |

### O

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `OnClose (vt slot 8)` | Disconnect reason to message | `0xc67290` | 04 §1.6 |
| `OnConnect (vt slot 6)` | What happens on connect | `0xc979f0` | 04 §1.2 |
| `OnError (vt slot 4)` | Network error | `0xca7080` | 04 §1.6 |
| `onRecv` | Where the client receives a LobbyGate packet | `0xc88240` | 04 §2.1 |
| `onResponse` | Hello / auth reply switch | `0xea4bd0` | 12 §1.3 |
| `Open items (all ❓)` | One list with next actions | — | 17 §3 |
| `OutputSkillErrorGameMessage` | Cast-fail message to the player | `0x10b90d0` | 07 §1.3 |

### P

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `Packet Def table (fields per cmd)` |  | `0x14a0f98` | 04 §2.2 |
| `PacketParser::ParseData` | Byte stream to packets | `0x5fd770` | 00 02-transport |
| `PacketProcessor::Create` | Make the processor | `0x6076b0` | 02 §3.1 |
| `PacketSession::setField` | Field wire encoding (read) | `0x5d4f20` | 05 §3.3 |
| `Parse a message` |  | `0xbd41d0` | 03 §2.3 |
| `PartyAuction::ProcessNetObjectEvent` | Party auction (loot bid) | `0xc1c850` | 11 §1.4 |
| `Ping` |  | `0x4e25a0` | 03 §7.1 |
| `PPUtil::EncryptHandshake` | Handshake packet | `0x60fb10` | 02 §3.4 |
| `PPUtil::PacketSize` | Envelope length | `0x60f8a0` | 02 §2.1 |
| `PPUtil::SetupSessionCrypto` | Session key from DH | `0x60f8f0` | 02 §3.4 |
| `PresentationWorld::encounterCreature` | Show a creature (3D) | `0xb53270` | 08 §2.4 |
| `ProcessInfoGateNetObjectEvent` | Reply fan-out to game objects | `0xea4c90` | 12 §1.6 |
| `ProcessSystemLoginFail` | Login fail to UI | `0xc72b30` | 04 §1.5 |
| `Property::SetSync` | Copy a sync into a property | `0x5d3d70` | 08 §3.4 |
| `PropertySync::Set` | Read a `T_PROP_SYNC` field | `0x5d4600` | 08 §3.2 |
| `Publisher / language globals` |  | `0x1955c38` | 01 §1.2 |
| `Publisher name → enum` |  | `0x5d0460` | 01 §1.3 |

### Q

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `Quest failure message keys` |  | `0xa9e9dc` | 10 §1.5 |
| `QuitGameConfirmPanel::endProgress` | `end.log` at exit | `0xa15f20` | 16 §2.1 |
| `QuitGameConfirmPanel::Open` | Quit panel text per state | `0xa1a840` | 16 §1.5 |

### R

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `Read a T_PROP_PAIR field` |  | `0x5d4810` | 08 §3.3 |
| `Read one pair value` |  | `0x5d4410` | 08 §3.3 |
| `Receive bytes` |  | `0xbce1e0` | 03 §2.3 |
| `receiveAuthTokenOk` | Auth token OK (ready) | `0xea4af0` | 12 §1.3 |
| `ReceiveHelloOk` | Handshake reply | `0xcb8660` | 04 §1.3 |
| `ReconnectInfoGate (from BNSClient::Tick)` | Reconnect | `0xe54130` | 12 §1.5 |
| `recvNext` | Receive and cut packets | `0x5cfb70` | 02 §4.4 |
| `Region (lobby address set 1 or 2)` |  | `0x1955c40` | 01 §1.2 |
| `Request a connect` |  | `0x5d2680` | 02 §4.1 |
| `RequestCancelAction` | Cancel a skill | `0x10d3ba0` | 07 §1.6 |
| `RequestTransformMultipleFeedItem` | Transform | `0xe8a490` | 09 §3.2 |
| `Resolve a host name` |  | `0x5d25d0` | 02 §4.1 |
| `Route a reply` |  | `0xc25140` | 03 §2.4 |
| `Rule-config option table` |  | `0x19dde64` | 16 §2.7 |

### S

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `Season ranking request` |  | `0x9d1360` | 12 §1.7 |
| `Select character (enter-world token)` |  | `0xc970a0` | 04 §3.5 |
| `Send KeyData / LoginFinish` |  | `0xc074b0` | 03 §3.2 |
| `SendAuthTokenReq` | Auth token to InfoGate | `0xe82b90` | 12 §1.3 |
| `SendCharacterListRequest` | Character list request | `0xc976a0` | 04 §3.2 |
| `SendMakeCharacterRequest` | Create character | `0xc97480` | 04 §3.3 |
| `SendWorldListRequest` | World list request | `0xc97710` | 04 §3.1 |
| `Server kick (3201)` |  | `0xc08430` | 03 §7.2 |
| `Server ping reply` |  | `0xf4b430` | 05 §1.5 |
| `Server-driven dash, knockback, mount` |  | `0xaaa440` | 06 §3.6 |
| `Session key from launcher` |  | `0x1a823d0` | 01 §1.2 |
| `Session::IsCompleted` | Fragments | `0x5d4a40` | 02 §2.3 |
| `Session::PacketSize` | Plain packet length | `0x5d4a20` | 02 §2.2 |
| `Set or clear one slot` |  | `0xe616a0` | 09 §1.4 |
| `Set quit state + codes` |  | `0x1129a90` | 16 §1.4 |
| `SetSessionKey` | Session key rotation | `0x608d10` | 02 §3.1 |
| `SetWorldInfo (vslot 81)` | Snap the player to the server position | `0xafb560` | 06 §3.2 |
| `SkillTimer::SetGlobalCoolTime_renewal` | Global cool-down | `0x916030` | 07 §5 |
| `SkillTimer::SetSkillRecycleTime_renewal` | Recycle (cool-down) group | `0x974fc0` | 07 §5 |
| `SkillTracer::Report` | Skill trace file | `0x96bcf0` | 16 §2.1 |
| `Slot → processor kind, max size, timeout` |  | `0x5d3030` | 02 §1.1 |
| `Sprint energy drain and refill` |  | `0xe94110` | 06 §4.1 |
| `start` | Program entry (loader stub, not game code) | `0x703cfa` | 00 §1.5 |
| `Start engine main loop` |  | `0x1190edc` | 15 §1.4 |
| `Start error text (show at exit)` |  | `0x114caf0` | 01 §3.6 |

### T

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `table` | Client option definitions (2,774) | `0x1955c58` | 14 §1.8 |
| `TaskSession::InitializeLatencyLogModule` | Latency log file | `0x876ed0` | 16 §2.1 |
| `TaskSession::IsLocked / Sent / Received` | "Request still pending" lock | `0x86ed10` | 05 §4 |
| `Text language code → enum` |  | `0x5d0540` | 01 §1.3 |
| `Token reply` |  | `0xc24340` | 03 §3.4 |
| `ToPacket` | Bytes → struct | `0x5d4c10` | 02 §2.4 |
| `ToStream` | Struct → bytes | `0x5d4d20` | 02 §2.4 |
| `TracableProperty<CreatureProperty>::SetPair` | Apply changes and call listeners | `0x465d40` | 08 §3.5 |

### U

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `Ui::ApplyEngineConfiguration` | Apply options to engine | `0x82bbe0` | 15 §2 |
| `Ui::Initialize` | UI start | `0x8135e0` | 14 §1.3 |
| `UIContextCommandHandler::ProcessUICommand` | Context menu commands | `0x83c780` | 14 §1.7 |
| `UIEngineInterfaceGetInstance (function pointer)` | Where the client gets the UI interface | `0x1a4c0cc` | 14 §1.2 |
| `UIPanel::GetWidgetId` | Widget name → id | `0x7fbe40` | 14 §1.4 |
| `UIPanel::IsVisible` | Panel visible? | `0x7e3410` | 14 §1.4 |
| `UIPanel::ShowPanel` | Show or hide a panel | `0x804830` | 14 §1.4 |
| `UIPanel::UIPanel (vtable 0x1685418)` | Panel base class | `0x7fbfe0` | 14 §1.4 |
| `UiStateTalk::EndTalk` | End NPC talk (cmd 61) | `0x8443f0` | 10 §2 |
| `UiTradeManager::ProcessNetObjectEvent` | Player trade packets | `0x7bf040` | 11 §7 |
| `UiTransitControl::loadZoneMap` | Zone id → umap name | `0x78fed0` | 15 §1.6 |
| `UiTransitControl::PostTick` | Zone load state machine | `0x7d9680` | 15 §1.5 |
| `UiTransitControl::ProcessEvent` | Engine event to transit | `0x7c1950` | 15 §1.5 |
| `UiTransitControl::SetTransitState` | Set transit state | `0x754610` | 15 §1.5 |
| `UIUtil::GetDefaultUICommand` | Default command for an icon | `0x762110` | 14 §1.7 |

### V

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `Vehicle (none in 381)` | False vehicle names, mount links | `0xac32c0` | 17 §4.5 |

### W

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `walkFail` | Handle `G_MoveFailCmd` 65 | `0xaa33a0` | 06 §3.5 |
| `Weapon types` | ItemWeaponTypeType 1–18 | — | 17 §4.2 |
| `WebRequester::RequestUri` | HTTP request thread | `0x7373c0` | 12 §3.2 |
| `Where GameD lost picks the panel` |  | `0x1157db0` | 16 §1.4 |
| `Where LobbyGate / NP lost picks the panel` |  | `0x1157e30` | 16 §1.4 |
| `Where the client gets the InfoGate address` |  | `0xcc3fa0` | 12 §1.2 |
| `Where the client sends a skill` |  | `0x10d26e0` | 07 §1.4 |
| `Where the client sends Hello` |  | `0xc978b0` | 04 §1.2 |
| `Where the server address file loads` |  | `0x10a9de0` | 01 §2.4 |
| `WinMain` | Where the game starts | `0x1190f60` | 01 §3.1 |
| `_WinMain@16` | Main function | `0x1190f60` | 00 01-startup |
| `Worked example` |  | — | 08 §7 |
| `World::AddInstantNotification` | Push a line to the message history | `0x746430` | 14 §2.2 |
| `World::createPlayer` | Create the own player | `0x737d30` | 08 §1.2 |
| `World::createPlayerSummoned` | Create the own cat | `0x7325b0` | 08 §5.2 |
| `World::enterCreature` | Create a creature (spawn) | `0x74b640` | 08 §2.2 |
| `World::EnterLeaveObject` | Handle cmd 1 (enter / leave) | `0x74c370` | 08 §2.1 |
| `World::find` | Find a world object by id | `0x72b670` | 08 §1.3 |
| `World::InvokeGameMessage(wchar_t const *, __int64)` | Show a system message by alias | `0x401ec0` | 14 §2.2 |
| `World::invokeGameMessage(wchar_t const *, …)` | Variadic message (core) | `0x72bae0` | 14 §2.2 |
| `World::InvokeGameMessageForArgs` | Route message by UI type | `0x747980` | 14 §2.2 |
| `World::invokeGameMessageWorld` | Message in the world UI (fields of the rec… | `0x7469b0` | 14 §2.2 |
| `World::leaveCreature` | Remove a creature (despawn) | `0x74b140` | 08 §2.3 |
| `World::OutSightObject` | Handle cmd 5 (out of sight) | `0x73f4d0` | 08 §2.1 |
| `World::processAction` | Server skill/hit/effect stream | `0x748b00` | 07 §2.1 |
| `World::ProcessNetObjectEvent` | Second-level dispatch (world objects, UI) | `0x749550` | 05 §2.2 |
| `World::processSkillActionToOwner` | Skill action of a creature | `0x737a60` | 07 §2.2 |
| `World::SendUserChatMessage` | Chat length / block check | `0x740700` | 11 §2.1 |

### Z

| Keyword | What | 381 | § |
| --- | --- | --- | --- |
| `ZSTD_decompress_usingDict` | ZSTD broadcast | `0x5ced10` | 02 §3.5 |
