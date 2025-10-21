use llm_connector::{LlmClient, types::{ChatRequest, Message, Role, Tool, Function}};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("ZHIPU_API_KEY")
        .expect("请设置环境变量 ZHIPU_API_KEY");
    
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
                        "description": "城市名称，例如：北京、上海"
                    },
                    "unit": {
                        "type": "string",
                        "enum": ["celsius", "fahrenheit"],
                        "description": "温度单位"
                    }
                },
                "required": ["location"]
            }),
        },
    }];
    
    // 使用更明确的提示词，引导模型使用工具
    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: vec![Message::text(Role::User, "请使用 get_weather 函数查询北京的天气")],
        tools: Some(tools),
        ..Default::default()
    };
    
    println!("🧪 测试智谱 tools 支持（明确要求使用工具）\n");
    
    println!("📤 请求信息:");
    println!("  - model: {}", request.model);
    println!("  - 提示词: {}", request.messages[0].content_as_text()_as_text());
    println!("  - tools 数量: {}\n", request.tools.as_ref().map(|t| t.len()).unwrap_or(0));
    
    let response = client.chat(&request).await?;
    
    println!("📥 响应信息:");
    println!("  - content: {}", response.content);
    println!("  - finish_reason: {:?}", response.choices.first().and_then(|c| c.finish_reason.as_ref()));
    
    if let Some(choice) = response.choices.first() {
        if let Some(tool_calls) = &choice.message.tool_calls {
            println!("\n✅ 成功触发工具调用！");
            for (i, call) in tool_calls.iter().enumerate() {
                println!("\n  工具调用 #{}:", i + 1);
                println!("  - ID: {}", call.id);
                println!("  - 类型: {}", call.call_type);
                println!("  - 函数: {}", call.function.name);
                println!("  - 参数: {}", call.function.arguments);
                
                // 解析参数验证
                if let Ok(args) = serde_json::from_str::<serde_json::Value>(&call.function.arguments) {
                    println!("  - 解析后的参数:");
                    println!("{}", serde_json::to_string_pretty(&args)?);
                }
            }
        } else {
            println!("\n⚠️  未触发工具调用");
            println!("  finish_reason: {:?}", choice.finish_reason);
        }
    }
    
    Ok(())
}
