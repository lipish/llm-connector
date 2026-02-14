//! Xiaomi MiMo basic example
//!
//! This example demonstrates the basic usage of Xiaomi MiMo API.
//!
//! Run with:
//! ```bash
//! XIAOMI_API_KEY=your-key cargo run --example xiaomi_basic
//! ```

use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("XIAOMI_API_KEY")
        .expect("XIAOMI_API_KEY environment variable not set");
    
    let client = LlmClient::xiaomi(&api_key)?;
    
    println!("Provider: {}", client.provider_name());
    
    let request = ChatRequest {
        model: "mimo-v2-flash".to_string(),
        messages: vec![Message::user("你好，请用一句话介绍你自己")],
        ..Default::default()
    };
    
    println!("Sending request to Xiaomi MiMo...");
    let response = client.chat(&request).await?;
    println!("Response: {}", response.content);
    
    Ok(())
}

