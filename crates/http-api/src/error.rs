//! API 错误处理
//!
//! 与 Java 版本错误码对齐

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("internal server error: {0}")]
    Internal(String),

    #[error("missing parameter: {0}")]
    MissingParameter(String),

    #[error("incorrect value: {0}")]
    IncorrectValue(String),

    #[error("unknown account")]
    UnknownAccount,

    #[error("unknown block")]
    UnknownBlock,

    #[error("unknown transaction")]
    UnknownTransaction,

    #[error("incorrect account")]
    IncorrectAccount,

    #[error("incorrect block")]
    IncorrectBlock,

    #[error("incorrect height")]
    IncorrectHeight,

    #[error("incorrect timestamp")]
    IncorrectTimestamp,

    #[error("blockchain error: {0}")]
    Blockchain(#[from] blockchain_types::BlockchainError),

    #[error("repository error: {0}")]
    Repository(#[from] orm::RepositoryError),

    #[error("account error: {0}")]
    Account(#[from] account::AccountError),

    #[error("tx engine error: {0}")]
    TxEngine(#[from] tx_engine::ProcessorError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl ApiError {
    pub fn error_code(&self) -> i32 {
        match self {
            ApiError::Validation(_) => 4,
            ApiError::NotFound(_) => 6,
            ApiError::Unauthorized(_) => 2,
            ApiError::Internal(_) => 4,
            ApiError::MissingParameter(_) => 12,
            ApiError::IncorrectValue(_) => 13,
            ApiError::UnknownAccount => 5,
            ApiError::UnknownBlock => 6,
            ApiError::UnknownTransaction => 7,
            ApiError::IncorrectAccount => 8,
            ApiError::IncorrectBlock => 9,
            ApiError::IncorrectHeight => 10,
            ApiError::IncorrectTimestamp => 11,
            ApiError::Blockchain(_) => 4,
            ApiError::Repository(_) => 4,
            ApiError::Account(_) => 4,
            ApiError::TxEngine(_) => 4,
            ApiError::Io(_) => 4,
        }
    }
}

/// API 响应信封（统一格式）
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: Option<T>) -> Self {
        Self {
            code: 0,
            message: "success".to_string(),
            data,
        }
    }

    pub fn error(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::error(400, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::error(404, message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::error(500, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, 400, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, 404, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, 401, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, 500, msg),
            ApiError::Blockchain(e) => (StatusCode::BAD_REQUEST, 400, e.to_string()),
            ApiError::Repository(e) => (StatusCode::INTERNAL_SERVER_ERROR, 500, e.to_string()),
            ApiError::Account(e) => (StatusCode::BAD_REQUEST, 400, e.to_string()),
            ApiError::TxEngine(e) => (StatusCode::BAD_REQUEST, 400, e.to_string()),
            ApiError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, 500, e.to_string()),
        };

        let body = Json(ApiResponse::<()>::error(code, message));
        (status, body).into_response()
    }
}

/// 结果类型别名
pub type ApiResult<T> = Result<T, ApiError>;

/// 从 (StatusCode, Json) 创建响应的便捷函数
pub fn response<T: Serialize>(status: StatusCode, data: ApiResponse<T>) -> impl IntoResponse {
    (status, Json(data))
}