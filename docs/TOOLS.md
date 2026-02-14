# Function Calling / Tools Guide

llm-connector supports OpenAI-compatible function calling (tools) with both streaming and non-streaming modes.

## Basic Usage

```rust
use llm_connector::{LlmClient, types::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::zhipu("your-api-key")?;

    // Define tools
    let tools = vec![Tool {
        tool_type: "function".to_string(),
        function: Function {
            name: "get_weather".to_string(),
            description: Some("Get weather information for a city".to_string()),
            parameters: serde_json::json!({
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
        },
    }];

    let request = ChatRequest {
        model: "glm-4-plus".to_string(),
        messages: vec![Message::text(Role::User, "What's the weather in Beijing?")],
        tools: Some(tools),
        tool_choice: Some(ToolChoice::auto()),
        ..Default::default()
    };

    let response = client.chat(&request).await?;

    // Check if tools were called
    if let Some(choice) = response.choices.first() {
        if let Some(tool_calls) = &choice.message.tool_calls {
            for call in tool_calls {
                println!("Tool: {}", call.function.name);
                println!("Arguments: {}", call.function.arguments);
            }
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

See `examples/zhipu_multiround_tools.rs` for a complete example.

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

