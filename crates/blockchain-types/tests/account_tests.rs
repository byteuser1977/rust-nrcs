use blockchain_types::*;

#[test]
fn test_account_creation() {
    let account = Account::new(1234567890, 1_000_000_000); // 10 NRC
    assert_eq!(account.id, 1234567890);
    assert_eq!(account.balance, 1_000_000_000);
    assert_eq!(account.unconfirmed_balance, 1_000_000_000);
    assert_eq!(account.reserved_balance, 0);
    assert!(account.address.is_none());
    assert!(account.assets.is_empty());
    assert!(account.lease.is_none());
}

#[test]
fn test_account_balance_credit() {
    let mut account = Account::new(1234567890, 1000);
    account.credit(500);
    assert_eq!(account.balance, 1500);
    assert_eq!(account.unconfirmed_balance, 1500);
}

#[test]
fn test_account_balance_debit_success() {
    let mut account = Account::new(1234567890, 1000);
    assert!(account.debit(300).is_ok());
    assert_eq!(account.balance, 700);
    assert_eq!(account.unconfirmed_balance, 700);
}

#[test]
fn test_account_balance_debit_insufficient() {
    let mut account = Account::new(1234567890, 1000);
    let result = account.debit(1500);
    assert!(result.is_err());
    if let Err(BlockchainError::InsufficientBalance { have, need }) = result {
        assert_eq!(have, 1000);
        assert_eq!(need, 1500);
    } else {
        panic!("Expected InsufficientBalance error");
    }
}

#[test]
fn test_account_effective_balance_with_reserved() {
    let mut account = Account::new(1234567890, 1000);
    account.reserved_balance = 300;
    assert_eq!(account.effective_balance(), 700);
}

#[test]
fn test_account_effective_balance_no_reserve() {
    let account = Account::new(1234567890, 1000);
    assert_eq!(account.effective_balance(), 1000);
}

#[test]
fn test_account_has_balance() {
    let account = Account::new(1234567890, 1000);
    assert!(account.has_balance(500));
    assert!(account.has_balance(1000));
    assert!(!account.has_balance(1001));
}

#[test]
fn test_account_assets_add() {
    let mut account = Account::new(1234567890, 0);
    account.add_asset(100, 1000);
    assert_eq!(account.asset_quantity(100), 1000);
    assert_eq!(account.assets.len(), 1);
}

#[test]
fn test_account_assets_add_multiple() {
    let mut account = Account::new(1234567890, 0);
    account.add_asset(100, 1000);
    account.add_asset(100, 500);
    assert_eq!(account.asset_quantity(100), 1500);
    assert_eq!(account.assets.len(), 1);
}

#[test]
fn test_account_assets_remove() {
    let mut account = Account::new(1234567890, 0);
    account.add_asset(100, 1000);
    assert!(account.remove_asset(100, 800).is_ok());
    assert_eq!(account.asset_quantity(100), 200);
    assert_eq!(account.assets.len(), 1);
}

#[test]
fn test_account_assets_remove_all() {
    let mut account = Account::new(1234567890, 0);
    account.add_asset(100, 1000);
    assert!(account.remove_asset(100, 1000).is_ok());
    assert_eq!(account.asset_quantity(100), 0);
    assert!(account.assets.is_empty());
}

#[test]
fn test_account_assets_remove_insufficient() {
    let mut account = Account::new(1234567890, 0);
    account.add_asset(100, 1000);
    let result = account.remove_asset(100, 1500);
    assert!(result.is_err());
}

#[test]
fn test_account_assets_multiple_different_assets() {
    let mut account = Account::new(1234567890, 0);
    account.add_asset(100, 1000);
    account.add_asset(200, 2000);
    account.add_asset(100, 500);
    account.add_asset(300, 5000);

    assert_eq!(account.asset_quantity(100), 1500);
    assert_eq!(account.asset_quantity(200), 2000);
    assert_eq!(account.asset_quantity(300), 5000);
    assert_eq!(account.assets.len(), 3);
}

#[test]
fn test_account_lease_creation() {
    let lease = AccountLease::new(9876543210, 100_000, 100, 1440); // 1 day
    assert_eq!(lease.lessee_id, 9876543210);
    assert_eq!(lease.amount, 100_000);
    assert_eq!(lease.start_height, 100);
    assert_eq!(lease.end_height, 1540);
}

#[test]
fn test_account_lease_is_active() {
    let lease = AccountLease::new(9876543210, 100_000, 100, 1440);

    assert!(lease.is_active(100));
    assert!(lease.is_active(1000));
    assert!(lease.is_active(1540)); // end inclusive
    assert!(!lease.is_active(99));
    assert!(!lease.is_active(1541));
}

#[test]
fn test_account_forging_weight_no_lease() {
    let account = Account::new(1234567890, 1000);
    let weight = account.forging_weight(100);
    assert_eq!(weight, 1000);
}

#[test]
fn test_account_forging_weight_with_lease_outside_period() {
    let mut account = Account::new(1234567890, 1000);
    account.lease = Some(AccountLease::new(9876543210, 500, 200, 1440));

    // Before lease starts
    let weight = account.forging_weight(100);
    assert_eq!(weight, 1000); // full balance counts

    // After lease ends
    let weight = account.forging_weight(2000);
    assert_eq!(weight, 1000);
}

#[test]
fn test_account_forging_weight_with_lease_active() {
    let mut account = Account::new(1234567890, 1000);
    account.lease = Some(AccountLease::new(9876543210, 500, 100, 1440));

    // During lease period
    let weight = account.forging_weight(500);
    assert_eq!(weight, 500); // (1000 - 500) = 500
}

#[test]
fn test_account_set_and_get_address() {
    let mut account = Account::new(1234567890, 0);
    assert!(account.get_address().is_none());

    account.set_address("NRCS-abc123-def456".to_string());
    assert_eq!(account.get_address(), Some("NRCS-abc123-def456"));
}

#[test]
fn test_account_properties() {
    let mut account = Account::new(1234567890, 0);
    assert!(account.properties.is_empty());

    account.properties.insert("name".to_string(), "Alice".to_string());
    account.properties.insert("email".to_string(), "alice@example.com".to_string());

    assert_eq!(account.properties.len(), 2);
    assert_eq!(account.properties.get("name"), Some(&"Alice".to_string()));
}

#[test]
fn test_account_default_fields() {
    let account = Account::new(1234567890, 0);
    assert_eq!(account.created_at, account.last_updated);
    assert_eq!(account.current_height, 0);
    assert_eq!(account.assets.len(), 0);
    assert_eq!(account.properties.len(), 0);
    assert!(account.lease.is_none());
}

#[test]
fn test_account_balance_saturation() {
    let mut account = Account::new(1234567890, u64::MAX - 100);
    // Credit should not overflow
    account.credit(50);
    assert_eq!(account.balance, u64::MAX - 50);

    // Debit to zero should work
    assert!(account.debit(u64::MAX - 50).is_ok());
    assert_eq!(account.balance, 0);
}

#[test]
fn test_account_asset_overflow() {
    let mut account = Account::new(1234567890, 0);
    account.add_asset(100, u64::MAX - 100);
    account.add_asset(100, 50);
    assert_eq!(account.asset_quantity(100), u64::MAX - 50);
}

#[test]
fn test_account_clone_independence() {
    let account1 = Account::new(1234567890, 1000);
    let mut account2 = account1.clone();

    account2.credit(500);
    assert_eq!(account2.balance, 1500);
    assert_eq!(account1.balance, 1000); // original unchanged
}
