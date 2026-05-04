//! Poll 和 Vote Repository 集成测试
//!
//! 测试 PollRepository 和 VoteRepository 的全量操作。

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
    let tables = ["poll", "vote", "poll_result", "account", "public_key"];
    for table in &tables {
        let _ = sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table)).execute(&pool).await;
    }
    pool
}

fn make_poll(id: i64, account_id: i64, name: &str, options: &str, finish_height: i32, voting_model: i16, height: i32) -> PollModel {
    PollModel {
        db_id: 0,
        id,
        account_id,
        name: name.to_string(),
        description: None,
        options: options.to_string(),
        min_num_options: Some(1),
        max_num_options: Some(3),
        min_range_value: Some(0),
        max_range_value: Some(100),
        timestamp: 0,
        finish_height,
        voting_model,
        min_balance: None,
        min_balance_model: None,
        holding_id: None,
        height,
    }
}

fn make_vote(id: i64, poll_id: i64, voter_id: i64, vote_bytes: Vec<u8>, height: i32) -> VoteModel {
    VoteModel {
        db_id: 0,
        id,
        poll_id,
        voter_id,
        vote_bytes,
        height,
    }
}

#[tokio::test]
async fn test_poll_insert_and_find_by_id() {
    let pool = setup().await;
    let repo = SqlitePollRepository::new(pool.clone());
    let poll = make_poll(1001, 111, "TestPoll", "[\"Yes\",\"No\"]", 1000, 0, 100);
    repo.insert(&poll).await.expect("insert failed");

    let found = repo.find_by_poll_id(1001).await.expect("find failed");
    assert!(found.is_some());
    let p = found.unwrap();
    assert_eq!(p.id, 1001);
    assert_eq!(p.name, "TestPoll");
    assert_eq!(p.voting_model, 0);
}

#[tokio::test]
async fn test_poll_find_by_account() {
    let pool = setup().await;
    let repo = SqlitePollRepository::new(pool.clone());
    repo.insert(&make_poll(1001, 111, "Poll1", "[\"A\",\"B\"]", 1000, 0, 100)).await.expect("insert failed");
    repo.insert(&make_poll(1002, 111, "Poll2", "[\"C\",\"D\"]", 1001, 1, 101)).await.expect("insert failed");
    repo.insert(&make_poll(1003, 222, "Poll3", "[\"E\",\"F\"]", 1002, 0, 102)).await.expect("insert failed");

    let polls = repo.find_by_account(111).await.expect("find failed");
    assert_eq!(polls.len(), 2);
}

#[tokio::test]
async fn test_poll_count() {
    let pool = setup().await;
    let repo = SqlitePollRepository::new(pool.clone());
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_poll(1, 1, "P", "[\"A\"]", 100, 0, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}

#[tokio::test]
async fn test_vote_insert_and_find_by_poll() {
    let pool = setup().await;
    let repo = SqliteVoteRepository::new(pool.clone());
    repo.insert(&make_vote(1, 1001, 111, vec![1, 0], 100)).await.expect("insert failed");
    repo.insert(&make_vote(2, 1001, 222, vec![0, 1], 101)).await.expect("insert failed");
    repo.insert(&make_vote(3, 1002, 111, vec![1], 102)).await.expect("insert failed");

    let votes = repo.find_by_poll(1001, 100).await.expect("find failed");
    assert_eq!(votes.len(), 2);
}

#[tokio::test]
async fn test_vote_find_by_voter() {
    let pool = setup().await;
    let repo = SqliteVoteRepository::new(pool.clone());
    repo.insert(&make_vote(1, 1001, 111, vec![1, 0], 100)).await.expect("insert failed");
    repo.insert(&make_vote(2, 1002, 111, vec![1], 101)).await.expect("insert failed");
    repo.insert(&make_vote(3, 1001, 222, vec![0, 1], 102)).await.expect("insert failed");

    let votes = repo.find_by_voter(111, 100).await.expect("find failed");
    assert_eq!(votes.len(), 2);
}

#[tokio::test]
async fn test_vote_find_by_poll_and_voter() {
    let pool = setup().await;
    let repo = SqliteVoteRepository::new(pool.clone());
    repo.insert(&make_vote(1, 1001, 111, vec![1, 0], 100)).await.expect("insert failed");

    let found = repo.find_by_poll_and_voter(1001, 111).await.expect("find failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().vote_bytes, vec![1, 0]);

    let not_found = repo.find_by_poll_and_voter(1001, 999).await.expect("find failed");
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_vote_count() {
    let pool = setup().await;
    let repo = SqliteVoteRepository::new(pool.clone());
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_vote(1, 1, 1, vec![1], 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}

// ==================== PostgreSQL Tests ====================

#[tokio::test]
#[ignore]
async fn pg_test_poll_insert_and_find_by_id() {
    let pool = setup_pg().await;
    let repo = PgPollRepository::new(pool.clone());
    let poll = make_poll(1001, 111, "TestPoll", "[\"Yes\",\"No\"]", 1000, 0, 100);
    repo.insert(&poll).await.expect("insert failed");

    let found = repo.find_by_poll_id(1001).await.expect("find failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "TestPoll");
}

#[tokio::test]
#[ignore]
async fn pg_test_poll_count() {
    let pool = setup_pg().await;
    let repo = PgPollRepository::new(pool.clone());
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_poll(1, 1, "P", "[\"A\"]", 100, 0, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}

#[tokio::test]
#[ignore]
async fn pg_test_vote_insert_and_find_by_poll() {
    let pool = setup_pg().await;
    let repo = PgVoteRepository::new(pool.clone());
    repo.insert(&make_vote(1, 1001, 111, vec![1, 0], 100)).await.expect("insert failed");
    repo.insert(&make_vote(2, 1001, 222, vec![0, 1], 101)).await.expect("insert failed");

    let votes = repo.find_by_poll(1001, 100).await.expect("find failed");
    assert_eq!(votes.len(), 2);
}
