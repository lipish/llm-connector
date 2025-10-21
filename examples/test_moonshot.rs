//! Moonshot（月之暗面）API 测试示例

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量获取 API key
    let api_key = std::env::var("MOONSHOT_API_KEY")
        .expect("MOONSHOT_API_KEY environment variable not set");

    println!("🧪 测试 Moonshot API");
    println!("{}", "=".repeat(80));

    // 创建客户端
    let client = LlmClient::moonshot(&api_key)?;

    println!("\n📝 测试 1: 非流式响应");
    println!("{}", "-".repeat(80));

    // 创建请求
    let request = ChatRequest {
        model: "moonshot-v1-8k".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "你好，请用一句话介绍你自己".to_string(),
            ..Default::default()
        }],
        max_tokens: Some(100),
        ..Default::default()
    };

    println!("\n📤 发送请求:");
    println!("   Model: {}", request.model);
    println!("   Message: {}", request.messages[0].content);

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

    // 测试流式响应
    #[cfg(feature = "streaming")]
    {
        println!("\n\n📝 测试 2: 流式响应");
        println!("{}", "-".repeat(80));

        let request = ChatRequest {
            model: "moonshot-v1-8k".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: "用一句话介绍北京".to_string(),
                ..Default::default()
            }],
            stream: Some(true),
            max_tokens: Some(100),
            ..Default::default()
        };

        println!("\n📤 发送流式请求:");
        println!("   Model: {}", request.model);
        println!("   Message: {}", request.messages[0].content);
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
                println!("   完整内容: {}", full_content);
            }
            Err(e) => {
                eprintln!("\n❌ 错误: {}", e);
                return Err(e.into());
            }
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("✅ Moonshot API 测试完成！");
    println!("{}", "=".repeat(80));

    Ok(())
}

