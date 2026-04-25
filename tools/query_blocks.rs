use sqlx::sqlite::SqlitePoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite://./nrcs_test.db?mode=ro")
        .await?;
    
    let blocks: Vec<(i64, i64, i64)> = sqlx::query_as(
        "SELECT id, height, generator_id FROM block ORDER BY height ASC LIMIT 10"
    )
    .fetch_all(&pool)
    .await?;
    
    println!("Rust NRCS 前 10 个区块:");
    for (id, height, generator_id) in blocks {
        println!("height={}, id={}, generator={}", height, id, generator_id);
    }
    
    Ok(())
}
