//! P2P 协议消息定义

use super::blockchain_types::*;
use serde::{Deserialize, Serialize};

/// P2P 协议协议标识符（用于 libp2p 的 protocol ID）
pub const BLOCK_PROTOCOL: &str = "/nrcs/block/1.0.0";
pub const TX_PROTOCOL: &str = "/nrcs/tx/1.0.0";
pub const PEER_PROTOCOL: &str = "/nrcs/peer/1.0.0";

/// P2P 网络消息枚举
///
/// 所有消息都应该是可序列化的，使用 `bincode` 编码。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2PMessage {
    /// 区块广播（Announce）
    /// 当节点接收到新区块时，向邻居广播区块头部元信息。
    BlockAnnounce(BlockAnnounce),

    /// 区块请求（Request full block data）
    BlockRequest(BlockRequest),

    /// 区块响应（Response full block）
    BlockResponse(BlockResponse),

    /// 交易广播（Broadcast transaction）
    TransactionBroadcast(TransactionBroadcast),

    /// 交易请求（Request transactions from pool）
    TransactionsRequest(TransactionsRequest),

    /// 交易响应
    TransactionsResponse(TransactionsResponse),

    /// 节点列表请求
    PeersRequest(PeersRequest),

    /// 节点列表响应
    PeersResponse(PeersResponse),

    /// Ping（心跳）
    Ping(Ping),

    /// Pong
    Pong(Pong),
}

/// 区块公告（轻量级头部信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockAnnounce {
    /// 区块 ID
    pub block_id: BlockId,
    /// 区块高度
    pub height: Height,
    /// 区块哈希
    pub hash: Hash256,
    /// 出块者
    pub generator_id: AccountId,
    /// 时间戳
    pub timestamp: Timestamp,
    /// 交易数量
    pub tx_count: u32,
    /// 总金额
    pub total_amount: Amount,
    /// 总手续费
    pub total_fee: Amount,
}

/// 区块请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRequest {
    /// 请求的区块高度
    pub height: Height,
    /// 请求的区块哈希（校验用）
    pub hash: Option<Hash256>,
}

/// 区块响应（完整区块数据）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResponse {
    /// 区块数据（如果不存在则为 None）
    pub block: Option<Block>,
    /// 错误信息（如果失败）
    pub error: Option<String>,
}

/// 交易广播
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBroadcast {
    /// 交易列表（通常单次只广播一个）
    pub transactions: Vec<Transaction>,
}

/// 交易请求（从内存池获取未确认交易）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsRequest {
    /// 请求者公钥（用于过滤自己已知的交易）
    /// 可选
    pub since_tx_id: Option<TransactionId>,
    /// 最多返回数量
    pub limit: u32,
}

/// 交易响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsResponse {
    /// 交易列表
    pub transactions: Vec<Transaction>,
    /// 是否还有更多
    pub has_more: bool,
}

/// 节点列表请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeersRequest {
    /// 请求数量上限
    pub limit: u32,
    /// 排除已知节点列表
    pub exclude: Vec<PeerId>,
}

/// 节点列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeersResponse {
    /// 节点信息列表（简化版）
    pub peers: Vec<PeerInfo>,
}

/// 节点信息（简化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// 节点 ID（libp2p PeerId）
    pub id: String,
    /// 节点地址列表（multiaddr 字符串）
    pub addresses: Vec<String>,
    /// 支持的协议列表
    pub protocols: Vec<String>,
    /// 节点版本
    pub version: String,
    /// 最后看到的时间戳
    pub last_seen: Timestamp,
}

/// Ping 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ping {
    /// 发送时间戳
    pub sent: Timestamp,
    /// 随机数（防缓存）
    pub nonce: u64,
}

/// Pong 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pong {
    /// 响应时间
    pub received: Timestamp,
    /// 对应 Ping 的 nonce
    pub nonce: u64,
}

/// Peer ID 类型别名（libp2p 的 PeerId）
pub type PeerId = libp2p::PeerId;

impl P2PMessage {
    /// 创建区块广播消息
    pub fn block_announce(announce: BlockAnnounce) -> Self {
        Self::BlockAnnounce(announce)
    }

    /// 创建交易广播消息
    pub fn transaction_broadcast(txs: Vec<Transaction>) -> Self {
        Self::TransactionBroadcast(TransactionBroadcast { transactions: txs })
    }

    /// 创建 Peer 请求
    pub fn peers_request(limit: u32) -> Self {
        Self::PeersRequest(PeersRequest { limit, exclude: vec![] })
    }
}

/// 消息编解码辅助函数
pub mod codec {
    use super::*;
    use bincode::{ serialize, deserialize };
    use std::io;

    /// 编码消息为二进制
    pub fn encode(msg: &P2PMessage) -> io::Result<Vec<u8>> {
        serialize(msg).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    /// 解码二进制消息
    pub fn decode(data: &[u8]) -> io::Result<P2PMessage> {
        deserialize(data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

#[cfg(test)]
mod protocol_tests {
    use super::*;

    #[test]
    fn test_block_announce_roundtrip() {
        let announce = BlockAnnounce {
            block_id: 100,
            height: 100,
            hash: [1u8; 32],
            generator_id: 123456,
            timestamp: 1_700_000_000,
            tx_count: 10,
            total_amount: 1_000_000_000,
            total_fee: 100_000,
        };

        let msg = P2PMessage::BlockAnnounce(announce.clone());
        let encoded = bincode::serialize(&msg).unwrap();
        let decoded: P2PMessage = bincode::deserialize(&encoded).unwrap();

        match decoded {
            P2PMessage::BlockAnnounce(a) => {
                assert_eq!(a.block_id, announce.block_id);
                assert_eq!(a.height, announce.height);
            }
            _ => panic!("wrong message type"),
        }
    }

    #[test]
    fn test_transaction_broadcast_roundtrip() {
        let tx = Transaction::new(
            TransactionType::Payment,
            123, Some(456), 1000, 10, 1_700_000_000, 32767
        );
        let msg = P2PMessage::transaction_broadcast(vec![tx.clone()]);
        let encoded = bincode::serialize(&msg).unwrap();
        let decoded: P2PMessage = bincode::deserialize(&encoded).unwrap();

        match decoded {
            P2PMessage::TransactionBroadcast(resp) => {
                assert_eq!(resp.transactions.len(), 1);
                assert_eq!(resp.transactions[0].sender_id, 123);
            }
            _ => panic!("wrong message type"),
        }
    }
}
