//! Test API keys from keys.yaml file
//!
//! This example loads keys from keys.yaml and tests each one
//! to verify they are valid and working.
//!
//! Run with: cargo run --example test_keys_yaml

use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
struct ProviderConfig {
    protocol: String,
    api_key: String,
    base_url: String,
    #[serde(default)]
    models: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct KeysConfig {
    providers: std::collections::HashMap<String, ProviderConfig>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing API Keys from keys.yaml\n");
    println!("{}", "=".repeat(80));
    println!();

    // Load keys.yaml
    let keys_content = match fs::read_to_string("keys.yaml") {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Failed to read keys.yaml: {}", e);
            eprintln!();
            eprintln!("💡 Make sure keys.yaml exists in the current directory");
            eprintln!("   Current directory: {:?}", std::env::current_dir()?);
            return Err(e.into());
        }
    };

    let config: KeysConfig = match serde_yaml::from_str(&keys_content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("❌ Failed to parse keys.yaml: {}", e);
            eprintln!();
            eprintln!("💡 Check your YAML syntax");
            return Err(e.into());
        }
    };

    println!("✅ Successfully loaded keys.yaml");
    println!("   Found {} provider(s)\n", config.providers.len());

    // Test each provider
    for (name, provider_config) in config.providers.iter() {
        test_provider(name, provider_config).await;
        println!();
    }

    Ok(())
}

async fn test_provider(name: &str, config: &ProviderConfig) {
    println!("{}", "━".repeat(80));
    println!("Testing: {}", name);
    println!("{}", "━".repeat(80));
    println!();

    // Validate configuration
    println!("📋 Configuration:");
    println!("   Protocol: {}", config.protocol);
    println!("   Base URL: {}", config.base_url);
    
    // Check API key format
    let api_key = config.api_key.trim();
    println!("   API Key: {}...{}", 
        &api_key[..8.min(api_key.len())],
        if api_key.len() > 4 { &api_key[api_key.len()-4..] } else { "" }
    );
    println!("   API Key Length: {} characters", api_key.len());
    
    // Validate API key format
    let mut warnings = Vec::new();
    
    if api_key != &config.api_key {
        warnings.push("⚠️  API key has leading/trailing whitespace");
    }
    
    if config.protocol == "openai" && !api_key.starts_with("sk-") {
        warnings.push("⚠️  OpenAI-compatible keys usually start with 'sk-'");
    }
    
    if api_key.len() < 20 {
        warnings.push("⚠️  API key seems too short");
    }
    
    if api_key.contains('\n') || api_key.contains('\r') {
        warnings.push("⚠️  API key contains newline characters");
    }
    
    if !warnings.is_empty() {
        println!();
        for warning in warnings {
            println!("   {}", warning);
        }
    }
    
    println!();

    // Test with curl-like HTTP request
    println!("🧪 Testing API Key...");
    
    match config.protocol.as_str() {
        "openai" => test_openai_compatible(name, api_key, &config.base_url).await,
        "anthropic" => test_anthropic(name, api_key).await,
        "aliyun" => test_aliyun(name, api_key).await,
        "ollama" => test_ollama(name, &config.base_url).await,
        _ => {
            println!("   ⚠️  Unknown protocol: {}", config.protocol);
        }
    }
}

async fn test_openai_compatible(name: &str, api_key: &str, base_url: &str) {
    use llm_connector::LlmClient;
    
    let client = LlmClient::openai_compatible(api_key, base_url);
    
    // Test 1: Fetch models
    println!("   Test 1: Fetching models from API...");
    match client.fetch_models().await {
        Ok(models) => {
            println!("   ✅ SUCCESS! API key is valid");
            println!("   📦 Found {} model(s): {:?}", models.len(), models);
        }
        Err(e) => {
            println!("   ❌ FAILED: {}", e);
            println!();
            
            use llm_connector::error::LlmConnectorError;
            match e {
                LlmConnectorError::AuthenticationError(_) => {
                    println!("   💡 Authentication Error - Possible causes:");
                    println!("      1. API key is incorrect or expired");
                    println!("      2. API key has been revoked");
                    println!("      3. Account is out of credits");
                    println!();
                    println!("   🔧 How to fix:");
                    match name {
                        "deepseek" => {
                            println!("      - Get a new key from: https://platform.deepseek.com/api_keys");
                            println!("      - Check credits at: https://platform.deepseek.com/usage");
                        }
                        "zhipu" => {
                            println!("      - Get a new key from: https://open.bigmodel.cn/usercenter/apikeys");
                        }
                        "moonshot" => {
                            println!("      - Get a new key from: https://platform.moonshot.cn/console/api-keys");
                        }
                        _ => {
                            println!("      - Check your provider's dashboard");
                            println!("      - Generate a new API key");
                        }
                    }
                    println!();
                    println!("   🧪 Test with curl:");
                    println!("      curl {}/models \\", base_url);
                    println!("        -H \"Authorization: Bearer YOUR_API_KEY\"");
                }
                LlmConnectorError::ConnectionError(_) => {
                    println!("   💡 Connection Error - Possible causes:");
                    println!("      1. No internet connection");
                    println!("      2. Firewall/proxy blocking the request");
                    println!("      3. Base URL is incorrect");
                }
                LlmConnectorError::NetworkError(_) => {
                    println!("   💡 Network Error - Possible causes:");
                    println!("      1. Temporary network issue");
                    println!("      2. API service is down");
                }
                _ => {
                    println!("   💡 Error type: {:?}", e);
                }
            }
            return;
        }
    }
    
    // Test 2: Simple chat request
    println!();
    println!("   Test 2: Sending chat request...");
    
    use llm_connector::{ChatRequest, Message};
    
    let model = match name {
        "deepseek" => "deepseek-chat",
        "zhipu" => "glm-4",
        "moonshot" => "moonshot-v1-8k",
        _ => "gpt-3.5-turbo",
    };
    
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![Message::user("Say 'Hello' in one word")],
        ..Default::default()
    };
    
    match client.chat(&request).await {
        Ok(response) => {
            println!("   ✅ SUCCESS! Chat request works");
            if let Some(choice) = response.choices.first() {
                println!("   💬 Response: {}", choice.message.content);
                if let Some(usage) = &response.usage {
                    println!("   📊 Tokens: {} input + {} output = {} total",
                        usage.prompt_tokens,
                        usage.completion_tokens,
                        usage.total_tokens
                    );
                }
            }
        }
        Err(e) => {
            println!("   ❌ FAILED: {}", e);
            println!("   💡 The API key works for listing models but not for chat");
            println!("      This might indicate:");
            println!("      - Insufficient permissions");
            println!("      - Model '{}' not available", model);
            println!("      - Account quota exceeded");
        }
    }
}

async fn test_anthropic(_name: &str, api_key: &str) {
    println!("   ℹ️  Anthropic protocol doesn't support model listing");
    println!("   Testing with a simple chat request...");
    
    use llm_connector::{LlmClient, ChatRequest, Message};
    
    let client = LlmClient::anthropic(api_key);
    
    let request = ChatRequest {
        model: "claude-3-5-sonnet-20241022".to_string(),
        messages: vec![Message::user("Say 'Hello' in one word")],
        ..Default::default()
    };
    
    match client.chat(&request).await {
        Ok(response) => {
            println!("   ✅ SUCCESS! API key is valid");
            if let Some(choice) = response.choices.first() {
                println!("   💬 Response: {}", choice.message.content);
            }
        }
        Err(e) => {
            println!("   ❌ FAILED: {}", e);
            println!();
            println!("   💡 Get a new key from: https://console.anthropic.com/settings/keys");
        }
    }
}

async fn test_aliyun(_name: &str, api_key: &str) {
    println!("   ℹ️  Aliyun protocol doesn't support model listing");
    println!("   Testing with a simple chat request...");
    
    use llm_connector::{LlmClient, ChatRequest, Message};
    
    let client = LlmClient::aliyun(api_key);
    
    let request = ChatRequest {
        model: "qwen-turbo".to_string(),
        messages: vec![Message::user("Say 'Hello' in one word")],
        ..Default::default()
    };
    
    match client.chat(&request).await {
        Ok(response) => {
            println!("   ✅ SUCCESS! API key is valid");
            if let Some(choice) = response.choices.first() {
                println!("   💬 Response: {}", choice.message.content);
            }
        }
        Err(e) => {
            println!("   ❌ FAILED: {}", e);
            println!();
            println!("   💡 Get a new key from: https://dashscope.console.aliyun.com/apiKey");
        }
    }
}

async fn test_ollama(_name: &str, base_url: &str) {
    println!("   ℹ️  Ollama is a local service (no API key needed)");
    println!("   Testing connection to {}...", base_url);
    
    use llm_connector::{LlmClient, ChatRequest, Message};
    
    let client = LlmClient::ollama_at(base_url);
    
    let request = ChatRequest {
        model: "llama3.2".to_string(),
        messages: vec![Message::user("Say 'Hello' in one word")],
        ..Default::default()
    };
    
    match client.chat(&request).await {
        Ok(response) => {
            println!("   ✅ SUCCESS! Ollama is running");
            if let Some(choice) = response.choices.first() {
                println!("   💬 Response: {}", choice.message.content);
            }
        }
        Err(e) => {
            println!("   ❌ FAILED: {}", e);
            println!();
            println!("   💡 Make sure Ollama is running:");
            println!("      - Install: https://ollama.ai");
            println!("      - Start: ollama serve");
            println!("      - Pull model: ollama pull llama3.2");
        }
    }
}

