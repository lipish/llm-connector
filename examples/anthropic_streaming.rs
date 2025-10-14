//! Anthropic 流式响应示例
//!
//! 展示如何使用增强的 Anthropic 流式响应功能

#[cfg(feature = "streaming")]
use llm_connector::{LlmClient, types::{ChatRequest, Message}};
#[cfg(feature = "streaming")]
use futures_util::StreamExt;

#[cfg(feature = "streaming")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Anthropic 流式响应示例\n");

    // 需要 Anthropic API key
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .unwrap_or_else(|_| {
            println!("❌ 请设置 ANTHROPIC_API_KEY 环境变量");
            std::process::exit(1);
        });

    // 创建 Anthropic 客户端
    let client = LlmClient::anthropic(&api_key);

    // 1. 普通聊天请求
    println!("💬 普通聊天请求:");
    let request = ChatRequest {
        model: "claude-3-5-sonnet-20241022".to_string(),
        messages: vec![
            Message::user("请简单介绍一下流式响应的优势。")
        ],
        max_tokens: Some(200),
        ..Default::default()
    };

    match client.chat(&request).await {
        Ok(response) => {
            println!("   Claude 回复: {}\n", response.choices[0].message.content);
        }
        Err(e) => {
            println!("   ❌ 聊天错误: {}\n", e);
        }
    }

    // 2. 流式聊天请求
    println!("🌊 流式聊天请求:");
    println!("   Claude 正在流式回复...");

    match client.chat_stream(&request).await {
        Ok(mut stream) => {
            print!("   ");
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(streaming_response) => {
                        if let Some(content) = streaming_response.get_content() {
                            print!("{}", content);
                            // 强制刷新输出缓冲区，以便实时显示
                            use std::io::{self, Write};
                            io::stdout().flush().unwrap();
                        }

                        // 检查是否完成
                        if let Some(finish_reason) = streaming_response.choices.first()
                            .and_then(|c| c.finish_reason.as_ref()) {
                            if finish_reason == "stop" {
                                println!("\n\n   ✅ 流式响应完成！");
                                if let Some(usage) = streaming_response.usage {
                                    println!("   📊 使用统计:");
                                    println!("     输入令牌: {}", usage.prompt_tokens);
                                    println!("     输出令牌: {}", usage.completion_tokens);
                                    println!("     总令牌: {}", usage.total_tokens);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("\n   ❌ 流式响应错误: {}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            println!("   ❌ 流式请求错误: {}", e);
        }
    }

    println!("\n✅ 示例完成！");
    println!("\n💡 提示:");
    println!("   - 流式响应提供更好的用户体验，可以实时显示生成内容");
    println!("   - 特别适合长文本生成和交互式应用");
    println!("   - 新的 Anthropic 流式实现正确处理了复杂的事件状态");

    Ok(())
}

#[cfg(not(feature = "streaming"))]
fn main() {
    println!("❌ 需要启用 'streaming' 功能才能运行此示例");
    println!("   请使用: cargo run --example anthropic_streaming --features streaming");
}