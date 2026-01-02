use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Zhipu OpenAI-compatible endpoint, defaults to the official base URL
    let api_key = std::env::var("ZHIPU_API_KEY")
        .expect("Please set environment variable ZHIPU_API_KEY");

    let client = LlmClient::zhipu(&api_key)?;

    let model = std::env::var("ZHIPU_MODEL").unwrap_or_else(|_| "glm-4.5".to_string());
    let request = ChatRequest {
        model,
        messages: vec![Message::text(Role::User, "Please briefly describe the benefits of streaming responses.")],
        max_tokens: Some(128),
        ..Default::default()
    };

    println!("🚀 Zhipu Non-streaming Connectivity Test (model={})\n", request.model);
    match client.chat(&request).await {
        Ok(resp) => {
            println!("✅ Success, output:\n{}", resp.content);
            if let Some(usage) = resp.usage {
                println!("\n📊 Token usage:");
                println!("  Input tokens: {}", usage.prompt_tokens);
                println!("  Output tokens: {}", usage.completion_tokens);
                println!("  Total tokens: {}", usage.total_tokens);
            }
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
        }
    }

    Ok(())
}