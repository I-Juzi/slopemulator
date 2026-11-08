//! Bounded stream owner for section 07. The caller owns the TCP connection.
//! Start with an explicitly connected password Session. The client Connect body
//! is not specified in section 07. Do not reuse the game-server Connect schema.
use crate::{
    messages::client::{self, LoginInfo, UserInfo},
    password::{self, AccountStore, Identity, LoginError, State},
    rc4::Rc4,
    Codec, Frame, Transaction,
};
use rand::{CryptoRng, RngCore};
use std::{sync::Arc, time::Instant};

// RC4 covers the whole message stream (request line, headers and body) in both
// directions, per 07-client-login-wire.md (confirmed in StsMsgCli64: send
// sub_18001C300/sub_180018780, receive sub_1800196A0). There is no body-only mode.
/// An explicit local app-token backend must check the AppId grant itself.
/// Authentication-token strings remain distinct from single-use game GUIDs.
pub trait AppTokenIssuer: Send + Sync {
    fn issue(&self, identity: &Identity, app_id: &str, now: Instant) -> Result<String, LoginError>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Framing,
    Xml,
    Login(LoginError),
    Closed,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "STS wire error: {self:?}")
    }
}
impl std::error::Error for Error {}

pub struct Session {
    login: password::Session,
    codec: Codec,
    send: Option<Rc4>,
    receive: Option<Rc4>,
    staged: Vec<u8>,
    total: Option<usize>,
    user_center: String,
    app_tokens: Option<Arc<dyn AppTokenIssuer>>,
}
impl Session {
    pub fn new(login: password::Session, user_center: String) -> Result<Self, Error> {
        if login.state() != State::Connected || user_center.len() > 256 {
            return Err(Error::Login(LoginError::InvalidInput));
        }
        Ok(Self {
            login,
            codec: Codec {
                max_header_bytes: 32_767,
                max_body_bytes: 65_536,
            },
            send: None,
            receive: None,
            staged: Vec::new(),
            total: None,
            user_center,
            app_tokens: None,
        })
    }
    pub fn with_app_tokens(mut self, issuer: Arc<dyn AppTokenIssuer>) -> Self {
        self.app_tokens = Some(issuer);
        self
    }
    pub fn state(&self) -> State {
        self.login.state()
    }
    /// Accept fragmented or coalesced bytes. Each output is one complete reply.
    /// 3102 is plaintext. The next byte in either direction uses a fresh RC4
    /// state keyed directly with K. State persists across all later frames.
    /// The caller must write replies in order. Close the stream after any error.
    pub fn receive<R: RngCore + CryptoRng>(
        &mut self,
        input: &[u8],
        store: &impl AccountStore,
        rng: &mut R,
        now: Instant,
    ) -> Result<Vec<Vec<u8>>, Error> {
        let result = self.read(input, store, rng, now);
        if result.is_err() {
            self.abort();
        }
        result
    }
    pub fn abort(&mut self) {
        self.login.abort();
        self.send = None;
        self.receive = None;
        self.staged.clear();
        self.total = None;
    }
    fn read<R: RngCore + CryptoRng>(
        &mut self,
        mut input: &[u8],
        store: &impl AccountStore,
        rng: &mut R,
        now: Instant,
    ) -> Result<Vec<Vec<u8>>, Error> {
        if self.state() == State::Closed {
            return Err(Error::Closed);
        }
        let mut replies = Vec::new();
        while !input.is_empty() {
            if self.total.is_none() {
                let mut byte = [input[0]];
                input = &input[1..];
                if let Some(cipher) = &mut self.receive {
                    cipher.apply(&mut byte);
                }
                self.staged.push(byte[0]);
                if self.staged.len() > self.codec.max_header_bytes {
                    return Err(Error::Framing);
                }
                if !self.staged.ends_with(b"\r\n\r\n") {
                    continue;
                }
                // Validate all headers before trusting l or reserving the body.
                self.codec
                    .decode(&self.staged)
                    .map_err(|_| Error::Framing)?;
                let head = std::str::from_utf8(&self.staged).map_err(|_| Error::Framing)?;
                let length = head
                    .split("\r\n")
                    .skip(1)
                    .filter_map(|line| line.split_once(':'))
                    .find(|(name, _)| name.eq_ignore_ascii_case("l"))
                    .ok_or(Error::Framing)?
                    .1
                    .parse::<usize>()
                    .map_err(|_| Error::Framing)?;
                self.total = Some(
                    self.staged
                        .len()
                        .checked_add(length)
                        .ok_or(Error::Framing)?,
                );
            }
            let total = self.total.ok_or(Error::Framing)?;
            let take = (total - self.staged.len()).min(input.len());
            let offset = self.staged.len();
            self.staged.extend_from_slice(&input[..take]);
            input = &input[take..];
            // Both scope choices encrypt all body bytes.
            if let Some(cipher) = &mut self.receive {
                cipher.apply(&mut self.staged[offset..]);
            }
            if self.staged.len() != total {
                continue;
            }
            let (frame, used) = self
                .codec
                .decode(&self.staged)
                .map_err(|_| Error::Framing)?
                .ok_or(Error::Framing)?;
            if used != total {
                return Err(Error::Framing);
            }
            self.staged.clear();
            self.total = None;
            let transaction = frame
                .headers
                .iter()
                .find(|h| h.name == "s")
                .map(|h| Transaction::parse(&h.value).map_err(|_| Error::Framing))
                .transpose()?;
            if transaction.is_some_and(|t| t.complete.is_some() || t.sequence != 0) {
                return Err(Error::Framing);
            }
            // Sts/Ping is one-way keepalive. StsMsgCli32.dll (0.122) has no
            // reply parser for 1002 (reply dispatch sub_100231E0 returns 9),
            // so send nothing and keep the session.
            if frame.start
                == (crate::StartLine::Request {
                    protocol: "Sts".into(),
                    command: "Ping".into(),
                })
            {
                if !frame.body.is_empty() {
                    return Err(Error::Framing);
                }
                continue;
            }
            let request = client::Request::from_frame(&frame).map_err(|_| Error::Xml)?;
            let reply = self.handle(request, store, rng, now)?;
            let frame = reply
                .frame(transaction.map(|t| t.id))
                .map_err(|_| Error::Xml)?;
            let bytes = self.encode(&frame)?;
            replies.push(bytes);
            if let Some(key) = self.login.take_session_key() {
                if self.send.is_some() {
                    return Err(Error::Login(LoginError::WrongOrder));
                }
                self.send = Rc4::new(&key);
                self.receive = Rc4::new(&key);
            } else if self.state() == State::AwaitFinish && self.send.is_none() {
                // The exchange completed without a session key. 07 requires RC4
                // both ways after 3102; continuing would leave the session in
                // plaintext. Close it instead.
                return Err(Error::Login(LoginError::Unavailable));
            }
            if self.state() == State::Closed {
                if !input.is_empty() {
                    return Err(Error::Closed);
                }
                break;
            }
        }
        Ok(replies)
    }
    fn encode(&mut self, frame: &Frame) -> Result<Vec<u8>, Error> {
        let mut bytes = self.codec.encode(frame).map_err(|_| Error::Framing)?;
        if let Some(cipher) = &mut self.send {
            cipher.apply(&mut bytes);
        }
        Ok(bytes)
    }
    fn handle<R: RngCore + CryptoRng>(
        &mut self,
        request: client::Request,
        store: &impl AccountStore,
        rng: &mut R,
        now: Instant,
    ) -> Result<client::Reply, Error> {
        use client::Request as W;
        use password::{Reply as P, Request as Q};
        let semantic = match request {
            W::LoginStart { login_name } => Q::LoginStart { login_name },
            W::KeyData { public, proof } => Q::KeyData { public, proof },
            W::LoginFinish { language } => Q::LoginFinish { language },
            W::RequestGameToken {
                game_code,
                account_alias,
            } => {
                if game_code != self.login.binding().game_code {
                    return Err(Error::Login(LoginError::Forbidden));
                }
                Q::RequestGameToken {
                    game_alias: account_alias,
                }
            }
            W::LogoutMyClient => Q::Logout,
            W::GetMyUserInfo => {
                let id = self
                    .login
                    .identity()
                    .ok_or(Error::Login(LoginError::WrongOrder))?;
                return Ok(client::Reply::UserInfo(UserInfo {
                    user_id: id.portal_id,
                    user_center: self.user_center.clone(),
                    user_name: id.user_name.clone(),
                    login_name: id.login_name.clone(),
                    external_account: Vec::new(),
                }));
            }
            W::RequestToken { app_id } => {
                let identity = self
                    .login
                    .identity()
                    .ok_or(Error::Login(LoginError::WrongOrder))?;
                if app_id.is_empty() || app_id.len() > 256 {
                    return Err(Error::Login(LoginError::InvalidInput));
                }
                let token = self
                    .app_tokens
                    .as_ref()
                    .ok_or(Error::Login(LoginError::Unavailable))?
                    .issue(identity, &app_id, now)
                    .map_err(Error::Login)?;
                if token.is_empty() || token.len() > 4096 {
                    return Err(Error::Login(LoginError::Unavailable));
                }
                return Ok(client::Reply::Token {
                    authentication_token: token,
                });
            }
        };
        Ok(
            match self
                .login
                .handle(semantic, store, rng, now)
                .map_err(Error::Login)?
            {
                P::LoginStart { salt, public } => client::Reply::LoginStart { salt, public },
                P::KeyData { proof } => client::Reply::KeyData {
                    proof: proof
                        .try_into()
                        .map_err(|_| Error::Login(LoginError::Unavailable))?,
                },
                P::LoginFinish {
                    portal_id,
                    role_ids,
                    auth_type,
                    status,
                    cafe_code,
                } => client::Reply::LoginFinish(LoginInfo {
                    user_id: Some(portal_id),
                    user_center: Some(self.user_center.clone()),
                    roles: role_ids
                        .into_iter()
                        .map(|r| i32::try_from(r).map_err(|_| Error::Xml))
                        .collect::<Result<_, _>>()?,
                    auth_type: Some(i32::try_from(auth_type).map_err(|_| Error::Xml)?),
                    status: Some(i32::try_from(status).map_err(|_| Error::Xml)?),
                    pc_cafe_code: cafe_code,
                }),
                P::GameToken { token } => client::Reply::GameToken { token },
                P::LoggedOut => client::Reply::LoggedOut,
                P::Connected => return Err(Error::Login(LoginError::WrongOrder)),
            },
        )
    }
}
