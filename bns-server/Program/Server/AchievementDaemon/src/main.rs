//! achievementd: `achievementdaemon [config-dir]`.
//! Reads achievementd.config.xml, achievementd-iplist.xml and the two rule
//! tables from data-path. Uses the in-memory store.
use achievementdaemon::config::{DaemonConfig, IpList};
use achievementdaemon::rules::Rules;
use achievementdaemon::server::Server;
use achievementdaemon::service::Daemon;
use achievementdaemon::store::MemoryStore;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::TcpListener;

fn read(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()).into())
}

/// First file in `dir` whose lower-case name starts with `prefix` and ends in `.xml`.
fn table(dir: &Path, prefix: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut names: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.to_ascii_lowercase())
                .is_some_and(|n| n.starts_with(prefix) && n.ends_with(".xml"))
        })
        .collect();
    names.sort();
    names
        .into_iter()
        .next()
        .ok_or_else(|| format!("{}: no {prefix}*.xml", dir.display()).into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "config".into()));
    let config = match std::fs::read_to_string(dir.join("achievementd.config.xml")) {
        Ok(xml) => DaemonConfig::parse(&xml)?,
        Err(_) => DaemonConfig::default(),
    };
    let iplist = IpList::parse(&String::from_utf8(read(
        &dir.join("achievementd-iplist.xml"),
    )?)?)?;
    let data = PathBuf::from(&config.data_path);
    let rules = Rules::from_files(
        &read(&table(&data, "achievementregisterdata")?)?,
        &read(&table(&data, "achievementdata")?)?,
    )?;
    eprintln!(
        "achievementd: {} registers, {} achievement records",
        rules.registers.len(),
        rules.achievements.len()
    );
    let (game_port, manage_port) = (config.internal_port, config.manage_port);
    let daemon = Arc::new(Daemon::new(config, iplist, rules, MemoryStore::new()));
    let server = Server::start(daemon);
    let games = TcpListener::bind(("0.0.0.0", game_port)).await?;
    let managers = TcpListener::bind(("0.0.0.0", manage_port)).await?;
    eprintln!("achievementd: game port {game_port}, manage port {manage_port}");
    tokio::try_join!(
        server.clone().serve_games(games),
        server.serve_managers(managers)
    )?;
    Ok(())
}
