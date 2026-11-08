//! Launcher token login (spec 09). The client half is transcribed from the spec here,
//! not taken from server helpers. Generated keys and accounts only.
use base64::{engine::general_purpose::STANDARD, Engine};
use md5::{Digest as _, Md5};
use num_bigint::BigUint;
use rand::{rngs::StdRng, RngCore, SeedableRng};
use sha2::Sha256;
use std::{
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};
use sts::{
    password::*,
    rc4::Rc4,
    token_login::{self, xml, RawRsaKey, Reply as TReply, Request as TRequest, TokenSession},
    tokens::TokenAuthority,
};

// ---- independent client side (spec 09) ----
fn md5(parts: &[&[u8]]) -> Vec<u8> {
    let mut h = Md5::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().to_vec()
}
fn sha256(parts: &[&[u8]]) -> Vec<u8> {
    let mut h = Sha256::new();
    for p in parts {
        sha2::Digest::update(&mut h, p);
    }
    sha2::Digest::finalize(h).to_vec()
}
fn prf(secret: &[u8], a: &[u8], b: &[u8], len: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for i in 0.. {
        if out.len() >= len {
            break;
        }
        let label = vec![b'A' + (i % 26) as u8 + 1; i + 1];
        let inner = sha256(&[&label, secret, a, b]);
        out.extend(md5(&[secret, &inner]));
    }
    out.truncate(len);
    out
}
fn word_le(n: &BigUint) -> Vec<u8> {
    n.to_u32_digits()
        .iter()
        .flat_map(|w| w.to_le_bytes())
        .collect()
}
fn import(bytes: &[u8]) -> BigUint {
    BigUint::from_bytes_le(bytes)
}
fn parse_blob(blob: &[u8]) -> (BigUint, BigUint) {
    assert_eq!(blob.len(), 268);
    assert_eq!(&blob[0..4], &4u32.to_le_bytes());
    assert_eq!(&blob[8..12], &256u32.to_le_bytes());
    let e = u32::from_le_bytes(blob[4..8].try_into().unwrap());
    (import(&blob[12..]), BigUint::from(e))
}
/// Client verification `sub_180001900`: `sig^e mod n`, first 16 bytes == MD5(blob).
fn verify(blob: &[u8], sig: &[u8], n: &BigUint, e: &BigUint) -> bool {
    let mut out = import(sig).modpow(e, n).to_bytes_le();
    out.resize(16.max(out.len()), 0);
    out[..16] == md5(&[blob])[..]
}

// ---- fixtures ----
fn identity_key() -> Arc<RawRsaKey> {
    static KEY: OnceLock<Arc<RawRsaKey>> = OnceLock::new();
    KEY.get_or_init(|| {
        use rsa::pkcs8::{EncodePrivateKey, LineEnding};
        let mut rng = StdRng::seed_from_u64(9);
        let key = rsa::RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let pem = key.to_pkcs8_pem(LineEnding::LF).unwrap();
        Arc::new(RawRsaKey::from_pem(&pem).unwrap())
    })
    .clone()
}
fn binding() -> Binding {
    Binding {
        program_id: 42,
        game_code: "TEST".into(),
    }
}
fn identity() -> Identity {
    Identity {
        portal_id: [1; 16],
        account_id: [2; 16],
        login_name: "generated-account".into(),
        user_name: "generated-user".into(),
        slot_id: 3,
    }
}
struct Fixture {
    session: TokenSession,
    store: MemoryAccounts,
    launcher: Arc<TokenAuthority>,
    game: Arc<TokenAuthority>,
    rng: StdRng,
    now: Instant,
}
impl Fixture {
    fn new() -> Self {
        Self::with_launcher(Arc::new(
            TokenAuthority::new(Duration::from_secs(60), 16).unwrap(),
        ))
    }
    fn with_launcher(launcher: Arc<TokenAuthority>) -> Self {
        let mut rng = StdRng::seed_from_u64(77);
        let account = Account::from_verifier(
            identity(),
            VerifierRecord {
                scheme: "unused".into(),
                salt: vec![6; 32],
                verifier: vec![91; 32],
            },
            vec![Grant {
                binding: binding(),
                game_alias: "generated-game".into(),
            }],
        )
        .unwrap();
        let mut store = MemoryAccounts::default();
        store.insert(account).unwrap();
        let game = Arc::new(TokenAuthority::new(Duration::from_secs(60), 16).unwrap());
        let inner = Session::new(binding(), game.clone(), &mut rng).unwrap();
        let session = TokenSession::new(inner, identity_key(), launcher.clone()).unwrap();
        Self {
            session,
            store,
            launcher,
            game,
            rng,
            now: Instant::now(),
        }
    }
    fn send(&mut self, r: TRequest) -> Result<TReply, LoginError> {
        self.session.handle(r, &self.store, &mut self.rng, self.now)
    }
    fn launcher_token(&mut self) -> [u8; 16] {
        self.launcher
            .issue_launcher(identity(), binding(), &mut self.rng, self.now)
            .unwrap()
    }
    /// Runs Connect + LoginTokenStart through XML; returns (client_rand, server_rand, n, e).
    fn start(&mut self) -> (Vec<u8>, Vec<u8>, BigUint, BigUint) {
        assert!(matches!(
            self.send(TRequest::Connect(binding())),
            Ok(TReply::Connected)
        ));
        let client_rand = vec![0x5a; 32];
        let body = xml::login_token_start(&client_rand).unwrap();
        let request = xml::parse_login_token_start(&body).unwrap();
        let TReply::TokenStart {
            server_rand,
            server_public_key,
            server_signature,
        } = self.send(request).unwrap()
        else {
            panic!("expected 3111")
        };
        let body =
            xml::token_start_reply(&server_rand, &server_public_key, &server_signature).unwrap();
        let (server_rand, blob, sig) = xml::parse_token_start_reply(&body).unwrap();
        let key = identity_key();
        assert!(verify(&blob, &sig, key.modulus(), key.exponent()));
        let (n, e) = parse_blob(&blob);
        assert_eq!(e, BigUint::from(65_537u32));
        assert_eq!(server_rand.len(), 28);
        (client_rand, server_rand, n, e)
    }
    fn key_data(&mut self, premaster: &[u8], token: &[u8], n: &BigUint, e: &BigUint) -> TRequest {
        let enc = |m: &[u8]| word_le(&import(m).modpow(e, n));
        let body = xml::token_key_data(&enc(premaster), &enc(token), "launcher", "bns").unwrap();
        xml::parse_token_key_data(&body).unwrap()
    }
}

#[test]
fn prf_matches_transcription_and_split() {
    let p = token_login::prf(b"secret", b"a", b"bb", 100);
    assert_eq!(p, prf(b"secret", b"a", b"bb", 100));
    let keys = token_login::KeyBlock::derive(&[7; 48], &[1; 32], &[2; 28]);
    let master = prf(&[7; 48], &[1; 32], &[2; 28], 48);
    let block = prf(&master, &[2; 28], &[1; 32], 288);
    assert_eq!(keys.client_mac, block[..16]);
    assert_eq!(keys.server_mac, block[16..32]);
    assert_eq!(keys.client_key, block[32..160]);
    assert_eq!(keys.server_key, block[160..288]);
}

#[test]
fn md5_known_answer() {
    assert_eq!(
        token_login::md5(&[b"abc"]),
        [
            0x90, 0x01, 0x50, 0x98, 0x3c, 0xd2, 0x4f, 0xb0, 0xd6, 0x96, 0x3f, 0x7d, 0x28, 0xe1,
            0x7f, 0x72
        ]
    );
}

#[test]
fn full_token_login_installs_rc4_and_continues_as_07() {
    let mut f = Fixture::new();
    let token = f.launcher_token();
    let (client_rand, server_rand, n, e) = f.start();
    let mut premaster = [0u8; 48];
    f.rng.fill_bytes(&mut premaster);
    premaster[47] |= 1; // keep the top word nonzero so the client form is 48 bytes
    let token_text = sts::messages::client::guid_text(token);
    let request = f.key_data(&premaster, token_text.as_bytes(), &n, &e);
    assert!(matches!(f.send(request), Ok(TReply::TokenKeyData)));
    assert_eq!(f.session.state(), State::AwaitFinish);

    // Client side keys: send = client_key, receive = server_key.
    let master = prf(&premaster, &client_rand, &server_rand, 48);
    let block = prf(&master, &server_rand, &client_rand, 288);
    let mut client_send = Rc4::new(&block[32..160]).unwrap();
    let mut client_recv = Rc4::new(&block[160..288]).unwrap();
    let (mut server_in, mut server_out) = f.session.take_ciphers().unwrap();
    assert!(f.session.take_ciphers().is_none());
    let mut msg = b"POST /Auth/LoginFinish STS/1.0".to_vec();
    client_send.apply(&mut msg);
    server_in.apply(&mut msg);
    assert_eq!(msg, b"POST /Auth/LoginFinish STS/1.0");
    let mut reply = b"STS/1.0 200 OK".to_vec();
    server_out.apply(&mut reply);
    client_recv.apply(&mut reply);
    assert_eq!(reply, b"STS/1.0 200 OK");

    assert!(matches!(
        f.send(TRequest::LoginFinish { language: 1 }),
        Ok(TReply::LoginFinish { portal_id, .. }) if portal_id == [1; 16]
    ));
    let TReply::GameToken { token: game } = f
        .send(TRequest::RequestGameToken {
            game_alias: "generated-game".into(),
        })
        .unwrap()
    else {
        panic!("expected 3107")
    };
    assert!(f.game.consume(game, [1; 16], &binding(), f.now).is_ok());
    assert_eq!(f.session.app_id(), Some("bns"));
    assert_eq!(f.session.auth_provider_code(), Some("launcher"));
}

#[test]
fn raw_token_bytes_accepted_and_single_use() {
    let mut f = Fixture::new();
    let token = f.launcher_token();
    let (_, _, n, e) = f.start();
    let request = f.key_data(&[3; 48], &token, &n, &e);
    assert!(matches!(f.send(request), Ok(TReply::TokenKeyData)));
    // Replay on a new session fails: the token was consumed.
    let mut g = Fixture::with_launcher(f.launcher.clone());
    let (_, _, n, e) = g.start();
    let request = g.key_data(&[3; 48], &token, &n, &e);
    assert_eq!(g.send(request).err(), Some(LoginError::Authentication));
    assert_eq!(g.session.state(), State::Closed);
}

#[test]
fn game_tokens_and_unknown_tokens_are_refused() {
    let mut f = Fixture::new();
    // A game token placed in the launcher store path must not log in.
    let (_, _, n, e) = f.start();
    let request = f.key_data(&[3; 48], &[0x11; 16], &n, &e);
    assert_eq!(f.send(request).err(), Some(LoginError::Authentication));
    assert!(f.session.take_ciphers().is_none());

    // A launcher token is not a game token.
    let mut f = Fixture::new();
    let token = f.launcher_token();
    assert!(f
        .launcher
        .consume(token, [1; 16], &binding(), f.now)
        .is_err());
}

#[test]
fn wrong_order_and_bad_ciphertext_close() {
    let mut f = Fixture::new();
    assert!(matches!(
        f.send(TRequest::Connect(binding())),
        Ok(TReply::Connected)
    ));
    let early = TRequest::TokenKeyData {
        premaster: vec![1; 4],
        token: vec![1; 4],
        auth_provider_code: String::new(),
        app_id: String::new(),
    };
    assert_eq!(f.send(early).err(), Some(LoginError::WrongOrder));
    assert_eq!(f.session.state(), State::Closed);

    let mut f = Fixture::new();
    let token = f.launcher_token();
    let (_, _, n, _) = f.start();
    // Ciphertext >= n is rejected, and the token is not consumed.
    let bad = TRequest::TokenKeyData {
        premaster: word_le(&n),
        token: vec![1; 4],
        auth_provider_code: String::new(),
        app_id: String::new(),
    };
    assert_eq!(f.send(bad).err(), Some(LoginError::Authentication));
    // The same token still logs in on a fresh session.
    let mut g = Fixture::with_launcher(f.launcher.clone());
    let (_, _, n, e) = g.start();
    let request = g.key_data(&[3; 48], &token, &n, &e);
    assert!(matches!(g.send(request), Ok(TReply::TokenKeyData)));
}

#[test]
fn second_token_start_is_refused() {
    let mut f = Fixture::new();
    f.start();
    let again = xml::parse_login_token_start(&xml::login_token_start(&[1; 8]).unwrap()).unwrap();
    assert_eq!(f.send(again).err(), Some(LoginError::WrongOrder));
}

#[test]
fn xml_rejects_bad_fields() {
    assert!(
        xml::parse_login_token_start(b"<Request><ClientRand>!!</ClientRand></Request>").is_err()
    );
    assert!(xml::parse_login_token_start(b"<Request><Other>AA==</Other></Request>").is_err());
    let long = STANDARD.encode([0u8; 65]);
    let body = format!("<Request><ClientRand>{long}</ClientRand></Request>");
    assert!(xml::parse_login_token_start(body.as_bytes()).is_err());
    assert!(token_login::parse_public_blob(&[0; 10]).is_err());
}
