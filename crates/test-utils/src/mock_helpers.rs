//! Mock 实现工厂
//!
//! 提供常用的 Mock 实现，用于 API 集成测试和业务逻辑测试。
//! 参照 http-api/tests/api_integration_test.rs 中的 Mock 模式。

use async_trait::async_trait;
use account::{AccountManager, AccountResult};
use tx_engine::{TransactionProcessor, ProcessorResult, TxReceiptInfo, TxStatus};
use blockchain_types::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct MockAccountManager;

#[async_trait]
impl AccountManager for MockAccountManager {
    async fn create_account(&self, _initial_balance: Option<Amount>) -> AccountResult<(crypto::KeyPair, AccountId, String)> {
        let kp = crypto::generate_keypair();
        let account_id = 1u64;
        let address = "NRCS-TEST-ADDR".to_string();
        Ok((kp, account_id, address))
    }

    async fn register_account(&self, _account_id: AccountId, _public_key: Vec<u8>) -> AccountResult<()> {
        Ok(())
    }

    async fn get_balance(&self, _account_id: AccountId) -> AccountResult<Amount> {
        Ok(1000)
    }

    async fn get_account_info(&self, account_id: AccountId) -> AccountResult<Account> {
        Ok(Account {
            id: account_id,
            address: Some("test_address".to_string()),
            balance: 1000,
            unconfirmed_balance: 1000,
            forged_balance: 0,
            reserved_balance: 0,
            guaranteed_balance: 0,
            assets: Default::default(),
            properties: Default::default(),
            lease: None,
            has_control_phasing: false,
            created_at: 0,
            last_updated: 0,
            current_height: 0,
        })
    }

    async fn transfer(&self, _from: AccountId, _to: AccountId, _amount: Amount) -> AccountResult<()> {
        Ok(())
    }

    async fn credit(&self, _account_id: AccountId, _amount: Amount) -> AccountResult<()> {
        Ok(())
    }

    async fn debit(&self, _account_id: AccountId, _amount: Amount) -> AccountResult<()> {
        Ok(())
    }

    async fn get_and_increment_nonce(&self, _sender_id: AccountId) -> AccountResult<u64> {
        Ok(0)
    }

    async fn current_nonce(&self, _account_id: AccountId) -> AccountResult<u64> {
        Ok(0)
    }

    async fn mint_asset(&self, _asset_id: AssetId, _to: AccountId, _amount: Amount) -> AccountResult<()> {
        Ok(())
    }

    async fn burn_asset(&self, _asset_id: AssetId, _from: AccountId, _amount: Amount) -> AccountResult<()> {
        Ok(())
    }

    async fn get_public_key(&self, _account_id: AccountId) -> AccountResult<Option<blockchain_types::PublicKey>> {
        Ok(None)
    }
}

#[derive(Clone)]
pub struct MockTxProcessor;

#[async_trait]
impl TransactionProcessor for MockTxProcessor {
    async fn validate(&self, _tx: &Transaction) -> ProcessorResult<()> {
        Ok(())
    }

    async fn apply_unconfirmed(&self, _tx: &Transaction) -> ProcessorResult<bool> {
        Ok(true)
    }

    async fn rollback_unconfirmed(&self, _tx: &Transaction) -> ProcessorResult<()> {
        Ok(())
    }

    async fn apply(&self, _tx: &Transaction) -> ProcessorResult<()> {
        Ok(())
    }

    async fn apply_phased_fee(&self, _tx: &Transaction) -> ProcessorResult<()> {
        Ok(())
    }

    async fn execute(&self, _tx: &Transaction) -> ProcessorResult<TxReceiptInfo> {
        Ok(TxReceiptInfo {
            transaction_id: 0,
            status: TxStatus::Success,
            block_height: None,
            gas_used: 0,
            logs: vec![],
            contract_address: None,
            executed_at: 0,
        })
    }

    fn set_current_block(&self, _block_id: i64, _height: i32, _timestamp: i32) {}

    async fn validate_batch(&self, _txs: &[Transaction]) -> ProcessorResult<()> {
        Ok(())
    }

    async fn execute_batch(&self, _txs: &[Transaction]) -> ProcessorResult<Vec<TxReceiptInfo>> {
        Ok(vec![])
    }
}

pub fn create_mock_account_manager() -> Arc<dyn AccountManager> {
    Arc::new(MockAccountManager)
}

pub fn create_mock_tx_processor() -> Arc<dyn TransactionProcessor> {
    Arc::new(MockTxProcessor)
}
