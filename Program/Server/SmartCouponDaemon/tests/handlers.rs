mod common;
use common::*;
use smartcoupondaemon::config::DaemonId;
use smartcoupondaemon::service::{DaemonConn, ManagerConn, Work};
use smartcoupondaemon::store::{EventLog, LogStatus, Target};
use std::net::Ipv4Addr;

const LOCAL: Ipv4Addr = Ipv4Addr::LOCALHOST;

fn authed(
    d: &smartcoupondaemon::service::Daemon<
        impl smartcoupondaemon::store::Store,
        impl smartcoupondaemon::np::Np,
    >,
) -> DaemonConn {
    let mut c = DaemonConn::new(1, LOCAL);
    let r = d.daemon_packet(&mut c, &hello(1, 1, 1));
    assert_eq!(r.replies, vec![h(2)]);
    assert!(!r.close);
    c
}

#[test]
fn hello_success_registers_session() {
    let d = daemon();
    let c = authed(&d);
    assert_eq!(c.authed, Some(DaemonId { kind: 1, wid: 1 }));
    assert!(d.has_session(&DaemonId { kind: 1, wid: 1 }));
    d.disconnect(&c);
    assert!(!d.has_session(&DaemonId { kind: 1, wid: 1 }));
}

#[test]
fn hello_failure_reasons() {
    let d = daemon();
    let check = |pkt: Vec<u8>, ip: Ipv4Addr, reason: i16| {
        let mut c = DaemonConn::new(9, ip);
        let r = d.daemon_packet(&mut c, &pkt);
        assert_eq!(r.replies, vec![hh(3, reason)]);
        assert!(r.close);
        assert!(c.authed.is_none());
    };
    check(hello(2, 1, 1), LOCAL, 9);
    check(hello(1, 1, 5), LOCAL, 2);
    check(hello(1, 0, 1), LOCAL, 2);
    check(hello(1, 1, 2), LOCAL, 11);
    check(h(1), LOCAL, 2);
    d.set_service_available(false);
    check(hello(1, 1, 1), LOCAL, 8);
}

#[test]
fn duplicate_world_closes_both_sessions() {
    let d = daemon();
    let first = authed(&d);
    let mut second = DaemonConn::new(2, LOCAL);
    let r = d.daemon_packet(&mut second, &hello(1, 1, 1));
    assert_eq!(r.replies, vec![hh(3, 13)]);
    assert!(r.close);
    assert!(!d.has_session(&DaemonId { kind: 1, wid: 1 }));
    // The old connection was told to close; its disconnect is harmless.
    d.disconnect(&first);
    let mut third = DaemonConn::new(3, LOCAL);
    assert_eq!(
        d.daemon_packet(&mut third, &hello(1, 1, 1)).replies,
        vec![h(2)]
    );
}

#[test]
fn unauthed_and_repeated_hello_close_silently() {
    let d = daemon();
    let mut c = DaemonConn::new(1, LOCAL);
    let r = d.daemon_packet(&mut c, &by_account(1, 10002, ACC));
    assert!(r.close && r.replies.is_empty());
    let mut c = authed(&d);
    let r = d.daemon_packet(&mut c, &hello(1, 1, 1));
    assert!(r.close && r.replies.is_empty());
}

#[test]
fn by_character_success_posts_log() {
    let d = daemon();
    let mut c = authed(&d);
    let r = d.daemon_packet(&mut c, &by_character(55, 10001, ACC, 1, 900));
    assert_eq!(r.replies, vec![hqq(5, 55, 1)]);
    assert_eq!(
        r.work,
        vec![Work::GameEventLog(EventLog {
            log_id: 1,
            target: Target::Character,
            account: ACC,
            world: 1,
            pcdbid: 900,
            event_id: 10001,
        })]
    );
    assert_eq!(d.store.log_status(1), Some(LogStatus::Registered));
}

#[test]
fn by_character_parameter_checks() {
    let d = daemon();
    let mut c = authed(&d);
    for pkt in [
        by_character(1, 10001, [0; 16], 1, 900),
        by_character(1, 10001, ACC, 0, 900),
        by_character(1, 10001, ACC, 2, 900),
        by_character(1, 10001, ACC, 1, 0),
        by_character(1, 0, ACC, 1, 900),
        by_character(1, 777, ACC, 1, 900),
    ] {
        let r = d.daemon_packet(&mut c, &pkt);
        assert_eq!(r.replies, vec![hqe(6, 1, 7)]);
        assert!(!r.close && r.work.is_empty());
    }
    // A short request keeps the fully decoded tid (0x140008360).
    let r = d.daemon_packet(&mut c, &hq(4, 3));
    assert_eq!(r.replies, vec![hqe(6, 3, 2)]);
    let r = d.daemon_packet(&mut c, &h(4));
    assert_eq!(r.replies, vec![hqe(6, 0, 2)]);
    d.store.set_failing(true);
    let r = d.daemon_packet(&mut c, &by_character(4, 10001, ACC, 1, 900));
    assert_eq!(r.replies, vec![hqe(6, 4, 6)]);
}

#[test]
fn by_account_binds_zero_world_and_pcdbid() {
    let d = daemon();
    let mut c = authed(&d);
    let r = d.daemon_packet(&mut c, &by_account(8, 10002, ACC));
    assert_eq!(r.replies, vec![hqq(8, 8, 1)]);
    match &r.work[..] {
        [Work::GameEventLog(log)] => {
            assert_eq!((log.target, log.world, log.pcdbid), (Target::Account, 0, 0))
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        d.daemon_packet(&mut c, &by_account(9, 0, ACC)).replies,
        vec![hqe(9, 9, 7)]
    );
    assert_eq!(
        d.daemon_packet(&mut c, &by_account(9, 10002, [0; 16]))
            .replies,
        vec![hqe(9, 9, 7)]
    );
}

#[test]
fn manager_hello_and_gating() {
    let d = daemon();
    let mut m = ManagerConn::default();
    let r = d.manager_packet(&mut m, &hq(7, 1));
    assert!(r.close && r.replies.is_empty());
    let hello = pkt(
        "hhh",
        &[
            bloodlib::packet::Value::I16(1),
            bloodlib::packet::Value::I16(1),
            bloodlib::packet::Value::I16(1),
        ],
    );
    assert_eq!(d.manager_packet(&mut m, &hello).replies, vec![hh(3, 9)]);
    let hello = pkt(
        "hhh",
        &[
            bloodlib::packet::Value::I16(1),
            bloodlib::packet::Value::I16(2),
            bloodlib::packet::Value::I16(1),
        ],
    );
    assert_eq!(d.manager_packet(&mut m, &hello).replies, vec![h(2)]);
    assert!(m.hello);
    assert_eq!(d.manager_packet(&mut m, &hello).replies, vec![hh(3, 8)]);
    assert!(d.manager_packet(&mut m, &h(10)).close);
}

#[test]
fn manager_repeated_hello_with_wid_zero_passes() {
    use bloodlib::packet::Value;
    let d = daemon();
    let mut m = ManagerConn::default();
    let hello = pkt("hhh", &[Value::I16(1), Value::I16(2), Value::I16(0)]);
    assert_eq!(d.manager_packet(&mut m, &hello).replies, vec![h(2)]);
    // GetId() is 0, so the guard at 0x14000EF40 does not fire.
    assert_eq!(d.manager_packet(&mut m, &hello).replies, vec![h(2)]);
}
