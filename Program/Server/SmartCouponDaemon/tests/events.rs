mod common;
use common::*;
use smartcoupondaemon::np::{classify, wtoi64, NpReply};
use smartcoupondaemon::service::{DaemonConn, ManagerConn, Work};
use smartcoupondaemon::store::{LogStatus, PublicationStatus, Store};
use std::net::Ipv4Addr;

fn retry(tid: i64, acc: [u8; 16], publication: i64) -> Vec<u8> {
    use bloodlib::packet::Value;
    pkt(
        "hqoq",
        &[
            Value::I16(4),
            Value::I64(tid),
            Value::Id(acc),
            Value::I64(publication),
        ],
    )
}
fn manager(
    d: &smartcoupondaemon::service::Daemon<impl Store, impl smartcoupondaemon::np::Np>,
) -> ManagerConn {
    use bloodlib::packet::Value;
    let mut m = ManagerConn::default();
    let hello = pkt("hhh", &[Value::I16(1), Value::I16(2), Value::I16(3)]);
    assert_eq!(d.manager_packet(&mut m, &hello).replies, vec![h(2)]);
    m
}

#[tokio::test]
async fn game_event_publishes_through_np() {
    let d = daemon();
    d.store.add_rule(10001, 501);
    d.store.add_rule(10001, 502);
    d.np.push(Ok(NpReply::Success { accept_tid: 9001 }));
    d.np.push(Ok(NpReply::Error { code: 100 }));
    let mut c = DaemonConn::new(1, Ipv4Addr::LOCALHOST);
    d.daemon_packet(&mut c, &hello(1, 1, 1));
    let r = d.daemon_packet(&mut c, &by_character(1, 10001, ACC, 1, 77));
    let mut out = Vec::new();
    for w in r.work {
        out.extend(d.process(w).await);
    }
    assert!(
        out.is_empty(),
        "game-event jobs do not reply to the manager"
    );
    let sent = d.np.sent();
    assert_eq!(sent.len(), 2);
    assert_eq!(
        (sent[0].event_id_text(), sent[0].request_key_text()),
        ("501".into(), "1".into())
    );
    assert!(sent[0].reward_changeover);
    assert_eq!(d.store.log_status(1), Some(LogStatus::Processed));
    assert_eq!(
        d.store.publication_status(1),
        Some(PublicationStatus::Success { accept_tid: 9001 })
    );
    assert_eq!(
        d.store.publication_status(2),
        Some(PublicationStatus::Failure)
    );
}

#[tokio::test]
async fn process_failure_marks_log() {
    let d = daemon();
    let log_id = d
        .store
        .register_game_event_log(smartcoupondaemon::store::Target::Account, &ACC, 0, 0, 10002)
        .unwrap();
    d.store.set_log_status(log_id, LogStatus::Processed);
    let log = smartcoupondaemon::store::EventLog {
        log_id,
        target: smartcoupondaemon::store::Target::Account,
        account: ACC,
        world: 0,
        pcdbid: 0,
        event_id: 10002,
    };
    assert!(d.process(Work::GameEventLog(log)).await.is_empty());
    assert_eq!(d.store.log_status(log_id), Some(LogStatus::ProcessFailure));
    // The manager can now load it as retry-ready.
    let mut m = manager(&d);
    let r = d.manager_packet(&mut m, &hq(7, 44));
    assert_eq!(r.replies, vec![hq(8, 44)]);
    assert_eq!(r.work.len(), 1);
}

#[tokio::test]
async fn manager_retry_outcomes() {
    let d = daemon();
    let mut m = manager(&d);
    let ok = d.store.insert_publication(ACC, 600);
    let keep = d.store.insert_publication(ACC, 601);
    let expire = d.store.insert_publication(ACC, 602);
    d.np.push(Ok(NpReply::Success { accept_tid: 77 }));
    d.np.push(Ok(NpReply::Error { code: 42 }));
    d.np.push(Ok(NpReply::Expired));
    let mut run = |tid, publication| {
        let r = d.manager_packet(&mut m, &retry(tid, ACC, publication));
        assert!(r.replies.is_empty());
        r.work
    };
    let jobs = [run(10, ok), run(11, keep), run(12, expire), run(13, 999)];
    let mut out = Vec::new();
    for w in jobs.into_iter().flatten() {
        out.extend(d.process(w).await);
    }
    assert_eq!(
        out,
        vec![hqq(5, 10, 77), hqe(6, 11, 6), hqe(6, 12, 3), hqe(6, 13, 10)]
    );
    assert_eq!(
        d.store.publication_status(keep),
        Some(PublicationStatus::Ready)
    );
    // No NP channel left in the script: reason 5.
    let r = d.manager_packet(&mut m, &retry(14, ACC, keep));
    let out = d.process(r.work.into_iter().next().unwrap()).await;
    assert_eq!(out, vec![hqe(6, 14, 5)]);
    // DB failure: reason 4.
    d.store.set_failing(true);
    let r = d.manager_packet(&mut m, &retry(15, ACC, keep));
    let out = d.process(r.work.into_iter().next().unwrap()).await;
    assert_eq!(out, vec![hqe(6, 15, 4)]);
    assert_eq!(
        d.manager_packet(&mut m, &hq(7, 16)).replies,
        vec![hqe(9, 16, 4)]
    );
}

#[test]
fn manager_retry_parameter_checks() {
    let d = daemon();
    let mut m = manager(&d);
    for (tid, acc, publication) in [(0, ACC, 1), (1, [0; 16], 1), (1, ACC, 0)] {
        let r = d.manager_packet(&mut m, &retry(tid, acc, publication));
        assert_eq!(r.replies, vec![hqe(6, tid, 7)]);
    }
    assert_eq!(
        d.manager_packet(&mut m, &hq(4, 1)).replies,
        vec![hqe(6, 0, 2)]
    );
    assert_eq!(
        d.manager_packet(&mut m, &hq(7, 0)).replies,
        vec![hqe(9, 0, 7)]
    );
    assert_eq!(d.manager_packet(&mut m, &h(7)).replies, vec![hqe(9, 0, 2)]);
}

#[test]
fn startup_reloads_unhandled_work() {
    let d = daemon();
    d.store
        .register_game_event_log(smartcoupondaemon::store::Target::Account, &ACC, 0, 0, 10002)
        .unwrap();
    d.store.insert_publication(ACC, 700);
    let work = d.startup_work().unwrap();
    assert_eq!(work.len(), 2);
    assert!(matches!(
        work[1],
        Work::SendPublication {
            management_tid: 0,
            ..
        }
    ));
}

#[test]
fn np_response_classification() {
    assert_eq!(
        classify(200, "", 2851, "123", 0),
        Some(NpReply::Success { accept_tid: 123 })
    );
    assert_eq!(
        classify(400, "Success", 2852, " -5x", 0),
        Some(NpReply::Success { accept_tid: -5 })
    );
    assert_eq!(classify(200, "", 1, "1", 0), None);
    assert_eq!(
        classify(400, "Fail", 2851, "", 5809),
        Some(NpReply::Error { code: 5809 })
    );
    assert_eq!(wtoi64("+42abc"), 42);
    assert!(smartcoupondaemon::np::keeps_publication_status(30002));
    assert!(!smartcoupondaemon::np::keeps_publication_status(1));
}
