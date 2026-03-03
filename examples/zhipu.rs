//! Zhipu GLM Example (V2)
//!
//! Demonstrates basic chat and streaming using Zhipu GLM.
//!
//! Run: cargo run --example zhipu

use dotenvy::dotenv;
use llm_connector::{
    LlmClient,
    types::{ChatRequest, Message},
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("🤖 Zhipu GLM Comprehensive Example\n");

    let api_key = env::var("ZHIPU_API_KEY").expect("ZHIPU_API_KEY not set");
    let base_url = env::var("ZHIPU_BASE_URL").unwrap_or_else(|_| "https://open.bigmodel.cn/api/paas/v4".to_string());

    let client = LlmClient::zhipu(&api_key, &base_url)?;

    println!("--- 1. Basic Chat ---");
    let request = ChatRequest::new("glm-4")
        .add_message(Message::user("Introduce yourself in a few words."));
    
    let response = client.chat(&request).await?;
    println!("Response: {}\n", response.content);

    #[cfg(feature = "streaming")]
    {
        println!("--- 2. Streaming Chat ---");
        let request = ChatRequest::new("glm-4")
            .add_message(Message::user("Tell a very short story about a robot learning to cook."))
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

    Ok(())
}
