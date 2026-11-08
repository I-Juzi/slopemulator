mod common;
use common::*;
use smartcoupondaemon::np::NpReply;
use smartcoupondaemon::protocol::read;
use smartcoupondaemon::server::Server;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

async fn next(s: &mut TcpStream) -> Option<Vec<u8>> {
    timeout(Duration::from_secs(5), read(s))
        .await
        .expect("timeout")
        .unwrap()
}

#[tokio::test]
async fn daemon_and_manager_end_to_end() {
    let d = Arc::new(daemon());
    let publication = d.store.insert_publication(ACC, 800);
    d.np.push(Ok(NpReply::Success { accept_tid: 4242 }));
    let server = Server::start(d.clone(), Vec::new());
    let dl = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ml = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (da, ma) = (dl.local_addr().unwrap(), ml.local_addr().unwrap());
    tokio::spawn(server.clone().serve_daemons(dl));
    tokio::spawn(server.serve_managers(ml));

    let mut gamed = TcpStream::connect(da).await.unwrap();
    gamed.write_all(&hello(1, 1, 1)).await.unwrap();
    assert_eq!(next(&mut gamed).await, Some(h(2)));
    gamed.write_all(&by_account(3, 10002, ACC)).await.unwrap();
    assert_eq!(next(&mut gamed).await, Some(hqq(8, 3, 1)));

    // A second world session closes both connections.
    let mut dup = TcpStream::connect(da).await.unwrap();
    dup.write_all(&hello(1, 1, 1)).await.unwrap();
    assert_eq!(next(&mut dup).await, Some(hh(3, 13)));
    assert_eq!(next(&mut dup).await, None);
    assert_eq!(next(&mut gamed).await, None);

    let mut mgr = TcpStream::connect(ma).await.unwrap();
    use bloodlib::packet::Value;
    mgr.write_all(&pkt("hhh", &[Value::I16(1), Value::I16(2), Value::I16(1)]))
        .await
        .unwrap();
    assert_eq!(next(&mut mgr).await, Some(h(2)));
    mgr.write_all(&pkt(
        "hqoq",
        &[
            Value::I16(4),
            Value::I64(21),
            Value::Id(ACC),
            Value::I64(publication),
        ],
    ))
    .await
    .unwrap();
    assert_eq!(next(&mut mgr).await, Some(hqq(5, 21, 4242)));
}

#[tokio::test]
async fn bad_frame_closes_connection() {
    let server = Server::start(Arc::new(daemon()), Vec::new());
    let dl = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let da = dl.local_addr().unwrap();
    tokio::spawn(server.serve_daemons(dl));
    let mut s = TcpStream::connect(da).await.unwrap();
    s.write_all(&[2, 0]).await.unwrap();
    assert!(timeout(Duration::from_secs(5), read(&mut s))
        .await
        .unwrap()
        .map_or(true, |p| p.is_none()));
}
