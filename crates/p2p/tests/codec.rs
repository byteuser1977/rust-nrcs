//! FrameCodec roundtrip tests for all P2P message types.

use p2p::codec::{FrameCodec, Message};
use tokio_util::codec::Decoder;
use bytes::BytesMut;
use std::error::Error;

#[tokio::test]
async fn test_getinfo_roundtrip() {
    let msg = Message::GetInfo;
    let mut buf = BytesMut::new();
    FrameCodec.encode(msg, &mut buf).unwrap();

    let mut decoder = FrameCodec;
    let decoded = decoder.decode(&mut buf).await.unwrap();
    assert!(matches!(decoded, Some(Message::GetInfo)));
}

#[tokio::test]
async fn test_getpeers_roundtrip() {
    let msg = Message::GetPeers;
    let mut buf = BytesMut::new();
    FrameCodec.encode(msg, &mut buf).unwrap();

    let mut decoder = FrameCodec;
    let decoded = decoder.decode(&mut buf).await.unwrap();
    assert!(matches!(decoded, Some(Message::GetPeers)));
}

#[tokio::test]
async fn test_addpeers_roundtrip() {
    let peers = vec!["1.2.3.4:1234".parse().unwrap()];
    let msg = Message::AddPeers(peers);
    let mut buf = BytesMut::new();
    FrameCodec.encode(msg, &mut buf).unwrap();

    let mut decoder = FrameCodec;
    let decoded = decoder.decode(&mut buf).await.unwrap();
    match decoded {
        Some(Message::AddPeers(decoded_peers)) => {
            assert_eq!(decoded_peers.len(), 1);
            assert_eq!(decoded_peers[0], "1.2.3.4:1234".parse().unwrap());
        }
        _ => panic!("Wrong variant"),
    }
}

#[tokio::test]
async fn test_processblock_roundtrip() {
    let block_msg = p2p::messages::BlockMessage {
        hash: "abcd".to_string(),
        previous_hash: None,
        timestamp: chrono::Utc::now(),
        nonce: None,
        difficulty: None,
        data: serde_json::json!({}),
    };
    let msg = Message::ProcessBlock(block_msg.clone());
    let mut buf = BytesMut::new();
    FrameCodec.encode(msg, &mut buf).unwrap();

    let mut decoder = FrameCodec;
    let decoded = decoder.decode(&mut buf).await.unwrap();
    match decoded {
        Some(Message::ProcessBlock(decoded_block)) => {
            assert_eq!(decoded_block.hash, block_msg.hash);
        }
        _ => panic!("Wrong variant"),
    }
}

#[tokio::test]
async fn test_processtransactions_roundtrip() {
    let tx_msgs = vec![p2p::messages::TransactionMessage {
        sender: "alice".to_string(),
        receiver: "bob".to_string(),
        amount: 100.0,
        // Optional fields:
        public_key: None,
        signature: None,
        transaction_type: None,
        timestamp: None,
        reference: None,
        block_id: None,
        height: None,
        fee: None,
    }];
    let msg = Message::ProcessTransactions(tx_msgs.clone());
    let mut buf = BytesMut::new();
    FrameCodec.encode(msg, &mut buf).unwrap();

    let mut decoder = FrameCodec;
    let decoded = decoder.decode(&mut buf).await.unwrap();
    match decoded {
        Some(Message::ProcessTransactions(decoded_txs)) => {
            assert_eq!(decoded_txs.len(), 1);
            assert_eq!(decoded_txs[0].sender, "alice");
        }
        _ => panic!("Wrong variant"),
    }
}
