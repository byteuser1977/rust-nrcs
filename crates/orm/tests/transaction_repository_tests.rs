//! Transaction Repository 集成测试
//!
//! 测试 TransactionRepository 的全量 CRUD 操作和扩展方法。

use orm::repository::*;
use orm::models::*;
use sqlx::SqlitePool;
use sqlx::PgPool;
use blockchain_types::constants::INITIAL_BASE_TARGET;

async fn setup() -> (SqlitePool, SqliteTransactionRepository, SqliteBlockRepository) {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool failed");
    let schema_sql = include_str!("../../../migrations/sqlite/0.sql");
    for statement in schema_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("--") {
            let _ = sqlx::query(trimmed).execute(&pool).await;
        }
    }
    let tx_repo = SqliteTransactionRepository::new(pool.clone());
    let block_repo = SqliteBlockRepository::new(pool.clone());

    block_repo.insert(&BlockModel {
        db_id: 0,
        id: 100,
        version: 3,
        timestamp: 0,
        previous_block_id: None,
        total_amount: 0,
        total_fee: 0,
        payload_length: 0,
        previous_block_hash: None,
        cumulative_difficulty: vec![0u8; 1],
        base_target: 1000,
        next_block_id: None,
        height: 0,
        generation_signature: vec![0u8; 64],
        block_signature: vec![0u8; 64],
        payload_hash: vec![0u8; 32],
        generator_id: 1,
    }).await.expect("block insert failed");

    (pool, tx_repo, block_repo)
}

async fn setup_pg() -> (PgPool, PgTransactionRepository, PgBlockRepository) {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://nrcs_user:password@localhost:5432/nrcs_db".to_string());
    let pool = PgPool::connect(&database_url).await.expect("pg pool failed");
    let tables = ["transaction", "block", "account", "public_key"];
    for table in &tables {
        let _ = sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table)).execute(&pool).await;
    }

    let tx_repo = PgTransactionRepository::new(pool.clone());
    let block_repo = PgBlockRepository::new(pool.clone());

    block_repo.insert(&BlockModel {
        db_id: 0,
        id: 100,
        version: 3,
        timestamp: 0,
        previous_block_id: None,
        total_amount: 0,
        total_fee: 0,
        payload_length: 0,
        previous_block_hash: None,
        cumulative_difficulty: vec![0u8; 1],
        base_target: INITIAL_BASE_TARGET as i64,
        next_block_id: None,
        height: 0,
        generation_signature: vec![0u8; 64],
        block_signature: vec![0u8; 64],
        payload_hash: vec![0u8; 32],
        generator_id: 1,
    }).await.expect("block insert failed");

    (pool, tx_repo, block_repo)
}

fn make_transaction(id: i64, sender_id: i64, recipient_id: i64, amount: i64, fee: i64, height: i32) -> TransactionModel {
    TransactionModel {
        db_id: 0,
        id,
        deadline: 1440,
        sender_id,
        recipient_id: Some(recipient_id),
        amount,
        fee,
        height,
        block_id: 100,
        block_timestamp: 0,
        transaction_index: 0,
        signature: vec![0u8; 64],
        full_hash: vec![0u8; 32],
        r#type: 0,
        subtype: 0,
        timestamp: 0,
        version: 3,
        attachment_bytes: None,
        referenced_transaction_full_hash: None,
        has_message: false,
        has_encrypted_message: false,
        has_public_key_announcement: false,
        has_prunable_message: false,
        has_prunable_attachment: false,
        phased: false,
        ec_block_height: None,
        ec_block_id: None,
        has_encrypttoself_message: false,
        has_prunable_encrypted_message: false,
    }
}

#[tokio::test]
async fn test_insert_and_find_by_sender() {
    let (_pool, repo, _) = setup().await;
    let tx = make_transaction(1001, 111, 222, 1000, 10, 0);
    repo.insert(&tx).await.expect("insert failed");

    let found = repo.find_by_sender(111, 100).await.expect("find_by_sender failed");
    assert!(!found.is_empty());
    let t = &found[0];
    assert_eq!(t.id, 1001);
    assert_eq!(t.sender_id, 111);
    assert_eq!(t.amount, 1000);
}

#[tokio::test]
async fn test_insert_and_find_by_recipient() {
    let (_pool, repo, _) = setup().await;
    let tx = make_transaction(1001, 111, 222, 1000, 10, 0);
    repo.insert(&tx).await.expect("insert failed");

    let found = repo.find_by_recipient(222, 100).await.expect("find_by_recipient failed");
    assert!(!found.is_empty());
    assert_eq!(found[0].recipient_id, Some(222));
}

#[tokio::test]
async fn test_count() {
    let (_pool, repo, _) = setup().await;
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_transaction(1, 1, 2, 100, 10, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}

#[tokio::test]
async fn test_multiple_transactions() {
    let (_pool, repo, _) = setup().await;
    for i in 1..=5 {
        repo.insert(&make_transaction(i, 1, 2, i * 100, 10, 0)).await.expect("insert failed");
    }
    assert_eq!(repo.count().await.expect("count failed"), 5);

    let by_sender = repo.find_by_sender(1, 100).await.expect("find failed");
    assert_eq!(by_sender.len(), 5);
}

// ==================== PostgreSQL Tests ====================

#[tokio::test]
#[ignore]
async fn pg_test_insert_and_find_by_sender() {
    let (_pool, repo, _) = setup_pg().await;
    let tx = make_transaction(1001, 111, 222, 1000, 10, 0);
    repo.insert(&tx).await.expect("insert failed");

    let found = repo.find_by_sender(111, 100).await.expect("find_by_sender failed");
    assert!(!found.is_empty());
    assert_eq!(found[0].id, 1001);
}

#[tokio::test]
#[ignore]
async fn pg_test_insert_and_find_by_recipient() {
    let (_pool, repo, _) = setup_pg().await;
    let tx = make_transaction(1001, 111, 222, 1000, 10, 0);
    repo.insert(&tx).await.expect("insert failed");

    let found = repo.find_by_recipient(222, 100).await.expect("find_by_recipient failed");
    assert!(!found.is_empty());
    assert_eq!(found[0].recipient_id, Some(222));
}

#[tokio::test]
#[ignore]
async fn pg_test_count() {
    let (_pool, repo, _) = setup_pg().await;
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_transaction(1, 1, 2, 100, 10, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}
