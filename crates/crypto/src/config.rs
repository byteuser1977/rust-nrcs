//! 加密算法配置管理
//!
//! 从配置文件读取当前使用的算法，并实例化对应的算法对象。
//! 使用 `once_cell` 或 `lazy_static` 实现全局单例配置。

use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

/// 算法配置
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoConfig {
    /// 哈希算法: "sha256" (默认) 或 "sm3"
    #[serde(default = "default_hash")]
    pub hash: String,

    /// 签名算法: "ed25519" (默认) 或 "sm2"
    #[serde(default = "default_signature")]
    pub signature: String,

    /// 对称加密算法 (CBC): "aes" (默认) 或 "sm4"
    #[serde(default = "default_cipher")]
    pub cipher: String,

    /// GCM 模式: "aes-gcm" (默认) 或 "sm4-gcm"
    #[serde(default = "default_gcm")]
    pub gcm: String,

    /// AES 密钥长度 (仅当 cipher = "aes" 时有效): 128, 192, 256 (默认)
    #[serde(default = "default_aes_key_len")]
    pub aes_key_len: usize,
}

fn default_hash() -> String {
    "sha256".to_string()
}

fn default_signature() -> String {
    "ed25519".to_string()
}

fn default_cipher() -> String {
    "aes".to_string()
}

fn default_gcm() -> String {
    "aes-gcm".to_string()
}

fn default_aes_key_len() -> usize {
    256
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            hash: default_hash(),
            signature: default_signature(),
            cipher: default_cipher(),
            gcm: default_gcm(),
            aes_key_len: default_aes_key_len(),
        }
    }
}

/// 全局配置实例
///
/// 使用 `once_cell` 在第一次访问时初始化
use once_cell::sync::Lazy;

static GLOBAL_CONFIG: Lazy<CryptoConfig> = Lazy::new(|| {
    // 尝试从环境变量或配置文件加载
    // 这里先使用默认值，实际项目应集成 config crate
    match load_from_file() {
        Ok(cfg) => cfg,
        Err(_) => CryptoConfig::default(),
    }
});

/// 从文件加载配置
///
/// 查找顺序：
/// 1. `config/default.toml`（项目根目录）
/// 2. 环境变量 `NRC_CRYPTO_CONFIG`
fn load_from_file() -> Result<CryptoConfig, Box<dyn std::error::Error>> {
    // 使用 config crate 读取 TOML
    let mut config = config::Config::new();
    config.merge(config::File::with_name("config/default"))?;
    // 可选：环境变量覆盖
    config.merge(config::Environment::with_prefix("NRC_CRYPTO"))?;

    let cfg: CryptoConfig = config.try_deserialize()?;
    Ok(cfg)
}

impl CryptoConfig {
    /// 获取全局配置（只读）
    pub fn global() -> &'static CryptoConfig {
        &GLOBAL_CONFIG
    }

    /// 重新加载配置（主要用于测试）
    pub fn reload(cfg: CryptoConfig) {
        let mut config = GLOBAL_CONFIG.as_ref().clone();
        // 这里无法修改 Lazy，需要 UnsafeCell 或 RwLock
        // 简单起见，测试时通过重置 env 或使用 `set_config_for_test`
        unimplemented!("生产环境应使用 RwLock 包裹配置")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = CryptoConfig::default();
        assert_eq!(cfg.hash, "sha256");
        assert_eq!(cfg.signature, "ed25519");
        assert_eq!(cfg.cipher, "aes");
        assert_eq!(cfg.gcm, "aes-gcm");
        assert_eq!(cfg.aes_key_len, 256);
    }

    #[test]
    fn test_config_values() {
        // 验证密钥长度选择
        let key_len_map = vec![
            (128, 16),
            (192, 24),
            (256, 32),
        ];
        for (bits, bytes) in key_len_map {
            let cfg = CryptoConfig { aes_key_len: bits, ..Default::default() };
            assert_eq!(cfg.aes_key_len, bytes);
        }
    }
}
