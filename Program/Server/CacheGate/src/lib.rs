//! CacheGate 348 (`CacheGate.exe`): relays economy packets between game
//! clients (GameDaemons) on the public port and the CacheD servers listed
//! in cached-list.config.xml. Spec: Docs/Server/CacheGate/README.md.
pub mod config;
pub mod gate;
pub mod protocol;
pub mod server;
