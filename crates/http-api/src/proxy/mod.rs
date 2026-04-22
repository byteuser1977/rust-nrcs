//! API 代理模块
//!
//! 对应 Java: APIProxyServlet, APIProxy, PasswordFilteringContentTransformer
//!
//! 功能：
//! - 密码过滤：检测请求中的敏感参数（secretPhrase, adminPassword, sharedKey）
//! - 请求代理：将请求转发到其他节点
//! - 黑名单管理：管理失败的代理节点

pub mod password_filter;
pub mod api_proxy;
pub mod handler;

pub use password_filter::{PasswordFilter, SensitiveParameter};
pub use api_proxy::ApiProxy;
pub use handler::ProxyRequestHandler;
