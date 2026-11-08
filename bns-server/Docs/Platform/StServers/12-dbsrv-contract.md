# 12 — Service ↔ DbSrv contract (original StServers)

Sources: `DbSrv.exe`, `AuthSrv.exe` (NP 1.20.3350), decompiles in `BNSX-files/decomp/`.
`Config/Db.ini` gives the logical-name map. Passwords stay out; the port writes `keyhere`.

## 1. Architecture

```
AuthSrv / GameAccountSrv / SlotSrv / SessionGateSrv / WorldSrv …
   │  POST /Db.<Logical>/Exec STS/1.0      (route grid)
   ▼
DbSrv ── [Sql.<Logical>] in Db.ini ── ODBC ──► MSSQL  {? = call <Schema>.p_<ProcName>(?, …)}
```

The services hold almost no SQL. The business rules live in the stored procedures
(`BNSX-files/sql-dump/<Db>/modules.sql`). Each service pins one schema version.

## 2. Request

| Item | Value | Source |
| --- | --- | --- |
| Command | `Exec` (one call) or `ExecBatched`; anything else → `Invalid` | AuthSrv sub_1401125E0 |
| Protocol | `Db.<Logical>`, for example `Db.AcctDb`; a name that starts with `$` is used as is | AuthSrv sub_1401125E0 |
| Body | `<Request><SchemaName/>?<ProcName/><Params>…</Params></Request>` | AuthSrv sub_140112330 |
| SchemaName | optional, default `dbo`; `[A-Za-z0-9]`, < 128 chars, else ErrBadParam line 621 | DbSrv sub_1400D3B90 |
| ProcName | `[A-Za-z0-9]`, < 128 chars (line 632); DbSrv adds the prefix `p_` | DbSrv sub_1400D3B90 |
| Params | positional, max 32 (line 658); optional `name=` attribute; `type="null"` sends NULL | DbSrv sub_1400D3B90 |

### Parameter elements

Only the element name `T` is special. DbSrv does not look at the other names
(`<I>`, `<S>`, `<G>` are caller conventions). Source: DbSrv sub_1400D3B90.

| Element | ODBC binding | Column size |
| --- | --- | --- |
| any, all bytes < 0x80 | SQL_VARCHAR (12), C type CHAR | length; 0 (max) when > 8000 |
| any, a byte ≥ 0x80 | SQL_WVARCHAR (−9), C type WCHAR (−8) | UTF-16 length; 0 when > 4000 |
| `type="null"` | NULL | — |
| `name="@x"` | bound by name (SQL_DESC_NAME); all params named or none, else code 10 line 828 | — |
| `<T><R><C/>…</R>…</T>` | SQL_SS_TABLE (−153); cells varchar, or wvarchar when a cell has a byte ≥ 0x80; same column count in every row (line 738) | — |

No database in the SQL dump has a table type (`AS TABLE`), so `T` is not used in build 381.

### Call text

`{?=call <Schema>.<p_Proc>(?,…)}` (sub_1400E6390). The return value is output param 1,
initialised to −1. With 32 user params the text is not built: code 4, line 977, no text.

## 2a. Connection handler (module 1)

| Order | Check | Error |
| --- | --- | --- |
| 1 | body missing | code 10, line 71, text `DbConn: NULL body` |
| 2 | protocol `Storage` | StorageSrv code path (sub_1400C6480), only with `Storage.ini` |
| 3 | protocol ≠ `Db.<Logical>` (case-sensitive, sub_1400C73A0) | code 10, line 101, `DbConn: unknown protocol %s` |
| 4 | command ∉ {`Exec`, `ExecBatched`} | code 51, line 119, `DbConn: unknown command %s` |

One process serves one logical database. The start-up argument gives the name (sub_1400D3550).

## 3. Reply

```xml
<Response type="array">
  <Set type="array">            <!-- one per result set -->
    <Row><ColumnName>text</ColumnName>…</Row>
  </Set>
</Response>
```

| Rule | Detail | Source |
| --- | --- | --- |
| Column element | the SQL column name | DbSrv sub_1400D35E0 |
| NULL column | element left out | sub_1400D35E0 (`ind == -1`) |
| Wide columns | read as SQL_C_WCHAR (−8), sent as UTF-8 | sub_1400D35E0 |
| Text | only `&` → `&amp;` and `<` → `&lt;` | sub_1400D30B0 |
| Column name | max 64 chars; data read in 4096-byte pieces | sub_1400D35E0 |
| Empty result (0 columns) | stops the set loop; all procs use `SET NOCOUNT ON` | sub_1400D35E0 |
| Result sets | `SQLMoreResults` loop, one `<Set>` each | DbSrv sub_1400D4E10 |
| Return value | `RETURN r` with r ≠ 0 → error reply, code = r + 100000 when 1 ≤ code ≤ 99 900, else 4 (ErrBadData); line 1001 | sub_1400D4E10 |
| Aux data | on a return-code error the `<Response>` text is attached under `<Error>`, except when it is ≤ 59 bytes and has an empty set (one empty set only) | sub_1400D2E30 |
| Error after a set | the loop stops, `</Response>` is closed, r stays −1 → code 4, line 1001 | sub_1400D35E0 |
| Error text | `[rc] [api] [proc]: message\n`, api = `SqlExecute`, `SQLMoreResults`, … | sub_1400D3A50, sub_1400E5420 |
| Slow call | log `Execute procedure(%s) took %u ms` above 500 ms | sub_1400D4E10 |
| Dead connection | retry once after 5 s (`connection dead, wait & try again`) | sub_1400D4E10 |
| Shutdown | error line 921 `server is shutting down` | sub_1400D4E10 |
| No statement handle | code 900 (ErrSqlTimeout) line 928 | sub_1400D4E10 |

Error module: 2 for execution errors (sub_1400D2B80 sets +1376 = 2), 1 for the handler.

### Partial replies (`ExecBatched`)

| Rule | Detail | Source |
| --- | --- | --- |
| Split | before a piece is added, when buffer + piece ≥ 0xE666, the buffer is sent as a partial reply | sub_1400D2F70 |
| Header | partials `s:<id>r1`, `r2`, …; final `s:<id>R<n+1>`; without partials plain `R` | sub_1400DF2D0, sub_1400EC5D0 |
| Receiver | joins the bodies ❓ (not traced) | |

### Db.ini `[Sql.<Logical>]`

| Key | Rule | Source |
| --- | --- | --- |
| Database | default: computer name | sub_1400D3550 |
| Driver | default `SQL Server Native Client 10.0` | sub_1400D3550 |
| Address | `ip`, `ip:port` (`:` → `,`), `host/instance` | sub_1400D3550 |
| Mirror | `FAILOVER_PARTNER` | sub_1400D3550 |
| Username + Password | SQL login; else `Trusted_Connection` | sub_1400D3550 |
| EncryptedPassword | Base64 + RSA with the `ServerPrName` key (default `plaync.pr`, `ServerKeyBits` 2048, `[DbSrvOptions]`) | sub_1400D3550 |
| TransTimeoutSeconds | read (default 20) but the value is not used | sub_1400D3550 |
| `[ThreadTasks] ThreadTasks` | worker count | |

A value with `;` is logged and cleared. Connection string `DRIVER=%s;NETWORK=DbNetLib;SERVER=%.64s;DATABASE=%s;`.

So the SQL convention `RETURN (3000 + n - 100000)` gives the STS error code `3000 + n`.

Callers read the first row as `Reply → Set → Row` (AuthSrv sub_140112500), and fields by column name.

## 4. Services, logical DBs and schemas

| Service | Logical DB (Db.ini) | Physical DB | Schema | Procs found |
| --- | --- | --- | --- | --- |
| AuthSrv | AcctDb | PlatformAcctDb | v2016102001 | 68 |
| GameAccountSrv | GameDb | PlatformGameDb | v2016120901 | 15 |
| WorldSrv | GameDb | PlatformGameDb | v2016120901 | 6 |
| SlotSrv | SlotDb | PlatformSlotDb | v2016041101 | 20 |
| SessionGateSrv | SessionDb | PlatformSessionDb | v2015102301 | 6 |
| CliGate, GameGate, RouteSrv | — | — | — | 0 (no SQL) |

`procs.tsv` lists every proc per service. Method: every string literal in the decompile that
equals a `p_<name>` in the pinned schema of the SQL dump.

## 5. Consequence for the Rust port

| Part | Port approach |
| --- | --- |
| DbSrv | done: `Program/Platform/StServers/DbSrv` (see §6) |
| StorageSrv | separate exe (`GetStorageByKey`, `CreateStorage`); port later |
| Services | build the same `Db.<Logical>/Exec` requests; read rows by column name |
| Stored procedures | reuse the original procedures from the restored databases |
| Secrets | `Db.ini` passwords → `keyhere` in the shipped config |

## 6. Rust port (`Program/Platform/StServers/DbSrv`)

```
DbSrv --config Db.ini --db AcctDb [--db GameDb …] [--listen 127.0.0.1:10900] [--server-id N]
```

| Item | Port | Parity |
| --- | --- | --- |
| Call | batch `DECLARE @__rv int = -1; EXEC @__rv = [S].[p_X] @P1, @n = @P2; SELECT @__rv AS [__DbSrvReturn]`; the last set is removed | same reply |
| Param types | every value is sent as nvarchar; the procedure converts | ❓ varchar code-page bytes |
| TVP | table variable typed from `sys.parameters` + `INSERT … VALUES` | not testable (no table types) |
| Values | int decimal; bit 1/0; decimal `.50`; money 4 decimals; datetime `.fff`; datetime2/time n digits; datetimeoffset `+hh:mm`; GUID and binary upper case | float text ❓ |
| Info messages | not seen by tiberius | ❓ SUCCESS_WITH_INFO text |
| Several `--db` | one process, several logical DBs | deviation |
| Route registration | direct STS listener only; RouteSrv provider registration comes with the RouteSrv port | not ported yet |
| EncryptedPassword | not supported; use `Password=keyhere` | deviation |
| Shutdown | queued calls get line 921, code 1 ❓ | |

Live test (bns-mssql, 8 Nov 2026): typed values, `&`/`<` escape, return code 3004 with aux,
empty-set aux dropped (3010), missing procedure → 977, error after a set → 1001 with aux,
NULL body → module 1 line 71, 32 params → 977 without text, ExecBatched → `r1`, `r2`, `R3`
with 58981-byte partials.

### Check against the shipped config and the SQL dump (8 Nov 2026)

"Not ported yet" = the original has it and the port will get it. "None" = the shipped setup does not use it.

| Item | Check | Effect on build 381 |
| --- | --- | --- |
| RouteSrv registration | DbSrv.c ~178913 looks up `RouteSrv` / `RouteSrv.<name>` and connects (`Can't find RouteSrv(%s) for %s(%u)`) | not ported yet: original services cannot reach the port until RouteSrv is ported |
| `Storage` protocol | sub_1400C5B40: the path is on only if `[Storage] DbSrv` equals this process's own DB name (word_1402216B0, set from `--db`, used in `Db.%S`). `Storage.ini` names `StorageDb`. `StServer.ini` starts no DbSrv for `StorageDb`, `Db.ini` has no `[Sql.StorageDb]`, the dump has no StorageDb, and `MongoDbSrv` is commented out | none: the path is off in all 33 shipped DbSrv processes. Port it with StorageSrv (Redis + Mongo) only if a deployment needs it |
| EncryptedPassword, Mirror, Driver | shipped `Db.ini`: 33 sections, keys Address, Database, Username, Password only | none |
| float / real / money columns | `columns.txt` of PlatformAcctDb, GameDb, SlotDb, SessionDb: 0 | none for these DBs |
| PRINT / RAISERROR (info messages) | `modules.sql` of the same DBs: 0 | none for these DBs |
| varchar columns | AcctDb 16, GameDb 13, SlotDb 5, SessionDb 2; collation `Chinese_Simplified_Stroke_Order_100_CI_AS` (code page 936). bns-mssql scan, `LIKE '%[^ -~]%'` (binary collation): 0 non-ASCII values in every column. The columns hold IPs, hashes, salts, codes and IDs | none: the text is ASCII, and ASCII is the same in CP936 and UTF-8 |
