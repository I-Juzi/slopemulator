use super::*;
use tokio::io::AsyncReadExt;

async fn open_idle(enabled: bool, close: bool) -> (SocketRig, TcpStream) {
    let mut config = socket_config();
    config.gate.max_idle_time_enable = enabled;
    config.gate.disconnect_after_idle_time = close;
    config.gate.max_idle_time = 0;
    let rig = SocketRig::start(config, minimal_hello()).await;
    let client = TcpStream::connect(rig.address).await.unwrap();
    (rig, client)
}
#[tokio::test]
async fn idle_disabled_and_log_only_keep_partial_hello_open() {
    for enabled in [false, true] {
        let (rig, mut client) = open_idle(enabled, false).await;
        client.write_all(&[0]).await.unwrap();
        let mut byte = [0];
        assert!(
            timeout(Duration::from_millis(15_300), client.read(&mut byte))
                .await
                .is_err()
        );
        // Finish the partial header with an invalid frame. EOF proves the server
        // still owned and read the connection after the third idle scan.
        client.write_all(&[0]).await.unwrap();
        expect_eof(&mut client).await;
        rig.stop().await;
    }
}
#[tokio::test]
async fn idle_forced_close_and_partial_receive_reset() {
    let (rig, mut client) = open_idle(true, true).await;
    let mut byte = [0];
    assert!(
        timeout(Duration::from_millis(10_300), client.read(&mut byte))
            .await
            .is_err()
    );
    // Reset after the candidate scan. One partial header byte is real receive
    // progress. The next scans must arm, mark, then report again.
    client.write_all(&[0]).await.unwrap();
    assert!(
        timeout(Duration::from_millis(10_000), client.read(&mut byte))
            .await
            .is_err()
    );
    assert_eq!(
        timeout(Duration::from_secs(6), client.read(&mut byte))
            .await
            .unwrap()
            .unwrap(),
        0
    );
    rig.stop().await;
}
#[tokio::test]
async fn idle_late_accept_waits_for_next_global_scan() {
    let mut config = socket_config();
    config.gate.max_idle_time_enable = true;
    config.gate.disconnect_after_idle_time = true;
    config.gate.max_idle_time = 0;
    let rig = SocketRig::start(config, minimal_hello()).await;
    tokio::time::sleep(Duration::from_millis(11_000)).await;
    let mut client = TcpStream::connect(rig.address).await.unwrap();
    client.write_all(&[0]).await.unwrap();
    let mut byte = [0];
    // Scans at 15, 20, 25 seconds. No catch-up scan at accept time.
    assert!(
        timeout(Duration::from_millis(10_000), client.read(&mut byte))
            .await
            .is_err()
    );
    assert_eq!(
        timeout(Duration::from_secs(5), client.read(&mut byte))
            .await
            .unwrap()
            .unwrap(),
        0
    );
    rig.stop().await;
}
