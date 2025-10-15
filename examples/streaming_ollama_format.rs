use llm_connector::{LlmClient, types::{ChatRequest, Message, StreamingConfig, StreamingFormat}};

#[cfg(feature = "streaming")]
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ 此示例需要启用 streaming 功能");
        println!("请使用: cargo run --example streaming_ollama_format --features streaming");
        return Ok(());
    }

    #[cfg(feature = "streaming")]
    {
        // 使用智谱AI作为示例（你可以替换为其他提供商）
        let api_key = std::env::var("ZHIPU_API_KEY")
            .expect("请设置环境变量 ZHIPU_API_KEY");

        let client = LlmClient::zhipu(&api_key);

        let request = ChatRequest {
            model: "glm-4-flash".to_string(),
            messages: vec![Message::user("请简要介绍一下人工智能的发展历程。")],
            max_tokens: Some(200),
            ..Default::default()
        };

        println!("🚀 Ollama格式流式输出演示\n");
        println!("📋 对比两种格式的输出：\n");

        // 1. 演示OpenAI格式（默认）
        println!("🔹 OpenAI格式流式输出：");
        println!("{}", "=".repeat(50));
        
        let mut openai_stream = client.chat_stream(&request).await?;
        let mut openai_content = String::new();
        
        while let Some(chunk) = openai_stream.next().await {
            match chunk {
                Ok(response) => {
                    if !response.content.is_empty() {
                        print!("{}", response.content);
                        openai_content.push_str(&response.content);
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    }
                    
                    // 显示原始JSON格式（仅第一个chunk）
                    if openai_content.len() < 10 {
                        println!("\n[OpenAI JSON示例]:");
                        println!("{}", serde_json::to_string_pretty(&response)?);
                        println!();
                    }
                }
                Err(e) => {
                    println!("\n❌ OpenAI格式错误：{}", e);
                    break;
                }
            }
        }
        
        println!("\n");
        println!("{}", "=".repeat(50));
        println!();

        // 2. 演示Ollama格式
        println!("🔹 Ollama格式流式输出：");
        println!("{}", "=".repeat(50));

        // 使用便利方法
        let mut ollama_stream = client.chat_stream_ollama(&request).await?;
        let mut ollama_content = String::new();
        let mut chunk_count = 0;
        
        while let Some(chunk) = ollama_stream.next().await {
            match chunk {
                Ok(response) => {
                    chunk_count += 1;
                    
                    // response.content 现在包含Ollama格式的JSON字符串
                    if !response.content.is_empty() {
                        // 解析Ollama JSON来提取实际内容
                        if let Ok(ollama_chunk) = serde_json::from_str::<serde_json::Value>(&response.content) {
                            if let Some(content) = ollama_chunk
                                .get("message")
                                .and_then(|m| m.get("content"))
                                .and_then(|c| c.as_str()) 
                            {
                                if !content.is_empty() {
                                    print!("{}", content);
                                    ollama_content.push_str(content);
                                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                                }
                            }
                            
                            // 显示Ollama JSON格式（仅前几个chunk）
                            if chunk_count <= 2 {
                                println!("\n[Ollama JSON示例 #{}]:", chunk_count);
                                println!("{}", serde_json::to_string_pretty(&ollama_chunk)?);
                                println!();
                            }
                            
                            // 检查是否是最终chunk
                            if ollama_chunk.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
                                println!("\n[最终Ollama chunk - done: true]:");
                                println!("{}", serde_json::to_string_pretty(&ollama_chunk)?);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("\n❌ Ollama格式错误：{}", e);
                    break;
                }
            }
        }
        
        println!("\n");
        println!("{}", "=".repeat(50));
        println!();

        // 3. 演示自定义配置的流式输出
        println!("🔹 自定义配置的Ollama格式：");
        println!("{}", "=".repeat(50));

        let custom_config = StreamingConfig {
            format: StreamingFormat::Ollama,
            include_usage: true,
            include_reasoning: false,
        };

        let mut custom_stream = client.chat_stream_with_format(&request, &custom_config).await?;
        let mut custom_content = String::new();
        
        while let Some(chunk) = custom_stream.next().await {
            match chunk {
                Ok(response) => {
                    if !response.content.is_empty() {
                        if let Ok(ollama_chunk) = serde_json::from_str::<serde_json::Value>(&response.content) {
                            if let Some(content) = ollama_chunk
                                .get("message")
                                .and_then(|m| m.get("content"))
                                .and_then(|c| c.as_str()) 
                            {
                                if !content.is_empty() {
                                    print!("{}", content);
                                    custom_content.push_str(content);
                                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                                }
                            }
                            
                            // 检查最终chunk的usage信息
                            if ollama_chunk.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
                                println!("\n[包含usage信息的最终chunk]:");
                                println!("{}", serde_json::to_string_pretty(&ollama_chunk)?);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("\n❌ 自定义格式错误：{}", e);
                    break;
                }
            }
        }

        println!("\n");
        println!("{}", "=".repeat(50));
        println!("\n✅ 演示完成！");
        println!("\n📊 总结：");
        println!("• OpenAI格式：标准的choices/delta结构");
        println!("• Ollama格式：message/content结构 + done标记");
        println!("• 两种格式都包含相同的内容，只是JSON结构不同");
        println!("• Ollama格式更适合与Zed.dev等工具集成");
    }

    Ok(())
}
