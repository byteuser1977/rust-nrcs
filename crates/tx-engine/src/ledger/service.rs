//! Account Ledger Service
//!
//! 对应 Java: AccountLedger.java, LedgerEntry.java
//!
//! 账户账本服务，记录账户余额变更历史

use std::collections::HashSet;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::ledger::types::{LedgerEvent, LedgerHolding};
use orm::models::AccountLedgerModel;
use orm::repository::{AccountLedgerRepository, RepositoryResult};

/// Ledger Entry
///
/// 对应 Java: LedgerEntry.java
#[derive(Debug, Clone, PartialEq)]
pub struct LedgerEntry {
    pub db_id: i64,
    pub account_id: i64,
    pub event: LedgerEvent,
    pub event_id: i64,
    pub holding: Option<LedgerHolding>,
    pub holding_id: Option<i64>,
    pub change: i64,
    pub balance: i64,
    pub block_id: i64,
    pub height: i32,
    pub timestamp: i32,
}

impl LedgerEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        event: LedgerEvent,
        event_id: i64,
        account_id: i64,
        holding: Option<LedgerHolding>,
        holding_id: Option<i64>,
        change: i64,
        balance: i64,
        block_id: i64,
        height: i32,
        timestamp: i32,
    ) -> Self {
        Self {
            db_id: 0,
            account_id,
            event,
            event_id,
            holding,
            holding_id,
            change,
            balance,
            block_id,
            height,
            timestamp,
        }
    }

    pub fn from_model(model: &AccountLedgerModel) -> Option<Self> {
        let event = LedgerEvent::from_code(model.event_type)?;
        let holding = LedgerHolding::from_code(model.holding_type);
        
        Some(Self {
            db_id: model.db_id,
            account_id: model.account_id,
            event,
            event_id: model.event_id,
            holding,
            holding_id: model.holding_id,
            change: model.change,
            balance: model.balance,
            block_id: model.block_id,
            height: model.height,
            timestamp: model.timestamp,
        })
    }

    pub fn to_model(&self) -> AccountLedgerModel {
        AccountLedgerModel {
            db_id: self.db_id,
            account_id: self.account_id,
            event_type: self.event.code(),
            event_id: self.event_id,
            holding_type: self.holding.map(|h| h.code()).unwrap_or(-1),
            holding_id: self.holding_id,
            change: self.change,
            balance: self.balance,
            block_id: self.block_id,
            height: self.height,
            timestamp: self.timestamp,
        }
    }

    pub fn update_change(&mut self, amount: i64) {
        self.change += amount;
    }

    pub fn ledger_id(&self) -> i64 {
        self.db_id
    }
}

impl std::hash::Hash for LedgerEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.account_id.hash(state);
        self.event.code().hash(state);
        self.event_id.hash(state);
        self.holding.map(|h| h.code()).hash(state);
        self.holding_id.hash(state);
    }
}

impl Eq for LedgerEntry {}

/// Account Ledger Event for listeners
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountLedgerEvent {
    AddEntry,
}

/// Account Ledger Configuration
#[derive(Debug, Clone)]
pub struct LedgerConfig {
    pub enabled: bool,
    pub track_all_accounts: bool,
    pub track_accounts: HashSet<i64>,
    pub log_unconfirmed: u8,
    pub trim_keep: i32,
}

impl Default for LedgerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            track_all_accounts: false,
            track_accounts: HashSet::new(),
            log_unconfirmed: 1,
            trim_keep: 30000,
        }
    }
}

/// Account Ledger Service
///
/// 对应 Java: AccountLedger.java
pub struct AccountLedger {
    config: LedgerConfig,
    repository: Arc<dyn AccountLedgerRepository>,
    pending_entries: RwLock<Vec<LedgerEntry>>,
    is_processing_block: RwLock<bool>,
    current_height: RwLock<i32>,
    #[allow(dead_code)]
    last_known_block: RwLock<i64>,
}

impl AccountLedger {
    pub fn new(config: LedgerConfig, repository: Arc<dyn AccountLedgerRepository>) -> Self {
        Self {
            config,
            repository,
            pending_entries: RwLock::new(Vec::new()),
            is_processing_block: RwLock::new(false),
            current_height: RwLock::new(0),
            last_known_block: RwLock::new(0),
        }
    }

    pub fn set_processing_block(&self, processing: bool) {
        *self.is_processing_block.write() = processing;
    }

    pub fn set_current_height(&self, height: i32) {
        *self.current_height.write() = height;
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn must_log_entry(&self, account_id: i64, is_unconfirmed: bool) -> bool {
        if !self.config.enabled {
            return false;
        }

        if !self.config.track_all_accounts && !self.config.track_accounts.contains(&account_id) {
            return false;
        }

        if !*self.is_processing_block.read() {
            return false;
        }

        if is_unconfirmed && self.config.log_unconfirmed == 0 {
            return false;
        }

        if !is_unconfirmed && self.config.log_unconfirmed == 2 {
            return false;
        }

        let height = *self.current_height.read();
        if self.config.trim_keep > 0 && height <= (self.config.trim_keep) {
            return false;
        }

        true
    }

    pub fn log_entry(&self, entry: LedgerEntry) {
        let mut pending = self.pending_entries.write();
        
        if let Some(pos) = pending.iter().position(|e| {
            e.account_id == entry.account_id
                && e.event == entry.event
                && e.event_id == entry.event_id
                && e.holding == entry.holding
                && e.holding_id == entry.holding_id
        }) {
            let existing = pending.remove(pos);
            let mut new_entry = entry;
            new_entry.update_change(existing.change);
            
            let adjusted_balance = existing.balance - existing.change;
            for e in pending.iter_mut().skip(pos) {
                if e.account_id == new_entry.account_id
                    && e.holding == new_entry.holding
                    && e.holding_id == new_entry.holding_id
                {
                    let adj = adjusted_balance + e.change;
                    e.balance = adj;
                }
            }
            pending.push(new_entry);
        } else {
            pending.push(entry);
        }
    }

    pub async fn commit_entries(&self) -> RepositoryResult<()> {
        let pending = self.pending_entries.read().clone();
        
        for entry in pending.iter() {
            let model = entry.to_model();
            self.repository.insert(&model).await?;
        }

        self.pending_entries.write().clear();
        Ok(())
    }

    pub fn clear_entries(&self) {
        self.pending_entries.write().clear();
    }

    pub async fn get_entry(&self, ledger_id: i64) -> RepositoryResult<Option<LedgerEntry>> {
        let model = self.repository.find_by_id(ledger_id).await?;
        Ok(model.and_then(|m| LedgerEntry::from_model(&m)))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn get_entries(
        &self,
        account_id: i64,
        event: Option<LedgerEvent>,
        event_id: i64,
        holding: Option<LedgerHolding>,
        holding_id: i64,
        first_index: i32,
        last_index: i32,
    ) -> RepositoryResult<Vec<LedgerEntry>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        let limit = if last_index > 0 {
            (last_index - first_index + 1) as i64
        } else {
            100
        };

        let records = self.repository.find_by_account(account_id, limit).await?;
        
        let entries: Vec<LedgerEntry> = records
            .iter()
            .filter_map(LedgerEntry::from_model)
            .filter(|e| {
                if let Some(ev) = event {
                    if e.event != ev {
                        return false;
                    }
                    if event_id > 0 && e.event_id != event_id {
                        return false;
                    }
                }
                if let Some(h) = holding {
                    if e.holding != Some(h) {
                        return false;
                    }
                    if holding_id > 0 && e.holding_id != Some(holding_id) {
                        return false;
                    }
                }
                true
            })
            .collect();

        Ok(entries)
    }

    pub async fn get_entries_by_block(&self, block_id: i64) -> RepositoryResult<Vec<LedgerEntry>> {
        let records = self.repository.find_by_block(block_id).await?;
        Ok(records.iter().filter_map(LedgerEntry::from_model).collect())
    }

    pub fn get_pending_count(&self) -> usize {
        self.pending_entries.read().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_entry_creation() {
        let entry = LedgerEntry::new(
            LedgerEvent::OrdinaryPayment,
            12345,
            100,
            Some(LedgerHolding::NrcsBalance),
            None,
            1000,
            5000,
            1,
            100,
            1234567890,
        );

        assert_eq!(entry.event, LedgerEvent::OrdinaryPayment);
        assert_eq!(entry.change, 1000);
        assert_eq!(entry.balance, 5000);
    }

    #[test]
    fn test_ledger_entry_update_change() {
        let mut entry = LedgerEntry::new(
            LedgerEvent::OrdinaryPayment,
            12345,
            100,
            Some(LedgerHolding::NrcsBalance),
            None,
            1000,
            5000,
            1,
            100,
            1234567890,
        );

        entry.update_change(500);
        assert_eq!(entry.change, 1500);
    }

    #[test]
    fn test_ledger_config_default() {
        let config = LedgerConfig::default();
        assert!(!config.enabled);
        assert!(!config.track_all_accounts);
        assert_eq!(config.log_unconfirmed, 1);
        assert_eq!(config.trim_keep, 30000);
    }
}
