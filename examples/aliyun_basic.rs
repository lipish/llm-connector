//! 阿里云通义千问基础示例
//!
//! 展示如何使用阿里云DashScope API进行基本的聊天对话
//!
//! 运行方式: cargo run --example aliyun_basic

use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 阿里云通义千问基础聊天示例\n");

    // 从环境变量获取API密钥
    let api_key = std::env::var("DASHSCOPE_API_KEY")
        .unwrap_or_else(|_| {
            println!("❌ 请设置环境变量 DASHSCOPE_API_KEY");
            println!("   export DASHSCOPE_API_KEY=your-api-key");
            println!("   获取API密钥: https://dashscope.console.aliyun.com/");
            std::process::exit(1);
        });

    // 创建阿里云客户端
    let client = LlmClient::aliyun(&api_key).unwrap();

    // 构建聊天请求
    let request = ChatRequest {
        model: "qwen-turbo".to_string(),
        messages: vec![
            Message::user("请简要介绍一下阿里云通义千问大模型的特点。")
        ],
        max_tokens: Some(200),
        temperature: Some(0.7),
        ..Default::default()
    };

    println!("🚀 发送请求到阿里云DashScope...");
    println!("📝 模型: {}", request.model);
    println!("💬 消息: {}", request.messages[0].content_as_text());
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
            println!("  1. DASHSCOPE_API_KEY 是否正确设置");
            println!("  2. 网络连接是否正常");
            println!("  3. API密钥是否有效");
            println!("  4. 账户余额是否充足");
        }
    }

    Ok(())
}
