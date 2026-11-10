# Client.exe reversing notes (build 0.0.381, 2020) — overview

This file is the short overview of the 2020 `Client.exe` (0.0.381, x86) reference.
The full reference is the chapter set in [`RE/`](RE/INDEX.md). Start at [`RE/INDEX.md`](RE/INDEX.md): chapter list, "how to find anything" guide and A–Z keyword index.
Datafile facts: [`client-datafiles.md`](client-datafiles.md) and the [Datafile book](../DataFile/book/00-index.md).
The old game command list is [`client-381-game-commands.tsv`](client-381-game-commands.tsv); the full table with handlers and builders is [`RE/game-commands.tsv`](RE/game-commands.tsv).

Rules:

- Each fact has a source: address, symbol, enum, table or file.
- "381" = the 2020 client (0.0.381.x, x86). "419 GM" = the 2021 GM client with its PDB.
- ❓ marks a fact that we did not verify.
- Passive reads only. No keys, no crypto internals, no protection details. Where a key is needed, the text writes `keyhere`.

---

## 1. Map of the client

```
 Client.exe 381 (WinMain 0x1190f60)
   │ BNSClient::Initialize 0x116f550 ── XmlReader DLL, datafile (ch. 13), UI (ch. 14)
   │ LoadEngine 0x114c6d0 ── bsengine_Shipping.dll (ch. 15)
   │
   ├─ STS client (NPStsMsgLib::Create 0xbc90a0) ──► StServers login          ch. 03
   ├─ LobbyGateSession (ctor 0xc673e0) ──► LobbyGate, L_* commands 1..110    ch. 04
   ├─ GameSession (ctor 0xefbf40) ──► GameD (and ArenaD = 2nd GameSession)   ch. 05, 12
   │     send: BNSClient::SendToGamed 0x114ce20 (PacketDest, sync lock)
   │     recv: GamePacketHandler[cmd] → World::ProcessNetObjectEvent 0x749550
   ├─ InfoGate (BNSClient::SendToInfoGate 0x114cdb0)                         ch. 12
   └─ transport: Network, Session, PacketProcessor (envelope, crypto)        ch. 02
```

| Area | Chapter | Key 381 address |
| --- | --- | --- |
| Binary map, classes, all functions, name-transfer method | [00-inventory](RE/00-inventory.md) | [`RE/classes.tsv`](RE/classes.tsv), `RE/functions-*.tsv` |
| Start, command line, config, main loop | [01-startup](RE/01-startup.md) | `WinMain` `0x1190f60` |
| Sockets, framing, session crypto | [02-transport](RE/02-transport.md) | `PPUtil::PacketSize` `0x60f8a0` |
| STS login | [03-sts-client](RE/03-sts-client.md) | `NPStsMsgLib::Create` `0xbc90a0` |
| LobbyGate | [04-lobbygate](RE/04-lobbygate.md) | `LobbyGateSession::ProcessLobbyNetObjectEvent` `0xcb86d0` |
| Game session, dispatch, packet layouts, sync groups | [05-game-dispatch](RE/05-game-dispatch.md) | `BNSClient::SendToGamed` `0x114ce20` |
| Movement | [06-movement](RE/06-movement.md) | `MoveListMaker::sendWalkPacket` `0xcc4ce0` |
| Skills and combat | [07-combat](RE/07-combat.md) | `BControlledPlayingActions::UseSkill` `0x1114f50` |
| Player, NPC, world objects, summoned cat | [08-creature](RE/08-creature.md) | `World::enterCreature` `0x74b640` |
| Items | [09-items](RE/09-items.md) | `Inventory::RequestMoveItemPosition` `0xe91e00` |
| Quests, NPC talk, zones | [10-quests-npc](RE/10-quests-npc.md) | `BQuestJournal::ProcessNetObjectEvent` `0xa9e5b0` |
| Party, guild, chat, mail, market | [11-social](RE/11-social.md) | `BParty::ProcessNetObjectEvent` `0xc4bac0` |
| InfoGate, ArenaD, other servers | [12-infogate-arena](RE/12-infogate-arena.md) | `BNSClient::SendToInfoGate` `0x114cdb0` |
| Datafile (link page) | [13-datafile](RE/13-datafile.md) | `Data::DataManager::Load` `0x5bc500` |
| UI, system messages, commands | [14-ui](RE/14-ui.md) | `Ui::Initialize` `0x8135e0` |
| Engine, map load, options, sound | [15-engine](RE/15-engine.md) | `LoadEngine` `0x114c6d0` |
| Error codes, logs, string anchors | [16-strings-errors](RE/16-strings-errors.md) | `GameSession::OnError` `0xf44150` |
| Coverage audit, open items, moved facts | [17-open-items](RE/17-open-items.md) | — |
| GM debugger (419 GM) | not written (blocked, [17 §3.3](RE/17-open-items.md)) | — |

---

## 2. Where each section of the old notes went

The old sections 1–10 (before 2026-11-10) are now in the chapters. Every old fact is kept there.

| Old § | Topic | Now in |
| --- | --- | --- |
| 1 | Symbols for the 381 exe (match from 419 GM PDB, 37,543 matched functions = 83.8 %, `c20_named.json`, `mkpdb.py`, `llvm-undname`, per-class counts) | [00 §3](RE/00-inventory.md) (method, now 36,965 names), [17 §4.1](RE/17-open-items.md), class counts in [`RE/classes.tsv`](RE/classes.tsv) |
| 2.1 | Server layout (LobbyGate → GameD / ArenaD, InfoGate, auth) | [04](RE/04-lobbygate.md), [05](RE/05-game-dispatch.md), [12](RE/12-infogate-arena.md) |
| 2.2 | Command id tables (LobbyGate 110, game 1,598 names at `0x1953620`, UI buttons at `0x1955858`, GM-only tables, id shift from 919) | [05 §2](RE/05-game-dispatch.md), [14 §3](RE/14-ui.md), [17 §4.1](RE/17-open-items.md) |
| 2.3 | `StructPacket::FieldType` (T_INT8 0 … T_PACKET_REF 16), `StructPacket` base | [05 §3](RE/05-game-dispatch.md) |
| 2.4 | `PacketDest`, `PacketEventType`, `GamePacketRightType`, `PacketProcessor::ErrCode`, `GamePacketSync` | [05 §1, §4](RE/05-game-dispatch.md), [02 §8](RE/02-transport.md), [17 §4.3](RE/17-open-items.md) |
| 2.5 | `PacketParser::ParseData` `0x5fd770` | [02 §6](RE/02-transport.md) (correction: chat and voice only; game framing is [02 §2](RE/02-transport.md)) |
| 2.6 | Packet layouts (`G_HelloReq`, `L_HelloReq`, lobby token, enter world, move sub-packet) | [05 §5](RE/05-game-dispatch.md), [04](RE/04-lobbygate.md), [`RE/packet-layouts.tsv`](RE/packet-layouts.tsv), [17 §4.3](RE/17-open-items.md) |
| 2.7 | Session class addresses (`0xefbf40`, `0xf2bf60`, `0xc97480`, `0xc97710`, `0x5fd5d0`, `0x5fd770`), lobby enums | [02](RE/02-transport.md), [04](RE/04-lobbygate.md), [05](RE/05-game-dispatch.md), [12](RE/12-infogate-arena.md) |
| 3 | Jobs, races, weapon types | [17 §4.2](RE/17-open-items.md), races in [08 §4](RE/08-creature.md) |
| 4 | Movement: move types, `MoveTypeData`, HyperMove, Glide, MoveListMaker, SendWalkPacketReason | [06](RE/06-movement.md), extras in [17 §4.4](RE/17-open-items.md) (correction: the client sends `G_MoveReqCmd` 64, not `GS_ActionsClientMoveCmd`) |
| 5 | Summoner cat (`BSummonedBrain` `0x90f140`, `0x968f30`, action and brain enums) | [08 §5](RE/08-creature.md), data fields in [17 §4.4](RE/17-open-items.md) |
| 6 | Mounts, vehicles, grabs; false vehicle names | [17 §4.5](RE/17-open-items.md) (callers of the 3 false names now checked) |
| 7 | Fishing | [17 §4.6](RE/17-open-items.md) |
| 8 | Datafile system | [13](RE/13-datafile.md), [`client-datafiles.md`](client-datafiles.md) |
| 9 | Skateboard / horse ideas | [17 §4.7](RE/17-open-items.md) |
| 10 | Open items | [17 §3.4](RE/17-open-items.md) (state of each old item) and [`RE/17-open-items.tsv`](RE/17-open-items.tsv) (all 509 items) |

---

## 3. Key facts for the server port (short form)

| Fact | Source |
| --- | --- |
| Use 381 game command ids. Ids 1–918 are the same in 419 GM; above 918 they are not. | [`RE/game-commands.tsv`](RE/game-commands.tsv), [17 §4.1](RE/17-open-items.md) |
| Wire: `u16 EncHeader` envelope, then plain `int16 size` (header included), `int16 command`, body. | [02 §2](RE/02-transport.md) |
| Field codes of the game packet descriptors (T_INT8 … T_PACKET_REF) give the wire order. | [05 §3](RE/05-game-dispatch.md), [`RE/packet-layouts.tsv`](RE/packet-layouts.tsv) |
| In 381 all 18 sync ids share one lock. Answer each sync request, or the client blocks other sync requests for 10 s. | [05 §4](RE/05-game-dispatch.md) |
| Moves arrive as `G_MoveReqCmd` 64 (88-byte packet), positions as server units. | [06 §2](RE/06-movement.md) |
| 381 has no vehicles. Mount links, catch and inhalation exist. | [17 §4.5](RE/17-open-items.md) |
