//! Volcengine streaming example
//!
//! Tests streaming responses from the Volcengine Ark API.
//!
//! Usage:
//! ```bash
//! cargo run --example volcengine_streaming --features streaming -- <api_key> <endpoint>
//! ```
//!
//! Example:
//! ```bash
//! cargo run --example volcengine_streaming --features streaming -- \
//!   xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx \
//!   ep-20250118155555-xxxxx
//! ```

use llm_connector::types::{ChatRequest, Message, MessageBlock, Role};
use std::env;

#[cfg(feature = "streaming")]
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <api_key> <endpoint>", args[0]);
        eprintln!(
            "Example: {} xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx ep-20250118155555-xxxxx",
            args[0]
        );
        std::process::exit(1);
    }

    let api_key = &args[1];
    let endpoint = &args[2];

    println!("🔧 Creating Volcengine provider...");
    println!(
        "   API Key: {}...{}",
        &api_key[..8],
        &api_key[api_key.len() - 4..]
    );
    println!("   Endpoint: {}", endpoint);

    let request = ChatRequest {
        model: endpoint.to_string(), // Volcengine uses endpoint ID as model
        messages: vec![Message {
            role: Role::User,
            content: vec![MessageBlock::Text {
                text: "Introduce yourself in one sentence".to_string(),
            }],
            name: None,
            tool_calls: None,
            tool_call_id: None,
            reasoning: None,
            reasoning_content: None,
            thinking: None,
            thought: None,
        }],
        temperature: Some(0.7),
        max_tokens: Some(100),
        stream: Some(true),
        ..Default::default()
    };

    println!("\n📤 Sending streaming request...");
    println!("   Model: {}", request.model);
    println!("   Message: {:?}", request.messages[0].content);

    #[cfg(feature = "streaming")]
    {
        let client = llm_connector::LlmClient::volcengine(api_key)?;
        let mut stream = client.chat_stream(&request).await?;

        println!("\n📥 Receiving streaming response:");
        println!("---");

        let mut chunk_count = 0;
        let mut total_content = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    chunk_count += 1;

                    // Debug: print the full chunk structure
                    if chunk_count <= 3 {
                        println!("\n[DEBUG] Chunk #{}: {:?}", chunk_count, chunk);
                    }

                    // Try to extract content
                    if let Some(content) = chunk.get_content() {
                        print!("{}", content);
                        total_content.push_str(content);
                    } else {
                        // If get_content() is empty, inspect raw data
                        if !chunk.choices.is_empty() {
                            let choice = &chunk.choices[0];
                            println!("\n[DEBUG] Choice delta: {:?}", choice.delta);
                        }
                    }

                    // Check finish_reason
                    if let Some(choice) = chunk.choices.first()
                        && let Some(reason) = &choice.finish_reason
                    {
                        println!("\n\n[Finish reason: {}]", reason);
                    }
                }
                Err(e) => {
                    eprintln!("\n❌ Error in stream: {}", e);
                    break;
                }
            }
        }

        println!("\n---");
        println!("\n✅ Streaming completed!");
        println!("   Total chunks: {}", chunk_count);
        println!("   Total content length: {} chars", total_content.len());

        if total_content.is_empty() {
            println!(
                "\n⚠️  WARNING: No content received! This indicates a streaming parsing issue."
            );
        } else {
            println!("\n📝 Complete response:");
            println!("{}", total_content);
        }
    }

    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ Streaming feature not enabled. Please run with --features streaming");
    }

    Ok(())
}
