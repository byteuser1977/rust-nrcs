//! # P2P Networking Layer
//!
//! 基于 libp2p-rs 构建的去中心化网络通信层。
//!
//! ## 协议设计
//! 采用 libp2p 的 multiplexing 和多协议支持：
//! - `/nrcs/block/1.0.0`：区块广播协议
//! - `/nrcs/tx/1.0.0`：交易广播协议
//! - `/nrcs/peer/1.0.0`：节点发现协议
//!
//! ## Message Format
//! 所有消息使用 `bincode` 序列化，为节省带宽采用紧凑二进制格式。

mod protocol;
mod service;

pub use protocol::*;
pub use service::{P2PService, P2PError, P2PResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let announce = BlockAnnounce {
            block_id: 12345,
            height: 100,
            hash: [1u8; 32],
            total_amount: 1_000_000_000,
            total_fee: 50_000,
        };

        let encoded = bincode::serialize(&announce).unwrap();
        let decoded: BlockAnnounce = bincode::deserialize(&encoded).unwrap();
        assert_eq!(announce.block_id, decoded.block_id);
        assert_eq!(announce.height, decoded.height);
    }
}
