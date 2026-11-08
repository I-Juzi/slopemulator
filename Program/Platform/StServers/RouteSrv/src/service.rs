//! Single-router service state. No multi-router consensus or transfers.
use crate::{
    config::MAX_BLOCK_BITS,
    relay::{unavailable, ConnId},
};
use std::collections::{BTreeMap, BTreeSet};
use sts_core::{
    route::{BlockSlot, Location, RouteCmd, RouteMessage, Target, Topology},
    StsError,
};

#[derive(Clone, Debug)]
pub(crate) struct Provider {
    pub conn: ConnId,
    pub location: Location,
}

pub(crate) struct Service {
    pub build: u32,
    pub topology: Topology,
    pub block_bits: u32,
    pub providers: BTreeMap<u32, Provider>,
    pub subscribers: BTreeSet<ConnId>,
    pub current: Vec<BlockSlot>,
    pub desired: Vec<BlockSlot>,
    next: usize,
    explicit_blocks: bool,
}

/// ❓ The spec does not state the key hash. FNV-1a is a local phase-1 rule.
/// Do not claim compatibility with the original key distribution.
pub fn key_block(key: &str, block_bits: u32) -> u32 {
    let hash = key.as_bytes().iter().fold(2_166_136_261u32, |h, byte| {
        (h ^ u32::from(*byte)).wrapping_mul(16_777_619)
    });
    hash & ((1u32 << block_bits.min(MAX_BLOCK_BITS)) - 1)
}

impl Service {
    pub fn new(block_bits: u32) -> Self {
        let slots = vec![BlockSlot(0); 1usize << block_bits.min(MAX_BLOCK_BITS)];
        Self {
            build: 0,
            topology: Topology::Invalid,
            block_bits: block_bits.min(MAX_BLOCK_BITS),
            providers: BTreeMap::new(),
            subscribers: BTreeSet::new(),
            current: slots.clone(),
            desired: slots,
            next: 0,
            explicit_blocks: false,
        }
    }

    pub fn metadata(&mut self, message: &RouteMessage, bits: u32) {
        self.build = message.build;
        self.topology = message.topology;
        if self.block_bits != bits {
            self.block_bits = bits;
            self.current = vec![BlockSlot(0); 1usize << bits];
            self.desired = self.current.clone();
            self.explicit_blocks = false;
        }
    }

    pub fn rebalance(&mut self) {
        if self.explicit_blocks {
            return;
        }
        let codes: Vec<_> = self.providers.keys().copied().collect();
        for (index, slot) in self.current.iter_mut().enumerate() {
            *slot = BlockSlot(if codes.is_empty() {
                0
            } else {
                codes[index % codes.len()]
            });
        }
        self.desired = self.current.clone();
    }

    pub fn remove(&mut self, conn: ConnId, location: Option<Location>) -> Vec<Location> {
        let mut removed = Vec::new();
        let mut codes = BTreeSet::new();
        self.providers.retain(|code, provider| {
            if provider.conn == conn && location.is_none_or(|l| l == provider.location) {
                removed.push(provider.location);
                codes.insert(*code);
                false
            } else {
                true
            }
        });
        for slot in self.current.iter_mut().chain(self.desired.iter_mut()) {
            if codes.contains(&slot.provider()) {
                *slot = BlockSlot(0);
            }
        }
        self.rebalance();
        removed
    }

    pub fn set_blocks(&mut self, desired: Vec<BlockSlot>) -> Result<(), StsError> {
        if desired.len() != 1usize << self.block_bits
            || desired
                .iter()
                .any(|slot| slot.provider() != 0 && !self.providers.contains_key(&slot.provider()))
        {
            // ❓ Validation line for Route command bodies is not in the spec.
            return Err(StsError::new(10, 2, 0));
        }
        // One router: desired ownership becomes current without an Xfer handshake.
        self.current = desired.clone();
        self.desired = desired;
        self.explicit_blocks = true;
        Ok(())
    }

    pub fn message(&self, cmd: RouteCmd, name: &str, provider: Option<Location>) -> RouteMessage {
        RouteMessage {
            cmd: Some(cmd),
            name: name.into(),
            provider,
            build: self.build,
            topology: self.topology,
            block_bits: self.block_bits,
            current: self.current.clone(),
            desired: self.desired.clone(),
            ..RouteMessage::default()
        }
    }

    pub fn resolve(&mut self, target: Option<&Target>) -> Result<ConnId, StsError> {
        let provider = match target {
            None => {
                let count = self.providers.len();
                if count == 0 {
                    return Err(unavailable());
                }
                let index = self.next % count;
                self.next = self.next.wrapping_add(1);
                self.providers.values().nth(index)
            }
            Some(Target::Direct(location)) => {
                self.providers.values().find(|p| p.location == *location)
            }
            Some(Target::Key { center: 0, key }) => {
                self.block_provider(key_block(key, self.block_bits))
            }
            Some(Target::Block {
                center: 0, block, ..
            }) => self.block_provider(*block),
            _ => None, // Forward and remote centers need phase 2.
        };
        provider.map(|p| p.conn).ok_or_else(unavailable)
    }

    fn block_provider(&self, block: u32) -> Option<&Provider> {
        self.providers
            .get(&self.current.get(block as usize)?.provider())
    }
}
