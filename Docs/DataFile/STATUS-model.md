# Main-session verification (2026-11-07)

| Check | Result |
| --- | --- |
| Build, 15 synthetic tests, Clippy `-D warnings`, fmt | ✅ after 1 Clippy fix (`edits.rs`, wildcard pattern) |
| `model_open_search_pack` on datafile, datafile64, localfile, localfile64 | ✅ 4/4 byte-identical (348/348/4/4 tables) |
| GUI flow on real files (`apps/bnsedit` `edit_pack_reopen`): set item price +1, undo, redo, pack, reopen 32 and 64 | ✅ value present in both widths |
| Worst-case search, `item` (144,439 records, 798 fields per record) | 170 s → 0.97 s |

Speed fixes:
- `base_fields`: one name → attribute map per record, instead of one attribute scan per field (quadratic).
- `search`: unedited records use the new `bnsdata::table::any_text_with_pool`, which decodes only `string` and `native` fields.
- Memory with all four files open: about 3.5 GB RSS.

---

# bnsmodel v1, part 1

| Part | State | Test counts |
| --- | --- | --- |
| Typed fields, project, search | ✅ Written; ❓ not compiled | Covered by 15 synthetic tests |
| Scalar writes and schema metadata | ✅ Written; ❓ not compiled | Included in the 15 tests |
| Mirror, history, validation | ✅ Written; ❓ not compiled | Included in the 15 tests |
| Pack and output safety | ✅ Written; ❓ not compiled | Included in the 15 tests |
| Tests, Clippy, fmt | ❓ Shell approval blocked all checks | 0 run; 1 real-file test ignored |

- Types: `Value`, `FieldView`, `Hint`, `TableId`, `RecordId`, `TableInfo`, `Edit`, `Problem`, `Severity`, `PackFile`, `PackReport`.
- `Project::{open, open_with_schemas, from_containers}`: open read-only sources and width-specific schemas.
- `Project::tables()`: list logical tables.
- `Project::record(table, index)`: read typed fields and resolve references.
- `Project::record_title(table, index)`: return an alias, text, or key.
- `Project::search(table, query, limit)`: search keys and text.
- `Project::validate(edit)`: check source values and schema rules.
- `Project::apply(edit)`: check and apply both physical views.
- `Project::{undo, redo}()`: restore each view's values.
- `Project::{changes, discard}(...)`: list or discard net changes.
- `Project::problems()`: check changed records.
- `Project::pack(out_dir)`: write new files and return counts.
- `bnsdata::table::Record::set_scalar(...)`: write checked scalar storage through `ScalarValue`.

Open: compilation and formatting; Cargo.lock refresh outside this job's write scope; client acceptance; outer size rules. `references_to` is deferred. No game files were read.
Files, relative to `bns-server/`: `Cargo.toml`; `Program/Tools/DataFile/BnsModel/{Cargo.toml,src/{lib,edits,files,tests}.rs,tests/real_files.rs}`; `Program/Tools/DataFile/BnsData/src/{tabledef.rs,table.rs,table/{scalar,tests,strings_tests}.rs}`; `Docs/DataFile/{09-model,STATUS-model}.md`.
```sh
BNSDATA_CLIENT_DAT='/path/datafile.bin' BNSDATA_DAT_WIDTH=32 BNSDATA_DAT_ROLE=main BNSDATA_TABLEDEFS='/path/TableDefs_x86' CARGO_NET_OFFLINE=true cargo test --manifest-path bns-server/Cargo.toml -p bnsmodel --test real_files model_open_search_pack -- --ignored --exact --nocapture
```

---

# Part checkpoints

| Part | State | Test counts |
| --- | --- | --- |
| 1. Typed values and field views | ✅ Written; ❓ not compiled | 1 synthetic written; 0 run |
| 2. Project, lazy typed reads, search | ✅ Written; ❓ not compiled | Covered by synthetic tests; 0 run |
| 3. Checked scalar writes and schema metadata | ✅ Written; ❓ not compiled | 1 synthetic written; 0 run |
| 4. Atomic mirror, undo/redo, discard | ✅ Written; ❓ not compiled | 2 synthetic written; 0 run |
| 5. Validation | ✅ Written; ❓ not compiled | Covered by synthetic tests; 0 run |
| 6. Pack and output safety | ✅ Written; ❓ not compiled | 2 synthetic written; 0 run |
| 7. Tests and API notes | ✅ 15 synthetic, 1 ignored written | 0 run |
| 8. Workspace tests, Clippy, fmt | ❓ Shell approval required | None ran |

No game files were read. No input files will be written.
