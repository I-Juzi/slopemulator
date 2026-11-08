//! Protocol core without sockets (spec README.md).
//! Every entry point returns the packets to send; server.rs routes them.
use crate::config::{DaemonConfig, IpList};
use crate::event::{id, Event, FieldValue, ParseError};
use crate::pc::{take, Pc, Profile};
use crate::protocol::{self, game_cmd, manager_cmd, reason};
use crate::rules::Rules;
use crate::store::Store;
use bloodlib::packet::Value;
use std::collections::{BTreeMap, HashMap};
use std::net::Ipv4Addr;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Out {
    /// To the sender of the request.
    Reply(Vec<u8>),
    /// To the GameDaemon session of one cluster.
    Cluster(i16, Vec<u8>),
    /// To every manager session that sent Hello.
    Managers(Vec<u8>),
    /// Remove this channel from routing. Close its socket only when requested.
    Expired {
        cluster: i16,
        token: u64,
        close: bool,
    },
}

#[derive(Debug, Clone)]
pub struct GameConn {
    pub ip: Ipv4Addr,
    pub cluster: Option<i16>,
    pub token: u64,
}

impl GameConn {
    pub fn new(ip: Ipv4Addr) -> Self {
        Self {
            ip,
            cluster: None,
            token: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ManagerConn {
    pub hello: bool,
}

#[derive(Debug, Clone)]
pub struct GameChannel {
    pub token: u64,
    pub last_ping_ms: u64,
}

#[derive(Default)]
pub struct State {
    pub pcs: HashMap<i64, Pc>,
    pub clusters: BTreeMap<i16, GameChannel>,
    /// Latest Hello owns cleanup even after CheckPing removes its channel.
    /// Keep the token until that session disconnects or a new Hello replaces it.
    cluster_owners: HashMap<i16, u64>,
    next_token: u64,
    /// Pc routing membership exists before a load completes (0x140009B50).
    pub routes: HashMap<i64, i16>,
}

pub struct Daemon<S> {
    pub config: DaemonConfig,
    pub iplist: IpList,
    pub rules: Rules,
    pub store: S,
    pub state: Mutex<State>,
}

fn int(ev: &Event, name: &str) -> i64 {
    ev.int(name) as i64
}

impl<S: Store> Daemon<S> {
    pub fn new(config: DaemonConfig, iplist: IpList, rules: Rules, store: S) -> Self {
        Self {
            config,
            iplist,
            rules,
            store,
            state: Mutex::default(),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn pc(&self, pcdbid: i64) -> Option<Pc> {
        self.lock().pcs.get(&pcdbid).cloned()
    }

    // ---- game port -------------------------------------------------------

    pub fn game_packet(&self, conn: &mut GameConn, packet: &[u8], now: i64) -> Vec<Out> {
        self.game_packet_at(conn, packet, now, now.max(0) as u64 * 1000)
    }

    /// Keep event time (seconds) separate from the monotonic Ping clock (ms).
    pub fn game_packet_at(
        &self,
        conn: &mut GameConn,
        packet: &[u8],
        now: i64,
        clock_ms: u64,
    ) -> Vec<Out> {
        let Some(cmd) = protocol::command(packet) else {
            return Vec::new();
        };
        if !(1..=game_cmd::MAX).contains(&cmd) {
            return Vec::new();
        }
        if cmd == game_cmd::HELLO_REQ {
            return self.game_hello(conn, packet, now, clock_ms);
        }
        let Some(cluster) = conn.cluster else {
            return Vec::new();
        };
        match cmd {
            game_cmd::PING => match protocol::fields("hqh", packet) {
                Some(v) => {
                    if let Some(channel) = self.lock().clusters.get_mut(&cluster) {
                        if channel.token == conn.token {
                            channel.last_ping_ms = clock_ms;
                        }
                    }
                    vec![Out::Reply(protocol::packet(
                        "hqh",
                        &[
                            Value::I16(game_cmd::PONG),
                            Value::I64(protocol::i64_at(&v, 1)),
                            Value::I16(protocol::i16_at(&v, 2)),
                        ],
                    ))]
                }
                None => Vec::new(),
            },
            game_cmd::EVENT_OCCURED_NOTIFY => match Event::parse(packet) {
                Ok(ev) => self.post(ev, now),
                Err(
                    ParseError::ShortHeader | ParseError::Decompose | ParseError::UnknownEvent(_),
                ) => Vec::new(),
            },
            game_cmd::COMPLETE_FROM_CLIENT_NOTIFY => self.complete_from_client(packet, now),
            _ => Vec::new(),
        }
    }

    fn game_hello(&self, conn: &mut GameConn, packet: &[u8], now: i64, clock_ms: u64) -> Vec<Out> {
        let fail = |r| vec![Out::Reply(protocol::hh(game_cmd::HELLO_FAIL, r))];
        let Some(v) = protocol::fields("hiih", packet) else {
            return fail(reason::SYSTEM);
        };
        let (packet_version, event_version, cluster) = (
            protocol::i32_at(&v, 1),
            protocol::i32_at(&v, 2),
            protocol::i16_at(&v, 3),
        );
        if packet_version != protocol::GAME_PACKET_VERSION
            || event_version != protocol::GAME_EVENT_VERSION
        {
            return fail(reason::VERSION);
        }
        if !self.iplist.allows(conn.ip, cluster) {
            return fail(reason::IP);
        }
        {
            let mut st = self.lock();
            if conn.cluster.is_some() || st.clusters.contains_key(&cluster) {
                return fail(reason::SYSTEM);
            }
            st.next_token += 1;
            conn.token = st.next_token;
            st.cluster_owners.insert(cluster, conn.token);
            st.clusters.insert(
                cluster,
                GameChannel {
                    token: conn.token,
                    last_ping_ms: clock_ms,
                },
            );
        }
        conn.cluster = Some(cluster);
        let mut out = vec![Out::Reply(protocol::hh(game_cmd::HELLO_ACK, cluster))];
        out.extend(self.post(
            Event::new(
                id::CLUSTER_CONNECTED,
                0,
                0,
                cluster,
                vec![FieldValue::Int(0)],
            ),
            now,
        ));
        out
    }

    /// Session closed: free the owned cluster and run ClusterDisconnected.
    pub fn game_disconnect(&self, conn: &GameConn, _now: i64) -> Vec<Out> {
        let Some(cluster) = conn.cluster else {
            return Vec::new();
        };
        let mut st = self.lock();
        if st.cluster_owners.get(&cluster) != Some(&conn.token) {
            return Vec::new();
        }
        st.cluster_owners.remove(&cluster);
        st.clusters.remove(&cluster);
        // CheckPing may already have removed the channel (0x14000CED0).
        // Keep the ownership check and Pc cleanup (0x140019990) atomic with Hello.
        self.cluster_disconnected(&mut st, cluster, false)
    }

    /// Native CheckPing removes up to 100 expired channels. Expiry is strict >.
    /// Socket closure causes the ordinary single-cluster disconnect path.
    pub fn check_ping(&self, clock_ms: u64) -> Vec<Out> {
        if !self.config.ping_enable {
            return Vec::new();
        }
        let mut st = self.lock();
        let expired: Vec<_> = st
            .clusters
            .iter()
            .filter(|(_, c)| {
                clock_ms.saturating_sub(c.last_ping_ms) > self.config.check_ping_time_ms
            })
            .take(100)
            .map(|(&cluster, c)| (cluster, c.token))
            .collect();
        expired
            .into_iter()
            .map(|(cluster, token)| {
                st.clusters.remove(&cluster);
                Out::Expired {
                    cluster,
                    token,
                    close: self.config.disconnect_when_ping_timeout,
                }
            })
            .collect()
    }

    /// CompleteFromClientNotify (cmd 8): each register row becomes event 1013,
    /// each achievement row event 1014.
    fn complete_from_client(&self, packet: &[u8], now: i64) -> Vec<Out> {
        let Some(v) = protocol::fields("hqqqqhChh;Chh;", packet) else {
            return Vec::new();
        };
        let header = |event: u16, a: i16, b: i16| Event {
            time: protocol::i64_at(&v, 1),
            tid: protocol::i64_at(&v, 2),
            pcdbid: protocol::i64_at(&v, 3),
            hint: protocol::i64_at(&v, 4),
            id: event,
            cluster_id: protocol::i16_at(&v, 5),
            values: vec![
                FieldValue::Int(a as u16 as u64),
                FieldValue::Int(b as u16 as u64),
            ],
        };
        let mut out = Vec::new();
        for row in protocol::rows_at(&v, 6) {
            let ev = header(
                id::INCREMENT_REGISTER_FROM_CLIENT,
                protocol::i16_at(row, 0),
                protocol::i16_at(row, 1),
            );
            out.extend(self.post(ev, now));
        }
        for row in protocol::rows_at(&v, 7) {
            let ev = header(
                id::COMPLETE_REQUEST_FROM_CLIENT,
                protocol::i16_at(row, 0),
                protocol::i16_at(row, 1),
            );
            out.extend(self.post(ev, now));
        }
        out
    }

    // ---- manager port ----------------------------------------------------

    pub fn manager_packet(&self, conn: &mut ManagerConn, packet: &[u8]) -> Vec<Out> {
        let Some(cmd) = protocol::command(packet) else {
            return Vec::new();
        };
        if cmd != manager_cmd::HELLO_REQ && !conn.hello {
            return Vec::new();
        }
        match cmd {
            manager_cmd::HELLO_REQ => {
                let Some(v) = protocol::fields("hi", packet) else {
                    return vec![Out::Reply(protocol::hh(
                        manager_cmd::HELLO_FAIL,
                        reason::SYSTEM,
                    ))];
                };
                if protocol::i32_at(&v, 1) != protocol::MANAGER_PACKET_VERSION {
                    return vec![Out::Reply(protocol::hh(
                        manager_cmd::HELLO_FAIL,
                        reason::VERSION,
                    ))];
                }
                conn.hello = true;
                vec![Out::Reply(protocol::packet(
                    "h",
                    &[Value::I16(manager_cmd::HELLO_ACK)],
                ))]
            }
            manager_cmd::PING => match protocol::fields("hqq", packet) {
                Some(v) => vec![Out::Reply(protocol::packet(
                    "hqq",
                    &[
                        Value::I16(manager_cmd::PONG),
                        Value::I64(protocol::i64_at(&v, 1)),
                        Value::I64(protocol::i64_at(&v, 2)),
                    ],
                ))],
                None => Vec::new(),
            },
            manager_cmd::CHECK_EVENT_PROCESSING_REQ if conn.hello => {
                let Some(v) = protocol::fields("hqq", packet) else {
                    return Vec::new();
                };
                let (pcdbid, tid) = (protocol::i64_at(&v, 1), protocol::i64_at(&v, 2));
                let logined = self.lock().routes.contains_key(&pcdbid);
                vec![Out::Managers(protocol::packet(
                    "hqqc",
                    &[
                        Value::I16(manager_cmd::CHECK_EVENT_PROCESSING_ACK),
                        Value::I64(pcdbid),
                        Value::I64(tid),
                        Value::U8(logined as u8),
                    ],
                ))]
            }
            _ => Vec::new(),
        }
    }

    // ---- event processing ------------------------------------------------

    /// Periodic update (`update-cycle`): updatePc(pc, clientUpdate=1) for every Pc.
    pub fn tick(&self) -> Vec<Out> {
        let mut st = self.lock();
        let mut out = Vec::new();
        for pc in st.pcs.values_mut() {
            out.extend(self.update_pc(pc, true));
        }
        out
    }

    /// EventManager::Post + processEvent.
    pub fn post(&self, ev: Event, now: i64) -> Vec<Out> {
        let mut st = self.lock();
        match ev.id {
            id::CLUSTER_CONNECTED | id::PC_IS_LOGINED => return Vec::new(),
            id::CLUSTER_DISCONNECTED => {
                return self.cluster_disconnected(
                    &mut st,
                    ev.cluster_id,
                    ev.int("disconnected_all") != 0,
                )
            }
            id::PC_LOGIN => {
                st.routes.insert(ev.pcdbid, ev.cluster_id);
                return self.login(&mut st, &ev, now);
            }
            _ => {}
        }
        if ev.id == id::PC_LOGOUT {
            st.routes.remove(&ev.pcdbid);
        }
        let Some(pc) = st.pcs.get_mut(&ev.pcdbid) else {
            return Vec::new(); // "pc not found"
        };
        if pc.tid != ev.tid {
            return Vec::new(); // "invalid tid"
        }
        if ev.id == id::PC_LOGOUT {
            let out = self.update_pc(pc, false);
            st.pcs.remove(&ev.pcdbid);
            return out;
        }
        let cluster = pc.cluster_id;
        let mut packets = Vec::new();
        let rules = &self.rules;
        match ev.id {
            id::INCREMENT_REGISTER_FROM_CLIENT => {
                let register = ev.int("register_id") as u16;
                if rules
                    .register(register, pc.job())
                    .is_some_and(|r| r.increment_from_client)
                {
                    pc.add_register(
                        rules,
                        register,
                        int(&ev, "register_value"),
                        now,
                        &mut packets,
                    );
                }
            }
            id::COMPLETE_REQUEST_FROM_CLIENT => {
                let (achievement, step) = (
                    ev.int("achievement_id") as u16,
                    ev.int("achievement_step") as u16,
                );
                if let Some(record) = rules.record(achievement, step, pc.job()) {
                    if record.complete_from_client {
                        pc.check_complete(rules, record, now, &mut packets);
                    }
                }
            }
            id::COMPLETE_TAKE_REWARD => {
                let transaction = int(&ev, "transaction_id");
                let (achievement, step, job) = (
                    ev.int("id") as u16,
                    ev.int("step") as u16,
                    ev.int("job") as u8,
                );
                let reason = match rules.record(achievement, step, job) {
                    Some(record) => pc.take_reward(rules, record, job),
                    None => take::NOT_FOUND,
                };
                if transaction != 0 || reason == take::NOT_FOUND {
                    packets.push(pc.take_reward_reply(transaction, achievement, step, reason));
                }
            }
            id::PC_LEVEL_UP => {
                pc.profile.faction = ev.int("faction") as u8;
                pc.profile.faction2 = ev.int("faction2") as u8;
                pc.profile.level = ev.int("level") as u16;
                pc.profile.mastery_level = ev.int("mastery_level") as u16;
                self.update_profile(pc, &["pc-level", "pc-mastery-level"], now, &mut packets);
            }
            id::EXPAND_INVENTORY => {
                pc.profile.inventory = ev.int("inventory") as u8;
                pc.profile.depot = ev.int("depot") as u8;
                pc.profile.wardrobe = ev.int("wardrobe") as u8;
                self.update_profile(
                    pc,
                    &["inventory-size", "depot-size", "wardrobe-size"],
                    now,
                    &mut packets,
                );
            }
            id::DUEL_RESULT => {
                pc.profile.duel_solo_grade = ev.int("duel_solo_grade") as u8;
                pc.profile.duel_team_grade = ev.int("duel_team_grade") as u8;
                self.update_profile(pc, &["duel-grade"], now, &mut packets);
            }
            id::PARTY_BATTLE_RESULT => {
                let kind = ev.int("party_battle_type") as usize;
                if (1..=3).contains(&kind) {
                    pc.profile.party_battle_grades[kind - 1] = ev.int("party_battle_grade") as u8;
                    self.update_profile(pc, &["party-battle-grade"], now, &mut packets);
                }
            }
            id::DEBUG_ADD_REGISTER => {
                pc.add_register(
                    rules,
                    ev.int("register_id") as u16,
                    int(&ev, "register_value"),
                    now,
                    &mut packets,
                );
            }
            id::DEBUG_SET_REGISTER => {
                pc.set_register(
                    rules,
                    ev.int("register_id") as u16,
                    int(&ev, "register_value"),
                    now,
                    &mut packets,
                );
            }
            id::DEBUG_DELETE_ACHIEVEMENT => {
                let achievement = ev.int("achievement_id") as u16;
                if self.store.delete_completed(pc.pcdbid, achievement).is_ok() {
                    pc.completed.remove(&achievement);
                    pc.rewards.remove(&achievement);
                    pc.score = pc.compute_score(rules);
                    packets.extend(pc.data_packets(false));
                }
            }
            id::DEBUG_DELETE_ALL if self.store.delete_all(pc.pcdbid).is_ok() => {
                pc.registers.clear();
                pc.completed.clear();
                pc.rewards.clear();
                pc.score = 0;
                packets.extend(pc.data_packets(false));
            }
            // Gameplay events (kill, quest, item, ...) are parsed and accepted.
            // Their register rules are phase 2 (spec README.md).
            _ => {}
        }
        packets
            .into_iter()
            .map(|p| Out::Cluster(cluster, p))
            .collect()
    }

    fn login(&self, st: &mut State, ev: &Event, now: i64) -> Vec<Out> {
        let name = ev.str("pcname");
        let profile = Profile {
            world_id: ev.int("worldId") as u16,
            level: ev.int("level") as u16,
            mastery_level: ev.int("mastery_level") as u16,
            faction: ev.int("faction") as u8,
            faction2: ev.int("faction2") as u8,
            inventory: ev.int("inventory") as u8,
            depot: ev.int("depot") as u8,
            wardrobe: ev.int("wardrobe") as u8,
            race: ev.int("race") as u8,
            sex: ev.int("sex") as u8,
            job: ev.int("job") as u8,
            pcid: int(ev, "pcid"),
            duel_solo_grade: ev.int("duel_solo_grade") as u8,
            duel_team_grade: ev.int("duel_team_grade") as u8,
            party_battle_grades: [
                ev.int("party_battle_occupation_war_grade") as u8,
                ev.int("party_battle_capture_the_flag_grade") as u8,
                ev.int("party_battle_lead_the_ball_grade") as u8,
            ],
            name,
        };
        if let Some(pc) = st.pcs.get_mut(&ev.pcdbid) {
            // Relogin: keep the rows, refresh the session fields.
            pc.tid = ev.tid;
            pc.cluster_id = ev.cluster_id;
            pc.profile = profile;
        } else {
            if st.pcs.len() >= self.config.max_pc_count {
                return Vec::new();
            }
            let Ok(rows) = self.store.load(ev.pcdbid) else {
                return Vec::new(); // Pc::Load failed: the login is dropped.
            };
            let pc = Pc::new(ev.pcdbid, ev.tid, ev.cluster_id, profile, rows, &self.rules);
            st.pcs.insert(ev.pcdbid, pc);
        }
        let pc = st.pcs.get_mut(&ev.pcdbid).expect("inserted above");
        pc.hint = ev.hint;
        let mut packets = std::mem::take(&mut pc.load_notifications);
        packets.extend(pc.data_packets(false));
        self.update_profile(
            pc,
            &[
                "pc-level",
                "pc-mastery-level",
                "inventory-size",
                "wardrobe-size",
                "depot-size",
                "duel-grade",
                "party-battle-grade",
            ],
            now,
            &mut packets,
        );
        let cluster = pc.cluster_id;
        packets
            .into_iter()
            .map(|p| Out::Cluster(cluster, p))
            .collect()
    }

    /// updatePcLevel / updateInventory / updateDuelGrade / updatePartyBattle:
    /// SetRegister on each register of the named types from the Pc profile.
    fn update_profile(&self, pc: &mut Pc, kinds: &[&str], now: i64, out: &mut Vec<Vec<u8>>) {
        for register in self.rules.registers.values() {
            if register.job != 0 && register.job != pc.job() {
                continue;
            }
            if !kinds.contains(&register.kind.as_str()) {
                continue;
            }
            let p = &pc.profile;
            let value = match register.kind.as_str() {
                "pc-level" => p.level as i64,
                "pc-mastery-level" => p.mastery_level as i64,
                "inventory-size" => p.inventory as i64,
                "depot-size" => p.depot as i64,
                "wardrobe-size" => p.wardrobe as i64,
                "duel-grade" => match register.sub_type {
                    1 => p.duel_solo_grade as i64,
                    2 => p.duel_team_grade as i64,
                    _ => p.duel_solo_grade.max(p.duel_team_grade) as i64,
                },
                "party-battle-grade" => match register.sub_type {
                    n @ 1..=3 => p.party_battle_grades[n as usize - 1] as i64,
                    _ => continue,
                },
                _ => continue,
            };
            if pc.register(register.id) != value || !pc.registers.contains_key(&register.id) {
                pc.set_register(&self.rules, register.id, value, now, out);
            }
        }
    }

    /// Native 0x140019990: select one cluster or all Pcs on the event thread.
    /// This port has one logical event thread.
    fn cluster_disconnected(&self, st: &mut State, cluster: i16, all: bool) -> Vec<Out> {
        st.routes.retain(|_, c| !all && *c != cluster);
        let ids: Vec<i64> = st
            .pcs
            .values()
            .filter(|p| all || p.cluster_id == cluster)
            .map(|p| p.pcdbid)
            .collect();
        for pcdbid in ids {
            if let Some(mut pc) = st.pcs.remove(&pcdbid) {
                let _ = self.update_pc(&mut pc, false);
            }
        }
        Vec::new()
    }

    /// updatePc: write marked rows to the store. Markers stay set when a
    /// store call fails. With `client`, SendChangedData first.
    fn update_pc(&self, pc: &mut Pc, client: bool) -> Vec<Out> {
        let changes = pc.changes();
        if changes.is_empty() {
            return Vec::new();
        }
        let out: Vec<Out> = if client {
            pc.data_packets(true)
                .into_iter()
                .map(|p| Out::Cluster(pc.cluster_id, p))
                .collect()
        } else {
            Vec::new()
        };
        let db = &self.store;
        // Each DB group is independent (0x140008B20).
        if db
            .insert_registers(pc.pcdbid, &changes.insert_registers)
            .is_ok()
        {
            pc.clear_registers(crate::pc::INSERT);
        }
        if db
            .update_registers(pc.pcdbid, &changes.update_registers)
            .is_ok()
        {
            pc.clear_registers(crate::pc::UPDATE);
        }
        if db
            .insert_completed(pc.pcdbid, &changes.insert_completed)
            .is_ok()
        {
            pc.clear_completed(crate::pc::INSERT);
        }
        if db
            .update_completed(pc.pcdbid, &changes.update_completed)
            .is_ok()
        {
            pc.clear_completed(crate::pc::UPDATE);
        }
        if db.take_step_reward(pc.pcdbid, &changes.rewards).is_ok() {
            pc.clear_rewards();
        }
        out
    }
}
