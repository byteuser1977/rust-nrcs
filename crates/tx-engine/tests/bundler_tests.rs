//! Bundler 外部测试
//!
//! 测试 Bundler、BundlerRule、FeeCalculator、BundlerFilter 等打包器功能

use tx_engine::bundler::{
    Bundler, BundlerError, BundlerRate, BundlerRule,
    FeeCalculator, MinFeeCalculator, ProportionalFeeCalculator,
    BundlerFilter,
};
use tx_engine::bundler::filter::{DefaultFilter, AccountFilter, TransactionTypeFilter};

fn make_bundler(account_id: u64, total_fees_limit: i64, rules: Vec<BundlerRule>) -> Bundler {
    Bundler::new(account_id, vec![1, 2, 3], total_fees_limit, rules)
}

fn make_rule(min_rate: i64, overpay: i64, calculator_name: &str) -> BundlerRule {
    BundlerRule::new(min_rate, overpay, calculator_name.to_string())
}

#[test]
fn test_bundler_new() {
    let rules = vec![make_rule(100, 0, "MIN_FEE")];
    let bundler = make_bundler(12345, 10000, rules);

    assert_eq!(bundler.account_id, 12345);
    assert_eq!(bundler.total_fees_limit, 10000);
    assert_eq!(bundler.current_total_fees, 0);
    assert_eq!(bundler.bundling_rules.len(), 1);
}

#[test]
fn test_bundler_get_bundler_rate() {
    let rules = vec![
        make_rule(100, 0, "MIN_FEE"),
        make_rule(200, 0, "MIN_FEE"),
    ];
    let bundler = make_bundler(12345, 10000, rules);

    let rate = bundler.get_bundler_rate().unwrap();
    assert_eq!(rate.min_rate_nqt_per_fxt, 100, "Should return minimum rate");
    assert_eq!(rate.remaining_fee_limit, 10000);
    assert_eq!(rate.account_id, 12345);
}

#[test]
fn test_bundler_get_bundler_rate_with_filters() {
    let rules = vec![
        make_rule(100, 0, "MIN_FEE").with_filters(vec!["ACCOUNT".to_string()]),
        make_rule(200, 0, "MIN_FEE"),
    ];
    let bundler = make_bundler(12345, 10000, rules);

    let rate = bundler.get_bundler_rate().unwrap();
    assert_eq!(rate.min_rate_nqt_per_fxt, 200, "Should skip filtered rules");
}

#[test]
fn test_bundler_get_bundler_rate_no_public_rules() {
    let rules = vec![
        make_rule(100, 0, "MIN_FEE").with_filters(vec!["ACCOUNT".to_string()]),
    ];
    let bundler = make_bundler(12345, 10000, rules);

    assert!(bundler.get_bundler_rate().is_none(), "No public rules");
}

#[test]
fn test_bundler_can_bundle_unlimited() {
    let bundler = make_bundler(12345, 0, vec![]);
    assert!(bundler.can_bundle(999999));
}

#[test]
fn test_bundler_can_bundle_within_limit() {
    let bundler = make_bundler(12345, 10000, vec![]);
    assert!(bundler.can_bundle(5000));
    assert!(bundler.can_bundle(10000));
    assert!(!bundler.can_bundle(10001));
}

#[test]
fn test_bundler_add_fees() {
    let mut bundler = make_bundler(12345, 10000, vec![]);
    bundler.add_fees(3000);
    assert_eq!(bundler.current_total_fees, 3000);
    assert!(bundler.can_bundle(7000));
    assert!(!bundler.can_bundle(7001));

    bundler.add_fees(3000);
    assert_eq!(bundler.current_total_fees, 6000);
}

#[test]
fn test_bundler_add_fees_saturating() {
    let mut bundler = make_bundler(12345, 10000, vec![]);
    bundler.add_fees(i64::MAX);
    assert_eq!(bundler.current_total_fees, i64::MAX);
}

#[test]
fn test_bundler_add_rule() {
    let mut bundler = make_bundler(12345, 10000, vec![]);
    assert_eq!(bundler.bundling_rules.len(), 0);

    bundler.add_rule(make_rule(100, 0, "MIN_FEE"));
    assert_eq!(bundler.bundling_rules.len(), 1);
}

#[test]
fn test_bundler_rate_new() {
    let rate = BundlerRate::new(100, 5000, 12345);
    assert_eq!(rate.min_rate_nqt_per_fxt, 100);
    assert_eq!(rate.remaining_fee_limit, 5000);
    assert_eq!(rate.account_id, 12345);
}

#[test]
fn test_bundler_rule_new() {
    let rule = make_rule(100, 10, "MIN_FEE");
    assert_eq!(rule.min_rate_nqt_per_fxt, 100);
    assert_eq!(rule.overpay_fqt_per_fxt, 10);
    assert_eq!(rule.fee_calculator_name, "MIN_FEE");
    assert!(rule.filter_names.is_empty());
}

#[test]
fn test_bundler_rule_with_filters() {
    let rule = make_rule(100, 10, "MIN_FEE")
        .with_filters(vec!["ACCOUNT".to_string(), "TX_TYPE".to_string()]);
    assert_eq!(rule.filter_names.len(), 2);
    assert_eq!(rule.filter_names[0], "ACCOUNT");
    assert_eq!(rule.filter_names[1], "TX_TYPE");
}

#[test]
fn test_bundler_rule_calculate_fee_min() {
    let rule = make_rule(100, 0, "MIN_FEE");
    let calculator = MinFeeCalculator::new();
    let fee = rule.calculate_fee(1000, 100, &calculator);
    assert_eq!(fee, 1000);
}

#[test]
fn test_bundler_rule_calculate_fee_proportional() {
    let rule = make_rule(100, 0, "PROPORTIONAL_FEE");
    let calculator = ProportionalFeeCalculator::new(10);
    let fee = rule.calculate_fee(500, 100, &calculator);
    assert_eq!(fee, 1000, "10*100=1000 > base 500");
}

#[test]
fn test_bundler_rule_apply_overpay_none() {
    let rule = make_rule(100, 0, "MIN_FEE");
    assert_eq!(rule.apply_overpay(1000), 1000);
}

#[test]
fn test_bundler_rule_apply_overpay_100percent() {
    let rule = BundlerRule::new(100, 100_000_000, "MIN_FEE".to_string());
    assert_eq!(rule.apply_overpay(1000), 2000);
}

#[test]
fn test_bundler_rule_apply_overpay_50percent() {
    let rule = BundlerRule::new(100, 50_000_000, "MIN_FEE".to_string());
    let fee = rule.apply_overpay(1000);
    assert_eq!(fee, 1500);
}

#[test]
fn test_bundler_rule_is_rate_acceptable() {
    let rule = BundlerRule::new(100_000_000, 0, "MIN_FEE".to_string());
    assert!(rule.is_rate_acceptable(2000, 1000), "200% rate is acceptable");
    assert!(rule.is_rate_acceptable(1000, 1000), "100% rate is acceptable");
    assert!(!rule.is_rate_acceptable(500, 1000), "50% rate is not acceptable");
}

#[test]
fn test_bundler_rule_is_rate_acceptable_no_min() {
    let rule = make_rule(0, 0, "MIN_FEE");
    assert!(rule.is_rate_acceptable(1, 1000), "No min rate means always acceptable");
}

#[test]
fn test_min_fee_calculator() {
    let calc = MinFeeCalculator::new();
    assert_eq!(calc.name(), "MIN_FEE");
    assert_eq!(calc.calculate_fee(100, 1000), 1000);
    assert_eq!(calc.calculate_fee(0, 500), 500);
}

#[test]
fn test_proportional_fee_calculator() {
    let calc = ProportionalFeeCalculator::new(10);
    assert_eq!(calc.name(), "PROPORTIONAL_FEE");
    assert_eq!(calc.calculate_fee(100, 500), 1000, "10*100=1000 > 500");
    assert_eq!(calc.calculate_fee(50, 2000), 2000, "10*50=500 < 2000");
}

#[test]
fn test_proportional_fee_default() {
    let calc = ProportionalFeeCalculator::default();
    assert_eq!(calc.fee_per_byte, 100);
}

#[test]
fn test_default_filter() {
    let filter = DefaultFilter::new();
    assert_eq!(filter.name(), "DEFAULT");
    assert!(filter.check(100, &[1, 2, 3]));
    assert!(filter.check(0, &[]));
}

#[test]
fn test_default_filter_parameter() {
    let mut filter = DefaultFilter::new();
    filter.set_parameter("test_param");
    assert_eq!(filter.get_parameter(), "test_param");
}

#[test]
fn test_account_filter_with_accounts() {
    let filter = AccountFilter::with_accounts(vec![100, 200]);
    assert_eq!(filter.name(), "ACCOUNT");

    let mut tx_data = vec![0u8; 100];
    tx_data[0..8].copy_from_slice(&100u64.to_le_bytes());
    assert!(filter.check(300, &tx_data));

    tx_data[0..8].copy_from_slice(&300u64.to_le_bytes());
    assert!(!filter.check(300, &tx_data));
}

#[test]
fn test_account_filter_empty_allows_all() {
    let filter = AccountFilter::new();
    assert!(filter.check(100, &[0u8; 100]));
}

#[test]
fn test_account_filter_short_data() {
    let filter = AccountFilter::with_accounts(vec![100]);
    assert!(!filter.check(100, &[1, 2, 3]), "Too short data");
}

#[test]
fn test_account_filter_set_parameter() {
    let mut filter = AccountFilter::new();
    filter.set_parameter("100,200,300");
    let mut tx_data = vec![0u8; 100];
    tx_data[0..8].copy_from_slice(&100u64.to_le_bytes());
    assert!(filter.check(300, &tx_data));
    tx_data[0..8].copy_from_slice(&200u64.to_le_bytes());
    assert!(filter.check(300, &tx_data));
    tx_data[0..8].copy_from_slice(&300u64.to_le_bytes());
    assert!(filter.check(300, &tx_data));
    tx_data[0..8].copy_from_slice(&400u64.to_le_bytes());
    assert!(!filter.check(300, &tx_data));
}

#[test]
fn test_transaction_type_filter_with_types() {
    let filter = TransactionTypeFilter::with_types(vec![0, 1, 2]);
    assert_eq!(filter.name(), "TX_TYPE");

    let mut tx_data = vec![0u8; 100];
    tx_data[16] = 0;
    assert!(filter.check(100, &tx_data));

    tx_data[16] = 5;
    assert!(!filter.check(100, &tx_data));
}

#[test]
fn test_transaction_type_filter_empty_allows_all() {
    let filter = TransactionTypeFilter::new();
    assert!(filter.check(100, &[0u8; 100]));
}

#[test]
fn test_transaction_type_filter_short_data() {
    let filter = TransactionTypeFilter::with_types(vec![0]);
    assert!(!filter.check(100, &[0u8; 10]), "Too short data");
}

#[test]
fn test_transaction_type_filter_set_parameter() {
    let mut filter = TransactionTypeFilter::new();
    filter.set_parameter("0,1,2,3");
    let mut tx_data = vec![0u8; 100];
    for t in [0u8, 1, 2, 3] {
        tx_data[16] = t;
        assert!(filter.check(100, &tx_data));
    }
    tx_data[16] = 4;
    assert!(!filter.check(100, &tx_data));
}

#[tokio::test]
async fn test_bundler_service() {
    use tx_engine::bundler::service::BundlerService;

    let service = BundlerService::new();

    let bundler = make_bundler(12345, 10000, vec![make_rule(100, 0, "MIN_FEE")]);
    service.add_bundler(bundler).await;

    let retrieved = service.get_bundler(12345).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().account_id, 12345);

    let all = service.get_all_bundlers().await;
    assert_eq!(all.len(), 1);

    let rates = service.get_bundler_rates().await;
    assert_eq!(rates.len(), 1);
    assert_eq!(rates[0].min_rate_nqt_per_fxt, 100);
}

#[tokio::test]
async fn test_bundler_service_stop() {
    use tx_engine::bundler::service::BundlerService;

    let service = BundlerService::new();

    let bundler = make_bundler(12345, 10000, vec![]);
    service.add_bundler(bundler).await;

    let stopped = service.stop_bundler(12345).await;
    assert!(stopped.is_some());
    assert_eq!(stopped.unwrap().account_id, 12345);

    let all = service.get_all_bundlers().await;
    assert!(all.is_empty());
}

#[tokio::test]
async fn test_bundler_service_stop_all() {
    use tx_engine::bundler::service::BundlerService;

    let service = BundlerService::new();

    service.add_bundler(make_bundler(1, 10000, vec![])).await;
    service.add_bundler(make_bundler(2, 20000, vec![])).await;

    assert_eq!(service.get_all_bundlers().await.len(), 2);

    service.stop_all_bundlers().await;
    assert!(service.get_all_bundlers().await.is_empty());
}

#[tokio::test]
async fn test_bundler_service_get_nonexistent() {
    use tx_engine::bundler::service::BundlerService;

    let service = BundlerService::new();
    let result = service.get_bundler(99999).await;
    assert!(result.is_none());
}
