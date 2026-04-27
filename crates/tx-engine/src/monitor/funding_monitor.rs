//! Funding Monitor
//!
//! 对应 Java: FundingMonitor.java

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use blockchain_types::{AccountId, Height};

use super::monitored_account::MonitoredAccount;

/// Holding Type
///
/// 对应 Java: HoldingType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HoldingType {
    Nrcs = 0,
    Asset = 1,
    Currency = 2,
}

impl HoldingType {
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(HoldingType::Nrcs),
            1 => Some(HoldingType::Asset),
            2 => Some(HoldingType::Currency),
            _ => None,
        }
    }

    pub fn code(&self) -> u8 {
        *self as u8
    }
}

/// Funding Monitor Error
#[derive(Debug, Error)]
pub enum FundingMonitorError {
    #[error("maximum monitors exceeded: {0}")]
    MaxMonitorsExceeded(String),
    
    #[error("monitor already exists")]
    MonitorAlreadyExists,
    
    #[error("invalid property value: {0}")]
    InvalidPropertyValue(String),
    
    #[error("insufficient balance: {0}")]
    InsufficientBalance(String),
    
    #[error("monitor not found")]
    MonitorNotFound,
}

pub type FundingMonitorResult<T> = Result<T, FundingMonitorError>;

/// Funding Monitor
///
/// 对应 Java: FundingMonitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingMonitor {
    pub holding_type: HoldingType,
    pub holding_id: Option<u64>,
    pub property: String,
    pub amount: i64,
    pub threshold: i64,
    pub interval: i32,
    pub account_id: AccountId,
    pub public_key: Vec<u8>,
}

impl FundingMonitor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        holding_type: HoldingType,
        holding_id: Option<u64>,
        property: String,
        amount: i64,
        threshold: i64,
        interval: i32,
        account_id: AccountId,
        public_key: Vec<u8>,
    ) -> Self {
        Self {
            holding_type,
            holding_id: if holding_type == HoldingType::Nrcs { None } else { holding_id },
            property,
            amount,
            threshold,
            interval,
            account_id,
            public_key,
        }
    }

    pub fn create_monitored_account(
        &self,
        target_account_id: AccountId,
        property_value: &str,
    ) -> FundingMonitorResult<MonitoredAccount> {
        MonitoredAccount::from_property_value(
            target_account_id,
            0,
            property_value,
            self.amount,
            self.threshold,
            self.interval,
        ).map_err(FundingMonitorError::InvalidPropertyValue)
    }

    pub fn should_fund(&self, account: &MonitoredAccount, current_balance: i64, current_height: Height) -> bool {
        account.needs_funding(current_balance, current_height)
    }
}

impl PartialEq for FundingMonitor {
    fn eq(&self, other: &Self) -> bool {
        self.holding_type == other.holding_type
            && self.holding_id == other.holding_id
            && self.property == other.property
            && self.account_id == other.account_id
    }
}

impl Eq for FundingMonitor {}

impl std::hash::Hash for FundingMonitor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.holding_type.hash(state);
        self.holding_id.hash(state);
        self.property.hash(state);
        self.account_id.hash(state);
    }
}

/// Funding Monitor Service
///
/// 管理 all monitors
pub struct FundingMonitorService {
    max_monitors: usize,
    monitors: Arc<RwLock<Vec<FundingMonitor>>>,
    accounts: Arc<RwLock<HashMap<AccountId, Vec<MonitoredAccount>>>>,
}

impl Default for FundingMonitorService {
    fn default() -> Self {
        Self::new()
    }
}

impl FundingMonitorService {
    pub fn new() -> Self {
        Self {
            max_monitors: 100,
            monitors: Arc::new(RwLock::new(Vec::new())),
            accounts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_max_monitors(max_monitors: usize) -> Self {
        Self {
            max_monitors,
            monitors: Arc::new(RwLock::new(Vec::new())),
            accounts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_monitor(
        &self,
        monitor: FundingMonitor,
        properties: Vec<(AccountId, String)>,
    ) -> FundingMonitorResult<()> {
        let mut monitors = self.monitors.write().await;
        
        if monitors.len() >= self.max_monitors {
            return Err(FundingMonitorError::MaxMonitorsExceeded(
                format!("Maximum of {} monitors already started", self.max_monitors)
            ));
        }
        
        if monitors.contains(&monitor) {
            return Err(FundingMonitorError::MonitorAlreadyExists);
        }
        
        let mut accounts = self.accounts.write().await;
        let monitor_index = monitors.len();
        
        for (account_id, property_value) in properties {
            match monitor.create_monitored_account(account_id, &property_value) {
                Ok(monitored_account) => {
                    let mut ma = monitored_account;
                    ma.monitor_index = monitor_index;
                    
                    accounts
                        .entry(account_id)
                        .or_insert_with(Vec::new)
                        .push(ma);
                }
                Err(e) => {
                    tracing::warn!("Failed to create monitored account: {}", e);
                }
            }
        }
        
        monitors.push(monitor);
        Ok(())
    }

    pub async fn stop_monitor(
        &self,
        holding_type: HoldingType,
        holding_id: Option<u64>,
        property: &str,
        account_id: AccountId,
    ) -> FundingMonitorResult<FundingMonitor> {
        let mut monitors = self.monitors.write().await;
        let mut accounts = self.accounts.write().await;
        
        let index = monitors.iter().position(|m| {
            m.holding_type == holding_type
                && m.holding_id == holding_id
                && m.property == property
                && m.account_id == account_id
        }).ok_or(FundingMonitorError::MonitorNotFound)?;
        
        let monitor = monitors.remove(index);
        
        for account_list in accounts.values_mut() {
            account_list.retain(|a| a.monitor_index != index);
        }
        
        Ok(monitor)
    }

    pub async fn stop_all_monitors(&self) -> usize {
        let mut monitors = self.monitors.write().await;
        let mut accounts = self.accounts.write().await;
        
        let count = monitors.len();
        monitors.clear();
        accounts.clear();
        count
    }

    pub async fn get_all_monitors(&self) -> Vec<FundingMonitor> {
        let monitors = self.monitors.read().await;
        monitors.clone()
    }

    pub async fn get_monitored_accounts(&self, monitor_index: usize) -> Vec<MonitoredAccount> {
        let accounts = self.accounts.read().await;
        accounts
            .values()
            .flat_map(|list| list.iter().filter(|a| a.monitor_index == monitor_index).cloned())
            .collect()
    }

    pub async fn get_account_monitors(&self, account_id: AccountId) -> Vec<MonitoredAccount> {
        let accounts = self.accounts.read().await;
        accounts.get(&account_id).cloned().unwrap_or_default()
    }

    pub async fn update_account_height(&self, account_id: AccountId, monitor_index: usize, height: Height) {
        let mut accounts = self.accounts.write().await;
        if let Some(account_list) = accounts.get_mut(&account_id) {
            for account in account_list.iter_mut() {
                if account.monitor_index == monitor_index {
                    account.set_height(height);
                    break;
                }
            }
        }
    }

    pub async fn process_account_event(
        &self,
        account_id: AccountId,
        current_balance: i64,
        current_height: Height,
    ) -> Vec<(usize, i64, AccountId)> {
        let accounts = self.accounts.read().await;
        let monitors = self.monitors.read().await;
        
        let mut funding_needed = Vec::new();
        
        if let Some(account_list) = accounts.get(&account_id) {
            for monitored in account_list {
                if let Some(monitor) = monitors.get(monitored.monitor_index) {
                    if monitor.should_fund(monitored, current_balance, current_height) {
                        funding_needed.push((
                            monitored.monitor_index,
                            monitored.amount,
                            account_id,
                        ));
                    }
                }
            }
        }
        
        funding_needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_holding_type_from_code() {
        assert_eq!(HoldingType::from_code(0), Some(HoldingType::Nrcs));
        assert_eq!(HoldingType::from_code(1), Some(HoldingType::Asset));
        assert_eq!(HoldingType::from_code(2), Some(HoldingType::Currency));
        assert_eq!(HoldingType::from_code(99), None);
    }

    #[test]
    fn test_funding_monitor_new() {
        let monitor = FundingMonitor::new(
            HoldingType::Nrcs,
            None,
            "fund".to_string(),
            1000,
            100,
            10,
            12345,
            vec![1, 2, 3],
        );
        
        assert_eq!(monitor.holding_type, HoldingType::Nrcs);
        assert_eq!(monitor.amount, 1000);
        assert_eq!(monitor.threshold, 100);
        assert_eq!(monitor.interval, 10);
    }

    #[test]
    fn test_funding_monitor_equality() {
        let m1 = FundingMonitor::new(
            HoldingType::Nrcs,
            None,
            "fund".to_string(),
            1000,
            100,
            10,
            12345,
            vec![1, 2, 3],
        );
        
        let m2 = FundingMonitor::new(
            HoldingType::Nrcs,
            None,
            "fund".to_string(),
            2000,
            200,
            20,
            12345,
            vec![4, 5, 6],
        );
        
        assert_eq!(m1, m2);
    }

    #[tokio::test]
    async fn test_funding_monitor_service() {
        let service = FundingMonitorService::new();
        
        let monitor = FundingMonitor::new(
            HoldingType::Nrcs,
            None,
            "fund".to_string(),
            1000,
            100,
            10,
            12345,
            vec![1, 2, 3],
        );
        
        let properties = vec![(100, r#"{"amount": 500}"#.to_string())];
        
        service.start_monitor(monitor.clone(), properties).await.unwrap();
        
        let all = service.get_all_monitors().await;
        assert_eq!(all.len(), 1);
        
        let monitored = service.get_monitored_accounts(0).await;
        assert_eq!(monitored.len(), 1);
        
        service.stop_all_monitors().await;
        let all = service.get_all_monitors().await;
        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn test_process_account_event() {
        let service = FundingMonitorService::new();
        
        let monitor = FundingMonitor::new(
            HoldingType::Nrcs,
            None,
            "fund".to_string(),
            1000,
            100,
            10,
            12345,
            vec![1, 2, 3],
        );
        
        let properties = vec![(100, "".to_string())];
        service.start_monitor(monitor, properties).await.unwrap();
        
        service.update_account_height(100, 0, 100).await;
        
        let funding = service.process_account_event(100, 50, 110).await;
        assert_eq!(funding.len(), 1);
        
        let funding = service.process_account_event(100, 150, 110).await;
        assert!(funding.is_empty());
    }
}
