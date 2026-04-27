//! Account Restriction Types
//!
//! 对应 Java: AccountControlType.java, VotingModel.java

use serde::{Deserialize, Serialize};

/// Account Control Type
///
/// 对应 Java: AccountControlType.java
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccountControlType {
    PhasingOnly,
}

impl AccountControlType {
    pub fn code(&self) -> u8 {
        match self {
            AccountControlType::PhasingOnly => 1,
        }
    }

    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(AccountControlType::PhasingOnly),
            _ => None,
        }
    }
}

/// Voting Model
///
/// 对应 Java: VotingModel.java
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VotingModel {
    None = 0,
    Account = 1,
    NqtBalance = 2,
    Asset = 3,
    Currency = 4,
    Transaction = 5,
    Bylance = 6,
    Hash = 7,
}

impl VotingModel {
    pub fn code(&self) -> u8 {
        *self as u8
    }

    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(VotingModel::None),
            1 => Some(VotingModel::Account),
            2 => Some(VotingModel::NqtBalance),
            3 => Some(VotingModel::Asset),
            4 => Some(VotingModel::Currency),
            5 => Some(VotingModel::Transaction),
            6 => Some(VotingModel::Bylance),
            7 => Some(VotingModel::Hash),
            _ => None,
        }
    }

    pub fn is_balance_independent(&self) -> bool {
        matches!(self, VotingModel::Account | VotingModel::None)
    }
}

/// Min Balance Model
///
/// 对应 Java: MinBalanceModel.java
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MinBalanceModel {
    None = 0,
    NqtBalance = 1,
    Asset = 2,
    Currency = 3,
}

impl MinBalanceModel {
    pub fn code(&self) -> u8 {
        *self as u8
    }

    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(MinBalanceModel::None),
            1 => Some(MinBalanceModel::NqtBalance),
            2 => Some(MinBalanceModel::Asset),
            3 => Some(MinBalanceModel::Currency),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_control_type_code() {
        assert_eq!(AccountControlType::PhasingOnly.code(), 1);
        assert_eq!(AccountControlType::from_code(1), Some(AccountControlType::PhasingOnly));
        assert_eq!(AccountControlType::from_code(0), None);
    }

    #[test]
    fn test_voting_model_code() {
        assert_eq!(VotingModel::None.code(), 0);
        assert_eq!(VotingModel::Account.code(), 1);
        assert_eq!(VotingModel::from_code(0), Some(VotingModel::None));
        assert_eq!(VotingModel::from_code(1), Some(VotingModel::Account));
        assert_eq!(VotingModel::from_code(99), None);
    }

    #[test]
    fn test_min_balance_model_code() {
        assert_eq!(MinBalanceModel::None.code(), 0);
        assert_eq!(MinBalanceModel::NqtBalance.code(), 1);
        assert_eq!(MinBalanceModel::from_code(0), Some(MinBalanceModel::None));
        assert_eq!(MinBalanceModel::from_code(99), None);
    }
}
