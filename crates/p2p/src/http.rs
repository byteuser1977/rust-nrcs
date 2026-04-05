use crate::{
    protocol::{PeerRequest, PeerResponse, FrameCodec},
    handlers::Handler,
    peer::Peers,
};
use axum::{
    body::Body,
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use serde_json;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info};

/// HTTP 服务器配置
#[derive(Clone, Debug)]
pub struct HttpConfig {
    pub listen_addr: SocketAddr,
}

/// 创建 HTTP 路由
pub fn create_router(
    peers: Arc<Peers>,
    handler: Arc<Handler>,
) -> Router {
    let handler_state = HttpHandlerState { peers, handler };

    Router::new()
        .route("/peer", post(handle_peer))
        .with_state(handler_state)
}

#[derive(Clone)]
struct HttpHandlerState {
    peers: Arc<Peers>,
    handler: Arc<Handler>,
}

async fn handle_peer(
    headers: HeaderMap,
    body: Body,
    state: axum::extract::State<HttpHandlerState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // 检查 Content-Type
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");

    let body_bytes = match axum::body::to_bytes(body, 10 * 1024 * 1024).await {
        Ok(bytes) => bytes.to_bytes().to_vec(),
        Err(e) => {
            error!("Failed to read body: {}", e);
            return Err((StatusCode::BAD_REQUEST, "Invalid body").into_response());
        }
    };

    // 如果是 JSON 格式，直接解析；否则尝试从文本解析
    let request = if content_type.contains("application/json") {
        match serde_json::from_slice::<PeerRequest>(&body_bytes) {
            Ok(req) => req,
            Err(e) => {
                error!("JSON parse error: {}", e);
                return Err((StatusCode::BAD_REQUEST, "Invalid JSON").into_response());
            }
        }
    } else {
        // 尝试解析为原始文本中的 JSON
        match body_bytes.as_slice() {
            data if !data.is_empty() => {
                match serde_json::from_slice::<PeerRequest>(data) {
                    Ok(req) => req,
                    Err(e) => {
                        error!("JSON parse error: {}", e);
                        return Err((StatusCode::BAD_REQUEST, "Invalid JSON").into_response());
                    }
                }
            }
            _ => {
                return Err((StatusCode::BAD_REQUEST, "Empty body").into_response());
            }
        }
    };

    info!("HTTP request: {:?}", request.request_type);

    // 调用处理器
    let response = state.handler.handle(request, state.peers).await;

    // 序列化响应
    let resp_json = match serde_json::to_vec(&response) {
        Ok(json) => json,
        Err(e) => {
            error!("Response serialization error: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Serialization error").into_response());
        }
    };

    // 返回响应（与 Java 兼容：text/plain; charset=UTF-8）
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        "content-type",
        "text/plain; charset=UTF-8".parse().unwrap(),
    );

    Ok((response_headers, resp_json).into_response())
}

/// 启动 HTTP 服务器
pub async fn serve(
    config: HttpConfig,
    peers: Arc<Peers>,
    handler: Arc<Handler>,
) -> Result<(), Box<dyn std::error::Error>> {
    let router = create_router(peers, handler);

    let addr = config.listen_addr;
    info!("HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}