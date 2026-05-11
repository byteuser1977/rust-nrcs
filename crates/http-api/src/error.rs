//! API 错误处理
//!
//! 与 Java 版本错误码对齐

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

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

    #[error("at least one of [{params}] must be specified")]
    MissingParameters { params: String },

    #[error("incorrect \"{0}\"")]
    IncorrectParameter(String),

    #[error("incorrect \"{param}\" {details}")]
    IncorrectParameterWithDetails { param: String, details: String },

    #[error("incorrect value: {0}")]
    IncorrectValue(String),

    #[error("unknown {0}")]
    UnknownObject(String),

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

    #[error("incorrect peer address")]
    IncorrectPeerAddress,

    #[error("not more than one of [{params}] can be specified")]
    EitherParameter { params: String },

    #[error("not yet available: {0}")]
    NotYetAvailable(String),

    #[error("feature not available: {0}")]
    FeatureNotAvailable(String),

    #[error("not enabled: {0}")]
    NotEnabled(String),

    #[error("pruned transaction data not available")]
    PrunedTransactionDataNotAvailable,

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
            ApiError::NotFound(_) => 5,
            ApiError::Unauthorized(_) => 2,
            ApiError::Internal(_) => 4,
            ApiError::MissingParameter(_) => 3,
            ApiError::MissingParameters { .. } => 3,
            ApiError::IncorrectParameter(_) => 4,
            ApiError::IncorrectParameterWithDetails { .. } => 4,
            ApiError::IncorrectValue(_) => 4,
            ApiError::UnknownObject(_) => 5,
            ApiError::UnknownAccount => 5,
            ApiError::UnknownBlock => 5,
            ApiError::UnknownTransaction => 5,
            ApiError::IncorrectAccount => 4,
            ApiError::IncorrectBlock => 4,
            ApiError::IncorrectHeight => 4,
            ApiError::IncorrectTimestamp => 4,
            ApiError::IncorrectPeerAddress => 4,
            ApiError::EitherParameter { .. } => 6,
            ApiError::NotYetAvailable(_) => 7,
            ApiError::FeatureNotAvailable(_) => 8,
            ApiError::NotEnabled(_) => 9,
            ApiError::PrunedTransactionDataNotAvailable => 15,
            ApiError::Blockchain(_) => 4,
            ApiError::Repository(_) => 4,
            ApiError::Account(_) => 4,
            ApiError::TxEngine(_) => 4,
            ApiError::Io(_) => 4,
        }
    }

    pub fn error_description(&self) -> String {
        match self {
            ApiError::MissingParameter(p) => format!("\"{}\" not specified", p),
            ApiError::MissingParameters { params } => format!("At least one of [{}] must be specified", params),
            ApiError::IncorrectParameter(p) => format!("Incorrect \"{}\"", p),
            ApiError::IncorrectParameterWithDetails { param, details } => format!("Incorrect \"{}\" {}", param, details),
            ApiError::IncorrectValue(p) => format!("Incorrect \"{}\"", p),
            ApiError::UnknownObject(o) => format!("Unknown {}", o),
            ApiError::EitherParameter { params } => format!("Not more than one of [{}] can be specified", params),
            ApiError::NotYetAvailable(msg) => format!("Not yet available: {}", msg),
            ApiError::FeatureNotAvailable(msg) => format!("Feature not available: {}", msg),
            ApiError::NotEnabled(msg) => format!("Not enabled: {}", msg),
            ApiError::PrunedTransactionDataNotAvailable => "Pruned transaction data not available".to_string(),
            ApiError::UnknownAccount => "Unknown account".to_string(),
            ApiError::UnknownBlock => "Unknown block".to_string(),
            ApiError::UnknownTransaction => "Unknown transaction".to_string(),
            ApiError::IncorrectAccount => "Incorrect account".to_string(),
            ApiError::IncorrectBlock => "Incorrect block".to_string(),
            ApiError::IncorrectHeight => "Incorrect height".to_string(),
            ApiError::IncorrectTimestamp => "Incorrect timestamp".to_string(),
            ApiError::IncorrectPeerAddress => "Incorrect peer address".to_string(),
            ApiError::Validation(msg) => msg.clone(),
            ApiError::NotFound(msg) => format!("Unknown {}", msg),
            ApiError::Unauthorized(msg) => msg.clone(),
            ApiError::Internal(msg) => msg.clone(),
            ApiError::Blockchain(e) => e.to_string(),
            ApiError::Repository(e) => e.to_string(),
            ApiError::Account(e) => e.to_string(),
            ApiError::TxEngine(e) => e.to_string(),
            ApiError::Io(e) => e.to_string(),
        }
    }
}

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
        let status = match &self {
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::NotFound(_) | ApiError::UnknownObject(_) | ApiError::UnknownAccount
            | ApiError::UnknownBlock | ApiError::UnknownTransaction => StatusCode::NOT_FOUND,
            ApiError::Repository(_) | ApiError::Io(_) | ApiError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            _ => StatusCode::BAD_REQUEST,
        };
        let code = self.error_code();
        let message = self.error_description();
        let body = Json(ApiResponse::<()>::error(code, message));
        (status, body).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

pub fn response<T: Serialize>(status: StatusCode, data: ApiResponse<T>) -> impl IntoResponse {
    (status, Json(data))
}
