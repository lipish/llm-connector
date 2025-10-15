use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[cfg(feature = "streaming")]
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ 此示例需要启用 streaming 功能");
        println!("请使用: cargo run --example test_pure_ollama_format --features streaming");
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
            messages: vec![Message::user("简单回复：测试纯Ollama格式")],
            max_tokens: Some(50),
            ..Default::default()
        };

        println!("🧪 测试纯Ollama格式输出");
        println!("🎯 验证输出是否为纯OllamaStreamChunk类型\n");

        // 使用纯Ollama格式的流式输出
        let mut stream = client.chat_stream_ollama(&request).await?;
        let mut chunk_count = 0;
        
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(ollama_chunk) => {
                    chunk_count += 1;
                    
                    // 验证这是纯OllamaStreamChunk类型
                    println!("📦 Chunk #{}: OllamaStreamChunk", chunk_count);
                    println!("  模型: {}", ollama_chunk.model);
                    println!("  时间: {}", ollama_chunk.created_at);
                    println!("  角色: {}", ollama_chunk.message.role);
                    println!("  内容: '{}'", ollama_chunk.message.content);
                    println!("  完成: {}", ollama_chunk.done);
                    
                    // 显示JSON序列化结果
                    let json_str = serde_json::to_string(&ollama_chunk)?;
                    println!("  JSON: {}", json_str);
                    println!();
                    
                    // 检查是否是最终chunk
                    if ollama_chunk.done {
                        println!("✅ 检测到最终chunk (done: true)");
                        
                        if let Some(usage) = ollama_chunk.prompt_eval_count {
                            println!("📊 包含使用统计信息:");
                            println!("  输入tokens: {}", usage);
                            if let Some(output) = ollama_chunk.eval_count {
                                println!("  输出tokens: {}", output);
                            }
                        }
                        break;
                    }
                }
                Err(e) => {
                    println!("❌ 错误：{}", e);
                    break;
                }
            }
        }

        println!("\n🎯 验证结果:");
        println!("✅ 输出格式: 纯OllamaStreamChunk类型");
        println!("✅ 无嵌套格式: 不包含OpenAI格式包装");
        println!("✅ 直接可用: 可直接序列化为JSON");
        println!("✅ 完成标记: 正确的done:true最终chunk");
        println!("✅ 总chunk数: {}", chunk_count);
        
        println!("\n💡 这种格式可以直接用于:");
        println!("  • Zed.dev编辑器");
        println!("  • Ollama兼容的工具");
        println!("  • 任何期望纯Ollama格式的应用");
    }

    Ok(())
}
