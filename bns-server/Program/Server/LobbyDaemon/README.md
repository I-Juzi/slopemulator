# lobbyd — first local milestone

This crate implements a local LobbyDaemon subset from the checked build 307 spec.
It does not load the original executable.

| Component | Scope |
| --- | --- |
| Binary `lobbyd` | Loopback listener. Optional port argument. Port 0 selects a free port. Ctrl-C stops the server. |
| `protocol` | D1/D2/D3, D7/D8, D10/D11, D20/D21/D22 and D36/D37 wire formats. |
| `jobs` | NP Init/Start, cached character list, close/logout and local notification controls. |
| `store` | 12 persistence methods. MemoryStore uses explicit generated fixtures. |
| `mssql` | Optional SQL Server Store. 16 fixed procedure calls use bound RPC parameters. See `Docs/Server/LobbyDaemon/08-db-binding.md` for limits. |
| `np` | Deterministic local fixtures, token replay checks, presence ownership and failure/timeout fixtures. No transport. |

Run `cargo run --manifest-path bns-server/Cargo.toml -p lobbyd -- 0` from the project root.
The binary starts with an empty NP stub. It rejects all tokens.
Use the library to install generated fixtures. The integration test shows this setup.
Do not use sample credentials or original platform services.

Run the workspace tests with `cargo test --manifest-path bns-server/Cargo.toml`.
Run the new-crate lint check with `cargo clippy --manifest-path bns-server/Cargo.toml -p lobbyd --all-targets -- -D warnings`.

See `Docs/Server/LobbyDaemon/05-rust-milestone.md` for local policy rules.
The five-second NP deadline is a local setting. It is not a recovered original value.
`Server::close_anonymous` emits D20. `Server::kick` emits D22.
These controls do not implement the original expiry or management scheduler.
Nonempty incomplete-service rows fail login explicitly.
Retained-session login and world entry use local deterministic controls.
Optional platform refresh and real GameDaemon transport stay disabled.
NP waits use separate ordered link lanes. Store calls stay synchronous.

## Optional SQL Server Store

Build with `--features mssql`. The CLI selects SQL only when `LOBBYD_MSSQL_URL`
is set at run time. Use an ADO-style value in the operator environment.
No endpoint or credential default is provided. The empty NP stub still rejects tokens.
Do not put connection values into source files or test output.

Run operator SQL tests with:
`cargo test --manifest-path bns-server/Cargo.toml -p lobbyd --features mssql --test mssql -- --nocapture --test-threads=1`.
The three SQL tests print SKIP when the variable is unset. They use generated rows
and scoped cleanup. Use a disposable recreated LobbyDB, not a production DB.
Non-NULL character timestamps reject until their wire units are specified.
Restricted rows and BotScoreIgnore also reject. Representative-world stays zero.
SQL calls use a dedicated worker but block the synchronous Store caller.
The local SQL operation timeout is ten seconds. Failed writes are never retried.
