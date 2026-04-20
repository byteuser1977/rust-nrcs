//! Genesis block creation aligned with Java GenesisGenerator.

use sqlx::PgPool;
use serde_json::Value;
use std::fs;
use chrono::{NaiveDate, FixedOffset, TimeZone};

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
    let mut accounts = Vec::new();
    for tx in transactions {
        let recipient = tx["recipient"].as_str()
            .ok_or_else(|| sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "recipient not a string",
            ))))?;
        let recipient_id: i64 = recipient.parse()
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
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
pub async fn ensure_genesis(pool: &PgPool) -> sqlx::Result<()> {
    // Check if any block exists
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM block")
        .fetch_one(pool)
        .await?;
    if count.0 > 0 {
        return Ok(());
    }

    let (timestamp, accounts) = load_genesis_config()?;
    let total_amount: i64 = accounts.iter().map(|(_, amt)| *amt).sum();
    let height = 1i32;
    let block_id = 1i64;

    // Insert genesis block
    sqlx::query!(
        r#"
        INSERT INTO block (
            id, version, timestamp, previous_block_id, total_amount,
            total_fee, payload_length, previous_block_hash, cumulative_difficulty,
            base_target, next_block_id, height, generation_signature,
            block_signature, payload_hash, generator_id
        ) VALUES (
            1, 1, $1, NULL, $2, 0, 0, NULL, '{}', 1000000, NULL, 1, NULL, NULL, NULL, 1
        )
        "#,
        timestamp as i32,
        total_amount
    )
    .execute(pool)
    .await?;

    // Insert initial accounts
    for (account_id, balance) in accounts {
        sqlx::query!(
            r#"
            INSERT INTO account (
                id, balance, unconfirmed_balance, forged_balance,
                active_lessee_id, has_control_phasing, height, latest
            ) VALUES ($1, $2, $2, 0, NULL, FALSE, $3, TRUE)
            "#,
            account_id,
            balance,
            height
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO account_ledger (
                account_id, event_type, event_id, holding_type, holding_id,
                "CHANGE", balance, block_id, height, timestamp
            ) VALUES ($1, 0, 1, 0, NULL, $2, $2, $3, $4, $5)
            "#,
            account_id,
            balance,
            block_id,
            height,
            timestamp as i32
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[sqlx::test]
    async fn test_genesis_creates_initial_state(pool: PgPool) -> sqlx::Result<()> {
        ensure_genesis(&pool).await?;
        let block_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM block")
            .fetch_one(&pool)
            .await?;
        assert_eq!(block_count.0, 1);
        Ok(())
    }
}
