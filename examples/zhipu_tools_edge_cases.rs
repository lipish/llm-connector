use llm_connector::{LlmClient, types::{ChatRequest, Message, MessageBlock, Role, Tool, Function}};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("ZHIPU_API_KEY")
        .expect("请设置环境变量 ZHIPU_API_KEY");
    
    let client = LlmClient::zhipu(&api_key)?;
    
    let tools = vec![
        Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "search_web".to_string(),
                description: Some("搜索网络".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"}
                    },
                    "required": ["query"]
                }),
            },
        },
        Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "get_weather".to_string(),
                description: Some("获取天气".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    },
                    "required": ["location"]
                }),
            },
        },
    ];
    
    println!("🧪 测试边缘情况\n");
    
    // === 测试1：多个工具调用 ===
    println!("📋 测试1：请求需要多个工具");
    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: vec![Message::text(Role::User, "请先搜索今天的新闻，然后查询北京的天气")],
        tools: Some(tools.clone()),
        ..Default::default()
    };
    
    let response = client.chat(&request).await?;
    
    if let Some(choice) = response.choices.first() {
        if let Some(tool_calls) = &choice.message.tool_calls {
            println!("  ✅ 返回 {} 个工具调用", tool_calls.len());
            for call in tool_calls {
                println!("    - {}: {}", call.function.name, call.function.arguments);
            }
        } else {
            println!("  ℹ️  返回文本: {}", response.content);
        }
    }
    
    // === 测试2：三轮对话 ===
    println!("\n📋 测试2：三轮工具调用对话");
    let mut messages = vec![Message::text(Role::User, "帮我查询上海的天气")];
    
    // 第一轮
    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: messages.clone(),
        tools: Some(tools.clone()),
        ..Default::default()
    };
    
    let response1 = client.chat(&request).await?;
    println!("  轮次1: finish_reason={:?}", response1.choices.first().and_then(|c| c.finish_reason.as_ref()));
    
    if let Some(choice) = response1.choices.first() {
        if let Some(tool_calls) = &choice.message.tool_calls {
            // 添加 assistant 和 tool 消息
            messages.push(Message {
                role: Role::Assistant,
                content: vec![],
                tool_calls: Some(tool_calls.clone()),
                ..Default::default()
            });

            messages.push(Message {
                role: Role::Tool,
                content: vec![MessageBlock::text(json!({"temperature": "20°C", "condition": "多云"}).to_string())],
                tool_call_id: Some(tool_calls[0].id.clone()),
                name: Some("get_weather".to_string()),
                ..Default::default()
            });
            
            // 第二轮
            let request2 = ChatRequest {
                model: "glm-4-flash".to_string(),
                messages: messages.clone(),
                tools: Some(tools.clone()),
                ..Default::default()
            };
            
            let response2 = client.chat(&request2).await?;
            println!("  轮次2: finish_reason={:?}", response2.choices.first().and_then(|c| c.finish_reason.as_ref()));
            println!("  轮次2: content={}", response2.content);
            
            // 继续追问
            messages.push(Message::text(Role::Assistant, &response2.content));
            
            messages.push(Message::text(Role::User, "那北京呢？"));
            
            // 第三轮
            let request3 = ChatRequest {
                model: "glm-4-flash".to_string(),
                messages: messages.clone(),
                tools: Some(tools),
                ..Default::default()
            };
            
            let response3 = client.chat(&request3).await?;
            println!("  轮次3: finish_reason={:?}", response3.choices.first().and_then(|c| c.finish_reason.as_ref()));
            
            if let Some(choice) = response3.choices.first() {
                if choice.message.tool_calls.is_some() {
                    println!("  轮次3: ✅ 正确触发新的工具调用");
                } else {
                    println!("  轮次3: content={}", response3.content);
                }
            }
        }
    }
    
    println!("\n✅ 所有边缘情况测试完成");
    
    Ok(())
}
