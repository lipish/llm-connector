//! Aliyun Qwen Basic Example
//!
//! Demonstrates how to use Aliyun DashScope API for a basic chat conversation.
//!
//! Run: cargo run --example aliyun_basic

use llm_connector::{
    LlmClient,
    types::{ChatRequest, Message},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Aliyun Qwen Basic Chat Example\n");

    // Read API key from environment variables
    let api_key = std::env::var("DASHSCOPE_API_KEY").unwrap_or_else(|_| {
        println!("❌ Please set the DASHSCOPE_API_KEY environment variable");
        println!("   export DASHSCOPE_API_KEY=your-api-key");
        println!("   Get an API key: https://dashscope.console.aliyun.com/");
        std::process::exit(1);
    });

    // Create Aliyun client
    let client =
        LlmClient::aliyun(&api_key, llm_connector::endpoints::ALIYUN_DASHSCOPE_V1).unwrap();

    // Build chat request
    let request = ChatRequest {
        model: "qwen-turbo".to_string(),
        messages: vec![Message::user(
            "Please briefly describe the features of Aliyun Qwen large language models.",
        )],
        max_tokens: Some(200),
        temperature: Some(0.7),
        ..Default::default()
    };

    println!("🚀 Sending request to Aliyun DashScope...");
    println!("📝 Model: {}", request.model);
    println!("💬 Message: {}", request.messages[0].content_as_text());
    println!();

    // Send request
    match client.chat(&request).await {
        Ok(response) => {
            println!("✅ Received response successfully:");
            println!("{}", response.content);
            println!();
            println!("📊 Token usage:");
            println!("  Input: {} tokens", response.prompt_tokens());
            println!("  Output: {} tokens", response.completion_tokens());
            println!("  Total: {} tokens", response.total_tokens());
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
            println!();
            println!("💡 Please check:");
            println!("  1. Whether DASHSCOPE_API_KEY is set correctly");
            println!("  2. Whether your network connection is working");
            println!("  3. Whether the API key is valid");
            println!("  4. Whether your account balance/quota is sufficient");
        }
    }

    Ok(())
}
