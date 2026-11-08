//! SmartCouponDaemon build 348 (spec: Docs/Server/SmartCouponDaemon).
//! GameDaemons report game events; the daemon records them, asks the store
//! for coupon publications and sends each publication to the NP service.
pub mod config;
pub mod np;
pub mod protocol;
pub mod server;
pub mod service;
pub mod store;
