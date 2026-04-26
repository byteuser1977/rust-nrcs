use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = "http://192.168.2.164:17974/nrcs";
    
    // 测试请求格式
    let request = json!({
        "requestType": "getNextBlocks",
        "protocol": 1,
        "blockId": "3488276486778630462",
        "blockIds": ["3488276486778630462", "3985281431710898053"]
    });
    
    println!("Sending request: {}", serde_json::to_string_pretty(&request)?);
    
    let resp = client.post(url).json(&request).send().await?;
    println!("Response status: {}", resp.status());
    
    let body = resp.text().await?;
    println!("Response body (first 500 chars): {}", &body.chars().take(500).collect::<String>());
    
    Ok(())
}
