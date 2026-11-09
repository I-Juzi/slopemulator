//! Host side (runs on Linux): accepts one plugin, checks the token, logs
//! events to a JSONL file and forwards commands from the control port.

use crate::addr::ida_to_live;
use crate::proto::{decode_hello, decode_request, Hello};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};

const HELLO_TIMEOUT: Duration = Duration::from_secs(5);
const CMD_TIMEOUT: Duration = Duration::from_secs(15);
const HELLO_MAX: usize = 4096;
const LINE_MAX: usize = 4 << 20;

/// Addresses the host is listening on (for tests that bind port 0).
#[derive(Debug, Clone, Copy)]
pub struct Bound {
    pub plugin: SocketAddr,
    pub ctl: SocketAddr,
}

pub struct ServeConfig {
    /// Address for the plugin, for example `100.77.181.120:47100`.
    pub listen: String,
    /// Control port, for example `127.0.0.1:47101`.
    pub ctl: String,
    pub token: String,
    /// Directory for `events-<date>.jsonl`.
    pub log_dir: PathBuf,
    /// Receives the bound addresses when both listeners are ready.
    pub ready: Option<oneshot::Sender<Bound>>,
}

struct Link {
    gen: u64,
    hello: Hello,
    tx: mpsc::UnboundedSender<String>,
    pending: Mutex<HashMap<u64, oneshot::Sender<String>>>,
}

struct EventLog {
    dir: PathBuf,
    cur: Mutex<Option<(String, File)>>,
}

impl EventLog {
    fn write_line(&self, line: &str) -> io::Result<()> {
        let date = today();
        let mut cur = self.cur.lock().unwrap_or_else(|e| e.into_inner());
        if cur.as_ref().map(|c| c.0 != date).unwrap_or(true) {
            fs::create_dir_all(&self.dir)?;
            let f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.dir.join(format!("events-{date}.jsonl")))?;
            *cur = Some((date, f));
        }
        if let Some((_, f)) = cur.as_mut() {
            f.write_all(format!("{line}\n").as_bytes())?;
        }
        Ok(())
    }
}

struct Shared {
    token: String,
    link: Mutex<Option<Arc<Link>>>,
    next_id: AtomicU64,
    next_gen: AtomicU64,
    events: EventLog,
}

impl Shared {
    fn current(&self) -> Option<Arc<Link>> {
        self.link.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

/// Run the host until an I/O error stops it.
pub async fn serve(cfg: ServeConfig) -> io::Result<()> {
    if cfg.token.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "token is empty",
        ));
    }
    let plugin_l = TcpListener::bind(&cfg.listen).await?;
    let ctl_l = TcpListener::bind(&cfg.ctl).await?;
    if let Some(r) = cfg.ready {
        let _ = r.send(Bound {
            plugin: plugin_l.local_addr()?,
            ctl: ctl_l.local_addr()?,
        });
    }
    let shared = Arc::new(Shared {
        token: cfg.token,
        link: Mutex::new(None),
        next_id: AtomicU64::new(1),
        next_gen: AtomicU64::new(1),
        events: EventLog {
            dir: cfg.log_dir,
            cur: Mutex::new(None),
        },
    });
    loop {
        tokio::select! {
            a = plugin_l.accept() => {
                let (s, peer) = a?;
                eprintln!("plugin connection from {peer}");
                tokio::spawn(handle_plugin(shared.clone(), s));
            }
            c = ctl_l.accept() => {
                let (s, _) = c?;
                tokio::spawn(handle_ctl(shared.clone(), s));
            }
        }
    }
}

/// Read one line of at most `max` bytes. `None` at end of stream.
async fn read_line_max<R: AsyncBufRead + Unpin>(
    r: &mut R,
    buf: &mut Vec<u8>,
    max: usize,
) -> io::Result<Option<String>> {
    buf.clear();
    let n = (&mut *r).take(max as u64).read_until(b'\n', buf).await?;
    if n == 0 {
        return Ok(None);
    }
    if buf.last() == Some(&b'\n') {
        buf.pop();
    } else if n >= max {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "line too long"));
    }
    if buf.last() == Some(&b'\r') {
        buf.pop();
    }
    Ok(Some(String::from_utf8_lossy(buf).into_owned()))
}

fn ct_eq(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |d, (x, y)| d | (x ^ y)) == 0
}

async fn handle_plugin(shared: Arc<Shared>, stream: TcpStream) {
    let _ = stream.set_nodelay(true);
    let (rd, mut wr) = stream.into_split();
    let mut rd = BufReader::new(rd);
    let mut buf = Vec::new();
    let first = match tokio::time::timeout(
        HELLO_TIMEOUT,
        read_line_max(&mut rd, &mut buf, HELLO_MAX),
    )
    .await
    {
        Ok(Ok(Some(l))) => l,
        _ => return,
    };
    let hello = match decode_hello(&first) {
        Ok(h) if ct_eq(&h.token, &shared.token) => h.hello,
        _ => {
            eprintln!("plugin rejected: bad hello or token");
            return;
        }
    };
    eprintln!("plugin connected: pid {} base {:#x}", hello.pid, hello.base);
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let link = Arc::new(Link {
        gen: shared.next_gen.fetch_add(1, Ordering::Relaxed),
        hello,
        tx,
        pending: Mutex::new(HashMap::new()),
    });
    *shared.link.lock().unwrap_or_else(|e| e.into_inner()) = Some(link.clone());
    let writer = tokio::spawn(async move {
        while let Some(l) = rx.recv().await {
            if wr.write_all(l.as_bytes()).await.is_err() || wr.write_all(b"\n").await.is_err() {
                break;
            }
        }
    });
    while let Ok(Some(line)) = read_line_max(&mut rd, &mut buf, LINE_MAX).await {
        let Ok(v) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if v.get("ev").is_some() {
            if let Err(e) = shared.events.write_line(&line) {
                eprintln!("event log error: {e}");
            }
        } else if let Some(id) = v.get("id").and_then(Value::as_u64) {
            let waiter = link
                .pending
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&id);
            if let Some(w) = waiter {
                let _ = w.send(line);
            }
        }
    }
    eprintln!("plugin disconnected");
    {
        let mut cur = shared.link.lock().unwrap_or_else(|e| e.into_inner());
        if cur.as_ref().map(|l| l.gen) == Some(link.gen) {
            *cur = None;
        }
    }
    link.pending
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
    writer.abort();
}

async fn handle_ctl(shared: Arc<Shared>, stream: TcpStream) {
    let (rd, mut wr) = stream.into_split();
    let mut rd = BufReader::new(rd);
    let mut buf = Vec::new();
    while let Ok(Some(line)) = read_line_max(&mut rd, &mut buf, HELLO_MAX * 16).await {
        if line.trim().is_empty() {
            continue;
        }
        let reply = ctl_request(&shared, &line).await;
        if wr.write_all(reply.as_bytes()).await.is_err() || wr.write_all(b"\n").await.is_err() {
            break;
        }
    }
}

fn err_line(msg: &str) -> String {
    json!({ "err": msg }).to_string()
}

/// Handle one control request and return the reply line.
async fn ctl_request(shared: &Shared, line: &str) -> String {
    let Ok(mut v) = serde_json::from_str::<Value>(line) else {
        return err_line("bad json");
    };
    let Some(obj) = v.as_object_mut() else {
        return err_line("request must be an object");
    };
    let link = shared.current();
    if obj.get("cmd").and_then(Value::as_str) == Some("status") {
        return match link {
            Some(l) => json!({ "ok": { "connected": true, "hello": l.hello } }).to_string(),
            None => json!({ "ok": { "connected": false } }).to_string(),
        };
    }
    let Some(link) = link else {
        return err_line("no plugin connected");
    };
    if obj.remove("ida").and_then(|x| x.as_bool()).unwrap_or(false) {
        if let Some(a) = obj.get("addr").and_then(Value::as_u64) {
            let live = ida_to_live(a as u32, link.hello.base);
            obj.insert("addr".into(), json!(live));
        }
    }
    let id = shared.next_id.fetch_add(1, Ordering::Relaxed);
    obj.insert("id".into(), json!(id));
    let text = v.to_string();
    if let Err(e) = decode_request(&text) {
        return err_line(&e.to_string());
    }
    let (otx, orx) = oneshot::channel();
    link.pending
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(id, otx);
    if link.tx.send(text).is_err() {
        link.pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&id);
        return err_line("plugin disconnected");
    }
    match tokio::time::timeout(CMD_TIMEOUT, orx).await {
        Ok(Ok(line)) => line,
        Ok(Err(_)) => err_line("plugin disconnected"),
        Err(_) => {
            link.pending
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&id);
            err_line("timeout")
        }
    }
}

/// Convert days since 1970-01-01 to (year, month, day).
pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m: u32 = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Today's date (UTC) as `YYYY-MM-DD`.
pub fn today() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, m, d) = civil_from_days((secs / 86_400) as i64);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Last `n` event lines (oldest first) that match `filter` (`key`, `value`).
pub fn tail(dir: &Path, n: usize, filter: Option<(&str, &str)>) -> io::Result<Vec<String>> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|f| f.to_str())
                .map(|f| f.starts_with("events-") && f.ends_with(".jsonl"))
                .unwrap_or(false)
        })
        .collect();
    files.sort();
    let mut out: Vec<String> = Vec::new();
    for f in files.iter().rev() {
        let text = fs::read_to_string(f)?;
        let mut part: Vec<String> = text
            .lines()
            .filter(|l| match filter {
                None => true,
                Some((k, want)) => serde_json::from_str::<Value>(l)
                    .ok()
                    .and_then(|v| {
                        v.get(k).map(|x| {
                            x.as_str()
                                .map(str::to_string)
                                .unwrap_or_else(|| x.to_string())
                        })
                    })
                    .map(|got| got == want)
                    .unwrap_or(false),
            })
            .map(str::to_string)
            .collect();
        part.extend(out);
        out = part;
        if out.len() >= n {
            break;
        }
    }
    let skip = out.len().saturating_sub(n);
    Ok(out.split_off(skip))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{encode, Command, Event, HelloMsg, Reply};

    const TOKEN: &str = "test-token";

    struct Env {
        dir: PathBuf,
        bound: Bound,
    }

    impl Drop for Env {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    async fn start(name: &str) -> Env {
        let dir =
            std::env::temp_dir().join(format!("clientdbg-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let (tx, rx) = oneshot::channel();
        let cfg = ServeConfig {
            listen: "127.0.0.1:0".into(),
            ctl: "127.0.0.1:0".into(),
            token: TOKEN.into(),
            log_dir: dir.join("logs"),
            ready: Some(tx),
        };
        tokio::spawn(serve(cfg));
        Env {
            dir,
            bound: rx.await.unwrap(),
        }
    }

    fn hello_line(token: &str) -> String {
        let h = HelloMsg {
            hello: Hello {
                pid: 1,
                exe: "Client.exe".into(),
                base: 0x400000,
                size: 0x1000,
                version: "t".into(),
            },
            token: token.into(),
        };
        format!("{}\n", encode(&h))
    }

    async fn wait_connected(ctl: SocketAddr) {
        for _ in 0..100 {
            let reply = ctl_once(ctl, r#"{"cmd":"status"}"#).await;
            if reply.contains(r#""connected":true"#) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
        panic!("plugin did not connect");
    }

    async fn ctl_once(ctl: SocketAddr, line: &str) -> String {
        let s = TcpStream::connect(ctl).await.unwrap();
        let (rd, mut wr) = s.into_split();
        wr.write_all(format!("{line}\n").as_bytes()).await.unwrap();
        let mut rd = BufReader::new(rd);
        let mut l = String::new();
        tokio::time::timeout(Duration::from_secs(5), rd.read_line(&mut l))
            .await
            .unwrap()
            .unwrap();
        l
    }

    #[tokio::test]
    async fn host_token_reject() {
        let env = start("reject").await;
        let mut s = TcpStream::connect(env.bound.plugin).await.unwrap();
        let ev = encode(&Event::Log {
            t: 1,
            msg: "x".into(),
        });
        let _ = s
            .write_all(format!("{}{ev}\n", hello_line("wrong")).as_bytes())
            .await;
        let mut b = [0u8; 16];
        let r = tokio::time::timeout(Duration::from_secs(5), s.read(&mut b))
            .await
            .unwrap();
        assert!(matches!(r, Ok(0) | Err(_)), "connection must close");
        tokio::time::sleep(Duration::from_millis(100)).await;
        let n = fs::read_dir(env.dir.join("logs"))
            .map(|d| d.count())
            .unwrap_or(0);
        assert_eq!(n, 0, "no event file");
        let st = ctl_once(env.bound.ctl, r#"{"cmd":"status"}"#).await;
        assert!(st.contains(r#""connected":false"#));
    }

    #[tokio::test]
    async fn host_cmd_forward() {
        let env = start("forward").await;
        let s = TcpStream::connect(env.bound.plugin).await.unwrap();
        let (rd, mut wr) = s.into_split();
        wr.write_all(hello_line(TOKEN).as_bytes()).await.unwrap();
        tokio::spawn(async move {
            let mut rd = BufReader::new(rd);
            let mut l = String::new();
            while rd.read_line(&mut l).await.unwrap_or(0) > 0 {
                if let Ok(r) = decode_request(&l) {
                    let rep = match r.cmd {
                        Command::Ping => Reply::ok(r.id, &"pong"),
                        _ => Reply::err(r.id, "unsupported"),
                    };
                    let _ = wr.write_all(format!("{}\n", encode(&rep)).as_bytes()).await;
                }
                l.clear();
            }
        });
        wait_connected(env.bound.ctl).await;
        let line = ctl_once(env.bound.ctl, r#"{"cmd":"ping"}"#).await;
        match crate::proto::decode_reply(&line).unwrap() {
            Reply::Ok { ok, .. } => assert_eq!(ok, json!("pong")),
            other => panic!("unexpected reply {other:?}"),
        }
        let bad = ctl_once(env.bound.ctl, r#"{"cmd":"nope"}"#).await;
        assert!(bad.contains("unknown cmd"));
    }

    #[tokio::test]
    async fn host_events_file() {
        let env = start("events").await;
        let mut s = TcpStream::connect(env.bound.plugin).await.unwrap();
        let mut out = hello_line(TOKEN);
        for i in 0..3 {
            out.push_str(&encode(&Event::Log {
                t: i,
                msg: format!("m{i}"),
            }));
            out.push('\n');
        }
        s.write_all(out.as_bytes()).await.unwrap();
        let file = env
            .dir
            .join("logs")
            .join(format!("events-{}.jsonl", today()));
        let mut lines = 0;
        for _ in 0..100 {
            lines = fs::read_to_string(&file)
                .map(|t| t.lines().count())
                .unwrap_or(0);
            if lines >= 3 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
        assert_eq!(lines, 3);
        let got = tail(&env.dir.join("logs"), 2, Some(("ev", "log"))).unwrap();
        assert_eq!(got.len(), 2);
        assert!(got[1].contains("m2"));
        assert!(tail(&env.dir.join("logs"), 5, Some(("ev", "hbp")))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn civil_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(18_993), (2022, 1, 1));
        assert_eq!(civil_from_days(19_000), (2022, 1, 8));
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
    }
}
