//! Anthropic Streaming Response Example

//! Demonstrates how to use enhanced Anthropic streaming response functionality

#[cfg(feature = "streaming")]
use llm_connector::{LlmClient, types::{ChatRequest, Message}};
#[cfg(feature = "streaming")]
use futures_util::StreamExt;

#[cfg(feature = "streaming")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Anthropic Streaming Response Example\n");

    // Need Anthropic API key
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .unwrap_or_else(|_| {
            println!("❌ Please set the ANTHROPIC_API_KEY environment variable");
            std::process::exit(1);
        });

    // Create Anthropic client
    let client = LlmClient::anthropic(&api_key).unwrap();

    // 1. Regular chat request
    println!("💬 Regular Chat Request:");
    let request = ChatRequest {
        model: "claude-3-5-sonnet-20241022".to_string(),
        messages: vec![
            Message::user("Please briefly introduce the advantages of streaming responses.")
        ],
        max_tokens: Some(200),
        ..Default::default()
    };

    match client.chat(&request).await {
        Ok(response) => {
            println!("   Claude's reply: {}\n", response.choices[0].message.content_as_text());
        }
        Err(e) => {
            println!("   ❌ Chat error: {}\n", e);
        }
    }

    // 2. Streaming chat request
    println!("🌊 Streaming Chat Request:");
    println!("   Claude is streaming a response...");

    match client.chat_stream(&request).await {
        Ok(mut stream) => {
            print!("   ");
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(streaming_response) => {
                        if let Some(content) = streaming_response.get_content() {
                            print!("{}", content);
                            // Force flush output buffer for real-time display
                            use std::io::{self, Write};
                            io::stdout().flush().unwrap();
                        }

                        // Check if completed
                        if let Some(finish_reason) = streaming_response.choices.first()
                            .and_then(|c| c.finish_reason.as_ref()) {
                            if finish_reason == "stop" {
                                println!("\n\n   ✅ Streaming response completed!");
                                if let Some(usage) = streaming_response.usage {
                                    println!("   📊 Usage Statistics:");
                                    println!("     Input tokens: {}", usage.prompt_tokens);
                                    println!("     Output tokens: {}", usage.completion_tokens);
                                    println!("     Total tokens: {}", usage.total_tokens);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("\n   ❌ Streaming response error: {}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            println!("   ❌ Streaming request error: {}", e);
        }
    }

    println!("\n✅ Example completed!");
    println!("\n💡 Tips:");
    println!("   - Streaming responses provide better user experience with real-time content display");
    println!("   - Especially suitable for long text generation and interactive applications");
    println!("   - The new Anthropic streaming implementation correctly handles complex event states");

    Ok(())
}

#[cfg(not(feature = "streaming"))]
fn main() {
    println!("❌ The 'streaming' feature needs to be enabled to run this example");
    println!("   Please use: cargo run --example anthropic_streaming --features streaming");
}