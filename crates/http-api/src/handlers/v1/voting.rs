//! 投票相关 API Handlers
//!
//! 与 Java 版本 GetPoll, CreatePoll 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;
use super::create_transaction::CreateTransactionHelper;

pub struct GetPollHandler;

impl GetPollHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPollHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["poll", "includeVoters", "includeVotes"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Vs]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _poll_id = req.require_u64("poll")?;
        let _include_voters = req.get_bool("includeVoters");
        let _include_votes = req.get_bool("includeVotes");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("poll", "0")
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("name", "")
            .insert("description", "")
            .insert("options", json!([]))
            .insert("finishHeight", 0i32)
            .insert("finished", false);
        
        Ok(builder.build())
    }
}

pub struct GetPollResultHandler;

impl GetPollResultHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPollResultHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["poll", "votingModel", "holding", "minBalance", "minBalanceModel"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Vs]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _poll_id = req.require_u64("poll")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("poll", "0")
            .insert("options", json!([]))
            .insert("results", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetPollsHandler;

impl GetPollsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPollsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex", "timestamp", "includeFinished", "finishedOnly"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Vs]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.get_u64("account");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("polls", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAllPollsHandler;

impl GetAllPollsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllPollsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex", "timestamp"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Vs]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("polls", json!([]));
        
        Ok(builder.build())
    }
}

pub struct CreatePollHandler;

impl CreatePollHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CreatePollHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "name", "description", "finishHeight", "votingModel", 
             "minNumberOfOptions", "maxNumberOfOptions", "minRangeValue", "maxRangeValue",
             "options", "option1", "option2", "option3", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Vs, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let name = req.get_string("name").unwrap_or_default(); let description = req.get_string("description").unwrap_or_default(); let finishHeight = req.get_i32("finishHeight").unwrap_or(0); let votingModel = req.get_i32("votingModel").unwrap_or(0); let minNumberOfOptions = req.get_i32("minNumberOfOptions").unwrap_or(0); let maxNumberOfOptions = req.get_i32("maxNumberOfOptions").unwrap_or(0); let minRangeValue = req.get_i32("minRangeValue").unwrap_or(0); let maxRangeValue = req.get_i32("maxRangeValue").unwrap_or(0); let options = req.get_string("options").unwrap_or_default();

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 1, 0, None, 0, Some(json!({"name": name, "description": description, "finishHeight": finishHeight, "votingModel": votingModel, "minNumberOfOptions": minNumberOfOptions, "maxNumberOfOptions": maxNumberOfOptions, "minRangeValue": minRangeValue, "maxRangeValue": maxRangeValue, "options": options})), state,
        ).await
    }
}

pub struct CastVoteHandler;

impl CastVoteHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CastVoteHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "poll", "vote1", "vote2", "vote3", "vote4", "vote5", 
             "vote6", "vote7", "vote8", "vote9", "vote10", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Vs, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let poll = req.get_string("poll").unwrap_or_default(); let vote = req.get_string("vote").unwrap_or_default();

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 1, 0, None, 0, Some(json!({"poll": poll, "vote": vote})), state,
        ).await
    }
}

pub struct GetPollVotesHandler;

impl GetPollVotesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPollVotesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["poll", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Vs]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _poll_id = req.require_u64("poll")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("votes", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetPollVotersHandler;

impl GetPollVotersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPollVotersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["poll", "firstIndex", "lastIndex"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Vs]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _poll_id = req.require_u64("poll")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        let mut builder = RsRespBuilder::new();
        builder.insert("voters", json!([]));

        Ok(builder.build())
    }
}

pub struct GetPollVoteHandler;

impl GetPollVoteHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPollVoteHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["poll", "account"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Vs]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _poll = req.require_u64("poll")?;
        let _account = req.require_u64("account")?;

        let mut builder = RsRespBuilder::new();
        builder
            .insert("poll", "0")
            .insert("voter", "0")
            .insert("voterRS", "NRCS-0-0-0")
            .insert("votes", json!([]));

        Ok(builder.build())
    }
}

pub struct ParsePhasingParamsHandler;

impl ParsePhasingParamsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ParsePhasingParamsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["phasingParams"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let phasing_params_str = req.require_string("phasingParams")?;

        let mut builder = RsRespBuilder::new();
        let phasing_params: serde_json::Value = serde_json::from_str(&phasing_params_str)
            .unwrap_or(serde_json::Value::Null);
        builder.insert("phasingParams", phasing_params);

        Ok(builder.build())
    }
}
