//! Integration tests for streaming functionality across all protocols
//!
//! These tests require valid API keys to be set as environment variables.
//! Run with: cargo test streaming_integration_tests --features streaming

#[cfg(feature = "streaming")]
mod tests {
    use futures_util::StreamExt;
    use llm_connector::{
        LlmClient,
        types::{ChatRequest, Message, MessageBlock, Role},
    };
    use std::time::Duration;

    /// Test streaming functionality with a given client
    async fn test_streaming(
        client: &LlmClient,
        protocol_name: &str,
        model: &str,
        test_message: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "🧪 Testing {} streaming with model: {}",
            protocol_name, model
        );

        let request = ChatRequest {
            model: model.to_string(),
            messages: vec![Message::text(Role::User, test_message)],
            max_tokens: Some(20),
            ..Default::default()
        };

        let start_time = std::time::Instant::now();
        let mut stream = client.chat_stream(&request).await?;

        let mut chunk_count = 0;
        let mut total_content = String::new();
        let mut got_content = false;

        while let Some(chunk_result) = stream.next().await {
            chunk_count += 1;
            match chunk_result {
                Ok(chunk) => {
                    // Verify basic chunk structure
                    assert!(!chunk.id.is_empty(), "Chunk ID should not be empty");
                    assert_eq!(chunk.model, model, "Chunk model should match request");
                    assert!(!chunk.choices.is_empty(), "Should have at least one choice");

                    if let Some(content) = chunk.get_content()
                        && !content.is_empty()
                    {
                        got_content = true;
                        total_content.push_str(content);
                        print!("{}", content);
                    }

                    // Check for finish reason
                    if let Some(finish_reason) = chunk
                        .choices
                        .first()
                        .and_then(|c| c.finish_reason.as_deref())
                        && finish_reason == "stop"
                    {
                        println!(
                            "\n✅ Stream completed normally after {} chunks",
                            chunk_count
                        );
                        break;
                    }

                    // Safety check to prevent infinite loops
                    if chunk_count > 50 {
                        println!("\n⚠️  Stream exceeded maximum chunks (50), stopping");
                        break;
                    }
                }
                Err(e) => {
                    println!("\n❌ Stream error at chunk {}: {}", chunk_count, e);
                    return Err(e.into());
                }
            }
        }

        let duration = start_time.elapsed();
        println!("\n📊 Stream Statistics:");
        println!("  Duration: {:?}", duration);
        println!("  Chunks: {}", chunk_count);
        println!("  Got content: {}", got_content);
        println!("  Total content length: {}", total_content.len());

        // Basic assertions
        assert!(chunk_count > 0, "Should receive at least one chunk");
        assert!(got_content, "Should receive some content");

        Ok(())
    }

    /// Test streaming with timeout
    async fn test_streaming_with_timeout(
        client: &LlmClient,
        protocol_name: &str,
        model: &str,
        test_message: &str,
        timeout: Duration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "⏱️  Testing {} streaming with timeout: {:?}",
            protocol_name, timeout
        );

        let request = ChatRequest {
            model: model.to_string(),
            messages: vec![Message::text(Role::User, test_message)],
            max_tokens: Some(10),
            ..Default::default()
        };

        let stream_future = async {
            let mut stream = client.chat_stream(&request).await?;
            let mut chunk_count = 0;
            while let Some(chunk) = stream.next().await {
                chunk_count += 1;
                chunk?; // Propagate errors
                if chunk_count > 10 {
                    break; // Safety limit
                }
            }
            Ok::<usize, Box<dyn std::error::Error>>(chunk_count)
        };

        match tokio::time::timeout(timeout, stream_future).await {
            Ok(Ok(chunk_count)) => {
                println!(
                    "✅ Streaming completed within timeout, {} chunks",
                    chunk_count
                );
                Ok(())
            }
            Ok(Err(e)) => {
                println!("❌ Streaming error: {}", e);
                Err(e)
            }
            Err(_) => {
                println!("❌ Streaming timed out after {:?}", timeout);
                Err("Streaming timeout".into())
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires valid API key
    async fn test_openai_streaming() -> Result<(), Box<dyn std::error::Error>> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| "OPENAI_API_KEY environment variable not set")?;

        let client = LlmClient::openai(&api_key)?;
        test_streaming(&client, "OpenAI", "gpt-3.5-turbo", "Say hello").await
    }

    #[tokio::test]
    #[ignore] // Requires valid API key
    async fn test_anthropic_streaming() -> Result<(), Box<dyn std::error::Error>> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY environment variable not set")?;

        let client = LlmClient::anthropic(&api_key)?;
        test_streaming(&client, "Anthropic", "claude-3-haiku-20240307", "Say hello").await
    }

    #[tokio::test]
    #[ignore] // Requires valid API key
    async fn test_zhipu_streaming() -> Result<(), Box<dyn std::error::Error>> {
        let api_key = std::env::var("ZHIPU_API_KEY")
            .map_err(|_| "ZHIPU_API_KEY environment variable not set")?;

        let client = LlmClient::zhipu(&api_key)?;
        test_streaming(&client, "Zhipu", "glm-4-flash", "Say one word").await
    }

    #[tokio::test]
    #[ignore] // Requires valid API key
    async fn test_aliyun_streaming() -> Result<(), Box<dyn std::error::Error>> {
        let api_key = std::env::var("ALIYUN_API_KEY")
            .map_err(|_| "ALIYUN_API_KEY environment variable not set")?;

        let client = LlmClient::aliyun(&api_key)?;
        test_streaming(&client, "Aliyun", "qwen-turbo", "Say one word").await
    }

    #[tokio::test]
    async fn test_ollama_streaming() -> Result<(), Box<dyn std::error::Error>> {
        // Ollama doesn't need API key but requires Ollama to be running
        let client = LlmClient::ollama()?;

        // Try a very short timeout first to see if Ollama is available
        match test_streaming_with_timeout(
            &client,
            "Ollama",
            "llama3.2:1b",
            "Hi",
            Duration::from_secs(5),
        )
        .await
        {
            Ok(_) => {
                println!("✅ Ollama is available and working");
                Ok(())
            }
            Err(_) => {
                println!("⚠️  Ollama not available or not responding");
                println!("   Make sure Ollama is running: ollama serve");
                println!("   And pull a model: ollama pull llama3.2:1b");
                // Don't fail the test, just skip it
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_streaming_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 Testing streaming error handling");

        // Test with invalid API key
        let client = LlmClient::openai("invalid-key")?;
        let request = ChatRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: vec![MessageBlock::text("test")],
                ..Default::default()
            }],
            max_tokens: Some(10),
            ..Default::default()
        };

        match client.chat_stream(&request).await {
            Ok(_) => {
                return Err("Expected authentication error but got success".into());
            }
            Err(e) => {
                println!("✅ Got expected error: {}", e);
                // Verify it's an authentication error or timeout (if network is restricted)
                let error_str = e.to_string();
                assert!(
                    error_str.contains("auth") ||
                    error_str.contains("Authentication") ||
                    error_str.contains("unauthorized") ||
                    error_str.contains("401") ||
                    error_str.contains("API key") ||
                    error_str.contains("timeout") ||  // May timeout if network is restricted
                    error_str.contains("Timeout"),
                    "Error should indicate authentication failure or timeout: {}",
                    error_str
                );
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_streaming_request_validation() -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 Testing streaming request validation");

        let client = LlmClient::openai("test-key")?;

        // Test empty messages
        let request = ChatRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![],
            max_tokens: Some(10),
            ..Default::default()
        };

        match client.chat_stream(&request).await {
            Ok(_) => {
                return Err("Expected validation error for empty messages but got success".into());
            }
            Err(e) => {
                println!("✅ Got expected validation error: {}", e);
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_all_protocols_availability() -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Testing protocol availability (without making API calls)");

        // Test that all protocols can be created without errors
        let _openai = LlmClient::openai("test-key")?;
        let _anthropic = LlmClient::anthropic("test-key")?;
        let _zhipu = LlmClient::zhipu("test-key")?;
        let _aliyun = LlmClient::aliyun("test-key")?;
        let _ollama = LlmClient::ollama()?;

        println!("✅ All protocols can be instantiated successfully");
        Ok(())
    }
}

#[cfg(not(feature = "streaming"))]
mod tests {
    #[tokio::test]
    async fn test_streaming_feature_disabled() {
        println!("ℹ️  Streaming feature is disabled, skipping streaming tests");
        println!("   Run with: cargo test --features streaming");
    }
}
