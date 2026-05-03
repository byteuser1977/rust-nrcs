//! Restrictions 外部测试
//!
//! 测试 AccountControlType、PhasingParams、AccountPhasingOnly、VotingModel 等账户限制功能

use tx_engine::restrictions::{
    AccountControlType, AccountPhasingOnly, PhasingParams, RestrictionError,
};
use tx_engine::restrictions::types::{VotingModel, MinBalanceModel};

#[test]
fn test_account_control_type_code() {
    assert_eq!(AccountControlType::PhasingOnly.code(), 1);
}

#[test]
fn test_account_control_type_from_code() {
    assert_eq!(AccountControlType::from_code(1), Some(AccountControlType::PhasingOnly));
    assert_eq!(AccountControlType::from_code(0), None);
    assert_eq!(AccountControlType::from_code(99), None);
}

#[test]
fn test_voting_model_code() {
    assert_eq!(VotingModel::None.code(), 0);
    assert_eq!(VotingModel::Account.code(), 1);
    assert_eq!(VotingModel::NqtBalance.code(), 2);
    assert_eq!(VotingModel::Asset.code(), 3);
    assert_eq!(VotingModel::Currency.code(), 4);
    assert_eq!(VotingModel::Transaction.code(), 5);
    assert_eq!(VotingModel::Bylance.code(), 6);
    assert_eq!(VotingModel::Hash.code(), 7);
}

#[test]
fn test_voting_model_from_code() {
    assert_eq!(VotingModel::from_code(0), Some(VotingModel::None));
    assert_eq!(VotingModel::from_code(1), Some(VotingModel::Account));
    assert_eq!(VotingModel::from_code(2), Some(VotingModel::NqtBalance));
    assert_eq!(VotingModel::from_code(3), Some(VotingModel::Asset));
    assert_eq!(VotingModel::from_code(4), Some(VotingModel::Currency));
    assert_eq!(VotingModel::from_code(5), Some(VotingModel::Transaction));
    assert_eq!(VotingModel::from_code(6), Some(VotingModel::Bylance));
    assert_eq!(VotingModel::from_code(7), Some(VotingModel::Hash));
    assert_eq!(VotingModel::from_code(99), None);
}

#[test]
fn test_voting_model_is_balance_independent() {
    assert!(VotingModel::None.is_balance_independent());
    assert!(VotingModel::Account.is_balance_independent());
    assert!(!VotingModel::NqtBalance.is_balance_independent());
    assert!(!VotingModel::Asset.is_balance_independent());
    assert!(!VotingModel::Currency.is_balance_independent());
}

#[test]
fn test_min_balance_model_code() {
    assert_eq!(MinBalanceModel::None.code(), 0);
    assert_eq!(MinBalanceModel::NqtBalance.code(), 1);
    assert_eq!(MinBalanceModel::Asset.code(), 2);
    assert_eq!(MinBalanceModel::Currency.code(), 3);
}

#[test]
fn test_min_balance_model_from_code() {
    assert_eq!(MinBalanceModel::from_code(0), Some(MinBalanceModel::None));
    assert_eq!(MinBalanceModel::from_code(1), Some(MinBalanceModel::NqtBalance));
    assert_eq!(MinBalanceModel::from_code(2), Some(MinBalanceModel::Asset));
    assert_eq!(MinBalanceModel::from_code(3), Some(MinBalanceModel::Currency));
    assert_eq!(MinBalanceModel::from_code(99), None);
}

#[test]
fn test_phasing_params_default() {
    let params = PhasingParams::default();
    assert!(params.is_none());
    assert_eq!(params.voting_model, VotingModel::None);
    assert!(params.quorum.is_none());
    assert!(params.min_balance.is_none());
    assert!(params.whitelist.is_empty());
}

#[test]
fn test_phasing_params_is_none() {
    let params = PhasingParams {
        voting_model: VotingModel::None,
        ..Default::default()
    };
    assert!(params.is_none());

    let params = PhasingParams {
        voting_model: VotingModel::Account,
        ..Default::default()
    };
    assert!(!params.is_none());
}

#[test]
fn test_phasing_params_check_approvable_none() {
    let params = PhasingParams::default();
    assert!(params.check_approvable().is_ok());
}

#[test]
fn test_phasing_params_check_approvable_account_valid() {
    let params = PhasingParams {
        voting_model: VotingModel::Account,
        quorum: Some(3),
        whitelist: vec![1, 2, 3, 4, 5],
        ..Default::default()
    };
    assert!(params.check_approvable().is_ok());
}

#[test]
fn test_phasing_params_check_approvable_account_quorum_exceeds() {
    let params = PhasingParams {
        voting_model: VotingModel::Account,
        quorum: Some(10),
        whitelist: vec![1, 2, 3],
        ..Default::default()
    };
    let result = params.check_approvable();
    assert!(result.is_err());
    if let Err(RestrictionError::NotCurrentlyValid(msg)) = result {
        assert!(msg.contains("Quorum exceeds"));
    } else {
        panic!("Expected NotCurrentlyValid error");
    }
}

#[test]
fn test_phasing_params_check_approvable_nqt_balance_no_min() {
    let params = PhasingParams {
        voting_model: VotingModel::NqtBalance,
        min_balance: None,
        ..Default::default()
    };
    let result = params.check_approvable();
    assert!(result.is_err());
    if let Err(RestrictionError::NotCurrentlyValid(msg)) = result {
        assert!(msg.contains("Min balance not set"));
    } else {
        panic!("Expected NotCurrentlyValid error");
    }
}

#[test]
fn test_phasing_params_check_approvable_nqt_balance_with_min() {
    let params = PhasingParams {
        voting_model: VotingModel::NqtBalance,
        min_balance: Some(10000),
        ..Default::default()
    };
    assert!(params.check_approvable().is_ok());
}

#[test]
fn test_phasing_params_check_approvable_asset_no_min() {
    let params = PhasingParams {
        voting_model: VotingModel::Asset,
        min_balance: None,
        ..Default::default()
    };
    assert!(params.check_approvable().is_err());
}

#[test]
fn test_phasing_params_check_approvable_currency_no_min() {
    let params = PhasingParams {
        voting_model: VotingModel::Currency,
        min_balance: None,
        ..Default::default()
    };
    assert!(params.check_approvable().is_err());
}

#[test]
fn test_account_phasing_only_check_transaction_no_phasing() {
    let phasing = AccountPhasingOnly {
        account_id: 100,
        phasing_params: PhasingParams::default(),
        max_fees: 0,
        min_duration: 0,
        max_duration: 0,
        height: 0,
    };

    let result = phasing.check_transaction(100, 500, 0, false, None);
    assert!(result.is_err(), "Non-phased transaction should fail when phasing control enabled");
}

#[test]
fn test_account_phasing_only_check_transaction_with_phasing() {
    let phasing = AccountPhasingOnly {
        account_id: 100,
        phasing_params: PhasingParams::default(),
        max_fees: 0,
        min_duration: 0,
        max_duration: 0,
        height: 0,
    };

    let result = phasing.check_transaction(100, 500, 0, false, Some(600));
    assert!(result.is_ok());
}

#[test]
fn test_account_phasing_only_check_transaction_max_fees() {
    let phasing = AccountPhasingOnly {
        account_id: 100,
        phasing_params: PhasingParams::default(),
        max_fees: 500,
        min_duration: 0,
        max_duration: 0,
        height: 0,
    };

    let result = phasing.check_transaction(600, 500, 0, false, Some(600));
    assert!(result.is_err(), "Fee exceeds max_fees");
}

#[test]
fn test_account_phasing_only_check_transaction_within_max_fees() {
    let phasing = AccountPhasingOnly {
        account_id: 100,
        phasing_params: PhasingParams::default(),
        max_fees: 1000,
        min_duration: 0,
        max_duration: 0,
        height: 0,
    };

    let result = phasing.check_transaction(500, 500, 0, false, Some(600));
    assert!(result.is_ok());
}

#[test]
fn test_account_phasing_only_check_transaction_validating_at_finish() {
    let phasing = AccountPhasingOnly {
        account_id: 100,
        phasing_params: PhasingParams::default(),
        max_fees: 500,
        min_duration: 0,
        max_duration: 0,
        height: 0,
    };

    let result = phasing.check_transaction(600, 500, 0, true, Some(600));
    assert!(result.is_ok(), "Validating at finish should skip max_fees check");
}

#[test]
fn test_account_phasing_only_check_transaction_min_duration() {
    let phasing = AccountPhasingOnly {
        account_id: 100,
        phasing_params: PhasingParams::default(),
        max_fees: 0,
        min_duration: 10,
        max_duration: 0,
        height: 0,
    };

    let result = phasing.check_transaction(100, 500, 0, false, Some(505));
    assert!(result.is_err(), "Duration 5 < min_duration 10");

    let result = phasing.check_transaction(100, 500, 0, false, Some(510));
    assert!(result.is_ok(), "Duration 10 >= min_duration 10");
}

#[test]
fn test_account_phasing_only_check_transaction_max_duration() {
    let phasing = AccountPhasingOnly {
        account_id: 100,
        phasing_params: PhasingParams::default(),
        max_fees: 0,
        min_duration: 0,
        max_duration: 100,
        height: 0,
    };

    let result = phasing.check_transaction(100, 500, 0, false, Some(700));
    assert!(result.is_err(), "Duration 200 > max_duration 100");

    let result = phasing.check_transaction(100, 500, 0, false, Some(550));
    assert!(result.is_ok(), "Duration 50 <= max_duration 100");
}
