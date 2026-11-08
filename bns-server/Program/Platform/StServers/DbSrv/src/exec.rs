//! Procedure execution (DbSrv sub_1400D4E10, sub_1400D35E0, sub_1400E6390).
//!
//! ODBC call `{?=call <Schema>.p_<Proc>(?,…)}` → tiberius batch:
//!
//! ```sql
//! DECLARE @__rv int = -1;                        -- v17 = -1
//! EXEC @__rv = [Schema].[p_Proc] @P1, @Name = @P2;
//! SELECT @__rv AS [__DbSrvReturn];               -- removed from the reply
//! ```
//!
//! | Case | Error | Source |
//! | --- | --- | --- |
//! | 32 params | code 4, line 977, no text | sub_1400E6390 |
//! | no connection | code 900, line 928, `stmt allocation is failed` | sub_1400D4E10 |
//! | call fails before the first set (also a missing procedure) | code 4, line 977, SQL message as text | sub_1400D4E10 |
//! | connection dead | log, wait 5 s, reconnect, try again once | sub_1400D4E10 |
//! | return value r ≠ 0 | code (i16)(r + 100000), 4 outside 1‥99900; line 1001; `<Response>` as aux | sub_1400D4E10, sub_1400D2E30 |
//! | error after a set | the reply is closed; r stays −1 → code 4, line 1001 | sub_1400D35E0 |
//!
//! `TransTimeoutSeconds` is read but its value is not used (sub_1400D3550
//! drops the result of sub_1400D6010), so no query timeout is set.

use crate::{
    config::SqlConfig,
    format,
    output::Output,
    request::{ExecRequest, Value, MODULE_EXEC},
};
use futures_util::TryStreamExt;
use std::time::{Duration, Instant};
use sts_core::{error::ERR_BAD_DATA, error::ERR_SQL_TIMEOUT, StsError};
use tiberius::{Client, Query, QueryItem};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

pub type Conn = Client<Compat<TcpStream>>;

const RETURN_COLUMN: &str = "__DbSrvReturn";
/// Highest procedure return code that maps to an error code (0x1863C).
const MAX_MAPPED: i32 = 99_900;
const SLOW_MS: u128 = 500;

/// `(i16)(r + 100000)` when 1 ≤ r + 100000 ≤ 99900, else 4 (ErrBadData).
pub fn return_code(rv: i32) -> u32 {
    let v = rv.wrapping_add(100_000);
    let v = if (1..=MAX_MAPPED).contains(&v) { v } else { 4 };
    v as i16 as i32 as u32
}

/// One ODBC diagnostic line (sub_1400D3A50): `[rc] [api] [proc]: message\n`.
pub fn diag(api: &str, proc_name: &str, driver: &str, e: &tiberius::error::Error) -> String {
    let msg = match e {
        tiberius::error::Error::Server(t) => format!("[Microsoft][{driver}][SQL Server]{}", t.message()),
        other => format!("[Microsoft][{driver}]{other}"),
    };
    format!("[-1] [{api}] [{proc_name}]: {msg}\n")
}

fn is_dead(e: &tiberius::error::Error) -> bool {
    !matches!(e, tiberius::error::Error::Server(_))
}

/// The `<Response>` text is attached to a return-code error unless it is
/// missing (partials sent) or it is one empty set (sub_1400D2E30).
pub fn aux(buf: &[u8]) -> Option<String> {
    let s = String::from_utf8_lossy(buf);
    let start = s.find("<Response type=\"array\">")?;
    let end = s.find("</Response>\n")? + 12;
    if end <= start {
        return None;
    }
    let r = &s[start..end];
    (r.len() > 0x3B || !r.contains("<Set type=\"array\"></Set>")).then(|| r.to_owned())
}

fn bracket(ident: &str) -> String {
    format!("[{}]", ident.replace(']', "]]"))
}

/// `@Name` with `[A-Za-z0-9_@#$]` only.
fn param_name(n: &str) -> Option<String> {
    let n = n.strip_prefix('@').unwrap_or(n);
    (!n.is_empty() && n.len() < 128 && n.bytes().all(|b| b.is_ascii_alphanumeric() || b"_@#$".contains(&b))).then(|| format!("@{n}"))
}

/// Builds the batch text and the bound values. `tvp_types[i]` is the
/// table type of param `i` (`[schema].[name]`), when it is a table.
pub fn batch(req: &ExecRequest, tvp_types: &[Option<String>]) -> Result<(String, Vec<Option<String>>), String> {
    let mut sql = String::from("DECLARE @__rv int = -1;\n");
    let mut binds: Vec<Option<String>> = Vec::new();
    let mut args = Vec::new();
    let bind = |v: Option<String>, binds: &mut Vec<Option<String>>| {
        binds.push(v);
        format!("@P{}", binds.len())
    };
    for (i, p) in req.params.iter().enumerate() {
        let arg = match &p.value {
            Value::Null => bind(None, &mut binds),
            Value::VarChar(s) | Value::NVarChar(s) => bind(Some(s.clone()), &mut binds),
            Value::Table(rows) => {
                let ty = tvp_types.get(i).cloned().flatten().ok_or_else(|| format!("param {} is not a table type", i + 1))?;
                let var = format!("@__t{}", i + 1);
                sql.push_str(&format!("DECLARE {var} {ty};\n"));
                for chunk in rows.chunks(1000) {
                    let values: Vec<String> = chunk
                        .iter()
                        .map(|r| format!("({})", r.iter().map(|c| bind(c.clone(), &mut binds)).collect::<Vec<_>>().join(",")))
                        .collect();
                    sql.push_str(&format!("INSERT INTO {var} VALUES {};\n", values.join(",")));
                }
                var
            }
        };
        args.push(match &p.name {
            Some(n) => format!("{} = {arg}", param_name(n).ok_or_else(|| format!("bad parameter name {n}"))?),
            None => arg,
        });
    }
    if binds.len() > 2000 {
        return Err("too many values".into());
    }
    sql.push_str(&format!("EXEC @__rv = {}.{}", bracket(&req.schema), bracket(&req.proc_name)));
    if !args.is_empty() {
        sql.push(' ');
        sql.push_str(&args.join(", "));
    }
    sql.push_str(&format!(";\nSELECT @__rv AS {};", bracket(RETURN_COLUMN)));
    Ok((sql, binds))
}

/// Connection pool of one logical database. One connection per worker.
pub struct Executor {
    pub cfg: SqlConfig,
    idle: Mutex<Vec<Conn>>,
}

enum Fail {
    /// Nothing was written; `dead` = the connection is gone.
    Before { text: String, dead: bool },
}

impl Executor {
    pub fn new(cfg: SqlConfig) -> Self {
        Self { cfg, idle: Mutex::new(Vec::new()) }
    }

    pub async fn connect(&self) -> Result<Conn, tiberius::error::Error> {
        let config = self.cfg.tiberius();
        let tcp = TcpStream::connect(config.get_addr()).await?;
        tcp.set_nodelay(true)?;
        Client::connect(config, tcp.compat_write()).await
    }

    async fn take(&self) -> Option<Conn> {
        if let Some(c) = self.idle.lock().await.pop() {
            return Some(c);
        }
        match self.connect().await {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("DbTrans[{}]: connect failed: {e}", self.cfg.logical);
                None
            }
        }
    }

    async fn tvp_types(&self, conn: &mut Conn, req: &ExecRequest) -> Result<Vec<Option<String>>, tiberius::error::Error> {
        if !req.params.iter().any(|p| matches!(p.value, Value::Table(_))) {
            return Ok(vec![]);
        }
        let full = format!("{}.{}", bracket(&req.schema), bracket(&req.proc_name));
        let rows = conn
            .query(
                "SELECT p.parameter_id, p.name, QUOTENAME(SCHEMA_NAME(t.schema_id)) + '.' + QUOTENAME(t.name) \
                 FROM sys.parameters p JOIN sys.table_types t ON p.user_type_id = t.user_type_id \
                 WHERE p.object_id = OBJECT_ID(@P1)",
                &[&full],
            )
            .await?
            .into_first_result()
            .await?;
        let types: Vec<(i32, String, String)> = rows
            .iter()
            .filter_map(|r| Some((r.get::<i32, _>(0)?, r.get::<&str, _>(1)?.to_owned(), r.get::<&str, _>(2)?.to_owned())))
            .collect();
        Ok(req
            .params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                types
                    .iter()
                    .find(|(id, name, _)| match &p.name {
                        Some(n) => param_name(n).is_some_and(|n| n.eq_ignore_ascii_case(name)),
                        None => *id == i as i32 + 1,
                    })
                    .map(|t| t.2.clone())
            })
            .collect())
    }

    /// Runs one call. `Ok` = the reply body is `out.buf`.
    pub async fn exec(&self, req: &ExecRequest, out: &mut Output) -> Result<(), StsError> {
        let started = Instant::now();
        let r = self.exec_inner(req, out).await;
        let ms = started.elapsed().as_millis();
        if ms > SLOW_MS {
            eprintln!("Execute procedure({}) took {ms} ms", req.proc_name);
        }
        r
    }

    async fn exec_inner(&self, req: &ExecRequest, out: &mut Output) -> Result<(), StsError> {
        // sub_1400E6390 builds the call text only for ≤ 31 params (32 `?` minus
        // the return value); 32 params fail with no diagnostic text.
        if req.params.len() >= crate::request::MAX_PARAMS {
            return Err(StsError::new(ERR_BAD_DATA, MODULE_EXEC, 977));
        }
        let mut retried = false;
        loop {
            let Some(mut conn) = self.take().await else {
                return Err(StsError::new(ERR_SQL_TIMEOUT, MODULE_EXEC, 928).with_text("stmt allocation is failed"));
            };
            match self.run(&mut conn, req, out).await {
                Ok(rv) => {
                    self.idle.lock().await.push(conn);
                    return match rv {
                        (0, _) => Ok(()),
                        (rv, text) => {
                            let mut e = StsError::new(return_code(rv), MODULE_EXEC, 1001).with_text(text);
                            match aux(&out.buf) {
                                Some(a) => match sts_core::Element::parse(a.as_bytes()) {
                                    Ok(el) => e = e.with_extra(el),
                                    Err(_) => eprintln!("Not completed auxillary error info for ({}:0:{}:{}), ignored", e.code, e.module, e.line),
                                },
                                None => eprintln!("Not completed auxillary error info for ({}:0:{}:{}), ignored", e.code, e.module, e.line),
                            }
                            Err(e)
                        }
                    };
                }
                Err(Fail::Before { text, dead }) => {
                    if dead && !retried {
                        retried = true;
                        drop(conn);
                        eprintln!("CDbTrans::Transact: SqlConnExecStoredProcedureA failed: connection dead, wait & try again");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                    if !dead {
                        self.idle.lock().await.push(conn);
                    }
                    return Err(StsError::new(ERR_BAD_DATA, MODULE_EXEC, 977).with_text(text));
                }
            }
        }
    }

    /// `Ok((return value, diagnostic text))` once the `<Response>` is written.
    async fn run(&self, conn: &mut Conn, req: &ExecRequest, out: &mut Output) -> Result<(i32, String), Fail> {
        let driver = self.cfg.driver.clone();
        let before = |api: &str, e: tiberius::error::Error| Fail::Before { text: diag(api, &req.proc_name, &driver, &e), dead: is_dead(&e) };
        let tvp = self.tvp_types(conn, req).await.map_err(|e| before("SQLBindParameter", e))?;
        let (sql, binds) = batch(req, &tvp).map_err(|m| Fail::Before { text: format!("[-1] [SQLBindParameter] [{}]: {m}\n", req.proc_name), dead: false })?;
        let mut q = Query::new(sql);
        for b in binds {
            q.bind(b);
        }
        let mut stream = q.query(conn).await.map_err(|e| before("SqlExecute", e))?;
        // The first item tells whether the call itself failed.
        let first = stream.try_next().await.map_err(|e| before("SqlExecute", e))?;
        let mark = out.buf.len();
        out.raw("<Response type=\"array\">");
        let mut rv = -1;
        let mut started = false;
        let mut text = String::new();
        let mut cols: Option<Vec<(String, tiberius::ColumnType)>> = None;
        let mut is_return = false;
        let mut item = first;
        loop {
            match item {
                None => break,
                Some(QueryItem::Metadata(m)) => {
                    if cols.take().is_some() && !is_return {
                        out.raw("</Set>");
                    }
                    let c: Vec<_> = m.columns().iter().map(|c| (c.name().chars().take(64).collect::<String>(), c.column_type())).collect();
                    is_return = c.len() == 1 && c[0].0 == RETURN_COLUMN;
                    if !is_return {
                        started = true;
                        out.raw("<Set type=\"array\">");
                    }
                    cols = Some(c);
                }
                Some(QueryItem::Row(row)) => {
                    if is_return {
                        rv = row.get::<i32, _>(0).unwrap_or(-1);
                    } else {
                        let cols = cols.as_ref().expect("row after metadata");
                        out.raw("<Row>");
                        for ((name, ty), (_, data)) in cols.iter().zip(row.cells()) {
                            if let Some(t) = format::cell(*ty, data) {
                                out.raw("<");
                                out.raw(name);
                                out.raw(">");
                                out.text(&t);
                                out.raw("</");
                                out.raw(name);
                                out.raw(">");
                            }
                        }
                        out.raw("</Row>");
                    }
                }
            }
            item = match stream.try_next().await {
                Ok(i) => i,
                Err(e) if !started => {
                    // No set yet: ODBC reports the error from SQLExecute (line 977).
                    out.buf.truncate(mark);
                    return Err(before("SqlExecute", e));
                }
                Err(e) => {
                    // sub_1400D35E0 stops; the return value stays −1.
                    text = diag("SQLMoreResults", &req.proc_name, &driver, &e);
                    rv = -1;
                    break;
                }
            };
        }
        if cols.is_some() && !is_return {
            out.raw("</Set>");
        }
        out.raw("</Response>\n");
        Ok((rv, text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::Param;

    #[test]
    fn return_codes() {
        assert_eq!(return_code(3001 - 100_000), 3001);
        assert_eq!(return_code(-1), 4);
        assert_eq!(return_code(5), 4);
        assert_eq!(return_code(-100_000), 4);
    }

    #[test]
    fn aux_rule() {
        assert_eq!(aux(b"<Response type=\"array\"><Set type=\"array\"></Set></Response>\n"), None);
        assert!(aux(b"<Response type=\"array\"></Response>\n").is_some());
        assert!(aux(b"<Response type=\"array\"><Set type=\"array\"><Row><a>1</a></Row></Set></Response>\n").is_some());
        assert_eq!(aux(b"<Row/></Set></Response>\n"), None);
    }

    #[test]
    fn batch_text() {
        let req = ExecRequest {
            schema: "v1".into(),
            proc_name: "p_Get".into(),
            params: vec![
                Param { name: None, value: Value::text("a") },
                Param { name: None, value: Value::Null },
                Param { name: None, value: Value::Table(vec![vec![Some("x".into()), None]]) },
            ],
        };
        let (sql, binds) = batch(&req, &[None, None, Some("[dbo].[T]".into())]).unwrap();
        assert_eq!(
            sql,
            "DECLARE @__rv int = -1;\nDECLARE @__t3 [dbo].[T];\nINSERT INTO @__t3 VALUES (@P3,@P4);\nEXEC @__rv = [v1].[p_Get] @P1, @P2, @__t3;\nSELECT @__rv AS [__DbSrvReturn];"
        );
        assert_eq!(binds, vec![Some("a".into()), None, Some("x".into()), None]);
        let named = ExecRequest { params: vec![Param { name: Some("@Id".into()), value: Value::text("5") }], ..req.clone() };
        assert!(batch(&named, &[]).unwrap().0.contains("[p_Get] @Id = @P1;"));
        let bad = ExecRequest { params: vec![Param { name: Some("a;b".into()), value: Value::Null }], ..req };
        assert!(batch(&bad, &[]).is_err());
    }
}
