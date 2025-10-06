//! Test All Providers Using Protocol-based YAML Configuration
//!
//! This test reads API keys from keys.yaml with protocol types and tests all providers

use llm_connector::{
    config::{Config, ProviderConfig},
    types::{ChatRequest, Message},
    Client,
};
use std::collections::HashMap;

#[derive(Debug, serde::Deserialize)]
struct ProviderConfigWithProtocol {
    protocol: String,
    api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_ms: Option<u32>,
}

#[derive(Debug, serde::Deserialize)]
struct ProtocolBasedConfig {
    providers: HashMap<String, ProviderConfigWithProtocol>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing All Providers Using Protocol-based Configuration");
    println!("========================================================\n");

    // Read API keys from keys.yaml
    println!("📖 Reading API keys from keys.yaml...");
    let yaml_content = std::fs::read_to_string("keys.yaml")?;
    let api_keys: HashMap<String, String> = serde_yaml::from_str(&yaml_content)?;

    println!("✅ Loaded {} API keys from keys.yaml", api_keys.len());

    // Create protocol-based configuration
    println!("\n🔧 Creating protocol-based configuration...");
    let mut config = Config {
        openai: None,
        anthropic: None,
        deepseek: None,
        zhipu: None,
        aliyun: None,
        moonshot: None,
        volcengine: None,
        longcat: None,
    };

    let mut configured_providers = Vec::new();

    // Configure DeepSeek (OpenAI protocol)
    if let Some(key) = api_keys.get("deepseek") {
        config.deepseek = Some(ProviderConfig::new(key.clone()));
        configured_providers.push(("deepseek", "OpenAI", "deepseek-chat"));
        println!("  ✅ DeepSeek configured (OpenAI protocol)");
    }

    // Configure Zhipu (OpenAI protocol)
    if let Some(key) = api_keys.get("zhipu") {
        config.zhipu = Some(ProviderConfig::new(key.clone()));
        configured_providers.push(("zhipu", "OpenAI", "glm-4-flash"));
        println!("  ✅ Zhipu (GLM) configured (OpenAI protocol)");
    }

    // Configure Aliyun (Aliyun protocol)
    if let Some(key) = api_keys.get("aliyun") {
        config.aliyun = Some(ProviderConfig::new(key.clone()));
        configured_providers.push(("aliyun", "Aliyun", "qwen-turbo"));
        println!("  ✅ Aliyun (DashScope) configured (Aliyun protocol)");
    }

    // Configure Moonshot (OpenAI protocol)
    if let Some(key) = api_keys.get("moonshot") {
        config.moonshot = Some(ProviderConfig::new(key.clone()));
        configured_providers.push(("moonshot", "OpenAI", "moonshot-v1-8k"));
        println!("  ✅ Moonshot (Kimi) configured (OpenAI protocol)");
    }

    // Configure VolcEngine (OpenAI protocol)
    if let Some(key) = api_keys.get("volcengine") {
        config.volcengine = Some(ProviderConfig::new(key.clone()));
        configured_providers.push(("volcengine", "OpenAI", "ep-*")); // Need endpoint ID
        println!("  ✅ VolcEngine (Doubao) configured (OpenAI protocol)");
    }

    // Configure LongCat (OpenAI protocol)
    if let Some(key) = api_keys.get("longcat") {
        config.longcat = Some(ProviderConfig::new(key.clone()));
        configured_providers.push(("longcat", "OpenAI", "LongCat-Flash-Chat"));
        println!("  ✅ LongCat configured (OpenAI protocol)");
    }

    println!("\n🚀 Creating client with protocol-based config...");
    let client = Client::with_config(config);

    // List configured providers
    let provider_list = client.list_providers();
    println!("📋 Configured providers: {:?}", provider_list);

    if provider_list.is_empty() {
        println!("❌ No providers configured. Please check your keys.yaml file.");
        return Ok(());
    }

    println!("\n📊 Protocol Distribution:");
    for (provider, protocol, model) in &configured_providers {
        println!("  • {} - {} protocol (model: {})", provider, protocol, model);
    }

    println!("\n🧪 Testing all configured providers...\n");

    // Test each configured provider
    for (provider_name, protocol_name, model) in &configured_providers {
        test_provider_with_protocol(&client, provider_name, protocol_name, model).await;
    }

    println!("\n🎉 All provider tests completed!");

    // Show summary
    println!("\n📈 Test Summary:");
    println!("  Total providers tested: {}", configured_providers.len());

    let (openai_count, aliyun_count) = configured_providers.iter()
        .fold((0, 0), |(openai, aliyun), (_, protocol, _)| {
            match protocol.as_ref() {
                "OpenAI" => (openai + 1, aliyun),
                "Aliyun" => (openai, aliyun + 1),
                _ => (openai, aliyun),
            }
        });

    println!("  OpenAI protocol providers: {}", openai_count);
    println!("  Aliyun protocol providers: {}", aliyun_count);

    Ok(())
}

async fn test_provider_with_protocol(
    client: &Client,
    provider_name: &str,
    protocol_name: &str,
    model: &str
) {
    println!("🔍 Testing {} provider ({} protocol)", provider_name, protocol_name);
    println!("   ──────────────────────────────────────────────");

    // Skip VolcEngine if we don't have a real endpoint ID
    if provider_name == "volcengine" && model == "ep-*" {
        println!("   ⚠️  Skipping VolcEngine - requires endpoint ID from console");
        println!();
        return;
    }

    // Test the connection with a simple chat request
    println!("   📝 Testing chat connection using model '{}/{}'...", provider_name, model);

    let test_message = format!("Hello from {} using {} protocol!", provider_name, protocol_name);

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

                // Show protocol-specific features
                match protocol_name {
                    "OpenAI" => println!("   🔧 Features: Standard OpenAI API compatibility"),
                    "Aliyun" => println!("   🔧 Features: Custom DashScope protocol"),
                    _ => println!("   🔧 Features: Unknown protocol"),
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