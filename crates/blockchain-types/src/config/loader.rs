//! 配置文件加载器
//!
//! 支持从TOML/JSON/YAML文件加载配置，支持环境变量覆盖

use super::GlobalConfig;
use super::chain::ChainConfig;
use super::p2p::P2PConfig;
use super::api::ApiConfig;
use super::crypto::CryptoConfig;

#[derive(Debug)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
    NotFound(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::NotFound(msg) => write!(f, "Config not found: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn load_config(config_path: Option<&str>) -> Result<GlobalConfig, Box<dyn std::error::Error + Send + Sync>> {
    let config_path = config_path.unwrap_or("config/nrcs");
    
    let chain = load_chain_config(config_path)?;
    let p2p = load_p2p_config(config_path)?;
    let api = load_api_config(config_path)?;
    let crypto = load_crypto_config(config_path)?;
    
    Ok(GlobalConfig { chain, p2p, api, crypto })
}

fn load_chain_config(config_path: &str) -> Result<ChainConfig, Box<dyn std::error::Error + Send + Sync>> {
    let config_file = format!("{}.toml", config_path);
    
    if let Ok(content) = std::fs::read_to_string(&config_file) {
        if let Ok(value) = toml::from_str::<toml::Value>(&content) {
            if let Some(chain) = value.get("chain") {
                return Ok(ChainConfig::from_toml(chain)?);
            }
        }
    }
    
    Ok(ChainConfig::default())
}

fn load_p2p_config(config_path: &str) -> Result<P2PConfig, Box<dyn std::error::Error + Send + Sync>> {
    let config_file = format!("{}.toml", config_path);
    
    if let Ok(content) = std::fs::read_to_string(&config_file) {
        if let Ok(value) = toml::from_str::<toml::Value>(&content) {
            if let Some(p2p) = value.get("p2p") {
                return Ok(P2PConfig::from_toml(p2p)?);
            }
        }
    }
    
    Ok(P2PConfig::default())
}

fn load_api_config(config_path: &str) -> Result<ApiConfig, Box<dyn std::error::Error + Send + Sync>> {
    let config_file = format!("{}.toml", config_path);
    
    if let Ok(content) = std::fs::read_to_string(&config_file) {
        if let Ok(value) = toml::from_str::<toml::Value>(&content) {
            if let Some(api) = value.get("api") {
                return Ok(ApiConfig::from_toml(api)?);
            }
        }
    }
    
    Ok(ApiConfig::default())
}

fn load_crypto_config(config_path: &str) -> Result<CryptoConfig, Box<dyn std::error::Error + Send + Sync>> {
    let config_file = format!("{}.toml", config_path);
    
    if let Ok(content) = std::fs::read_to_string(&config_file) {
        if let Ok(value) = toml::from_str::<toml::Value>(&content) {
            if let Some(crypto) = value.get("crypto") {
                return Ok(CryptoConfig::from_toml(crypto)?);
            }
        }
    }
    
    Ok(CryptoConfig::default())
}

fn get_env(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

fn get_env_bool(key: &str, default: bool) -> bool {
    get_env(key).map(|v| v.to_lowercase() == "true").unwrap_or(default)
}

fn get_env_u32(key: &str, default: u32) -> u32 {
    get_env(key).and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn get_env_u64(key: &str, default: u64) -> u64 {
    get_env(key).and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn get_env_usize(key: &str, default: usize) -> usize {
    get_env(key).and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn get_env_string(key: &str, default: &str) -> String {
    get_env(key).unwrap_or_else(|| default.to_string())
}
