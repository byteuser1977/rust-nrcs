//! 助记词生成和验证测试
//!
//! 验证 NRCS 助记词生成器的正确性，包括生成、验证、密钥对派生。

use crypto::passphrase::{
    generate_passphrase, passphrase_to_keypair, validate_passphrase,
    secret_to_number, number_to_secret, is_12_words_secret,
};

#[test]
fn test_generate_passphrase() {
    let passphrase = generate_passphrase().expect("passphrase generation failed");
    let words: Vec<&str> = passphrase.split_whitespace().collect();
    assert_eq!(words.len(), 12, "Passphrase should have 12 words");
}

#[test]
fn test_generate_passphrase_unique() {
    let p1 = generate_passphrase().expect("passphrase generation failed");
    let p2 = generate_passphrase().expect("passphrase generation failed");
    assert_ne!(p1, p2, "Two generated passphrases should be different");
}

#[test]
fn test_validate_passphrase_valid() {
    let passphrase = generate_passphrase().expect("passphrase generation failed");
    assert!(validate_passphrase(&passphrase).is_ok());
}

#[test]
fn test_validate_passphrase_invalid_word_count() {
    assert!(validate_passphrase("word1 word2 word3").is_err());
}

#[test]
fn test_passphrase_to_keypair() {
    let passphrase = "concern entire frozen witch away creak dot drink need season clutch truly";
    let kp = passphrase_to_keypair(passphrase).expect("keypair derivation failed");
    let pk = kp.public_key();
    assert_eq!(pk.len(), 32, "Public key should be 32 bytes");
}

#[test]
fn test_passphrase_to_keypair_deterministic() {
    let passphrase = "trickle pierce warm early gentle another thorn gotta illuminate everywhere glare determine";
    let kp1 = passphrase_to_keypair(passphrase).expect("keypair derivation failed");
    let kp2 = passphrase_to_keypair(passphrase).expect("keypair derivation failed");
    assert_eq!(kp1.public_key(), kp2.public_key(), "Same passphrase should produce same keypair");
}

#[test]
fn test_is_12_words_secret() {
    let passphrase = generate_passphrase().expect("passphrase generation failed");
    let words: Vec<&str> = passphrase.split_whitespace().collect();
    assert!(is_12_words_secret(&words));
    let short_words: Vec<&str> = vec!["not", "twelve", "words"];
    assert!(!is_12_words_secret(&short_words));
}

#[test]
fn test_secret_to_number_roundtrip() {
    let passphrase = generate_passphrase().expect("passphrase generation failed");
    let number = secret_to_number(&passphrase).expect("secret_to_number failed");
    let recovered = number_to_secret(&number, true).expect("number_to_secret failed");
    assert_eq!(passphrase, recovered, "Roundtrip should recover original passphrase");
}

#[test]
fn test_different_passphrases_different_keys() {
    let kp1 = passphrase_to_keypair("concern entire frozen witch away creak dot drink need season clutch truly")
        .expect("keypair derivation failed");
    let kp2 = passphrase_to_keypair("trickle pierce warm early gentle another thorn gotta illuminate everywhere glare determine")
        .expect("keypair derivation failed");
    assert_ne!(kp1.public_key(), kp2.public_key(), "Different passphrases should produce different keys");
}
