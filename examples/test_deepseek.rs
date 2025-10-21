//! DeepSeek API 测试示例
//!
//! DeepSeek 支持标准对话模型和推理模型（reasoning content）

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量获取 API key
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .expect("DEEPSEEK_API_KEY environment variable not set");

    println!("🧪 测试 DeepSeek API");
    println!("{}", "=".repeat(80));

    // 创建客户端
    let client = LlmClient::deepseek(&api_key)?;

    println!("\n📝 测试 1: 标准对话模型（deepseek-chat）");
    println!("{}", "-".repeat(80));

    // 创建请求
    let request = ChatRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![Message::text(Role::User, "你好，请用一句话介绍你自己")],
        max_tokens: Some(100),
        ..Default::default()
    };

    println!("\n📤 发送请求:");
    println!("   Model: {}", request.model);
    println!("   Message: {}", request.messages[0].content_as_text()_as_text());

    // 发送请求
    match client.chat(&request).await {
        Ok(response) => {
            println!("\n✅ 请求成功！");
            println!("\n📥 响应:");
            println!("   Model: {}", response.model);
            println!("   Content: {}", response.content);

            if let Some(usage) = response.usage {
                println!("\n📊 Usage:");
                println!("   prompt_tokens: {}", usage.prompt_tokens);
                println!("   completion_tokens: {}", usage.completion_tokens);
                println!("   total_tokens: {}", usage.total_tokens);
            }

            if !response.choices.is_empty() {
                println!("\n✅ Choices 数组不为空");
                if let Some(reason) = &response.choices[0].finish_reason {
                    println!("   choices[0].finish_reason: Some(\"{}\")", reason);
                }
            }
        }
        Err(e) => {
            eprintln!("\n❌ 错误: {}", e);
            return Err(e.into());
        }
    }

    println!("\n\n📝 测试 2: 推理模型（deepseek-reasoner）");
    println!("{}", "-".repeat(80));

    let request = ChatRequest {
        model: "deepseek-reasoner".to_string(),
        messages: vec![Message::text(Role::User, "9.11 和 9.9 哪个更大？")],
        max_tokens: Some(500),
        ..Default::default()
    };

    println!("\n📤 发送请求:");
    println!("   Model: {}", request.model);
    println!("   Message: {}", request.messages[0].content_as_text()_as_text());

    match client.chat(&request).await {
        Ok(response) => {
            println!("\n✅ 请求成功！");
            println!("\n📥 响应:");
            println!("   Model: {}", response.model);
            
            // 推理内容（思考过程）
            if let Some(reasoning) = response.reasoning_content {
                println!("\n🧠 推理过程:");
                println!("{}", reasoning);
            }
            
            // 最终答案
            println!("\n💡 最终答案:");
            println!("{}", response.content);

            if let Some(usage) = response.usage {
                println!("\n📊 Usage:");
                println!("   prompt_tokens: {}", usage.prompt_tokens);
                println!("   completion_tokens: {}", usage.completion_tokens);
                println!("   total_tokens: {}", usage.total_tokens);
            }
        }
        Err(e) => {
            eprintln!("\n❌ 错误: {}", e);
            return Err(e.into());
        }
    }

    // 测试流式响应
    #[cfg(feature = "streaming")]
    {
        println!("\n\n📝 测试 3: 流式响应（deepseek-chat）");
        println!("{}", "-".repeat(80));

        let request = ChatRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![Message::text(Role::User, "用一句话介绍北京")],
            stream: Some(true),
            max_tokens: Some(100),
            ..Default::default()
        };

        println!("\n📤 发送流式请求:");
        println!("   Model: {}", request.model);
        println!("   Message: {}", request.messages[0].content_as_text()_as_text());
        println!("   Stream: true");

        println!("\n📥 接收流式响应:");
        println!("{}", "-".repeat(80));

        use futures_util::StreamExt;

        match client.chat_stream(&request).await {
            Ok(mut stream) => {
                let mut full_content = String::new();
                let mut chunk_count = 0;
                let mut content_chunk_count = 0;
                let mut finish_reason = None;
                let mut usage = None;

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(chunk) => {
                            chunk_count += 1;

                            // 提取内容
                            if let Some(content) = chunk.choices.first()
                                .and_then(|c| c.delta.content.as_ref()) {
                                print!("{}", content);
                                use std::io::{self, Write};
                                io::stdout().flush().unwrap();
                                full_content.push_str(content);
                                content_chunk_count += 1;
                            }

                            // 提取 finish_reason
                            if let Some(reason) = chunk.choices.first()
                                .and_then(|c| c.finish_reason.as_ref()) {
                                finish_reason = Some(reason.clone());
                            }

                            // 提取 usage
                            if chunk.usage.is_some() {
                                usage = chunk.usage;
                            }
                        }
                        Err(e) => {
                            eprintln!("\n❌ 错误: {}", e);
                            break;
                        }
                    }
                }

                println!("\n");

                if let Some(reason) = finish_reason {
                    println!("\n🏁 finish_reason: {}", reason);
                }

                if let Some(u) = usage {
                    println!("\n📊 Usage:");
                    println!("   prompt_tokens: {}", u.prompt_tokens);
                    println!("   completion_tokens: {}", u.completion_tokens);
                    println!("   total_tokens: {}", u.total_tokens);
                }

                println!("\n{}", "-".repeat(80));
                println!("📊 统计:");
                println!("   总流式块数: {}", chunk_count);
                println!("   包含内容的块数: {}", content_chunk_count);
                println!("   完整内容长度: {} 字符", full_content.len());

                println!("\n✅ 流式响应正常！");
            }
            Err(e) => {
                eprintln!("\n❌ 错误: {}", e);
                return Err(e.into());
            }
        }

        println!("\n\n📝 测试 4: 推理模型流式响应（deepseek-reasoner）");
        println!("{}", "-".repeat(80));

        let request = ChatRequest {
            model: "deepseek-reasoner".to_string(),
            messages: vec![Message::text(Role::User, "计算 15 * 23")],
            stream: Some(true),
            max_tokens: Some(500),
            ..Default::default()
        };

        println!("\n📤 发送流式请求:");
        println!("   Model: {}", request.model);
        println!("   Message: {}", request.messages[0].content_as_text()_as_text());

        println!("\n📥 接收流式响应:");
        println!("{}", "-".repeat(80));

        match client.chat_stream(&request).await {
            Ok(mut stream) => {
                let mut reasoning_content = String::new();
                let mut answer_content = String::new();
                let mut chunk_count = 0;

                println!("\n🧠 推理过程:");
                println!("{}", "-".repeat(80));

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(chunk) => {
                            chunk_count += 1;

                            // 提取推理内容
                            if let Some(reasoning) = chunk.choices.first()
                                .and_then(|c| c.delta.reasoning_content.as_ref()) {
                                print!("{}", reasoning);
                                use std::io::{self, Write};
                                io::stdout().flush().unwrap();
                                reasoning_content.push_str(reasoning);
                            }

                            // 提取答案内容
                            if let Some(content) = chunk.choices.first()
                                .and_then(|c| c.delta.content.as_ref()) {
                                if !answer_content.is_empty() || !content.trim().is_empty() {
                                    if answer_content.is_empty() {
                                        println!("\n\n💡 最终答案:");
                                        println!("{}", "-".repeat(80));
                                    }
                                    print!("{}", content);
                                    use std::io::{self, Write};
                                    io::stdout().flush().unwrap();
                                    answer_content.push_str(content);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("\n❌ 错误: {}", e);
                            break;
                        }
                    }
                }

                println!("\n\n{}", "-".repeat(80));
                println!("📊 统计:");
                println!("   总流式块数: {}", chunk_count);
                println!("   推理内容长度: {} 字符", reasoning_content.len());
                println!("   答案内容长度: {} 字符", answer_content.len());

                println!("\n✅ 推理模型流式响应正常！");
            }
            Err(e) => {
                eprintln!("\n❌ 错误: {}", e);
                return Err(e.into());
            }
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("✅ DeepSeek API 测试完成！");
    println!("{}", "=".repeat(80));

    Ok(())
}

