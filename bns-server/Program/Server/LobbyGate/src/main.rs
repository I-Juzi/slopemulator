use lobbygate::{
    config::Config,
    daemon::TcpLobbyDaemon,
    server::{bind, load_handshake_key, Server},
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::watch;

struct Args {
    config_dir: PathBuf,
    handshake_key: PathBuf,
}
fn args() -> Result<Option<Args>, String> {
    let mut args = std::env::args_os().skip(1);
    let mut config_dir = None;
    let mut handshake_key = None;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--help" | "-h") => return Ok(None),
            Some("--config-dir") if config_dir.is_none() => {
                config_dir = Some(PathBuf::from(
                    args.next().ok_or("--config-dir needs a path")?,
                ))
            }
            Some("--handshake-key") if handshake_key.is_none() => {
                handshake_key = Some(PathBuf::from(
                    args.next().ok_or("--handshake-key needs a path")?,
                ))
            }
            _ => return Err("unknown or duplicate option".into()),
        }
    }
    let handshake_key = handshake_key.ok_or("--handshake-key is required")?;
    let config_dir = match config_dir {
        Some(dir) => dir,
        None => std::env::current_exe()
            .map_err(|e| e.to_string())?
            .parent()
            .ok_or("binary has no parent directory")?
            .to_path_buf(),
    };
    Ok(Some(Args {
        config_dir,
        handshake_key,
    }))
}
fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Some(args) = args().map_err(std::io::Error::other)? else {
        println!("Usage: lobbygate [--config-dir DIR] --handshake-key FILE");
        return Ok(());
    };
    let config = Config::load(&args.config_dir)?;
    let key = load_handshake_key(&args.handshake_key)?;
    std::fs::create_dir_all(&config.gate.log_path)?;
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(config.gate.log_path.join("lobbygate.log"))?;
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(Mutex::new(log))
        .try_init()?;
    tracing::info!(
        dump_type = config.gate.dump_type,
        "Windows dump flags are ignored"
    );
    // Internal-only hash and preallocation settings need no Rust implementation.
    // cert is accepted but never opened. There is no licence check.
    // TODO(spec 01 §4): delayed release, alerts, cooldown, and runtime GUID reload.
    // Idle scans and the close-job cadence run in Server.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(config.gate.worker_count())
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let daemon = TcpLobbyDaemon::start(&config.gate, shutdown_rx.clone()).await?;
        let server =
            Arc::new(Server::new(&config.gate, key, daemon)?.with_contents(&config.contents)?);
        let listener = bind(&config.gate).await?;
        tracing::info!(address=%listener.local_addr()?,"client listener started");
        let mut task = tokio::spawn(server.serve(listener, shutdown_rx));
        tokio::select! {
            result=&mut task => result.map_err(std::io::Error::other)?,
            signal=tokio::signal::ctrl_c() => {
                signal?;
                let _=shutdown_tx.send(true);
                task.await.map_err(std::io::Error::other)?
            }
        }
    })?;
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("lobbygate: {error}");
        std::process::exit(1);
    }
}
