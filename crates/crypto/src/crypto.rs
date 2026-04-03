//! 统一加密服务
//!
//! 根据配置实例化具体算法，提供向后兼容的 API。
//! 外部代码应通过 `Crypto::global()` 获取单例，或使用便捷函数。

use crate::{
    algorithms::{CipherAlgorithm, GcmAlgorithm, HashAlgorithm, SignatureAlgorithm},
    config::CryptoConfig,
    CryptoError, CryptoResult, Hash256, PublicKey, SecretKey, Signature, keypair::KeyPair,
};
use once_cell::sync::OnceCell;
use std::sync::Arc;

/// 统一加密服务
///
/// 持有实际算法实现的实例（通过 trait object），提供统一 API。
/// 内部使用 `Arc` 共享算法实例，避免克隆开销。
#[derive(Debug, Clone)]
pub struct Crypto {
    hash: Arc<dyn HashAlgorithm>,
    signer: Arc<dyn SignatureAlgorithm>,
    cipher: Arc<dyn CipherAlgorithm>,
    gcm: Arc<dyn GcmAlgorithm>,
}

impl Crypto {
    /// 从配置创建新的 `Crypto` 实例
    pub fn new(config: &CryptoConfig) -> CryptoResult<Self> {
        let hash: Arc<dyn HashAlgorithm> = match config.hash.as_str() {
            "sha256" => Arc::new(impls::Sha256),
            "sm3" => Arc::new(impls::Sm3),
            _ => return Err(CryptoError::ConfigurationError(format!("unknown hash algorithm: {}", config.hash))),
        };

        let signer: Arc<dyn SignatureAlgorithm> = match config.signature.as_str() {
            "ed25519" => Arc::new(impls::Ed25519),
            "sm2" => Arc::new(impls::Sm2),
            _ => return Err(CryptoError::ConfigurationError(format!("unknown signature algorithm: {}", config.signature))),
        };

        let cipher: Arc<dyn CipherAlgorithm> = match config.cipher.as_str() {
            // "aes" => { /* AES-CBC temporarily disabled */ return Err(CryptoError::ConfigurationError("AES-CBC not available yet".into())) }
            "sm4" => Arc::new(impls::Sm4Cbc),
            _ => return Err(CryptoError::ConfigurationError(format!("unknown cipher algorithm: {}", config.cipher))),
        };

        let gcm: Arc<dyn GcmAlgorithm> = match config.gcm.as_str() {
            "aes-gcm" => Arc::new(impls::AesGcm::new(
                match config.aes_key_len {
                    128 => 16,
                    192 => 24,
                    256 => 32,
                    _ => 32,
                }
            )),
            "sm4-gcm" => Arc::new(impls::Sm4Gcm),
            _ => return Err(CryptoError::ConfigurationError(format!("unknown GCM algorithm: {}", config.gcm))),
        };

        Ok(Self { hash, signer, cipher, gcm })
    }

    /// 获取全局单例（惰性初始化）
    pub fn global() -> &'static CryptoResult<Self> {
        static INSTANCE: OnceCell<CryptoResult<Crypto>> = OnceCell::new();
        INSTANCE.get_or_init(|| {
            let config = CryptoConfig::global();
            Crypto::new(config)
        })
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

    // =========================================================================
    // Cipher API (CBC)
    // =========================================================================

    /// CBC 加密
    ///
    /// 返回 `iv || ciphertext`
    pub fn encrypt_cbc(&self, key: &[u8], iv: &[u8], plaintext: &[u8]) -> CryptoResult<Vec<u8>> {
        if key.len() != self.cipher.key_len() {
            return Err(CryptoError::CipherError(format!(
                "key length mismatch: expected {}, got {}",
                self.cipher.key_len(),
                key.len()
            )));
        }
        Ok(self.cipher.encrypt_cbc(key, iv, plaintext))
    }

    /// CBC 解密
    pub fn decrypt_cbc(&self, key: &[u8], iv_ciphertext: &[u8]) -> CryptoResult<Vec<u8>> {
        if key.len() != self.cipher.key_len() {
            return Err(CryptoError::CipherError(format!(
                "key length mismatch: expected {}, got {}",
                self.cipher.key_len(),
                key.len()
            )));
        }
        self.cipher.decrypt_cbc(key, iv_ciphertext)
            .map_err(|e| CryptoError::CipherError(e.to_string()))
    }

    // =========================================================================
    // GCM API
    // =========================================================================

    /// GCM 加密
    pub fn encrypt_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        plaintext: &[u8],
    ) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
        if key.len() != self.gcm.key_len() {
            return Err(CryptoError::CipherError(format!(
                "key length mismatch: expected {}, got {}",
                self.gcm.key_len(),
                key.len()
            )));
        }
        if nonce.len() != self.gcm.nonce_len() {
            return Err(CryptoError::CipherError(format!(
                "nonce length mismatch: expected {}, got {}",
                self.gcm.nonce_len(),
                nonce.len()
            )));
        }
        self.gcm.encrypt_gcm(key, nonce, aad, plaintext)
            .map_err(|e| CryptoError::CipherError(e.to_string()))
    }

    /// GCM 解密
    pub fn decrypt_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        ciphertext: &[u8],
        tag: &[u8],
    ) -> CryptoResult<Vec<u8>> {
        if key.len() != self.gcm.key_len() {
            return Err(CryptoError::CipherError(format!(
                "key length mismatch: expected {}, got {}",
                self.gcm.key_len(),
                key.len()
            )));
        }
        if nonce.len() != self.gcm.nonce_len() {
            return Err(CryptoError::CipherError(format!(
                "nonce length mismatch: expected {}, got {}",
                self.gcm.nonce_len(),
                nonce.len()
            )));
        }
        if tag.len() != self.gcm.tag_len() {
            return Err(CryptoError::CipherError(format!(
                "tag length mismatch: expected {}, got {}",
                self.gcm.tag_len(),
                tag.len()
            )));
        }
        self.gcm.decrypt_gcm(key, nonce, aad, ciphertext, tag)
            .map_err(|e| CryptoError::CipherError(e.to_string()))
    }

    /// 获取当前使用的哈希算法名
    pub fn hash_algorithm_name(&self) -> &'static str {
        self.hash.name()
    }

    /// 获取当前使用的签名算法名
    pub fn signature_algorithm_name(&self) -> &'static str {
        self.signer.name()
    }

    /// 获取当前使用的对称加密算法名（CBC）
    pub fn cipher_algorithm_name(&self) -> &'static str {
        self.cipher.name()
    }

    /// 获取当前使用的 GCM 算法名
    pub fn gcm_algorithm_name(&self) -> &'static str {
        self.gcm.name()
    }
}

// ============================================================================
// 向后兼容的便捷函数
// ============================================================================
//
// 这些函数保持原有签名不变，内部转发到 `Crypto::global()`

/// 计算哈希（使用配置的哈希算法）
pub fn hash(data: &[u8]) -> Hash256 {
    Crypto::global().unwrap().hash(data)
}

/// 生成密钥对（使用配置的签名算法）
pub fn generate_keypair() -> KeyPair {
    Crypto::global().unwrap().generate_keypair()
}

/// 从种子生成密钥对（使用配置的签名算法）
pub fn keypair_from_seed(seed: &[u8; 32]) -> KeyPair {
    Crypto::global().unwrap().keypair_from_seed(seed)
}

/// 签名消息
pub fn sign(key: &SecretKey, message: &[u8]) -> Signature {
    Crypto::global().unwrap().sign(key, message)
}

/// 验证签名
pub fn verify(public_key: &PublicKey, message: &[u8], signature: &Signature) -> CryptoResult<()> {
    Crypto::global().unwrap().verify(public_key, message, signature)
}

/// CBC 加密（返回 `iv || ciphertext`）
pub fn encrypt_cbc(key: &[u8], iv: &[u8], plaintext: &[u8]) -> CryptoResult<Vec<u8>> {
    Crypto::global().unwrap().encrypt_cbc(key, iv, plaintext)
}

/// CBC 解密
pub fn decrypt_cbc(key: &[u8], iv_ciphertext: &[u8]) -> CryptoResult<Vec<u8>> {
    Crypto::global().unwrap().decrypt_cbc(key, iv_ciphertext)
}

/// GCM 加密
pub fn encrypt_gcm(
    key: &[u8],
    nonce: &[u8],
    aad: &[u8],
    plaintext: &[u8],
) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
    Crypto::global().unwrap().encrypt_gcm(key, nonce, aad, plaintext)
}

/// GCM 解密
pub fn decrypt_gcm(
    key: &[u8],
    nonce: &[u8],
    aad: &[u8],
    ciphertext: &[u8],
    tag: &[u8],
) -> CryptoResult<Vec<u8>> {
    Crypto::global().unwrap().decrypt_gcm(key, nonce, aad, ciphertext, tag)
}

// ============================================================================
// 特定算法便捷函数（保持函数名不变）
// ============================================================================

/// 计算 SHA-256 哈希（始终使用 SHA-256，不依赖配置）
pub fn sha256(data: &[u8]) -> Hash256 {
    use impls::Sha256;
    Sha256.hash(data)
}

/// 计算 BLAKE3 哈希（始终使用 BLAKE3）
pub fn blake3(data: &[u8]) -> Hash256 {
    use blake3::Hasher;
    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.into()
}

/// 计算 SM3 哈希（始终使用 SM3）
pub fn sm3(data: &[u8]) -> Hash256 {
    use impls::Sm3;
    Sm3.hash(data)
}

/// 生成随机 32 字节（用于 nonce、密钥等）
pub fn random_32() -> [u8; 32] {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill(&mut buf);
    buf
}

/// 零化密钥对内存（安全销毁）
pub fn zeroize_keypair(keypair: &mut KeyPair) {
    match keypair {
        KeyPair::Ed25519(_) => {
            // Ed25519 keypair not auto zeroize, but could wipe secret manually if needed
        }
        KeyPair::Sm2(kp) => {
            kp.inner.0.zeroize();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_default() {
        let crypto = Crypto::global().unwrap();
        assert_eq!(crypto.hash_algorithm_name(), "sha256");
        assert_eq!(crypto.signature_algorithm_name(), "ed25519");
        assert!(crypto.cipher_algorithm_name().contains("aes"));
        assert!(crypto.gcm_algorithm_name().contains("aes"));
    }

    #[test]
    fn test_hash_functions() {
        let data = b"hello";
        let h1 = sha256(data);
        assert_eq!(h1.len(), 32);

        let h2 = blake3(data);
        assert_ne!(h1, h2);

        let h3 = sm3(data);
        assert_eq!(h3.len(), 32);
    }

    #[test]
    fn test_roundtrip_ed25519() {
        let kp = generate_keypair();
        let msg = b"test";
        let sig = sign(&kp.secret_key(), msg);
        assert!(verify(&kp.public_key(), msg, &sig).is_ok());
    }
}
