//! Tokio listeners: game port (GameDaemons), manage port, update timer.
use crate::protocol;
use crate::service::{Daemon, GameConn, ManagerConn, Out};
use crate::store::Store;
use std::collections::HashMap;
use std::io;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

type Sink = mpsc::UnboundedSender<Vec<u8>>;

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

pub struct Server<S> {
    pub daemon: Arc<Daemon<S>>,
    clusters: Mutex<HashMap<i16, Sink>>,
    managers: Mutex<HashMap<u64, Sink>>,
    next_id: AtomicU64,
}

impl<S: Store> Server<S> {
    /// Create the server and start the `update-cycle` timer.
    pub fn start(daemon: Arc<Daemon<S>>) -> Arc<Self> {
        let server = Arc::new(Self {
            daemon,
            clusters: Mutex::default(),
            managers: Mutex::default(),
            next_id: AtomicU64::new(1),
        });
        let weak = Arc::downgrade(&server);
        let period = Duration::from_millis(server.daemon.config.update_cycle_ms);
        tokio::spawn(async move {
            let mut timer = tokio::time::interval(period);
            timer.tick().await;
            loop {
                timer.tick().await;
                let Some(server) = weak.upgrade() else { break };
                let out = server.daemon.tick();
                server.route(out, None);
            }
        });
        server
    }

    fn route(&self, out: Vec<Out>, reply: Option<&Sink>) {
        for o in out {
            match o {
                Out::Reply(p) => {
                    if let Some(sink) = reply {
                        let _ = sink.send(p);
                    }
                }
                Out::Cluster(c, p) => {
                    let clusters = self.clusters.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(sink) = clusters.get(&c) {
                        let _ = sink.send(p);
                    }
                }
                Out::Managers(p) => {
                    let managers = self.managers.lock().unwrap_or_else(|e| e.into_inner());
                    for sink in managers.values() {
                        let _ = sink.send(p.clone());
                    }
                }
            }
        }
    }

    pub async fn serve_games(self: Arc<Self>, listener: TcpListener) -> io::Result<()> {
        loop {
            let (stream, addr) = listener.accept().await?;
            let ip = match addr.ip() {
                IpAddr::V4(ip) => ip,
                IpAddr::V6(ip) => ip.to_ipv4_mapped().unwrap_or(Ipv4Addr::UNSPECIFIED),
            };
            tokio::spawn(self.clone().game_connection(stream, GameConn::new(ip)));
        }
    }

    pub async fn serve_managers(self: Arc<Self>, listener: TcpListener) -> io::Result<()> {
        loop {
            let (stream, _) = listener.accept().await?;
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            tokio::spawn(self.clone().manager_connection(stream, id));
        }
    }

    async fn game_connection(self: Arc<Self>, stream: TcpStream, mut conn: GameConn) {
        let (mut reader, writer) = stream.into_split();
        let (out, writer_task) = spawn_writer(writer);
        while let Ok(Some(packet)) = protocol::read(&mut reader).await {
            let had_cluster = conn.cluster.is_some();
            let result = self.daemon.game_packet(&mut conn, &packet, now());
            if !had_cluster {
                if let Some(c) = conn.cluster {
                    self.clusters
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .insert(c, out.clone());
                }
            }
            self.route(result, Some(&out));
        }
        if let Some(c) = conn.cluster {
            self.clusters
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&c);
        }
        let result = self.daemon.game_disconnect(&conn, now());
        self.route(result, None);
        drop(out);
        let _ = writer_task.await;
    }

    async fn manager_connection(self: Arc<Self>, stream: TcpStream, id: u64) {
        let (mut reader, writer) = stream.into_split();
        let (out, writer_task) = spawn_writer(writer);
        let mut conn = ManagerConn::default();
        while let Ok(Some(packet)) = protocol::read(&mut reader).await {
            let was_hello = conn.hello;
            let result = self.daemon.manager_packet(&mut conn, &packet);
            if !was_hello && conn.hello {
                self.managers
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(id, out.clone());
            }
            self.route(result, Some(&out));
        }
        self.managers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&id);
        drop(out);
        let _ = writer_task.await;
    }
}

fn spawn_writer(
    mut writer: tokio::net::tcp::OwnedWriteHalf,
) -> (Sink, tokio::task::JoinHandle<()>) {
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let task = tokio::spawn(async move {
        while let Some(packet) = rx.recv().await {
            if writer.write_all(&packet).await.is_err() {
                break;
            }
        }
        let _ = writer.shutdown().await;
    });
    (tx, task)
}
