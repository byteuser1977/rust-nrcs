//! Ledger 外部测试
//!
//! 测试 AccountLedger、LedgerEntry、LedgerConfig 等核心账本功能

use tx_engine::ledger::{
    AccountLedger, LedgerConfig, LedgerEntry,
};
use tx_engine::ledger::types::{LedgerEvent, LedgerHolding};
use std::collections::HashSet;
use std::sync::Arc;

fn make_config() -> LedgerConfig {
    LedgerConfig {
        enabled: true,
        track_all_accounts: true,
        track_accounts: HashSet::new(),
        log_unconfirmed: 1,
        trim_keep: 0,
    }
}

fn make_entry(event: LedgerEvent, account_id: i64, change: i64, balance: i64) -> LedgerEntry {
    LedgerEntry::new(
        event,
        12345,
        account_id,
        Some(LedgerHolding::NrcsBalance),
        None,
        change,
        balance,
        1,
        100,
        1234567890,
    )
}

#[test]
fn test_ledger_entry_creation() {
    let entry = make_entry(LedgerEvent::OrdinaryPayment, 100, 1000, 5000);
    assert_eq!(entry.account_id, 100);
    assert_eq!(entry.event, LedgerEvent::OrdinaryPayment);
    assert_eq!(entry.change, 1000);
    assert_eq!(entry.balance, 5000);
    assert_eq!(entry.holding, Some(LedgerHolding::NrcsBalance));
}

#[test]
fn test_ledger_entry_update_change() {
    let mut entry = make_entry(LedgerEvent::OrdinaryPayment, 100, 1000, 5000);
    entry.update_change(500);
    assert_eq!(entry.change, 1500);
    entry.update_change(-300);
    assert_eq!(entry.change, 1200);
}

#[test]
fn test_ledger_entry_to_model_roundtrip() {
    let entry = make_entry(LedgerEvent::OrdinaryPayment, 100, 1000, 5000);
    let model = entry.to_model();
    assert_eq!(model.account_id, 100);
    assert_eq!(model.event_type, LedgerEvent::OrdinaryPayment.code());
    assert_eq!(model.change, 1000);
    assert_eq!(model.balance, 5000);
    assert_eq!(model.holding_type, LedgerHolding::NrcsBalance.code());

    let restored = LedgerEntry::from_model(&model);
    assert!(restored.is_some());
    let restored = restored.unwrap();
    assert_eq!(restored.account_id, entry.account_id);
    assert_eq!(restored.event, entry.event);
    assert_eq!(restored.change, entry.change);
    assert_eq!(restored.balance, entry.balance);
}

#[test]
fn test_ledger_entry_from_model_invalid_event() {
    let mut model = make_entry(LedgerEvent::OrdinaryPayment, 100, 1000, 5000).to_model();
    model.event_type = 9999;
    let result = LedgerEntry::from_model(&model);
    assert!(result.is_none(), "Should return None for invalid event type");
}

#[test]
fn test_ledger_config_default() {
    let config = LedgerConfig::default();
    assert!(!config.enabled);
    assert!(!config.track_all_accounts);
    assert!(config.track_accounts.is_empty());
    assert_eq!(config.log_unconfirmed, 1);
    assert_eq!(config.trim_keep, 30000);
}

#[test]
fn test_ledger_entry_with_none_holding() {
    let entry = LedgerEntry::new(
        LedgerEvent::OrdinaryPayment,
        12345,
        100,
        None,
        None,
        1000,
        5000,
        1,
        100,
        1234567890,
    );
    assert!(entry.holding.is_none());
    let model = entry.to_model();
    assert_eq!(model.holding_type, -1);
}

#[test]
fn test_ledger_entry_ledger_id() {
    let entry = make_entry(LedgerEvent::OrdinaryPayment, 100, 1000, 5000);
    assert_eq!(entry.ledger_id(), 0);
}

#[test]
fn test_ledger_entry_hash_and_eq() {
    let e1 = make_entry(LedgerEvent::OrdinaryPayment, 100, 1000, 5000);
    let e2 = make_entry(LedgerEvent::OrdinaryPayment, 100, 1000, 5000);
    let e3 = make_entry(LedgerEvent::AssetIssuance, 100, 1000, 5000);

    assert_eq!(e1, e2);
    assert_ne!(e1, e3);

    let mut set = HashSet::new();
    set.insert(e1.clone());
    assert!(set.contains(&e2));
}

#[test]
fn test_ledger_event_variants() {
    assert_eq!(LedgerEvent::from_code(1), Some(LedgerEvent::BlockGenerated));
    assert_eq!(LedgerEvent::from_code(3), Some(LedgerEvent::OrdinaryPayment));
    assert_eq!(LedgerEvent::from_code(9), Some(LedgerEvent::ArbitraryMessage));
    assert_eq!(LedgerEvent::from_code(19), Some(LedgerEvent::AssetIssuance));
    assert_eq!(LedgerEvent::from_code(38), Some(LedgerEvent::CurrencyIssuance));
    assert_eq!(LedgerEvent::from_code(999), None);
}

#[test]
fn test_ledger_holding_variants() {
    assert_eq!(LedgerHolding::from_code(1), Some(LedgerHolding::UnconfirmedNrcsBalance));
    assert_eq!(LedgerHolding::from_code(2), Some(LedgerHolding::NrcsBalance));
    assert_eq!(LedgerHolding::from_code(3), Some(LedgerHolding::UnconfirmedAssetBalance));
    assert_eq!(LedgerHolding::from_code(4), Some(LedgerHolding::AssetBalance));
    assert_eq!(LedgerHolding::from_code(5), Some(LedgerHolding::UnconfirmedCurrencyBalance));
    assert_eq!(LedgerHolding::from_code(6), Some(LedgerHolding::CurrencyBalance));
    assert_eq!(LedgerHolding::from_code(999), None);
}

#[test]
fn test_ledger_holding_is_unconfirmed() {
    assert!(LedgerHolding::UnconfirmedNrcsBalance.is_unconfirmed());
    assert!(LedgerHolding::UnconfirmedAssetBalance.is_unconfirmed());
    assert!(LedgerHolding::UnconfirmedCurrencyBalance.is_unconfirmed());
    assert!(!LedgerHolding::NrcsBalance.is_unconfirmed());
    assert!(!LedgerHolding::AssetBalance.is_unconfirmed());
    assert!(!LedgerHolding::CurrencyBalance.is_unconfirmed());
}

#[test]
fn test_ledger_event_is_transaction() {
    assert!(!LedgerEvent::BlockGenerated.is_transaction());
    assert!(LedgerEvent::OrdinaryPayment.is_transaction());
    assert!(LedgerEvent::AssetIssuance.is_transaction());
}

#[test]
fn test_ledger_event_code_roundtrip() {
    for event in [
        LedgerEvent::BlockGenerated,
        LedgerEvent::OrdinaryPayment,
        LedgerEvent::ArbitraryMessage,
        LedgerEvent::AssetIssuance,
        LedgerEvent::CurrencyIssuance,
        LedgerEvent::ShufflingRegistration,
        LedgerEvent::CoinExchangeOrderIssue,
    ] {
        let code = event.code();
        assert_eq!(LedgerEvent::from_code(code), Some(event));
    }
}

#[test]
fn test_ledger_holding_code_roundtrip() {
    for holding in [
        LedgerHolding::UnconfirmedNrcsBalance,
        LedgerHolding::NrcsBalance,
        LedgerHolding::UnconfirmedAssetBalance,
        LedgerHolding::AssetBalance,
        LedgerHolding::UnconfirmedCurrencyBalance,
        LedgerHolding::CurrencyBalance,
    ] {
        let code = holding.code();
        assert_eq!(LedgerHolding::from_code(code), Some(holding));
    }
}

#[tokio::test]
async fn test_ledger_must_log_disabled() {
    let config = LedgerConfig {
        enabled: false,
        ..make_config()
    };
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let ledger = AccountLedger::new(config, Arc::new(orm::repository::SqliteAccountLedgerRepository::new(pool)));

    assert!(!ledger.must_log_entry(100, false));
    assert!(!ledger.must_log_entry(100, true));
}

#[tokio::test]
async fn test_ledger_must_log_not_tracking_account() {
    let mut accounts = HashSet::new();
    accounts.insert(200);
    let config = LedgerConfig {
        enabled: true,
        track_all_accounts: false,
        track_accounts: accounts,
        log_unconfirmed: 1,
        trim_keep: 0,
    };
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let ledger = AccountLedger::new(config, Arc::new(orm::repository::SqliteAccountLedgerRepository::new(pool)));
    ledger.set_processing_block(true);

    assert!(!ledger.must_log_entry(100, false), "Account 100 not in track list");
    assert!(ledger.must_log_entry(200, false), "Account 200 in track list");
}

#[tokio::test]
async fn test_ledger_must_log_unconfirmed_setting() {
    let config_no_unconfirmed = LedgerConfig {
        enabled: true,
        track_all_accounts: true,
        track_accounts: HashSet::new(),
        log_unconfirmed: 0,
        trim_keep: 0,
    };
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let ledger = AccountLedger::new(config_no_unconfirmed, Arc::new(orm::repository::SqliteAccountLedgerRepository::new(pool)));
    ledger.set_processing_block(true);

    assert!(!ledger.must_log_entry(100, true), "Unconfirmed logging disabled");
    assert!(ledger.must_log_entry(100, false), "Confirmed should still work");
}

#[tokio::test]
async fn test_ledger_must_log_not_processing_block() {
    let config = make_config();
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let ledger = AccountLedger::new(config, Arc::new(orm::repository::SqliteAccountLedgerRepository::new(pool)));

    assert!(!ledger.must_log_entry(100, false), "Not processing block");
    ledger.set_processing_block(true);
    assert!(ledger.must_log_entry(100, false), "Processing block");
}

#[tokio::test]
async fn test_ledger_log_entry() {
    let config = make_config();
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let ledger = AccountLedger::new(config, Arc::new(orm::repository::SqliteAccountLedgerRepository::new(pool)));

    let entry = make_entry(LedgerEvent::OrdinaryPayment, 100, 1000, 5000);
    ledger.log_entry(entry);
    assert_eq!(ledger.get_pending_count(), 1);
}

#[tokio::test]
async fn test_ledger_log_entry_merge() {
    let config = make_config();
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let ledger = AccountLedger::new(config, Arc::new(orm::repository::SqliteAccountLedgerRepository::new(pool)));

    let entry1 = make_entry(LedgerEvent::OrdinaryPayment, 100, 1000, 5000);
    ledger.log_entry(entry1);
    let entry2 = make_entry(LedgerEvent::OrdinaryPayment, 100, 500, 5500);
    ledger.log_entry(entry2);

    assert_eq!(ledger.get_pending_count(), 1, "Same event should merge");
}

#[tokio::test]
async fn test_ledger_log_different_events() {
    let config = make_config();
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let ledger = AccountLedger::new(config, Arc::new(orm::repository::SqliteAccountLedgerRepository::new(pool)));

    let entry1 = make_entry(LedgerEvent::OrdinaryPayment, 100, 1000, 5000);
    let entry2 = make_entry(LedgerEvent::AssetIssuance, 100, 500, 5500);
    ledger.log_entry(entry1);
    ledger.log_entry(entry2);

    assert_eq!(ledger.get_pending_count(), 2, "Different events should not merge");
}

#[tokio::test]
async fn test_ledger_clear_entries() {
    let config = make_config();
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let ledger = AccountLedger::new(config, Arc::new(orm::repository::SqliteAccountLedgerRepository::new(pool)));

    let entry = make_entry(LedgerEvent::OrdinaryPayment, 100, 1000, 5000);
    ledger.log_entry(entry);
    assert_eq!(ledger.get_pending_count(), 1);

    ledger.clear_entries();
    assert_eq!(ledger.get_pending_count(), 0);
}

#[tokio::test]
async fn test_ledger_is_enabled() {
    let config = make_config();
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let ledger = AccountLedger::new(config, Arc::new(orm::repository::SqliteAccountLedgerRepository::new(pool)));
    assert!(ledger.is_enabled());

    let config_disabled = LedgerConfig {
        enabled: false,
        ..make_config()
    };
    let pool2 = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let ledger_disabled = AccountLedger::new(config_disabled, Arc::new(orm::repository::SqliteAccountLedgerRepository::new(pool2)));
    assert!(!ledger_disabled.is_enabled());
}

#[tokio::test]
async fn test_ledger_log_different_accounts() {
    let config = make_config();
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let ledger = AccountLedger::new(config, Arc::new(orm::repository::SqliteAccountLedgerRepository::new(pool)));

    let entry1 = make_entry(LedgerEvent::OrdinaryPayment, 100, 1000, 5000);
    let entry2 = make_entry(LedgerEvent::OrdinaryPayment, 200, 2000, 8000);
    ledger.log_entry(entry1);
    ledger.log_entry(entry2);

    assert_eq!(ledger.get_pending_count(), 2, "Different accounts should not merge");
}

#[tokio::test]
async fn test_ledger_log_different_holdings() {
    let config = make_config();
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let ledger = AccountLedger::new(config, Arc::new(orm::repository::SqliteAccountLedgerRepository::new(pool)));

    let entry1 = LedgerEntry::new(
        LedgerEvent::OrdinaryPayment, 12345, 100,
        Some(LedgerHolding::NrcsBalance), None,
        1000, 5000, 1, 100, 1234567890,
    );
    let entry2 = LedgerEntry::new(
        LedgerEvent::OrdinaryPayment, 12345, 100,
        Some(LedgerHolding::AssetBalance), Some(999),
        500, 3000, 1, 100, 1234567890,
    );
    ledger.log_entry(entry1);
    ledger.log_entry(entry2);

    assert_eq!(ledger.get_pending_count(), 2, "Different holdings should not merge");
}

#[tokio::test]
async fn test_ledger_set_current_height() {
    let config = make_config();
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let ledger = AccountLedger::new(config, Arc::new(orm::repository::SqliteAccountLedgerRepository::new(pool)));

    ledger.set_current_height(500);
    ledger.set_processing_block(true);
    assert!(ledger.must_log_entry(100, false));
}

#[tokio::test]
async fn test_ledger_must_log_trim_keep() {
    let config = LedgerConfig {
        enabled: true,
        track_all_accounts: true,
        track_accounts: HashSet::new(),
        log_unconfirmed: 1,
        trim_keep: 100,
    };
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let ledger = AccountLedger::new(config, Arc::new(orm::repository::SqliteAccountLedgerRepository::new(pool)));
    ledger.set_processing_block(true);

    ledger.set_current_height(50);
    assert!(!ledger.must_log_entry(100, false), "Height below trim_keep");

    ledger.set_current_height(200);
    assert!(ledger.must_log_entry(100, false), "Height above trim_keep");
}
