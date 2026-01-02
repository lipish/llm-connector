//! Google Gemini Basic Example
//!
//! Demonstrates how to use Google Gemini provider for basic chat.
//!
//! Run: cargo run --example google_basic

use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Google Gemini Basic Chat Example\n");

    // Get API key from environment variable
    let api_key = std::env::var("GEMINI_API_KEY")
        .unwrap_or_else(|_| {
            println!("❌ Please set environment variable GEMINI_API_KEY");
            println!("   export GEMINI_API_KEY=your-api-key");
            std::process::exit(1);
        });

    // Create Google client
    let client = LlmClient::google(&api_key)?;

    // Build chat request
    let request = ChatRequest {
        model: "gemini-3-pro-preview".to_string(),
        messages: vec![
            Message::user("Briefly explain the features of Rust programming language.")
        ],
        max_tokens: Some(1024),
        temperature: Some(0.7),
        ..Default::default()
    };

    println!("🚀 Sending request to Google Gemini...");
    println!("📝 Model: {}", request.model);
    println!("💬 Message: {}", request.messages[0].content_as_text());
    println!();

    // Send request
    match client.chat(&request).await {
        Ok(response) => {
            println!("✅ Response received:");
            println!("{}", response.content);
            println!();
            println!("📊 Token Usage:");
            println!("  Prompt: {} tokens", response.prompt_tokens());
            println!("  Completion: {} tokens", response.completion_tokens());
            println!("  Total: {} tokens", response.total_tokens());
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
        }
    }

    Ok(())
}
