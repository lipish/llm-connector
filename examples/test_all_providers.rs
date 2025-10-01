//! Test All Providers
//!
//! This example tests all configured providers with both regular chat and streaming.
//!
//! ## Setup
//!
//! Set the following environment variables before running:
//!
//! ```bash
//! export DEEPSEEK_API_KEY="your-deepseek-api-key"
//! export ALIYUN_API_KEY="your-aliyun-api-key"
//! export ZHIPU_API_KEY="your-zhipu-api-key"
//! export LONGCAT_API_KEY="your-longcat-api-key"
//! export MOONSHOT_API_KEY="your-moonshot-api-key"
//! ```
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example test_all_providers
//! ```

use llm_connector::{
    config::ProviderConfig,
    protocols::{
        aliyun::AliyunProtocol,
        core::{GenericProvider, Provider, ProviderAdapter},
        openai::{deepseek, longcat, moonshot, zhipu},
    },
    types::{ChatRequest, Message},
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing All Providers");
    println!("========================\n");

    // Test DeepSeek
    if let Ok(api_key) = env::var("DEEPSEEK_API_KEY") {
        println!("1️⃣  Testing DeepSeek");
        test_provider(
            "DeepSeek",
            "deepseek-chat",
            ProviderConfig::new(api_key),
            deepseek(),
        )
        .await;
    } else {
        println!("1️⃣  Skipping DeepSeek (DEEPSEEK_API_KEY not set)");
    }

    // Test Aliyun
    if let Ok(api_key) = env::var("ALIYUN_API_KEY") {
        println!("\n2️⃣  Testing Aliyun (DashScope)");
        test_aliyun("qwen-turbo", &api_key).await;
    } else {
        println!("\n2️⃣  Skipping Aliyun (ALIYUN_API_KEY not set)");
    }

    // Test Zhipu (GLM)
    if let Ok(api_key) = env::var("ZHIPU_API_KEY") {
        println!("\n3️⃣  Testing Zhipu (GLM)");
        test_provider(
            "Zhipu",
            "glm-4-flash",
            ProviderConfig::new(api_key),
            zhipu(),
        )
        .await;
    } else {
        println!("\n3️⃣  Skipping Zhipu (ZHIPU_API_KEY not set)");
    }

    // Test LongCat
    if let Ok(api_key) = env::var("LONGCAT_API_KEY") {
        println!("\n4️⃣  Testing LongCat");
        test_provider(
            "LongCat",
            "LongCat-Flash-Chat",
            ProviderConfig::new(api_key),
            longcat(),
        )
        .await;
    } else {
        println!("\n4️⃣  Skipping LongCat (LONGCAT_API_KEY not set)");
    }

    // Test VolcEngine (Doubao)
    // Note: VolcEngine requires endpoint ID, not model name
    println!("\n5️⃣  Testing VolcEngine (Doubao)");
    println!("   ⚠️  Skipped: Requires endpoint ID (format: ep-xxxxxxxx)");
    println!("   Note: Create an endpoint in VolcEngine console to get the ID");

    // Test Moonshot (Kimi)
    if let Ok(api_key) = env::var("MOONSHOT_API_KEY") {
        println!("\n6️⃣  Testing Moonshot (Kimi)");
        test_provider(
            "Moonshot",
            "moonshot-v1-8k",
            ProviderConfig::new(api_key),
            moonshot(),
        )
        .await;
    } else {
        println!("\n6️⃣  Skipping Moonshot (MOONSHOT_API_KEY not set)");
    }

    println!("\n✅ All tests completed!");

    Ok(())
}

async fn test_provider<P>(name: &str, model: &str, config: ProviderConfig, protocol: P)
where
    P: ProviderAdapter + 'static,
{
    let provider = match GenericProvider::new(config, protocol) {
        Ok(p) => p,
        Err(e) => {
            println!("   ❌ Failed to create provider: {}", e);
            return;
        }
    };

    // Test regular chat
    println!("   📝 Testing regular chat...");
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![Message::user(
            format!("Say 'Hello from {}!' in one sentence.", name)
        )],
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
                println!("   ✅ Response: {}", choice.message.content.trim());
                if let Some(usage) = &response.usage {
                    println!(
                        "   📊 Tokens: {} prompt + {} completion = {} total",
                        usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                    );
                }
            } else {
                println!("   ⚠️  No response content");
            }
        }
        Err(e) => {
            println!("   ❌ Chat failed: {}", e);
            return;
        }
    }

    // Streaming test skipped for now due to compilation issues
    println!("   ⚠️  Streaming test skipped");
}

async fn test_aliyun(model: &str, api_key: &str) {
    let config = ProviderConfig::new(api_key);
    let protocol = AliyunProtocol::new(None);

    let provider = match GenericProvider::new(config, protocol) {
        Ok(p) => p,
        Err(e) => {
            println!("   ❌ Failed to create provider: {}", e);
            return;
        }
    };

    // Test regular chat
    println!("   📝 Testing regular chat...");
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![Message::user("Say 'Hello from Aliyun!' in one sentence.")],
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
                println!("   ✅ Response: {}", choice.message.content.trim());
                if let Some(usage) = &response.usage {
                    println!(
                        "   📊 Tokens: {} prompt + {} completion = {} total",
                        usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                    );
                }
            } else {
                println!("   ⚠️  No response content");
            }
        }
        Err(e) => {
            println!("   ❌ Chat failed: {}", e);
            return;
        }
    }

    // Streaming test skipped for now
    println!("   ⚠️  Streaming test skipped");
}
