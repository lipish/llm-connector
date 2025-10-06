//! Test online model fetching
//!
//! This example demonstrates the new fetch_models() functionality
//! that retrieves available models from the API.

use llm_connector::LlmClient;
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
    println!("🔍 Testing Online Model Fetching\n");

    // Load keys.yaml
    let keys_content = fs::read_to_string("keys.yaml")?;
    let config: KeysConfig = serde_yaml::from_str(&keys_content)
        .map_err(|e| format!("Failed to parse keys.yaml: {}", e))?;

    println!("📋 Testing fetch_models() for OpenAI protocol providers\n");

    // Test OpenAI protocol providers
    let openai_providers: Vec<_> = config
        .providers
        .iter()
        .filter(|(_, cfg)| cfg.protocol == "openai")
        .collect();

    for (name, provider_config) in openai_providers {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Provider: {}", name);
        println!("Base URL: {}", provider_config.base_url);

        // Create client
        let client = LlmClient::openai_compatible(
            &provider_config.api_key,
            &provider_config.base_url,
        );

        // Test online fetch_models
        println!("\n🌐 fetch_models():");
        match client.fetch_models().await {
            Ok(models) => {
                println!("   ✅ Success! Found {} models", models.len());
                if models.len() <= 10 {
                    println!("   Models: {:?}", models);
                } else {
                    println!("   First 10 models: {:?}", &models[..10]);
                    println!("   ... and {} more", models.len() - 10);
                }
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
            }
        }

        println!();
    }

    // Test Aliyun protocol
    if let Some((name, provider_config)) = config.providers.iter().find(|(_, cfg)| cfg.protocol == "aliyun") {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Provider: {} (Aliyun Protocol)", name);
        println!("Base URL: {}", provider_config.base_url);

        let client = LlmClient::aliyun(&provider_config.api_key);

        println!("\n🌐 fetch_models():");
        match client.fetch_models().await {
            Ok(models) => {
                println!("   ✅ Success! Found {} models", models.len());
                println!("   Models: {:?}", models);
            }
            Err(e) => {
                println!("   ℹ️  Error (expected for Aliyun): {}", e);
            }
        }

        println!();
    }

    // Test Anthropic protocol
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Provider: Anthropic (test)");
    let anthropic_client = LlmClient::anthropic("test-key");

    println!("\n🌐 fetch_models():");
    match anthropic_client.fetch_models().await {
        Ok(models) => {
            println!("   ✅ Success! Found {} models", models.len());
            println!("   Models: {:?}", models);
        }
        Err(e) => {
            println!("   ℹ️  Error (expected for Anthropic): {}", e);
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✨ Testing complete!");
    println!("\n📝 Summary:");
    println!("   - fetch_models() retrieves models online from the API");
    println!("   - OpenAI protocol supports model listing via /v1/models endpoint");
    println!("   - Other protocols (Anthropic, Aliyun, Ollama) do not support model listing");

    Ok(())
}

