use crate::{
    protocol::{FrameCodec, PeerRequest, PeerResponse, ProtocolError},
    handlers::Handler,
    peer::Peers,
};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tracing::{error, info, warn};

/// WebSocket 服务器配置
#[derive(Clone, Debug)]
pub struct WebsocketConfig {
    pub listen_addr: SocketAddr,
    pub max_connections: usize,
}

/// WebSocket 服务器
pub struct WebsocketServer {
    config: WebsocketConfig,
    peers: Arc<Mutex<Peers>>,
    handler: Arc<Handler>,
}

impl WebsocketServer {
    pub fn new(config: WebsocketConfig, peers: Arc<Mutex<Peers>>, handler: Arc<Handler>) -> Self {
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
        peers: Arc<Mutex<Peers>>,
        handler: Arc<Handler>,
        max_connections: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();

        // 注册新连接
        let mut peer_count = {
            let peers = peers.lock().await;
            peers.connection_count()
        };

        if peer_count >= max_connections {
            warn!("Max connections reached, rejecting: {}", addr);
            return Ok(());
        }

        {
            let mut peers = peers.lock().await;
            peers.add_connection(addr);
            peer_count = peers.connection_count();
        }

        info!("Connection established. Active: {}", peer_count);

        let codec = FrameCodec;

        loop {
            match read.next().await {
                Some(Ok(Message::Binary(data))) => {
                    match codec.decode(&data) {
                        Ok((header, body)) => {
                            info!("Received frame: req_id={}, len={}", header.request_id, body.len());

                            match serde_json::from_slice::<PeerRequest>(&body) {
                                Ok(request) => {
                                    let response = handler.handle(request, Arc::clone(&peers)).await;
                                    let resp_json = serde_json::to_vec(&response)
                                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

                                    let frame = codec.encode(&resp_json, false); // 暂时不压缩响应

                                    if let Err(e) = write.send(Message::Binary(frame)).await {
                                        error!("Failed to send response: {}", e);
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
        let count = {
            let mut peers = peers.lock().await;
            peers.remove_connection(addr);
            peers.connection_count()
        };
        info!("Connection removed. Active: {}", count);

        Ok(())
    }
}