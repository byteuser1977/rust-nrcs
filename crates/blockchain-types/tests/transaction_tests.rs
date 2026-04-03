use blockchain_types::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_transaction_creation() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let tx = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        1_000_000_000, // 10 NRC
        100_000,       // fee
        now,
        32767,
    );

    assert_eq!(tx.version, TRANSACTION_VERSION);
    assert_eq!(tx.type_id, TransactionType::Payment);
    assert_eq!(tx.sender_id, 1234567890);
    assert_eq!(tx.recipient_id, Some(9876543210));
    assert_eq!(tx.amount, 1_000_000_000);
    assert_eq!(tx.fee, 100_000);
    assert_eq!(tx.timestamp, now);
    assert_eq!(tx.deadline, 32767);
    assert!(tx.signature == [0u8; 64]);
    assert!(tx.full_hash == [0u8; 32]);
}

#[test]
fn test_transaction_hash_consistency() {
    let now = 1700000000u32;

    let mut tx = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        1_000_000_000,
        100_000,
        now,
        32767,
    );

    let hash1 = tx.compute_hash().unwrap();
    let hash2 = tx.compute_hash().unwrap();

    assert_eq!(hash1, hash2);
    assert_ne!(hash1, [0u8; 32]);
}

#[test]
fn test_transaction_hash_different_params() {
    let now = 1700000000u32;

    let tx1 = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        1_000_000_000,
        100_000,
        now,
        32767,
    );

    let tx2 = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        2_000_000_000, // different amount
        100_000,
        now,
        32767,
    );

    let hash1 = tx1.compute_hash().unwrap();
    let hash2 = tx2.compute_hash().unwrap();

    assert_ne!(hash1, hash2);
}

#[test]
fn test_transaction_validate_basic_success() {
    let mut tx = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        1_000_000_000,
        100_000,
        1700000000,
        32767,
    );
    tx.full_hash = tx.compute_hash().unwrap();

    assert!(tx.validate_basic().is_ok());
}

#[test]
fn test_transaction_validate_basic_invalid_version() {
    let mut tx = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        1_000_000_000,
        100_000,
        1700000000,
        32767,
    );
    tx.version = 99;
    tx.full_hash = tx.compute_hash().unwrap();

    let result = tx.validate_basic();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("version"));
}

#[test]
fn test_transaction_validate_basic_zero_fee() {
    let mut tx = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        1_000_000_000,
        0, // zero fee
        1700000000,
        32767,
    );
    tx.full_hash = tx.compute_hash().unwrap();

    let result = tx.validate_basic();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("fee"));
}

#[test]
fn test_transaction_validate_basic_future_timestamp() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let mut tx = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        1_000_000_000,
        100_000,
        now + 7200, // 2 hours in future
        32767,
    );
    tx.full_hash = tx.compute_hash().unwrap();

    let result = tx.validate_basic();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("timestamp"));
}

#[test]
fn test_transaction_validate_basic_zero_sender() {
    let mut tx = Transaction::new(
        TransactionType::Payment,
        0, // zero sender
        Some(9876543210),
        1_000_000_000,
        100_000,
        1700000000,
        32767,
    );
    tx.full_hash = tx.compute_hash().unwrap();

    let result = tx.validate_basic();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("sender"));
}

#[test]
fn test_transaction_validate_basic_zero_amount() {
    let mut tx = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        0, // zero amount
        100_000,
        1700000000,
        32767,
    );
    tx.full_hash = tx.compute_hash().unwrap();

    let result = tx.validate_basic();
    assert!(result.is_err());
}

#[test]
fn test_transaction_size_calculation() {
    let tx = Transaction {
        version: 1,
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
        signature: [0u8; 64],
        full_hash: [0u8; 32],
        attachment_bytes: vec![],
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

    let size = tx.size();
    assert!(size > 100);
    assert!(size < 300); // reasonable size for minimal transaction
}

#[test]
fn test_transaction_with_attachment() {
    let mut tx = Transaction::new(
        TransactionType::ContractInvocation,
        1234567890,
        None,
        0, // contract calls can have 0 amount
        100_000,
        1700000000,
        32767,
    );
    // Add JSON attachment
    let args = serde_json::json!({"method": "transfer", "params": {"to": "0x123", "amount": 100}});
    tx.attachment_bytes = serde_json::to_vec(&args).unwrap();
    tx.full_hash = tx.compute_hash().unwrap();

    assert!(tx.attachment_bytes.len() > 0);
    assert!(tx.validate_basic().is_ok());
}

#[test]
fn test_transaction_type_conversion() {
    assert_eq!(u8::from(TransactionType::Payment), 0);
    assert_eq!(u8::from(TransactionType::AssetTransfer), 1);
    assert_eq!(u8::from(TransactionType::AssetIssuance), 2);
    assert_eq!(u8::from(TransactionType::ContractInvocation), 3);
    assert_eq!(u8::from(TransactionType::ContractDeployment), 4);
    assert_eq!(u8::from(TransactionType::Lease), 5);
    assert_eq!(u8::from(TransactionType::SetProperty), 6);

    assert_eq!(TransactionType::from(0), TransactionType::Payment);
    assert_eq!(TransactionType::from(1), TransactionType::AssetTransfer);
    assert_eq!(TransactionType::from(255), TransactionType::Custom(255));
}

#[test]
fn test_transaction_serialization_roundtrip() {
    let mut tx = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        1_000_000_000,
        100_000,
        1700000000,
        32767,
    );
    tx.full_hash = tx.compute_hash().unwrap();

    // JSON roundtrip (public fields)
    let json = tx.to_json().unwrap();
    let _decoded: Transaction = serde_json::from_str(&json).unwrap();

    // Bincode roundtrip (full struct)
    let binary = tx.to_bincode().unwrap();
    let decoded: Transaction = Transaction::from_bincode(&binary).unwrap();
    assert_eq!(tx, decoded);
}

#[test]
fn test_transaction_is_payment() {
    let payment_tx = Transaction {
        type_id: TransactionType::Payment,
        ..Default::default()
    };
    assert!(payment_tx.is_payment());

    let contract_tx = Transaction {
        type_id: TransactionType::ContractInvocation,
        ..Default::default()
    };
    assert!(!contract_tx.is_payment());
}

#[test]
fn test_transaction_is_contract_call() {
    let deploy_tx = Transaction {
        type_id: TransactionType::ContractDeployment,
        ..Default::default()
    };
    assert!(deploy_tx.is_contract_call());

    let invoke_tx = Transaction {
        type_id: TransactionType::ContractInvocation,
        ..Default::default()
    };
    assert!(invoke_tx.is_contract_call());

    let payment_tx = Transaction {
        type_id: TransactionType::Payment,
        ..Default::default()
    };
    assert!(!payment_tx.is_contract_call());
}

#[test]
fn test_contract_args_parsing() {
    let mut tx = Transaction::new(
        TransactionType::ContractInvocation,
        1234567890,
        None,
        0,
        100_000,
        1700000000,
        32767,
    );

    #[derive(serde::Deserialize)]
    struct Args {
        method: String,
        params: serde_json::Value,
    }

    let args = Args {
        method: "transfer".to_string(),
        params: serde_json::json!({"to": "0x123", "amount": 100}),
    };
    tx.attachment_bytes = serde_json::to_vec(&args).unwrap();
    tx.full_hash = tx.compute_hash().unwrap();

    let parsed: Args = tx.contract_args().unwrap();
    assert_eq!(parsed.method, "transfer");
}

#[test]
fn test_transaction_default() {
    let tx = Transaction::default();
    assert_eq!(tx.version, TRANSACTION_VERSION);
    assert_eq!(tx.type_id, TransactionType::Payment);
    assert_eq!(tx.timestamp, 0);
    assert_eq!(tx.fee, 0);
    assert!(tx.signature == [0u8; 64]);
}
