/// 测试腾讯云混元（Tencent Hunyuan）API
/// 
/// 测试非流式和流式响应

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("TENCENT_API_KEY")
        .unwrap_or_else(|_| "sk-YMiR2Q7LNWVKVWKivkfPn49geQXT27OZXumFkSS3Ef6FlQ50".to_string());
    
    println!("🧪 测试腾讯云混元（Tencent Hunyuan）API");
    println!("{}", "=".repeat(80));
    
    // 创建腾讯云混元客户端
    // 使用专用的 tencent() 方法，更简洁方便
    let client = LlmClient::tencent(&api_key)?;
    
    // 测试 1: 非流式响应
    println!("\n📝 测试 1: 非流式响应");
    println!("{}", "-".repeat(80));
    
    let request = ChatRequest {
        model: "hunyuan-lite".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "你好，请用一句话介绍你自己".to_string(),
            ..Default::default()
        }],
        max_tokens: Some(1000),
        temperature: Some(0.7),
        ..Default::default()
    };
    
    println!("\n📤 发送请求:");
    println!("   Endpoint: https://api.hunyuan.cloud.tencent.com/v1");
    println!("   Model: {}", request.model);
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
            println!("\n提示:");
            println!("   1. 请确认 API Key 是否正确");
            println!("   2. 请确认模型名称是否正确");
            println!("   3. 可用模型: hunyuan-lite, hunyuan-standard, hunyuan-pro 等");
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
        streaming_request.messages = vec![Message {
            role: Role::User,
            content: "用一句话介绍北京".to_string(),
            ..Default::default()
        }];
        streaming_request.stream = Some(true);
        
        println!("\n📤 发送流式请求:");
        println!("   Model: {}", streaming_request.model);
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
                return Err(e.into());
            }
        }
    }
    
    #[cfg(not(feature = "streaming"))]
    {
        println!("\n\n⚠️  流式测试跳过（需要 --features streaming）");
    }
    
    println!("\n{}", "=".repeat(80));
    println!("✅ 腾讯云混元 API 测试完成！");
    println!("\n说明:");
    println!("   腾讯云混元使用 OpenAI 兼容的 API 格式");
    println!("   端点: https://api.hunyuan.cloud.tencent.com/v1");
    println!("   可用模型: hunyuan-lite, hunyuan-standard, hunyuan-pro 等");
    println!("{}", "=".repeat(80));
    
    Ok(())
}

