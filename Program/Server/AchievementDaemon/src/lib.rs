//! AchievementDaemon build 348 (spec: Docs/Server/AchievementDaemon).
//! GameDaemons report Pc events; the daemon keeps achievement registers,
//! completes steps, stores rows and sends data back to the cluster.
pub mod config;
pub mod event;
pub mod pc;
pub mod protocol;
pub mod rules;
pub mod schema;
pub mod server;
pub mod service;
pub mod store;
