use lobbygate::wire::*;
use tokio::io::{duplex, AsyncWriteExt};
#[tokio::test]
async fn split_headers_and_coalesced_frames() {
    let (mut tx, mut rx) = duplex(64);
    let task = tokio::spawn(async move {
        tx.write_all(&[4]).await.unwrap();
        tokio::task::yield_now().await;
        tx.write_all(&[0x80]).await.unwrap();
        tx.write_all(&[0; 16]).await.unwrap();
        write_frame(
            &mut tx,
            &[4, 0x90, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        )
        .await
        .unwrap();
    });
    assert_eq!(read_frame(&mut rx).await.unwrap().unwrap().len(), 18);
    assert_eq!(read_frame(&mut rx).await.unwrap().unwrap()[1], 0x90);
    assert!(read_frame(&mut rx).await.unwrap().is_none());
    task.await.unwrap();
}
#[tokio::test]
async fn bad_sizes_plain_and_truncation() {
    for input in [vec![0xff, 0x8f], vec![0, 0], vec![4], vec![4, 0x80, 1]] {
        assert!(read_frame(&mut input.as_slice()).await.is_err());
    }
    let mut sink = tokio::io::sink();
    assert!(write_frame(&mut sink, &[4, 0x80]).await.is_err());
    assert!(inner_packet(&[3, 0, 0]).is_err());
    assert_eq!(inner_packet(&[4, 0, 1, 0, 0]).unwrap(), &[4, 0, 1, 0]);
}
