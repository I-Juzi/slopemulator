# clientdbg

Live debugger for `Client.exe` 0.0.381.8884 (32-bit). It has two parts:

- **Plugin** (`clientdbg.dll`): runs inside the client. It connects OUT to the host.
- **Host** (`clientdbg`, Linux): accepts the plugin, logs events, and sends commands.

Part 1 has: link, memory read/write, modules, threads, registers, and hardware
breakpoints and watchpoints. Software breakpoints, socket tap and break-and-wait
mode are in part 2.

## Build

```
cargo build -p clientdbg --release                                   # host (Linux)
cargo build -p clientdbg --release --target i686-pc-windows-gnu      # plugin DLL
cargo test  -p clientdbg
```

The DLL is `target/i686-pc-windows-gnu/release/clientdbg.dll`. It must import
only system DLLs and must export `GPluginInfo`. Check with
`i686-w64-mingw32-objdump -p clientdbg.dll`.

## Install (laptop with the client)

Copy these files to the client folder:

| File | Content |
| --- | --- |
| `bin\plugins\clientdbg.dll` | the plugin |
| `bin\plugins\clientdbg.ini` | settings (see below) |
| `bin\plugins\clientdbg.token` | the token, one line |

`clientdbg.ini`:

```
host=100.77.181.120
port=47100
allow_write=0
```

`allow_write=1` turns on the `write` command. Keep it at 0 unless you need it.
The plugin tries to connect again every 5000 ms.

On the host, put the same token in `secrets/clientdbg.token`. Do not commit it.
In the repo, a token is always written as `keyhere`.

## Use

Start the host:

```
clientdbg serve                      # plugin 100.77.181.120:47100, control 127.0.0.1:47101
clientdbg serve --listen 0.0.0.0:47100 --ctl 127.0.0.1:47101 --token-file secrets/clientdbg.token
```

Events go to `logs/clientdbg/events-<date>.jsonl` (UTC date).

Send commands in a second shell. Add `--ida` to give an address from the IDA
database. The host converts it with the image base from the plugin hello
(`addr - 0x400000 + base`).

```
clientdbg status
clientdbg ping
clientdbg modules
clientdbg threads
clientdbg rebase
clientdbg --ida read 0x5fd770 64
clientdbg regs 1234
clientdbg --ida write 0x5fd770 90                 # needs allow_write=1
clientdbg --ida hbp 0x5fd770 exec 1 0 --capture regs,stack:8,mem:ecx+0x10:64,deref:esp+4 --limit 20
clientdbg hbp 0x12345678 write 4 1                # watchpoint, slot 1
clientdbg hbp-clear 0                             # one slot
clientdbg hbp-clear                               # all slots
clientdbg tail --n 50 --filter ev=hbp
```

`hbp ADDR KIND LEN SLOT`: KIND is `exec`, `write` or `rw`. LEN is 1, 2 or 4
(ignored for `exec`). A data address must be aligned to LEN. SLOT is 0 to 3.
`--limit N` stops recording after N hits (0 = no limit).

Capture items (separate with `,`; empty = registers only):

| Item | Meaning |
| --- | --- |
| `regs` | general registers (always in the event) |
| `stack:N` | N dwords from esp (N ≤ 64) |
| `mem:REG+OFF:LEN` | LEN bytes at REG+OFF (LEN ≤ 256) |
| `deref:REG+OFF` | read a pointer at REG+OFF, then 64 bytes at the pointer |

At most 8 items and 1024 bytes of data per hit. Unreadable memory gives an `err`
field in the item, not a crash.

## Protocol

One JSON object per line. Request `{"id":n,"cmd":"ping"}`; reply
`{"id":n,"ok":...}` or `{"id":n,"err":"..."}`; event `{"ev":"hbp","t":ms,...}`.
The first line from the plugin is `{"hello":{...},"token":"keyhere"}`.

## Safety notes

- The host accepts one plugin. A wrong token closes the connection and writes nothing.
- Listen only on the Tailscale address or on localhost. Do not open the port to the internet.
- The control port has no authentication. Keep it on `127.0.0.1`.
- Hardware breakpoints are set on all client threads except the plugin thread
  (suspend, set, resume). New threads get them within 1000 ms.
- The exception handler copies data into a lock-free ring (4096 slots). If the ring
  is full, events are dropped and counted (`stats` event).
- A hit on a very hot address can slow the client. Use `--limit`.
- `read` checks the pages first. `write` restores the old page protection.
- Do not use this on a client that you do not own.
