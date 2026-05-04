//! Account Restrictions Service
//!
//! 对应 Java: AccountRestrictions.java, AccountPhasingOnly.java

use std::collections::HashSet;
use std::sync::Arc;
use thiserror::Error;

use blockchain_types::{AccountId, Amount, Height};
use blockchain_types::constants::ONE_NRCS;

use crate::restrictions::types::{AccountControlType, VotingModel, MinBalanceModel};
use orm::models::{AccountModel, AccountControlPhasingModel};
use orm::repository::AccountRepository;

/// Account Restriction Error
#[derive(Debug, Error)]
pub enum RestrictionError {
    #[error("account control error: {0}")]
    ControlError(String),
    #[error("not currently valid: {0}")]
    NotCurrentlyValid(String),
    #[error("account does not exist: {0}")]
    AccountNotFound(AccountId),
    #[error("database error: {0}")]
    Database(String),
}

pub type RestrictionResult<T> = Result<T, RestrictionError>;

/// Phasing Parameters
///
/// 对应 Java: PhasingParams.java
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhasingParams {
    pub voting_model: VotingModel,
    pub quorum: Option<i64>,
    pub min_balance: Option<i64>,
    pub holding_id: Option<i64>,
    pub min_balance_model: MinBalanceModel,
    pub whitelist: Vec<AccountId>,
}

impl Default for PhasingParams {
    fn default() -> Self {
        Self {
            voting_model: VotingModel::None,
            quorum: None,
            min_balance: None,
            holding_id: None,
            min_balance_model: MinBalanceModel::None,
            whitelist: Vec::new(),
        }
    }
}

impl PhasingParams {
    pub fn is_none(&self) -> bool {
        self.voting_model == VotingModel::None
    }

    pub fn check_approvable(&self) -> RestrictionResult<()> {
        if self.voting_model == VotingModel::None {
            return Ok(());
        }

        match self.voting_model {
            VotingModel::Account
                if self.quorum.unwrap_or(0) as usize > self.whitelist.len() =>
            {
                return Err(RestrictionError::NotCurrentlyValid(
                    "Quorum exceeds whitelist size".to_string(),
                ));
            }
            VotingModel::NqtBalance | VotingModel::Asset | VotingModel::Currency
                if self.min_balance.unwrap_or(0) == 0 =>
            {
                return Err(RestrictionError::NotCurrentlyValid(
                    "Min balance not set".to_string(),
                ));
            }
            _ => {}
        }

        Ok(())
    }
}

/// Account Phasing Only Control
///
/// 对应 Java: AccountPhasingOnly.java
#[derive(Debug, Clone)]
pub struct AccountPhasingOnly {
    pub account_id: AccountId,
    pub phasing_params: PhasingParams,
    pub max_fees: i64,
    pub min_duration: i16,
    pub max_duration: i16,
    pub height: Height,
}

impl AccountPhasingOnly {
    pub fn from_model(model: &AccountControlPhasingModel) -> Option<Self> {
        let voting_model = VotingModel::from_code(model.voting_model as u8)?;
        let min_balance_model = MinBalanceModel::from_code(model.min_balance_model.unwrap_or(0) as u8)
            .unwrap_or(MinBalanceModel::None);

        let whitelist: Vec<AccountId> = model
            .whitelist
            .as_ref()
            .map(|s| s.split(',').filter_map(|id| id.parse().ok()).collect())
            .unwrap_or_default();

        Some(Self {
            account_id: model.account_id as AccountId,
            phasing_params: PhasingParams {
                voting_model,
                quorum: model.quorum,
                min_balance: model.min_balance,
                holding_id: model.holding_id,
                min_balance_model,
                whitelist,
            },
            max_fees: model.max_fees.unwrap_or(0),
            min_duration: model.min_duration.unwrap_or(0),
            max_duration: model.max_duration.unwrap_or(0),
            height: model.height as Height,
        })
    }

    pub fn check_transaction(
        &self,
        fee: Amount,
        current_height: Height,
        phased_fees: Amount,
        validating_at_finish: bool,
        phasing_finish_height: Option<Height>,
    ) -> RestrictionResult<()> {
        if !validating_at_finish
            && self.max_fees > 0
            && fee + phased_fees > self.max_fees as Amount
        {
            return Err(RestrictionError::ControlError(format!(
                "Maximum total fees limit of {} NRCS exceeded",
                self.max_fees as f64 / ONE_NRCS as f64
            )));
        }

        self.phasing_params.check_approvable()?;

        if !validating_at_finish {
            if let Some(finish_height) = phasing_finish_height {
                let duration = finish_height as i32 - current_height as i32;
                if (self.max_duration > 0 && duration > self.max_duration as i32)
                    || (self.min_duration > 0 && duration < self.min_duration as i32)
                {
                    return Err(RestrictionError::ControlError(format!(
                        "Invalid phasing duration {}",
                        duration
                    )));
                }
            } else {
                return Err(RestrictionError::ControlError(
                    "Non-phased transaction when phasing account control is enabled".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Account Restrictions Service
///
/// 对应 Java: AccountRestrictions.java
pub struct AccountRestrictions {
    account_repo: Arc<dyn AccountRepository>,
}

impl AccountRestrictions {
    pub fn new(account_repo: Arc<dyn AccountRepository>) -> Self {
        Self { account_repo }
    }

    pub async fn check_transaction(
        &self,
        sender_id: AccountId,
        fee: Amount,
        current_height: Height,
        phasing_only: Option<&AccountPhasingOnly>,
        validating_at_finish: bool,
        phasing_finish_height: Option<Height>,
    ) -> RestrictionResult<()> {
        let account = self
            .account_repo
            .find_by_account_id(sender_id as i64)
            .await
            .map_err(|e| RestrictionError::Database(e.to_string()))?
            .ok_or(RestrictionError::AccountNotFound(sender_id))?;

        let controls = self.get_account_controls(&account);

        if controls.contains(&AccountControlType::PhasingOnly) {
            if let Some(phasing) = phasing_only {
                phasing.check_transaction(fee, current_height, 0, validating_at_finish, phasing_finish_height)?;
            }
        }

        Ok(())
    }

    fn get_account_controls(&self, account: &AccountModel) -> HashSet<AccountControlType> {
        let mut controls = HashSet::new();

        if account.has_control_phasing {
            controls.insert(AccountControlType::PhasingOnly);
        }

        controls
    }

    pub fn is_block_duplicate(
        &self,
        account: Option<&AccountModel>,
        phasing_only: Option<&AccountPhasingOnly>,
    ) -> bool {
        let controls = account
            .map(|a| self.get_account_controls(a))
            .unwrap_or_default();

        if !controls.contains(&AccountControlType::PhasingOnly) {
            return false;
        }

        if let Some(phasing) = phasing_only {
            if phasing.max_fees == 0 {
                return false;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phasing_params_default() {
        let params = PhasingParams::default();
        assert!(params.is_none());
        assert_eq!(params.voting_model, VotingModel::None);
    }

    #[test]
    fn test_phasing_params_check_approvable() {
        let params = PhasingParams::default();
        assert!(params.check_approvable().is_ok());

        let mut params = PhasingParams {
            voting_model: VotingModel::Account,
            quorum: Some(10),
            whitelist: vec![1, 2, 3],
            ..Default::default()
        };
        assert!(params.check_approvable().is_err());

        params.whitelist = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        assert!(params.check_approvable().is_ok());
    }
}
