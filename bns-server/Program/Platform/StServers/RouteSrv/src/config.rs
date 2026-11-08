//! Route.ini settings. TLS and passwords are not used by this crate.
use std::{collections::HashMap, time::Duration};
use sts_core::ini::Ini;

/// Local allocation limit, not a traced protocol limit.
pub const MAX_BLOCK_BITS: u32 = 16;

#[derive(Clone, Debug)]
pub struct RouteConfig {
    pub reconnect_timeout: Duration,
    pub leftover_timeout: Duration,
    pub block_bits: u32,
    pub protocol_block_bits: HashMap<String, u32>,
}

impl Default for RouteConfig {
    fn default() -> Self {
        Self {
            reconnect_timeout: Duration::from_millis(30_000),
            leftover_timeout: Duration::from_millis(30_000),
            block_bits: 7,
            protocol_block_bits: HashMap::new(),
        }
    }
}

impl RouteConfig {
    /// Read numeric routing settings only. Never access TLS or password entries.
    pub fn from_ini(ini: &Ini) -> Result<Self, String> {
        let mut config = Self::default();
        for section in &ini.sections {
            for (key, value) in &section.entries {
                if key.eq_ignore_ascii_case("ReconnectTimeout")
                    || key.eq_ignore_ascii_case("LeftoverTimeout")
                {
                    let ms = value.parse::<u64>().map_err(|_| format!("invalid {key}"))?;
                    let timeout = Duration::from_millis(ms);
                    if key.eq_ignore_ascii_case("ReconnectTimeout") {
                        config.reconnect_timeout = timeout;
                    } else {
                        config.leftover_timeout = timeout;
                    }
                }
                if !section.name.to_ascii_lowercase().starts_with("network.") {
                    continue;
                }
                let (prefix, protocol) = key.split_once('.').unwrap_or((key, ""));
                if !prefix.eq_ignore_ascii_case("BlockMask") {
                    continue;
                }
                let bits = sts_core::xml::parse_u32(value)
                    .filter(|n| *n <= MAX_BLOCK_BITS)
                    .ok_or_else(|| format!("invalid {key}: expected 0..={MAX_BLOCK_BITS}"))?;
                if protocol.is_empty() {
                    config.block_bits = bits;
                } else {
                    config.protocol_block_bits.insert(protocol.to_owned(), bits);
                }
            }
        }
        Ok(config)
    }

    pub fn bits_for(&self, protocol: &str) -> u32 {
        self.protocol_block_bits
            .get(protocol)
            .copied()
            .unwrap_or(self.block_bits)
            .min(MAX_BLOCK_BITS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_settings_and_overrides() {
        let ini = Ini::parse("[Route]\nReconnectTimeout=12\nLeftoverTimeout=34\n[Network.Test]\nBlockMask=7\n[Network.Test.Test]\nBlockMask.Auth=3\n[TLS]\nEncryptedKey=keyhere\nPassword=keyhere\n");
        let c = RouteConfig::from_ini(&ini).unwrap();
        assert_eq!(c.reconnect_timeout.as_millis(), 12);
        assert_eq!(c.leftover_timeout.as_millis(), 34);
        assert_eq!((c.bits_for("Auth"), c.bits_for("Other")), (3, 7));
        assert!(RouteConfig::from_ini(&Ini::parse("[Network.Test]\nBlockMask=32")).is_err());
    }
}
