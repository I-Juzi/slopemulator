# GameDaemon data: step 2 report

| Part | State | Evidence |
| --- | --- | --- |
| Survey | done | `02-tables.md`: 252 types, 21,062 table files, 1,018,183 direct elements; attribute types and reference candidates. |
| Minimum-world chain | partial | `03-minimum-world.md`: spatial chain specified. All 30 level-one settings resolve to zone 1101 and terrain 1001. ❓ Full gameplay dependencies remain open. |
| Typed loader | done | `crates/gamedata`: 12 types; id/alias indexes; initial-setting keys; unknown attributes and children retained; reference diagnostics. |
| Tests | done | 14 generated tests; separate real-folder test passes; unset `BNS_GAMEDATA` prints `SKIP`. |
| Workspace checks | done | 290 passed, 0 failed, 6 ignored. Clippy passes with `--workspace --all-targets -- -D warnings`. |

Opt-in run: **12 table types, 10,608 files, 138,440 indexed records**.
Skipped: 10,454 other table files, 2 other binary roots, 1 plain XML file.

| Loaded table | Records |
| --- | ---: |
| PcInitialSetting | 93 |
| zone | 1,533 |
| Terrain | 164 |
| ZonePcSpawn | 2,951 |
| ZoneRespawn | 1,652 |
| ZoneArea | 76,936 |
| ZoneEnv | 98 |
| ZoneEnv2 | 11,626 |
| ZoneEnv2Place | 11,682 |
| ZoneEnv2Spawn | 11,832 |
| script | 17,263 |
| quest | 2,610 |

Reference checks report **8,500 unresolved candidate links**: 6,217 script, 1,974 placement, 175 environment-template, 83 zone, 33 player-spawn, 16 phase-zone, and 2 respawn links.
Three extra script-table elements remain in file metadata. The survey counts them; the script index does not.
❓ Open questions: missing-link causes, placement scope, initial spawn selection, zero defaults, coordinate units, and terrain geometry.
Next job: resolve reference naming and scope from bytes. Extend dependency checks. Specify initial spawn selection before server integration.
No game records were added. No IDA or network tools were used. Downloads and `tools/emu/` were not changed.

## Step 1 evidence

# GameDaemon data: step 1

| Part | State | Evidence |
| --- | --- | --- |
| Header | done | 21,064 v3 files; exact size fields; 64 zero reserved bytes. See lobbygate-rs/spec/gamed/01-lmxboslb.md:1. |
| Body | done | 1,786,132 elements; 2,202,819 text nodes; maximum depth 7. Strings use UTF-16LE and a repeating 16-byte XOR. |
| Decoder and encoder | done | New dependency-free `lmxb` crate. Bounds checks, metadata retention, XML escaping, and malformed-input tests pass. |
| Round trip | done | Generated data and all 21,064 binary files pass byte-exact round trips. |
| XML dump | done | `lmxb-dump <file>` prints XML. Generated CLI tests pass. |
| Workspace checks | done | 275 passed, 0 failed, 6 ignored. Workspace Clippy passes with `--all-targets -- -D warnings`. |

## Opt-in results

- Found 21,065 XML paths. Decoded 21,064 LMXBOSLB files.
- Read 1,674,199,807 binary bytes. Counted 1,786,132 elements.
- Decode failures: **0**. Byte-exact round-trip failures: **0**. XML render failures: **0**.
- Skipped `accountleveldata.xml`: verified plain UTF-8 XML with a BOM.
- Binary groups: root 1,098; quest 2,610; script 401; social 88; zone 16,867.
- Without `BNS_GAMEDATA`, the opt-in test prints `SKIP`. The normal test total includes that test.

## Open questions and limits

- ❓ The meanings of the opaque flags and reserved header fields remain unknown.
- ❓ The source-line field meaning is inferred. No source XML pair was available.
- ❓ Other format versions and node kinds were not verified.
- The decoder accepts version 3 and at most 128 node levels. Encoded documents must fit a u32 file size.
- No game data was added to the repository. `lobbygate-rs/tools/emu/` was not changed.

## Next job

Write a data-loader specification from decoded table metadata. Start with zone and quest tables. Define typed Rust records and reference checks. Keep plain XML handling separate from this binary codec.
