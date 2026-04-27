use blockchain_types::*;
use sha2::{Sha256, Digest};

pub fn calculate_generation_signature(
    prev_generation_signature: &[u8],
    generator_id: AccountId,
) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(prev_generation_signature);
    hasher.update(generator_id.to_be_bytes());
    let hash = hasher.finalize();
    hash.to_vec()
}

pub fn verify_generation_signature(
    signature: &[u8],
    prev_signature: &[u8],
    generator_id: AccountId,
) -> bool {
    let expected = calculate_generation_signature(prev_signature, generator_id);
    signature == expected.as_slice()
}

pub fn calculate_generation_signature_with_public_key(
    prev_generation_signature: &[u8],
    generator_public_key: &[u8],
) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(prev_generation_signature);
    hasher.update(generator_public_key);
    let hash = hasher.finalize();
    hash.to_vec()
}

pub fn calculate_generation_signature_hash(
    generation_signature: &[u8],
    public_key: &[u8],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(generation_signature);
    hasher.update(public_key);
    let hash = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&hash);
    arr
}

pub fn derive_hit_from_signature(
    generation_signature: &[u8],
    public_key: &[u8],
) -> u64 {
    let hash = calculate_generation_signature_hash(generation_signature, public_key);

    let mut hit_bytes = [0u8; 8];
    hit_bytes.copy_from_slice(&hash[..8]);
    hit_bytes.reverse();

    u64::from_be_bytes(hit_bytes)
}

pub fn calculate_next_generation_signature(
    prev_block_gen_sig: &[u8],
    generator_public_key: &[u8],
) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(prev_block_gen_sig);
    hasher.update(generator_public_key);
    let hash = hasher.finalize();
    hash.to_vec()
}

pub fn is_valid_generation_signature(signature: &[u8]) -> bool {
    !signature.iter().all(|&b| b == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_signature_calculation() {
        let prev_sig = [1u8; 32];
        let account_id: AccountId = 12345;
        let sig = calculate_generation_signature(&prev_sig, account_id);
        assert!(is_valid_generation_signature(&sig));
    }
}
