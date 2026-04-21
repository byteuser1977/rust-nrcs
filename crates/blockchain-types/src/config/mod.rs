//! 全局配置管理 - 集中管理所有配置文件的导入
//!
//! 所有模块的配置通过此模块统一管理，避免配置分散

pub mod chain;
pub mod p2p;
pub mod api;
pub mod crypto;
pub mod loader;

use once_cell::sync::OnceCell;

use self::chain::ChainConfig;
use self::p2p::P2PConfig;
use self::api::ApiConfig;
use self::crypto::CryptoConfig;

pub static GLOBAL_CONFIG: OnceCell<GlobalConfig> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct GlobalConfig {
    pub chain: ChainConfig,
    pub p2p: P2PConfig,
    pub api: ApiConfig,
    pub crypto: CryptoConfig,
}

impl GlobalConfig {
    pub fn init(config_path: Option<&str>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let config = loader::load_config(config_path)?;
        GLOBAL_CONFIG.set(config).map_err(|_| "Config already initialized")?;
        Ok(())
    }
    
    pub fn get() -> &'static GlobalConfig {
        GLOBAL_CONFIG.get().expect("Config not initialized. Call GlobalConfig::init() first.")
    }
    
    pub fn is_initialized() -> bool {
        GLOBAL_CONFIG.get().is_some()
    }
    
    pub fn chain() -> &'static ChainConfig {
        &Self::get().chain
    }
    
    pub fn p2p() -> &'static P2PConfig {
        &Self::get().p2p
    }
    
    pub fn api() -> &'static ApiConfig {
        &Self::get().api
    }
    
    pub fn crypto() -> &'static CryptoConfig {
        &Self::get().crypto
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            chain: ChainConfig::default(),
            p2p: P2PConfig::default(),
            api: ApiConfig::default(),
            crypto: CryptoConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GlobalConfig::default();
        assert_eq!(config.chain.version, crate::constants::VERSION);
        assert!(config.p2p.max_connections > 0);
    }
}
