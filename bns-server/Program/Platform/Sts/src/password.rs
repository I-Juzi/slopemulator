//! Password-path orchestration for the 2020 SRP profile in section 07.
//! The backend trait remains available for explicit trusted integrations.
use crate::tokens::TokenAuthority;
use rand::{CryptoRng, RngCore};
use std::{collections::HashMap, sync::Arc, time::Instant};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Binding {
    pub program_id: u32,
    pub game_code: String,
}
impl Binding {
    pub fn valid(&self) -> bool {
        self.program_id != 0
            && !self.game_code.is_empty()
            && self.game_code.len() <= 8
            && self.game_code.bytes().all(|b| b.is_ascii_alphanumeric())
    }
}

/// Generated/local identity. No Debug: account data must not enter logs.
#[derive(Clone)]
pub struct Identity {
    pub portal_id: [u8; 16],
    pub account_id: [u8; 16],
    pub login_name: String,
    pub user_name: String,
    pub slot_id: u64,
}
impl Identity {
    pub(crate) fn valid(&self) -> bool {
        self.portal_id != [0; 16]
            && self.account_id != [0; 16]
            && !self.login_name.is_empty()
            && self.login_name.len() <= 256
            && !self.user_name.is_empty()
            && self.user_name.len() <= 256
            && self.slot_id != 0
    }
}

#[derive(Clone)]
pub struct Grant {
    pub binding: Binding,
    pub game_alias: String,
}

/// The selected backend must define and validate the named verifier scheme.
/// No Debug: verifier records are sensitive account data.
#[derive(Clone)]
pub struct VerifierRecord {
    pub scheme: String,
    pub salt: Vec<u8>,
    pub verifier: Vec<u8>,
}
#[derive(Clone)]
pub struct Account {
    identity: Identity,
    verifier: VerifierRecord,
    grants: Vec<Grant>,
}
impl Account {
    /// Import a verifier, never a plaintext password. Backend validation is required.
    pub fn from_verifier(
        identity: Identity,
        verifier: VerifierRecord,
        grants: Vec<Grant>,
    ) -> Result<Self, LoginError> {
        if !identity.valid()
            || verifier.scheme.is_empty()
            || verifier.scheme.len() > 128
            || verifier.salt.is_empty()
            || verifier.salt.len() > 1024
            || verifier.verifier.is_empty()
            || verifier.verifier.len() > 4096
            || grants.is_empty()
            || grants
                .iter()
                .any(|g| !g.binding.valid() || g.game_alias.is_empty() || g.game_alias.len() > 64)
        {
            return Err(LoginError::InvalidInput);
        }
        Ok(Self {
            identity,
            verifier,
            grants,
        })
    }
}

/// Trusted extension point, not a peer-provided authentication result.
/// Validate the scheme, verifier, public value and proof. Never log these data.
/// The bundled SRP backend implements section 07. It is not independently audited.
pub trait PasswordBackend: Send + Sync {
    fn start(
        &self,
        login_name: &str,
        record: &VerifierRecord,
        entropy: &[u8; 64],
    ) -> Result<Challenge, LoginError>;
}
pub struct Challenge {
    pub public: Vec<u8>,
    pub exchange: Box<dyn PasswordExchange>,
}
pub struct ExchangeResult {
    pub proof: Vec<u8>,
    pub key: Option<[u8; 64]>,
}
pub trait PasswordExchange: Send {
    fn verify(self: Box<Self>, public: &[u8], proof: &[u8]) -> Result<Vec<u8>, LoginError>;
    /// Legacy injected backends can verify proofs but cannot open an encrypted wire session.
    fn verify_with_key(
        self: Box<Self>,
        public: &[u8],
        proof: &[u8],
    ) -> Result<ExchangeResult, LoginError> {
        self.verify(public, proof)
            .map(|proof| ExchangeResult { proof, key: None })
    }
}
pub struct SrpPasswordBackend;
impl PasswordBackend for SrpPasswordBackend {
    fn start(
        &self,
        login: &str,
        record: &VerifierRecord,
        entropy: &[u8; 64],
    ) -> Result<Challenge, LoginError> {
        if record.scheme != crate::srp6::SCHEME {
            return Err(LoginError::Unavailable);
        }
        let random: &[u8; 32] = entropy[..32]
            .try_into()
            .map_err(|_| LoginError::Unavailable)?;
        let exchange =
            crate::srp6::ServerExchange::start(login, &record.salt, &record.verifier, random)
                .map_err(|_| LoginError::Authentication)?;
        Ok(Challenge {
            public: exchange.public().to_vec(),
            exchange: Box::new(exchange),
        })
    }
}
impl PasswordExchange for crate::srp6::ServerExchange {
    fn verify(self: Box<Self>, public: &[u8], proof: &[u8]) -> Result<Vec<u8>, LoginError> {
        crate::srp6::ServerExchange::verify(*self, public, proof)
            .map(|result| result.proof.to_vec())
            .map_err(|_| LoginError::Authentication)
    }
    fn verify_with_key(
        self: Box<Self>,
        public: &[u8],
        proof: &[u8],
    ) -> Result<ExchangeResult, LoginError> {
        crate::srp6::ServerExchange::verify(*self, public, proof)
            .map(|result| ExchangeResult {
                proof: result.proof.to_vec(),
                key: Some(result.key),
            })
            .map_err(|_| LoginError::Authentication)
    }
}
impl VerifierRecord {
    pub fn enroll<R: RngCore + CryptoRng>(login: &str, password: &[u8], rng: &mut R) -> Self {
        let record = crate::srp6::enroll(login, password, rng);
        Self {
            scheme: crate::srp6::SCHEME.into(),
            salt: record.salt,
            verifier: record.verifier,
        }
    }
}
/// Explicit fail-closed backend for callers that disable password login.
pub struct UnavailablePasswordBackend;
impl PasswordBackend for UnavailablePasswordBackend {
    fn start(&self, _: &str, _: &VerifierRecord, _: &[u8; 64]) -> Result<Challenge, LoginError> {
        Err(LoginError::Unavailable)
    }
}

pub trait AccountStore {
    fn find(&self, login_name: &str) -> Option<Account>;
}
#[derive(Default)]
pub struct MemoryAccounts(HashMap<String, Account>);
impl MemoryAccounts {
    /// Do not overwrite an existing account during enrollment.
    pub fn insert(&mut self, account: Account) -> Result<(), LoginError> {
        match self.0.entry(account.identity.login_name.clone()) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(account);
                Ok(())
            }
            std::collections::hash_map::Entry::Occupied(_) => Err(LoginError::InvalidInput),
        }
    }
}
impl AccountStore for MemoryAccounts {
    fn find(&self, login_name: &str) -> Option<Account> {
        self.0.get(login_name).cloned()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoginError {
    WrongOrder,
    InvalidInput,
    Authentication,
    Forbidden,
    Unavailable,
}
impl std::fmt::Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "local STS login error: {self:?}")
    }
}
impl std::error::Error for LoginError {}

/// No password field. The selected backend defines and verifies the proof.
pub enum Request {
    Connect(Binding),
    LoginStart { login_name: String },
    KeyData { public: Vec<u8>, proof: Vec<u8> },
    LoginFinish { language: u32 },
    RequestGameToken { game_alias: String },
    Logout,
}
/// No Debug: challenges, proofs and bearer tokens must not enter logs.
pub enum Reply {
    Connected,
    LoginStart {
        salt: Vec<u8>,
        public: Vec<u8>,
    },
    KeyData {
        proof: Vec<u8>,
    },
    LoginFinish {
        portal_id: [u8; 16],
        role_ids: Vec<u32>,
        auth_type: u32,
        status: u32,
        cafe_code: Option<String>,
    },
    GameToken {
        token: [u8; 16],
    },
    LoggedOut,
}
/// Shared PDB API record only. No local issuer or wire serializer is supplied.
pub struct RequestToken {
    pub app_id: String,
}
pub struct TokenReply {
    pub authentication_token: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    New,
    Connected,
    AwaitKeyData,
    AwaitFinish,
    Authenticated,
    Closed,
}
struct Pending {
    account: Account,
    exchange: Box<dyn PasswordExchange>,
}

/// Pure protocol logic: no sockets, clock reads or hidden random source.
/// The caller supplies a cryptographic RNG and monotonic time.
pub struct Session {
    state: State,
    binding: Binding,
    pending: Option<Pending>,
    account: Option<Account>,
    authority: Arc<TokenAuthority>,
    owner: [u8; 16],
    backend: Arc<dyn PasswordBackend>,
    session_key: Option<[u8; 64]>,
}
impl Session {
    pub fn new<R: RngCore + CryptoRng>(
        binding: Binding,
        authority: Arc<TokenAuthority>,
        rng: &mut R,
    ) -> Result<Self, LoginError> {
        Self::with_backend(binding, authority, rng, Arc::new(SrpPasswordBackend))
    }
    /// Explicit trusted backend injection. Wire sessions require a session key.
    pub fn with_backend<R: RngCore + CryptoRng>(
        binding: Binding,
        authority: Arc<TokenAuthority>,
        rng: &mut R,
        backend: Arc<dyn PasswordBackend>,
    ) -> Result<Self, LoginError> {
        if !binding.valid() {
            return Err(LoginError::InvalidInput);
        }
        let mut owner = [0; 16];
        rng.fill_bytes(&mut owner);
        Ok(Self {
            state: State::New,
            binding,
            pending: None,
            account: None,
            authority,
            owner,
            backend,
            session_key: None,
        })
    }
    pub fn state(&self) -> State {
        self.state
    }
    /// Move K to the transport exactly once after the KeyData reply.
    pub fn take_session_key(&mut self) -> Option<[u8; 64]> {
        self.session_key.take()
    }
    pub fn identity(&self) -> Option<&Identity> {
        if self.state == State::Authenticated {
            self.account.as_ref().map(|a| &a.identity)
        } else {
            None
        }
    }
    pub fn binding(&self) -> &Binding {
        &self.binding
    }
    /// Game aliases granted for this binding. Only after authentication.
    pub fn game_aliases(&self) -> Option<Vec<String>> {
        if self.state != State::Authenticated {
            return None;
        }
        let account = self.account.as_ref()?;
        Some(
            account
                .grants
                .iter()
                .filter(|g| g.binding == self.binding)
                .map(|g| g.game_alias.clone())
                .collect(),
        )
    }
    /// Token login (spec 09): the launcher token replaces LoginStart/KeyData.
    /// The account must match the token identity and hold a grant for this binding.
    pub(crate) fn accept_token_login(
        &mut self,
        identity: &Identity,
        store: &impl AccountStore,
    ) -> Result<(), LoginError> {
        if self.state != State::Connected {
            return Err(LoginError::WrongOrder);
        }
        let account = store
            .find(&identity.login_name)
            .ok_or(LoginError::Authentication)?;
        if account.identity.account_id != identity.account_id
            || account.identity.portal_id != identity.portal_id
            || !account.grants.iter().any(|g| g.binding == self.binding)
        {
            return Err(LoginError::Authentication);
        }
        self.account = Some(account);
        self.state = State::AwaitFinish;
        Ok(())
    }
    /// Close on framing, XML, transport or authorization errors too.
    pub fn abort(&mut self) {
        self.close();
    }
    pub fn handle<R: RngCore + CryptoRng>(
        &mut self,
        request: Request,
        store: &impl AccountStore,
        rng: &mut R,
        now: Instant,
    ) -> Result<Reply, LoginError> {
        let result = self.step(request, store, rng, now);
        if result.is_err() {
            self.close();
        }
        result
    }
    fn step<R: RngCore + CryptoRng>(
        &mut self,
        request: Request,
        store: &impl AccountStore,
        rng: &mut R,
        now: Instant,
    ) -> Result<Reply, LoginError> {
        match (self.state, request) {
            (state, Request::Logout) if state != State::Closed => {
                self.close();
                Ok(Reply::LoggedOut)
            }
            (State::New, Request::Connect(binding)) => {
                if binding != self.binding {
                    return Err(LoginError::Forbidden);
                }
                self.state = State::Connected;
                Ok(Reply::Connected)
            }
            (State::Connected, Request::LoginStart { login_name }) => {
                if login_name.is_empty() || login_name.len() > 256 {
                    return Err(LoginError::InvalidInput);
                }
                let account = store.find(&login_name).ok_or(LoginError::Authentication)?;
                if !account.grants.iter().any(|g| g.binding == self.binding) {
                    return Err(LoginError::Authentication);
                }
                let mut secret = [0; 64];
                rng.fill_bytes(&mut secret);
                let challenge = self
                    .backend
                    .start(&login_name, &account.verifier, &secret)?;
                if challenge.public.is_empty() || challenge.public.len() > 4096 {
                    return Err(LoginError::Unavailable);
                }
                let salt = account.verifier.salt.clone();
                let public = challenge.public;
                self.pending = Some(Pending {
                    account,
                    exchange: challenge.exchange,
                });
                self.state = State::AwaitKeyData;
                Ok(Reply::LoginStart { salt, public })
            }
            (State::AwaitKeyData, Request::KeyData { public, proof }) => {
                // Local envelope limits. The backend must validate exact sizes.
                if public.is_empty()
                    || public.len() > 4096
                    || proof.is_empty()
                    || proof.len() > 1024
                {
                    return Err(LoginError::Authentication);
                }
                let pending = self.pending.take().ok_or(LoginError::WrongOrder)?;
                let verified = pending
                    .exchange
                    .verify_with_key(&public, &proof)
                    .map_err(|_| LoginError::Authentication)?;
                if verified.proof.is_empty() || verified.proof.len() > 1024 {
                    return Err(LoginError::Unavailable);
                }
                self.session_key = verified.key;
                self.account = Some(pending.account);
                self.state = State::AwaitFinish;
                Ok(Reply::KeyData {
                    proof: verified.proof,
                })
            }
            (State::AwaitFinish, Request::LoginFinish { language }) => {
                if language > 10 {
                    return Err(LoginError::InvalidInput);
                }
                let account = self.account.as_ref().ok_or(LoginError::WrongOrder)?;
                self.state = State::Authenticated;
                Ok(Reply::LoginFinish {
                    portal_id: account.identity.portal_id,
                    role_ids: vec![],
                    auth_type: 0,
                    status: 0,
                    cafe_code: None,
                })
            }
            (State::Authenticated, Request::RequestGameToken { game_alias }) => {
                let account = self.account.as_ref().ok_or(LoginError::WrongOrder)?;
                if !account
                    .grants
                    .iter()
                    .any(|g| g.binding == self.binding && g.game_alias == game_alias)
                {
                    return Err(LoginError::Forbidden);
                }
                let token = self.authority.issue(
                    account.identity.clone(),
                    self.binding.clone(),
                    self.owner,
                    rng,
                    now,
                )?;
                Ok(Reply::GameToken { token })
            }
            _ => Err(LoginError::WrongOrder),
        }
    }
    fn close(&mut self) {
        self.pending = None;
        self.account = None;
        self.session_key = None;
        self.state = State::Closed;
        self.authority.revoke(self.owner);
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.close();
    }
}
