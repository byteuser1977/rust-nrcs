//! Genesis block creation aligned with Java GenesisGenerator.

use serde_json::Value;
use std::fs;
use chrono::{NaiveDate, FixedOffset, TimeZone};
use blockchain_types::constants::{GENESIS_BLOCK_ID, INITIAL_BASE_TARGET, ONE_NRCS};
use crate::models::*;
use crate::repository::*;
use crate::RepositoryResult;
use tracing::info;

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
            recipient.parse::<i64>()
                .map_err(|e| crate::RepositoryError::Validation(format!("invalid recipient: {}", e)))?
        } else {
            match recipient.parse::<u64>() {
                Ok(unsigned_val) => unsigned_val as i64,
                Err(_) => {
                    recipient.parse::<i64>()
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
        info!("[Genesis] Genesis block already exists, skipping creation");
        return Ok(());
    }

    let (_unix_timestamp, accounts) = load_genesis_config()?;
    
    let one_nrcs_nqt: i64 = ONE_NRCS as i64;
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

    info!("[Genesis] Creating genesis block (id={}, height={})", block_id, height);
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
    }
    info!("[Genesis] ✓ Genesis block and {} accounts created successfully", _unix_timestamp);

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

    // 为 Genesis 区块的所有交易生成完整的账户账本记录
    //
    // 每笔交易生成 3 条 account_ledger 记录：
    // 1. Generator 手续费扣减记录 (event_type=50)
    // 2. Generator 金额扣减记录 (event_type=3)
    // 3. Recipient 金额接收记录 (event_type=3)
    //
    // 对于默认的 2 笔 genesis 交易，共生成 6 条账本记录
    let mut gen_cumulative_balance: i64 = 0;

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

        // 生成 Generator 的手续费扣除账本记录
        //
        // - event_type: 50 (TRANSACTION_FEE)
        // - holding_type: 1 (UNCONFIRMED_NRCS_BALANCE)
        // - change: 负数表示扣减
        // - balance: 累计余额（反映所有交易的累计扣减）
        gen_cumulative_balance -= fee;
        let gen_fee_ledger = AccountLedgerModel {
            db_id: 0,
            account_id: generator_id,
            event_type: 50,
            event_id: tx_id,
            holding_type: 1,
            holding_id: None,
            change: -fee,
            balance: gen_cumulative_balance,
            block_id,
            height,
            timestamp,
        };
        ledger_repo.insert(&gen_fee_ledger).await?;

        // 生成 Generator 的金额扣除账本记录
        //
        // - event_type: 3 (ORDINARY_PAYMENT)
        // - holding_type: 1 (UNCONFIRMED_NRCS_BALANCE)
        // - change: 负数表示扣减
        // - balance: 累计余额（包含本次 fee 和 amount 扣减）
        gen_cumulative_balance -= amount;
        let gen_amount_ledger = AccountLedgerModel {
            db_id: 0,
            account_id: generator_id,
            event_type: 3,
            event_id: tx_id,
            holding_type: 1,
            holding_id: None,
            change: -amount,
            balance: gen_cumulative_balance,
            block_id,
            height,
            timestamp,
        };
        ledger_repo.insert(&gen_amount_ledger).await?;

        // 生成 Recipient 的金额接收账本记录
        //
        // - event_type: 3 (ORDINARY_PAYMENT)
        // - holding_type: 1 (UNCONFIRMED_NRCS_BALANCE)
        // - change: 正数表示增加
        // - balance: 接收者收到的金额
        let recipient_ledger = AccountLedgerModel {
            db_id: 0,
            account_id: recipient_id,
            event_type: 3,
            event_id: tx_id,
            holding_type: 1,
            holding_id: None,
            change: amount,
            balance: amount,
            block_id,
            height,
            timestamp,
        };
        ledger_repo.insert(&recipient_ledger).await?;
    }

    let creator_account = AccountModel {
        db_id: 0,
        id: generator_id,
        balance: gen_cumulative_balance,
        unconfirmed_balance: gen_cumulative_balance,
        forged_balance: 0,
        active_lessee_id: None,
        has_control_phasing: false,
        height,
        latest: true,
    };
    account_repo.insert(&creator_account).await?;

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
                db_id INTEGER PRIMARY KEY AUTOINCREMENT,
                id INTEGER NOT NULL,
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
        .await
        .expect("Failed to create block table");

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS account (
                db_id INTEGER PRIMARY KEY AUTOINCREMENT,
                id INTEGER NOT NULL,
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
        .await
        .expect("Failed to create account table");

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS account_ledger (
                db_id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                event_type INTEGER NOT NULL,
                event_id INTEGER NOT NULL,
                holding_type INTEGER NOT NULL,
                holding_id INTEGER,
                "change" INTEGER NOT NULL,
                balance INTEGER NOT NULL,
                block_id INTEGER NOT NULL,
                height INTEGER NOT NULL,
                timestamp INTEGER NOT NULL
            )
        "#)
        .execute(&pool)
        .await
        .expect("Failed to create account_ledger table");

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS "transaction" (
                db_id INTEGER PRIMARY KEY AUTOINCREMENT,
                id INTEGER NOT NULL,
                deadline INTEGER NOT NULL,
                recipient_id INTEGER,
                amount INTEGER NOT NULL,
                fee INTEGER NOT NULL,
                full_hash BLOB NOT NULL,
                height INTEGER NOT NULL,
                block_id INTEGER NOT NULL,
                signature BLOB NOT NULL,
                timestamp INTEGER NOT NULL,
                type INTEGER NOT NULL,
                subtype INTEGER NOT NULL,
                sender_id INTEGER NOT NULL,
                block_timestamp INTEGER NOT NULL,
                referenced_transaction_full_hash BLOB,
                transaction_index INTEGER NOT NULL,
                phased INTEGER NOT NULL DEFAULT 0,
                attachment_bytes BLOB,
                version INTEGER NOT NULL DEFAULT 0,
                has_message INTEGER NOT NULL DEFAULT 0,
                has_encrypted_message INTEGER NOT NULL DEFAULT 0,
                has_public_key_announcement INTEGER NOT NULL DEFAULT 0,
                has_prunable_message INTEGER NOT NULL DEFAULT 0,
                has_prunable_attachment INTEGER NOT NULL DEFAULT 0,
                ec_block_height INTEGER,
                ec_block_id INTEGER,
                has_encrypttoself_message INTEGER NOT NULL DEFAULT 0,
                has_prunable_encrypted_message INTEGER NOT NULL DEFAULT 0
            )
        "#)
        .execute(&pool)
        .await
        .expect("Failed to create transaction table");

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS account_guaranteed_balance (
                db_id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                additions INTEGER NOT NULL,
                height INTEGER NOT NULL
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

    #[tokio::test]
    async fn test_genesis_account_ledger_records_complete() {
        let (block_repo, account_repo, tx_repo, ledger_repo, guaranteed_balance_repo) = setup_repos().await;

        let genesis_config = r#"{
            "genesis_time": "2024-1-1 00:00:00.000",
            "transactions": [
                {"recipient": "2794603741293765856", "amount": 999999999},
                {"recipient": "-891382425467438890", "amount": 1}
            ]
        }"#;

        let temp_dir = std::env::temp_dir().join("nrcs_test_config_ledger");
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

        let all_ledgers = ledger_repo.find_all(None, None)
            .await
            .expect("Failed to fetch all account_ledger records");

        assert_eq!(all_ledgers.len(), 6,
            "Expected 6 account_ledger records (2 txs × 3 records each), got {}", all_ledgers.len());

        let generator_id: i64 = 18365787021584764528u64 as i64;
        let tx1_id: i64 = -6309664432798542337;
        let tx2_id: i64 = 2830446832482296829;
        let recipient1_id: i64 = 2794603741293765856;
        let recipient2_id: i64 = -891382425467438890;
        let one_nrcs_nqt: i64 = 100_000_000;
        let fee: i64 = 100_000_000;
        let tx1_amount: i64 = 999999999 * one_nrcs_nqt;
        let tx2_amount: i64 = 1 * one_nrcs_nqt;

        let gen_records: Vec<&AccountLedgerModel> = all_ledgers.iter()
            .filter(|l| l.account_id == generator_id)
            .collect();
        assert_eq!(gen_records.len(), 4,
            "Generator should have 4 ledger records (fee + amount for each of 2 txs), got {}", gen_records.len());

        let gen_fee_tx1 = gen_records.iter()
            .find(|l| l.event_id == tx1_id && l.event_type == 50)
            .expect("Should find Generator TX1 FEE record (event_type=50)");
        assert_eq!(gen_fee_tx1.holding_type, 1,
            "Generator TX1 FEE holding_type should be 1 (UNCONFIRMED_NRCS_BALANCE)");
        assert_eq!(gen_fee_tx1.change, -fee,
            "Generator TX1 FEE change should be -{}", fee);
        assert_eq!(gen_fee_tx1.balance, -fee,
            "Generator TX1 FEE balance should be -{} (cumulative after first fee)", fee);

        let gen_amt_tx1 = gen_records.iter()
            .find(|l| l.event_id == tx1_id && l.event_type == 3)
            .expect("Should find Generator TX1 AMOUNT record (event_type=3)");
        assert_eq!(gen_amt_tx1.holding_type, 1,
            "Generator TX1 AMOUNT holding_type should be 1 (UNCONFIRMED_NRCS_BALANCE)");
        assert_eq!(gen_amt_tx1.change, -tx1_amount,
            "Generator TX1 AMOUNT change should be -{}", tx1_amount);
        assert_eq!(gen_amt_tx1.balance, -(fee + tx1_amount),
            "Generator TX1 AMOUNT balance should be -{} (cumulative after fee+amount)", fee + tx1_amount);

        let gen_fee_tx2 = gen_records.iter()
            .find(|l| l.event_id == tx2_id && l.event_type == 50)
            .expect("Should find Generator TX2 FEE record (event_type=50)");
        assert_eq!(gen_fee_tx2.holding_type, 1,
            "Generator TX2 FEE holding_type should be 1 (UNCONFIRMED_NRCS_BALANCE)");
        assert_eq!(gen_fee_tx2.change, -fee,
            "Generator TX2 FEE change should be -{}", fee);
        assert_eq!(gen_fee_tx2.balance, -(fee + tx1_amount + fee),
            "Generator TX2 FEE balance should be cumulative after TX1+TX2_fee");

        let gen_amt_tx2 = gen_records.iter()
            .find(|l| l.event_id == tx2_id && l.event_type == 3)
            .expect("Should find Generator TX2 AMOUNT record (event_type=3)");
        assert_eq!(gen_amt_tx2.holding_type, 1,
            "Generator TX2 AMOUNT holding_type should be 1 (UNCONFIRMED_NRCS_BALANCE)");
        assert_eq!(gen_amt_tx2.change, -tx2_amount,
            "Generator TX2 AMOUNT change should be -{}", tx2_amount);
        assert_eq!(gen_amt_tx2.balance, -(fee + tx1_amount + fee + tx2_amount),
            "Generator TX2 AMOUNT balance should be final cumulative balance");

        for record in &gen_records {
            assert!(record.change < 0,
                "Generator record change should be negative, got {} for event_id={} event_type={}",
                record.change, record.event_id, record.event_type);
        }

        let recip1_records: Vec<&AccountLedgerModel> = all_ledgers.iter()
            .filter(|l| l.account_id == recipient1_id)
            .collect();
        assert_eq!(recip1_records.len(), 1,
            "Recipient1 should have exactly 1 ledger record, got {}", recip1_records.len());
        let r1 = &recip1_records[0];
        assert_eq!(r1.event_id, tx1_id,
            "Recipient1 event_id should be TX1 id ({})", tx1_id);
        assert_eq!(r1.event_type, 3,
            "Recipient1 event_type should be 3 (ORDINARY_PAYMENT)");
        assert_eq!(r1.holding_type, 1,
            "Recipient1 holding_type should be 1 (UNCONFIRMED_NRCS_BALANCE)");
        assert_eq!(r1.change, tx1_amount,
            "Recipient1 change should be +{}", tx1_amount);
        assert_eq!(r1.balance, tx1_amount,
            "Recipient1 balance should be {}", tx1_amount);
        assert!(r1.change > 0,
            "Recipient1 change should be positive");

        let recip2_records: Vec<&AccountLedgerModel> = all_ledgers.iter()
            .filter(|l| l.account_id == recipient2_id)
            .collect();
        assert_eq!(recip2_records.len(), 1,
            "Recipient2 should have exactly 1 ledger record, got {}", recip2_records.len());
        let r2 = &recip2_records[0];
        assert_eq!(r2.event_id, tx2_id,
            "Recipient2 event_id should be TX2 id ({})", tx2_id);
        assert_eq!(r2.event_type, 3,
            "Recipient2 event_type should be 3 (ORDINARY_PAYMENT)");
        assert_eq!(r2.holding_type, 1,
            "Recipient2 holding_type should be 1 (UNCONFIRMED_NRCS_BALANCE)");
        assert_eq!(r2.change, tx2_amount,
            "Recipient2 change should be +{}", tx2_amount);
        assert_eq!(r2.balance, tx2_amount,
            "Recipient2 balance should be {}", tx2_amount);
        assert!(r2.change > 0,
            "Recipient2 change should be positive");

        for record in &all_ledgers {
            assert_eq!(record.holding_type, 1,
                "All records should have holding_type=1 (UNCONFIRMED_NRCS_BALANCE), got {} for account_id={}",
                record.holding_type, record.account_id);
            assert_eq!(record.block_id, blockchain_types::constants::GENESIS_BLOCK_ID as i64,
                "All records should reference genesis block_id");
            assert_eq!(record.height, 0,
                "All records should have height=0 (genesis block)");
        }

        let total_change: i64 = all_ledgers.iter().map(|l| l.change).sum();
        let expected_total = -2 * fee;
        assert_eq!(total_change, expected_total,
            "Sum of all changes should be {} (negative total fees, fees are consumed by system), got {}",
            expected_total, total_change);

        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
