use llm_connector::{
    types::{ChatRequest, Function, Message, Role, Tool},
    LlmClient,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("ZHIPU_API_KEY").expect("请设置环境变量 ZHIPU_API_KEY");

    let client = LlmClient::zhipu(&api_key)?;

    // 定义工具
    let tools = vec![Tool {
        tool_type: "function".to_string(),
        function: Function {
            name: "get_weather".to_string(),
            description: Some("获取指定城市的天气信息".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "城市名称"
                    }
                },
                "required": ["location"]
            }),
        },
    }];

    println!("🧪 测试智谱多轮工具调用\n");

    // === 第一轮：用户提问 ===
    let mut messages = vec![Message::text(Role::User, "请使用 get_weather 函数查询北京的天气")];

    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: messages.clone(),
        tools: Some(tools.clone()),
        ..Default::default()
    };

    println!("📤 第一轮：用户提问");
    println!("  消息数量: {}", request.messages.len());

    let response = client.chat(&request).await?;

    println!("\n📥 第一轮：LLM 响应");
    println!(
        "  finish_reason: {:?}",
        response
            .choices
            .first()
            .and_then(|c| c.finish_reason.as_ref())
    );

    if let Some(choice) = response.choices.first() {
        if let Some(tool_calls) = &choice.message.tool_calls {
            println!("  ✅ 触发工具调用: {} 个", tool_calls.len());
            for call in tool_calls {
                println!("    - 函数: {}", call.function.name);
                println!("      参数: {}", call.function.arguments);
            }

            // === 第二轮：添加 assistant 消息和 tool 消息 ===

            // 添加 assistant 的工具调用消息
            messages.push(Message {
                role: Role::Assistant,
                content: String::new(),
                tool_calls: Some(tool_calls.clone()),
                ..Default::default()
            });

            // 添加 tool 执行结果消息
            for call in tool_calls {
                messages.push(Message {
                    role: Role::Tool,
                    content: json!({
                        "location": "北京",
                        "temperature": "15°C",
                        "condition": "晴天"
                    })
                    .to_string(),
                    tool_call_id: Some(call.id.clone()),
                    name: Some(call.function.name.clone()),
                    ..Default::default()
                });
            }

            println!("\n📤 第二轮：发送工具执行结果");
            println!("  消息数量: {}", messages.len());
            println!("  消息历史:");
            for (i, msg) in messages.iter().enumerate() {
                println!(
                    "    [{}] role={:?}, content={}, tool_calls={}, tool_call_id={:?}",
                    i,
                    msg.role,
                    if msg.content.len() > 50 {
                        format!("{}...", &msg.content[..50])
                    } else {
                        msg.content.clone()
                    },
                    msg.tool_calls.as_ref().map(|t| t.len()).unwrap_or(0),
                    msg.tool_call_id
                );
            }

            let request2 = ChatRequest {
                model: "glm-4-flash".to_string(),
                messages: messages.clone(),
                tools: Some(tools),
                ..Default::default()
            };

            let response2 = client.chat(&request2).await?;

            println!("\n📥 第二轮：LLM 最终响应");
            println!(
                "  finish_reason: {:?}",
                response2
                    .choices
                    .first()
                    .and_then(|c| c.finish_reason.as_ref())
            );
            println!("  content: {}", response2.content);

            if let Some(choice) = response2.choices.first() {
                if choice.message.tool_calls.is_some() {
                    println!("  ❌ 仍然返回工具调用（应该返回文本）");
                } else {
                    println!("  ✅ 返回文本响应（正确）");
                }
            }
        } else {
            println!("  ❌ 未触发工具调用");
        }
    }

    Ok(())
}
