//! Debug example for DeepSeek API authentication
//!
//! This example helps debug authentication issues with DeepSeek API
//!
//! Usage:
//!   export DEEPSEEK_API_KEY="sk-..."
//!   cargo run --example debug_deepseek
//!
//! Or pass the key as an argument:
//!   cargo run --example debug_deepseek -- sk-...

use llm_connector::{LlmClient, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 DeepSeek API Debug Tool\n");

    // Get API key from command line argument or environment variable
    let api_key = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("DEEPSEEK_API_KEY").ok())
        .expect("Please provide API key via:\n  1. Command line: cargo run --example debug_deepseek -- sk-...\n  2. Environment: export DEEPSEEK_API_KEY=sk-...");

    // Validate API key format
    if !api_key.starts_with("sk-") {
        eprintln!("⚠️  Warning: DeepSeek API keys should start with 'sk-'");
        eprintln!("   Your key starts with: {}", &api_key[..3.min(api_key.len())]);
    }

    if api_key.len() < 20 {
        eprintln!("⚠️  Warning: API key seems too short (length: {})", api_key.len());
        eprintln!("   DeepSeek API keys are typically much longer");
    }

    println!("📋 Configuration:");
    println!("   API Key: {}...{}",
        &api_key[..8.min(api_key.len())],
        if api_key.len() > 4 { &api_key[api_key.len()-4..] } else { "" }
    );
    println!("   API Key Length: {} characters", api_key.len());
    println!("   Base URL: https://api.deepseek.com/v1");
    println!("   Model: deepseek-chat");
    println!();

    // Test 1: Fetch models
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 1: Fetch Available Models");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let client = LlmClient::openai(
        &api_key,
        Some("https://api.deepseek.com/v1")
    );

    match client.fetch_models().await {
        Ok(models) => {
            println!("✅ Successfully fetched models!");
            println!("   Available models: {:?}", models);
        }
        Err(e) => {
            println!("❌ Failed to fetch models");
            println!("   Error: {}", e);
            println!();

            // Provide specific troubleshooting based on error type
            use llm_connector::error::LlmConnectorError;
            match &e {
                LlmConnectorError::AuthenticationError(_) => {
                    println!("💡 Authentication Error - Troubleshooting:");
                    println!("   1. Verify your API key is correct");
                    println!("      - Get your key from: https://platform.deepseek.com/api_keys");
                    println!("      - Make sure there are no extra spaces or newlines");
                    println!("   2. Check if your API key has expired");
                    println!("   3. Ensure your account has sufficient credits");
                    println!();
                    println!("   Test your key with curl:");
                    println!("   curl https://api.deepseek.com/v1/models \\");
                    println!("     -H \"Authorization: Bearer YOUR_API_KEY\"");
                }
                LlmConnectorError::ConnectionError(_) => {
                    println!("💡 Connection Error - Troubleshooting:");
                    println!("   1. Check your internet connection");
                    println!("   2. Verify you can reach api.deepseek.com");
                    println!("   3. Check if you're behind a proxy/firewall");
                }
                LlmConnectorError::NetworkError(_) => {
                    println!("💡 Network Error - Troubleshooting:");
                    println!("   1. Check your internet connection");
                    println!("   2. Try again in a few moments");
                }
                _ => {
                    println!("💡 Unexpected error type: {:?}", e);
                }
            }
            println!();
            return Err(e.into());
        }
    }

    println!();

    // Test 2: Simple chat request
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 2: Simple Chat Request");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let request = ChatRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![Message::user("Say 'Hello' in one word")],
        ..Default::default()
    };

    match client.chat(&request).await {
        Ok(response) => {
            println!("✅ Chat request successful!");
            if let Some(choice) = response.choices.first() {
                println!("   Response: {}", choice.message.content);
                if let Some(usage) = &response.usage {
                    println!("   Tokens: {} input + {} output = {} total",
                        usage.prompt_tokens,
                        usage.completion_tokens,
                        usage.total_tokens
                    );
                }
            }
        }
        Err(e) => {
            println!("❌ Chat request failed");
            println!("   Error: {}", e);
            println!("   Error type: {:?}", e);
            return Err(e.into());
        }
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✨ All tests passed!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

