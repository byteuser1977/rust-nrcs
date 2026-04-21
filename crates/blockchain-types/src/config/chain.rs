//! 链配置
//!
//! 对应 Java: nrcs-default.properties 中的链相关配置

use serde::{Deserialize, Serialize};
use crate::constants::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub version: String,
    pub is_testnet: bool,
    pub max_rollback: u32,
    pub guaranteed_balance_confirmations: u32,
    pub leasing_delay: u32,
    pub forging_delay: u32,
    pub forging_speedup: u32,
    pub batch_commit_size: u32,
    pub max_prunable_lifetime: u32,
    pub enable_pruning: bool,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            version: VERSION.to_string(),
            is_testnet: false,
            max_rollback: MAX_ROLLBACK,
            guaranteed_balance_confirmations: GUARANTEED_BALANCE_CONFIRMATIONS,
            leasing_delay: LEASING_DELAY,
            forging_delay: FORGING_DELAY,
            forging_speedup: FORGING_SPEEDUP,
            batch_commit_size: BATCH_COMMIT_SIZE,
            max_prunable_lifetime: MIN_PRUNABLE_LIFETIME,
            enable_pruning: true,
        }
    }
}

impl ChainConfig {
    pub fn from_toml(value: &toml::Value) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut config = Self::default();
        
        if let Some(version) = value.get("version").and_then(|v| v.as_str()) {
            config.version = version.to_string();
        }
        if let Some(is_testnet) = value.get("is_testnet").and_then(|v| v.as_bool()) {
            config.is_testnet = is_testnet;
        }
        if let Some(max_rollback) = value.get("max_rollback").and_then(|v| v.as_integer()) {
            config.max_rollback = max_rollback as u32;
        }
        if let Some(guaranteed_balance_confirmations) = value.get("guaranteed_balance_confirmations").and_then(|v| v.as_integer()) {
            config.guaranteed_balance_confirmations = guaranteed_balance_confirmations as u32;
        }
        if let Some(leasing_delay) = value.get("leasing_delay").and_then(|v| v.as_integer()) {
            config.leasing_delay = leasing_delay as u32;
        }
        if let Some(forging_delay) = value.get("forging_delay").and_then(|v| v.as_integer()) {
            config.forging_delay = forging_delay as u32;
        }
        if let Some(forging_speedup) = value.get("forging_speedup").and_then(|v| v.as_integer()) {
            config.forging_speedup = forging_speedup as u32;
        }
        
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_chain_config() {
        let config = ChainConfig::default();
        assert_eq!(config.version, VERSION);
        assert!(!config.is_testnet);
        assert_eq!(config.max_rollback, MAX_ROLLBACK);
    }
}
