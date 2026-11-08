use bloodlib::packet::{compose, decompose, DecodeLimits, Value};
use lobbygate::{
    config::Config,
    daemon::{read_trusted, LobbyDaemon, TcpLobbyDaemon},
};
use packet::fragments::compose_fragments;
use std::time::Duration;
use tokio::{io::AsyncWriteExt, net::TcpListener, sync::watch, time::timeout};
fn config() -> Config {
    Config::from_xml(
        include_str!("fixtures/lobbygate.config.xml"),
        include_str!("fixtures/bns.common.config.xml"),
        include_str!("fixtures/ContentsRule.lobbygate.config.xml"),
    )
    .unwrap()
}
#[tokio::test]
async fn hello_wait_split_coalesced_frames_and_shutdown() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut config = config();
    config.gate.lobby_daemon_addr = std::net::Ipv4Addr::LOCALHOST;
    config.gate.lobby_daemon_port = listener.local_addr().unwrap().port();
    let (stop, shutdown) = watch::channel(false);
    let start = tokio::spawn(async move { TcpLobbyDaemon::start(&config.gate, shutdown).await });
    let (mut peer, _) = listener.accept().await.unwrap();
    let hello = read_trusted(&mut peer).await.unwrap().unwrap();
    let fields = decompose("hhhh", &hello, DecodeLimits::default()).unwrap();
    assert_eq!(fields[0], Value::I16(1));
    assert_eq!(fields[2], Value::I16(58));
    assert!(!start.is_finished());
    let ok = compose_fragments(2, "hiQ", &[Value::I8(1)])
        .unwrap()
        .remove(0);
    peer.write_all(&ok[..1]).await.unwrap();
    tokio::task::yield_now().await;
    peer.write_all(&ok[1..]).await.unwrap();
    let daemon = timeout(Duration::from_secs(2), start)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(daemon.service_available());
    let mut events = daemon.take_events().unwrap();
    assert_eq!(events.recv().await.unwrap(), ok);
    let a = compose("hqE", &[Value::I16(8), Value::U64(5), Value::I16(19)]).unwrap();
    let b = compose("hqE", &[Value::I16(14), Value::U64(6), Value::I16(20)]).unwrap();
    peer.write_all(&[a.clone(), b.clone()].concat())
        .await
        .unwrap();
    assert_eq!(events.recv().await.unwrap(), a);
    assert_eq!(events.recv().await.unwrap(), b);
    // Keep a partial incoming header while sending in the other direction.
    peer.write_all(&a[..1]).await.unwrap();
    daemon.send_packet(b.clone()).await.unwrap();
    assert_eq!(read_trusted(&mut peer).await.unwrap().unwrap(), b);
    peer.write_all(&a[1..]).await.unwrap();
    assert_eq!(events.recv().await.unwrap(), a);
    stop.send(true).unwrap();
    assert!(timeout(Duration::from_secs(2), read_trusted(&mut peer))
        .await
        .unwrap()
        .unwrap()
        .is_none());
}
#[tokio::test]
async fn hello_fail_and_bad_trusted_lengths() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut config = config();
    config.gate.lobby_daemon_addr = std::net::Ipv4Addr::LOCALHOST;
    config.gate.lobby_daemon_port = listener.local_addr().unwrap().port();
    let (stop, shutdown) = watch::channel(false);
    let start = tokio::spawn(async move { TcpLobbyDaemon::start(&config.gate, shutdown).await });
    let (mut peer, _) = listener.accept().await.unwrap();
    read_trusted(&mut peer).await.unwrap();
    peer.write_all(&compose("hh", &[Value::I16(3), Value::I16(2)]).unwrap())
        .await
        .unwrap();
    assert!(start.await.unwrap().is_err());
    stop.send(true).unwrap();
    for size in [-1i16, 0, 2, 3, 4001] {
        let mut bytes = size.to_le_bytes().as_slice().to_vec();
        assert!(read_trusted(&mut bytes.as_slice()).await.is_err());
        bytes.clear();
    }
}
#[tokio::test]
async fn queued_hello_events_keep_their_own_snapshots() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut config = config();
    config.gate.lobby_daemon_addr = std::net::Ipv4Addr::LOCALHOST;
    config.gate.lobby_daemon_port = listener.local_addr().unwrap().port();
    let (stop, shutdown) = watch::channel(false);
    let start = tokio::spawn(async move { TcpLobbyDaemon::start(&config.gate, shutdown).await });
    let (mut peer, _) = listener.accept().await.unwrap();
    read_trusted(&mut peer).await.unwrap();
    let mut expected = vec![];
    for value in [1, 2, 3] {
        let packet = compose_fragments(2, "hiQ", &[Value::I8(value)])
            .unwrap()
            .remove(0);
        expected.push(packet[8..].to_vec());
        peer.write_all(&packet).await.unwrap();
    }
    let daemon = start.await.unwrap().unwrap();
    let mut events = daemon.take_events().unwrap();
    for _ in 0..3 {
        timeout(Duration::from_secs(2), events.recv())
            .await
            .unwrap()
            .unwrap();
    }
    assert_eq!(daemon.hello_lists(), expected.last().cloned());
    for body in expected {
        assert_eq!(daemon.take_hello_snapshot(), Some(body));
    }
    assert!(daemon.take_hello_snapshot().is_none());
    stop.send(true).unwrap();
}
#[tokio::test]
async fn reconnect_sends_hello_again() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut config = config();
    config.gate.lobby_daemon_addr = std::net::Ipv4Addr::LOCALHOST;
    config.gate.lobby_daemon_port = listener.local_addr().unwrap().port();
    let (stop, shutdown) = watch::channel(false);
    let start = tokio::spawn(async move { TcpLobbyDaemon::start(&config.gate, shutdown).await });
    let (mut peer, _) = listener.accept().await.unwrap();
    let hello = read_trusted(&mut peer).await.unwrap().unwrap();
    let ok = compose_fragments(2, "hiQ", &[Value::I8(1)])
        .unwrap()
        .remove(0);
    peer.write_all(&ok).await.unwrap();
    let daemon = start.await.unwrap().unwrap();
    drop(peer);
    let (mut peer, _) = timeout(Duration::from_secs(7), listener.accept())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(read_trusted(&mut peer).await.unwrap().unwrap(), hello);
    assert!(!daemon.service_available());
    peer.write_all(&ok).await.unwrap();
    timeout(Duration::from_secs(2), async {
        while !daemon.service_available() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    stop.send(true).unwrap();
}
