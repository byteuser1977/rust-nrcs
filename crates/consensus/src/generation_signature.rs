//! 生成签名模块
//!
//! 对应 Java: Generator.getHit() 中的 generation signature 计算
//!
//! 负责:
//! - 计算生成签名 (generation_signature)
//! - 验证生成签名

use blockchain_types::*;
use sha2::{Sha256, Digest};

pub fn calculate_generation_signature(
    prev_generation_signature: &[u8; 64],
    generator_id: AccountId,
) -> Hash512 {
    let mut hasher = Sha256::new();
    hasher.update(prev_generation_signature);
    hasher.update(&generator_id.to_be_bytes());
    let hash = hasher.finalize();
    
    let mut result = [0u8; 64];
    result[..32].copy_from_slice(&hash);
    
    let mut hasher2 = Sha256::new();
    hasher2.update(&hash);
    hasher2.update(prev_generation_signature);
    let hash2 = hasher2.finalize();
    result[32..].copy_from_slice(&hash2);
    
    Hash512(result)
}

pub fn verify_generation_signature(
    signature: &Hash512,
    prev_signature: &[u8; 64],
    generator_id: AccountId,
) -> bool {
    let expected = calculate_generation_signature(prev_signature, generator_id);
    signature == &expected
}

pub fn calculate_generation_signature_hash(
    generation_signature: &[u8; 64],
    public_key: &[u8],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(generation_signature);
    hasher.update(public_key);
    let hash = hasher.finalize();
    
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash);
    result
}

pub fn derive_hit_from_signature(
    generation_signature: &[u8; 64],
    public_key: &[u8],
) -> u64 {
    let hash = calculate_generation_signature_hash(generation_signature, public_key);
    
    let mut hit_bytes = [0u8; 8];
    hit_bytes.copy_from_slice(&hash[..8]);
    hit_bytes.reverse();
    
    u64::from_be_bytes(hit_bytes)
}

pub fn calculate_next_generation_signature(
    prev_block_gen_sig: &[u8; 64],
    generator_public_key: &[u8],
) -> Hash512 {
    let mut hasher = Sha256::new();
    hasher.update(prev_block_gen_sig);
    hasher.update(generator_public_key);
    let hash = hasher.finalize();
    
    let mut result = [0u8; 64];
    result[..32].copy_from_slice(&hash);
    
    let mut hasher2 = Sha256::new();
    hasher2.update(&hash);
    hasher2.update(prev_block_gen_sig);
    let hash2 = hasher2.finalize();
    result[32..].copy_from_slice(&hash2);
    
    Hash512(result)
}

pub fn is_valid_generation_signature(signature: &Hash512) -> bool {
    !signature.0.iter().all(|&b| b == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_signature_calculation() {
        let prev_sig = [1u8; 64];
        let generator_id: AccountId = 12345;
        
        let sig = calculate_generation_signature(&prev_sig, generator_id);
        
        assert!(is_valid_generation_signature(&sig));
        assert_ne!(sig.0, prev_sig);
    }

    #[test]
    fn test_generation_signature_verification() {
        let prev_sig = [1u8; 64];
        let generator_id: AccountId = 12345;
        
        let sig = calculate_generation_signature(&prev_sig, generator_id);
        
        assert!(verify_generation_signature(&sig, &prev_sig, generator_id));
        assert!(!verify_generation_signature(&sig, &prev_sig, 99999));
    }

    #[test]
    fn test_hit_derivation() {
        let gen_sig = [2u8; 64];
        let public_key = [3u8; 32];
        
        let hit = derive_hit_from_signature(&gen_sig, &public_key);
        
        assert!(hit > 0);
    }

    #[test]
    fn test_next_generation_signature() {
        let prev_sig = [1u8; 64];
        let public_key = [2u8; 32];
        
        let next_sig = calculate_next_generation_signature(&prev_sig, &public_key);
        
        assert!(is_valid_generation_signature(&next_sig));
    }

    #[test]
    fn test_signature_determinism() {
        let prev_sig = [1u8; 64];
        let generator_id: AccountId = 12345;
        
        let sig1 = calculate_generation_signature(&prev_sig, generator_id);
        let sig2 = calculate_generation_signature(&prev_sig, generator_id);
        
        assert_eq!(sig1, sig2);
    }
}
