# Card dbg-2-rust: clientdbg part 2 — software breakpoints, socket tap, break mode

## Goal
Add to `clientdbg` (part 1 is done, read `Program/Tools/ClientDbg/README.md` and run `python3 tools/rust_api.py Program/Tools/ClientDbg`): unlimited software breakpoints, a socket tap, a break-and-wait mode, and function call tracing.

## Context
- Same plugin rules as part 1 (README). Events go through the ring buffer. The vectored handler must not block, except in break mode.

## Work
1. `bp addr capture count`: software breakpoint. Write 0xCC (VirtualProtect, FlushInstructionCache). On `EXCEPTION_BREAKPOINT` at a known address: record capture, write back the original byte, set EIP = addr, set TF. On the next `EXCEPTION_SINGLE_STEP` for that thread (per-thread state, TLS slot): write 0xCC again, clear TF. Not more than 256 breakpoints. `bp_clear addr|all` restores bytes. On plugin unload or host disconnect, clear all.
2. `break addr`: like `bp`, but the hit thread waits on an event until the host sends `cont tid` or 30000 ms pass. During the wait the host can `regs tid`, `read`, `setreg tid reg value` (needs allow_write).
3. Socket tap: IAT hooks in the client module for `ws2_32` `connect`, `send`, `recv`, `WSASend`, `WSARecv`, `closesocket`. Event: socket, peer address, direction, length, first `tap_bytes` bytes as hex (ini, default 256). Command `tap on|off`.
4. `trace addr depth`: on hit, record the return address chain (walk EBP frames, max depth 16), converted to IDA addresses on the host.
5. Host: `clientdbg tail --pretty` prints hits as `0x5fd770 PacketParser::ParseData  ecx=… ` using names from `Docs/Client/RE/functions.tsv` when the file exists (columns: address, name).
6. Update the README.

## Files
- `Program/Tools/ClientDbg/`

## Acceptance
`cargo test -p clientdbg`, clippy and the cross build must pass (same gate as part 1).
| Test name | Input | Expected result |
| --- | --- | --- |
| bp_table_limits | add 257 breakpoints | the last add fails |
| bp_table_restore | add then clear | original byte returned for each address |
| tap_event_encode | send of 300 bytes, tap_bytes 256 | event len 300, 256 bytes of hex |
| frames_walk | fake stack memory with 3 frames | 3 return addresses |
| names_lookup | functions.tsv sample | address → name, nearest lower match with +offset |
| proto_new_cmds | bp, break, cont, setreg, tap, trace | round trip |

## Out of scope
- No anti-cheat code. No keys or tokens in the repo.
