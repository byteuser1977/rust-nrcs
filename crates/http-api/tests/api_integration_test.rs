//! HTTP API 集成测试
//!
//! 测试所有 API 端点的请求/响应格式、状态码和业务逻辑

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use http_api::{handlers, state::ApiState};
use blockchain_types::*;
use std::sync::Arc;
use std::net::SocketAddr;

/// 模拟账户管理器
#[derive(Clone)]
struct MockAccountManager;

impl crate::account::AccountManager for MockAccountManager {
    async fn create_account(&self, pubkey: [u8; 32]) -> anyhow::Result<AccountId> {
        Ok(1)
    }
    
    async fn get_account(&self, id: AccountId) -> anyhow::Result<Account> {
        Ok(Account {
            id,
            pubkey: [1u8; 32],
            balance: 1000,
            nonce: 0,
        })
    }
    
    async fn get_balance(&self, id: AccountId) -> anyhow::Result<u64> {
        Ok(1000)
    }
    
    async fn transfer(&self, tx: Transaction) -> anyhow::Result<()> {
        Ok(())
    }
}

/// 模拟交易处理器
#[derive(Clone)]
struct MockTxProcessor;

impl tx_engine::TransactionProcessor for MockTxProcessor {
    async fn submit_transaction(&self, tx: SignedTransaction) -> anyhow::Result<()> {
        Ok(())
    }
    
    async fn get_transaction(&self, hash: Hash256) -> anyhow::Result<Option<SignedTransaction>> {
        Ok(None)
    }
    
    async fn list_pending_transactions(&self) -> anyhow::Result<Vec<SignedTransaction>> {
        Ok(vec![])
    }
}

/// 模拟区块仓库
#[derive(Clone)]
struct MockBlockRepository;

impl orm::BlockRepository for MockBlockRepository {
    async fn get_latest_block(&self) -> anyhow::Result<Block> {
        Ok(Block::new(1, [0u8; 32], 1000, vec![]))
    }
    
    async fn get_block_by_height(&self, height: Height) -> anyhow::Result<Option<Block>> {
        Ok(None)
    }
    
    async fn get_block_by_hash(&self, hash: Hash256) -> anyhow::Result<Option<Block>> {
        Ok(None)
    }
    
    async fn list_blocks(&self, limit: usize, offset: usize) -> anyhow::Result<Vec<Block>> {
        Ok(vec![])
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
        p2p_service: None,
    }
}

/// 辅助函数：发送请求
async fn send_request(
    addr: SocketAddr,
    method: &str,
    path: &str,
    body: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    let state = create_test_state();
    let router = handlers::create_router(state);
    
    let mut request = Request::builder()
        .uri(path)
        .method(method);
    
    if let Some(body) = body {
        request = request.body(Body::from(body.to_string()));
    } else {
        request = request.body(Body::empty());
    }
    
    let request = request.build().unwrap();
    
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    
    let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({}));
    
    (status, body)
}

#[tokio::test]
async fn test_health_check() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let (status, body) = send_request(addr, "GET", "/health", None).await;
    
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_create_account() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let pubkey = [1u8; 32];
    let body = serde_json::json!({ "pubkey": format!("{:x}", hex::encode(pubkey)) });
    
    let (status, response) = send_request(addr, "POST", "/api/v1/accounts", Some(body)).await;
    
    assert_eq!(status, StatusCode::OK);
    assert!(response.get("address").is_some());
}

#[tokio::test]
async fn test_get_account() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let (status, body) = send_request(addr, "GET", "/api/v1/accounts/1", None).await;
    
    assert_eq!(status, StatusCode::OK);
    // TODO: 根据实际 Response 结构断言
}

#[tokio::test]
async fn test_get_balance() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let (status, body) = send_request(addr, "GET", "/api/v1/accounts/1/balance", None).await;
    
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_submit_transaction() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let tx = Transaction::new_transfer(1, 2, 100, 1);
    let body = serde_json::json!({
        "from": 1,
        "to": 2,
        "amount": 100,
        "nonce": 1,
        "gas_price": 1,
        "gas_limit": 1000,
        "signature": "01".repeat(64)
    });
    
    let (status, response) = send_request(addr, "POST", "/api/v1/transactions", Some(body)).await;
    
    assert_eq!(status, StatusCode::OK);
    assert!(response.get("hash").is_some());
}

#[tokio::test]
async fn test_get_latest_block() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let (status, body) = send_request(addr, "GET", "/api/v1/blocks/latest", None).await;
    
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_get_node_info() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let (status, body) = send_request(addr, "GET", "/api/v1/node/info", None).await;
    
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_deploy_contract() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let body = serde_json::json!({
        "owner": 1,
        "wasm_bytes": "00",
        "init_method": "init",
        "args": []
    });
    
    let (status, response) = send_request(addr, "POST", "/api/v1/contracts/deploy", Some(body)).await;
    
    assert_eq!(status, StatusCode::OK);
    assert!(response.get("contract_id").is_some());
}

#[tokio::test]
async fn test_call_contract() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let body = serde_json::json!({
        "method": "transfer",
        "args": [],
        "gas_limit": 10000
    });
    
    let (status, response) = send_request(addr, "POST", "/api/v1/contracts/1/call", Some(body)).await;
    
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_get_contract() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let (status, body) = send_request(addr, "GET", "/api/v1/contracts/1", None).await;
    
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_list_peers() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let (status, body) = send_request(addr, "GET", "/api/v1/node/peers", None).await;
    
    assert_eq!(status, StatusCode::OK);
    assert!(body.as_array().is_some());
}

#[tokio::test]
async fn test_metrics() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let (status, _body) = send_request(addr, "GET", "/metrics", None).await;
    
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_endpoint() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let (status, _body) = send_request(addr, "GET", "/api/v1/invalid", None).await;
    
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_malformed_json() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let mut request = Request::builder()
        .uri("/api/v1/accounts")
        .method("POST")
        .body(Body::from("invalid json")).unwrap();
    
    let state = create_test_state();
    let router = handlers::create_router(state);
    let response = router.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}