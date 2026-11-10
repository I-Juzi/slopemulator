//! DbConn request handler (DbSrv off_1401D1538 → sub_1400D2860 area).
//!
//! | Order | Check | Error (module 1) |
//! | --- | --- | --- |
//! | 1 | body missing | code 10, line 71, `DbConn: NULL body` |
//! | 2 | protocol `Storage` | StorageSrv path (not in this crate) → as 3 ❓ |
//! | 3 | protocol ≠ `Db.<Logical>` (case-sensitive) | code 10, line 101, `DbConn: unknown protocol %s` |
//! | 4 | command ∉ {`Exec`, `ExecBatched`} | code 51, line 119, `DbConn: unknown command %s` |
//!
//! Deviation: one process may serve several logical databases (the
//! original serves one, given at start-up).

use crate::{
    exec::Executor,
    output::{Output, Sink},
    request::{ExecRequest, MODULE_CONN},
};
use std::{collections::HashMap, sync::Arc};
use sts_core::{
    conn::{reply_raw, Incoming, Peer},
    error::ERR_BAD_PARAM,
    StsError,
};
use tokio::{net::TcpListener, sync::Semaphore};

/// ErrUnknownCommand? (code 51, DbConn line 119).
pub const ERR_UNKNOWN_COMMAND: u32 = 51;
/// Code of the line-921 reply when a queued call is cancelled at shutdown ❓.
pub const SHUTDOWN_CODE: u32 = 1;

pub struct DbServer {
    /// `Db.<Logical>` → executor.
    pub dbs: HashMap<String, Arc<Executor>>,
    pub server_id: u32,
    pub workers: Arc<Semaphore>,
}

fn conn_err(code: u32, line: u32, text: String) -> StsError {
    StsError::new(code, MODULE_CONN, line).with_text(text)
}

impl DbServer {
    pub fn new(executors: Vec<Executor>, server_id: u32, workers: usize) -> Self {
        let dbs = executors
            .into_iter()
            .map(|e| (format!("Db.{}", e.cfg.logical), Arc::new(e)))
            .collect();
        Self {
            dbs,
            server_id,
            workers: Arc::new(Semaphore::new(workers.max(1))),
        }
    }

    /// Checks 1–4; `Ok((executor, batched, request))`.
    pub fn route(
        &self,
        protocol: &str,
        command: &str,
        body: Option<&sts_core::Element>,
    ) -> Result<(Arc<Executor>, bool, ExecRequest), StsError> {
        let body = body.ok_or_else(|| conn_err(ERR_BAD_PARAM, 71, "DbConn: NULL body".into()))?;
        let exec = self.dbs.get(protocol).ok_or_else(|| {
            conn_err(
                ERR_BAD_PARAM,
                101,
                format!("DbConn: unknown protocol {protocol}"),
            )
        })?;
        let batched = match command {
            "Exec" => false,
            "ExecBatched" => true,
            _ => {
                return Err(conn_err(
                    ERR_UNKNOWN_COMMAND,
                    119,
                    format!("DbConn: unknown command {command}"),
                ))
            }
        };
        Ok((exec.clone(), batched, ExecRequest::parse(body)?))
    }

    pub async fn handle(self: Arc<Self>, req: Incoming) {
        let (protocol, command) = req.command();
        if protocol == "Sts" {
            // Sts/Connect and other link messages: plain OK.
            req.reply(None);
            return;
        }
        let body = if req.frame.body.is_empty() {
            None
        } else {
            req.body()
        };
        let (exec, batched, call) = match self.route(protocol, command, body.as_ref()) {
            Ok(r) => r,
            Err(e) => return req.reply_error(&e, self.server_id),
        };
        let Ok(_permit) = self.workers.clone().acquire_owned().await else {
            let e = StsError::new(SHUTDOWN_CODE, crate::request::MODULE_EXEC, 921)
                .with_text("server is shutting down");
            return req.reply_error(&e, self.server_id);
        };
        let id = req.id();
        let sink: Option<Sink> = batched.then(|| {
            let peer: Peer = req.peer.clone();
            let mut seq = 0u32;
            Box::new(move |body: Vec<u8>| {
                seq += 1;
                if id != 0 {
                    peer.send(reply_raw(id, 200, "OK", body, false, seq));
                }
            }) as Sink
        });
        let mut out = Output::new(sink);
        let result = exec.exec(&call, &mut out).await;
        if id == 0 {
            return;
        }
        // Final reply: `R` alone, or `R<partials + 1>` after partials.
        let seq = if out.partials > 0 {
            out.partials + 1
        } else {
            0
        };
        let frame = match result {
            Ok(()) => reply_raw(id, 200, "OK", std::mem::take(&mut out.buf), true, seq),
            Err(e) => reply_raw(
                id,
                400,
                e.name(),
                e.to_element(self.server_id).to_xml().into_bytes(),
                true,
                seq,
            ),
        };
        req.peer.send(frame);
    }

    /// Accepts STS connections until the listener fails or `shutdown` resolves.
    pub async fn serve(
        self: Arc<Self>,
        listener: TcpListener,
        shutdown: impl std::future::Future<Output = ()>,
    ) -> std::io::Result<()> {
        tokio::pin!(shutdown);
        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    self.workers.close();
                    return Ok(());
                }
                acc = listener.accept() => {
                    let (sock, addr) = acc?;
                    let _ = sock.set_nodelay(true);
                    eprintln!("DbSrv: connection from {addr}");
                    let (_peer, mut rx) = Peer::spawn(sock);
                    let me = self.clone();
                    tokio::spawn(async move {
                        while let Some(req) = rx.recv().await {
                            tokio::spawn(me.clone().handle(req));
                        }
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SqlConfig;
    use sts_core::{ini::Ini, Element};

    fn server() -> DbServer {
        let ini = Ini::parse("[Sql.AcctDb]\nDatabase=PlatformAcctDb\nAddress=127.0.0.1:1\nUsername=sa\nPassword=keyhere\n");
        DbServer::new(
            vec![Executor::new(SqlConfig::from_ini(&ini, "AcctDb").unwrap())],
            7,
            2,
        )
    }

    #[test]
    fn conn_errors() {
        let s = server();
        let body = Element::parse(b"<Request><ProcName>X</ProcName></Request>").unwrap();
        let e = s.route("Db.AcctDb", "Exec", None).err().unwrap();
        assert_eq!(
            (e.code, e.module, e.line, e.text.as_str()),
            (10, 1, 71, "DbConn: NULL body")
        );
        let e = s.route("Db.acctdb", "Exec", Some(&body)).err().unwrap();
        assert_eq!(
            (e.code, e.line, e.text.as_str()),
            (10, 101, "DbConn: unknown protocol Db.acctdb")
        );
        let e = s.route("Db.AcctDb", "Query", Some(&body)).err().unwrap();
        assert_eq!((e.code, e.line), (51, 119));
        let (_, batched, r) = s.route("Db.AcctDb", "ExecBatched", Some(&body)).unwrap();
        assert!(batched);
        assert_eq!(r.proc_name, "p_X");
    }
}
