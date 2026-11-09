# 11 — Party, guild, chat, mail, market (Client.exe 0.0.381)

1. Party state is one `BParty` object (168 bytes, max 6 members, `G_PREDEFINEDPARTYMEMBERCOUNT_MAX`). Party packets 147–225 go to GameD through `BNSClient::SendToGamed` `0x114ce20`. One handler, `BParty::ProcessNetObjectEvent` `0xc4bac0`, applies them (§1). Raid is `BTeam` (packets 226–275, handler `0x8f9c90`).
2. Normal chat (say, party, guild, whisper, team) is the game packet `MessageReq` 51 with a `MessageRange` (§2.1). Channel chat (district, trade, party search, voice) uses a separate chat server, `ChannelChattingClient`, with its own 6-byte header protocol (§2.2).
3. Guild is the `Guild` object (2,336 bytes, packets 431–623, 1274–1289). Some guild lists go to **InfoGate** (ids 46, 52, 58) (§3). Friends use **NP (STS) messages 3751–3766** and InfoGate 7 (§4).
4. Mail is `DeliveryBox` (packets 368–376, 1500–1511). The market is `AuctionHouse` (packets 378–386, 1530–1556). The NP auction is `NPAuction` (STS). Trade is `UiTradeManager` + `TradeShelf` (packets 311–341) (§5–§7).
5. All lists: `11-social-commands.tsv` (668 game commands), `11-social-infogate.tsv` (InfoGate commands), `11-social-enums-gm.tsv`, `11-social-functions.tsv` (8,414 functions).

```
 UI (PartyPanel, ChatInputPanel, GuildPanel2, DeliveryPanel, AuctionPanel2, TradeShelfPanel)
   v
 BParty::Request*  BTeam::Request*  Guild::Request*  DeliveryBox::Request*  AuctionHouse::Request*  TradeShelf::Request*
   |                                                  ChatChannel::SendChatMessageReq 0x116f260
   v                                                     |-- this+696 != 0 --> sendToChattingServer --> ChannelChattingClient (own TCP)
 BNSClient::SendToGamed 0x114ce20  ----------------------+-- else ----------> G_MessageReq 51
 BNSClient::SendToInfoGate 0x114cdb0  (guild renewal, guild point, friend list, search)
 NPSession::SendMsgRequest (STS)      (Friends 3751-3758, NPAuction)
   v  S2C
 World::ProcessNetObjectEvent 0x749550
   |-> BParty::ProcessNetObjectEvent 0xc4bac0     |-> BTeam::ProcessNetObjectEvent 0x8f9c90
   |-> PartyAuction::ProcessNetObjectEvent 0xc1c850|-> Guild::ProcessNetObjectEvent 0xf6de30
   |-> UiTradeManager::ProcessNetObjectEvent 0x7bf040 / TradeShelf::ProcessNetObjectEvent 0x8091e0
   |-> AuctionHouse::ProcessNetObjectEvent 0x1201b20 / DeliveryBox::receive*
 Guild::ProcessInfoGateNetObjectEvent 0xf3df60, Friends::ProcessInfoGateNetObjectEvent 0x100a180
 Friends::ProcessNPEvent 0x100a3f0, NPAuction::ProcessNPEvent 0xc73590
 ChannelChattingClient::PacketHandler 0x5eaff0 -> ChattingClient2::processMessage 0x1160c60
```

Sources: Client.exe 0.0.381 IDA database (all addresses are 381). Type, member and enum names come from the 381 database types (imported from the 419 GM PDB); enum values are 419 GM. Packet ids, sizes and fields come from `game-commands.tsv` and `packet-layouts.tsv` (chapter 05).

Related chapters: [05 — game dispatch](05-game-dispatch.md) (framing, `SendToGamed`, sync groups, `World::ProcessNetObjectEvent`), [02 — transport](02-transport.md) §6 (`PacketParser` for chat), [03 — STS client](03-sts-client.md) §7.4 (NP presence 3202, 1601), [09 — items](09-items.md) (inventory, item pouch), [10 — quests](10-quests-npc.md) (party match 424–430, raid dungeon). This chapter does not repeat them.

## Files

| File | Content |
| --- | --- |
| `11-social-commands.tsv` | 668 game commands: id, name, topic (party, chat, guild, faction, friend, mail, market, trade, duel), direction, 381 handler, GM handler name, route, 381 builder, struct size, sync group, fields (offset:name:type/size, `(dN)` = nested depth). |
| `11-social-infogate.tsv` | 73 `InfoGateCommand` values (419 GM enum) with topic and the 381 sender found by a scan of `SendToInfoGate` `0x114cdb0` callers. |
| `11-social-enums-gm.tsv` | 506 enums of the area (2,481 rows: enum, member, value). |
| `11-social-functions.tsv` | 8,414 named functions of the area with class, size, topic and first string. |

## Quick find

| Task | Function | 381 address | GM name | Section |
| --- | --- | --- | --- | --- |
| Where the client applies party packets | `BParty::ProcessNetObjectEvent` | `0xc4bac0` | same | §1.3 |
| Invite a player to the party | `BParty::RequestInvite` / `RequestInviteByName` | `0xc14d60` / `0xc14bc0` | same | §1.2 |
| Accept or reject a party invite | `JoinPartyReq` builder / `RejectPartyReq` builder | `0xc14a70` / `0xc148e0` | — ❓ | §1.2 |
| Leave the party | `BParty::RequestLeave` | `0xc147b0` | same | §1.2 |
| Change loot (pickup) rule | `BParty::RequestChangePickupOption` | `0xc14000` | same | §1.4 |
| Party timers (logout check, member sync) | `BParty::Tick` | `0xc1cb10` | same | §1.5 |
| Party auction (loot bid) | `PartyAuction::ProcessNetObjectEvent` | `0xc1c850` | same | §1.4 |
| Raid (team) packets | `BTeam::ProcessNetObjectEvent` | `0x8f9c90` | same | §1.6 |
| Where the client sends a chat line | `ChatChannel::SendChatMessageReq` | `0x116f260` | same | §2.1 |
| Chat length / block check | `World::SendUserChatMessage` | `0x740700` | same | §2.1 |
| Chat server packet switch | `ChannelChattingClient::PacketHandler` | `0x5eaff0` | same | §2.2 |
| Chat server login signature | `ChatChannel::requestLoginSignature` (387) | `0x11564e0` | same | §2.2 |
| Guild packets | `Guild::ProcessNetObjectEvent` | `0xf6de30` | same | §3 |
| Guild InfoGate replies | `Guild::ProcessInfoGateNetObjectEvent` | `0xf3df60` | same | §3.2 |
| Friend NP replies | `Friends::ProcessNPEvent` | `0x100a3f0` | same | §4 |
| Read a mail | `DeliveryBox::RequestRetrievePost` | `0x1060290` | same | §5 |
| Send a mail | `DeliveryBox::RequestDeliverPost` | `0x106d530` | same | §5 |
| Market search | `MarketSearchItemReq` builder | `0x11db510` | — | §6 |
| Market packets | `AuctionHouse::ProcessNetObjectEvent` | `0x1201b20` | same | §6 |
| NP auction bid | `NPAuction::RequestBid` | `0xc98750` | same | §6.3 |
| Player trade packets | `UiTradeManager::ProcessNetObjectEvent` | `0x7bf040` | same | §7 |

Search anchors: `Msg.Party.` (UTF-16 keys, §1.3), `Msg.Team.`, `Msg.Chatting.SendFail.Legnth`, `Msg.Trade.`, `Msg.Friends.RequestProposal.Fail.AreadyFriend`, `ccChannelChat called.`, `ChannelChattingClient::Login`, `[%p]:: KeepAlive started`.

---

## 1. Party and raid

### 1.1 Objects

`BParty` (168 bytes, 419 GM type; constructor `0xbdafd0`, called from `BPlayer::BPlayer` at `0xb8a808`):

| Offset | Member | Notes |
| ---: | --- | --- |
| 8 | `_partyId` (u64) | 0 = no party ❓ |
| 16 | `_memberList` (16) | list of `BMember` (328 bytes each) |
| 32 | `_prevMemberIDList` | `updatePrevMemberIDList` `0xbdaad0` |
| 48 | `_pickupRule` (u8) | `PartyPickupRule` §1.4 |
| 49 | `_auctionItemGrade` (u8) | min. item grade for party auction |
| 56 | `_auctionStartPrice` (i64) | |
| 64 | `_dedicatedMember` (u64) | creature id for rule 2 |
| 72 | `_clearLogoutMemberCheckTimer` | period field at +80 = 1.0 s (`0xbdafd0`) |
| 88 | `_memberPropUpdateTimer` | period field at +96 = 0.5 s (`0xbdafd0`) |
| 104 | `_difficultyType` | `BParty::InviteType`: DEFAULT 0, HARD 1, NORMAL 2, EASY 3 |
| 108 | `_partyInviteAuthType` | `PartyInviteAuth`: LEADER 0, MEMBER 1 |
| 112 | `_invitedPcList` | `addInvitedPc` `0xbda530`, `removeInvitedPc` `0xbd2580`, `checkAndRemoveInvitedPc` `0xbd24e0` |
| 128 / 130 | `_bindedDungeonDataId` / `_bindedDungeonStep` | from `InPartyMember` 189 |
| 136 | `_banishAgreementTimer` | vote kick |
| 154–164 | attraction, fill vacancy, challenge party mode, `_gmNoParty` | |

`BMember` (328 bytes): `_memberKey` +0, `_creatureId` +8, `_property` +16, `_summonedDataId` +116, `_zoneChannel` +120, `_race` +124, `_job` +128, `_inSight` +132, `_login` +133, `_banishAgreement` +134, `_logoutTimer` +152, `_name` +168, `_worldId` +196, aggro boss ids +232…+256, `_newbieCareStae` +326.

### 1.2 Requests (C2S)

All builders call `BNSClient::SendToGamed` `0x114ce20`. Fields: `11-social-commands.tsv`.

| Id | Packet | Builder (381) | Fields (offset:name) | Client check before send |
| --- | --- | --- | --- | --- |
| 147 | `SuggestPartyReq` | `BParty::RequestSuggestJoinByName` `0xc14af0` | 16 targetId u64, 24 targetName str, 32 fromPartySearch u8, 33 difficultyType, 34 level, 35 mastery_level, 36 job, 37 faction2 | `CheckSuggestJoinByName` `0xbef710` |
| 155 | `CreatePartyReq` | `BParty::RequestCreateSoloParty` `0xc149c0` | (no body) | `CheckCreateSoloParty` `0xbdaf70` |
| 157 | `InvitePartyReq` | `RequestInvite` `0xc14d60`, `RequestInviteByName` `0xc14bc0` | 16 targetId, 24 targetName, 32 partyMatchGoal i32, 36 fromSuggestion u8, 37 difficultyType, 38 level, 39 mastery_level, 40 job, 41 faction2 | `CheckInvite` `0xbefc50`; duel → `Msg.Party.InviteFail.Duel`; arena → `…ArenaDungeon` / `…ArenaLobby`. On send OK with `INVITETYPE_DEFAULT` it adds the target to `_invitedPcList`. |
| 161 | `JoinPartyReq` | `0xc14a70` (also `0x5a1020`) | 16 partyId, 24 fromId, 32 partyMatchGoal, 36 difficultyType | called from case 160 after `CheckJoin` `0xbcd3b0` |
| 163 | `RejectPartyReq` | `0xc148e0` (also `0x5a10a0`) | 16 partyId, 24 requester, 32 difficultyType, 33 reason (`InviteRejectReason`) | auto reject: `CheckInvitePartyAutoReject` `0xbef2b0` |
| 168 | `LeavePartyReq` | `BParty::RequestLeave` `0xc147b0` | (none) | challenge dungeon → `Msg.DungeonChallenge.Party.Fail.Leave`; spawn state → `Msg.Party.LeaveFail.PcSpawn` |
| 170 | `DelegatePartyLeaderReq` | `0xc14750` (also `0x5a1220`) | 16 creatureId | `CheckDelegateLeader` `0xbdae70` |
| 172 | `BanishPartyReq` | `BParty::RequestBanish` `0xc142e0` | 16 memberKey u8 | |
| 174 | `ClearLogoutPartyMemberReq` | `RequestClearLogoutMember` `0xc13fb0` | (none) | sent by `Tick` §1.5 |
| 176 | `ChangePartyPickupRuleReq` | `RequestChangePickupOption` `0xc14000`; `RequestChangePickupRule` `0xc14230`, `…AuctionStartPrice` `0xc14120`, `…AuctionItemGrade` `0xc141b0` | 16 pickupRule u8, 17 auctionItemGrade u8, 24 dedicated_member u64, 32 partyAuctionStartPrice i64 | rule 3 (random) only when `GRuleConfig+1697` ≠ 0 |
| 179 | `ChangePartyDifficultyTypeReq` | `RequestPromotion` `0xc14a10` ❓ | see TSV | `CheckPromotion` `0xbef580` |
| 182 | `ChangePartyInviteAuthReq` | `RequestChagnePartyInviteAuth` `0xc14960` | 16 inviteAuth u8 | `CheckChangePartyInviteAuth` `0xbdaf10` |
| 151 | `RejectSuggestPartyReq` | `RequestRejectSuggestJoin` `0xc14860` | see TSV | |
| 216 | `PartyVoteCastReq` | `RequestPartyVoteCast` `0xc144b0` | 16 targetPartyMemberKey u8, 17 markType (`PartyVoteMarkType` NONE 0, POSITIVE 1, NEGATIVE 2) | |
| 1272 | `LootReq` | ❓ (no builder found) | 16 targetId u64 | |

Worked example — `InvitePartyReq` 157 body (struct size 48; `0xc14d60` sets the command id 157 in the packet object). Example values: target id `0x0000000100000ABC`, name "Abc", goal 0, fromSuggestion 0, difficulty 0, level 50, mastery 10, job 1, faction2 1. Fields at struct offsets: 16 = `BC 0A 00 00 01 00 00 00`, 24 = string "Abc", 32 = `00 00 00 00`, 36 = `00`, 37 = `00`, 38 = `32`, 39 = `0A`, 40 = `01`, 41 = `01`. The wire order and string encoding follow the serializer of chapter 05 ❓ (not traced for this packet).

### 1.3 Party packet handler `0xc4bac0`

| Id | Packet | Client action / message key (UTF-16) |
| --- | --- | --- |
| 148 | `SuggestPartyOk` | `Msg.Party.SuggestJoin.Req.Success` |
| 149/150 | `SuggestPartyNotify/Fail` | fail by `SuggestJoinRejectReason`: 1 `…Reject.NoHaveInviteAuth`, 2 `…FullParty`, 3 `…InTeam`; `BParty::InvokePartyErrorMessage` `0xc4b380` |
| 158 | `InvitePartyOk` | `Msg.Party.Invite.Success` |
| 159 | `InvitePartyFail` | `Msg.Party.InviteFail.*`: AlreadyJoin, AlreadyJoinTeam, Fatigue, TargetFatigue, AutoReject.HostileFactionInFactionBattleField, AutoReject.DiffGuildInGuildBattleField, InvalidInvitesPartyType, Buyudo.DiffFaction, Buyudo.TargetOutside, Attraction.PlayerOutside, Buyudo.DiffChannel, InvalidId.InArena, InvalidId, InvalidTargetZone; then `removeInvitedPc` |
| 160 | `InviteParty` | `CheckJoin` `0xbcd3b0` → auto join `0xc14a70` or `CheckInvitePartyAutoReject` → reject `0xc148e0`; else the UI keeps it (`UiPartyManager::addRecvInvited` `0x7f36d0`) ❓ |
| 162 | `JoinPartyFail` | `Msg.Party.JoinFail.*`: Fatigue, Buyudo.DiffFaction, Buyudo.TargetOutside, Attraction.PlayerOutside, Buyudo.DiffChannel |
| 173 | `BanishPartyFail` | `Msg.Party.BanishFail.PartyAuction`, `…InArena` |
| 177 | `ChangePartyPickupRuleOk` | rule 0 `Msg.Party.Change.PickupRule.Free`, 1 `…Sequence`, 2 `…Dedicated`, 3 `…Random`; `Msg.Party.Change.ItemShareRule.Auction/None`, `Msg.Party.Change.PartyAuctionSetup.Price/NoPrice`, `Msg.Party.Change.Dedicated.Member` |
| 180 | `ChangePartyDifficultyTypeOk` | `CheckRecruitMsgDelete` `0xc462b0` |
| 181 | `ChangePartyDifficultyTypeFail` | `Msg.Party.PromotionFail.*`: NoParty, NotLeader, SamePromotionPartyType, TooManyMemberCount, ExistInvalidZoneMember, Common |
| 184 | `ChangePartyInviteAuthFail` | `Msg.Party.ChangeInviteAuthFail.NotLeader` |
| 185 | `DelegatePartyLeader` | `BParty::DelegateLeader` `0xc462e0` |
| 186 | `RejectParty` | `removeInvitedPc` |
| 187 | `JoinParty` | `BMember::SetMemberInfo`, `BMember::Initialize`, `BParty::Add` `0xc05830`; `Msg.Party.ReJoin.Others`, `Msg.Party.Reinforcement.Others`, `Msg.Party.Join.Others2` (by `JoinPartyReason`); `History::AddHistory`; leader → `Mark::RequestBroadcastPartyMemberChannel` |
| 188 | `LeaveParty` | by `LeavePartyReason`: self → `BParty::Clear` `0xc14f10`; 1 `Msg.Party.Banished`, 11 `Msg.Party.BanishVote.Banished`; other → `Msg.Party.Banished.Others`, `Msg.Party.BanishVote.Approved`, `Msg.Party.Member.RrturnToArena.TimeOut`, `Msg.Party.Leave.Others2`; `BParty::Remove` `0xc05770` |
| 189 | `InPartyMember` | full sync: `Clear`, then `Add` for each member; `Msg.Party.Join.Others2`, `Msg.Party.Join.KeepSession`; `Msg.Party.Notify.PartyAuctionSetup.Price/NoPrice` |
| 190 | `BroadcastPartyProperty` | `BMember::Update`; dead → `Msg.Party.Member.Dead` (duel / battle field variants) |
| 191 | `BroadcastPartyMemberState` | `BMember::Logout`; `Msg.Party.Change.State.Logout` |
| 193 | `ChangePartyDifficultyType` | `Msg.Party.Promotion.Notify` |
| 217–225 | vote kick | `Msg.Party.BanishVote.*` (Agreement, AgreementNotify, Fail.PartyAuction/NotEnoughMember/Cooltime/Common, Withdraw, WithdrawNotify, Cancel.*); reset by `PartyVoteResetReason` 0–4 → `Msg.PartyBanishmentVote.Reset.Me/Group` |
| 269 | `BroadcastTeamPartyMemberProperty` | same as 190 |
| 957, 961–967 | arena return choice, arena recruit | `Msg.Party.Member.Choice.ReturnToArena.Yes/No` |

Before the switch: `Msg.Party.Member.PickupItem` (party loot log).

Key S2C layouts:

| Id | Size | Fields |
| --- | ---: | --- |
| 160 `InviteParty` | 56 | 16 partyId u64, 24 fromId u64, 32 fromName str, 40 partyMatchGoal i32, 44 fromSuggestion u8, 45 difficultyType, 46 level, 47 mastery_level, 48 job, 49 faction2 |
| 187 `JoinParty` | 160 | 16 worldId u16, 18 memberKey u8, 24 creatureId u64, 32 creatureName str, 40 race, 41 job, 44 summonedId i32, 48 zoneChannel u16, 56 prop struct (hp i64, level, mastery_level, x/y/z i16, geo_zone, …) |
| 188 `LeaveParty` | 20 | 16 memberKey u8, 17 reason u8 (`LeavePartyReason` 0–15) |
| 189 `InPartyMember` | 96 | 16 partyId, 24 difficultyType, 25 inviteAuth, 26 pickupRule, 27 auctionItemGrade, 32 dedicated_member, 40 partyAuctionStartPrice, 48 dungeonId u16, 50 dungeonStep, 56 challengePartyAttractionKey, 64 attraction mode, 68 members (repeat) |
| 191 `BroadcastPartyMemberState` | 32 | 16 memberId u64, 24 partyMemberState (`PartyMemberState`: NONE 0, RESERVE_ARENA 1, RESERVE 2, JOIN 3, DISCONNECT 4) |

### 1.4 Loot rules, party auction and distribution

| Enum (419 GM) | Values |
| --- | --- |
| `PartyPickupRule` | FREE 0, REQULAR_SEQUENCE 1, DEDICATED_MEMBER 2, RANDOM_MEMBER 3 |
| `PartyAuctionResult` | SUCCESS 0, ALL_PASS 1, FAILED_BIDDING 2, FAILED_NOT_ENOUGH_MONEY 3, FAILED_INVALID_PC 4, FAILED_INVENTORY_FULL 5 |
| `PartyAuction::AuctionBidAuth` | NONE 0, ENABLE 1, GIVEUP 2, NO_AUTH 3 |
| `PartyAuction::AuctionFinishState` | NONE 0, WAIT 1, SUCCESS 2, SUCCESS_DISTRIBUTION 3, SUCCESS_UNKNOWN 4, BID_FAIL 5 |
| `ItemPouch::DistributionState` | NONE 0, START 1, FINISH 2 |

`PartyAuction` (136 bytes). Handler `PartyAuction::ProcessNetObjectEvent` `0xc1c850` (packets 195–215). Senders: `RequestAuctionBid` `0xc138d0` (205 `BidCurrentAuctionItemReq`), `RequestAuctionGiveup` `0xc13820` (208), `RequestAuctionFinish` `0xc137c0` (202), `RequestDistributionAuctionItem` `0xc13770` (211), `RequestPassAuction` `0xc13720` (214). Checks: `CheckEnableBid` `0xbd2280`, `CheckEnableGiveup` `0xbd21e0`, `checkAndRequestGiveup` `0xc1c7d0`.

Master distribution (pouch): `DistributeItemReq` 1316 (16 pouchId, 24 receiver_name, 32 items repeat of index u8; builders `0x44a370`, `0x5980e0`, `0x598170`), `CompleteDistributionReq` 1321 (16 pouchId; builders `0xc9f610`, `0x10d4670`, `0x10d49b0`). UI: `MasterDistributionPanel` (`11-social-functions.tsv`). Item pouch open/close: 1296–1305.

### 1.5 Party tick (timers)

`BParty::Tick` `0xc1cb10`:

| Timer | Period | Action |
| --- | ---: | --- |
| `_clearLogoutMemberCheckTimer` (+72, period +80) | 1000 ms | if `checkClearLogoutMember` `0xbda960` → `RequestClearLogoutMember` (174); `checkAndRemoveInvitedPc` |
| `_memberPropUpdateTimer` (+88, period +96) | 500 ms | `updateMemberListProp` `0xbef4b0`, then `World::SendClientEvent` |

Party sync summary: GameD sends `InPartyMember` 189 (full list), `JoinParty` 187 / `LeaveParty` 188 (delta), `BroadcastPartyProperty` 190 (hp, level, position, effects of one member) and `BroadcastPartyMemberState` 191 (login state).

### 1.6 Raid (team)

`BTeam` (232 bytes, constructor `0x880a50`). Handler `BTeam::ProcessNetObjectEvent` `0x8f9c90`; the decompiled switch uses `id − 229`.

| Id | Packet | Action |
| --- | --- | --- |
| 234 | `InviteTeamReq` | `BTeam::RequestInviteTeam` `0xc050b0`, `RequestInviteTeamByName` `0x8b7be0` |
| 235 | `InviteTeamOk` | `Msg.Team.Invite.Success.TossPartyLeader` / `…TossTeamLeader` |
| 237 | `InviteTeam` | `CheckJoinTeam` `0x880880`, `CheckInviteTeamAutoReject` `0x8a5530`, then join `0x8b7b70` or reject `0x8b7af0` ❓ |
| 238 | `InviteTeamTossNotify` | `Msg.Team.Invite.TossPartyLeader.Notify` / `…TossTeamLeader.Notify` |
| 243 | `LeaveTeamReq` | `RequestLeaveTeam` `0x8b7a90` |
| 245 | `BanishTeamReq` | `RequestBanishTeam` `0x8b79b0` |
| 247 | `TeamMemberPartyChangeReq` | `RequestTeamMemberPartyChange` `0x8b78b0` |
| 256 | `ChangeTeamInviteAuthReq` | `RequestChagneTeamInviteAuth` `0x8b7a20` |
| 259 | `ChangeTeamPartyDifficultyTypeReq` | `RequestTeamPromotionPartyType` `0x8b7790` |
| 265 | `JoinTeam` | `Msg.Team.Join.Others`, `History::AddHistory` |
| 266 | `LeaveTeam` | `Msg.Team.Banished.Others`, `Msg.Team.Leave.TargetEnteredFatique`, `Msg.Team.Leave.Others`; `leaveTeamMember` `0x86e320` |
| 267 | `InTeamMember` | `BTeam::Clear` `0x876c50`, `TeamMember::Initialize`, `Msg.Team.Join.Others` |
| 271 | `TeamMemberPartyChangeNotify` | `Msg.Team.TeamMemberPartyChange.Player/Member` |
| 272 | `DelegateTeamLeaderNotify` | `Msg.Team.DelegateTeamLeader.Others` |
| 273 | `NominatePartyLeaderNotify` | `Msg.Team.NominatePartyLeader` |
| 275 | `ChangeTeamPartyDifficultyTypeNotify` | `Msg.Team.TeamPromotionPartyType` |

Loot log: `Msg.Team.Member.DistributeItem`, `Msg.Team.Member.PickupItem`. `TeamPartyMemberCondition`: HP_ALERT 0, NEWBIE_INFO_FLAG 1.

Other party groups (only in the TSV): challenge party 1068–1096 (`ChallengeParty::ProcessNetObjectEvent` `0x11b8f00`), party battle 289–295 and 1021–1024, raid dungeon bind 912–927 (`RaidDungeon`, chapter 10), public raid 1056–1061, arena lobby party 760–764.

---

## 2. Chat

### 2.1 Game chat (GameD)

`ChatChannel::SendChatMessageReq` `0x116f260` (and `ChatChannelNP::SendChatMessageReq`, same caller of `0x43ff10`):

1. Range 100 with the QQ option → `ChatChannel::sendQQC2C` / `QQMessenger::RequestSendGroupMessage` (Tencent only).
2. If `this+696` ≠ 0 (a chat server channel is current) → `ChatChannel::sendToChattingServer` (§2.2).
3. Else it builds `G_MessageReq` 51 (`0x43ff10`) and calls `SendToGamed`.

`World::SendUserChatMessage` `0x740700` checks first: target in `BlackList` (`BlackList::FindPlayer`) → `Msg.Chatting.SendFail.BlockedTarget`; `BPlayer::IsRestrictSpamCondition` → spam message; text length after `ConvertLinkTagFromEngineToPacket` ≥ 400 (`0x190`) → `Msg.Chatting.SendFail.Legnth`. A whisper to a friend on another cluster goes through `World::CheckFriendPresenceMessageWhisper` `0x72e0e0` → `World::sendPresenceMessageWhisper` `0x7402d0` → NP 3202 (chapter 03 §7.4).

| Id | Packet | Dir | Size | Fields |
| --- | --- | --- | ---: | --- |
| 51 | `MessageReq` | C2S | 48 | 16 range u8 (`MessageRange`), 20 toName str (whisper target), 28 id i32, 32 header str, 40 text str |
| 52 | `Message` | S2C | 80 | 16 toWorldId u16, 20 toName, 32 from u64, 40 fromWorldId u16, 44 fromName, 52 range u8, 56 id i32, 60 header, 68 text. Handled in `World::ProcessNetObjectEvent` (`World::AddChatMessage` `0x744ff0` is called at `0x749b53`; case link ❓) |
| 53 | `MessageNotify` | S2C | 44 | 16 toName, 24 header, 32 text, 40 toQQId i32 (echo of own whisper ❓) |
| 54 | `MessageFail` | S2C | 20 | 16 error u16 |
| 55 | `MessageByItemReq` | C2S | 28 | 16 text, 24 font_color_index u8, 26 consumeItemBagPos u16 (megaphone; builder `World::SendMessageByItem` `0x7378e0`) |
| 56 | `MessageByItem` | S2C | 44 | 16 fromWorldId, 20 fromName, 28 range u8 (`ItemMessageRange`: DISTRICT 1, ALL 2), 32 text, 40 font_color_index |
| 58 | `MessageByDiceReq` | C2S | 20 | 16 rangeMin u16, 18 rangeMax u16 (builder `0xf30180` in `game-commands.tsv` is a false match: it sends InfoGate 58, §3.2) |
| 59 | `MessageByDice` | S2C | 40 | 16 rangeMin, 18 rangeMax, 24 pcId u64, 32 randomValue u16 |
| 50 | `GodSay` | S2C | 40 | 16 at i32, 20 repeatInterval i32, 24 repeatCount i32, 28 type u8, 29 position u8, 32 msg (`World::GodSays`) |

`MessageRange` (419 GM): ONE 0 (whisper), SIGHT 1, ZONE 2, PARTY 3, TEAM 4, CHANNEL 5, GUILD 6, PARTY_SEARCH 7, ARENA 8, FIELD_DUNGEON 9, TENCENT_ONE 10, RAID 11, RAID_SQUAD 12, RAID_LEADERS 13, RANDOM_STORE 14, GUILD_SEARCH 15, WATCH 16, MENTORING 17.

Worked example — whisper "hi" to "Bob": `MessageReq` 51, range = 0, toName = "Bob", id = 0 ❓, header = "", text = "hi".

Social (emote): 47 `PlaySocial`, 48 `PlaySurroundSocial`, 68–70 `SetPcSocialReq/Ok/Fail` (TSV). Spam report: 419–421 `ReportBotSpammer*`.

### 2.2 Chat server (`ChannelChattingClient`)

The client gets a login signature from GameD: `ChattingServerLoginSignatureReq` 387 (16 gameServerType u8: `ChattingGameServerType` NONE 0, GAME 1, ARENALOBBY 2; builder `ChatChannel::requestLoginSignature` `0x11564e0`) → `…Ok` 388 (16 gameServerType, 24 chattingServerId u64, 32 loginSignature bytes) or `…Fail` 389. GameD channel join: 342 `JoinChattingChannelReq` (16 domainId u8; `ChatChannelNP::requestJoinChat` `0x11557c0`) → 343 Ok (16 domainId, 17 channel), 345–347 leave.

`ChannelChattingClient` (544 bytes): `Connect` `0x5e9d40` (`Connect success. ip:%s, port:%d`), `Login` `0x5ea030` (sends `SIGNATURE_LOGIN` or `NP_LOGIN`), `SendVersionInfo` `0x5e9e80`, keepalive `SendKeepAlive` `0x5ea720` / `StartKeepAlive` `0x5ea7e0` (thread; period ❓), `PacketHandler` `0x5eaff0`, state `SetClientState` `0x5eb5a0` (names in `ECLIENT_STATE_STRING`). The parser is `PacketParser` (chapter 02 §6).

Header (from `WritePacket_C2S_CHANNEL_CHATTING::Write` `0x5dcde0`): `+0` length u16 (whole packet), `+2` type u16, `+4` seq u16.

| Type | Name | Dir | 381 builder / handler |
| ---: | --- | --- | --- |
| 24 | `C2S_CHANNEL_CHATTING` | C2S | `_ccChannelChat` `0x5e0030`; body: +6 gameServerId u64, +14 gameUserId u64, +22 channelId u32, +26 byte ❓, +27 text length u16 (chars), +29 UTF-16 text. Fixed length 29 + 2×chars (`0x5e4600`) |
| 255 | `C2S_KEEP_ALIVE` | C2S | `0x5ea720` |
| 16395 | `C2S_CREATE_CHANNEL` | C2S | `_ccCreateChannel` `0x5de310` |
| 16397 | `C2S_JOIN_CHANNEL` | C2S | `_ccJoinChannel` `0x5ded30` |
| 16400 | `C2S_JOIN_OPEN_CHANNEL` | C2S | `_ccJoinOpenChannel` `0x5df2a0` |
| 16418 | `C2S_SEARCH_CHANNEL` | C2S | `_ccSearchChannel` `0x5dde40` |
| 16420 | `C2S_CHANGE_CHANNEL_OPERATOR` | C2S | `_ccChannelHostPromote` `0x5e06e0` |
| 16432 | `C2S_VERSION_INFO` | C2S | `SendVersionInfo` `0x5e9e80` |
| 16436 | `C2S_SPAMMER_REGIST` | C2S | `_ccSpammerRegist` `0x5e0410` |
| 18689 | `C2S_ACTIVATE_VOICECHAT` | C2S | `_ccActivateVoiceChat` `0x5e0d40` |
| 18695 | `C2S_START_VOICECHAT` | C2S | `_ccStartVoiceChat` `0x5e1320` |
| 18707 | `C2S_MUTE_USER` | C2S | `_ccMuteVoiceChatMember` `0x5e2020` |
| ❓ | `LOGIN` (signature / NP), `LEAVE_CHANNEL`, `STOP_VOICECHAT`, `UNMUTE_USER` | C2S | `Login` `0x5ea030`, `_ccLeaveChannel` `0x5df840` |
| 16386 | `S2C_LOGIN_RESULT` | S2C | `0x5f44b0` |
| 16388 | `GET_CHAT_ID_FROM_GLOBAL_RESULT` | S2C | `0x5f9370` |
| 16396 | `CREATE_CHANNEL_RESULT` | S2C | `0x5f4e00` |
| 16398 | `JOIN_CHANNEL_RESULT` | S2C | `0x5f5970` |
| 16401 | `JOIN_OPEN_CHANNEL_RESULT` | S2C | `0x5f60d0` |
| 16412 / 16417 | `BAN_` / `UNBAN_FROM_CHANNEL_RESULT` | S2C | `0x5f8620` / `0x5f8ee0` |
| 16419 | `SEARCH_CHANNEL_RESULT` | S2C | `0x5f4900` |
| 16421 / 16424 | `CHANGE_CHANNEL_OPERATOR_` / `PROPERTY_RESULT` | S2C | `0x5f7520` / `0x5f7f70` |
| 16427 / 16430 | `GET_USER_GAME_INFO_` / `GET_CHAT_SERVER_ADDR_RESULT` | S2C | `0x5f9670` / `0x5f9b80` |
| 16433 / 16435 / 16437 | `VERSION_INFO_` / `CREATE_AND_JOIN_CHANNEL_` / `SPAMMER_REGIST_RESULT` | S2C | `0x5f4290` / `0x5f51d0` / `0x5fd050` |
| 18690–18720 | voice results (ACTIVATE 18690, DEACTIVATE 18693, START 18696, STOP 18705, MUTE 18708, UNMUTE 18711, PROPERTY 18720) | S2C | `0x5fa030`…`0x5fcae0` |
| 2307–2337 | voice notifies (ACTIVATED 2307, DEACTIVATED 2310, STARTED 2313, STOPPED 2322, USER_MUTED 2325, USER_UNMUTED 2328, PROPERTY_CHANGED 2337) | S2C | `0x5fa380`…`0x5fcdf0` |
| 15 / 19 | `JOINED_CHANNEL` / `LEFT_CHANNEL` | S2C | `0x5f64d0` / `0x5f67d0` |
| 25 / 26 | `CHANNEL_CHATTING_FAILED` / `CHANNEL_CHATTING` | S2C | `0x5f6fa0` / `0x5f7260` |
| 29 / 38 / 41 | `BANNED_FROM_CHANNEL` / `CHANNEL_OPERATOR_CHANGED` / `CHANNEL_PROPERTY_CHANGED` | S2C | `0x5f8ab0` / `0x5f79b0` / `0x5f8300` |

Rule seen: a request `0x40xx` / `0x49xx` has its result at request + 1 (16397 → 16398, 18689 → 18690).

Game side: `ChattingClient2` (224 bytes, `processMessage` `0x1160c60`, `SendMessageW` `0x1161230`, `Login` `0x116b180`) and `ChattingClientNP` (NP transport). Channel types (`ChatChannelInterface::ChannelType`): NONE 0, DISTRICT 1, TRADE 4, PARTYSEARCH 5, PARTY_LOBBY_ARENA 7, DISTRICT_LOBBY_ARENA 8, GUILD_VOICE_1 10, GUILD_VOICE_2 11, TEAM 12, GUILDSEARCH 13, MENTORING 15. Errors: `ChatChannelInterface::ChannelErr` 0–10. Commands: `ChatChannelCommand` (`11-social-enums-gm.tsv`).

---

## 3. Guild (clan) and faction

### 3.1 GameD packets

`Guild` (2,336 bytes; constructor `0xf3e5e0`, `Tick` `0xf3e140`, handler `Guild::ProcessNetObjectEvent` `0xf6de30`). The 172 `Guild::` functions are in `11-social-functions.tsv`; 51 of them are `receive*` handlers.

| Id | Packet | Builder | Fields |
| --- | --- | --- | --- |
| 431 | `CreateGuildReq` | `0xf33530` | 16 GuildType u8 (`GuildType`: INVALID 0, EXCLUSIVE 1, OPEN 2), 20 guildName str, 32 npcId u64 |
| 503 | `InviteGuildReq` | `0x47cb30` (`RequestInviteGuild` `0xf32af0`) | 16 target_name str, 24 guildInvitationType u8 |
| 505 | `InviteGuild` (S2C) | handler `0xef5150` | 16 guildId, 24 guildName, 32 fromId, 40 fromName, 48 guildInvitationType |
| 506 / 508 | `ConfirmInvitationGuildReq` / `RejectInvitationGuildReq` | `0xf32a20` / `0xf329d0` | (none) |
| 509 | `JoinGuildReq` | `RequestJoinGuild` `0xf313c0` | 16 guildId |
| 511 | `LeaveGuildReq` | `0xf32980` | (none). `LeaveGuildReason`: INVALID 0, LEAVE 1, BANISH 2 |
| 513 | `BanishGuildMemberReq` | `0x47cb50` (`RequestBanishGuildMember` `0xf324f0`) | 16 index u8, 20 target_name |
| 477 / 478 | `RetrieveGuildMembersReq/Ok` | `RequestRetrieveGuildMembers` `0xf333c0` / handler `0xef5610` | 478: repeat of index u8, name, pc_id u64, level u16, faction2, mastery_level, race, job, grade u8, … |
| 491 | `ChangeGuildMemberGradeReq` | `0xf32fa0` | grade: `CONTEXT_MENU::GuildMemberGrade` STAFF_CAPTAIN 0, ELDER 1, GENERAL 2, LOWER 3 |

All other guild requests (bank, craft, uniform, permission, notice, leader, application, invitation tickets, guild point, benefit, alliance, warp/call) are in `11-social-commands.tsv` (topic `guild`, 218 rows) with their `Guild::Request*` builder. The guild tick re-sends `RetrieveGuildMembersReq` when its timer (+2032, period float +2040) runs out in guild battle mode; period values are set at run time (0.0 in the constructor) ❓.

### 3.2 InfoGate path (server side: InfoGate, not GameD)

These requests call `BNSClient::SendToInfoGate` `0x114cdb0`; replies come to `Guild::ProcessInfoGateNetObjectEvent` `0xf3df60`.

| InfoGate id | Name (419 GM) | 381 sender |
| ---: | --- | --- |
| 46 | `I_RetrieveGuildJoinRenewalPcInfoReq` | `Guild::RequestJoinRenewalPc` `0xf304b0` |
| 52 | `I_RetrieveGuildJoinRenewalGuildInfoReq` | `Guild::RequestJoinRenewalGuild` `0xf30750` |
| 58 | `I_RetrieveGuildPointInfoListReq` | `Guild::RequestGuildPointInfoList` `0xf30180` |
| 31 | `I_RetrieveGuildMemberRankingInfoListReq` | `BSeason::RequestRetireveCurGuildRanker` `0x9d1240` |
| 16 | `I_SearchCharacterListReq` | `0xfbd020` (friend search ❓) |
| 7 | `I_RetrieveFriendListReq` | `Friends::SendRetrieveFriendListReq` `0xfbdbe0` (send path ❓) |

`RequestJoinRenewalPcList` `0xf302f0` and `RequestJoinRenewalGuildList` `0xf30590` do not call `SendToInfoGate` directly ❓. Full list: `11-social-infogate.tsv`.

### 3.3 Faction

| Id | Packet | Builder | Fields |
| --- | --- | --- | --- |
| 125 | `JoinFactionReq` | `0x5a0570` | 16 factionId u8, 24 npcId u64 |
| 127 / 129 | `LeaveFactionReq` / `TransferFactionReq` | TSV | |
| 83 | `ExchangeFactionScoreReq` | TSV | |
| 131 | `RetrieveFactionStatisticsReq` | TSV | |

---

## 4. Friends, block list, mentoring

`Friends` (120 bytes). The friend list lives in the NP platform (STS, chapter 03). Requests use `NPSession::SendMsgRequest`; replies go to `Friends::ProcessNPEvent` `0x100a3f0`.

| NP id | Request (381) | Reply handler (381) |
| ---: | --- | --- |
| 3751 | `Friends::requestProposal` `0xfbd4a0` (from `RequestProposal` `0x10032f0`; already friend → `Msg.Friends.RequestProposal.Fail.AreadyFriend`) | ❓ |
| 3752 | `RequestDenyProposal` `0xfbdf50` | `onReceiveDenyProposal` `0x1002580` |
| 3753 | `RequestAcceptProposal` `0xfbe100` | `onReceiveAcceptProposal` `0x1002170` |
| 3754 | — | `onReceivePageRecvProposal` `0xfd20c0` |
| 3755 | `RequestRemove` `0xfbdec0` | `onReceiveFriendRemove` `0x100a340` |
| 3756 | `RequestChangeDescription` `0xfbdd50` | `onReceiveChangeAlias` `0xf8e9d0` |
| 3757 | — | `onReceiveGetUserInfo` `0xfd2030` |
| 3758 | `requestReleation` `0xfbd7c0` | `onReceiveGetRelations` `0xfd54e0` |
| 3759 | — | `onReceivePageFriends` `0xfd5350` |
| 3760 | — (notify) | `onReceiveFriendProposedNotify` `0xfd1e60` |
| 3761 | — (notify) | `onReceiveAcceptProposalNotify` `0xffbb90` |
| 3763 | — (notify) | `onReceiveFriendInfoNotify` `0x1001f50` |
| 3764 | — (notify) | `onReceiveFriendMessageNotify` `0x1005070` |
| 3765 | — (notify) | `onReceiveUserNameChangedNotify` `0x1003160` |
| 3766+ | — | `onReceiveFriendRemoveNotify` `0x100a2e0`, `onReceiveProposeResponse` `0xf8ec10`, `onReceivePresenceMessage` `0x1005d60`, `onReceiveAuthGetUserInfo` `0x10031e0` (exact ids ❓) |

InfoGate replies: `Friends::ProcessInfoGateNetObjectEvent` `0x100a180`. Block list: `BlackList` (12 bytes), `BlackList::FindPlayer` (used by `0x740700`); UI tabs `FriendBlackListTab`, `FriendSpammerListTab`. Block list storage path ❓.

Mentoring: packets 1097–1117 (TSV topic `friend`), classes `Mentoring` (56 bytes), `MentoringPanel`.

---

## 5. Mail (`DeliveryBox`)

`DeliveryBox` (200 bytes). All mail packets go to GameD (`GAMED_ONLY`). S2C handlers route through `World::ProcessNetObjectEvent`.

| Id | Packet | Builder / handler | Fields |
| --- | --- | --- | --- |
| 368 / 369 | `RetrievePostCountReq/Ok` | ❓ / `0xeed320` | 369: 16 total_system, 20 unread_system, 24 attached_system, 28 total_user, 32 unread_user, 36 attached_user (all i32) |
| 371 / 372 | `RetrievePostBriefListReq/Ok` | `0x1060310` / `DeliveryBox::receiveRetrievePostBriefListOk` `0x1095220` | 372: repeat system_post_brief_list: post_id u64, post_type u8, sender_service_type u8, delivery_reason u8, service_instance_id u64, hint1 u64, receipt_time i64, total_attachment_count … |
| 374 / 375 | `RetrievePostReq/Ok` | `RequestRetrievePost` `0x1060290` / `receiveRetrievePostOk` `0x1077cb0` | 374: 16 post_kind u8, 24 postId u64. 375: post_kind, post_id, post_message str, attachment repeat (type u8, item_instance_id, item_data_id i32, item_level i16, amount i64, item sub, status) |
| 1500 | `ArchivePostReq` | `0x105ff90` | 16 post_kind, 24 postId; Ok → `receiveArchivePostOk` `0x10951e0` |
| 1503 | `MarkExpressPostReq` | `0x105ff90` ❓ | 16 post_kind, 24 postId |
| 1506 | `DetachPostAttachmentReq` | `RequestDetachPostAttachment` `0x1060160` | 16 post_kind, 24 postId, 32 inst_id, 40 npcId |
| 1509 | `DeliverPostReq` | `RequestDeliverPost` `0x106d530` (packet ctor `0x45d420`) | 16 post_type u8, 24 npcId, 32 recipient_world_Id u16, 36 recipient_name, 44 post_title, 52 post_message, 64 money i64, 72 Itemlist repeat (pos u16, count u16), 84 ChargeItemlist repeat. Ok → `receiveDeliverPostOk` `0x1082270` |

Enum: `AuctionDeliveryReason` (market mail reasons, TSV).

---

## 6. Market (auction house) and NP auction

### 6.1 `AuctionHouse` (GameD)

`AuctionHouse` (480 bytes; `GetInstance` `0x119a110`). Handler `AuctionHouse::ProcessNetObjectEvent` `0x1201b20`.

| Id | Packet | Builder / handler | Fields |
| --- | --- | --- | --- |
| 378 / 379 | `RetrieveSaleListReq/Ok` | `RequestListOfMySales` `0x11f47d0` / `receiveListOfMySalesOk` `0x12014b0` | 16 search_type u8, 17 order_type u8, 24 search_key u64, 32 page_no u8; 379 adds sale_list (sale_id, sale_type, sale_price, unit_price, top_bid_price, …) |
| 381 / 382 | `RetrieveBidListReq/Ok` | `RequestListOfMyBids` `0x11db4a0` / `receiveListOfMyBidsOk` `0x1201220` | TSV |
| 384 / 385 | `RetrieveBidRankStatusReq/Ok` | `RequestRetrieveBidRankStatus` `0x11db360` / `0x11c8d60` | TSV |
| 1530 | `RegisterSaleReq` | `0x11dba50` | 16 npcId, 24 sale_type u8, 32 sale_price i64, 40 sale_duration i32, 44 item_pos u16, 46 item_count u16. Ok/Fail → `0x119a1a0` / `0x119a140` |
| 1533 | `AbandonSaleReq` | `0x11db9d0` | 16 saleId. → `0x11eae80` / `0x11eae50` |
| 1536 | `UpdateSalePriceReq` | ❓ | 16 saleId, 24 new_sale_price |
| 1539 | `RegisterBidReq` | `0x11db900` | 16 sale_id, 24 bid_price. → `0x11db880` / `0x11ac3b0` |
| 1542 / 1545 | `AbandonBidReq` / `UpdateBidPriceReq` | TSV | → `0x11db7d0`/`0x11db7b0`, `0x11ac330`/`0x11ac2d0` |
| 1548 | `RegisterPurchaseReq` | `0x11db610` | 16 sale_id, 24 purchase_price. → `0x11ac2a0` / `0x11ac260` |
| 1551 / 1552 | `MarketSearchItemReq/Ok` | `0x11db510` / `receiveSearchArticlesResultOk` `0x12016d0` | 16 search_type, 17 order_type, 18 price_type, 19 search_key_sale_type, 20 search_key_data_id i32, 24 sale_price, 32 unit_price, 40 index_id; Ok has prev/next search key structs |
| 1554 / 1555 | `RetrieveInstantSaleItemLowestPriceReq/Ok` | `0x11db2e0` / `0x11954d0` | 16 item_data_id i32 |

Fees: `getTaxRateBySalesPeriod` `0x11dadb0`, `GetRegisteringCommission` `0x11db000`, `getExtraCommissionRateByItemAmount` `0x119f410`, `GetPcCafeDiscountRateForRegisteringSalesFee` `0x11ac0d0`. NPC rule: `IsTalkingToMarketNpc` `0x11c8c00`, `IsSellEnabledWithoutTalkingToMarketNpc` `0x1195450`. Cool time: `IsRequestSalesListCoolTimerOver` `0x119f5d0`.

### 6.2 Party auction (loot)

See §1.4 (packets 195–215).

### 6.3 `NPAuction` (STS / NP platform)

`NPAuction` (184 bytes). Requests: `RequestGetBalance` `0xc98100`, `RequestGetStatistics` `0xc98260`, `RequestPageMySaleHistoryList` `0xc98430`, `RequestPageMySales` `0xc98530`, `RequestPageItems` `0xc98630`, `RequestBid` `0xc98750`, `RequestCancelSale` `0xc98900`, `RequestSell` `0xc989a0`, `RequestNotice` `0xc5fb30`. Replies: `NPAuction::ProcessNPEvent` `0xc73590` → `onReceive*` (`0xc5a090`…`0xc73510`). `NPAuction::RequestBid` also has game packet `NPAuctionBidReq` 1013 (16 auction_id 16 bytes, 32 bid_currency_group_id u16, 40 bid_price i64) and calls NP 1601 (chapter 03 §7.4) ❓ which path is live.

---

## 7. Trade

`UiTradeManager` (2,280 bytes) + `TradeShelf` (344 bytes: `_playerShelf` +8, `_opponentShelf` +168, `_lastRegisterMoney` +328, `_player` +332, `_bWaitingTradeConfirmResult` +336). Max 8 items (`G_PREDEFINEDTRADEITEMCOUNT_MAX`).

State flow (`UiTradeManager::ProcessNetObjectEvent` `0x7bf040`):

```
 InviteTradeReq 311 (BPlayer::RequestInviteTrade 0xb7d790, guest u64)
   -> S2C InviteTrade 313 (inviter, guest): guest -> BPlayer::CheckInviteTradeAutoDecline, GuestTradeConfirmPanel::Show; inviter -> InviterTradeConfirmPanel::Open
 JoinTradeReq 314 (BPlayer::RequestJoinTrade 0xb7d660, inviter u64) | DeclineTradeReq 317
   -> JoinTrade 316: UiTradeManager::startTrade 0x7bef40, Msg.Trade.Join.Mine / .Other
   -> DeclineTrade 319: Msg.Trade.Decline / .BlockedPlayer / .TargetIsBusy, GuestTradeConfirmPanel::Hide
 open: AppendTradeSuggestionReq 324 (TradeShelf::RequestRegisterItem 0x814640: invenPos u16, itemCount u16)
       RemoveTradeSuggestionReq 327, SuggestTradeMoneyReq 330 (RequestRegisterMoney 0x8146d0: money i64)
       -> 325/326, 328/329, 331/332 (Other/Mine) -> TradeShelf::ProcessNetObjectEvent 0x8091e0; 331 Msg.Trade.Suggest.Change
 SubmitTradeReq 333 (0x814510) -> SubmitTrade 335: TradeShelfPanel::UpdateSubmit, Msg.Trade.Submit.Mine / .Other
 RetractTradeReq 336 -> RetractTrade 338: confirm UI.Confirm.TradeNotify.RetractByUserReq
 ConfirmTradeReq 339 (RequestConfirmTrade 0x814460) -> ConfirmTrade 341 (trader u64): endTrade 0x7beef0
 CancelTradeReq 320 -> CancelTrade 322: endTrade, invokeCancelTradeMessage 0x7684d0
 SuggestTradeFail 323 -> invokeFailMessage
```

Enums (419 GM, `11-social-enums-gm.tsv`): trade reason OK 0, CAN_NOT_TRADE 1, INTERNAL 2; `TRADE_CHECK_ERR_*` 1–18; `TRADE_DECLINE_REASON` NORMAL 0, BLOCKED_PLAYER 1, TARGET_IS_BUSY 2; `G_TRADECANCELREASON` NONE 0, USER_REQUEST 1, DEAD 2, IN_COMBAT 3, LEAVE_ZONE 4.

NPC trade: `TradeWithNpcReq` 1352 (16 seller u64, 24 index u8, 26 count u16, 28 consumeItems repeat). Item exchange: `ExchangeItemReq` 1010.

---

## 8. Open items ❓

- Chat server: login packet type ids, `LEAVE_CHANNEL`/`STOP_VOICECHAT`/`UNMUTE_USER` type ids, keepalive period, byte at +26 of `C2S_CHANNEL_CHATTING`.
- `ChattingClient2` vs `ChattingClientNP`: which one the 381 live build uses.
- NP ids of friend notify replies after 3766; `Friends::SendRetrieveFriendListReq` send path.
- Guild tick timer periods (set at run time).
- Builders without address: `LootReq` 1272, `UpdateSalePriceReq` 1536, `RetrievePostCountReq` 368.
- Duel packets (topic `duel`, 45 rows) are listed in the TSV only; arena chapter covers them.
