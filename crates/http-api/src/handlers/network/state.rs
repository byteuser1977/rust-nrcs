//! Network API State
//!
//! 网络 API 的状态管理

use std::sync::Arc;
use p2p::P2PManager;

/// 网络 API 状态
#[derive(Clone)]
pub struct NetworkApiState {
    pub p2p_manager: Option<Arc<P2PManager>>,
}

impl NetworkApiState {
    pub fn new(p2p_manager: Option<Arc<P2PManager>>) -> Self {
        Self { p2p_manager }
    }
}
