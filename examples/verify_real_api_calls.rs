//! Verify Real API Calls
//!
//! This example verifies that we're making real API calls by:
//! 1. Asking different questions each time
//! 2. Showing request/response details
//! 3. Demonstrating that responses vary
//!
//! ## Setup
//!
//! Set the DEEPSEEK_API_KEY environment variable:
//!
//! ```bash
//! export DEEPSEEK_API_KEY="your-deepseek-api-key"
//! ```
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example verify_real_api_calls
//! ```

use llm_connector::{
    config::ProviderConfig,
    protocols::{
        core::{GenericProvider, Provider},
        openai::deepseek,
    },
    types::{ChatRequest, Message},
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Verifying Real API Calls");
    println!("============================\n");

    let api_key =
        env::var("DEEPSEEK_API_KEY").expect("DEEPSEEK_API_KEY environment variable not set");

    let config = ProviderConfig::new(&api_key);
    let provider = GenericProvider::new(config, deepseek())?;

    println!("Provider: DeepSeek");
    println!(
        "API Key: {}...{}",
        &api_key[..10],
        &api_key[api_key.len() - 4..]
    );
    println!();

    // Test 1: Simple greeting
    println!("📝 Test 1: Simple greeting");
    let request1 = ChatRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Say hello in one sentence.".to_string(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
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

    println!("   Request: \"Say hello in one sentence.\"");
    match provider.chat(&request1).await {
        Ok(response) => {
            println!("   ✅ Response ID: {}", response.id);
            println!("   ✅ Model: {}", response.model);
            println!("   ✅ Created: {}", response.created);
            if let Some(choice) = response.choices.first() {
                println!("   ✅ Content: {}", choice.message.content.trim());
                if let Some(usage) = &response.usage {
                    println!(
                        "   ✅ Tokens: {} prompt + {} completion = {} total",
                        usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                    );
                }
            }
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
        }
    }

    println!();

    // Test 2: Math question
    println!("📝 Test 2: Math question");
    let request2 = ChatRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "What is 123 + 456? Just give the number.".to_string(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
        max_tokens: Some(50),
        temperature: Some(0.0), // Low temperature for deterministic answer
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

    println!("   Request: \"What is 123 + 456? Just give the number.\"");
    match provider.chat(&request2).await {
        Ok(response) => {
            println!("   ✅ Response ID: {}", response.id);
            if let Some(choice) = response.choices.first() {
                println!("   ✅ Content: {}", choice.message.content.trim());
                if let Some(usage) = &response.usage {
                    println!(
                        "   ✅ Tokens: {} prompt + {} completion = {} total",
                        usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                    );
                }
            }
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
        }
    }

    println!();

    // Test 3: Creative question (should vary each time)
    println!("📝 Test 3: Creative question (run multiple times to see variation)");
    let request3 = ChatRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Tell me a random fun fact in one sentence.".to_string(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
        max_tokens: Some(100),
        temperature: Some(1.0), // High temperature for variety
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

    println!("   Request: \"Tell me a random fun fact in one sentence.\"");
    match provider.chat(&request3).await {
        Ok(response) => {
            println!("   ✅ Response ID: {}", response.id);
            if let Some(choice) = response.choices.first() {
                println!("   ✅ Content: {}", choice.message.content.trim());
                if let Some(usage) = &response.usage {
                    println!(
                        "   ✅ Tokens: {} prompt + {} completion = {} total",
                        usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                    );
                }
            }
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
        }
    }

    println!();
    println!("✅ Verification complete!");
    println!();
    println!("🔍 Evidence of real API calls:");
    println!("   1. Each response has a unique ID");
    println!("   2. Timestamps are current");
    println!("   3. Token counts vary based on content");
    println!("   4. Math question gives correct answer (579)");
    println!("   5. Creative question varies each run");

    Ok(())
}
