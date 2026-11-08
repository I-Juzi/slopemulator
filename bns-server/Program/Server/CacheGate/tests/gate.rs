use cachegate::config::{parse_cached_list, CachedInfo, GateConfig, SAMPLE_CACHED_LIST};
use cachegate::gate::{Flow, Gate, Out, Peer};
use cachegate::protocol::{self, cmd, reason, RelayHeader};
use std::net::Ipv4Addr;

const C1: Peer = Peer::Client(1);
const C2: Peer = Peer::Client(2);

fn row(wid: u16, port: u16) -> CachedInfo {
    CachedInfo {
        wid,
        ip: Ipv4Addr::new(10, 10, 110, 55),
        port,
    }
}

fn hello(version: i16, economy: i16, wid: u16) -> Vec<u8> {
    let mut f = vec![10, 0, 1, 0];
    f.extend_from_slice(&version.to_le_bytes());
    f.extend_from_slice(&economy.to_le_bytes());
    f.extend_from_slice(&wid.to_le_bytes());
    f
}

fn header(from: u16, to: u16) -> RelayHeader {
    RelayHeader {
        from_wid: from,
        to_wid: to,
        tid: 0x1122334455667788,
        hint: 7,
        reserved: 99,
    }
}

/// Gate with CacheD 1 ready and client wid 100 on session 1.
fn ready_gate() -> Gate {
    let mut g = Gate::new(10060);
    g.apply_cached_list(&[row(1, 20202)]);
    g.cached_connected(1, Ipv4Addr::new(10, 10, 110, 55), 20202);
    assert_eq!(
        g.handle(Peer::Cached(1), &protocol::hello_ok()).1,
        Flow::Keep
    );
    assert_eq!(g.handle(C1, &hello(4, 10, 100)).1, Flow::Keep);
    g
}

fn fragment(mark: i32, data: &[u8]) -> Vec<u8> {
    let mut f = Vec::new();
    f.extend_from_slice(&((8 + data.len()) as i16).to_le_bytes());
    f.extend_from_slice(&cmd::FRAGMENT_RELAY.to_le_bytes());
    f.extend_from_slice(&mark.to_le_bytes());
    f.extend_from_slice(data);
    f
}

#[test]
fn builders_match_native_sizes() {
    assert_eq!(protocol::hello(1), vec![10, 0, 1, 0, 4, 0, 10, 0, 1, 0]);
    assert_eq!(protocol::hello_ok(), vec![4, 0, 2, 0]);
    assert_eq!(protocol::hello_fail(5), vec![6, 0, 3, 0, 5, 0]);
    let fail = protocol::relay_fail(10060, &header(100, 1), reason::NO_TARGET);
    assert_eq!(fail.len(), 28);
    let back = RelayHeader::read(&fail, 4).unwrap();
    assert_eq!(
        back,
        RelayHeader {
            from_wid: 10060,
            to_wid: 100,
            ..header(100, 1)
        }
    );
    assert_eq!(protocol::i16_at(&fail, 26), Some(8));
}

#[test]
fn hello_ok_and_version_errors() {
    let mut g = Gate::new(0);
    let (out, flow) = g.handle(C1, &hello(4, 10, 5));
    assert_eq!(
        (out, flow),
        (vec![Out::Send(C1, protocol::hello_ok())], Flow::Keep)
    );
    assert_eq!(g.client(5), Some(C1));
    for bad in [hello(3, 10, 6), hello(4, 9, 6)] {
        let (out, flow) = g.handle(C2, &bad);
        assert_eq!(
            out,
            vec![Out::Send(C2, protocol::hello_fail(reason::VERSION))]
        );
        assert_eq!(flow, Flow::Close);
    }
    // Short frame: decompose fails, no reply.
    assert_eq!(g.handle(C2, &[6, 0, 1, 0, 4, 0]), (vec![], Flow::Close));
}

#[test]
fn hello_duplicate_wid_closes_both() {
    let mut g = Gate::new(0);
    g.handle(C1, &hello(4, 10, 5));
    let (out, flow) = g.handle(C2, &hello(4, 10, 5));
    assert_eq!(
        out,
        vec![
            Out::Send(C2, protocol::hello_fail(reason::DUPLICATE)),
            Out::Close(C1)
        ]
    );
    assert_eq!(flow, Flow::Close);
}

#[test]
fn hello_twice_on_one_session_is_system_error() {
    let mut g = Gate::new(0);
    g.handle(C1, &hello(4, 10, 5));
    let (out, flow) = g.handle(C1, &hello(4, 10, 6));
    assert_eq!(
        out,
        vec![Out::Send(C1, protocol::hello_fail(reason::SYSTEM))]
    );
    assert_eq!(flow, Flow::Close);
    assert_eq!(g.client(6), None);
}

#[test]
fn client_close_releases_wid() {
    let mut g = Gate::new(0);
    g.handle(C1, &hello(4, 10, 5));
    g.client_closed(1);
    assert_eq!(g.client(5), None);
    assert_eq!(g.handle(C2, &hello(4, 10, 5)).1, Flow::Keep);
}

#[test]
fn hello_ok_needs_cached_channel() {
    let mut g = Gate::new(0);
    assert_eq!(g.handle(C1, &protocol::hello_ok()).1, Flow::Close);
    assert_eq!(
        g.handle(Peer::Cached(9), &protocol::hello_ok()).1,
        Flow::Close
    );
}

#[test]
fn cached_connect_sends_hello_with_cached_wid() {
    let mut g = Gate::new(10060);
    g.apply_cached_list(&[row(1, 20202)]);
    let out = g.cached_connected(1, Ipv4Addr::new(10, 10, 110, 55), 20202);
    assert_eq!(out, vec![Out::Send(Peer::Cached(1), protocol::hello(1))]);
    assert!(!g.cached(1).unwrap().ready);
    // Verify fails on a changed address.
    let out = g.cached_connected(1, Ipv4Addr::new(10, 0, 0, 1), 20202);
    assert_eq!(out, vec![Out::Close(Peer::Cached(1))]);
}

#[test]
fn relay_client_to_cached_is_verbatim() {
    let mut g = ready_gate();
    let frame = protocol::relay(&header(100, 1), b"payload");
    let (out, flow) = g.handle(C1, &frame);
    assert_eq!(
        (out, flow),
        (vec![Out::Send(Peer::Cached(1), frame)], Flow::Keep)
    );
}

#[test]
fn relay_cached_to_client() {
    let mut g = ready_gate();
    let frame = protocol::relay(&header(1, 100), b"x");
    assert_eq!(
        g.handle(Peer::Cached(1), &frame).0,
        vec![Out::Send(C1, frame)]
    );
}

#[test]
fn relay_failures() {
    let mut g = ready_gate();
    // Unknown CacheD.
    let h = header(100, 2);
    let (out, flow) = g.handle(C1, &protocol::relay(&h, b""));
    assert_eq!(
        out,
        vec![Out::Send(
            C1,
            protocol::relay_fail(10060, &h, reason::NO_TARGET)
        )]
    );
    assert_eq!(flow, Flow::Keep);
    // CacheD not ready after close.
    g.cached_closed(1);
    let h = header(100, 1);
    let (out, _) = g.handle(C1, &protocol::relay(&h, b""));
    assert_eq!(
        out,
        vec![Out::Send(
            C1,
            protocol::relay_fail(10060, &h, reason::NOT_READY)
        )]
    );
    // Unknown client.
    let h = header(1, 200);
    let (out, _) = g.handle(Peer::Cached(1), &protocol::relay(&h, b""));
    assert_eq!(
        out,
        vec![Out::Send(
            Peer::Cached(1),
            protocol::relay_fail(10060, &h, reason::NO_TARGET)
        )]
    );
    // Short relay header.
    assert_eq!(g.handle(C1, &[8, 0, 4, 0, 1, 0, 1, 0]).1, Flow::Close);
}

#[test]
fn fail_commands_and_unknown_slots() {
    let mut g = ready_gate();
    assert_eq!(g.handle(C1, &protocol::hello_fail(2)).1, Flow::Close);
    assert_eq!(
        g.handle(C1, &protocol::relay_fail(0, &header(1, 2), 8)).1,
        Flow::Close
    );
    for c in [6i16, 7, 8] {
        let mut f = vec![4, 0];
        f.extend_from_slice(&c.to_le_bytes());
        assert_eq!(g.handle(C1, &f), (vec![], Flow::Keep));
    }
    for c in [0i16, 10, -1, 300] {
        let mut f = vec![4, 0];
        f.extend_from_slice(&c.to_le_bytes());
        assert_eq!(g.handle(C1, &f).1, Flow::Close);
    }
}

#[test]
fn fragment_chain_forwards_every_frame() {
    let mut g = ready_gate();
    let mut data = Vec::new();
    header(100, 1).write(&mut data);
    data.extend_from_slice(&[0xAB; 40]);
    let total = data.len() as i32;
    let f1 = fragment(total, &data[..30]);
    let f2 = fragment(-1, &data[30..50]);
    let f3 = fragment(0, &data[50..]);
    assert_eq!(g.handle(C1, &f1), (vec![], Flow::Keep));
    assert_eq!(g.handle(C1, &f2), (vec![], Flow::Keep));
    let (out, flow) = g.handle(C1, &f3);
    assert_eq!(flow, Flow::Keep);
    let to = Peer::Cached(1);
    assert_eq!(
        out,
        vec![Out::Send(to, f1), Out::Send(to, f2), Out::Send(to, f3)]
    );
}

#[test]
fn fragment_single_frame_completes_on_mark() {
    let mut g = ready_gate();
    let mut data = Vec::new();
    header(1, 100).write(&mut data);
    let f = fragment(data.len() as i32, &data);
    assert_eq!(g.handle(Peer::Cached(1), &f).0, vec![Out::Send(C1, f)]);
}

#[test]
fn fragment_errors() {
    let mut g = ready_gate();
    // Continuation without a start.
    assert_eq!(g.handle(C1, &fragment(-1, b"ab")).1, Flow::Close);
    assert_eq!(g.handle(C1, &fragment(0, b"ab")).1, Flow::Close);
    // A second start while a chain is open.
    assert_eq!(g.handle(C1, &fragment(100, b"ab")).1, Flow::Keep);
    assert_eq!(g.handle(C1, &fragment(100, b"ab")).1, Flow::Close);
    // The buffer is clear again: a new chain starts.
    assert_eq!(g.handle(C1, &fragment(100, b"ab")).1, Flow::Keep);
    // Routing failure: non-fragment RelayFail, chain cleared.
    let mut g = ready_gate();
    let mut data = Vec::new();
    let h = header(100, 7);
    h.write(&mut data);
    let (out, flow) = g.handle(C1, &fragment(data.len() as i32, &data));
    assert_eq!(
        out,
        vec![Out::Send(
            C1,
            protocol::relay_fail(10060, &h, reason::NO_TARGET)
        )]
    );
    assert_eq!(flow, Flow::Keep);
}

#[test]
fn cached_list_merge_rules() {
    let mut g = Gate::new(0);
    let c = g.apply_cached_list(&[row(1, 20202), row(2, 20203)]);
    assert_eq!((c.connect.len(), c.drop.len()), (2, 0));
    let c = g.apply_cached_list(&[row(1, 20202), row(2, 30000), row(3, 1)]);
    assert_eq!(c.connect, vec![row(2, 30000), row(3, 1)]);
    assert_eq!(c.drop, vec![2]);
    // Removed rows stay.
    let c = g.apply_cached_list(&[]);
    assert_eq!(c, Default::default());
    assert!(g.cached(1).is_some() && g.cached(3).is_some());
}

#[test]
fn cached_list_parse() {
    let xml = "<?xml version='1.0'?><cached-list>\
        <cached wid='1' ip-address='10.10.110.55' internal-port='20200' economy-port='20202'/>\
        <cached wid=' 2x' ip-address='bad' economy-port='-1'/>\
        </cached-list>";
    let rows = parse_cached_list(xml).unwrap();
    assert_eq!(rows[0], row(1, 20202));
    assert_eq!(
        rows[1],
        CachedInfo {
            wid: 2,
            ip: Ipv4Addr::BROADCAST,
            port: 0xFFFF
        }
    );
    assert!(parse_cached_list("<cached-list><Cached wid='1'/></cached-list>").is_err());
    assert_eq!(parse_cached_list(SAMPLE_CACHED_LIST).unwrap(), vec![]);
}

#[test]
fn gate_config_parse() {
    let xml = "<config module='cacheGate'>\
        <option name='root-path' value='..'/><option name='log-path' value='log'/>\
        <option name='config-path' value='cfg'/>\
        <option name='max-cached-session-count' value='1000'/>\
        <option name='max-public-session-count' value='100'/>\
        <option name='wid' value='10060'/></config>";
    let c = GateConfig::parse(xml).unwrap();
    assert_eq!(
        (c.wid, c.public_port, c.dump_type, c.worker_thread),
        (10060, 20800, 1, 0)
    );
    assert_eq!(
        (c.max_cached_session_count, c.max_public_session_count),
        (1000, 100)
    );
    let missing = xml.replace("<option name='max-public-session-count' value='100'/>", "");
    assert!(GateConfig::parse(&missing).is_err());
    assert!(GateConfig::parse(&xml.replace("cacheGate", "other")).is_err());
}

/// Hello @0x140004C00 has no session type check: a CacheD session that
/// sends Hello enters ClientManager. CacheDChannel::onClose does not
/// release it.
#[test]
fn cached_hello_registers_in_client_manager() {
    let mut g = ready_gate();
    let d1 = Peer::Cached(1);
    assert_eq!(
        g.handle(d1, &hello(4, 10, 200)),
        (vec![Out::Send(d1, protocol::hello_ok())], Flow::Keep)
    );
    assert_eq!(g.client(200), Some(d1));
    // A client Hello with the same wid closes the CacheD session.
    let (out, flow) = g.handle(C2, &hello(4, 10, 200));
    assert_eq!(
        out,
        vec![
            Out::Send(C2, protocol::hello_fail(reason::DUPLICATE)),
            Out::Close(d1),
        ]
    );
    assert_eq!(flow, Flow::Close);
    g.cached_closed(1);
    assert_eq!(g.client(200), Some(d1));
}
