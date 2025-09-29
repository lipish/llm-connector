//! Real DeepSeek API test - requires valid API key

use llm_connector::{ChatRequest, Client, Message};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if API key is set
    let api_key = match env::var("DEEPSEEK_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            println!("❌ DEEPSEEK_API_KEY environment variable is not set or empty");
            println!("💡 Please set your DeepSeek API key:");
            println!("   export DEEPSEEK_API_KEY=\"your-api-key-here\"");
            println!("💡 Get your API key from: https://platform.deepseek.com/api_keys");
            return Ok(());
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

    // Initialize client
    let client = Client::from_env();

    // Verify DeepSeek is configured
    if !client.supports_model("deepseek-chat") {
        println!("❌ DeepSeek provider is not properly configured");
        return Ok(());
    }

    println!("✅ DeepSeek provider is configured");
    println!("📋 Supported models: {:?}", client.list_models());

    // Test 1: Simple chat completion
    println!("\n=== Test 1: Simple Chat Completion ===");

    let request = ChatRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant. Please respond concisely.".to_string(),
                ..Default::default()
            },
            Message {
                role: "user".to_string(),
                content: "Hello! Please tell me what 2+2 equals and explain briefly.".to_string(),
                ..Default::default()
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(100),
        ..Default::default()
    };

    println!("🚀 Sending request to DeepSeek API...");

    match client.chat(request).await {
        Ok(response) => {
            println!("✅ Success! Received response:");
            println!("📝 Response ID: {}", response.id);
            println!("🤖 Model: {}", response.model);

            if let Some(choice) = response.choices.first() {
                println!("💬 Assistant: {}", choice.message.content);
                println!("🏁 Finish reason: {:?}", choice.finish_reason);
            }

            if let Some(usage) = &response.usage {
                println!("📊 Token usage:");
                println!("   Prompt tokens: {}", usage.prompt_tokens);
                println!("   Completion tokens: {}", usage.completion_tokens);
                println!("   Total tokens: {}", usage.total_tokens);

                if let Some(cache_hit) = usage.prompt_cache_hit_tokens {
                    println!("   Cache hit tokens: {}", cache_hit);
                }
                if let Some(cache_miss) = usage.prompt_cache_miss_tokens {
                    println!("   Cache miss tokens: {}", cache_miss);
                }
            }
        }
        Err(e) => {
            println!("❌ Error: {}", e);
            println!("🔍 Error details: {:?}", e);

            // Provide helpful debugging information
            match &e {
                llm_connector::LlmConnectorError::NetworkError(msg) => {
                    println!("🌐 Network issue - check your internet connection");
                    println!("   Details: {}", msg);
                }
                llm_connector::LlmConnectorError::AuthenticationError(msg) => {
                    println!("🔐 Authentication failed - check your API key");
                    println!("   Details: {}", msg);
                }
                llm_connector::LlmConnectorError::RateLimitError(msg) => {
                    println!("⏱️  Rate limit exceeded - please wait and try again");
                    println!("   Details: {}", msg);
                }
                llm_connector::LlmConnectorError::ProviderError(msg) => {
                    println!("🏭 Provider error - issue with DeepSeek API");
                    println!("   Details: {}", msg);
                }
                _ => {
                    println!("❓ Other error type");
                }
            }
            return Err(e.into());
        }
    }

    // Test 2: DeepSeek Reasoner model
    println!("\n=== Test 2: DeepSeek Reasoner Model ===");

    let reasoner_request = ChatRequest {
        model: "deepseek-reasoner".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "If I have 3 apples and I buy 2 more, then give away 1, how many apples do I have? Please show your reasoning.".to_string(),
                ..Default::default()
            }
        ],
        temperature: Some(0.1), // Lower temperature for reasoning
        max_tokens: Some(200),
        ..Default::default()
    };

    println!("🧠 Testing DeepSeek Reasoner...");

    match client.chat(reasoner_request).await {
        Ok(response) => {
            println!("✅ Reasoner response received:");

            if let Some(choice) = response.choices.first() {
                println!("🤔 Reasoning: {}", choice.message.content);
            }

            if let Some(usage) = &response.usage {
                if let Some(details) = &usage.completion_tokens_details {
                    if let Some(reasoning_tokens) = details.reasoning_tokens {
                        println!("🧮 Reasoning tokens used: {}", reasoning_tokens);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Reasoner test failed: {}", e);
        }
    }

    // Test 3: Streaming (if feature is enabled)
    #[cfg(feature = "streaming")]
    {
        println!("\n=== Test 3: Streaming Response ===");

        let stream_request = ChatRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Count from 1 to 30, one number per response.".to_string(),
                ..Default::default()
            }],
            stream: Some(true),
            max_tokens: Some(300),
            ..Default::default()
        };

        println!("🌊 Testing streaming...");

        match client.chat_stream(stream_request).await {
            Ok(mut stream) => {
                use futures_util::StreamExt;

                println!("📡 Streaming response:");
                print!("🤖 ");

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(response) => {
                            if let Some(choice) = response.choices.first() {
                                if let Some(content) = &choice.delta.content {
                                    print!("{}", content);
                                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                                }
                            }
                        }
                        Err(e) => {
                            println!("\n❌ Streaming error: {}", e);
                            break;
                        }
                    }
                }
                println!("\n✅ Streaming completed");
            }
            Err(e) => {
                println!("❌ Streaming failed: {}", e);
            }
        }
    }

    #[cfg(not(feature = "streaming"))]
    {
        println!("\n=== Test 3: Streaming (Disabled) ===");
        println!("💡 Streaming is not enabled. Add 'streaming' feature to test streaming.");
    }

    println!("\n🎉 DeepSeek integration test completed!");
    println!("💡 If all tests passed, your DeepSeek integration is working correctly!");

    Ok(())
}
