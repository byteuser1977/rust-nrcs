use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct SetContractReferenceHandler;

impl SetContractReferenceHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SetContractReferenceHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![
            "secretPhrase",
            "contractName",
            "contractParams",
            "contract",
            "feeNQT",
            "deadline",
            "publicKey",
            "broadcast",
        ]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::CreateTransaction]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let contract_name = req.require_string("contractName")?;
        let _contract_params = req.get_string("contractParams").unwrap_or_default();
        let _contract = req.get_string("contract");

        if contract_name.is_empty() {
            return Err(ApiError::Validation("contractName cannot be empty".to_string()));
        }

        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("transactionBytes", "");

        Ok(builder.build())
    }
}

pub struct DeleteContractReferenceHandler;

impl DeleteContractReferenceHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DeleteContractReferenceHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![
            "secretPhrase",
            "contractName",
            "feeNQT",
            "deadline",
            "publicKey",
            "broadcast",
        ]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::CreateTransaction]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let contract_name = req.require_string("contractName")?;

        if contract_name.is_empty() {
            return Err(ApiError::MissingParameter("contractName".to_string()));
        }

        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("transactionBytes", "");

        Ok(builder.build())
    }
}

pub struct GetContractReferencesHandler;

impl GetContractReferencesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetContractReferencesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![
            "account",
            "contractName",
            "includeContract",
            "firstIndex",
            "lastIndex",
        ]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;
        let _contract_name = req.get_string("contractName");
        let _include_contract = req.get_bool("includeContract");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        let mut builder = RsRespBuilder::new();
        builder
            .insert("account", account_id.to_string())
            .insert("contractReferences", json!([]));

        Ok(builder.build())
    }
}
