use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// P2P 协议帧头（对应 Java 二进制封装）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameHeader {
    pub version: i32,      // 版本 (1)
    pub request_id: i64,  // 请求 ID（客户端生成）
    pub flags: i32,       // 标志位（FLAG_COMPRESSED = 1）
    pub length: i32,      // 消息体长度
}

impl FrameHeader {
    pub fn new(version: i32, request_id: i64, flags: i32, length: i32) -> Self {
        Self {
            version,
            request_id,
            flags,
            length,
        }
    }

    pub fn is_compressed(&self) -> bool {
        (self.flags & 1) != 0
    }
}

/// 请求类型枚举（与 Java 完全匹配）
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum RequestType {
    GetInfo,
    GetPeers,
    AddPeers,
    GetCumulativeDifficulty,
    GetMilestoneBlockIds,
    GetNextBlockIds,
    GetNextBlocks,
    GetTransactions,
    GetUnconfirmedTransactions,
    ProcessBlock,
    ProcessTransactions,
    BundlerRate,
    // 未来可能扩展
    Unknown(String),
}

impl From<&str> for RequestType {
    fn from(s: &str) -> Self {
        match s {
            "getInfo" => RequestType::GetInfo,
            "getPeers" => RequestType::GetPeers,
            "addPeers" => RequestType::AddPeers,
            "getCumulativeDifficulty" => RequestType::GetCumulativeDifficulty,
            "getMilestoneBlockIds" => RequestType::GetMilestoneBlockIds,
            "getNextBlockIds" => RequestType::GetNextBlockIds,
            "getNextBlocks" => RequestType::GetNextBlocks,
            "getTransactions" => RequestType::GetTransactions,
            "getUnconfirmedTransactions" => RequestType::GetUnconfirmedTransactions,
            "processBlock" => RequestType::ProcessBlock,
            "processTransactions" => RequestType::ProcessTransactions,
            "bundlerRate" => RequestType::BundlerRate,
            other => RequestType::Unknown(other.to_string()),
        }
    }
}

/// 基础请求（所有 RPC 共享字段）
#[derive(Debug, Serialize, Deserialize)]
pub struct PeerRequest {
    #[serde(rename = "requestType")]
    pub request_type: RequestType,
    #[serde(rename = "protocol")]
    pub protocol: i32, // 必须是 1 或 2

    // 其他字段使用 serde_json::Value 动态处理
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl PeerRequest {
    pub fn new(request_type: RequestType, protocol: i32) -> Self {
        Self {
            request_type,
            protocol,
            extra: HashMap::new(),
        }
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.extra.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn set<T: Serialize>(&mut self, key: &str, value: T) {
        if let Ok(v) = serde_json::to_value(value) {
            self.extra.insert(key.to_string(), v);
        }
    }
}

/// 基础响应
#[derive(Debug, Serialize, Deserialize)]
pub struct PeerResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    // 成功时的数据字段（根据 requestType 不同而不同）
    #[serde(flatten)]
    pub data: Option<serde_json::Value>,
}

impl PeerResponse {
    pub fn success(data: Option<serde_json::Value>) -> Self {
        Self { error: None, data }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            error: Some(msg.to_string()),
            data: None,
        }
    }

    pub fn unsupported_request_type() -> Self {
        Self::error("UNSUPPORTED_REQUEST_TYPE")
    }

    pub fn unsupported_protocol() -> Self {
        Self::error("UNSUPPORTED_PROTOCOL")
    }

    pub fn sequence_error() -> Self {
        Self::error("SEQUENCE_ERROR")
    }

    pub fn downloading() -> Self {
        Self::error("DOWNLOADING")
    }

    pub fn max_inbound_connections() -> Self {
        Self::error("MAX_INBOUND_CONNECTIONS")
    }

    pub fn blacklisted() -> Self {
        Self::error("BLACKLISTED")
    }
}

/// 帧编码器/解码器
pub struct FrameCodec;

impl FrameCodec {
    const MAGIC: [u8; 4] = [0x50, 0x32, 0x50, 0x00]; // "P2P\0"

    pub fn encode(&self, payload: &[u8], compressed: bool) -> Vec<u8> {
        let mut flags = 0;
        let body = if compressed {
            flags |= 1;
            self::compress_gzip(payload)
        } else {
            payload.to_vec()
        };

        let length = body.len() as i32;
        let mut buf = Vec::new();

        // Magic + Version(4) + RequestID(8) + Flags(4) + Length(4) + Body
        buf.extend(&Self::MAGIC);
        buf.extend(&1i32.to_be_bytes()); // version = 1
        buf.extend(&0i64.to_be_bytes()); // request_id (占位)
        buf.extend(&flags.to_be_bytes());
        buf.extend(&length.to_be_bytes());
        buf.extend(body);

        buf
    }

    pub fn decode(&self, data: &[u8]) -> Result<(FrameHeader, Vec<u8>), ProtocolError> {
        if data.len() < 20 {
            return Err(ProtocolError::InvalidFrame("Frame too short".into()));
        }

        if &data[0..4] != &Self::MAGIC {
            return Err(ProtocolError::InvalidFrame("Invalid magic bytes".into()));
        }

        let version = i32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let request_id = i64::from_be_bytes([
            data[8], data[9], data[10], data[11],
            data[12], data[13], data[14], data[15],
        ]);
        let flags = i32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let length = i32::from_be_bytes([data[20], data[21], data[22], data[23]]);

        let body_start = 24;
        let body_end = body_start + length as usize;

        if data.len() < body_end {
            return Err(ProtocolError::InvalidFrame("Incomplete frame".into()));
        }

        let mut body = data[body_start..body_end].to_vec();

        if flags & 1 != 0 {
            body = self::decompress_gzip(&body)?;
        }

        let header = FrameHeader {
            version,
            request_id,
            flags,
            length,
        };

        Ok((header, body))
    }
}

fn compress_gzip(data: &[u8]) -> Vec<u8> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)
        .map_err(|e| ProtocolError::DecompressionError(e.to_string()))?;

    Ok(decompressed)
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Invalid frame: {0}")]
    InvalidFrame(String),

    #[error("Decompression error: {0}")]
    DecompressionError(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}