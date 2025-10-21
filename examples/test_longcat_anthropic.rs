/// 测试 LongCat Anthropic 格式 API
/// 
/// 测试非流式和流式响应

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("LONGCAT_API_KEY")
        .unwrap_or_else(|_| "ak_11o3bI6O03mx2yS8jb2hD61q7DJ4d".to_string());
    
    println!("🧪 测试 LongCat Anthropic 格式 API");
    println!("{}", "=".repeat(80));
    
    // 创建 LongCat Anthropic 客户端
    // 注意：LongCat 使用 Bearer 认证而不是标准 Anthropic 的 x-api-key
    let client = LlmClient::longcat_anthropic(&api_key)?;
    
    // 测试 1: 非流式响应
    println!("\n📝 测试 1: 非流式响应");
    println!("{}", "-".repeat(80));
    
    let request = ChatRequest {
        model: "LongCat-Flash-Chat".to_string(),
        messages: vec![Message::text(Role::User, "你好，请用一句话介绍你自己")],
        max_tokens: Some(1000),
        temperature: Some(0.7),
        ..Default::default()
    };
    
    println!("\n📤 发送请求:");
    println!("   Model: LongCat-Flash-Chat");
    println!("   Message: 你好，请用一句话介绍你自己");
    
    match client.chat(&request).await {
        Ok(response) => {
            println!("\n✅ 请求成功！");
            println!("\n📥 响应:");
            println!("   Model: {}", response.model);
            println!("   Content: {}", response.content);
            
            if let Some(usage) = &response.usage {
                println!("\n📊 Usage:");
                println!("   prompt_tokens: {}", usage.prompt_tokens);
                println!("   completion_tokens: {}", usage.completion_tokens);
                println!("   total_tokens: {}", usage.total_tokens);
            }
            
            if !response.choices.is_empty() {
                println!("\n✅ Choices 数组不为空");
                println!("   choices[0].finish_reason: {:?}", response.choices[0].finish_reason);
            }
        }
        Err(e) => {
            println!("\n❌ 请求失败: {}", e);
            println!("   错误详情: {:?}", e);
            return Err(e.into());
        }
    }
    
    // 测试 2: 流式响应
    #[cfg(feature = "streaming")]
    {
        use futures_util::StreamExt;
        
        println!("\n\n📝 测试 2: 流式响应");
        println!("{}", "-".repeat(80));
        
        let mut streaming_request = request.clone();
        streaming_request.messages = vec![Message::text(Role::User, "用一句话介绍北京")];
        streaming_request.stream = Some(true);
        
        println!("\n📤 发送流式请求:");
        println!("   Model: LongCat-Flash-Chat");
        println!("   Message: 用一句话介绍北京");
        println!("   Stream: true");
        
        match client.chat_stream(&streaming_request).await {
            Ok(mut stream) => {
                println!("\n📥 接收流式响应:");
                println!("{}", "-".repeat(80));
                
                let mut chunk_count = 0;
                let mut content_chunks = 0;
                let mut full_content = String::new();
                
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(response) => {
                            chunk_count += 1;
                            
                            if let Some(choice) = response.choices.first() {
                                if let Some(ref content) = choice.delta.content {
                                    if !content.is_empty() {
                                        content_chunks += 1;
                                        full_content.push_str(content);
                                        print!("{}", content);
                                        std::io::Write::flush(&mut std::io::stdout())?;
                                    }
                                }
                                
                                if let Some(ref reason) = choice.finish_reason {
                                    println!("\n\n🏁 finish_reason: {}", reason);
                                }
                            }
                            
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
                
                println!("\n{}", "-".repeat(80));
                println!("📊 统计:");
                println!("   总流式块数: {}", chunk_count);
                println!("   包含内容的块数: {}", content_chunks);
                println!("   完整内容长度: {} 字符", full_content.len());
                
                if content_chunks > 0 {
                    println!("\n✅ 流式响应正常！");
                    println!("   完整内容: {}", full_content);
                } else {
                    println!("\n❌ 没有收到内容块");
                }
            }
            Err(e) => {
                println!("\n❌ 流式请求失败: {}", e);
                println!("   错误详情: {:?}", e);
                return Err(e.into());
            }
        }
    }
    
    #[cfg(not(feature = "streaming"))]
    {
        println!("\n\n⚠️  流式测试跳过（需要 --features streaming）");
    }
    
    println!("\n{}", "=".repeat(80));
    println!("✅ LongCat Anthropic 格式测试完成！");
    println!("{}", "=".repeat(80));
    
    Ok(())
}

