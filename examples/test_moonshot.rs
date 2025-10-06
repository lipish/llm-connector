//! Test Moonshot Connection
//!
//! This test reads the Moonshot API key from keys.yaml and tests the connection.

use llm_connector::{
    config::ProviderConfig,
    protocols::{
        core::{GenericProvider},
        openai::moonshot,
    },
    types::{ChatRequest, Message},
    Provider,
};
use serde::Deserialize;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing Moonshot Connection");
    println!("=============================\n");

    // Read API keys from keys.yaml
    println!("📖 Reading API keys from keys.yaml...");
    let yaml_content = std::fs::read_to_string("keys.yaml")?;
    let api_keys: HashMap<String, String> = serde_yaml::from_str(&yaml_content)?;

    let moonshot_key = match api_keys.get("moonshot") {
        Some(key) => key,
        None => {
            println!("❌ Moonshot API key not found in keys.yaml");
            return Ok(());
        }
    };

    println!("✅ Moonshot API key loaded successfully");

    // Create Moonshot provider
    println!("\n🔧 Creating Moonshot provider...");
    let config = ProviderConfig::new(moonshot_key.clone());
    let protocol = moonshot();

    let provider = match GenericProvider::new(config, protocol) {
        Ok(p) => {
            println!("✅ Provider created successfully");
            p
        }
        Err(e) => {
            println!("❌ Failed to create provider: {}", e);
            return Ok(());
        }
    };

    // Test the connection with a simple chat request
    println!("\n📝 Testing chat connection...");
    let request = ChatRequest {
        model: "moonshot-v1-8k".to_string(),
        messages: vec![Message::user("Hello! Please respond with 'Moonshot connection successful!'")],
        max_tokens: Some(50),
        temperature: Some(0.7),
        stream: None,
        top_p: None,
        stop: None,
        tools: None,
        tool_choice: None,
        frequency_penalty: None,
        logit_bias: None,
        presence_penalty: None,
        response_format: None,
        seed: None,
        user: None,
    };

    match provider.chat(&request).await {
        Ok(response) => {
            if let Some(choice) = response.choices.first() {
                println!("✅ Chat successful!");
                println!("📄 Response: {}", choice.message.content.trim());

                if let Some(usage) = &response.usage {
                    println!("📊 Token usage:");
                    println!("   - Prompt tokens: {}", usage.prompt_tokens);
                    println!("   - Completion tokens: {}", usage.completion_tokens);
                    println!("   - Total tokens: {}", usage.total_tokens);
                }

                println!("\n🎉 Moonshot connection test completed successfully!");
            } else {
                println!("⚠️  No response content received");
            }
        }
        Err(e) => {
            println!("❌ Chat failed: {}", e);
            return Ok(());
        }
    }

    Ok(())
}