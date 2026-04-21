//! 全局常量定义 - 对应 Java Constant.java
//!
//! 所有常量集中管理，确保与NRCS Java版本一致
//! 参考: /mnt/d/workspace/git/nrcs/nrcs-common/src/main/java/com/bytechain/nrcs/common/Constant.java

// ============ 版本信息 ============

pub const VERSION: &str = "2.1.0";
pub const APPLICATION: &str = "NRCS";
pub const CHAIN: &str = "NRCS";

// ============ 区块链核心常量 ============

pub const BLOCK_VERSION: u32 = 1;
pub const TRANSACTION_VERSION: u8 = 1;

pub const BLOCK_TIME: u32 = 60;
pub const MAX_NUMBER_OF_TRANSACTIONS: usize = 5000;
pub const MIN_TRANSACTION_SIZE: usize = 176;
pub const MAX_PAYLOAD_LENGTH: usize = MAX_NUMBER_OF_TRANSACTIONS * MIN_TRANSACTION_SIZE;

pub const MAX_BALANCE_NRCS: u64 = 1_000_000_000;
pub const ONE_NRCS: u64 = 100_000_000;
pub const ONE_FXT: u64 = ONE_NRCS;
pub const MAX_BALANCE_NQT: u64 = MAX_BALANCE_NRCS * ONE_NRCS;
pub const MAX_BALANCE_FXT: u64 = MAX_BALANCE_NRCS;

// ============ PoS共识常量 ============

pub const INITIAL_BASE_TARGET: u64 = 153722867;
pub const MAX_BASE_TARGET: u64 = MAX_BALANCE_NRCS * INITIAL_BASE_TARGET;
pub const MIN_BASE_TARGET: u64 = INITIAL_BASE_TARGET * 9 / 10;
pub const MIN_BLOCKTIME_LIMIT: u32 = 53;
pub const MAX_BLOCKTIME_LIMIT: u32 = 67;
pub const BASE_TARGET_GAMMA: u32 = 64;
pub const MIN_FORGING_BALANCE_NQT: u64 = 1000 * ONE_NRCS;

// ============ 交易相关常量 ============

pub const MAX_TIMEDRIFT: u32 = 15;
pub const MAX_PHASING_VOTE_TRANSACTIONS: u8 = 10;
pub const MAX_PHASING_WHITELIST_SIZE: u8 = 10;
pub const MAX_PHASING_LINKED_TRANSACTIONS: u8 = 10;
pub const MAX_PHASING_DURATION: u32 = 14 * 1440;
pub const MAX_PHASING_REVEALED_SECRETS_COUNT: u32 = 10;
pub const MAX_PHASING_REVEALED_SECRET_LENGTH: u32 = 100;
pub const MAX_PHASING_COMPOSITE_VOTE_EXPRESSION_LENGTH: u32 = 1000;
pub const MAX_PHASING_COMPOSITE_VOTE_SUBPOLL_NAME_LENGTH: u32 = 10;
pub const MAX_PHASING_COMPOSITE_VOTE_VARIABLES_COUNT: u32 = 20;
pub const MAX_PHASING_COMPOSITE_VOTE_LITERALS_COUNT: u32 = 30;

pub const MAX_ALIAS_URI_LENGTH: usize = 1000;
pub const MAX_ALIAS_LENGTH: usize = 100;
pub const MAX_ARBITRARY_MESSAGE_LENGTH: usize = 160;
pub const MAX_ENCRYPTED_MESSAGE_LENGTH: usize = 160 + 16;
pub const MAX_PRUNABLE_MESSAGE_LENGTH: usize = 42 * 10 * 1024;
pub const MAX_PRUNABLE_ENCRYPTED_MESSAGE_LENGTH: usize = 42 * 10 * 1024;

// ============ 账户相关常量 ============

pub const MAX_ACCOUNT_NAME_LENGTH: usize = 100;
pub const MAX_ACCOUNT_DESCRIPTION_LENGTH: usize = 1000;
pub const MAX_ACCOUNT_PROPERTY_NAME_LENGTH: usize = 32;
pub const MAX_ACCOUNT_PROPERTY_VALUE_LENGTH: usize = 255;
pub const MAX_ACCOUNT_PROPERTY_LONG_VALUE_LENGTH: usize = 8 * 1024;

// ============ 资产相关常量 ============

pub const MAX_ASSET_QUANTITY_QNT: u64 = 1_000_000_000 * 100_000_000;
pub const MIN_ASSET_NAME_LENGTH: usize = 3;
pub const MAX_ASSET_NAME_LENGTH: usize = 10;
pub const MAX_ASSET_DESCRIPTION_LENGTH: usize = 1000;
pub const MAX_ASSET_PROPERTY_NAME_LENGTH: usize = 32;
pub const MAX_ASSET_PROPERTY_VALUE_LENGTH: usize = 255;
pub const MAX_ASSET_PROPERTY_LONG_VALUE_LENGTH: usize = 8 * 1024;
pub const MAX_SINGLETON_ASSET_DESCRIPTION_LENGTH: usize = 160;
pub const MAX_ASSET_TRANSFER_COMMENT_LENGTH: usize = 1000;
pub const MAX_DIVIDEND_PAYMENT_ROLLBACK: u32 = 1441;

// ============ 投票相关常量 ============

pub const MAX_POLL_NAME_LENGTH: usize = 100;
pub const MAX_POLL_DESCRIPTION_LENGTH: usize = 1000;
pub const MAX_POLL_OPTION_LENGTH: usize = 100;
pub const MAX_POLL_OPTION_COUNT: usize = 100;
pub const MAX_POLL_DURATION: u32 = 14 * 1440;
pub const MIN_VOTE_VALUE: i8 = -92;
pub const MAX_VOTE_VALUE: i8 = 92;
pub const NO_VOTE_VALUE: i8 = i8::MIN;

// ============ 数字商品相关常量 ============

pub const MAX_DGS_LISTING_QUANTITY: u32 = 1_000_000_000;
pub const MAX_DGS_LISTING_NAME_LENGTH: usize = 100;
pub const MAX_DGS_LISTING_DESCRIPTION_LENGTH: usize = 1000;
pub const MAX_DGS_LISTING_TAGS_LENGTH: usize = 100;
pub const MAX_DGS_GOODS_LENGTH: usize = 1000;

// ============ Hub相关常量 ============

pub const MAX_HUB_ANNOUNCEMENT_URIS: usize = 100;
pub const MAX_HUB_ANNOUNCEMENT_URI_LENGTH: usize = 1000;
pub const MIN_HUB_EFFECTIVE_BALANCE: u64 = 100_000;

// ============ 货币系统常量 ============

pub const MIN_CURRENCY_NAME_LENGTH: usize = 3;
pub const MAX_CURRENCY_NAME_LENGTH: usize = 10;
pub const MIN_CURRENCY_CODE_LENGTH: usize = 3;
pub const MAX_CURRENCY_CODE_LENGTH: usize = 5;
pub const MAX_CURRENCY_DESCRIPTION_LENGTH: usize = 1000;
pub const MAX_CURRENCY_TOTAL_SUPPLY: u64 = 1_000_000_000 * 100_000_000;
pub const MAX_MINTING_RATIO: u32 = 10000;
pub const CURRENCY_MINT_FEE: u64 = ONE_NRCS;

// ============ Shuffling相关常量 ============

pub const MIN_NUMBER_OF_SHUFFLING_PARTICIPANTS: u8 = 3;
pub const MAX_NUMBER_OF_SHUFFLING_PARTICIPANTS: u8 = 30;
pub const MAX_SHUFFLING_REGISTRATION_PERIOD: u16 = 1440 * 7;

// ============ 标签数据相关常量 ============

pub const MAX_TAGGED_DATA_NAME_LENGTH: usize = 100;
pub const MAX_TAGGED_DATA_DESCRIPTION_LENGTH: usize = 1000;
pub const MAX_TAGGED_DATA_TAGS_LENGTH: usize = 100;
pub const MAX_TAGGED_DATA_TYPE_LENGTH: usize = 100;
pub const MAX_TAGGED_DATA_CHANNEL_LENGTH: usize = 100;
pub const MAX_TAGGED_DATA_FILENAME_LENGTH: usize = 100;
pub const MAX_TAGGED_DATA_DATA_LENGTH: usize = 42 * 10 * 1024;

// ============ 合约相关常量 ============

pub const MAX_CONTRACT_NAME_LENGTH: usize = 32;
pub const MAX_CONTRACT_PARAMS_LENGTH: usize = 160;

// ============ 区块高度相关常量 ============

pub const TRANSPARENT_FORGING_BLOCK: i32 = -1;
pub const TRANSPARENT_FORGING_BLOCK_3: u32 = 51000;
pub const TRANSPARENT_FORGING_BLOCK_5: i32 = -1;
pub const TRANSPARENT_FORGING_BLOCK_7: u32 = u32::MAX;
pub const MAX_REFERENCED_TRANSACTION_TIMESPAN: u32 = 60 * 1440 * 60;

// ============ 时间相关常量 ============

pub const MAX_ROLLBACK: u32 = 720;
pub const GUARANTEED_BALANCE_CONFIRMATIONS: u32 = 1440;
pub const LEASING_DELAY: u32 = 1440;
pub const FORGING_DELAY: u32 = 0;
pub const FORGING_SPEEDUP: u32 = 0;
pub const BATCH_COMMIT_SIZE: u32 = u32::MAX;
pub const MIN_PRUNABLE_LIFETIME: u32 = 14 * 1440 * 60;

// ============ 费用相关常量 ============

pub const UNCONFIRMED_POOL_DEPOSIT_NQT: u64 = 100 * ONE_NRCS;
pub const UNCONFIRMED_POOL_DEPOSIT_FQT: u64 = 10 * ONE_NRCS;
pub const SHUFFLING_DEPOSIT_NQT: u64 = 1000 * ONE_NRCS;

// ============ P2P网络常量 ============

pub const MAX_KNOWN_PEERS: usize = 2000;
pub const MIN_KNOWN_PEERS: usize = 100;
pub const MAX_CONNECTIONS: usize = 20;
pub const MAX_INBOUND_CONNECTIONS: usize = 100;
pub const MAX_OUTBOUND_CONNECTIONS: usize = 20;
pub const MAX_REQUEST_SIZE: usize = 64 * 1024 * 1024;
pub const MAX_RESPONSE_SIZE: usize = 64 * 1024 * 1024;
pub const MAX_MESSAGE_SIZE: usize = 40 * 1024 * 1024;
pub const MIN_COMPRESS_SIZE: usize = 256;

pub const MAX_VERSION_LENGTH: usize = 10;
pub const MAX_APPLICATION_LENGTH: usize = 20;
pub const MAX_PLATFORM_LENGTH: usize = 30;
pub const MAX_ANNOUNCED_ADDRESS_LENGTH: usize = 100;

pub const CONNECT_TIMEOUT_MS: u64 = 4000;
pub const READ_TIMEOUT_MS: u64 = 4000;
pub const WEBSOCKET_IDLE_TIMEOUT_SECS: u64 = 300;
pub const BLACKLISTING_PERIOD_SECS: i64 = 3600;
pub const BLACKLISTING_THRESHOLD: i32 = 10;

pub const CONNECTION_DAEMON_INTERVAL_SECS: u64 = 5;
pub const DISCOVERY_DAEMON_INTERVAL_SECS: u64 = 30;
pub const UNBLACKLIST_DAEMON_INTERVAL_SECS: u64 = 60;
pub const TRANSACTION_DAEMON_INTERVAL_SECS: u64 = 30;
pub const SEND_TRANSACTIONS_BATCH_SIZE: usize = 10;
pub const BUNDLER_RATE_BROADCAST_INTERVAL_SECS: u64 = 30 * 60;

// ============ 字符集常量 ============

pub const ALPHABET: &str = "0123456789abcdefghijklmnopqrstuvwxyz";
pub const ALLOWED_CURRENCY_CODE_LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

// ============ 区块链状态常量 ============

pub const LAST_KNOWN_BLOCK: i32 = -1;

// ============ 辅助函数 ============

pub fn one_nrcs() -> u64 {
    ONE_NRCS
}

pub fn max_balance_nqt() -> u64 {
    MAX_BALANCE_NQT
}

pub fn initial_base_target() -> u64 {
    INITIAL_BASE_TARGET
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_consistency() {
        assert_eq!(ONE_NRCS, ONE_FXT);
        assert_eq!(ONE_NRCS, 100_000_000);
        assert_eq!(MAX_BALANCE_NQT, MAX_BALANCE_NRCS * ONE_NRCS);
        assert_eq!(MAX_PAYLOAD_LENGTH, MAX_NUMBER_OF_TRANSACTIONS * MIN_TRANSACTION_SIZE);
    }

    #[test]
    fn test_pos_constants() {
        assert!(INITIAL_BASE_TARGET > 0);
        assert!(MAX_BASE_TARGET > INITIAL_BASE_TARGET);
        assert!(MIN_BASE_TARGET < INITIAL_BASE_TARGET);
        assert!(MIN_FORGING_BALANCE_NQT > 0);
    }

    #[test]
    fn test_time_constants() {
        assert_eq!(BLOCK_TIME, 60);
        assert_eq!(MAX_TIMEDRIFT, 15);
        assert!(MAX_ROLLBACK >= 720);
    }

    #[test]
    fn test_p2p_constants() {
        assert!(MAX_KNOWN_PEERS > MIN_KNOWN_PEERS);
        assert!(MAX_CONNECTIONS > 0);
        assert!(MAX_INBOUND_CONNECTIONS > MAX_OUTBOUND_CONNECTIONS);
    }
}
