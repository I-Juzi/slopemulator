//! Generated data only. These tests do not assert vendor interoperability.
use rand::{rngs::StdRng, SeedableRng};
// Deliberately not cryptography. This backend exists only in this test binary.
struct GeneratedBackend;
struct GeneratedExchange;
impl PasswordBackend for GeneratedBackend {
    fn start(
        &self,
        _: &str,
        record: &VerifierRecord,
        _: &[u8; 64],
    ) -> Result<Challenge, LoginError> {
        if record.scheme != "generated-test-only" || record.verifier != vec![91; 32] {
            return Err(LoginError::Unavailable);
        }
        Ok(Challenge {
            public: vec![8; 32],
            exchange: Box::new(GeneratedExchange),
        })
    }
}
impl PasswordExchange for GeneratedExchange {
    fn verify(self: Box<Self>, public: &[u8], proof: &[u8]) -> Result<Vec<u8>, LoginError> {
        if public == [7; 32] && proof == [91; 32] {
            Ok(vec![9; 32])
        } else {
            Err(LoginError::Authentication)
        }
    }
}
struct GeneratedClient {
    accepted: bool,
}
impl GeneratedClient {
    fn proof(&self) -> Vec<u8> {
        vec![if self.accepted { 91 } else { 92 }; 32]
    }
    fn verify_server(&self, proof: &[u8]) -> Result<(), LoginError> {
        if self.accepted && proof == [9; 32] {
            Ok(())
        } else {
            Err(LoginError::Authentication)
        }
    }
}
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use sts::{password::*, tokens::TokenAuthority};

fn binding() -> Binding {
    Binding {
        program_id: 42,
        game_code: "TEST".into(),
    }
}
struct Fixture {
    session: Session,
    store: MemoryAccounts,
    authority: Arc<TokenAuthority>,
    rng: StdRng,
    now: Instant,
}
impl Fixture {
    fn new() -> Self {
        let mut rng = StdRng::seed_from_u64(73);
        let identity = Identity {
            portal_id: [1; 16],
            account_id: [2; 16],
            login_name: "generated-account".into(),
            user_name: "generated-user".into(),
            slot_id: 3,
        };
        let account = Account::from_verifier(
            identity,
            VerifierRecord {
                scheme: "generated-test-only".into(),
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
        let authority = Arc::new(TokenAuthority::new(Duration::from_secs(60), 16).unwrap());
        let session = Session::with_backend(
            binding(),
            authority.clone(),
            &mut rng,
            Arc::new(GeneratedBackend),
        )
        .unwrap();
        Self {
            session,
            store,
            authority,
            rng,
            now: Instant::now(),
        }
    }
    fn send(&mut self, request: Request) -> Result<Reply, LoginError> {
        self.session
            .handle(request, &self.store, &mut self.rng, self.now)
    }
    fn connect(&mut self) {
        assert!(matches!(
            self.send(Request::Connect(binding())),
            Ok(Reply::Connected)
        ));
    }
    fn proof(&mut self, password: &[u8]) -> (Vec<u8>, GeneratedClient) {
        let Reply::LoginStart { salt, public } = self
            .send(Request::LoginStart {
                login_name: "generated-account".into(),
            })
            .unwrap()
        else {
            panic!("expected challenge")
        };
        assert_eq!(salt, vec![6; 32]);
        assert_eq!(public, vec![8; 32]);
        // This selects an explicit fixture result, not a password algorithm.
        (
            vec![7; 32],
            GeneratedClient {
                accepted: password == b"generated-password",
            },
        )
    }
    fn authenticate(&mut self) {
        self.connect();
        let (public, verifier) = self.proof(b"generated-password");
        let Reply::KeyData { proof } = self
            .send(Request::KeyData {
                public,
                proof: verifier.proof().to_vec(),
            })
            .unwrap()
        else {
            panic!("expected server proof")
        };
        verifier.verify_server(&proof).unwrap();
        assert_eq!(self.session.state(), State::AwaitFinish);
        let Reply::LoginFinish { portal_id, .. } =
            self.send(Request::LoginFinish { language: 0 }).unwrap()
        else {
            panic!("expected finish")
        };
        assert_eq!(portal_id, [1; 16]);
    }
    fn token(&mut self) -> [u8; 16] {
        let Reply::GameToken { token } = self
            .send(Request::RequestGameToken {
                game_alias: "generated-game".into(),
            })
            .unwrap()
        else {
            panic!("expected token")
        };
        token
    }
}
fn error(result: Result<Reply, LoginError>, expected: LoginError) {
    assert!(matches!(result, Err(e) if e == expected));
}

#[test]
fn good_password_and_single_use_token() {
    let mut f = Fixture::new();
    f.authenticate();
    assert_eq!(f.session.state(), State::Authenticated);
    let token = f.token();
    let identity = f
        .authority
        .consume(token, [1; 16], &binding(), f.now)
        .unwrap();
    assert_eq!(identity.account_id, [2; 16]);
    assert!(matches!(
        f.authority.consume(token, [1; 16], &binding(), f.now),
        Err(LoginError::Authentication)
    ));
}
#[test]
fn bad_password_closes_session() {
    let mut f = Fixture::new();
    f.connect();
    let (public, verifier) = f.proof(b"generated-wrong-password");
    error(
        f.send(Request::KeyData {
            public,
            proof: verifier.proof().to_vec(),
        }),
        LoginError::Authentication,
    );
    assert_eq!(f.session.state(), State::Closed);
    error(
        f.send(Request::LoginFinish { language: 0 }),
        LoginError::WrongOrder,
    );
}
#[test]
fn unknown_user_fails_without_token() {
    let mut f = Fixture::new();
    f.connect();
    error(
        f.send(Request::LoginStart {
            login_name: "generated-missing".into(),
        }),
        LoginError::Authentication,
    );
    assert_eq!(f.session.state(), State::Closed);
}
#[test]
fn wrong_order_is_terminal() {
    for request in [
        Request::LoginStart {
            login_name: "generated-account".into(),
        },
        Request::LoginFinish { language: 0 },
        Request::RequestGameToken {
            game_alias: "generated-game".into(),
        },
        Request::KeyData {
            public: vec![1],
            proof: vec![0; 32],
        },
    ] {
        let mut f = Fixture::new();
        error(f.send(request), LoginError::WrongOrder);
        assert_eq!(f.session.state(), State::Closed);
    }
}
#[test]
fn finish_without_proof_is_rejected() {
    let mut f = Fixture::new();
    f.connect();
    f.proof(b"generated-password");
    error(
        f.send(Request::LoginFinish { language: 0 }),
        LoginError::WrongOrder,
    );
}
#[test]
fn token_before_finish_is_rejected() {
    let mut f = Fixture::new();
    f.connect();
    let (public, verifier) = f.proof(b"generated-password");
    f.send(Request::KeyData {
        public,
        proof: verifier.proof().to_vec(),
    })
    .unwrap();
    error(
        f.send(Request::RequestGameToken {
            game_alias: "generated-game".into(),
        }),
        LoginError::WrongOrder,
    );
}
#[test]
fn invalid_public_data_and_proof_are_rejected() {
    for (public, proof) in [
        (vec![0], vec![0; 32]),
        (vec![1; 257], vec![0; 32]),
        (vec![], vec![0; 32]),
        (vec![1], vec![0; 31]),
    ] {
        let mut f = Fixture::new();
        f.connect();
        f.proof(b"generated-password");
        error(
            f.send(Request::KeyData { public, proof }),
            LoginError::Authentication,
        );
        assert_eq!(f.session.state(), State::Closed);
    }
}
#[test]
fn bad_bindings_do_not_consume_token() {
    let mut f = Fixture::new();
    f.authenticate();
    let token = f.token();
    for b in [
        Binding {
            program_id: 99,
            ..binding()
        },
        Binding {
            game_code: "OTHER".into(),
            ..binding()
        },
    ] {
        assert!(f.authority.consume(token, [1; 16], &b, f.now).is_err());
    }
    assert!(f
        .authority
        .consume(token, [9; 16], &binding(), f.now)
        .is_err());
    assert!(f
        .authority
        .consume(token, [1; 16], &binding(), f.now)
        .is_ok());
}
#[test]
fn expiry_is_exact() {
    let mut f = Fixture::new();
    f.authenticate();
    let token = f.token();
    assert!(f
        .authority
        .consume(token, [1; 16], &binding(), f.now + Duration::from_secs(60))
        .is_err());
}
#[test]
fn valid_just_before_expiry() {
    let mut f = Fixture::new();
    f.authenticate();
    let token = f.token();
    assert!(f
        .authority
        .consume(
            token,
            [1; 16],
            &binding(),
            f.now + Duration::from_secs(60) - Duration::from_nanos(1)
        )
        .is_ok());
}
#[test]
fn logout_revokes_tokens() {
    let mut f = Fixture::new();
    f.authenticate();
    let token = f.token();
    assert!(matches!(f.send(Request::Logout), Ok(Reply::LoggedOut)));
    assert!(f
        .authority
        .consume(token, [1; 16], &binding(), f.now)
        .is_err());
}
#[test]
fn disconnect_revokes_tokens() {
    let mut f = Fixture::new();
    f.authenticate();
    let token = f.token();
    drop(f.session);
    assert!(f
        .authority
        .consume(token, [1; 16], &binding(), f.now)
        .is_err());
}
#[test]
fn wrong_alias_closes_and_revokes() {
    let mut f = Fixture::new();
    f.authenticate();
    let token = f.token();
    error(
        f.send(Request::RequestGameToken {
            game_alias: "generated-other".into(),
        }),
        LoginError::Forbidden,
    );
    assert!(f
        .authority
        .consume(token, [1; 16], &binding(), f.now)
        .is_err());
}
#[test]
fn wrong_connect_binding_fails() {
    let mut f = Fixture::new();
    error(
        f.send(Request::Connect(Binding {
            program_id: 99,
            ..binding()
        })),
        LoginError::Forbidden,
    );
}
#[test]
fn duplicate_account_rejected() {
    let mut f = Fixture::new();
    let account = f.store.find("generated-account").unwrap();
    assert_eq!(f.store.insert(account), Err(LoginError::InvalidInput));
}
#[test]
fn invalid_policy_rejected() {
    assert!(TokenAuthority::new(Duration::ZERO, 1).is_err());
    assert!(TokenAuthority::new(Duration::from_secs(1), 0).is_err());
    assert!(!Binding {
        program_id: 0,
        ..binding()
    }
    .valid());
    assert!(!Binding {
        game_code: "TOO_LONG_GAME".into(),
        ..binding()
    }
    .valid());
}
#[test]
fn concurrent_consume_has_one_winner() {
    let mut f = Fixture::new();
    f.authenticate();
    let token = f.token();
    let now = f.now;
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let authority = f.authority.clone();
            std::thread::spawn(move || authority.consume(token, [1; 16], &binding(), now).is_ok())
        })
        .collect();
    assert_eq!(
        handles
            .into_iter()
            .map(|h| usize::from(h.join().unwrap()))
            .sum::<usize>(),
        1
    );
}
#[cfg(feature = "lobbyd-adapter")]
#[test]
fn adapter_consumes_once_without_fallback() {
    use lobbydaemon::np::{Np, NpError, StubNp};
    use std::{
        future::Future,
        task::{Context, Poll, Waker},
    };
    fn ready<T>(future: impl Future<Output = T>) -> T {
        let mut future = std::pin::pin!(future);
        match future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("unexpected pending future"),
        }
    }
    let mut f = Fixture::new();
    f.authenticate();
    let token = f.token();
    let adapter =
        sts::lobbyd_adapter::AuthorityNp::new(f.authority.clone(), binding(), StubNp::default())
            .unwrap();
    // Dropping an unpolled future must not consume the token.
    drop(adapter.consume(token, [1; 16]));
    let identity = ready(adapter.consume(token, [1; 16])).unwrap();
    assert_eq!(identity.account, [2; 16]);
    assert_eq!(
        ready(adapter.consume(token, [1; 16])),
        Err(NpError::RejectedToken)
    );
    assert_eq!(
        ready(adapter.consume([8; 16], [1; 16])),
        Err(NpError::RejectedToken)
    );
}
