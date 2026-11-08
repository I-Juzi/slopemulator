# bnsedit

Find, change and pack client data files (`datafile.bin`, `datafile64.bin`, `localfile.bin`, `localfile64.bin`).

## Build

| Target | Command (in `apps/bnsedit`) | Output |
| --- | --- | --- |
| Linux | `cargo build --release` | `target/release/bnsedit` |
| Windows, on Windows | `cargo build --release` | `target\release\bnsedit.exe` |
| Windows, from Linux | `rustup target add x86_64-pc-windows-gnu`, install `gcc-mingw-w64-x86-64`, then `cargo build --release --target x86_64-pc-windows-gnu` | `target/x86_64-pc-windows-gnu/release/bnsedit.exe` |

## Use

```
Game setup ──► Change one thing ─┐
           └─► Change many  ─────┼─► My changes (packs) ──► Build & Play
                                 │      on / off, share       back up, pack, put into game
```

1. Click **Game setup**. Pick the game folder (the folder with `datafile.bin`).
2. Set the 32-bit and the 64-bit game layout. Click **Read from Client.exe…** / **Read from BNSR.exe…**, or **Use a schema folder…**. The layout must come from the same game version as the data files.
3. After a good read, click **Save this layout as files…** to keep a copy of the layout.
4. **Change one thing**: search, pick a result, change fields with the buttons. **Show all N fields (for experts)** shows the code names.
5. **Change many**: make a rule (for example: every Item where Level is 50, change Sell price, make it 50 % of now). Click **See what it changes**, then **Add N changes**.
6. **My changes**: each pack is like a mod. Turn it on or off, rename it, share it as one `.json` file, or load a shared pack. New changes go into the pack marked **ADDING HERE**.
7. Click **Build & Play**. Close the game first. bnsedit checks, backs up, packs and puts the files into the game.
8. If the game does not accept the files: click **Put the original files back**.

Undo and redo: the header buttons, or Ctrl+Z / Ctrl+Y.

Command line: `bnsedit [GAME_FOLDER] [--schema32 DIR] [--schema64 DIR]`.

## Files that bnsedit writes

| Path | What |
| --- | --- |
| `<game>/bnsedit-backup/` | Original game files, copied once at the first install. Never overwritten. |
| `<game>/<datafile…>.bin` | Replaced by **Build & Play** (temporary name, then rename). |
| Linux `~/.config/bnsedit/`, Windows `%APPDATA%\bnsedit\` | `settings.json` (last folders), `packs.json` (your change packs) and `schemas/` (layouts read from exe files). |

## Limits

- Memory: about 3.5 GB with all four files open.
- A layout from another game version does not open the files. bnsedit says so.
- Protected (packed) exe files are refused. bnsedit never unpacks, runs or patches an exe.
- Client acceptance of repacked files is not tested yet.

## Tests

- `cargo test` (unit tests, install/backup test).
- Real files, operator only: `BNSEDIT_GAME=... BNSEDIT_SCHEMA32=... BNSEDIT_SCHEMA64=... cargo test --release -- --ignored --nocapture`.
- Screenshots: `BNSEDIT_SCREEN=home|find|many|changes|setup` opens that screen, `BNSEDIT_QUERY=text` fills the search box, `BNSEDIT_PICK=1` selects the first result.
