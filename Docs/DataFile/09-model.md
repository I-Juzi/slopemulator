# bnsmodel v1 — typed edit layer

The crate has no GUI dependency. It uses `bnsdata` for file storage.
Tests use synthetic data. The ignored test uses operator paths only.

## Public API

| API | Contract |
| --- | --- |
| `Value` | Int, Float, Bool, Enum, Ref, Text, Native, Raw. Text retains UTF-16 code units. Float equality compares storage bits. |
| `FieldView` | Name, repeat index, kind, value, editable flag, hints, deprecated flag, hidden flag. |
| `TableId` / `RecordId` | Project-local table identity and stored record index. Do not reuse IDs across projects. |
| `Project::open(folder, width, schema)` | Read the four standard file names. A flat schema folder supports one width. Use `schema/x86` and `schema/x64` for both widths. |
| `Project::open_with_schemas(folder, width, schema32, schema64)` | Select each width-specific schema folder explicitly. Missing data files are allowed. Missing definitions for present files are errors. |
| `Project::from_containers` | Accept owned containers and definitions. Useful for tests and custom file selection. |
| `tables()` | Return one logical table per role and name. Prefer the selected width. Use the other width when the selected table is absent. |
| `record(table, index)` | Return typed fields. Resolve nonzero references through eight-byte keys. Preserve unresolved bytes. |
| `record_title(table, index)` | Use a nonempty alias, then the first nonempty text field, then the signed key number. Invalid UTF-16 is not replaced. |
| `search(table, query, limit)` | Match signed/unsigned key numbers and all text fields. Use Unicode lowercase comparison. Return stored indexes. |
| `validate(edit)` | Check stale old values, storage type and width, and schema rules. Return errors and reference warnings as problems. |
| `apply(edit)` | Check both physical views before changing either. Mirror by role, table name, record index, field name, and repeat index. |
| `undo()` / `redo()` | Return false when the history is empty. Restore each width's own old value. |
| `changes()` | Return net logical changes. This is not an event log. |
| `discard(change)` | Restore the baseline in both physical views. The discard can be undone. Reject stale changes. |
| `problems()` | Check all fields in changed physical records. Reference warnings do not prevent packing. |
| `pack(out_dir)` | Write all present files. Keep original bodies for unchanged tables. Return file paths, changed table names, and byte counts. |
| `Record::set_scalar` in `bnsdata` | Set int, float, bool, enum, or ref storage. Check type, stored width, repeat index, and bounds. Reject header overlap. |

## Storage and memory

Opening parses raw table bodies and inflates compressed blocks. It does not decode typed fields.
The existing `bnsdata` model retains fixed records and pools in memory.
The typed cache holds at most 128 records. It clears when full.
Compressed record locations support direct indexed reads after open.
Reference indexes are built on demand. They read record keys, not all typed fields.
Edits use a separate overlay. They do not change source containers.
A same-value edit is a no-op, including an existing out-of-range enum.
Native word `a` cannot change. Each mirror keeps its own word `a`.
A mirror error leaves both views and history unchanged.
A missing opposite-width file is not a mirror error.
A present opposite-width file with a missing table is a mirror error.

## Validation and output safety

Ranges are inactive when both endpoints are zero.
Enum edits must select an existing name.
A nonzero unresolved reference gives a warning. An all-zero reference means none.
Noncanonical bool storage stays Raw and read-only.
Unknown field kinds stay Raw and read-only.
Size-zero attributes have no field view.
Required text must not be empty. Text cannot contain a NUL code unit.
Discard and undo can restore invalid original values.
Packing checks every field in each changed record, including unchanged fields.
Packing never truncates existing paths. It reserves output names with create-new semantics.
This also blocks source paths, symlinks, and hard links at those output names.
An I/O error can leave partial new output files. Use a new output directory for each pack.
The original project and edit history remain available after a pack error.
Outer total-size and maximum-buffer fields remain unchanged, as in `bnsdata`.
Client acceptance of changed output is not verified.
`references_to` is deferred.

## Sources

Storage contracts: `06-client-body.md` and `08-strings.md`, client build 0.0.381.8884.
Model implementation: `crates/bnsmodel/src/{lib,edits,files}.rs`.
Synthetic tests: `crates/bnsmodel/src/tests.rs`.
No new client-format fact was inferred from game data in this job.
