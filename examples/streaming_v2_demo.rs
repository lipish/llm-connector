//! V2 Streaming API Demo
//!
//! This example demonstrates the current V2 streaming API.
//! Run with: cargo run --example streaming_v2_demo --features streaming

use llm_connector::{LlmClient, types::{ChatRequest, Message}};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 V2 Streaming API Demo");
    println!("========================\n");

    // Create a mock client (will fail without real API key, but shows the API)
    let client = LlmClient::openai("sk-test-key")?;
    
    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message::user("Tell me a short story about a robot")],
        stream: Some(true),
        max_tokens: Some(100),
        ..Default::default()
    };

    println!("📝 Request:");
    println!("Model: {}", request.model);
    println!("Messages: {:?}", request.messages);
    println!("Stream: {:?}", request.stream);
    println!();

    println!("🔄 Current V2 Streaming API:");
    println!("```rust");
    println!("let mut stream = client.chat_stream(&request).await?;");
    println!("while let Some(chunk) = stream.next().await {{");
    println!("    let chunk = chunk?;");
    println!("    if let Some(content) = chunk.get_content() {{");
    println!("        print!(\"{{}}\", content);");
    println!("    }}");
    println!("}}");
    println!("```");
    println!();

    #[cfg(feature = "streaming")]
    {
        println!("⚠️  Note: This would make an actual API call:");
        println!("   To test with real API keys, update the key above");
        println!("   and ensure you have credits in your account.");
        
        // Uncomment to test with real API key:
        // let mut stream = client.chat_stream(&request).await?;
        // while let Some(chunk) = stream.next().await {
        //     let chunk = chunk?;
        //     if let Some(content) = chunk.get_content() {
        //         print!("{}", content);
        //     }
        // }
    }

    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ Streaming feature not enabled.");
        println!("   Run with: cargo run --example streaming_v2_demo --features streaming");
    }

    println!("\n✅ V2 API Demo Complete!");
    println!("\n📚 Key Differences from V1:");
    println!("   • Single unified chat_stream() method");
    println!("   • Rich StreamingResponse with convenience methods");
    println!("   • Built-in support for reasoning content");
    println!("   • Simplified, consistent API across all providers");

    Ok(())
}
