//! 调试 LongCat Anthropic 流式响应
//!
//! 需要 streaming feature: cargo run --example debug_longcat_anthropic_stream --features streaming

#[cfg(feature = "streaming")]
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
#[cfg(feature = "streaming")]
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "streaming"))]
    {
        eprintln!("❌ 此示例需要 streaming feature");
        eprintln!("   运行方式: cargo run --example debug_longcat_anthropic_stream --features streaming");
        return Ok(());
    }

    #[cfg(feature = "streaming")]
    run().await
}

#[cfg(feature = "streaming")]
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("LONGCAT_API_KEY")
        .expect("LONGCAT_API_KEY environment variable not set");

    println!("🔍 调试 LongCat Anthropic 流式响应\n");

    // 创建客户端
    let client = LlmClient::longcat_anthropic(&api_key)?;

    // 创建请求
    let request = ChatRequest {
        model: "LongCat-Flash-Chat".to_string(),
        messages: vec![Message::text(Role::User, "你好")],
        stream: Some(true),
        max_tokens: Some(50),
        ..Default::default()
    };

    println!("📤 发送流式请求...\n");

    match client.chat_stream(&request).await {
        Ok(mut stream) => {
            println!("✅ 获取到流\n");
            println!("📥 接收流式响应:\n");

            let mut chunk_count = 0;
            while let Some(result) = stream.next().await {
                chunk_count += 1;
                match result {
                    Ok(chunk) => {
                        println!("📦 Chunk #{}: {:?}", chunk_count, chunk);
                        if let Some(content) = chunk.choices.first().and_then(|c| c.delta.content.as_ref()) {
                            print!("{}", content);
                            use std::io::{self, Write};
                            io::stdout().flush().unwrap();
                        }
                    }
                    Err(e) => {
                        eprintln!("\n❌ 错误: {}", e);
                        break;
                    }
                }
            }

            println!("\n\n✅ 总共收到 {} 个块", chunk_count);
        }
        Err(e) => {
            eprintln!("❌ 创建流失败: {}", e);
        }
    }

    Ok(())
}

