# slopemulator

A Linux Rust reimplementation of the Blade & Soul server set (client build 381,
LobbyDaemon build 307), with its tools, launcher and reverse-engineering notes.

Game client files, server binaries, databases, IDA databases and keys are **not**
in this repository. See [`.gitignore`](.gitignore).

## Map

```
slopemulator/
├── bns-server/                 Rust workspace (full git history)
│   ├── Program/
│   │   ├── Server/             Server crates
│   │   │   ├── BloodLib/       Session, framing, packet fields, ciphers
│   │   │   ├── Packet/         Fragment packets, command dispatch tables
│   │   │   ├── Shared/         NP channel trait, shared types
│   │   │   ├── LobbyGate/      Gate server (build 381)
│   │   │   ├── LobbyDaemon/    Lobby daemon (build 307)
│   │   │   ├── GameDaemon/     Game server (build 381)
│   │   │   └── Data/
│   │   │       ├── Lmxb/       LMXBOSLB codec
│   │   │       └── GameData/   Typed game data
│   │   ├── Platform/
│   │   │   └── Sts/            NP / STS login server
│   │   └── Tools/
│   │       ├── BnsEdit/        BnsEdit GUI editor (own Cargo project)
│   │       ├── ClientKey/      Handshake RSA-1024 key generator and client patcher
│   │       ├── DataFile/       Data file crates (BnsData, BnsModel, BnsSchema)
│   │       └── Emu/            Python handshake / session emulator for build 381
│   ├── Docs/                   Specs and status for each part
│   │   ├── Server/             LobbyGate, LobbyDaemon, GameDaemon
│   │   ├── Platform/Sts/       STS login
│   │   └── DataFile/           Data file formats, tabledefs, client exe scripts
│   └── tools/                  Pipeline helper scripts
├── launcher-src/               BnsLauncher fork for the private server (C#, MIT)
├── notes/                      Client reverse-engineering notes
├── tabledefs/BNSR_116_x64/     Table definitions (519 JSON files)
└── dist/bnsedit-v1/            BnsEdit v1 builds (Linux, Windows)
```

## Server parity

How much of each original NCSoft server binary is ported to Rust.
The original binaries come from the `Service_348_8349` server set (not in this repository).
**Ported %** = handlers implemented / handlers in the original dispatch tables.

| Original binary | What it does | Rust crate | Build | Handlers ported | Ported % |
| --- | --- | --- | --- | --- | --- |
| `LobbyGate.exe` | Client entry gate: handshake, login, world list, relay to LobbyDaemon | `lobbygate` | 381 | 56 / 56 | `██████████` **100 %** |
| `LobbyDaemon.exe` | Account sessions, character list, create / delete / restore, enter-world token | `lobbydaemon` | 307 | 18 / 118 | `█▌░░░░░░░░` **15 %** |
| `GameDaemon.exe` | World server: movement, skills, items, party, guild, chat, quests, shop, post, market | `gamedaemon` | 381 | 72 / 1598 (+365 partial) | `▌░░░░░░░░░` **4.5 %** (27 % started) |
| STS / NP login (`StsMsgSrv64.dll` peer) | Platform login: SRP-6a password login, RC4 session, game tokens | `sts` | 2020 client | 7 / 7 client commands | `██████████` **100 %** ❓ |
| `CacheDaemon.exe` | Shared character / world data cache | — | — | 0 | `░░░░░░░░░░` 0 % |
| `CacheGate.exe` | Gate in front of CacheDaemon | — | — | 0 | `░░░░░░░░░░` 0 % |
| `InfoGateDaemon.exe` | Info gate for LobbyDaemon and GameDaemon | — | — | 0 | `░░░░░░░░░░` 0 % |
| `AccountInventoryDaemon.exe` | Account-wide inventory (wardrobe, account storage) | — | — | 0 | `░░░░░░░░░░` 0 % |
| `AchievementDaemon.exe` | Achievements | — | — | 0 | `░░░░░░░░░░` 0 % |
| `ArenaLobby.exe` | Arena matchmaking lobby | — | — | 0 | `░░░░░░░░░░` 0 % |
| `ArenaDaemon` | Arena match server | — | — | 0 | `░░░░░░░░░░` 0 % |
| `DuelBotDaemon.exe` | Duel bot matches | — | — | 0 | `░░░░░░░░░░` 0 % |
| `MarketDealerDaemon.exe` | Marketplace back end | — | — | 0 | `░░░░░░░░░░` 0 % |
| `MarketAgent.exe` | Marketplace agent between GameDaemon and MarketDealerDaemon | — | — | 0 | `░░░░░░░░░░` 0 % |
| `PostOfficeDaemon.exe` | In-game mail | — | — | 0 | `░░░░░░░░░░` 0 % |
| `RankingDaemon.exe` | Rankings and leaderboards | — | — | 0 | `░░░░░░░░░░` 0 % |
| `SmartCouponDaemon.exe` | Coupon codes | — | — | 0 | `░░░░░░░░░░` 0 % |
| `ChannelChattingServer.exe` | Channel chat | — | — | 0 | `░░░░░░░░░░` 0 % |

Not counted: `ncgagent.exe`, `ncguardserver64.dll` and `ggauth64_27.dll` (anti-cheat, left out on purpose), `bsw.exe` and `StartServer` (service tools).

How each number was measured:

| Server | Source of the count |
| --- | --- |
| LobbyGate | All 56 commands in `Program/Server/Packet/tables/381.json` (client: not-authed 3, any-authed 2, authed 2 + 28 relayed; LobbyDaemon side 21) have a Rust handler in `Program/Server/LobbyGate/src/`. |
| LobbyDaemon | `Docs/Server/LobbyDaemon/command-tables.json` lists 118 handler rows in 5 tables. Rust handles 18 rows of the LobbyGate-side table (35 rows). The Manage (6), InfoGate (3), Game (51) and AccountInventory (23) tables have no Rust handlers yet; GameDaemon is a local stub. |
| GameDaemon | `Docs/Server/GameDaemon/parity-ledger.tsv`: 1598 commands; `rust` column = 72 ported, 365 partial, 1161 none. |
| STS | `Docs/Platform/Sts/STATUS.md`: 7 client XML commands are implemented. ❓ No real-client login has run yet. |

Descriptions of the servers with 0 % come from the binary names only. ❓ They are not verified.

## Rust crates (`bns-server/`)

| Crate path | Contents | In workspace |
| --- | --- | --- |
| `Program/Server/BloodLib` | Session, framing, packet fields, ciphers | Yes |
| `Program/Server/Packet` | Fragment packets, dispatch tables (`tables/381.json`) | Yes |
| `Program/Server/Shared` | NP channel trait, shared types, local stub | Yes |
| `Program/Server/LobbyGate` | Gate server, build 381 | Yes |
| `Program/Server/LobbyDaemon` | Lobby daemon, build 307 | Yes |
| `Program/Server/GameDaemon` | Game server, build 381 | Yes |
| `Program/Server/Data/Lmxb` | LMXBOSLB codec | Yes |
| `Program/Server/Data/GameData` | Typed game data | Yes |
| `Program/Platform/Sts` | NP / STS login server | Yes |
| `Program/Tools/DataFile/BnsData` | Reader and writer for LMXBOSLB / TADBOSLB data files | Yes |
| `Program/Tools/DataFile/BnsModel` | Data model for BnsData | Yes |
| `Program/Tools/DataFile/BnsSchema` | Passive TableDefs export from PE client files | Yes |
| `Program/Tools/ClientKey` | Handshake key pair generator and client patcher | Yes |
| `Program/Tools/BnsEdit` | BnsEdit GUI editor | **No**: build it from its own folder |

Build and check (from `bns-server/`):

```
cargo test
cargo clippy --workspace --all-targets -- -D warnings
```

Build BnsEdit:

```
cd bns-server/Program/Tools/BnsEdit && cargo build --release
```

## Documents

| Path | Contents |
| --- | --- |
| `bns-server/Docs/Server/LobbyGate/` | 01–05: config, handshake, framing, dispatch, handlers |
| `bns-server/Docs/Server/LobbyDaemon/` | 00–11: overview, config, links, DB, login, world entry, GameDaemon link, character select; JSON evidence |
| `bns-server/Docs/Server/GameDaemon/` | 01–25: tables, world connect, enter world, session, handshake key, parity, handler groups (movement, items, skills, party, guild, chat, quests, shop, post, market, arena); parity ledger and evidence |
| `bns-server/Docs/Platform/Sts/` | 01–09: transport, messages, login flow, token validation, password login, token key patch, token login server |
| `bns-server/Docs/DataFile/` | 01–10: LMXBOSLB, TADBOSLB, tabledefs, table body, strings, model, schema from exe; JSON exports |
| `bns-server/Docs/DataFile/client-exe/` | Python scripts for client exe and PDB analysis |
| `notes/client-datafiles.md` | `datafile.bin`, `datafile64.bin`, `locale.bin`: generation, format, keys, versions |
| `notes/client-exe-reversing.md` | Client exe reverse-engineering notes |
| `notes/client-381-game-commands.tsv` | Client build 381 game command table |
| `notes/sol-usage.jsonl` | Token usage log of the delegated coding sessions |

## Launcher (`launcher-src/`)

Fork of [notscuffed/BnsLauncher](https://github.com/notscuffed/BnsLauncher) (MIT, see `launcher-src/LICENSE`).

| Path | Contents |
| --- | --- |
| `Source/BnsLauncher/` | WPF app: views, view models, logging |
| `Source/BnsLauncher.Core/` | Profiles, services, DPAPI password storage, private server sample profile |
| `Source/BnsLauncher.Tests/` | Unit tests |
| `Source/BnsLauncher.sln` | Visual Studio solution |
| `Source/Publish.bat` | Publish script |

## Not in this repository

| Excluded | Reason |
| --- | --- |
| Game clients, server binaries, `.exe` / `.dll` / `.pdb` (except `dist/`) | NCSoft copyright |
| Databases (`.mdf`, `.ldf`, `.rar`), `datafile*.bin`, `locale.bin` | Game data |
| IDA databases, decompiled output | Derived from NCSoft binaries |
| `server-keys/`, `*.key`, `*.pem`, `*.blob` | Secrets |
| `target/`, `Program/output/`, `bin/`, `obj/` | Build output |
