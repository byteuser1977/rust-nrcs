//! API Middleware
//!
//! HTTP 中间件

use axum::{
    http::{Request, Response},
    middleware::Next,
    body::Body,
};

/// 请求日志中间件
pub async fn request_logging(
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    tracing::info!("Request: {} {}", method, uri);
    
    let response = next.run(req).await;
    
    tracing::info!("Response: {} {}", method, uri);
    
    response
}
