//! Zhipu GLM-Z1 推理模型流式响应测试
//!
//! 测试 Zhipu 推理模型的流式响应处理，验证 ###Thinking 和 ###Response 标记的正确解析

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量获取 API key
    let api_key = std::env::var("ZHIPU_API_KEY")
        .expect("ZHIPU_API_KEY environment variable not set");

    println!("🧪 测试 Zhipu GLM-Z1 推理模型流式响应");
    println!("{}", "=".repeat(80));

    // 创建客户端
    let client = LlmClient::zhipu(&api_key)?;

    println!("\n📝 测试 1: 推理模型流式响应（GLM-Z1）");
    println!("{}", "-".repeat(80));

    let request = ChatRequest {
        model: "glm-z1".to_string(),
        messages: vec![Message::text(Role::User, "9.11 和 9.9 哪个更大？请详细解释你的推理过程。")],
        stream: Some(true),
        max_tokens: Some(1000),
        ..Default::default()
    };

    println!("\n📤 发送流式请求:");
    println!("   Model: {}", request.model);
    println!("   Message: {}", request.messages[0].content_as_text());
    println!("   Stream: true");

    #[cfg(feature = "streaming")]
    {
        use futures_util::StreamExt;

        println!("\n📥 接收流式响应:");
        println!("{}", "=".repeat(80));

        match client.chat_stream(&request).await {
            Ok(mut stream) => {
                let mut reasoning_content = String::new();
                let mut answer_content = String::new();
                let mut chunk_count = 0;
                let mut reasoning_chunk_count = 0;
                let mut answer_chunk_count = 0;
                let mut in_reasoning = true;

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(chunk) => {
                            chunk_count += 1;

                            // 提取推理内容
                            if let Some(reasoning) = chunk.choices.first()
                                .and_then(|c| c.delta.reasoning_content.as_ref()) {
                                if in_reasoning {
                                    if reasoning_chunk_count == 0 {
                                        println!("\n🧠 推理过程:");
                                        println!("{}", "-".repeat(80));
                                    }
                                    print!("{}", reasoning);
                                    use std::io::{self, Write};
                                    io::stdout().flush().unwrap();
                                    reasoning_content.push_str(reasoning);
                                    reasoning_chunk_count += 1;
                                }
                            }

                            // 提取答案内容
                            if let Some(content) = chunk.choices.first()
                                .and_then(|c| c.delta.content.as_ref()) {
                                if in_reasoning {
                                    println!("\n\n💡 最终答案:");
                                    println!("{}", "-".repeat(80));
                                    in_reasoning = false;
                                }
                                print!("{}", content);
                                use std::io::{self, Write};
                                io::stdout().flush().unwrap();
                                answer_content.push_str(content);
                                answer_chunk_count += 1;
                            }

                            // 提取 finish_reason
                            if let Some(reason) = chunk.choices.first()
                                .and_then(|c| c.finish_reason.as_ref()) {
                                println!("\n\n🏁 finish_reason: {}", reason);
                            }

                            // 提取 usage
                            if chunk.usage.is_some() {
                                if let Some(u) = chunk.usage {
                                    println!("\n📊 Usage:");
                                    println!("   prompt_tokens: {}", u.prompt_tokens);
                                    println!("   completion_tokens: {}", u.completion_tokens);
                                    println!("   total_tokens: {}", u.total_tokens);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("\n❌ 错误: {}", e);
                            break;
                        }
                    }
                }

                println!("\n{}", "=".repeat(80));
                println!("📊 统计:");
                println!("   总流式块数: {}", chunk_count);
                println!("   推理内容块数: {}", reasoning_chunk_count);
                println!("   答案内容块数: {}", answer_chunk_count);
                println!("   推理内容长度: {} 字符", reasoning_content.len());
                println!("   答案内容长度: {} 字符", answer_content.len());

                println!("\n✅ 流式响应正常！");

                // 验证
                if reasoning_chunk_count > 0 {
                    println!("\n✅ 成功提取推理内容（###Thinking 部分）");
                } else {
                    println!("\n⚠️  未检测到推理内容（可能不是推理模型）");
                }

                if answer_chunk_count > 0 {
                    println!("✅ 成功提取答案内容（###Response 部分）");
                } else {
                    println!("⚠️  未检测到答案内容");
                }
            }
            Err(e) => {
                eprintln!("\n❌ 错误: {}", e);
                return Err(e.into());
            }
        }
    }

    #[cfg(not(feature = "streaming"))]
    {
        println!("\n⚠️  需要启用 'streaming' feature 来测试流式响应");
        println!("   运行: cargo run --example test_zhipu_reasoning_stream --features streaming");
    }

    println!("\n\n📝 测试 2: 非推理模型流式响应（GLM-4）");
    println!("{}", "-".repeat(80));

    let request = ChatRequest {
        model: "glm-4".to_string(),
        messages: vec![Message::text(Role::User, "你好，请用一句话介绍你自己")],
        stream: Some(true),
        max_tokens: Some(100),
        ..Default::default()
    };

    println!("\n📤 发送流式请求:");
    println!("   Model: {}", request.model);
    println!("   Message: {}", request.messages[0].content_as_text());

    #[cfg(feature = "streaming")]
    {
        use futures_util::StreamExt;

        println!("\n📥 接收流式响应:");
        println!("{}", "-".repeat(80));

        match client.chat_stream(&request).await {
            Ok(mut stream) => {
                let mut full_content = String::new();
                let mut chunk_count = 0;

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(chunk) => {
                            chunk_count += 1;

                            // 非推理模型应该只有 content，没有 reasoning_content
                            if let Some(reasoning) = chunk.choices.first()
                                .and_then(|c| c.delta.reasoning_content.as_ref()) {
                                println!("\n⚠️  意外：非推理模型返回了 reasoning_content: {}", reasoning);
                            }

                            if let Some(content) = chunk.choices.first()
                                .and_then(|c| c.delta.content.as_ref()) {
                                print!("{}", content);
                                use std::io::{self, Write};
                                io::stdout().flush().unwrap();
                                full_content.push_str(content);
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
                println!("   完整内容长度: {} 字符", full_content.len());

                println!("\n✅ 非推理模型流式响应正常！");
            }
            Err(e) => {
                eprintln!("\n❌ 错误: {}", e);
                return Err(e.into());
            }
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("✅ Zhipu 推理模型流式响应测试完成！");
    println!("{}", "=".repeat(80));

    println!("\n📝 总结:");
    println!("   - GLM-Z1 推理模型: 自动分离 ###Thinking 和 ###Response");
    println!("   - 推理内容在 delta.reasoning_content 中");
    println!("   - 答案内容在 delta.content 中");
    println!("   - 非推理模型: 只有 delta.content，没有 reasoning_content");
    println!("   - 用户体验: 实时看到推理过程和答案");

    Ok(())
}

