use std::time::Instant;
use futures_util::StreamExt;
use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 诊断 llm-connector Ollama 集成");

    // 创建 Ollama 客户端
    let client = LlmClient::ollama(Some("http://localhost:11434"));
    println!("✅ Ollama 客户端创建成功");

    // 测试获取模型列表
    println!("\n📋 测试获取模型列表...");
    let models_start = Instant::now();
    match client.fetch_models().await {
        Ok(models) => {
            println!("✅ 模型列表获取成功 ({:?}): {:?}", models_start.elapsed(), models);
        }
        Err(e) => {
            println!("❌ 模型列表获取失败: {}", e);
        }
    }

    // 测试非流式聊天
    println!("\n💬 测试非流式聊天...");
    let non_stream_start = Instant::now();
    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: vec![Message::user("Say hello")],
        max_tokens: Some(50),
        ..Default::default()
    };

    println!("📝 发送非流式请求...");
    match client.chat(&request).await {
        Ok(response) => {
            println!("✅ 非流式响应成功 ({:?})", non_stream_start.elapsed());
            println!("📄 响应: {}", response.choices[0].message.content);
        }
        Err(e) => {
            println!("❌ 非流式响应失败: {}", e);
        }
    }

    // 测试流式聊天
    println!("\n🌊 测试流式聊天...");
    let stream_start = Instant::now();

    println!("📝 发送流式请求...");
    match client.chat_stream(&request).await {
        Ok(mut stream) => {
            println!("✅ 流式连接建立成功 ({:?})", stream_start.elapsed());
            let mut chunk_count = 0;
            let mut content = String::new();

            while let Some(chunk_result) = stream.next().await {
                chunk_count += 1;
                match chunk_result {
                    Ok(chunk) => {
                        if let Some(chunk_content) = chunk.get_content() {
                            print!("{}", chunk_content);
                            content.push_str(chunk_content);
                        }

                        if let Some(finish_reason) = chunk.choices.first()
                            .and_then(|c| c.finish_reason.as_deref()) {
                            if finish_reason == "stop" {
                                println!("\n✅ 流式响应完成 ({} 块, {:?})", chunk_count, stream_start.elapsed());
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        println!("\n❌ 流式块错误: {}", e);
                        break;
                    }
                }

                // 安全限制
                if chunk_count > 50 {
                    println!("\n⚠️ 超过最大块数限制");
                    break;
                }
            }

            println!("📄 流式完整内容: {}", content);
        }
        Err(e) => {
            println!("❌ 流式连接失败: {}", e);
        }
    }

    println!("\n🎯 诊断完成");
    Ok(())
}

#[cfg(not(feature = "streaming"))]
fn main() {
    println!("❌ 需要启用 'streaming' 功能来运行此示例");
    println!("   请使用: cargo run --example debug_ollama --features streaming");
}