use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 构造一个包含 tool 消息的请求
    let messages = vec![
        Message {
            role: Role::User,
            content: "查询天气".to_string(),
            ..Default::default()
        },
        Message {
            role: Role::Assistant,
            content: String::new(),
            tool_calls: Some(vec![
                llm_connector::types::ToolCall {
                    id: "call_123".to_string(),
                    call_type: "function".to_string(),
                    function: llm_connector::types::FunctionCall {
                        name: "get_weather".to_string(),
                        arguments: r#"{"location":"北京"}"#.to_string(),
                    },
                }
            ]),
            ..Default::default()
        },
        Message {
            role: Role::Tool,
            content: r#"{"temperature":"15°C"}"#.to_string(),
            tool_call_id: Some("call_123".to_string()),
            name: Some("get_weather".to_string()),
            ..Default::default()
        },
    ];
    
    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages,
        ..Default::default()
    };
    
    // 序列化请求，查看 JSON 格式
    println!("📋 序列化后的请求（前500字符）:");
    let json_str = serde_json::to_string_pretty(&request)?;
    println!("{}", &json_str[..json_str.len().min(800)]);
    
    // 检查关键字段
    let json_value: serde_json::Value = serde_json::from_str(&json_str)?;
    
    println!("\n✅ 验证关键字段:");
    
    // 检查 assistant 消息的 tool_calls
    if let Some(tool_calls) = json_value["messages"][1]["tool_calls"].as_array() {
        println!("  ✅ Assistant 消息包含 tool_calls: {} 个", tool_calls.len());
    } else {
        println!("  ❌ Assistant 消息缺少 tool_calls");
    }
    
    // 检查 tool 消息的 role
    if let Some(role) = json_value["messages"][2]["role"].as_str() {
        println!("  ✅ Tool 消息的 role: '{}'", role);
        if role == "tool" {
            println!("     ✅ role 正确序列化为 'tool'");
        } else {
            println!("     ❌ role 错误: 应该是 'tool'，实际是 '{}'", role);
        }
    }
    
    // 检查 tool 消息的 tool_call_id
    if let Some(id) = json_value["messages"][2]["tool_call_id"].as_str() {
        println!("  ✅ Tool 消息包含 tool_call_id: '{}'", id);
    } else {
        println!("  ❌ Tool 消息缺少 tool_call_id");
    }
    
    // 检查 tool 消息的 name
    if let Some(name) = json_value["messages"][2]["name"].as_str() {
        println!("  ✅ Tool 消息包含 name: '{}'", name);
    } else {
        println!("  ❌ Tool 消息缺少 name");
    }
    
    Ok(())
}
