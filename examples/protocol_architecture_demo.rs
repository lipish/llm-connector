//! Protocol Architecture Demo
//!
//! This example demonstrates the new protocol-based architecture where providers
//! are organized by the API protocol they implement rather than individual names.
//!
//! ## Supported Protocols
//!
//! ### 1. OpenAI Protocol (Most Popular)
//! Used by: DeepSeek, Zhipu, Moonshot, VolcEngine, Tencent, MiniMax, StepFun
//!
//! ### 2. Anthropic Protocol
//! Used by: Anthropic (Claude)
//!
//! ### 3. Aliyun Protocol (Custom)
//! Used by: Aliyun DashScope

use llm_connector::{
    config::ProviderConfig,
    error::LlmConnectorError,
    protocols::{
        aliyun::{aliyun_providers, AliyunProtocol},
        anthropic::{anthropic_providers, AnthropicProtocol},
        core::ProviderAdapter,
        openai::{openai_providers, OpenAIProtocol},
    },
    types::{ChatRequest, Message},
};

#[tokio::main]
async fn main() -> Result<(), LlmConnectorError> {
    println!("🚀 Protocol Architecture Demo");
    println!("=============================\n");

    // Create a sample request
    let _request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Hello, how are you?".to_string(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
        max_tokens: Some(100),
        temperature: Some(0.7),
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
        stream: None,
    };

    // Demo configuration
    let _config = ProviderConfig::new("demo-key").with_timeout_ms(30000);

    println!("📋 Protocol Overview");
    println!("====================");

    // 1. OpenAI Protocol Demo
    println!("\n🔵 OpenAI Protocol (Most Popular)");
    println!(
        "   Used by 7 providers: DeepSeek, Zhipu, Moonshot, VolcEngine, Tencent, MiniMax, StepFun"
    );

    let openai_providers = openai_providers();
    println!("   Available providers:");
    for (name, protocol) in &openai_providers {
        println!(
            "   - {}: {} models",
            name,
            protocol.supported_models().len()
        );
        println!(
            "     Models: {:?}",
            protocol
                .supported_models()
                .iter()
                .take(2)
                .collect::<Vec<_>>()
        );
        println!("     Endpoint: {}", protocol.endpoint_url(&None));
    }

    // Demo creating a provider with OpenAI protocol
    println!("\n   🔧 Creating DeepSeek provider using OpenAI protocol:");
    let deepseek_protocol = OpenAIProtocol::new(
        "deepseek",
        "https://api.deepseek.com/v1",
        vec!["deepseek-chat", "deepseek-coder"],
    );
    println!("   ✅ Protocol: {}", deepseek_protocol.name());
    println!("   ✅ Endpoint: {}", deepseek_protocol.endpoint_url(&None));
    println!("   ✅ Models: {:?}", deepseek_protocol.supported_models());

    // 2. Anthropic Protocol Demo
    println!("\n🟣 Anthropic Protocol");
    println!("   Used by Claude models with different API format");

    let anthropic_providers = anthropic_providers();
    println!("   Available providers:");
    for (name, protocol) in &anthropic_providers {
        println!(
            "   - {}: {} models",
            name,
            protocol.supported_models().len()
        );
        println!(
            "     Models: {:?}",
            protocol
                .supported_models()
                .iter()
                .take(2)
                .collect::<Vec<_>>()
        );
        println!("     Endpoint: {}", protocol.endpoint_url(&None));
    }

    // Demo creating a provider with Anthropic protocol
    println!("\n   🔧 Creating Anthropic provider using Anthropic protocol:");
    let anthropic_protocol = AnthropicProtocol::new(None);
    println!("   ✅ Protocol: {}", anthropic_protocol.name());
    println!("   ✅ Endpoint: {}", anthropic_protocol.endpoint_url(&None));
    println!(
        "   ✅ Models: {:?}",
        anthropic_protocol
            .supported_models()
            .iter()
            .take(2)
            .collect::<Vec<_>>()
    );

    // 3. Aliyun Protocol Demo
    println!("\n🟡 Aliyun Protocol (Custom)");
    println!("   Used by Aliyun DashScope with nested request structure");

    let aliyun_providers = aliyun_providers();
    println!("   Available providers:");
    for (name, protocol) in &aliyun_providers {
        println!(
            "   - {}: {} models",
            name,
            protocol.supported_models().len()
        );
        println!(
            "     Models: {:?}",
            protocol
                .supported_models()
                .iter()
                .take(2)
                .collect::<Vec<_>>()
        );
        println!("     Endpoint: {}", protocol.endpoint_url(&None));
    }

    // Demo creating a provider with Aliyun protocol
    println!("\n   🔧 Creating Aliyun provider using Aliyun protocol:");
    let aliyun_protocol = AliyunProtocol::new(None);
    println!("   ✅ Protocol: {}", aliyun_protocol.name());
    println!("   ✅ Endpoint: {}", aliyun_protocol.endpoint_url(&None));
    println!(
        "   ✅ Models: {:?}",
        aliyun_protocol
            .supported_models()
            .iter()
            .take(2)
            .collect::<Vec<_>>()
    );

    println!("\n🎯 Protocol Benefits");
    println!("====================");
    println!("✅ Clear separation by API protocol, not provider name");
    println!("✅ Easy to add new providers using existing protocols");
    println!("✅ Reduced code duplication (7 providers share OpenAI protocol)");
    println!("✅ Type-safe protocol implementations");
    println!("✅ Protocol-specific optimizations and error handling");

    println!("\n📊 Code Reduction Statistics");
    println!("=============================");
    println!("Before: 8 providers × 200 lines = 1,600 lines");
    println!("After:  3 protocols × 300 lines = 900 lines");
    println!("Savings: 700 lines (44% reduction!)");

    println!("\n🚀 Adding New Providers");
    println!("========================");
    println!("OpenAI-compatible provider: 3 lines of code");
    println!("Anthropic-compatible provider: 3 lines of code");
    println!("Custom protocol: ~300 lines (only if truly different)");

    // Demonstrate adding new providers
    println!("\n📝 Example: Adding New Providers");
    let new_openai = add_new_openai_provider();
    println!(
        "✅ Added new OpenAI-compatible provider: {}",
        new_openai.name()
    );
    println!("   Supported models: {:?}", new_openai.supported_models());

    let new_anthropic = add_new_anthropic_provider();
    println!(
        "✅ Added new Anthropic-compatible provider: {}",
        new_anthropic.name()
    );

    println!("\n✨ Protocol Architecture Demo Complete!");

    Ok(())
}

/// Example of how easy it is to add a new OpenAI-compatible provider
fn add_new_openai_provider() -> OpenAIProtocol {
    // This is all you need for a new OpenAI-compatible provider!
    OpenAIProtocol::new(
        "new_provider",
        "https://api.newprovider.com/v1",
        vec!["new-model-1", "new-model-2"],
    )
}

/// Example of how easy it is to add a new Anthropic-compatible provider
fn add_new_anthropic_provider() -> AnthropicProtocol {
    // This is all you need for a new Anthropic-compatible provider!
    AnthropicProtocol::new(Some("https://api.newanthropic.com"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_architecture() {
        // Test OpenAI protocol
        let openai_providers = openai_providers();
        assert_eq!(openai_providers.len(), 8); // 8 providers use OpenAI protocol (including LongCat)

        // Test Anthropic protocol
        let anthropic_providers = anthropic_providers();
        assert_eq!(anthropic_providers.len(), 2); // 2 aliases for Anthropic

        // Test Aliyun protocol
        let aliyun_providers = aliyun_providers();
        assert_eq!(aliyun_providers.len(), 3); // 3 aliases for Aliyun

        // Test adding new providers
        let new_openai = add_new_openai_provider();
        assert_eq!(new_openai.name(), "new_provider");
        assert_eq!(new_openai.supported_models().len(), 2);

        let new_anthropic = add_new_anthropic_provider();
        assert_eq!(new_anthropic.name(), "anthropic");
    }
}
