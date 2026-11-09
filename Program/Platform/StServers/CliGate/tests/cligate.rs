//! CliGate tests. An in-process RouteServer, a fake AuthSrv (provider `Auth` and
//! `Access` on the bus) and a raw TCP client with its own RC4 state.
use base64::{engine::general_purpose::STANDARD, Engine as _};
use cligate::gate::{CliGate, GateConfig};
use routesrv::{config::RouteConfig, server::RouteServer};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use sts::{rc4::Rc4, Codec, Frame, StartLine};
use sts_core::{
    conn::{header, reply_error, reply_ok, reply_result, request, set_header, Incoming},
    connect::ConnectInfo,
    error::StsError,
    route::{Location, Topology},
    route_client::{Registration, RouteClient, RouteClientConfig, RouteEvent, RouteIni},
    xml::Element,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

/// Test session key bytes. Not a real key.
const KEY: [u8; 16] = [7, 1, 9, 3, 5, 2, 8, 4, 6, 0, 11, 13, 17, 19, 23, 29];
const UID: &str = "0A0B0C0D-0102-0304-0506-0708090A0B0C";

// ------------------------------------------------------------------ harness

struct Seen {
    protocol: String,
    command: String,
    t: Option<String>,
    body: Vec<u8>,
}

struct Harness {
    gate: Arc<CliGate>,
    gate_addr: SocketAddr,
    requester: RouteClient,
    seen: Arc<Mutex<Vec<Seen>>>,
    _fake: RouteClient,
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

fn route_cfg(
    router: SocketAddr,
    own: Location,
    providers: Vec<Registration>,
    subscriptions: Vec<String>,
) -> RouteClientConfig {
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
        providers,
        subscriptions,
        reconnect_delay: Duration::from_millis(3000),
        watchdog: Duration::from_millis(10_000),
    }
}

fn provider(name: &str, topology: Topology) -> Registration {
    Registration {
        name: name.into(),
        build: 1,
        topology,
        block_bits: 0,
    }
}

async fn wait_registered(c: &RouteClient) {
    for _ in 0..500 {
        if c.is_registered() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("route client not registered");
}

async fn harness(
    idle: Duration,
    auth_timeout: Duration,
    handler: impl Fn(&Incoming) + Send + 'static,
) -> Harness {
    let server = Arc::new(RouteServer::new(RouteConfig::default(), 9));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let router = listener.local_addr().unwrap();
    tokio::spawn(server.serve(listener, std::future::pending::<()>()));

    // Fake AuthSrv: provider `Auth` and `Access`.
    let (fake, mut fake_events) = RouteClient::start(route_cfg(
        router,
        loc(7001),
        vec![
            provider("Auth", Topology::Grid),
            provider("Access", Topology::Grid),
        ],
        vec![],
    ));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let seen2 = seen.clone();
    tokio::spawn(async move {
        while let Some(event) = fake_events.recv().await {
            if let RouteEvent::Request(inc) = event {
                let (protocol, command) = {
                    let (p, c) = inc.command();
                    (p.to_owned(), c.to_owned())
                };
                seen2.lock().unwrap().push(Seen {
                    protocol,
                    command,
                    t: header(&inc.frame, "t").map(str::to_owned),
                    body: inc.frame.body.clone(),
                });
                handler(&inc);
            }
        }
    });
    wait_registered(&fake).await;

    // The gate, as main.rs builds it.
    let (route, events) = RouteClient::start(route_cfg(
        router,
        loc(7002),
        vec![provider("Cli", Topology::Basic)],
        vec!["Auth".into(), "Access".into()],
    ));
    let gate = CliGate::new(
        GateConfig {
            server_id: 5,
            center: 0,
            idle_timeout: idle,
            auth_timeout,
        },
        route.clone(),
    );
    tokio::spawn(gate.clone().handle_bus(events));
    let gate_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let gate_addr = gate_listener.local_addr().unwrap();
    tokio::spawn(gate.clone().serve(gate_listener));
    wait_registered(&route).await;

    let (requester, _events) = RouteClient::start(route_cfg(router, loc(7003), vec![], vec![]));
    Harness {
        gate,
        gate_addr,
        requester,
        seen,
        _fake: fake,
    }
}

async fn default_harness(handler: impl Fn(&Incoming) + Send + 'static) -> Harness {
    harness(Duration::from_secs(60), Duration::from_secs(5), handler).await
}

/// Reply of a provider. The router needs `t` = the `p` header of the request.
fn bus_reply(inc: &Incoming, result: Result<Option<Element>, StsError>) {
    let mut frame = match &result {
        Ok(body) => reply_ok(inc.id(), body.as_ref()),
        Err(e) => reply_error(inc.id(), e, 9),
    };
    set_header(&mut frame, "t", header(&inc.frame, "p").expect("origin"));
    inc.peer.send(frame);
}

/// Fake AuthSrv that lets a login pass.
fn happy(inc: &Incoming) {
    let reply = match inc.command() {
        ("Auth", "LoginStart") => Element::new("Reply")
            .child(Element::text("Session", "S1"))
            .child(Element::text("UserId", UID))
            .child(Element::int("UserCenter", 1))
            .child(Element::text("KeyData", "server-key-data")),
        ("Auth", "KeyData") => Element::new("Reply")
            .child(Element::new("Complete"))
            .child(Element::text("SessionKey", STANDARD.encode(KEY))),
        ("Auth", "LoginFinish") => Element::new("Reply").child(Element::new("Licenses")),
        ("Access", "GetAppUserRights") => {
            Element::new("Reply").child(Element::text("AccessMask", "1"))
        }
        _ => return bus_reply(inc, Err(StsError::new(10, 2, 1))),
    };
    bus_reply(inc, Ok(Some(reply)));
}

// ------------------------------------------------------------------ client

struct Client {
    stream: TcpStream,
    codec: Codec,
    buf: Vec<u8>,
    enc: Option<Rc4>,
    dec: Option<Rc4>,
    next_id: u32,
}

impl Client {
    async fn connect(addr: SocketAddr) -> Self {
        Self {
            stream: TcpStream::connect(addr).await.unwrap(),
            codec: Codec::default(),
            buf: Vec::new(),
            enc: None,
            dec: None,
            next_id: 1,
        }
    }

    /// RC4 on for both directions. Call it only when no bytes wait.
    fn enable(&mut self, key: &[u8]) {
        assert!(self.buf.is_empty());
        self.enc = Some(Rc4::new(key).unwrap());
        self.dec = Some(Rc4::new(key).unwrap());
    }

    async fn send(&mut self, protocol: &str, command: &str, body: Option<Element>) {
        let mut frame = request(protocol, command, body.as_ref());
        set_header(&mut frame, "s", self.next_id.to_string());
        self.next_id += 1;
        let mut bytes = Codec::default().encode(&frame).unwrap();
        if let Some(enc) = &mut self.enc {
            enc.apply(&mut bytes);
        }
        self.stream.write_all(&bytes).await.unwrap();
    }

    /// `None` = the socket is closed.
    async fn recv(&mut self) -> Option<Frame> {
        loop {
            if let Some((frame, used)) = self.codec.decode(&self.buf).unwrap() {
                self.buf.drain(..used);
                return Some(frame);
            }
            let mut chunk = [0u8; 4096];
            let n = tokio::time::timeout(Duration::from_secs(10), self.stream.read(&mut chunk))
                .await
                .expect("reply in time")
                .unwrap_or(0);
            if n == 0 {
                return None;
            }
            if let Some(dec) = &mut self.dec {
                dec.apply(&mut chunk[..n]);
            }
            self.buf.extend_from_slice(&chunk[..n]);
        }
    }

    async fn call(&mut self, protocol: &str, command: &str, body: Option<Element>) -> Frame {
        self.send(protocol, command, body).await;
        self.recv().await.expect("reply, not EOF")
    }
}

fn status(f: &Frame) -> u32 {
    match &f.start {
        StartLine::Response { status, .. } => *status,
        _ => panic!("not a reply"),
    }
}

/// `(code, line)` of an error reply.
fn error_of(f: &Frame) -> (u32, u32) {
    assert_eq!(status(f), 400);
    let e = StsError::from_element(&Element::parse(&f.body).unwrap());
    (e.code, e.line)
}

fn login_start_body() -> Element {
    Element::new("Request").child(Element::text("LoginName", "tester"))
}

async fn login(h: &Harness) -> Client {
    let mut c = Client::connect(h.gate_addr).await;
    let r = c.call("Auth", "LoginStart", Some(login_start_body())).await;
    assert_eq!(status(&r), 200);
    let body = Element::new("Request").child(Element::text("KeyData", "client-key-data"));
    let r = c.call("Auth", "KeyData", Some(body)).await;
    assert_eq!(status(&r), 200);
    // The session key never goes to the client.
    assert!(!String::from_utf8_lossy(&r.body).contains("SessionKey"));
    c.enable(&KEY);
    let r = c
        .call("Auth", "LoginFinish", Some(Element::new("Request")))
        .await;
    assert_eq!(status(&r), 200);
    let body = Element::parse(&r.body).unwrap();
    assert_eq!(body.get_text("UserId"), Some(UID));
    assert_eq!(body.get_text("UserCenter"), Some("1"));
    assert!(body.has("Licenses"));
    c
}

// -------------------------------------------------------------------- tests

#[tokio::test]
async fn logout_not_logged_in() {
    let h = default_harness(happy).await;
    let mut c = Client::connect(h.gate_addr).await;
    for _ in 0..2 {
        let r = c.call("Auth", "LogoutMyClient", None).await;
        assert_eq!(error_of(&r), (3007, 1736));
    }
}

#[tokio::test]
async fn login_flow() {
    let h = default_harness(happy).await;
    let _c = login(&h).await;
    let seen = h.seen.lock().unwrap();
    let find = |p: &str, c: &str| seen.iter().find(|s| s.protocol == p && s.command == c);
    let expected = format!("0${UID}");
    assert_eq!(find("Auth", "LoginStart").unwrap().t, None);
    assert_eq!(
        find("Auth", "KeyData").unwrap().t.as_deref(),
        Some(expected.as_str())
    );
    let finish = find("Auth", "LoginFinish").unwrap();
    assert_eq!(finish.t.as_deref(), Some(expected.as_str()));
    let body = Element::parse(&finish.body).unwrap();
    assert_eq!(body.get_text("Session"), Some("S1"));
    assert_eq!(
        find("Access", "GetAppUserRights").unwrap().t.as_deref(),
        Some(expected.as_str())
    );
    assert_eq!(h.gate.client_ids().len(), 1);
}

#[tokio::test]
async fn login_start_three_fails() {
    let h = default_harness(|inc: &Incoming| {
        bus_reply(inc, Err(StsError::new(3, 5, 77)));
    })
    .await;
    let mut c = Client::connect(h.gate_addr).await;
    for _ in 0..3 {
        let r = c.call("Auth", "LoginStart", Some(login_start_body())).await;
        assert_eq!(error_of(&r), (3, 77));
    }
    assert!(c.recv().await.is_none());
}

#[tokio::test]
async fn login_start_bad_userref() {
    let h = default_harness(|inc: &Incoming| {
        bus_reply(
            inc,
            Ok(Some(
                Element::new("Reply")
                    .child(Element::text("Session", "S1"))
                    .child(Element::text("KeyData", "x")),
            )),
        );
    })
    .await;
    let mut c = Client::connect(h.gate_addr).await;
    let r = c.call("Auth", "LoginStart", Some(login_start_body())).await;
    assert_eq!(error_of(&r), (10, 1302));
}

#[tokio::test]
async fn key_data_before_login_start() {
    let h = default_harness(happy).await;
    let mut c = Client::connect(h.gate_addr).await;
    let body = Element::new("Request").child(Element::text("KeyData", "x"));
    let r = c.call("Auth", "KeyData", Some(body)).await;
    assert_eq!(error_of(&r).0, 49);
    let r = c
        .call("Auth", "LoginFinish", Some(Element::new("Request")))
        .await;
    assert_eq!(error_of(&r), (49, 1681));
}

#[tokio::test]
async fn logout_after_login() {
    let h = default_harness(happy).await;
    let mut c = login(&h).await;
    let r = c.call("Auth", "LogoutMyClient", None).await;
    assert_eq!(status(&r), 200);
    assert!(c.recv().await.is_none());
    let lines = h.gate.log_lines();
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with(&format!("LOGOUT, {UID}"))),
        "{lines:?}"
    );
}

fn cli_logout(client_id: u32, force: bool) -> sts::Frame {
    let mut body = Element::new("Request")
        .child(Element::int("ClientId", i64::from(client_id)))
        .child(Element::text("UserId", UID));
    if force {
        body.push(Element::new("Force"));
    }
    request("Cli", "Logout", Some(&body))
}

#[tokio::test]
async fn cli_logout_force() {
    let h = default_harness(happy).await;
    let mut c = login(&h).await;
    let id = h.gate.client_ids()[0];
    let reply = h
        .requester
        .request(cli_logout(id, true), Duration::from_secs(5))
        .await
        .unwrap();
    assert!(reply_result(&reply).is_ok());
    assert!(c.recv().await.is_none());
}

#[tokio::test]
async fn cli_logout_without_force_acts_the_same() {
    let h = default_harness(happy).await;
    let mut c = login(&h).await;
    let id = h.gate.client_ids()[0];
    let reply = h
        .requester
        .request(cli_logout(id, false), Duration::from_secs(5))
        .await
        .unwrap();
    assert!(reply_result(&reply).is_ok());
    assert!(c.recv().await.is_none());
}

#[tokio::test]
async fn cli_logout_unknown() {
    let h = default_harness(happy).await;
    let reply = h
        .requester
        .request(cli_logout(999, true), Duration::from_secs(5))
        .await
        .unwrap();
    assert_eq!(reply_result(&reply).unwrap_err().code, 3018);
}

#[tokio::test]
async fn idle_close() {
    let h = harness(Duration::from_millis(200), Duration::from_secs(5), happy).await;
    let mut c = Client::connect(h.gate_addr).await;
    let r = tokio::time::timeout(Duration::from_secs(1), c.recv())
        .await
        .expect("closed in under 1 s");
    assert!(r.is_none());
    assert!(h
        .gate
        .log_lines()
        .iter()
        .any(|l| l.contains("Disconnect - idle timeout")));
}

#[tokio::test]
async fn auth_timeout() {
    let h = harness(
        Duration::from_secs(60),
        Duration::from_millis(300),
        |_inc: &Incoming| {},
    )
    .await;
    let mut c = Client::connect(h.gate_addr).await;
    let r = c.call("Auth", "LoginStart", Some(login_start_body())).await;
    assert_eq!(error_of(&r).0, 42);
    // The socket stays open.
    let r = c.call("Auth", "LogoutMyClient", None).await;
    assert_eq!(error_of(&r).0, 3007);
}

#[tokio::test]
async fn not_logged_in_other_protocol_is_refused() {
    let h = default_harness(happy).await;
    let mut c = Client::connect(h.gate_addr).await;
    let r = c.call("Profile", "Get", None).await;
    assert_eq!(error_of(&r), (49, 2603));
}

#[tokio::test]
async fn login_token_start_without_certificate() {
    let h = default_harness(happy).await;
    let mut c = Client::connect(h.gate_addr).await;
    let body =
        Element::new("Request").child(Element::text("ClientRand", STANDARD.encode([1u8; 32])));
    let r = c.call("Auth", "LoginTokenStart", Some(body)).await;
    assert_eq!(error_of(&r), (4, 1466));
    let body =
        Element::new("Request").child(Element::text("ClientRand", STANDARD.encode([1u8; 8])));
    let r = c.call("Auth", "LoginTokenStart", Some(body)).await;
    assert_eq!(error_of(&r), (10, 1456));
}
