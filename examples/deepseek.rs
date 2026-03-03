//! DeepSeek Example (V2)
//!
//! Run: cargo run --example deepseek

use dotenvy::dotenv;
#[allow(unused_imports)]
use llm_providers;
use llm_connector::{
    LlmClient,
    types::{ChatRequest, Message},
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("🤖 DeepSeek Example\n");

    let api_key = env::var("DEEPSEEK_API_KEY").expect("DEEPSEEK_API_KEY not set");
    let base_url = env::var("DEEPSEEK_BASE_URL").unwrap_or_else(|_| "https://api.deepseek.com".to_string());

    let client = LlmClient::openai(&api_key, &base_url)?;

    println!("--- 1. Basic Chat ---");
    let request = ChatRequest::new("deepseek-chat")
        .add_message(Message::user("Hello DeepSeek, what's new today?"));
    
    let response = client.chat(&request).await?;
    println!("Response: {}\n", response.content);

    Ok(())
}
