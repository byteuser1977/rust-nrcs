//! Public key repository for account public keys.

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{RepositoryResult, RepositoryError};
use blockchain_types::account_ext::AccountPublicKey;
use blockchain_types::prelude::*;

/// Repository for account public keys.
#[async_trait]
pub trait PublicKeyRepository: Send + Sync {
    /// Get the latest public key for an account.
    async fn find_latest_by_account_id(&self, account_id: i64) -> RepositoryResult<Option<AccountPublicKey>>;

    /// Insert a new public key record.
    async fn insert(&self, pk: &AccountPublicKey) -> RepositoryResult<()>;
}

/// PostgreSQL implementation of PublicKeyRepository.
pub struct PgPublicKeyRepository {
    pool: PgPool,
}

impl PgPublicKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PublicKeyRepository for PgPublicKeyRepository {
    async fn find_latest_by_account_id(&self, account_id: i64) -> RepositoryResult<Option<AccountPublicKey>> {
        // The PUBLIC_KEY table has columns: account_id, public_key (bytea), height, latest.
        let row: Option<(Vec<u8>, i32)> = sqlx::query_as(
            "SELECT public_key, height FROM public_key WHERE account_id = $1 AND latest = TRUE ORDER BY height DESC LIMIT 1"
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((pk_vec, height)) = row {
            if let Ok(bytes) = pk_vec.try_into() {
                let pk = AccountPublicKey {
                    account_id: account_id as AccountId,
                    public_key: bytes,
                    height: height as Height,
                };
                return Ok(Some(pk));
            }
        }
        Ok(None)
    }

    async fn insert(&self, pk: &AccountPublicKey) -> RepositoryResult<()> {
        sqlx::query(
            "INSERT INTO public_key (account_id, public_key, height) VALUES ($1, $2, $3)"
        )
        .bind(pk.account_id as i64)
        .bind(pk.public_key.to_vec())
        .bind(pk.height as i64)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}
