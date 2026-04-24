//! Blockchain verifier implementation
//!
//! Implements BlockVerifier trait using BlockchainProcessor

use async_trait::async_trait;
use blockchain_types::prelude::*;
use blockchain_types::processor::BlockchainProcessor;
use sqlx::PgPool;

use crate::handlers::BlockVerifier;

pub struct BlockchainVerifier {
    processor: BlockchainProcessor,
}

impl BlockchainVerifier {
    pub fn new(pool: PgPool) -> Self {
        Self {
            processor: BlockchainProcessor::new(pool),
        }
    }
}

#[async_trait]
impl BlockVerifier for BlockchainVerifier {
    async fn verify_and_process(&self, block: Block) -> anyhow::Result<()> {
        self.processor.process_block(&block).await?;
        Ok(())
    }
}
