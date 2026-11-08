//! STS accept loop and Route commands. All state changes run under one lock.
use crate::{
    config::{RouteConfig, MAX_BLOCK_BITS},
    relay::{error_frame, unavailable, ConnId, Link, Relay},
    service::{Provider, Service},
};
use std::{
    collections::{BTreeSet, HashMap},
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use sts::{Frame, StartLine};
use sts_core::{
    conn::{header, headers_all, reply_ok, request, transaction, write_frame, FrameReader},
    connect::{header_param, ConnectInfo},
    route::{Location, Member, RouteCmd, RouteMessage, Target},
    Element, StsError,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpListener,
    sync::mpsc,
    task::JoinSet,
};

struct State {
    next_conn: ConnId,
    links: HashMap<ConnId, Link>,
    services: HashMap<String, Service>,
    relay: Relay,
}

pub struct RouteServer {
    pub config: RouteConfig,
    pub server_id: u32,
    state: Mutex<State>,
}

fn bad_body() -> StsError {
    // ❓ The spec does not give a source line for invalid Route command bodies.
    StsError::new(10, 2, 0)
}

impl RouteServer {
    pub fn new(config: RouteConfig, server_id: u32) -> Self {
        Self {
            config,
            server_id,
            state: Mutex::new(State {
                next_conn: 0,
                links: HashMap::new(),
                services: HashMap::new(),
                relay: Relay::default(),
            }),
        }
    }

    /// Local inspection API. It is not an extra Route wire command.
    pub fn snapshot(&self, protocol: &str) -> Option<RouteMessage> {
        let state = self.state.lock().unwrap();
        let service = state.services.get(protocol)?;
        let mut message = service.message(RouteCmd::ServiceChanged, protocol, None);
        message.providers = service
            .providers
            .iter()
            .map(|(code, provider)| Member {
                code: *code,
                location: provider.location,
                ..Member::default()
            })
            .collect();
        Some(message)
    }

    /// Accept until listener failure or shutdown. Stop all accepted sockets too.
    pub async fn serve(
        self: Arc<Self>,
        listener: TcpListener,
        shutdown: impl std::future::Future<Output = ()>,
    ) -> std::io::Result<()> {
        tokio::pin!(shutdown);
        let mut connections = JoinSet::new();
        let result = loop {
            tokio::select! {
                _ = &mut shutdown => break Ok(()),
                Some(_) = connections.join_next(), if !connections.is_empty() => {},
                accepted = listener.accept() => {
                    let (socket, addr) = match accepted { Ok(value) => value, Err(error) => break Err(error) };
                    let _ = socket.set_nodelay(true);
                    eprintln!("RouteSrv: connection from {addr}");
                    let server = self.clone();
                    connections.spawn(async move { let _ = server.serve_connection(socket, Some(addr)).await; });
                }
            }
        };
        connections.abort_all();
        while connections.join_next().await.is_some() {}
        result
    }

    /// Raw transport keeps reply frames and their sequence numbers intact.
    /// Also supports duplex streams for tests.
    pub async fn serve_connection<S>(
        self: Arc<Self>,
        stream: S,
        addr: Option<SocketAddr>,
    ) -> std::io::Result<()>
    where
        S: AsyncRead + AsyncWrite + Send + 'static,
    {
        let (out, mut rx) = mpsc::unbounded_channel();
        let id = {
            let mut state = self.state.lock().unwrap();
            state.next_conn += 1;
            let id = state.next_conn;
            state.links.insert(
                id,
                Link {
                    location: Location {
                        addr,
                        ..Location::default()
                    },
                    connected: false,
                    out,
                },
            );
            id
        };
        // Cleanup also runs when serve() aborts this connection task.
        let _guard = ConnectionGuard {
            server: self.clone(),
            id,
        };
        let (rd, mut wr) = tokio::io::split(stream);
        let mut reader = FrameReader::new(rd);
        let mut expiry = tokio::time::interval(Duration::from_millis(100));
        // Keep the write future outside the select: cancellation during a read
        // must not truncate an outgoing frame.
        let writer = async {
            while let Some(frame) = rx.recv().await {
                write_frame(&mut wr, &frame).await?;
            }
            wr.shutdown().await
        };
        tokio::pin!(writer);
        loop {
            tokio::select! {
                result = &mut writer => return result,
                _ = expiry.tick() => self.state.lock().unwrap().relay.expire(),
                frame = reader.next() => match frame? {
                    Some(frame) => if !self.handle(id, frame) { return Ok(()); },
                    None => return Ok(()),
                }
            }
        }
    }

    /// False closes an invalid Connect. Other failures are transaction replies.
    fn handle(&self, caller: ConnId, frame: Frame) -> bool {
        let mut state = self.state.lock().unwrap();
        if matches!(frame.start, StartLine::Response { .. }) {
            let State { links, relay, .. } = &mut *state;
            relay.reply(links, caller, frame, self.server_id);
            return true;
        }
        let StartLine::Request { protocol, command } = &frame.start else {
            unreachable!()
        };
        if protocol == "Sts" && command == "Connect" {
            let info = if header(&frame, "x-sts-connect").is_none() && !frame.body.is_empty() {
                Element::parse(&frame.body)
                    .ok()
                    .and_then(|body| ConnectInfo::from_body(&body, self.server_id))
            } else {
                ConnectInfo::from_frame(&frame, self.server_id)
            };
            let Some(info) = info else { return false };
            let link = state.links.get_mut(&caller).unwrap();
            // A second identity must not leave old registrations behind.
            if link.connected {
                return false;
            }
            if info.location.addr.is_some() {
                link.location = info.location;
            }
            link.connected = true;
            Self::ok(&state, caller, &frame, None);
            return true;
        }
        if header(&frame, "s").is_some() && transaction(&frame).is_none() {
            return false;
        }
        if transaction(&frame).is_some_and(|txn| txn.complete.is_some()) {
            self.send_error(&state, caller, &frame, &StsError::new(10, 2, 1170));
            return true;
        }
        let result = if protocol == "Route" {
            self.route_command(&mut state, caller, command, &frame)
        } else {
            self.forward(&mut state, caller, &frame)
        };
        if let Err(error) = result {
            self.send_error(&state, caller, &frame, &error);
        }
        true
    }

    fn ok(state: &State, caller: ConnId, frame: &Frame, body: Option<&Element>) {
        if let Some(txn) = transaction(frame) {
            let mut reply = reply_ok(txn.id, body);
            if let Some(origin) = header(frame, "p") {
                sts_core::conn::set_header(&mut reply, "t", origin);
            }
            if let Some(link) = state.links.get(&caller) {
                let _ = link.out.send(reply);
            }
        }
    }

    fn send_error(&self, state: &State, caller: ConnId, frame: &Frame, error: &StsError) {
        if let (Some(link), Some(reply)) = (
            state.links.get(&caller),
            error_frame(frame, error, self.server_id),
        ) {
            let _ = link.out.send(reply);
        }
    }

    fn notify(state: &State, subscribers: &[ConnId], message: RouteMessage) {
        let cmd = message.cmd.expect("notification command");
        let frame = request(
            "Route",
            cmd.name(),
            Some(&message.to_element_masked(cmd.mask())),
        );
        for id in subscribers {
            if let Some(link) = state.links.get(id) {
                let _ = link.out.send(frame.clone());
            }
        }
    }

    fn lost(state: &mut State, caller: ConnId, protocol: Option<&str>, location: Option<Location>) {
        let mut notices = Vec::new();
        for (name, service) in &mut state.services {
            if protocol.is_some_and(|p| p != name) {
                continue;
            }
            let subscribers: Vec<_> = service.subscribers.iter().copied().collect();
            for location in service.remove(caller, location) {
                notices.push((
                    subscribers.clone(),
                    service.message(RouteCmd::ProviderLost, name, Some(location)),
                ));
            }
        }
        for (subscribers, message) in notices {
            Self::notify(state, &subscribers, message);
        }
    }

    fn route_command(
        &self,
        state: &mut State,
        caller: ConnId,
        command: &str,
        frame: &Frame,
    ) -> Result<(), StsError> {
        let cmd = RouteCmd::from_name(command).ok_or_else(unavailable)?;
        match cmd {
            RouteCmd::CheckRegistration => {
                // ❓ The spec gives no response body. Phase 1 returns plain OK.
                Self::ok(state, caller, frame, None);
                return Ok(());
            }
            RouteCmd::ShutdownServer => {
                // ❓ Interpret this as shutdown of the sending service, not the router.
                Self::lost(state, caller, None, None);
                Self::ok(state, caller, frame, None);
                return Ok(());
            }
            RouteCmd::RegisterProvider
            | RouteCmd::UnregisterProvider
            | RouteCmd::SubscribeService
            | RouteCmd::SetBlocks
            | RouteCmd::GetBlocks
            | RouteCmd::Snapshot => {}
            _ => return Err(unavailable()),
        }
        let body = Element::parse(&frame.body).map_err(|_| bad_body())?;
        let message = RouteMessage::from_element(Some(cmd), &body).ok_or_else(bad_body)?;
        if message.name.is_empty() || message.block_bits > MAX_BLOCK_BITS {
            return Err(bad_body());
        }
        let name = &message.name;
        let bits = if body.get("Protocol").is_some_and(|p| p.has("BlockBits")) {
            message.block_bits
        } else {
            self.config.bits_for(name)
        };
        match cmd {
            RouteCmd::RegisterProvider => {
                let location = message.provider.unwrap_or(state.links[&caller].location);
                // ❓ Provider codes are local connection ids (1..0xFFFFFF).
                // The spec gives the code width, but not the assignment rule.
                let code = u32::try_from(caller)
                    .ok()
                    .filter(|code| *code <= 0xFF_FFFF)
                    .ok_or_else(unavailable)?;
                let service = state
                    .services
                    .entry(name.clone())
                    .or_insert_with(|| Service::new(bits));
                if service
                    .providers
                    .values()
                    .any(|p| p.location == location && p.conn != caller)
                {
                    return Err(bad_body());
                }
                if !service.providers.is_empty()
                    && (service.build != message.build
                        || service.topology != message.topology
                        || service.block_bits != bits)
                {
                    return Err(bad_body());
                }
                service.metadata(&message, bits);
                let previous = service.providers.insert(
                    code,
                    Provider {
                        conn: caller,
                        location,
                    },
                );
                service.rebalance();
                let subscribers: Vec<_> = service.subscribers.iter().copied().collect();
                let found = service.message(RouteCmd::ProviderFound, name, Some(location));
                let lost = previous
                    .as_ref()
                    .filter(|old| old.location != location)
                    .map(|old| service.message(RouteCmd::ProviderLost, name, Some(old.location)));
                Self::ok(state, caller, frame, None);
                if let Some(lost) = lost {
                    Self::notify(state, &subscribers, lost);
                }
                if previous.is_none_or(|old| old.location != location) {
                    Self::notify(state, &subscribers, found);
                }
            }
            RouteCmd::UnregisterProvider => {
                Self::lost(state, caller, Some(name), message.provider);
                Self::ok(state, caller, frame, None);
            }
            RouteCmd::SubscribeService => {
                let service = state
                    .services
                    .entry(name.clone())
                    .or_insert_with(|| Service::new(bits));
                let added = service.subscribers.insert(caller);
                let found: Vec<_> = if added {
                    service
                        .providers
                        .values()
                        .map(|p| service.message(RouteCmd::ProviderFound, name, Some(p.location)))
                        .collect()
                } else {
                    Vec::new()
                };
                Self::ok(state, caller, frame, None);
                for message in found {
                    Self::notify(state, &[caller], message);
                }
            }
            RouteCmd::SetBlocks => {
                let service = state.services.get_mut(name).ok_or_else(unavailable)?;
                if message.desired.len() != 1usize << bits
                    || message.desired.iter().any(|slot| {
                        slot.provider() != 0 && !service.providers.contains_key(&slot.provider())
                    })
                {
                    return Err(bad_body());
                }
                service.metadata(&message, bits);
                service.set_blocks(message.desired)?;
                let blocks = service.message(RouteCmd::BlocksFound, name, None);
                let subscribers: Vec<_> = service.subscribers.iter().copied().collect();
                Self::ok(state, caller, frame, None);
                Self::notify(state, &subscribers, blocks);
            }
            RouteCmd::GetBlocks | RouteCmd::Snapshot => {
                let service = state.services.get(name).ok_or_else(unavailable)?;
                let response_cmd = if cmd == RouteCmd::GetBlocks {
                    RouteCmd::BlocksFound
                } else {
                    RouteCmd::Snapshot
                };
                let provider = message
                    .provider
                    .or_else(|| service.providers.values().next().map(|p| p.location));
                let reply = service
                    .message(response_cmd, name, provider)
                    .to_element_masked(response_cmd.mask());
                Self::ok(state, caller, frame, Some(&reply));
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn forward(&self, state: &mut State, caller: ConnId, frame: &Frame) -> Result<(), StsError> {
        let StartLine::Request { protocol, .. } = &frame.start else {
            return Err(StsError::new(10, 2, 1115));
        };
        let mut targets = Vec::new();
        for value in headers_all(frame, "t") {
            let target = Target::parse(value).ok_or_else(|| {
                if value.starts_with('*') {
                    StsError::new(10, 2, 1573)
                } else {
                    unavailable()
                }
            })?;
            targets.push((target, value));
        }
        let protocol = targets
            .iter()
            .find_map(|(target, value)| match target {
                Target::Block { protocol, .. } => {
                    header_param(value, Some("protocol")).or(protocol.as_deref())
                }
                _ => None,
            })
            .filter(|p| !p.is_empty())
            .unwrap_or(if protocol.is_empty() { "Sts" } else { protocol });
        let service = state.services.get_mut(protocol).ok_or_else(unavailable)?;
        let mut providers = BTreeSet::new();
        if targets.is_empty() {
            providers.insert(service.resolve(None)?);
        } else {
            // Resolve first. Do not partly deliver an invalid multicast request.
            for (target, _) in &targets {
                providers.insert(service.resolve(Some(target))?);
            }
        }
        let State { links, relay, .. } = state;
        for provider in providers {
            relay.forward(links, caller, provider, protocol, frame.clone())?;
        }
        Ok(())
    }

    fn disconnect(&self, id: ConnId) {
        let mut state = self.state.lock().unwrap();
        state.links.remove(&id);
        for service in state.services.values_mut() {
            service.subscribers.remove(&id);
        }
        Self::lost(&mut state, id, None, None);
        let State { links, relay, .. } = &mut *state;
        relay.disconnect(links, id, self.server_id);
    }
}

struct ConnectionGuard {
    server: Arc<RouteServer>,
    id: ConnId,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.server.disconnect(self.id);
    }
}
