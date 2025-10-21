//! OpenAI基础示例
//!
//! 展示如何使用OpenAI协议进行基本的聊天对话
//!
//! 运行方式: cargo run --example openai_basic

use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 OpenAI基础聊天示例\n");

    // 从环境变量获取API密钥
    let api_key = std::env::var("OPENAI_API_KEY")
        .unwrap_or_else(|_| {
            println!("❌ 请设置环境变量 OPENAI_API_KEY");
            println!("   export OPENAI_API_KEY=your-api-key");
            std::process::exit(1);
        });

    // 创建OpenAI客户端
    let client = LlmClient::openai(&api_key)?;

    // 构建聊天请求
    let request = ChatRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![
            Message::user("请简要介绍一下Rust编程语言的特点。")
        ],
        max_tokens: Some(200),
        temperature: Some(0.7),
        ..Default::default()
    };

    println!("🚀 发送请求到OpenAI...");
    println!("📝 模型: {}", request.model);
    println!("💬 消息: {}", request.messages[0].content_as_text()_as_text());
    println!();

    // 发送请求
    match client.chat(&request).await {
        Ok(response) => {
            println!("✅ 成功收到回复:");
            println!("{}", response.content);
            println!();
            println!("📊 Token使用情况:");
            println!("  输入: {} tokens", response.prompt_tokens());
            println!("  输出: {} tokens", response.completion_tokens());
            println!("  总计: {} tokens", response.total_tokens());
        }
        Err(e) => {
            println!("❌ 请求失败: {}", e);
            println!();
            println!("💡 请检查:");
            println!("  1. OPENAI_API_KEY 是否正确设置");
            println!("  2. 网络连接是否正常");
            println!("  3. API密钥是否有效");
        }
    }

    Ok(())
}
