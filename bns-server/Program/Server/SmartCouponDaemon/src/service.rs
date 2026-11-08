//! Handlers and event pipeline (spec 03-handlers.md, 05-np-job.md).
use crate::config::{DaemonConfig, DaemonId, GameEventConfig, GamedList};
use crate::np::{keeps_publication_status, EventNoti, Np, NpReply};
use crate::protocol::{self as p, daemon_cmd as d, manager_cmd as m, reason};
use crate::store::{Account, EventLog, Publication, Store, Target};
use std::collections::{HashMap, VecDeque};
use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

/// Work posted to the game-event IOCP.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Work {
    /// GameEventLogEvent.
    GameEventLog(EventLog),
    /// SendPublicationEvent. `management_tid` > 0 when a manager asked for it.
    SendPublication {
        publication: Publication,
        management_tid: i64,
    },
    /// RetryCouponPublicationEvent.
    RetryPublication {
        tid: i64,
        account: Account,
        publication_id: i64,
    },
}

/// Result of one packet. `close` is the handler returning false.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Handled {
    pub replies: Vec<Vec<u8>>,
    pub close: bool,
    pub work: Vec<Work>,
}
impl Handled {
    fn reply(packet: Vec<u8>) -> Self {
        Self {
            replies: vec![packet],
            ..Self::default()
        }
    }
    fn close_with(packet: Option<Vec<u8>>) -> Self {
        Self {
            replies: packet.into_iter().collect(),
            close: true,
            work: Vec::new(),
        }
    }
}

pub struct DaemonConn {
    pub id: u64,
    pub ip: Ipv4Addr,
    pub authed: Option<DaemonId>,
    pub closer: Arc<Notify>,
}
impl DaemonConn {
    pub fn new(id: u64, ip: Ipv4Addr) -> Self {
        Self {
            id,
            ip,
            authed: None,
            closer: Arc::new(Notify::new()),
        }
    }
}

#[derive(Default)]
pub struct ManagerConn {
    pub hello: bool,
    pub manager_wid: i16,
}

pub struct Daemon<S, N> {
    pub config: DaemonConfig,
    pub events: GameEventConfig,
    pub gamed: GamedList,
    pub store: S,
    pub np: N,
    available: AtomicBool,
    sessions: Mutex<HashMap<DaemonId, (u64, Arc<Notify>)>>,
}

impl<S: Store, N: Np> Daemon<S, N> {
    pub fn new(
        config: DaemonConfig,
        events: GameEventConfig,
        gamed: GamedList,
        store: S,
        np: N,
    ) -> Self {
        Self {
            config,
            events,
            gamed,
            store,
            np,
            available: AtomicBool::new(true),
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn set_service_available(&self, available: bool) {
        self.available.store(available, Ordering::SeqCst);
    }

    fn sessions(&self) -> std::sync::MutexGuard<'_, HashMap<DaemonId, (u64, Arc<Notify>)>> {
        self.sessions.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn has_session(&self, id: &DaemonId) -> bool {
        self.sessions().contains_key(id)
    }

    /// Channel close: drop the world session if this connection owns it.
    pub fn disconnect(&self, conn: &DaemonConn) {
        if let Some(id) = conn.authed {
            let mut sessions = self.sessions();
            if sessions
                .get(&id)
                .is_some_and(|(owner, _)| *owner == conn.id)
            {
                sessions.remove(&id);
            }
        }
    }

    /// loadUnhandledGameEventLog and loadUnhandledCouponPublication.
    pub fn startup_work(&self) -> Result<Vec<Work>, crate::store::StoreError> {
        let mut work: Vec<Work> = self
            .store
            .load_game_event_log_list()?
            .into_iter()
            .map(Work::GameEventLog)
            .collect();
        work.extend(
            self.store
                .load_coupon_publication_list()?
                .into_iter()
                .map(|publication| Work::SendPublication {
                    publication,
                    management_tid: 0,
                }),
        );
        Ok(work)
    }

    /// DChannelMgr::PacketHandler @0x14000DFE0.
    pub fn daemon_packet(&self, conn: &mut DaemonConn, packet: &[u8]) -> Handled {
        let Some(cmd) = p::command(packet) else {
            return Handled::close_with(None);
        };
        if cmd < 0 || (conn.authed.is_none() && cmd != d::HELLO_REQ) {
            return Handled::close_with(None);
        }
        match cmd {
            d::HELLO_REQ => self.hello(conn, packet),
            d::BY_CHARACTER_REQ => self.by_character(conn, packet),
            d::BY_ACCOUNT_REQ => self.by_account(conn, packet),
            // Empty table slots ❓: treated as a failed handler.
            _ => Handled::close_with(None),
        }
    }

    /// HandleSmartCoupon_HelloReq @0x140007DC0.
    fn hello(&self, conn: &mut DaemonConn, packet: &[u8]) -> Handled {
        let fail = |r| Handled::close_with(Some(p::hello_fail(d::HELLO_FAIL, r)));
        if conn.authed.is_some() {
            return Handled::close_with(None);
        }
        let Some(v) = p::fields("hhQhh", packet) else {
            return fail(reason::SYSTEM);
        };
        if p::i16_at(&v, 1) != p::DAEMON_PACKET_VERSION {
            return fail(reason::VERSION);
        }
        if !self.available.load(Ordering::SeqCst) {
            return fail(reason::STATE);
        }
        let id = DaemonId {
            kind: p::i8_at(&v, 2),
            wid: p::i16_at(&v, 3),
        };
        let Some(info) = self.gamed.get(&id) else {
            return fail(reason::SYSTEM);
        };
        if info.ip != conn.ip {
            return fail(reason::IP);
        }
        let mut sessions = self.sessions();
        if let Some((_, old)) = sessions.remove(&id) {
            // ReplaceDaemon + Channel::Close: the old session closes and the
            // new one is refused, so the world must hello again.
            old.notify_one();
            return fail(reason::DUPLICATE_SESSION);
        }
        sessions.insert(id, (conn.id, conn.closer.clone()));
        conn.authed = Some(id);
        Handled::reply(p::ack(d::HELLO_ACK))
    }

    /// HandleSmartCoupon_GameEventByCharacterReq @0x140008360.
    fn by_character(&self, conn: &DaemonConn, packet: &[u8]) -> Handled {
        let Some(v) = p::fields("hqiohq", packet) else {
            return Handled::reply(p::tid_fail(
                d::BY_CHARACTER_FAIL,
                p::partial_tid(packet),
                reason::SYSTEM,
            ));
        };
        let channel = conn.authed.expect("authed");
        let (tid, event_id, account, world, pcdbid) = (
            p::i64_at(&v, 1),
            p::i32_at(&v, 2),
            p::id_at(&v, 3),
            p::i16_at(&v, 4),
            p::i64_at(&v, 5),
        );
        let fail = |r| Handled::reply(p::tid_fail(d::BY_CHARACTER_FAIL, tid, r));
        if account == [0; 16] || world <= 0 || world != channel.wid || pcdbid <= 0 || event_id <= 0
        {
            return fail(reason::INVALID_PARAM);
        }
        self.register(
            channel,
            Target::Character,
            tid,
            account,
            world,
            pcdbid,
            event_id,
            (d::BY_CHARACTER_ACK, d::BY_CHARACTER_FAIL),
        )
    }

    /// HandleSmartCoupon_GameEventByAccountReq @0x140008E80.
    fn by_account(&self, conn: &DaemonConn, packet: &[u8]) -> Handled {
        let Some(v) = p::fields("hqio", packet) else {
            return Handled::reply(p::tid_fail(
                d::BY_ACCOUNT_FAIL,
                p::partial_tid(packet),
                reason::SYSTEM,
            ));
        };
        let channel = conn.authed.expect("authed");
        let (tid, event_id, account) = (p::i64_at(&v, 1), p::i32_at(&v, 2), p::id_at(&v, 3));
        if account == [0; 16] || event_id <= 0 {
            return Handled::reply(p::tid_fail(d::BY_ACCOUNT_FAIL, tid, reason::INVALID_PARAM));
        }
        self.register(
            channel,
            Target::Account,
            tid,
            account,
            0,
            0,
            event_id,
            (d::BY_ACCOUNT_ACK, d::BY_ACCOUNT_FAIL),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn register(
        &self,
        channel: DaemonId,
        target: Target,
        tid: i64,
        account: Account,
        world: i16,
        pcdbid: i64,
        event_id: i32,
        (ok, fail): (i16, i16),
    ) -> Handled {
        match self.events.get(event_id) {
            Some(info) if info.daemon_type == channel.kind => (),
            _ => return Handled::reply(p::tid_fail(fail, tid, reason::INVALID_PARAM)),
        }
        let Ok(log_id) = self
            .store
            .register_game_event_log(target, &account, world, pcdbid, event_id)
        else {
            return Handled::reply(p::tid_fail(fail, tid, reason::PROCESS));
        };
        Handled {
            replies: vec![p::tid_ok(ok, tid, log_id)],
            close: false,
            work: vec![Work::GameEventLog(EventLog {
                log_id,
                target,
                account,
                world,
                pcdbid,
                event_id,
            })],
        }
    }

    /// ManagerSessionHandler::PacketHandler @0x14000E980.
    pub fn manager_packet(&self, conn: &mut ManagerConn, packet: &[u8]) -> Handled {
        let Some(cmd) = p::command(packet) else {
            return Handled::close_with(None);
        };
        if !(1..=9).contains(&cmd) || (!conn.hello && cmd != m::HELLO_REQ) {
            return Handled::close_with(None);
        }
        match cmd {
            m::HELLO_REQ => manager_hello(conn, packet),
            m::RETRY_REQ => retry(packet),
            m::LOAD_RETRY_REQ => self.load_retry(packet),
            _ => Handled::close_with(None),
        }
    }

    /// HandleManager_LoadRetryReadyGameEventLogListReq @0x14000F7F0.
    fn load_retry(&self, packet: &[u8]) -> Handled {
        let Some(v) = p::fields("hq", packet) else {
            return Handled::reply(p::tid_fail(m::LOAD_RETRY_FAIL, 0, reason::SYSTEM));
        };
        let tid = p::i64_at(&v, 1);
        if tid <= 0 {
            return Handled::reply(p::tid_fail(m::LOAD_RETRY_FAIL, tid, reason::INVALID_PARAM));
        }
        let Ok(logs) = self.store.load_retry_ready_game_event_log_list() else {
            return Handled::reply(p::tid_fail(m::LOAD_RETRY_FAIL, tid, reason::DB_LOAD));
        };
        Handled {
            replies: vec![p::tid_only(m::LOAD_RETRY_ACK, tid)],
            close: false,
            work: logs
                .into_iter()
                .take(crate::store::MAX_LIST)
                .map(Work::GameEventLog)
                .collect(),
        }
    }

    /// Run one posted job and every job it posts. Returns manager packets.
    pub async fn process(&self, work: Work) -> Vec<Vec<u8>> {
        let mut queue = VecDeque::from([work]);
        let mut out = Vec::new();
        while let Some(work) = queue.pop_front() {
            match work {
                Work::GameEventLog(log) => match self.store.process_game_event_log(&log) {
                    Ok(list) => queue.extend(list.into_iter().take(crate::store::MAX_LIST).map(
                        |publication| Work::SendPublication {
                            publication,
                            management_tid: 0,
                        },
                    )),
                    Err(_) => {
                        let _ = self.store.update_status_to_process_failure(&log);
                    }
                },
                Work::RetryPublication {
                    tid,
                    account,
                    publication_id,
                } => match self
                    .store
                    .load_single_coupon_publication(publication_id, &account)
                {
                    Err(_) => out.push(p::tid_fail(m::RETRY_FAIL, tid, reason::DB_LOAD)),
                    Ok(0) => out.push(p::tid_fail(m::RETRY_FAIL, tid, reason::NO_COUPON)),
                    Ok(coupon_event_id) => queue.push_back(Work::SendPublication {
                        publication: Publication {
                            publication_id,
                            account,
                            coupon_event_id,
                        },
                        management_tid: tid,
                    }),
                },
                Work::SendPublication {
                    publication,
                    management_tid,
                } => out.extend(self.send_publication(publication, management_tid).await),
            }
        }
        out
    }

    /// SendPublicationEvent @0x14000BE30 and the EventNotiJob life cycle.
    async fn send_publication(&self, publication: Publication, tid: i64) -> Option<Vec<u8>> {
        if publication.account == [0; 16]
            || publication.coupon_event_id == 0
            || publication.publication_id == 0
        {
            return (tid > 0).then(|| p::tid_fail(m::RETRY_FAIL, tid, reason::SYSTEM));
        }
        let request = EventNoti {
            account: publication.account,
            coupon_event_id: publication.coupon_event_id,
            publication_id: publication.publication_id,
            reward_changeover: self.config.np_reward_changeover_enable,
        };
        let mut accept_tid = 0;
        let outcome = match self.np.event_noti(request).await {
            Err(()) => reason::NP_SEND,
            Ok(NpReply::Expired) => reason::EXPIRED,
            Ok(NpReply::Success { accept_tid: accept }) => {
                accept_tid = accept;
                let _ = self
                    .store
                    .update_status_to_publication_success(&publication, accept);
                0
            }
            Ok(NpReply::Error { code }) => {
                if !keeps_publication_status(code) {
                    let _ = self
                        .store
                        .update_status_to_publication_failure(&publication);
                }
                reason::PROCESS
            }
        };
        // EventNotiJob::finalize @0x140009CC0: only manager-started jobs reply.
        if tid <= 0 {
            return None;
        }
        Some(if outcome == 0 {
            p::tid_ok(m::RETRY_OK, tid, accept_tid)
        } else {
            p::tid_fail(m::RETRY_FAIL, tid, outcome)
        })
    }
}

/// HandleManager_HelloReq @0x14000EF40.
fn manager_hello(conn: &mut ManagerConn, packet: &[u8]) -> Handled {
    let fail = |r| Handled::close_with(Some(p::hello_fail(m::HELLO_FAIL, r)));
    // The guard tests the channel id (GetId() != 0), so a channel that
    // said hello with manager wid 0 can say hello again.
    if conn.hello && conn.manager_wid != 0 {
        return fail(reason::STATE);
    }
    let Some(v) = p::fields("hhh", packet) else {
        return fail(reason::SYSTEM);
    };
    if p::i16_at(&v, 1) != p::MANAGER_PACKET_VERSION {
        return fail(reason::VERSION);
    }
    conn.hello = true;
    conn.manager_wid = p::i16_at(&v, 2);
    Handled::reply(p::ack(m::HELLO_ACK))
}

/// HandleManager_RetryCouponPublicationReq @0x14000F2F0.
fn retry(packet: &[u8]) -> Handled {
    let Some(v) = p::fields("hqoq", packet) else {
        return Handled::reply(p::tid_fail(
            m::RETRY_FAIL,
            p::partial_tid(packet),
            reason::SYSTEM,
        ));
    };
    let (tid, account, publication_id) = (p::i64_at(&v, 1), p::id_at(&v, 2), p::i64_at(&v, 3));
    if tid <= 0 || account == [0; 16] || publication_id <= 0 {
        return Handled::reply(p::tid_fail(m::RETRY_FAIL, tid, reason::INVALID_PARAM));
    }
    Handled {
        replies: Vec::new(),
        close: false,
        work: vec![Work::RetryPublication {
            tid,
            account,
            publication_id,
        }],
    }
}
