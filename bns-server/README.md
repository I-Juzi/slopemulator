# bns-server

A Linux Rust reimplementation of the Blade & Soul server set (client build 381).
The folder layout follows the original NCSoft source tree, as seen in the build
paths inside the original binaries (`...\depot\Branch\381NC\Program\Server\...`).

| Folder | Original | Contents |
| --- | --- | --- |
| `Program/Server/BloodLib` | `Program\Server\BloodLib` | Session, framing, packet fields, ciphers (crate `bloodlib`) |
| `Program/Server/Packet` | `Program\Server\Packet` | Fragment packets, command dispatch tables `tables/381.json` (crate `packet`) |
| `Program/Server/Shared` | `Program\Server\Shared` | NP channel trait, shared types and local stub (crate `shared`) |
| `Program/Server/LobbyGate` | LobbyGate | Gate server, build 381 (crate `lobbygate`) |
| `Program/Server/LobbyDaemon` | LobbyDaemon | Lobby daemon, build 307 (crate `lobbydaemon`) |
| `Program/Server/GameDaemon` | GameDaemon | World server, build 381 (crate `gamedaemon`) |
| `Program/Server/CacheGate` | CacheGate | Cache gate (crate `cachegate`) |
| `Program/Server/AchievementDaemon` | AchievementDaemon | Achievements (crate `achievementdaemon`) |
| `Program/Server/RankingDaemon` | RankingDaemon | Ranking boards and seasons (crate `rankingdaemon`) |
| `Program/Server/SmartCouponDaemon` | SmartCouponDaemon | Smart coupons (crate `smartcoupondaemon`) |
| `Program/Server/Data` | `Program\Server\Data` | LMXBOSLB codec (`Lmxb`) and typed game data (`GameData`) |
| `Program/Platform/Sts` | platform-common | NP client-login server (crate `sts`) |
| `Program/Platform/StServers/StsCore` | StServers framework | STS messages, Connect, Route types, INI, errors (crate `sts-core`) |
| `Program/Platform/StServers/DbSrv` | `DbSrv.exe` | Stored-procedure gateway to SQL Server (crate `dbsrv`) |
| `Program/Platform/StServers/RouteSrv` | `RouteSrv.exe` | Service bus router (crate `routesrv`, in progress) |
| `Program/Tools` | — | Data file tools, BnsEdit, emulator scripts, client key tool |
| `Program/output` | `Program\output` | Build output (not in git) |
| `Docs/` | — | Specs and status for each part |

## Status (8 Nov 2026)

| Part | State | Tracker |
| --- | --- | --- |
| LobbyGate 381 | Ported; handshake, framing, dispatch, handlers | `Docs/Server/LobbyGate/` |
| LobbyDaemon 307 | Ported; login and links | `Docs/Server/LobbyDaemon/` |
| GameDaemon 381 | 502 client handlers: 74 ported, 423 partial, 5 not started | `Docs/Server/GameDaemon/09-parity.md`, `parity-ledger.tsv` |
| CacheGate, AchievementDaemon, RankingDaemon, SmartCouponDaemon | Ported; parity reviews done | `Docs/Server/<name>/README.md` |
| STS client login (`sts`) | SRP-6a login server; real-client login not yet verified | `Docs/Platform/Sts/STATUS.md` |
| StServers (original NP platform) | DbSrv ported; RouteSrv in progress; 47 other services not ported yet | `Docs/Platform/StServers/README.md` |

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
