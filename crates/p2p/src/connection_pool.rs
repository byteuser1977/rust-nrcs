//! WebSocket 连接池管理
//!
//! 对应 Java NRCS: PeerWebSocket.java + PeerPostRequest.java
//!
//! 功能:
//! - 连接复用：维护与远程节点的持久化 WebSocket 连接
//! - 请求-响应匹配：使用 oneshot channel 替代 Java 的 CountDownLatch
//! - GZIP 压缩/解压：自动处理大消息压缩

use crate::config::P2PConfig;
use crate::error::{ErrorCode, P2PError};
use crate::protocol::{FrameCodec, PeerRequest};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{oneshot, RwLock, Mutex as TokioMutex};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, warn};

/// 等待中的请求（对应 Java PeerPostRequest）
///
/// 使用 oneshot channel 实现异步等待响应，
/// 替代 Java 中的 CountDownLatch 同步阻塞机制
struct PendingRequest {
    /// 完成信号通道发送端
    complete: oneshot::Sender<Result<String, String>>,
}

/// WebSocket 连接实例（对应 Java PeerWebSocket）
pub struct WebSocketConnection {
    /// 远程地址
    pub addr: SocketAddr,
    /// WebSocket 写入端（需要 Mutex 保护并发写入）
    write: Arc<TokioMutex<futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    /// 待响应请求映射 (requestId -> PendingRequest)
    pending_requests: Arc<RwLock<HashMap<i64, PendingRequest>>>,
    /// 下一个 requestId（原子递增）
    next_request_id: Arc<AtomicI64>,
    /// 是否已连接
    is_connected: Arc<std::sync::atomic::AtomicBool>,
}

impl WebSocketConnection {
    /// 创建新的 WebSocket 连接
    pub async fn new(addr: SocketAddr, config: &P2PConfig) -> Result<Self, P2PError> {
        let url = format!("ws://{}/nrcs", addr);

        // 连接超时
        let ws_stream = tokio::time::timeout(
            Duration::from_millis(config.connect_timeout_ms),
            tokio_tungstenite::connect_async(&url),
        ).await
        .map_err(|_| P2PError::connection_timeout())?
        .map_err(|e| P2PError::from_str(ErrorCode::ConnectionFailed, e.to_string()))?;

        let (ws_stream, _response) = ws_stream;
        let (write, read) = ws_stream.split();

        let conn = Self {
            addr,
            write: Arc::new(TokioMutex::new(write)),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            next_request_id: Arc::new(AtomicI64::new(0)),
            is_connected: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        };

        // 启动读取线程处理响应
        let pending = Arc::clone(&conn.pending_requests);
        let connected = Arc::clone(&conn.is_connected);
        let addr_clone = addr;

        tokio::spawn(async move {
            Self::read_loop(read, pending, connected, addr_clone).await;
        });

        debug!("[ConnectionPool] New connection established to {}", addr);
        Ok(conn)
    }

    /// 发送请求并等待响应（核心方法）
    ///
    /// 对应 Java: PeerPostRequest.send() + CountDownLatch.await()
    pub async fn send_and_wait(
        &self,
        request: &PeerRequest,
        config: &P2PConfig,
    ) -> Result<serde_json::Value, P2PError> {
        if !self.is_connected.load(Ordering::Relaxed) {
            return Err(P2PError::connection_closed());
        }

        // 1. 生成唯一 requestId
        let request_id = self.next_request_id.fetch_add(1, Ordering::SeqCst);

        // 2. 创建等待通道
        let (tx, rx) = oneshot::channel::<Result<String, String>>();

        // 3. 注册待响应请求
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(request_id, PendingRequest { complete: tx });
        }

        debug!("[ConnectionPool] Sending request {} to {}", request_id, self.addr);

        // 4. 序列化请求
        let payload = serde_json::to_vec(request)
            .map_err(|e| P2PError::serialization_error(format!("{}", e)))?;

        // 5. 判断是否 GZIP 压缩
        let should_compress = config.gzip_enabled && payload.len() >= config.min_compress_size;

        // 6. 编码帧
        let frame = FrameCodec.encode(&payload, should_compress);

        // 7. 发送帧
        {
            let mut write_guard = self.write.lock().await;
            write_guard.send(Message::Binary(frame)).await
                .map_err(|e| P2PError::from_str(ErrorCode::WriteFailed, e.to_string()))?;
        }

        debug!("[ConnectionPool] Request {} sent to {}, waiting response...", request_id, self.addr);

        // 8. 等待响应（带超时）
        match tokio::time::timeout(
            Duration::from_millis(config.read_timeout_ms),
            rx,
        ).await {
            Ok(Ok(Ok(response_str))) => {
                debug!("[ConnectionPool] Response received for request {} from {}", request_id, self.addr);
                serde_json::from_str::<serde_json::Value>(&response_str)
                    .map_err(|e| P2PError::deserialization_error(format!("{}", e)))
            }
            Ok(Ok(Err(e))) => {
                warn!("[ConnectionPool] Response error for request {}: {}", request_id, e);
                Err(P2PError::from_str(ErrorCode::InternalError, e.to_string()))
            }
            Ok(Err(e)) => {
                warn!("[ConnectionPool] Error for request {}: {}", request_id, e);
                Err(P2PError::from_str(ErrorCode::InternalError, format!("{}", e)))
            }
            Err(_) => {
                warn!("[ConnectionPool] Timeout waiting for response {} from {}", request_id, self.addr);
                // 清理过期的 pending request
                let mut pending = self.pending_requests.write().await;
                pending.remove(&request_id);
                Err(P2PError::read_timeout())
            }
        }
    }

    /// 检查连接是否有效
    pub fn is_valid(&self) -> bool {
        self.is_connected.load(Ordering::Relaxed)
    }

    /// 关闭连接并清理所有待响应请求
    pub async fn close(&self) {
        self.is_connected.store(false, Ordering::Relaxed);

        // 通知所有等待中的请求失败
        let mut pending = self.pending_requests.write().await;
        for (_, req) in pending.drain() {
            let _ = req.complete.send(Err("Connection closed".to_string()));
        }

        debug!("[ConnectionPool] Connection to {} closed", self.addr);
    }

    /// 读取循环（在后台线程运行）
    ///
    /// 处理从远程节点收到的所有响应帧，
    /// 根据 requestId 分发到对应的等待者
    async fn read_loop(
        mut read: futures_util::stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        pending_requests: Arc<RwLock<HashMap<i64, PendingRequest>>>,
        is_connected: Arc<std::sync::atomic::AtomicBool>,
        addr: SocketAddr,
    ) {
        let codec = FrameCodec;

        loop {
            match read.next().await {
                Some(Ok(Message::Binary(data))) => {
                    // 解码二进制帧
                    match codec.decode(&data) {
                        Ok((header, body)) => {
                            debug!("[ConnectionPool] Received frame from {}: req_id={}, len={}",
                                  addr, header.request_id, body.len());

                            // 尝试匹配到对应的 pending request
                            let result = {
                                let mut pending = pending_requests.write().await;
                                pending.remove(&header.request_id)
                            };

                            if let Some(req) = result {
                                // 将响应体转换为字符串发送给等待者
                                let _ = req.complete.send(
                                    String::from_utf8(body)
                                        .map_err(|e| e.to_string())
                                );
                            } else {
                                warn!("[ConnectionPool] No matching request for id {} from {}",
                                      header.request_id, addr);
                            }
                        }
                        Err(e) => {
                            error!("[ConnectionPool] Frame decode error from {}: {}", addr, e);
                        }
                    }
                }
                Some(Ok(Message::Text(text))) => {
                    // 处理文本帧（兼容模式）
                    let data = text.into_bytes();
                    match codec.decode(&data) {
                        Ok((header, body)) => {
                            debug!("[ConnectionPool] Received text frame from {}: req_id={}",
                                  addr, header.request_id);

                            let result = {
                                let mut pending = pending_requests.write().await;
                                pending.remove(&header.request_id)
                            };

                            if let Some(req) = result {
                                let _ = req.complete.send(
                                    String::from_utf8(body)
                                        .map_err(|e| e.to_string())
                                );
                            }
                        }
                        Err(e) => {
                            error!("[ConnectionPool] Text frame decode error from {}: {}", addr, e);
                        }
                    }
                }
                Some(Ok(Message::Close(_))) | None => {
                    debug!("[ConnectionPool] Connection closed by remote: {}", addr);
                    break;
                }
                Some(Ok(Message::Ping(_))) => {
                    // Ping/Pong 自动处理，无需手动回复
                    debug!("[ConnectionPool] Ping from {}", addr);
                }
                Some(Err(e)) => {
                    error!("[ConnectionPool] Stream error from {}: {}", addr, e);
                    break;
                }
                Some(Ok(_)) => {
                    // 其他消息类型忽略
                }
            }
        }

        // 标记为断开
        is_connected.store(false, Ordering::Relaxed);

        // 清理所有待响应请求
        let mut pending = pending_requests.write().await;
        for (_, req) in pending.drain() {
            let _ = req.complete.send(Err("Connection lost".to_string()));
        }

        warn!("[ConnectionPool] Read loop ended for {}", addr);
    }
}

/// WebSocket 连接池管理器
///
/// 管理所有活跃的 WebSocket 连接，
/// 支持连接复用和自动清理
pub struct ConnectionPool {
    /// 活跃连接映射 (addr -> Connection)
    connections: Arc<RwLock<HashMap<SocketArcKey, Arc<WebSocketConnection>>>>,
    /// 配置引用
    config: Arc<P2PConfig>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SocketArcKey(SocketAddr);

impl From<SocketAddr> for SocketArcKey {
    fn from(addr: SocketAddr) -> Self {
        Self(addr)
    }
}

impl std::fmt::Display for SocketArcKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ConnectionPool {
    /// 创建新的连接池
    pub fn new(config: Arc<P2PConfig>) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 获取或创建到指定地址的连接
    ///
    /// 如果已有有效连接则复用，否则新建连接
    pub async fn get_or_create_connection(
        &self,
        addr: SocketAddr,
    ) -> Result<Arc<WebSocketConnection>, P2PError> {
        // 1. 先查找现有连接
        {
            let conns = self.connections.read().await;
            if let Some(conn) = conns.get(&addr.into()) {
                if conn.is_valid() {
                    debug!("[ConnectionPool] Reusing existing connection to {}", addr);
                    return Ok(Arc::clone(conn));
                }
            }
        }

        // 2. 创建新连接
        debug!("[ConnectionPool] Creating new connection to {}", addr);
        let conn = WebSocketConnection::new(addr, &self.config).await?;
        let conn_arc = Arc::new(conn);

        // 3. 注册到连接池
        {
            let mut conns = self.connections.write().await;
            conns.insert(addr.into(), Arc::clone(&conn_arc));
        }

        Ok(conn_arc)
    }

    /// 通过连接池发送请求（自动复用连接）
    ///
    /// 这是主要的外部接口，封装了连接获取、发送、响应等待的全流程
    pub async fn send_request(
        &self,
        addr: SocketAddr,
        request: &PeerRequest,
    ) -> Result<serde_json::Value, P2PError> {
        let conn = self.get_or_create_connection(addr).await?;
        conn.send_and_wait(request, &self.config).await
    }

    /// 移除指定地址的连接
    pub async fn remove_connection(&self, addr: &SocketAddr) {
        let mut conns = self.connections.write().await;
        if let Some(conn) = conns.remove(&(*addr).into()) {
            conn.close().await;
        }
    }

    /// 清理所有无效连接
    pub async fn cleanup_invalid_connections(&self) -> usize {
        let mut conns = self.connections.write().await;
        let before = conns.len();
        conns.retain(|_, conn| conn.is_valid());
        let after = conns.len();
        before - after
    }

    /// 获取当前活跃连接数
    pub async fn active_count(&self) -> usize {
        let conns = self.connections.read().await;
        conns.values().filter(|c| c.is_valid()).count()
    }

    /// 关闭所有连接
    pub async fn shutdown(&self) {
        let mut conns = self.connections.write().await;
        for (_, conn) in conns.drain() {
            conn.close().await;
        }
        debug!("[ConnectionPool] All connections shut down");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_arc_key() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let key = SocketArcKey::from(addr);
        assert_eq!(format!("{}", key), "127.0.0.1:8080");
    }
}
