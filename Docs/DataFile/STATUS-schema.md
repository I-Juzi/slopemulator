# bnsschema status

Date: 2026-11-07.
Job: bnsedit version 1, part 2.

## Result

The implementation and specification are written.
Shell verification is blocked. No build or test result is claimed.

| Part | State | Evidence |
| --- | --- | --- |
| Reference review | Complete | Read `07-client-exe-tabledefs.md`, `exportdefs.py`, `pe.py`, `findarr.py`, and `arr64.py`. |
| Workspace | Updated | Added `Program/Tools/DataFile/BnsSchema` to the member list. No other crate was changed. |
| PE parser | Written; not compiled | `src/pe.rs`: DOS and PE signatures, machine, width, image base, entry point, section fields, full-section entropy, and checked VA reads. |
| Protection gate | Written; not tested | `src/protect.rs`: all listed names, entry-point rules, high entropy, and virtual-only executable sections. |
| Layouts | Written; not tested | `src/layout.rs`: `x86` and `x64p4` const structs from the Python reference. |
| Array detection | Written; not tested | `src/find.rs`: initialized non-executable sections, pointer alignment, slot 0 null, consecutive signed IDs, safe UTF-16 table names, longest run, automatic minimum 16, explicit minimum 1. |
| JSON export | Written; not tested | `src/export.rs`: reference keys and value reads; known type names; `type_N` fallback; sorted unknown code set. |
| Public API | Written; not compiled | `Found`, `Width32or64`, `Error`, `inspect`, and `export`. Protection precedes all schema reads and output. |
| CLI | Written; not compiled | `bnsschema EXE OUTDIR [--array 0xVA]`. Summary or error reason. |
| Synthetic tests | Written; not run | Eleven active tests. Both widths, exact JSON, folder load through `bnsdata`, automatic detection, longest run, scan section flags, protection rules, output conflicts, unknown types, strings, counts, and malformed input. |
| Malformed input tests | Written; not run | Truncate both small fake PE files at every byte length. Mutate each used byte. Check invalid pointers, UTF-16, names, signed counts, and non-finite floats. |
| Real-file test | Written; ignored; not run | Reads `BNSSCHEMA_EXE` and `BNSSCHEMA_REF`. Compares table count and per-table base-plus-subtype attribute count by ID. Prints counts only. |
| Specification | Complete | `10-schema-from-exe.md`: exact refusal strings, layouts, JSON contract, limits, API, CLI, and test limits. |

## Command results

All commands used `CARGO_NET_OFFLINE=true`.
All commands targeted the workspace manifest by absolute path.

| Requested command | Result |
| --- | --- |
| `cargo build --locked` | Blocked: `This command requires approval`. Did not run. |
| `cargo test -p bnsschema --locked` | Blocked: `This command requires approval`. Did not run. |
| `cargo clippy -p bnsschema --all-targets --locked -- -D warnings` | Blocked: `This command requires approval`. Did not run. |
| `cargo fmt -p bnsschema` | Blocked: `This command requires approval`. Did not run. |
| Workspace `git status --short` | Blocked: `This command requires approval`. Did not run. |

The main session must run the checks.
The build, test, and Clippy requests used `--locked` to prevent a write outside the permitted scope.
`Cargo.lock` was not changed. The main session must add the new local package entry if Cargo requires it.
Formatting remains pending. No checks were retried after the shell blocked them.

## What is proved

No Rust behavior is proved by executed tests in this session.
The source files and test cases exist.
The reference exporter has prior real-file results in `07-client-exe-tabledefs.md`.
Those results do not prove this Rust port.

## Important contract details

| Detail | Behavior |
| --- | --- |
| Protection refusal | Normal `Error::Refused(String)` result. No schema scan or output follows it. |
| Array count | Excludes null slot 0. Maximum 4,096 total slots. |
| Optional data | Unmapped optional pointers become JSON `null`, as in the reference. Mapped malformed data returns an error. |
| Reference `maxid` read | Reads `u32` at offset 20 or 28, exactly as `exportdefs.py` does. It does not read the whole wider source field. |
| Entropy | Uses all raw bytes in each section, not the Python helper's sample limit. |
| Extra input limits | Maximum 4,096 UTF-16 units per string. Counts follow their field widths. Export work has a 2,000,000-entry budget. |
| Invalid JSON floats | Non-finite f32 values return a format error. |
| Output safety | Safe lowercase table names only. Duplicate names fail. Preflight rejects existing files. `create_new` prevents replacement after a race. |
| Partial output | An I/O error can leave files already created. The tool does not delete them. Use a fresh folder to retry. |

## Not verified

- Compilation, formatting, Clippy, and test success.
- Automatic detection or export on a real executable.
- Full JSON equality between this Rust port and the Python reference on real input.
- A future PE32+ client's use of the `x64p4` layout.
- Protection detection beyond the listed conservative rules.

## Data and scope

No real executable was read. No game data was added.
No executable was run, loaded, mapped as code, patched, unpacked, decrypted, or emulated.
No network or IDA tool was used. No subagent was used.
Only the new crate, the datafile specification folder, and the workspace member list were changed.
The write scope excludes `notes/client-datafiles.md`. No change was made there.

## Main-session verification

Run from `bns-server`:

```sh
CARGO_NET_OFFLINE=true cargo build
CARGO_NET_OFFLINE=true cargo test -p bnsschema
CARGO_NET_OFFLINE=true cargo clippy -p bnsschema --all-targets -- -D warnings
CARGO_NET_OFFLINE=true cargo fmt -p bnsschema
```

Keep the real-file test ignored unless the operator supplies both environment paths.
