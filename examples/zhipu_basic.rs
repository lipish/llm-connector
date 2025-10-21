use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Zhipu OpenAI 兼容端点，缺省为官方地址
    let api_key = std::env::var("ZHIPU_API_KEY")
        .expect("请设置环境变量 ZHIPU_API_KEY");

    let client = LlmClient::zhipu(&api_key)?;

    let model = std::env::var("ZHIPU_MODEL").unwrap_or_else(|_| "glm-4.5".to_string());
    let request = ChatRequest {
        model,
        messages: vec![Message::text(Role::User, "请简要说明流式响应的好处。")],
        max_tokens: Some(128),
        ..Default::default()
    };

    println!("🚀 Zhipu 非流式连接测试 (model={})\n", request.model);
    match client.chat(&request).await {
        Ok(resp) => {
            println!("✅ 成功，输出：\n{}", resp.content);
            if let Some(usage) = resp.usage {
                println!("\n📊 Token 使用情况:");
                println!("  输入 tokens: {}", usage.prompt_tokens);
                println!("  输出 tokens: {}", usage.completion_tokens);
                println!("  总计 tokens: {}", usage.total_tokens);
            }
        }
        Err(e) => {
            println!("❌ 失败：{}", e);
        }
    }

    Ok(())
}