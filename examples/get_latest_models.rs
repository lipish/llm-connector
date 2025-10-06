//! Get Latest Models from All Providers
//!
//! This program queries each provider to get their latest available models
//! and updates the keys.yaml file with current model information.

use llm_connector::{
    config::ProviderConfig,
    Provider,
    protocols::{
        core::{GenericProvider},
        openai::{deepseek, zhipu, moonshot, longcat},
        aliyun::aliyun,
    },
    types::{ChatRequest, Message},
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
    println!("🔍 Getting Latest Models from All Providers");
    println!("=============================================\n");

    // Read API keys from keys.yaml
    println!("📖 Reading API keys from keys.yaml...");
    let yaml_content = std::fs::read_to_string("keys.yaml")?;

    // Try to parse as nested format first
    let api_keys: HashMap<String, String> = if let Ok(config) = serde_yaml::from_str::<ProtocolBasedConfig>(&yaml_content) {
        println!("✅ Loaded {} providers from nested format", config.providers.len());
        config.providers.into_iter().map(|(k, v)| (k, v.api_key)).collect()
    } else {
        // Fallback to simple format
        let keys: HashMap<String, String> = serde_yaml::from_str(&yaml_content)?;
        println!("✅ Loaded {} API keys from simple format", keys.len());
        keys
    };

    println!("✅ Loaded {} API keys\n", api_keys.len());

    // Provider information with latest models
    let mut provider_info = HashMap::new();

    // Test DeepSeek models
    if let Some(key) = api_keys.get("deepseek") {
        println!("🚀 Testing DeepSeek models...");
        let config = ProviderConfig::new(key.clone());
        let provider = GenericProvider::new(config, deepseek())?;

        let models_to_test = vec![
            "deepseek-chat",
            "deepseek-coder",
            "deepseek-reasoner",
            "deepseek-chat-v3",
        ];

        for model in models_to_test {
            if test_model(&provider, model).await {
                provider_info.entry("deepseek".to_string()).or_insert_with(Vec::new).push(model.to_string());
                println!("  ✅ {} - Available", model);
            } else {
                println!("  ❌ {} - Not available", model);
            }
        }
    }

    // Test Zhipu models
    if let Some(key) = api_keys.get("zhipu") {
        println!("\n🚀 Testing Zhipu (GLM) models...");
        let config = ProviderConfig::new(key.clone());
        let provider = GenericProvider::new(config, zhipu())?;

        let models_to_test = vec![
            "glm-4",
            "glm-4-plus",
            "glm-4-flash",
            "glm-4-flashx",
            "glm-4-long",
            "glm-4-air",
            "glm-4-airx",
        ];

        for model in models_to_test {
            if test_model(&provider, model).await {
                provider_info.entry("zhipu".to_string()).or_insert_with(Vec::new).push(model.to_string());
                println!("  ✅ {} - Available", model);
            } else {
                println!("  ❌ {} - Not available", model);
            }
        }
    }

    // Test Moonshot models
    if let Some(key) = api_keys.get("moonshot") {
        println!("\n🚀 Testing Moonshot models...");
        let config = ProviderConfig::new(key.clone());
        let provider = GenericProvider::new(config, moonshot())?;

        let models_to_test = vec![
            "moonshot-v1-8k",
            "moonshot-v1-32k",
            "moonshot-v1-128k",
            "moonshot-v1-8k-preview",
            "moonshot-v1-32k-preview",
        ];

        for model in models_to_test {
            if test_model(&provider, model).await {
                provider_info.entry("moonshot".to_string()).or_insert_with(Vec::new).push(model.to_string());
                println!("  ✅ {} - Available", model);
            } else {
                println!("  ❌ {} - Not available", model);
            }
        }
    }

    // Test LongCat models
    if let Some(key) = api_keys.get("longcat") {
        println!("\n🚀 Testing LongCat models...");
        let config = ProviderConfig::new(key.clone());
        let provider = GenericProvider::new(config, longcat())?;

        let models_to_test = vec![
            "LongCat-Flash-Chat",
            "LongCat-Flash-Thinking",
            "LongCat-Pro-Chat",
            "LongCat-Pro-Thinking",
        ];

        for model in models_to_test {
            if test_model(&provider, model).await {
                provider_info.entry("longcat".to_string()).or_insert_with(Vec::new).push(model.to_string());
                println!("  ✅ {} - Available", model);
            } else {
                println!("  ❌ {} - Not available", model);
            }
        }
    }

    // Test Aliyun models
    if let Some(key) = api_keys.get("aliyun") {
        println!("\n🚀 Testing Aliyun (DashScope) models...");
        let config = ProviderConfig::new(key.clone());
        let provider = GenericProvider::new(config, aliyun())?;

        let models_to_test = vec![
            "qwen-turbo",
            "qwen-plus",
            "qwen-max",
            "qwen-long",
            "qwen2-72b-instruct",
            "qwen2-57b-a14b-instruct",
            "qwen2-7b-instruct",
        ];

        for model in models_to_test {
            if test_model(&provider, model).await {
                provider_info.entry("aliyun".to_string()).or_insert_with(Vec::new).push(model.to_string());
                println!("  ✅ {} - Available", model);
            } else {
                println!("  ❌ {} - Not available", model);
            }
        }
    }

    // Generate updated keys.yaml
    println!("\n📝 Generating updated keys.yaml...");
    generate_updated_keys_yaml(&api_keys, &provider_info)?;

    println!("\n🎉 Latest model information saved to keys.yaml!");

    Ok(())
}

async fn test_model(provider: &GenericProvider<impl llm_connector::protocols::core::ProviderAdapter>, model: &str) -> bool {
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![Message::user("test")],
        max_tokens: Some(5),
        temperature: Some(0.1),
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
        Ok(_) => true,
        Err(_) => false,
    }
}

fn generate_updated_keys_yaml(
    api_keys: &HashMap<String, String>,
    provider_info: &HashMap<String, Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let yaml_content = format!(
        r#"# API Keys with Protocol Types and Latest Models
# This file demonstrates the recommended format for configuring LLM providers
# Each provider specifies its protocol type, API key, and latest available models

# OpenAI Protocol Providers
# All providers using the OpenAI-compatible API format
providers:
  deepseek:
    protocol: "openai"
    api_key: "{}"
    base_url: "https://api.deepseek.com/v1"
    models: {:?}

  zhipu:
    protocol: "openai"
    api_key: "{}"
    base_url: "https://open.bigmodel.cn/api/paas/v4"
    models: {:?}

  moonshot:
    protocol: "openai"
    api_key: "{}"
    base_url: "https://api.moonshot.cn/v1"
    models: {:?}

  longcat:
    protocol: "openai"
    api_key: "{}"
    base_url: "https://api.longcat.chat/openai"
    models: {:?}

  volcengine:
    protocol: "openai"
    api_key: "{}"
    base_url: "https://ark.cn-beijing.volces.com/api/v3"
    note: "Requires endpoint ID from console (format: ep-xxxxxxxx)"
    models: ["ep-*"] # Endpoint IDs from console

  # Aliyun Protocol Providers
  # Custom protocol used by Aliyun DashScope
  aliyun:
    protocol: "aliyun"
    api_key: "{}"
    base_url: "https://dashscope.aliyuncs.com/api/v1"
    models: {:?}

# Anthropic Protocol Providers
# Custom protocol used by Anthropic Claude
# anthropic:
#   protocol: "anthropic"
#   api_key: "sk-ant-xxxxx"
#   base_url: "https://api.anthropic.com"
#   models: ["claude-3-5-sonnet", "claude-3-opus", "claude-3-haiku"]
"#,
        api_keys.get("deepseek").unwrap_or(&"YOUR_DEEPSEEK_KEY".to_string()),
        provider_info.get("deepseek").unwrap_or(&vec!["deepseek-chat".to_string()]),

        api_keys.get("zhipu").unwrap_or(&"YOUR_ZHIPU_KEY".to_string()),
        provider_info.get("zhipu").unwrap_or(&vec!["glm-4-flash".to_string()]),

        api_keys.get("moonshot").unwrap_or(&"YOUR_MOONSHOT_KEY".to_string()),
        provider_info.get("moonshot").unwrap_or(&vec!["moonshot-v1-8k".to_string()]),

        api_keys.get("longcat").unwrap_or(&"YOUR_LONGCAT_KEY".to_string()),
        provider_info.get("longcat").unwrap_or(&vec!["LongCat-Flash-Chat".to_string()]),

        api_keys.get("volcengine").unwrap_or(&"YOUR_VOLCENGINE_KEY".to_string()),

        api_keys.get("aliyun").unwrap_or(&"YOUR_ALIYUN_KEY".to_string()),
        provider_info.get("aliyun").unwrap_or(&vec!["qwen-turbo".to_string()])
    );

    std::fs::write("keys.yaml", yaml_content)?;
    Ok(())
}