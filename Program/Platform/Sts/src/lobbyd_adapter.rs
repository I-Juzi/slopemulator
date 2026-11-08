//! Opt-in local NP adapter. Default LobbyDaemon behavior is unchanged.
//! Token consumption uses the authority only. Other operations use the explicit backend.
use crate::{
    password::{Binding, LoginError},
    tokens::TokenAuthority,
};
use lobbydaemon::{
    np::{Identity, Np, NpError, NpFuture},
    store::AccountId,
};
use std::{sync::Arc, time::Instant};

pub struct AuthorityNp<N> {
    authority: Arc<TokenAuthority>,
    binding: Binding,
    backend: N,
}
impl<N: Np> AuthorityNp<N> {
    pub fn new(
        authority: Arc<TokenAuthority>,
        binding: Binding,
        backend: N,
    ) -> Result<Self, LoginError> {
        if !binding.valid() {
            return Err(LoginError::InvalidInput);
        }
        Ok(Self {
            authority,
            binding,
            backend,
        })
    }
}
impl<N: Np> Np for AuthorityNp<N> {
    fn consume(&self, token: [u8; 16], portal: [u8; 16]) -> NpFuture<'_, Identity> {
        // Consumption occurs when polled, not when the future is constructed.
        Box::pin(async move {
            let identity = self
                .authority
                .consume(token, portal, &self.binding, Instant::now())
                .map_err(|e| match e {
                    LoginError::Unavailable => NpError::Unavailable,
                    _ => NpError::RejectedToken,
                })?;
            Ok(Identity {
                account: identity.account_id,
                login_name: identity.login_name.encode_utf16().collect(),
                user_name: identity.user_name.encode_utf16().collect(),
                slot_id: identity.slot_id,
                profile: vec![],
                grades: vec![],
            })
        })
    }
    fn create_initial_user_name(
        &self,
        account: AccountId,
        name: Vec<u16>,
    ) -> lobbydaemon::gamed::GameFuture<'_, ()> {
        self.backend.create_initial_user_name(account, name)
    }
    fn login_presence(&self, account: AccountId, owner: u64) -> NpFuture<'_, ()> {
        self.backend.login_presence(account, owner)
    }
    fn logout_presence(&self, account: AccountId, owner: u64) {
        self.backend.logout_presence(account, owner);
    }
    fn world_entry(
        &self,
        account: AccountId,
        pcid: u64,
        slot: [u8; 16],
        ipv4: u32,
    ) -> lobbydaemon::gamed::GameFuture<'_, lobbydaemon::gamed::ChargeMode> {
        self.backend.world_entry(account, pcid, slot, ipv4)
    }
    fn expire(&self) {
        self.authority.expire(Instant::now());
        self.backend.expire();
    }
}
