# Card gmt-1: GM tools and Builder tools, full reverse engineering

## Goal
Write a complete book about the developer and GM tools of the client and the server: the in-game debugger, all debug commands, all Builder commands, the Builder network protocol (client and server), and the data-table pipeline (XML load, reload, save). The result must be complete enough to use these tools against our server and to port the server side to Rust.

This card is deep work. Thoroughness is more important than speed. Do every topic. Where a topic has a list (commands, packets, panels, files), export the **full** list with a script. Do not stop at examples.

## Context (verified facts, source: `Docs/DataFile/11-gm-dev-tools.md`)
- GM client `Client_gm.exe` (2021, branch 419NC) has full PDB names. Debugger sources are in `client2\debugger\src\` (wxWidgets). Classes: `DataBrowser`, `DataBrowserEditingDialog`, `GameEditPanel`, `GameEditModel`, `GameEditCommandProcessor`, `GameEditNetObjectEventHandler`, `BuilderCommandPanel`, `BuilderCommandHandler`, `BuilderInspectorViewer`, `BuilderShortcutEditPanel`, `BuilderData`, `QuestBrowser`, `SkillViewer`, `TeleportViewer`, `ObjectBrowser`, `SummonBrowser`, `InventoryBrowser`, `HateViewer`, `WatchBrowser`, `MacroEditPanel`, `LogViewer`, `ScriptTraceLogViewer`, `ClientDebugger`, `ContentsDebugger`, `PanelManager`.
- `DataManager::Load` `0x6ab580`: binary first, XML fallback, then validators. `DataManager::Reload` `0x6abca0`. `BNSClient::HandleBuilderCommand` `0x1426290` type `0x12` = local table reload from `..\contents\local\<locale>\data\`.
- `DebugCommandDefList` initializers: `0x16a4710`, `0x16b2ed0`, `0x16c17e0`, `0x16d02d0`. Examples: `reload-table` 0x4C, `reload-localize-table` 0x4D, `auto-refresh-table` 0x4E, `save-table` 0x50. Handlers not found yet (not in `BNSClient::HandleDebugCommand` `0x1469540`). About 95 classes have a `HandleDebugCommand` method.
- `GameEditCommandProcessor` `0x651d70`–`0x652bd0`: `ProcessBuilderCommand`, `ProcessDebugCommand`, `ProcessBuilderCommandResponsePacket`, `ProcessPacketTestCommand`, handler sets.
- `GameBuilderSession`: about 155 symbols, `Handle<Name>OkPacket` / `Handle<Name>FailPacket` pairs.
- `DataBrowser::OnContextEdit` `0x64cf10` changes records in memory only.
- `DrDataReaderImpl::SaveBinaryDataFile` `0x789a90` (vtable +0x40) is linked.
- Server, GameDaemon 348 PDB names: `BuilderSessionHandler`, `VBuilderSessionHandler`, `GameBuilderPacket`, `GameBuilderPacketHandler(List)`, `NotAuthedGameBuilderPacketHandlerList`, `AuthedGameBuilderPacketHandlerList`, `AnyAuthedGameBuilderPacketHandlerList`, `gameBuilderPacketVersion`, `GameBuilderMainCommand`, `UseBuilderCommand`, `AcquireByBuilderCommand`.
- Server config: `BnS_2020/config/GameDaemon/bin/gamed.config.xml` line 82 `builder-port` = 40100, line 306 `builder-log-enable`. `BnS_2020/config/AdminWeb/Web.config` line 56 `use-builder-level-QA`.
- Builder level is a per-character DB field `CreatureProperty.Builderright`, range -1 (QA) to 7. AdminWeb sets it through ManagementDaemon `ChangeBuilderReq` (`Docs/Server/ManagementDaemon/01-overview.md`). Find where GameDaemon reads it and which level each Builder command needs.
- The GM client is build 419 (386 tables). Our data is build 381 (362 tables). The 2020 `Client.exe` has no PDB.

## Inputs
| What | Path | Use |
| --- | --- | --- |
| GM client 2021 (names, types) | `/home/ubuntu/Documents/Codex/private-project/ida/lanes/gm/Client_gm.exe.i64` | main source |
| Client 2020 (no PDB) | `/home/ubuntu/Documents/Codex/private-project/ida/lanes/gm/Client2020.exe.i64` | what is left in our client build |
| GameDaemon 348 (PDB names) | `/home/ubuntu/Documents/Codex/private-project/ida/lanes/gm/GameDaemon348.exe.i64` | server side, names |
| GameDaemon 2020 (no PDB) | `/home/ubuntu/Documents/Codex/private-project/ida/lanes/gm/GameDaemon2020.exe.i64` | server side, parity target |
| Server config (read only) | `BnS_2020/config/GameDaemon/bin/gamed.config.xml`, `BnS_2020/config/AdminWeb/Web.config` (paths under `/home/ubuntu/Documents/Codex/private-project/`) | builder options |
| Earlier findings | `Docs/DataFile/11-gm-dev-tools.md`, `Docs/DataFile/book/05-client-loader.md`, `09-tools-and-editing.md` | use `tools/spec_index.py` |
| Server session spec | `Docs/Server/GameDaemon/10-session-handlers.md` | packet framing to compare |

You may read the 2 config files above. Do not copy passwords, keys or connection strings from them. Write `keyhere`.

## Work
Use all 4 databases. Open them in one turn and set up each with `boot(WT, stservers=False)`. Write each verified fact into the book at once.

1. **Inventory.** Export every class and function of `client2\debugger` and of `GameBuilderSession` with a script (PDB names). Write `inventory.tsv`: class, function count, main functions, source file. Do the same for the server Builder classes in GameDaemon 348.
2. **Debugger panels.** For each panel class: how it opens (command, key, menu), what it shows, which data it reads (table, game object, packet), which commands it sends, and what it saves (`ClientDebuggerData.xml`, `BuilderData.xml`, registry through `RegistryUtility`). One section per panel. Give the format of each saved XML file.
3. **Debug commands.** Decode the 4 `DebugCommandDefList` initializers with a script. Find the struct layout of one entry (`_command`, `_debuggerCommandType`, `_commandMethod`, `_handler`, `_argument[]`: `_argumentType`, `_candidateArgumentType`, `_argumentDescription`). Write `debug-commands.tsv` with **all** entries: name, type id, arguments with types and descriptions, handler class and address, effect in one line, local only or sends a packet. Find the dispatch path from typed text (`Commander`, `CommandAutoComplete`, `ChatInputPanel`) to `ProcessDebugCommand` and the handler sets. Find the handlers for 0x4C–0x50.
4. **Builder commands.** Do the same for the Builder command list (`BuilderCommandListDefine`, `BuilderCommandDefList`, `BuilderLevelStrings`, `BuilderShortcutTypeStrings`). Write `builder-commands.tsv` with **all** entries: name, type id, arguments, required Builder level, client handler, packet sent, server handler. Explain Builder levels and how the server checks them.
5. **Builder protocol.** Client side: how `GameBuilderSession` connects (address and port source, when it connects), the handshake, `gameBuilderPacketVersion`, authentication, packet framing, encryption (compare with the game session), and every request and response packet. Server side (GameDaemon 348, then confirm in 2020): the listener on `builder-port`, `BuilderSessionHandler`, the not-authed/authed/any-authed handler lists, the auth check (account, IP, GM level, `use-builder-level-QA`), `GameBuilderMainCommand`, `UseBuilderCommand`, `AcquireByBuilderCommand`, and the `builder-log-enable` log. Write `builder-packets.tsv`: id, name, direction, fields (offset, type, name), client handler, server handler, auth list. Give a hex example of the handshake with `keyhere` for secret bytes.
6. **GameEdit and packet test.** `GameEditModel`, `GameEditPanel`, `GameEditNetObjectEventHandler`, `DataBrowser::ProcessGameEditNetObjectEvent` `0x61c460`, `ProcessGameEditActionSubEvent`, and `ProcessPacketTestCommand` / `SetPacketTestLink`. Find out which session they use and what they can change live.
7. **Data pipeline.** Complete the table flow: `Load`, `Reload`, piece-by-piece reload, alias reload, `constructDataOnlyForXML`, the hidden-field loaders, all validators (list each check), `reload-localize-table`, `auto-refresh-table` (file watcher or timer?), and the `save-table` handler and its output format (same as retail `datafile.bin`?). Export every XML file mask with its table to `xml-files.tsv`. Find the switch that selects XML mode (command line, ini, file presence).
8. **How the tools are switched on.** All gates: build flags, command-line arguments, ini keys, GM or Builder level from the server, the debugger window key, and checks that are compiled out of retail builds.
9. **Client 2020.** For each feature in topics 2–8, find whether it exists in `Client2020.exe.i64`. Match by strings and code shape. Write `client2020-map.tsv`: feature, GM address, 2020 address or "absent", and the evidence.
10. **Use with our server.** Write a practical chapter: the steps to use the GM tools as a live data editor and a Builder console against bns-server, with the gaps for each step (version 381 against 419, missing server side, client patches that are needed).

## Rust card
Write `Docs/Plan/cards/gmt-1-rust.md` with `Docs/Plan/card-template.md`, early (after topic 5). Scope: a Builder listener in `Program/Server/GameDaemon` (config `builder-port`, framing, handshake, auth lists, dispatch table, Ok/Fail responses, and 5 simple commands from `builder-commands.tsv`). Give exact spec line ranges, signatures and tests. Run `python3 tools/rust_api.py Program/Server/GameDaemon` for the interfaces. Keep it to one Sonnet session; list the remaining commands in the handoff.

## Budget
- This card overrides the IDA call target in `spec-rules.md`. Use up to 250 IDA calls. All other context rules stay.
- Export lists with scripts. Write long lists into `.scratch/` first, then into the TSV files. Do not print long lists in the conversation; print counts and 20 sample rows.
- At each stage end, list the topics that are done and the next exact step in the progress note.

## Style
- Book style: a "Quick find" table first, ASCII diagrams, a source (address or function) for each fact, ❓ for facts that you did not verify.
- Each `.md` file stays under 40,000 bytes. Split a chapter if needed.
- Abuse cases (for example a Builder command that works without auth): write them in `Docs/Plan/abuse-notes/gmt-1.md` only, not in the book.

## Files
- `Docs/Client/GmTools/` (all chapters and TSV files; `00-index.md` first)
- `Docs/DataFile/11-gm-dev-tools.md` (add a link to the new book; correct facts that topic work disproves)
- `Docs/Plan/cards/gmt-1-rust.md`
- `Docs/Plan/handoffs/gmt-1.md`
- `Docs/Plan/abuse-notes/gmt-1.md`

## Out of scope
- Do not change files under `Program/`.
- Do not patch binaries. Do not save the `.i64` files unless names help later topics (then save only the lane copies).
- Do not copy Hex-Rays code, table data or game text beyond small examples.
