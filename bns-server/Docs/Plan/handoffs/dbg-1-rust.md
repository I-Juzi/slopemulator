# Handoff dbg-1-rust

## Done
- Crate `Program/Tools/ClientDbg` (lib cdylib+rlib, bin `clientdbg`), workspace member added.
- Modules: proto, capture, ring, addr, ini, hex, dr, cli, host (all Linux); win, plugin (32-bit Windows only).
- Tests (21, all pass): proto_roundtrip, proto_bad_line, proto_wire_shape, capture_parse,
  capture_limits, capture_offsets_and_display, capture_run, ring_full, ring_order, ring_threads,
  addr_convert, ini_parse, host_token_reject, host_cmd_forward, host_events_file, civil_dates,
  dr7_encoding, dr_validate, dr6_decode, hex_roundtrip, cli_requests.
- `cargo fmt`, `cargo test -p clientdbg`, `cargo clippy --all-targets -D warnings` pass.
  Clippy also passes for `--lib --target i686-pc-windows-gnu`.
- `cargo build --release --target i686-pc-windows-gnu` passes. `GPluginInfo` is in the DLL.
- `build.rs` links libgcc statically for the DLL (so no `libgcc_s_dw2-1.dll` import).
- `panic = "abort"` not set. Panics are caught with `catch_unwind` at the plugin edges.
- README.md written.

## Not done
- No run on a real client. The Windows code (VEH, DR set, thread suspend) has only been compiled.
- Import check with `i686-w64-mingw32-objdump -p` was denied in this session. I used Grep on the DLL:
  the import table lists only system DLLs. The string `libgcc_s_dw2-1.dll` is in the file at
  another place (probably C runtime start-up code, not an import). Gate: run objdump to confirm.
- `Request` is a struct `{id, cmd: Command}`, not one enum (id is shared by all commands).

- Gate failure: the gate reads `target/i686-pc-windows-gnu/release/clientdbg.dll`. That path does not
  exist. `.cargo/config.toml` sets `target-dir = "Program/output"`, and my shell env put the build in
  `/home/ubuntu/.cache/bns-wt-target/`. The build is good. The gate path is wrong. I did not change
  `.cargo/config.toml` (not in the card "Files"; it changes all crates). Fix: use the real target dir.

## ❓
- Card test `addr_convert` says ida 0x5fd770 with base 0x1000000 gives 0xdfd770. The card formula
  `addr - 0x400000 + base` gives 0x11fd770. I tested for 0x11fd770.
- `hello` field `version`: I send the plugin version (`clientdbg 0.1.0`), not the client version.

## Blocked
- Nothing.
