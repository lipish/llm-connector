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
        
        let models = vec!["glm-4", "glm-4.5", "glm-4-flash"];
        
        for model in models {
            println!("\n{}", "=".repeat(70));
            println!("🧪 测试模型: {}", model);
            println!("{}\n", "=".repeat(70));
            
            test_first_request(&client, model, &tools).await?;
            println!("\n");
            test_with_tool_result(&client, model, &tools).await?;
        }
    }
    
    Ok(())
}

#[cfg(feature = "streaming")]
async fn test_first_request(
    client: &LlmClient, 
    model: &str, 
    tools: &[Tool]
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 第一轮请求（触发工具调用）");
    
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "上海的天气怎么样？".to_string(),
            ..Default::default()
        }],
        tools: Some(tools.to_vec()),
        ..Default::default()
    };
    
    let mut stream = client.chat_stream(&request).await?;
    let mut has_tool_calls = false;
    let mut content_received = false;
    let mut chunk_count = 0;
    
    while let Some(chunk) = stream.next().await {
        chunk_count += 1;
        match chunk {
            Ok(response) => {
                if let Some(content) = response.get_content() {
                    if !content.trim().is_empty() {
                        content_received = true;
                        print!("{}", content);
                    }
                }
                
                if let Some(choice) = response.choices.first() {
                    if choice.delta.tool_calls.is_some() {
                        has_tool_calls = true;
                    }
                    
                    if let Some(reason) = &choice.finish_reason {
                        println!("\n✅ finish_reason: {}", reason);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ 流式错误: {}", e);
                break;
            }
        }
    }
    
    println!("\n📊 统计:");
    println!("  - 收到 {} 个流式块", chunk_count);
    println!("  - 文本内容: {}", if content_received { "✅ 有" } else { "❌ 无" });
    println!("  - 工具调用: {}", if has_tool_calls { "✅ 有" } else { "❌ 无" });
    
    Ok(())
}

#[cfg(feature = "streaming")]
async fn test_with_tool_result(
    client: &LlmClient, 
    model: &str, 
    tools: &[Tool]
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 第二轮请求（包含 tool 结果）");
    
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            Message {
                role: Role::User,
                content: "上海的天气怎么样？".to_string(),
                ..Default::default()
            },
            Message {
                role: Role::Assistant,
                content: String::new(),
                tool_calls: Some(vec![llm_connector::types::ToolCall {
                    id: "call_123".to_string(),
                    call_type: "function".to_string(),
                    function: llm_connector::types::FunctionCall {
                        name: "get_weather".to_string(),
                        arguments: r#"{"location":"上海"}"#.to_string(),
                    },
                }]),
                ..Default::default()
            },
            Message {
                role: Role::Tool,
                content: r#"{"temperature": 22, "condition": "晴天"}"#.to_string(),
                tool_call_id: Some("call_123".to_string()),
                name: Some("get_weather".to_string()),
                ..Default::default()
            },
        ],
        tools: Some(tools.to_vec()),
        ..Default::default()
    };
    
    let mut stream = client.chat_stream(&request).await?;
    let mut content_received = false;
    let mut chunk_count = 0;
    
    println!("\n📨 响应:");
    while let Some(chunk) = stream.next().await {
        chunk_count += 1;
        match chunk {
            Ok(response) => {
                if let Some(content) = response.get_content() {
                    if !content.trim().is_empty() {
                        content_received = true;
                        print!("{}", content);
                    }
                }
                
                if let Some(choice) = response.choices.first() {
                    if let Some(reason) = &choice.finish_reason {
                        println!("\n✅ finish_reason: {}", reason);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ 流式错误: {}", e);
                break;
            }
        }
    }
    
    println!("\n📊 统计:");
    println!("  - 收到 {} 个流式块", chunk_count);
    println!("  - 文本内容: {}", if content_received { "✅ 有" } else { "⚠️ 无（可能被自动切换为非流式）" });
    
    if chunk_count == 1 && content_received {
        println!("\n💡 提示: 只收到 1 个块且有内容，说明自动切换到了非流式模式（这是正确的行为）");
    }
    
    Ok(())
}
