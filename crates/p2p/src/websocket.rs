use crate::{
    protocol::{FrameCodec, PeerRequest, PeerResponse, RequestType},
    handlers::Handler,
    peer::Peers,
};
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use serde_json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};

/// WebSocket 服务器配置
#[derive(Clone, Debug)]
pub struct WebsocketConfig {
    pub listen_addr: SocketAddr,
    pub max_connections: usize,
}

/// WebSocket 服务器
pub struct WebsocketServer {
    config: WebsocketConfig,
    peers: Arc<Peers>,
    handler: Arc<Handler>,
}

impl WebsocketServer {
    pub fn new(config: WebsocketConfig, peers: Arc<Peers>, handler: Arc<Handler>) -> Self {
        Self {
            config,
            peers,
            handler,
        }
    }

    pub async fn serve(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.config.listen_addr).await?;
        info!("WebSocket server listening on {}", self.config.listen_addr);

        loop {
            let (stream, addr) = listener.accept().await?;
            info!("New connection from: {}", addr);

            let peers = Arc::clone(&self.peers);
            let handler = Arc::clone(&self.handler);
            let max_conns = self.config.max_connections;

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, addr, peers, handler, max_conns).await {
                    error!("Connection error from {}: {}", addr, e);
                }
            });
        }
    }

    async fn handle_connection(
        stream: tokio::net::TcpStream,
        addr: SocketAddr,
        peers: Arc<Peers>,
        handler: Arc<Handler>,
        max_connections: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();

        // 检查当前活跃连接数
        let peer_count = peers.connection_count().await;
        if peer_count >= max_connections {
            warn!("Max connections reached, rejecting: {}", addr);
            return Ok(());
        }

        // 注册新连接
        peers.add_connection(addr).await;
        let peer_count = peers.connection_count().await;
        info!("Connection established. Active: {}", peer_count);

        let codec = FrameCodec;

        loop {
            match read.next().await {
                Some(Ok(Message::Binary(data))) => {
                    match codec.decode(&data) {
                        Ok((header, body)) => {
                            debug!("[SERVER] Received binary frame: req_id={}, len={}", header.request_id, body.len());

                            match serde_json::from_slice::<PeerRequest>(&body) {
                                Ok(request) => {
                                    // Peers 已实现 Clone，直接克隆引用
                                    let peers_arc = peers.clone();
                                    debug!("[SERVER] Received request: {:?}", request);
                                    let response = handler.handle(request, peers_arc).await;
                                    debug!("[SERVER] Sending response: {:?}", response);
                                    let resp_json = serde_json::to_vec(&response)
                                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
                                    debug!("[SERVER] Response JSON ({} bytes)", resp_json.len());
                                    let frame = codec.encode(&resp_json, false); // 暂时不压缩响应
                                    debug!("[SERVER] Sending frame ({} bytes)", frame.len());
                                    if let Err(e) = write.send(Message::Binary(frame)).await {
                                        error!("[SERVER] Failed to send response: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("JSON parse error: {}", e);
                                    let error_resp = PeerResponse::error("INVALID_JSON");
                                    let resp_json = serde_json::to_vec(&error_resp).unwrap();
                                    let frame = codec.encode(&resp_json, false);
                                    let _ = write.send(Message::Binary(frame)).await;
                                }
                            }
                        }
                        Err(e) => {
                            error!("Frame decode error: {}", e);
                        }
                    }
                }
                Some(Ok(Message::Text(text))) => {
                    debug!("[SERVER] Received text frame (len={})", text.len());
                    let data = text.as_bytes();
                    match codec.decode(data) {
                        Ok((header, body)) => {
                            debug!("[SERVER] Request: req_id={}, len={}", header.request_id, body.len());
                            match serde_json::from_slice::<PeerRequest>(&body) {
                                Ok(request) => {
                                    let peers_arc = peers.clone();
                                    let response = handler.handle(request, peers_arc).await;
                                    let resp_json = serde_json::to_vec(&response)
                                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
                                    let frame = codec.encode(&resp_json, false);
                                    if let Err(e) = write.send(Message::Binary(frame)).await {
                                        error!("[SERVER] Failed to send response: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("[SERVER] JSON parse error: {}", e);
                                    let error_resp = PeerResponse::error("INVALID_JSON");
                                    let resp_json = serde_json::to_vec(&error_resp).unwrap();
                                    let frame = codec.encode(&resp_json, false);
                                    let _ = write.send(Message::Binary(frame)).await;
                                }
                            }
                        }
                        Err(e) => {
                            error!("[SERVER] Frame decode error: {}", e);
                        }
                    }
                }
                Some(Ok(Message::Close(_))) => {
                    info!("Connection closed by client: {}", addr);
                    break;
                }
                Some(Ok(Message::Ping(p))) => {
                    if let Err(e) = write.send(Message::Pong(p)).await {
                        error!("Ping error: {}", e);
                        break;
                    }
                }
                Some(Ok(_)) => {
                    // 其他消息类型忽略
                }
                Some(Err(e)) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                None => {
                    info!("Connection ended: {}", addr);
                    break;
                }
            }
        }

        // 清理连接
        peers.remove_connection(&addr).await;
        let count = peers.connection_count().await;
        info!("Connection removed. Active: {}", count);

        Ok(())
    }
}

/// WebSocket 客户端（出站连接）
pub struct WebsocketClient;

impl WebsocketClient {
    /// 连接到远程 P2P 节点（WebSocket 优先，失败降级为 HTTP POST）
    pub async fn connect(
        addr: SocketAddr,
        peers: Arc<Peers>,
        _handler: Arc<Handler>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("[CLIENT] Attempting to connect to {} via WebSocket", addr);

        let url = format!("ws://{}/nrcs", addr);
        info!("[CLIENT] Connecting to WebSocket URL: {}", url);
        // 10 秒连接超时
        let ws_stream = match tokio::time::timeout(
            tokio::time::Duration::from_secs(10),
            tokio_tungstenite::connect_async(&url),
        ).await {
            Ok(Ok((ws_stream, _))) => {
                info!("[CLIENT] WebSocket connected to {}", addr);
                ws_stream
            }
            Ok(Err(e)) => {
                warn!("[CLIENT] WebSocket handshake failed with {}: {}, falling back to HTTP", addr, e);
                return Self::http_fallback_handshake(addr, &peers).await;
            }
            Err(_) => {
                warn!("[CLIENT] WebSocket connection timeout to {}", addr);
                return Self::http_fallback_handshake(addr, &peers).await;
            }
        };

        let (mut write, mut read) = ws_stream.split();
        let codec = FrameCodec;

        // 发送 getInfo (使用二进制帧协议)
        let req = PeerRequest::new(RequestType::GetInfo, 1);
        debug!("[CLIENT] Creating getInfo request");
        let payload = serde_json::to_vec(&req)?;
        debug!("[CLIENT] Serialized getInfo request ({} bytes)", payload.len());
        let frame = codec.encode(&payload, false);
        debug!("[CLIENT] Sending binary frame ({} bytes)", frame.len());
        if let Err(e) = write.send(Message::Binary(frame)).await {
            error!("[CLIENT] Failed to send getInfo to {}: {}", addr, e);
            return Ok(());
        }
        debug!("[CLIENT] Sent getInfo (binary) to {}", addr);

        // 等待响应（10秒超时）
        let response_body = match tokio::time::timeout(
            tokio::time::Duration::from_secs(10),
            read.next(),
        ).await {
            Ok(Some(Ok(Message::Binary(data)))) => {
                debug!("[CLIENT] Received binary frame ({} bytes) from {}", data.len(), addr);
                match codec.decode(&data) {
                    Ok((header, body)) => {
                        debug!("[CLIENT] Decoded frame: req_id={}, body_len={}", header.request_id, body.len());
                        Some(body)
                    },
                    Err(e) => {
                        error!("[CLIENT] Frame decode error from {}: {}", addr, e);
                        None
                    }
                }
            }
            Ok(Some(Ok(Message::Text(text)))) => {
                debug!("[CLIENT] Received text frame (len={}) from {}", text.len(), addr);
                let data = text.as_bytes();
                match codec.decode(data) {
                    Ok((header, body)) => {
                        debug!("[CLIENT] Decoded text frame: req_id={}, body_len={}", header.request_id, body.len());
                        Some(body)
                    },
                    Err(e) => {
                        error!("[CLIENT] Frame decode error from {}: {}", addr, e);
                        None
                    }
                }
            }
            Ok(Some(Ok(_))) => {
                warn!("[CLIENT] Unexpected message type from {}", addr);
                None
            }
            Ok(Some(Err(e))) => {
                error!("[CLIENT] Stream error from {}: {}", addr, e);
                None
            }
            Ok(None) => {
                warn!("[CLIENT] Stream closed by {}", addr);
                None
            }
            Err(_) => {
                warn!("[CLIENT] Timeout waiting for response from {}", addr);
                None
            }
        };

        if let Some(body) = response_body {
            debug!("[CLIENT] Parsing PeerResponse from {} ({} bytes)", addr, body.len());
            match serde_json::from_slice::<PeerResponse>(&body) {
                Ok(resp) => {
                    debug!("[CLIENT] Received PeerResponse: {:?}", resp);
                    if resp.error.is_none() {
                        info!("Handshake successful with {}", addr);
                        let mut p = crate::peer::Peer::new(addr, false);
                        p.set_state(crate::peer::PeerState::Connected);
                        peers.register_peer(p).await;
                    } else {
                        warn!("Handshake error from {}: {:?}", addr, resp.error);
                    }
                }
                Err(e) => error!("[CLIENT] Invalid PeerResponse from {}: {}", addr, e),
            }
        }

        // 关闭连接
        let _ = write.send(Message::Close(None)).await;
        Ok(())
    }

    pub async fn send_request(
        addr: SocketAddr,
        request: PeerRequest,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("[CLIENT] Sending {:?} request to {} via HTTP", request.request_type, addr);

        let client = Client::new();
        let url = format!("http://{}/nrcs", addr);

        match tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            async {
                let resp = client.post(&url).json(&request).send().await?;
                if !resp.status().is_success() {
                    return Ok(None);
                }
                let body = resp.bytes().await?;
                Ok::<_, reqwest::Error>(Some(body))
            },
        ).await {
            Ok(Ok(Some(body))) => {
                match serde_json::from_slice::<serde_json::Value>(&body) {
                    Ok(resp) => {
                        debug!("[CLIENT] Received response from {}", addr);
                        Ok(resp)
                    }
                    Err(e) => {
                        error!("[CLIENT] Failed to parse response from {}: {}", addr, e);
                        Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                    }
                }
            }
            Ok(Ok(None)) => {
                warn!("[CLIENT] HTTP non-success status from {}", addr);
                Err(Box::new(std::io::Error::other("HTTP request failed")) as Box<dyn std::error::Error + Send + Sync>)
            }
            Ok(Err(e)) => {
                error!("[CLIENT] HTTP request error to {}: {}", addr, e);
                Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            }
            Err(_) => {
                warn!("[CLIENT] HTTP request timeout to {}", addr);
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::TimedOut, "Request timeout")) as Box<dyn std::error::Error + Send + Sync>)
            }
        }
    }

    async fn http_fallback_handshake(
        addr: SocketAddr,
        peers: &Arc<Peers>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Attempting HTTP fallback handshake with {}", addr);
        let client = Client::new();
        let url = format!("http://{}/nrcs", addr);
        let req = PeerRequest::new(RequestType::GetInfo, 1);

        match tokio::time::timeout(
            tokio::time::Duration::from_secs(10),
            async {
                let resp = client.post(&url).json(&req).send().await?;
                if !resp.status().is_success() {
                    return Ok(None);
                }
                let body = resp.bytes().await?;
                Ok::<_, reqwest::Error>(Some(body))
            },
        ).await {
            Ok(Ok(Some(body))) => {
                if let Ok(resp) = serde_json::from_slice::<PeerResponse>(&body) {
                    if resp.error.is_none() {
                        info!("HTTP handshake succeeded with {}", addr);
                        let mut p = crate::peer::Peer::new(addr, false);
                        p.set_state(crate::peer::PeerState::Connected);
                        peers.register_peer(p).await;
                    } else {
                        warn!("HTTP handshake error from {}: {:?}", addr, resp.error);
                    }
                }
            }
            Ok(Err(e)) => warn!("HTTP request error to {}: {}", addr, e),
            Ok(Ok(None)) => warn!("HTTP non-success status from {}", addr),
            Err(_) => warn!("HTTP handshake timeout to {}", addr),
        }
        Ok(())
    }
}