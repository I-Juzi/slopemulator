//! `DbSrv --config Db.ini --db AcctDb [--db GameDb …] [--listen 127.0.0.1:10900] [--server-id N]`

use dbsrv::{config, exec::Executor, server::DbServer};
use std::sync::Arc;
use sts_core::{
    ini::Ini,
    secrets::{names, SecretStore},
};

fn usage() -> ! {
    eprintln!("usage: DbSrv --config Db.ini --db <Logical> [--db <Logical>…] [--listen addr:port] [--server-id N]");
    std::process::exit(2)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut config_path = None;
    let mut dbs = Vec::new();
    let mut listen = "127.0.0.1:10900".to_owned();
    let mut server_id = 0u32;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut val = || args.next().unwrap_or_else(|| usage());
        match a.as_str() {
            "--config" => config_path = Some(val()),
            "--db" => dbs.push(val()),
            "--listen" => listen = val(),
            "--server-id" => server_id = val().parse().unwrap_or_else(|_| usage()),
            _ => usage(),
        }
    }
    let (Some(path), false) = (config_path, dbs.is_empty()) else {
        usage()
    };
    let ini = Ini::load(&path)?;
    let secrets = SecretStore::from_env();
    let mut execs = Vec::new();
    for db in &dbs {
        let cfg = config::SqlConfig::from_ini(&ini, db).and_then(|mut c| {
            c.password = secrets
                .resolve(&names::db_password(db), &c.password)
                .map_err(|e| e.to_string())?;
            Ok(c)
        });
        match cfg {
            Ok(c) => execs.push(Executor::new(c)),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
    let workers = config::thread_tasks(&ini);
    let server = Arc::new(DbServer::new(execs, server_id, workers));
    let listener = tokio::net::TcpListener::bind(&listen).await?;
    eprintln!("DbSrv: {} on {listen}, {workers} workers", dbs.join(", "));
    server
        .serve(listener, async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
}
