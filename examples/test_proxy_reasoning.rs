use llm_connector::{
    LlmClient,
    types::{ChatRequest, Message},
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = "sk-2OX1Jcpm81AY9UY2TJEZYl2o1FPgD26Zip0DqXtsbRCJZjD5";
    let base_url = "http://123.129.219.111:3000/v1";

    let client = LlmClient::builder()
        .openai_compatible(api_key, "ProxyService")
        .base_url(base_url)
        .build()?;

    // Test 1: Claude Sonnet Thinking via Proxy
    println!("--- Testing Claude 4.5 Thinking via Proxy ---");
    let claude_req = ChatRequest::new("claude-sonnet-4-5-20250929-thinking")
        .add_message(Message::user("How many R's are in the word Strawberry?"));

    let claude_res = client.chat(&claude_req).await?;
    
    if let Some(reasoning) = &claude_res.reasoning_content {
        println!("\n[Claude Reasoning (Extracted)]:\n{}", reasoning);
    }
    println!("\n[Claude Response Mode] Content length: {}", claude_res.content.len());
    if claude_res.reasoning_content.is_none() {
        println!("\n[Content with inline thinking?]:\n{}", claude_res.content);
    } else {
        println!("\n[Final Answer]:\n{}", claude_res.content);
    }
    
    // Test 2: Streaming Claude Thinking 
    println!("\n--- Testing Claude 4.5 Thinking STREAMING via Proxy ---");
    let mut claude_stream = client.chat_stream(&claude_req).await?;
    
    let mut has_reasoning = false;
    print!("[Streaming output]: ");
    use tokio_stream::StreamExt;
    while let Some(chunk_res) = claude_stream.next().await {
        if let Ok(chunk) = chunk_res {
            if let Some(reasoning) = &chunk.reasoning_content {
                if !has_reasoning {
                    print!("\n[Stream Reasoning]: ");
                    has_reasoning = true;
                }
                print!("{}", reasoning);
            }
            if !chunk.content.is_empty() {
                if has_reasoning {
                    print!("\n[Stream Content]: ");
                    has_reasoning = false; // reset formatting
                }
                print!("{}", chunk.content);
            }
        }
    }
    println!();

    Ok(())
}
