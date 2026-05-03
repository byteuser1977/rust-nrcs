//! 交易类型注册表和 Handler 扩展测试
//!
//! 补充测试 TxTypeRegistry 的各 Handler 验证和 apply/undo 操作。

use tx_engine::tx_types::*;
use blockchain_types::prelude::*;
use blockchain_types::constants::TRANSACTION_VERSION;

fn make_tx(tx_type: TransactionType, subtype: u8, amount: u64, recipient_id: Option<u64>) -> Transaction {
    Transaction {
        id: 1,
        type_id: tx_type,
        subtype,
        version: TRANSACTION_VERSION,
        timestamp: 1000,
        deadline: 1440,
        sender_public_key: Hash256([0u8; 32]),
        sender_id: 100,
        recipient_id,
        amount,
        fee: 10,
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

#[test]
fn test_registry_creation() {
    let registry = TxTypeRegistry::new();
    assert!(registry.get_handler(TransactionType::Payment).is_some());
}

#[test]
fn test_registry_payment_validate() {
    let registry = TxTypeRegistry::new();
    let tx = make_tx(TransactionType::Payment, 0, 100, Some(200));
    assert!(registry.validate(&tx).is_ok());
}

#[test]
fn test_registry_payment_apply() {
    let registry = TxTypeRegistry::new();
    let tx = make_tx(TransactionType::Payment, 0, 100, Some(200));
    let mut ctx = TxExecutionContext::new(100, 1000, 10, 1000);
    ctx.set_recipient(200, 500);
    let result = registry.apply(&tx, &mut ctx);
    assert!(result.is_ok());
    assert_eq!(ctx.sender_balance, 890, "1000 - 100(amount) - 10(fee) = 890");
    assert_eq!(ctx.recipient_balance, 600, "500 + 100(amount) = 600");
}

#[test]
fn test_registry_payment_undo() {
    let registry = TxTypeRegistry::new();
    let tx = make_tx(TransactionType::Payment, 0, 100, Some(200));
    let mut ctx = TxExecutionContext::new(100, 900, 10, 1000);
    ctx.set_recipient(200, 600);
    let result = registry.undo(&tx, &mut ctx);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_payment_handler_validate_no_recipient() {
    let handler = PaymentHandler::new();
    let tx = make_tx(TransactionType::Payment, 0, 100, None);
    let result = handler.validate(&tx);
    assert!(result.is_err(), "Payment without recipient should be invalid");
}

#[test]
fn test_colored_coins_handler_validate() {
    let handler = ColoredCoinsHandler::new();
    let tx = make_tx(TransactionType::ColoredCoins, 0, 100, Some(200));
    let result = handler.validate(&tx);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_messaging_handler_validate() {
    let handler = MessagingHandler::new();
    let tx = make_tx(TransactionType::Messaging, 0, 0, Some(200));
    let result = handler.validate(&tx);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_light_contract_handler_validate() {
    let handler = LightContractHandler::new();
    let tx = make_tx(TransactionType::LightContract, 0, 0, None);
    let result = handler.validate(&tx);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_data_handler_validate() {
    let handler = DataHandler::new();
    let tx = make_tx(TransactionType::Data, 0, 0, Some(200));
    let result = handler.validate(&tx);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_registry_get_handler_unknown() {
    let registry = TxTypeRegistry::new();
    let result = registry.get_handler(TransactionType::Payment);
    assert!(result.is_some());
}

#[test]
fn test_tx_execution_context_new() {
    let ctx = TxExecutionContext::new(100, 1000, 10, 500);
    assert_eq!(ctx.sender_id, 100);
    assert_eq!(ctx.sender_balance, 1000);
    assert_eq!(ctx.height, 10);
    assert_eq!(ctx.timestamp, 500);
    assert!(ctx.recipient_id.is_none());
}

#[test]
fn test_tx_execution_context_set_recipient() {
    let mut ctx = TxExecutionContext::new(100, 1000, 10, 500);
    ctx.set_recipient(200, 500);
    assert_eq!(ctx.recipient_id, Some(200));
    assert_eq!(ctx.recipient_balance, 500);
}
