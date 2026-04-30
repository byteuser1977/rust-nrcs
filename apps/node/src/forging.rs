//! Forging Service - 出块循环
//!
//! 对应 Java: Generator.generateBlocksThread()
//!
//! 每 500ms 检查是否有注册的锻造者可以出块。
//! 流程:
//! 1. 获取最新区块
//! 2. 计算 generationLimit = epoch_seconds - FORGING_DELAY
//! 3. 更新每个 generator 的 hit_time
//! 4. 按 hit_time 排序，找到第一个 hit_time <= generation_limit 的锻造者
//! 5. 收集未确认交易，构建区块，签名，入链，广播

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{error, info, warn};

use blockchain_types::prelude::*;
use blockchain_types::{ONE_NRCS, FORGING_DELAY};
use consensus::{BlockGenerator, GeneratorRegistry, GeneratorInfo};
use orm::{BlockRepository, TransactionRepository, AccountRepository};
use p2p::handlers::BlockVerifier;
use p2p::peer::Peers;
use p2p::config::P2PConfig;
use p2p::broadcast::BroadcastManager;

const FORGING_INTERVAL_MS: u64 = 500;

pub struct ForgingService {
    registry: GeneratorRegistry,
    block_generator: BlockGenerator,
    block_repo: Arc<dyn BlockRepository>,
    tx_repo: Arc<dyn TransactionRepository>,
    account_repo: Arc<dyn AccountRepository>,
    block_verifier: Arc<dyn BlockVerifier>,
    peers: Arc<Peers>,
    p2p_config: Arc<P2PConfig>,
    /// account_id -> secret_phrase 映射，用于反向查找
    secret_map: RwLock<HashMap<AccountId, String>>,
}

impl ForgingService {
    pub fn new(
        block_repo: Arc<dyn BlockRepository>,
        tx_repo: Arc<dyn TransactionRepository>,
        account_repo: Arc<dyn AccountRepository>,
        block_verifier: Arc<dyn BlockVerifier>,
        peers: Arc<Peers>,
        p2p_config: Arc<P2PConfig>,
    ) -> Self {
        Self {
            registry: GeneratorRegistry::new(),
            block_generator: BlockGenerator::new(),
            block_repo,
            tx_repo,
            account_repo,
            block_verifier,
            peers,
            p2p_config,
            secret_map: RwLock::new(HashMap::new()),
        }
    }

    /// 注册锻造者
    pub async fn start_forging(&self, secret_phrase: &str) -> Result<()> {
        let generator = consensus::Generator::new(secret_phrase)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let account_id = generator.get_account_id();

        self.registry.start_forging(secret_phrase)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        self.secret_map.write().await.insert(account_id, secret_phrase.to_string());

        info!("[Forging] Started forging for account {}", account_id);
        Ok(())
    }

    /// 注销锻造者
    pub async fn stop_forging(&self, secret_phrase: &str) -> Result<()> {
        let generator = consensus::Generator::new(secret_phrase)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let account_id = generator.get_account_id();

        self.registry.stop_forging(secret_phrase)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        self.secret_map.write().await.remove(&account_id);

        info!("[Forging] Stopped forging for account {}", account_id);
        Ok(())
    }

    /// 获取所有注册的锻造者信息
    #[allow(dead_code)]
    pub fn get_forgers(&self) -> Vec<GeneratorInfo> {
        self.registry.get_all_generators()
    }
    ///获取锻造者数量
    #[allow(dead_code)]
    pub fn get_forger_count(&self) -> usize {
        self.registry.get_generator_count()
    }

    /// 启动出块主循环
    pub async fn run(self: Arc<Self>) {
        info!("[Forging] Starting forging loop (interval={}ms)", FORGING_INTERVAL_MS);

        loop {
            tokio::time::sleep(Duration::from_millis(FORGING_INTERVAL_MS)).await;

            if self.registry.get_generator_count() == 0 {
                continue;
            }

            if let Err(e) = self.try_forge().await {
                error!("[Forging] Error in forging loop: {}", e);
            }
        }
    }

    /// 单次出块尝试
    async fn try_forge(&self) -> Result<()> {
        // 1. 获取最新区块
        let last_block_model = self.block_repo.find_latest().await?;

        let last_block = match last_block_model {
            Some(model) => model.to_domain()?,
            None => return Ok(()),
        };

        // 2. 计算 generationLimit
        let epoch_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let forging_delay = FORGING_DELAY as u64;
        let generation_limit = epoch_seconds.saturating_sub(forging_delay);

        // 3. 更新所有锻造者的 hit_time
        self.update_generators(&last_block).await?;

        // 4. 获取排序后的锻造者列表
        let sorted_forgers = self.registry.get_sorted_forgers();

        // 5. 遍历寻找可以出块的锻造者
        for forger in &sorted_forgers {
            if forger.hit_time > generation_limit {
                break;
            }

            let secret_phrase = {
                let map = self.secret_map.read().await;
                match map.get(&forger.account_id) {
                    Some(s) => s.clone(),
                    None => continue,
                }
            };

            let timestamp = self.get_timestamp(forger, generation_limit);

            if !self.verify_hit(forger, &last_block, timestamp) {
                warn!("[Forging] Hit verification failed for account {}", forger.account_id);
                continue;
            }

            info!("[Forging] Account {} eligible to forge at timestamp {}",
                  forger.account_id, timestamp);

            // 6. 收集未确认交易
            let unconfirmed_txs = self.collect_unconfirmed_transactions().await;

            // 7. 构建区块
            let template = self.block_generator.create_block_template(
                &last_block,
                forger.account_id,
                timestamp,
                unconfirmed_txs,
            );

            let block = match self.block_generator.build_block(template, &secret_phrase) {
                Ok(b) => b,
                Err(e) => {
                    warn!("[Forging] Failed to build block: {}", e);
                    continue;
                }
            };

            // 8. 入链
            if let Err(e) = self.block_verifier.verify_and_process(block.clone()).await {
                warn!("[Forging] Failed to verify and process block: {}", e);
                continue;
            }

            info!("[Forging] Successfully forged block at height {} by account {}",
                  block.height, forger.account_id);

            // 9. 广播
            self.broadcast_block(&block).await;

            return Ok(());
        }

        Ok(())
    }

    /// 更新所有锻造者的 hit_time
    async fn update_generators(&self, last_block: &Block) -> Result<()> {
        let secret_map = self.secret_map.read().await;

        for (account_id, secret_phrase) in secret_map.iter() {
            let effective_balance = self.get_effective_balance(*account_id).await;

            if let Err(e) = self.registry.update_generator(secret_phrase, last_block, effective_balance) {
                warn!("[Forging] Failed to update generator {}: {}", account_id, e);
            }
        }

        Ok(())
    }

    /// 获取账户的有效余额（NRCS 单位）
    async fn get_effective_balance(&self, account_id: AccountId) -> u64 {
        match self.account_repo.find_by_account_id(account_id as i64).await {
            Ok(Some(account)) => {
                let balance = account.balance.max(0) as u64;
                balance / ONE_NRCS
            }
            _ => 0,
        }
    }

    /// 计算出块时间戳
    /// 对应 Java: Generator.getTimestamp(generationLimit)
    fn get_timestamp(&self, forger: &GeneratorInfo, generation_limit: u64) -> u32 {
        if generation_limit > forger.hit_time + 3600 {
            generation_limit as u32
        } else {
            forger.hit_time as u32 + 1
        }
    }

    /// 验证 hit
    /// 对应 Java: Generator.verifyHit(lastBlock, timestamp)
    fn verify_hit(&self, forger: &GeneratorInfo, last_block: &Block, timestamp: u32) -> bool {
        use num_bigint::BigUint;

        let elapsed_time = timestamp as i64 - last_block.timestamp as i64;
        if elapsed_time <= 0 {
            return false;
        }

        let effective_base_target = BigUint::from(last_block.base_target)
            * &forger.effective_balance;
        let prev_target = &effective_base_target * BigUint::from(elapsed_time as u64 - 1);
        let target = &prev_target + &effective_base_target;

        forger.hit < target && (forger.hit >= prev_target || elapsed_time as u64 > 600)
    }

    /// 收集未确认交易
    async fn collect_unconfirmed_transactions(&self) -> Vec<Transaction> {
        match self.tx_repo.find_unconfirmed(100).await {
            Ok(models) => {
                models.into_iter()
                    .filter_map(|m| m.to_domain().ok())
                    .collect()
            }
            Err(e) => {
                warn!("[Forging] Failed to fetch unconfirmed transactions: {}", e);
                vec![]
            }
        }
    }

    /// 广播区块到 P2P 网络
    async fn broadcast_block(&self, block: &Block) {
        let block_json = match serde_json::to_value(block) {
            Ok(json) => json,
            Err(e) => {
                warn!("[Forging] Failed to serialize block for broadcast: {}", e);
                return;
            }
        };

        let epoch_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let previous_block_id = block.previous_block_id.unwrap_or(0);

        let broadcast_mgr = BroadcastManager::new(Arc::clone(&self.p2p_config));
        broadcast_mgr.broadcast_block(
            &block_json,
            previous_block_id,
            epoch_seconds,
            &self.peers,
        ).await;
    }
}

/// 实现 http-api 的 ForgingApi trait
#[async_trait::async_trait]
impl http_api::state::ForgingApi for ForgingService {
    async fn start_forging(&self, secret_phrase: &str) -> std::result::Result<(), String> {
        ForgingService::start_forging(self, secret_phrase).await
            .map_err(|e| e.to_string())
    }

    async fn stop_forging(&self, secret_phrase: &str) -> std::result::Result<(), String> {
        ForgingService::stop_forging(self, secret_phrase).await
            .map_err(|e| e.to_string())
    }

    fn get_forgers(&self) -> Vec<http_api::state::ForgingInfo> {
        self.registry.get_all_generators().into_iter().map(|g| {
            http_api::state::ForgingInfo {
                account_id: g.account_id,
                hit_time: g.hit_time,
                effective_balance: g.effective_balance.to_string(),
                deadline: g.deadline,
            }
        }).collect()
    }

    fn get_forger_count(&self) -> usize {
        self.registry.get_generator_count()
    }
}
