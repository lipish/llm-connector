use llm_connector::types::MessageBlock;
use llm_connector::{
    LlmClient,
    types::{ChatRequest, Message, Role},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Zhipu OpenAI-compatible endpoint, defaults to the official base URL
    let api_key =
        std::env::var("ZHIPU_API_KEY").expect("Please set environment variable ZHIPU_API_KEY");

    let client = LlmClient::zhipu(&api_key)?;

    let model = std::env::var("ZHIPU_MODEL").unwrap_or_else(|_| "glm-4v-flash".to_string());
    let request = ChatRequest {
        model,
        messages: vec![Message::new(
            Role::User,
            vec![
                MessageBlock::image_url(
                    "https://sfile.chatglm.cn/testpath/8b01b0b4-51fd-5b51-90a1-3ad8fec8b00d_0.png",
                ),
                MessageBlock::text("What's this?"),
            ],
        )],
        max_tokens: Some(128),
        ..Default::default()
    };

    println!(
        "🚀 Zhipu Non-streaming Connectivity Test (model={})\n",
        request.model
    );
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
