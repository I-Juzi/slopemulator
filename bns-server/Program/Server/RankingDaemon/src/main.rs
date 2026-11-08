//! rankingd: `rankingdaemon [config-dir]`.
//! Reads rankingd.config.xml from config-dir (default `config`), the game
//! and InfoGate lists from config-path, and SeasonContentsScheduleData.xml
//! from data-path. No RankingDB: jobs that need the DB fail with reason 4.
use rankingdaemon::config::{
    parse_game_list, parse_infogate_list, parse_season_data, RankingConfig, CONFIG_FILE,
    GAME_LIST_FILE, INFOGATE_LIST_FILE, SEASON_DATA_FILE,
};
use rankingdaemon::server::{Port, Server};
use rankingdaemon::service::Daemon;
use std::path::{Path, PathBuf};
use tokio::net::TcpListener;

fn read(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()).into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "config".into()));
    let config = RankingConfig::parse(&read(&dir.join(CONFIG_FILE))?)?;
    let lists = PathBuf::from(&config.config_path);
    let games = parse_game_list(&read(&lists.join(GAME_LIST_FILE))?)?;
    let infogates = parse_infogate_list(&read(&lists.join(INFOGATE_LIST_FILE))?)?;
    let seasons = parse_season_data(&read(
        &PathBuf::from(&config.data_path).join(SEASON_DATA_FILE),
    )?)?;
    let daemon = Daemon::new(&config, &games, &infogates, seasons);
    eprintln!(
        "rankingd: {} games, {} infogates, {} seasons for service-mode {}",
        games.len(),
        infogates.len(),
        daemon.seasons().len(),
        config.service_mode
    );
    let server = Server::new(daemon);
    let manage = TcpListener::bind(("0.0.0.0", config.manage_listen_port)).await?;
    let infogate = TcpListener::bind(("0.0.0.0", config.infogate_listen_port)).await?;
    let game = TcpListener::bind(("0.0.0.0", config.world_listen_port)).await?;
    tokio::try_join!(
        server.clone().serve(Port::Manage, manage),
        server.clone().serve(Port::InfoGate, infogate),
        server.serve(Port::Game, game),
    )?;
    Ok(())
}
