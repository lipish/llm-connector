//! Ollama Streaming Response Example
//!
//! Demonstrates how to use streaming chat output with a local Ollama instance.

#[cfg(feature = "streaming")]
use futures_util::StreamExt;
#[cfg(feature = "streaming")]
use llm_connector::{
    LlmClient,
    types::{ChatRequest, Message},
};

#[cfg(feature = "streaming")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦙 Ollama Streaming Response Example\n");

    // Create Ollama client (default http://localhost:11434)
    let client = LlmClient::ollama().unwrap();

    // Prepare request (ensure the model is installed, e.g. llama3.2)
    let request = ChatRequest {
        model: "llama3.2".to_string(),
        messages: vec![Message::user(
            "Please briefly explain the benefits of streaming output.",
        )],
        max_tokens: Some(128),
        ..Default::default()
    };

    println!("🌊 Starting streaming response...\n");
    match client.chat_stream(&request).await {
        Ok(mut stream) => {
            print!("   ");
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(sr) => {
                        if let Some(content) = sr.get_content() {
                            print!("{}", content);
                            use std::io::{self, Write};
                            io::stdout().flush().unwrap();
                        }

                        if let Some(reason) =
                            sr.choices.first().and_then(|c| c.finish_reason.as_ref())
                            && reason == "stop"
                        {
                            println!("\n\n✅ Streaming completed");
                        }
                    }
                    Err(e) => {
                        println!("\n❌ Error: {}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to start streaming: {}", e);
            println!(
                "💡 Please ensure Ollama is running and the model is installed, e.g.: 'ollama pull llama3.2'"
            );
        }
    }

    Ok(())
}

#[cfg(not(feature = "streaming"))]
fn main() {
    println!(
        "❌ The 'streaming' feature must be enabled: cargo run --example ollama_streaming --features streaming"
    );
}
