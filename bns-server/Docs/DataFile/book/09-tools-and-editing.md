# 9. Tools and editing

This chapter lists the tools that read and write the data files. It tells how to change a file safely and how to verify the result. Section 9.6 is a step-by-step walkthrough: change one value in one table.

## Quick find

| Question | Section | Address, file or API |
| --- | --- | --- |
| Which public tools exist, and can we use their code? | 9.1 | journal `Docs/Client/client-datafiles.md` |
| Which tools are ours? | 9.2 | `Program/Tools/DataFile/*`, `Program/Tools/BnsEdit` |
| How do I prove that a tool reads and writes a file correctly? | 9.3 | `bnsdata` `tests/real_files.rs`; `bnsmodel` test `model_open_search_pack` |
| What does the client check when it loads a file? | 9.4 | `0x624440` (5.8) |
| Which rules keep an edit safe? | 9.5 | — |
| Change one value in one table, step by step | 9.6 | `Project::open_with_schemas` … `pack` |
| How do I put the file into the client, and how do I undo it? | 9.7 | `BnsEdit` `install::install` / `install::restore`; `binreplace.ini` |
| How do I change server data? | 9.8 | `lmxb::decode` / `encode` |

## 9.1 Public tools

We use the public tools to cross-check our results only. We do not copy their code.

| Tool | Author | Language | Licence | What it does | Status | Our use |
| --- | --- | --- | --- | --- | --- | --- |
| BnsBinTool | notscuffed | C# | none | dump, compile, transform to XML; `-64` for x64; uses ISA-L deflate; has its own definition format (one TableDef folder for each version); no AES code | last commit 2022-04-13; archived 2022-09-14 | cross-check of the container and bodies |
| BnsPluginTables | leanleon93 | C++ | "Non-Commercial, No Monetization" | C++ structs for the NEO client (EU/KR), 528 tables | later than BNSR_116 | cross-check only: 926 offsets match our BNSR_116 export in 73 tables |
| BnsTools | war100ck | C# WPF | none | edits plain server XML; `BNSDat.cs` / `BDat.cs` read old `.dat` files | 2024-04-09 | none |

Source: journal `Docs/Client/client-datafiles.md` ✅ (repository read, not run).

No licence means: we have no right to copy the code. Read it to compare results. Write our own code from the binary and the files.

## 9.2 Our tools

```text
          Client.exe / BNSR_116.exe
                 |  bnsschema (export TableDefs JSON)
                 v
  schema folder (x86, x64) ----+
                               v
  datafile*.bin  ---> bnsdata (container, bodies, records, strings)
  localfile*.bin           |
                           v
                      bnsmodel (typed fields, edits, undo, pack)
                           |
                           v
                      BnsEdit (GUI: search, edit, rules, install, restore)

  GameDaemon/data/*.xml ---> lmxb (LMXBOSLB) ---> gamedata (12 kinds, links)
```

The crate APIs below come from `python3 tools/rust_api.py <crate dir>`.

### 9.2.1 `bnsdata` (`Program/Tools/DataFile/BnsData`)

| Module | Main items | Use |
| --- | --- | --- |
| `dat` | `Width`, `Role`, `Header`, `Table`, `Container::parse(bytes, width, role)`, `replace_tables`, `write` | the `TADBOSLB` container (2.5) |
| `table` | `Record`, `RecordHeader`, `TableBody::parse(bytes, width, kind, def)`, `write`, `record_count`, `discard_original` | table bodies (ch. 3) |
| `fields` | `decode_fields_with_pool`, `any_text_with_pool` | typed field read |
| `scalar` | `set_scalar(def, name, repeat, ScalarValue)` | set int, float, bool, enum or ref |
| `text` | `edit_record`, `RecordEditor::set_text`, `set_text_str` | change a string in a pool |
| `tabledef` | `TableDefinition::from_json`, `validate`, `layout`; `Definitions::load_folder`, `get` | load a schema folder |
| `lmx` | `load_key`, `Document::parse(bytes, &Key)`, `binary_size`, `write(&Key)`, `walk` | LMXBOSLB (second reader; see 9.2.4) |

Environment variables of the real-file tests: `BNSDATA_CLIENT_DAT`, `BNSDATA_DAT_WIDTH` (32 or 64), `BNSDATA_DAT_ROLE` (`main` or `local`), `BNSDATA_TABLEDEFS`, `BNSDATA_SERVER_XML`, `BNSDATA_KEY_FILE`. The key file holds 16 raw bytes or 32 hex characters (`keyhere`; `Docs/DataFile/01-lmxboslb.md`).

### 9.2.2 `bnsmodel` (`Program/Tools/DataFile/BnsModel`)

`bnsmodel` is a typed edit layer on `bnsdata`. It has no GUI (`Docs/DataFile/09-model.md`).

| API | What it does |
| --- | --- |
| `Project::open(folder, width, schema)` | reads the four standard file names |
| `Project::open_with_schemas(folder, width, schema32, schema64)` | the same, with one schema folder for each width |
| `tables()`, `record(table, index)`, `record_key`, `record_title`, `search(table, query, limit)` | find and read records |
| `validate(edit)` | checks the old value, the storage type and width, and schema rules |
| `apply(edit)` | changes the value in both widths (the "mirror") |
| `undo()`, `redo()`, `changes()`, `discard(change)` | edit history |
| `problems()` | checks all fields of each changed record |
| `pack(out_dir)` | writes all files to a new folder; returns a `PackReport` |

Values: `Value` = Int, Float, Bool, Enum, Ref, Text, Native, Raw. `Raw` fields are read-only.

### 9.2.3 `bnsschema` (`Program/Tools/DataFile/BnsSchema`)

Command: `bnsschema EXE OUTDIR [--array 0xVA]`. It finds the TableDefs array in an exe (`find::find`, at most 4096 slots) and writes one JSON definition for each table (`export`). Layouts: `X86` (381) and `X64P4` (2026). `protect::check` refuses an exe with packer sections (for example `.themida`, `.winlice`, `.vmp0`). Test variables: `BNSSCHEMA_EXE`, `BNSSCHEMA_REF`. See 4.8.

### 9.2.4 `lmxb` and `gamedata` (server data)

`lmxb` reads and writes LMXBOSLB v3: `decode`, `encode`, `to_xml_string`, `element_count`, `MAX_DEPTH` = 128. Tools: example `inspect`, example `survey`, binary `lmxb-dump`. Test `real_data` with `BNS_GAMEDATA` (`Docs/Server/GameDaemon/01-lmxboslb.md`).

Two LMXBOSLB readers exist: `lmxb` and `bnsdata::lmx`. They differ (journal table "BnsData vs Lmxb"): depth limit 256 vs 128, and `bnsdata::lmx` takes the key as an argument. `lmxb` is the reader that the server uses. `gamedata` builds the world from `lmxb` documents (8.5.2).

### 9.2.5 `BnsEdit` (`Program/Tools/BnsEdit`)

`BnsEdit` is a GUI (egui) on `bnsmodel`.

| Part | What it does |
| --- | --- |
| `real::Real::open(OpenSpec)` | opens a game folder with a schema |
| `labels::KINDS` | short lists: Items (`item`), NPCs (`npc`), Skills (`skill3` + `skill`), Text (`text`) |
| `rules` | bulk edits. Test: `Is`, `IsNot`, `AtLeast`, `AtMost`, `Has`. Change: `Set`, `Percent`, `Add` |
| Packs | export and import a change set as JSON |
| Schema export | writes `schemas/<stem>-<stamp>` |
| Pack folder | `<game>/bnsedit-backup/pack-<stamp>` |
| `install::FILES` | the 4 `.bin` files |
| `install::install` | copies the originals to `<game>/bnsedit-backup/` one time only, then copies the packed files into the game |
| `install::restore` | copies the backups back |

Source: `tools/rust_api.py Program/Tools/BnsEdit` ✅ (API names). The GUI flow was not run ❓.

## 9.3 Proof that a tool is correct

A tool is correct for a file if it reads the file and writes the same bytes back (a round trip). Then a change to one value changes only the bytes of that value and the sizes that depend on it.

| Test | Crate | Proves |
| --- | --- | --- |
| `client_roundtrip` | `bnsdata` | container parse + write gives the same bytes |
| `client_table_bodies_roundtrip` | `bnsdata` | each table body parse + write gives the same bytes |
| `client_strings_resolve` | `bnsdata` | all string offsets point into the pools |
| `server_roundtrip` | `bnsdata` | one LMXBOSLB file, decode + write |
| `real_data` | `lmxb` | 21,064 files, 0 decode failures, 0 round trip failures |
| `model_open_search_pack` | `bnsmodel` | open, search, edit, pack, reopen |
| `edit_pack_reopen` | `bnsmodel` | item price + 1, undo, redo, pack, reopen in both widths |

These tests are `#[ignore]`. They need operator files. Example command (`Docs/DataFile/STATUS-model.md`):

```sh
BNSDATA_CLIENT_DAT=<folder> BNSDATA_DAT_WIDTH=32 BNSDATA_DAT_ROLE=main \
BNSDATA_TABLEDEFS=<schema folder> \
cargo test -p bnsmodel --test real_files model_open_search_pack -- --ignored
```

Our tests prove that the files are correct for our reader. They do not prove that the client accepts a changed file. Client acceptance is ❓ (`09-model.md`; part 1 open question 5).

## 9.4 What the client checks on load

From `0x624440` (5.8):

| # | Check | If it fails |
| --- | --- | --- |
| 1 | buffer size ≥ header size (99 B x86) | the file fails |
| 2 | local file: the main file was read first | the file fails |
| 3 | signature `TADBOSLB` and version 5 (`IsCorrectHeader` `0x624130`) | the file fails |
| 4 | main file: the alias block reads | the file fails |
| 5 | each table: unknown type | the table is skipped |
| 6 | each table: version ≠ compiled `DrTableDef` version | **the whole file fails** |
| 7 | each table: kind not 1 or 2 | the table is skipped |
| 8 | each table: the body does not read | the table is freed; the client tries XML (5.6) |

The client does **not** check: a hash, a signature, `_gameVersion` (7.3), `_saveTime`, `_totalTableSize`, `_maxBufferSize` ✅ (5.8; 2.5.2). The RSA signature is on the `.dat` archive only (2.4).

```text
 header (99 B)        alias block        table frames ...
+-------------------+-----------------+--------------------------------+
| TADBOSLB | ver 5  | (main only)     | kind|type|major|minor|size|body|
+-------------------+-----------------+--------------------------------+
   must stay            must read          major.minor must stay
```

## 9.5 Rules for a safe edit

| # | Rule | Why | Source |
| --- | --- | --- | --- |
| 1 | Use the schema from the same build as the data | a different layout reads wrong fields | 7.6; journal (2026 layout fails on 381 data) |
| 2 | Never change a table version (major, minor) | one wrong version fails the whole file | 5.8 step 8 |
| 3 | Never change the native word `a` | the client uses it as a byte size | 3.7; `09-model.md` |
| 4 | Edit both widths together (x86 and x64) | the 32-bit and 64-bit clients read their own file | 1.4; `bnsmodel` mirror |
| 5 | Keep the bodies of unchanged tables | fewer changed bytes, less risk | `bnsmodel` `pack` |
| 6 | Write to a new, empty folder | `pack` does not overwrite; an I/O error can leave part files | `09-model.md` |
| 7 | Keep a backup of the original files | to restore the game | `BnsEdit` `install` |
| 8 | Do not edit `Raw`, size-0 or deprecated fields | no field view; the client does not read them | 4.3.4; `09-model.md` |
| 9 | Keep text without NUL; required text not empty | schema rule | `09-model.md` |
| 10 | Check refs: a nonzero ref with no target gives a warning | the client can find nothing | `bnsmodel` `problems` |
| 11 | Keep server data in sync for shared values | the server uses its own copy | 8.6 |

The undo of each layer (decompress, decrypt and so on) is in 2.7.

## 9.6 Walkthrough: change one value in one table

Example: raise the price of one item by 1. This is the same change as the test `edit_pack_reopen`. Table `item` (id 159 in 381, version 1.398, key `id`, `level`; `tables.tsv`).

**Step 1. Get the files.** Copy `datafile.bin`, `datafile64.bin`, `localfile.bin` and `localfile64.bin` into a work folder. Do not work in the game folder. In the private client, the files are in `contents/local/NCSoft/data/` (`binreplace.ini`; journal).

**Step 2. Get the schema.** Export the definitions from the same build:

```sh
bnsschema <path>/Client.exe   <work>/schema/x86
bnsschema <path>/Client64.exe <work>/schema/x64    # x64 exe name ❓
```

Or use the operator TableDefs 0.0.381.8876 (7.11). Check that `tables.tsv` shows `item` 1.398.

**Step 3. Open the project.**

```text
project = Project::open_with_schemas(<work>, Width::X86, <schema/x86>, <schema/x64>)
```

The project opens the four files. A missing file is allowed. A present file with no definition is an error.

**Step 4. Find the record.**

```text
t    = the table in project.tables() with the name "item"
hits = project.search(t, "<item name or id>", 10)
view = project.record(t, hits[0])          // list of FieldView
```

Find the price field in `view`. Note its name, repeat index and old value. The exact field name is in the schema JSON ❓ (not checked here).

**Step 5. Make the edit and check it.**

```text
edit = Edit { table: "item", record: hits[0], field: <price field>,
              repeat: 0, old: <old value>, new: <old value + 1> }   // field names ❓
project.validate(edit)    -> must give no error
project.apply(edit)       -> changes the x86 and the x64 view
project.problems()        -> must give no error (warnings are allowed)
project.changes()         -> shows one change
```

`validate` fails if the old value is stale or the new value does not fit the storage type (`09-model.md`).

**Step 6. Pack to a new folder.**

```text
report = project.pack(<work>/out-1)       // out-1 must not exist
```

The report gives the file paths, the changed table names (only `item`) and the byte counts. The other tables keep their original bodies. `_totalTableSize` and `_maxBufferSize` stay as they were (2.5.1).

**Step 7. Verify the output.**

1. Open `<work>/out-1` again with `Project::open_with_schemas`. Read the record. The price must be old + 1 in both widths.
2. Compare the old and new files. Only the `item` frame and the header fields that depend on its size may differ ❓ (exact list of header fields not checked).
3. Check that the `item` frame still has version 1.398.
4. Run the `bnsdata` round trip tests on the new files (9.3).

**Step 8. Install with a backup.** In `BnsEdit`, use install. It copies the originals to `<game>/bnsedit-backup/` one time, then copies the four new files into the game. By hand: copy the originals to a safe place first.

**Step 9. Test in the client.** Start the private client (with `binreplace`). Look at the item in a shop. If the client fails to load data, it shows an error (5.11). Then restore (9.7).

**Step 10. Change the server copy.** If the server also uses the price, change it in the server data too (9.8).

```text
 originals --copy--> work/ --open--> Project --apply--> pack --> out-1/
     ^                                                            |
     |                                    reopen + tests <--------+
     +------ restore <--- bnsedit-backup/ <--- install <----------+
```

## 9.7 Install and restore

| Client | How the client gets the file | Undo |
| --- | --- | --- |
| Private client with `binreplace` | `bin/plugins/binreplace.ini` points `datafile` and `localfile` to `contents/local/NCSoft/data/` | `BnsEdit` `install::restore`, or copy the backups back |
| Retail client without `binreplace` | through the `.dat` archive layers (ch. 2) | the RSA check (2.4) refuses a changed archive without the private key ❓; we do not support this |

Source: journal `Docs/Client/client-datafiles.md` (`binreplace.ini`); 2.4.

## 9.8 Change server data

1. Decode the file with `lmxb::decode`. Or view it as XML with `lmxb-dump` or `to_xml_string`.
2. Change the attribute value in the `Document`.
3. Write it with `lmxb::encode` (version 3). The file size field is set again.
4. Decode the new file again. Compare `element_count` with the old file.
5. Load the folder with `gamedata::World::load_dir`. Check `dangling_references`.

`to_xml_string` output is semantic XML. It is not the original source text (01-lmxboslb.md). Do not use it as the source for a new binary file unless you parse it back with the same tool ❓ (no XML-to-binary path is listed in the API).

## 9.9 Open questions

1. Does the client accept a changed `item` table (9.3)? Not tested in a running client.
2. Which header fields change after `pack` (9.6 step 7)?
3. The x64 exe name and the x64 schema export path (9.6 step 2).
4. The retail path through the `.dat` archive (9.7).
5. The name of the price field in `item` (9.6 step 4).

## Journal (old documents, not changed)

- `Docs/Client/client-datafiles.md`
- `Docs/DataFile/09-model.md`, `STATUS-model.md`, `10-schema-from-exe.md`, `STATUS-schema.md`
- `Docs/Server/GameDaemon/01-lmxboslb.md`
