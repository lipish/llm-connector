use llm_connector::{
    LlmClient,
    types::{ChatRequest, Function, Message, MessageBlock, Role, Tool},
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("ZHIPU_API_KEY").expect("Please set environment variable ZHIPU_API_KEY");

    let client = LlmClient::zhipu(&api_key)?;

    // Define tools
    let tools = vec![Tool {
        tool_type: "function".to_string(),
        function: Function {
            name: "get_weather".to_string(),
            description: Some("Get weather information for specified city".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "City name, e.g., New York, London"
                    }
                },
                "required": ["location"]
            }),
        },
    }];

    println!("🧪 Testing Zhipu Multi-round Tool Calling\n");

    // === Round 1: User question ===
    let mut messages = vec![Message::text(
        Role::User,
        "Please use the get_weather function to query the weather in New York",
    )];

    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: messages.clone(),
        tools: Some(tools.clone()),
        ..Default::default()
    };

    println!("📤 Round 1: User Question");
    println!("  Message count: {}", request.messages.len());

    let response = client.chat(&request).await?;

    println!("\n📥 Round 1: LLM Response");
    println!(
        "  finish_reason: {:?}",
        response
            .choices
            .first()
            .and_then(|c| c.finish_reason.as_ref())
    );

    if let Some(choice) = response.choices.first() {
        if let Some(tool_calls) = &choice.message.tool_calls {
            println!("  ✅ Triggered tool calls: {}", tool_calls.len());
            for call in tool_calls {
                println!("    - Function: {}", call.function.name);
                println!("      Arguments: {}", call.function.arguments);
            }

            // === Round 2: Add assistant message and tool message ===

            // Add assistant's tool call message
            messages.push(Message {
                role: Role::Assistant,
                content: vec![],
                tool_calls: Some(tool_calls.clone()),
                ..Default::default()
            });

            // Add tool execution result message
            for call in tool_calls {
                messages.push(Message {
                    role: Role::Tool,
                    content: vec![MessageBlock::text(
                        json!({
                            "location": "New York",
                            "temperature": "15°C",
                            "condition": "Clear"
                        })
                        .to_string(),
                    )],
                    tool_call_id: Some(call.id.clone()),
                    name: Some(call.function.name.clone()),
                    ..Default::default()
                });
            }

            println!("\n📤 Round 2: Send tool execution result");
            println!("  Message count: {}", messages.len());
            println!("  Message history:");
            for (i, msg) in messages.iter().enumerate() {
                let content_text = msg.content_as_text();
                println!(
                    "    [{}] role={:?}, content={}, tool_calls={}, tool_call_id={:?}",
                    i,
                    msg.role,
                    if content_text.len() > 50 {
                        format!("{}...", &content_text[..50])
                    } else {
                        content_text
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

            println!("\n📥 Round 2: LLM Final Response");
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
                    println!("  ❌ Still returns tool calls (should return text)");
                } else {
                    println!("  ✅ Returns text response (correct)");
                }
            }
        } else {
            println!("  ❌ No tool calls triggered");
        }
    }

    Ok(())
}
