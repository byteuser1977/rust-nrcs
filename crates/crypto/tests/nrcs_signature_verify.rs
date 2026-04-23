//! NRCS 签名验证测试 - 使用真实交易数据
//!
//! 测试数据来自 NRCS Java 实现的真实交易

use sha2::{Digest, Sha256};
use x25519_dalek::StaticSecret;

/// 测试向量 1
const PASSPHRASE_1: &str = "concern entire frozen witch away creak dot drink need season clutch truly";
const PUBLIC_KEY_1: &str = "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c";
const ACCOUNT_ID_1: &str = "NRCS-LVB4-2ATD-4KZZ-2BXNV";

/// 签名 1 (来自交易 9344731544105219785)
const SIGNATURE_1: &str = "7f2bb75686349576e634083b1f0a30cb30209cb8db952b9b1b1c2072354cf2076cc640b3ca04d03b82c3cb6f1e44efbf18761f9b334f35ab2bcba98b7e1b06d5";

/// 签名 2 (来自交易 2452132941054689657)
const SIGNATURE_2: &str = "eab9a9fd3d73950a372e17a76a38fb206b875a84f9bc4fa80b18307ea683f204fcffc4413d187302a84ccdf89d796130a6e9d161afc30a45fc41e4257af4f0c5";

/// 未签名交易字节 (交易 2452132941054689657)
/// 这是用于签名的原始数据
const UNSIGNED_TRANSACTION_BYTES: &str = "001037b138053c002d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c5fe6cbd7bfb374290065cd1d0000000000e1f505000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000007cb1400e278bbd91da7aae30163fc7d52fb0c3d2cd86b77a1a4f32b233735c3108a68d6a525909d4efe887dde";

/// 已签名交易字节 (交易 2452132941054689657)
const TRANSACTION_BYTES: &str = "001037b138053c002d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c5fe6cbd7bfb374290065cd1d0000000000e1f505000000000000000000000000000000000000000000000000000000000000000000000000eab9a9fd3d73950a372e17a76a38fb206b875a84f9bc4fa80b18307ea683f204fcffc4413d187302a84ccdf89d796130a6e9d161afc30a45fc41e4257af4f0c52000000007cb1400e278bbd91da7aae30163fc7d52fb0c3d2cd86b77a1a4f32b233735c3108a68d6a525909d4efe887dde";

/// 从 passphrase 派生公钥
fn derive_public_key(passphrase: &str) -> [u8; 32] {
    let seed: [u8; 32] = Sha256::digest(passphrase.as_bytes()).into();
    let secret = StaticSecret::from(seed);
    *x25519_dalek::PublicKey::from(&secret).as_bytes()
}

/// NRCS 签名实现
/// 签名格式: (v, h) 其中 h = SHA256(m || Y)
fn nrcs_sign(message: &[u8], passphrase: &str) -> [u8; 64] {
    use crypto::algorithms::signature::curve25519::core;
    
    // 1. 从 passphrase 生成公钥 P 和签名私钥 s
    let seed = Sha256::digest(passphrase.as_bytes());
    let mut private_key: [u8; 32] = seed.into();
    let mut public_key = [0u8; 32];
    let mut signing_key = [0u8; 32];
    
    core::keygen(&mut public_key, Some(&mut signing_key), &mut private_key);
    
    // 2. m = SHA256(message)
    let m = Sha256::digest(message);
    
    // 3. x = SHA256(m || s)
    let mut hasher = Sha256::new();
    hasher.update(&m);
    hasher.update(&signing_key);
    let x = hasher.finalize();
    let mut x_array: [u8; 32] = x.into();
    
    // 4. Y = keygen(x) - 临时公钥
    let mut y = [0u8; 32];
    core::keygen(&mut y, None, &mut x_array);
    
    // 5. h = SHA256(m || Y)
    let mut hasher2 = Sha256::new();
    hasher2.update(&m);
    hasher2.update(&y);
    let h = hasher2.finalize();
    let h_array: [u8; 32] = h.into();
    
    // 6. v = sign(h, x, s)
    let mut v = [0u8; 32];
    core::sign(&mut v, &h_array, &x_array, &signing_key);
    
    // 7. signature = v || h
    let mut signature = [0u8; 64];
    signature[..32].copy_from_slice(&v);
    signature[32..].copy_from_slice(&h_array);
    signature
}

/// NRCS 验签实现
/// 签名格式: (v, h) 其中 h = SHA256(m || Y)
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
    
    // 1. m = SHA256(message)
    let m = Sha256::digest(message);
    
    // 2. Y = verify(v, h, P)
    let mut y = [0u8; 32];
    core::verify(&mut y, &v, &h, public_key);
    
    // 3. h2 = SHA256(m || Y)
    let mut hasher = Sha256::new();
    hasher.update(&m);
    hasher.update(&y);
    let expected_h = hasher.finalize();
    
    // 4. 验证 h == h2
    h == expected_h.as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_derivation() {
        let derived_pk = derive_public_key(PASSPHRASE_1);
        let derived_pk_hex = hex::encode(derived_pk);
        
        println!("Expected public key: {}", PUBLIC_KEY_1);
        println!("Derived public key:  {}", derived_pk_hex);
        
        assert_eq!(derived_pk_hex, PUBLIC_KEY_1);
    }

    #[test]
    fn test_parse_transaction_bytes() {
        let unsigned_bytes = hex::decode(UNSIGNED_TRANSACTION_BYTES).expect("Invalid hex");
        let signed_bytes = hex::decode(TRANSACTION_BYTES).expect("Invalid hex");
        
        println!("Unsigned transaction bytes length: {}", unsigned_bytes.len());
        println!("Signed transaction bytes length: {}", signed_bytes.len());
        
        // 解析交易结构
        // type (1 byte) + version|subtype (1 byte) + timestamp (4 bytes) + deadline (2 bytes)
        // + senderPublicKey (32 bytes) + recipient (8 bytes) + amountNQT (8 bytes)
        // + feeNQT (8 bytes) + referencedTransactionFullHash (32 bytes) + signature (64 bytes)
        // + flags (4 bytes) + ecBlockHeight (4 bytes) + ecBlockId (8 bytes) + attachment
        
        let tx_type = unsigned_bytes[0];
        let version_subtype = unsigned_bytes[1];
        let version = version_subtype >> 4;
        let subtype = version_subtype & 0x0F;
        
        let timestamp = u32::from_le_bytes([unsigned_bytes[2], unsigned_bytes[3], unsigned_bytes[4], unsigned_bytes[5]]);
        let deadline = u16::from_le_bytes([unsigned_bytes[6], unsigned_bytes[7]]);
        
        let sender_pk = &unsigned_bytes[8..40];
        let recipient = u64::from_le_bytes(unsigned_bytes[40..48].try_into().unwrap());
        let amount = u64::from_le_bytes(unsigned_bytes[48..56].try_into().unwrap());
        let fee = u64::from_le_bytes(unsigned_bytes[56..64].try_into().unwrap());
        
        println!("Type: {}", tx_type);
        println!("Version: {}, Subtype: {}", version, subtype);
        println!("Timestamp: {}", timestamp);
        println!("Deadline: {}", deadline);
        println!("Sender Public Key: {}", hex::encode(sender_pk));
        println!("Recipient: {}", recipient);
        println!("Amount: {}", amount);
        println!("Fee: {}", fee);
        
        // 验证签名位置
        // 交易结构: type(1) + version|subtype(1) + timestamp(4) + deadline(2) + senderPublicKey(32)
        //         + recipient(8) + amountNQT(8) + feeNQT(8) + referencedTransactionFullHash(32)
        //         + signature(64) + flags(4) + ecBlockHeight(4) + ecBlockId(8) + attachment
        // 签名位置: 1+1+4+2+32+8+8+8+32 = 96 字节后开始
        let signature_offset = 96;
        let signature_in_signed = &signed_bytes[signature_offset..signature_offset+64];
        let signature_in_unsigned = &unsigned_bytes[signature_offset..signature_offset+64];
        
        println!("\nSignature offset: {}", signature_offset);
        println!("Signature in signed bytes: {}", hex::encode(signature_in_signed));
        println!("Signature in unsigned bytes: {}", hex::encode(signature_in_unsigned));
        
        // 验证签名是否匹配
        assert_eq!(hex::encode(signature_in_signed), SIGNATURE_2);
        
        // 验证未签名的签名部分是全零
        let all_zero: bool = signature_in_unsigned.iter().all(|&b| b == 0);
        assert!(all_zero, "Unsigned signature should be all zeros");
    }

    #[test]
    fn test_sign_transaction_2() {
        let unsigned_bytes = hex::decode(UNSIGNED_TRANSACTION_BYTES).expect("Invalid hex");
        
        let signature = nrcs_sign(&unsigned_bytes, PASSPHRASE_1);
        let signature_hex = hex::encode(signature);
        
        println!("Expected signature: {}", SIGNATURE_2);
        println!("Derived signature:  {}", signature_hex);
        
        // 验证我们生成的签名可以被验签
        let public_key_bytes = hex::decode(PUBLIC_KEY_1).expect("Invalid public key hex");
        let public_key: [u8; 32] = public_key_bytes.try_into().unwrap();
        
        let result = nrcs_verify(&signature, &unsigned_bytes, &public_key);
        println!("Self-generated signature verification: {}", result);
        
        // 调试：逐步检查签名过程
        use crypto::algorithms::signature::curve25519::core;
        
        let m = Sha256::digest(&unsigned_bytes);
        let seed = Sha256::digest(PASSPHRASE_1.as_bytes());
        let mut private_key: [u8; 32] = seed.into();
        let mut computed_pk = [0u8; 32];
        let mut signing_key = [0u8; 32];
        core::keygen(&mut computed_pk, Some(&mut signing_key), &mut private_key);
        
        // x = SHA256(m || s)
        let mut hasher = Sha256::new();
        hasher.update(&m);
        hasher.update(&signing_key);
        let x = hasher.finalize();
        let mut x_array: [u8; 32] = x.into();
        
        // Y = keygen(x)
        let mut y_computed = [0u8; 32];
        core::keygen(&mut y_computed, None, &mut x_array);
        
        // h = SHA256(m || Y)
        let mut hasher2 = Sha256::new();
        hasher2.update(&m);
        hasher2.update(&y_computed);
        let h = hasher2.finalize();
        let h_array: [u8; 32] = h.into();
        
        println!("\n=== Debug Sign Process ===");
        println!("m: {}", hex::encode(m));
        println!("signing_key (s): {}", hex::encode(signing_key));
        println!("x: {}", hex::encode(x));
        println!("Y: {}", hex::encode(y_computed));
        println!("h: {}", hex::encode(h));
        
        // v = sign(h, x, s)
        let mut v = [0u8; 32];
        let sign_result = core::sign(&mut v, &h_array, &x_array, &signing_key);
        println!("sign result: {}", sign_result);
        println!("v: {}", hex::encode(v));
        
        // 验证 v 是否 canonical
        println!("v is canonical: {}", core::is_canonical_signature(&v));
        
        // 使用 verify 验证
        let mut y_from_verify = [0u8; 32];
        core::verify(&mut y_from_verify, &v, &h_array, &public_key);
        println!("Y from verify: {}", hex::encode(y_from_verify));
        println!("Y computed:    {}", hex::encode(y_computed));
        println!("Y match: {}", y_from_verify == y_computed);
        
        assert!(result, "Self-generated signature should verify");
    }

    #[test]
    fn test_verify_transaction_2() {
        let public_key_bytes = hex::decode(PUBLIC_KEY_1).expect("Invalid public key hex");
        let public_key: [u8; 32] = public_key_bytes.try_into().unwrap();
        
        let signature_bytes = hex::decode(SIGNATURE_2).expect("Invalid signature hex");
        let signature: [u8; 64] = signature_bytes.try_into().unwrap();
        
        let unsigned_bytes = hex::decode(UNSIGNED_TRANSACTION_BYTES).expect("Invalid hex");
        
        let result = nrcs_verify(&signature, &unsigned_bytes, &public_key);
        
        println!("Verification result: {}", result);
        
        assert!(result, "Signature verification should succeed");
    }

    #[test]
    fn test_verify_transaction_1() {
        // 第一组测试向量
        let public_key_bytes = hex::decode(PUBLIC_KEY_1).expect("Invalid public key hex");
        let public_key: [u8; 32] = public_key_bytes.try_into().unwrap();
        
        let signature_bytes = hex::decode(SIGNATURE_1).expect("Invalid signature hex");
        let signature: [u8; 64] = signature_bytes.try_into().unwrap();
        
        // 第一笔交易的数据需要从 JSON 构建
        // 暂时跳过，因为需要完整的 unsignedTransactionBytes
    }

    #[test]
    fn test_signature_components_detailed() {
        let unsigned_bytes = hex::decode(UNSIGNED_TRANSACTION_BYTES).expect("Invalid hex");
        let public_key_bytes = hex::decode(PUBLIC_KEY_1).expect("Invalid public key hex");
        let public_key: [u8; 32] = public_key_bytes.try_into().unwrap();
        
        let signature_bytes = hex::decode(SIGNATURE_2).expect("Invalid signature hex");
        let v: [u8; 32] = signature_bytes[..32].try_into().unwrap();
        let h: [u8; 32] = signature_bytes[32..].try_into().unwrap();
        
        use crypto::algorithms::signature::curve25519::core;
        
        println!("=== 签名组件分析 ===");
        println!("v: {}", hex::encode(v));
        println!("h: {}", hex::encode(h));
        println!("v is canonical: {}", core::is_canonical_signature(&v));
        println!("public key is canonical: {}", core::is_canonical_public_key(&public_key));
        
        // 计算中间值
        let m = Sha256::digest(&unsigned_bytes);
        println!("\nm = SHA256(message): {}", hex::encode(m));
        
        // 从 passphrase 计算 signing key
        let seed = Sha256::digest(PASSPHRASE_1.as_bytes());
        let mut private_key: [u8; 32] = seed.into();
        let mut computed_pk = [0u8; 32];
        let mut signing_key = [0u8; 32];
        core::keygen(&mut computed_pk, Some(&mut signing_key), &mut private_key);
        
        println!("\nComputed public key: {}", hex::encode(computed_pk));
        println!("Signing key (s): {}", hex::encode(signing_key));
        
        // x = SHA256(m || s)
        let mut hasher = Sha256::new();
        hasher.update(&m);
        hasher.update(&signing_key);
        let x = hasher.finalize();
        println!("x = SHA256(m || s): {}", hex::encode(x));
        
        // Y = keygen(x)
        let mut x_array: [u8; 32] = x.into();
        let mut y_computed = [0u8; 32];
        core::keygen(&mut y_computed, None, &mut x_array);
        println!("Y = keygen(x): {}", hex::encode(y_computed));
        
        // h_computed = SHA256(m || Y)
        let mut hasher2 = Sha256::new();
        hasher2.update(&m);
        hasher2.update(&y_computed);
        let h_computed = hasher2.finalize();
        println!("h_computed = SHA256(m || Y): {}", hex::encode(h_computed));
        println!("h from signature: {}", hex::encode(h));
        
        // 使用 verify 计算 Y
        let mut y_from_verify = [0u8; 32];
        core::verify(&mut y_from_verify, &v, &h, &public_key);
        println!("\nY from verify(v, h, P): {}", hex::encode(y_from_verify));
        
        // h2 = SHA256(m || Y_from_verify)
        let mut hasher3 = Sha256::new();
        hasher3.update(&m);
        hasher3.update(&y_from_verify);
        let h2 = hasher3.finalize();
        println!("h2 = SHA256(m || Y_from_verify): {}", hex::encode(h2));
        
        println!("\n=== 验证结果 ===");
        println!("h == h2: {}", h == h2.as_slice());
    }
}
