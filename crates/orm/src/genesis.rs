//! Genesis block creation aligned with Java GenesisGenerator.

use sqlx::AnyPool;
use serde_json::Value;
use std::fs;
use chrono::{NaiveDate, FixedOffset, TimeZone};
use blockchain_types::constants::{GENESIS_BLOCK_ID, INITIAL_BASE_TARGET};

/// Load genesis configuration from config/genesis.json.
/// Returns (timestamp, Vec<(account_id, balance)>)
fn load_genesis_config() -> sqlx::Result<(i64, Vec<(i64, i64)>)> {
    let content = fs::read_to_string("config/genesis.json")
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    let json: Value = serde_json::from_str(&content)
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

    let time_str = json["genesis_time"].as_str()
        .ok_or_else(|| sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "missing genesis_time",
        ))))?;

    // secret_phrase is optional (Java may prompt separately). Ignored here.
    let _ = json.get("secret_phrase");

    let transactions = json["transactions"].as_array()
        .ok_or_else(|| sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "missing transactions array",
        ))))?;

    // Parse time in GMT+8, format: "yyyy-M-d HH:mm:ss.SSS"
    let timestamp = parse_genesis_time(time_str)?;

    // Parse transactions
    // Note: Java uses Long.parseUnsignedLong for recipient IDs
    // We need to handle potentially large unsigned values
    let mut accounts = Vec::new();
    for tx in transactions {
        let recipient = tx["recipient"].as_str()
            .ok_or_else(|| sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "recipient not a string",
            ))))?;
        
        // Parse as unsigned first, then convert to signed (for database storage)
        let recipient_id: i64 = if recipient.starts_with('-') {
            // Handle negative string representation of unsigned values
            let unsigned_val: u64 = recipient.parse()
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            unsigned_val as i64
        } else {
            // Try parsing as unsigned first (for values > i64::MAX)
            match recipient.parse::<u64>() {
                Ok(unsigned_val) => unsigned_val as i64,
                Err(_) => {
                    // Fall back to signed parsing
                    recipient.parse()
                        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?
                }
            }
        };
        
        let amount = tx["amount"].as_i64()
            .ok_or_else(|| sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid amount",
            ))))?;
        accounts.push((recipient_id, amount));
    }

    Ok((timestamp, accounts))
}

/// Parse genesis_time string in GMT+8, format "yyyy-M-d HH:mm:ss.SSS"
fn parse_genesis_time(s: &str) -> sqlx::Result<i64> {
    // Split date and time
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "expected date and time parts",
        ))));
    }

    // Date: yyyy-M-d
    let date_fields: Vec<&str> = parts[0].split('-').collect();
    if date_fields.len() != 3 {
        return Err(sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid date format",
        ))));
    }
    let year = date_fields[0].parse::<i32>()
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    let month = date_fields[1].parse::<u32>()
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    let day = date_fields[2].parse::<u32>()
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

    // Time: HH:mm:ss.SSS
    let time_str = parts[1];
    let mut time_parts = time_str.split(':');
    let hour = time_parts.next()
        .and_then(|s| s.parse::<u32>().ok())
        .ok_or_else(|| sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid hour",
        ))))?;
    let minute = time_parts.next()
        .and_then(|s| s.parse::<u32>().ok())
        .ok_or_else(|| sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid minute",
        ))))?;
    let sec_ms = time_parts.next()
        .ok_or_else(|| sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "missing seconds",
        ))))?;

    let (second, millisecond) = if let Some((sec, ms)) = sec_ms.split_once('.') {
        let sec = sec.parse::<u32>()
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        let ms = ms.parse::<u32>()
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        (sec, ms)
    } else {
        let sec = sec_ms.parse::<u32>()
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        (sec, 0)
    };

    // Build NaiveDateTime
    let ndt = NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|d| d.and_hms_milli_opt(hour, minute, second, millisecond))
        .ok_or_else(|| sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid date/time values",
        ))))?;

    // Convert to GMT+8 timestamp (seconds since epoch)
    let tz = FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid timezone offset",
        ))))?;
    let dt = tz.from_local_datetime(&ndt)
        .single()
        .ok_or_else(|| sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "datetime ambiguity or out-of-range",
        ))))?;
    Ok(dt.timestamp())
}

/// Create and insert the genesis block if none exists.
/// Mirrors Java GenesisGenerator logic but without secret phrase requirement.
pub async fn ensure_genesis(pool: &AnyPool) -> sqlx::Result<()> {
    // Check if any block exists
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM block")
        .fetch_one(pool)
        .await?;
    if count.0 > 0 {
        return Ok(());
    }

    let (timestamp, accounts) = load_genesis_config()?;
    let total_amount: i64 = accounts.iter().map(|(_, amt)| *amt).sum();
    let height = 0i32; // 创世区块高度为 0
    let block_id = GENESIS_BLOCK_ID as i64;

    // Genesis block constants from Java NRCS
    // generation_signature: 64 bytes of zeros
    let generation_signature: Vec<u8> = vec![0u8; 64];
    
    // block_signature from Java NRCS genesis block
    let block_signature_hex = "47b1aa800d657ccad4aaa8c946b2b0d2a7337fd3ab8e8c9ed6a06a49b7756e04a3ff13b15f6471afdff30313e1c47c4c2ab0e209c78a0673a42c254b74cc0201";
    let block_signature = hex::decode(block_signature_hex)
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    
    // payload_hash from Java NRCS genesis block
    let payload_hash_hex = "8f58dc2f809613424e608586df83b42513056861a864dff3cd00d88baca681ce";
    let payload_hash = hex::decode(payload_hash_hex)
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    
    // generator_id from Java NRCS genesis block
    // Note: This value exceeds i64::MAX, so we parse as u64 first
    let generator_id: i64 = 18365787021584764528u64 as i64;
    
    // cumulative_difficulty: 0 as a single byte
    let cumulative_difficulty: Vec<u8> = vec![0u8; 1];

    // Insert genesis block
    // version = -1 表示创世区块
    // base_target = INITIAL_BASE_TARGET (153722867)
    sqlx::query(
        r#"
        INSERT INTO block (
            id, version, timestamp, previous_block_id, total_amount,
            total_fee, payload_length, previous_block_hash, cumulative_difficulty,
            base_target, next_block_id, height, generation_signature,
            block_signature, payload_hash, generator_id
        ) VALUES (
            ?, -1, ?, NULL, ?, 0, ?, NULL, ?, ?, NULL, ?, ?, ?, ?, ?
        )
        "#
    )
    .bind(block_id)                           // id
    .bind(timestamp as i32)                   // timestamp
    .bind(total_amount)                       // total_amount
    .bind(256i32)                             // payload_length (2 transactions * 128 bytes each)
    .bind(&cumulative_difficulty)             // cumulative_difficulty
    .bind(INITIAL_BASE_TARGET as i64)         // base_target
    .bind(height)                             // height
    .bind(&generation_signature)              // generation_signature
    .bind(&block_signature)                   // block_signature
    .bind(&payload_hash)                      // payload_hash
    .bind(generator_id)                       // generator_id
    .execute(pool)
    .await?;

    // Insert initial accounts
    for (account_id, balance) in accounts {
        sqlx::query(
            r#"
            INSERT INTO account (
                id, balance, unconfirmed_balance, forged_balance,
                active_lessee_id, has_control_phasing, height, latest
            ) VALUES (?, ?, ?, 0, NULL, 0, ?, 1)
            "#
        )
        .bind(account_id)
        .bind(balance)
        .bind(balance)
        .bind(height)
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO account_ledger (
                account_id, event_type, event_id, holding_type, holding_id,
                "CHANGE", balance, block_id, height, timestamp
            ) VALUES (?, 0, 1, 0, NULL, ?, ?, ?, ?, ?)
            "#
        )
        .bind(account_id)
        .bind(balance)
        .bind(balance)
        .bind(block_id)
        .bind(height)
        .bind(timestamp as i32)
        .execute(pool)
        .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::any::AnyPoolOptions;

    async fn setup_sqlite_pool() -> sqlx::Result<AnyPool> {
        let pool = AnyPoolOptions::new()
            .connect("sqlite::memory:")
            .await?;
        
        // 创建表结构
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS block (
                id INTEGER PRIMARY KEY,
                version INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                previous_block_id INTEGER,
                total_amount INTEGER NOT NULL,
                total_fee INTEGER NOT NULL,
                payload_length INTEGER NOT NULL,
                previous_block_hash BLOB,
                cumulative_difficulty BLOB NOT NULL,
                base_target INTEGER NOT NULL,
                next_block_id INTEGER,
                height INTEGER NOT NULL,
                generation_signature BLOB,
                block_signature BLOB,
                payload_hash BLOB,
                generator_id INTEGER NOT NULL
            )
        "#)
        .execute(&pool)
        .await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS account (
                id INTEGER PRIMARY KEY,
                balance INTEGER NOT NULL,
                unconfirmed_balance INTEGER NOT NULL,
                forged_balance INTEGER NOT NULL,
                active_lessee_id INTEGER,
                has_control_phasing INTEGER NOT NULL DEFAULT 0,
                height INTEGER NOT NULL,
                latest INTEGER NOT NULL DEFAULT 1
            )
        "#)
        .execute(&pool)
        .await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS account_ledger (
                account_id INTEGER NOT NULL,
                event_type INTEGER NOT NULL,
                event_id INTEGER NOT NULL,
                holding_type INTEGER NOT NULL,
                holding_id INTEGER,
                "CHANGE" INTEGER NOT NULL,
                balance INTEGER NOT NULL,
                block_id INTEGER NOT NULL,
                height INTEGER NOT NULL,
                timestamp INTEGER NOT NULL
            )
        "#)
        .execute(&pool)
        .await?;

        Ok(pool)
    }

    #[tokio::test]
    async fn test_genesis_creates_initial_state() -> sqlx::Result<()> {
        let pool = setup_sqlite_pool().await?;
        
        // 创建测试用的 genesis.json 文件
        let genesis_config = r#"{
            "genesis_time": "2024-1-1 00:00:00.000",
            "transactions": [
                {"recipient": "123456789", "amount": 1000000000},
                {"recipient": "987654321", "amount": 500000000}
            ]
        }"#;
        
        // 写入临时文件（使用绝对路径）
        // load_genesis_config 读取 config/genesis.json，所以需要创建 config 子目录
        let temp_dir = std::env::temp_dir().join("nrcs_test_config_1");
        let config_dir = temp_dir.join("config");
        std::fs::create_dir_all(&config_dir).ok();
        let config_path = config_dir.join("genesis.json");
        std::fs::write(&config_path, genesis_config)?;
        
        // 临时更改工作目录
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(&temp_dir).ok();
        
        let result = ensure_genesis(&pool).await;
        
        // 恢复工作目录
        if let Some(dir) = original_dir {
            std::env::set_current_dir(dir).ok();
        }
        
        result?;
        
        let block_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM block")
            .fetch_one(&pool)
            .await?;
        assert_eq!(block_count.0, 1);

        // 清理临时文件
        std::fs::remove_dir_all(&temp_dir).ok();
        
        Ok(())
    }

    #[tokio::test]
    async fn test_genesis_block_height_is_zero() -> sqlx::Result<()> {
        let pool = setup_sqlite_pool().await?;
        
        let genesis_config = r#"{
            "genesis_time": "2024-1-1 00:00:00.000",
            "transactions": []
        }"#;
        
        // 写入临时文件（使用绝对路径）
        let temp_dir = std::env::temp_dir().join("nrcs_test_config_2");
        let config_dir = temp_dir.join("config");
        std::fs::create_dir_all(&config_dir).ok();
        let config_path = config_dir.join("genesis.json");
        std::fs::write(&config_path, genesis_config)?;
        
        // 临时更改工作目录
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(&temp_dir).ok();
        
        let result = ensure_genesis(&pool).await;
        
        // 恢复工作目录
        if let Some(dir) = original_dir {
            std::env::set_current_dir(dir).ok();
        }
        
        result?;
        
        let height: (i32,) = sqlx::query_as("SELECT height FROM block LIMIT 1")
            .fetch_one(&pool)
            .await?;
        assert_eq!(height.0, 0, "Genesis block height should be 0");

        std::fs::remove_dir_all(&temp_dir).ok();
        
        Ok(())
    }

    #[tokio::test]
    async fn test_genesis_base_target() -> sqlx::Result<()> {
        let pool = setup_sqlite_pool().await?;
        
        let genesis_config = r#"{
            "genesis_time": "2024-1-1 00:00:00.000",
            "transactions": []
        }"#;
        
        // 写入临时文件（使用绝对路径）
        let temp_dir = std::env::temp_dir().join("nrcs_test_config_3");
        let config_dir = temp_dir.join("config");
        std::fs::create_dir_all(&config_dir).ok();
        let config_path = config_dir.join("genesis.json");
        std::fs::write(&config_path, genesis_config)?;
        
        // 临时更改工作目录
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(&temp_dir).ok();
        
        let result = ensure_genesis(&pool).await;
        
        // 恢复工作目录
        if let Some(dir) = original_dir {
            std::env::set_current_dir(dir).ok();
        }
        
        result?;
        
        let base_target: (i64,) = sqlx::query_as("SELECT base_target FROM block LIMIT 1")
            .fetch_one(&pool)
            .await?;
        assert_eq!(base_target.0, blockchain_types::constants::INITIAL_BASE_TARGET as i64, 
            "Genesis block base_target should be INITIAL_BASE_TARGET");

        std::fs::remove_dir_all(&temp_dir).ok();
        
        Ok(())
    }
}
