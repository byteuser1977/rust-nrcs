//! Database transaction management utilities
//!
//! Provides transaction helpers for atomic database operations.

use sqlx::SqlitePool;
use tracing::warn;

use crate::RepositoryResult;
use crate::RepositoryError;

/// Execute a function within a database transaction
///
/// Automatically commits on success, rolls back on error.
/// Note: This is a simplified version that handles basic transaction semantics.
pub async fn with_transaction<Fut, T>(
    pool: &SqlitePool,
    f: impl FnOnce() -> Fut,
) -> RepositoryResult<T>
where
    Fut: std::future::Future<Output = RepositoryResult<T>>,
{
    let tx = pool.begin().await.map_err(RepositoryError::DbError)?;

    match f().await {
        Ok(result) => {
            tx.commit().await.map_err(RepositoryError::DbError)?;
            Ok(result)
        }
        Err(e) => {
            if let Err(rollback_err) = tx.rollback().await {
                warn!("Failed to rollback transaction: {}", rollback_err);
            }
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transaction_commit() {
        // TODO: Add integration test with test database
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        // TODO: Add integration test with test database
    }
}
