#[cfg(feature = "streaming")]
use {
    futures_util::StreamExt,
    llm_connector::{LlmClient, types::{ChatRequest, Message, Role, Tool, Function}},
    serde_json::json,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ 需要启用 'streaming' 功能");
        return Ok(());
    }
    
    #[cfg(feature = "streaming")]
    {
        let api_key = std::env::var("OPENAI_API_KEY")
            .expect("请设置环境变量 OPENAI_API_KEY");

        println!("🔧 使用 OpenAI");
        let client = LlmClient::openai(&api_key)?;

        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "get_weather".to_string(),
                description: Some("Get weather information for a city".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string", "description": "City name"},
                        "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}
                    },
                    "required": ["location"]
                }),
            },
        }];

        let model = "gpt-4o-mini";
        
        println!("\n{}", "=".repeat(70));
        println!("🧪 测试 OpenAI 工具调用流式支持");
        println!("{}\n", "=".repeat(70));
        
        println!("📝 第一轮请求（触发工具调用）");
        
        let request = ChatRequest {
            model: model.to_string(),
            messages: vec![Message::text(Role::User, "What's the weather in Shanghai?")],
            tools: Some(tools.clone()),
            ..Default::default()
        };
        
        let mut stream = client.chat_stream(&request).await?;
        let mut tool_calls_buffer = Vec::new();
        let mut chunk_count = 0;
        
        while let Some(chunk) = stream.next().await {
            chunk_count += 1;
            match chunk {
                Ok(response) => {
                    if let Some(content) = response.get_content() {
                        print!("{}", content);
                    }
                    
                    if let Some(choice) = response.choices.first() {
                        if let Some(calls) = &choice.delta.tool_calls {
                            tool_calls_buffer.extend(calls.clone());
                        }
                        
                        if let Some(reason) = &choice.finish_reason {
                            println!("\n✅ finish_reason: {}", reason);
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ 错误: {}", e);
                    break;
                }
            }
        }
        
        println!("\n📊 第一轮统计:");
        println!("  - 收到 {} 个流式块", chunk_count);
        println!("  - 工具调用: {}", if !tool_calls_buffer.is_empty() { "✅ 有" } else { "❌ 无" });
        
        if !tool_calls_buffer.is_empty() {
            println!("\n📝 第二轮请求（包含 tool 结果）");

            let first_call = tool_calls_buffer[0].clone();
            
            let request2 = ChatRequest {
                model: model.to_string(),
                messages: vec![
                    Message::text(Role::User, "What's the weather in Shanghai?"),
                    Message {
                        role: Role::Assistant,
                        content: String::new(),
                        tool_calls: Some(vec![first_call.clone()]),
                        ..Default::default()
                    },
                    Message {
                        role: Role::Tool,
                        content: r#"{"temperature": 22, "condition": "sunny"}"#.to_string(),
                        tool_call_id: Some(first_call.id.clone()),
                        name: Some(first_call.function.name.clone()),
                        ..Default::default()
                    },
                ],
                tools: Some(tools),
                ..Default::default()
            };
            
            println!("\n📨 流式响应:");
            let mut stream2 = client.chat_stream(&request2).await?;
            let mut chunk_count2 = 0;
            
            while let Some(chunk) = stream2.next().await {
                chunk_count2 += 1;
                match chunk {
                    Ok(response) => {
                        if let Some(content) = response.get_content() {
                            print!("{}", content);
                        }
                        
                        if let Some(choice) = response.choices.first() {
                            if let Some(reason) = &choice.finish_reason {
                                println!("\n✅ finish_reason: {}", reason);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ 错误: {}", e);
                        break;
                    }
                }
            }
            
            println!("\n📊 第二轮统计:");
            println!("  - 收到 {} 个流式块", chunk_count2);
            
            if chunk_count2 > 1 {
                println!("\n✅ OpenAI **支持**在包含 Role::Tool 时使用流式！");
            } else if chunk_count2 == 1 {
                println!("\n⚠️ 只收到 1 个块（可能被自动切换为非流式）");
            } else {
                println!("\n❌ 未收到任何响应");
            }
        }
    }
    
    Ok(())
}
