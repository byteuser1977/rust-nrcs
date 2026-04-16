use orm::models::*;
use blockchain_types::*;
use serde_json;

#[test]
fn test_account_model_creation() {
    let account = AccountModel {
        id: 1234567890,
        address: Some("NRCS-test123".to_string()),
        balance: 1_000_000_000,
        unconfirmed_balance: 1_000_000_000,
        reserved_balance: 0,
        guaranteed_balance: 0,
        assets: serde_json::json!({ "100": 5000, "200": 3000 }),
        properties: serde_json::json!({ "name": "Alice", "email": "alice@example.com" }),
        lease: None,
        created_at: 1700000000,
        last_updated: 1700000000,
        current_height: 100,
    };

    assert_eq!(account.id, 1234567890);
    assert_eq!(account.balance, 1_000_000_000);
    assert!(account.address.is_some());
}

#[test]
fn test_account_model_with_lease() {
    let lease = AccountLeaseModel {
        lessee_id: 9876543210,
        amount: 100_000,
        start_height: 100,
        end_height: 1540,
    };

    let account = AccountModel {
        lease: Some(lease),
        ..AccountModel::default()
    };

    assert!(account.lease.is_some());
    assert_eq!(account.lease.as_ref().unwrap().lessee_id, 9876543210);
}

#[test]
fn test_transaction_model_creation() {
    let tx = TransactionModel {
        id: 12345,
        version: 1,
        type_id: 0, // Payment
        subtype: 0,
        timestamp: 1700000000,
        deadline: 32767,
        sender_id: 1234567890,
        recipient_id: Some(9876543210),
        amount: 1_000_000_000,
        fee: 100_000,
        height: 100,
        block_id: 1,
        signature: "signature_hex_here".to_string(),
        full_hash: "hash_hex_here".to_string(),
        attachment_bytes: None,
        phased: false,
        has_message: false,
        created_at: 1700000000,
    };

    assert_eq!(tx.id, 12345);
    assert_eq!(tx.type_id, 0);
    assert!(tx.recipient_id.is_some());
}

#[test]
fn test_transaction_model_with_attachment() {
    let attachment = r#"{"method":"transfer"}"#.to_string();
    let tx = TransactionModel {
        attachment_bytes: Some(attachment),
        ..TransactionModel::default()
    };

    assert!(tx.attachment_bytes.is_some());
    assert!(tx.attachment_bytes.as_ref().unwrap().contains("transfer"));
}

#[test]
fn test_block_model_creation() {
    let block = BlockModel {
        id: 1,
        version: 1,
        timestamp: 1700000000,
        height: 100,
        previous_block_hash: "prev_hash_hex".to_string(),
        payload_hash: "payload_hash_hex".to_string(),
        generator_id: 1234567890,
        nonce: 42,
        base_target: 1_000_000,
        cumulative_difficulty: "difficulty_bigint".to_string(),
        total_amount: 1_000_000_000,
        total_fee: 100_000,
        payload_length: 0,
        generation_signature: "gen_sig_hex".to_string(),
        block_signature: "block_sig_hex".to_string(),
        created_at: 1700000000,
    };

    assert_eq!(block.height, 100);
    assert_eq!(block.generator_id, 1234567890);
    assert_eq!(block.total_amount, 1_000_000_000);
}

#[test]
fn test_block_model_serialization() {
    let block = BlockModel {
        height: 100,
        ..Default::default()
    };

    // Test JSON serialization (for API responses)
    let json = serde_json::to_string(&block).unwrap();
    let decoded: BlockModel = serde_json::from_str(&json).unwrap();

    assert_eq!(block.height, decoded.height);
}

#[test]
fn test_assets_serialization() {
    // Assets field is JSON in database, should serialize properly
    let assets = serde_json::json!({
        "100": 5000,
        "200": 3000,
        "300": 1000
    });

    let account = AccountModel {
        assets,
        ..AccountModel::default()
    };

    let serialized = serde_json::to_value(&account).unwrap();
    assert!(serialized.get("assets").is_some());
}

#[test]
fn test_properties_serialization() {
    let props = serde_json::json!({
        "name": "Alice",
        "email": "alice@example.com",
        "created_reason": "initial mint"
    });

    let account = AccountModel {
        properties: props,
        ..AccountModel::default()
    };

    let serialized = serde_json::to_value(&account).unwrap();
    assert!(serialized.get("properties").is_some());
}

#[test]
fn test_model_defaults() {
    let account = AccountModel::default();
    assert_eq!(account.balance, 0);
    assert_eq!(account.id, 0);
    assert!(account.address.is_none());
    assert!(account.lease.is_none());
}

#[test]
fn test_tx_receipt_model() {
    let receipt = TxReceiptModel {
        tx_id: 12345,
        status: 1,
        gas_used: 50000,
        logs: r#"[{"event":"Transfer"}]"#.to_string(),
        contract_address: None,
        executed_at: 1700000000,
        created_at: 1700000000,
    };

    assert_eq!(receipt.tx_id, 12345);
    assert_eq!(receipt.status, 1);
    assert!(receipt.logs.contains("Transfer"));
    assert!(receipt.contract_address.is_none());
}

#[test]
fn test_tx_receipt_model_with_contract() {
    let receipt = TxReceiptModel {
        contract_address: Some([1u8; 20].to_vec()),
        ..TxReceiptModel::default()
    };

    assert!(receipt.contract_address.is_some());
    assert_eq!(receipt.contract_address.as_ref().unwrap().len(), 20);
}
