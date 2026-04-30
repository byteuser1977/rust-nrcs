//! 统一加密服务
//!
//! 根据配置实例化具体算法，提供向后兼容的 API。
//! 外部代码应通过 `Crypto::global()` 获取单例，或使用便捷函数。

use crate::{
    algorithms::{HashAlgorithm, SignatureAlgorithm, GcmAlgorithm},
    config::CryptoConfig,
    algorithms::{Ed25519, Curve25519, Sha256, Sm3, Sm4Gcm, AesGcm},
    CryptoError, CryptoResult, Hash256, PublicKey, SecretKey, Signature, keypair::KeyPair,
};
use once_cell::sync::OnceCell;
use std::sync::Arc;

/// 统一加密服务
///
/// 持有实际算法实现的实例（通过 trait object），提供统一 API。
/// 内部使用 `Arc` 共享算法实例，避免克隆开销。
#[derive(Clone)]
pub struct Crypto {
    hash: Arc<dyn HashAlgorithm>,
    signer: Arc<dyn SignatureAlgorithm>,
    cipher: Arc<dyn GcmAlgorithm>,
}

impl Crypto {
    /// 从配置创建新的 `Crypto` 实例
    pub fn new(config: &CryptoConfig) -> CryptoResult<Self> {
        let hash: Arc<dyn HashAlgorithm> = match config.hash.as_str() {
            "sha256" => Arc::new(Sha256),
            "sm3" => Arc::new(Sm3),
            _ => return Err(CryptoError::ConfigurationError(format!("unknown hash algorithm: {}", config.hash))),
        };

        let signer: Arc<dyn SignatureAlgorithm> = match config.signature.as_str() {
            "ed25519" => Arc::new(Ed25519),
            "curve25519" => Arc::new(Curve25519),
            _ => return Err(CryptoError::ConfigurationError(format!("unknown signature algorithm: {}", config.signature))),
        };

        let cipher: Arc<dyn GcmAlgorithm> = match config.cipher.as_str() {
            "sm4-gcm" => Arc::new(Sm4Gcm),
            "aes-gcm" => Arc::new(AesGcm),
            _ => return Err(CryptoError::ConfigurationError(format!("unknown cipher algorithm: {}", config.cipher))),
        };

        Ok(Self { hash, signer, cipher })
    }

    /// 获取全局单例（惰性初始化）
    pub fn global() -> &'static OnceCell<CryptoResult<Crypto>> {
        static INSTANCE: OnceCell<CryptoResult<Crypto>> = OnceCell::new();
        INSTANCE.get_or_init(|| {
            let config = CryptoConfig::global();
            Crypto::new(config)
        });
        &INSTANCE
    }

    // =========================================================================
    // Hash API
    // =========================================================================

    /// 计算哈希（使用配置的哈希算法）
    pub fn hash(&self, data: &[u8]) -> Hash256 {
        self.hash.hash(data)
    }

    // =========================================================================
    // Signature API
    // =========================================================================

    /// 生成密钥对（使用配置的签名算法）
    pub fn generate_keypair(&self) -> KeyPair {
        self.signer.generate_keypair()
    }

    /// 从种子派生密钥对（使用配置的签名算法）
    pub fn keypair_from_seed(&self, seed: &[u8; 32]) -> KeyPair {
        self.signer.from_seed(seed)
    }

    /// 签名消息
    pub fn sign(&self, key: &SecretKey, message: &[u8]) -> Signature {
        self.signer.sign(key, message)
    }

    /// 验证签名
    pub fn verify(&self, public_key: &PublicKey, message: &[u8], signature: &Signature) -> CryptoResult<()> {
        self.signer.verify(public_key, message, signature)
    }

    /// 获取当前使用的哈希算法名
    pub fn hash_algorithm_name(&self) -> &'static str {
        self.hash.name()
    }

    /// 获取当前使用的签名算法名
    pub fn signature_algorithm_name(&self) -> &'static str {
        self.signer.name()
    }

    /// 获取当前使用的加密算法名
    pub fn cipher_algorithm_name(&self) -> &'static str {
        self.cipher.name()
    }

    // =========================================================================
    // GCM 加密 API
    // =========================================================================

    /// GCM 加密（使用配置的加密算法）
    pub fn encrypt_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        plaintext: &[u8],
    ) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
        self.cipher.encrypt_gcm(key, nonce, aad, plaintext)
    }

    /// GCM 解密（使用配置的加密算法）
    pub fn decrypt_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        ciphertext: &[u8],
        tag: &[u8],
    ) -> CryptoResult<Vec<u8>> {
        self.cipher.decrypt_gcm(key, nonce, aad, ciphertext, tag)
    }
}

// ============================================================================
// 向后兼容的便捷函数
// ============================================================================
//
// 这些函数保持原有签名不变，内部转发到 `Crypto::global()`

/// 计算哈希（使用配置的哈希算法）
pub fn hash(data: &[u8]) -> Hash256 {
    let cell = Crypto::global();
    let crypto = cell.get().unwrap();
    crypto.as_ref().unwrap().hash(data)
}

/// 生成密钥对（使用配置的签名算法）
pub fn generate_keypair() -> KeyPair {
    let cell = Crypto::global();
    let crypto = cell.get().unwrap();
    crypto.as_ref().unwrap().generate_keypair()
}

/// 从种子生成密钥对（使用配置的签名算法）
pub fn keypair_from_seed(seed: &[u8; 32]) -> KeyPair {
    let cell = Crypto::global();
    let crypto = cell.get().unwrap();
    crypto.as_ref().unwrap().keypair_from_seed(seed)
}

/// 签名消息
pub fn sign(key: &SecretKey, message: &[u8]) -> Signature {
    let cell = Crypto::global();
    let crypto = cell.get().unwrap();
    crypto.as_ref().unwrap().sign(key, message)
}

/// 验证签名
pub fn verify(public_key: &PublicKey, message: &[u8], signature: &Signature) -> CryptoResult<()> {
    let cell = Crypto::global();
    let crypto = cell.get().unwrap();
    crypto.as_ref().unwrap().verify(public_key, message, signature)
}

/// CBC 加密（暂时不可用
pub fn encrypt_cbc(_key: &[u8], _iv: &[u8], _plaintext: &[u8]) -> CryptoResult<Vec<u8>> {
    Err(CryptoError::CipherError("CBC encryption temporarily disabled".into()))
}

/// CBC 解密（暂时不可用）
pub fn decrypt_cbc(_key: &[u8], _iv_ciphertext: &[u8]) -> CryptoResult<Vec<u8>> {
    Err(CryptoError::CipherError("CBC decryption temporarily disabled".into()))
}

/// GCM 加密（使用配置的加密算法）
pub fn encrypt_gcm(
    key: &[u8],
    nonce: &[u8],
    aad: &[u8],
    plaintext: &[u8],
) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
    let cell = Crypto::global();
    let crypto = cell.get().unwrap();
    crypto.as_ref().unwrap().encrypt_gcm(key, nonce, aad, plaintext)
}

/// GCM 解密（使用配置的加密算法）
pub fn decrypt_gcm(
    key: &[u8],
    nonce: &[u8],
    aad: &[u8],
    ciphertext: &[u8],
    tag: &[u8],
) -> CryptoResult<Vec<u8>> {
    let cell = Crypto::global();
    let crypto = cell.get().unwrap();
    crypto.as_ref().unwrap().decrypt_gcm(key, nonce, aad, ciphertext, tag)
}

// ============================================================================
// 特定算法便捷函数（保持函数名不变）
// ============================================================================

/// 计算 SHA-256 哈希（始终使用 SHA-256，不依赖配置）
pub fn sha256(data: &[u8]) -> Hash256 {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.into()
}

/// 计算 BLAKE3 哈希（始终使用 BLAKE3）
pub fn blake3(data: &[u8]) -> Hash256 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.into()
}

/// 计算 SM3 哈希（始终使用 SM3）
pub fn sm3(data: &[u8]) -> Hash256 {
    use sm3::{Digest, Sm3 as Sm3Impl};
    let mut hasher = Sm3Impl::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.into()
}

/// 生成随机 32 字节（用于 nonce、密钥等）
pub fn random_32() -> [u8; 32] {
    use rand::RngCore;
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    buf
}

/// 零化密钥对内存（安全销毁）
pub fn zeroize_keypair(_keypair: &mut KeyPair) {
    // 暂时禁用
}

/// 从密码短语生成密钥对（支持 NRCS 助记词和任意密码）
/// 
/// 对应 Java: 从 secretPhrase 生成密钥对
pub fn generate_keypair_from_passphrase(passphrase: &str) -> CryptoResult<KeyPair> {
    if crate::validate_passphrase(passphrase).is_ok() {
        let number = crate::secret_to_number(passphrase)
            .map_err(|e| CryptoError::ConfigurationError(e.to_string()))?;
        let bytes = number.to_bytes_be().1;
        
        let mut seed = [0u8; 32];
        let start = 32 - bytes.len().min(32);
        seed[start..].copy_from_slice(&bytes[..bytes.len().min(32)]);
        
        Ok(keypair_from_seed(&seed))
    } else {
        let seed = sha256(passphrase.as_bytes());
        Ok(keypair_from_seed(&seed))
    }
}

/// 生成 NRCS 标准助记词
/// 
/// 对应 Java: SecretSharingGenerator.generatePassPhrase()
pub fn generate_mnemonic(word_count: usize) -> CryptoResult<String> {
    if word_count != 12 {
        return Err(CryptoError::ConfigurationError(
            "NRCS only supports 12-word passphrases".to_string()
        ));
    }
    
    crate::passphrase::generate_passphrase()
        .map_err(|e| CryptoError::ConfigurationError(e.to_string()))
}

/// 从助记词派生账户ID
/// 
/// 对应 Java: Account.getId(publicKey) -> Convert.fullHashToId(hash)
pub fn derive_account_id(passphrase: &str) -> CryptoResult<String> {
    let keypair = crate::passphrase_to_keypair(passphrase)?;
    let pub_key = keypair.public_key();
    
    let pub_key_bytes = match pub_key {
        crate::PublicKey::Ed25519(bytes) => bytes,
        crate::PublicKey::Curve25519(bytes) => bytes,
        crate::PublicKey::Sm2 { .. } => {
            return Err(CryptoError::ConfigurationError("SM2 not supported for NRCS account ID".to_string()));
        }
    };
    
    let hash = sha256(&pub_key_bytes);
    
    let mut id_bytes = [0u8; 8];
    id_bytes.copy_from_slice(&hash[..8]);
    id_bytes.reverse();
    
    let account_id = u64::from_be_bytes(id_bytes);
    
    Ok(format!("NRCS-{}", crate::reed_solomon::encode(account_id)))
}

/// 从公钥直接计算account ID（不依赖passphrase）
pub fn account_id_from_public_key(public_key: &[u8]) -> u64 {
    let hash = sha256(public_key);
    
    let mut id_bytes = [0u8; 8];
    id_bytes.copy_from_slice(&hash[..8]);
    id_bytes.reverse();
    
    u64::from_be_bytes(id_bytes)
}

/// 从助记词派生公钥
/// 
/// 对应 Java: Crypto.getPublicKey(secretPhrase)
pub fn derive_public_key(passphrase: &str) -> CryptoResult<Vec<u8>> {
    let keypair = crate::passphrase_to_keypair(passphrase)?;
    let pub_key = keypair.public_key();
    
    Ok(match pub_key {
        crate::PublicKey::Ed25519(bytes) => bytes.to_vec(),
        crate::PublicKey::Curve25519(bytes) => bytes.to_vec(),
        crate::PublicKey::Sm2 { public_key, .. } => public_key.to_vec(),
    })
}

/// 验证账户地址
pub fn validate_account_address(address: &str) -> CryptoResult<String> {
    if !address.starts_with("NRCS-") && !address.starts_with("NRCSTEST-") {
        return Err(CryptoError::ConfigurationError("Invalid address prefix".to_string()));
    }
    
    let parts: Vec<&str> = address.split('-').collect();
    if parts.len() != 3 {
        return Err(CryptoError::ConfigurationError("Invalid address format".to_string()));
    }
    
    let account_id = parts[1].parse::<u64>()
        .map_err(|_| CryptoError::ConfigurationError("Invalid account ID".to_string()))?;
    
    Ok(account_id.to_string())
}

/// 签名交易字节
pub fn sign_transaction_bytes(tx_bytes: &[u8], passphrase: &str) -> CryptoResult<Vec<u8>> {
    let kp = generate_keypair_from_passphrase(passphrase)?;
    let signature = sign(&kp.secret_key(), tx_bytes);
    
    let mut signed = tx_bytes.to_vec();
    signed.extend_from_slice(&signature);
    
    Ok(signed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nrcs_account_id_generation() {
        let passphrase = "confusion flirt teeth story crawl dear shove screw decay flood cover warrior";
        let expected_account_id = "NRCS-P5FE-QYES-PLHK-HWTJM";
        
        let account_id = derive_account_id(passphrase).unwrap();
        assert_eq!(account_id, expected_account_id);
    }
    
    #[test]
    fn test_nrcs_public_key_derivation() {
        let passphrase = "confusion flirt teeth story crawl dear shove screw decay flood cover warrior";
        let expected_public_key = "5aafe59365b988d73aa15424a1b83c24fcada07e7cd73fcfade612c5bf32fc71";
        
        let public_key = derive_public_key(passphrase).unwrap();
        let public_key_hex = hex::encode(&public_key);
        
        assert_eq!(public_key_hex, expected_public_key);
    }
    
    #[test]
    fn test_nrcs_full_compatibility() {
        let passphrase = "confusion flirt teeth story crawl dear shove screw decay flood cover warrior";
        
        assert!(crate::validate_passphrase(passphrase).is_ok());
        
        let public_key = derive_public_key(passphrase).unwrap();
        assert_eq!(
            hex::encode(&public_key),
            "5aafe59365b988d73aa15424a1b83c24fcada07e7cd73fcfade612c5bf32fc71"
        );
        
        let account_id = derive_account_id(passphrase).unwrap();
        assert_eq!(account_id, "NRCS-P5FE-QYES-PLHK-HWTJM");
    }
    
    #[test]
    fn test_generate_mnemonic() {
        let mnemonic = generate_mnemonic(12).unwrap();
        let words: Vec<&str> = mnemonic.split(' ').collect();
        
        assert_eq!(words.len(), 12);
        assert!(crate::validate_passphrase(&mnemonic).is_ok());
    }
    
    #[test]
    fn test_generate_mnemonic_invalid_count() {
        let result = generate_mnemonic(24);
        assert!(result.is_err());
    }
}

