//! P2P Error Codes (对应 NRCS Errors.java)
//!
//! 所有错误码与 NRCS Java 版本一一对应

use serde::{Deserialize, Serialize};

/// P2P Error Codes
/// 
/// 对应 NRCS Java: Errors.java
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// 不支持的请求类型
    /// NRCS: UNSUPPORTED_REQUEST_TYPE
    UnsupportedRequestType = 1,

    /// 不支持的协议版本
    /// NRCS: UNSUPPORTED_PROTOCOL
    UnsupportedProtocol = 2,

    /// 未知节点
    /// NRCS: UNKNOWN_PEER
    UnknownPeer = 3,

    /// 序列错误
    /// NRCS: SEQUENCE_ERROR
    SequenceError = 4,

    /// 最大入站连接数已满
    /// NRCS: MAX_INBOUND_CONNECTIONS
    MaxInboundConnections = 5,

    /// 节点正在下载区块
    /// NRCS: DOWNLOADING
    Downloading = 6,

    /// 轻客户端不支持
    /// NRCS: LIGHT_CLIENT
    LightClient = 7,

    /// 无效的 JSON
    InvalidJson = 8,

    /// 连接超时
    ConnectionTimeout = 9,

    /// 读取超时
    ReadTimeout = 10,

    /// 黑名单节点
    Blacklisted = 11,

    /// 版本过旧
    OldVersion = 12,

    /// 无效的签名
    InvalidSignature = 13,

    /// 无效的区块
    InvalidBlock = 14,

    /// 无效的交易
    InvalidTransaction = 15,

    /// 数据库错误
    DatabaseError = 16,

    /// 内部错误
    InternalError = 99,
}

impl ErrorCode {
    /// Get error message
    pub fn message(&self) -> &'static str {
        match self {
            ErrorCode::UnsupportedRequestType => "Unsupported request type",
            ErrorCode::UnsupportedProtocol => "Unsupported protocol",
            ErrorCode::UnknownPeer => "Unknown peer",
            ErrorCode::SequenceError => "Sequence error",
            ErrorCode::MaxInboundConnections => "Max inbound connections reached",
            ErrorCode::Downloading => "Peer is downloading blockchain",
            ErrorCode::LightClient => "Light client not supported",
            ErrorCode::InvalidJson => "Invalid JSON",
            ErrorCode::ConnectionTimeout => "Connection timeout",
            ErrorCode::ReadTimeout => "Read timeout",
            ErrorCode::Blacklisted => "Peer is blacklisted",
            ErrorCode::OldVersion => "Old version",
            ErrorCode::InvalidSignature => "Invalid signature",
            ErrorCode::InvalidBlock => "Invalid block",
            ErrorCode::InvalidTransaction => "Invalid transaction",
            ErrorCode::DatabaseError => "Database error",
            ErrorCode::InternalError => "Internal error",
        }
    }
}

/// P2P Error
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct P2PError {
    /// Error code
    pub code: ErrorCode,
    /// Error message
    pub message: String,
    /// Error description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl P2PError {
    /// Create a new P2P error
    pub fn new(code: ErrorCode) -> Self {
        Self {
            code,
            message: code.message().to_string(),
            description: None,
        }
    }

    /// Create a new P2P error with custom message
    pub fn with_message(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            description: None,
        }
    }

    /// Create a new P2P error with description
    pub fn with_description(code: ErrorCode, description: impl Into<String>) -> Self {
        Self {
            code,
            message: code.message().to_string(),
            description: Some(description.into()),
        }
    }

    /// Create unsupported request type error
    pub fn unsupported_request_type() -> Self {
        Self::new(ErrorCode::UnsupportedRequestType)
    }

    /// Create unsupported protocol error
    pub fn unsupported_protocol() -> Self {
        Self::new(ErrorCode::UnsupportedProtocol)
    }

    /// Create unknown peer error
    pub fn unknown_peer() -> Self {
        Self::new(ErrorCode::UnknownPeer)
    }

    /// Create sequence error
    pub fn sequence_error() -> Self {
        Self::new(ErrorCode::SequenceError)
    }

    /// Create max inbound connections error
    pub fn max_inbound_connections() -> Self {
        Self::new(ErrorCode::MaxInboundConnections)
    }

    /// Create downloading error
    pub fn downloading() -> Self {
        Self::new(ErrorCode::Downloading)
    }

    /// Create light client error
    pub fn light_client() -> Self {
        Self::new(ErrorCode::LightClient)
    }

    /// Create invalid JSON error
    pub fn invalid_json(details: impl Into<String>) -> Self {
        Self::with_description(ErrorCode::InvalidJson, details)
    }

    /// Create connection timeout error
    pub fn connection_timeout() -> Self {
        Self::new(ErrorCode::ConnectionTimeout)
    }

    /// Create read timeout error
    pub fn read_timeout() -> Self {
        Self::new(ErrorCode::ReadTimeout)
    }

    /// Create blacklisted error
    pub fn blacklisted(reason: impl Into<String>) -> Self {
        Self::with_description(ErrorCode::Blacklisted, reason)
    }

    /// Create old version error
    pub fn old_version(version: impl Into<String>) -> Self {
        Self::with_description(ErrorCode::OldVersion, version)
    }

    /// Create internal error
    pub fn internal(details: impl Into<String>) -> Self {
        Self::with_description(ErrorCode::InternalError, details)
    }
}

impl std::fmt::Display for P2PError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.description {
            Some(desc) => write!(f, "{}: {}", self.message, desc),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for P2PError {}

/// Convert to JSON response format (NRCS compatible)
impl P2PError {
    /// Convert to JSON value for response
    pub fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("error".to_string(), serde_json::Value::String(self.message.clone()));
        map.insert("errorCode".to_string(), serde_json::Value::Number(serde_json::Number::from(self.code as i64)));
        if let Some(ref desc) = self.description {
            map.insert("errorDescription".to_string(), serde_json::Value::String(desc.clone()));
        }
        serde_json::Value::Object(map)
    }
}

/// P2P Result type
pub type P2PResult<T> = std::result::Result<T, P2PError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(ErrorCode::UnsupportedRequestType as i32, 1);
        assert_eq!(ErrorCode::UnsupportedProtocol as i32, 2);
        assert_eq!(ErrorCode::UnknownPeer as i32, 3);
        assert_eq!(ErrorCode::SequenceError as i32, 4);
        assert_eq!(ErrorCode::MaxInboundConnections as i32, 5);
        assert_eq!(ErrorCode::Downloading as i32, 6);
        assert_eq!(ErrorCode::LightClient as i32, 7);
    }

    #[test]
    fn test_error_messages() {
        assert_eq!(ErrorCode::UnsupportedRequestType.message(), "Unsupported request type");
        assert_eq!(ErrorCode::UnknownPeer.message(), "Unknown peer");
    }

    #[test]
    fn test_error_to_json() {
        let error = P2PError::unsupported_request_type();
        let json = error.to_json();
        
        assert_eq!(json["error"], "Unsupported request type");
        assert_eq!(json["errorCode"], 1);
    }

    #[test]
    fn test_error_with_description() {
        let error = P2PError::blacklisted("Spam detected");
        let json = error.to_json();
        
        assert_eq!(json["error"], "Peer is blacklisted");
        assert_eq!(json["errorCode"], 11);
        assert_eq!(json["errorDescription"], "Spam detected");
    }
}
