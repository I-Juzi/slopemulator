# bns-server

A Linux Rust reimplementation of the Blade & Soul server set (client build 381).
The folder layout follows the original NCSoft source tree, as seen in the build
paths inside the original binaries (`...\depot\Branch\381NC\Program\Server\...`).

| Folder | Original | Contents |
| --- | --- | --- |
| `Program/Server/BloodLib` | `Program\Server\BloodLib` | Session, framing, packets, fragments, dispatch (crate `bloodlib`) |
| `Program/Server/Shared` | `Program\Server\Shared` | NP channel trait, shared types and local stub (crate `shared`) |
| `Program/Server/LobbyGate` | LobbyGate | Gate server, build 381 (crate `lobbygate`) |
| `Program/Server/LobbyDaemon` | LobbyDaemon | Lobby daemon, build 307 (crate `lobbydaemon`) |
| `Program/Server/Data` | `Program\Server\Data` | LMXBOSLB codec (`Lmxb`) and typed game data (`GameData`) |
| `Program/Platform/Sts` | platform-common | NP login server (crate `sts`) |
| `Program/Tools` | — | Data file tools, BnsEdit, emulator scripts |
| `Program/output` | `Program\output` | Build output (not in git) |
| `Docs/` | — | Specs and status for each part |

Build and check:

```
cargo test
cargo clippy --workspace --all-targets -- -D warnings
```

LobbyDaemon keeps the public paths `lobbydaemon::np`, `store::AccountId`,
and `gamed::{GameFuture, ChargeMode}` as re-exports of shared types.
STS uses `shared` directly. Enable its `np-adapter` feature to use
`sts::np_adapter::AuthorityNp`.

The two LMX readers remain separate. BnsData keeps raw UTF-16 units,
three header padding bytes, and children on text nodes. It takes a key
from the caller and permits depth 256. Lmxb uses Unicode strings,
requires zero header padding and leaf text nodes, and permits depth 128.
It does not take a key from the caller. These differences prevent a
behavior-preserving replacement. See `Program/Tools/DataFile/BnsData/src/lmx.rs`
and `Program/Server/Data/Lmxb/src/lib.rs`.
The source comparison is in `notes/client-datafiles.md`.

No keys, passwords or game data go into this repository.
