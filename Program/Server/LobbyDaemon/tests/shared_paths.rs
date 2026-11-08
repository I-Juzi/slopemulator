use lobbydaemon::{gamed, np, store};
use shared::np_channel;

#[test]
fn public_paths_keep_the_same_shared_types() {
    let (stub, _, _, identity): (np::StubNp, _, _, np::Identity) = np_channel::StubNp::fixture(73);
    let account: store::AccountId = identity.account;
    let shared_account: np_channel::AccountId = account;
    assert_eq!(shared_account, [75; 16]);
    let charge: gamed::ChargeMode = np_channel::ChargeMode::Trial;
    assert_eq!(charge, np::ChargeMode::Trial);
    fn same_future<'a>(future: gamed::GameFuture<'a, ()>) -> np_channel::GameFuture<'a, ()> {
        future
    }
    drop(same_future(Box::pin(async { Ok(()) })));
    fn accepts_shared_np(_: &impl np_channel::Np) {}
    accepts_shared_np(&stub);
}

#[tokio::test]
async fn shared_stub_keeps_presence_and_token_behavior() {
    use np::Np;
    let (stub, token, portal, expected) = np::StubNp::fixture(74);
    assert_eq!(stub.consume(token, portal).await.unwrap(), expected);
    assert_eq!(
        stub.consume(token, portal).await,
        Err(np::NpError::RejectedToken)
    );
    stub.login_presence(expected.account, 1).await.unwrap();
    assert_eq!(
        stub.login_presence(expected.account, 2).await,
        Err(np::NpError::PresenceConflict)
    );
    stub.logout_presence(expected.account, 1);
    stub.login_presence(expected.account, 2).await.unwrap();
    assert_eq!(
        stub.world_entry(expected.account, 1, [0; 16], 0).await,
        Err(2)
    );
    assert_eq!(
        stub.create_initial_user_name(expected.account, vec![])
            .await,
        Err(7)
    );
}
