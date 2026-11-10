mod common;
use common::{generated, Client};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use sts::{
    messages::client::{self, Reply as V, Request as W},
    password::{self, *},
    rc4::Rc4,
    tokens::TokenAuthority,
    wire::{self},
    Codec,
};

struct Fixture {
    wire: wire::Session,
    store: MemoryAccounts,
    authority: Arc<TokenAuthority>,
    rng: StdRng,
    now: Instant,
    binding: Binding,
    login: String,
    password: String,
    alias: String,
    portal: [u8; 16],
}
/// A backend that proves the password (via SRP) but yields no session key, like
/// a legacy injected verifier. Exercises the SR1-01 guard in `wire::Session`.
struct KeylessBackend(SrpPasswordBackend);
impl PasswordBackend for KeylessBackend {
    fn start(
        &self,
        login: &str,
        record: &VerifierRecord,
        entropy: &[u8; 64],
    ) -> Result<Challenge, LoginError> {
        let inner = self.0.start(login, record, entropy)?;
        Ok(Challenge {
            public: inner.public,
            exchange: Box::new(KeylessExchange(inner.exchange)),
        })
    }
}
struct KeylessExchange(Box<dyn PasswordExchange>);
impl PasswordExchange for KeylessExchange {
    // No verify_with_key override, so the default returns key: None.
    fn verify(self: Box<Self>, public: &[u8], proof: &[u8]) -> Result<Vec<u8>, LoginError> {
        self.0.verify(public, proof)
    }
}

impl Fixture {
    fn new() -> Self {
        Self::build(None)
    }
    fn new_keyless() -> Self {
        Self::build(Some(Arc::new(KeylessBackend(SrpPasswordBackend))))
    }
    fn build(backend: Option<Arc<dyn PasswordBackend>>) -> Self {
        let mut rng = StdRng::seed_from_u64(411);
        let login = generated(&mut rng);
        let password = generated(&mut rng);
        let alias = generated(&mut rng);
        let mut portal = [0; 16];
        let mut account_id = [0; 16];
        rng.fill_bytes(&mut portal);
        rng.fill_bytes(&mut account_id);
        let binding = Binding {
            program_id: 43,
            game_code: "TEST".into(),
        };
        let account = Account::from_verifier(
            Identity {
                portal_id: portal,
                account_id,
                login_name: login.clone(),
                user_name: generated(&mut rng),
                slot_id: 1,
            },
            VerifierRecord::enroll(&login, password.as_bytes(), &mut rng),
            vec![Grant {
                binding: binding.clone(),
                game_alias: alias.clone(),
            }],
        )
        .unwrap();
        let mut store = MemoryAccounts::default();
        store.insert(account).unwrap();
        let authority = Arc::new(TokenAuthority::new(Duration::from_secs(60), 16).unwrap());
        let now = Instant::now();
        let mut session = match backend {
            Some(backend) => password::Session::with_backend(
                binding.clone(),
                authority.clone(),
                &mut rng,
                backend,
            )
            .unwrap(),
            None => password::Session::new(binding.clone(), authority.clone(), &mut rng).unwrap(),
        };
        session
            .handle(Request::Connect(binding.clone()), &store, &mut rng, now)
            .unwrap();
        let wire = wire::Session::new(session, generated(&mut rng)).unwrap();
        Self {
            wire,
            store,
            authority,
            rng,
            now,
            binding,
            login,
            password,
            alias,
            portal,
        }
    }
    fn receive(&mut self, bytes: &[u8]) -> Result<Vec<Vec<u8>>, wire::Error> {
        self.wire
            .receive(bytes, &self.store, &mut self.rng, self.now)
    }
    fn challenge(&mut self) -> Client {
        let bytes = encode(
            &W::LoginStart {
                login_name: self.login.clone(),
            },
            1,
        );
        let replies = self.receive(&bytes).unwrap();
        assert_eq!(replies.len(), 1);
        let V::LoginStart { salt, public } = parse(&replies[0], 3101, 1) else {
            panic!("challenge")
        };
        Client::new(
            &self.login,
            self.password.as_bytes(),
            &salt,
            &public,
            &mut self.rng,
        )
    }
}
fn encode(request: &W, id: u32) -> Vec<u8> {
    Codec::default()
        .encode(&request.frame(Some(id)).unwrap())
        .unwrap()
}
fn parse(bytes: &[u8], reply_id: u32, transaction: u32) -> V {
    let (frame, used) = Codec::default().decode(bytes).unwrap().unwrap();
    assert_eq!(used, bytes.len());
    assert_eq!(frame.headers[0].value, format!("{transaction}R"));
    V::parse(reply_id, &frame.body).unwrap()
}
fn crypt(cipher: &mut Rc4, bytes: &mut [u8]) {
    cipher.apply(bytes);
}
#[test]
fn plaintext_proof_then_encrypted_finish_and_single_use_game_guid() {
    for fragment in [1, 7, 4096] {
        for rep in 0..1 {
            let _ = rep;
            let mut f = Fixture::new();
            let client = f.challenge();
            let mut send = Rc4::new(&client.key).unwrap();
            let mut receive = Rc4::new(&client.key).unwrap();
            let mut bytes = encode(
                &W::KeyData {
                    public: client.public.clone(),
                    proof: client.proof.clone(),
                },
                2,
            );
            let mut finish = encode(&W::LoginFinish { language: 0 }, 3);
            crypt(&mut send, &mut finish);
            bytes.extend(finish);
            let mut replies = Vec::new();
            for part in bytes.chunks(fragment) {
                replies.extend(f.receive(part).unwrap());
            }
            assert_eq!(replies.len(), 2);
            let V::KeyData { proof } = parse(&replies[0], 3102, 2) else {
                panic!("M2")
            };
            assert_eq!(proof.as_slice(), client.server_proof);
            crypt(&mut receive, &mut replies[1]);
            assert!(matches!(parse(&replies[1], 3103, 3), V::LoginFinish(_)));
            assert_eq!(f.wire.state(), State::Authenticated);
            let mut game = encode(
                &W::RequestGameToken {
                    game_code: f.binding.game_code.clone(),
                    account_alias: f.alias.clone(),
                },
                4,
            );
            crypt(&mut send, &mut game);
            let mut info = encode(&W::GetMyUserInfo, 5);
            crypt(&mut send, &mut info);
            game.extend(info);
            let mut replies = f.receive(&game).unwrap();
            assert_eq!(replies.len(), 2);
            crypt(&mut receive, &mut replies[0]);
            crypt(&mut receive, &mut replies[1]);
            let V::GameToken { token } = parse(&replies[0], 3107, 4) else {
                panic!("token")
            };
            let V::UserInfo(info) = parse(&replies[1], 3114, 5) else {
                panic!("info")
            };
            assert_eq!(info.login_name, f.login);
            assert_eq!(info.user_id, f.portal);
            let mut wrong = f.binding.clone();
            wrong.game_code = "OTHER".into();
            assert!(f.authority.consume(token, f.portal, &wrong, f.now).is_err());
            assert!(f
                .authority
                .consume(token, f.portal, &f.binding, f.now)
                .is_ok());
            assert!(f
                .authority
                .consume(token, f.portal, &f.binding, f.now)
                .is_err());
        }
    }
}
fn authenticate(f: &mut Fixture) -> (Rc4, Rc4) {
    let client = f.challenge();
    let bytes = encode(
        &W::KeyData {
            public: client.public,
            proof: client.proof,
        },
        2,
    );
    f.receive(&bytes).unwrap();
    let mut send = Rc4::new(&client.key).unwrap();
    let mut receive = Rc4::new(&client.key).unwrap();
    let mut bytes = encode(&W::LoginFinish { language: 0 }, 3);
    send.apply(&mut bytes);
    let mut replies = f.receive(&bytes).unwrap();
    receive.apply(&mut replies[0]);
    assert!(matches!(parse(&replies[0], 3103, 3), V::LoginFinish(_)));
    (send, receive)
}
#[test]
fn wrong_game_code_is_terminal_and_revokes_issued_tokens() {
    let mut f = Fixture::new();
    let (mut send, mut receive) = authenticate(&mut f);
    let mut bytes = encode(
        &W::RequestGameToken {
            game_code: f.binding.game_code.clone(),
            account_alias: f.alias.clone(),
        },
        4,
    );
    send.apply(&mut bytes);
    let mut replies = f.receive(&bytes).unwrap();
    receive.apply(&mut replies[0]);
    let V::GameToken { token } = parse(&replies[0], 3107, 4) else {
        panic!("token")
    };
    let mut bytes = encode(
        &W::RequestGameToken {
            game_code: "OTHER".into(),
            account_alias: f.alias.clone(),
        },
        5,
    );
    send.apply(&mut bytes);
    assert!(matches!(
        f.receive(&bytes),
        Err(wire::Error::Login(LoginError::Forbidden))
    ));
    assert_eq!(f.wire.state(), State::Closed);
    assert!(f
        .authority
        .consume(token, f.portal, &f.binding, f.now)
        .is_err());
}
#[test]
fn malformed_xml_frames_and_bad_m1_close_the_session() {
    for bytes in [
        b"POST /Auth/LoginStart STS/1.0\r\nl:10\r\n\r\n<Request/>".to_vec(),
        b"POST /Auth/LoginStart STS/1.0\r\nl:99999999\r\n\r\n".to_vec(),
        vec![b'x'; 32_768],
    ] {
        let mut f = Fixture::new();
        assert!(f.receive(&bytes).is_err());
        assert_eq!(f.wire.state(), State::Closed);
    }
    let mut f = Fixture::new();
    let client = f.challenge();
    let mut proof = client.proof;
    proof[0] ^= 1;
    assert!(f
        .receive(&encode(
            &W::KeyData {
                public: client.public,
                proof
            },
            2
        ))
        .is_err());
    assert_eq!(f.wire.state(), State::Closed);
}
#[test]
fn requests_before_authentication_fail_closed() {
    for request in [
        W::GetMyUserInfo,
        W::RequestToken {
            app_id: "TEST".into(),
        },
        W::LoginFinish { language: 0 },
    ] {
        let mut f = Fixture::new();
        assert!(f.receive(&encode(&request, 1)).is_err());
        assert_eq!(f.wire.state(), State::Closed);
    }
}
struct Issuer {
    app: String,
    token: String,
}
impl wire::AppTokenIssuer for Issuer {
    fn issue(&self, _: &Identity, app: &str, _: Instant) -> Result<String, LoginError> {
        if app == self.app {
            Ok(self.token.clone())
        } else {
            Err(LoginError::Forbidden)
        }
    }
}
#[test]
fn request_token_uses_explicit_app_grant_backend() {
    let mut f = Fixture::new();
    let app = generated(&mut f.rng);
    let token = generated(&mut f.rng);
    f.wire = f.wire.with_app_tokens(Arc::new(Issuer {
        app: app.clone(),
        token: token.clone(),
    }));
    let (mut send, mut receive) = authenticate(&mut f);
    let mut bytes = encode(&W::RequestToken { app_id: app }, 4);
    send.apply(&mut bytes);
    let mut replies = f.receive(&bytes).unwrap();
    receive.apply(&mut replies[0]);
    let V::Token {
        authentication_token,
    } = parse(&replies[0], 3110, 4)
    else {
        panic!("app token")
    };
    assert_eq!(authentication_token, token);
    let mut bytes = encode(&W::LogoutMyClient, 5);
    send.apply(&mut bytes);
    let mut replies = f.receive(&bytes).unwrap();
    receive.apply(&mut replies[0]);
    let (frame, _) = Codec::default().decode(&replies[0]).unwrap().unwrap();
    assert_eq!(frame.body, b"<Reply/>");
    assert_eq!(f.wire.state(), State::Closed);
}
#[test]
fn missing_app_token_backend_fails_without_issuing_a_game_token() {
    let mut f = Fixture::new();
    let (mut send, _) = authenticate(&mut f);
    let mut bytes = encode(
        &W::RequestToken {
            app_id: generated(&mut f.rng),
        },
        4,
    );
    send.apply(&mut bytes);
    assert!(matches!(
        f.receive(&bytes),
        Err(wire::Error::Login(LoginError::Unavailable))
    ));
}
#[test]
fn reply_metadata_is_not_accepted_on_requests() {
    let mut f = Fixture::new();
    let mut frame = W::LoginStart {
        login_name: f.login.clone(),
    }
    .frame(Some(1))
    .unwrap();
    frame.headers[0].value = "1R".into();
    assert!(f
        .receive(&Codec::default().encode(&frame).unwrap())
        .is_err());
}
#[test]
fn xml_error_does_not_expose_inputs() {
    assert!(!client::Error::KeyData.to_string().contains("password"));
}

#[test]
fn keydata_without_session_key_closes_the_session() {
    // SR1-01: a backend can prove the password yet yield no K. 07 requires RC4
    // both ways after reply 3102, so the wire layer must refuse rather than
    // continue the session in plaintext.
    let mut f = Fixture::new_keyless();
    let client = f.challenge();
    let bytes = encode(
        &W::KeyData {
            public: client.public.clone(),
            proof: client.proof.clone(),
        },
        2,
    );
    assert!(f.receive(&bytes).is_err());
    // The session is closed: any further input is rejected.
    assert!(f.receive(&bytes).is_err());
}
#[test]
fn unknown_requests_after_login_get_400_and_keep_the_session() {
    let mut f = Fixture::new();
    let (mut send, mut receive) = authenticate(&mut f);
    let mut bytes = b"POST /Shop/Unknown STS/1.0\r\ns:9\r\nl:10\r\n\r\n<Request/>".to_vec();
    send.apply(&mut bytes);
    let mut replies = f.receive(&bytes).unwrap();
    receive.apply(&mut replies[0]);
    let (frame, _) = Codec::default().decode(&replies[0]).unwrap().unwrap();
    assert_eq!(
        frame.start,
        sts::StartLine::Response {
            status: 400,
            reason: "ErrBadData".into()
        }
    );
    assert_eq!(frame.headers[0].value, "9R");
    assert_eq!(frame.body, b"<Error code=\"4\"/>");
    assert_eq!(f.wire.take_unsupported(), vec!["Shop/Unknown".to_string()]);
    let mut bytes = encode(&W::ListSlots, 10);
    send.apply(&mut bytes);
    let mut replies = f.receive(&bytes).unwrap();
    receive.apply(&mut replies[0]);
    assert!(matches!(parse(&replies[0], 1501, 10), V::EmptySlotList));
    assert_eq!(f.wire.state(), State::Authenticated);
}
#[test]
fn unknown_requests_before_login_still_close() {
    let mut f = Fixture::new();
    assert!(f
        .receive(b"POST /Shop/Unknown STS/1.0\r\ns:1\r\nl:10\r\n\r\n<Request/>")
        .is_err());
    assert_eq!(f.wire.state(), State::Closed);
}
