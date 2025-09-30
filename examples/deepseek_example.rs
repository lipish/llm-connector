//! Example demonstrating DeepSeek provider usage

use llm_connector::{ChatRequest, Client, Config, Message, ProviderConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Using environment variables
    println!("=== DeepSeek Example with Environment Variables ===");

    // Set environment variable: DEEPSEEK_API_KEY=your_api_key_here
    let client = Client::from_env();

    // Check if DeepSeek is configured
    if client.supports_model("deepseek-chat") {
        println!("✅ DeepSeek provider is configured and ready!");

        let _request = ChatRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a helpful AI assistant.".to_string(),
                    ..Default::default()
                },
                Message {
                    role: "user".to_string(),
                    content: "Hello! Can you tell me about DeepSeek?".to_string(),
                    ..Default::default()
                },
            ],
            temperature: Some(0.7),
            max_tokens: Some(150),
            ..Default::default()
        };

        // Note: This will fail without a valid API key
        // Uncomment the following lines to test with a real API key:
        /*
        match client.chat(request).await {
            Ok(response) => {
                println!("DeepSeek Response:");
                println!("{}", response.choices[0].message.content);

                if let Some(usage) = response.usage {
                    println!("\nToken Usage:");
                    println!("  Prompt tokens: {}", usage.prompt_tokens);
                    println!("  Completion tokens: {}", usage.completion_tokens);
                    println!("  Total tokens: {}", usage.total_tokens);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        */

        println!("💡 To test with real API calls, set DEEPSEEK_API_KEY environment variable");
    } else {
        println!("❌ DeepSeek provider is not configured");
        println!("💡 Set DEEPSEEK_API_KEY environment variable to configure DeepSeek");
    }

    // Example 2: Using manual configuration
    println!("\n=== DeepSeek Example with Manual Configuration ===");

    let config = Config {
        deepseek: Some(
            ProviderConfig::new("your-deepseek-api-key-here")
                .with_base_url("https://api.deepseek.com")
                .with_timeout_ms(30000),
        ),
        ..Default::default()
    };

    let client_manual = Client::with_config(config);

    println!("✅ DeepSeek provider configured manually");
    println!("Supported models: {:?}", client_manual.list_models());

    // Example 3: Model detection
    println!("\n=== Model Detection Examples ===");

    let test_models = vec![
        "deepseek-chat",
        "deepseek-reasoner",
        "deepseek/deepseek-chat",
        "gpt-4", // Should not be supported by DeepSeek provider
    ];

    for model in test_models {
        if client_manual.supports_model(model) {
            if let Some(provider) = client_manual.get_provider_info(model) {
                println!(
                    "✅ Model '{}' is supported by provider '{}'",
                    model, provider
                );
            }
        } else {
            println!("❌ Model '{}' is not supported", model);
        }
    }

    // Example 4: Streaming (requires streaming feature)
    #[cfg(feature = "streaming")]
    {
        println!("\n=== Streaming Example ===");

        let streaming_request = ChatRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Count from 1 to 5".to_string(),
                ..Default::default()
            }],
            stream: Some(true),
            ..Default::default()
        };

        println!("💡 Streaming is available with the 'streaming' feature");
        println!("💡 To test streaming, uncomment the streaming code and set a valid API key");

        /*
        use futures_util::StreamExt;

        match client.chat_stream(streaming_request).await {
            Ok(mut stream) => {
                println!("Streaming response:");
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(response) => {
                            if let Some(choice) = response.choices.first() {
                                if let Some(content) = &choice.delta.content {
                                    print!("{}", content);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Streaming error: {}", e);
                            break;
                        }
                    }
                }
                println!(); // New line after streaming
            }
            Err(e) => {
                println!("Failed to start streaming: {}", e);
            }
        }
        */
    }

    // Example 5: Error handling
    println!("\n=== Error Handling Example ===");

    let invalid_request = ChatRequest {
        model: "invalid-model".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "This should fail".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    match client_manual.chat(invalid_request).await {
        Ok(_) => {
            println!("Unexpected success");
        }
        Err(e) => {
            println!("Expected error for invalid model: {}", e);

            // Check error type
            match e {
                llm_connector::LlmConnectorError::UnsupportedModel(msg) => {
                    println!("✅ Correctly identified as unsupported model: {}", msg);
                }
                _ => {
                    println!("Different error type: {:?}", e);
                }
            }
        }
    }

    println!("\n=== DeepSeek Integration Complete ===");
    println!("🎉 DeepSeek provider is now fully integrated into llm-connector!");

    Ok(())
}
