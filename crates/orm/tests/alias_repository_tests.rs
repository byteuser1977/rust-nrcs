//! Alias Repository 集成测试
//!
//! 测试 AliasRepository 的全量操作。

use orm::repository::*;
use orm::models::*;
use sqlx::SqlitePool;
use sqlx::PgPool;

async fn setup() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool failed");
    let schema_sql = include_str!("../../../migrations/sqlite/0.sql");
    for statement in schema_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("--") {
            let _ = sqlx::query(trimmed).execute(&pool).await;
        }
    }
    pool
}

async fn setup_pg() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://nrcs_user:password@localhost:5432/nrcs_db".to_string());
    let pool = PgPool::connect(&database_url).await.expect("pg pool failed");
    let tables = ["alias", "alias_offer", "account", "public_key"];
    for table in &tables {
        let _ = sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table)).execute(&pool).await;
    }
    pool
}

fn make_alias(id: i64, account_id: i64, name: &str, uri: &str, timestamp: i32, height: i32) -> AliasModel {
    AliasModel {
        db_id: 0,
        id,
        account_id,
        alias_name: name.to_string(),
        alias_name_lower: name.to_lowercase(),
        alias_uri: uri.to_string(),
        timestamp,
        height,
        latest: true,
    }
}

#[tokio::test]
async fn test_alias_insert_and_find_by_id() {
    let pool = setup().await;
    let repo = SqliteAliasRepository::new(pool.clone());
    let alias = make_alias(1001, 111, "myalias", "http://example.com", 100, 100);
    repo.insert(&alias).await.expect("insert failed");

    let found = repo.find_by_alias_id(1001).await.expect("find failed");
    assert!(found.is_some());
    let a = found.unwrap();
    assert_eq!(a.id, 1001);
    assert_eq!(a.alias_name, "myalias");
}

#[tokio::test]
async fn test_alias_find_by_name() {
    let pool = setup().await;
    let repo = SqliteAliasRepository::new(pool.clone());
    repo.insert(&make_alias(1001, 111, "MyAlias", "http://example.com", 100, 100)).await.expect("insert failed");

    let found = repo.find_by_name("myalias").await.expect("find failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, 1001);
}

#[tokio::test]
async fn test_alias_find_by_owner() {
    let pool = setup().await;
    let repo = SqliteAliasRepository::new(pool.clone());
    repo.insert(&make_alias(1001, 111, "alias1", "uri1", 100, 100)).await.expect("insert failed");
    repo.insert(&make_alias(1002, 111, "alias2", "uri2", 101, 101)).await.expect("insert failed");
    repo.insert(&make_alias(1003, 222, "alias3", "uri3", 102, 102)).await.expect("insert failed");

    let aliases = repo.find_by_owner(111).await.expect("find failed");
    assert_eq!(aliases.len(), 2);
}

#[tokio::test]
async fn test_alias_update_uri() {
    let pool = setup().await;
    let repo = SqliteAliasRepository::new(pool.clone());
    repo.insert(&make_alias(1001, 111, "myalias", "http://old.com", 100, 100)).await.expect("insert failed");

    repo.update_uri(1001, "http://new.com").await.expect("update failed");
    let found = repo.find_by_alias_id(1001).await.expect("find failed").unwrap();
    assert_eq!(found.alias_uri, "http://new.com");
}

#[tokio::test]
async fn test_alias_update_owner() {
    let pool = setup().await;
    let repo = SqliteAliasRepository::new(pool.clone());
    repo.insert(&make_alias(1001, 111, "myalias", "uri", 100, 100)).await.expect("insert failed");

    repo.update_owner(1001, 222).await.expect("update failed");
    let found = repo.find_by_alias_id(1001).await.expect("find failed").unwrap();
    assert_eq!(found.account_id, 222);
}

#[tokio::test]
async fn test_alias_count() {
    let pool = setup().await;
    let repo = SqliteAliasRepository::new(pool.clone());
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_alias(1, 1, "a", "u", 0, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}

// ==================== PostgreSQL Tests ====================

#[tokio::test]
#[ignore]
async fn pg_test_alias_insert_and_find_by_id() {
    let pool = setup_pg().await;
    let repo = PgAliasRepository::new(pool.clone());
    let alias = make_alias(1001, 111, "myalias", "http://example.com", 100, 100);
    repo.insert(&alias).await.expect("insert failed");

    let found = repo.find_by_alias_id(1001).await.expect("find failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().alias_name, "myalias");
}

#[tokio::test]
#[ignore]
async fn pg_test_alias_count() {
    let pool = setup_pg().await;
    let repo = PgAliasRepository::new(pool.clone());
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_alias(1, 1, "a", "u", 0, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}
