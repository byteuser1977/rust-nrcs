//! NRCS 兼容性测试
//!
//! 测试数据来自 NRCS Java 实现，验证 Rust 实现的完全兼容性

use sha2::{Digest, Sha256};
use x25519_dalek::StaticSecret;

/// NRCS 测试向量
struct NrcsTestVector {
    passphrase: &'static str,
    account_id: &'static str,
    public_key: &'static str,
}

/// 所有 NRCS 测试向量（只包含完整数据的向量）
const NRCS_TEST_VECTORS: &[NrcsTestVector] = &[
    NrcsTestVector {
        passphrase: "confusion flirt teeth story crawl dear shove screw decay flood cover warrior",
        account_id: "NRCS-P5FE-QYES-PLHK-HWTJM",
        public_key: "5aafe59365b988d73aa15424a1b83c24fcada07e7cd73fcfade612c5bf32fc71",
    },
    NrcsTestVector {
        passphrase: "despair belief enter dragon glove protect known rabbit ceiling creature satisfy forgive",
        account_id: "NRCS-GK8K-KXAS-UJHW-F8YQE",
        public_key: "776ee6f3f7814b29b4d89e9db1fd95ec7afc3ea91e511270cbde1a76b8e7f71b",
    },
    NrcsTestVector {
        passphrase: "eat anyone spin dream follow vain cute clay blush throat change reality",
        account_id: "NRCS-RBU5-RCCU-R42K-ESGM9",
        public_key: "e17d3de4d16fd1a859f45ee1be0f0f742070cc727ffb9e439d9e60382142454c",
    },
    NrcsTestVector {
        passphrase: "grow pay sway ready grand trail boat strain concrete joy queen three",
        account_id: "NRCS-SR64-JH9W-6V6C-GXTNT",
        public_key: "011f991a3c9cc91f8493c2551ec2acf94cfbc4f32b14fd44b1521dc68f2c2829",
    },
    NrcsTestVector {
        passphrase: "heaven distance mercy use tumble loud taste everything stumble company add finally",
        account_id: "NRCS-JLXR-QSFN-EBWE-9DKPH",
        public_key: "3e28e5368ce3d9b933f7a26312917fe90dd019f9668ad26842b5ea67cb88ff09",
    },
    NrcsTestVector {
        passphrase: "caress concern page heavy collapse present probably admit finish chair should college",
        account_id: "NRCS-VFSN-W739-Q9YQ-8JELM",
        public_key: "994facb04d62111fbc72b9e488cb30472e3a8e8c31365491d3abde56d2c7e437",
    },
    NrcsTestVector {
        passphrase: "repeat knowledge confidence comfort dew minute chest stood shiny spoken connect foot",
        account_id: "NRCS-LWSL-QGTD-HADP-3WBSX",
        public_key: "6e45e2ac4c1905eaca5d090b650269e852a98d7a59676f2583fc97b7cf39de7e",
    },
    NrcsTestVector {
        passphrase: "shall crystal everyone boyfriend two look black beside measure lace god third",
        account_id: "NRCS-5LUR-LNFD-RENJ-5MHE3",
        public_key: "f4081f096d506d5b8a0fc580f141e80a0d75d889085f88f885816fe4d7de1262",
    },
];

/// 只有公钥的测试向量（用于验证公钥派生）
const PUBLIC_KEY_ONLY_VECTORS: &[(&str, &str)] = &[
    ("like just love know never want time out there make look eye", 
     "7262c9647f585353ca987cf6aa39842788ffa23d5a3674234168f0f85eb31061"),
];

/// 从 passphrase 派生公钥
fn derive_public_key(passphrase: &str) -> [u8; 32] {
    let seed: [u8; 32] = Sha256::digest(passphrase.as_bytes()).into();
    let secret = StaticSecret::from(seed);
    *x25519_dalek::PublicKey::from(&secret).as_bytes()
}

/// 从公钥计算 Account ID
fn calculate_account_id(public_key: &[u8; 32]) -> u64 {
    let hash = Sha256::digest(public_key);
    u64::from_le_bytes([
        hash[0], hash[1], hash[2], hash[3],
        hash[4], hash[5], hash[6], hash[7],
    ])
}

/// Reed-Solomon 编码常量
const GEXP: [i32; 32] = [1, 2, 4, 8, 16, 5, 10, 20, 13, 26, 17, 7, 14, 28, 29, 31, 27, 19, 3, 6, 12, 24, 21, 15, 30, 25, 23, 11, 22, 9, 18, 1];
const GLOG: [i32; 32] = [0, 0, 1, 18, 2, 5, 19, 11, 3, 29, 6, 27, 20, 8, 12, 23, 4, 10, 30, 17, 7, 22, 28, 26, 21, 25, 9, 16, 13, 14, 24, 15];
const CODEWORD_MAP: [usize; 17] = [3, 2, 1, 0, 7, 6, 5, 4, 13, 14, 15, 16, 12, 8, 9, 10, 11];
const ALPHABET: &str = "23456789ABCDEFGHJKLMNPQRSTUVWXYZ";

/// Reed-Solomon 编码
fn encode_reed_solomon(plain: u64) -> String {
    let plain_string = plain.to_string();
    let length = plain_string.len();
    let mut plain_string_10 = [0i32; 20];
    for (i, c) in plain_string.chars().enumerate() {
        plain_string_10[i] = (c as i32) - ('0' as i32);
    }

    let mut codeword_length = 0;
    let mut codeword = [0i32; 17];
    let mut length = length;

    while length > 0 {
        let mut new_length = 0;
        let mut digit_32 = 0;
        for i in 0..length {
            digit_32 = digit_32 * 10 + plain_string_10[i];
            if digit_32 >= 32 {
                plain_string_10[new_length] = digit_32 >> 5;
                digit_32 &= 31;
                new_length += 1;
            } else if new_length > 0 {
                plain_string_10[new_length] = 0;
                new_length += 1;
            }
        }
        length = new_length;
        codeword[codeword_length] = digit_32;
        codeword_length += 1;
    }

    let mut p = [0i32; 4];
    for i in (0..13).rev() {
        let fb = codeword[i] ^ p[3];
        p[3] = p[2] ^ gmult(30, fb);
        p[2] = p[1] ^ gmult(6, fb);
        p[1] = p[0] ^ gmult(9, fb);
        p[0] = gmult(17, fb);
    }

    for i in 0..4 {
        codeword[13 + i] = p[i];
    }

    let mut result = String::new();
    for i in 0..17 {
        let codework_index = CODEWORD_MAP[i];
        let alphabet_index = codeword[codework_index] as usize;
        result.push(ALPHABET.chars().nth(alphabet_index).unwrap());

        if (i & 3) == 3 && i < 13 {
            result.push('-');
        }
    }
    result
}

fn gmult(a: i32, b: i32) -> i32 {
    if a == 0 || b == 0 {
        return 0;
    }
    let a_idx = a as usize;
    let b_idx = b as usize;
    let idx = (GLOG[a_idx] + GLOG[b_idx]) as usize % 31;
    GEXP[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_public_keys() {
        println!("\n=== 公钥派生测试 ===\n");
        
        for (i, tv) in NRCS_TEST_VECTORS.iter().enumerate() {
            let derived_pk = derive_public_key(tv.passphrase);
            let derived_pk_hex = hex::encode(derived_pk);
            
            let match_result = derived_pk_hex == tv.public_key;
            
            println!("测试 {}: {}", i + 1, if match_result { "✓ PASS" } else { "✗ FAIL" });
            println!("  Passphrase: {}", tv.passphrase);
            println!("  Expected:   {}", tv.public_key);
            println!("  Derived:    {}", derived_pk_hex);
            println!();
            
            assert_eq!(derived_pk_hex, tv.public_key, 
                "Public key mismatch for passphrase: {}", tv.passphrase);
        }
    }

    #[test]
    fn test_all_account_ids() {
        println!("\n=== Account ID 测试 ===\n");
        
        for (i, tv) in NRCS_TEST_VECTORS.iter().enumerate() {
            let public_key_bytes = hex::decode(tv.public_key).expect("Invalid hex");
            let public_key: [u8; 32] = public_key_bytes.try_into().expect("Invalid length");
            
            let account_id_num = calculate_account_id(&public_key);
            let derived_account_id = format!("NRCS-{}", encode_reed_solomon(account_id_num));
            
            let match_result = derived_account_id == tv.account_id;
            
            println!("测试 {}: {}", i + 1, if match_result { "✓ PASS" } else { "✗ FAIL" });
            println!("  Passphrase: {}", tv.passphrase);
            println!("  Expected:   {}", tv.account_id);
            println!("  Derived:    {}", derived_account_id);
            println!("  Numeric ID: {}", account_id_num);
            println!();
            
            assert_eq!(derived_account_id, tv.account_id,
                "Account ID mismatch for passphrase: {}", tv.passphrase);
        }
    }

    #[test]
    fn test_full_compatibility() {
        println!("\n=== 完整兼容性测试 ===\n");
        
        let mut pass_count = 0;
        let total = NRCS_TEST_VECTORS.len();
        
        for (i, tv) in NRCS_TEST_VECTORS.iter().enumerate() {
            let derived_pk = derive_public_key(tv.passphrase);
            let derived_pk_hex = hex::encode(derived_pk);
            
            let public_key_bytes = hex::decode(tv.public_key).expect("Invalid hex");
            let public_key: [u8; 32] = public_key_bytes.try_into().expect("Invalid length");
            let account_id_num = calculate_account_id(&public_key);
            let derived_account_id = format!("NRCS-{}", encode_reed_solomon(account_id_num));
            
            let pk_match = derived_pk_hex == tv.public_key;
            let id_match = derived_account_id == tv.account_id;
            
            if pk_match && id_match {
                pass_count += 1;
                println!("测试 {}: ✓ PASS", i + 1);
            } else {
                println!("测试 {}: ✗ FAIL", i + 1);
            }
            
            if !pk_match {
                println!("  公钥不匹配!");
                println!("    Expected: {}", tv.public_key);
                println!("    Got:      {}", derived_pk_hex);
            }
            
            if !id_match {
                println!("  Account ID 不匹配!");
                println!("    Expected: {}", tv.account_id);
                println!("    Got:      {}", derived_account_id);
            }
        }
        
        println!("\n结果: {}/{} 通过", pass_count, total);
        
        assert_eq!(pass_count, total, "Not all tests passed");
    }

    #[test]
    fn test_individual_vector_1() {
        let tv = &NRCS_TEST_VECTORS[0];
        let derived_pk = derive_public_key(tv.passphrase);
        assert_eq!(hex::encode(derived_pk), tv.public_key);
        
        let public_key_bytes = hex::decode(tv.public_key).unwrap();
        let public_key: [u8; 32] = public_key_bytes.try_into().unwrap();
        let account_id_num = calculate_account_id(&public_key);
        let derived_id = format!("NRCS-{}", encode_reed_solomon(account_id_num));
        assert_eq!(derived_id, tv.account_id);
    }

    #[test]
    fn test_public_key_only_vectors() {
        println!("\n=== 只有公钥的测试向量 ===\n");
        
        for (i, (passphrase, expected_pk)) in PUBLIC_KEY_ONLY_VECTORS.iter().enumerate() {
            let derived_pk = derive_public_key(passphrase);
            let derived_pk_hex = hex::encode(derived_pk);
            
            let match_result = derived_pk_hex == *expected_pk;
            
            println!("测试 {}: {}", i + 1, if match_result { "✓ PASS" } else { "✗ FAIL" });
            println!("  Passphrase: {}", passphrase);
            println!("  Expected:   {}", expected_pk);
            println!("  Derived:    {}", derived_pk_hex);
            
            // 计算并显示 Account ID（供参考）
            let public_key_bytes = hex::decode(expected_pk).unwrap();
            let public_key: [u8; 32] = public_key_bytes.try_into().unwrap();
            let account_id_num = calculate_account_id(&public_key);
            let derived_account_id = format!("NRCS-{}", encode_reed_solomon(account_id_num));
            println!("  Account ID: {} (derived)", derived_account_id);
            println!("  Numeric ID: {}", account_id_num);
            println!();
            
            assert_eq!(derived_pk_hex, *expected_pk, 
                "Public key mismatch for passphrase: {}", passphrase);
        }
    }
}
