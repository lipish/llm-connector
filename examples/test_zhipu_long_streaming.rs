//! Test Zhipu long streaming request
//!
//! This example tests whether Zhipu streaming works for longer responses.
//! 
//! Run with:
//! ```bash
//! cargo run --example test_zhipu_long_streaming --features streaming
//! ```

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
use futures_util::StreamExt;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Zhipu Long Streaming Request\n");
    
    let api_key = "6b4c24a7a3df47a8898b006f9f5c23b6.PXpYUIvTdUU9uKPS";
    
    // Test with longer response (should take more time)
    println!("📝 Testing long streaming response");
    println!("===================================");
    test_long_streaming(api_key).await?;
    
    Ok(())
}

async fn test_long_streaming(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::zhipu_openai_compatible(api_key)?;
    
    // Request a longer response
    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: vec![Message::text(
            Role::User, 
            "请详细介绍一下人工智能的发展历史，从图灵测试开始，一直到现代的大语言模型。请尽可能详细。"
        )],
        max_tokens: Some(2000), // Request longer response
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
            let mut last_chunk_time = start.elapsed();
            
            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => {
                        let current_time = start.elapsed();
                        
                        if first_chunk_time.is_none() {
                            first_chunk_time = Some(current_time);
                            println!("⏱️  First chunk received: {:?}", first_chunk_time.unwrap());
                        }
                        
                        chunk_count += 1;
                        
                        if let Some(content) = chunk.get_content() {
                            total_content.push_str(content);
                            print!("{}", content);
                        }
                        
                        last_chunk_time = current_time;
                        
                        // Log every 50 chunks
                        if chunk_count % 50 == 0 {
                            println!("\n[Chunk {}, Time: {:?}]", chunk_count, current_time);
                        }
                    }
                    Err(e) => {
                        let elapsed = start.elapsed();
                        println!("\n❌ Stream error at chunk {}", chunk_count);
                        println!("⏱️  Time: {:?}", elapsed);
                        println!("🔴 Error: {}", e);
                        
                        // Check if it's a timeout error
                        if e.to_string().contains("timeout") || e.to_string().contains("Timeout") {
                            println!("⚠️  This is a TIMEOUT error!");
                            println!("⚠️  The stream was interrupted after {:?}", elapsed);
                        }
                        
                        return Err(e.into());
                    }
                }
            }
            
            let elapsed = start.elapsed();
            println!("\n\n✅ Stream completed successfully");
            println!("⏱️  Total time: {:?}", elapsed);
            println!("⏱️  First chunk: {:?}", first_chunk_time.unwrap_or_default());
            println!("⏱️  Last chunk: {:?}", last_chunk_time);
            println!("📊 Total chunks: {}", chunk_count);
            println!("📊 Total content length: {} chars", total_content.len());
            
            if chunk_count == 0 {
                println!("⚠️  WARNING: Received 0 chunks! This indicates a problem.");
                return Err("No chunks received".into());
            }
            
            if elapsed.as_secs() > 30 {
                println!("✅ SUCCESS: Stream lasted longer than 30 seconds without timeout!");
            }
        }
        Err(e) => {
            let elapsed = start.elapsed();
            println!("❌ Failed to create stream");
            println!("⏱️  Time: {:?}", elapsed);
            println!("🔴 Error: {}", e);
            
            if e.to_string().contains("timeout") || e.to_string().contains("Timeout") {
                println!("⚠️  This is a TIMEOUT error!");
            }
            
            return Err(e.into());
        }
    }
    
    Ok(())
}

