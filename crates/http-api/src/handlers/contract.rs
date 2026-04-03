//! 合约相关处理器

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::error::ApiResult;

/// 合约部署请求
#[derive(Debug, Deserialize)]
pub struct DeployContractRequest {
    pub sender_id: AccountId,
    pub bytecode: Vec<u8>,
    pub gas_limit: u64,
    pub initial_data: Option<serde_json::Value>,
}

/// 合约调用请求
#[derive(Debug, Deserialize)]
pub struct CallContractRequest {
    pub sender_id: AccountId,
    pub contract_address: String,
    pub method: String,
    pub args: serde_json::Value,
    pub gas_limit: u64,
}

/// 获取合约信息响应
#[derive(Debug, Serialize)]
pub struct ContractResponse {
    pub contract_address: String,
    pub creator_id: AccountId,
    pub code_hash: String,
    pub storage: serde_json::Value,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub created_at: u64,
}

/// 部署合约
pub async fn deploy_contract(
    State(state): State<ApiState>,
    Json(req): Json<DeployContractRequest>,
) -> ApiResult<Json<ContractResponse>> {
    // TODO: 调用合约引擎部署
    // let contract_addr = contract_engine.deploy(req.bytecode, req.sender_id, req.gas_limit, req.initial_data)?;

    return Err(crate::error::ApiError::Internal("not implemented".to_string()));
}

/// 调用合约
pub async fn call_contract(
    State(state): State<ApiState>,
    Path(address): Path<String>,
    Json(req): Json<CallContractRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: 调用合约引擎
    return Err(crate::error::ApiError::Internal("not implemented".to_string()));
}

/// 查询合约信息
pub async fn get_contract(
    State(state): State<ApiState>,
    Path(address): Path<String>,
) -> ApiResult<Json<ContractResponse>> {
    // TODO: 从数据库查询
    return Err(crate::error::ApiError::NotFound("contract not found".to_string()));
}