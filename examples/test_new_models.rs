//! Test New Models: GLM-4.6 and Qwen3
//!
//! This test verifies the availability of the newest models

use llm_connector::{
    config::{Config, ProviderConfig},
    types::{ChatRequest, Message},
    Client,
};
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ProviderConfigWithProtocol {
    protocol: String,
    api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_ms: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ProtocolBasedConfig {
    providers: HashMap<String, ProviderConfigWithProtocol>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing New Models: GLM-4.6 and Qwen3");
    println!("==========================================\n");

    // Read API keys from keys.yaml
    println!("📖 Reading API keys from keys.yaml...");
    let yaml_content = std::fs::read_to_string("keys.yaml")?;

    let api_keys: HashMap<String, String> = if let Ok(config) = serde_yaml::from_str::<ProtocolBasedConfig>(&yaml_content) {
        println!("✅ Loaded {} providers from nested format", config.providers.len());
        config.providers.into_iter().map(|(k, v)| (k, v.api_key)).collect()
    } else {
        let keys: HashMap<String, String> = serde_yaml::from_str(&yaml_content)?;
        println!("✅ Loaded {} API keys from simple format", keys.len());
        keys
    };

    // Test GLM-4.6
    if let Some(zhipu_key) = api_keys.get("zhipu") {
        println!("🚀 Testing GLM-4.6 (Zhipu)...");
        let config = Config {
            openai: None,
            anthropic: None,
            deepseek: None,
            zhipu: Some(ProviderConfig::new(zhipu_key.clone())),
            aliyun: None,
            moonshot: None,
            volcengine: None,
            longcat: None,
        };

        let client = Client::with_config(config);

        let request = ChatRequest {
            model: "zhipu/glm-4.6".to_string(),
            messages: vec![Message::user("Hello! Please respond with 'GLM-4.6 is working!'")],
            max_tokens: Some(30),
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
                    println!("  ✅ GLM-4.6 successful!");
                    println!("  📄 Response: {}", choice.message.content.trim());
                    if let Some(usage) = &response.usage {
                        println!("  📊 Tokens: {} total", usage.total_tokens);
                    }
                } else {
                    println!("  ⚠️  No response content");
                }
            }
            Err(e) => {
                println!("  ❌ GLM-4.6 failed: {}", e);
            }
        }
    }

    println!();

    // Test Qwen3 models
    if let Some(aliyun_key) = api_keys.get("aliyun") {
        println!("🚀 Testing Qwen3 models (Aliyun)...");

        let qwen3_models = vec!["qwen3-turbo", "qwen3-plus", "qwen3-max"];

        for model in qwen3_models {
            let config = Config {
                openai: None,
                anthropic: None,
                deepseek: None,
                zhipu: None,
                aliyun: Some(ProviderConfig::new(aliyun_key.clone())),
                moonshot: None,
                volcengine: None,
                longcat: None,
            };

            let client = Client::with_config(config);

            let request = ChatRequest {
                model: format!("aliyun/{}", model),
                messages: vec![Message::user(format!("Hello! Please respond with '{} is working!'", model))],
                max_tokens: Some(30),
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
                        println!("  ✅ {} successful!", model);
                        println!("  📄 Response: {}", choice.message.content.trim());
                        if let Some(usage) = &response.usage {
                            println!("  📊 Tokens: {} total", usage.total_tokens);
                        }
                    } else {
                        println!("  ⚠️  {} - No response content", model);
                    }
                }
                Err(e) => {
                    println!("  ❌ {} failed: {}", model, e);
                }
            }
            println!();
        }
    }

    println!("🎉 New model testing completed!");
    Ok(())
}