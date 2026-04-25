//! 测试数据库模型与域对象之间的转换

use orm::models::*;
use blockchain_types::prelude::*;

#[test]
fn test_asset_model_conversion() {
    let asset = Asset {
        id: 123,
        owner_id: 456,
        name: "Test Asset".to_string(),
        description: "This is a test asset".to_string(),
        quantity: 1_000_000_000,
        decimals: 8,
        mintable: false,
        transferable: true,
        data: vec![1, 2, 3, 4, 5],
        created_at: 1_700_000_000,
        last_updated: 1_700_000_000,
        deleted: false,
    };

    let model = AssetModel::from_domain(&asset).expect("failed to convert from domain");
    assert_eq!(model.id, 123);
    assert_eq!(model.account_id, 456);
    assert_eq!(model.name, "Test Asset");
    assert_eq!(model.description, Some("This is a test asset".to_string()));
    assert_eq!(model.quantity, 1_000_000_000);
    assert_eq!(model.decimals, 8);

    let converted_asset = model.to_domain().expect("failed to convert to domain");
    assert_eq!(converted_asset.id, asset.id);
    assert_eq!(converted_asset.owner_id, asset.owner_id);
    assert_eq!(converted_asset.name, asset.name);
    assert_eq!(converted_asset.description, asset.description);
    assert_eq!(converted_asset.quantity, asset.quantity);
    assert_eq!(converted_asset.decimals, asset.decimals);
    assert_eq!(converted_asset.mintable, asset.mintable);
    assert_eq!(converted_asset.transferable, asset.transferable);
    assert_eq!(converted_asset.created_at, asset.created_at);
    assert_eq!(converted_asset.last_updated, asset.last_updated);
    assert_eq!(converted_asset.deleted, asset.deleted);
}

#[test]
fn test_account_asset_model_conversion() {
    let account_asset = AccountAsset {
        account_id: 123,
        asset_id: 456,
        quantity: 1000,
        last_updated: 1_700_000_000,
    };

    let model = AccountAssetModel::from_domain(&account_asset).expect("failed to convert from domain");
    assert_eq!(model.account_id, 123);
    assert_eq!(model.asset_id, 456);
    assert_eq!(model.quantity, 1000);

    let converted = model.to_domain().expect("failed to convert to domain");
    assert_eq!(converted.account_id, account_asset.account_id);
    assert_eq!(converted.asset_id, account_asset.asset_id);
    assert_eq!(converted.quantity, account_asset.quantity);
}

#[test]
fn test_block_model_conversion() {
    let block = Block {
        version: BLOCK_VERSION,
        timestamp: 1_700_000_000,
        height: 100,
        previous_block_hash: Hash256([0x11; 32]),
        payload_hash: Hash256([0x22; 32]),
        generator_id: Some(12345),
        nonce: 0,
        base_target: 1_000_000,
        cumulative_difficulty: vec![1, 2, 3, 4],
        total_amount: 10_000,
        total_fee: 100,
        payload_length: 0,
        generation_signature: Hash512([0x33; 64]),
        block_signature: Hash512([0x44; 64]),
        transactions: vec![],
    };

    let model = BlockModel::from_domain(&block).expect("failed to convert from domain");
    assert_eq!(model.height, 100);
    assert_eq!(model.generator_id, 12345);
    assert_eq!(model.total_amount, 10_000);

    let converted = model.to_domain().expect("failed to convert to domain");
    assert_eq!(converted.version, block.version);
    assert_eq!(converted.timestamp, block.timestamp);
    assert_eq!(converted.height, block.height);
    assert_eq!(converted.generator_id, block.generator_id);
    assert_eq!(converted.total_amount, block.total_amount);
    assert_eq!(converted.total_fee, block.total_fee);
}

#[test]
fn test_account_model_conversion() {
    let account = Account::new(1234567890, 10_000_000_000);

    let model = AccountModel::from_domain(&account).expect("failed to convert from domain");
    assert_eq!(model.id, 1234567890);
    assert_eq!(model.balance, 10_000_000_000);

    let converted = model.to_domain().expect("failed to convert to domain");
    assert_eq!(converted.id, account.id);
    assert_eq!(converted.balance, account.balance);
    assert_eq!(converted.unconfirmed_balance, account.unconfirmed_balance);
}

#[test]
fn test_transaction_model_conversion() {
    let mut tx = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        1_000_000_000,
        100_000,
        1_700_000_000,
        32767,
    );
    tx.full_hash = tx.compute_hash().unwrap();

    let model = TransactionModel::from_domain(&tx).expect("failed to convert from domain");
    assert_eq!(model.sender_id, 1234567890);
    assert_eq!(model.amount, 1_000_000_000);
    assert_eq!(model.r#type, 0); // Payment is 0

    let converted = model.to_domain().expect("failed to convert to domain");
    assert_eq!(converted.type_id, TransactionType::Payment);
    assert_eq!(converted.sender_id, tx.sender_id);
    assert_eq!(converted.recipient_id, tx.recipient_id);
    assert_eq!(converted.amount, tx.amount);
    assert_eq!(converted.fee, tx.fee);
    assert_eq!(converted.full_hash, tx.full_hash);
}
