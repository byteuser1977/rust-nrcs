//! Genesis block creation aligned with Java GenesisGenerator.

use serde_json::Value;
use std::fs;
use chrono::{NaiveDate, FixedOffset, TimeZone};
use blockchain_types::constants::{GENESIS_BLOCK_ID, INITIAL_BASE_TARGET};
use crate::models::*;
use crate::repository::*;
use crate::RepositoryResult;

/// Load genesis configuration from config/genesis.json.
/// Returns (timestamp, Vec<(account_id, balance)>)
fn load_genesis_config() -> RepositoryResult<(i64, Vec<(i64, i64)>)> {
    let content = fs::read_to_string("config/genesis.json")
        .map_err(crate::RepositoryError::Io)?;
    let json: Value = serde_json::from_str(&content)
        .map_err(|e| crate::RepositoryError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        )))?;

    let time_str = json["genesis_time"].as_str()
        .ok_or_else(|| crate::RepositoryError::Validation("missing genesis_time".to_string()))?;

    let _ = json.get("secret_phrase");

    let transactions = json["transactions"].as_array()
        .ok_or_else(|| crate::RepositoryError::Validation("missing transactions array".to_string()))?;

    // Parse time in GMT+8, format: "yyyy-M-d HH:mm:ss.SSS"
    let timestamp = parse_genesis_time(time_str)?;

    let mut accounts = Vec::new();
    for tx in transactions {
        let recipient = tx["recipient"].as_str()
            .ok_or_else(|| crate::RepositoryError::Validation("recipient not a string".to_string()))?;
        
        let recipient_id: i64 = if recipient.starts_with('-') {
            let unsigned_val: u64 = recipient.parse()
                .map_err(|e| crate::RepositoryError::Validation(format!("invalid recipient: {}", e)))?;
            unsigned_val as i64
        } else {
            match recipient.parse::<u64>() {
                Ok(unsigned_val) => unsigned_val as i64,
                Err(_) => {
                    recipient.parse()
                        .map_err(|e| crate::RepositoryError::Validation(format!("invalid recipient: {}", e)))?
                }
            }
        };
        
        let amount = tx["amount"].as_i64()
            .ok_or_else(|| crate::RepositoryError::Validation("invalid amount".to_string()))?;
        accounts.push((recipient_id, amount));
    }

    Ok((timestamp, accounts))
}

/// Parse genesis_time string in GMT+8, format "yyyy-M-d HH:mm:ss.SSS"
fn parse_genesis_time(s: &str) -> RepositoryResult<i64> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(crate::RepositoryError::Validation("expected date and time parts".to_string()));
    }

    let date_fields: Vec<&str> = parts[0].split('-').collect();
    if date_fields.len() != 3 {
        return Err(crate::RepositoryError::Validation("invalid date format".to_string()));
    }
    let year = date_fields[0].parse::<i32>()
        .map_err(|e| crate::RepositoryError::Validation(format!("invalid year: {}", e)))?;
    let month = date_fields[1].parse::<u32>()
        .map_err(|e| crate::RepositoryError::Validation(format!("invalid month: {}", e)))?;
    let day = date_fields[2].parse::<u32>()
        .map_err(|e| crate::RepositoryError::Validation(format!("invalid day: {}", e)))?;

    let time_str = parts[1];
    let mut time_parts = time_str.split(':');
    let hour = time_parts.next()
        .and_then(|s| s.parse::<u32>().ok())
        .ok_or_else(|| crate::RepositoryError::Validation("invalid hour".to_string()))?;
    let minute = time_parts.next()
        .and_then(|s| s.parse::<u32>().ok())
        .ok_or_else(|| crate::RepositoryError::Validation("invalid minute".to_string()))?;
    let sec_ms = time_parts.next()
        .ok_or_else(|| crate::RepositoryError::Validation("missing seconds".to_string()))?;

    let (second, millisecond) = if let Some((sec, ms)) = sec_ms.split_once('.') {
        let sec = sec.parse::<u32>()
            .map_err(|e| crate::RepositoryError::Validation(format!("invalid seconds: {}", e)))?;
        let ms = ms.parse::<u32>()
            .map_err(|e| crate::RepositoryError::Validation(format!("invalid milliseconds: {}", e)))?;
        (sec, ms)
    } else {
        let sec = sec_ms.parse::<u32>()
            .map_err(|e| crate::RepositoryError::Validation(format!("invalid seconds: {}", e)))?;
        (sec, 0)
    };

    let ndt = NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|d| d.and_hms_milli_opt(hour, minute, second, millisecond))
        .ok_or_else(|| crate::RepositoryError::Validation("invalid date/time values".to_string()))?;

    let tz = FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| crate::RepositoryError::Validation("invalid timezone offset".to_string()))?;
    let dt = tz.from_local_datetime(&ndt)
        .single()
        .ok_or_else(|| crate::RepositoryError::Validation("datetime ambiguity or out-of-range".to_string()))?;
    Ok(dt.timestamp())
}

/// Create and insert the genesis block if none exists.
/// Mirrors Java GenesisGenerator logic but without secret phrase requirement.
pub async fn ensure_genesis(
    block_repo: &dyn BlockRepository,
    account_repo: &dyn AccountRepository,
    tx_repo: &dyn TransactionRepository,
    ledger_repo: &dyn AccountLedgerRepository,
    guaranteed_balance_repo: &dyn AccountGuaranteedBalanceRepository,
) -> RepositoryResult<()> {
    let count = block_repo.count().await?;
    if count > 0 {
        return Ok(());
    }

    let (_unix_timestamp, accounts) = load_genesis_config()?;
    
    let one_nrcs_nqt: i64 = 100_000_000;
    let total_amount: i64 = accounts.iter().map(|(_, amt)| *amt * one_nrcs_nqt).sum();
    let height = 0i32;
    let block_id = GENESIS_BLOCK_ID as i64;
    let timestamp = 0i32;

    let generation_signature: Vec<u8> = vec![0u8; 64];
    
    let block_signature_hex = "47b1aa800d657ccad4aaa8c946b2b0d2a7337fd3ab8e8c9ed6a06a49b7756e04a3ff13b15f6471afdff30313e1c47c4c2ab0e209c78a0673a42c254b74cc0201";
    let block_signature = hex::decode(block_signature_hex)
        .map_err(|e| crate::RepositoryError::Validation(format!("invalid block signature: {}", e)))?;
    
    let payload_hash_hex = "8f58dc2f809613424e608586df83b42513056861a864dff3cd00d88baca681ce";
    let payload_hash = hex::decode(payload_hash_hex)
        .map_err(|e| crate::RepositoryError::Validation(format!("invalid payload hash: {}", e)))?;
    
    let generator_id: i64 = 18365787021584764528u64 as i64;
    
    let cumulative_difficulty: Vec<u8> = vec![0u8; 1];

    let block_model = BlockModel {
        db_id: 0,
        id: block_id,
        version: -1,
        timestamp,
        previous_block_id: None,
        total_amount,
        total_fee: 0,
        payload_length: 256,
        previous_block_hash: None,
        cumulative_difficulty,
        base_target: INITIAL_BASE_TARGET as i64,
        next_block_id: None,
        height,
        generation_signature,
        block_signature,
        payload_hash,
        generator_id,
    };

    block_repo.insert(&block_model).await?;

    for (account_id, balance) in accounts {
        let balance_nqt = balance * one_nrcs_nqt;
        
        let account_model = AccountModel {
            db_id: 0,
            id: account_id,
            balance: balance_nqt,
            unconfirmed_balance: balance_nqt,
            forged_balance: 0,
            active_lessee_id: None,
            has_control_phasing: false,
            height,
            latest: true,
        };
        account_repo.insert(&account_model).await?;

        if balance_nqt > 0 {
            guaranteed_balance_repo.upsert_additions(account_id, height, balance_nqt).await?;
        }

        let ledger_model = AccountLedgerModel {
            db_id: 0,
            account_id,
            event_type: 3, // LedgerEvent.ORDINARY_PAYMENT
            event_id: 1,
            holding_type: 2, // LedgerHolding.NRCS_BALANCE
            holding_id: None,
            change: balance_nqt,
            balance: balance_nqt,
            block_id,
            height,
            timestamp,
        };
        ledger_repo.insert(&ledger_model).await?;
    }

    let genesis_tx_ids: Vec<i64> = vec![
        -6309664432798542337i64,
        2830446832482296829i64,
    ];
    let genesis_recipients: Vec<i64> = vec![
        2794603741293765856i64,
        -891382425467438890i64,
    ];
    let genesis_amounts: Vec<i64> = vec![
        999999999i64 * one_nrcs_nqt,
        one_nrcs_nqt,
    ];
    let genesis_fees: Vec<i64> = vec![
        100000000i64,
        100000000i64,
    ];
    let genesis_full_hashes: Vec<&str> = vec![
        "ff5520dfa0916fa8d3dd275b4cde3b6175a8b7599bd9181baeb8d26d67a04b09",
        "fda3bcd565c447276763a9de4a97e1da95949559e050ecfd72fa81a4fd62a9d8",
    ];
    let genesis_signatures: Vec<&str> = vec![
        "182dd1a3abb456961ac0d7852a0892bfa57e306f727f00a7dce6c0ce7bd53d09f0a7be7bd5c25ff172e17133646f8afae51d",
        "a8745bcf08a361a6baed9611df10bafd037051b26c9b8167bf1cbb3a357ddb072adfd1d43bf93bb9379446ed5aa7ecbb21ae",
    ];

    for idx in 0..genesis_tx_ids.len() {
        let tx_id = genesis_tx_ids[idx];
        let recipient_id = genesis_recipients[idx];
        let amount = genesis_amounts[idx];
        let fee = genesis_fees[idx];
        let full_hash = hex::decode(genesis_full_hashes[idx])
            .map_err(|e| crate::RepositoryError::Validation(format!("invalid full hash: {}", e)))?;
        let signature = hex::decode(genesis_signatures[idx])
            .map_err(|e| crate::RepositoryError::Validation(format!("invalid signature: {}", e)))?;

        let tx_model = TransactionModel {
            db_id: 0,
            id: tx_id,
            deadline: 1440,
            recipient_id: Some(recipient_id),
            amount,
            fee,
            full_hash,
            height,
            block_id,
            signature,
            timestamp,
            r#type: 0,
            subtype: 0,
            sender_id: generator_id,
            block_timestamp: timestamp,
            referenced_transaction_full_hash: None,
            transaction_index: idx as i16,
            phased: false,
            attachment_bytes: None,
            version: 0,
            has_message: false,
            has_encrypted_message: false,
            has_public_key_announcement: false,
            has_prunable_message: false,
            has_prunable_attachment: false,
            ec_block_height: None,
            ec_block_id: None,
            has_encrypttoself_message: false,
            has_prunable_encrypted_message: false,
        };

        tx_repo.insert(&tx_model).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use crate::repository::{SqliteBlockRepository, SqliteAccountRepository, SqliteTransactionRepository, SqliteAccountLedgerRepository};

    /// 创建测试所需的表
    ///
    /// 测试使用内联 SQL 而非 migration 文件，原因：
    /// 1. 测试应与外部文件解耦（隔离性）
    /// 2. migration 使用 PostgreSQL 语法 (BIGSERIAL)，不适合 SQLite 内存库
    /// 3. 测试只创建所需表，更快更清晰
    async fn setup_repos() -> (
        SqliteBlockRepository,
        SqliteAccountRepository,
        SqliteTransactionRepository,
        SqliteAccountLedgerRepository,
        SqliteAccountGuaranteedBalanceRepository,
    ) {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create pool");

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS block (
                DB_ID INTEGER PRIMARY KEY AUTOINCREMENT,
                ID INTEGER NOT NULL,
                VERSION INTEGER NOT NULL,
                TIMESTAMP INTEGER NOT NULL,
                PREVIOUS_BLOCK_ID INTEGER,
                TOTAL_AMOUNT INTEGER NOT NULL,
                TOTAL_FEE INTEGER NOT NULL,
                PAYLOAD_LENGTH INTEGER NOT NULL,
                PREVIOUS_BLOCK_HASH BLOB,
                CUMULATIVE_DIFFICULTY BLOB NOT NULL,
                BASE_TARGET INTEGER NOT NULL,
                NEXT_BLOCK_ID INTEGER,
                HEIGHT INTEGER NOT NULL,
                GENERATION_SIGNATURE BLOB,
                BLOCK_SIGNATURE BLOB,
                PAYLOAD_HASH BLOB,
                GENERATOR_ID INTEGER NOT NULL
            )
        "#)
        .execute(&pool)
        .await
        .expect("Failed to create block table");

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS account (
                DB_ID INTEGER PRIMARY KEY AUTOINCREMENT,
                ID INTEGER NOT NULL,
                BALANCE INTEGER NOT NULL,
                UNCONFIRMED_BALANCE INTEGER NOT NULL,
                FORGED_BALANCE INTEGER NOT NULL,
                ACTIVE_LESSEE_ID INTEGER,
                HAS_CONTROL_PHASING INTEGER NOT NULL DEFAULT 0,
                HEIGHT INTEGER NOT NULL,
                LATEST INTEGER NOT NULL DEFAULT 1
            )
        "#)
        .execute(&pool)
        .await
        .expect("Failed to create account table");

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS account_ledger (
                DB_ID INTEGER PRIMARY KEY AUTOINCREMENT,
                ACCOUNT_ID INTEGER NOT NULL,
                EVENT_TYPE INTEGER NOT NULL,
                EVENT_ID INTEGER NOT NULL,
                HOLDING_TYPE INTEGER NOT NULL,
                HOLDING_ID INTEGER,
                "CHANGE" INTEGER NOT NULL,
                BALANCE INTEGER NOT NULL,
                BLOCK_ID INTEGER NOT NULL,
                HEIGHT INTEGER NOT NULL,
                TIMESTAMP INTEGER NOT NULL
            )
        "#)
        .execute(&pool)
        .await
        .expect("Failed to create account_ledger table");

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS "transaction" (
                DB_ID INTEGER PRIMARY KEY AUTOINCREMENT,
                ID INTEGER NOT NULL,
                DEADLINE INTEGER NOT NULL,
                RECIPIENT_ID INTEGER,
                AMOUNT INTEGER NOT NULL,
                FEE INTEGER NOT NULL,
                FULL_HASH BLOB NOT NULL,
                HEIGHT INTEGER NOT NULL,
                BLOCK_ID INTEGER NOT NULL,
                SIGNATURE BLOB NOT NULL,
                TIMESTAMP INTEGER NOT NULL,
                TYPE INTEGER NOT NULL,
                SUBTYPE INTEGER NOT NULL,
                SENDER_ID INTEGER NOT NULL,
                BLOCK_TIMESTAMP INTEGER NOT NULL,
                REFERENCED_TRANSACTION_FULL_HASH BLOB,
                TRANSACTION_INDEX INTEGER NOT NULL,
                PHASED INTEGER NOT NULL DEFAULT 0,
                ATTACHMENT_BYTES BLOB,
                VERSION INTEGER NOT NULL DEFAULT 0,
                HAS_MESSAGE INTEGER NOT NULL DEFAULT 0,
                HAS_ENCRYPTED_MESSAGE INTEGER NOT NULL DEFAULT 0,
                HAS_PUBLIC_KEY_ANNOUNCEMENT INTEGER NOT NULL DEFAULT 0,
                HAS_PRUNABLE_MESSAGE INTEGER NOT NULL DEFAULT 0,
                HAS_PRUNABLE_ATTACHMENT INTEGER NOT NULL DEFAULT 0,
                EC_BLOCK_HEIGHT INTEGER,
                EC_BLOCK_ID INTEGER,
                HAS_ENCRYPTTOSELF_MESSAGE INTEGER NOT NULL DEFAULT 0,
                HAS_PRUNABLE_ENCRYPTED_MESSAGE INTEGER NOT NULL DEFAULT 0
            )
        "#)
        .execute(&pool)
        .await
        .expect("Failed to create transaction table");

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS account_guaranteed_balance (
                DB_ID INTEGER PRIMARY KEY AUTOINCREMENT,
                ACCOUNT_ID INTEGER NOT NULL,
                ADDITIONS INTEGER NOT NULL,
                HEIGHT INTEGER NOT NULL
            )
        "#)
        .execute(&pool)
        .await
        .expect("Failed to create account_guaranteed_balance table");

        (
            SqliteBlockRepository::new(pool.clone()),
            SqliteAccountRepository::new(pool.clone()),
            SqliteTransactionRepository::new(pool.clone()),
            SqliteAccountLedgerRepository::new(pool.clone()),
            SqliteAccountGuaranteedBalanceRepository::new(pool),
        )
    }

    #[tokio::test]
    async fn test_genesis_creates_initial_state() {
        let (block_repo, account_repo, tx_repo, ledger_repo, guaranteed_balance_repo) = setup_repos().await;
        
        let genesis_config = r#"{
            "genesis_time": "2024-1-1 00:00:00.000",
            "transactions": [
                {"recipient": "123456789", "amount": 1000000000},
                {"recipient": "987654321", "amount": 500000000}
            ]
        }"#;
        
        let temp_dir = std::env::temp_dir().join("nrcs_test_config_1");
        let config_dir = temp_dir.join("config");
        std::fs::create_dir_all(&config_dir).ok();
        let config_path = config_dir.join("genesis.json");
        std::fs::write(&config_path, genesis_config).ok();
        
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(&temp_dir).ok();
        
        let result = ensure_genesis(&block_repo, &account_repo, &tx_repo, &ledger_repo, &guaranteed_balance_repo).await;
        
        if let Some(dir) = original_dir {
            std::env::set_current_dir(dir).ok();
        }
        
        result.expect("Genesis creation failed");
        
        let count = block_repo.count().await.expect("Failed to count blocks");
        assert_eq!(count, 1);

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_genesis_block_height_is_zero() {
        let (block_repo, account_repo, tx_repo, ledger_repo, guaranteed_balance_repo) = setup_repos().await;
        
        let genesis_config = r#"{
            "genesis_time": "2024-1-1 00:00:00.000",
            "transactions": []
        }"#;
        
        let temp_dir = std::env::temp_dir().join("nrcs_test_config_2");
        let config_dir = temp_dir.join("config");
        std::fs::create_dir_all(&config_dir).ok();
        let config_path = config_dir.join("genesis.json");
        std::fs::write(&config_path, genesis_config).ok();
        
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(&temp_dir).ok();
        
        let result = ensure_genesis(&block_repo, &account_repo, &tx_repo, &ledger_repo, &guaranteed_balance_repo).await;
        
        if let Some(dir) = original_dir {
            std::env::set_current_dir(dir).ok();
        }
        
        result.expect("Genesis creation failed");
        
        let block = block_repo.find_latest().await.expect("Failed to get latest block");
        assert!(block.is_some());
        let block = block.unwrap();
        assert_eq!(block.height, 0, "Genesis block height should be 0");

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_genesis_base_target() {
        let (block_repo, account_repo, tx_repo, ledger_repo, guaranteed_balance_repo) = setup_repos().await;
        
        let genesis_config = r#"{
            "genesis_time": "2024-1-1 00:00:00.000",
            "transactions": []
        }"#;
        
        let temp_dir = std::env::temp_dir().join("nrcs_test_config_3");
        let config_dir = temp_dir.join("config");
        std::fs::create_dir_all(&config_dir).ok();
        let config_path = config_dir.join("genesis.json");
        std::fs::write(&config_path, genesis_config).ok();
        
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(&temp_dir).ok();
        
        let result = ensure_genesis(&block_repo, &account_repo, &tx_repo, &ledger_repo, &guaranteed_balance_repo).await;
        
        if let Some(dir) = original_dir {
            std::env::set_current_dir(dir).ok();
        }
        
        result.expect("Genesis creation failed");
        
        let block = block_repo.find_latest().await.expect("Failed to get latest block");
        assert!(block.is_some());
        let block = block.unwrap();
        assert_eq!(block.base_target, blockchain_types::constants::INITIAL_BASE_TARGET as i64, 
            "Genesis block base_target should be INITIAL_BASE_TARGET");

        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
