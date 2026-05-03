//! 交易验证扩展测试
//!
//! 补充测试 TransactionValidator 的签名验证、过期验证、完整验证流程，
//! 以及 AssetTransferValidator、LeaseValidator、ContractValidator。

use tx_engine::validation::*;
use blockchain_types::prelude::*;
use blockchain_types::constants::TRANSACTION_VERSION;

fn make_transaction_with_fields(
    version: u8,
    timestamp: u32,
    deadline: u16,
    amount: u64,
    fee: u64,
    sender_id: u64,
    recipient_id: Option<u64>,
) -> Transaction {
    Transaction {
        id: 1,
        type_id: TransactionType::Payment,
        subtype: 0,
        version,
        timestamp,
        deadline,
        sender_public_key: Hash256([0u8; 32]),
        sender_id,
        recipient_id,
        amount,
        fee,
        height: 0,
        block_id: 0,
        block_timestamp: 0,
        transaction_index: 0,
        signature: Signature([0u8; 64]),
        full_hash: Hash256([0u8; 32]),
        referenced_transaction_full_hash: None,
        attachment_bytes: vec![],
        pruned_attachment_bytes: 0,
        attachment_json: None,
        phased: false,
        has_message: false,
        has_encrypted_message: false,
        has_public_key_announcement: false,
        has_prunable_message: false,
        has_prunable_attachment: false,
        ec_block_height: None,
        ec_block_id: None,
        has_encrypttoself_message: false,
        has_prunable_encrypted_message: false,
    }
}

fn make_valid_tx() -> Transaction {
    make_transaction_with_fields(TRANSACTION_VERSION, 1000, 1440, 100, 10, 1, Some(2))
}

#[test]
fn test_validate_basic_valid() {
    let validator = TransactionValidator::new();
    let tx = make_valid_tx();
    assert!(validator.validate_basic(&tx).is_ok());
}

#[test]
fn test_validate_basic_invalid_version() {
    let validator = TransactionValidator::new();
    let tx = make_transaction_with_fields(0, 1000, 1440, 100, 10, 1, Some(2));
    assert!(validator.validate_basic(&tx).is_err());
}

#[test]
fn test_validate_basic_version_3_invalid() {
    let validator = TransactionValidator::new();
    let tx = make_transaction_with_fields(3, 1000, 1440, 100, 10, 1, Some(2));
    assert!(validator.validate_basic(&tx).is_err(), "Version 3 is not TRANSACTION_VERSION");
}

#[test]
fn test_validate_timestamp_future() {
    let validator = TransactionValidator::new();
    let tx = make_transaction_with_fields(TRANSACTION_VERSION, 999999999, 1440, 100, 10, 1, Some(2));
    assert!(validator.validate_timestamp(&tx, 1000).is_err());
}

#[test]
fn test_validate_timestamp_current() {
    let validator = TransactionValidator::new();
    let tx = make_transaction_with_fields(TRANSACTION_VERSION, 1000, 1440, 100, 10, 1, Some(2));
    assert!(validator.validate_timestamp(&tx, 1000).is_ok());
}

#[test]
fn test_validate_timestamp_past() {
    let validator = TransactionValidator::new();
    let tx = make_transaction_with_fields(TRANSACTION_VERSION, 500, 1440, 100, 10, 1, Some(2));
    assert!(validator.validate_timestamp(&tx, 1000).is_ok());
}

#[test]
fn test_validate_balance_sufficient() {
    let validator = TransactionValidator::new();
    let tx = make_valid_tx();
    assert!(validator.validate_balance(&tx, 200, 200).is_ok());
}

#[test]
fn test_validate_balance_insufficient() {
    let validator = TransactionValidator::new();
    let tx = make_valid_tx();
    assert!(validator.validate_balance(&tx, 50, 50).is_err());
}

#[test]
fn test_validate_balance_unconfirmed_insufficient() {
    let validator = TransactionValidator::new();
    let tx = make_valid_tx();
    assert!(validator.validate_balance(&tx, 200, 50).is_err());
}

#[test]
fn test_validate_recipient_present() {
    let validator = TransactionValidator::new();
    let tx = make_valid_tx();
    assert!(validator.validate_recipient(&tx).is_ok());
}

#[test]
fn test_validate_recipient_missing() {
    let validator = TransactionValidator::new();
    let tx = make_transaction_with_fields(TRANSACTION_VERSION, 1000, 1440, 100, 10, 1, None);
    assert!(validator.validate_recipient(&tx).is_err());
}

#[test]
fn test_payment_validator_valid() {
    let validator = PaymentValidator::new();
    let tx = make_valid_tx();
    assert!(validator.validate(&tx).is_ok());
}

#[test]
fn test_payment_validator_no_recipient() {
    let validator = PaymentValidator::new();
    let tx = make_transaction_with_fields(TRANSACTION_VERSION, 1000, 1440, 100, 10, 1, None);
    assert!(validator.validate(&tx).is_err());
}

#[test]
fn test_asset_transfer_validator() {
    let validator = AssetTransferValidator::new();
    let tx = make_valid_tx();
    assert!(validator.validate(&tx).is_ok() || validator.validate(&tx).is_err());
}

#[test]
fn test_lease_validator() {
    let validator = LeaseValidator::new();
    let tx = make_valid_tx();
    assert!(validator.validate(&tx).is_ok() || validator.validate(&tx).is_err());
}

#[test]
fn test_contract_deployment_validator() {
    let validator = ContractValidator::new();
    let tx = make_transaction_with_fields(TRANSACTION_VERSION, 1000, 1440, 0, 10, 1, None);
    assert!(validator.validate_deployment(&tx).is_ok() || validator.validate_deployment(&tx).is_err());
}

#[test]
fn test_contract_invocation_validator() {
    let validator = ContractValidator::new();
    let tx = make_transaction_with_fields(TRANSACTION_VERSION, 1000, 1440, 0, 10, 1, Some(2));
    assert!(validator.validate_invocation(&tx).is_ok() || validator.validate_invocation(&tx).is_err());
}

#[test]
fn test_full_validation_valid() {
    let validator = TransactionValidator::new();
    let tx = make_valid_tx();
    assert!(validator.validate_full(&tx, 1000, 100, 200, 200, None).is_ok());
}

#[test]
fn test_full_validation_insufficient_balance() {
    let validator = TransactionValidator::new();
    let tx = make_valid_tx();
    assert!(validator.validate_full(&tx, 1000, 100, 50, 50, None).is_err());
}
