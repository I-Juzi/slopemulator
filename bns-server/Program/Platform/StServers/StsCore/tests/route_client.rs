//! Route bus client tests. Fake routers talk over TCP on 127.0.0.1.
use routesrv::{config::RouteConfig, server::RouteServer};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use sts_core::{
    conn::{header, request, Incoming, Peer},
    connect::ConnectInfo,
    ini::Ini,
    route::{Location, Topology},
    route_client::*,
    Element, StsError,
};
use tokio::{net::TcpListener, sync::mpsc, time::Instant};

type Conn = (Peer, mpsc::UnboundedReceiver<Incoming>);

/// Listener that gives every accepted socket as a `Peer`.
struct Fake {
    addr: SocketAddr,
    conns: mpsc::UnboundedReceiver<Conn>,
}

async fn fake_router() -> Fake {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, conns) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            let _ = tx.send(Peer::spawn(socket));
        }
    });
    Fake { addr, conns }
}

fn loc(port: u16) -> Location {
    Location {
        addr: Some(format!("127.0.0.1:{port}").parse().unwrap()),
        product: 1,
        index: 1,
        deployment: 0,
        epoch: 5,
    }
}

fn cfg(router: SocketAddr, own: Location) -> RouteClientConfig {
    RouteClientConfig {
        router,
        identity: ConnectInfo {
            conn_type: 404,
            location: own,
            process: 77,
            program: 101,
            build: 381,
            ..ConnectInfo::default()
        },
        own,
        route_ini: RouteIni::default(),
        providers: vec![],
        subscriptions: vec![],
        reconnect_delay: Duration::from_millis(3000),
        watchdog: Duration::from_millis(10_000),
    }
}

fn echo() -> Registration {
    Registration {
        name: "Echo".into(),
        build: 1,
        topology: Topology::Basic,
        block_bits: 0,
    }
}

async fn next_event(rx: &mut mpsc::UnboundedReceiver<RouteEvent>) -> RouteEvent {
    tokio::time::timeout(Duration::from_secs(120), rx.recv())
        .await
        .expect("event in time")
        .expect("event channel open")
}

/// Fake router task: Connect gets OK; `handler` decides about each other request.
fn serve_fake(
    mut rx: mpsc::UnboundedReceiver<Incoming>,
    handler: impl Fn(&Incoming) + Send + 'static,
) {
    tokio::spawn(async move {
        while let Some(inc) = rx.recv().await {
            if inc.command() == ("Sts", "Connect") {
                inc.reply(None);
            } else {
                handler(&inc);
            }
        }
    });
}

fn parse(text: &str) -> Vec<SystemEntry> {
    parse_systems(&Ini::parse(text)).unwrap()
}

// ---------------------------------------------------------------- selection

#[test]
fn select_default_router() {
    let e = parse("[Systems]\nAddr = 10.10.66.88, Test\n[Systems.Test]\nRouteSrv = 6605, 1\nDbSrv = 6605, 1, AcctDb\n");
    assert_eq!(
        select_router(&e, "DbSrv", 1).unwrap(),
        "10.10.66.88:6605".parse().unwrap()
    );
}

#[test]
fn select_named_router() {
    let e = parse(
        "[Systems]\nAddr = 10.0.0.1, Test\n[Systems.Test]\nDbSrv = 6605, 1, AcctDb, R2\nRouteSrv = 6605, 1\nRouteSrv = 6606, 1, R2\n",
    );
    let own = e.iter().find(|x| x.server_name == "DbSrv").unwrap();
    assert_eq!(router_name(own), "RouteSrv.R2");
    assert_eq!(select_router(&e, "DbSrv", 1).unwrap().port(), 6606);
}

#[test]
fn select_missing_own() {
    let e = parse("[Systems]\nAddr = 10.0.0.1, Test\n[Systems.Test]\nRouteSrv = 6605, 1\n");
    assert!(select_router(&e, "DbSrv", 1)
        .unwrap_err()
        .contains("in stserver.ini"));
}

#[test]
fn select_bad_port() {
    let ini = Ini::parse("[Systems]\nAddr = 10.0.0.1, Test\n[Systems.Test]\nRouteSrv = 70000, 1\n");
    assert!(parse_systems(&ini)
        .unwrap_err()
        .text
        .contains("invalid port number on line"));
}

#[test]
fn route_ini_defaults() {
    let d = RouteIni::from_ini(&Ini::parse(""));
    assert_eq!(
        (
            d.shutdown_timeout.as_millis(),
            d.registration_retry.as_millis(),
            d.registration_retry_limit,
            d.check_registration_retry.as_millis(),
            d.check_registration_retry_limit
        ),
        (60000, 3000, 1200, 5000, 720)
    );
}

// ---------------------------------------------------------------- connect

#[tokio::test]
async fn connect_body() {
    let mut fake = fake_router().await;
    let (_client, _events) = RouteClient::start(cfg(fake.addr, loc(7001)));
    let (_peer, mut rx) = fake.conns.recv().await.unwrap();
    let first = rx.recv().await.unwrap();
    assert_eq!(first.command(), ("Sts", "Connect"));
    let body = first.body().unwrap();
    let names: Vec<&str> = body.children.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "ConnType",
            "Address",
            "ProductType",
            "AppIndex",
            "Epoch",
            "Program",
            "Build",
            "Process"
        ]
    );
    assert_eq!(body.get_text("ConnType"), Some("404"));
    assert_eq!(body.get_text("Address"), Some("127.0.0.1:7001"));
}

#[tokio::test(start_paused = true)]
async fn watchdog() {
    let mut fake = fake_router().await;
    let started = Instant::now();
    let (_client, _events) = RouteClient::start(cfg(fake.addr, loc(7001)));
    let (_p1, mut rx1) = fake.conns.recv().await.unwrap();
    let _connect = rx1.recv().await.unwrap(); // no reply
    assert!(rx1.recv().await.is_none());
    let closed = started.elapsed();
    assert!(
        closed >= Duration::from_millis(10_000) && closed < Duration::from_millis(10_500),
        "{closed:?}"
    );
    let _second = fake.conns.recv().await.unwrap();
    let again = started.elapsed() - closed;
    assert!(
        again >= Duration::from_millis(3000) && again < Duration::from_millis(3500),
        "{again:?}"
    );
}

// ---------------------------------------------------------------- with RouteSrv

async fn routesrv() -> (Arc<RouteServer>, SocketAddr) {
    let server = Arc::new(RouteServer::new(RouteConfig::default(), 9));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(server.clone().serve(listener, std::future::pending::<()>()));
    (server, addr)
}

#[tokio::test]
async fn register_with_routesrv() {
    let (server, addr) = routesrv().await;
    let own = loc(7001);
    let mut c = cfg(addr, own);
    c.providers = vec![echo()];
    let (client, mut events) = RouteClient::start(c);
    assert!(matches!(
        next_event(&mut events).await,
        RouteEvent::Registered
    ));
    assert!(client.is_registered());
    let snap = server.snapshot("Echo").expect("service Echo");
    assert_eq!(snap.providers.len(), 1);
    assert_eq!(snap.providers[0].location.format(0), own.format(0));
}

#[tokio::test]
async fn subscribe_gets_found() {
    let (_server, addr) = routesrv().await;
    let a_own = loc(7001);
    let mut a = cfg(addr, a_own);
    a.providers = vec![echo()];
    let (_a, mut a_events) = RouteClient::start(a);
    assert!(matches!(
        next_event(&mut a_events).await,
        RouteEvent::Registered
    ));
    let mut b = cfg(addr, loc(7002));
    b.subscriptions = vec!["Echo".into()];
    let (_b, mut b_events) = RouteClient::start(b);
    loop {
        if let RouteEvent::ProviderFound { name, provider } = next_event(&mut b_events).await {
            assert_eq!(name, "Echo");
            assert_eq!(provider, a_own);
            break;
        }
    }
}

// ---------------------------------------------------------------- registration retry

fn pending_error() -> StsError {
    StsError::new(ERR_ROUTE_REGISTER_PENDING, 0, 0)
}

#[tokio::test]
async fn retry_on_pending() {
    let mut fake = fake_router().await;
    let mut c = cfg(fake.addr, loc(7001));
    c.providers = vec![echo()];
    let (_client, mut events) = RouteClient::start(c);
    let (_peer, rx) = fake.conns.recv().await.unwrap();
    let (seen_tx, mut seen) = mpsc::unbounded_channel();
    serve_fake(rx, move |inc| {
        let (_, cmd) = inc.command();
        let _ = seen_tx.send((cmd.to_owned(), inc.frame.body.clone()));
        if cmd == "RegisterProvider" {
            inc.reply_error(&pending_error(), 9);
        } else {
            inc.reply(None);
        }
    });
    let (cmd1, body1) = seen.recv().await.unwrap();
    assert_eq!(cmd1, "RegisterProvider");
    let (cmd2, body2) = seen.recv().await.unwrap();
    assert_eq!(cmd2, "CheckRegistration");
    assert_eq!(body1, body2);
    assert!(matches!(
        next_event(&mut events).await,
        RouteEvent::Registered
    ));
}

#[tokio::test(start_paused = true)]
async fn retry_limit() {
    let mut fake = fake_router().await;
    let mut c = cfg(fake.addr, loc(7001));
    c.providers = vec![echo()];
    c.route_ini.check_registration_retry_limit = 2;
    let (_client, mut events) = RouteClient::start(c);
    let (_peer, rx) = fake.conns.recv().await.unwrap();
    serve_fake(rx, |inc| inc.reply_error(&pending_error(), 9));
    assert!(matches!(
        next_event(&mut events).await,
        RouteEvent::RegistrationFailed
    ));
}

// ---------------------------------------------------------------- router messages

#[tokio::test]
async fn unknown_route_cmd() {
    let mut fake = fake_router().await;
    let (_client, _events) = RouteClient::start(cfg(fake.addr, loc(7001)));
    let (peer, mut rx) = fake.conns.recv().await.unwrap();
    rx.recv().await.unwrap().reply(None); // Sts/Connect
    let reply = peer
        .call(
            request("Route", "GetMembership", None),
            Duration::from_secs(5),
        )
        .await
        .unwrap();
    let err = sts_core::conn::reply_result(&reply).unwrap_err();
    assert_eq!(err.code, 10);
}

// ---------------------------------------------------------------- requests

#[tokio::test]
async fn request_adds_p() {
    let mut fake = fake_router().await;
    let own = loc(7001);
    let (client, mut events) = RouteClient::start(cfg(fake.addr, own));
    let (_peer, rx) = fake.conns.recv().await.unwrap();
    let (seen_tx, mut seen) = mpsc::unbounded_channel();
    serve_fake(rx, move |inc| {
        let _ = seen_tx.send((
            inc.command().0.to_owned(),
            header(&inc.frame, "p").map(str::to_owned),
        ));
        inc.reply(None);
    });
    assert!(matches!(
        next_event(&mut events).await,
        RouteEvent::Registered
    ));
    let mut f = request("Echo", "Ping", Some(&Element::int("V", 1)));
    sts_core::conn::set_header(&mut f, "t", "0$1");
    client.request(f, Duration::from_secs(5)).await.unwrap();
    let (protocol, p) = seen.recv().await.unwrap();
    assert_eq!(protocol, "Echo");
    assert!(p.expect("p header").starts_with("*127.0.0.1:7001 "));
}

#[tokio::test(start_paused = true)]
async fn request_not_connected() {
    // A port with no listener: connect is refused.
    let closed = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    let (client, _events) = RouteClient::start(cfg(closed, loc(7001)));
    let started = Instant::now();
    let err = client
        .request(request("Echo", "Ping", None), Duration::from_secs(5))
        .await
        .unwrap_err();
    assert_eq!(err.code, 42);
    let waited = started.elapsed();
    assert!(
        waited >= Duration::from_millis(3000) && waited < Duration::from_millis(3100),
        "{waited:?}"
    );
}

// ---------------------------------------------------------------- shutdown

#[tokio::test]
async fn shutdown_sends() {
    let mut fake = fake_router().await;
    let (client, mut events) = RouteClient::start(cfg(fake.addr, loc(7001)));
    let (_peer, mut rx) = fake.conns.recv().await.unwrap();
    rx.recv().await.unwrap().reply(None); // Sts/Connect
    assert!(matches!(
        next_event(&mut events).await,
        RouteEvent::Registered
    ));
    let stopper = tokio::spawn({
        let client = client.clone();
        async move { client.shutdown().await }
    });
    let req = rx.recv().await.unwrap();
    assert_eq!(req.command(), ("Route", "ShutdownServer"));
    req.reply(None);
    stopper.await.unwrap();
    let end = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("socket closes");
    assert!(end.is_none());
    assert!(!client.is_registered());
}
