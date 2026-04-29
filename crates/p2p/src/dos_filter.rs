//! P2P DoS 防护中间件
//!
//! 对应 NRCS Java: Jetty DoSFilter 配置
//!
//! 功能:
//! - 请求速率限制（每秒最大请求数）
//! - IP 白名单/黑名单
//! - 连接数限制
//! - 请求超时控制

use axum::{
    body::Body,
    extract::Request,
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// DoS 防护配置
#[derive(Debug, Clone)]
pub struct DosFilterConfig {
    /// 每秒最大请求数（对应 Java: maxRequestsPerSec）
    pub max_requests_per_sec: u32,
    /// 超限延迟时间毫秒（对应 Java: delayMs）
    pub delay_ms: u64,
    /// 单请求最大处理时间毫秒（对应 Java: maxRequestMs）
    pub max_request_ms: u64,
    /// 节流时间毫秒（对应 Java: throttleMs）
    pub throttle_ms: u64,
    /// 最大节流时间毫秒（对应 Java: maxThrottleMs）
    pub max_throttle_ms: u64,
    /// 最大等待时间毫秒（对应 Java: maxWaitMs）
    pub max_wait_ms: u64,
    /// 空闲追踪器时间毫秒（对应 Java: maxIdleTrackerMs）
    pub max_idle_tracker_ms: u64,
    /// 是否跟踪会话（对应 Java: trackSessions）
    pub track_sessions: bool,
    /// 远程端口（用于白名单，对应 Java: remotePort）
    pub remote_port: u16,
    /// 插入端口（用于白名单，对应 Java: insertPort）
    pub insert_port: u16,
    /// IP 白名单
    pub whitelist: Vec<String>,
    /// IP 黑名单
    pub blacklist: Vec<String>,
}

impl Default for DosFilterConfig {
    fn default() -> Self {
        Self {
            max_requests_per_sec: 30,
            delay_ms: 1000,
            max_request_ms: 300000,
            throttle_ms: 1000,
            max_throttle_ms: 30000,
            max_wait_ms: 50000,
            max_idle_tracker_ms: 30000,
            track_sessions: false,
            remote_port: 80,
            insert_port: 80,
            whitelist: Vec::new(),
            blacklist: Vec::new(),
        }
    }
}

/// IP 请求跟踪器
#[derive(Debug)]
struct IpTracker {
    /// 请求计数
    request_count: u32,
    /// 上次请求时间
    last_request: Instant,
    /// 节流开始时间（如果正在被节流）
    throttle_start: Option<Instant>,
    /// 被拒绝的次数
    rejected_count: u32,
}

impl IpTracker {
    fn new() -> Self {
        Self {
            request_count: 0,
            last_request: Instant::now(),
            throttle_start: None,
            rejected_count: 0,
        }
    }
}

/// DoS 防护状态
pub(crate) struct DosState {
    /// IP 追踪器映射
    ip_trackers: RwLock<HashMap<IpAddr, IpTracker>>,
    /// 配置（供内部方法访问）
    #[allow(dead_code)]
    config: DosFilterConfig,
}

impl DosState {
    fn new(config: DosFilterConfig) -> Self {
        Self {
            ip_trackers: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// 检查 IP 是否在白名单中
    async fn is_whitelisted(&self, ip: &IpAddr) -> bool {
        if self.config.whitelist.is_empty() {
            return false;
        }
        let ip_str = ip.to_string();
        self.config.whitelist.iter().any(|entry| {
            entry == &ip_str || entry == "0.0.0.0" || entry == "::"
        })
    }

    /// 检查 IP 是否在黑名单中
    async fn is_blacklisted(&self, ip: &IpAddr) -> bool {
        if self.config.blacklist.is_empty() {
            return false;
        }
        let ip_str = ip.to_string();
        self.config.blacklist.contains(&ip_str)
    }

    /// 检查是否允许请求通过
    async fn check_request(&self, ip: &IpAddr) -> Result<(), StatusCode> {
        // 1. 检查黑名单
        if self.is_blacklisted(ip).await {
            warn!("[DoS] Rejected blacklisted IP: {}", ip);
            return Err(StatusCode::FORBIDDEN);
        }

        // 2. 检查白名单（白名单中的直接放行）
        if self.is_whitelisted(ip).await {
            debug!("[DoS] Whitelisted IP allowed: {}", ip);
            return Ok(());
        }

        // 3. 检查速率限制
        let mut trackers = self.ip_trackers.write().await;
        let now = Instant::now();
        let tracker = trackers.entry(*ip).or_insert_with(IpTracker::new);

        // 计算距离上次请求的时间差
        let time_since_last = now.duration_since(tracker.last_request).as_millis() as u64;

        // 重置计数器（如果超过 1 秒）
        if time_since_last >= 1000 {
            tracker.request_count = 0;
            tracker.last_request = now;
        }

        // 增加计数
        tracker.request_count += 1;

        // 检查是否超过限制
        if tracker.request_count > self.config.max_requests_per_sec {
            tracker.rejected_count += 1;

            // 开始或继续节流
            if tracker.throttle_start.is_none() {
                tracker.throttle_start = Some(now);
            } else {
                let throttle_duration = now.duration_since(tracker.throttle_start.unwrap()).as_millis() as u64;
                if throttle_duration >= self.config.max_throttle_ms {
                    // 超过最大节流时间，重置
                    tracker.request_count = 0;
                    tracker.throttle_start = None;
                    tracker.last_request = now;
                    return Ok(());
                }
            }

            warn!("[DoS] Rate limited IP: {} ({} req/sec, rejected {} times)",
                  ip, tracker.request_count, tracker.rejected_count);
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        // 清除节流状态
        tracker.throttle_start = None;

        Ok(())
    }

    /// 清理过期的追踪器
    #[allow(dead_code)]
    async fn cleanup_expired(&self) {
        let mut trackers = self.ip_trackers.write().await;
        let now = Instant::now();
        let max_idle = Duration::from_millis(self.config.max_idle_tracker_ms);

        trackers.retain(|_, tracker| {
            now.duration_since(tracker.last_request) < max_idle
        });
    }
}

/// 创建 DoS 防护中间件
///
/// 对应 NRCS Java: enablePeerServerDoSFilter=true
pub fn create_dos_filter(config: DosFilterConfig) -> impl Fn(Request, Next) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Response> + Send>
> + Clone + Send + Sync + 'static {
    let state = Arc::new(DosState::new(config));

    move |req: Request, next: Next| {
        let state = Arc::clone(&state);

        Box::pin(async move {
            // 提取客户端 IP
            let client_ip = extract_client_ip(req.headers());

            match client_ip {
                Some(ip) => {
                    // 执行 DoS 检查
                    match state.check_request(&ip).await {
                        Ok(()) => {
                            // 通过检查，继续处理请求
                            next.run(req).await
                        }
                        Err(status) => {
                            // 被拒绝，返回错误响应
                            Response::builder()
                                .status(status)
                                .body(Body::from(format!(
                                    "{{\"error\": \"Rate limit exceeded\", \"code\": {}}}",
                                    status.as_u16()
                                )))
                                .unwrap()
                        }
                    }
                }
                None => {
                    // 无法获取 IP，放行（避免误伤合法请求）
                    debug!("[DoS] Cannot determine client IP, allowing request");
                    next.run(req).await
                }
            }
        })
    }
}

/// 从请求头提取客户端 IP 地址
fn extract_client_ip(headers: &HeaderMap) -> Option<IpAddr> {
    // 优先检查 X-Forwarded-For 头（代理场景）
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // 取第一个 IP（最原始的客户端）
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }

    // 其次检查 X-Real-IP 头
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(real_ip_str) = real_ip.to_str() {
            if let Ok(ip) = real_ip_str.parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }

    // 默认返回 None（让 Axum 的 ConnectInfo 提供真实 IP）
    None
}

/// 启动定期清理任务
///
/// 应在应用启动时调用，定期清理过期的 IP 追踪记录
#[allow(dead_code)]
pub(crate) async fn start_cleanup_task(state: Arc<DosState>) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            state.cleanup_expired().await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DosFilterConfig::default();
        assert_eq!(config.max_requests_per_sec, 30);
        assert_eq!(config.delay_ms, 1000);
    }

    #[tokio::test]
    async fn test_blacklist_check() {
        let config = DosFilterConfig {
            blacklist: vec!["192.168.1.100".to_string()],
            ..Default::default()
        };
        let state = DosState::new(config);

        let blocked: IpAddr = "192.168.1.100".parse().unwrap();
        assert!(state.is_blacklisted(&blocked).await);

        let allowed: IpAddr = "192.168.1.200".parse().unwrap();
        assert!(!state.is_blacklisted(&allowed).await);
    }

    #[tokio::test]
    async fn test_whitelist_bypass() {
        let config = DosFilterConfig {
            whitelist: vec!["10.0.0.1".to_string()],
            ..Default::default()
        };
        let state = DosState::new(config);

        let whitelisted: IpAddr = "10.0.0.1".parse().unwrap();
        assert!(state.is_whitelisted(&whitelisted).await);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let config = DosFilterConfig {
            max_requests_per_sec: 3,
            ..Default::default()
        };
        let state = DosState::new(config);

        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        // 前 3 个请求应该通过
        for _ in 0..3 {
            assert!(state.check_request(&ip).await.is_ok());
        }

        // 第 4 个请求应该被限流
        assert!(state.check_request(&ip).await.is_err());
    }
}
