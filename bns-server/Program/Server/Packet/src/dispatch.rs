//! Build-specific dispatch data and byte-preserving relay rewriting (spec 04).
//! Initialize once at process startup, before accepting sessions. A new build
//! can pass a replacement JSON file to `initialize_tables`; no code change is
//! needed. This module does not perform I/O, encryption, or session closure.

use bloodlib::packet::{packet_size, PacketError, MAX_PACKET};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};
use thiserror::Error;

pub const DEFAULT_TABLES: &str = include_str!("../tables/381.json");
static TABLES: OnceLock<Tables> = OnceLock::new();

#[derive(Debug, Error)]
pub enum TableError {
    #[error("invalid table data: {0}")]
    Json(#[from] serde_json::Error),
    #[error("duplicate or inconsistent dispatch entries")]
    InvalidEntries,
    #[error("dispatch tables already initialized")]
    AlreadyInitialized,
    #[error("initialize dispatch tables at startup first")]
    NotInitialized,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LobbyEntry {
    pub handler: String,
    pub fragment: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tables {
    pub build: String,
    pub version: u32,
    pub any_max: i16,
    pub client_max: i16,
    pub ping: i16,
    pub any_authed: BTreeMap<i16, String>,
    pub not_authed: BTreeMap<i16, String>,
    pub authed: BTreeMap<i16, String>,
    pub lobbyd: BTreeMap<i16, LobbyEntry>,
    pub bypass: BTreeSet<i16>,
    pub c2d: Vec<[u8; 2]>,
    pub d2c: Vec<[u8; 2]>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Route<'a> {
    Handler { name: &'a str, fragment: bool },
    Relay,
    Ping,
    Reject,
}
impl Tables {
    pub fn parse(json: &str) -> Result<Self, TableError> {
        let tables: Self = serde_json::from_str(json)?;
        for pairs in [&tables.c2d, &tables.d2c] {
            let unique: BTreeSet<_> = pairs.iter().map(|p| p[0]).collect();
            if unique.len() != pairs.len() {
                return Err(TableError::InvalidEntries);
            }
        }
        if tables.bypass.iter().any(|c| tables.lobbyd.contains_key(c))
            || tables
                .any_authed
                .keys()
                .any(|c| *c < 0 || *c > tables.any_max)
            || tables
                .not_authed
                .keys()
                .chain(tables.authed.keys())
                .any(|c| *c < 0 || *c > tables.client_max)
        {
            return Err(TableError::InvalidEntries);
        }
        Ok(tables)
    }

    pub fn client_route(&self, cmd: i16, authed: bool) -> Route<'_> {
        if cmd < 0 {
            return Route::Reject;
        }
        if cmd <= self.any_max {
            if let Some(name) = self.any_authed.get(&cmd) {
                return Route::Handler {
                    name,
                    fragment: false,
                };
            }
        }
        if !authed {
            return match self.not_authed.get(&cmd).filter(|_| cmd <= self.client_max) {
                Some(name) => Route::Handler {
                    name,
                    fragment: false,
                },
                None => Route::Reject,
            };
        }
        if cmd == self.ping {
            return Route::Ping;
        }
        match self.authed.get(&cmd).filter(|_| cmd <= self.client_max) {
            Some(name) if name == "relay" => Route::Relay,
            Some(name) => Route::Handler {
                name,
                fragment: false,
            },
            None => Route::Reject,
        }
    }

    pub fn lobbyd_route(&self, cmd: i16) -> Route<'_> {
        if self.bypass.contains(&cmd) {
            return Route::Relay;
        }
        match self.lobbyd.get(&cmd) {
            Some(entry) => Route::Handler {
                name: &entry.handler,
                fragment: entry.fragment,
            },
            None => Route::Reject,
        }
    }

    pub fn map_c2d(&self, cmd: i16) -> i16 {
        map(&self.c2d, cmd)
    }
    pub fn map_d2c(&self, cmd: i16) -> i16 {
        map(&self.d2c, cmd)
    }

    /// Call only after dispatch authorizes relay. In particular, the 41 -> 41
    /// map exists but the client table rejects command 41.
    /// Any error requires the caller to log and close the client session.
    pub fn relay_to_lobbyd(
        &self,
        packet: &[u8],
        cli_key: u64,
        connected: bool,
    ) -> Result<Vec<u8>, RelayError> {
        packet_size(packet, 4)?;
        if cli_key == 0 {
            return Err(RelayError::InvalidClientKey);
        }
        if !connected {
            return Err(RelayError::NotConnected);
        }
        let body = &packet[4..];
        if body.len() >= MAX_PACKET - 12 {
            return Err(RelayError::OversizedBody);
        }
        let cmd = i16::from_le_bytes(packet[2..4].try_into().unwrap());
        let mut output = ((body.len() + 12) as i16).to_le_bytes().to_vec();
        output.extend(self.map_c2d(cmd).to_le_bytes());
        output.extend(cli_key.to_le_bytes());
        output.extend(body);
        Ok(output)
    }

    /// Return the lookup key and plain client packet. The caller must drop a
    /// malformed packet or an absent client, without closing either link.
    /// Encrypt the returned packet as one client frame after client lookup.
    pub fn relay_to_client(&self, packet: &[u8]) -> Result<(u64, Vec<u8>), RelayError> {
        packet_size(packet, 12)?;
        let cmd = i16::from_le_bytes(packet[2..4].try_into().unwrap());
        let cli_key = u64::from_le_bytes(packet[4..12].try_into().unwrap());
        let body = &packet[12..];
        let mut output = ((body.len() + 4) as i16).to_le_bytes().to_vec();
        output.extend(self.map_d2c(cmd).to_le_bytes());
        output.extend(body);
        Ok((cli_key, output))
    }
}
fn map(pairs: &[[u8; 2]], cmd: i16) -> i16 {
    pairs
        .iter()
        .find(|pair| pair[0] == cmd as u8)
        .map_or(0, |pair| i16::from(pair[1]))
}

/// Parse and publish exactly once. Pass `DEFAULT_TABLES` for build 381, or the
/// text of a replacement data file. A second initialization is an error.
pub fn initialize_tables(json: &str) -> Result<&'static Tables, TableError> {
    if TABLES.get().is_some() {
        return Err(TableError::AlreadyInitialized);
    }
    let parsed = Tables::parse(json)?;
    TABLES
        .set(parsed)
        .map_err(|_| TableError::AlreadyInitialized)?;
    tables()
}
pub fn tables() -> Result<&'static Tables, TableError> {
    TABLES.get().ok_or(TableError::NotInitialized)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RelayError {
    #[error(transparent)]
    Packet(#[from] PacketError),
    #[error("invalid clikey")]
    InvalidClientKey,
    #[error("failed to send, not connected")]
    NotConnected,
    #[error("client relay body is at least 3988 bytes; log and close client")]
    OversizedBody,
}
