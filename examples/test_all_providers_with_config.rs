//! Test All Providers Using Client::with_config and keys.yaml
//!
//! This test reads API keys from keys.yaml and uses Client::with_config to test all providers

use llm_connector::{
    config::{Config, ProviderConfig},
    types::{ChatRequest, Message},
    Client,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing All Providers Using Client::with_config");
    println!("===================================================\n");

    // Read API keys from keys.yaml
    println!("📖 Reading API keys from keys.yaml...");
    let yaml_content = std::fs::read_to_string("keys.yaml")?;
    let api_keys: HashMap<String, String> = serde_yaml::from_str(&yaml_content)?;

    println!("✅ Loaded {} API keys from keys.yaml", api_keys.len());

    // Create config with all available providers (without using Default::default())
    println!("\n🔧 Creating Config with all available providers...");

    let mut config = Config {
        openai: None,
        anthropic: None,
        deepseek: None,
        zhipu: None,
        aliyun: None,
        moonshot: None,
    };

    // Configure DeepSeek
    if let Some(key) = api_keys.get("deepseek") {
        config.deepseek = Some(ProviderConfig::new(key.clone()));
        println!("  ✅ DeepSeek configured");
    }

    // Configure Aliyun (DashScope)
    if let Some(key) = api_keys.get("aliyun") {
        config.aliyun = Some(ProviderConfig::new(key.clone()));
        println!("  ✅ Aliyun (DashScope) configured");
    }

    // Configure Zhipu (GLM)
    if let Some(key) = api_keys.get("zhipu") {
        config.zhipu = Some(ProviderConfig::new(key.clone()));
        println!("  ✅ Zhipu (GLM) configured");
    }

    // Configure Moonshot (Kimi)
    if let Some(key) = api_keys.get("moonshot") {
        config.moonshot = Some(ProviderConfig::new(key.clone()));
        println!("  ✅ Moonshot (Kimi) configured");
    }

    println!("\n🚀 Creating client with config...");
    let client = Client::with_config(config);

    // List configured providers
    let configured_providers = client.list_providers();
    println!("📋 Configured providers: {:?}", configured_providers);

    if configured_providers.is_empty() {
        println!("❌ No providers configured. Please check your keys.yaml file.");
        return Ok(());
    }

    println!("\n🧪 Testing all configured providers...\n");

    // Test each configured provider
    for provider_name in &configured_providers {
        test_provider(&client, provider_name).await;
    }

    println!("\n🎉 All provider tests completed!");
    Ok(())
}

async fn test_provider(client: &Client, provider_name: &str) {
    println!("1️⃣  Testing {} provider", provider_name);
    println!("   ─────────────────────────────────");

    // Determine model and test message based on provider
    let (model, test_message) = match provider_name {
        "deepseek" => ("deepseek-chat", "Hello from DeepSeek!"),
        "aliyun" => ("qwen-turbo", "Hello from Aliyun (DashScope)!"),
        "zhipu" => ("glm-4-flash", "Hello from Zhipu (GLM)!"),
        "moonshot" => ("moonshot-v1-8k", "Hello from Moonshot (Kimi)!"),
        _ => ("default-model", "Hello from unknown provider!"),
    };

    // Test the connection with a simple chat request
    println!("   📝 Testing chat connection using model '{}/{}'...", provider_name, model);

    let request = ChatRequest {
        model: format!("{}/{}", provider_name, model),
        messages: vec![Message::user(format!("Please respond with '{}'", test_message))],
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

    match client.chat(request).await {
        Ok(response) => {
            if let Some(choice) = response.choices.first() {
                println!("   ✅ Chat successful!");
                println!("   📄 Response: {}", choice.message.content.trim());

                if let Some(usage) = &response.usage {
                    println!("   📊 Token usage:");
                    println!("      - Prompt tokens: {}", usage.prompt_tokens);
                    println!("      - Completion tokens: {}", usage.completion_tokens);
                    println!("      - Total tokens: {}", usage.total_tokens);
                }
            } else {
                println!("   ⚠️  No response content received");
            }
        }
        Err(e) => {
            println!("   ❌ Chat failed: {}", e);
        }
    }

    println!();
}