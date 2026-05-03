//! Monitor 外部测试
//!
//! 测试 FundingMonitor、FundingMonitorService、HoldingType、MonitoredAccount 等资金监控功能

use tx_engine::monitor::{
    FundingMonitor, FundingMonitorService, HoldingType,
};
use tx_engine::monitor::funding_monitor::FundingMonitorError;
use tx_engine::monitor::monitored_account::MonitoredAccount;

#[test]
fn test_holding_type_from_code() {
    assert_eq!(HoldingType::from_code(0), Some(HoldingType::Nrcs));
    assert_eq!(HoldingType::from_code(1), Some(HoldingType::Asset));
    assert_eq!(HoldingType::from_code(2), Some(HoldingType::Currency));
    assert_eq!(HoldingType::from_code(99), None);
}

#[test]
fn test_holding_type_code() {
    assert_eq!(HoldingType::Nrcs.code(), 0);
    assert_eq!(HoldingType::Asset.code(), 1);
    assert_eq!(HoldingType::Currency.code(), 2);
}

#[test]
fn test_holding_type_code_roundtrip() {
    for ht in [HoldingType::Nrcs, HoldingType::Asset, HoldingType::Currency] {
        assert_eq!(HoldingType::from_code(ht.code()), Some(ht));
    }
}

#[test]
fn test_funding_monitor_new_nrcs() {
    let monitor = FundingMonitor::new(
        HoldingType::Nrcs,
        Some(999),
        "fund".to_string(),
        1000,
        100,
        10,
        12345,
        vec![1, 2, 3],
    );

    assert_eq!(monitor.holding_type, HoldingType::Nrcs);
    assert_eq!(monitor.holding_id, None, "NRCS holding_id should be None");
    assert_eq!(monitor.amount, 1000);
    assert_eq!(monitor.threshold, 100);
    assert_eq!(monitor.interval, 10);
    assert_eq!(monitor.account_id, 12345);
}

#[test]
fn test_funding_monitor_new_asset() {
    let monitor = FundingMonitor::new(
        HoldingType::Asset,
        Some(555),
        "fund".to_string(),
        1000,
        100,
        10,
        12345,
        vec![1, 2, 3],
    );

    assert_eq!(monitor.holding_type, HoldingType::Asset);
    assert_eq!(monitor.holding_id, Some(555), "Asset holding_id should be preserved");
}

#[test]
fn test_funding_monitor_equality() {
    let m1 = FundingMonitor::new(
        HoldingType::Nrcs, None, "fund".to_string(), 1000, 100, 10, 12345, vec![1, 2, 3],
    );
    let m2 = FundingMonitor::new(
        HoldingType::Nrcs, None, "fund".to_string(), 2000, 200, 20, 12345, vec![4, 5, 6],
    );

    assert_eq!(m1, m2, "Same holding_type/property/account_id should be equal");
}

#[test]
fn test_funding_monitor_inequality() {
    let m1 = FundingMonitor::new(
        HoldingType::Nrcs, None, "fund".to_string(), 1000, 100, 10, 12345, vec![],
    );
    let m2 = FundingMonitor::new(
        HoldingType::Asset, Some(1), "fund".to_string(), 1000, 100, 10, 12345, vec![],
    );

    assert_ne!(m1, m2, "Different holding_type should not be equal");
}

#[test]
fn test_funding_monitor_inequality_different_property() {
    let m1 = FundingMonitor::new(
        HoldingType::Nrcs, None, "fund".to_string(), 1000, 100, 10, 12345, vec![],
    );
    let m2 = FundingMonitor::new(
        HoldingType::Nrcs, None, "other".to_string(), 1000, 100, 10, 12345, vec![],
    );

    assert_ne!(m1, m2);
}

#[test]
fn test_monitored_account_new() {
    let account = MonitoredAccount::new(100, 0, 1000, 100, 10).unwrap();
    assert_eq!(account.account_id, 100);
    assert_eq!(account.amount, 1000);
    assert_eq!(account.threshold, 100);
    assert_eq!(account.interval, 10);
    assert_eq!(account.height, 0);
}

#[test]
fn test_monitored_account_validation_amount() {
    assert!(MonitoredAccount::new(100, 0, 0, 100, 10).is_err());
    assert!(MonitoredAccount::new(100, 0, 1, 100, 10).is_ok());
}

#[test]
fn test_monitored_account_validation_threshold() {
    assert!(MonitoredAccount::new(100, 0, 1000, 0, 10).is_err());
    assert!(MonitoredAccount::new(100, 0, 1000, 1, 10).is_ok());
}

#[test]
fn test_monitored_account_validation_interval() {
    assert!(MonitoredAccount::new(100, 0, 1000, 100, 0).is_err());
    assert!(MonitoredAccount::new(100, 0, 1000, 100, 10).is_ok());
}

#[test]
fn test_monitored_account_needs_funding() {
    let mut account = MonitoredAccount::new(100, 0, 1000, 100, 10).unwrap();
    account.height = 100;

    assert!(account.needs_funding(50, 110), "Below threshold and past interval");
    assert!(!account.needs_funding(150, 110), "Above threshold");
    assert!(!account.needs_funding(50, 105), "Before interval");
}

#[test]
fn test_monitored_account_needs_funding_exact_boundary() {
    let mut account = MonitoredAccount::new(100, 0, 1000, 100, 10).unwrap();
    account.height = 100;

    assert!(!account.needs_funding(100, 110), "At threshold boundary");
    assert!(account.needs_funding(99, 110), "Just below threshold");
}

#[test]
fn test_monitored_account_set_amount() {
    let mut account = MonitoredAccount::new(100, 0, 1000, 100, 10).unwrap();
    account.set_amount(2000).unwrap();
    assert_eq!(account.amount, 2000);
    assert!(account.set_amount(0).is_err());
}

#[test]
fn test_monitored_account_set_threshold() {
    let mut account = MonitoredAccount::new(100, 0, 1000, 100, 10).unwrap();
    account.set_threshold(200).unwrap();
    assert_eq!(account.threshold, 200);
    assert!(account.set_threshold(0).is_err());
}

#[test]
fn test_monitored_account_set_interval() {
    let mut account = MonitoredAccount::new(100, 0, 1000, 100, 10).unwrap();
    account.set_interval(20).unwrap();
    assert_eq!(account.interval, 20);
    assert!(account.set_interval(5).is_err());
}

#[test]
fn test_monitored_account_set_height() {
    let mut account = MonitoredAccount::new(100, 0, 1000, 100, 10).unwrap();
    account.set_height(500);
    assert_eq!(account.height, 500);
}

#[test]
fn test_monitored_account_from_property_value_json() {
    let account = MonitoredAccount::from_property_value(
        100,
        0,
        r#"{"amount": 500, "threshold": 50, "interval": 15}"#,
        1000,
        100,
        10,
    ).unwrap();

    assert_eq!(account.amount, 500);
    assert_eq!(account.threshold, 50);
    assert_eq!(account.interval, 15);
}

#[test]
fn test_monitored_account_from_property_value_empty() {
    let account = MonitoredAccount::from_property_value(
        100,
        0,
        "",
        1000,
        100,
        10,
    ).unwrap();

    assert_eq!(account.amount, 1000, "Should use defaults");
    assert_eq!(account.threshold, 100);
    assert_eq!(account.interval, 10);
}

#[test]
fn test_monitored_account_from_property_value_partial_json() {
    let account = MonitoredAccount::from_property_value(
        100,
        0,
        r#"{"amount": 500}"#,
        1000,
        100,
        10,
    ).unwrap();

    assert_eq!(account.amount, 500);
    assert_eq!(account.threshold, 100, "Should use default");
    assert_eq!(account.interval, 10, "Should use default");
}

#[test]
fn test_monitored_account_from_property_value_invalid_json() {
    let result = MonitoredAccount::from_property_value(
        100,
        0,
        "not json",
        1000,
        100,
        10,
    );
    assert!(result.is_err());
}

#[tokio::test]
async fn test_funding_monitor_service_start() {
    let service = FundingMonitorService::new();

    let monitor = FundingMonitor::new(
        HoldingType::Nrcs, None, "fund".to_string(), 1000, 100, 10, 12345, vec![1, 2, 3],
    );

    let properties = vec![(100, r#"{"amount": 500}"#.to_string())];
    service.start_monitor(monitor, properties).await.unwrap();

    let all = service.get_all_monitors().await;
    assert_eq!(all.len(), 1);
}

#[tokio::test]
async fn test_funding_monitor_service_duplicate() {
    let service = FundingMonitorService::new();

    let monitor = FundingMonitor::new(
        HoldingType::Nrcs, None, "fund".to_string(), 1000, 100, 10, 12345, vec![1, 2, 3],
    );

    service.start_monitor(monitor.clone(), vec![]).await.unwrap();
    let result = service.start_monitor(monitor, vec![]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_funding_monitor_service_max_exceeded() {
    let service = FundingMonitorService::with_max_monitors(1);

    let m1 = FundingMonitor::new(
        HoldingType::Nrcs, None, "fund1".to_string(), 1000, 100, 10, 1, vec![],
    );
    let m2 = FundingMonitor::new(
        HoldingType::Nrcs, None, "fund2".to_string(), 1000, 100, 10, 2, vec![],
    );

    service.start_monitor(m1, vec![]).await.unwrap();
    let result = service.start_monitor(m2, vec![]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_funding_monitor_service_stop() {
    let service = FundingMonitorService::new();

    let monitor = FundingMonitor::new(
        HoldingType::Nrcs, None, "fund".to_string(), 1000, 100, 10, 12345, vec![],
    );

    service.start_monitor(monitor, vec![]).await.unwrap();
    assert_eq!(service.get_all_monitors().await.len(), 1);

    let stopped = service.stop_monitor(HoldingType::Nrcs, None, "fund", 12345).await;
    assert!(stopped.is_ok());
    assert!(service.get_all_monitors().await.is_empty());
}

#[tokio::test]
async fn test_funding_monitor_service_stop_not_found() {
    let service = FundingMonitorService::new();

    let result = service.stop_monitor(HoldingType::Nrcs, None, "fund", 99999).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_funding_monitor_service_stop_all() {
    let service = FundingMonitorService::new();

    service.start_monitor(
        FundingMonitor::new(HoldingType::Nrcs, None, "fund1".to_string(), 1000, 100, 10, 1, vec![]),
        vec![],
    ).await.unwrap();
    service.start_monitor(
        FundingMonitor::new(HoldingType::Nrcs, None, "fund2".to_string(), 1000, 100, 10, 2, vec![]),
        vec![],
    ).await.unwrap();

    let count = service.stop_all_monitors().await;
    assert_eq!(count, 2);
    assert!(service.get_all_monitors().await.is_empty());
}

#[tokio::test]
async fn test_funding_monitor_service_process_account_event() {
    let service = FundingMonitorService::new();

    let monitor = FundingMonitor::new(
        HoldingType::Nrcs, None, "fund".to_string(), 1000, 100, 10, 12345, vec![1, 2, 3],
    );

    let properties = vec![(100, "".to_string())];
    service.start_monitor(monitor, properties).await.unwrap();

    service.update_account_height(100, 0, 100).await;

    let funding = service.process_account_event(100, 50, 110).await;
    assert_eq!(funding.len(), 1, "Should need funding");
    assert_eq!(funding[0].1, 1000, "Should fund with monitor amount");

    let funding = service.process_account_event(100, 150, 110).await;
    assert!(funding.is_empty(), "Above threshold, no funding needed");
}

#[tokio::test]
async fn test_funding_monitor_service_get_monitored_accounts() {
    let service = FundingMonitorService::new();

    let monitor = FundingMonitor::new(
        HoldingType::Nrcs, None, "fund".to_string(), 1000, 100, 10, 12345, vec![],
    );

    let properties = vec![
        (100, r#"{"amount": 500}"#.to_string()),
        (200, r#"{"amount": 800}"#.to_string()),
    ];
    service.start_monitor(monitor, properties).await.unwrap();

    let monitored = service.get_monitored_accounts(0).await;
    assert_eq!(monitored.len(), 2);
}

#[tokio::test]
async fn test_funding_monitor_service_get_account_monitors() {
    let service = FundingMonitorService::new();

    let monitor = FundingMonitor::new(
        HoldingType::Nrcs, None, "fund".to_string(), 1000, 100, 10, 12345, vec![],
    );

    let properties = vec![(100, "".to_string())];
    service.start_monitor(monitor, properties).await.unwrap();

    let account_monitors = service.get_account_monitors(100).await;
    assert_eq!(account_monitors.len(), 1);

    let account_monitors = service.get_account_monitors(999).await;
    assert!(account_monitors.is_empty());
}
