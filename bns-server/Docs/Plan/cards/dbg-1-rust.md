# Card dbg-1-rust: clientdbg — live debugger plugin for Client.exe 381 (part 1)

## Goal
Build `clientdbg`: a plugin DLL for the 32-bit client and a host program on Linux. The plugin connects to the host. The host sends debug commands and records events. Part 1: link, memory, modules, threads, hardware breakpoints and watchpoints.

## Context
- The client is `Client.exe` 0.0.381.8884, 32-bit, not packed. It is the same file as the IDA database, so IDA addresses are live addresses (image base 0x400000 ❓: the plugin reports the real base).
- The proxy `winmm.dll` (bnspatch loader) loads each `bin\plugins\*.dll`. A plugin MUST export a static `GPluginInfo` with this exact layout (SDK 3.1.0.0), or the loader crashes:
```c
typedef union Version { struct { uint16_t major, minor, build, revision; } f; uint64_t version; } Version;
typedef struct PluginInfo {
    Version sdk_version;            /* 3 | (1 << 16) */
    unsigned char hide_from_peb;    /* 0 */
    unsigned char erase_pe_header;  /* 0 */
    int  (__cdecl *init)(const Version);        /* return 1 */
    void (__cdecl *oep_notify)(const Version);  /* start the worker thread here */
    int  priority;                  /* 1 */
    const wchar_t *target_apps;     /* L"Client.exe" */
} PluginInfo;
```
  In Rust: `#[no_mangle] pub static GPluginInfo: PluginInfo` with `extern "cdecl" fn` pointers and `#[repr(C)]`. `Version` is passed by value (8 bytes).
- The plugin connects OUT to the host (no firewall rule on the laptop). Host Tailscale address and port come from `clientdbg.ini` next to the DLL: `host=100.77.181.120`, `port=47100`. Reconnect every 5000 ms.
- Auth: the first line from the plugin is `{"hello":..,"token":"<token>"}`. The token is in `clientdbg.token` next to the DLL, and on the host in `secrets/clientdbg.token`. Never commit a token. Tests use a fixed test token.

## Inputs
| What | Path |
| --- | --- |
| Workspace | `Cargo.toml` (add member) |
| Targets installed | `i686-pc-windows-gnu`, `x86_64-unknown-linux-gnu`. MinGW: `i686-w64-mingw32-gcc`. |

## Work
1. New crate `clientdbg` at `Program/Tools/ClientDbg`: `crate-type = ["cdylib", "rlib"]`, and a bin `clientdbg` (host). Windows code is under `#[cfg(windows)]`. All logic that does not need Windows is in plain modules and has tests on Linux. Use `windows-sys` for Win32 calls. Set `panic = "abort"` in the workspace profile only if it does not change other crates; else catch panics at every FFI edge with `catch_unwind`.
2. Wire protocol (`proto.rs`): one JSON object per line (`serde_json`). Requests: `{"id":n,"cmd":...}`. Replies: `{"id":n,"ok":..}` or `{"id":n,"err":"..."}`. Events: `{"ev":...,"t":<ms since start>,...}`.
3. Commands (part 1):
| cmd | Arguments | Result |
| --- | --- | --- |
| `hello` (plugin → host) | pid, exe path, image base, exe size, version | — |
| `ping` | — | `pong` |
| `modules` | — | list: name, base, size |
| `threads` | — | list: tid, start address, eip |
| `read` | addr, len (≤ 65536) | hex. Check with `VirtualQuery` first; unreadable pages → `err` |
| `write` | addr, hex | ok. Use `VirtualProtect` and restore. Refuse unless `allow_write=1` in the ini. |
| `regs` | tid | all general registers, eflags, DR0–DR7 |
| `hbp` | addr, kind (`exec`, `write`, `rw`), len (1, 2, 4), slot 0–3, capture spec, count limit | ok. Set DR0–3/DR7 on all client threads (suspend, set, resume; skip own thread). Re-apply every 1000 ms so new threads get them. |
| `hbp_clear` | slot or all | ok |
| `rebase` | — | image base; host converts IDA addresses with `addr - 0x400000 + base` |
4. Breakpoint hits: a vectored exception handler (`AddVectoredExceptionHandler(1, ..)`) handles `EXCEPTION_SINGLE_STEP` from DR hits. It builds the event and continues (set RF in eflags for exec breakpoints). It must not allocate in a way that can deadlock: copy data into a lock-free ring buffer (fixed size, 4096 slots), and a worker thread sends it. Drop events when full and count the drops.
5. Capture spec: a list of items run at each hit: `regs`, `stack:N` (N dwords from esp), `mem:REG+OFF:LEN` (LEN ≤ 256, read with a guard), `deref:REG+OFF` (read a pointer, then 64 bytes). Parse it in `capture.rs` (Linux-tested).
6. Host bin `clientdbg`:
   - `clientdbg serve [--listen 100.77.181.120:47100] [--ctl 127.0.0.1:47101]`: accepts one plugin, checks the token, writes every event to `logs/clientdbg/events-<date>.jsonl`, and forwards commands from the control port.
   - `clientdbg <cmd> [args]`: sends one command to the control port, prints the reply as JSON. Address arguments accept IDA addresses with `--ida` (converted with the image base from `hello`).
   - `clientdbg tail [--n 50] [--filter ev=hbp]`: prints recent events.
7. Write `Program/Tools/ClientDbg/README.md`: build, install (`bin\plugins\clientdbg.dll`, `clientdbg.ini`, `clientdbg.token`), use with examples, safety notes.

## Interfaces
```rust
pub mod proto { pub enum Request {..} pub enum Reply {..} pub enum Event {..}
  pub fn encode<T: Serialize>(v: &T) -> String; pub fn decode_request(line: &str) -> Result<Request, ProtoError>; }
pub mod capture { pub enum Item { Regs, Stack(u32), Mem{reg: Reg, off: i32, len: u32}, Deref{reg: Reg, off: i32} }
  pub fn parse(spec: &str) -> Result<Vec<Item>, CaptureError>; }
pub mod ring { pub struct Ring<T, const N: usize>; impl Ring { pub fn push(&self, v: T) -> bool; pub fn pop(&self) -> Option<T>; pub fn dropped(&self) -> u64; } }
pub mod addr { pub fn ida_to_live(ida: u32, base: u32) -> u32; pub fn live_to_ida(live: u32, base: u32) -> u32; }
pub mod host { pub async fn serve(cfg: ServeConfig) -> io::Result<()>; }
pub mod ini { pub fn parse(text: &str) -> Config; }   // host, port, allow_write
```

## Files
- `Program/Tools/ClientDbg/` (new crate)
- `Cargo.toml` (workspace member)

## Acceptance
`cargo test -p clientdbg`, clippy, and the cross build `cargo build -q -p clientdbg --release --target i686-pc-windows-gnu` must pass. The DLL must import only system DLLs (check with `i686-w64-mingw32-objdump -p`), and must export `GPluginInfo`.
| Test name | Input | Expected result |
| --- | --- | --- |
| proto_roundtrip | each Request and Event | encode then decode gives the same value |
| proto_bad_line | `{"id":1,"cmd":"nope"}` | `ProtoError::UnknownCmd` |
| capture_parse | `regs,stack:8,mem:ecx+0x10:64,deref:esp+4` | 4 items with those values |
| capture_limits | `mem:ecx:257` | error (LEN > 256) |
| ring_full | push 4097 into N=4096 | last push false, dropped() == 1 |
| ring_order | push 1,2,3, pop 3 times | 1,2,3 |
| addr_convert | ida 0x5fd770, base 0x400000 and 0x1000000 | 0x5fd770 and 0xdfd770, and back |
| ini_parse | `host=1.2.3.4\nport=47100\nallow_write=1` | values set; defaults when missing |
| host_token_reject | fake plugin with wrong token | connection closed, no events written |
| host_cmd_forward | fake plugin answers `ping` | control client gets `pong` |
| host_events_file | fake plugin sends 3 events | 3 lines in the jsonl file |

## Out of scope
- Software (INT3) breakpoints, socket tap, break-and-wait mode: card dbg-2-rust.
- No anti-cheat code. No keys or tokens in the repo (`keyhere`).
