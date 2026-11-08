# bns-server

A Linux Rust reimplementation of the Blade & Soul server set (client build 381).
The folder layout follows the original NCSoft source tree, as seen in the build
paths inside the original binaries (`...\depot\Branch\381NC\Program\Server\...`).

| Folder | Original | Contents |
| --- | --- | --- |
| `Program/Server/BloodLib` | `Program\Server\BloodLib` | Session, framing, packets, dispatch (crate `bloodlib`) |
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

No keys, passwords or game data go into this repository.
