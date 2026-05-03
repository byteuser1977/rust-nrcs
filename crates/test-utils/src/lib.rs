//! # NRCS Test Utilities
//!
//! 共享测试基础设施，参照 Java NRCS 的 `AbstractBlockchainTest` → `BlockchainTest` → `Tester` 层次。
//!
//! 提供：
//! - 标准测试账户（FORGY/ALICE/BOB/CHUCK/DAVE/RIKER）
//! - 内存数据库创建与 Schema 初始化
//! - 自定义断言宏
//! - Mock 实现工厂

pub mod fixtures;
pub mod db_helper;
pub mod assertions;
pub mod mock_helpers;

pub use fixtures::*;
pub use db_helper::*;
pub use assertions::*;
