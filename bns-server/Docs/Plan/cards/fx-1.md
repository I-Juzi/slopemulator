# Card fx-1: GM debugger and builder UI (419 GM) — port study

## Goal
Find out how to bring the original GM tool UI of the 2021 GM client (419, UE4 build) into the 2020 client (381, UE3). Write chapter `Docs/Client/RE/20-gm-tools.md`. It must say which port path works and what it costs.

## Context
- Facts so far: `notes/client-exe-reversing.md` §12 is not in this repo. Summary: the 419 GM exe has 21 source files under `Program\client2\debugger\src\` (ClientDebugger, PanelManager, BuilderCommandPanel, BuilderCommandHandler, BuilderData, Commander, ContentsDebugger, DebugCommandHandler, GameEditCommand, GameEditCommandProcessor, GameEditPanel, HateViewer, InventoryBrowser, BattleObjectBrowser, LobbyWindow, MacroEditPanel, ShortcutEditPanel, PanelManager, RegistryUtility, UtilFuncs, Candidate). Titles `- Builder Command -`, `- Builder Inspector -`, `-- Contents Debugger --`. Files `debug\BuilderData.xml`, `debug\ClientDebuggerData.xml`. Sessions `GameBuilderSession`, `GameDebugSession`. No `CreateWindowEx` import name was found, so the panels are probably drawn in the game UI system ❓.
- 381 has none of these strings.
- Client chapters `Docs/Client/RE/` (cx-00..cx-17): UI framework is chapter 14.

## Inputs
| What | Path |
| --- | --- |
| 419 GM client with PDB (main) | `/home/ubuntu/Documents/Codex/private-project/ida/lanes/cx/Client_gm.exe.i64` |
| 381 client | `/home/ubuntu/Documents/Codex/private-project/ida/lanes/cx/Client.exe.i64` |

## Work (3 topics)
1. Render layer: how the debugger panels draw (UE4 Slate/UMG, the game UI system, or own code). Give the base classes, the panel-to-engine calls and the asset or layout files each panel needs.
2. Logic layer: for each of the 21 debugger classes give functions, addresses, the commands it sends (GameDebugMainCommand 214, GameBuilderMainCommand 212: write `Docs/Client/RE/gm-commands.tsv` with id, name, fields, sender function) and the GameBuilder/GameDebug session protocol (connect, auth, ping).
3. Port paths, each with steps, risks and an effort estimate in slices:
   a. Run the 419 GM client itself against our server as a GM console (what the server must add: 419 packet table, builder and debug sessions).
   b. Port the panel logic into a 381 plugin DLL and draw the panels with the 381 UI system (map each 419 UI call to a 381 call; use chapter 14).
   c. Any other path you find.
   Give a recommendation.

## Lookup design
- Start with a "Quick find" table: question → function → address (419 and 381 where a match exists) → section.
- Give an address for every name. Each `.tsv` has a header row and an address column.

## Detail level
- Cover every function of the 21 debugger classes. If you run out of turns, list what is left with addresses in the handoff.

## Files
- `Docs/Client/RE/20-gm-tools.md` (new)
- `Docs/Client/RE/` (TSV files)

## Out of scope
- No Rust card for this slice. Write `keyhere` for every key (the builder auth has an INVALID_KEY reason). Do not change files under `Program/`.
