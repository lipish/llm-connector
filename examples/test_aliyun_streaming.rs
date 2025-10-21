/// 测试阿里云 DashScope 流式响应
/// 
/// 验证修复后的流式响应功能

#[cfg(feature = "streaming")]
use {
    futures_util::StreamExt,
    llm_connector::{LlmClient, types::{ChatRequest, Message, Role}},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ 需要启用 'streaming' 功能");
        println!("运行: cargo run --example test_aliyun_streaming --features streaming");
        return Ok(());
    }
    
    #[cfg(feature = "streaming")]
    {
        let api_key = std::env::var("ALIYUN_API_KEY")
            .expect("请设置环境变量 ALIYUN_API_KEY");
        
        let client = LlmClient::aliyun(&api_key)?;
        
        println!("🧪 测试阿里云 DashScope 流式响应");
        println!("{}", "=".repeat(80));
        
        let request = ChatRequest {
            model: "qwen-turbo".to_string(),
            messages: vec![Message::text(Role::User, "用一句话介绍北京")],
            stream: Some(true),
            ..Default::default()
        };
        
        println!("\n📤 发送流式请求:");
        println!("   Model: qwen-turbo");
        println!("   Message: 用一句话介绍北京");
        println!("   Stream: true");
        
        let mut stream = client.chat_stream(&request).await?;
        let mut chunk_count = 0;
        let mut content_chunks = 0;
        let mut full_content = String::new();
        
        println!("\n📥 接收流式响应:");
        println!("{}", "-".repeat(80));
        
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(response) => {
                    chunk_count += 1;
                    
                    if let Some(choice) = response.choices.first() {
                        // 检查 delta.content
                        if let Some(ref content) = choice.delta.content {
                            if !content.is_empty() {
                                content_chunks += 1;
                                full_content.push_str(content);
                                print!("{}", content);
                                std::io::Write::flush(&mut std::io::stdout())?;
                            }
                        }
                        
                        // 检查 finish_reason
                        if let Some(ref reason) = choice.finish_reason {
                            println!("\n\n🏁 finish_reason: {}", reason);
                        }
                    }
                    
                    // 检查 usage（最后一个块）
                    if let Some(ref usage) = response.usage {
                        println!("\n📊 Usage:");
                        println!("   prompt_tokens: {}", usage.prompt_tokens);
                        println!("   completion_tokens: {}", usage.completion_tokens);
                        println!("   total_tokens: {}", usage.total_tokens);
                    }
                }
                Err(e) => {
                    println!("\n❌ 错误: {}", e);
                    break;
                }
            }
        }
        
        println!("\n{}", "=".repeat(80));
        println!("📊 统计:");
        println!("   总流式块数: {}", chunk_count);
        println!("   包含内容的块数: {}", content_chunks);
        println!("   完整内容长度: {} 字符", full_content.len());
        
        if content_chunks == 0 {
            println!("\n❌ 问题: 没有收到任何内容块！");
            println!("   这表明流式响应解析有问题。");
        } else {
            println!("\n✅ 流式响应正常！");
            println!("   完整内容: {}", full_content);
        }
        
        println!("\n{}", "=".repeat(80));
    }
    
    Ok(())
}

