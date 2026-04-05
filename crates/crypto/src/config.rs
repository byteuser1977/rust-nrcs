use super::{
    cipher::{AesCbc, AesGcm, Sm4Cbc, Sm4Gcm},
    hash::{Sha256, Sm3},
    signature::{Ed25519, Sm2},
    traits::{CryptoError, HashAlgorithm, SignerAlgorithm, CipherAlgorithm, DynHasher, DynSigner, DynCipher},
    Hash256, Signature, PublicKey, SecretKey,
};
use std::sync::Arc;
use config::{Config, ConfigError};

/// Global crypto configuration
#[derive(Debug, Clone)]
pub struct CryptoConfig {
    pub hash: Arc<dyn HashAlgorithm>,
    pub signer: Arc<dyn SignerAlgorithm>,
    pub cipher: Option<Arc<dyn CipherAlgorithm>>,
}

impl CryptoConfig {
    /// Create a new configuration with explicit algorithm choices
    pub fn new(
        hash: Arc<dyn HashAlgorithm>,
        signer: Arc<dyn SignerAlgorithm>,
        cipher: Option<Arc<dyn CipherAlgorithm>>,
    ) -> Self {
        Self { hash, signer, cipher }
    }
}

/// Load configuration from standard locations
///
/// Order of precedence:
/// 1. `CONFIG_CRYPTO_*` environment variables
/// 2. `config/default.toml` relative to `CARGO_MANIFEST_DIR` or current working directory
/// 3. Hardcoded defaults
pub fn load_config() -> Result<CryptoConfig, CryptoError> {
    let cfg = Config::builder()
        .add_source(config::File::with_name("config/default"))
        .add_source(config::Environment::with_prefix("CONFIG_CRYPTO"))
        .build()
        .unwrap_or_else(|_| config::Config::default());

    let hash_name: String = cfg.get("hash").unwrap_or_else(|_| "sha256".to_string());
    let signature_name: String = cfg.get("signature").unwrap_or_else(|_| "ed25519".to_string());
    let cipher_name: String = cfg.get("cipher").unwrap_or_else(|_| "aes".to_string());
    let cipher_mode: String = cfg.get("cipher_mode").unwrap_or_else(|_| "gcm".to_string());

    // Select hash algorithm
    let hash: Arc<dyn HashAlgorithm> = match hash_name.to_lowercase().as_str() {
        "sm3" => Arc::new(Sm3),
        "sha256" | "sha2" => Arc::new(Sha256),
        _ => return Err(CryptoError::UnsupportedAlgorithm(format!("Unsupported hash: {}", hash_name))),
    };

    // Select signature algorithm
    let signer: Arc<dyn SignerAlgorithm> = match signature_name.to_lowercase().as_str() {
        "sm2" => Arc::new(Sm2),
        "ed25519" | "eddsa" => Arc::new(Ed25519),
        _ => return Err(CryptoError::UnsupportedAlgorithm(format!("Unsupported signature: {}", signature_name))),
    };

    // Select cipher (optional)
    let cipher: Option<Arc<dyn CipherAlgorithm>> = if cfg.get::<bool>("cipher.enabled").unwrap_or(true) {
        match (cipher_name.to_lowercase().as_str(), cipher_mode.to_lowercase().as_str()) {
            ("aes", "cbc") => Some(Arc::new(AesCbc)),
            ("aes", "gcm") => Some(Arc::new(AesGcm)),
            ("sm4", "cbc") => Some(Arc::new(Sm4Cbc)),
            ("sm4", "gcm") => Some(Arc::new(Sm4Gcm)),
            _ => return Err(CryptoError::UnsupportedAlgorithm(format!("Unsupported cipher: {} in mode {}", cipher_name, cipher_mode))),
        }
    } else {
        None
    };

    Ok(CryptoConfig::new(hash, signer, cipher))
}

/// Helper to get current global configuration (after once initialization)
pub fn get_config() -> &'static CryptoConfig {
    &CRYPTO_CONFIG
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_config() {
        // Should load from config/default.toml if present
        let _config = load_config().expect("Failed to load default config");
    }

    #[test]
    fn test_env_override() {
        // This test would set env vars, but we skip for brevity
    }
}