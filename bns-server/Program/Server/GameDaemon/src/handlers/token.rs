//! Verified state-1 subset of LobbyTokenAuthTran, build 381, 0x14014CCB0.
//! Construct this only after initialization 0x14014EE00 and pending-user indexing.
//! Later DB/recovery/charge states remain unsupported. No auth success is inferred.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenAuthRequest {
    pub world: u16,
    pub client_key: u64,
    pub lobby_token: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Registered,
    Sending,
    ConsumeToken,
    Finished,
}

pub struct LobbyTokenAuthTransaction {
    request: TokenAuthRequest,
    transaction_id: u64,
    accepted_address_bits: u32,
    phase: Phase,
}
impl LobbyTokenAuthTransaction {
    /// The adapter must complete registration and pending-user indexing first.
    /// The address comes from the accepted connection, never the request body.
    pub fn registered(
        request: TokenAuthRequest,
        transaction_id: u64,
        accepted_address_bits: u32,
    ) -> Self {
        Self {
            request,
            transaction_id,
            accepted_address_bits,
            phase: Phase::Registered,
        }
    }

    /// State 1/event 0. Sending Lobby 5 does not send client 24.
    pub fn start(&mut self) -> Outcome {
        if self.phase != Phase::Registered {
            return Outcome::unsupported("TODO ❓ token-auth start outside registered state");
        }
        self.phase = Phase::Sending;
        Outcome {
            daemon_requests: vec![DaemonRequest::LobbyConsumeToken {
                world: self.request.world,
                transaction_id: self.transaction_id,
                client_key: self.request.client_key,
                lobby_token: self.request.lobby_token,
                accepted_address_bits: self.accepted_address_bits,
            }],
            ..Outcome::unsupported("TODO ❓ Lobby consume-token send result")
        }
    }

    pub fn sent(&mut self, result: Gate) -> Outcome {
        if self.phase != Phase::Sending {
            return Outcome::unsupported("TODO ❓ token-auth send callback outside sending state");
        }
        match result {
            Gate::Unknown => Outcome::unsupported("TODO ❓ Lobby consume-token send result"),
            Gate::Fail => self.finish_failure(2),
            Gate::Pass => {
                self.phase = Phase::ConsumeToken;
                Outcome::unsupported("TODO ❓ waiting for consume-token response")
            }
        }
    }

    /// State 1/event 28 only. Do not reuse this mapping for later callbacks.
    pub fn consume_token_failed(&mut self, backend_reason: u16) -> Outcome {
        if self.phase != Phase::ConsumeToken {
            return Outcome::unsupported("TODO ❓ consume-token failure outside state 1");
        }
        let reason = match backend_reason {
            15 | 16 => 1939,
            17 => 1936,
            18..=20 => 2,
            _ => 1,
        };
        self.finish_failure(reason)
    }

    fn finish_failure(&mut self, reason: u16) -> Outcome {
        self.phase = Phase::Finished;
        let mut out = Outcome::reason(25, reason);
        out.changes.extend([
            StateChange::RemovePendingTokenUser(self.transaction_id),
            StateChange::TokenAuthFinished(self.transaction_id),
        ]);
        out
    }
}
