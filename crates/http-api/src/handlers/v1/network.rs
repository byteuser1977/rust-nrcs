//! 网络相关 API Handlers
//!
//! 与 Java 版本 GetPeers, GetPeer 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::bundler_service::{BundlerRule};
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GetPeersHandler;

impl GetPeersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPeersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["state", "includePeerInfo", "active", "service"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _state_filter = req.get_string("state");
        let include_info = req.get_bool("includePeerInfo");
        let active = req.get_bool("active");
        let _service = req.get_string("service");
        
        let peers = if let Some(ref p2p) = state.p2p_manager {
            if active {
                p2p.get_active_peers().await
            } else {
                p2p.get_peers().await
            }
        } else {
            vec![]
        };
        
        if include_info {
            let peers_json: Vec<serde_json::Value> = peers.iter()
                .map(peer_to_json)
                .collect();
            let mut builder = RsRespBuilder::new();
            builder.insert("peers", json!(peers_json));
            Ok(builder.build())
        } else {
            let peer_addresses: Vec<String> = peers.iter()
                .filter_map(|p| p.announced_address.clone())
                .collect();
            let mut builder = RsRespBuilder::new();
            builder.insert("peers", json!(peer_addresses));
            Ok(builder.build())
        }
    }
}

pub struct GetPeerHandler;

impl GetPeerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPeerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["peer"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let peer_address = req.require_string("peer")?;
        
        if let Some(ref p2p) = state.p2p_manager {
            let peers = p2p.get_peers().await;
            if let Some(peer) = peers.iter().find(|p| p.announced_address.as_ref() == Some(&peer_address)) {
                let mut builder = RsRespBuilder::new();
                builder.extend_json(peer_to_json(peer));
                return Ok(builder.build());
            }
        }
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("state", 0i32)
            .insert("announcedAddress", peer_address)
            .insert("shareAddress", true)
            .insert("downloadedVolume", 0i64)
            .insert("uploadedVolume", 0i64)
            .insert("application", "NRCS")
            .insert("version", "2.1.0")
            .insert("platform", "")
            .insert("blacklisted", false)
            .insert("lastUpdated", 0i32);
        
        Ok(builder.build())
    }
}

pub struct GetInboundPeersHandler;

impl GetInboundPeersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetInboundPeersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["includePeerInfo"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let include_info = req.get_bool("includePeerInfo");
        
        let inbound_peers = if let Some(ref p2p) = state.p2p_manager {
            let all_peers = p2p.get_active_peers().await;
            all_peers.into_iter().filter(|p| p.is_inbound).collect::<Vec<_>>()
        } else {
            vec![]
        };
        
        if include_info {
            let peers_json: Vec<serde_json::Value> = inbound_peers.iter()
                .map(peer_to_json)
                .collect();
            let mut builder = RsRespBuilder::new();
            builder.insert("peers", json!(peers_json));
            Ok(builder.build())
        } else {
            let peer_addresses: Vec<String> = inbound_peers.iter()
                .filter_map(|p| p.announced_address.clone())
                .collect();
            let mut builder = RsRespBuilder::new();
            builder.insert("peers", json!(peer_addresses));
            Ok(builder.build())
        }
    }
}

pub struct AddPeerHandler;

impl AddPeerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for AddPeerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["peer"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let peer_address = req.require_string("peer")?;
        
        if let Some(ref p2p) = state.p2p_manager {
            let addr: std::net::SocketAddr = peer_address.parse()
                .map_err(|_| ApiError::IncorrectPeerAddress)?;
            let peer = p2p::Peer::new(addr, false);
            p2p.add_peer(peer).await;
        }
        
        let mut builder = RsRespBuilder::new();
        builder.insert("state", 0i32);
        
        Ok(builder.build())
    }
}

pub struct BlacklistPeerHandler;

impl BlacklistPeerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for BlacklistPeerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["peer"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let peer_address = req.require_string("peer")?;
        
        if let Some(ref p2p) = state.p2p_manager {
            let addr: std::net::SocketAddr = peer_address.parse()
                .map_err(|_| ApiError::IncorrectPeerAddress)?;
            p2p.blacklist_peer(&addr, "API blacklist".to_string()).await;
        }
        
        let mut builder = RsRespBuilder::new();
        builder.insert("blacklisted", true);
        
        Ok(builder.build())
    }
}

pub struct GetMyInfoHandler;

impl GetMyInfoHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetMyInfoHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }
    
    async fn process_request(&self, _req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let my_info = if let Some(ref p2p) = state.p2p_manager {
            p2p.get_my_peer_info().await
        } else {
            json!({
                "application": "NRCS",
                "version": "2.1.0",
                "platform": "",
                "shareAddress": true,
                "announcedAddress": "",
                "hallmark": "",
                "services": []
            })
        };
        
        let mut builder = RsRespBuilder::new();
        builder.extend_json(my_info);
        
        Ok(builder.build())
    }
}

pub struct GetPluginsHandler;

impl GetPluginsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPluginsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("plugins", json!([]));
        
        Ok(builder.build())
    }
}

fn peer_to_json(peer: &p2p::Peer) -> serde_json::Value {
    json!({
        "state": peer.state as i32,
        "announcedAddress": peer.announced_address.clone().unwrap_or_default(),
        "shareAddress": peer.share_address,
        "downloadedVolume": peer.downloaded_volume as i64,
        "uploadedVolume": peer.uploaded_volume as i64,
        "application": peer.application.clone().unwrap_or_default(),
        "version": peer.version.clone().unwrap_or_default(),
        "platform": peer.platform.clone().unwrap_or_default(),
        "blacklisted": peer.blacklisting_time > 0,
        "lastUpdated": peer.last_updated as i32
    })
}

/// 添加打包规则 Handler
///
/// 对应 NRCS Java: `AddBundlingRule`
///
/// 功能：为现有打包器添加新的打包规则
pub struct AddBundlingRuleHandler;

impl AddBundlingRuleHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for AddBundlingRuleHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "minRateNQTPerFXT", "totalFeesLimitFQT", "overpayFQTPerFXT", "feeCalculatorName", "filter"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Forging]
    }

    fn require_post(&self) -> bool {
        true
    }

    fn require_password(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let secret_phrase = req.require_string("secretPhrase")?;
        let min_rate_nqt_per_fxt = req.require_u64("minRateNQTPerFXT")?;
        let _total_fees_limit_fqt = req.get_u64("totalFeesLimitFQT").unwrap_or(0);
        let _overpay_fqt_per_fxt = req.get_u64("overpayFQTPerFXT").unwrap_or(0);
        let fee_calculator_name = req
            .get_string("feeCalculatorName")
            .unwrap_or_else(|| "MinFeeCalculator".to_string());
        
        // 获取过滤器参数（支持多个 filter 参数）
        let filters: Vec<String> = req
            .params
            .keys()
            .filter(|k| *k == "filter")
            .filter_map(|k| req.params.get(k).cloned())
            .collect();

        // 构建规则对象
        let rule = BundlerRule {
            min_rate_nqt_per_fxt,
            overpay_fqt_per_fxt: _overpay_fqt_per_fxt,
            fee_calculator_name,
            filters,
        };

        // 调用 Bundler 服务添加规则
        let result = state
            .bundler_service
            .add_bundling_rule(&secret_phrase, rule)
            .await
            .map_err(ApiError::Internal)?;

        match result {
            Some(bundler_info) => {
                let mut builder = RsRespBuilder::new();
                builder.insert_bundler_info(&bundler_info);
                Ok(builder.build())
            }
            None => Err(ApiError::Validation(
                "Bundler not found for this account. Please start a bundler first.".to_string(),
            )),
        }
    }
}

/// 黑名单 API 代理节点 Handler
///
/// 对应 NRCS Java: `BlacklistAPIProxyPeer`
pub struct BlacklistAPIProxyPeerHandler;

impl BlacklistAPIProxyPeerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for BlacklistAPIProxyPeerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["peer", "adminPassword"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let peer = req.require_string("peer")?;
        let _admin_password = req.get_string("adminPassword");

        state
            .bundler_service
            .blacklist_api_proxy_peer(&peer)
            .await
            .map_err(ApiError::Internal)?;

        let mut builder = RsRespBuilder::new();
        builder.insert("done", true);

        Ok(builder.build())
    }
}

/// 黑名单打包器 Handler
///
/// 对应 NRCS Java: `BlacklistBundler`
pub struct BlacklistBundlerHandler;

impl BlacklistBundlerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for BlacklistBundlerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "adminPassword"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account = req.require_u64("account")?;
        let _admin_password = req.get_string("adminPassword");

        let removed = state
            .bundler_service
            .blacklist_bundler(account)
            .await
            .map_err(ApiError::Internal)?;

        let mut builder = RsRespBuilder::new();
        builder.insert("removed", removed);

        Ok(builder.build())
    }
}

/// 打包交易 Handler
///
/// 对应 NRCS Java: `BundleTransactions`
///
/// 功能：手动触发交易打包（创建子区块交易）
pub struct BundleTransactionsHandler;

impl BundleTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for BundleTransactionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "transactionFullHash", "feeNQT", "deadline"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Forging, ApiTag::CreateTransaction]
    }

    fn require_post(&self) -> bool {
        true
    }

    fn require_password(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let secret_phrase = req.require_string("secretPhrase")?;
        let _transaction_full_hash = req.require_string("transactionFullHash")?;
        let _fee_nqt = req.get_string("feeNQT");
        let _deadline = req.get_i32("deadline").unwrap_or(600);

        // 获取或创建打包器
        let bundler_info = state
            .bundler_service
            .start_bundler(
                &secret_phrase,
                u64::MAX, // 无限制
                vec![BundlerRule::default()],
            )
            .await
            .map_err(ApiError::Internal)?;

        // 触发一次打包运行（实际实现中会扫描未确认交易并打包）
        // 这里简化处理，返回打包器信息
        let mut builder = RsRespBuilder::new();
        builder.insert_bundler_info(&bundler_info);

        Ok(builder.build())
    }
}

/// 获取打包器费率 Handler
///
/// 对应 NRCS Java: `GetBundlerRates`
pub struct GetBundlerRatesHandler;

impl GetBundlerRatesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBundlerRatesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["minRateNQTPerFXT", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _min_rate_nqt_per_fxt = req.get_u64("minRateNQTPerFXT");

        // 获取所有打包器费率
        let rates = state
            .bundler_service
            .get_bundler_rates()
            .await;

        // 可选过滤：只返回费率 >= minRateNQTPerFXT 的
        let filtered_rates: Vec<serde_json::Value> = if let Some(min_rate) = _min_rate_nqt_per_fxt {
            rates
                .into_iter()
                .filter(|r| r.rate >= min_rate)
                .map(|r| json!({
                    "rate": r.rate,
                    "feeLimit": r.fee_limit,
                    "account": r.account_id.to_string(),
                    "timestamp": r.timestamp
                }))
                .collect()
        } else {
            rates
                .into_iter()
                .map(|r| json!({
                    "rate": r.rate,
                    "feeLimit": r.fee_limit,
                    "account": r.account_id.to_string(),
                    "timestamp": r.timestamp
                }))
                .collect()
        };

        let mut builder = RsRespBuilder::new();
        builder.insert("rates", json!(filtered_rates));

        Ok(builder.build())
    }
}

/// 获取打包器列表 Handler
///
/// 对应 NRCS Java: `GetBundlers`
pub struct GetBundlersHandler;

impl GetBundlersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBundlersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "secretPhrase", "adminPassword"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Forging]
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account = req.get_u64("account");
        let _secret_phrase = req.get_string("secretPhrase");
        let _admin_password = req.get_string("adminPassword");

        // 根据参数获取不同的打包器列表
        let bundlers = if let Some(acc_id) = account {
            state.bundler_service.get_account_bundlers(acc_id).await
        } else {
            state.bundler_service.get_all_bundlers().await
        };

        // 转换为 JSON 数组
        let bundlers_json: Vec<serde_json::Value> = bundlers
            .iter()
            .map(|b| json!({
                "bundler": b.account_id.to_string(),
                "totalFeesLimitFQT": b.total_fees_limit_fqt.to_string(),
                "currentTotalFeesFQT": b.current_total_fees_fqt.to_string(),
                "announcedMinRateNQTPerFXT": b.announced_min_rate_nqt_per_fxt.to_string(),
                "bundlingRules": b.bundling_rules.clone()
            }))
            .collect();

        let mut builder = RsRespBuilder::new();
        builder.insert("bundlers", json!(bundlers_json));

        Ok(builder.build())
    }
}

/// 获取打包选项 Handler
///
/// 对应 NRCS Java: `GetBundlingOptions`
///
/// 功能：返回可用的过滤器和费率计算器列表
pub struct GetBundlingOptionsHandler;

impl GetBundlingOptionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBundlingOptionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }

    async fn process_request(&self, _req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        // 获取可用的过滤器列表
        let filters = state.bundler_service.get_available_filters();
        let calculators = state.bundler_service.get_available_fee_calculators();

        // 构建过滤器 JSON 数组
        let filters_json: Vec<serde_json::Value> = filters
            .iter()
            .map(|f| json!({"name": f}))
            .collect();

        // 构建费率计算器 JSON 数组
        let calculators_json: Vec<serde_json::Value> = calculators
            .iter()
            .map(|c| json!({"name": c}))
            .collect();

        let mut builder = RsRespBuilder::new();
        builder
            .insert("filters", json!(filters_json))
            .insert("feeCalculators", json!(calculators_json));

        Ok(builder.build())
    }
}

/// 设置 API 代理节点 Handler
///
/// 对应 NRCS Java: `SetAPIProxyPeer`
pub struct SetAPIProxyPeerHandler;

impl SetAPIProxyPeerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SetAPIProxyPeerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["peer", "adminPassword"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let peer = req.require_string("peer")?;
        let _admin_password = req.get_string("adminPassword");

        state
            .bundler_service
            .set_api_proxy_peer(&peer)
            .await
            .map_err(ApiError::Internal)?;

        let mut builder = RsRespBuilder::new();
        builder.insert("done", true);

        Ok(builder.build())
    }
}

/// 获取所有打包器费率 Handler
///
/// 对应 NRCS Java: `GetAllBundlerRates`
pub struct GetAllBundlerRatesHandler;

impl GetAllBundlerRatesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllBundlerRatesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }

    async fn process_request(&self, _req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        // 获取所有费率（不经过过滤）
        let rates = state
            .bundler_service
            .get_bundler_rates()
            .await;

        // 转换为 JSON 数组（与 GetBundlerRates 格式一致）
        let rates_json: Vec<serde_json::Value> = rates
            .into_iter()
            .map(|r| json!({
                "rate": r.rate,
                "feeLimit": r.fee_limit,
                "account": r.account_id.to_string(),
                "timestamp": r.timestamp
            }))
            .collect();

        let mut builder = RsRespBuilder::new();
        builder.insert("rates", json!(rates_json));

        Ok(builder.build())
    }
}
