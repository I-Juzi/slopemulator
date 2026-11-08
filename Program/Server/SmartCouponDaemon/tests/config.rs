mod common;
use smartcoupondaemon::config::*;

#[test]
fn shipped_game_event_sample() {
    let c = GameEventConfig::parse(common::GAME_EVENTS).unwrap();
    let e = c.get(10001).unwrap();
    assert_eq!(e.name, "LevelUp15");
    assert_eq!(e.event_type, 2);
    assert_eq!(e.daemon_type, DAEMON_TYPE_GAMED);
    assert!(e.enable);
}

#[test]
fn none_group_is_skipped_and_type_is_case_insensitive() {
    let xml = "<l><NONE><event id='1' name='a' type='1' enable='true'/></NONE>\
               <GameD><event id='2' name='b' type='1' enable='False'/></GameD></l>";
    let c = GameEventConfig::parse(xml).unwrap();
    assert!(c.get(1).is_none());
    assert!(!c.get(2).unwrap().enable);
}

#[test]
fn event_needs_all_attributes() {
    let xml = "<l><gamed><event id='1' name='a' type='1'/></gamed></l>";
    assert!(GameEventConfig::parse(xml).is_err());
}

#[test]
fn gamed_list_skips_unknown_and_incomplete() {
    let xml = "<gamed-list><gamed wid='1' ip-address='10.0.0.1' internal-port='20100'/>\
               <other wid='2' ip-address='10.0.0.2' internal-port='1'/>\
               <gamed wid='3' ip-address='' internal-port='1'/></gamed-list>";
    let l = GamedList::parse(xml).unwrap();
    assert_eq!(l.daemons.len(), 1);
    let d = l.get(&DaemonId { kind: 1, wid: 1 }).unwrap();
    assert_eq!(d.ip, "10.0.0.1".parse::<std::net::Ipv4Addr>().unwrap());
    assert_eq!(d.port, 20100);
}

#[test]
fn daemon_options() {
    let xml = "<?xml version='1.0'?><config module='smartcoupond'>\
        <option name='np-enabled' value='no'/>\
        <option name='manage-listen-port' value='32001'/>\
        <option name='daemon-listen-port' value='22001'/>\
        <option name='np-reward-changeover-enable' value='no'/>\
        <option name='init-job-count' value='4096'/></config>";
    let c = DaemonConfig::parse(xml).unwrap();
    assert!(!c.np_enabled);
    assert_eq!((c.manage_listen_port, c.daemon_listen_port), (32001, 22001));
    assert!(!c.np_reward_changeover_enable);
    assert_eq!(c.options["init-job-count"], "4096");
    assert_eq!(c.np_port, 6601);
}

#[test]
fn rejects_bad_xml() {
    assert!(parse("<a><b></a>").is_err());
    assert!(parse("<!DOCTYPE x><a/>").is_err());
    assert!(DaemonConfig::parse("<config module='other'/>").is_err());
    assert!(DaemonConfig::parse(
        "<config><option name='a' value='1'/><option name='a' value='2'/></config>"
    )
    .is_err());
}
