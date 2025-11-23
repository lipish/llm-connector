//! Test Zhipu streaming timeout issue
//!
//! This example tests the streaming timeout problem with Zhipu GLM API.
//! 
//! Run with:
//! ```bash
//! cargo run --example test_zhipu_streaming_timeout --features streaming
//! ```

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
use futures_util::StreamExt;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Zhipu Streaming Timeout Issue\n");
    
    let api_key = "6b4c24a7a3df47a8898b006f9f5c23b6.PXpYUIvTdUU9uKPS";
    
    // Test 1: Non-streaming request (should work)
    println!("📝 Test 1: Non-streaming request");
    println!("================================");
    test_non_streaming(api_key).await?;
    
    println!("\n");
    
    // Test 2: Streaming request (may timeout)
    println!("📝 Test 2: Streaming request");
    println!("============================");
    test_streaming(api_key).await?;
    
    Ok(())
}

async fn test_non_streaming(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::zhipu_openai_compatible(api_key)?;
    
    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: vec![Message::text(Role::User, "你好，请简单介绍一下你自己")],
        max_tokens: Some(100),
        stream: Some(false),
        ..Default::default()
    };
    
    let start = Instant::now();
    
    match client.chat(&request).await {
        Ok(response) => {
            let elapsed = start.elapsed();
            println!("✅ Non-streaming request succeeded");
            println!("⏱️  Time: {:?}", elapsed);
            println!("📊 Response length: {} chars", response.content.len());
            println!("💬 Content: {}", response.content);
        }
        Err(e) => {
            let elapsed = start.elapsed();
            println!("❌ Non-streaming request failed");
            println!("⏱️  Time: {:?}", elapsed);
            println!("🔴 Error: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

async fn test_streaming(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::zhipu_openai_compatible(api_key)?;
    
    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: vec![Message::text(Role::User, "你好，请简单介绍一下你自己")],
        max_tokens: Some(100),
        stream: Some(true),
        ..Default::default()
    };
    
    let start = Instant::now();
    
    match client.chat_stream(&request).await {
        Ok(mut stream) => {
            println!("✅ Stream created successfully");
            
            let mut chunk_count = 0;
            let mut total_content = String::new();
            let mut first_chunk_time = None;
            
            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => {
                        if first_chunk_time.is_none() {
                            first_chunk_time = Some(start.elapsed());
                            println!("⏱️  First chunk received: {:?}", first_chunk_time.unwrap());
                        }
                        
                        chunk_count += 1;
                        
                        if let Some(content) = chunk.get_content() {
                            total_content.push_str(content);
                            print!("{}", content);
                        }
                    }
                    Err(e) => {
                        let elapsed = start.elapsed();
                        println!("\n❌ Stream error at chunk {}", chunk_count);
                        println!("⏱️  Time: {:?}", elapsed);
                        println!("🔴 Error: {}", e);
                        return Err(e.into());
                    }
                }
            }
            
            let elapsed = start.elapsed();
            println!("\n\n✅ Stream completed successfully");
            println!("⏱️  Total time: {:?}", elapsed);
            println!("📊 Total chunks: {}", chunk_count);
            println!("📊 Total content length: {} chars", total_content.len());
            
            if chunk_count == 0 {
                println!("⚠️  WARNING: Received 0 chunks! This indicates a problem.");
                return Err("No chunks received".into());
            }
        }
        Err(e) => {
            let elapsed = start.elapsed();
            println!("❌ Failed to create stream");
            println!("⏱️  Time: {:?}", elapsed);
            println!("🔴 Error: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

