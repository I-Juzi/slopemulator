use super::*;
use bloodlib::{
    fragments::{compose_fragments, FragmentReader},
    packet::decode_fields,
};

fn character_row(text: Vec<u16>, bytes: usize) -> Vec<Value> {
    vec![
        Value::U64(11),
        Value::Id([22; 16]),
        Value::I16(-3),
        Value::String(text),
        Value::Bytes(vec![44; bytes]),
        Value::U64(55),
        Value::I8(-128),
        Value::I8(-1),
        Value::U32(66),
    ]
}
async fn send_characters(rig: &mut SocketRig, rows: Vec<Vec<Value>>) -> usize {
    let packets = compose_fragments(
        36,
        "hiqqhCqohsbqQQi;",
        &[
            Value::U64(300),
            Value::U64(123456),
            Value::I16(-60),
            Value::Repeat(rows),
        ],
    )
    .unwrap();
    for packet in &packets {
        rig.peer.write_all(packet).await.unwrap();
    }
    packets.len()
}
async fn character_reply(
    client: &mut TcpStream,
    crypt: &mut SessionPacketCrypt,
) -> (Vec<Value>, Vec<usize>) {
    let mut reader = FragmentReader::default();
    let mut sizes = vec![];
    loop {
        let packet = receive_packet(client, crypt).await;
        assert_eq!(&packet[2..4], &28i16.to_le_bytes());
        assert!(packet.len() <= 4000);
        sizes.push(packet.len());
        if let Some(joined) = reader.add(&packet).unwrap() {
            return (
                decode_fields("qhCqohsbqQQi;", &joined.data, Default::default()).unwrap(),
                sizes,
            );
        }
    }
}
#[tokio::test]
async fn d36_boundaries_multifragment_and_string_reserve() {
    let mut rig = SocketRig::start(socket_config(), minimal_hello()).await;
    let (mut client, mut crypt, _) = rig.login(300).await;
    let row = character_row(vec![65; 32], 1024);
    send_characters(&mut rig, vec![row.clone()]).await;
    let (got, sizes) = character_reply(&mut client, &mut crypt).await;
    assert_eq!(
        got,
        vec![
            Value::U64(123456),
            Value::I16(-60),
            Value::Repeat(vec![row])
        ]
    );
    assert_eq!(sizes.len(), 1);
    let mut rows: Vec<_> = [1024, 1024, 1024, 648, 0]
        .into_iter()
        .map(|n| character_row(vec![], n))
        .collect();
    let many = vec![character_row(vec![65; 32], 1024); 10];
    assert!(send_characters(&mut rig, many.clone()).await > 1);
    let (got, sizes) = character_reply(&mut client, &mut crypt).await;
    assert!(sizes.len() > 2);
    assert_eq!(got[2], Value::Repeat(many));
    send_characters(&mut rig, rows.clone()).await;
    let (got, sizes) = character_reply(&mut client, &mut crypt).await;
    assert_eq!(sizes, vec![3942, 26]); // The 68-byte string reserve forces the split.
    assert_eq!(got[2], Value::Repeat(rows.clone()));
    rows[0][3] = Value::String(vec![65, 0, 66]);
    send_characters(&mut rig, rows.clone()).await;
    rows[0][3] = Value::String(vec![65]);
    assert_eq!(
        character_reply(&mut client, &mut crypt).await.0[2],
        Value::Repeat(rows)
    );
    rig.stop().await;
}
fn raw_characters(data: &[u8]) -> Vec<u8> {
    let mut packet = ((data.len() + 8) as i16).to_le_bytes().to_vec();
    packet.extend(36i16.to_le_bytes());
    packet.extend((data.len() as i32).to_le_bytes());
    packet.extend(data);
    packet
}
#[tokio::test]
async fn d36_signed_counts_malformed_drop_and_link_survival() {
    let mut rig = SocketRig::start(socket_config(), minimal_hello()).await;
    let (mut client, mut crypt, _) = rig.login(300).await;
    let empty = compose_fragments(
        36,
        "hiqqhCqohsbqQQi;",
        &[
            Value::U64(300),
            Value::U64(99),
            Value::I16(0),
            Value::Repeat(vec![]),
        ],
    )
    .unwrap()[0][8..]
        .to_vec();
    for count in [0u16, 0x8000, 0xffff] {
        let mut body = empty.clone();
        body[18..20].copy_from_slice(&count.to_le_bytes());
        body.extend([255; 7]); // Trailing bytes are not rejected.
        rig.peer.write_all(&raw_characters(&body)).await.unwrap();
        let (got, sizes) = character_reply(&mut client, &mut crypt).await;
        assert_eq!(
            got,
            vec![Value::U64(99), Value::I16(0), Value::Repeat(vec![])]
        );
        assert_eq!(sizes, vec![20]);
    }
    let valid = compose_fragments(
        36,
        "hiqqhCqohsbqQQi;",
        &[
            Value::U64(300),
            Value::U64(99),
            Value::I16(0),
            Value::Repeat(vec![character_row(vec![], 0)]),
        ],
    )
    .unwrap()[0][8..]
        .to_vec();
    let mut bad_string = valid.clone();
    bad_string[46..48].copy_from_slice(&(-1i16).to_le_bytes());
    let mut bad_blob = valid.clone();
    bad_blob[48..50].copy_from_slice(&(-1i16).to_le_bytes());
    let too_long = compose_fragments(
        36,
        "hiqqhCqohsbqQQi;",
        &[
            Value::U64(300),
            Value::U64(99),
            Value::I16(0),
            Value::Repeat(vec![character_row(vec![65; 33], 0)]),
        ],
    )
    .unwrap()[0][8..]
        .to_vec();
    let big_blob = compose_fragments(
        36,
        "hiqqhCqohsbqQQi;",
        &[
            Value::U64(300),
            Value::U64(99),
            Value::I16(0),
            Value::Repeat(vec![character_row(vec![], 1025)]),
        ],
    )
    .unwrap()[0][8..]
        .to_vec();
    for body in [
        empty[..17].to_vec(),
        valid[..valid.len() - 1].to_vec(),
        bad_string,
        bad_blob,
        too_long,
        big_blob,
    ] {
        rig.peer.write_all(&raw_characters(&body)).await.unwrap();
        // A valid message is an ordered barrier. No Fail or partial C28 may precede it.
        rig.peer.write_all(&raw_characters(&empty)).await.unwrap();
        assert_eq!(
            character_reply(&mut client, &mut crypt).await.0,
            vec![Value::U64(99), Value::I16(0), Value::Repeat(vec![])]
        );
        assert!(timeout(Duration::from_millis(10), read_frame(&mut client))
            .await
            .is_err());
    }
    rig.stop().await;
}
