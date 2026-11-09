# Card dbg-3-rust: clientdbg part 3 — sampling profiler

## Goal
Add a sampling profiler to `clientdbg`, so we can find the slowest code of the client by measurement. Read `Program/Tools/ClientDbg/README.md` and run `python3 tools/rust_api.py Program/Tools/ClientDbg` first.

## Work
1. `profile start [tid|main] [interval_us=1000] [stack_depth=8]`: a sampler thread suspends the target thread, reads EIP and walks up to `stack_depth` EBP frames, then resumes it. Default target: the main thread (the thread that runs `_WinMain@16`, 381 0x1190f60; find it by start address or the thread with the most CPU time).
2. Samples go into a fixed-size histogram in the plugin (address → count, plus stack → count). No send per sample.
3. `profile stop` and `profile dump`: send the histogram once. Also count samples per module (Client.exe, bsengine_Shipping.dll, d3d9, other).
4. Host: `clientdbg profile report [--top 50]` prints flat and inclusive time per function, with names from `Docs/Client/RE/functions-*.tsv` (nearest lower address). Module samples outside Client.exe are shown by module and offset. Write `--folded` output for flame graph tools.
5. Update the README with a "how to profile a scene" example.

## Files
- `Program/Tools/ClientDbg/`

## Acceptance
`cargo test -p clientdbg`, clippy and the cross build must pass.
| Test name | Input | Expected result |
| --- | --- | --- |
| hist_add | 1,000 samples on 3 addresses | counts match |
| hist_full | more unique addresses than the table size | overflow counter grows, no panic |
| report_flat | histogram + functions TSV sample | functions sorted by count, with percent |
| report_inclusive | stacks with shared parents | parent count ≥ sum of its own samples |
| folded_format | 2 stacks | lines `a;b;c N` |
| proto_profile_cmds | start, stop, dump | round trip |

## Out of scope
- No code changes inside the client functions. Hot-spot fixes are separate cards (perf-2 and later).
