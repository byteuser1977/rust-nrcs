//! Chain Service
//!
//! 负责区块生成、验证、存储、同步

use std::sync::Arc;

use blockchain_types::*;
use orm::{BlockRepository, BlockModel};
use tx_engine::TransactionProcessor;
use account::AccountManager;
use chrono::Utc;

pub struct ChainService {
    block_repo: Arc<dyn BlockRepository>,
    tx_processor: Arc<dyn TransactionProcessor>,
    account_manager: Arc<dyn AccountManager>,
    current_height: Arc<tokio::sync::Mutex<Height>>,
}

impl ChainService {
    pub fn new(
        block_repo: Arc<dyn BlockRepository>,
        tx_processor: Arc<dyn TransactionProcessor>,
        account_manager: Arc<dyn AccountManager>,
    ) -> Self {
        Self {
            block_repo,
            tx_processor,
            account_manager,
            current_height: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }

    /// 获取当前链高度
    pub async fn current_height(&self) -> Height {
        *self.current_height.lock().await
    }

    /// 处理接收到的区块
    pub async fn handle_received_block(&self, block: Block) -> anyhow::Result<()> {
        let current_height = self.current_height().await;

        // 1. 验证区块
        block.validate_full(current_height + 1)?;

        // 2. 执行所有交易
        for tx in &block.transactions {
            self.tx_processor.execute(tx).await?;
        }

        // 3. 保存到数据库
        let block_model = BlockModel::from_domain(&block)?;
        self.block_repo.insert(&block_model).await?;

        // 4. 更新链高度
        *self.current_height.lock().await = block.height;

        info!("Block {} accepted (hash: {:x})", block.height, block.compute_hash()?);

        Ok(())
    }

    /// 启动区块同步
    pub async fn start_sync(&self) -> anyhow::Result<()> {
        info!("Starting chain synchronization...");

        // TODO: 从 peers 请求缺失的区块

        Ok(())
    }

    /// 创建新区块（挖矿/出块）
    pub async fn create_block(&self, generator_id: AccountId, transactions: Vec<Transaction>) -> anyhow::Result<Block> {
        let height = self.current_height().await + 1;

        // 计算 payload hash (merkle root)
        let payload_hash = Block::compute_merkle_root(&transactions)?;

        // 获取最新区块哈希
        let previous_hash = if height > 1 {
            // TODO: 从 DB 查询最新区块
            [0u8; 32]
        } else {
            [0u8; 32] // 创世区块
        };

        let mut block = Block::new(height, previous_hash, generator_id);
        block.timestamp = Utc::now().timestamp() as u32;
        block.payload_hash = payload_hash;
        block.transactions = transactions;
        block.payload_length = transactions.iter().map(|tx| tx.size()).sum::<usize>() as u32;

        // 计算总金额和手续费
        let (total_amount, total_fee) = transactions.iter().fold((0, 0), |(acc_a, acc_f), tx| {
            (acc_a.saturating_add(tx.amount), acc_f.saturating_add(tx.fee))
        });
        block.total_amount = total_amount;
        block.total_fee = total_fee;

        // TODO: 根据共识算法计算 nonce 和签名

        Ok(block)
    }
}