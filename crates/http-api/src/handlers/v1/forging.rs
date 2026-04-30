//! 锻造相关 API Handlers
//!
//! 与 Java 版本 StartForging, StopForging 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct StartForgingHandler;

impl StartForgingHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for StartForgingHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Forging]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let secret_phrase = req.require_string("secretPhrase")?;

        let forging = state.forging_service.as_ref()
            .ok_or_else(|| ApiError::Internal("Forging service not available".to_string()))?;

        forging.start_forging(&secret_phrase).await
            .map_err(|e| ApiError::Validation(e))?;

        // 获取锻造者信息
        let seed = crypto::sha256(secret_phrase.as_bytes());
        let kp = crypto::keypair_from_seed(&seed);
        let public_key = kp.public_key();
        let pk_bytes = public_key.as_bytes();
        let account_id = crypto::account_id_from_public_key(pk_bytes);

        let forgers = forging.get_forgers();
        let forger = forgers.iter().find(|f| f.account_id == account_id);

        let mut builder = RsRespBuilder::new();
        builder
            .insert("account", account_id.to_string())
            .insert("accountRS", format_account_rs(account_id))
            .insert("deadline", forger.map(|f| f.deadline as i64).unwrap_or(0))
            .insert("hitTime", forger.map(|f| f.hit_time as i64).unwrap_or(0));

        Ok(builder.build())
    }
}

pub struct StopForgingHandler;

impl StopForgingHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for StopForgingHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Forging]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let secret_phrase = req.require_string("secretPhrase")?;

        let forging = state.forging_service.as_ref()
            .ok_or_else(|| ApiError::Internal("Forging service not available".to_string()))?;

        // 计算 account_id 用于返回
        let seed = crypto::sha256(secret_phrase.as_bytes());
        let kp = crypto::keypair_from_seed(&seed);
        let public_key = kp.public_key();
        let pk_bytes = public_key.as_bytes();
        let account_id = crypto::account_id_from_public_key(pk_bytes);

        forging.stop_forging(&secret_phrase).await
            .map_err(|e| ApiError::Validation(e))?;

        let mut builder = RsRespBuilder::new();
        builder
            .insert("account", account_id.to_string())
            .insert("accountRS", format_account_rs(account_id))
            .insert("stopped", true);

        Ok(builder.build())
    }
}

pub struct GetForgingHandler;

impl GetForgingHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetForgingHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "adminPassword"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Forging]
    }

    async fn process_request(&self, _req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let forging = state.forging_service.as_ref()
            .ok_or_else(|| ApiError::Internal("Forging service not available".to_string()))?;

        let forgers = forging.get_forgers();

        let forger_values: Vec<serde_json::Value> = forgers.iter().map(|f| {
            json!({
                "account": f.account_id.to_string(),
                "accountRS": format_account_rs(f.account_id),
                "effectiveBalanceNRCS": f.effective_balance,
                "hitTime": f.hit_time,
                "deadline": f.deadline
            })
        }).collect();

        let mut builder = RsRespBuilder::new();
        builder.insert("forgers", json!(forger_values));

        Ok(builder.build())
    }
}

pub struct GetNextBlockGeneratorsHandler;

impl GetNextBlockGeneratorsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetNextBlockGeneratorsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["limit"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Forging]
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let limit = req.get_i32("limit").unwrap_or(10) as usize;

        let latest_block = state.block_repo
            .find_latest()
            .await
            .map_err(ApiError::Repository)?;

        let (last_block_id, last_block_height, timestamp) = match latest_block {
            Some(b) => (b.id.to_string(), b.height, b.timestamp),
            None => ("0".to_string(), 0, 0),
        };

        let forging = state.forging_service.as_ref();
        let generators: Vec<serde_json::Value> = if let Some(f) = forging {
            let forgers = f.get_forgers();
            forgers.into_iter().take(limit).map(|g| {
                json!({
                    "account": g.account_id.to_string(),
                    "accountRS": format_account_rs(g.account_id),
                    "effectiveBalanceNRCS": g.effective_balance,
                    "hitTime": g.hit_time,
                    "deadline": g.deadline
                })
            }).collect()
        } else {
            vec![]
        };

        let mut builder = RsRespBuilder::new();
        builder
            .insert("lastBlock", last_block_id)
            .insert("height", last_block_height)
            .insert("generators", json!(generators));

        Ok(builder.build())
    }
}

fn format_account_rs(account_id: u64) -> String {
    format!("NRCS-{}-{}-{}",
        account_id % 10000,
        (account_id / 10000) % 10000,
        (account_id / 100000000) % 10000
    )
}
