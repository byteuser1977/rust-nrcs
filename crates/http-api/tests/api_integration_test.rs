//! HTTP API 集成测试
//!
//! 测试已实现 API 端点的请求/响应格式和状态码

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_api::{routes::create_router, state::ApiState};
use blockchain_types::prelude::*;
use orm::{BlockRepository, RepositoryResult, BlockModel};
use account::AccountManager;
use tx_engine::TransactionProcessor;
use async_trait::async_trait;
use std::sync::Arc;

/// 模拟账户管理器
#[derive(Clone)]
struct MockAccountManager;

#[async_trait]
impl AccountManager for MockAccountManager {
    async fn create_account(&self, _initial_balance: Option<Amount>) -> account::AccountResult<(crypto::KeyPair, AccountId, String)> {
        let kp = crypto::generate_keypair();
        let account_id = 1;
        let address = "test_address".to_string();
        Ok((kp, account_id, address))
    }
    
    async fn register_account(&self, _account_id: AccountId, _public_key: Vec<u8>) -> account::AccountResult<()> {
        Ok(())
    }
    
    async fn get_balance(&self, _account_id: AccountId) -> account::AccountResult<Amount> {
        Ok(1000)
    }
    
    async fn get_account_info(&self, account_id: AccountId) -> account::AccountResult<Account> {
        Ok(Account {
            id: account_id,
            address: Some("test_address".to_string()),
            balance: 1000,
            unconfirmed_balance: 1000,
            reserved_balance: 0,
            guaranteed_balance: 0,
            assets: Default::default(),
            properties: Default::default(),
            lease: None,
            created_at: 0,
            last_updated: 0,
            current_height: 0,
        })
    }
    
    async fn transfer(&self, _from: AccountId, _to: AccountId, _amount: Amount) -> account::AccountResult<()> {
        Ok(())
    }
    
    async fn credit(&self, _account_id: AccountId, _amount: Amount) -> account::AccountResult<()> {
        Ok(())
    }
    
    async fn debit(&self, _account_id: AccountId, _amount: Amount) -> account::AccountResult<()> {
        Ok(())
    }
    
    async fn get_and_increment_nonce(&self, _sender_id: AccountId) -> account::AccountResult<u64> {
        Ok(0)
    }
    
    async fn current_nonce(&self, _account_id: AccountId) -> account::AccountResult<u64> {
        Ok(0)
    }
    
    async fn mint_asset(&self, _asset_id: AssetId, _to: AccountId, _amount: Amount) -> account::AccountResult<()> {
        Ok(())
    }
    
    async fn burn_asset(&self, _asset_id: AssetId, _from: AccountId, _amount: Amount) -> account::AccountResult<()> {
        Ok(())
    }
}

/// 模拟交易处理器
#[derive(Clone)]
struct MockTxProcessor;

#[async_trait]
impl TransactionProcessor for MockTxProcessor {
    async fn validate(&self, _tx: &Transaction) -> tx_engine::ProcessorResult<()> {
        Ok(())
    }
    
    async fn apply(&self, _tx: &Transaction) -> tx_engine::ProcessorResult<()> {
        Ok(())
    }
    
    async fn execute(&self, _tx: &Transaction) -> tx_engine::ProcessorResult<tx_engine::TxReceiptInfo> {
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
    
    async fn validate_batch(&self, _txs: &[Transaction]) -> tx_engine::ProcessorResult<()> {
        Ok(())
    }
    
    async fn execute_batch(&self, _txs: &[Transaction]) -> tx_engine::ProcessorResult<Vec<tx_engine::TxReceiptInfo>> {
        Ok(vec![])
    }
}

/// 模拟区块仓库
#[derive(Clone)]
struct MockBlockRepository;

#[async_trait]
impl BlockRepository for MockBlockRepository {
    async fn find_latest(&self) -> RepositoryResult<Option<BlockModel>> {
        Ok(Some(BlockModel {
            db_id: 0,
            id: 1,
            version: 1,
            timestamp: 1000,
            previous_block_id: None,
            total_amount: 0,
            total_fee: 0,
            payload_length: 0,
            previous_block_hash: Some(vec![0; 32]),
            cumulative_difficulty: vec![0; 32],
            base_target: 0,
            next_block_id: None,
            height: 1,
            generation_signature: vec![0; 64],
            block_signature: vec![0; 64],
            payload_hash: vec![0; 32],
            generator_id: 1,
        }))
    }
    
    async fn find_by_height(&self, _height: i32) -> RepositoryResult<Option<BlockModel>> {
        Ok(None)
    }
    
    async fn find_by_hash(&self, _hash: &[u8]) -> RepositoryResult<Option<BlockModel>> {
        Ok(None)
    }
    
    async fn find_by_id_column(&self, _id: i64) -> RepositoryResult<Option<BlockModel>> {
        Ok(None)
    }
    
    async fn find_range(&self, _start_height: i32, _end_height: i32) -> RepositoryResult<Vec<BlockModel>> {
        Ok(vec![])
    }
    
    async fn find_by_generator(&self, _generator_id: i64) -> RepositoryResult<Vec<BlockModel>> {
        Ok(vec![])
    }
}

/// 实现 Repository trait，仅保留必需的方法
#[async_trait]
impl orm::Repository<BlockModel> for MockBlockRepository {
    async fn insert(&self, _item: &BlockModel) -> RepositoryResult<()> {
        Ok(())
    }
    async fn find_by_id(&self, _db_id: i64) -> RepositoryResult<Option<BlockModel>> {
        Ok(None)
    }
    async fn update(&self, _item: &BlockModel) -> RepositoryResult<()> {
        Ok(())
    }
    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Ok(())
    }
    async fn find_all(&self, _limit: Option<i64>, _offset: Option<i64>) -> RepositoryResult<Vec<BlockModel>> {
        Ok(vec![])
    }
    async fn count(&self) -> RepositoryResult<i64> {
        Ok(0)
    }
}

/// 创建测试用的 ApiState
fn create_test_state() -> ApiState {
    let account_manager = Arc::new(MockAccountManager) as Arc<dyn AccountManager>;
    let tx_processor = Arc::new(MockTxProcessor) as Arc<dyn TransactionProcessor>;
    let block_repo = Arc::new(MockBlockRepository) as Arc<dyn BlockRepository>;
    
    ApiState {
        account_manager,
        tx_processor,
        block_repo,
    }
}

#[tokio::test]
async fn test_health_check() {
    use tower::util::ServiceExt;
    
    let state = create_test_state();
    let router = create_router(state);
    
    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_account() {
    use tower::util::ServiceExt;
    use serde_json::json;
    
    let state = create_test_state();
    let router = create_router(state);
    
    let request = Request::builder()
        .uri("/api/v1/accounts")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(json!({ "initial_balance": 100 }).to_string()))
        .unwrap();
    
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_account() {
    use tower::util::ServiceExt;
    
    let state = create_test_state();
    let router = create_router(state);
    
    let request = Request::builder()
        .uri("/api/v1/accounts/1")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_balance() {
    use tower::util::ServiceExt;
    
    let state = create_test_state();
    let router = create_router(state);
    
    let request = Request::builder()
        .uri("/api/v1/accounts/1/balance")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_latest_block() {
    use tower::util::ServiceExt;
    
    let state = create_test_state();
    let router = create_router(state);
    
    let request = Request::builder()
        .uri("/api/v1/blocks/latest")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_metrics() {
    use tower::util::ServiceExt;
    
    let state = create_test_state();
    let router = create_router(state);
    
    let request = Request::builder()
        .uri("/metrics")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_endpoint() {
    use tower::util::ServiceExt;
    
    let state = create_test_state();
    let router = create_router(state);
    
    let request = Request::builder()
        .uri("/api/v1/invalid")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
