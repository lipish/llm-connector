use futures_util::StreamExt;
use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ 需要启用 'streaming' 功能才能运行此示例");
        println!("   请使用: cargo run --example debug_zhipu_streaming --features streaming");
        return Ok(());
    }

    #[cfg(feature = "streaming")]
    {
    // 从环境变量读取 API Key
    let api_key = std::env::var("ZHIPU_API_KEY")
        .expect("请设置环境变量 ZHIPU_API_KEY");

    println!("🔍 调试 Zhipu 流式响应示例");
    println!("API Key: {}...", &api_key[..8.min(api_key.len())]);

    // 使用 Zhipu 协议
    let client = LlmClient::zhipu(&api_key);
    println!("客户端协议: {}", client.protocol_name());

    let request = ChatRequest {
        model: "glm-4-flash".to_string(), // 使用更快的模型进行测试
        messages: vec![Message::user("说一个字")],
        max_tokens: Some(10),
        ..Default::default()
    };

    println!("\n🚀 开始流式请求...");
    println!("模型: {}", request.model);
    println!("消息: {}", request.messages[0].content);

    match client.chat_stream(&request).await {
        Ok(mut stream) => {
            println!("\n✅ 流式响应成功建立！");
            let mut chunk_count = 0;

            while let Some(item) = stream.next().await {
                chunk_count += 1;
                match item {
                    Ok(chunk) => {
                        println!("\n📦 块 {}:", chunk_count);
                        println!("  ID: {}", chunk.id);
                        println!("  模型: {}", chunk.model);
                        println!("  对象: {}", chunk.object);
                        println!("  选择数量: {}", chunk.choices.len());

                        if let Some(content) = chunk.get_content() {
                            println!("  内容: '{}'", content);
                        } else {
                            println!("  内容: (无)");
                        }

                        if let Some(usage) = chunk.usage {
                            println!("  使用量: {:?}", usage);
                        }

                        for (i, choice) in chunk.choices.iter().enumerate() {
                            println!("  选择 {}: finish_reason={:?}, delta.role={:?}, delta.content={:?}",
                                i, choice.finish_reason, choice.delta.role, choice.delta.content);
                        }
                    }
                    Err(e) => {
                        println!("❌ 块 {} 错误: {}", chunk_count, e);
                        break;
                    }
                }

                // 限制块数量，避免无限循环
                if chunk_count > 10 {
                    println!("⚠️  达到最大块数量限制，停止处理");
                    break;
                }
            }

            println!("\n🏁 总共处理了 {} 个块", chunk_count);
        }
        Err(e) => {
            println!("❌ 流式请求失败: {}", e);

            // 尝试非流式请求作为对比
            println!("\n🔄 尝试非流式请求作为对比...");
            match client.chat(&request).await {
                Ok(response) => {
                    println!("✅ 非流式请求成功:");
                    println!("  响应: {}", response.choices[0].message.content);
                    println!("  使用量: {:?}", response.usage);
                }
                Err(e2) => {
                    println!("❌ 非流式请求也失败: {}", e2);
                }
            }
        }
    }

        Ok(())
    } // end of #[cfg(feature = "streaming")]
}