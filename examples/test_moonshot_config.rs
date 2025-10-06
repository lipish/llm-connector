//! Test Moonshot Configuration Using Config.moonshot Field
//!
//! This test verifies that the moonshot field in Config works correctly

use llm_connector::{
    config::{Config, ProviderConfig},
    types::{ChatRequest, Message},
    Client,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Moonshot Configuration via Config.moonshot");
    println!("====================================================\n");

    // Read API keys from keys.yaml
    println!("📖 Reading API keys from keys.yaml...");
    let yaml_content = std::fs::read_to_string("keys.yaml")?;
    let api_keys: std::collections::HashMap<String, String> = serde_yaml::from_str(&yaml_content)?;

    let moonshot_key = match api_keys.get("moonshot") {
        Some(key) => key,
        None => {
            println!("❌ Moonshot API key not found in keys.yaml");
            return Ok(());
        }
    };

    println!("✅ Moonshot API key loaded successfully");

    // Create config with moonshot field set (without using Default::default())
    println!("\n🔧 Creating Config with moonshot field set...");
    let config = Config {
        openai: None,
        anthropic: None,
        deepseek: None,
        zhipu: None,
        aliyun: None,
        moonshot: Some(ProviderConfig::new(moonshot_key.clone())),
    };

    println!("✅ Config created with moonshot field");

    // Create client with the config
    println!("\n🚀 Creating client with config...");
    let client = Client::with_config(config);

    // List configured providers
    println!("📋 Configured providers: {:?}", client.list_providers());

    // Test the connection with a simple chat request
    println!("\n📝 Testing chat connection using model name 'moonshot/moonshot-v1-8k'...");
    let request = ChatRequest {
        model: "moonshot/moonshot-v1-8k".to_string(),
        messages: vec![Message::user("Hello! Please respond with 'Moonshot configuration successful!'")],
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
                println!("✅ Chat successful!");
                println!("📄 Response: {}", choice.message.content.trim());

                if let Some(usage) = &response.usage {
                    println!("📊 Token usage:");
                    println!("   - Prompt tokens: {}", usage.prompt_tokens);
                    println!("   - Completion tokens: {}", usage.completion_tokens);
                    println!("   - Total tokens: {}", usage.total_tokens);
                }

                println!("\n🎉 Moonshot configuration test completed successfully!");
            } else {
                println!("⚠️  No response content received");
            }
        }
        Err(e) => {
            println!("❌ Chat failed: {}", e);
            return Ok(());
        }
    }

    Ok(())
}