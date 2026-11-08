# Schema export from an unprotected client executable

Date: 2026-11-07.

## Scope

`bnsschema` reads a Windows PE file as bytes. It does not execute the file.
It writes a TableDefs JSON folder for `bnsdata::tabledef::Definitions::load_folder`.

The tool does not load executable code, map code, patch files, unpack files,
decrypt files, or emulate code. It does not use IDA or the network.
A protection refusal is a normal result. There is no bypass option.

## Sources and build limits

| Source | Use | Build |
| --- | --- | --- |
| `07-client-exe-tabledefs.md` | Previous reference results | x86 client definitions match 0.0.381.8876; exact executable build is not verified. |
| `client-exe/exportdefs.py` | Layouts, reads, type codes, and JSON fields | `Client.exe` x86 and `BNSR_116.exe` x64p4. |
| `client-exe/pe.py` | PE header and section fields | Both reference files. |
| `client-exe/findarr.py`, `client-exe/arr64.py` | Null slot and consecutive table IDs | x86 and x64 reference arrays. |
| `Program/Tools/DataFile/BnsData/src/tabledef.rs` | Output consumer and count checks | Current Rust workspace. |

No new real-file result is claimed here. See `STATUS-schema.md` for command results.
The Rust port has not established the build number of any executable.
❓ A different PE32+ build can use a layout other than `x64p4`.
PE width alone cannot prove that a future client uses this schema layout.

## Processing order

1. Read file bytes.
2. Parse and validate DOS, COFF, optional, and section headers.
3. Check all protection rules.
4. Select `x86` for PE32 or `x64p4` for PE32+.
5. Find the array, or validate the explicit array VA.
6. Read all table definitions and collect unknown type codes.
7. Validate output names and check for output conflicts.
8. Create the output folder.
9. Create each JSON file with `create_new`.

`inspect` stops before step 7. It reads schema fields to report unknown types.
It does not create a folder or a file.
Malformed PE headers return a format error before the protection check can run.
A parsed PE must pass the protection check before any schema search.

## Protection gate

`protect::check(&Pe) -> Result<(), Refused>` applies these rules in order.
The public API converts a refusal to `Error::Refused(String)`.
The CLI adds the prefix `refused: ` when it displays this error.

| Rule | Refusal reason string |
| --- | --- |
| A section has a listed packer/protector name. Match without case. | `packer/protector section: {name}` |
| No section contains the entry-point RVA. | `entry point is not in a section` |
| The entry-point section is not the first executable section in section-header order. | `entry point is not in the first executable section` |
| The entry-point section has `IMAGE_SCN_MEM_WRITE` (`0x80000000`). | `entry-point section is writable` |
| A section has raw size >= 65,536 bytes and entropy >= 7.4 bits per byte. | `high-entropy section: {name} (entropy >= 7.4)` |
| An executable section has raw size 0 and virtual size > 0. | `executable section has no raw bytes: {name}` |

The packer/protector section list is:

```text
.themida .winlice .vmp0 .vmp1 .vmp2 .enigma1 .enigma2
upx0 upx1 .aspack .adata .mpress1 .mpress2 .petite
.nsp0 .nsp1 .perplex .boom .arxan
```

The entry-point section span is `max(virtual_size, raw_size)`.
An executable section has `IMAGE_SCN_MEM_EXECUTE` (`0x20000000`).
Entropy uses every raw byte in a section. It uses Shannon entropy with base 2.
The Python PE helper samples at most 4,000,000 bytes. The Rust check does not sample.
These checks are conservative. A legitimate high-entropy section can cause refusal.
Passing these checks does not prove that a file has no protection.

## PE reads

The parser accepts machine `0x14c` with PE32 and machine `0x8664` with PE32+.
It checks both signatures and the minimum optional header length.
It reads machine, image base, entry-point RVA, and all section fields.
It validates every non-empty raw section range against the file length.

`va_to_offset` translates file-backed section bytes only.
It does not translate zero-filled virtual tails or header addresses.
Null pointers do not translate.
Each structure read must fit inside one file-backed section.
Offset arithmetic uses checked additions or bounded field widths.

## Layout constants

All offsets are in bytes. Reads use little-endian order.
`P` is the pointer width. Both layouts use 4-byte packing where the reference requires it.

| Structure | x86 | x64p4 |
| --- | --- | --- |
| P | 4 | 8 |
| Table | name 0; type i16 4; major i16 6; minor i16 8; el 12; elcount u8 16; maxid u32 20; autokey u8 28; module u32 32 | name 0; type i16 8; major i16 10; minor i16 12; el 16; elcount u8 24; maxid u32 28; autokey u8 36; module u32 40 |
| Element | stride 44; name 0; type u8 4; body 8; child[32] 12 | stride 52; name 0; type u32 8; body 12; child[32] 20 |
| Body | stride 20; attr 0; sub 4; default 8; attrcount i16 12; subcount i16 14; size u16 16; keyoffset u16 18 | stride 32; attr 0; sub 8; default 16; attrcount i16 24; subcount i16 26; size u16 28; keyoffset u16 30 |
| Subtype | stride 16; name 0; attr 4; default 8; attrcount u16 12; size u16 14 | stride 28; name 0; attr 8; default 16; attrcount u16 24; size u16 26 |
| Name hash | hash u32 0; name pointer 4 | hash u32 0; name pointer 4 |

The `maxid` field occupies a wider area in the source structure.
The Rust port reads only `u32`, exactly as `exportdefs.py` does.
It does not silently change this read to `u64`.

| Attribute field | Width | x86 offset | x64p4 offset |
| --- | --- | ---: | ---: |
| stride | — | 64 | 84 |
| name | P | 0 | 0 |
| namehash | P | 4 | 8 |
| type | u8 | 8 | 16 |
| key | u8 → bool | 9 | 17 |
| repeat | u8 | 10 | 18 |
| required | u8 | 11 | 19 |
| defvalue | P → UTF-16 | 12 | 20 |
| min | i64 | 16 | 28 |
| max | i64 | 24 | 36 |
| fmin | f32 | 32 | 44 |
| fmax | f32 | 36 | 48 |
| seq | P → pointer list | 40 | 52 |
| seqcount | u16 | 44 | 60 |
| reftable | i16 | 46 | 62 |
| refel | u8 | 48 | 64 |
| deprecated | u8 → bool | 49 | 65 |
| hidden | u8 → bool | 50 | 66 |
| offset | u16 | 52 | 68 |
| size | u16 | 54 | 70 |
| propfield | u32 | 56 | 72 |

## Array detection

The automatic scan reads pointer-aligned VAs in initialized data sections.
A candidate section must have `IMAGE_SCN_CNT_INITIALIZED_DATA` (`0x40`).
It must not be executable.

A candidate array has a null slot 0.
For each slot `i` from 1, the pointer must resolve to a table structure.
The structure's signed 16-bit type must equal `i`.
Its name pointer must resolve to a non-empty UTF-16LE string.
Each name character must match `[a-z0-9_-]`.
A failed slot ends the run.

The scan selects the longest run with at least 16 tables.
For equal lengths, it keeps the first candidate in section-header and slot order.
The report gives the array VA and table count. The count excludes slot 0.
An explicit VA uses the same slot checks but accepts one or more tables.
The explicit option does not skip the protection gate.

An array can have at most 4,096 slots, including null slot 0.
A continuation beyond this limit returns a format error.
A short run or an absent automatic candidate returns `NotFound`.

## JSON contract

| Object | Keys |
| --- | --- |
| Table | `name`, `type`, `major_ver`, `minor_ver`, `el`, `elcount`, `maxid`, `autokey`, `module` |
| Element | `name`, `type`, `body`, `child` |
| Body | `attr`, `sub`, `defaultRecord`, `attrcount`, `subcount`, `size`, `keyoffset` |
| Subtype | `name`, `attr`, `defaultRecord`, `attrcount`, `size` |
| Attribute | `name`, `namehash`, `type`, `key`, `repeat`, `required`, `defvalue`, `min`, `max`, `fmin`, `fmax`, `seq`, `seqcount`, `reftable`, `refel`, `deprecated`, `hidden`, `offset`, `size`, `propfield` |
| Name hash | `namehash`, `name` |
| Default record | `type` (i8), `subtype` (i16), `size` (u16) |

Empty attribute and subtype lists become JSON `null`.
The element list stays a JSON array, including when empty.
Each child list has 32 unsigned byte values.
A mapped sequence pointer with count 0 produces `[]`.
An unmapped sequence pointer produces `null`.
Unmapped optional strings, name hashes, and default records produce `null`.
A mapped but malformed string or structure returns a format error.

The type names match `exportdefs.py`:

```text
1 int8; 2 int16; 3 int32; 4 int64; 5 float32; 6 bool; 7 string;
8 seq; 9 seq16; 10 ref; 11 tref; 12 sub; 15 vector32; 16 icolor;
18 box; 20 msec; 21 distance; 22 velocity; 23 prop_seq;
24 prop_field; 25 script_obj; 26 native; 28 icon; 30 time64.
```

An unknown code becomes `type_N`.
`Found.unknown_types` contains unique codes in ascending order.
The port does not guess the names of codes 31, 32, or 33.

Object key order and JSON whitespace can differ from Python output.
Keys and finite values must match. Strings use UTF-8 JSON encoding.
Non-finite f32 values return a format error. JSON cannot represent them.
Negative signed body counts also return a format error.
These malformed-input outcomes are stricter than the Python reference.

## Input and output limits

| Item | Limit or rule |
| --- | --- |
| UTF-16 string | At most 4,096 code units, followed by a zero terminator. Invalid surrogate pairs fail. |
| Array | At most 4,096 slots, including slot 0. |
| Elements | u8 count, at most 255. |
| Body attributes and subtypes | Non-negative i16 counts, at most 32,767 each. |
| Subtype attributes and sequence values | u16 counts, at most 65,535 each. |
| Export work | At most 2,000,000 combined element, subtype, attribute, and sequence entries. |
| Output names | Non-empty `[a-z0-9_-]`; one `<name>.json` per table. Duplicate names fail. |
| Existing files | Preflight rejects conflicts. `create_new` also prevents replacement after a race. |

The tool reads the complete input file into memory.
The caller must also apply suitable file-size and process-memory limits.
An I/O error can leave a partial output folder.
No automatic removal or replacement takes place.
Use a new empty folder for a retry.

## API and CLI

```rust
pub struct Found {
    pub width: Width32or64,
    pub layout: &'static str,
    pub array_va: u64,
    pub tables: usize,
    pub unknown_types: Vec<u8>,
}
pub fn inspect(exe: &Path) -> Result<Found, Error>;
pub fn export(exe: &Path, array_va: Option<u64>, out_dir: &Path)
    -> Result<Found, Error>;
```

Width variants are `Width32or64::Width32` and `Width32or64::Width64`.
Errors are `Refused(String)`, `NotFound(String)`, `Io(std::io::Error)`, and `Format(String)`.
All errors implement `Display`. The CLI returns a non-zero exit code on error.

```text
bnsschema EXE OUTDIR [--array 0xVA]
```

The success summary reports width, layout, array VA, table count, and unknown codes.
It does not print table names or game records.

## Tests

The tests contain synthetic data only.
`Fake` builds a PE32 or PE32+ file in memory.
The small fixtures have one or two table definitions.
Larger synthetic fixtures test the automatic 16-table minimum and longest-run rule.

| Test group | Intended check |
| --- | --- |
| Exact JSON | Every exported key and representative value, for both layouts. Includes signed fields, f32 conversion, Unicode, name hash, sequences, subtype, default record, and `maxid` low-word behavior. |
| Folder round trip | Export two synthetic tables and load them through `Definitions::load_folder`. |
| Detection | Automatic scan, longest run, explicit small array, and automatic minimum. |
| Protection | Every listed packer name in uppercase; writable entry section; entry in data, second executable section, or no section; high entropy; virtual-only executable section. |
| Output safety | Existing-file refusal; refused input creates no output folder. |
| Malformed input | Truncate both fake PE widths at every byte length; mutate each used byte; invalid pointers, strings, counts, names, and floats. |
| Ignored real test | Read only paths from `BNSSCHEMA_EXE` and `BNSSCHEMA_REF`. Export outside the repository. Compare table count and per-table base-plus-subtype attribute count by ID. Print counts only. |

These are test definitions, not test results.
The ignored test does not prove full JSON equality on a real executable.
No real test runs without explicit operator paths.

## Offline verification

Run these commands from `bns-server`:

```sh
CARGO_NET_OFFLINE=true cargo build
CARGO_NET_OFFLINE=true cargo test -p bnsschema
CARGO_NET_OFFLINE=true cargo clippy -p bnsschema --all-targets -- -D warnings
CARGO_NET_OFFLINE=true cargo fmt -p bnsschema
```

This job restricts writes outside the crate and specification folder.
The only source-file exception is the workspace member list.
The submitted shell checks use `--locked` to prevent an out-of-scope lockfile write.
The main session must add the new local package to `Cargo.lock` if Cargo requires it.
