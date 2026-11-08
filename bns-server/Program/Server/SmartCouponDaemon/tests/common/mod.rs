#![allow(dead_code)]
use bloodlib::packet::{compose, Value};
use smartcoupondaemon::config::{DaemonConfig, GameEventConfig, GamedList};
use smartcoupondaemon::np::ScriptedNp;
use smartcoupondaemon::service::Daemon;
use smartcoupondaemon::store::MemoryStore;

pub const GAME_EVENTS: &str = "<?xml version='1.0' encoding='UTF-8'?>
<game-event-list>
\t<gamed>
\t\t<event id='10001' name='LevelUp15' type='2' enable='true'/>
\t\t<event id='10002' name='FirstLogin' type='1' enable='true'/>
\t</gamed>
</game-event-list>";

pub const GAMED_LIST: &str = "<?xml version='1.0' encoding='UTF-8'?>
<gamed-list>
\t<gamed wid='1' ip-address='127.0.0.1' internal-port='20100' />
\t<gamed wid='2' ip-address='10.63.10.2' internal-port='20100' />
</gamed-list>";

pub const ACC: [u8; 16] = [7; 16];

pub fn daemon() -> Daemon<MemoryStore, ScriptedNp> {
    Daemon::new(
        DaemonConfig::default(),
        GameEventConfig::parse(GAME_EVENTS).unwrap(),
        GamedList::parse(GAMED_LIST).unwrap(),
        MemoryStore::new(),
        ScriptedNp::default(),
    )
}

pub fn pkt(format: &str, values: &[Value]) -> Vec<u8> {
    compose(format, values).unwrap()
}
pub fn hello(version: i16, kind: i8, wid: i16) -> Vec<u8> {
    pkt(
        "hhQhh",
        &[
            Value::I16(1),
            Value::I16(version),
            Value::I8(kind),
            Value::I16(wid),
            Value::I16(20100),
        ],
    )
}
pub fn by_character(tid: i64, event: i32, acc: [u8; 16], world: i16, pcdbid: i64) -> Vec<u8> {
    pkt(
        "hqiohq",
        &[
            Value::I16(4),
            Value::I64(tid),
            Value::I32(event),
            Value::Id(acc),
            Value::I16(world),
            Value::I64(pcdbid),
        ],
    )
}
pub fn by_account(tid: i64, event: i32, acc: [u8; 16]) -> Vec<u8> {
    pkt(
        "hqio",
        &[
            Value::I16(7),
            Value::I64(tid),
            Value::I32(event),
            Value::Id(acc),
        ],
    )
}
pub fn hh(cmd: i16, reason: i16) -> Vec<u8> {
    pkt("hh", &[Value::I16(cmd), Value::I16(reason)])
}
pub fn h(cmd: i16) -> Vec<u8> {
    pkt("h", &[Value::I16(cmd)])
}
pub fn hqq(cmd: i16, a: i64, b: i64) -> Vec<u8> {
    pkt("hqq", &[Value::I16(cmd), Value::I64(a), Value::I64(b)])
}
pub fn hqe(cmd: i16, tid: i64, reason: i16) -> Vec<u8> {
    pkt(
        "hqE",
        &[Value::I16(cmd), Value::I64(tid), Value::I16(reason)],
    )
}
pub fn hq(cmd: i16, tid: i64) -> Vec<u8> {
    pkt("hq", &[Value::I16(cmd), Value::I64(tid)])
}
