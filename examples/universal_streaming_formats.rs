use llm_connector::{LlmClient, types::{ChatRequest, Message, StreamingConfig, StreamingFormat, StreamFormat}};

#[cfg(feature = "streaming")]
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ 此示例需要启用 streaming 功能");
        println!("请使用: cargo run --example universal_streaming_formats --features streaming");
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
            messages: vec![Message::user("简单回复：测试通用格式抽象")],
            max_tokens: Some(50),
            ..Default::default()
        };

        println!("🚀 通用流式格式抽象演示");
        println!("🎯 展示不同的流式输出格式\n");

        // 1. 演示JSON格式（默认）
        println!("📋 1. JSON格式 (默认)");
        println!("{}", "=".repeat(50));
        
        let config = StreamingConfig {
            format: StreamingFormat::OpenAI,
            stream_format: StreamFormat::Json,
            include_usage: true,
            include_reasoning: false,
        };

        let mut stream = client.chat_stream_universal(&request, &config).await?;
        let mut chunk_count = 0;
        
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    chunk_count += 1;
                    if chunk_count <= 2 {
                        println!("Chunk #{}: {}", chunk_count, chunk.to_format());
                    }
                    if let Some(content) = chunk.extract_content() {
                        if !content.is_empty() {
                            print!("{}", content);
                            std::io::Write::flush(&mut std::io::stdout()).unwrap();
                        }
                    }
                }
                Err(e) => {
                    println!("错误: {}", e);
                    break;
                }
            }
        }
        
        println!("\n{}", "=".repeat(50));
        println!();

        // 2. 演示SSE格式
        println!("📋 2. Server-Sent Events (SSE) 格式");
        println!("{}", "=".repeat(50));
        
        let mut sse_stream = client.chat_stream_sse(&request).await?;
        let mut sse_count = 0;
        
        while let Some(chunk) = sse_stream.next().await {
            match chunk {
                Ok(chunk) => {
                    sse_count += 1;
                    if sse_count <= 2 {
                        println!("SSE Chunk #{}: {}", sse_count, chunk.to_format().trim());
                    }
                    if let Some(content) = chunk.extract_content() {
                        if !content.is_empty() {
                            print!("{}", content);
                            std::io::Write::flush(&mut std::io::stdout()).unwrap();
                        }
                    }
                }
                Err(e) => {
                    println!("错误: {}", e);
                    break;
                }
            }
        }
        
        println!("\n{}", "=".repeat(50));
        println!();

        // 3. 演示NDJSON格式
        println!("📋 3. Newline-Delimited JSON (NDJSON) 格式");
        println!("{}", "=".repeat(50));
        
        let mut ndjson_stream = client.chat_stream_ndjson(&request).await?;
        let mut ndjson_count = 0;
        
        while let Some(chunk) = ndjson_stream.next().await {
            match chunk {
                Ok(chunk) => {
                    ndjson_count += 1;
                    if ndjson_count <= 2 {
                        println!("NDJSON Chunk #{}: {}", ndjson_count, chunk.to_format().trim());
                    }
                    if let Some(content) = chunk.extract_content() {
                        if !content.is_empty() {
                            print!("{}", content);
                            std::io::Write::flush(&mut std::io::stdout()).unwrap();
                        }
                    }
                }
                Err(e) => {
                    println!("错误: {}", e);
                    break;
                }
            }
        }
        
        println!("\n{}", "=".repeat(50));
        println!();

        // 4. 演示Ollama格式 + SSE
        println!("📋 4. Ollama格式 + SSE输出");
        println!("{}", "=".repeat(50));
        
        let ollama_sse_config = StreamingConfig {
            format: StreamingFormat::Ollama,
            stream_format: StreamFormat::SSE,
            include_usage: true,
            include_reasoning: false,
        };

        let mut ollama_sse_stream = client.chat_stream_universal(&request, &ollama_sse_config).await?;
        let mut ollama_count = 0;
        
        while let Some(chunk) = ollama_sse_stream.next().await {
            match chunk {
                Ok(chunk) => {
                    ollama_count += 1;
                    if ollama_count <= 2 {
                        println!("Ollama+SSE Chunk #{}: {}", ollama_count, chunk.to_format().trim());
                    }
                    if let Some(content) = chunk.extract_content() {
                        if !content.is_empty() {
                            print!("{}", content);
                            std::io::Write::flush(&mut std::io::stdout()).unwrap();
                        }
                    }
                    if chunk.is_final() {
                        println!("\n✅ 检测到最终chunk (done: true)");
                        break;
                    }
                }
                Err(e) => {
                    println!("错误: {}", e);
                    break;
                }
            }
        }
        
        println!("\n{}", "=".repeat(50));
        println!();

        // 总结
        println!("🎯 格式对比总结:");
        println!("✅ JSON格式: 纯JSON字符串，适合API调用");
        println!("✅ SSE格式: 'data: {{...}}\\n\\n'，适合Web实时流");
        println!("✅ NDJSON格式: '{{...}}\\n'，适合日志和数据管道");
        println!("✅ 通用抽象: 统一接口，灵活格式选择");
        println!();
        println!("💡 使用场景:");
        println!("  • Web应用: 使用SSE格式");
        println!("  • API服务: 使用JSON格式");
        println!("  • 数据处理: 使用NDJSON格式");
        println!("  • Ollama兼容: 使用Ollama+任意格式");
    }

    Ok(())
}
