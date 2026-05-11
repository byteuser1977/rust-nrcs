//! Test database pool utilities
//!
//! Provides helper functions for creating test database connections.
//! Tests that require a database will skip gracefully if no test database is available.

use sqlx::any::AnyPool;
use std::env;

pub async fn create_test_pool() -> Option<AnyPool> {
    let database_url = env::var("TEST_DATABASE_URL").ok()?;
    let pool = sqlx::any::AnyPool::connect(&database_url).await.ok()?;
    Some(pool)
}
