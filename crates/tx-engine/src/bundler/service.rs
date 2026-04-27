//! Bundler
//!
//! 对应 Java: Bundler.java

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use blockchain_types::AccountId;
use super::rule::BundlerRule;

/// Bundler Error
#[derive(Debug, Error)]
pub enum BundlerError {
    #[error("insufficient balance: {0}")]
    InsufficientBalance(String),
    
    #[error("fee limit exceeded: {0}")]
    FeeLimitExceeded(String),
    
    #[error("validation error: {0}")]
    Validation(String),
    
    #[error("unknown fee calculator: {0}")]
    UnknownFeeCalculator(String),
    
    #[error("unknown filter: {0}")]
    UnknownFilter(String),
}

pub type BundlerResult<T> = Result<T, BundlerError>;

/// Bundler Rate
///
/// 对应 Java: BundlerRate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundlerRate {
    pub min_rate_nqt_per_fxt: i64,
    pub remaining_fee_limit: i64,
    pub account_id: AccountId,
}

impl BundlerRate {
    pub fn new(min_rate_nqt_per_fxt: i64, remaining_fee_limit: i64, account_id: AccountId) -> Self {
        Self {
            min_rate_nqt_per_fxt,
            remaining_fee_limit,
            account_id,
        }
    }
}

/// Bundler
///
/// 对应 Java: Bundler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundler {
    pub account_id: AccountId,
    pub public_key: Vec<u8>,
    pub total_fees_limit: i64,
    pub bundling_rules: Vec<BundlerRule>,
    pub current_total_fees: i64,
}

impl Bundler {
    pub fn new(
        account_id: AccountId,
        public_key: Vec<u8>,
        total_fees_limit: i64,
        bundling_rules: Vec<BundlerRule>,
    ) -> Self {
        Self {
            account_id,
            public_key,
            total_fees_limit,
            bundling_rules,
            current_total_fees: 0,
        }
    }

    pub fn get_bundler_rate(&self) -> Option<BundlerRate> {
        let min_public_rate = self.bundling_rules
            .iter()
            .filter(|r| r.filter_names.is_empty())
            .map(|r| r.min_rate_nqt_per_fxt)
            .min()?;
        
        let remaining = if self.total_fees_limit > 0 {
            self.total_fees_limit.saturating_sub(self.current_total_fees)
        } else {
            i64::MAX
        };
        
        Some(BundlerRate::new(min_public_rate, remaining, self.account_id))
    }

    pub fn add_rule(&mut self, rule: BundlerRule) {
        self.bundling_rules.push(rule);
    }

    pub fn can_bundle(&self, additional_fee: i64) -> bool {
        if self.total_fees_limit == 0 {
            return true;
        }
        self.current_total_fees.saturating_add(additional_fee) <= self.total_fees_limit
    }

    pub fn add_fees(&mut self, fees: i64) {
        self.current_total_fees = self.current_total_fees.saturating_add(fees);
    }
}

/// Bundler Service
///
/// 管理 all bundlers
pub struct BundlerService {
    bundlers: Arc<RwLock<HashMap<AccountId, Bundler>>>,
}

impl Default for BundlerService {
    fn default() -> Self {
        Self::new()
    }
}

impl BundlerService {
    pub fn new() -> Self {
        Self {
            bundlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_bundler(&self, bundler: Bundler) {
        let mut bundlers = self.bundlers.write().await;
        bundlers.insert(bundler.account_id, bundler);
    }

    pub async fn get_bundler(&self, account_id: AccountId) -> Option<Bundler> {
        let bundlers = self.bundlers.read().await;
        bundlers.get(&account_id).cloned()
    }

    pub async fn get_all_bundlers(&self) -> Vec<Bundler> {
        let bundlers = self.bundlers.read().await;
        bundlers.values().cloned().collect()
    }

    pub async fn stop_bundler(&self, account_id: AccountId) -> Option<Bundler> {
        let mut bundlers = self.bundlers.write().await;
        bundlers.remove(&account_id)
    }

    pub async fn stop_all_bundlers(&self) {
        let mut bundlers = self.bundlers.write().await;
        bundlers.clear();
    }

    pub async fn get_bundler_rates(&self) -> Vec<BundlerRate> {
        let bundlers = self.bundlers.read().await;
        bundlers.values()
            .filter_map(|b| b.get_bundler_rate())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_bundler() -> Bundler {
        let rules = vec![BundlerRule::new(100, 0, "MIN_FEE".to_string())];
        Bundler::new(100, vec![1, 2, 3], 10000, rules)
    }

    #[test]
    fn test_bundler_new() {
        let bundler = create_test_bundler();
        assert_eq!(bundler.account_id, 100);
        assert_eq!(bundler.total_fees_limit, 10000);
        assert_eq!(bundler.bundling_rules.len(), 1);
    }

    #[test]
    fn test_bundler_get_bundler_rate() {
        let bundler = create_test_bundler();
        let rate = bundler.get_bundler_rate().unwrap();
        assert_eq!(rate.min_rate_nqt_per_fxt, 100);
        assert_eq!(rate.remaining_fee_limit, 10000);
    }

    #[test]
    fn test_bundler_can_bundle() {
        let bundler = create_test_bundler();
        assert!(bundler.can_bundle(5000));
        assert!(bundler.can_bundle(10000));
        assert!(!bundler.can_bundle(15000));
    }

    #[test]
    fn test_bundler_add_fees() {
        let mut bundler = create_test_bundler();
        bundler.add_fees(1000);
        assert_eq!(bundler.current_total_fees, 1000);
        
        bundler.add_fees(2000);
        assert_eq!(bundler.current_total_fees, 3000);
    }

    #[tokio::test]
    async fn test_bundler_service() {
        let service = BundlerService::new();
        
        let bundler = create_test_bundler();
        let account_id = bundler.account_id;
        
        service.add_bundler(bundler).await;
        
        let retrieved = service.get_bundler(account_id).await;
        assert!(retrieved.is_some());
        
        let all = service.get_all_bundlers().await;
        assert_eq!(all.len(), 1);
        
        service.stop_bundler(account_id).await;
        let all = service.get_all_bundlers().await;
        assert!(all.is_empty());
    }

    #[test]
    fn test_bundler_rate_new() {
        let rate = BundlerRate::new(100, 1000, 12345);
        assert_eq!(rate.min_rate_nqt_per_fxt, 100);
        assert_eq!(rate.remaining_fee_limit, 1000);
        assert_eq!(rate.account_id, 12345);
    }
}
