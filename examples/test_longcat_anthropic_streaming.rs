//! 测试 LongCat Anthropic 接口流式功能
//!
//! LongCat 提供了 Anthropic 兼容的接口，可以用来测试 Anthropic 流式实现

#[cfg(feature = "streaming")]
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
#[cfg(feature = "streaming")]
use futures_util::StreamExt;

#[cfg(feature = "streaming")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试 LongCat Anthropic 接口流式功能\n");
    println!("{}", "=".repeat(80));

    // LongCat API key
    let api_key = "ak_11o3bI6O03mx2yS8jb2hD61q7DJ4d";
    
    // 创建 LongCat Anthropic 客户端
    println!("\n📡 创建 LongCat Anthropic 客户端...");
    let client = LlmClient::longcat_anthropic(api_key)?;
    println!("✅ 客户端创建成功");

    // 测试请求 - 使用 LongCat 支持的模型名
    let request = ChatRequest {
        model: "LongCat-Flash-Chat".to_string(),
        messages: vec![
            Message::text(Role::User, "请用一句话介绍一下流式响应的优势。")
        ],
        max_tokens: Some(200),
        ..Default::default()
    };

    println!("\n📝 请求信息:");
    println!("  - Model: {}", request.model);
    println!("  - Message: {}", request.messages[0].content_as_text());
    println!("  - Max tokens: {:?}", request.max_tokens);

    // 测试 1: 非流式请求
    println!();
    println!("{}", "=".repeat(80));
    println!("测试 1: 非流式请求");
    println!("{}", "=".repeat(80));
    
    match client.chat(&request).await {
        Ok(response) => {
            println!("\n✅ 非流式请求成功！");
            println!("\n📄 响应内容:");
            println!("{}", response.content);
            
            if let Some(usage) = response.usage {
                println!("\n📊 Token 使用:");
                println!("  - 输入: {}", usage.prompt_tokens);
                println!("  - 输出: {}", usage.completion_tokens);
                println!("  - 总计: {}", usage.total_tokens);
            }
        }
        Err(e) => {
            println!("\n❌ 非流式请求失败: {}", e);
            println!("   错误详情: {:?}", e);
        }
    }

    // 测试 2: 流式请求
    println!();
    println!("{}", "=".repeat(80));
    println!("测试 2: 流式请求");
    println!("{}", "=".repeat(80));
    
    match client.chat_stream(&request).await {
        Ok(mut stream) => {
            println!("\n✅ 流式请求启动成功！");
            println!("\n🌊 流式响应内容:");
            print!("   ");
            
            let mut chunk_count = 0;
            let mut total_content = String::new();
            let mut final_usage = None;
            
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(streaming_response) => {
                        chunk_count += 1;
                        
                        // 提取内容
                        if let Some(content) = streaming_response.get_content() {
                            print!("{}", content);
                            total_content.push_str(&content);
                            
                            // 强制刷新输出
                            use std::io::{self, Write};
                            io::stdout().flush().unwrap();
                        }
                        
                        // 检查是否完成
                        if let Some(choice) = streaming_response.choices.first() {
                            if let Some(finish_reason) = &choice.finish_reason {
                                println!("\n\n   ✅ 流式响应完成！");
                                println!("   Finish reason: {}", finish_reason);
                            }
                        }
                        
                        // 保存 usage 信息
                        if streaming_response.usage.is_some() {
                            final_usage = streaming_response.usage;
                        }
                    }
                    Err(e) => {
                        println!("\n\n   ❌ 流式响应错误: {}", e);
                        println!("   错误详情: {:?}", e);
                        break;
                    }
                }
            }
            
            println!("\n📊 流式统计:");
            println!("  - 收到的 chunk 数量: {}", chunk_count);
            println!("  - 总内容长度: {} 字符", total_content.len());
            
            if let Some(usage) = final_usage {
                println!("\n📊 Token 使用:");
                println!("  - 输入: {}", usage.prompt_tokens);
                println!("  - 输出: {}", usage.completion_tokens);
                println!("  - 总计: {}", usage.total_tokens);
            }
        }
        Err(e) => {
            println!("\n❌ 流式请求失败: {}", e);
            println!("   错误详情: {:?}", e);
        }
    }

    // 测试 3: 长文本流式请求
    println!();
    println!("{}", "=".repeat(80));
    println!("测试 3: 长文本流式请求");
    println!("{}", "=".repeat(80));
    
    let long_request = ChatRequest {
        model: "LongCat-Flash-Chat".to_string(),
        messages: vec![
            Message::text(Role::User, "请详细介绍一下 Rust 语言的所有权系统，包括借用、生命周期等概念。")
        ],
        max_tokens: Some(500),
        ..Default::default()
    };
    
    match client.chat_stream(&long_request).await {
        Ok(mut stream) => {
            println!("\n✅ 长文本流式请求启动成功！");
            println!("\n🌊 流式响应内容:");
            print!("   ");
            
            let mut chunk_count = 0;
            
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(streaming_response) => {
                        chunk_count += 1;
                        
                        if let Some(content) = streaming_response.get_content() {
                            print!("{}", content);
                            use std::io::{self, Write};
                            io::stdout().flush().unwrap();
                        }
                        
                        if let Some(choice) = streaming_response.choices.first() {
                            if choice.finish_reason.is_some() {
                                println!("\n\n   ✅ 长文本流式响应完成！");
                                println!("   收到的 chunk 数量: {}", chunk_count);
                                
                                if let Some(usage) = streaming_response.usage {
                                    println!("\n📊 Token 使用:");
                                    println!("  - 输入: {}", usage.prompt_tokens);
                                    println!("  - 输出: {}", usage.completion_tokens);
                                    println!("  - 总计: {}", usage.total_tokens);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("\n\n   ❌ 错误: {}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            println!("\n❌ 长文本流式请求失败: {}", e);
        }
    }

    // 总结
    println!();
    println!("{}", "=".repeat(80));
    println!("测试总结");
    println!("{}", "=".repeat(80));
    println!("\n✅ LongCat Anthropic 接口流式功能测试完成！");
    println!("\n💡 测试结果:");
    println!("  - 非流式请求: 查看上面的结果");
    println!("  - 流式请求: 查看上面的结果");
    println!("  - 长文本流式: 查看上面的结果");
    println!("\n📝 注意事项:");
    println!("  - LongCat 使用 Anthropic 兼容接口");
    println!("  - 流式响应应该实时显示内容");
    println!("  - 最后一个 chunk 应该包含 finish_reason 和 usage");

    Ok(())
}

#[cfg(not(feature = "streaming"))]
fn main() {
    println!("❌ 需要启用 'streaming' 功能才能运行此示例");
    println!("   请使用: cargo run --example test_longcat_anthropic_streaming --features streaming");
}

