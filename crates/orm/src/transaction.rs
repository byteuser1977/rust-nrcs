//! Database Transaction Manager
//!
//! Provides transaction management for database operations

use sqlx::{PgPool, Postgres, Transaction};
use std::ops::{Deref, DerefMut};

pub struct DatabaseTransaction {
    tx: Option<Transaction<'static, Postgres>>,
}

impl DatabaseTransaction {
    pub async fn new(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let tx = pool.begin().await?;
        Ok(Self { tx: Some(tx) })
    }

    pub async fn commit(mut self) -> Result<(), sqlx::Error> {
        if let Some(tx) = self.tx.take() {
            tx.commit().await?;
        }
        Ok(())
    }

    pub async fn rollback(mut self) -> Result<(), sqlx::Error> {
        if let Some(tx) = self.tx.take() {
            tx.rollback().await?;
        }
        Ok(())
    }

    pub fn as_mut(&mut self) -> Option<&mut Transaction<'static, Postgres>> {
        self.tx.as_mut()
    }

    pub fn as_ref(&self) -> Option<&Transaction<'static, Postgres>> {
        self.tx.as_ref()
    }
}

impl Deref for DatabaseTransaction {
    type Target = Transaction<'static, Postgres>;

    fn deref(&self) -> &Self::Target {
        self.tx.as_ref().expect("transaction already consumed")
    }
}

impl DerefMut for DatabaseTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tx.as_mut().expect("transaction already consumed")
    }
}

pub trait TransactionalRepository {
    fn with_transaction(&self, tx: DatabaseTransaction) -> Self;
}
