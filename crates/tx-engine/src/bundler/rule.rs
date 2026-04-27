//! Bundler Rule
//!
//! 对应 Java: BundlerRule.java

use serde::{Deserialize, Serialize};

/// Bundler Rule
///
/// 对应 Java: BundlerRule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundlerRule {
    pub min_rate_nqt_per_fxt: i64,
    pub overpay_fqt_per_fxt: i64,
    pub fee_calculator_name: String,
    pub filter_names: Vec<String>,
}

impl BundlerRule {
    pub fn new(
        min_rate_nqt_per_fxt: i64,
        overpay_fqt_per_fxt: i64,
        fee_calculator_name: String,
    ) -> Self {
        Self {
            min_rate_nqt_per_fxt,
            overpay_fqt_per_fxt,
            fee_calculator_name,
            filter_names: Vec::new(),
        }
    }

    pub fn with_filters(mut self, filter_names: Vec<String>) -> Self {
        self.filter_names = filter_names;
        self
    }

    pub fn calculate_fee(&self, base_fee: i64, transaction_size: usize, calculator: &dyn super::FeeCalculator) -> i64 {
        let fee = calculator.calculate_fee(transaction_size, base_fee);
        self.apply_overpay(fee)
    }

    pub fn apply_overpay(&self, fee: i64) -> i64 {
        if self.overpay_fqt_per_fxt == 0 {
            return fee;
        }
        
        let overpay = (fee as i128 * self.overpay_fqt_per_fxt as i128 / 1_0000_0000) as i64;
        fee.saturating_add(overpay)
    }

    pub fn is_rate_acceptable(&self, transaction_fee: i64, min_fee: i64) -> bool {
        if self.min_rate_nqt_per_fxt == 0 {
            return true;
        }
        
        let min_required_fee = (min_fee as i128 * self.min_rate_nqt_per_fxt as i128 / 1_0000_0000) as i64;
        transaction_fee >= min_required_fee
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::fee_calculator::MinFeeCalculator;

    #[test]
    fn test_bundler_rule_new() {
        let rule = BundlerRule::new(100, 10, "MIN_FEE".to_string());
        assert_eq!(rule.min_rate_nqt_per_fxt, 100);
        assert_eq!(rule.overpay_fqt_per_fxt, 10);
    }

    #[test]
    fn test_bundler_rule_with_filters() {
        let rule = BundlerRule::new(100, 10, "MIN_FEE".to_string())
            .with_filters(vec!["ACCOUNT".to_string(), "TX_TYPE".to_string()]);
        
        assert_eq!(rule.filter_names.len(), 2);
    }

    #[test]
    fn test_bundler_rule_calculate_fee() {
        let rule = BundlerRule::new(100, 0, "MIN_FEE".to_string());
        let calculator = MinFeeCalculator::new();
        
        let fee = rule.calculate_fee(1000, 100, &calculator);
        assert_eq!(fee, 1000);
    }

    #[test]
    fn test_bundler_rule_apply_overpay() {
        let rule = BundlerRule::new(100, 100_000_000, "MIN_FEE".to_string());
        
        let fee = rule.apply_overpay(1000);
        assert_eq!(fee, 2000);
        
        let rule_no_overpay = BundlerRule::new(100, 0, "MIN_FEE".to_string());
        let fee = rule_no_overpay.apply_overpay(1000);
        assert_eq!(fee, 1000);
    }

    #[test]
    fn test_bundler_rule_is_rate_acceptable() {
        let rule = BundlerRule::new(100_000_000, 0, "MIN_FEE".to_string());
        
        assert!(rule.is_rate_acceptable(2000, 1000));
        assert!(!rule.is_rate_acceptable(500, 1000));
        
        let rule_no_min = BundlerRule::new(0, 0, "MIN_FEE".to_string());
        assert!(rule_no_min.is_rate_acceptable(100, 1000));
    }
}
