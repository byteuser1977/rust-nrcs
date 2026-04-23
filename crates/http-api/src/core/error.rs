//! API Error Handling
//!
//! 统一的 API 错误处理

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// API 错误类型
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// API 错误响应
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    #[serde(rename = "errorCode")]
    pub error_code: u16,
    #[serde(rename = "errorDescription")]
    pub error_description: String,
    #[serde(rename = "requestProcessingTime")]
    pub request_processing_time: u32,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code) = match &self {
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, 1),
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, 5),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, 6),
            ApiError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, 7),
            ApiError::InvalidParameter(_) => (StatusCode::BAD_REQUEST, 2),
        };
        
        let body = Json(ErrorResponse {
            error_code,
            error_description: self.to_string(),
            request_processing_time: 1,
        });
        
        (status, body).into_response()
    }
}
