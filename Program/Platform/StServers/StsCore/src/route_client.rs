//! Route bus client, phase 1: one router, no TLS, no forward scope.
//! Spec: Docs/Platform/StServers/14-route-client.md.
//!
//! A service uses `RouteClient` to connect to one RouteSrv, register its
//! protocols, get provider events and send requests through the bus.

use crate::{
    conn::{header, reply_result, request, set_header, Incoming, Peer, DEFAULT_TIMEOUT},
    connect::ConnectInfo,
    error::{ERR_BAD_CONFIG, ERR_BAD_PARAM, ERR_TIMEOUT},
    ini::Ini,
    route::{Location, RouteCmd, RouteMessage, Target, Topology},
    xml::parse_u32,
    Element, StsError,
};
use std::{
    collections::VecDeque,
    fmt,
    net::{IpAddr, Shutdown, SocketAddr},
    sync::{Arc, Mutex},
    time::Duration,
};
use sts::{Frame, StartLine};
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time::{sleep_until, Instant},
};

/// Connection type of a service that connects to a router (§3.1).
const CONN_TYPE_ROUTE: u32 = 404;
/// ErrRouteRegisterPending (§4.2).
pub const ERR_ROUTE_REGISTER_PENDING: u32 = 3053;
/// Error module of the route client (§5).
const MODULE_ROUTE_CLIENT: u32 = 6000;
/// Error module of a bus timeout (§6.2).
const MODULE_TIMEOUT: u32 = 9001;
/// Largest request queue while not connected (§6.1).
const MAX_QUEUE_BYTES: usize = 10_485_760;
/// A queued request fails with error 42 after this time (§6.1).
const NOT_CONNECTED_TIMEOUT: Duration = Duration::from_millis(3000);
/// Wait for RouterConnect after the Sts/Connect OK reply (§3.3 note).
const ROUTER_CONNECT_WAIT: Duration = Duration::from_millis(200);
/// After this time the check backlog slows down and retries count (§4.4).
const REGISTER_TIMER: Duration = Duration::from_millis(10_000);
/// First period of the check backlog (§4.4).
const FIRST_CHECK_PERIOD: Duration = Duration::from_millis(500);

// ---------------------------------------------------------------- §2 selection

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemEntry {
    pub server_name: String,
    pub addr: SocketAddr,
    pub remote_addr: Option<SocketAddr>,
    pub app_index: u32,
    pub config_name: String,
    pub route_config_name: String,
    pub use_tls: bool,
    pub use_remote_ip: bool,
    pub system_type: String,
}

/// Reads StServer.ini (§2.1). The ini reader keeps no line numbers, so
/// "line" in an error text is the position of the entry in its section.
/// The list is sorted and has no duplicates.
pub fn parse_systems(ini: &Ini) -> Result<Vec<SystemEntry>, StsError> {
    let bad = |text: String| {
        eprintln!("{text}");
        StsError::new(ERR_BAD_CONFIG, 0, 0).with_text(text)
    };
    let mut out = Vec::new();
    let Some(systems) = ini.section("Systems") else {
        return Ok(out);
    };
    let addr_lines = systems
        .entries
        .iter()
        .enumerate()
        .filter(|(_, (k, _))| k.eq_ignore_ascii_case("Addr"));
    for (i, (_, value)) in addr_lines {
        let line = i + 1;
        let cols: Vec<&str> = value.split(',').map(str::trim).collect();
        let ip: IpAddr = cols[0]
            .parse()
            .map_err(|_| bad(format!("StsServer: invalid address on line {line}")))?;
        let system_type = match cols.get(1) {
            Some(t) if !t.is_empty() => *t,
            _ => {
                return Err(bad(format!(
                    "StsServer: missing system type on line {line}"
                )))
            }
        };
        let remote_ip: Option<IpAddr> = match cols.get(2) {
            Some(r) if !r.is_empty() => Some(
                r.parse()
                    .map_err(|_| bad(format!("StsServer: invalid address on line {line}")))?,
            ),
            _ => None,
        };
        let Some(section) = ini.section(&format!("Systems.{system_type}")) else {
            return Err(bad(format!(
                "StsServer: undefined system type, {system_type}"
            )));
        };
        for (j, (name, v)) in section.entries.iter().enumerate() {
            let line = j + 1;
            let col: Vec<&str> = v.split(',').map(str::trim).collect();
            let at = |n: usize| col.get(n).copied().unwrap_or("");
            let port = at(0)
                .parse::<u16>()
                .ok()
                .filter(|p| *p != 0)
                .ok_or_else(|| bad(format!("StsServer: invalid port number on line {line}")))?;
            if at(2).chars().count() > 63 {
                return Err(bad(format!(
                    "StsServer: config name on line {line} was more than 63 characters"
                )));
            }
            if at(3).chars().count() > 31 {
                return Err(bad(format!(
                    "StsServer: route config name on line {line} was more than 31 characters"
                )));
            }
            out.push(SystemEntry {
                server_name: name.clone(),
                addr: SocketAddr::new(ip, port),
                remote_addr: remote_ip.map(|r| SocketAddr::new(r, port)),
                app_index: parse_u32(at(1)).unwrap_or(0),
                config_name: at(2).to_owned(),
                route_config_name: at(3).to_owned(),
                use_tls: parse_u32(at(4)).unwrap_or(0) != 0,
                use_remote_ip: parse_u32(at(5)).unwrap_or(0) != 0,
                system_type: system_type.to_owned(),
            });
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

/// `RouteSrv` or `RouteSrv.<RouteConfigName>` (§2.3).
pub fn router_name(own: &SystemEntry) -> String {
    if own.route_config_name.is_empty() {
        "RouteSrv".to_owned()
    } else {
        format!("RouteSrv.{}", own.route_config_name)
    }
}

/// Same /24 network for IPv4. IPv6 compares the first 8 bytes ❓ (mask not traced).
fn same_net(a: IpAddr, b: IpAddr) -> bool {
    match (a, b) {
        (IpAddr::V4(a), IpAddr::V4(b)) => a.octets()[..3] == b.octets()[..3],
        (IpAddr::V6(a), IpAddr::V6(b)) => a.octets()[..8] == b.octets()[..8],
        _ => false,
    }
}

/// Finds the own entry and the router address (§2.2, §2.3). Phase 1 uses
/// the first matching address, local entries first.
pub fn select_router(
    entries: &[SystemEntry],
    program: &str,
    app_index: u32,
) -> Result<SocketAddr, String> {
    let Some(own) = entries
        .iter()
        .find(|e| e.server_name == program && e.app_index == app_index)
    else {
        let text = format!("Remove {program}({app_index}) in stserver.ini");
        eprintln!("{text}");
        return Err(text);
    };
    let name = router_name(own);
    let (server, config) = match name.split_once('.') {
        Some((x, y)) => (x, Some(y)),
        None => (name.as_str(), None),
    };
    let found: Vec<&SystemEntry> = entries
        .iter()
        .filter(|e| e.server_name == server && config.is_none_or(|c| e.config_name == c))
        .collect();
    let pick = if let Some(e) = found.iter().find(|e| same_net(e.addr.ip(), own.addr.ip())) {
        e
    } else if let Some(e) = found.first() {
        eprintln!(
            "{program}: no local servers found in config file; using remote servers ({name})"
        );
        e
    } else {
        eprintln!("no servers found in config file");
        let text = format!(
            "Can't find RouteSrv({}) for {program}({app_index})",
            own.route_config_name
        );
        eprintln!("{text}");
        return Err(text);
    };
    Ok(if own.use_remote_ip {
        pick.remote_addr.unwrap_or(pick.addr)
    } else {
        pick.addr
    })
}

// ---------------------------------------------------------------- §2.4 Route.ini

#[derive(Clone, Debug)]
pub struct RouteIni {
    pub shutdown_timeout: Duration,
    pub registration_retry: Duration,
    pub registration_retry_limit: u32,
    pub check_registration_retry: Duration,
    pub check_registration_retry_limit: u32,
}

impl Default for RouteIni {
    fn default() -> Self {
        Self {
            shutdown_timeout: Duration::from_millis(60_000),
            registration_retry: Duration::from_millis(3000),
            registration_retry_limit: 1200,
            check_registration_retry: Duration::from_millis(5000),
            check_registration_retry_limit: 720,
        }
    }
}

impl RouteIni {
    /// Reads `[General]`. A missing or bad key gives the default.
    pub fn from_ini(ini: &Ini) -> Self {
        let d = Self::default();
        let ms = |key: &str, d: Duration| {
            u64::try_from(ini.int_or("General", key, d.as_millis() as i64))
                .map_or(d, Duration::from_millis)
        };
        let count = |key: &str, d: u32| {
            u32::try_from(ini.int_or("General", key, i64::from(d))).unwrap_or(d)
        };
        Self {
            shutdown_timeout: ms("ShutdownTimeoutMs", d.shutdown_timeout),
            registration_retry: ms("RegistrationRetry", d.registration_retry),
            registration_retry_limit: count("RegistrationRetryLimit", d.registration_retry_limit),
            check_registration_retry: ms("CheckRegistrationRetryMs", d.check_registration_retry),
            check_registration_retry_limit: count(
                "CheckRegistrationRetryLimit",
                d.check_registration_retry_limit,
            ),
        }
    }
}

// ---------------------------------------------------------------- client types

/// One protocol that the service provides.
#[derive(Clone, Debug)]
pub struct Registration {
    pub name: String,
    pub build: u32,
    pub topology: Topology,
    pub block_bits: u32,
}

#[derive(Clone, Debug)]
pub struct RouteClientConfig {
    pub router: SocketAddr,
    pub identity: ConnectInfo,
    pub own: Location,
    pub route_ini: RouteIni,
    pub providers: Vec<Registration>,
    pub subscriptions: Vec<String>,
    pub reconnect_delay: Duration,
    pub watchdog: Duration,
}

pub enum RouteEvent {
    Registered,
    RegistrationFailed,
    Unregistered,
    ProviderFound { name: String, provider: Location },
    ProviderLost { name: String, provider: Location },
    Request(Incoming),
}

impl fmt::Debug for RouteEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registered => f.write_str("Registered"),
            Self::RegistrationFailed => f.write_str("RegistrationFailed"),
            Self::Unregistered => f.write_str("Unregistered"),
            Self::ProviderFound { name, provider } => f
                .debug_struct("ProviderFound")
                .field("name", name)
                .field("provider", provider)
                .finish(),
            Self::ProviderLost { name, provider } => f
                .debug_struct("ProviderLost")
                .field("name", name)
                .field("provider", provider)
                .finish(),
            Self::Request(inc) => {
                let (protocol, command) = inc.command();
                write!(f, "Request({protocol}/{command})")
            }
        }
    }
}

/// A request that waits for the connection (§6.1).
struct Waiter {
    id: u64,
    bytes: usize,
    tx: oneshot::Sender<Peer>,
}

#[derive(Default)]
struct State {
    /// Set after the Sts/Connect OK reply, cleared on disconnect.
    peer: Option<Peer>,
    /// Handle to close the TCP socket (`shutdown` works on the socket, not on one fd).
    ctl: Option<Arc<std::net::TcpStream>>,
    registered: bool,
    shutting_down: bool,
    queue: VecDeque<Waiter>,
    queued_bytes: usize,
    next_waiter: u64,
    task: Option<JoinHandle<()>>,
}

struct Inner {
    cfg: RouteClientConfig,
    events: mpsc::UnboundedSender<RouteEvent>,
    state: Mutex<State>,
}

impl Inner {
    fn emit(&self, event: RouteEvent) {
        let _ = self.events.send(event);
    }

    /// Closes the socket, clears the peer and the registered flag (§5.3).
    fn mark_down(&self) {
        let (ctl, was_registered) = {
            let mut s = self.state.lock().unwrap();
            s.peer = None;
            (s.ctl.take(), std::mem::take(&mut s.registered))
        };
        if let Some(ctl) = ctl {
            let _ = ctl.shutdown(Shutdown::Both);
        }
        if was_registered {
            self.emit(RouteEvent::Unregistered);
        }
    }
}

#[derive(Clone)]
pub struct RouteClient {
    inner: Arc<Inner>,
}

impl RouteClient {
    /// Starts the connect loop. Needs a tokio runtime.
    pub fn start(cfg: RouteClientConfig) -> (Self, mpsc::UnboundedReceiver<RouteEvent>) {
        let (events, rx) = mpsc::unbounded_channel();
        let inner = Arc::new(Inner {
            cfg,
            events,
            state: Mutex::new(State::default()),
        });
        let task = tokio::spawn(run(inner.clone()));
        inner.state.lock().unwrap().task = Some(task);
        (Self { inner }, rx)
    }

    pub fn is_registered(&self) -> bool {
        self.inner.state.lock().unwrap().registered
    }

    /// Sends a request through the bus (§6). Adds `p: *<own location>` when
    /// absent. Not connected: the request waits up to 3000 ms, then fails
    /// with error 42.
    pub async fn request(&self, mut f: Frame, timeout: Duration) -> Result<Frame, StsError> {
        let inner = &self.inner;
        if header(&f, "p").is_none() {
            set_header(&mut f, "p", Target::Direct(inner.cfg.own).format());
        }
        let wait = {
            let mut s = inner.state.lock().unwrap();
            if let Some(peer) = &s.peer {
                Ok(peer.clone())
            } else {
                let bytes = frame_size(&f);
                let (tx, rx) = oneshot::channel();
                let id = s.next_waiter;
                s.next_waiter += 1;
                if bytes <= MAX_QUEUE_BYTES {
                    // The oldest request is dropped. It still fails after 3000 ms.
                    while s.queued_bytes + bytes > MAX_QUEUE_BYTES {
                        let Some(old) = s.queue.pop_front() else {
                            break;
                        };
                        s.queued_bytes -= old.bytes;
                    }
                    s.queued_bytes += bytes;
                    s.queue.push_back(Waiter { id, bytes, tx });
                }
                Err((id, rx))
            }
        };
        let (id, rx) = match wait {
            Ok(peer) => return peer.call(f, timeout).await,
            Err(w) => w,
        };
        let deadline = Instant::now() + NOT_CONNECTED_TIMEOUT;
        match tokio::time::timeout_at(deadline, rx).await {
            Ok(Ok(peer)) => peer.call(f, timeout).await,
            Ok(Err(_)) => {
                sleep_until(deadline).await;
                Err(StsError::new(ERR_TIMEOUT, MODULE_TIMEOUT, 0))
            }
            Err(_) => {
                let mut s = inner.state.lock().unwrap();
                if let Some(pos) = s.queue.iter().position(|w| w.id == id) {
                    if let Some(w) = s.queue.remove(pos) {
                        s.queued_bytes -= w.bytes;
                    }
                }
                Err(StsError::new(ERR_TIMEOUT, MODULE_TIMEOUT, 0))
            }
        }
    }

    /// Sends ShutdownServer, then closes the socket and stops the client (§7).
    pub async fn shutdown(&self) {
        let inner = &self.inner;
        let (peer, registered) = {
            let mut s = inner.state.lock().unwrap();
            s.shutting_down = true;
            (s.peer.clone(), s.registered)
        };
        if let Some(peer) = peer {
            let started = Instant::now();
            let frame = route_frame(&RouteMessage::new(RouteCmd::ShutdownServer));
            let result = peer
                .call(frame, inner.cfg.route_ini.shutdown_timeout)
                .await
                .and_then(|r| reply_result(&r).map(|_| ()));
            let secs = started.elapsed().as_secs();
            if secs >= 10 {
                eprintln!("shutdown txn takes over {secs} sec");
            }
            match result {
                Ok(()) => {}
                Err(e) if e.code == ERR_TIMEOUT => {
                    eprintln!(
                        "StsRoute: shutdown server txn timedout({secs}), [{}:{}:{}]",
                        e.code, e.module, e.line
                    );
                }
                Err(_) => eprintln!("StsRoute: Shutdown transaction"),
            }
        } else if registered {
            eprintln!("StsRoute: s_registered(1) still not 0 in RouterStartCleanup()");
        }
        inner.mark_down();
        if let Some(task) = inner.state.lock().unwrap().task.take() {
            task.abort();
        }
    }
}

fn frame_size(f: &Frame) -> usize {
    f.body.len()
        + f.headers
            .iter()
            .map(|h| h.name.len() + h.value.len() + 4)
            .sum::<usize>()
        + 64
}

fn addr_text(l: &Location) -> String {
    l.addr
        .map_or_else(|| "0.0.0.0:0".to_owned(), |a| a.to_string())
}

fn far() -> Instant {
    Instant::now() + Duration::from_secs(86_400 * 365)
}

// ---------------------------------------------------------------- frames

fn int(name: &str, v: u32) -> Element {
    Element::int(name, i64::from(v as i32))
}

/// `POST /Sts/Connect` with the `<Connect>` body in the order of §3.1.
fn connect_frame(id: &ConnectInfo) -> Frame {
    let l = &id.location;
    let mut e = Element::new("Connect");
    e.push(int("ConnType", CONN_TYPE_ROUTE));
    e.push_nonzero("ConnProductType", id.conn_product_type);
    e.push_nonzero("ConnAppIndex", id.conn_app_index);
    e.push_nonzero("ConnDeployment", id.conn_deployment);
    e.push_nonzero("ConnEpoch", id.conn_epoch);
    e.push(Element::text(
        "Address",
        l.addr.map(|a| a.to_string()).unwrap_or_default(),
    ));
    e.push(int("ProductType", l.product));
    e.push(int("AppIndex", l.index));
    e.push_nonzero("Deployment", l.deployment);
    e.push(int("Epoch", l.epoch));
    e.push(int("Program", id.program));
    e.push(int("Build", id.build));
    e.push(int("Process", id.process));
    e.push_nonzero("NotifyFlags", id.notify_flags);
    e.push_nonzero("VersionFlags", id.version_flags);
    request("Sts", "Connect", Some(&e))
}

/// Request line, `t` header and body of a Route message (`13-routesrv.md` §3).
fn route_frame(m: &RouteMessage) -> Frame {
    let (protocol, target) = m.request_target();
    let mut f = request(
        &protocol,
        m.cmd.map_or("", RouteCmd::name),
        Some(&m.to_element()),
    );
    if let Some(t) = target {
        set_header(&mut f, "t", t.format());
    }
    f
}

/// All registration requests, each with its check twin (§4.1, §4.2).
fn registration_items(cfg: &RouteClientConfig) -> Vec<(Frame, Frame)> {
    let mut msgs = Vec::new();
    for r in &cfg.providers {
        let grid = r.topology == Topology::Grid;
        let mut m = RouteMessage::new(RouteCmd::RegisterProvider);
        m.provider = Some(cfg.own);
        m.name = r.name.clone();
        m.build = if grid { 0 } else { r.build };
        m.topology = r.topology;
        m.block_bits = if grid { r.block_bits } else { 0 };
        msgs.push(m);
    }
    for name in &cfg.subscriptions {
        let mut m = RouteMessage::new(RouteCmd::SubscribeService);
        m.name = name.clone();
        msgs.push(m);
    }
    msgs.iter()
        .map(|m| {
            let req = route_frame(m);
            let mut check = req.clone();
            check.start = StartLine::Request {
                protocol: "Route".into(),
                command: RouteCmd::CheckRegistration.name().into(),
            };
            (req, check)
        })
        .collect()
}

// ---------------------------------------------------------------- connect loop

async fn run(inner: Arc<Inner>) {
    let router = inner.cfg.router;
    loop {
        if inner.state.lock().unwrap().shutting_down {
            return;
        }
        if let Ok(stream) = TcpStream::connect(router).await {
            session(&inner, stream).await;
            eprintln!(
                "StsRoute: CStsRouteConn::OnSocketDisconnect ({}) --x ({router})",
                addr_text(&inner.cfg.own)
            );
            inner.mark_down();
            if inner.state.lock().unwrap().shutting_down {
                return;
            }
        }
        tokio::time::sleep(inner.cfg.reconnect_delay).await;
    }
}

struct AbortOnDrop(JoinHandle<()>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

enum Flow {
    Continue,
    RouterConnect,
}

/// One TCP connection, from connect to close.
async fn session(inner: &Arc<Inner>, stream: TcpStream) {
    let cfg = &inner.cfg;
    let _ = stream.set_nodelay(true);
    let Ok(std_stream) = stream.into_std() else {
        return;
    };
    let Ok(ctl) = std_stream.try_clone() else {
        return;
    };
    let Ok(stream) = TcpStream::from_std(std_stream) else {
        return;
    };
    inner.state.lock().unwrap().ctl = Some(Arc::new(ctl));
    let (peer, mut incoming) = Peer::spawn(stream);
    eprintln!(
        "StsRoute: CStsRouteConn::OnSocketConnect ({}) --> ({})",
        addr_text(&cfg.own),
        cfg.router
    );

    let connect = peer.call(connect_frame(&cfg.identity), cfg.watchdog);
    tokio::pin!(connect);
    let watchdog = tokio::time::sleep(cfg.watchdog);
    tokio::pin!(watchdog);
    let (reg_tx, mut reg_rx) = mpsc::unbounded_channel::<bool>();
    let mut guard: Option<AbortOnDrop> = None;
    let mut connect_done = false;
    let mut started = false;
    let mut fallback: Option<Instant> = None;

    loop {
        let fallback_at = fallback.unwrap_or_else(far);
        tokio::select! {
            result = &mut connect, if !connect_done => {
                connect_done = true;
                match result.and_then(|f| reply_result(&f)) {
                    Ok(_) => {
                        let waiters: Vec<Waiter> = {
                            let mut s = inner.state.lock().unwrap();
                            s.peer = Some(peer.clone());
                            s.queued_bytes = 0;
                            s.queue.drain(..).collect()
                        };
                        for w in waiters {
                            let _ = w.tx.send(peer.clone());
                        }
                        fallback = Some(Instant::now() + ROUTER_CONNECT_WAIT);
                    }
                    Err(e) => {
                        if e.code == ERR_TIMEOUT {
                            eprintln!("StsRoute: CStsRouteConn::OnTimer(), disconnect!");
                        } else {
                            eprintln!("StsRoute: Sts/Connect failed, {e}");
                        }
                        break;
                    }
                }
            }
            _ = &mut watchdog, if !started => {
                eprintln!("StsRoute: CStsRouteConn::OnTimer(), disconnect!");
                break;
            }
            _ = sleep_until(fallback_at), if fallback.is_some() && !started => {
                started = true;
                guard = Some(start_registration(inner, &peer, reg_tx.clone()));
            }
            msg = incoming.recv() => {
                let Some(inc) = msg else { break };
                if matches!(handle_incoming(inner, inc), Flow::RouterConnect) && !started {
                    started = true;
                    guard = Some(start_registration(inner, &peer, reg_tx.clone()));
                }
            }
            Some(ok) = reg_rx.recv() => {
                let text = if ok { "success" } else { "failure" };
                eprintln!("StsRoute: CStsRouteConn::OnRouteRegisterFinished, registration finished({text})");
                if ok {
                    inner.state.lock().unwrap().registered = true;
                    inner.emit(RouteEvent::Registered);
                } else {
                    inner.emit(RouteEvent::RegistrationFailed);
                }
            }
        }
    }
    drop(guard);
}

fn start_registration(
    inner: &Inner,
    peer: &Peer,
    done: mpsc::UnboundedSender<bool>,
) -> AbortOnDrop {
    eprintln!("StsRoute: CStsRouteConn::DoRouterConnect, registration started");
    let items = registration_items(&inner.cfg);
    let (peer, ini) = (peer.clone(), inner.cfg.route_ini.clone());
    AbortOnDrop(tokio::spawn(async move {
        let ok = register(peer, items, ini).await;
        let _ = done.send(ok);
    }))
}

// ---------------------------------------------------------------- §5 router messages

fn handle_incoming(inner: &Inner, inc: Incoming) -> Flow {
    let (protocol, command) = {
        let (p, c) = inc.command();
        (p.to_owned(), c.to_owned())
    };
    // Cmd 10 and 11 always go to protocol Route (§5).
    let route = protocol == "Route" || command == "RouteXferRequest" || command == "RouteXferBlock";
    if !route {
        inner.emit(RouteEvent::Request(inc));
        return Flow::Continue;
    }
    match RouteCmd::from_name(&command) {
        Some(RouteCmd::ShutdownServer) => {
            inc.reply(None);
            let was_registered = std::mem::take(&mut inner.state.lock().unwrap().registered);
            if was_registered {
                inner.emit(RouteEvent::Unregistered);
            }
        }
        Some(cmd @ (RouteCmd::ProviderFound | RouteCmd::ProviderLost)) => {
            let msg = inc
                .body()
                .and_then(|b| RouteMessage::from_element(Some(cmd), &b));
            match msg {
                Some(m) if !m.name.is_empty() => {
                    if inner.cfg.subscriptions.contains(&m.name) {
                        let provider = m.provider.unwrap_or_default();
                        inner.emit(if cmd == RouteCmd::ProviderFound {
                            RouteEvent::ProviderFound {
                                name: m.name,
                                provider,
                            }
                        } else {
                            RouteEvent::ProviderLost {
                                name: m.name,
                                provider,
                            }
                        });
                    }
                    inc.reply(None);
                }
                _ => {
                    let what = if cmd == RouteCmd::ProviderFound {
                        "found"
                    } else {
                        "lost"
                    };
                    eprintln!("StsRoute: provider {what}, invalid notice");
                    inc.reply_error(&StsError::new(ERR_BAD_PARAM, MODULE_ROUTE_CLIENT, 0), 0);
                }
            }
        }
        // No reply: the spec has none for RouterConnect (§3.3 item 5).
        Some(RouteCmd::RouterConnect) => return Flow::RouterConnect,
        _ => {
            eprintln!("StsRoute: unknown route command, {command}");
            inc.reply_error(&StsError::new(ERR_BAD_PARAM, MODULE_ROUTE_CLIENT, 0), 0);
        }
    }
    Flow::Continue
}

// ---------------------------------------------------------------- §4 registration

type TxnResult = (usize, bool, Result<(), StsError>);

/// Registration state machine (§4.2, §4.4). `true` = success.
async fn register(peer: Peer, items: Vec<(Frame, Frame)>, ini: RouteIni) -> bool {
    let (tx, mut rx) = mpsc::unbounded_channel::<TxnResult>();
    let spawn_txn = |idx: usize, check: bool, pending: &mut usize| {
        *pending += 1;
        let frame = if check {
            items[idx].1.clone()
        } else {
            items[idx].0.clone()
        };
        let (peer, tx) = (peer.clone(), tx.clone());
        tokio::spawn(async move {
            let r = peer
                .call(frame, DEFAULT_TIMEOUT)
                .await
                .and_then(|f| reply_result(&f).map(|_| ()));
            let _ = tx.send((idx, check, r));
        });
    };
    let n = items.len();
    let mut pending = 0usize;
    let mut request_tries = vec![0u32; n];
    let mut check_tries = vec![0u32; n];
    let mut request_backlog: VecDeque<usize> = VecDeque::new();
    let mut check_backlog: VecDeque<usize> = VecDeque::new();
    let mut next_request: Option<Instant> = None;
    let mut next_check: Option<Instant> = None;
    let mut check_period = FIRST_CHECK_PERIOD;
    let mut counting = false;
    let counting_at = Instant::now() + REGISTER_TIMER;
    for idx in 0..n {
        spawn_txn(idx, false, &mut pending);
    }
    loop {
        if pending == 0 && request_backlog.is_empty() && check_backlog.is_empty() {
            return true;
        }
        tokio::select! {
            Some((idx, check, result)) = rx.recv() => {
                pending -= 1;
                let kind = if check { "check" } else { "request" };
                match result {
                    Ok(()) => {}
                    Err(e) if e.code == ERR_ROUTE_REGISTER_PENDING => {
                        if check && counting {
                            check_tries[idx] += 1;
                            if check_tries[idx] > ini.check_registration_retry_limit {
                                eprintln!(
                                    "StsRoute: CStsRouteRegister::Retry: retry limit exceed, type: {kind}, try count: {}",
                                    check_tries[idx]
                                );
                                return false;
                            }
                        }
                        check_backlog.push_back(idx);
                        next_check.get_or_insert_with(|| Instant::now() + check_period);
                    }
                    Err(e) => {
                        eprintln!("StsRoute: txn({kind}) err({})", e.code);
                        request_tries[idx] += 1;
                        if request_tries[idx] > ini.registration_retry_limit {
                            eprintln!(
                                "StsRoute: CStsRouteRegister::Retry: retry limit exceed, type: {kind}, try count: {}",
                                request_tries[idx]
                            );
                            return false;
                        }
                        request_backlog.push_back(idx);
                        next_request.get_or_insert_with(|| Instant::now() + ini.registration_retry);
                    }
                }
            }
            _ = sleep_until(counting_at), if !counting => {
                counting = true;
                check_period = ini.check_registration_retry;
            }
            _ = sleep_until(next_check.unwrap_or_else(far)), if next_check.is_some() => {
                next_check = None;
                if let Some(idx) = check_backlog.pop_front() {
                    spawn_txn(idx, true, &mut pending);
                }
                if !check_backlog.is_empty() {
                    next_check = Some(Instant::now() + check_period);
                }
            }
            _ = sleep_until(next_request.unwrap_or_else(far)), if next_request.is_some() => {
                next_request = None;
                if let Some(idx) = request_backlog.pop_front() {
                    spawn_txn(idx, false, &mut pending);
                }
                if !request_backlog.is_empty() {
                    next_request = Some(Instant::now() + ini.registration_retry);
                }
            }
        }
    }
}
