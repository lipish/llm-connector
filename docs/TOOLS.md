# Function Calling / Tools Guide

llm-connector supports OpenAI-compatible function calling (tools) with both streaming and non-streaming modes.

## Basic Usage

```rust
use llm_connector::{LlmClient, ChatRequest, Message, Tool, ToolChoice};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::openai("your-api-key")?;

    // Define tools with convenience constructor
    let tools = vec![Tool::function(
        "get_weather",
        Some("Get weather information for a city".to_string()),
        serde_json::json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name, e.g. Beijing, Shanghai"
                },
                "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "description": "Temperature unit"
                }
            },
            "required": ["location"]
        }),
    )];

    let request = ChatRequest::new("gpt-4")
        .add_message(Message::user("What's the weather in Beijing?"))
        .with_tools(tools)
        .with_tool_choice(ToolChoice::auto());

    let response = client.chat(&request).await?;

    // Convenience methods for checking tool calls
    if response.is_tool_call() {
        for tc in response.tool_calls() {
            println!("Tool: {}", tc.function.name);
            // Parse arguments into typed struct or generic Value
            let args: serde_json::Value = tc.parse_arguments()?;
            println!("Arguments: {:?}", args);
        }
    }

    Ok(())
}
```

## Streaming Tool Calls

```rust
use futures_util::StreamExt;

let request = ChatRequest {
    model: "glm-4-plus".to_string(),
    messages: vec![Message::text(Role::User, "What's the weather?")],
    tools: Some(tools),
    stream: Some(true),
    ..Default::default()
};

let mut stream = client.chat_stream(&request).await?;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;

    if let Some(choice) = chunk.choices.first() {
        if let Some(tool_calls) = &choice.delta.tool_calls {
            for call in tool_calls {
                println!("Tool: {}", call.function.name);
            }
        }
    }
}
```

## Key Features

- **Automatic deduplication** of streaming tool_calls
- **Incremental accumulation** support
- **Compatible** with OpenAI streaming format
- Works with **Zhipu, OpenAI, Aliyun** and other compatible providers

## Multi-Round Conversations with Tools

When using tools in multi-round conversations, you need to:

1. Send the initial request with tools
2. Receive tool_calls from the model
3. Execute the tool and get results
4. Send the tool results back to continue the conversation

```rust
// After receiving tool_calls from step 2:
let tool_calls = response.tool_calls().to_vec();

// Build follow-up request with tool results
let mut messages = vec![
    Message::user("What's the weather in Beijing?"),
    Message::assistant_with_tool_calls(tool_calls.clone()),
];

// Add tool results for each call
for tc in &tool_calls {
    messages.push(Message::tool(
        r#"{"temperature": 22, "unit": "celsius"}"#,
        &tc.id,
    ));
}

let follow_up = ChatRequest::new("gpt-4")
    .with_messages(messages)
    .with_tools(tools);

let final_response = client.chat(&follow_up).await?;
```

See `examples/zhipu_multiround_tools.rs` and `examples/test_wishlist.rs` for complete examples.

## Supported Providers

| Provider | Tool Support | Streaming Tools |
|----------|-------------|-----------------|
| OpenAI | ✅ | ✅ |
| Zhipu | ✅ | ✅ |
| Aliyun | ✅ | ✅ |
| Anthropic | ✅ | ✅ |
| DeepSeek | ✅ | ✅ |

## Examples

```bash
cargo run --example zhipu_tools
cargo run --example zhipu_multiround_tools
```

For technical implementation details, see [ARCHITECTURE.md](ARCHITECTURE.md).

