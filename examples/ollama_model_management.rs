//! Ollama Model Management Example
//!
//! Demonstrates how to use the new Ollama model management features.

use llm_connector::{
    LlmClient, Provider,
    types::{ChatRequest, Message, Role},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦙 Ollama Model Management Example\n");

    // Create Ollama client
    let client = LlmClient::ollama(llm_connector::endpoints::OLLAMA_LOCAL)?;

    // Get Ollama-specific interface
    let ollama = match client.as_ollama() {
        Some(ollama) => ollama,
        None => {
            println!("❌ Failed to get Ollama-specific interface");
            return Ok(());
        }
    };

    // 1. List all available models
    println!("📋 List all available models:");
    match ollama.models().await {
        Ok(models) => {
            if models.is_empty() {
                println!("   No installed models found");
            } else {
                for (i, model) in models.iter().enumerate() {
                    println!("   {}. {}", i + 1, model);
                }
            }
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
            println!("   💡 Please ensure Ollama is running on localhost:11434");
        }
    }

    println!();

    // 2. Pull a new model (commented out to avoid real download)
    println!("📥 Pull a new model:");
    println!("   // The code below shows how to pull a new model");
    println!("   // ollama.pull_model(\"llama3.2:1b\").await?;");
    println!("   // println!(\"Model pulled successfully!\");");

    println!();

    // 4. Use models() method (generic interface)
    println!("🌐 Get model list via generic interface:");
    match client.models().await {
        Ok(models) => {
            if models.is_empty() {
                println!("   No models found");
            } else {
                for (i, model) in models.iter().enumerate() {
                    println!("   {}. {}", i + 1, model);
                }
            }
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
        }
    }

    println!();

    // 5. Simple chat test
    println!("💬 Chat test:");
    let chat_request = ChatRequest {
        model: "llama3.2".to_string(), // Use a model you actually have
        messages: vec![Message::text(
            Role::User,
            "Hello! Please answer in English.",
        )],
        ..Default::default()
    };

    match client.chat(&chat_request).await {
        Ok(response) => {
            println!("   Model reply: {}", response.content);
        }
        Err(e) => {
            println!("   ❌ Chat error: {}", e);
            println!(
                "   💡 Ensure the model '{}' is installed and available",
                chat_request.model
            );
        }
    }

    println!("\n✅ Example completed!");
    println!("\n💡 Notes:");
    println!("   - Use 'ollama list' to view installed models");
    println!("   - Use 'ollama pull <model>' to download a new model");
    println!("   - Use 'ollama rm <model>' to remove a model");

    Ok(())
}
