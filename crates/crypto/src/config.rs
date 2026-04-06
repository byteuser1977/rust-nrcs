//! 加密算法配置管理
//!
//! 提供默认的加密算法配置，简化实现，暂时不读取外部配置文件。

use serde::{Deserialize};

/// 算法配置
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoConfig {
    /// 哈希算法: "sha256" (默认) 或 "sm3"
    pub hash: String,

    /// 签名算法: "ed25519" (默认) 或 "sm2"
    pub signature: String,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            hash: "sha256".to_string(),
            signature: "ed25519".to_string(),
        }
    }
}

/// 全局配置实例
///
/// 使用 `once_cell` 在第一次访问时初始化
use once_cell::sync::Lazy;

static GLOBAL_CONFIG: Lazy<CryptoConfig> = Lazy::new(|| {
    CryptoConfig::default()
});

impl CryptoConfig {
    /// 获取全局配置（只读）
    pub fn global() -> &'static CryptoConfig {
        &GLOBAL_CONFIG
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
    }
}
