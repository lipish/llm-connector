//! Ollama 流式响应示例
//!
//! 展示如何在本地 Ollama 下使用流式聊天输出。

#[cfg(feature = "streaming")]
use llm_connector::{LlmClient, types::{ChatRequest, Message}};
#[cfg(feature = "streaming")]
use futures_util::StreamExt;

#[cfg(feature = "streaming")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦙 Ollama 流式响应示例\n");

    // 创建 Ollama 客户端（默认 http://localhost:11434）
    let client = LlmClient::ollama(None);

    // 准备请求（确保模型已安装，如 llama3.2）
    let request = ChatRequest {
        model: "llama3.2".to_string(),
        messages: vec![
            Message::user("请用中文简要说明流式输出的优势。"),
        ],
        max_tokens: Some(128),
        ..Default::default()
    };

    println!("🌊 开始流式回复...\n");
    match client.chat_stream(&request).await {
        Ok(mut stream) => {
            print!("   ");
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(sr) => {
                        if let Some(content) = sr.get_content() {
                            print!("{}", content);
                            use std::io::{self, Write};
                            io::stdout().flush().unwrap();
                        }

                        if let Some(reason) = sr.choices.first().and_then(|c| c.finish_reason.as_ref()) {
                            if reason == "stop" {
                                println!("\n\n✅ 流式完成");
                            }
                        }
                    }
                    Err(e) => {
                        println!("\n❌ 错误: {}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ 启动流式失败: {}", e);
            println!("💡 请确保 Ollama 正在运行，且模型已安装，例如: 'ollama pull llama3.2' ");
        }
    }

    Ok(())
}

#[cfg(not(feature = "streaming"))]
fn main() {
    println!("❌ 需要启用 'streaming' 功能: cargo run --example ollama_streaming --features streaming");
}