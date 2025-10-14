//! Test with keys.yaml configuration
//!
//! This example tests the OpenAI protocol with various providers from keys.yaml
//!
//! Run with: cargo run --example test_with_keys --features yaml

use llm_connector::{LlmClient, ChatRequest, Message};
use llm_connector::types::Role;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
struct ProviderConfig {
    protocol: String,
    api_key: String,
    base_url: String,
    models: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct KeysConfig {
    providers: std::collections::HashMap<String, ProviderConfig>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔑 Testing with keys.yaml configuration\n");

    // Load keys.yaml
    let keys_content = fs::read_to_string("keys.yaml")?;

    // Parse YAML using serde_yaml (available via 'yaml' feature)
    let config: KeysConfig = serde_yaml::from_str(&keys_content)
        .map_err(|e| format!("Failed to parse keys.yaml: {}", e))?;

    println!("📋 Found {} providers in keys.yaml\n", config.providers.len());

    // Test OpenAI protocol providers only
    let openai_providers: Vec<_> = config
        .providers
        .iter()
        .filter(|(_, cfg)| cfg.protocol == "openai")
        .collect();

    println!("🧪 Testing {} OpenAI protocol providers:\n", openai_providers.len());

    for (name, provider_config) in openai_providers {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Provider: {}", name);
        println!("Base URL: {}", provider_config.base_url);
        println!("Models: {:?}", provider_config.models);
        
        if let Some(note) = &provider_config.note {
            println!("Note: {}", note);
        }

        // Create client
        let client = LlmClient::openai_compatible(
            &provider_config.api_key,
            &provider_config.base_url,
        );

        println!("Protocol: {}", client.protocol_name());

        // Test with first available model
        if let Some(model) = provider_config.models.first() {
            println!("\n🚀 Testing with model: {}", model);

            let request = ChatRequest {
                model: model.clone(),
                messages: vec![Message {
                    role: Role::User,
                    content: "Say 'Hello' in one word only.".to_string(),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                }],
                temperature: Some(0.7),
                max_tokens: Some(10),
                ..Default::default()
            };

            match client.chat(&request).await {
                Ok(response) => {
                    println!("✅ Success!");
                    println!("Response ID: {}", response.id);
                    println!("Model: {}", response.model);
                    if let Some(choice) = response.choices.first() {
                        println!("Content: {}", choice.message.content);
                        println!("Finish reason: {:?}", choice.finish_reason);
                    }
                    if let Some(usage) = &response.usage {
                        println!("Usage: {} prompt + {} completion = {} total tokens",
                            usage.prompt_tokens,
                            usage.completion_tokens,
                            usage.total_tokens
                        );
                    }
                }
                Err(e) => {
                    println!("❌ Error: {}", e);
                }
            }
        }

        println!();
    }

    // Test Aliyun protocol if available
    if let Some((name, provider_config)) = config.providers.iter().find(|(_, cfg)| cfg.protocol == "aliyun") {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Provider: {} (Aliyun Protocol)", name);
        println!("Base URL: {}", provider_config.base_url);
        println!("Models: {:?}", provider_config.models);

        let client = LlmClient::aliyun(&provider_config.api_key);
        println!("Protocol: {}", client.protocol_name());

        if let Some(model) = provider_config.models.first() {
            println!("\n🚀 Testing with model: {}", model);

            let request = ChatRequest {
                model: model.clone(),
                messages: vec![Message {
                    role: Role::User,
                    content: "Say 'Hello' in one word only.".to_string(),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                }],
                temperature: Some(0.7),
                max_tokens: Some(10),
                ..Default::default()
            };

            match client.chat(&request).await {
                Ok(response) => {
                    println!("✅ Success!");
                    println!("Response ID: {}", response.id);
                    println!("Model: {}", response.model);
                    if let Some(choice) = response.choices.first() {
                        println!("Content: {}", choice.message.content);
                        println!("Finish reason: {:?}", choice.finish_reason);
                    }
                    if let Some(usage) = &response.usage {
                        println!("Usage: {} prompt + {} completion = {} total tokens",
                            usage.prompt_tokens,
                            usage.completion_tokens,
                            usage.total_tokens
                        );
                    }
                }
                Err(e) => {
                    println!("❌ Error: {}", e);
                }
            }
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✨ Testing complete!");

    Ok(())
}

