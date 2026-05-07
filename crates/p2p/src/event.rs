//! Peer Event System (节点事件系统)
//!
//! 对应 NRCS Java: Peers.Event enum + Listeners<IPeer, Event> + Listener<IPeer>
//!
//! 提供 peer 生命周期事件的注册、分发和通知机制。
//! 事件在 Peer/Peers 状态变更时触发，监听器可以是 DB 持久化、HTTP API 事件流等。

use crate::peer::Peer;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// Peer 事件类型
///
/// 对应 NRCS Java: Peers.Event enum (13 种事件)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PeerEvent {
    /// 节点被加入黑名单
    Blacklist,
    /// 节点从黑名单移除
    Unblacklist,
    /// 节点被停用（断开连接）
    Deactivate,
    /// 节点被移除（从已知节点列表删除）
    Remove,
    /// 下载数据量更新
    DownloadedVolume,
    /// 上传数据量更新
    UploadedVolume,
    /// Hallmark 权重变更
    Weight,
    /// 新增活跃节点（NonConnected → 其他状态）
    AddedActivePeer,
    /// 活跃节点状态变更（Connected ↔ Disconnected 等）
    ChangedActivePeer,
    /// 新节点加入
    NewPeer,
    /// 新入站连接
    AddInbound,
    /// 移除入站连接
    RemoveInbound,
    /// 服务标志变更
    ChangedServices,
}

/// Peer 事件监听器 trait
///
/// 对应 NRCS Java: Listener<IPeer> / IListener<IPeer>
#[async_trait::async_trait]
pub trait PeerListener: Send + Sync {
    /// 接收 peer 事件通知
    async fn notify(&self, peer: &Peer, event: PeerEvent);
}

/// Peer 事件分发器
///
/// 对应 NRCS Java: Listeners<IPeer, Peers.Event>
///
/// 线程安全的事件监听器注册表，支持按事件类型注册和分发。
#[allow(clippy::type_complexity)]
pub struct PeerEventDispatcher {
    listeners: Arc<RwLock<HashMap<PeerEvent, Vec<Arc<dyn PeerListener>>>>>,
}

impl std::fmt::Debug for PeerEventDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PeerEventDispatcher")
            .field("listeners", &"<opaque>")
            .finish()
    }
}

impl PeerEventDispatcher {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册事件监听器
    ///
    /// 对应 NRCS Java: Listeners.addListener(Listener, Enum)
    pub async fn add_listener(&self, listener: Arc<dyn PeerListener>, event_type: PeerEvent) -> bool {
        let mut map = self.listeners.write().await;
        let list = map.entry(event_type).or_default();
        // 防止重复注册（通过 Arc 指针地址判断）
        let listener_ptr = Arc::as_ptr(&listener) as *const ();
        if list.iter().any(|l| Arc::as_ptr(l) as *const () == listener_ptr) {
            return false;
        }
        list.push(listener);
        debug!("[PeerEvent] Listener registered for {:?}", event_type);
        true
    }

    /// 移除事件监听器
    ///
    /// 对应 NRCS Java: Listeners.removeListener(Listener, Enum)
    pub async fn remove_listener(&self, listener: &Arc<dyn PeerListener>, event_type: PeerEvent) -> bool {
        let mut map = self.listeners.write().await;
        if let Some(list) = map.get_mut(&event_type) {
            let ptr = Arc::as_ptr(listener) as *const ();
            let len_before = list.len();
            list.retain(|l| Arc::as_ptr(l) as *const () != ptr);
            if list.len() < len_before {
                debug!("[PeerEvent] Listener removed for {:?}", event_type);
                return true;
            }
        }
        false
    }

    /// 分发事件到所有注册的监听器（fire-and-forget）
    ///
    /// 对应 NRCS Java: Listeners.notify(T, Enum)
    ///
    /// 为每个监听器 spawn 独立任务，避免单个监听器阻塞其他监听器。
    pub fn notify(&self, peer: &Peer, event: PeerEvent) {
        let peer_clone = peer.clone();
        let listeners = Arc::clone(&self.listeners);

        tokio::spawn(async move {
            let map = listeners.read().await;
            if let Some(list) = map.get(&event) {
                for listener in list {
                    let listener = Arc::clone(listener);
                    let peer = peer_clone.clone();
                    tokio::spawn(async move {
                        listener.notify(&peer, event).await;
                    });
                }
            }
        });
    }

    /// 同步分发事件（在已有 async 上下文中使用）
    pub async fn notify_async(&self, peer: &Peer, event: PeerEvent) {
        let map = self.listeners.read().await;
        if let Some(list) = map.get(&event) {
            for listener in list {
                let listener = Arc::clone(listener);
                let peer = peer.clone();
                tokio::spawn(async move {
                    listener.notify(&peer, event).await;
                });
            }
        }
    }

    /// 获取指定事件类型的监听器数量
    pub async fn listener_count(&self, event_type: PeerEvent) -> usize {
        let map = self.listeners.read().await;
        map.get(&event_type).map_or(0, |l| l.len())
    }
}

impl Default for PeerEventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestListener {
        count: AtomicUsize,
    }

    #[async_trait::async_trait]
    impl PeerListener for TestListener {
        async fn notify(&self, _peer: &Peer, _event: PeerEvent) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn test_add_and_notify() {
        let dispatcher = PeerEventDispatcher::new();
        let listener = Arc::new(TestListener { count: AtomicUsize::new(0) });

        dispatcher.add_listener(listener.clone(), PeerEvent::Blacklist).await;
        assert_eq!(dispatcher.listener_count(PeerEvent::Blacklist).await, 1);

        let peer = Peer::new("127.0.0.1:8000".parse().unwrap(), false);
        dispatcher.notify_async(&peer, PeerEvent::Blacklist).await;

        // 等待异步任务完成
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(listener.count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_remove_listener() {
        let dispatcher = PeerEventDispatcher::new();
        let listener: Arc<dyn PeerListener> = Arc::new(TestListener { count: AtomicUsize::new(0) });

        dispatcher.add_listener(listener.clone(), PeerEvent::Blacklist).await;
        assert!(dispatcher.remove_listener(&listener, PeerEvent::Blacklist).await);
        assert_eq!(dispatcher.listener_count(PeerEvent::Blacklist).await, 0);
    }

    #[tokio::test]
    async fn test_no_duplicate_registration() {
        let dispatcher = PeerEventDispatcher::new();
        let listener: Arc<dyn PeerListener> = Arc::new(TestListener { count: AtomicUsize::new(0) });

        assert!(dispatcher.add_listener(listener.clone(), PeerEvent::Blacklist).await);
        assert!(!dispatcher.add_listener(listener.clone(), PeerEvent::Blacklist).await);
        assert_eq!(dispatcher.listener_count(PeerEvent::Blacklist).await, 1);
    }
}
