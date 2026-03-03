//! MiniMax Example (V2)
//!
//! Run: cargo run --example minimax

use dotenvy::dotenv;
use llm_connector::{
    LlmClient,
    types::{ChatRequest, Message},
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("🤖 MiniMax Example\n");

    let api_key = env::var("MINIMAX_API_KEY").expect("MINIMAX_API_KEY not set");
    let base_url = env::var("MINIMAX_BASE_URL").unwrap_or_else(|_| "https://api.minimax.io/v1".to_string());

    let client = LlmClient::openai(&api_key, &base_url)?;

    println!("--- 1. Basic Chat ---");
    let request = ChatRequest::new("abab6.5s-chat")
        .add_message(Message::user("Hello MiniMax, describe your strengths."));
    
    let response = client.chat(&request).await?;
    println!("Response: {}\n", response.content);

    Ok(())
}
