//! Generated account records only. No real password algorithm is supplied.
use rand::{rngs::StdRng, SeedableRng};
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
fn account(verifier: Vec<u8>) -> Result<Account, LoginError> {
    Account::from_verifier(
        Identity {
            portal_id: [1; 16],
            account_id: [2; 16],
            login_name: "generated-account".into(),
            user_name: "generated-user".into(),
            slot_id: 1,
        },
        VerifierRecord {
            scheme: "generated-test-only".into(),
            salt: vec![6; 32],
            verifier,
        },
        vec![Grant {
            binding: binding(),
            game_alias: "generated-game".into(),
        }],
    )
}
#[test]
fn default_backend_cannot_authenticate_or_issue_tokens() {
    let mut rng = StdRng::seed_from_u64(94);
    let authority = Arc::new(TokenAuthority::new(Duration::from_secs(60), 1).unwrap());
    let mut session = Session::new(binding(), authority, &mut rng).unwrap();
    let mut store = MemoryAccounts::default();
    store.insert(account(vec![91; 32]).unwrap()).unwrap();
    let now = Instant::now();
    session
        .handle(Request::Connect(binding()), &store, &mut rng, now)
        .unwrap();
    assert!(matches!(
        session.handle(
            Request::LoginStart {
                login_name: "generated-account".into()
            },
            &store,
            &mut rng,
            now
        ),
        Err(LoginError::Unavailable)
    ));
    assert_eq!(session.state(), State::Closed);
    assert!(matches!(
        session.handle(
            Request::RequestGameToken {
                game_alias: "generated-game".into()
            },
            &store,
            &mut rng,
            now
        ),
        Err(LoginError::WrongOrder)
    ));
}
#[test]
fn verifier_import_requires_nonempty_bounded_data() {
    assert!(account(vec![]).is_err());
    assert!(account(vec![1; 4097]).is_err());
    assert!(account(vec![91; 32]).is_ok());
}
#[test]
fn default_session_rejects_invalid_program_binding() {
    let mut rng = StdRng::seed_from_u64(95);
    let authority = Arc::new(TokenAuthority::new(Duration::from_secs(60), 1).unwrap());
    assert!(Session::new(
        Binding {
            program_id: 0,
            ..binding()
        },
        authority,
        &mut rng
    )
    .is_err());
}
