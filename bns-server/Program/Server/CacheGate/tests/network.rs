use cachegate::config::CachedInfo;
use cachegate::protocol::{self, RelayHeader};
use cachegate::server::Server;
use std::net::Ipv4Addr;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

async fn read(s: &mut TcpStream) -> Vec<u8> {
    timeout(Duration::from_secs(5), protocol::read(s))
        .await
        .unwrap()
        .unwrap()
        .unwrap()
}

#[tokio::test]
async fn client_relays_through_cached() {
    let cached = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = cached.local_addr().unwrap().port();
    let server = Server::new(10060, 0);
    server.reload(&[CachedInfo {
        wid: 1,
        ip: Ipv4Addr::LOCALHOST,
        port,
    }]);
    let (mut cd, _) = timeout(Duration::from_secs(5), cached.accept())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(read(&mut cd).await, protocol::hello(1));
    cd.write_all(&protocol::hello_ok()).await.unwrap();

    let public = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = public.local_addr().unwrap();
    tokio::spawn(server.clone().serve_public(public));
    let mut client = TcpStream::connect(addr).await.unwrap();
    client
        .write_all(&[10, 0, 1, 0, 4, 0, 10, 0, 100, 0])
        .await
        .unwrap();
    assert_eq!(read(&mut client).await, protocol::hello_ok());

    // HelloOk may race the first relay; wait until the CacheD is ready.
    for _ in 0..100 {
        if server
            .gate
            .lock()
            .unwrap()
            .cached(1)
            .is_some_and(|c| c.ready)
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let h = RelayHeader {
        from_wid: 100,
        to_wid: 1,
        tid: 5,
        hint: 0,
        reserved: 0,
    };
    let up = protocol::relay(&h, b"up");
    client.write_all(&up).await.unwrap();
    assert_eq!(read(&mut cd).await, up);
    let down = protocol::relay(
        &RelayHeader {
            from_wid: 1,
            to_wid: 100,
            ..h
        },
        b"down",
    );
    cd.write_all(&down).await.unwrap();
    assert_eq!(read(&mut client).await, down);
}
