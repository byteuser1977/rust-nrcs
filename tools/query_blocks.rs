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
    
    let blocks: Vec<(i64, i32, i32, i64)> = sqlx::query_as(
        "SELECT id, height, timestamp, base_target FROM block ORDER BY height ASC LIMIT 10"
    )
    .fetch_all(&pool)
    .await?;
    
    println!("First 10 blocks:");
    for (id, height, timestamp, base_target) in blocks {
        println!("height={}, id={}, timestamp={}, base_target={}", height, id, timestamp, base_target);
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
    
    Ok(())
}
