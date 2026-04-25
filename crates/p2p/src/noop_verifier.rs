//! No-op blockchain verifier for SQLite mode
//!
//! This is a temporary solution for SQLite mode where full blockchain verification
//! is not yet implemented.

use async_trait::async_trait;
use blockchain_types::prelude::*;
use anyhow::Result;

use crate::handlers::BlockVerifier;

pub struct NoOpBlockVerifier;

impl NoOpBlockVerifier {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BlockVerifier for NoOpBlockVerifier {
    async fn verify_and_process(&self, _block: Block) -> Result<()> {
        Ok(())
    }

    async fn has_block(&self, _block_id: u64) -> Result<bool> {
        Ok(false)
    }
}
