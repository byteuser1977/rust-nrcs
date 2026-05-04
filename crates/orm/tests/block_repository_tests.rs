//! Block Repository 集成测试
//!
//! 测试 BlockRepository 的全量 CRUD 操作和扩展方法。

use orm::repository::*;
use orm::models::*;
use sqlx::SqlitePool;
use sqlx::PgPool;
use blockchain_types::constants::INITIAL_BASE_TARGET;

async fn setup() -> (SqlitePool, SqliteBlockRepository) {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool failed");
    let schema_sql = include_str!("../../../migrations/sqlite/0.sql");
    for statement in schema_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("--") {
            let _ = sqlx::query(trimmed).execute(&pool).await;
        }
    }
    let repo = SqliteBlockRepository::new(pool.clone());
    (pool, repo)
}

async fn setup_pg() -> (PgPool, PgBlockRepository) {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://nrcs_user:password@localhost:5432/nrcs_db".to_string());
    let pool = PgPool::connect(&database_url).await.expect("pg pool failed");

    let tables = ["block", "transaction", "account", "public_key"];
    for table in &tables {
        let _ = sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table)).execute(&pool).await;
    }

    let repo = PgBlockRepository::new(pool.clone());
    (pool, repo)
}

fn make_block(id: i64, height: i32, timestamp: i32, generator_id: i64) -> BlockModel {
    BlockModel {
        db_id: 0,
        id,
        version: 3,
        timestamp,
        previous_block_id: None,
        total_amount: 0,
        total_fee: 0,
        payload_length: 0,
        previous_block_hash: None,
        cumulative_difficulty: vec![0u8; 1],
        base_target: INITIAL_BASE_TARGET as i64,
        next_block_id: None,
        height,
        generation_signature: vec![0u8; 64],
        block_signature: vec![0u8; 64],
        payload_hash: vec![0u8; 32],
        generator_id,
    }
}

#[tokio::test]
async fn test_insert_and_find_by_height() {
    let (_pool, repo) = setup().await;
    let block = make_block(100, 0, 0, 12345);
    repo.insert(&block).await.expect("insert failed");

    let found = repo.find_by_height(0).await.expect("find failed");
    assert!(found.is_some());
    let b = found.unwrap();
    assert_eq!(b.id, 100);
    assert_eq!(b.height, 0);
    assert_eq!(b.generator_id, 12345);
}

#[tokio::test]
async fn test_find_latest() {
    let (_pool, repo) = setup().await;
    repo.insert(&make_block(100, 0, 0, 1)).await.expect("insert failed");
    repo.insert(&make_block(200, 1, 60, 2)).await.expect("insert failed");
    repo.insert(&make_block(300, 2, 120, 3)).await.expect("insert failed");

    let latest = repo.find_latest().await.expect("find_latest failed");
    assert!(latest.is_some());
    assert_eq!(latest.unwrap().height, 2);
}

#[tokio::test]
async fn test_find_by_id_column() {
    let (_pool, repo) = setup().await;
    repo.insert(&make_block(12345, 0, 0, 1)).await.expect("insert failed");

    let found = repo.find_by_id_column(12345).await.expect("find failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, 12345);
}

#[tokio::test]
async fn test_count() {
    let (_pool, repo) = setup().await;
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_block(100, 0, 0, 1)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
    repo.insert(&make_block(200, 1, 60, 2)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 2);
}

#[tokio::test]
async fn test_has_block() {
    let (_pool, repo) = setup().await;
    repo.insert(&make_block(100, 0, 0, 1)).await.expect("insert failed");
    assert!(repo.has_block(100).await.expect("has_block failed"));
    assert!(!repo.has_block(999).await.expect("has_block failed"));
}

#[tokio::test]
async fn test_get_height() {
    let (_pool, repo) = setup().await;
    repo.insert(&make_block(100, 0, 0, 1)).await.expect("insert failed");
    repo.insert(&make_block(200, 5, 300, 2)).await.expect("insert failed");
    let height = repo.get_height().await.expect("get_height failed");
    assert_eq!(height, 5);
}

#[tokio::test]
async fn test_find_all() {
    let (_pool, repo) = setup().await;
    repo.insert(&make_block(100, 0, 0, 1)).await.expect("insert failed");
    repo.insert(&make_block(200, 1, 60, 2)).await.expect("insert failed");

    let all = repo.find_all(None, None).await.expect("find_all failed");
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn test_find_range() {
    let (_pool, repo) = setup().await;
    for i in 0..10 {
        repo.insert(&make_block(100 + i as i64, i, i * 60, 1)).await.expect("insert failed");
    }

    let range = repo.find_range(3, 7).await.expect("find_range failed");
    assert_eq!(range.len(), 5);
    for b in &range {
        assert!(b.height >= 3 && b.height <= 7);
    }
}

#[tokio::test]
async fn test_update_next_block_id() {
    let (_pool, repo) = setup().await;
    repo.insert(&make_block(100, 0, 0, 1)).await.expect("insert failed");
    repo.insert(&make_block(200, 1, 60, 2)).await.expect("insert failed");

    repo.update_next_block_id(100, 200).await.expect("update failed");
    let block = repo.find_by_id_column(100).await.expect("find failed").unwrap();
    assert_eq!(block.next_block_id, Some(200));
}

#[tokio::test]
async fn test_find_by_generator() {
    let (_pool, repo) = setup().await;
    repo.insert(&make_block(100, 0, 0, 111)).await.expect("insert failed");
    repo.insert(&make_block(200, 1, 60, 222)).await.expect("insert failed");
    repo.insert(&make_block(300, 2, 120, 111)).await.expect("insert failed");

    let blocks = repo.find_by_generator(111).await.expect("find_by_generator failed");
    assert_eq!(blocks.len(), 2);
}

// ==================== PostgreSQL Tests ====================

#[tokio::test]
#[ignore]
async fn pg_test_insert_and_find_by_height() {
    let (_pool, repo) = setup_pg().await;
    let block = make_block(100, 0, 0, 12345);
    repo.insert(&block).await.expect("insert failed");

    let found = repo.find_by_height(0).await.expect("find failed");
    assert!(found.is_some());
    let b = found.unwrap();
    assert_eq!(b.id, 100);
    assert_eq!(b.height, 0);
    assert_eq!(b.generator_id, 12345);
}

#[tokio::test]
#[ignore]
async fn pg_test_find_latest() {
    let (_pool, repo) = setup_pg().await;
    repo.insert(&make_block(100, 0, 0, 1)).await.expect("insert failed");
    repo.insert(&make_block(200, 1, 60, 2)).await.expect("insert failed");
    repo.insert(&make_block(300, 2, 120, 3)).await.expect("insert failed");

    let latest = repo.find_latest().await.expect("find_latest failed");
    assert!(latest.is_some());
    assert_eq!(latest.unwrap().height, 2);
}

#[tokio::test]
#[ignore]
async fn pg_test_find_by_id_column() {
    let (_pool, repo) = setup_pg().await;
    repo.insert(&make_block(12345, 0, 0, 1)).await.expect("insert failed");

    let found = repo.find_by_id_column(12345).await.expect("find failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, 12345);
}

#[tokio::test]
#[ignore]
async fn pg_test_count() {
    let (_pool, repo) = setup_pg().await;
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_block(100, 0, 0, 1)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}

#[tokio::test]
#[ignore]
async fn pg_test_has_block() {
    let (_pool, repo) = setup_pg().await;
    repo.insert(&make_block(100, 0, 0, 1)).await.expect("insert failed");
    assert!(repo.has_block(100).await.expect("has_block failed"));
    assert!(!repo.has_block(999).await.expect("has_block failed"));
}

#[tokio::test]
#[ignore]
async fn pg_test_get_height() {
    let (_pool, repo) = setup_pg().await;
    repo.insert(&make_block(100, 0, 0, 1)).await.expect("insert failed");
    repo.insert(&make_block(200, 5, 300, 2)).await.expect("insert failed");
    let height = repo.get_height().await.expect("get_height failed");
    assert_eq!(height, 5);
}

#[tokio::test]
#[ignore]
async fn pg_test_find_all() {
    let (_pool, repo) = setup_pg().await;
    repo.insert(&make_block(100, 0, 0, 1)).await.expect("insert failed");
    repo.insert(&make_block(200, 1, 60, 2)).await.expect("insert failed");

    let all = repo.find_all(None, None).await.expect("find_all failed");
    assert_eq!(all.len(), 2);
}

#[tokio::test]
#[ignore]
async fn pg_test_find_range() {
    let (_pool, repo) = setup_pg().await;
    for i in 0..10 {
        repo.insert(&make_block(100 + i as i64, i, i * 60, 1)).await.expect("insert failed");
    }

    let range = repo.find_range(3, 7).await.expect("find_range failed");
    assert_eq!(range.len(), 5);
}

#[tokio::test]
#[ignore]
async fn pg_test_update_next_block_id() {
    let (_pool, repo) = setup_pg().await;
    repo.insert(&make_block(100, 0, 0, 1)).await.expect("insert failed");
    repo.insert(&make_block(200, 1, 60, 2)).await.expect("insert failed");

    repo.update_next_block_id(100, 200).await.expect("update failed");
    let block = repo.find_by_id_column(100).await.expect("find failed").unwrap();
    assert_eq!(block.next_block_id, Some(200));
}

#[tokio::test]
#[ignore]
async fn pg_test_find_by_generator() {
    let (_pool, repo) = setup_pg().await;
    repo.insert(&make_block(100, 0, 0, 111)).await.expect("insert failed");
    repo.insert(&make_block(200, 1, 60, 222)).await.expect("insert failed");
    repo.insert(&make_block(300, 2, 120, 111)).await.expect("insert failed");

    let blocks = repo.find_by_generator(111).await.expect("find_by_generator failed");
    assert_eq!(blocks.len(), 2);
}
