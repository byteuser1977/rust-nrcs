//! Query Blocks Tool
//!
//! 用于查询区块数据的命令行工具

use sqlx::sqlite::SqlitePool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: query_blocks <database_path> [height]");
        std::process::exit(1);
    }
    
    let db_path = &args[1];
    let db_url = format!("sqlite:{}?mode=ro", db_path);
    
    let pool = SqlitePool::connect(&db_url).await?;
    
    let height: i32 = if args.len() > 2 {
        args[2].parse()?
    } else {
        -1
    };
    
    if height >= 0 {
        let row: (i64, i32, i64, i64) = sqlx::query_as(
            "SELECT id, height, timestamp, generator_id FROM block WHERE height = ?"
        )
        .bind(height)
        .fetch_one(&pool)
        .await?;
        
        println!("Block at height {}: id={}, timestamp={}, generator={}", 
                 row.1, row.0, row.2, row.3);
    } else {
        let rows: Vec<(i64, i32, i64, i64)> = sqlx::query_as(
            "SELECT id, height, timestamp, generator_id FROM block ORDER BY height DESC LIMIT 10"
        )
        .fetch_all(&pool)
        .await?;
        
        println!("Latest 10 blocks:");
        for row in rows {
            println!("  height {}: id={}, timestamp={}, generator={}", 
                     row.1, row.0, row.2, row.3);
        }
    }
    
    Ok(())
}
