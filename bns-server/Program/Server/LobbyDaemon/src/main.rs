use lobbydaemon::{jobs::Settings, np::StubNp, store::MemoryStore, Server};
use std::{
    io,
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
};
use tokio::{net::TcpListener, sync::watch};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut args = std::env::args().skip(1);
    let port: u16 = args
        .next()
        .unwrap_or_else(|| "0".into())
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "usage: lobbyd [local-port]"))?;
    if args.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: lobbyd [local-port]",
        ));
    }
    // Loopback only. Empty stub rejects all tokens until a test fixture is installed.
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)).await?;
    println!("lobbyd local stub listens on {}", listener.local_addr()?);
    #[cfg(feature = "mssql")]
    if std::env::var_os("LOBBYD_MSSQL_URL").is_some() {
        let store = tokio::task::spawn_blocking(lobbydaemon::mssql::MssqlStore::from_env)
            .await
            .map_err(io::Error::other)??;
        return serve(store, listener).await;
    }
    serve(MemoryStore::default(), listener).await
}

async fn serve<S: lobbydaemon::store::Store>(store: S, listener: TcpListener) -> io::Result<()> {
    let server = Arc::new(Server::new(store, StubNp::default(), Settings::default())?);
    let (stop, shutdown) = watch::channel(false);
    let mut task = tokio::spawn(server.serve(listener, shutdown));
    tokio::select! {
        result = &mut task => result.map_err(io::Error::other)?,
        result = tokio::signal::ctrl_c() => {
            result?;
            let _ = stop.send(true);
            task.await.map_err(io::Error::other)?
        }
    }
}
