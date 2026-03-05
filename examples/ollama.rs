//! Ollama Example (V2)
//!
//! Demonstrates basic chat, streaming, and model listing using local Ollama.
//!
//! Run: cargo run --example ollama

use llm_connector::{
    LlmClient,
    types::{ChatRequest, Message},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Ollama Local Example\n");

    let client = LlmClient::ollama("http://localhost:11434")?;
    let mut selected_model = "llama3".to_string();

    println!("--- 1. List Local Models ---");
    match client.models().await {
        Ok(models) => {
            println!("Available models: {:?}", models);
            if let Some(model) = models.iter().find(|m| m.as_str() == "kimi-k2.5:cloud") {
                selected_model = model.clone();
            } else if let Some(model) = models.first() {
                selected_model = model.clone();
            }
            println!("Using model: {}\n", selected_model);
        }
        Err(e) => println!(
            "⚠️ Could not list models (Ollama might not be running): {}\n",
            e
        ),
    }

    println!("--- 2. Basic Chat ---");
    let request = ChatRequest::new(selected_model.clone())
        .add_message(Message::user("Why is Rust so popular?"))
        .with_stream(false);

    match client.chat(&request).await {
        Ok(response) => println!("Response: {}\n", response.content),
        Err(e) => println!("⚠️ Chat failed: {}\n", e),
    }

    #[cfg(feature = "streaming")]
    {
        println!("--- 3. Streaming Chat ---");
        let request = ChatRequest::new(selected_model)
            .add_message(Message::user("Tell me a joke about programming."))
            .with_stream(true);

        match client.chat_stream(&request).await {
            Ok(mut stream) => {
                print!("Streaming: ");
                while let Some(chunk) = futures_util::StreamExt::next(&mut stream).await {
                    match chunk {
                        Ok(chunk) => {
                            print!("{}", chunk.content);
                            std::io::Write::flush(&mut std::io::stdout())?;
                        }
                        Err(e) => {
                            println!("\n⚠️ Streaming chunk parse failed: {}", e);
                            break;
                        }
                    }
                }
                println!("\n");
            }
            Err(e) => println!("⚠️ Streaming failed: {}\n", e),
        }
    }

    Ok(())
}
