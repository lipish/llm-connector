//! OpenAI Basic Example
//!
//! Demonstrates how to use the OpenAI protocol for a basic chat conversation.
//!
//! Run: cargo run --example openai_basic

use llm_connector::{
    LlmClient,
    types::{ChatRequest, Message},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 OpenAI Basic Chat Example\n");

    // Read API key from environment variables
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        println!("❌ Please set the OPENAI_API_KEY environment variable");
        println!("   export OPENAI_API_KEY=your-api-key");
        std::process::exit(1);
    });

    // Create OpenAI client
    // Use official endpoint constant
    let client = LlmClient::openai(&api_key, llm_connector::endpoints::OPENAI_API_V1)?;

    // Build chat request
    let request = ChatRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![Message::user(
            "Please briefly describe the characteristics of the Rust programming language.",
        )],
        max_tokens: Some(200),
        temperature: Some(0.7),
        ..Default::default()
    };

    println!("🚀 Sending request to OpenAI...");
    println!("📝 Model: {}", request.model);
    println!("💬 Message: {}", request.messages[0].content_as_text());
    println!();

    // Send request
    match client.chat(&request).await {
        Ok(response) => {
            println!("✅ Received response successfully:");
            println!("{}", response.content);
            println!();
            println!("📊 Token usage:");
            println!("  Input: {} tokens", response.prompt_tokens());
            println!("  Output: {} tokens", response.completion_tokens());
            println!("  Total: {} tokens", response.total_tokens());
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
            println!();
            println!("💡 Please check:");
            println!("  1. Whether OPENAI_API_KEY is set correctly");
            println!("  2. Whether your network connection is working");
            println!("  3. Whether the API key is valid");
        }
    }

    Ok(())
}
