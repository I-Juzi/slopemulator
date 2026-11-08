//! smartcoupond: `smartcoupondaemon [config-dir]`.
//! Uses the in-memory store and no NP channel; see Docs/Server/SmartCouponDaemon.
use smartcoupondaemon::config::{DaemonConfig, GameEventConfig, GamedList};
use smartcoupondaemon::np::NoNp;
use smartcoupondaemon::server::Server;
use smartcoupondaemon::service::Daemon;
use smartcoupondaemon::store::MemoryStore;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;

fn read(dir: &std::path::Path, name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path = dir.join(name);
    std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()).into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "config".into()));
    let config = match read(&dir, "smartcoupond.config.xml") {
        Ok(xml) => DaemonConfig::parse(&xml)?,
        Err(_) => DaemonConfig::default(),
    };
    let events = GameEventConfig::parse(&read(&dir, "game-event.config.xml")?)?;
    let gamed = GamedList::parse(&read(&dir, "gamed-list.config.xml")?)?;
    let daemon_port = config.daemon_listen_port;
    let manage_port = config.manage_listen_port;
    let daemon = Arc::new(Daemon::new(config, events, gamed, MemoryStore::new(), NoNp));
    let startup = daemon.startup_work().map_err(|_| "start-up load failed")?;
    let server = Server::start(daemon, startup);
    let daemons = TcpListener::bind(("0.0.0.0", daemon_port)).await?;
    let managers = TcpListener::bind(("0.0.0.0", manage_port)).await?;
    eprintln!("smartcoupond: daemon port {daemon_port}, manage port {manage_port}");
    tokio::try_join!(
        server.clone().serve_daemons(daemons),
        server.serve_managers(managers)
    )?;
    Ok(())
}
