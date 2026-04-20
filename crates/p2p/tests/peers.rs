//! Peers manager unit tests.

use p2p::peer::{Peers, compute_backoff_duration};
use std::time::{Duration as StdDuration, Instant};
use chrono::Utc;
use sqlx::PgPool;
use std::env;

// ---------- Backoff Tests ----------

#[test]
fn test_compute_backoff_duration() {
    assert_eq!(compute_backoff_duration(1), StdDuration::from_secs(10));
    assert_eq!(compute_backoff_duration(2), StdDuration::from_secs(20));
    assert_eq!(compute_backoff_duration(3), StdDuration::from_secs(40));
    assert_eq!(compute_backoff_duration(7), StdDuration::from_secs(600)); // capped
}

// ---------- Peer Manager Tests ----------

// Helper to get test pool (requires DATABASE_URL)
async fn get_test_pool() -> Option<PgPool> {
    let db_url = env::var("DATABASE_URL").ok()?;
    Some(
        sqlx::PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .expect("Failed to connect to test DB")
    )
}

// Cleanup helper
async fn cleanup(pool: &PgPool) {
    let _ = sqlx::query("DELETE FROM peers").execute(pool).await;
    let _ = sqlx::query("DELETE FROM accounts").execute(pool).await;
    let _ = sqlx::query("DELETE FROM blocks").execute(pool).await;
    let _ = sqlx::query("DELETE FROM transactions").execute(pool).await;
}

#[tokio::test]
async fn test_find_or_create_peer_creates_new() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Skipping DB test: DATABASE_URL not set");
        return;
    };
    cleanup(&pool).await;

    let peers = Peers::new_for_test(pool.clone());
    let addr = "9.9.9.9:9999".parse().unwrap();
    let peer_arc = peers.find_or_create_peer(addr, false).await.unwrap();

    assert_eq!(peer_arc.lock().await.address, addr);
    // Verify persisted
    let repo = orm::repository::PeerRepository::new();
    let found = repo.find_by_address_port(&pool, "9.9.9.9", 9999).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, peer_arc.lock().await.id);

    cleanup(&pool).await;
}

#[tokio::test]
async fn test_find_or_create_peer_existing() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Skipping DB test: DATABASE_URL not set");
        return;
    };
    cleanup(&pool).await;

    let repo = orm::repository::PeerRepository::new();
    let existing = repo.create(
        &pool,
        orm::models::Peer::new("1.1.1.1:1111".parse().unwrap(), false)
    ).await.unwrap();

    let peers = Peers::new_for_test(pool.clone());
    let found = peers.find_or_create_peer("1.1.1.1:1111".parse().unwrap(), false).await.unwrap();
    assert_eq!(found.id, existing.id);

    cleanup(&pool).await;
}

#[tokio::test]
async fn test_is_blacklisted() {
    let peers = Peers::new_for_test(
        sqlx::PgPoolOptions::new()
            .connect("postgres://user:pass@localhost/db").await.ok()?
    );

    let pid = uuid::Uuid::new_v4();
    let sqlx_pid = sqlx::types::Uuid::from_slice(pid.as_bytes()).unwrap();
    assert!(!peers.is_blacklisted(&sqlx_pid).await);

    // Set cooldown
    {
        let mut inner = peers.inner.write().await;
        let until = Utc::now() + chrono::Duration::minutes(10);
        inner.cooldown_until.insert(sqlx_pid, until);
    }
    assert!(peers.is_blacklisted(&sqlx_pid).await);

    // Expired cooldown
    {
        let mut inner = peers.inner.write().await;
        let past = Utc::now() - chrono::Duration::minutes(1);
        inner.cooldown_until.insert(sqlx_pid, past);
    }
    assert!(!peers.is_blacklisted(&sqlx_pid).await);
}

#[tokio::test]
async fn test_get_known_peers() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Skipping DB test: DATABASE_URL not set");
        return;
    };
    cleanup(&pool).await;

    let repo = orm::repository::PeerRepository::new();
    let p1 = repo.create(&pool, orm::models::Peer::new("1.2.3.4:1234".parse().unwrap(), false)).await.unwrap();
    let p2 = repo.create(&pool, orm::models::Peer::new("5.6.7.8:5678".parse().unwrap(), false)).await.unwrap();

    let peers = Peers::new_for_test(pool.clone());
    let known = peers.get_known_peers().await.unwrap();
    assert_eq!(known.len(), 2);

    cleanup(&pool).await;
}

#[tokio::test]
async fn test_backoff_logic_with_cooldown() {
    // Simulate a peer that fails repeatedly
    let pool = get_test_pool().await.unwrap();
    cleanup(&pool).await;
    let peers = Peers::new_for_test(pool.clone());

    let addr = "1.2.3.4:1234".parse().unwrap();
    let peer = peers.find_or_create_peer(addr, false).await.unwrap();
    {
        let mut inner = peers.inner.write().await;
        inner.cooldown_until.insert(peer.id, Utc::now() + chrono::Duration::minutes(2));
    }

    // Should be blacklisted
    assert!(peers.is_blacklisted(&peer.id).await);
}
