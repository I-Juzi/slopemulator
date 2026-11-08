use super::*;
use bloodlib::{crypto::SessionCrypt, packet::decode_fields};
use lobbygate::lists::{SCHEDULE, WORLD};
use packet::fragments::compose_fragments;
fn world(id: i16) -> Vec<Value> {
    vec![
        Value::I16(id),
        Value::I16(2),
        Value::String(vec![65]),
        Value::I8(3),
        Value::I8(4),
        Value::U32(5),
        Value::U32(6),
        Value::I16(7),
        Value::I16(8),
        Value::U32(9),
        Value::I8(10),
        Value::I8(11),
        Value::I8(12),
        Value::I8(13),
        Value::I8(14),
        Value::I8(15),
        Value::I8(16),
    ]
}
fn hello_lists(id: i16) -> Vec<Vec<u8>> {
    compose_fragments(
        2,
        &format!("hi{WORLD}{SCHEDULE}"),
        &[
            Value::Repeat(vec![world(id)]),
            Value::I8(2),
            Value::I8(1),
            Value::U64(123),
            Value::Repeat(vec![]),
        ],
    )
    .unwrap()
}
async fn broadcast_plain(client: &mut TcpStream, key: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let frame = timeout(Duration::from_secs(2), read_frame(client))
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(frame[1] >> 4, 0xe);
    let plain = SessionCrypt::new(key[..16].try_into().unwrap())
        .decrypt(&frame[2..], 8)
        .unwrap();
    (frame, inner_packet(&plain).unwrap().to_vec())
}
async fn lists_request(
    client: &mut TcpStream,
    crypt: &mut SessionPacketCrypt,
    key: &[u8],
    cmd: i16,
) -> (Vec<u8>, Vec<Value>) {
    send_packet(client, crypt, compose("h", &[Value::I16(cmd)]).unwrap()).await;
    let (frame, plain) = broadcast_plain(client, key).await;
    assert_eq!(
        &plain[2..4],
        &(if cmd == 24 { 25i16 } else { 104i16 }).to_le_bytes()
    );
    let values = decode_fields(
        if cmd == 24 {
            "ChhsQQihiQQQQQQQ;"
        } else {
            SCHEDULE
        },
        &plain[8..],
        Default::default(),
    )
    .unwrap();
    (frame, values)
}
#[tokio::test]
async fn hello_d4_cached_lists_announcements_and_reconnect() {
    use lobbygate::daemon::read_trusted;
    let mut rig = SocketRig::start(socket_config(), hello_lists(1)).await;
    let (mut client, mut crypt, _) = rig.login(400).await;
    let key = rig.broadcast_key.clone();
    let (first, values) = lists_request(&mut client, &mut crypt, &key, 24).await;
    let expected: Vec<_> = world(1)
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !matches!(i, 6 | 8))
        .map(|(_, v)| v)
        .collect();
    assert_eq!(values, vec![Value::Repeat(vec![expected])]);
    assert_eq!(
        lists_request(&mut client, &mut crypt, &key, 24).await.0,
        first
    );
    let (schedule, values) = lists_request(&mut client, &mut crypt, &key, 103).await;
    assert_eq!(
        values,
        vec![
            Value::I8(0),
            Value::I8(1),
            Value::U64(123),
            Value::Repeat(vec![])
        ]
    );
    let (mut anonymous, mut anonymous_crypt) = handshake(rig.address).await;
    rig.peer
        .write_all(&compose("hs", &[Value::I16(5), Value::String(vec![65, 66])]).unwrap())
        .await
        .unwrap();
    let (_, plain) = broadcast_plain(&mut client, &key).await;
    assert_eq!(
        decompose("hs", &plain, Default::default()).unwrap(),
        vec![Value::I16(11), Value::String(vec![65, 66])]
    );
    assert!(
        timeout(Duration::from_millis(20), read_frame(&mut anonymous))
            .await
            .is_err()
    );
    send_packet(
        &mut anonymous,
        &mut anonymous_crypt,
        compose("h", &[Value::I16(24)]).unwrap(),
    )
    .await;
    expect_eof(&mut anonymous).await;
    for packet in compose_fragments(4, &format!("hi{WORLD}"), &[Value::Repeat(vec![])]).unwrap() {
        rig.peer.write_all(&packet).await.unwrap();
    }
    // D25 is an ordered daemon-event barrier.
    rig.peer
        .write_all(
            &compose(
                "hqqh",
                &[
                    Value::I16(25),
                    Value::U64(400),
                    Value::U64(42),
                    Value::I16(1),
                ],
            )
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        &receive_packet(&mut client, &mut crypt).await[2..4],
        &14i16.to_le_bytes()
    );
    assert_eq!(
        lists_request(&mut client, &mut crypt, &key, 24).await.1,
        vec![Value::Repeat(vec![])]
    );
    assert_eq!(
        lists_request(&mut client, &mut crypt, &key, 103).await.0,
        schedule
    );
    rig.peer.shutdown().await.unwrap();
    let (mut peer, _) = timeout(Duration::from_secs(7), rig.fake.accept())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        decompose(
            "hhhh",
            &read_trusted(&mut peer).await.unwrap().unwrap(),
            Default::default()
        )
        .unwrap()[0],
        Value::I16(1)
    );
    for packet in hello_lists(9) {
        peer.write_all(&packet).await.unwrap();
    }
    for packet in compose_fragments(4, &format!("hi{WORLD}"), &[Value::Repeat(vec![])]).unwrap() {
        peer.write_all(&packet).await.unwrap();
    }
    peer.write_all(
        &compose(
            "hqqh",
            &[
                Value::I16(25),
                Value::U64(400),
                Value::U64(43),
                Value::I16(1),
            ],
        )
        .unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(
        decompose(
            "hq",
            &receive_packet(&mut client, &mut crypt).await,
            Default::default()
        )
        .unwrap(),
        vec![Value::I16(14), Value::U64(43)]
    );
    assert_eq!(
        lists_request(&mut client, &mut crypt, &key, 24).await.1,
        vec![Value::Repeat(vec![])]
    );
    assert_eq!(
        lists_request(&mut client, &mut crypt, &key, 103).await.0,
        schedule
    );
    rig.stop().await;
}
