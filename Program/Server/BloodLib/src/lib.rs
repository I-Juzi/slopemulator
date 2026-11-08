//! Synchronous wire primitives for the legacy LobbyGate protocol.
//!
//! These ciphers reproduce a legacy format. The permissive OAEP decoder and
//! unauthenticated ECB mode are not suitable for new protocols.

pub mod crypto;
pub mod dispatch;
pub mod fragments;
pub mod frames;
pub mod packet;
pub mod session;
