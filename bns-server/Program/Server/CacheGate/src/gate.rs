//! Pure relay state machine (spec README.md sections 3-5).
//!
//! The network layer gives every event to [`Gate`] and executes the
//! returned [`Out`] list. When a handler returns `Flow::Close`, the caller
//! closes the session that sent the packet (native handler returned false).
use crate::config::CachedInfo;
use crate::protocol::{self, cmd, reason, RelayHeader};
use std::collections::{BTreeMap, HashMap};
use std::net::Ipv4Addr;

/// Who sent a packet. Port 1 sessions are clients; port 2 sessions are
/// the outbound CacheD channels (`HIDWORD(session->_id) == 1` test).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Peer {
    Client(u64),
    Cached(u16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Out {
    Send(Peer, Vec<u8>),
    Close(Peer),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Keep,
    Close,
}

/// Reload result for the network layer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListChange {
    /// New or changed rows: start a connector.
    pub connect: Vec<CachedInfo>,
    /// Replaced rows: stop the reconnect timer and close.
    pub drop: Vec<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CachedState {
    pub info: CachedInfo,
    /// CacheD::_status: 1 after HelloOk, 0 on close.
    pub ready: bool,
}

/// Rust-only cap on one fragment chain (native has none ❓).
pub const MAX_FRAGMENT_BYTES: usize = 1 << 20;

#[derive(Debug, Default)]
struct Fragments {
    frames: Vec<Vec<u8>>,
    data: Vec<u8>,
    total: usize,
}

#[derive(Debug, Default)]
pub struct Gate {
    pub own_wid: u16,
    /// ClientManager::clientMap (wid -> session).
    clients: HashMap<u16, u64>,
    /// ClientManager::idIndexMap (session -> wid).
    index: HashMap<u64, u16>,
    /// CacheDList.
    cached: BTreeMap<u16, CachedState>,
    fragments: HashMap<Peer, Fragments>,
}

impl Gate {
    pub fn new(own_wid: u16) -> Self {
        Self {
            own_wid,
            ..Self::default()
        }
    }

    pub fn cached(&self, wid: u16) -> Option<&CachedState> {
        self.cached.get(&wid)
    }

    pub fn client(&self, wid: u16) -> Option<u64> {
        self.clients.get(&wid).copied()
    }

    /// CacheDList::LoadCachedListConfigFile merge rules. Unchanged rows
    /// stay, changed rows are replaced, new rows are added, rows missing
    /// from the file stay. At start (`reload == false`) the caller connects
    /// every row itself (MakeSessionAll); on reload only new or changed rows
    /// connect.
    pub fn apply_cached_list(&mut self, rows: &[CachedInfo]) -> ListChange {
        let mut change = ListChange::default();
        for row in rows {
            match self.cached.get(&row.wid) {
                Some(old) if old.info.ip == row.ip && old.info.port == row.port => {}
                Some(_) => {
                    change.drop.push(row.wid);
                    self.fragments.remove(&Peer::Cached(row.wid));
                    self.cached.insert(
                        row.wid,
                        CachedState {
                            info: *row,
                            ready: false,
                        },
                    );
                    change.connect.push(*row);
                }
                None => {
                    self.cached.insert(
                        row.wid,
                        CachedState {
                            info: *row,
                            ready: false,
                        },
                    );
                    change.connect.push(*row);
                }
            }
        }
        change
    }

    /// CacheDChannel::onConnect @0x140003C80: CacheD::Verify, then Hello
    /// with the CacheD's own wid as the 4th field ❓.
    pub fn cached_connected(&mut self, wid: u16, ip: Ipv4Addr, port: u16) -> Vec<Out> {
        match self.cached.get(&wid) {
            Some(c) if c.info.ip == ip && c.info.port == port => {
                vec![Out::Send(Peer::Cached(wid), protocol::hello(wid))]
            }
            _ => vec![Out::Close(Peer::Cached(wid))],
        }
    }

    /// CacheDChannel::onClose: status 0.
    pub fn cached_closed(&mut self, wid: u16) {
        if let Some(c) = self.cached.get_mut(&wid) {
            c.ready = false;
        }
        self.fragments.remove(&Peer::Cached(wid));
    }

    /// CacheGateSessionHandler::OnClose: ClientManager::ReleaseSession.
    pub fn client_closed(&mut self, session: u64) {
        if let Some(wid) = self.index.remove(&session) {
            if self.clients.get(&wid) == Some(&session) {
                self.clients.remove(&wid);
            }
        }
        self.fragments.remove(&Peer::Client(session));
    }

    /// PacketHandler @0x140004150.
    pub fn handle(&mut self, from: Peer, frame: &[u8]) -> (Vec<Out>, Flow) {
        let mut out = Vec::new();
        let flow = match protocol::command(frame) {
            Some(cmd::HELLO) => self.hello(from, frame, &mut out),
            Some(cmd::HELLO_OK) => self.hello_ok(from),
            // HelloFail and RelayFail: log only, return false.
            Some(cmd::HELLO_FAIL) | Some(cmd::RELAY_FAIL) => Flow::Close,
            Some(cmd::RELAY) => self.relay(from, frame, &mut out),
            // TheOtherSideListAck @0x14000CD80 fills slots 6, 7 and 8.
            Some(cmd::RELAY_INVALID)
            | Some(cmd::THE_OTHER_SIDE_LIST_REQ)
            | Some(cmd::THE_OTHER_SIDE_LIST_ACK) => Flow::Keep,
            Some(cmd::FRAGMENT_RELAY) => self.fragment(from, frame, &mut out),
            // Slot 0 is null; cmds outside 0..10 read past the table.
            _ => Flow::Close,
        };
        (out, flow)
    }

    /// Hello @0x140004C00 (`hhhh`).
    fn hello(&mut self, from: Peer, frame: &[u8], out: &mut Vec<Out>) -> Flow {
        let fields = (
            protocol::i16_at(frame, 4),
            protocol::i16_at(frame, 6),
            protocol::i16_at(frame, 8),
        );
        let (Some(version), Some(economy), Some(wid)) = fields else {
            return Flow::Close;
        };
        let wid = wid as u16;
        if version != protocol::PACKET_VERSION || economy != protocol::ECONOMY_PACKET_VERSION {
            out.push(Out::Send(from, protocol::hello_fail(reason::VERSION)));
            return Flow::Close;
        }
        let Peer::Client(session) = from else {
            // Only port 1 sessions register in ClientManager ❓.
            return Flow::Close;
        };
        if let Some(&old) = self.clients.get(&wid) {
            out.push(Out::Send(from, protocol::hello_fail(reason::DUPLICATE)));
            out.push(Out::Close(Peer::Client(old)));
            return Flow::Close;
        }
        if self.index.contains_key(&session) {
            out.push(Out::Send(from, protocol::hello_fail(reason::SYSTEM)));
            return Flow::Close;
        }
        self.clients.insert(wid, session);
        self.index.insert(session, wid);
        out.push(Out::Send(from, protocol::hello_ok()));
        Flow::Keep
    }

    /// HelloOk @0x140004F20. Needs a CacheD channel ("unknow hello ok").
    /// Verify already passed on connect, so status becomes 1.
    fn hello_ok(&mut self, from: Peer) -> Flow {
        let Peer::Cached(wid) = from else {
            return Flow::Close;
        };
        match self.cached.get_mut(&wid) {
            Some(c) => {
                c.ready = true;
                Flow::Keep
            }
            None => Flow::Close,
        }
    }

    /// Route for Relay and FragmentRelay: Ok(target) or Err(reason).
    fn route(&self, from: Peer, to_wid: u16) -> Result<Peer, i16> {
        match from {
            Peer::Client(_) => match self.cached.get(&to_wid) {
                Some(c) if c.ready => Ok(Peer::Cached(to_wid)),
                Some(_) => Err(reason::NOT_READY),
                None => Err(reason::NO_TARGET),
            },
            Peer::Cached(_) => self
                .clients
                .get(&to_wid)
                .map(|&s| Peer::Client(s))
                .ok_or(reason::NO_TARGET),
        }
    }

    /// Relay @0x1400051C0 (`hhhqhq` + payload). The forwarded frame is
    /// byte-identical to the input.
    fn relay(&mut self, from: Peer, frame: &[u8], out: &mut Vec<Out>) -> Flow {
        let Some(header) = RelayHeader::read(frame, 4) else {
            return Flow::Close;
        };
        match self.route(from, header.to_wid) {
            Ok(target) => out.push(Out::Send(target, frame.to_vec())),
            Err(code) => out.push(Out::Send(
                from,
                protocol::relay_fail(self.own_wid, &header, code),
            )),
        }
        Flow::Keep
    }

    /// FragmentRelay @0x140005880 with the per-session concatenater.
    fn fragment(&mut self, from: Peer, frame: &[u8], out: &mut Vec<Out>) -> Flow {
        let Some(mark) = protocol::i32_at(frame, 4) else {
            self.fragments.remove(&from);
            return Flow::Close;
        };
        let data = &frame[protocol::FRAGMENT_DATA_AT..];
        let buf = self.fragments.entry(from).or_default();
        if mark > 0 {
            if !buf.frames.is_empty() {
                self.fragments.remove(&from);
                return Flow::Close;
            }
            buf.total = mark as usize;
        } else if buf.frames.is_empty() {
            self.fragments.remove(&from);
            return Flow::Close;
        }
        if buf.data.len() + data.len() > MAX_FRAGMENT_BYTES.min(buf.total) {
            self.fragments.remove(&from);
            return Flow::Close;
        }
        buf.frames.push(frame.to_vec());
        buf.data.extend_from_slice(data);
        let complete = if mark > 0 {
            buf.data.len() == buf.total
        } else {
            mark == 0
        };
        if !complete {
            return Flow::Keep;
        }
        let Some(chain) = self.fragments.remove(&from) else {
            return Flow::Close;
        };
        let Some(header) = RelayHeader::read(&chain.data, 0) else {
            return Flow::Close;
        };
        match self.route(from, header.to_wid) {
            Ok(target) => {
                for f in chain.frames {
                    out.push(Out::Send(target, f));
                }
            }
            Err(code) => out.push(Out::Send(
                from,
                protocol::relay_fail(self.own_wid, &header, code),
            )),
        }
        Flow::Keep
    }
}
