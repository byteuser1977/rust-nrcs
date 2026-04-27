//! Monitor Module
//!
//! 对应 Java: FundingMonitor.java, MonitoredAccount.java
//!
//! 资金监控系统，用于自动为账户充值

pub mod funding_monitor;
pub mod monitored_account;

pub use funding_monitor::{FundingMonitor, FundingMonitorService, HoldingType};
pub use monitored_account::MonitoredAccount;
