//! 加密配置
//!
//! 对应 Java: nrcs-default.properties 中的加密配置

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub hash: String,
    pub signature: String,
    pub cipher: String,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            hash: "sha256".to_string(),
            signature: "ed25519".to_string(),
            cipher: "sm4-gcm".to_string(),
        }
    }
}

impl CryptoConfig {
    pub fn from_toml(value: &toml::Value) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut config = Self::default();
        
        if let Some(hash) = value.get("hash").and_then(|v| v.as_str()) {
            config.hash = hash.to_string();
        }
        if let Some(signature) = value.get("signature").and_then(|v| v.as_str()) {
            config.signature = signature.to_string();
        }
        if let Some(cipher) = value.get("cipher").and_then(|v| v.as_str()) {
            config.cipher = cipher.to_string();
        }
        
        Ok(config)
    }
    
    pub fn is_sha256(&self) -> bool {
        self.hash == "sha256"
    }
    
    pub fn is_sm3(&self) -> bool {
        self.hash == "sm3"
    }
    
    pub fn is_ed25519(&self) -> bool {
        self.signature == "ed25519"
    }
    
    pub fn is_sm2(&self) -> bool {
        self.signature == "sm2"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_crypto_config() {
        let config = CryptoConfig::default();
        assert_eq!(config.hash, "sha256");
        assert_eq!(config.signature, "ed25519");
        assert!(config.is_sha256());
        assert!(config.is_ed25519());
        assert!(!config.is_sm3());
        assert!(!config.is_sm2());
    }
}
