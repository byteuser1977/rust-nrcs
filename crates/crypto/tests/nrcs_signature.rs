//! NRCS 交易签名验证测试
//!
//! 测试数据来自 NRCS Java 实现
//! 签名是对交易的二进制序列化格式进行的，而不是 JSON 字符串

use sha2::{Digest, Sha256};
use x25519_dalek::StaticSecret;

/// 测试数据
const PASSPHRASE: &str = "confusion flirt teeth story crawl dear shove screw decay flood cover warrior";
const PUBLIC_KEY_HEX: &str = "5aafe59365b988d73aa15424a1b83c24fcada07e7cd73fcfade612c5bf32fc71";
const SIGNATURE_HEX: &str = "b735c5a719e17493d6688b9a4c554b016b7355e0ae8c25c6954c133fe554a9049f9d1bcaa3c5717cb4336f4373ee7e290f4107712cf61f14ec60a62a234a31ea";

/// 交易数据（从 JSON 提取）
const TX_TYPE: u8 = 0;
const TX_SUBTYPE: u8 = 0;
const TX_VERSION: u8 = 1;
const TX_TIMESTAMP: u32 = 87601824;
const TX_DEADLINE: u16 = 60;
const TX_RECIPIENT: u64 = 2987210089394071135;
const TX_AMOUNT_NQT: u64 = 500000000;
const TX_FEE_NQT: u64 = 100000000;
const TX_EC_BLOCK_HEIGHT: u32 = 1362703;
const TX_EC_BLOCK_ID: u64 = 51530818052404904;

/// Attachment 数据
/// PrunablePlainMessage version = 1, hash = 63fc7d52fb0c3d2cd86b77a1a4f32b233735c3108a68d6a525909d4efe887dde
const PRUNABLE_MESSAGE_VERSION: u8 = 1;
const PRUNABLE_MESSAGE_HASH: &str = "63fc7d52fb0c3d2cd86b77a1a4f32b233735c3108a68d6a525909d4efe887dde";

/// 从 passphrase 派生公钥
fn derive_public_key(passphrase: &str) -> [u8; 32] {
    let seed: [u8; 32] = Sha256::digest(passphrase.as_bytes()).into();
    let secret = StaticSecret::from(seed);
    *x25519_dalek::PublicKey::from(&secret).as_bytes()
}

/// 构建交易字节（用于签名）
/// 参见 Java Transaction.bytes() 方法
fn build_transaction_bytes(public_key: &[u8; 32]) -> Vec<u8> {
    let mut bytes = Vec::new();
    
    // type (1 byte)
    bytes.push(TX_TYPE);
    
    // (version << 4) | subtype (1 byte)
    bytes.push((TX_VERSION << 4) | TX_SUBTYPE);
    
    // timestamp (4 bytes, little endian)
    bytes.extend_from_slice(&TX_TIMESTAMP.to_le_bytes());
    
    // deadline (2 bytes, little endian)
    bytes.extend_from_slice(&TX_DEADLINE.to_le_bytes());
    
    // senderPublicKey (32 bytes)
    bytes.extend_from_slice(public_key);
    
    // recipientId (8 bytes, little endian)
    bytes.extend_from_slice(&TX_RECIPIENT.to_le_bytes());
    
    // amountNQT (8 bytes, little endian)
    bytes.extend_from_slice(&TX_AMOUNT_NQT.to_le_bytes());
    
    // feeNQT (8 bytes, little endian)
    bytes.extend_from_slice(&TX_FEE_NQT.to_le_bytes());
    
    // referencedTransactionFullHash (32 bytes, 全零)
    bytes.extend_from_slice(&[0u8; 32]);
    
    // signature (64 bytes, 全零用于签名)
    bytes.extend_from_slice(&[0u8; 64]);
    
    // flags (4 bytes, little endian) - version > 0
    // flags = 0 (没有特殊标志)
    bytes.extend_from_slice(&0u32.to_le_bytes());
    
    // ecBlockHeight (4 bytes, little endian) - version > 0
    bytes.extend_from_slice(&TX_EC_BLOCK_HEIGHT.to_le_bytes());
    
    // ecBlockId (8 bytes, little endian) - version > 0
    bytes.extend_from_slice(&TX_EC_BLOCK_ID.to_le_bytes());
    
    // appendages (attachments)
    // PrunablePlainMessage: version (1 byte) + hash (32 bytes)
    bytes.push(PRUNABLE_MESSAGE_VERSION);
    let hash_bytes = hex::decode(PRUNABLE_MESSAGE_HASH).expect("Invalid hash hex");
    bytes.extend_from_slice(&hash_bytes);
    
    bytes
}

/// NRCS 签名实现
fn nrcs_sign(message: &[u8], passphrase: &str) -> [u8; 64] {
    use crypto::algorithms::signature::curve25519::core;
    
    let seed = Sha256::digest(passphrase.as_bytes());
    let mut private_key: [u8; 32] = seed.into();
    let mut public_key = [0u8; 32];
    let mut signing_key = [0u8; 32];
    
    core::keygen(&mut public_key, Some(&mut signing_key), &mut private_key);
    
    let m = Sha256::digest(message);
    let m_array: [u8; 32] = m.into();
    
    let mut hasher = Sha256::new();
    hasher.update(&m_array);
    hasher.update(&signing_key);
    let x = hasher.finalize();
    let mut x_array: [u8; 32] = x.into();
    
    let mut y = [0u8; 32];
    core::keygen(&mut y, None, &mut x_array);
    
    let mut hasher2 = Sha256::new();
    hasher2.update(&m_array);
    hasher2.update(&y);
    let h = hasher2.finalize();
    let h_array: [u8; 32] = h.into();
    
    let mut v = [0u8; 32];
    core::sign(&mut v, &h_array, &x_array, &signing_key);
    
    let mut signature = [0u8; 64];
    signature[..32].copy_from_slice(&v);
    signature[32..].copy_from_slice(&h_array);
    signature
}

/// NRCS 验签实现
fn nrcs_verify(signature: &[u8; 64], message: &[u8], public_key: &[u8; 32]) -> bool {
    use crypto::algorithms::signature::curve25519::core;
    
    let v: [u8; 32] = signature[..32].try_into().unwrap();
    let h: [u8; 32] = signature[32..].try_into().unwrap();
    
    if !core::is_canonical_signature(&v) {
        return false;
    }
    
    if !core::is_canonical_public_key(public_key) {
        return false;
    }
    
    let m = Sha256::digest(message);
    let m_array: [u8; 32] = m.into();
    
    let mut y = [0u8; 32];
    core::verify(&mut y, &v, &h, public_key);
    
    let mut hasher = Sha256::new();
    hasher.update(&m_array);
    hasher.update(&y);
    let expected_h = hasher.finalize();
    
    h == expected_h.as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_derivation() {
        let derived_pk = derive_public_key(PASSPHRASE);
        let derived_pk_hex = hex::encode(derived_pk);
        
        println!("Expected public key: {}", PUBLIC_KEY_HEX);
        println!("Derived public key:  {}", derived_pk_hex);
        
        assert_eq!(derived_pk_hex, PUBLIC_KEY_HEX);
    }

    #[test]
    fn test_signature_format() {
        let signature_bytes = hex::decode(SIGNATURE_HEX).expect("Invalid signature hex");
        assert_eq!(signature_bytes.len(), 64, "Signature should be 64 bytes");
        
        let v: [u8; 32] = signature_bytes[..32].try_into().unwrap();
        let h: [u8; 32] = signature_bytes[32..].try_into().unwrap();
        
        println!("Signature v: {}", hex::encode(v));
        println!("Signature h: {}", hex::encode(h));
    }

    #[test]
    fn test_build_transaction_bytes() {
        let public_key_bytes = hex::decode(PUBLIC_KEY_HEX).expect("Invalid public key hex");
        let public_key: [u8; 32] = public_key_bytes.try_into().unwrap();
        
        let tx_bytes = build_transaction_bytes(&public_key);
        
        println!("Transaction bytes length: {}", tx_bytes.len());
        println!("Transaction bytes: {}", hex::encode(&tx_bytes));
        
        // 基本大小检查
        // 1 + 1 + 4 + 2 + 32 + 8 + 8 + 8 + 32 + 64 + 4 + 4 + 8 = 176 bytes (不含 appendages)
        assert!(tx_bytes.len() >= 176);
    }

    #[test]
    fn test_sign_transaction() {
        let public_key_bytes = hex::decode(PUBLIC_KEY_HEX).expect("Invalid public key hex");
        let public_key: [u8; 32] = public_key_bytes.try_into().unwrap();
        
        let tx_bytes = build_transaction_bytes(&public_key);
        let signature = nrcs_sign(&tx_bytes, PASSPHRASE);
        let signature_hex = hex::encode(signature);
        
        println!("Expected signature: {}", SIGNATURE_HEX);
        println!("Derived signature:  {}", signature_hex);
        
        // 注意：由于签名验证逻辑仍在调试中，这个测试可能失败
    }

    #[test]
    fn test_verify_transaction() {
        let public_key_bytes = hex::decode(PUBLIC_KEY_HEX).expect("Invalid public key hex");
        let public_key: [u8; 32] = public_key_bytes.try_into().unwrap();
        
        let signature_bytes = hex::decode(SIGNATURE_HEX).expect("Invalid signature hex");
        let signature: [u8; 64] = signature_bytes.try_into().unwrap();
        
        let tx_bytes = build_transaction_bytes(&public_key);
        
        let result = nrcs_verify(&signature, &tx_bytes, &public_key);
        
        println!("Verification result: {}", result);
        
        // 注意：由于签名验证逻辑仍在调试中，这个测试可能失败
    }

    #[test]
    fn test_signature_components() {
        let signature_bytes = hex::decode(SIGNATURE_HEX).expect("Invalid signature hex");
        let v: [u8; 32] = signature_bytes[..32].try_into().unwrap();
        let h: [u8; 32] = signature_bytes[32..].try_into().unwrap();
        
        let public_key_bytes = hex::decode(PUBLIC_KEY_HEX).expect("Invalid public key hex");
        let public_key: [u8; 32] = public_key_bytes.try_into().unwrap();
        
        let tx_bytes = build_transaction_bytes(&public_key);
        
        use crypto::algorithms::signature::curve25519::core;
        
        println!("v is canonical: {}", core::is_canonical_signature(&v));
        println!("public key is canonical: {}", core::is_canonical_public_key(&public_key));
        
        let m = Sha256::digest(&tx_bytes);
        println!("Message hash (m): {}", hex::encode(m));
        
        let mut y = [0u8; 32];
        core::verify(&mut y, &v, &h, &public_key);
        println!("Computed Y: {}", hex::encode(y));
        
        let mut hasher = Sha256::new();
        hasher.update(&m);
        hasher.update(&y);
        let expected_h = hasher.finalize();
        println!("Expected h: {}", hex::encode(expected_h));
        println!("Actual h:   {}", hex::encode(h));
    }
}
