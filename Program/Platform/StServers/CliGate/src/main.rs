//! `CliGate --config StServer.ini --route-ini Route.ini [--index N] [--listen addr:port]
//! [--server-id N] [--center N]`

use cligate::{
    gate::{CliGate, GateConfig},
    session::idle_timeout_ms,
};
use std::{
    io,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use sts_core::{
    connect::ConnectInfo,
    ini::Ini,
    route::{Location, Topology},
    route_client::{
        parse_systems, select_router, Registration, RouteClient, RouteClientConfig, RouteIni,
    },
};
use tokio::net::TcpListener;

/// ❓ ConnType, program id and build of a server socket are not traced.
const CONN_TYPE_SERVER: u32 = 404;
const PROGRAM_ID: u32 = 101;
const BUILD: u32 = 381;
const AUTH_TIMEOUT: Duration = Duration::from_millis(25_000);

fn usage() -> ! {
    eprintln!(
        "usage: CliGate --config StServer.ini --route-ini Route.ini [--index N] [--listen addr:port] [--server-id N] [--center N]"
    );
    std::process::exit(2)
}

fn invalid(text: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, text.into())
}

/// ❓ The section of `IdleTimeout` is not traced. Any section is searched.
fn idle_seconds(ini: &Ini) -> Option<i64> {
    ini.sections
        .iter()
        .flat_map(|s| s.entries.iter())
        .find(|(k, _)| k.eq_ignore_ascii_case("IdleTimeout"))
        .and_then(|(_, v)| v.trim().parse().ok())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let (mut config, mut route_ini_path, mut listen) = (None, None, None);
    let (mut index, mut server_id, mut center) = (1u32, 0u32, 0u32);
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().unwrap_or_else(|| usage());
        match arg.as_str() {
            "--config" => config = Some(value()),
            "--route-ini" => route_ini_path = Some(value()),
            "--listen" => listen = Some(value()),
            "--index" => index = value().parse().unwrap_or_else(|_| usage()),
            "--server-id" => server_id = value().parse().unwrap_or_else(|_| usage()),
            "--center" => center = value().parse().unwrap_or_else(|_| usage()),
            _ => usage(),
        }
    }
    let (Some(config), Some(route_ini_path)) = (config, route_ini_path) else {
        usage()
    };
    let ini = Ini::load(config)?;
    let entries = parse_systems(&ini).map_err(|e| invalid(e.text))?;
    let router = select_router(&entries, "CliGate", index).map_err(invalid)?;
    let own_entry = entries
        .iter()
        .find(|e| e.server_name == "CliGate" && e.app_index == index)
        .ok_or_else(|| invalid("CliGate is not in stserver.ini"))?;
    let route_ini = RouteIni::from_ini(&Ini::load(route_ini_path)?);

    let listen = listen.unwrap_or_else(|| own_entry.addr.to_string());
    let listener = TcpListener::bind(&listen).await?;
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |d| d.as_secs() as u32);
    let own = Location {
        addr: Some(listener.local_addr()?),
        product: 1,
        index,
        deployment: 0,
        epoch,
    };
    let route_cfg = RouteClientConfig {
        router,
        identity: ConnectInfo {
            conn_type: CONN_TYPE_SERVER,
            location: own,
            process: std::process::id(),
            program: PROGRAM_ID,
            build: BUILD,
            ..ConnectInfo::default()
        },
        own,
        route_ini,
        providers: vec![Registration {
            name: "Cli".into(),
            build: 1,
            topology: Topology::Basic,
            block_bits: 0,
        }],
        subscriptions: vec!["Auth".into(), "Access".into()],
        reconnect_delay: Duration::from_millis(3000),
        watchdog: Duration::from_millis(10_000),
    };
    let idle_ms = idle_timeout_ms(idle_seconds(&ini));
    eprintln!(
        "The idle timeout of ClientManager({}) : {} seconds",
        own_entry.config_name,
        idle_ms / 1000
    );
    let (route, events) = RouteClient::start(route_cfg);
    let gate = CliGate::new(
        GateConfig {
            server_id,
            center,
            idle_timeout: Duration::from_millis(idle_ms),
            auth_timeout: AUTH_TIMEOUT,
        },
        route,
    );
    eprintln!("CliGate: clients on {listen}, router {router}");
    tokio::spawn(gate.clone().handle_bus(events));
    gate.serve(listener).await
}
