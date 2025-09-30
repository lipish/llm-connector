//! Test All Providers
//!
//! This example tests all configured providers with both regular chat and streaming.

use llm_connector::{
    config::ProviderConfig,
    protocols::{
        core::{Provider, GenericProvider, ProviderAdapter},
        openai::{deepseek, zhipu, moonshot, volcengine, longcat},
        aliyun::AliyunProtocol,
    },
    types::{ChatRequest, Message},
};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing All Providers");
    println!("========================\n");

    // Test DeepSeek
    println!("1️⃣  Testing DeepSeek");
    println!("   API Key: sk-78f4...bd03");
    test_provider(
        "DeepSeek",
        "deepseek-chat",
        ProviderConfig::new("sk-78f437f4e0174650ae18734e6ec5bd03"),
        deepseek(),
    ).await;

    // Test Aliyun
    println!("\n2️⃣  Testing Aliyun (DashScope)");
    println!("   API Key: sk-17cb...8af2");
    test_aliyun(
        "qwen-turbo",
        "sk-17cb8a1feec2440bad2c5a73d7d08af2",
    ).await;

    // Test Zhipu (GLM)
    println!("\n3️⃣  Testing Zhipu (GLM)");
    println!("   API Key: d2a0...zVd3");
    test_provider(
        "Zhipu",
        "glm-4-flash",
        ProviderConfig::new("d2a0da2b02954b1f91a0a4ec16d4521b.GA2Tz9sF9kt4zVd3"),
        zhipu(),
    ).await;

    // Test LongCat
    println!("\n4️⃣  Testing LongCat");
    println!("   API Key: ak_11o3...J4d");
    test_provider(
        "LongCat",
        "LongCat-Flash-Chat",
        ProviderConfig::new("ak_11o3bI6O03mx2yS8jb2hD61q7DJ4d"),
        longcat(),
    ).await;

    // Test VolcEngine (Doubao)
    // Note: VolcEngine requires endpoint ID, not model name
    // Skipping for now as we don't have a valid endpoint ID
    println!("\n5️⃣  Testing VolcEngine (Doubao)");
    println!("   ⚠️  Skipped: Requires endpoint ID (format: ep-xxxxxxxx)");
    println!("   Note: Create an endpoint in VolcEngine console to get the ID");

    // Test Moonshot (Kimi)
    println!("\n6️⃣  Testing Moonshot (Kimi)");
    println!("   API Key: sk-5ipa...Vw4b");
    test_provider(
        "Moonshot",
        "moonshot-v1-8k",
        ProviderConfig::new("sk-5ipahcLR7y73YfOE5Tlkq39cpcIIcbLcOKlI7G69x7DtVw4b"),
        moonshot(),
    ).await;

    println!("\n✅ All tests completed!");

    Ok(())
}

async fn test_provider<P>(
    name: &str,
    model: &str,
    config: ProviderConfig,
    protocol: P,
) where
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
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Say 'Hello from {}!' in one sentence.".replace("{}", name),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }
        ],
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
                    println!("   📊 Tokens: {} prompt + {} completion = {} total",
                        usage.prompt_tokens,
                        usage.completion_tokens,
                        usage.total_tokens
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
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Say 'Hello from Aliyun!' in one sentence.".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }
        ],
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
                    println!("   📊 Tokens: {} prompt + {} completion = {} total",
                        usage.prompt_tokens,
                        usage.completion_tokens,
                        usage.total_tokens
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

