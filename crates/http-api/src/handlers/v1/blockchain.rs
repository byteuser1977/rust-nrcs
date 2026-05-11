//! 区块链状态相关 API Handlers
//!
//! 与 Java 版本 GetBlockchainStatus, GetTime 等完全对齐

use async_trait::async_trait;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GetBlockchainStatusHandler;

impl GetBlockchainStatusHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBlockchainStatusHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Blocks, ApiTag::Info]
    }
    
    fn allow_required_block_parameters(&self) -> bool {
        false
    }
    
    async fn process_request(&self, _req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let latest_block = state.block_repo
            .find_latest()
            .await
            .map_err(ApiError::Repository)?;
        
        let (last_block_id, last_block_height, cumulative_difficulty) = match latest_block {
            Some(b) => (b.id.to_string(), b.height, hex::encode(&b.cumulative_difficulty)),
            None => ("0".to_string(), 0, "0".to_string()),
        };
        
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("application", "NRCS")
            .insert("version", "2.1.0")
            .insert("time", get_epoch_time())
            .insert("lastBlock", last_block_id)
            .insert("lastBlockHeight", last_block_height)
            .insert("cumulativeDifficulty", cumulative_difficulty)
            .insert("numberOfBlocks", last_block_height + 1)
            .insert("lastBlockchainFeeder", "")
            .insert("lastBlockchainFeederHeight", 0i32)
            .insert("isScanning", false)
            .insert("isDownloading", false)
            .insert("maxRollback", 1440i32)
            .insert("currentMinRollbackHeight", 0i32)
            .insert("isTestnet", false)
            .insert("maxPrunableLifetime", 1209600i32)
            .insert("includeExpiredPrunable", false)
            .insert("correctInvalidFees", false)
            .insert("ledgerTrimKeep", 0i32)
            .insert("services", serde_json::json!([]))
            .insert("apiProxy", false)
            .insert("isLightClient", false)
            .insert("maxAPIRecords", 100i32)
            .insert("blockchainState", "UP_TO_DATE");
        
        Ok(builder.build())
    }
}

pub struct GetTimeHandler;

impl GetTimeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetTimeHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Info]
    }
    
    fn allow_required_block_parameters(&self) -> bool {
        false
    }
    
    fn require_blockchain(&self) -> bool {
        false
    }
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("time", get_epoch_time());
        
        Ok(builder.build())
    }
}

pub struct GetStateHandler;

impl GetStateHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetStateHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Info]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let include_counts = req.get_bool("includeCounts");
        
        let latest_block = state.block_repo
            .find_latest()
            .await
            .map_err(ApiError::Repository)?;
        
        let (last_block_id, last_block_height, cumulative_difficulty) = match latest_block {
            Some(b) => (b.id.to_string(), b.height, hex::encode(&b.cumulative_difficulty)),
            None => ("0".to_string(), 0, "0".to_string()),
        };
        
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("application", "NRCS")
            .insert("version", "2.1.0")
            .insert("time", get_epoch_time())
            .insert("lastBlock", last_block_id)
            .insert("cumulativeDifficulty", cumulative_difficulty)
            .insert("numberOfBlocks", last_block_height + 1);
        
        if include_counts {
            builder
                .insert("numberOfTransactions", 0i32)
                .insert("numberOfAccounts", 0i32)
                .insert("numberOfAssets", 0i32)
                .insert("numberOfOrders", 0i32);
        }
        
        Ok(builder.build())
    }
}

pub struct GetConstantsHandler;

impl GetConstantsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetConstantsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Info]
    }
    
    fn allow_required_block_parameters(&self) -> bool {
        false
    }
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("maxBlockPayloadLength", 32760i32)
            .insert("maxArbitraryMessageLength", 1000i32)
            .insert("transactionTypes", get_transaction_types())
            .insert("peerStates", get_peer_states());
        
        Ok(builder.build())
    }
}

fn get_epoch_time() -> i32 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    
    let genesis_timestamp: i64 = 1514764800;
    ((now.as_secs() as i64) - genesis_timestamp) as i32
}

fn get_transaction_types() -> serde_json::Value {
    serde_json::json!({
        "0": {
            "name": "Payment",
            "subtypes": {
                "0": { "name": "Ordinary Payment" }
            }
        },
        "1": {
            "name": "Messaging",
            "subtypes": {
                "0": { "name": "Arbitrary Message" },
                "1": { "name": "Alias Assignment" },
                "2": { "name": "Poll Creation" },
                "3": { "name": "Vote Casting" },
                "4": { "name": "Hub Announcement" },
                "5": { "name": "Account Info" },
                "6": { "name": "Alias Sell" },
                "7": { "name": "Alias Buy" },
                "8": { "name": "Alias Delete" },
                "9": { "name": "Phasing Vote Casting" },
                "10": { "name": "Account Property" },
                "11": { "name": "Account Property Delete" },
                "12": { "name": "Account Long Value Property" }
            }
        },
        "2": {
            "name": "Asset Issuance",
            "subtypes": {
                "0": { "name": "Asset Issuance" },
                "1": { "name": "Asset Transfer" },
                "2": { "name": "Ask Order Placement" },
                "3": { "name": "Bid Order Placement" },
                "4": { "name": "Ask Order Cancellation" },
                "5": { "name": "Bid Order Cancellation" },
                "6": { "name": "Dividend Payment" },
                "7": { "name": "Asset Delete" },
                "8": { "name": "Asset Increase" },
                "10": { "name": "Property Set" },
                "12": { "name": "Long Value Property Set" },
                "11": { "name": "Property Delete" }
            }
        },
        "3": {
            "name": "Digital Goods",
            "subtypes": {
                "0": { "name": "Listing" },
                "1": { "name": "Delisting" },
                "2": { "name": "Price Change" },
                "3": { "name": "Quantity Change" },
                "4": { "name": "Purchase" },
                "5": { "name": "Delivery" },
                "6": { "name": "Feedback" },
                "7": { "name": "Refund" }
            }
        },
        "4": {
            "name": "Account Control",
            "subtypes": {
                "0": { "name": "Effective Balance Leasing" },
                "1": { "name": "Phasing Only" }
            }
        },
        "5": {
            "name": "Monetary System",
            "subtypes": {
                "0": { "name": "Currency Issuance" },
                "1": { "name": "Reserve Increase" },
                "2": { "name": "Reserve Claim" },
                "3": { "name": "Currency Transfer" },
                "4": { "name": "Publish Exchange Offer" },
                "5": { "name": "Exchange Buy" },
                "6": { "name": "Exchange Sell" },
                "7": { "name": "Currency Minting" },
                "8": { "name": "Currency Deletion" }
            }
        },
        "6": {
            "name": "Data",
            "subtypes": {
                "0": { "name": "Tagged Data Upload" },
                "1": { "name": "Tagged Data Extend" }
            }
        },
        "7": {
            "name": "Shuffling",
            "subtypes": {
                "0": { "name": "Shuffling Creation" },
                "1": { "name": "Shuffling Processing" },
                "2": { "name": "Shuffling Verification" },
                "3": { "name": "Shuffling Cancel" },
                "4": { "name": "Shuffling Registration" }
            }
        },
        "8": {
            "name": "Aliases",
            "subtypes": {}
        },
        "9": {
            "name": "Voting",
            "subtypes": {}
        },
        "10": {
            "name": "Account Property",
            "subtypes": {}
        },
        "11": {
            "name": "Coin Exchange",
            "subtypes": {}
        },
        "12": {
            "name": "Light Contract",
            "subtypes": {}
        }
    })
}

fn get_peer_states() -> serde_json::Value {
    serde_json::json!({
        "0": "NonConnected",
        "1": "Connected",
        "2": "Disconnected"
    })
}
