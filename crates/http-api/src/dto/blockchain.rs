//! 区块链状态相关 DTO
//!
//! 与 Java APIGetBlockchainStatus 等完全对齐

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiGetBlockchainStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<i32>,
    
    #[serde(rename = "lastBlock", skip_serializing_if = "Option::is_none")]
    pub last_block: Option<String>,
    
    #[serde(rename = "cumulativeDifficulty", skip_serializing_if = "Option::is_none")]
    pub cumulative_difficulty: Option<String>,
    
    #[serde(rename = "numberOfBlocks", skip_serializing_if = "Option::is_none")]
    pub number_of_blocks: Option<i32>,
    
    #[serde(rename = "lastBlockchainFeeder", skip_serializing_if = "Option::is_none")]
    pub last_blockchain_feeder: Option<String>,
    
    #[serde(rename = "lastBlockchainFeederHeight", skip_serializing_if = "Option::is_none")]
    pub last_blockchain_feeder_height: Option<i32>,
    
    #[serde(rename = "isScanning", skip_serializing_if = "Option::is_none")]
    pub is_scanning: Option<bool>,
    
    #[serde(rename = "isDownloading", skip_serializing_if = "Option::is_none")]
    pub is_downloading: Option<bool>,
    
    #[serde(rename = "maxRollback", skip_serializing_if = "Option::is_none")]
    pub max_rollback: Option<i32>,
    
    #[serde(rename = "currentMinRollbackHeight", skip_serializing_if = "Option::is_none")]
    pub current_min_rollback_height: Option<i32>,
    
    #[serde(rename = "isTestnet", skip_serializing_if = "Option::is_none")]
    pub is_testnet: Option<bool>,
    
    #[serde(rename = "maxPrunableLifetime", skip_serializing_if = "Option::is_none")]
    pub max_prunable_lifetime: Option<i32>,
    
    #[serde(rename = "includeExpiredPrunable", skip_serializing_if = "Option::is_none")]
    pub include_expired_prunable: Option<bool>,
    
    #[serde(rename = "correctInvalidFees", skip_serializing_if = "Option::is_none")]
    pub correct_invalid_fees: Option<bool>,
    
    #[serde(rename = "ledgerTrimKeep", skip_serializing_if = "Option::is_none")]
    pub ledger_trim_keep: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<String>>,
    
    #[serde(rename = "apiProxy", skip_serializing_if = "Option::is_none")]
    pub api_proxy: Option<bool>,
    
    #[serde(rename = "apiProxyPeer", skip_serializing_if = "Option::is_none")]
    pub api_proxy_peer: Option<String>,
    
    #[serde(rename = "isLightClient", skip_serializing_if = "Option::is_none")]
    pub is_light_client: Option<bool>,
    
    #[serde(rename = "maxAPIRecords", skip_serializing_if = "Option::is_none")]
    pub max_api_records: Option<i32>,
    
    #[serde(rename = "blockchainState", skip_serializing_if = "Option::is_none")]
    pub blockchain_state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiGetTime {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<i32>,
}

impl ApiGetTime {
    pub fn new(time: i32) -> Self {
        Self { time: Some(time) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiGetState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<i32>,
    
    #[serde(rename = "lastBlock", skip_serializing_if = "Option::is_none")]
    pub last_block: Option<String>,
    
    #[serde(rename = "cumulativeDifficulty", skip_serializing_if = "Option::is_none")]
    pub cumulative_difficulty: Option<String>,
    
    #[serde(rename = "numberOfBlocks", skip_serializing_if = "Option::is_none")]
    pub number_of_blocks: Option<i32>,
    
    #[serde(rename = "totalEffectiveBalanceNRCS", skip_serializing_if = "Option::is_none")]
    pub total_effective_balance_nrcs: Option<i64>,
    
    #[serde(rename = "totalEffectiveBalanceNQT", skip_serializing_if = "Option::is_none")]
    pub total_effective_balance_nqt: Option<String>,
    
    #[serde(rename = "totalForgedBalanceNQT", skip_serializing_if = "Option::is_none")]
    pub total_forged_balance_nqt: Option<String>,
    
    #[serde(rename = "numberOfTransactions", skip_serializing_if = "Option::is_none")]
    pub number_of_transactions: Option<i32>,
    
    #[serde(rename = "numberOfAccounts", skip_serializing_if = "Option::is_none")]
    pub number_of_accounts: Option<i32>,
    
    #[serde(rename = "numberOfAssets", skip_serializing_if = "Option::is_none")]
    pub number_of_assets: Option<i32>,
    
    #[serde(rename = "numberOfOrders", skip_serializing_if = "Option::is_none")]
    pub number_of_orders: Option<i32>,
    
    #[serde(rename = "numberOfAskOrders", skip_serializing_if = "Option::is_none")]
    pub number_of_ask_orders: Option<i32>,
    
    #[serde(rename = "numberOfBidOrders", skip_serializing_if = "Option::is_none")]
    pub number_of_bid_orders: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiGetConstants {
    #[serde(rename = "maxBlockPayloadLength", skip_serializing_if = "Option::is_none")]
    pub max_block_payload_length: Option<i32>,
    
    #[serde(rename = "maxArbitraryMessageLength", skip_serializing_if = "Option::is_none")]
    pub max_arbitrary_message_length: Option<i32>,
    
    #[serde(rename = "transactionTypes", skip_serializing_if = "Option::is_none")]
    pub transaction_types: Option<serde_json::Value>,
    
    #[serde(rename = "peerStates", skip_serializing_if = "Option::is_none")]
    pub peer_states: Option<serde_json::Value>,
    
    #[serde(rename = "requestTypes", skip_serializing_if = "Option::is_none")]
    pub request_types: Option<Vec<String>>,
}
