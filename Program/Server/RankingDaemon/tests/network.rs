use bloodlib::packet::Value;
use rankingdaemon::config::{parse_infogate_list, RankingConfig};
use rankingdaemon::protocol::{self, packet};
use rankingdaemon::server::{Port, Server};
use rankingdaemon::service::Daemon;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

const CONFIG: &str = r#"<config module='rankingd'>
  <option name='root-path' value='.'/><option name='data-path' value='d'/>
  <option name='log-path' value='l'/><option name='config-path' value='c'/>
  <option name='transaction-log-path' value='t'/><option name='cert-file' value='x'/>
  <option name='service-world-id' value='1'/><option name='pc-list-hash-count' value='10'/>
  <option name='ranking-list-update-period' value='600000'/>
</config>"#;

#[tokio::test]
async fn manage_and_infogate_round_trip() {
    let config = RankingConfig::parse(CONFIG).unwrap();
    let gates =
        parse_infogate_list("<infogate-list><infogate wid='1' ip-address='127.0.0.1' internal-port='22100'/></infogate-list>")
            .unwrap();
    let server = Server::new(Daemon::new(&config, &[], &gates, vec![]));
    let manage = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let infogate = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (mp, ip) = (
        manage.local_addr().unwrap().port(),
        infogate.local_addr().unwrap().port(),
    );
    tokio::spawn(server.clone().serve(Port::Manage, manage));
    tokio::spawn(server.serve(Port::InfoGate, infogate));

    let mut m = TcpStream::connect(("127.0.0.1", mp)).await.unwrap();
    m.write_all(&packet(
        "hhh",
        &[Value::I16(1), Value::I16(1), Value::I16(1)],
    ))
    .await
    .unwrap();
    let reply = timeout(Duration::from_secs(5), protocol::read(&mut m))
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(reply, packet("h", &[Value::I16(2)]));

    let mut g = TcpStream::connect(("127.0.0.1", ip)).await.unwrap();
    g.write_all(&packet(
        "hhhh",
        &[
            Value::I16(1),
            Value::I16(6),
            Value::I16(1),
            Value::I16(22100),
        ],
    ))
    .await
    .unwrap();
    let reply = timeout(Duration::from_secs(5), protocol::read(&mut g))
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(
        reply,
        packet("hCcc;", &[Value::I16(2), Value::Repeat(vec![])])
    );
}
