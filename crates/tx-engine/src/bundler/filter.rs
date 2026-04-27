//! Bundler Filter
//!
//! 对应 Java: IBundlerFilter

use async_trait::async_trait;
use blockchain_types::AccountId;

/// Bundler Filter Trait
///
/// 对应 Java: IBundlerFilter
#[async_trait]
pub trait BundlerFilter: Send + Sync {
    fn name(&self) -> &str;
    
    fn set_parameter(&mut self, parameter: &str);
    
    fn get_parameter(&self) -> &str;
    
    fn check(&self, bundler_account_id: AccountId, transaction_data: &[u8]) -> bool;
}

/// Default Filter - accepts all transactions
pub struct DefaultFilter {
    parameter: String,
}

impl DefaultFilter {
    pub const NAME: &'static str = "DEFAULT";
    
    pub fn new() -> Self {
        Self {
            parameter: String::new(),
        }
    }
}

impl Default for DefaultFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BundlerFilter for DefaultFilter {
    fn name(&self) -> &str {
        Self::NAME
    }
    
    fn set_parameter(&mut self, parameter: &str) {
        self.parameter = parameter.to_string();
    }
    
    fn get_parameter(&self) -> &str {
        &self.parameter
    }
    
    fn check(&self, _bundler_account_id: AccountId, _transaction_data: &[u8]) -> bool {
        true
    }
}

/// Account Filter - only accepts transactions from specific accounts
pub struct AccountFilter {
    parameter: String,
    allowed_accounts: Vec<AccountId>,
}

impl AccountFilter {
    pub const NAME: &'static str = "ACCOUNT";
    
    pub fn new() -> Self {
        Self {
            parameter: String::new(),
            allowed_accounts: Vec::new(),
        }
    }
    
    pub fn with_accounts(accounts: Vec<AccountId>) -> Self {
        Self {
            parameter: accounts.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(","),
            allowed_accounts: accounts,
        }
    }
}

impl Default for AccountFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BundlerFilter for AccountFilter {
    fn name(&self) -> &str {
        Self::NAME
    }
    
    fn set_parameter(&mut self, parameter: &str) {
        self.parameter = parameter.to_string();
        self.allowed_accounts = parameter
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();
    }
    
    fn get_parameter(&self) -> &str {
        &self.parameter
    }
    
    fn check(&self, _bundler_account_id: AccountId, transaction_data: &[u8]) -> bool {
        if self.allowed_accounts.is_empty() {
            return true;
        }
        
        if transaction_data.len() < 8 {
            return false;
        }
        
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&transaction_data[0..8]);
        let sender_id = u64::from_le_bytes(buf);
        
        self.allowed_accounts.contains(&sender_id)
    }
}

/// Transaction Type Filter - only accepts specific transaction types
pub struct TransactionTypeFilter {
    parameter: String,
    allowed_types: Vec<u8>,
}

impl TransactionTypeFilter {
    pub const NAME: &'static str = "TX_TYPE";
    
    pub fn new() -> Self {
        Self {
            parameter: String::new(),
            allowed_types: Vec::new(),
        }
    }
    
    pub fn with_types(types: Vec<u8>) -> Self {
        Self {
            parameter: types.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(","),
            allowed_types: types,
        }
    }
}

impl Default for TransactionTypeFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BundlerFilter for TransactionTypeFilter {
    fn name(&self) -> &str {
        Self::NAME
    }
    
    fn set_parameter(&mut self, parameter: &str) {
        self.parameter = parameter.to_string();
        self.allowed_types = parameter
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();
    }
    
    fn get_parameter(&self) -> &str {
        &self.parameter
    }
    
    fn check(&self, _bundler_account_id: AccountId, transaction_data: &[u8]) -> bool {
        if self.allowed_types.is_empty() {
            return true;
        }
        
        if transaction_data.len() < 18 {
            return false;
        }
        
        let type_byte = transaction_data[16];
        
        self.allowed_types.contains(&type_byte)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_filter() {
        let filter = DefaultFilter::new();
        assert_eq!(filter.name(), "DEFAULT");
        assert!(filter.check(100, &[1, 2, 3]));
    }

    #[test]
    fn test_account_filter() {
        let filter = AccountFilter::with_accounts(vec![100, 200]);
        assert_eq!(filter.name(), "ACCOUNT");
        
        let mut tx_data = vec![0u8; 100];
        tx_data[0..8].copy_from_slice(&100u64.to_le_bytes());
        assert!(filter.check(300, &tx_data));
        
        tx_data[0..8].copy_from_slice(&300u64.to_le_bytes());
        assert!(!filter.check(300, &tx_data));
    }

    #[test]
    fn test_account_filter_set_parameter() {
        let mut filter = AccountFilter::new();
        filter.set_parameter("100,200,300");
        
        assert_eq!(filter.allowed_accounts.len(), 3);
        assert!(filter.allowed_accounts.contains(&100));
        assert!(filter.allowed_accounts.contains(&200));
        assert!(filter.allowed_accounts.contains(&300));
    }

    #[test]
    fn test_transaction_type_filter() {
        let filter = TransactionTypeFilter::with_types(vec![0, 1, 2]);
        assert_eq!(filter.name(), "TX_TYPE");
        
        let mut tx_data = vec![0u8; 100];
        tx_data[16] = 0;
        tx_data[17] = 0;
        assert!(filter.check(100, &tx_data));
        
        tx_data[16] = 1;
        tx_data[17] = 0;
        assert!(filter.check(100, &tx_data));
        
        tx_data[16] = 5;
        tx_data[17] = 0;
        assert!(!filter.check(100, &tx_data));
    }
}
