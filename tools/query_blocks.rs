use sqlx::sqlite::SqlitePoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite:///mnt/d/workspace/git/rust-nrcs/nrcs.db?mode=ro")
        .await?;
    
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM block")
        .fetch_one(&pool)
        .await?;
    println!("Total blocks: {}", count.0);
    
    let blocks: Vec<(i64, i32, i32, i64, i64, i64, i32, Option<i64>, i64)> = sqlx::query_as(
        "SELECT id, height, timestamp, base_target, total_amount, total_fee, payload_length, previous_block_id, generator_id FROM block ORDER BY height ASC LIMIT 10"
    )
    .fetch_all(&pool)
    .await?;
    
    println!("First 10 blocks:");
    for (id, height, timestamp, base_target, total_amount, total_fee, payload_length, previous_block_id, generator_id) in blocks {
        println!("height={}, id={}, timestamp={}, base_target={}, total_amount={}, total_fee={}, payload_length={}, prev_block_id={}, generator_id={}", 
                 height, id, timestamp, base_target, total_amount, total_fee, payload_length, previous_block_id.unwrap_or(0), generator_id);
    }
    
    let latest: Vec<(i64, i32, i32, i64)> = sqlx::query_as(
        "SELECT id, height, timestamp, base_target FROM block ORDER BY height DESC LIMIT 5"
    )
    .fetch_all(&pool)
    .await?;
    
    println!("\nLatest 5 blocks:");
    for (id, height, timestamp, base_target) in latest {
        println!("height={}, id={}, timestamp={}, base_target={}", height, id, timestamp, base_target);
    }
    
    // 检查交易数据
    let tx_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM \"transaction\"")
        .fetch_one(&pool)
        .await?;
    println!("\nTotal transactions: {}", tx_count.0);
    
    if tx_count.0 > 0 {
        let txs: Vec<(i64, i32, i64, Option<i64>, i64, i64)> = sqlx::query_as(
            "SELECT id, type, sender_id, recipient_id, amount, fee FROM \"transaction\" ORDER BY height ASC LIMIT 10"
        )
        .fetch_all(&pool)
        .await?;
        
        println!("First 10 transactions:");
        for (id, type_id, sender_id, recipient_id, amount, fee) in txs {
            println!("id={}, type={}, sender={}, recipient={:?}, amount={}, fee={}", 
                     id, type_id, sender_id, recipient_id, amount, fee);
        }
    }
    
    // 检查账户数据
    let account_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account")
        .fetch_one(&pool)
        .await?;
    println!("\nTotal accounts: {}", account_count.0);
    
    Ok(())
}
