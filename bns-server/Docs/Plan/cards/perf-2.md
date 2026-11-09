# Card perf-2: profile a busy scene and plan hot-spot fixes (Opus, IDA + profile data)

## Goal
Use the profile from `clientdbg profile` (dbg-3) to find the top 20 functions by time in a busy scene. For each, decide if a plugin fix is possible.

## Inputs
- Profile reports from `logs/clientdbg/` (the operator runs the scene first).
- `Docs/Client/RE/` chapters and `functions-*.tsv`. IDA lane `ida/lanes/cx/`.

## Work
1. For each top function: what it does, why it is slow (loop size, allocation, repeated lookup, per-frame rebuild), callers.
2. Fix idea for each: hook and cache, skip redundant work, lower the update rate, or server-side limit. Give risk and expected gain.
3. Write `Docs/Client/RE/perf.md` §Hot spots and one build card per fix that is worth it.

## Out of scope
- Engine rewrites and multithreading changes.
