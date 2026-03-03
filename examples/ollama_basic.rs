//! Ollama Basic Example
//!
//! Demonstrates how to use a local Ollama service for a basic chat conversation.
//!
//! Run: cargo run --example ollama_basic

use llm_connector::{
    LlmClient,
    types::{ChatRequest, Message},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Ollama Local Model Basic Chat Example\n");

    // Create Ollama client
    let client = LlmClient::ollama(llm_connector::endpoints::OLLAMA_LOCAL).unwrap();

    // Fetch available models
    println!("🔍 Fetching available models...");
    match client.models().await {
        Ok(models) => {
            if models.is_empty() {
                println!("❌ No available models found");
                println!("💡 Please download a model first, e.g.:");
                println!("   ollama pull llama2");
                println!("   ollama pull qwen:7b");
                return Ok(());
            }

            println!("✅ Found {} available models:", models.len());
            for (i, model) in models.iter().enumerate() {
                println!("  {}. {}", i + 1, model);
            }
            println!();
        }
        Err(e) => {
            println!("❌ Failed to fetch model list: {}", e);
            println!("💡 Please check:");
            println!("  1. Whether the Ollama service is running (ollama serve)");
            println!("  2. Whether the service URL is correct (default: http://localhost:11434)");
            return Ok(());
        }
    }

    // Use the first available model or a default model
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama2".to_string());

    // Build chat request
    let request = ChatRequest {
        model: model.clone(),
        messages: vec![Message::user(
            "Please briefly introduce yourself and what you can help me with.",
        )],
        max_tokens: Some(200),
        temperature: Some(0.7),
        ..Default::default()
    };

    println!("🚀 Sending request to Ollama...");
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
            println!("  1. Whether the Ollama service is running");
            println!("  2. Whether the model '{}' has been downloaded", model);
            println!("  3. Whether your network connection is working");
            println!();
            println!("🔧 Common commands:");
            println!("  ollama serve          # Start Ollama service");
            println!("  ollama pull {}   # Download model", model);
            println!("  ollama list           # List downloaded models");
        }
    }

    Ok(())
}
