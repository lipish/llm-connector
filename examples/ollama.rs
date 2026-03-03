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

    println!("--- 1. List Local Models ---");
    match client.models().await {
        Ok(models) => println!("Available models: {:?}\n", models),
        Err(e) => println!(
            "⚠️ Could not list models (Ollama might not be running): {}\n",
            e
        ),
    }

    println!("--- 2. Basic Chat ---");
    let request = ChatRequest::new("llama3").add_message(Message::user("Why is Rust so popular?"));

    match client.chat(&request).await {
        Ok(response) => println!("Response: {}\n", response.content),
        Err(e) => println!("⚠️ Chat failed (Make sure llama3 is pulled): {}\n", e),
    }

    #[cfg(feature = "streaming")]
    {
        println!("--- 3. Streaming Chat ---");
        let request = ChatRequest::new("llama3")
            .add_message(Message::user("Tell me a joke about programming."))
            .with_stream(true);

        match client.chat_stream(&request).await {
            Ok(mut stream) => {
                print!("Streaming: ");
                while let Some(chunk) = futures_util::StreamExt::next(&mut stream).await {
                    let chunk = chunk?;
                    print!("{}", chunk.content);
                    std::io::Write::flush(&mut std::io::stdout())?;
                }
                println!("\n");
            }
            Err(e) => println!("⚠️ Streaming failed: {}\n", e),
        }
    }

    Ok(())
}
