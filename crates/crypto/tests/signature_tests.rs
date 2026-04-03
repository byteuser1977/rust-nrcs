use crypto::*;
use ed25519_dalek::{Keypair, Signer, Verifier};

#[test]
fn test_sign_ed25519() {
    let keypair = Keypair::generate(&mut rand::rngs::OsRng);
    let message = b"Important transaction";

    let signature = sign_ed25519(message, &keypair.secret);

    // Verify with public key
    assert!(keypair.public.verify(message, &signature).is_ok());
}

#[test]
fn test_verify_ed25519_valid() {
    let keypair = Keypair::generate(&mut rand::rngs::OsRng);
    let message = b"Test message";
    let signature = keypair.sign(message);

    let result = verify_ed25519(message, &signature, &keypair.public);
    assert!(result.is_ok());
}

#[test]
fn test_verify_ed25519_invalid() {
    let keypair1 = Keypair::generate(&mut rand::rngs::OsRng);
    let keypair2 = Keypair::generate(&mut rand::rngs::OsRng);
    let message = b"Test message";

    let signature = keypair1.sign(message);
    let result = verify_ed25519(message, &signature, &keypair2.public);

    assert!(result.is_err());
}

#[test]
fn test_verify_ed25519_tampered_message() {
    let keypair = Keypair::generate(&mut rand::rngs::OsRng);
    let message = b"Original message";
    let signature = keypair.sign(message);

    let tampered_message = b"Tampered message";
    let result = verify_ed25519(tampered_message, &signature, &keypair.public);

    assert!(result.is_err());
}

#[test]
fn test_sign_deterministic() {
    let keypair = Keypair::generate(&mut rand::rngs::OsRng);
    let message = b"Deterministic test";

    let sig1 = sign_ed25519(message, &keypair.secret);
    let sig2 = sign_ed25519(message, &keypair.secret);

    // Ed25519 signatures should be deterministic given same key and message
    assert_eq!(sig1, sig2);
}

#[test]
fn test_sign_large_message() {
    let keypair = Keypair::generate(&mut rand::rngs::OsRng);
    let message = vec![0x55; 100000]; // 100KB

    let signature = sign_ed25519(&message, &keypair.secret);
    assert!(keypair.public.verify(&message, &signature).is_ok());
}

#[test]
fn test_sign_empty_message() {
    let keypair = Keypair::generate(&mut rand::rngs::OsRng);
    let message = b"";

    let signature = sign_ed25519(message, &keypair.secret);
    assert!(keypair.public.verify(message, &signature).is_ok());
}

#[test]
fn test_signature_size() {
    let keypair = Keypair::generate(&mut rand::rngs::OsRng);
    let message = b"test";
    let signature = sign_ed25519(message, &keypair.secret);

    assert_eq!(signature.len(), 64); // Ed25519 signature is 64 bytes
}

#[test]
fn test_verify_multiple_signatures() {
    let keypair = Keypair::generate(&mut rand::rngs::OsRng);
    let messages = vec![b"msg1", b"msg2", b"msg3"];

    let signatures: Vec<_> = messages.iter()
        .map(|m| sign_ed25519(m, &keypair.secret))
        .collect();

    for (msg, sig) in messages.iter().zip(signatures.iter()) {
        assert!(verify_ed25519(msg, sig, &keypair.public).is_ok());
    }
}

#[test]
fn test_sign_and_verify_random_messages() {
    use rand::Rng;
    let mut rng = rand::rngs::OsRng;
    let keypair = Keypair::generate(&mut rng);

    for _ in 0..100 {
        let mut message = vec![0u8; 100];
        rng.fill(&mut message[..]);

        let signature = sign_ed25519(&message, &keypair.secret);
        assert!(verify_ed25519(&message, &signature, &keypair.public).is_ok());
    }
}
