//! LongCat API Demo
//!
//! This example demonstrates how to use LongCat API with llm-connector.
//!
//! LongCat (https://longcat.chat) is a Chinese LLM API platform that provides:
//! - OpenAI-compatible API format
//! - Anthropic-compatible API format
//! - Free daily quota: 500,000 tokens (can be increased to 5,000,000)
//! - Models: LongCat-Flash-Chat, LongCat-Flash-Thinking
//!
//! ## Setup
//!
//! 1. Register at https://longcat.chat/platform/
//! 2. Get your API key from the API Keys page
//! 3. Set environment variable: export LONGCAT_API_KEY="your-api-key"
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example longcat_demo
//! ```

use llm_connector::{
    config::ProviderConfig,
    protocols::{
        core::{GenericProvider, Provider},
        factory::ProtocolFactoryRegistry,
        openai::longcat,
    },
    types::{ChatRequest, Message},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐱 LongCat API Demo");
    println!("===================\n");

    // Get API key from environment
    let api_key = std::env::var("LONGCAT_API_KEY").unwrap_or_else(|_| {
        println!("⚠️  LONGCAT_API_KEY not set, using demo key");
        "demo-key".to_string()
    });

    // ============================================================================
    // Method 1: Direct Protocol Usage
    // ============================================================================
    println!("📋 Method 1: Direct Protocol Usage");
    println!("   Using longcat() function directly\n");

    let config = ProviderConfig::new(&api_key).with_timeout_ms(30000);

    let provider = GenericProvider::new(config.clone(), longcat())?;

    println!("   ✅ Provider created");
    println!("   ├─ Name: {}", provider.name());
    println!("   ├─ Base URL: https://api.longcat.chat/openai");
    println!("   └─ Models: {:?}\n", provider.supported_models());

    // ============================================================================
    // Method 2: Factory Pattern
    // ============================================================================
    println!("📋 Method 2: Factory Pattern");
    println!("   Using ProtocolFactoryRegistry\n");

    let registry = ProtocolFactoryRegistry::with_defaults();

    // Check if longcat is registered
    if let Some(protocol) = registry.get_protocol_for_provider("longcat") {
        println!("   ✅ LongCat registered in factory");
        println!("   ├─ Protocol: {}", protocol);
        println!("   └─ Provider: longcat\n");
    }

    // ============================================================================
    // Supported Models
    // ============================================================================
    println!("📋 Supported Models");
    println!("===================\n");

    println!("   1. LongCat-Flash-Chat");
    println!("      ├─ Type: General conversation model");
    println!("      ├─ Performance: High-speed responses");
    println!("      └─ Use case: Chat, Q&A, general tasks\n");

    println!("   2. LongCat-Flash-Thinking");
    println!("      ├─ Type: Deep thinking model");
    println!("      ├─ Performance: More thorough reasoning");
    println!("      └─ Use case: Complex problems, analysis\n");

    // ============================================================================
    // Example Request
    // ============================================================================
    println!("📋 Example Request");
    println!("==================\n");

    let request = ChatRequest {
        model: "LongCat-Flash-Chat".to_string(),
        messages: vec![Message::user("你好！请用一句话介绍一下 LongCat。")],
        max_tokens: Some(1000),
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

    println!("   Request:");
    println!("   ├─ Model: {}", request.model);
    println!("   ├─ Messages: {} message(s)", request.messages.len());
    println!("   ├─ Max tokens: {:?}", request.max_tokens);
    println!("   └─ Temperature: {:?}\n", request.temperature);

    // Note: Actual API call would be made here
    println!("   ℹ️  To make actual API calls:");
    println!("   1. Set LONGCAT_API_KEY environment variable");
    println!("   2. Uncomment the API call code below\n");

    // Make actual API call:
    match provider.chat(&request).await {
        Ok(response) => {
            println!("   ✅ Response received:");
            if let Some(choice) = response.choices.first() {
                println!("   {}\n", choice.message.content);
            }
            if let Some(usage) = response.usage {
                println!("   Token usage:");
                println!("   ├─ Prompt: {}", usage.prompt_tokens);
                println!("   ├─ Completion: {}", usage.completion_tokens);
                println!("   └─ Total: {}", usage.total_tokens);
            }
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
        }
    }

    // ============================================================================
    // API Features
    // ============================================================================
    println!("📋 API Features");
    println!("===============\n");

    println!("   ✅ OpenAI-compatible format");
    println!("   ✅ Anthropic-compatible format");
    println!("   ✅ Free daily quota: 500,000 tokens");
    println!("   ✅ Can increase to: 5,000,000 tokens/day");
    println!("   ✅ Streaming support");
    println!("   ✅ Rate limiting: Automatic retry recommended\n");

    // ============================================================================
    // Rate Limiting
    // ============================================================================
    println!("📋 Rate Limiting");
    println!("================\n");

    println!("   When rate limited (HTTP 429):");
    println!("   {{");
    println!("     \"error\": {{");
    println!("       \"code\": \"rate_limit_exceeded\",");
    println!("       \"message\": \"请求频率超限，请稍后重试\",");
    println!("       \"type\": \"rate_limit_error\",");
    println!("       \"retry_after\": 60");
    println!("     }}");
    println!("   }}\n");

    println!("   💡 Recommendation: Use RetryMiddleware");
    println!("   ```rust");
    println!("   use llm_connector::middleware::RetryMiddleware;");
    println!("   let retry = RetryMiddleware::default();");
    println!("   retry.execute(|| provider.chat(&request)).await?;");
    println!("   ```\n");

    // ============================================================================
    // Configuration Examples
    // ============================================================================
    println!("📋 Configuration Examples");
    println!("=========================\n");

    println!("   1. Basic Configuration:");
    println!("   ```rust");
    println!("   let config = ProviderConfig::new(\"your-api-key\");");
    println!("   let provider = GenericProvider::new(config, longcat())?;");
    println!("   ```\n");

    println!("   2. With Retry:");
    println!("   ```rust");
    println!("   use llm_connector::config::RetryConfig;");
    println!("   let config = ProviderConfig::new(\"your-api-key\")");
    println!("       .with_retry(RetryConfig::default());");
    println!("   ```\n");

    println!("   3. With Custom Headers:");
    println!("   ```rust");
    println!("   let config = ProviderConfig::new(\"your-api-key\")");
    println!("       .with_header(\"X-Request-ID\", \"12345\");");
    println!("   ```\n");

    // ============================================================================
    // Links
    // ============================================================================
    println!("📋 Useful Links");
    println!("===============\n");

    println!("   🌐 Platform: https://longcat.chat/platform/");
    println!("   📚 Documentation: https://longcat.chat/platform/docs/zh/");
    println!("   🔑 API Keys: https://longcat.chat/platform/api-keys");
    println!("   📊 Usage: https://longcat.chat/platform/usage\n");

    // ============================================================================
    // Summary
    // ============================================================================
    println!("📋 Summary");
    println!("==========\n");

    println!("   ✅ LongCat support added to llm-connector");
    println!("   ✅ Compatible with OpenAI protocol");
    println!("   ✅ Easy to use with existing code");
    println!("   ✅ Free daily quota available");
    println!("   ✅ Production-ready with retry support\n");

    println!("✨ LongCat Demo Complete!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_connector::protocols::core::ProviderAdapter;

    #[test]
    fn test_longcat_protocol() {
        let protocol = longcat();
        assert_eq!(protocol.name(), "longcat");
        assert_eq!(protocol.supported_models().len(), 2);
        assert!(protocol
            .supported_models()
            .contains(&"LongCat-Flash-Chat".to_string()));
        assert!(protocol
            .supported_models()
            .contains(&"LongCat-Flash-Thinking".to_string()));
    }

    #[test]
    fn test_longcat_in_factory() {
        let registry = ProtocolFactoryRegistry::with_defaults();
        let protocol = registry.get_protocol_for_provider("longcat");
        assert_eq!(protocol, Some("openai".to_string()));
    }
}
