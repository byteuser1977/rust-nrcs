use blockchain_types::*;
use bincode::{options::Options, config::Standard};
use serde_json;

#[test]
fn test_block_bincode_compact_config() {
    let block = Block {
        version: BLOCK_VERSION,
        timestamp: 1700000000,
        height: 100,
        previous_block_hash: [1u8; 32],
        payload_hash: [2u8; 32],
        generator_id: 1234567890,
        nonce: 42,
        base_target: 1_000_000,
        cumulative_difficulty: vec![0u8; 16],
        total_amount: 1_000_000_000,
        total_fee: 100_000,
        payload_length: 0,
        generation_signature: [3u8; 64],
        block_signature: [4u8; 64],
        transactions: vec![],
    };

    let encoded = bincode::serialize(&block).unwrap();
    assert!(encoded.len() > 0);
    assert!(encoded.len() < 500); // reasonable size

    let decoded: Block = bincode::deserialize(&encoded).unwrap();
    assert_eq!(block, decoded);
}

#[test]
fn test_transaction_bincode_compact() {
    let tx = Transaction {
        version: TRANSACTION_VERSION,
        type_id: TransactionType::Payment,
        subtype: 0,
        timestamp: 1700000000,
        deadline: 32767,
        sender_id: 1234567890,
        recipient_id: Some(9876543210),
        amount: 1_000_000_000,
        fee: 100_000,
        height: 0,
        block_id: 0,
        signature: [1u8; 64],
        full_hash: [2u8; 32],
        attachment_bytes: vec![3u8, 4u8, 5u8],
        phased: false,
        has_message: false,
        has_encrypted_message: false,
        has_public_key_announcement: false,
        has_prunable_attachment: false,
        ec_block_height: None,
        ec_block_id: None,
        has_encrypttoself_message: false,
        has_prunable_encrypted_message: false,
    };

    let encoded = bincode::serialize(&tx).unwrap();
    let decoded: Transaction = bincode::deserialize(&encoded).unwrap();
    assert_eq!(tx, decoded);
}

#[test]
fn test_json_serialization_size() {
    let block = Block {
        version: BLOCK_VERSION,
        timestamp: 1700000000,
        height: 100,
        previous_block_hash: [0u8; 32],
        payload_hash: [0u8; 32],
        generator_id: 1234567890,
        nonce: 0,
        base_target: 1_000_000,
        cumulative_difficulty: vec![0u8; 16],
        total_amount: 1_000_000_000,
        total_fee: 100_000,
        payload_length: 0,
        generation_signature: [0u8; 64],
        block_signature: [0u8; 64],
        transactions: vec![],
    };

    let json = block.to_json().unwrap();
    assert!(json.len() > 0);
    // JSON should be human-readable
    assert!(json.contains('\n'));
}

#[test]
fn test_json_deserialize_valid() {
    let json = r#"{
        "version": 1,
        "timestamp": 1700000000,
        "height": 100,
        "previous_block_hash": "0000000000000000000000000000000000000000000000000000000000000000",
        "payload_hash": "0000000000000000000000000000000000000000000000000000000000000000",
        "generator_id": 1234567890,
        "nonce": 42,
        "base_target": 1000000,
        "cumulative_difficulty": [0,0,0,0],
        "total_amount": 1000000000,
        "total_fee": 100000,
        "payload_length": 0,
        "generation_signature": "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "block_signature": "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "transactions": []
    }"#;

    // This should parse without error (though some fields may need adjustment)
    let _block: Block = serde_json::from_str(json).unwrap();
}

#[test]
fn test_hash_types() {
    let hash256: Hash256 = [1u8; 32];
    let hash512: Hash512 = [2u8; 64];
    let pubkey: PublicKey = [3u8; 32];
    let secret: SecretKey = [4u8; 64];
    let sig: Signature = [5u8; 64];

    // These are just type aliases, ensure they work correctly
    assert_eq!(hash256.len(), 32);
    assert_eq!(hash512.len(), 64);
    assert_eq!(pubkey.len(), 32);
    assert_eq!(secret.len(), 64);
    assert_eq!(sig.len(), 64);
}

#[test]
fn test_transaction_attachment_serialization() {
    let original = serde_json::json!({
        "method": "transfer",
        "params": {
            "to": "0x1234567890abcdef",
            "amount": 1000000000,
            "memo": "payment for services"
        }
    });

    let bytes = serde_json::to_vec(&original).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(original, parsed);
}

#[test]
fn test_account_serialization_roundtrip() {
    let mut account = Account::new(1234567890, 1000);
    account.set_address("NRCS-test123".to_string());
    account.properties.insert("name".to_string(), "Test".to_string());

    let json = serde_json::to_string(&account).unwrap();
    let decoded: Account = serde_json::from_str(&json).unwrap();

    assert_eq!(account.id, decoded.id);
    assert_eq!(account.balance, decoded.balance);
    // address is skipped, won't roundtrip
    assert!(decoded.address.is_none());
    assert_eq!(account.properties.len(), decoded.properties.len());
}

#[test]
fn test_tx_receipt_serialization() {
    let receipt = TxReceipt {
        transaction_id: 12345,
        status: 1,
        gas_used: 50000,
        logs: r#"[{"event":"Transfer","args":{"from":"0x123","to":"0x456"}}]"#.to_string(),
        contract_address: Some([1u8; 20]),
        executed_at: 1700000000,
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let decoded: TxReceipt = serde_json::from_str(&json).unwrap();

    assert_eq!(receipt.transaction_id, decoded.transaction_id);
    assert_eq!(receipt.status, decoded.status);
    assert_eq!(receipt.gas_used, decoded.gas_used);
    assert_eq!(receipt.executed_at, decoded.executed_at);
}

#[test]
fn test_tx_receipt_without_contract() {
    let receipt = TxReceipt {
        transaction_id: 12345,
        status: 1,
        gas_used: 0,
        logs: "[]".to_string(),
        contract_address: None,
        executed_at: 1700000000,
    };

    let json = serde_json::to_string(&receipt).unwrap();
    assert!(json.contains("null") || !json.contains("contract_address"));
}

#[test]
fn test_hex_serialization() {
    let hash: Hash256 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
    let hex = hex::encode(hash);
    assert_eq!(hex.len(), 64);
    assert!(hex.starts_with("01020304"));
}
