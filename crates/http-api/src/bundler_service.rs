//! Bundler 服务模块
//!
//! 对应 NRCS Java: `com.bytechain.nrcs.service.bundler.Bundler`
//!
//! 功能：
//! - 管理交易打包器（Bundler）实例
//! - 支持添加/删除打包规则
//! - 提供费率查询接口
//! - 支持启动/停止打包器

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 打包规则
///
/// 对应 NRCS Java: `BundlerRule`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundlerRule {
    /// 最低费率 (NQT per FXT)
    pub min_rate_nqt_per_fxt: u64,
    /// 超付金额 (FQT)
    pub overpay_fqt_per_fxt: u64,
    /// 费率计算器名称
    #[serde(rename = "feeCalculatorName")]
    pub fee_calculator_name: String,
    /// 过滤器参数
    pub filters: Vec<String>,
}

impl Default for BundlerRule {
    fn default() -> Self {
        Self {
            min_rate_nqt_per_fxt: 0,
            overpay_fqt_per_fxt: 0,
            fee_calculator_name: "MinFeeCalculator".to_string(),
            filters: vec![],
        }
    }
}

/// 打包器实例
///
/// 对应 NRCS Java: `Bundler` 类
#[derive(Debug, Clone, Serialize)]
pub struct BundlerInfo {
    /// 账户 ID
    #[serde(rename = "bundler")]
    pub account_id: u64,
    /// 公钥 (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    /// 总费用限制 (FQT)
    #[serde(rename = "totalFeesLimitFQT")]
    pub total_fees_limit_fqt: u64,
    /// 当前已用费用 (FQT)
    #[serde(rename = "currentTotalFeesFQT")]
    pub current_total_fees_fqt: u64,
    /// 公布的最低费率
    #[serde(rename = "announcedMinRateNQTPerFXT")]
    pub announced_min_rate_nqt_per_fxt: u64,
    /// 打包规则列表
    #[serde(rename = "bundlingRules")]
    pub bundling_rules: Vec<BundlerRule>,
}

/// 打包器费率信息
///
/// 对应 NRCS Java: `IBundlerRate` 接口
#[derive(Debug, Clone, Serialize)]
pub struct BundlerRate {
    /// 费率
    pub rate: u64,
    /// 费用限制
    #[serde(rename = "feeLimit")]
    pub fee_limit: u64,
    /// 账户 ID
    #[serde(rename = "account")]
    pub account_id: u64,
    /// 时间戳
    pub timestamp: i32,
}

/// Bundler 服务 Trait
///
/// 定义打包器管理的 API 接口，由外部实现或使用默认的内存实现
#[async_trait::async_trait]
pub trait BundlerServiceApi: Send + Sync {
    /// 启动打包器
    ///
    /// 对应 NRCS Java: `Bundler.addOrChangeBundler()`
    async fn start_bundler(
        &self,
        secret_phrase: &str,
        total_fees_limit_fqt: u64,
        rules: Vec<BundlerRule>,
    ) -> Result<BundlerInfo, String>;

    /// 停止打包器
    ///
    /// 对应 NRCS Java: `Bundler.stopBundler()`
    async fn stop_bundler(&self, account_id: u64) -> Result<Option<BundlerInfo>, String>;

    /// 停止所有打包器
    ///
    /// 对应 NRCS Java: `Bundler.stopAllBundlers()`
    async fn stop_all_bundlers(&self) -> Result<Vec<BundlerInfo>, String>;

    /// 添加打包规则
    ///
    /// 对应 NRCS Java: `Bundler.addBundlingRule()`
    async fn add_bundling_rule(
        &self,
        secret_phrase: &str,
        rule: BundlerRule,
    ) -> Result<Option<BundlerInfo>, String>;

    /// 获取所有打包器列表
    ///
    /// 对应 NRCS Java: `Bundler.getAllBundlers()`
    async fn get_all_bundlers(&self) -> Vec<BundlerInfo>;

    /// 获取指定账户的打包器
    ///
    /// 对应 NRCS Java: `Bundler.getAccountBundlers()`
    async fn get_account_bundlers(&self, account_id: u64) -> Vec<BundlerInfo>;

    /// 获取所有打包器费率
    ///
    /// 对应 NRCS Java: `Bundler.getBundlerRates()`
    async fn get_bundler_rates(&self) -> Vec<BundlerRate>;

    /// 黑名单指定账户的打包器
    async fn blacklist_bundler(&self, account_id: u64) -> Result<bool, String>;

    /// 设置 API 代理节点
    async fn set_api_proxy_peer(&self, peer_address: &str) -> Result<(), String>;

    /// 黑名单 API 代理节点
    async fn blacklist_api_proxy_peer(&self, peer_address: &str) -> Result<(), String>;

    /// 获取可用的过滤器列表
    fn get_available_filters(&self) -> Vec<String>;

    /// 可用的费率计算器列表
    fn get_available_fee_calculators(&self) -> Vec<String>;
}

/// 内存实现的 Bundler 服务
///
/// 用于开发和测试环境，生产环境应使用持久化实现
#[derive(Clone)]
pub struct MemoryBundlerService {
    bundlers: Arc<RwLock<HashMap<u64, BundlerInfo>>>,
    blacklisted_bundlers: Arc<RwLock<std::collections::HashSet<u64>>>,
    api_proxy_peer: Arc<RwLock<Option<String>>>,
    blacklisted_proxy_peers: Arc<RwLock<std::collections::HashSet<String>>>,
}

impl Default for MemoryBundlerService {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryBundlerService {
    /// 创建新的内存 Bundler 服务
    pub fn new() -> Self {
        Self {
            bundlers: Arc::new(RwLock::new(HashMap::new())),
            blacklisted_bundlers: Arc::new(RwLock::new(std::collections::HashSet::new())),
            api_proxy_peer: Arc::new(RwLock::new(None)),
            blacklisted_proxy_peers: Arc::new(RwLock::new(std::collections::HashSet::new())),
        }
    }

    /// 计算账户 ID 从公钥
    ///
    /// 对应 NRCS Java: `Account.getId(publicKey)`
    pub fn account_id_from_secret_phrase(secret_phrase: &str) -> u64 {
        let seed = crypto::sha256(secret_phrase.as_bytes());
        let kp = crypto::keypair_from_seed(&seed);
        let public_key = kp.public_key();
        let pk_bytes = public_key.as_bytes();
        let hash = crypto::sha256(pk_bytes);
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&hash[0..8]);
        u64::from_le_bytes(bytes)
    }

    /// 计算最低公布费率
    fn calculate_announced_min_rate(rules: &[BundlerRule]) -> u64 {
        rules
            .iter()
            .filter(|r| r.filters.is_empty())
            .map(|r| r.min_rate_nqt_per_fxt)
            .min()
            .unwrap_or(0)
    }
}

#[async_trait::async_trait]
impl BundlerServiceApi for MemoryBundlerService {
    async fn start_bundler(
        &self,
        secret_phrase: &str,
        total_fees_limit_fqt: u64,
        rules: Vec<BundlerRule>,
    ) -> Result<BundlerInfo, String> {
        let account_id = Self::account_id_from_secret_phrase(secret_phrase);

        // 生成公钥 hex（简化版）
        let seed = crypto::sha256(secret_phrase.as_bytes());
        let kp = crypto::keypair_from_seed(&seed);
        let public_key_hex = hex::encode(kp.public_key().as_bytes());

        let announced_min_rate = Self::calculate_announced_min_rate(&rules);

        let bundler_info = BundlerInfo {
            account_id,
            public_key: Some(public_key_hex),
            total_fees_limit_fqt,
            current_total_fees_fqt: 0,
            announced_min_rate_nqt_per_fxt: announced_min_rate,
            bundling_rules: rules,
        };

        let mut bundlers = self.bundlers.write().await;
        bundlers.insert(account_id, bundler_info.clone());

        Ok(bundler_info)
    }

    async fn stop_bundler(&self, account_id: u64) -> Result<Option<BundlerInfo>, String> {
        let mut bundlers = self.bundlers.write().await;
        Ok(bundlers.remove(&account_id))
    }

    async fn stop_all_bundlers(&self) -> Result<Vec<BundlerInfo>, String> {
        let mut bundlers = self.bundlers.write().await;
        let removed: Vec<BundlerInfo> = bundlers.drain().map(|(_, v)| v).collect();
        Ok(removed)
    }

    async fn add_bundling_rule(
        &self,
        secret_phrase: &str,
        rule: BundlerRule,
    ) -> Result<Option<BundlerInfo>, String> {
        let account_id = Self::account_id_from_secret_phrase(secret_phrase);

        let mut bundlers = self.bundlers.write().await;

        if let Some(bundler) = bundlers.get_mut(&account_id) {
            bundler.bundling_rules.push(rule);
            bundler.announced_min_rate_nqt_per_fxt =
                Self::calculate_announced_min_rate(&bundler.bundling_rules);
            return Ok(Some(bundler.clone()));
        }

        Ok(None)
    }

    async fn get_all_bundlers(&self) -> Vec<BundlerInfo> {
        let bundlers = self.bundlers.read().await;
        bundlers.values().cloned().collect()
    }

    async fn get_account_bundlers(&self, account_id: u64) -> Vec<BundlerInfo> {
        let bundlers = self.bundlers.read().await;
        bundlers
            .get(&account_id)
            .map(|b| vec![b.clone()])
            .unwrap_or_default()
    }

    async fn get_bundler_rates(&self) -> Vec<BundlerRate> {
        let bundlers = self.bundlers.read().await;
        let now = current_epoch_time();

        bundlers
            .values()
            .filter(|b| b.announced_min_rate_nqt_per_fxt > 0)
            .map(|b| BundlerRate {
                rate: b.announced_min_rate_nqt_per_fxt,
                fee_limit: if b.total_fees_limit_fqt > 0 {
                    b.total_fees_limit_fqt - b.current_total_fees_fqt
                } else {
                    u64::MAX
                },
                account_id: b.account_id,
                timestamp: now,
            })
            .collect()
    }

    async fn blacklist_bundler(&self, account_id: u64) -> Result<bool, String> {
        let mut blacklisted = self.blacklisted_bundlers.write().await;
        let removed = self.stop_bundler(account_id).await?.is_some();
        blacklisted.insert(account_id);
        Ok(removed)
    }

    async fn set_api_proxy_peer(&self, peer_address: &str) -> Result<(), String> {
        let mut proxy = self.api_proxy_peer.write().await;
        *proxy = Some(peer_address.to_string());
        Ok(())
    }

    async fn blacklist_api_proxy_peer(&self, peer_address: &str) -> Result<(), String> {
        let mut blacklisted = self.blacklisted_proxy_peers.write().await;
        blacklisted.insert(peer_address.to_string());

        // 如果当前代理节点被黑名单，清除它
        let mut proxy = self.api_proxy_peer.write().await;
        if proxy.as_deref() == Some(peer_address) {
            *proxy = None;
        }

        Ok(())
    }

    fn get_available_filters(&self) -> Vec<String> {
        vec!["TransactionFilter".to_string(), "MinAmountFilter".to_string()]
    }

    fn get_available_fee_calculators(&self) -> Vec<String> {
        vec![
            "MinFeeCalculator".to_string(),
            "ProportionalFeeCalculator".to_string(),
        ]
    }
}

/// Get current epoch time (seconds since genesis)
fn current_epoch_time() -> i32 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let genesis_timestamp: i64 = 1514764800; // 2018-01-01 00:00:00 UTC
    ((now.as_secs() as i64) - genesis_timestamp) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_bundler_service_creation() {
        let service = MemoryBundlerService::new();
        let bundlers = service.get_all_bundlers().await;
        assert!(bundlers.is_empty());
    }

    #[tokio::test]
    async fn test_start_and_stop_bundler() {
        let service = MemoryBundlerService::new();

        let rule = BundlerRule {
            min_rate_nqt_per_fxt: 1000,
            ..Default::default()
        };

        let bundler = service
            .start_bundler("test_secret", 1000000, vec![rule])
            .await
            .unwrap();

        assert_eq!(bundler.total_fees_limit_fqt, 1000000);
        assert_eq!(bundler.bundling_rules.len(), 1);

        let stopped = service.stop_bundler(bundler.account_id).await.unwrap();
        assert!(stopped.is_some());

        let bundlers = service.get_all_bundlers().await;
        assert!(bundlers.is_empty());
    }

    #[tokio::test]
    async fn test_add_bundling_rule() {
        let service = MemoryBundlerService::new();

        // 先启动打包器
        let rule1 = BundlerRule {
            min_rate_nqt_per_fxt: 1000,
            ..Default::default()
        };
        service
            .start_bundler("test_secret", 1000000, vec![rule1])
            .await
            .unwrap();

        // 添加新规则
        let rule2 = BundlerRule {
            min_rate_nqt_per_fxt: 2000,
            fee_calculator_name: "ProportionalFeeCalculator".to_string(),
            ..Default::default()
        };

        let result = service.add_bundling_rule("test_secret", rule2).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().bundling_rules.len(), 2);
    }

    #[tokio::test]
    async fn test_get_bundler_rates() {
        let service = MemoryBundlerService::new();

        let rule = BundlerRule {
            min_rate_nqt_per_fxt: 1000,
            ..Default::default()
        };
        service
            .start_bundler("test_secret", 1000000, vec![rule])
            .await
            .unwrap();

        let rates = service.get_bundler_rates().await;
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].rate, 1000);
    }

    #[tokio::test]
    async fn test_blacklist_bundler() {
        let service = MemoryBundlerService::new();

        let rule = BundlerRule::default();
        let bundler = service
            .start_bundler("test_secret", 1000000, vec![rule])
            .await
            .unwrap();

        let result = service.blacklist_bundler(bundler.account_id).await.unwrap();
        assert!(result);

        let bundlers = service.get_all_bundlers().await;
        assert!(bundlers.is_empty());
    }

    #[tokio::test]
    async fn test_api_proxy_peer_management() {
        let service = MemoryBundlerService::new();

        service.set_api_proxy_peer("127.0.0.1:7876").await.unwrap();
        service
            .blacklist_api_proxy_peer("127.0.0.1:7876")
            .await
            .unwrap();

        // 验证可用过滤器和计算器
        let filters = service.get_available_filters();
        assert!(!filters.is_empty());

        let calculators = service.get_available_fee_calculators();
        assert!(!calculators.is_empty());
    }
}
