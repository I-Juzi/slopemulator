//! Launcher token login (`LoginTokenStart` / `TokenKeyData`), spec
//! Docs/Platform/Sts/09-token-login-server.md. Launcher-only; SRP (07) stays the default.
//! Built from static analysis of `StsMsgCli64.dll`. No real handshake is captured yet:
//! every ❓ below must be confirmed against one real launcher login.
//! BigUint arithmetic is variable-time. Use only for the local service.
use crate::{
    messages::client::{field, parse_xml, Error as XmlError},
    password::{self, AccountStore, Binding, LoginError, State},
    rc4::Rc4,
    srp6::{number_bytes, read_number},
    tokens::TokenAuthority,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use md5::Md5;
use num_bigint::BigUint;
use num_traits::Zero;
use rand::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};
use std::{sync::Arc, time::Instant};

/// Library reply ID of the `LoginTokenStart` reply.
pub const TOKEN_START_REPLY_ID: u32 = 3111;
/// ❓ Reply ID of the `TokenKeyData` reply. 3112 is the best guess from the ID table.
pub const TOKEN_KEY_DATA_REPLY_ID: u32 = 3112;
/// Suite 4 sizes (`unk_180056928`): MAC 16, cipher key 128, IV 0.
pub const MAC_LEN: usize = 16;
pub const CIPHER_KEY_LEN: usize = 128;
pub const MASTER_LEN: usize = 48;
pub const KEY_BLOCK_LEN: usize = 2 * MAC_LEN + 2 * CIPHER_KEY_LEN;
/// ❓ ServerRand length. TLS uses 28 bytes.
pub const SERVER_RAND_LEN: usize = 28;
/// ❓ Premaster length. TLS uses 48 bytes. The decrypted number is zero-padded to it.
pub const PREMASTER_LEN: usize = 48;
/// Ephemeral RSA size. The CKeyRsa blob carries a 256-byte modulus.
pub const RSA_BITS: usize = 2048;
pub const RSA_MODULUS_LEN: usize = RSA_BITS / 8;
/// `[u32 type][u32 e][u32 modlen][modulus LE]`.
pub const PUBLIC_BLOB_LEN: usize = 12 + RSA_MODULUS_LEN;
const BLOB_TYPE: u32 = 4;
const RSA_E: u32 = 65_537;
/// Local envelope limit for ClientRand. ❓ Real length unknown.
const MAX_CLIENT_RAND: usize = 64;

/// One-shot MD5 (`sub_18002DFF0`, classic IV).
pub fn md5(parts: &[&[u8]]) -> [u8; 16] {
    let mut h = Md5::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}
fn sha256(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

/// ❓ PRF label of block `i`: `i+1` copies of `'A' + (i mod 26) + 1` ("B", "CC", "DDD", ...),
/// read from the table at `off_18006C000`. Confirm with a capture.
fn label(i: usize) -> Vec<u8> {
    vec![b'A' + (i % 26) as u8 + 1; i + 1]
}

/// PRF primitive `sub_180001040`: 16 bytes per block,
/// `MD5(secret ‖ SHA256(label_i ‖ secret ‖ rand_a ‖ rand_b))`.
pub fn prf(secret: &[u8], rand_a: &[u8], rand_b: &[u8], out_len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(out_len.next_multiple_of(16));
    let mut i = 0;
    while out.len() < out_len {
        let inner = sha256(&[&label(i), secret, rand_a, rand_b]);
        out.extend_from_slice(&md5(&[secret, &inner]));
        i += 1;
    }
    out.truncate(out_len);
    out
}

/// The 288-byte suite-4 key block. No Debug: these are session keys.
pub struct KeyBlock {
    /// Derived but not used on the message path (no per-message MAC).
    pub client_mac: [u8; MAC_LEN],
    pub server_mac: [u8; MAC_LEN],
    pub client_key: [u8; CIPHER_KEY_LEN],
    pub server_key: [u8; CIPHER_KEY_LEN],
}
impl KeyBlock {
    /// Two-stage expansion `sub_1800013E0`, suite 4.
    /// ❓ Random order per stage follows TLS: stage 1 = client‖server, stage 2 = server‖client.
    pub fn derive(premaster: &[u8], client_rand: &[u8], server_rand: &[u8]) -> Self {
        let master = prf(premaster, client_rand, server_rand, MASTER_LEN);
        let block = prf(&master, server_rand, client_rand, KEY_BLOCK_LEN);
        let take = |at: usize, out: &mut [u8]| out.copy_from_slice(&block[at..at + out.len()]);
        let mut keys = Self {
            client_mac: [0; MAC_LEN],
            server_mac: [0; MAC_LEN],
            client_key: [0; CIPHER_KEY_LEN],
            server_key: [0; CIPHER_KEY_LEN],
        };
        take(0, &mut keys.client_mac);
        take(MAC_LEN, &mut keys.server_mac);
        take(2 * MAC_LEN, &mut keys.client_key);
        take(2 * MAC_LEN + CIPHER_KEY_LEN, &mut keys.server_key);
        keys
    }
    /// Server ciphers `(incoming, outgoing)`: decrypt with `client_key`, encrypt with
    /// `server_key` (the client mirror is `sub_1800135A0`). One stream each, never reset.
    pub fn server_ciphers(&self) -> (Rc4, Rc4) {
        (
            Rc4::new(&self.client_key).expect("128-byte key"),
            Rc4::new(&self.server_key).expect("128-byte key"),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RsaError {
    Generate,
    Key,
    Number,
    Blob,
}
impl std::fmt::Display for RsaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "STS token RSA error: {self:?}")
    }
}
impl std::error::Error for RsaError {}

/// CKeyRsa (`sub_18002E9E0`): raw `in^exp mod n`, no padding, word-LE numbers.
/// No Debug: holds a private exponent.
pub struct RawRsaKey {
    n: BigUint,
    e: BigUint,
    d: BigUint,
}
fn from_dig(n: &rsa::BigUint) -> BigUint {
    BigUint::from_bytes_le(&n.to_bytes_le())
}
impl RawRsaKey {
    fn from_rsa(key: &rsa::RsaPrivateKey) -> Result<Self, RsaError> {
        use rsa::traits::{PrivateKeyParts, PublicKeyParts};
        let key = Self {
            n: from_dig(key.n()),
            e: from_dig(key.e()),
            d: from_dig(key.d()),
        };
        if key.n.bits() as usize != RSA_BITS || key.e != BigUint::from(RSA_E) {
            return Err(RsaError::Key);
        }
        Ok(key)
    }
    /// A fresh ephemeral RSA-2048 key (e = 65537), one per connection.
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Result<Self, RsaError> {
        let key = rsa::RsaPrivateKey::new(rng, RSA_BITS).map_err(|_| RsaError::Generate)?;
        Self::from_rsa(&key)
    }
    /// The identity key (`sts_token_private.pem`), PKCS#1 or PKCS#8 PEM.
    /// Never commit the file. Never log the key.
    pub fn from_pem(pem: &str) -> Result<Self, RsaError> {
        use rsa::pkcs1::DecodeRsaPrivateKey;
        use rsa::pkcs8::DecodePrivateKey;
        let key = rsa::RsaPrivateKey::from_pkcs1_pem(pem)
            .or_else(|_| rsa::RsaPrivateKey::from_pkcs8_pem(pem))
            .map_err(|_| RsaError::Key)?;
        key.validate().map_err(|_| RsaError::Key)?;
        Self::from_rsa(&key)
    }
    pub fn modulus(&self) -> &BigUint {
        &self.n
    }
    pub fn exponent(&self) -> &BigUint {
        &self.e
    }
    /// The 268-byte CKeyRsa public blob.
    pub fn public_blob(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(PUBLIC_BLOB_LEN);
        blob.extend_from_slice(&BLOB_TYPE.to_le_bytes());
        blob.extend_from_slice(&RSA_E.to_le_bytes());
        blob.extend_from_slice(&(RSA_MODULUS_LEN as u32).to_le_bytes());
        let mut modulus = self.n.to_bytes_le();
        modulus.resize(RSA_MODULUS_LEN, 0);
        blob.extend_from_slice(&modulus);
        blob
    }
    /// `ServerSignature = RSA_raw(MD5(blob), d, n)`, word-LE.
    pub fn sign_md5(&self, data: &[u8]) -> Vec<u8> {
        let digest = read_number(&md5(&[data])).expect("16 bytes");
        number_bytes(&digest.modpow(&self.d, &self.n))
    }
    /// Raw private operation on a word-LE ciphertext. Rejects 0 and values >= n.
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, RsaError> {
        if ciphertext.is_empty() || ciphertext.len() > RSA_MODULUS_LEN {
            return Err(RsaError::Number);
        }
        let c = read_number(ciphertext).map_err(|_| RsaError::Number)?;
        if c.is_zero() || c >= self.n {
            return Err(RsaError::Number);
        }
        Ok(number_bytes(&c.modpow(&self.d, &self.n)))
    }
}

/// Parse a CKeyRsa public blob into `(n, e)`. Client side and tests.
pub fn parse_public_blob(blob: &[u8]) -> Result<(BigUint, BigUint), RsaError> {
    if blob.len() != PUBLIC_BLOB_LEN {
        return Err(RsaError::Blob);
    }
    let word = |at: usize| u32::from_le_bytes(blob[at..at + 4].try_into().expect("4 bytes"));
    if word(0) != BLOB_TYPE || word(8) as usize != RSA_MODULUS_LEN || word(4) == 0 {
        return Err(RsaError::Blob);
    }
    Ok((BigUint::from_bytes_le(&blob[12..]), BigUint::from(word(4))))
}

/// Pad a word-LE number to `len` bytes. Rejects longer values.
fn fixed(bytes: Vec<u8>, len: usize) -> Result<Vec<u8>, LoginError> {
    if bytes.len() > len {
        return Err(LoginError::Authentication);
    }
    let mut bytes = bytes;
    bytes.resize(len, 0);
    Ok(bytes)
}

/// ❓ Decrypted AuthnToken format. Best guess: the 3110 `AuthnToken` text (a GUID,
/// optionally NUL-terminated), else 16 raw token bytes.
fn launcher_token(bytes: &[u8]) -> Result<[u8; 16], LoginError> {
    let end = bytes.iter().rposition(|b| *b != 0).map_or(0, |p| p + 1);
    let bytes = &bytes[..end];
    if bytes.len() == 36 {
        let text = std::str::from_utf8(bytes).map_err(|_| LoginError::Authentication)?;
        return crate::messages::client::parse_guid(text).map_err(|_| LoginError::Authentication);
    }
    if bytes.is_empty() || bytes.len() > 16 {
        return Err(LoginError::Authentication);
    }
    let mut token = [0; 16];
    token[..bytes.len()].copy_from_slice(bytes);
    Ok(token)
}

/// No Debug: tokens and secrets must not enter logs.
pub enum Request {
    Connect(Binding),
    LoginTokenStart {
        client_rand: Vec<u8>,
    },
    TokenKeyData {
        premaster: Vec<u8>,
        token: Vec<u8>,
        /// ❓ Values the client sends are unknown. Kept, not interpreted.
        auth_provider_code: String,
        app_id: String,
    },
    LoginFinish {
        language: u32,
    },
    RequestGameToken {
        game_alias: String,
    },
    Logout,
}
/// No Debug: signatures and tokens must not enter logs.
pub enum Reply {
    Connected,
    /// Reply 3111.
    TokenStart {
        server_rand: Vec<u8>,
        server_public_key: Vec<u8>,
        server_signature: Vec<u8>,
    },
    /// ❓ Reply 3112 with an empty body.
    TokenKeyData,
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

struct Pending {
    client_rand: Vec<u8>,
    server_rand: Vec<u8>,
    ephemeral: RawRsaKey,
}

/// Token-login state machine. After `TokenKeyData` it continues as the 07 session
/// (`LoginFinish` → 3103, `RequestGameToken` → 3107). Pure logic: no sockets or clock reads.
pub struct TokenSession {
    session: password::Session,
    identity_key: Arc<RawRsaKey>,
    launcher_tokens: Arc<TokenAuthority>,
    pending: Option<Pending>,
    keys: Option<KeyBlock>,
    auth_provider_code: Option<String>,
    app_id: Option<String>,
}
impl TokenSession {
    /// `session` is a new 07 session. `launcher_tokens` holds tokens from
    /// `TokenAuthority::issue_launcher`; game tokens are refused on this path.
    pub fn new(
        session: password::Session,
        identity_key: Arc<RawRsaKey>,
        launcher_tokens: Arc<TokenAuthority>,
    ) -> Result<Self, LoginError> {
        if session.state() != State::New {
            return Err(LoginError::WrongOrder);
        }
        Ok(Self {
            session,
            identity_key,
            launcher_tokens,
            pending: None,
            keys: None,
            auth_provider_code: None,
            app_id: None,
        })
    }
    pub fn state(&self) -> State {
        self.session.state()
    }
    pub fn session(&self) -> &password::Session {
        &self.session
    }
    /// ❓ Move the RC4 pair `(incoming, outgoing)` to the transport exactly once, after
    /// the TokenKeyData reply is sent (same rule as 07: that reply is plaintext).
    pub fn take_ciphers(&mut self) -> Option<(Rc4, Rc4)> {
        self.keys.take().map(|k| k.server_ciphers())
    }
    pub fn auth_provider_code(&self) -> Option<&str> {
        self.auth_provider_code.as_deref()
    }
    pub fn app_id(&self) -> Option<&str> {
        self.app_id.as_deref()
    }
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
    fn close(&mut self) {
        self.pending = None;
        self.keys = None;
        self.session.abort();
    }
    fn step<R: RngCore + CryptoRng>(
        &mut self,
        request: Request,
        store: &impl AccountStore,
        rng: &mut R,
        now: Instant,
    ) -> Result<Reply, LoginError> {
        let forward = match request {
            Request::LoginTokenStart { client_rand } => {
                if self.state() != State::Connected || self.pending.is_some() {
                    return Err(LoginError::WrongOrder);
                }
                if client_rand.is_empty() || client_rand.len() > MAX_CLIENT_RAND {
                    return Err(LoginError::InvalidInput);
                }
                let ephemeral = RawRsaKey::generate(rng).map_err(|_| LoginError::Unavailable)?;
                let mut server_rand = vec![0; SERVER_RAND_LEN];
                rng.fill_bytes(&mut server_rand);
                let server_public_key = ephemeral.public_blob();
                let server_signature = self.identity_key.sign_md5(&server_public_key);
                self.pending = Some(Pending {
                    client_rand,
                    server_rand: server_rand.clone(),
                    ephemeral,
                });
                return Ok(Reply::TokenStart {
                    server_rand,
                    server_public_key,
                    server_signature,
                });
            }
            Request::TokenKeyData {
                premaster,
                token,
                auth_provider_code,
                app_id,
            } => {
                if self.state() != State::Connected {
                    return Err(LoginError::WrongOrder);
                }
                let pending = self.pending.take().ok_or(LoginError::WrongOrder)?;
                if auth_provider_code.len() > 64 || app_id.len() > 128 {
                    return Err(LoginError::InvalidInput);
                }
                let decrypt = |c: &[u8]| {
                    pending
                        .ephemeral
                        .decrypt(c)
                        .map_err(|_| LoginError::Authentication)
                };
                let premaster = fixed(decrypt(&premaster)?, PREMASTER_LEN)?;
                let token = launcher_token(&decrypt(&token)?)?;
                let identity =
                    self.launcher_tokens
                        .consume_launcher(token, self.session.binding(), now)?;
                self.session.accept_token_login(&identity, store)?;
                self.keys = Some(KeyBlock::derive(
                    &premaster,
                    &pending.client_rand,
                    &pending.server_rand,
                ));
                self.auth_provider_code = Some(auth_provider_code);
                self.app_id = Some(app_id);
                return Ok(Reply::TokenKeyData);
            }
            Request::Connect(binding) => password::Request::Connect(binding),
            Request::LoginFinish { language } => password::Request::LoginFinish { language },
            Request::RequestGameToken { game_alias } => {
                password::Request::RequestGameToken { game_alias }
            }
            Request::Logout => password::Request::Logout,
        };
        Ok(match self.session.handle(forward, store, rng, now)? {
            password::Reply::Connected => Reply::Connected,
            password::Reply::LoginFinish {
                portal_id,
                role_ids,
                auth_type,
                status,
                cafe_code,
            } => Reply::LoginFinish {
                portal_id,
                role_ids,
                auth_type,
                status,
                cafe_code,
            },
            password::Reply::GameToken { token } => Reply::GameToken { token },
            password::Reply::LoggedOut => {
                self.pending = None;
                self.keys = None;
                Reply::LoggedOut
            }
            password::Reply::LoginStart { .. } | password::Reply::KeyData { .. } => {
                return Err(LoginError::WrongOrder)
            }
        })
    }
}

/// XML bodies of the two token messages. ❓ Element names follow spec 09's field names.
pub mod xml {
    use super::*;

    fn b64(text: &str, max: usize) -> Result<Vec<u8>, XmlError> {
        if text.len() > max.div_ceil(3) * 4 {
            return Err(XmlError::KeyData);
        }
        let bytes = STANDARD.decode(text).map_err(|_| XmlError::KeyData)?;
        if bytes.is_empty() || bytes.len() > max {
            return Err(XmlError::KeyData);
        }
        Ok(bytes)
    }

    /// Client `LoginTokenStart` body: `<Request><ClientRand>b64</ClientRand></Request>`.
    pub fn login_token_start(client_rand: &[u8]) -> Result<Vec<u8>, XmlError> {
        let mut out = String::from("<Request>");
        field(&mut out, "ClientRand", &STANDARD.encode(client_rand))?;
        out.push_str("</Request>");
        Ok(out.into_bytes())
    }
    pub fn parse_login_token_start(body: &[u8]) -> Result<Request, XmlError> {
        let n = parse_xml(body, "Request")?;
        n.only(&["ClientRand"])?;
        Ok(Request::LoginTokenStart {
            client_rand: b64(n.value("ClientRand")?, MAX_CLIENT_RAND)?,
        })
    }
    /// Client `TokenKeyData` body.
    pub fn token_key_data(
        premaster: &[u8],
        token: &[u8],
        auth_provider_code: &str,
        app_id: &str,
    ) -> Result<Vec<u8>, XmlError> {
        let mut out = String::from("<Request>");
        field(&mut out, "PremasterSecret", &STANDARD.encode(premaster))?;
        field(&mut out, "AuthnToken", &STANDARD.encode(token))?;
        field(&mut out, "AuthProviderCode", auth_provider_code)?;
        field(&mut out, "AppId", app_id)?;
        out.push_str("</Request>");
        Ok(out.into_bytes())
    }
    pub fn parse_token_key_data(body: &[u8]) -> Result<Request, XmlError> {
        let n = parse_xml(body, "Request")?;
        n.only(&["PremasterSecret", "AuthnToken", "AuthProviderCode", "AppId"])?;
        Ok(Request::TokenKeyData {
            premaster: b64(n.value("PremasterSecret")?, RSA_MODULUS_LEN)?,
            token: b64(n.value("AuthnToken")?, RSA_MODULUS_LEN)?,
            auth_provider_code: n.value("AuthProviderCode")?.into(),
            app_id: n.value("AppId")?.into(),
        })
    }
    /// Reply 3111 body.
    pub fn token_start_reply(
        server_rand: &[u8],
        server_public_key: &[u8],
        server_signature: &[u8],
    ) -> Result<Vec<u8>, XmlError> {
        let mut out = String::from("<Reply>");
        field(&mut out, "ServerRand", &STANDARD.encode(server_rand))?;
        field(
            &mut out,
            "ServerPublicKey",
            &STANDARD.encode(server_public_key),
        )?;
        field(
            &mut out,
            "ServerSignature",
            &STANDARD.encode(server_signature),
        )?;
        out.push_str("</Reply>");
        Ok(out.into_bytes())
    }
    /// `(server_rand, server_public_key, server_signature)` of reply 3111.
    pub type TokenStartFields = (Vec<u8>, Vec<u8>, Vec<u8>);
    /// Parse reply 3111. Client side and tests.
    pub fn parse_token_start_reply(body: &[u8]) -> Result<TokenStartFields, XmlError> {
        let n = parse_xml(body, "Reply")?;
        n.only(&["ServerRand", "ServerPublicKey", "ServerSignature"])?;
        Ok((
            b64(n.value("ServerRand")?, MAX_CLIENT_RAND)?,
            b64(n.value("ServerPublicKey")?, PUBLIC_BLOB_LEN)?,
            b64(n.value("ServerSignature")?, RSA_MODULUS_LEN)?,
        ))
    }
    /// ❓ Reply 3112 body: empty.
    pub fn token_key_data_reply() -> Vec<u8> {
        b"<Reply/>".to_vec()
    }
}
