use llm_connector::{
    LlmClient,
    types::{ChatRequest, Function, Message, Role, Tool},
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
            description: Some("Get weather information for the specified city".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "City name, e.g., New York, London"
                    },
                    "unit": {
                        "type": "string",
                        "enum": ["celsius", "fahrenheit"],
                        "description": "Temperature unit"
                    }
                },
                "required": ["location"]
            }),
        },
    }];

    // Use an explicit prompt to encourage the model to use the tool
    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: vec![Message::text(
            Role::User,
            "Please use the get_weather function to query the weather in New York",
        )],
        tools: Some(tools),
        ..Default::default()
    };

    println!("🧪 Testing Zhipu tools support (explicit tool usage request)\n");

    println!("📤 Request info:");
    println!("  - model: {}", request.model);
    println!("  - prompt: {}", request.messages[0].content_as_text());
    println!(
        "  - tools count: {}\n",
        request.tools.as_ref().map(|t| t.len()).unwrap_or(0)
    );

    let response = client.chat(&request).await?;

    println!("📥 Response info:");
    println!("  - content: {}", response.content);
    println!(
        "  - finish_reason: {:?}",
        response
            .choices
            .first()
            .and_then(|c| c.finish_reason.as_ref())
    );

    if let Some(choice) = response.choices.first() {
        if let Some(tool_calls) = &choice.message.tool_calls {
            println!("\n✅ Successfully triggered tool calls!");
            for (i, call) in tool_calls.iter().enumerate() {
                println!("\n  Tool call #{}:", i + 1);
                println!("  - ID: {}", call.id);
                println!("  - type: {}", call.call_type);
                println!("  - function: {}", call.function.name);
                println!("  - arguments: {}", call.function.arguments);

                // Parse arguments for verification
                if let Ok(args) =
                    serde_json::from_str::<serde_json::Value>(&call.function.arguments)
                {
                    println!("  - parsed arguments:");
                    println!("{}", serde_json::to_string_pretty(&args)?);
                }
            }
        } else {
            println!("\n⚠️  No tool calls were triggered");
            println!("  finish_reason: {:?}", choice.finish_reason);
        }
    }

    Ok(())
}
