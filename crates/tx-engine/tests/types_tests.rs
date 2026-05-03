//! Types 外部测试
//!
//! 测试 TxPriority、TxStatus、TxReceiptInfo 等核心类型

use tx_engine::{TxPriority, TxStatus, TxReceiptInfo};
use blockchain_types::prelude::*;

#[test]
fn test_tx_status_variants() {
    assert_eq!(TxStatus::Pending, TxStatus::Pending);
    assert_eq!(TxStatus::Confirmed, TxStatus::Confirmed);
    assert_eq!(TxStatus::Success, TxStatus::Success);
    assert_eq!(TxStatus::Failed, TxStatus::Failed);
    assert_ne!(TxStatus::Pending, TxStatus::Confirmed);
}

#[test]
fn test_tx_priority_score() {
    let p1 = TxPriority {
        gas_price: 100,
        timestamp: 1000,
        size: 256,
    };
    let p2 = TxPriority {
        gas_price: 200,
        timestamp: 1000,
        size: 256,
    };
    let p3 = TxPriority {
        gas_price: 100,
        timestamp: 2000,
        size: 256,
    };

    assert!(p2.score() > p1.score(), "Higher gas_price should have higher score");
    assert!(p1.score() > p3.score(), "Earlier timestamp should have higher score");
}

#[test]
fn test_tx_priority_ordering() {
    let p1 = TxPriority {
        gas_price: 100,
        timestamp: 1000,
        size: 256,
    };
    let p2 = TxPriority {
        gas_price: 200,
        timestamp: 1000,
        size: 256,
    };

    assert!(p2 < p1, "Higher score (p2) should sort before lower (p1) via reverse ordering");
}

#[test]
fn test_tx_priority_same_gas_price() {
    let p1 = TxPriority {
        gas_price: 100,
        timestamp: 500,
        size: 256,
    };
    let p2 = TxPriority {
        gas_price: 100,
        timestamp: 1000,
        size: 256,
    };

    assert!(p1 < p2, "Same gas_price, earlier timestamp has higher score, sorts first via reverse");
}

#[test]
fn test_tx_receipt_info_creation() {
    let receipt = TxReceiptInfo {
        transaction_id: 12345,
        status: TxStatus::Success,
        block_height: Some(100),
        gas_used: 21000,
        logs: vec!["log1".to_string(), "log2".to_string()],
        contract_address: None,
        executed_at: 1234567890,
    };

    assert_eq!(receipt.transaction_id, 12345);
    assert_eq!(receipt.status, TxStatus::Success);
    assert_eq!(receipt.block_height, Some(100));
    assert_eq!(receipt.gas_used, 21000);
    assert_eq!(receipt.logs.len(), 2);
    assert!(receipt.contract_address.is_none());
    assert_eq!(receipt.executed_at, 1234567890);
}

#[test]
fn test_tx_receipt_info_from_domain() {
    let domain_receipt = TxReceipt {
        transaction_id: 99999,
        status: 1,
        gas_used: 50000,
        logs: String::new(),
        contract_address: None,
        executed_at: 9999,
    };

    let receipt = TxReceiptInfo::from_domain(&domain_receipt, Some(50));

    assert_eq!(receipt.transaction_id, 99999);
    assert_eq!(receipt.status, TxStatus::Success);
    assert_eq!(receipt.block_height, Some(50));
    assert_eq!(receipt.gas_used, 50000);
}

#[test]
fn test_tx_receipt_info_from_domain_failed() {
    let domain_receipt = TxReceipt {
        transaction_id: 88888,
        status: 2,
        gas_used: 10000,
        logs: String::new(),
        contract_address: None,
        executed_at: 8888,
    };

    let receipt = TxReceiptInfo::from_domain(&domain_receipt, None);
    assert_eq!(receipt.status, TxStatus::Failed);
    assert!(receipt.block_height.is_none());
}

#[test]
fn test_tx_receipt_info_from_domain_pending() {
    let domain_receipt = TxReceipt {
        transaction_id: 77777,
        status: 0,
        gas_used: 0,
        logs: String::new(),
        contract_address: None,
        executed_at: 7777,
    };

    let receipt = TxReceiptInfo::from_domain(&domain_receipt, None);
    assert_eq!(receipt.status, TxStatus::Pending);
}

#[test]
fn test_tx_receipt_info_with_contract_address() {
    let addr = [1u8; 20];
    let receipt = TxReceiptInfo {
        transaction_id: 55555,
        status: TxStatus::Success,
        block_height: Some(200),
        gas_used: 100000,
        logs: vec![],
        contract_address: Some(addr),
        executed_at: 5555,
    };

    assert_eq!(receipt.contract_address, Some(addr));
}

#[test]
fn test_tx_priority_score_calculation() {
    let p = TxPriority {
        gas_price: 50,
        timestamp: 10000,
        size: 512,
    };

    let expected_score = 50i64 * 1000 - 10000i64;
    assert_eq!(p.score(), expected_score);
}

#[test]
fn test_tx_priority_zero_gas_price() {
    let p = TxPriority {
        gas_price: 0,
        timestamp: 0,
        size: 100,
    };

    assert_eq!(p.score(), 0);
}

#[test]
fn test_tx_status_copy() {
    let status = TxStatus::Success;
    let copied = status;
    assert_eq!(status, copied);
}

#[test]
fn test_tx_priority_equality() {
    let p1 = TxPriority {
        gas_price: 100,
        timestamp: 1000,
        size: 256,
    };
    let p2 = TxPriority {
        gas_price: 100,
        timestamp: 1000,
        size: 256,
    };

    assert_eq!(p1, p2);
}
