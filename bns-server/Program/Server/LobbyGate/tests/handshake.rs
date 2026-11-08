//! A localhost peer with generated test-only RSA keys. No original key is read.
use bloodlib::{
    crypto::RsaKey,
    frames::{decode_client, encode_client, ClientDecrypt, ClientEncrypt},
    packet::{compose, decompose, DecodeLimits, Value},
    session::SessionPacketCrypt,
};
use lobbygate::{
    config::Config,
    daemon::{InitSession, LobbyDaemon, LoginToken, SendFuture},
    server::{load_handshake_key, Server},
    wire::{inner_packet, read_frame, write_frame},
};
use num_bigint::BigUint;
use rand::{rngs::StdRng, SeedableRng};
use rsa::{
    traits::{PrivateKeyParts, PublicKeyParts},
    RsaPrivateKey,
};
use std::{
    io,
    net::SocketAddr,
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::{mpsc, watch},
    task::JoinHandle,
    time::timeout,
};

fn field(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend((bytes.len() as u32).to_le_bytes());
    out.extend(bytes);
}
struct Keys {
    server_public: RsaKey,
    server_private: RsaKey,
    server_blob: Vec<u8>,
    client_private: RsaKey,
    client_blob: Vec<u8>,
}
fn keypair(bits: usize) -> (RsaKey, RsaKey, Vec<u8>, Vec<u8>) {
    let mut rng = StdRng::seed_from_u64(bits as u64 + 381);
    let key = RsaPrivateKey::new(&mut rng, bits).unwrap();
    let mut public = Vec::new();
    field(&mut public, &key.n().to_bytes_le());
    let mut private = public.clone();
    let mut d = key.d().to_bytes_le();
    d.resize(bits / 8, 0);
    field(&mut private, &d);
    (
        RsaKey::from_public_blob(&public).unwrap(),
        RsaKey::from_private_blob(&private).unwrap(),
        public,
        private,
    )
}
fn keys() -> &'static Keys {
    static KEYS: OnceLock<Keys> = OnceLock::new();
    KEYS.get_or_init(|| {
        let (server_public, server_private, _, server_blob) = keypair(1024);
        let (_, client_private, client_blob, _) = keypair(512);
        Keys {
            server_public,
            server_private,
            server_blob,
            client_private,
            client_blob,
        }
    })
}
struct StandIn {
    tx: mpsc::Sender<InitSession>,
    accept: bool,
}
impl LobbyDaemon for StandIn {
    fn init_session(&self, request: InitSession) -> SendFuture<'_> {
        Box::pin(async move {
            self.tx
                .send(request)
                .await
                .map_err(|_| io::Error::other("test receiver closed"))?;
            if self.accept {
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::NotConnected,
                    "test daemon send failure",
                ))
            }
        })
    }
}
struct Running {
    address: SocketAddr,
    shutdown: watch::Sender<bool>,
    task: JoinHandle<io::Result<()>>,
    rx: mpsc::Receiver<InitSession>,
}
impl Running {
    async fn start(accept: bool, max_clients: i32) -> Self {
        let mut config = Config::from_xml(
            include_str!("fixtures/lobbygate.config.xml"),
            include_str!("fixtures/bns.common.config.xml"),
            include_str!("fixtures/ContentsRule.lobbygate.config.xml"),
        )
        .unwrap();
        config.gate.max_client = max_clients;
        let (tx, rx) = mpsc::channel(16);
        let server = Arc::new(
            Server::new(
                &config.gate,
                keys().server_private.clone(),
                Arc::new(StandIn { tx, accept }),
            )
            .unwrap(),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (shutdown, stop) = watch::channel(false);
        let task = tokio::spawn(server.serve(listener, stop));
        Self {
            address,
            shutdown,
            task,
            rx,
        }
    }
    async fn stop(self) {
        self.shutdown.send(true).unwrap();
        timeout(Duration::from_secs(2), self.task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }
}
fn hello_blob() -> Vec<u8> {
    let mut blob = keys().client_blob.clone();
    // p = 2^127-1, g = 3, client exponent = 123456789.
    let p = (BigUint::from(1u8) << 127) - BigUint::from(1u8);
    let g = BigUint::from(3u8);
    let a = g.modpow(&BigUint::from(123456789u64), &p);
    field(&mut blob, &g.to_bytes_le());
    field(&mut blob, &p.to_bytes_le());
    field(&mut blob, &a.to_bytes_le());
    blob
}
async fn send_hello(stream: &mut TcpStream, version: i16, blob: Vec<u8>) {
    let packet = compose(
        "hhb",
        &[Value::I16(1), Value::I16(version), Value::Bytes(blob)],
    )
    .unwrap();
    let frame = encode_client(
        &packet,
        ClientEncrypt::Handshake {
            key: &keys().server_public,
            code: 4,
        },
        &mut rand::rngs::OsRng,
    )
    .unwrap();
    write_frame(stream, &frame).await.unwrap();
}
async fn handshake(address: SocketAddr) -> (TcpStream, SessionPacketCrypt) {
    let mut stream = TcpStream::connect(address).await.unwrap();
    send_hello(&mut stream, 58, hello_blob()).await;
    let frame = timeout(Duration::from_secs(3), read_frame(&mut stream))
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(frame[1] >> 4, 5);
    let reply = decode_client(
        &frame,
        ClientDecrypt::Handshake {
            key: &keys().client_private,
            expected_code: 5,
        },
    )
    .unwrap()
    .unwrap();
    let fields = decompose("hb", &reply.packet, DecodeLimits::default()).unwrap();
    let [Value::I16(2), Value::Bytes(b)] = fields.as_slice() else {
        panic!("invalid hello reply");
    };
    let p = (BigUint::from(1u8) << 127) - BigUint::from(1u8);
    let shared = BigUint::from_bytes_le(b)
        .modpow(&BigUint::from(123456789u64), &p)
        .to_bytes_le();
    let mut key = [0; 16];
    for (i, byte) in key.iter_mut().enumerate() {
        *byte = shared[i % shared.len()];
    }
    (stream, SessionPacketCrypt::new(key))
}
async fn expect_eof(stream: &mut TcpStream) {
    assert!(timeout(Duration::from_secs(2), read_frame(stream))
        .await
        .unwrap()
        .unwrap()
        .is_none());
}
#[tokio::test]
async fn full_handshake_encrypted_packets_both_directions_and_repeated_login() {
    let mut running = Running::start(false, 4).await;
    let (mut stream, mut crypt) = handshake(running.address).await;
    for (cmd, login) in [
        (
            3,
            LoginToken::Np {
                token: [0x13; 16],
                portal: [0x37; 16],
            },
        ),
        (
            6,
            LoginToken::Lobby {
                client_key: 44,
                token: 55,
            },
        ),
    ] {
        let packet = match login {
            LoginToken::Np { token, portal } => compose(
                "hoo",
                &[Value::I16(cmd), Value::Id(token), Value::Id(portal)],
            )
            .unwrap(),
            LoginToken::Lobby { client_key, token } => compose(
                "hqq",
                &[Value::I16(cmd), Value::U64(client_key), Value::U64(token)],
            )
            .unwrap(),
        };
        let frame = crypt.encrypt(&packet, 4002, false).unwrap();
        assert_eq!(frame[1] >> 4, 8);
        write_frame(&mut stream, &frame).await.unwrap();
        let request = timeout(Duration::from_secs(2), running.rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(request.login, login);
        assert_ne!(request.gate_key, 0);
        assert_eq!(request.ip, u32::from_le_bytes([127, 0, 0, 1]));
        let d = request.packet().unwrap();
        assert_eq!(
            i16::from_le_bytes([d[2], d[3]]),
            if cmd == 3 { 6 } else { 12 }
        );
        let reply = timeout(Duration::from_secs(2), read_frame(&mut stream))
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(reply[1] >> 4, 8);
        let (plain, _) = crypt.decrypt(&reply, 4000).unwrap();
        let reply =
            decompose("hh", inner_packet(&plain).unwrap(), DecodeLimits::default()).unwrap();
        assert_eq!(
            reply,
            vec![Value::I16(if cmd == 3 { 5 } else { 8 }), Value::I16(2)]
        );
    }
    stream.shutdown().await.unwrap();
    expect_eof(&mut stream).await;
    running.stop().await;
}
#[tokio::test]
async fn successful_daemon_handoff_and_unique_keys() {
    let mut running = Running::start(true, 2).await;
    let (mut stream, mut crypt) = handshake(running.address).await;
    let packet = compose("hqq", &[Value::I16(6), Value::U64(44), Value::U64(55)]).unwrap();
    let mut previous = 0;
    for _ in 0..2 {
        write_frame(&mut stream, &crypt.encrypt(&packet, 4002, false).unwrap())
            .await
            .unwrap();
        let request = timeout(Duration::from_secs(2), running.rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert!(request.gate_key > previous);
        previous = request.gate_key;
    }
    assert!(timeout(Duration::from_millis(30), read_frame(&mut stream))
        .await
        .is_err());
    running.stop().await;
    expect_eof(&mut stream).await;
}
#[tokio::test]
async fn failed_hello_closes_without_a_reply() {
    let running = Running::start(false, 4).await;
    for (version, blob) in [
        (57, hello_blob()),
        (58, vec![]),
        (58, vec![0; 4]),
        (58, keys().client_blob.clone()),
    ] {
        let mut stream = TcpStream::connect(running.address).await.unwrap();
        send_hello(&mut stream, version, blob).await;
        expect_eof(&mut stream).await;
    }
    let mut stream = TcpStream::connect(running.address).await.unwrap();
    write_frame(
        &mut stream,
        &[0x20, 0x40].into_iter().chain([0; 128]).collect::<Vec<_>>(),
    )
    .await
    .unwrap();
    expect_eof(&mut stream).await;
    running.stop().await;
}
#[tokio::test]
async fn malformed_login_inactive_tag_and_connection_limit() {
    let running = Running::start(false, 1).await;
    let (mut first, mut crypt) = handshake(running.address).await;
    let mut excess = TcpStream::connect(running.address).await.unwrap();
    expect_eof(&mut excess).await;
    let malformed = compose("h", &[Value::I16(3)]).unwrap();
    write_frame(&mut first, &crypt.encrypt(&malformed, 4002, false).unwrap())
        .await
        .unwrap();
    expect_eof(&mut first).await;
    running.stop().await;
    let running = Running::start(false, 1).await;
    let (mut stream, mut crypt) = handshake(running.address).await;
    let mut frame = crypt.encrypt(&malformed, 4002, false).unwrap();
    frame[1] |= 0x10;
    write_frame(&mut stream, &frame).await.unwrap();
    expect_eof(&mut stream).await;
    running.stop().await;
}
/// Exercise the real daemon socket and the encrypted client socket together.
#[tokio::test]
async fn tcp_daemon_both_login_paths_and_every_failure_reason() {
    use bloodlib::packet::decode_fields;
    use lobbygate::daemon::{read_trusted, TcpLobbyDaemon};
    use packet::fragments::{compose_fragments, FragmentReader};
    let fake = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut config = Config::from_xml(
        include_str!("fixtures/lobbygate.config.xml"),
        include_str!("fixtures/bns.common.config.xml"),
        include_str!("fixtures/ContentsRule.lobbygate.config.xml"),
    )
    .unwrap();
    config.gate.lobby_daemon_addr = std::net::Ipv4Addr::LOCALHOST;
    config.gate.lobby_daemon_port = fake.local_addr().unwrap().port();
    let (shutdown, stop) = watch::channel(false);
    let gate = config.gate.clone();
    let start = tokio::spawn(async move { TcpLobbyDaemon::start(&gate, stop).await });
    let (mut peer, _) = fake.accept().await.unwrap();
    let hello = read_trusted(&mut peer).await.unwrap().unwrap();
    assert_eq!(
        decompose("hhhh", &hello, DecodeLimits::default()).unwrap(),
        vec![
            Value::I16(1),
            Value::I16(config.gate.predefined_world_id as i16),
            Value::I16(58),
            Value::I16(config.gate.internal_port as i16)
        ]
    );
    // Complete fragment framing is enough to enable service. A malformed body
    // does not undo HelloOk, as specified in 05 §2.2.
    peer.write_all(&compose_fragments(2, "hiQ", &[Value::I8(1)]).unwrap()[0])
        .await
        .unwrap();
    let daemon = start.await.unwrap().unwrap();
    config.gate.client_close_timeout = 30;
    let server = Arc::new(
        Server::new(&config.gate, keys().server_private.clone(), daemon)
            .unwrap()
            .with_contents(&config.contents)
            .unwrap(),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(server.serve(listener, shutdown.subscribe()));
    let (mut client, mut crypt) = handshake(address).await;
    let mut client_id = 100u64;
    for (path, failure, reason, expected) in [
        (3, 8, 15, 15),
        (3, 8, 234, 234),
        (3, 11, 15, 10),
        (3, 11, 16, 11),
        (3, 11, 17, 12),
        (3, 11, 18, 13),
        (3, 11, 234, 234),
        (6, 14, 19, 14),
        (6, 14, 20, 15),
        (6, 14, 21, 16),
        (6, 14, 234, 234),
        (6, 17, 19, 19),
        (6, 17, 234, 234),
    ] {
        let login = if path == 3 {
            compose(
                "hoo",
                &[Value::I16(3), Value::Id([3; 16]), Value::Id([4; 16])],
            )
        } else {
            compose("hqq", &[Value::I16(6), Value::U64(44), Value::U64(55)])
        }
        .unwrap();
        write_frame(&mut client, &crypt.encrypt(&login, 4002, false).unwrap())
            .await
            .unwrap();
        let init = timeout(Duration::from_secs(2), read_trusted(&mut peer))
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(&init[init.len() - 4..], &[127, 0, 0, 1]);
        let gate_key = u64::from_le_bytes(init[4..12].try_into().unwrap());
        if failure == 11 || failure == 17 {
            client_id += 1;
            let ok = if path == 3 {
                compose(
                    "hqqoo",
                    &[
                        Value::I16(7),
                        Value::U64(gate_key),
                        Value::U64(client_id),
                        Value::Id([3; 16]),
                        Value::Id([4; 16]),
                    ],
                )
            } else {
                compose(
                    "hqq",
                    &[Value::I16(13), Value::U64(gate_key), Value::U64(client_id)],
                )
            }
            .unwrap();
            peer.write_all(&ok).await.unwrap();
            let start = read_trusted(&mut peer).await.unwrap().unwrap();
            assert_eq!(
                i16::from_le_bytes(start[2..4].try_into().unwrap()),
                if path == 3 { 9 } else { 15 }
            );
        }
        let key = if failure == 11 || failure == 17 {
            client_id
        } else {
            gate_key
        };
        peer.write_all(
            &compose(
                "hqE",
                &[Value::I16(failure), Value::U64(key), Value::I16(reason)],
            )
            .unwrap(),
        )
        .await
        .unwrap();
        let frame = timeout(Duration::from_secs(2), read_frame(&mut client))
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let (plain, _) = crypt.decrypt(&frame, 4000).unwrap();
        assert_eq!(
            decompose("hh", inner_packet(&plain).unwrap(), DecodeLimits::default()).unwrap(),
            vec![
                Value::I16(if path == 3 { 5 } else { 8 }),
                Value::I16(expected)
            ]
        );
    }
    // AnyAuthed ping has no reply and does not close an anonymous session.
    let ping = compose("h", &[Value::I16(9)]).unwrap();
    write_frame(&mut client, &crypt.encrypt(&ping, 4002, false).unwrap())
        .await
        .unwrap();
    assert!(timeout(Duration::from_millis(30), read_frame(&mut client))
        .await
        .is_err());
    // Finish both paths. Each uses a fresh client after authentication.
    for (attempt, path) in [3, 6, 6].into_iter().enumerate() {
        let (mut client, mut crypt) = handshake(address).await;
        let login = if path == 3 {
            compose(
                "hoo",
                &[Value::I16(3), Value::Id([3; 16]), Value::Id([4; 16])],
            )
        } else {
            compose("hqq", &[Value::I16(6), Value::U64(44), Value::U64(55)])
        }
        .unwrap();
        write_frame(&mut client, &crypt.encrypt(&login, 4002, false).unwrap())
            .await
            .unwrap();
        let init = read_trusted(&mut peer).await.unwrap().unwrap();
        let gate = u64::from_le_bytes(init[4..12].try_into().unwrap());
        client_id += 1;
        let ok = if path == 3 {
            compose(
                "hqqoo",
                &[
                    Value::I16(7),
                    Value::U64(gate),
                    Value::U64(client_id),
                    Value::Id([3; 16]),
                    Value::Id([4; 16]),
                ],
            )
        } else {
            compose(
                "hqq",
                &[Value::I16(13), Value::U64(gate), Value::U64(client_id)],
            )
        }
        .unwrap();
        peer.write_all(&ok).await.unwrap();
        read_trusted(&mut peer).await.unwrap().unwrap();
        let format = if path == 3 {
            "hiqQoihCiiiii;CiiQiqq;Qqiqiq"
        } else {
            "hiqCiiiii;CiiQiqq;iqiq"
        };
        let mut values = vec![Value::U64(client_id)];
        if path == 3 {
            values.extend([
                Value::I8(2),
                Value::Id([9; 16]),
                Value::U32(0x01020304),
                Value::U16(1234),
            ]);
        }
        // Enough entries to force several fragments and a new split after K.
        values.push(Value::Repeat(vec![vec![Value::I32(1); 5]; 405]));
        values.push(Value::Repeat(vec![]));
        if path == 3 {
            values.extend([Value::I8(-1), Value::U64(66)]);
        }
        values.extend([Value::I32(7), Value::U64(8), Value::I32(9), Value::U64(10)]);
        for packet in compose_fragments(if path == 3 { 10 } else { 16 }, format, &values).unwrap() {
            peer.write_all(&packet).await.unwrap();
        }
        let mut reader = FragmentReader::default();
        let got = loop {
            let frame = timeout(Duration::from_secs(2), read_frame(&mut client))
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            let (plain, _) = crypt.decrypt(&frame, 4000).unwrap();
            let packet = inner_packet(&plain).unwrap();
            assert_eq!(
                i16::from_le_bytes(packet[2..4].try_into().unwrap()),
                if path == 3 { 4 } else { 7 }
            );
            if let Some(joined) = reader.add(packet).unwrap() {
                break decode_fields(
                    &format!("{}K", &format[2..]),
                    &joined.data,
                    DecodeLimits::default(),
                )
                .unwrap();
            }
        };
        assert_eq!(got[0], Value::U64(client_id));
        let Value::Bytes(key) = got.last().unwrap() else {
            panic!("missing broadcast key")
        };
        assert_eq!(key.len(), 17);
        assert_eq!(key[16], 1);
        if path == 3 {
            assert_eq!(got[1], Value::I8(0));
            assert_eq!(got[7], Value::I8(1));
        }
        write_frame(&mut client, &crypt.encrypt(&ping, 4002, false).unwrap())
            .await
            .unwrap();
        // Relay opaque bytes in each direction through the TCP LobbyDaemon.
        let request = compose("hb", &[Value::I16(12), Value::Bytes(vec![0, 255, 128])]).unwrap();
        write_frame(&mut client, &crypt.encrypt(&request, 4002, false).unwrap())
            .await
            .unwrap();
        let relayed = timeout(Duration::from_secs(2), read_trusted(&mut peer))
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(
            decompose("hqb", &relayed, Default::default()).unwrap(),
            vec![
                Value::I16(23),
                Value::U64(client_id),
                Value::Bytes(vec![0, 255, 128])
            ]
        );
        let reply = compose(
            "hqb",
            &[
                Value::I16(26),
                Value::U64(client_id),
                Value::Bytes(vec![128, 255, 0]),
            ],
        )
        .unwrap();
        peer.write_all(&reply).await.unwrap();
        let frame = timeout(Duration::from_secs(2), read_frame(&mut client))
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let (plain, _) = crypt.decrypt(&frame, 4000).unwrap();
        assert_eq!(
            decompose("hb", inner_packet(&plain).unwrap(), Default::default()).unwrap(),
            vec![Value::I16(15), Value::Bytes(vec![128, 255, 0])]
        );
        // Exercise non-goods handlers through the TCP LobbyDaemon.
        let request = compose("h", &[Value::I16(106)]).unwrap();
        write_frame(&mut client, &crypt.encrypt(&request, 4002, false).unwrap())
            .await
            .unwrap();
        let frame = timeout(Duration::from_secs(2), read_frame(&mut client))
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let (plain, _) = crypt.decrypt(&frame, 4000).unwrap();
        assert_eq!(&inner_packet(&plain).unwrap()[2..4], &107i16.to_le_bytes());
        let token = compose(
            "hqqh",
            &[
                Value::I16(25),
                Value::U64(client_id),
                Value::U64(123456),
                Value::I16(7),
            ],
        )
        .unwrap();
        peer.write_all(&token).await.unwrap();
        let frame = timeout(Duration::from_secs(2), read_frame(&mut client))
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let (plain, _) = crypt.decrypt(&frame, 4000).unwrap();
        assert_eq!(
            decompose("hq", inner_packet(&plain).unwrap(), Default::default()).unwrap(),
            vec![Value::I16(14), Value::U64(123456)]
        );
        if attempt == 2 {
            // A kick sends C10 before the delayed close on a third session.
            let kick = compose(
                "hqQ",
                &[Value::I16(22), Value::U64(client_id), Value::I8(-1)],
            )
            .unwrap();
            peer.write_all(&kick).await.unwrap();
            let frame = timeout(Duration::from_secs(2), read_frame(&mut client))
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            let (plain, _) = crypt.decrypt(&frame, 4000).unwrap();
            assert_eq!(
                decompose("hQ", inner_packet(&plain).unwrap(), Default::default()).unwrap(),
                vec![Value::I16(10), Value::I8(-1)]
            );
            expect_eof(&mut client).await;
            continue;
        }
        // A new login is invalid after StartSessionOk. This proves state = 1.
        write_frame(&mut client, &crypt.encrypt(&login, 4002, false).unwrap())
            .await
            .unwrap();
        expect_eof(&mut client).await;
    }
    shutdown.send(true).unwrap();
    task.await.unwrap().unwrap();
}
#[path = "support/goods_sockets.rs"]
mod goods_sockets;
#[path = "support/idle_sockets.rs"]
mod idle_sockets;
#[path = "support/list_sockets.rs"]
mod list_sockets;
#[path = "support/socket_edges.rs"]
mod socket_edges;

// Shared socket rig for close jobs, character replies, lists, and idle scans.
struct SocketRig {
    address: SocketAddr,
    fake: TcpListener,
    broadcast_key: Vec<u8>,
    peer: TcpStream,
    shutdown: watch::Sender<bool>,
    task: JoinHandle<io::Result<()>>,
}
impl SocketRig {
    async fn start(config: Config, hello: Vec<Vec<u8>>) -> Self {
        use lobbygate::daemon::{read_trusted, TcpLobbyDaemon};
        let fake = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mut config = config;
        config.gate.lobby_daemon_addr = std::net::Ipv4Addr::LOCALHOST;
        config.gate.lobby_daemon_port = fake.local_addr().unwrap().port();
        let (shutdown, stop) = watch::channel(false);
        let gate = config.gate.clone();
        let start = tokio::spawn(async move { TcpLobbyDaemon::start(&gate, stop).await });
        let (mut peer, _) = fake.accept().await.unwrap();
        assert_eq!(
            decompose(
                "hhhh",
                &read_trusted(&mut peer).await.unwrap().unwrap(),
                Default::default()
            )
            .unwrap()[0],
            Value::I16(1)
        );
        for packet in hello {
            peer.write_all(&packet).await.unwrap();
        }
        let daemon = start.await.unwrap().unwrap();
        let server =
            Arc::new(Server::new(&config.gate, keys().server_private.clone(), daemon).unwrap());
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(server.serve(listener, shutdown.subscribe()));
        Self {
            address,
            fake,
            broadcast_key: vec![],
            peer,
            shutdown,
            task,
        }
    }
    async fn login(&mut self, client_key: u64) -> (TcpStream, SessionPacketCrypt, u64) {
        use lobbygate::daemon::read_trusted;
        use packet::fragments::compose_fragments;
        let (mut client, mut crypt) = handshake(self.address).await;
        send_packet(
            &mut client,
            &mut crypt,
            compose(
                "hqq",
                &[Value::I16(6), Value::U64(client_key), Value::U64(55)],
            )
            .unwrap(),
        )
        .await;
        let init = timeout(Duration::from_secs(2), read_trusted(&mut self.peer))
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let gate = u64::from_le_bytes(init[4..12].try_into().unwrap());
        self.peer
            .write_all(
                &compose(
                    "hqq",
                    &[Value::I16(13), Value::U64(gate), Value::U64(client_key)],
                )
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            decompose(
                "hq",
                &read_trusted(&mut self.peer).await.unwrap().unwrap(),
                Default::default()
            )
            .unwrap(),
            vec![Value::I16(15), Value::U64(client_key)]
        );
        for packet in compose_fragments(
            16,
            "hiqCiiiii;CiiQiqq;iqiq",
            &[
                Value::U64(client_key),
                Value::Repeat(vec![]),
                Value::Repeat(vec![]),
                Value::I32(7),
                Value::U64(8),
                Value::I32(9),
                Value::U64(10),
            ],
        )
        .unwrap()
        {
            self.peer.write_all(&packet).await.unwrap();
        }
        let reply = receive_packet(&mut client, &mut crypt).await;
        assert_eq!(&reply[2..4], &7i16.to_le_bytes());
        let values = bloodlib::packet::decode_fields(
            "qCiiiii;CiiQiqq;iqiqK",
            &reply[8..],
            Default::default(),
        )
        .unwrap();
        let Value::Bytes(key) = values.last().unwrap() else {
            panic!("missing broadcast key")
        };
        self.broadcast_key = key.clone();
        (client, crypt, gate)
    }
    async fn stop(self) {
        self.shutdown.send(true).unwrap();
        timeout(Duration::from_secs(2), self.task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }
}
fn socket_config() -> Config {
    let mut config = Config::from_xml(
        include_str!("fixtures/lobbygate.config.xml"),
        include_str!("fixtures/bns.common.config.xml"),
        include_str!("fixtures/ContentsRule.lobbygate.config.xml"),
    )
    .unwrap();
    config.gate.timer_interval = 5;
    config.gate.client_close_timeout = 40;
    config.gate.max_idle_time_enable = false;
    config
}
fn minimal_hello() -> Vec<Vec<u8>> {
    packet::fragments::compose_fragments(2, "hiQ", &[Value::I8(1)]).unwrap()
}
async fn send_packet(client: &mut TcpStream, crypt: &mut SessionPacketCrypt, packet: Vec<u8>) {
    write_frame(client, &crypt.encrypt(&packet, 4002, false).unwrap())
        .await
        .unwrap();
}
async fn receive_packet(client: &mut TcpStream, crypt: &mut SessionPacketCrypt) -> Vec<u8> {
    let frame = timeout(Duration::from_secs(2), read_frame(client))
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let (plain, _) = crypt.decrypt(&frame, 4000).unwrap();
    inner_packet(&plain).unwrap().to_vec()
}
#[tokio::test]
async fn d20_log_only_d21_delayed_duplicate_missing_and_malformed() {
    let mut rig = SocketRig::start(socket_config(), minimal_hello()).await;
    let (mut client, mut crypt, gate) = rig.login(200).await;
    for packet in [
        compose("hq", &[Value::I16(20), Value::U64(gate)]).unwrap(),
        compose("hq", &[Value::I16(21), Value::U64(999)]).unwrap(),
        compose("h", &[Value::I16(21)]).unwrap(),
    ] {
        rig.peer.write_all(&packet).await.unwrap();
    }
    assert!(timeout(Duration::from_millis(80), read_frame(&mut client))
        .await
        .is_err());
    // A valid response proves that dropped commands did not close the daemon link.
    rig.peer
        .write_all(
            &compose(
                "hqqh",
                &[
                    Value::I16(25),
                    Value::U64(200),
                    Value::U64(42),
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
        vec![Value::I16(14), Value::U64(42)]
    );
    let close = compose("hq", &[Value::I16(21), Value::U64(200)]).unwrap();
    rig.peer
        .write_all(&[close.clone(), close].concat())
        .await
        .unwrap();
    assert!(timeout(Duration::from_millis(10), read_frame(&mut client))
        .await
        .is_err());
    expect_eof(&mut client).await;
    rig.stop().await;
}
#[tokio::test]
async fn close_timeout_zero_suppresses_d21_and_d22_jobs() {
    let mut config = socket_config();
    config.gate.client_close_timeout = 0;
    let mut rig = SocketRig::start(config, minimal_hello()).await;
    let (mut client, mut crypt, _) = rig.login(201).await;
    rig.peer
        .write_all(&compose("hq", &[Value::I16(21), Value::U64(201)]).unwrap())
        .await
        .unwrap();
    rig.peer
        .write_all(&compose("hqQ", &[Value::I16(22), Value::U64(201), Value::I8(-128)]).unwrap())
        .await
        .unwrap();
    assert_eq!(
        decompose(
            "hQ",
            &receive_packet(&mut client, &mut crypt).await,
            Default::default()
        )
        .unwrap(),
        vec![Value::I16(10), Value::I8(-128)]
    );
    assert!(timeout(Duration::from_millis(100), read_frame(&mut client))
        .await
        .is_err());
    rig.stop().await;
    expect_eof(&mut client).await;
}
#[test]
fn operator_key_file_is_required_bounded_and_validated() {
    static FILE_LOCK: Mutex<()> = Mutex::new(());
    let _guard = FILE_LOCK.lock().unwrap();
    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("generated-handshake-test-key.bin");
    assert!(load_handshake_key(&path.with_extension("absent")).is_err());
    std::fs::write(&path, &keys().server_blob).unwrap();
    assert_eq!(load_handshake_key(&path).unwrap().block_len(), 128);
    std::fs::write(&path, [0; 265]).unwrap();
    assert!(load_handshake_key(&path).is_err());
    std::fs::write(&path, [0; 264]).unwrap();
    assert!(load_handshake_key(&path).is_err());
    std::fs::remove_file(path).unwrap();
}
