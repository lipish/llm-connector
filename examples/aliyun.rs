//! Aliyun Qwen Example (V2)
//!
//! Demonstrates basic chat, streaming, and reasoning/thinking capabilities.
//!
//! Run: cargo run --example aliyun

use dotenvy::dotenv;
use llm_connector::{
    LlmClient,
    types::{ChatRequest, Message},
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("🤖 Aliyun Qwen Comprehensive Example\n");

    let api_key = env::var("ALIYUN_API_KEY").or_else(|_| env::var("DASHSCOPE_API_KEY")).unwrap_or_else(|_| {
        println!("❌ Please set ALIYUN_API_KEY in .env or environment");
        std::process::exit(1);
    });

    let client = LlmClient::aliyun(&api_key, "https://dashscope.aliyuncs.com/api/v1")?;

    println!("--- 1. Basic Chat ---");
    let request = ChatRequest::new("qwen-max")
        .add_message(Message::user("What are the advantages of Qwen models?"));
    
    let response = client.chat(&request).await?;
    println!("Response: {}\n", response.content);

    #[cfg(feature = "streaming")]
    {
        println!("--- 2. Streaming Chat ---");
        let request = ChatRequest::new("qwen-max")
            .add_message(Message::user("Write a short poem about Rust programming."))
            .with_stream(true);
        
        let mut stream = client.chat_stream(&request).await?;
        print!("Streaming: ");
        while let Some(chunk) = futures_util::StreamExt::next(&mut stream).await {
            let chunk = chunk?;
            print!("{}", chunk.content);
            std::io::Write::flush(&mut std::io::stdout())?;
        }
        println!("\n");
    }

    println!("--- 3. Reasoning (Thinking) ---");
    let request = ChatRequest::new("qwen-plus")
        .add_message(Message::user("Which is larger, 9.11 or 9.9?"))
        .with_enable_thinking(true);
    
    let response = client.chat(&request).await?;
    if let Some(reasoning) = response.reasoning_content {
        println!("🧠 Thinking process:\n{}\n", reasoning);
    }
    println!("Final Answer: {}\n", response.content);

    Ok(())
}
