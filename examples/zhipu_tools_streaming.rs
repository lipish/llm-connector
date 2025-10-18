#[cfg(feature = "streaming")]
use futures_util::StreamExt;
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role, Tool, Function}};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ 需要启用 'streaming' 功能");
        return Ok(());
    }
    
    #[cfg(feature = "streaming")]
    {
        let api_key = std::env::var("ZHIPU_API_KEY")
            .expect("请设置环境变量 ZHIPU_API_KEY");
        
        let client = LlmClient::zhipu(&api_key)?;
        
        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "get_weather".to_string(),
                description: Some("获取指定城市的天气信息".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string", "description": "城市名称"},
                        "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}
                    },
                    "required": ["location"]
                }),
            },
        }];
        
        let request = ChatRequest {
            model: "glm-4-flash".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: "请使用 get_weather 函数查询上海的天气".to_string(),
                ..Default::default()
            }],
            tools: Some(tools),
            ..Default::default()
        };
        
        println!("🧪 测试智谱 tools 支持（流式 + 明确要求使用工具）\n");
        
        let mut stream = client.chat_stream(&request).await?;
        let mut tool_call_buffer = String::new();
        let mut has_tool_calls = false;
        
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(response) => {
                    // 检查工具调用
                    if let Some(choice) = response.choices.first() {
                        if let Some(tool_calls) = &choice.delta.tool_calls {
                            has_tool_calls = true;
                            for call in tool_calls {
                                println!("🔧 流式工具调用片段:");
                                println!("  - ID: {}", call.id);
                                println!("  - 函数: {}", call.function.name);
                                println!("  - 参数片段: {}", call.function.arguments);
                                tool_call_buffer.push_str(&call.function.arguments);
                            }
                        }
                        
                        if let Some(reason) = &choice.finish_reason {
                            println!("\n✅ 流式响应完成（finish_reason: {}）", reason);
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\n❌ 流式错误: {}", e);
                    break;
                }
            }
        }
        
        if has_tool_calls {
            println!("\n✅ 成功触发工具调用（流式）！");
            println!("\n📋 完整参数:");
            if let Ok(args) = serde_json::from_str::<serde_json::Value>(&tool_call_buffer) {
                println!("{}", serde_json::to_string_pretty(&args)?);
            } else {
                println!("{}", tool_call_buffer);
            }
        } else {
            println!("\n❌ 未触发工具调用");
        }
    }
    
    Ok(())
}
