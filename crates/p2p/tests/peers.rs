//! Peers manager unit tests.

use p2p::peer::{Peer, PeerState, Peers};
use std::net::SocketAddr;

// ---------- Peer Tests ----------

#[test]
fn test_peer_creation() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let peer = Peer::new(addr, false);
    
    assert_eq!(peer.address, addr);
    assert_eq!(peer.state, PeerState::NonConnected);
    assert!(!peer.is_inbound);
    assert!(peer.version.is_none());
    assert!(peer.application.is_none());
    assert!(peer.platform.is_none());
}

#[test]
fn test_peer_blacklist() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let mut peer = Peer::new(addr, false);
    
    assert_eq!(peer.blacklisting_time, 0);
    assert!(peer.blacklisting_cause.is_none());
    
    peer.blacklist("Test reason".to_string());
    
    assert!(peer.blacklisting_time > 0);
    assert_eq!(peer.blacklisting_cause, Some("Test reason".to_string()));
    assert_eq!(peer.state, PeerState::NonConnected);
}

#[test]
fn test_peer_un_blacklist() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let mut peer = Peer::new(addr, false);
    
    peer.blacklist("Test".to_string());
    assert!(peer.blacklisting_time > 0);
    
    peer.un_blacklist();
    assert_eq!(peer.blacklisting_time, 0);
    assert!(peer.blacklisting_cause.is_none());
}

#[test]
fn test_peer_update_blacklisted_status() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let mut peer = Peer::new(addr, false);
    
    peer.blacklist("Test".to_string());
    
    // Not expired yet
    peer.update_blacklisted_status(peer.blacklisting_time + 100, 3600);
    assert!(peer.blacklisting_time > 0);
    
    // Expired
    peer.update_blacklisted_status(peer.blacklisting_time + 3601, 3600);
    assert_eq!(peer.blacklisting_time, 0);
}

#[test]
fn test_peer_provides_service() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let mut peer = Peer::new(addr, false);
    
    peer.services = 0x01 | 0x02;
    
    assert!(peer.provides_service(0x01));
    assert!(peer.provides_service(0x02));
    assert!(!peer.provides_service(0x04));
}

// ---------- Peers Tests ----------

#[tokio::test]
async fn test_peers_creation() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let my_peer = Peer::new(addr, false);
    let peers = Peers::new(my_peer);
    
    // known_peers_count 只计算已知的外部节点，不包括自己节点
    assert_eq!(peers.known_peers_count().await, 0);
}

#[tokio::test]
async fn test_peers_register_peer() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let my_peer = Peer::new(addr, false);
    let peers = Peers::new(my_peer);
    
    let new_addr: SocketAddr = "192.168.1.1:16974".parse().unwrap();
    let new_peer = Peer::new(new_addr, false);
    
    peers.register_peer(new_peer).await;
    
    // 1 external peer registered
    assert_eq!(peers.known_peers_count().await, 1);
    assert!(peers.contains_peer(&new_addr).await);
}

#[tokio::test]
async fn test_peers_find_or_create_peer() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let my_peer = Peer::new(addr, false);
    let peers = Peers::new(my_peer);
    
    let new_addr: SocketAddr = "192.168.1.1:16974".parse().unwrap();
    
    // Create new peer
    let peer1 = peers.find_or_create_peer(new_addr, false).await;
    {
        let p = peer1.lock().await;
        assert_eq!(p.address, new_addr);
    }
    
    // Find existing peer
    let peer2 = peers.find_or_create_peer(new_addr, false).await;
    {
        let p2 = peer2.lock().await;
        assert_eq!(p2.address, new_addr);
    }
    
    // 1 external peer
    assert_eq!(peers.known_peers_count().await, 1);
}

#[tokio::test]
async fn test_peers_blacklist() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let my_peer = Peer::new(addr, false);
    let peers = Peers::new(my_peer);
    
    let new_addr: SocketAddr = "192.168.1.1:16974".parse().unwrap();
    
    // Create peer
    let _ = peers.find_or_create_peer(new_addr, false).await;
    
    // Add to blacklist
    peers.blacklist(new_addr).await;
    
    assert!(peers.is_blacklisted_addr(&new_addr).await);
}

#[tokio::test]
async fn test_peers_get_known_peers() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let my_peer = Peer::new(addr, false);
    let peers = Peers::new(my_peer);
    
    let addr1: SocketAddr = "192.168.1.1:16974".parse().unwrap();
    let addr2: SocketAddr = "192.168.1.2:16974".parse().unwrap();
    
    let _ = peers.find_or_create_peer(addr1, false).await;
    let _ = peers.find_or_create_peer(addr2, false).await;
    
    let known = peers.get_known_peers().await;
    // 2 external peers (not including self)
    assert_eq!(known.len(), 2);
}

#[tokio::test]
async fn test_peers_remove_peer() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let my_peer = Peer::new(addr, false);
    let peers = Peers::new(my_peer);
    
    let new_addr: SocketAddr = "192.168.1.1:16974".parse().unwrap();
    
    let _ = peers.find_or_create_peer(new_addr, false).await;
    // 1 external peer
    assert_eq!(peers.known_peers_count().await, 1);
    
    peers.remove_peer(&new_addr).await;
    // 0 external peers after removal
    assert_eq!(peers.known_peers_count().await, 0);
    assert!(!peers.contains_peer(&new_addr).await);
}

#[tokio::test]
async fn test_peers_get_any_peer() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let my_peer = Peer::new(addr, false);
    let peers = Peers::new(my_peer);
    
    // No connected peers initially
    let result = peers.get_any_peer(PeerState::Connected, false).await;
    assert!(result.is_none());
    
    // Add a connected peer
    let new_addr: SocketAddr = "192.168.1.1:16974".parse().unwrap();
    let peer = peers.find_or_create_peer(new_addr, false).await;
    {
        let mut p = peer.lock().await;
        p.state = PeerState::Connected;
    }
    
    let result = peers.get_any_peer(PeerState::Connected, false).await;
    assert!(result.is_some());
}

// ---------- BlacklistManager Tests ----------

#[tokio::test]
async fn test_blacklist_manager() {
    use p2p::BlacklistManager;
    
    let manager = BlacklistManager::new();
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    
    // Not blacklisted initially
    assert!(!manager.is_blacklisted(&addr).await);
    
    // Add to blacklist
    manager.add_to_blacklist(addr, "Test".to_string(), Some(3600), false).await;
    assert!(manager.is_blacklisted(&addr).await);
    
    // Remove from blacklist
    manager.remove_from_blacklist(&addr).await;
    assert!(!manager.is_blacklisted(&addr).await);
}

#[tokio::test]
async fn test_blacklist_manager_whitelist() {
    use p2p::BlacklistManager;
    
    let manager = BlacklistManager::new();
    let account_id = 12345u64;
    
    // Not whitelisted initially
    assert!(!manager.is_whitelisted(account_id).await);
    
    // Add to whitelist
    manager.add_to_whitelist(account_id, true).await;
    assert!(manager.is_whitelisted(account_id).await);
    
    // Remove from whitelist
    manager.remove_from_whitelist(account_id).await;
    assert!(!manager.is_whitelisted(account_id).await);
}

// ---------- P2PConfig Tests ----------

#[test]
fn test_p2p_config_default() {
    use p2p::P2PConfig;
    
    let config = P2PConfig::default();
    
    assert_eq!(config.min_compress_size, 256);
    assert_eq!(config.max_connections, 20);
    assert_eq!(config.max_inbound_connections, 100);
    assert_eq!(config.max_outbound_connections, 20);
    assert_eq!(config.connect_timeout_ms, 2000);
    assert_eq!(config.read_timeout_ms, 4000);
    assert_eq!(config.blacklisting_period_secs, 600);
    assert!(config.use_websockets);
    assert!(config.blacklisting_enabled);
}

#[test]
fn test_p2p_config_validation() {
    use p2p::P2PConfig;
    
    let config = P2PConfig::default();
    assert!(config.validate().is_ok());
}

// ---------- ErrorCode Tests ----------

#[test]
fn test_error_codes() {
    use p2p::ErrorCode;
    
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
    use p2p::ErrorCode;
    
    assert_eq!(ErrorCode::UnsupportedRequestType.message(), "Unsupported request type");
    assert_eq!(ErrorCode::UnknownPeer.message(), "Unknown peer");
}

#[test]
fn test_p2p_error_to_json() {
    use p2p::{ErrorCode, P2PError};
    
    let error = P2PError::unsupported_request_type();
    let json = error.to_json();
    
    assert_eq!(json["error"], "Unsupported request type");
    assert_eq!(json["errorCode"], 1);
}
