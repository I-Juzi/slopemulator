use routesrv::{config::RouteConfig, server::RouteServer, service::key_block};
use std::{sync::Arc, time::Duration};
use sts::{Frame, StartLine, Transaction};
use sts_core::{
    conn::{
        header, reply_raw, reply_result, request, set_header, transaction, write_frame, FrameReader,
    },
    connect::ConnectInfo,
    route::{BlockSlot, Location, RouteCmd, RouteMessage, Topology},
    Element,
};
use tokio::io::{DuplexStream, ReadHalf, WriteHalf};

struct Endpoint {
    reader: FrameReader<ReadHalf<DuplexStream>>,
    writer: WriteHalf<DuplexStream>,
    location: Location,
}

impl Endpoint {
    async fn open(server: &Arc<RouteServer>, index: u32) -> Self {
        let (client, router) = tokio::io::duplex(65536);
        let server = server.clone();
        tokio::spawn(async move {
            server.serve_connection(router, None).await.unwrap();
        });
        let (reader, writer) = tokio::io::split(client);
        let location = Location {
            addr: Some(format!("127.0.0.1:{}", 7000 + index).parse().unwrap()),
            product: 1,
            index,
            deployment: 2,
            epoch: 3,
        };
        let mut endpoint = Self {
            reader: FrameReader::new(reader),
            writer,
            location,
        };
        let info = ConnectInfo {
            conn_type: 2,
            location,
            process: index,
            program: 2,
            build: 381,
            ..ConnectInfo::default()
        };
        let mut connect = request("Sts", "Connect", None);
        set_header(&mut connect, "x-sts-connect", info.to_header_value());
        endpoint.call(connect, 1).await;
        endpoint
    }

    async fn send(&mut self, frame: &Frame) {
        write_frame(&mut self.writer, frame).await.unwrap();
    }

    async fn recv(&mut self) -> Frame {
        tokio::time::timeout(Duration::from_secs(3), self.reader.next())
            .await
            .expect("frame timeout")
            .unwrap()
            .expect("closed socket")
    }

    async fn call(&mut self, mut frame: Frame, id: u32) -> Frame {
        set_header(&mut frame, "s", id.to_string());
        self.send(&frame).await;
        let reply = self.recv().await;
        assert_eq!(
            transaction(&reply),
            Some(Transaction {
                id,
                complete: Some(true),
                sequence: 0
            })
        );
        assert!(reply_result(&reply).is_ok(), "{reply:?}");
        reply
    }

    async fn route(&mut self, message: RouteMessage, id: u32) -> Frame {
        self.call(
            request(
                "Route",
                message.cmd.unwrap().name(),
                Some(&message.to_element()),
            ),
            id,
        )
        .await
    }

    async fn register(&mut self, protocol: &str) {
        self.route(registration(protocol, self.location), 10).await;
    }
}

fn registration(protocol: &str, location: Location) -> RouteMessage {
    RouteMessage {
        cmd: Some(RouteCmd::RegisterProvider),
        name: protocol.into(),
        provider: Some(location),
        build: 381,
        topology: Topology::Grid,
        block_bits: 2,
        ..RouteMessage::default()
    }
}

fn message(cmd: RouteCmd, name: &str) -> RouteMessage {
    RouteMessage {
        name: name.into(),
        ..RouteMessage::new(cmd)
    }
}

fn notification(frame: &Frame, cmd: RouteCmd) -> RouteMessage {
    assert_eq!(
        frame.start,
        StartLine::Request {
            protocol: "Route".into(),
            command: cmd.name().into()
        }
    );
    assert!(header(frame, "s").is_none());
    let body = Element::parse(&frame.body).unwrap();
    let message = RouteMessage::from_element(Some(cmd), &body).unwrap();
    assert_eq!(body, message.to_element_masked(cmd.mask()));
    message
}

fn response(forwarded: &Frame, complete: bool, sequence: u32, body: &[u8]) -> Frame {
    let mut reply = reply_raw(
        transaction(forwarded).unwrap().id,
        200,
        "OK",
        body.to_vec(),
        complete,
        sequence,
    );
    set_header(&mut reply, "t", header(forwarded, "p").expect("origin"));
    reply
}

fn assert_error(frame: &Frame, id: u32, code: u32, module: u32, line: u32) {
    assert_eq!(transaction(frame).unwrap().id, id);
    let error = reply_result(frame).unwrap_err();
    assert_eq!(
        (error.code, error.module, error.line, error.server),
        (code, module, line, 9)
    );
}

struct Fixture {
    server: Arc<RouteServer>,
    p1: Endpoint,
    p2: Endpoint,
    caller: Endpoint,
}

impl Fixture {
    async fn new() -> Self {
        let server = Arc::new(RouteServer::new(RouteConfig::default(), 9));
        let mut p1 = Endpoint::open(&server, 1).await;
        let mut p2 = Endpoint::open(&server, 2).await;
        let caller = Endpoint::open(&server, 3).await;
        p1.register("Auth").await;
        p2.register("Auth").await;
        Self {
            server,
            p1,
            p2,
            caller,
        }
    }

    async fn subscribe(&mut self) {
        self.caller
            .route(message(RouteCmd::SubscribeService, "Auth"), 20)
            .await;
        let first = notification(&self.caller.recv().await, RouteCmd::ProviderFound);
        let second = notification(&self.caller.recv().await, RouteCmd::ProviderFound);
        assert_eq!(first.provider, Some(self.p1.location));
        assert_eq!(second.provider, Some(self.p2.location));
        assert_eq!(first.name, "Auth");
    }
}

#[tokio::test]
async fn registration_subscription_and_partial_reply_relay() {
    let mut f = Fixture::new().await;
    f.subscribe().await;
    let mut call = request(
        "Auth",
        "LoginStart",
        Some(&Element::text("Request", "data")),
    );
    set_header(&mut call, "s", "700");
    f.caller.send(&call).await;
    let forwarded = f.p1.recv().await;
    assert_eq!(forwarded.start, call.start);
    assert_eq!(forwarded.body, call.body);
    assert_ne!(transaction(&forwarded).unwrap().id, 700);
    assert_eq!(
        header(&forwarded, "p"),
        Some(format!("*{}; protocol=Auth", f.caller.location.format(0)).as_str())
    );
    for (complete, sequence, bytes) in [
        (false, 1, b"first".as_slice()),
        (false, 2, b"second"),
        (true, 3, b"last"),
    ] {
        f.p1.send(&response(&forwarded, complete, sequence, bytes))
            .await;
        let relayed = f.caller.recv().await;
        assert_eq!(
            transaction(&relayed),
            Some(Transaction {
                id: 700,
                complete: Some(complete),
                sequence
            })
        );
        assert_eq!(relayed.body, bytes);
        assert_eq!(header(&relayed, "t"), header(&forwarded, "p"));
    }
    // The second request must use the second provider.
    set_header(&mut call, "s", "701");
    f.caller.send(&call).await;
    let second = f.p2.recv().await;
    f.p2.send(&response(&second, true, 0, b"ok")).await;
    assert_eq!(transaction(&f.caller.recv().await).unwrap().id, 701);
}

#[tokio::test]
async fn provider_loss_notifies_subscriber_and_pending_caller() {
    let mut f = Fixture::new().await;
    f.subscribe().await;
    let mut call = request("Auth", "Pending", None);
    set_header(&mut call, "s", "800");
    f.caller.send(&call).await;
    let _forwarded = f.p1.recv().await;
    let lost_location = f.p1.location;
    drop(f.p1);
    let lost = notification(&f.caller.recv().await, RouteCmd::ProviderLost);
    assert_eq!(lost.provider, Some(lost_location));
    let error = f.caller.recv().await;
    assert_error(&error, 800, 49, 2, 1160);
    assert_eq!(
        header(&error, "t"),
        Some(format!("*{}; protocol=Auth", f.caller.location.format(0)).as_str())
    );
    let snapshot = f.server.snapshot("Auth").unwrap();
    assert_eq!(snapshot.providers.len(), 1);
    assert_eq!(snapshot.providers[0].location, f.p2.location);
    // The remaining provider can serve new requests.
    set_header(&mut call, "s", "801");
    f.caller.send(&call).await;
    let forwarded = f.p2.recv().await;
    f.p2.send(&response(&forwarded, true, 0, b"ok")).await;
    assert_eq!(transaction(&f.caller.recv().await).unwrap().id, 801);
}

#[tokio::test]
async fn set_get_blocks_and_key_block_direct_targets() {
    let mut f = Fixture::new().await;
    f.subscribe().await;
    let snapshot = f.server.snapshot("Auth").unwrap();
    let codes: Vec<_> = snapshot.providers.iter().map(|p| p.code).collect();
    let mut blocks = message(RouteCmd::SetBlocks, "Auth");
    blocks.block_bits = 2;
    blocks.build = 381;
    blocks.topology = Topology::Grid;
    blocks.desired = vec![
        BlockSlot(codes[1]),
        BlockSlot(codes[0]),
        BlockSlot(codes[1]),
        BlockSlot(codes[0]),
    ];
    f.caller.route(blocks.clone(), 30).await;
    let found = notification(&f.caller.recv().await, RouteCmd::BlocksFound);
    assert_eq!(found.current, blocks.desired);
    assert!(found.name.is_empty()); // BlocksFound mask has no Name.
    let reply = f
        .caller
        .route(message(RouteCmd::GetBlocks, "Auth"), 31)
        .await;
    let got = RouteMessage::from_element(
        Some(RouteCmd::BlocksFound),
        &Element::parse(&reply.body).unwrap(),
    )
    .unwrap();
    assert_eq!(got.current, blocks.desired);
    // A type-4 attribute must override the request-line protocol.
    for (id, target, protocol, second) in [
        (900, "0#0; protocol=Auth".to_owned(), "NotAuth", true),
        (
            901,
            "0$alice".to_owned(),
            "Auth",
            key_block("alice", 2) % 2 == 0,
        ),
        (902, format!("*{}", f.p1.location.format(0)), "Auth", false),
        (903, "0#1;protocol=Auth".to_owned(), "NotAuth", false),
    ] {
        let mut call = request(protocol, "Test", None);
        set_header(&mut call, "s", id.to_string());
        set_header(&mut call, "t", target);
        f.caller.send(&call).await;
        let provider = if second { &mut f.p2 } else { &mut f.p1 };
        let forwarded = provider.recv().await;
        assert!(header(&forwarded, "p").unwrap().ends_with("protocol=Auth"));
        provider
            .send(&response(&forwarded, true, 0, b"selected"))
            .await;
        assert_eq!(transaction(&f.caller.recv().await).unwrap().id, id);
    }
}

#[tokio::test]
async fn routing_errors_use_documented_codes_and_lines() {
    let mut f = Fixture::new().await;
    for (id, protocol, target, code, line) in [
        (1000, "Unknown", None, 49, 1160),
        (1001, "Auth", Some("0@Names:alice"), 49, 1160),
        (1002, "Auth", Some("*bad"), 10, 1573),
        (1003, "Auth", Some("0#FF"), 49, 1160),
        (1004, "Auth", Some("1#0"), 49, 1160),
        (1005, "Auth", Some("64$key"), 49, 1160),
        (1006, "Auth", Some("*127.0.0.1:1 1 2 3 4"), 49, 1160),
    ] {
        let mut call = request(protocol, "Test", None);
        set_header(&mut call, "s", id.to_string());
        set_header(
            &mut call,
            "p",
            format!("*{}; protocol={protocol}", f.caller.location.format(0)),
        );
        if let Some(target) = target {
            set_header(&mut call, "t", target);
        }
        f.caller.send(&call).await;
        let reply = f.caller.recv().await;
        assert_error(&reply, id, code, 2, line);
        assert_eq!(header(&reply, "t"), header(&call, "p"));
    }
}

#[tokio::test]
async fn subscribe_before_register_unregister_and_shutdown() {
    let server = Arc::new(RouteServer::new(RouteConfig::default(), 9));
    let mut caller = Endpoint::open(&server, 1).await;
    let mut provider = Endpoint::open(&server, 2).await;
    caller
        .route(message(RouteCmd::SubscribeService, "Auth"), 40)
        .await;
    provider.register("Auth").await;
    assert_eq!(
        notification(&caller.recv().await, RouteCmd::ProviderFound).provider,
        Some(provider.location)
    );
    let snapshot = caller.route(message(RouteCmd::Snapshot, "Auth"), 41).await;
    let body = Element::parse(&snapshot.body).unwrap();
    let parsed = RouteMessage::from_element(Some(RouteCmd::Snapshot), &body).unwrap();
    assert_eq!(
        (parsed.build, parsed.block_bits, parsed.topology),
        (381, 2, Topology::Grid)
    );
    assert_eq!(body, parsed.to_element_masked(RouteCmd::Snapshot.mask()));
    provider
        .route(
            RouteMessage {
                cmd: Some(RouteCmd::UnregisterProvider),
                ..registration("Auth", provider.location)
            },
            42,
        )
        .await;
    notification(&caller.recv().await, RouteCmd::ProviderLost);
    provider.register("Auth").await;
    notification(&caller.recv().await, RouteCmd::ProviderFound);
    provider
        .route(message(RouteCmd::CheckRegistration, ""), 43)
        .await;
    provider
        .route(message(RouteCmd::ShutdownServer, ""), 44)
        .await;
    notification(&caller.recv().await, RouteCmd::ProviderLost);
    assert!(server.snapshot("Auth").unwrap().providers.is_empty());
}

#[tokio::test]
async fn caller_id_collisions_and_existing_origin_are_preserved() {
    let mut f = Fixture::new().await;
    let mut other = Endpoint::open(&f.server, 4).await;
    let mut call = request("Auth", "Test", None);
    set_header(&mut call, "s", "77");
    set_header(&mut call, "t", format!("*{}", f.p1.location.format(0)));
    let origin = format!("*{}; protocol=Original", f.caller.location.format(0));
    set_header(&mut call, "p", &origin);
    f.caller.send(&call).await;
    let first = f.p1.recv().await;
    assert_eq!(header(&first, "p"), Some(origin.as_str()));
    call.headers.retain(|h| h.name != "p");
    other.send(&call).await;
    let second = f.p1.recv().await;
    assert_ne!(
        transaction(&first).unwrap().id,
        transaction(&second).unwrap().id
    );
    f.p1.send(&response(&second, true, 0, b"other")).await;
    f.p1.send(&response(&first, true, 0, b"caller")).await;
    assert_eq!(other.recv().await.body, b"other");
    assert_eq!(f.caller.recv().await.body, b"caller");
    // A duplicate final reply must not reach the caller.
    f.p1.send(&response(&first, true, 0, b"duplicate")).await;
    let mut check = request("Missing", "Barrier", None);
    set_header(&mut check, "s", "78");
    f.caller.send(&check).await;
    assert_error(&f.caller.recv().await, 78, 49, 2, 1160);
}

#[tokio::test(start_paused = true)]
async fn timeout_drops_mapping_and_late_reply() {
    let mut f = Fixture::new().await;
    let mut call = request("Auth", "Test", None);
    set_header(&mut call, "s", "1100");
    f.caller.send(&call).await;
    let forwarded = f.p1.recv().await;
    tokio::time::advance(Duration::from_millis(25_001)).await;
    f.p1.send(&response(&forwarded, true, 0, b"late")).await;
    let mut check = request("Missing", "Barrier", None);
    set_header(&mut check, "s", "1101");
    f.caller.send(&check).await;
    assert_error(&f.caller.recv().await, 1101, 49, 2, 1160);
    drop(f.p1);
    // Expired pending calls must not receive provider-loss errors either.
    set_header(&mut check, "s", "1102");
    f.caller.send(&check).await;
    assert_error(&f.caller.recv().await, 1102, 49, 2, 1160);
}

#[tokio::test]
async fn reply_target_and_provider_connection_are_checked() {
    let mut f = Fixture::new().await;
    let mut call = request("Auth", "Test", None);
    set_header(&mut call, "s", "1200");
    f.caller.send(&call).await;
    let forwarded = f.p1.recv().await;
    let valid = response(&forwarded, true, 0, b"valid");
    // An unrelated provider cannot resolve another connection's mapping.
    f.p2.send(&valid).await;
    let mut invalid = valid.clone();
    set_header(&mut invalid, "t", format!("*{}", f.p2.location.format(0)));
    f.p1.send(&invalid).await;
    assert_error(
        &f.p1.recv().await,
        transaction(&forwarded).unwrap().id,
        10,
        2,
        1115,
    );
    f.p1.send(&valid).await;
    assert_eq!(f.caller.recv().await.body, b"valid");
}

#[tokio::test]
async fn one_way_requests_do_not_create_pending_errors() {
    let mut f = Fixture::new().await;
    f.caller.send(&request("Auth", "OneWay", None)).await;
    let forwarded = f.p1.recv().await;
    assert!(transaction(&forwarded).is_none());
    assert!(header(&forwarded, "p").is_some());
    drop(f.p1);
    let mut check = request("Missing", "Barrier", None);
    set_header(&mut check, "s", "1300");
    f.caller.send(&check).await;
    assert_error(&f.caller.recv().await, 1300, 49, 2, 1160);
}

#[tokio::test]
async fn caller_disconnect_does_not_send_pending_reply_to_another_caller() {
    let mut f = Fixture::new().await;
    let mut call = request("Auth", "Pending", None);
    set_header(&mut call, "s", "1400");
    f.caller.send(&call).await;
    let old = f.p1.recv().await;
    drop(f.caller);
    let mut caller = Endpoint::open(&f.server, 4).await;
    set_header(&mut call, "t", format!("*{}", f.p1.location.format(0)));
    caller.send(&call).await;
    let new = f.p1.recv().await;
    f.p1.send(&response(&old, true, 0, b"old caller")).await;
    f.p1.send(&response(&new, true, 0, b"new caller")).await;
    assert_eq!(caller.recv().await.body, b"new caller");
}

#[tokio::test]
async fn sts_commands_other_than_connect_route_to_sts_provider() {
    let server = Arc::new(RouteServer::new(RouteConfig::default(), 9));
    let mut provider = Endpoint::open(&server, 1).await;
    let mut caller = Endpoint::open(&server, 2).await;
    provider.register("Sts").await;
    let mut call = request("Sts", "Test", None);
    set_header(&mut call, "s", "1500");
    caller.send(&call).await;
    let forwarded = provider.recv().await;
    assert!(header(&forwarded, "p").unwrap().ends_with("protocol=Sts"));
    provider
        .send(&response(&forwarded, true, 0, b"default"))
        .await;
    assert_eq!(caller.recv().await.body, b"default");
}

#[tokio::test]
async fn multicast_targets_are_deduplicated_per_provider() {
    let mut f = Fixture::new().await;
    let mut call = request("Auth", "OneWay", None);
    for location in [f.p1.location, f.p1.location, f.p2.location] {
        call.headers.push(sts::Header {
            name: "t".into(),
            value: format!("*{}", location.format(0)),
        });
    }
    f.caller.send(&call).await;
    f.p1.recv().await;
    f.p2.recv().await;
    // The provider's next frame must be its own local CheckRegistration reply.
    f.p1.route(message(RouteCmd::CheckRegistration, ""), 60)
        .await;
}

#[tokio::test]
async fn xml_connect_and_invalid_connect() {
    let server = Arc::new(RouteServer::new(RouteConfig::default(), 9));
    let (client, router) = tokio::io::duplex(65536);
    let me = server.clone();
    tokio::spawn(async move {
        me.serve_connection(router, None).await.unwrap();
    });
    let (reader, writer) = tokio::io::split(client);
    let mut endpoint = Endpoint {
        reader: FrameReader::new(reader),
        writer,
        location: Location::default(),
    };
    let body = Element::new("Connect")
        .child(Element::int("ConnType", 2))
        .child(Element::int("Process", 5))
        .child(Element::int("ProductType", 1))
        .child(Element::int("AppIndex", 2))
        .child(Element::text("Location", "127.0.0.1:7777;2;381;7"));
    endpoint
        .call(request("Sts", "Connect", Some(&body)), 61)
        .await;
    let location = Location {
        addr: Some("127.0.0.1:7777".parse().unwrap()),
        product: 1,
        index: 2,
        epoch: 7,
        ..Location::default()
    };
    let mut register = registration("Auth", location);
    register.provider = None;
    // Build the XML without Provider so the connected identity is used.
    let mut body = register.to_element();
    body.children.retain(|child| child.name != "Provider");
    endpoint
        .call(request("Route", "RegisterProvider", Some(&body)), 62)
        .await;
    assert_eq!(
        server.snapshot("Auth").unwrap().providers[0].location,
        location
    );
    // A second identity closes the socket and removes its providers.
    endpoint.send(&request("Sts", "Connect", None)).await;
    assert!(
        tokio::time::timeout(Duration::from_secs(3), endpoint.reader.next())
            .await
            .unwrap()
            .unwrap()
            .is_none()
    );
    assert!(server.snapshot("Auth").unwrap().providers.is_empty());

    let (client, router) = tokio::io::duplex(4096);
    tokio::spawn(async move {
        server.serve_connection(router, None).await.unwrap();
    });
    let (reader, mut writer) = tokio::io::split(client);
    let mut reader = FrameReader::new(reader);
    let mut invalid = request("Sts", "Connect", None);
    set_header(&mut invalid, "x-sts-connect", "invalid");
    write_frame(&mut writer, &invalid).await.unwrap();
    assert!(tokio::time::timeout(Duration::from_secs(3), reader.next())
        .await
        .unwrap()
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn tcp_accept_loop_and_shutdown_close_active_sockets() {
    let server = Arc::new(RouteServer::new(RouteConfig::default(), 9));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let me = server.clone();
    let serving = tokio::spawn(async move {
        me.serve(listener, async {
            let _ = stopped.await;
        })
        .await
    });
    let socket = tokio::net::TcpStream::connect(address).await.unwrap();
    let (reader, mut writer) = socket.into_split();
    let mut reader = FrameReader::new(reader);
    let mut connect = request("Sts", "Connect", None);
    set_header(&mut connect, "s", "63");
    write_frame(&mut writer, &connect).await.unwrap();
    let reply = tokio::time::timeout(Duration::from_secs(3), reader.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(reply_result(&reply).is_ok());
    let mut register = request(
        "Route",
        "RegisterProvider",
        Some(
            &registration(
                "Auth",
                Location {
                    addr: Some(address),
                    ..Location::default()
                },
            )
            .to_element(),
        ),
    );
    set_header(&mut register, "s", "64");
    write_frame(&mut writer, &register).await.unwrap();
    let reply = tokio::time::timeout(Duration::from_secs(3), reader.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(reply_result(&reply).is_ok());
    stop.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(3), serving)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(server.snapshot("Auth").unwrap().providers.is_empty());
    assert!(tokio::time::timeout(Duration::from_secs(3), reader.next())
        .await
        .unwrap()
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn zero_bit_service_uses_one_block_and_config_override() {
    let config = RouteConfig::from_ini(&sts_core::ini::Ini::parse(
        "[Network.Test]\nBlockMask=3\n[Network.Test.Test]\nBlockMask.Auth=0\n",
    ))
    .unwrap();
    let server = Arc::new(RouteServer::new(config, 9));
    let mut provider = Endpoint::open(&server, 1).await;
    let mut caller = Endpoint::open(&server, 2).await;
    let mut register = registration("Auth", provider.location);
    register.block_bits = 0; // Omitted on the wire. Use config override.
    provider.route(register, 50).await;
    let blocks = caller.route(message(RouteCmd::GetBlocks, "Auth"), 51).await;
    let parsed = RouteMessage::from_element(
        Some(RouteCmd::BlocksFound),
        &Element::parse(&blocks.body).unwrap(),
    )
    .unwrap();
    assert_eq!(parsed.current, vec![BlockSlot(1)]);
    assert_eq!(parsed.block_bits, 0);
    let mut clear = message(RouteCmd::SetBlocks, "Auth");
    clear.build = 381;
    clear.topology = Topology::Grid;
    clear.desired = vec![BlockSlot(0)];
    caller.route(clear, 52).await;
    let blocks = caller.route(message(RouteCmd::GetBlocks, "Auth"), 53).await;
    let parsed = RouteMessage::from_element(
        Some(RouteCmd::BlocksFound),
        &Element::parse(&blocks.body).unwrap(),
    )
    .unwrap();
    assert_eq!(parsed.current, vec![BlockSlot(0)]);
}
