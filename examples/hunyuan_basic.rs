use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 腾讯混元 OpenAI 兼容端点
    let api_key = std::env::var("HUNYUAN_API_KEY")
        .expect("请设置环境变量 HUNYUAN_API_KEY");

    let client = LlmClient::hunyuan(&api_key);

    let model = std::env::var("HUNYUAN_MODEL").unwrap_or_else(|_| "hunyuan-lite".to_string());
    let request = ChatRequest {
        model,
        messages: vec![Message::user("请简要介绍一下腾讯混元大模型的特点。")],
        max_tokens: Some(256),
        ..Default::default()
    };

    println!("🚀 腾讯混元非流式连接测试 (model={})\n", request.model);
    match client.chat(&request).await {
        Ok(resp) => {
            println!("✅ 成功，输出：\n{}", resp.choices[0].message.content);
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
