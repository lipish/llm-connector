//! Real DeepSeek API test - requires valid API key
#![allow(dead_code)]
#![allow(unused_imports)]

use llm_connector::{ChatRequest, Client, Message};
use std::env;

/// Real-world test for DeepSeek API.
///
/// This test is marked as `#[ignore]` because it requires a valid `DEEPSEEK_API_KEY`
/// environment variable to be set. It also makes real API calls, which may incur costs.
///
/// To run this test:
/// 1. Set the environment variable:
///    `export DEEPSEEK_API_KEY="your-api-key-here"`
/// 2. Run the test specifically:
///    `cargo test -- --ignored test_deepseek_real_chat_completion`
///
/// This test verifies:
/// - Correct API key handling.
/// - Successful chat completion requests.
/// - Correct response parsing.
#[tokio::test]
#[ignore]
async fn test_deepseek_real_chat_completion() {
    let api_key = match env::var("DEEPSEEK_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            println!("⚠️ DEEPSEEK_API_KEY not set, skipping test.");
            return;
        }
    };

    println!(
        "🔑 Found DeepSeek API key: {}...{}",
        &api_key[..8.min(api_key.len())],
        if api_key.len() > 16 {
            &api_key[api_key.len() - 8..]
        } else {
            ""
        }
    );

    let client = Client::from_env();
    assert!(
        client.supports_model("deepseek-chat"),
        "DeepSeek provider should be configured"
    );

    let request = ChatRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
                ..Default::default()
            },
            Message {
                role: "user".to_string(),
                content: "Hello!".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let result = client.chat(request).await;
    assert!(result.is_ok(), "API call should succeed");

    let response = result.unwrap();
    assert!(!response.choices.is_empty(), "Should receive choices");
    assert!(
        !response.choices[0].message.content.is_empty(),
        "Response content should not be empty"
    );

    println!("✅ Real DeepSeek API test passed!");
}
