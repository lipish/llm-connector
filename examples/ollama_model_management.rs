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

    // Create Ollama client (default local address)
    let client = LlmClient::ollama()?;

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

    // 2. Get model details
    println!("🔍 Get model details:");
    let model_name = "llama3.2"; // Change this to a model you actually have
    match ollama.show_model(model_name).await {
        Ok(model_info) => {
            println!("   Model details:");
            println!("     Format: {}", model_info.details.format);
            println!("     Family: {}", model_info.details.family);
            println!("     Parameter size: {}", model_info.details.parameter_size);
            println!(
                "     Quantization level: {}",
                model_info.details.quantization_level
            );
            if let Some(families) = &model_info.details.families {
                println!("     Supported families: {:?}", families);
            }
            println!("     Template length: {} chars", model_info.template.len());
            println!(
                "     Parameters length: {} chars",
                model_info.parameters.len()
            );
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
            println!("   💡 Ensure the model '{}' is installed", model_name);
        }
    }

    println!();

    // 3. Pull a new model (commented out to avoid real download)
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
