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
        let api_key = std::env::var("DEEPSEEK_API_KEY")
            .expect("请设置环境变量 DEEPSEEK_API_KEY");
        
        let client = LlmClient::openai_with_base_url(&api_key, "https://api.deepseek.com")?;
        
        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "get_weather".to_string(),
                description: Some("Get the current weather in a given location".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "The city name, e.g. San Francisco"
                        },
                        "unit": {
                            "type": "string",
                            "enum": ["celsius", "fahrenheit"]
                        }
                    },
                    "required": ["location"]
                }),
            },
        }];
        
        println!("🧪 测试 DeepSeek 工具调用\n");
        
        println!("📝 第一轮请求（非流式，确认工具调用支持）");
        
        let request1 = ChatRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: "Call the get_weather function for San Francisco".to_string(),
                ..Default::default()
            }],
            tools: Some(tools.clone()),
            tool_choice: Some(llm_connector::types::ToolChoice::required()),
            ..Default::default()
        };
        
        let response1 = client.chat(&request1).await?;
        
        println!("\n📥 响应:");
        println!("  - finish_reason: {:?}", response1.choices.first().and_then(|c| c.finish_reason.as_ref()));
        println!("  - content: {}", response1.content);
        
        if let Some(choice) = response1.choices.first() {
            if let Some(tool_calls) = &choice.message.tool_calls {
                println!("  - ✅ 工具调用: {} 个", tool_calls.len());
                for call in tool_calls {
                    println!("    - 函数: {}", call.function.name);
                    println!("      参数: {}", call.function.arguments);
                }
                
                println!("\n📝 第二轮请求（流式，包含 Role::Tool 结果）");
                
                let first_call = &tool_calls[0];
                
                let request2 = ChatRequest {
                    model: "deepseek-chat".to_string(),
                    messages: vec![
                        Message {
                            role: Role::User,
                            content: "Call the get_weather function for San Francisco".to_string(),
                            ..Default::default()
                        },
                        Message {
                            role: Role::Assistant,
                            content: String::new(),
                            tool_calls: Some(vec![first_call.clone()]),
                            ..Default::default()
                        },
                        Message {
                            role: Role::Tool,
                            content: r#"{"location":"San Francisco","temperature":18,"unit":"celsius","condition":"sunny","humidity":65}"#.to_string(),
                            tool_call_id: Some(first_call.id.clone()),
                            name: Some(first_call.function.name.clone()),
                            ..Default::default()
                        },
                    ],
                    tools: Some(tools),
                    ..Default::default()
                };
                
                println!("\n📨 流式响应:");
                let mut stream = client.chat_stream(&request2).await?;
                let mut chunk_count = 0;
                let mut content = String::new();
                
                while let Some(chunk) = stream.next().await {
                    chunk_count += 1;
                    match chunk {
                        Ok(response) => {
                            if let Some(delta_content) = response.get_content() {
                                print!("{}", delta_content);
                                content.push_str(delta_content);
                            }
                            
                            if let Some(choice) = response.choices.first() {
                                if let Some(reason) = &choice.finish_reason {
                                    println!("\n\n✅ finish_reason: {}", reason);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("\n❌ 错误: {}", e);
                            break;
                        }
                    }
                }
                
                println!("\n📊 统计:");
                println!("  - 收到 {} 个流式块", chunk_count);
                println!("  - 内容长度: {} 字符", content.len());
                
                if chunk_count > 1 {
                    println!("\n✅ DeepSeek 支持在包含 Role::Tool 时使用流式！");
                } else if chunk_count == 1 {
                    println!("\n⚠️ 只收到 1 个块（可能被强制切换为非流式）");
                } else {
                    println!("\n❌ 未收到响应");
                }
                
            } else {
                println!("  - ❌ 未触发工具调用");
                println!("\n💡 DeepSeek 可能不支持工具调用，或者需要特殊的配置");
            }
        }
    }
    
    Ok(())
}
