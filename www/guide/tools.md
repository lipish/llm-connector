# Tools / Function Calling

llm-connector supports OpenAI-compatible function calling (tools) with both streaming and non-streaming modes.

## Basic Usage

```rust
use llm_connector::{LlmClient, ChatRequest, Message, Tool, ToolChoice};

let client = LlmClient::openai("your-api-key")?;

let tools = vec![Tool::function(
    "get_weather",
    Some("Get weather information for a city".to_string()),
    serde_json::json!({
        "type": "object",
        "properties": {
            "location": {
                "type": "string",
                "description": "City name, e.g. Beijing, Shanghai"
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

if response.is_tool_call() {
    for tc in response.tool_calls() {
        let args: serde_json::Value = tc.parse_arguments()?;
        println!("{} => {:?}", tc.function.name, args);
    }
}
```

## Streaming Tool Calls

```rust
use futures_util::StreamExt;

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

## Examples

```bash
cargo run --example zhipu_tools
cargo run --example zhipu_multiround_tools
```
