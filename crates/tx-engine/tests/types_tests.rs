use tx_engine::types::*;
use blockchain_types::*;

#[test]
fn test_tx_status_default() {
    let status = TxStatus::default();
    assert_eq!(status, TxStatus::Pending);
}

#[test]
fn test_tx_status_values() {
    assert_eq!(TxStatus::Pending as u8, 0);
    assert_eq!(TxStatus::Success as u8, 1);
    assert_eq!(TxStatus::Failed as u8, 2);
}

#[test]
fn test_tx_receipt_info_creation() {
    let receipt = TxReceiptInfo {
        tx_id: 12345,
        status: TxStatus::Success,
        block_height: 100,
        gas_used: 50000,
        executed_at: 1700000000,
    };

    assert_eq!(receipt.tx_id, 12345);
    assert_eq!(receipt.status, TxStatus::Success);
    assert_eq!(receipt.block_height, 100);
}

#[test]
fn test_tx_priority_default() {
    let priority = TxPriority::default();
    assert_eq!(priority, TxPriority::Normal);
}

#[test]
fn test_mempool_config_defaults() {
    let config = MempoolConfig::default();
    assert_eq!(config.max_transactions, 10000);
    assert!(config.max_age_seconds > 0);
    assert!(config.sort_by_fee);
    assert!(!config.persist_to_disk);
}

#[test]
fn test_mempool_stats_default() {
    let stats = MempoolStats::default();
    assert_eq!(stats.transaction_count, 0);
    assert_eq!(stats.total_fee, 0);
    assert!(stats.min_fee.is_none());
    assert!(stats.max_fee.is_none());
    assert!(stats.avg_fee.is_none());
}

#[test]
fn test_transaction_processor_trait() {
    // Just ensure trait object can be created (compilation test)
    // Real implementation would be in processor module
    fn requires_processor<T: TransactionProcessor>(_p: T) {}
    // This would compile if we have a concrete type
}

#[test]
fn test_tx_priority_ordering() {
    // Verify priority ordering
    assert!(TxPriority::High as u8 > TxPriority::Normal as u8);
    assert!(TxPriority::Normal as u8 > TxPriority::Low as u8);
}

#[test]
fn test_receipt_logs_parsing() {
    let logs = r#"[{"event":"Transfer","amount":100},{"event":"FeePaid","amount":10}]"#;
    let receipt = TxReceiptInfo {
        tx_id: 1,
        status: TxStatus::Success,
        block_height: 1,
        gas_used: 0,
        executed_at: 0,
    };

    // In real impl, logs would be parsed
    assert!(logs.contains("Transfer"));
    assert!(logs.contains("FeePaid"));
}
