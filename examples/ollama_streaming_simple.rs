use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[cfg(feature = "streaming")]
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ 此示例需要启用 streaming 功能");
        println!("请使用: cargo run --example ollama_streaming_simple --features streaming");
        return Ok(());
    }

    #[cfg(feature = "streaming")]
    {
        // 使用智谱AI作为示例
        let api_key = std::env::var("ZHIPU_API_KEY")
            .expect("请设置环境变量 ZHIPU_API_KEY");

        let client = LlmClient::zhipu(&api_key);

        let request = ChatRequest {
            model: "glm-4-flash".to_string(),
            messages: vec![Message::user("你好！请简单介绍一下自己。")],
            max_tokens: Some(100),
            ..Default::default()
        };

        println!("🚀 Ollama格式流式输出示例");
        println!("🎯 这种格式与Zed.dev兼容\n");

        // 使用Ollama格式的流式输出
        let mut stream = client.chat_stream_ollama(&request).await?;
        
        println!("💬 AI回复（Ollama格式）：");
        println!("{}", "-".repeat(40));

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(response) => {
                    // response.content 包含Ollama格式的JSON字符串
                    if !response.content.is_empty() {
                        // 解析JSON并提取内容
                        if let Ok(ollama_chunk) = serde_json::from_str::<serde_json::Value>(&response.content) {
                            // 提取消息内容
                            if let Some(content) = ollama_chunk
                                .get("message")
                                .and_then(|m| m.get("content"))
                                .and_then(|c| c.as_str()) 
                            {
                                if !content.is_empty() {
                                    print!("{}", content);
                                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                                }
                            }
                            
                            // 检查是否是最终chunk
                            if ollama_chunk.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
                                println!("\n");
                                println!("{}", "-".repeat(40));
                                println!("✅ 流式输出完成");
                                
                                // 显示最终chunk的详细信息
                                if let Some(usage_info) = ollama_chunk.get("prompt_eval_count") {
                                    println!("\n📊 使用统计：");
                                    if let Some(prompt_tokens) = ollama_chunk.get("prompt_eval_count").and_then(|v| v.as_u64()) {
                                        println!("  输入tokens: {}", prompt_tokens);
                                    }
                                    if let Some(completion_tokens) = ollama_chunk.get("eval_count").and_then(|v| v.as_u64()) {
                                        println!("  输出tokens: {}", completion_tokens);
                                    }
                                    if let Some(total_duration) = ollama_chunk.get("total_duration").and_then(|v| v.as_u64()) {
                                        println!("  总耗时: {}ms", total_duration / 1_000_000);
                                    }
                                }
                                
                                println!("\n🔍 最终chunk JSON:");
                                println!("{}", serde_json::to_string_pretty(&ollama_chunk)?);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("\n❌ 流式输出错误：{}", e);
                    break;
                }
            }
        }

        println!("\n💡 说明：");
        println!("• 这种Ollama格式的输出可以直接用于Zed.dev");
        println!("• 每个chunk都是完整的JSON对象");
        println!("• 最后一个chunk包含 'done: true' 标记");
        println!("• 包含详细的使用统计信息");
    }

    Ok(())
}
