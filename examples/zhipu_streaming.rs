#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ 需要启用 'streaming' 功能才能运行此示例");
        println!("   请使用: cargo run --example zhipu_streaming --features streaming");
        return Ok(());
    }

    #[cfg(feature = "streaming")]
    {
        use futures_util::StreamExt;
        use llm_connector::{
            types::{ChatRequest, Message, Role},
            LlmClient,
        };
        // 从环境变量读取 API Key
        let api_key = std::env::var("ZHIPU_API_KEY").expect("请设置环境变量 ZHIPU_API_KEY");

        // 使用 Zhipu 协议（默认端点）
        let client = LlmClient::zhipu(&api_key)?;

        let request = ChatRequest {
            model: "glm-4-flash".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: "请简要说明流式响应的好处。".to_string(),
                ..Default::default()
            }],
            max_tokens: Some(128),
            ..Default::default()
        };

        println!("🚀 开始 Zhipu 流式响应示例 (model=glm-4-flash)\n");
        
        // 添加调试信息
        println!("📡 使用智谱专用流式解析器 (单换行分隔)");
        println!("   标准 SSE: data: {{...}}\\n\\n");
        println!("   智谱格式: data: {{...}}\\n\n");
        
        let mut stream = client.chat_stream(&request).await?;

        let mut full_text = String::new();
        let mut chunk_count = 0;
        
        while let Some(item) = stream.next().await {
            match item {
                Ok(chunk) => {
                    chunk_count += 1;
                    
                    if let Some(content) = chunk.get_content() {
                        print!("{}", content);
                        full_text.push_str(content);
                        use std::io::{self, Write};
                        io::stdout().flush().ok();
                    }

                    if let Some(fr) = chunk
                        .choices
                        .first()
                        .and_then(|c| c.finish_reason.as_deref())
                    {
                        if fr == "stop" {
                            println!("\n\n✅ 流式响应完成！");
                            if let Some(usage) = chunk.usage {
                                println!(
                                    "📊 使用统计: prompt={}, completion={}, total={}",
                                    usage.prompt_tokens,
                                    usage.completion_tokens,
                                    usage.total_tokens
                                );
                            }
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\n❌ 流式响应错误: {}", e);
                    break;
                }
            }
        }

        println!("\n\n📝 完整文本:\n{}", full_text);
        println!("\n📊 总字符数: {}", full_text.len());
        println!("📦 收到数据块: {} 个", chunk_count);
        
        if chunk_count == 0 {
            eprintln!("\n⚠️  警告: 没有收到任何数据块！");
            eprintln!("   这可能是流式解析器的问题。");
        }
        
        Ok(())
    } // end of #[cfg(feature = "streaming")]
}