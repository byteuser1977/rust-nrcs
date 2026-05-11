//! Node services coordination (simplified placeholder)
//!
//! **DEPRECATED**: This module is not used by main.rs. The actual node initialization
//! is in main.rs::start_node(). This file is retained for reference only.

#![allow(deprecated)]

use std::sync::Arc;

use anyhow::Result;

use crate::config::NodeConfig;

pub struct NodeService;

impl NodeService {
    pub async fn init(_config: NodeConfig) -> Result<(
        Arc<crate::p2p::P2PService>,
        Arc<crate::chain::ChainService>,
        Arc<crate::api::ApiService>,
    )> {
        let p2p = Arc::new(crate::p2p::P2PService::new(
            "0.0.0.0:4001".to_string(),
            vec![],
            Arc::new(DummyTxProcessor),
            Arc::new(crate::chain::ChainService::new(
                Arc::new(DummyBlockRepo),
                Arc::new(DummyTxRepo),
                Arc::new(DummyTxProcessor),
                Arc::new(DummyAccountManager),
            )),
        ));

        let chain = Arc::new(crate::chain::ChainService::new(
            Arc::new(DummyBlockRepo),
            Arc::new(DummyTxRepo),
            Arc::new(DummyTxProcessor),
            Arc::new(DummyAccountManager),
        ));

        let api = Arc::new(crate::api::ApiService::new(
            "127.0.0.1:8080".parse()?,
            Arc::new(DummyAccountManager),
            Arc::new(DummyTxProcessor),
            Arc::new(DummyBlockRepo),
            None,
        ));

        Ok((p2p, chain, api))
    }

    pub async fn start(
        _p2p_service: Arc<crate::p2p::P2PService>,
        _chain_service: Arc<crate::chain::ChainService>,
        api_service: Arc<crate::api::ApiService>,
    ) -> Result<()> {
        api_service.start().await?;
        Ok(())
    }
}

struct DummyAccountManager;
impl account::AccountManager for DummyAccountManager {
    fn create_account(&self) -> Result<(crypto::Keypair, account::AccountId, account::Address), account::AccountError> {
        Err(account::AccountError::InvalidOperation("DummyAccountManager: not available".to_string()))
    }

    fn get_account(&self, _account_id: account::AccountId) -> Result<Option<account::Account>, account::AccountError> {
        Err(account::AccountError::InvalidOperation("DummyAccountManager: not available".to_string()))
    }
}

struct DummyBlockRepo;
impl orm::BlockRepository for DummyBlockRepo {
    async fn find_by_id(&self, _id: i64) -> Result<Option<orm::BlockModel>, orm::RepositoryError> {
        Ok(None)
    }

    async fn find_by_height(&self, _height: i64) -> Result<Option<orm::BlockModel>, orm::RepositoryError> {
        Ok(None)
    }

    async fn find_all(&self, _limit: i64, _offset: i64) -> Result<Vec<orm::BlockModel>, orm::RepositoryError> {
        Ok(vec![])
    }

    async fn count(&self) -> Result<i64, orm::RepositoryError> {
        Ok(0)
    }

    async fn insert(&self, _model: orm::BlockModel) -> Result<(), orm::RepositoryError> {
        Ok(())
    }

    async fn update(&self, _model: orm::BlockModel) -> Result<(), orm::RepositoryError> {
        Ok(())
    }
}

struct DummyTxRepo;
impl orm::TransactionRepository for DummyTxRepo {
    async fn find_by_id(&self, _id: i64) -> Result<Option<orm::TransactionModel>, orm::RepositoryError> {
        Ok(None)
    }

    async fn find_by_sender(&self, _sender_id: i64, _limit: i64, _offset: i64) -> Result<Vec<orm::TransactionModel>, orm::RepositoryError> {
        Ok(vec![])
    }

    async fn find_all(&self, _limit: i64, _offset: i64) -> Result<Vec<orm::TransactionModel>, orm::RepositoryError> {
        Ok(vec![])
    }

    async fn count(&self) -> Result<i64, orm::RepositoryError> {
        Ok(0)
    }

    async fn insert(&self, _model: orm::TransactionModel) -> Result<(), orm::RepositoryError> {
        Ok(())
    }
}

struct DummyTxProcessor;
#[async_trait::async_trait]
impl tx_engine::TransactionProcessor for DummyTxProcessor {
    async fn validate(&self, _tx: &blockchain_types::prelude::Transaction) -> tx_engine::ProcessorResult<()> {
        Ok(())
    }

    async fn apply(&self, _tx: &blockchain_types::prelude::Transaction) -> tx_engine::ProcessorResult<()> {
        Ok(())
    }

    async fn execute(&self, _tx: &blockchain_types::prelude::Transaction) -> tx_engine::ProcessorResult<tx_engine::TxReceiptInfo> {
        Ok(tx_engine::TxReceiptInfo {
            transaction_id: 0,
            status: tx_engine::TxStatus::Success,
            block_height: None,
            gas_used: 0,
            logs: vec![],
            contract_address: None,
            executed_at: 0,
        })
    }
}
