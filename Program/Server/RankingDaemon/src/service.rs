//! Pure session state and packet handlers (spec README.md sections 3-6).
//!
//! The network layer gives each frame to [`Daemon`] with the sender and
//! executes the returned [`Out`] list. `Flow::Close` means the native
//! handler returned false: the caller closes the sending session.
use crate::config::{GameInfo, InfoGateInfo, RankingConfig, Season, SERVICE_ARENA, SERVICE_GAME};
use crate::protocol::{
    self, fields, game_cmd, hh, i16_at, i32_at, i8_at, infogate_cmd, job_reason, manage_cmd,
    packet, reason, u64_at, u8_at, wide,
};
use bloodlib::packet::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::net::Ipv4Addr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Peer {
    Game(u64),
    InfoGate(u64),
    Manage(u64),
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

/// FetchSeasonList format (game HelloAck, fragment cmd 2).
pub const SEASON_LIST_FORMAT: &str = "hiCcischqqqhiCcqqqqQ;;CcischqqqhiCcqqqqQ;;";
/// FetchCurrentSeasonContentsList format (InfoGate HelloAck, cmd 2).
pub const CURRENT_SEASON_FORMAT: &str = "hCcc;";

/// RankingBoardManager::convertSeasonContentsTypeToMaxRankingListIndex @0x14001BAE0.
pub fn max_ranking_list_index(season_contents_type: u8) -> u8 {
    match season_contents_type {
        1 | 3..=7 | 9..=12 | 14 | 15 => 14,
        8 => 2,
        13 => 1,
        _ => 0,
    }
}

#[derive(Debug, Clone)]
struct Channel<I> {
    info: I,
    session: Option<u64>,
}

#[derive(Debug)]
pub struct Daemon {
    pub service_mode: i32,
    /// RankingDaemon::_serviceAvailable.
    pub service_available: bool,
    /// GameChannelMgr: (service type, id) -> channel.
    games: BTreeMap<(i16, i16), Channel<GameInfo>>,
    game_sessions: HashMap<u64, (i16, i16)>,
    /// InfoGateChannelMgr: wid -> channel.
    infogates: BTreeMap<i16, Channel<InfoGateInfo>>,
    infogate_sessions: HashMap<u64, i16>,
    /// Manage sessions with a ManageChannel (after HelloReq).
    managers: HashSet<u64>,
    /// RankingSeasonManager seasons for this service mode.
    seasons: Vec<Season>,
    /// RankingDB black list: (playerType, playerId). In memory ❓.
    black_list: BTreeSet<(i8, u64)>,
}

impl Daemon {
    pub fn new(
        config: &RankingConfig,
        games: &[GameInfo],
        infogates: &[InfoGateInfo],
        seasons: Vec<Season>,
    ) -> Self {
        Self {
            service_mode: config.service_mode,
            service_available: true,
            games: games
                .iter()
                .map(|g| {
                    (
                        (g.service_type, g.id),
                        Channel {
                            info: *g,
                            session: None,
                        },
                    )
                })
                .collect(),
            game_sessions: HashMap::new(),
            infogates: infogates
                .iter()
                .map(|i| {
                    (
                        i.wid,
                        Channel {
                            info: *i,
                            session: None,
                        },
                    )
                })
                .collect(),
            infogate_sessions: HashMap::new(),
            managers: HashSet::new(),
            seasons: seasons
                .into_iter()
                .filter(|s| s.service_mode == config.service_mode)
                .collect(),
            black_list: BTreeSet::new(),
        }
    }

    pub fn seasons(&self) -> &[Season] {
        &self.seasons
    }

    pub fn is_black_listed(&self, player_type: i8, player_id: u64) -> bool {
        self.black_list.contains(&(player_type, player_id))
    }

    pub fn game_session(&self, service_type: i16, id: i16) -> Option<u64> {
        self.games.get(&(service_type, id)).and_then(|c| c.session)
    }

    pub fn infogate_session(&self, wid: i16) -> Option<u64> {
        self.infogates.get(&wid).and_then(|c| c.session)
    }

    /// Seasons with start <= now <= end (GetActivatedSeasonContentsTypeList).
    pub fn active_seasons(&self, now: i64) -> impl Iterator<Item = &Season> {
        self.seasons
            .iter()
            .filter(move |s| s.start_time <= now && now <= s.end_time)
    }

    /// Session close: the channel loses its session.
    pub fn closed(&mut self, peer: Peer) {
        match peer {
            Peer::Game(id) => {
                if let Some(key) = self.game_sessions.remove(&id) {
                    if let Some(c) = self.games.get_mut(&key) {
                        if c.session == Some(id) {
                            c.session = None;
                        }
                    }
                }
            }
            Peer::InfoGate(id) => {
                if let Some(wid) = self.infogate_sessions.remove(&id) {
                    if let Some(c) = self.infogates.get_mut(&wid) {
                        if c.session == Some(id) {
                            c.session = None;
                        }
                    }
                }
            }
            Peer::Manage(id) => {
                self.managers.remove(&id);
            }
        }
    }

    /// Dispatch one frame. `ip` is the peer address of the session.
    pub fn handle(&mut self, from: Peer, ip: Ipv4Addr, frame: &[u8], now: i64) -> (Vec<Out>, Flow) {
        let mut out = Vec::new();
        let flow = match protocol::command(frame) {
            None => Flow::Close,
            Some(cmd) => match from {
                Peer::Game(id) => self.game(id, ip, cmd, frame, now, &mut out),
                Peer::InfoGate(id) => self.infogate(id, ip, cmd, frame, now, &mut out),
                Peer::Manage(id) => self.manage(id, cmd, frame, now, &mut out),
            },
        };
        (out, flow)
    }

    // ---- game port -----------------------------------------------------

    /// GameChannelMgr::PacketHandler @0x140009610.
    fn game(
        &mut self,
        id: u64,
        ip: Ipv4Addr,
        cmd: i16,
        frame: &[u8],
        now: i64,
        out: &mut Vec<Out>,
    ) -> Flow {
        use game_cmd::*;
        let me = Peer::Game(id);
        let handled = matches!(
            cmd,
            HELLO_REQ
                | PING_REQ
                | RETRIEVE_PLAYER_RANKING_INFO_REQ
                | RETRIEVE_PC_RANKING_RESULT_LIST_REQ
                | LOAD_GUILD_RANKING_RESULT_LIST_REQ
                | LOAD_PLAYER_SEASON_RESULT_REQ
                | INSERT_PLAYER_RATING_SCORE_REQ
                | INSERT_RANKING_SCORE_REQ
                | DELETE_PLAYER_NOTIFY
        );
        if !handled {
            return Flow::Close;
        }
        let authed = self.game_sessions.contains_key(&id);
        if !authed {
            return if cmd == HELLO_REQ {
                self.game_hello(id, ip, frame, now, out)
            } else {
                Flow::Close
            };
        }
        let reply = |out: &mut Vec<Out>, p: Vec<u8>| out.push(Out::Send(me, p));
        match cmd {
            // Already authed: HelloReq returns 0.
            HELLO_REQ => Flow::Close,
            // PingReq @0x14000A4D0: echo `hqh`, always returns 1.
            PING_REQ => {
                if let Some(v) = fields("hqh", frame) {
                    reply(
                        out,
                        packet("hqh", &[Value::I16(PING_ACK), v[1].clone(), v[2].clone()]),
                    );
                }
                Flow::Keep
            }
            // RetrievePlayerRankingInfoReq @0x14000A630 `hqQqci`.
            RETRIEVE_PLAYER_RANKING_INFO_REQ => {
                let Some(v) = fields("hqQqci", frame) else {
                    return Flow::Close;
                };
                // No ranking board yet: "failed to get ranking with top ranker", 2.
                let mut f = v.clone();
                f[0] = Value::I16(RETRIEVE_PLAYER_RANKING_INFO_FAIL);
                f.push(Value::I16(job_reason::NO_BOARD));
                reply(out, packet("hqQqciE", &f));
                Flow::Keep
            }
            // RetrieveRankingResultListJob (verify: worldId and playerId != 0).
            RETRIEVE_PC_RANKING_RESULT_LIST_REQ => {
                let Some(v) = fields("hhq", frame) else {
                    return Flow::Close;
                };
                let code = if i16_at(&v, 1) == 0 || u64_at(&v, 2) == 0 {
                    job_reason::DEFAULT
                } else {
                    job_reason::DB
                };
                reply(
                    out,
                    packet(
                        "hhqE",
                        &[
                            Value::I16(RETRIEVE_PC_RANKING_RESULT_LIST_FAIL),
                            v[1].clone(),
                            v[2].clone(),
                            Value::I16(code),
                        ],
                    ),
                );
                Flow::Keep
            }
            // LoadGuildRankingResultListJob (verify: clusterId != 0).
            LOAD_GUILD_RANKING_RESULT_LIST_REQ => {
                let Some(v) = fields("hhq", frame) else {
                    return Flow::Close;
                };
                let code = if i16_at(&v, 1) == 0 {
                    job_reason::DEFAULT
                } else {
                    job_reason::DB
                };
                reply(
                    out,
                    packet(
                        "hhqE",
                        &[
                            Value::I16(LOAD_GUILD_RANKING_RESULT_LIST_FAIL),
                            v[1].clone(),
                            v[2].clone(),
                            Value::I16(code),
                        ],
                    ),
                );
                Flow::Keep
            }
            // LoadPlayerSeasonResultByPlayerTypeListJob (verify: clusterId != 0).
            LOAD_PLAYER_SEASON_RESULT_REQ => {
                let Some(v) = fields("hhhq", frame) else {
                    return Flow::Close;
                };
                let code = if i16_at(&v, 1) == 0 {
                    job_reason::DEFAULT
                } else {
                    job_reason::DB
                };
                reply(
                    out,
                    packet(
                        "hhhqE",
                        &[
                            Value::I16(LOAD_PLAYER_SEASON_RESULT_FAIL),
                            v[1].clone(),
                            v[2].clone(),
                            v[3].clone(),
                            Value::I16(code),
                        ],
                    ),
                );
                Flow::Keep
            }
            INSERT_PLAYER_RATING_SCORE_REQ => self.insert_rating(me, frame, now, out),
            INSERT_RANKING_SCORE_REQ => self.insert_ranking(me, frame, now, out),
            DELETE_PLAYER_NOTIFY => {
                // DeletePlayerNotify @0x14000B670 `hQq`: DeletePlayerJob, no reply.
                let Some(v) = fields("hQq", frame) else {
                    return Flow::Close;
                };
                let (player_type, player_id) = (i8_at(&v, 1), u64_at(&v, 2));
                if (1..3).contains(&player_type) && player_id != 0 {
                    self.black_list.insert((player_type, player_id));
                }
                Flow::Keep
            }
            _ => Flow::Close,
        }
    }

    /// HandleRankingGame_HelloReq @0x140009F00 (`hhh`: cmd, version, clusterId).
    fn game_hello(
        &mut self,
        id: u64,
        ip: Ipv4Addr,
        frame: &[u8],
        now: i64,
        out: &mut Vec<Out>,
    ) -> Flow {
        let me = Peer::Game(id);
        let fail = |out: &mut Vec<Out>, code| {
            out.push(Out::Send(me, hh(game_cmd::HELLO_FAIL, code)));
            Flow::Close
        };
        let Some(v) = fields("hhh", frame) else {
            return fail(out, reason::SYSTEM);
        };
        if i16_at(&v, 1) != protocol::GAME_PACKET_VERSION {
            return fail(out, reason::VERSION);
        }
        if !self.service_available {
            return fail(out, reason::SYSTEM);
        }
        let cluster = i16_at(&v, 2);
        let kind = if cluster >= protocol::ARENA_ID_BASE {
            SERVICE_ARENA
        } else {
            SERVICE_GAME
        };
        let key = (kind, cluster);
        let Some(channel) = self.games.get_mut(&key) else {
            return fail(out, reason::SYSTEM);
        };
        if channel.info.ip != ip {
            return fail(out, reason::IP);
        }
        if let Some(old) = channel.session.take() {
            // ReplaceGame with a new channel, close the old one, fail.
            self.game_sessions.remove(&old);
            out.push(Out::Close(Peer::Game(old)));
            return fail(out, reason::DUPLICATE);
        }
        let Some(frames) = self.season_list(now) else {
            return fail(out, reason::SEASON_LIST);
        };
        if let Some(channel) = self.games.get_mut(&key) {
            channel.session = Some(id);
        }
        self.game_sessions.insert(id, key);
        out.extend(frames.into_iter().map(|f| Out::Send(me, f)));
        Flow::Keep
    }

    /// RankingSeasonManager::FetchSeasonList @0x14002C550 as HelloAck fragments.
    pub fn season_list(&self, now: i64) -> Option<Vec<Vec<u8>>> {
        let row = |s: &Season| {
            vec![
                Value::U8(s.season_contents_type),
                Value::U32(s.schedule_number as u32),
                wide(&s.name),
                Value::U8(s.season_type),
                Value::I16(s.player_type),
                Value::U64(s.start_time as u64),
                Value::U64(s.end_time as u64),
                Value::U64(s.result_expiration_time as u64),
                Value::I16(s.reward_group_id),
                Value::U32(s.default_rating_score as u32),
                Value::Repeat(
                    s.results
                        .iter()
                        .map(|r| {
                            vec![
                                Value::U8(r.number),
                                Value::U64(r.ranking_result_start_time as u64),
                                Value::U64(r.closing_noti_time as u64),
                                Value::U64(r.reward_start_time as u64),
                                Value::U64(r.result_expiration_time as u64),
                                Value::I8(r.is_finality_result as i8),
                            ]
                        })
                        .collect(),
                ),
            ]
        };
        // Activated list: now <= endTime. Result list: now <= resultExpirationTime.
        // This port keeps one season list; result seasons are those past their end ❓.
        let active = self.seasons.iter().filter(|s| now <= s.end_time).map(row);
        let result = self
            .seasons
            .iter()
            .filter(|s| now > s.end_time && now <= s.result_expiration_time)
            .map(row);
        ::packet::fragments::compose_fragments(
            game_cmd::HELLO_ACK,
            SEASON_LIST_FORMAT,
            &[
                Value::Repeat(active.collect()),
                Value::Repeat(result.collect()),
            ],
        )
        .ok()
    }

    fn season(&self, kind: u8, schedule: i32) -> Option<&Season> {
        self.seasons
            .iter()
            .find(|s| s.season_contents_type == kind && s.schedule_number == schedule)
    }

    /// InsertPlayerRatingScoreReq @0x14000B2F0 (`hqQqhQiQqcisqcqss`).
    fn insert_rating(&mut self, me: Peer, frame: &[u8], _now: i64, out: &mut Vec<Out>) -> Flow {
        let Some(v) = fields("hqQqhQiQqcisqcqss", frame) else {
            return Flow::Close;
        };
        let (request_id, player_type, player_id, world_id) =
            (u64_at(&v, 1), i8_at(&v, 2), u64_at(&v, 3), i16_at(&v, 4));
        let (is_win, request_time, kind, schedule, job) = (
            i8_at(&v, 7),
            u64_at(&v, 8),
            u8_at(&v, 9),
            i32_at(&v, 10),
            u8_at(&v, 13),
        );
        let valid = request_id != 0
            && request_time != 0
            && (1..16).contains(&kind)
            && schedule != 0
            && (1..4).contains(&player_type)
            && player_id != 0
            && is_win <= 1
            && world_id != 0
            && (1..15).contains(&job);
        let code = if !valid {
            job_reason::PARAMETER
        } else if self.season(kind, schedule).is_none() {
            job_reason::NO_BOARD
        } else {
            // No RankingDB in this port yet ❓.
            job_reason::DB
        };
        out.push(Out::Send(
            me,
            packet(
                "hqQqhQciE",
                &[
                    Value::I16(game_cmd::INSERT_PLAYER_RATING_SCORE_FAIL),
                    v[1].clone(),
                    v[2].clone(),
                    v[3].clone(),
                    v[4].clone(),
                    v[5].clone(),
                    v[9].clone(),
                    v[10].clone(),
                    Value::I16(code),
                ],
            ),
        ));
        Flow::Keep
    }

    /// InsertRankingScoreReq @0x14000B4B0 (`hqqhQqqqciscqss`).
    fn insert_ranking(&mut self, me: Peer, frame: &[u8], _now: i64, out: &mut Vec<Out>) -> Flow {
        let Some(v) = fields("hqqhQqqqciscqss", frame) else {
            return Flow::Close;
        };
        let (request_id, player_id, world_id, request_time) =
            (u64_at(&v, 1), u64_at(&v, 2), i16_at(&v, 3), u64_at(&v, 7));
        let (kind, schedule, job) = (u8_at(&v, 8), i32_at(&v, 9), u8_at(&v, 11));
        let valid = request_id != 0
            && request_time != 0
            && kind.wrapping_sub(1) <= 14
            && schedule != 0
            && player_id != 0
            && world_id != 0
            && job.wrapping_sub(1) <= 13;
        let code = if !valid {
            job_reason::PARAMETER
        } else if self.season(kind, schedule).is_none() {
            job_reason::NO_BOARD
        } else {
            job_reason::DB
        };
        out.push(Out::Send(
            me,
            packet(
                "hciqqhQE",
                &[
                    Value::I16(game_cmd::INSERT_RANKING_SCORE_FAIL),
                    v[8].clone(),
                    v[9].clone(),
                    v[1].clone(),
                    v[2].clone(),
                    v[3].clone(),
                    v[4].clone(),
                    Value::I16(code),
                ],
            ),
        ));
        Flow::Keep
    }

    // ---- infogate port -------------------------------------------------

    /// InfoGateChannelMgr::PacketHandler @0x14000D300.
    fn infogate(
        &mut self,
        id: u64,
        ip: Ipv4Addr,
        cmd: i16,
        frame: &[u8],
        now: i64,
        out: &mut Vec<Out>,
    ) -> Flow {
        use infogate_cmd::*;
        let me = Peer::InfoGate(id);
        if !matches!(
            cmd,
            HELLO_REQ
                | RETRIEVE_RANKING_LIST_REQ
                | RETRIEVE_TIME_ATTACK_BEST_RECORD_REQ
                | RETRIEVE_CHALLENGE_PARTY_LOG_SUMMARY_REQ
        ) {
            return Flow::Close;
        }
        if !self.infogate_sessions.contains_key(&id) {
            return if cmd == HELLO_REQ {
                self.infogate_hello(id, ip, frame, now, out)
            } else {
                Flow::Close
            };
        }
        match cmd {
            RETRIEVE_RANKING_LIST_REQ => {
                let Some(v) = fields("hcc", frame) else {
                    return Flow::Close;
                };
                // No cached ranking list packet yet ❓.
                out.push(Out::Send(
                    me,
                    packet(
                        "hccE",
                        &[
                            Value::I16(RETRIEVE_RANKING_LIST_FAIL),
                            v[1].clone(),
                            v[2].clone(),
                            Value::I16(job_reason::NO_BOARD),
                        ],
                    ),
                ));
                Flow::Keep
            }
            RETRIEVE_TIME_ATTACK_BEST_RECORD_REQ => {
                let Some(v) = fields("hocciq", frame) else {
                    return Flow::Close;
                };
                out.push(Out::Send(
                    me,
                    packet(
                        "hoE",
                        &[
                            Value::I16(RETRIEVE_TIME_ATTACK_BEST_RECORD_FAIL),
                            v[1].clone(),
                            Value::I16(job_reason::NO_BOARD),
                        ],
                    ),
                ));
                Flow::Keep
            }
            RETRIEVE_CHALLENGE_PARTY_LOG_SUMMARY_REQ => {
                let Some(v) = fields("hocq", frame) else {
                    return Flow::Close;
                };
                out.push(Out::Send(
                    me,
                    packet(
                        "hoE",
                        &[
                            Value::I16(RETRIEVE_CHALLENGE_PARTY_LOG_SUMMARY_FAIL),
                            v[1].clone(),
                            Value::I16(job_reason::NO_BOARD),
                        ],
                    ),
                ));
                Flow::Keep
            }
            // HELLO_REQ while authed returns 0.
            _ => Flow::Close,
        }
    }

    /// HandleRankingInfoGate_HelloReq @0x14000DF60 (`hhhh`).
    fn infogate_hello(
        &mut self,
        id: u64,
        ip: Ipv4Addr,
        frame: &[u8],
        now: i64,
        out: &mut Vec<Out>,
    ) -> Flow {
        let me = Peer::InfoGate(id);
        let fail = |out: &mut Vec<Out>, code| {
            out.push(Out::Send(me, hh(infogate_cmd::HELLO_FAIL, code)));
            Flow::Close
        };
        let Some(v) = fields("hhhh", frame) else {
            return fail(out, reason::SYSTEM);
        };
        if i16_at(&v, 1) != protocol::INFOGATE_PACKET_VERSION {
            return fail(out, reason::VERSION);
        }
        if !self.service_available {
            return fail(out, reason::SYSTEM);
        }
        let (wid, internal_port) = (i16_at(&v, 2), i16_at(&v, 3) as u16);
        let Some(channel) = self.infogates.get_mut(&wid) else {
            return fail(out, reason::SYSTEM);
        };
        if channel.info.ip != ip {
            return fail(out, reason::IP);
        }
        if channel.info.port != internal_port {
            return fail(out, reason::PORT);
        }
        if let Some(old) = channel.session.take() {
            self.infogate_sessions.remove(&old);
            out.push(Out::Close(Peer::InfoGate(old)));
            return fail(out, reason::DUPLICATE);
        }
        channel.session = Some(id);
        self.infogate_sessions.insert(id, wid);
        let rows = self
            .active_seasons(now)
            .map(|s| {
                vec![
                    Value::U8(s.season_contents_type),
                    Value::U8(max_ranking_list_index(s.season_contents_type) + 1),
                ]
            })
            .collect();
        out.push(Out::Send(
            me,
            packet(
                CURRENT_SEASON_FORMAT,
                &[Value::I16(infogate_cmd::HELLO_ACK), Value::Repeat(rows)],
            ),
        ));
        Flow::Keep
    }

    // ---- manage port ---------------------------------------------------

    /// ManageSessionHandler::PacketHandler @0x140016820 (cmds 1..9; 4 and 7
    /// need a ManageChannel).
    fn manage(&mut self, id: u64, cmd: i16, frame: &[u8], now: i64, out: &mut Vec<Out>) -> Flow {
        use manage_cmd::*;
        let me = Peer::Manage(id);
        match cmd {
            HELLO_REQ => {
                // HandleRankingManage_HelloReq @0x1400335E0 (`hhh`).
                let Some(v) = fields("hhh", frame) else {
                    out.push(Out::Send(me, hh(HELLO_FAIL, reason::MANAGE_DECOMPOSE)));
                    return Flow::Close;
                };
                if i16_at(&v, 2) != protocol::MANAGE_PACKET_VERSION {
                    out.push(Out::Send(me, hh(HELLO_FAIL, reason::MANAGE_VERSION)));
                    return Flow::Close;
                }
                self.managers.insert(id);
                out.push(Out::Send(me, packet("h", &[Value::I16(HELLO_ACK)])));
                Flow::Keep
            }
            DELETE_BLACK_LIST_REQ | INSERT_BLACK_LIST_REQ => {
                if !self.managers.contains(&id) {
                    return Flow::Close;
                }
                let Some(v) = fields("hqqQ", frame) else {
                    return Flow::Close;
                };
                let (tid, player_id, player_type) = (u64_at(&v, 1), u64_at(&v, 2), i8_at(&v, 3));
                let insert = cmd == INSERT_BLACK_LIST_REQ;
                let (ok, fail) = if insert {
                    (INSERT_BLACK_LIST_OK, INSERT_BLACK_LIST_FAIL)
                } else {
                    (DELETE_BLACK_LIST_OK, DELETE_BLACK_LIST_FAIL)
                };
                let result = if tid == 0 || player_id == 0 || !(1..3).contains(&player_type) {
                    Err(job_reason::DEFAULT)
                } else if insert {
                    if !self.black_list.insert((player_type, player_id)) {
                        Err(job_reason::ALREADY_IN_BLACK_LIST)
                    } else if self.active_seasons(now).next().is_none() {
                        Err(job_reason::NO_BOARD)
                    } else {
                        Ok(())
                    }
                } else if self.black_list.remove(&(player_type, player_id)) {
                    Ok(())
                } else {
                    Err(job_reason::NOT_IN_BLACK_LIST)
                };
                let reply = match result {
                    Ok(()) => packet("hq", &[Value::I16(ok), Value::U64(tid)]),
                    Err(code) => packet(
                        "hqE",
                        &[Value::I16(fail), Value::U64(tid), Value::I16(code)],
                    ),
                };
                out.push(Out::Send(me, reply));
                Flow::Keep
            }
            // Empty slots and commands outside 1..9.
            _ => Flow::Close,
        }
    }
}
