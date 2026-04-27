//! NRCS 助记词生成器
//!
//! 对应 Java: PassPhraseGenerator, SecretSharingGenerator
//!
//! 功能：
//! - 生成 12 词助记词
//! - 助记词与 128 位数字互相转换
//! - 从助记词派生密钥对

#![allow(clippy::assign_op_pattern)]
#![allow(clippy::unnecessary_cast)]

mod words;

use num_bigint::BigInt;
use num_traits::Zero;
use rand::RngCore;
use std::collections::HashMap;
use thiserror::Error;

use crate::{sha256, CryptoResult, KeyPair};

pub use words::{NRCS_WORDS, WORD_COUNT};

#[derive(Debug, Error)]
pub enum PassPhraseError {
    #[error("Invalid word count: expected 12, got {0}")]
    InvalidWordCount(usize),
    
    #[error("Invalid word: {0}")]
    InvalidWord(String),
    
    #[error("Number too large for 128 bits")]
    NumberTooLarge,
    
    #[error("Invalid checksum")]
    InvalidChecksum,
}

pub type PassPhraseResult<T> = std::result::Result<T, PassPhraseError>;

lazy_static::lazy_static! {
    static ref WORDS_MAP: HashMap<&'static str, usize> = {
        NRCS_WORDS
            .iter()
            .enumerate()
            .map(|(i, &w)| (w, i))
            .collect()
    };
}

/// 生成 12 词助记词
///
/// 对应 Java: SecretSharingGenerator.generatePassPhrase()
pub fn generate_passphrase() -> PassPhraseResult<String> {
    let mut random = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut random);
    
    let big_random = BigInt::from_bytes_be(num_bigint::Sign::Plus, &random);
    
    number_to_secret(&big_random, true)
}

/// 从助记词派生密钥对（使用 Curve25519，NRCS 兼容）
///
/// 对应 Java: 从 secretPhrase 生成密钥对
pub fn passphrase_to_keypair(passphrase: &str) -> CryptoResult<KeyPair> {
    let seed = sha256(passphrase.as_bytes());
    let public_key = crate::algorithms::Curve25519::derive_public_key(&seed);
    
    Ok(KeyPair::Curve25519 {
        public_key,
        secret_key: seed,
    })
}

/// 将 128 位数字转换为助记词
///
/// 对应 Java: SecretSharingGenerator.numberToSecret()
pub fn number_to_secret(secret_integer: &BigInt, is_12_words: bool) -> PassPhraseResult<String> {
    if is_12_words {
        let words = from_128bit(secret_integer)?;
        Ok(words.join(" "))
    } else {
        let bytes = secret_integer.to_bytes_be().1;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }
}

/// 将助记词转换为 128 位数字
///
/// 对应 Java: SecretSharingGenerator.secretToNumber()
pub fn secret_to_number(passphrase: &str) -> PassPhraseResult<BigInt> {
    let words: Vec<&str> = passphrase.split(' ').collect();
    
    if is_12_words_secret(&words) {
        to_128bit(&words)
    } else {
        Ok(BigInt::from_bytes_be(
            num_bigint::Sign::Plus,
            passphrase.as_bytes(),
        ))
    }
}

/// 检查是否为有效的 12 词助记词
///
/// 对应 Java: SecretSharingGenerator.is12WordsSecret()
pub fn is_12_words_secret(words: &[&str]) -> bool {
    words.len() == 12 && words.iter().all(|w| WORDS_MAP.contains_key(*w))
}

/// 验证助记词
pub fn validate_passphrase(passphrase: &str) -> PassPhraseResult<()> {
    let words: Vec<&str> = passphrase.split(' ').collect();
    
    if words.len() != 12 {
        return Err(PassPhraseError::InvalidWordCount(words.len()));
    }
    
    for word in &words {
        if !WORDS_MAP.contains_key(*word) {
            return Err(PassPhraseError::InvalidWord(word.to_string()));
        }
    }
    
    Ok(())
}

/// 将 12 个单词转换为 128 位数字
///
/// 对应 Java: SecretSharingGenerator.to128bit()
fn to_128bit(words: &[&str]) -> PassPhraseResult<BigInt> {
    if words.len() != 12 {
        return Err(PassPhraseError::InvalidWordCount(words.len()));
    }
    
    let mut n128 = BigInt::zero();
    
    for i in 0..4 {
        let w1 = *WORDS_MAP.get(words[3 * i]).ok_or_else(|| PassPhraseError::InvalidWord(words[3 * i].to_string()))?;
        let w2 = *WORDS_MAP.get(words[3 * i + 1]).ok_or_else(|| PassPhraseError::InvalidWord(words[3 * i + 1].to_string()))?;
        let w3 = *WORDS_MAP.get(words[3 * i + 2]).ok_or_else(|| PassPhraseError::InvalidWord(words[3 * i + 2].to_string()))?;
        
        let x = get_signed_int(w1, w2, w3);
        
        n128 = n128 + BigInt::from(x);
        
        if i < 3 {
            n128 = n128 << 32;
        }
    }
    
    Ok(n128)
}

/// 将 128 位数字转换为 12 个单词
///
/// 对应 Java: SecretSharingGenerator.from128bit()
fn from_128bit(n128_orig: &BigInt) -> PassPhraseResult<Vec<String>> {
    let n = WORD_COUNT as i64;
    let mut words = vec![String::new(); 12];
    let mut n128 = n128_orig.clone();
    
    for i in 0..4 {
        let x = (&n128 & &BigInt::from(0xffffffffu64)).to_i64_checked().unwrap_or(0) as i64;
        n128 = &n128 >> 32;
        
        let w1 = ((x % n) + n) % n;
        let w2 = (((x / n) + w1) % n + n) % n;
        let w3 = ((((x / n) / n) + w2) % n + n) % n;
        
        if w2 < 0 || w2 >= n || w3 < 0 || w3 >= n {
            return Err(PassPhraseError::InvalidChecksum);
        }
        
        let index = 3 * (4 - i - 1);
        words[index] = NRCS_WORDS[w1 as usize].to_string();
        words[index + 1] = NRCS_WORDS[w2 as usize].to_string();
        words[index + 2] = NRCS_WORDS[w3 as usize].to_string();
    }
    
    if n128 > BigInt::zero() {
        return Err(PassPhraseError::NumberTooLarge);
    }
    
    Ok(words)
}

/// 计算签名整数
///
/// 对应 Java: SecretSharingGenerator.getSignedInt()
fn get_signed_int(w1: usize, w2: usize, w3: usize) -> i64 {
    let n = WORD_COUNT as i64;
    let w1 = w1 as i64;
    let w2 = w2 as i64;
    let w3 = w3 as i64;
    
    let diff1 = ((w2 - w1) % n + n) % n;
    let diff2 = ((w3 - w2) % n + n) % n;
    
    let result = w1 + diff1 * n + diff2 * n * n;
    
    result & 0x00000000ffffffff
}

trait ToI64 {
    fn to_i64_checked(&self) -> Option<i64>;
}

impl ToI64 for BigInt {
    fn to_i64_checked(&self) -> Option<i64> {
        num_traits::ToPrimitive::to_i64(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_passphrase() {
        let passphrase = generate_passphrase().unwrap();
        let words: Vec<&str> = passphrase.split(' ').collect();
        assert_eq!(words.len(), 12);
    }
    
    #[test]
    fn test_roundtrip() {
        let passphrase = generate_passphrase().unwrap();
        let number = secret_to_number(&passphrase).unwrap();
        let recovered = number_to_secret(&number, true).unwrap();
        assert_eq!(passphrase, recovered);
    }
    
    #[test]
    fn test_is_12_words_secret() {
        let passphrase = generate_passphrase().unwrap();
        let words: Vec<&str> = passphrase.split(' ').collect();
        assert!(is_12_words_secret(&words));
        
        let invalid_words = vec!["invalid", "word", "list"];
        assert!(!is_12_words_secret(&invalid_words));
    }
    
    #[test]
    fn test_validate_passphrase() {
        let passphrase = generate_passphrase().unwrap();
        assert!(validate_passphrase(&passphrase).is_ok());
        
        let invalid = "invalid passphrase with only three words";
        assert!(validate_passphrase(invalid).is_err());
    }
    
    #[test]
    fn test_passphrase_to_keypair() {
        let passphrase = generate_passphrase().unwrap();
        let keypair = passphrase_to_keypair(&passphrase).unwrap();
        let public_key = keypair.public_key();
        
        match public_key {
            crate::PublicKey::Curve25519(bytes) => {
                assert_eq!(bytes.len(), 32);
            }
            _ => panic!("Expected Curve25519 public key"),
        }
    }
    
    #[test]
    fn test_known_passphrase() {
        let passphrase = "like just love know never want time out there make look eye";
        let result = validate_passphrase(passphrase);
        assert!(result.is_ok());
        
        let number = secret_to_number(passphrase).unwrap();
        let recovered = number_to_secret(&number, true).unwrap();
        assert_eq!(passphrase, recovered);
    }
    
    #[test]
    fn test_nrcs_compatibility() {
        let passphrase = "confusion flirt teeth story crawl dear shove screw decay flood cover warrior";
        
        assert!(validate_passphrase(passphrase).is_ok());
        
        let number = secret_to_number(passphrase).unwrap();
        let recovered = number_to_secret(&number, true).unwrap();
        assert_eq!(passphrase, recovered);
    }
}
