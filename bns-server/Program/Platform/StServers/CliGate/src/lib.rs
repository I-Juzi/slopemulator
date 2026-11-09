//! CliGate phase 1: client sockets, SRP login through AuthSrv on the route bus,
//! session state, logout and idle timeout. Spec: Docs/Platform/StServers/11-cligate-protocol.md.

pub mod auth_cmd;
pub mod conn;
pub mod crypt;
pub mod gate;
pub mod session;
