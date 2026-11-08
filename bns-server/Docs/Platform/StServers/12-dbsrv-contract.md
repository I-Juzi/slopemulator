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

| Element | Caller type | ODBC binding |
| --- | --- | --- |
| `<I>` | integer (0) | integer |
| `<S>` | string (2, 3, 5) | SQL_VARCHAR (12), or SQL_WVARCHAR (−9) when the text has bytes ≥ 0x80 |
| `<G>` | GUID (4) | GUID |
| `<T>` | table-valued param: `<T><row><col/>…</row>…</T>`; all rows must have the same column count (line 738) | SQL_SS_TABLE (−153) |

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
| Text | XML-escaped (sub_1400D30B0) ❓ escape set not checked | |
| Result sets | `SQLMoreResults` loop, one `<Set>` each | DbSrv sub_1400D4E10 |
| Return value | `RETURN r` with r ≠ 0 → error reply, code = r + 100000 when 1 ≤ code ≤ 99 900, else 4 (ErrBadData); line 1001 | sub_1400D4E10 |
| Dead connection | retry once after 5 s (`connection dead, wait & try again`) | sub_1400D4E10 |
| Shutdown | error line 921 `server is shutting down` | sub_1400D4E10 |
| No statement handle | code 900 (ErrSqlTimeout) line 928 | sub_1400D4E10 |

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
| DbSrv | STS listener; `Exec`/`ExecBatched` → `tiberius` RPC call `<Schema>.p_<Proc>` with the typed params; same XML reply and return-code rule |
| Services | build the same `Db.<Logical>/Exec` requests; read rows by column name |
| Stored procedures | reuse the original procedures from the restored databases |
| Secrets | `Db.ini` passwords → `keyhere` in the shipped config |
