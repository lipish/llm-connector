//! Google Gemini Example (V2)
//!
//! Demonstrates basic chat and streaming using Google Gemini.
//!
//! Run: cargo run --example google

use dotenvy::dotenv;
use llm_connector::{
    LlmClient,
    types::{ChatRequest, Message},
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("🤖 Google Gemini Comprehensive Example\n");

    let api_key = env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY not set");
    let base_url = env::var("GOOGLE_BASE_URL").unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta".to_string());

    let client = LlmClient::google(&api_key, &base_url)?;

    println!("--- 1. Basic Chat ---");
    let request = ChatRequest::new("gemini-1.5-flash")
        .add_message(Message::user("What's interesting about the Big Bang?"));
    
    let response = client.chat(&request).await?;
    println!("Response: {}\n", response.content);

    #[cfg(feature = "streaming")]
    {
        println!("--- 2. Streaming Chat ---");
        let request = ChatRequest::new("gemini-1.5-flash")
            .add_message(Message::user("Describe the aurora borealis."))
            .with_stream(true);
        
        let mut stream = client.chat_stream(&request).await?;
        print!("Streaming: ");
        while let Some(chunk) = futures_util::StreamExt::next(&mut stream).await {
            let chunk = chunk?;
            print!("{}", chunk.content);
            std::io::Write::flush(&mut std::io::stdout())?;
        }
        println!("\n");
    }

    Ok(())
}
