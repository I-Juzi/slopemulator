use super::*;
use bloodlib::{
    fragments::{compose_fragments, FragmentReader},
    packet::decode_fields,
};

async fn reply(
    client: &mut TcpStream,
    crypt: &mut SessionPacketCrypt,
    cmd: i16,
    format: &str,
) -> (Vec<Value>, Vec<usize>) {
    let mut fragments = FragmentReader::default();
    let mut sizes = vec![];
    loop {
        let packet = receive_packet(client, crypt).await;
        assert_eq!(&packet[2..4], &cmd.to_le_bytes());
        assert!(packet.len() <= 4000);
        sizes.push(packet.len());
        if let Some(joined) = fragments.add(&packet).unwrap() {
            return (
                decode_fields(format, &joined.data, Default::default()).unwrap(),
                sizes,
            );
        }
    }
}
async fn request(
    rig: &mut SocketRig,
    client: &mut TcpStream,
    crypt: &mut SessionPacketCrypt,
    cmd: i16,
    daemon_cmd: i16,
) {
    send_packet(
        client,
        crypt,
        compose("hb", &[Value::I16(cmd), Value::Bytes(vec![0, 255, 42])]).unwrap(),
    )
    .await;
    let packet = timeout(
        Duration::from_secs(2),
        lobbygate::daemon::read_trusted(&mut rig.peer),
    )
    .await
    .unwrap()
    .unwrap()
    .unwrap();
    assert_eq!(
        decompose("hqb", &packet, Default::default()).unwrap(),
        vec![
            Value::I16(daemon_cmd),
            Value::U64(300),
            Value::Bytes(vec![0, 255, 42])
        ]
    );
}
async fn send(rig: &mut SocketRig, cmd: i16, format: &str, values: &[Value]) -> usize {
    let packets = compose_fragments(cmd, format, values).unwrap();
    for packet in &packets {
        rig.peer.write_all(packet).await.unwrap();
    }
    packets.len()
}
#[tokio::test]
async fn d71_atomic_items_and_request_relay() {
    let mut rig = SocketRig::start(socket_config(), minimal_hello()).await;
    let (mut client, mut crypt, _) = rig.login(300).await;
    request(&mut rig, &mut client, &mut crypt, 62, 70).await;
    let row = vec![
        Value::U64(1),
        Value::U64(2),
        Value::U64(3),
        Value::U32(4),
        Value::I8(-128),
    ];
    let rows = Value::Repeat(vec![row; 300]);
    send(
        &mut rig,
        71,
        "hiqqCqqqiQ;",
        &[Value::U64(300), Value::U64(99), rows.clone()],
    )
    .await;
    let (got, sizes) = reply(&mut client, &mut crypt, 63, "qCqqqiQ;").await;
    assert_eq!(got, vec![Value::U64(99), rows]);
    assert_eq!(sizes, vec![3991, 3981, 762]);
    rig.stop().await;
}
#[tokio::test]
async fn d74_atomic_items_and_request_relay() {
    let mut rig = SocketRig::start(socket_config(), minimal_hello()).await;
    let (mut client, mut crypt, _) = rig.login(300).await;
    request(&mut rig, &mut client, &mut crypt, 65, 73).await;
    let row = vec![
        Value::U64(1),
        Value::U64(2),
        Value::U64(3),
        Value::U32(4),
        Value::I8(-1),
    ];
    let rows = Value::Repeat(vec![row; 300]);
    send(
        &mut rig,
        74,
        "hiqQiCqqqiQ;",
        &[
            Value::U64(300),
            Value::I8(-128),
            Value::U32(0xffffffff),
            rows.clone(),
        ],
    )
    .await;
    let (got, sizes) = reply(&mut client, &mut crypt, 66, "QiCqqqiQ;").await;
    assert_eq!(got, vec![Value::I8(-128), Value::U32(0xffffffff), rows]);
    assert_eq!(sizes, vec![3988, 3981, 762]);
    rig.stop().await;
}
#[tokio::test]
async fn d80_atomic_entries_and_request_relay() {
    let mut rig = SocketRig::start(socket_config(), minimal_hello()).await;
    let (mut client, mut crypt, _) = rig.login(300).await;
    request(&mut rig, &mut client, &mut crypt, 71, 79).await;
    let row = vec![
        Value::U64(1),
        Value::I8(-128),
        Value::U64(2),
        Value::Id([255; 16]),
        Value::I16(-3),
        Value::U64(4),
    ];
    let rows = Value::Repeat(vec![row; 300]);
    send(
        &mut rig,
        80,
        "hiqCqQqohq;",
        &[Value::U64(300), rows.clone()],
    )
    .await;
    let (got, sizes) = reply(&mut client, &mut crypt, 72, "CqQqohq;").await;
    assert_eq!(got, vec![rows]);
    assert_eq!(sizes, vec![3966, 3964, 3964, 1040]);
    rig.stop().await;
}
#[tokio::test]
async fn goods_signed_empty_malformed_drop_and_link_survival() {
    let mut rig = SocketRig::start(socket_config(), minimal_hello()).await;
    let (mut client, mut crypt, _) = rig.login(300).await;
    for (cmd, client_cmd, prefix, body_format) in [
        (68, 60, "", "CqiQQqqssissbbCiiibb;;"),
        (71, 63, "q", "qCqqqiQ;"),
        (74, 66, "Qi", "QiCqqqiQ;"),
        (80, 72, "", "CqQqohq;"),
    ] {
        let mut header = vec![Value::U64(300)];
        header.extend(match prefix {
            "q" => vec![Value::U64(99)],
            "Qi" => vec![Value::I8(-1), Value::U32(42)],
            _ => vec![],
        });
        let mut body = compose(&format!("q{prefix}"), &header).unwrap()[2..].to_vec();
        let count_at = body.len();
        body.extend(0u16.to_le_bytes());
        let raw = |body: &[u8]| {
            let mut packet = ((body.len() + 8) as i16).to_le_bytes().to_vec();
            packet.extend((cmd as i16).to_le_bytes());
            packet.extend((body.len() as i32).to_le_bytes());
            packet.extend(body);
            packet
        };
        // Truncated fields must not yield a partial reply or a Fail packet.
        rig.peer.write_all(&raw(&body[..7])).await.unwrap();
        for count in [0u16, 0x8000, 0xffff] {
            body[count_at..count_at + 2].copy_from_slice(&count.to_le_bytes());
            let mut trailing = body.clone();
            trailing.extend([255; 7]);
            rig.peer.write_all(&raw(&trailing)).await.unwrap();
            let mut expected = header[1..].to_vec();
            expected.push(Value::Repeat(vec![]));
            assert_eq!(
                reply(&mut client, &mut crypt, client_cmd, body_format)
                    .await
                    .0,
                expected
            );
        }
    }
    rig.stop().await;
}
fn goods_row() -> Vec<Value> {
    vec![
        Value::U64(1),
        Value::U32(2),
        Value::I8(-1),
        Value::I8(-128),
        Value::U64(3),
        Value::U64(4),
        Value::String(vec![65, 0, 66]),
        Value::String(vec![67; 80]),
        Value::U32(5),
        Value::String(vec![]),
        Value::String(vec![68; 200]),
        Value::Bytes(vec![255; 1000]),
        Value::Bytes(vec![0; 1000]),
        Value::Repeat(vec![vec![
            Value::U32(6),
            Value::U32(7),
            Value::U32(8),
            Value::Bytes(vec![9; 1000]),
            Value::Bytes(vec![10; 1000]),
        ]]),
    ]
}
#[tokio::test]
async fn d68_nested_goods_multifragment_and_request_relay() {
    let mut rig = SocketRig::start(socket_config(), minimal_hello()).await;
    let (mut client, mut crypt, _) = rig.login(300).await;
    request(&mut rig, &mut client, &mut crypt, 59, 67).await;
    let format = "CqiQQqqssissbbCiiibb;;";
    let row = goods_row();
    assert!(
        send(
            &mut rig,
            68,
            &format!("hiq{format}"),
            &[Value::U64(300), Value::Repeat(vec![row.clone(); 3])]
        )
        .await
            > 1
    );
    let mut expected = row;
    expected[6] = Value::String(vec![65]);
    let (values, sizes) = reply(&mut client, &mut crypt, 60, format).await;
    assert_eq!(values, vec![Value::Repeat(vec![expected; 3])]);
    assert!(sizes.len() > 2);
    send(
        &mut rig,
        68,
        &format!("hiq{format}"),
        &[Value::U64(999), Value::Repeat(vec![])],
    )
    .await;
    send(
        &mut rig,
        68,
        &format!("hiq{format}"),
        &[Value::U64(300), Value::Repeat(vec![])],
    )
    .await;
    assert_eq!(
        reply(&mut client, &mut crypt, 60, format).await.0,
        vec![Value::Repeat(vec![])]
    );
    rig.stop().await;
}
