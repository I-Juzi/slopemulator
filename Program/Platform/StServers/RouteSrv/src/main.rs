//! `RouteSrv --config Route.ini [--listen 127.0.0.1:6605] [--server-id N]`
use routesrv::{config::RouteConfig, server::RouteServer};
use std::sync::Arc;
use sts_core::ini::Ini;

fn usage() -> ! {
    eprintln!("usage: RouteSrv --config Route.ini [--listen addr:port] [--server-id N]");
    std::process::exit(2)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut config_path = None;
    let mut listen = "127.0.0.1:6605".to_owned();
    let mut server_id = 0u32;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().unwrap_or_else(|| usage());
        match arg.as_str() {
            "--config" => config_path = Some(value()),
            "--listen" => listen = value(),
            "--server-id" => server_id = value().parse().unwrap_or_else(|_| usage()),
            _ => usage(),
        }
    }
    let path = config_path.unwrap_or_else(|| usage());
    let config = RouteConfig::from_ini(&Ini::load(path)?)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let server = Arc::new(RouteServer::new(config, server_id));
    let listener = tokio::net::TcpListener::bind(&listen).await?;
    eprintln!("RouteSrv: one router on {listen}");
    server
        .serve(listener, async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
}
