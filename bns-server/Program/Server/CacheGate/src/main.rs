//! cachegate: `cachegate [config-dir]`.
//! Reads cachegate.config.xml from config-dir (default `config`) and
//! cached-list.config.xml from the config-path option. When the cached
//! list is missing or bad, writes a sample file (only if none exists) and
//! stops, like CacheDList::Initialize.
use cachegate::config::{
    parse_cached_list, GateConfig, CACHED_LIST_FILE, CONFIG_FILE, SAMPLE_CACHED_LIST,
};
use cachegate::server::Server;
use std::io::Write;
use std::path::PathBuf;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "config".into()));
    let path = dir.join(CONFIG_FILE);
    let xml = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let config = GateConfig::parse(&xml)?;
    let list_path = PathBuf::from(&config.config_path).join(CACHED_LIST_FILE);
    let rows = match std::fs::read_to_string(&list_path)
        .map_err(|e| e.to_string())
        .and_then(|xml| parse_cached_list(&xml).map_err(|e| e.to_string()))
    {
        Ok(rows) => rows,
        Err(e) => {
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&list_path)
            {
                let _ = f.write_all(SAMPLE_CACHED_LIST.as_bytes());
            }
            return Err(format!("{}: {e}", list_path.display()).into());
        }
    };
    eprintln!(
        "cachegate: wid {}, {} cached, public port {}",
        config.wid,
        rows.len(),
        config.public_port
    );
    let server = Server::new(config.wid, config.max_public_session_count);
    // initNetwork: OpenPort 2 (MakeSessionAll), then OpenPort 1.
    server.reload(&rows);
    let listener = TcpListener::bind(("0.0.0.0", config.public_port)).await?;
    server.serve_public(listener).await?;
    Ok(())
}
