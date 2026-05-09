//! API 分类标签定义
//!
//! 与 Java 版本 APITag 枚举完全对齐

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ApiTag {
    #[serde(rename = "Accounts")]
    Accounts,
    
    #[serde(rename = "Account Control")]
    AccountControl,
    
    #[serde(rename = "Aliases")]
    Aliases,
    
    #[serde(rename = "Asset Exchange")]
    Ae,
    
    #[serde(rename = "Blocks")]
    Blocks,
    
    #[serde(rename = "Create Transaction")]
    CreateTransaction,
    
    #[serde(rename = "Digital Goods Store")]
    Dgs,
    
    #[serde(rename = "Forging")]
    Forging,
    
    #[serde(rename = "Messages")]
    Messages,
    
    #[serde(rename = "Monetary System")]
    Ms,
    
    #[serde(rename = "Networking")]
    Network,
    
    #[serde(rename = "Phasing")]
    Phasing,
    
    #[serde(rename = "Search")]
    Search,
    
    #[serde(rename = "Server Info")]
    Info,
    
    #[serde(rename = "Shuffling")]
    Shuffling,
    
    #[serde(rename = "Tagged Data")]
    Data,
    
    #[serde(rename = "Tokens")]
    Tokens,
    
    #[serde(rename = "Transactions")]
    Transactions,
    
    #[serde(rename = "Voting System")]
    Vs,
    
    #[serde(rename = "Utils")]
    Utils,
    
    #[serde(rename = "Debug")]
    Debug,
    
    #[serde(rename = "Add-ons")]
    Addons,
    
    #[serde(rename = "Coin Exchange")]
    Ce,
}

impl ApiTag {
    pub fn name(&self) -> &'static str {
        match self {
            ApiTag::Accounts => "ACCOUNTS",
            ApiTag::AccountControl => "ACCOUNT_CONTROL",
            ApiTag::Aliases => "ALIASES",
            ApiTag::Ae => "AE",
            ApiTag::Blocks => "BLOCKS",
            ApiTag::CreateTransaction => "CREATE_TRANSACTION",
            ApiTag::Dgs => "DGS",
            ApiTag::Forging => "FORGING",
            ApiTag::Messages => "MESSAGES",
            ApiTag::Ms => "MS",
            ApiTag::Network => "NETWORK",
            ApiTag::Phasing => "PHASING",
            ApiTag::Search => "SEARCH",
            ApiTag::Info => "INFO",
            ApiTag::Shuffling => "SHUFFLING",
            ApiTag::Data => "DATA",
            ApiTag::Tokens => "TOKENS",
            ApiTag::Transactions => "TRANSACTIONS",
            ApiTag::Vs => "VS",
            ApiTag::Utils => "UTILS",
            ApiTag::Debug => "DEBUG",
            ApiTag::Addons => "ADDONS",
            ApiTag::Ce => "CE",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ApiTag::Accounts => "Accounts",
            ApiTag::AccountControl => "Account Control",
            ApiTag::Aliases => "Aliases",
            ApiTag::Ae => "Asset Exchange",
            ApiTag::Blocks => "Blocks",
            ApiTag::CreateTransaction => "Create Transaction",
            ApiTag::Dgs => "Digital Goods Store",
            ApiTag::Forging => "Forging",
            ApiTag::Messages => "Messages",
            ApiTag::Ms => "Monetary System",
            ApiTag::Network => "Networking",
            ApiTag::Phasing => "Phasing",
            ApiTag::Search => "Search",
            ApiTag::Info => "Server Info",
            ApiTag::Shuffling => "Shuffling",
            ApiTag::Data => "Tagged Data",
            ApiTag::Tokens => "Tokens",
            ApiTag::Transactions => "Transactions",
            ApiTag::Vs => "Voting System",
            ApiTag::Utils => "Utils",
            ApiTag::Debug => "Debug",
            ApiTag::Addons => "Add-ons",
            ApiTag::Ce => "Coin Exchange",
        }
    }
    
    pub fn all() -> &'static [ApiTag] {
        &[
            ApiTag::Accounts,
            ApiTag::AccountControl,
            ApiTag::Aliases,
            ApiTag::Ae,
            ApiTag::Blocks,
            ApiTag::CreateTransaction,
            ApiTag::Dgs,
            ApiTag::Forging,
            ApiTag::Messages,
            ApiTag::Ms,
            ApiTag::Network,
            ApiTag::Phasing,
            ApiTag::Search,
            ApiTag::Info,
            ApiTag::Shuffling,
            ApiTag::Data,
            ApiTag::Tokens,
            ApiTag::Transactions,
            ApiTag::Vs,
            ApiTag::Utils,
            ApiTag::Debug,
            ApiTag::Addons,
            ApiTag::Ce,
        ]
    }
}

impl fmt::Display for ApiTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
