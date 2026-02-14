# llm-connector

Next-generation Rust library for LLM protocol abstraction with **native multi-modal support**.

Supports **12+ providers**: OpenAI, Anthropic, Google, Aliyun, Zhipu, Ollama, Tencent, Volcengine, DeepSeek, Moonshot, Xiaomi, and more.

## Key Features

- **12+ Provider Support** - OpenAI, Anthropic, Google, Aliyun, Zhipu, Ollama, Tencent, Volcengine, DeepSeek, Moonshot, Xiaomi
- **Unified Output Format** - All providers return the same `StreamingResponse` type
- **Multi-modal Support** - Text + images in a single message ([details](docs/MULTIMODAL.md))
- **Function Calling** - OpenAI-compatible tools with streaming ([details](docs/TOOLS.md))
- **Reasoning Models** - Universal support for reasoning models ([details](docs/REASONING_MODELS_SUPPORT.md))
- **Real-time Streaming** - Universal streaming with format abstraction ([details](docs/STREAMING.md))
- **Extreme Performance** - 7,000x+ faster client creation (7µs vs 53ms)
- **Type-Safe** - Full Rust type safety with Result-based error handling

## Installation

```toml
[dependencies]
llm-connector = "0.5.17"
tokio = { version = "1", features = ["full"] }

# With streaming support
llm-connector = { version = "0.5.17", features = ["streaming"] }
```

## Quick Start

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::openai("sk-...")?;

    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message::text(Role::User, "Hello!")],
        ..Default::default()
    };

    let response = client.chat(&request).await?;
    println!("Response: {}", response.content);
    Ok(())
}
```

## Supported Providers

| Provider | Quick Start | Features |
|----------|-------------|----------|
| **OpenAI** | `LlmClient::openai("sk-...")` | Chat, Streaming, Tools, Multi-modal |
| **Anthropic** | `LlmClient::anthropic("sk-ant-...")` | Chat, Streaming, Multi-modal |
| **Google Gemini** | `LlmClient::google("key")` | Chat, Streaming |
| **Aliyun** | `LlmClient::aliyun("sk-...")` | Chat, Streaming, Qwen models |
| **Zhipu** | `LlmClient::zhipu("key")` | Chat, Streaming, Tools, GLM models |
| **Tencent** | `LlmClient::tencent("id", "key")` | Chat, Streaming, Hunyuan |
| **Volcengine** | `LlmClient::volcengine("key")` | Chat, Streaming, Reasoning |
| **DeepSeek** | `LlmClient::deepseek("sk-...")` | Chat, Streaming, Reasoning (R1) |
| **Moonshot** | `LlmClient::moonshot("sk-...")` | Chat, Streaming |
| **Xiaomi MiMo** | `LlmClient::xiaomi("sk-...")` | Chat, Streaming |
| **Ollama** | `LlmClient::ollama()` | Chat, Streaming, Local models |

📖 **Detailed documentation**: [docs/PROVIDERS.md](docs/PROVIDERS.md)

## Streaming

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
use futures_util::StreamExt;

let client = LlmClient::openai("sk-...")?;
let request = ChatRequest {
    model: "gpt-4".to_string(),
    messages: vec![Message::text(Role::User, "Tell me a story")],
    stream: Some(true),
    ..Default::default()
};

let mut stream = client.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.get_content() {
        print!("{}", content);
    }
}
```

**All providers return the same `StreamingResponse` type.** 📖 [Streaming Guide](docs/STREAMING.md)

## Function Calling / Tools

```rust
use llm_connector::{LlmClient, ChatRequest, Message, Tool, ToolChoice};

let tools = vec![Tool::function(
    "get_weather",
    Some("Get weather info".to_string()),
    serde_json::json!({
        "type": "object",
        "properties": { "location": { "type": "string" } },
        "required": ["location"]
    }),
)];

let request = ChatRequest::new("gpt-4")
    .add_message(Message::user("What's the weather in Beijing?"))
    .with_tools(tools)
    .with_tool_choice(ToolChoice::auto());

let response = client.chat(&request).await?;

// Convenience methods for tool call handling
if response.is_tool_call() {
    for tc in response.tool_calls() {
        println!("Call: {} args: {}", tc.function.name, tc.function.arguments);
        let args: serde_json::Value = tc.parse_arguments()?;
    }
}
```

📖 **Detailed documentation**: [docs/TOOLS.md](docs/TOOLS.md)

## Multi-modal Content

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, MessageBlock, Role}};

let request = ChatRequest {
    model: "gpt-4o".to_string(),
    messages: vec![
        Message::new(
            Role::User,
            vec![
                MessageBlock::text("What's in this image?"),
                MessageBlock::image_url("https://example.com/image.jpg"),
            ],
        ),
    ],
    ..Default::default()
};
```

📖 **Detailed documentation**: [docs/MULTIMODAL.md](docs/MULTIMODAL.md)

## Reasoning Models

Universal support for reasoning models (DeepSeek R1, OpenAI o1, Volcengine Doubao-Seed-Code, etc.):

```rust
let request = ChatRequest {
    model: "deepseek-reasoner".to_string(),
    messages: vec![Message::text(Role::User, "Solve: 9.11 vs 9.9?")],
    ..Default::default()
};

let response = client.chat(&request).await?;
if let Some(reasoning) = response.reasoning_content {
    println!("Thinking: {}", reasoning);
}
println!("Answer: {}", response.content);
```

📖 **Detailed documentation**: [docs/REASONING_MODELS_SUPPORT.md](docs/REASONING_MODELS_SUPPORT.md)

## Structured Output

```rust
use llm_connector::{ChatRequest, Message, ResponseFormat};

// JSON mode
let request = ChatRequest::new("gpt-4")
    .add_message(Message::user("List 3 languages as JSON array"))
    .with_response_format(ResponseFormat::json_object());

// JSON Schema mode (Structured Outputs)
let request = ChatRequest::new("gpt-4o")
    .add_message(Message::user("Extract person info"))
    .with_response_format(ResponseFormat::json_schema(
        "person",
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" }
            },
            "required": ["name", "age"]
        }),
    ));
```

## Error Handling

```rust
use llm_connector::LlmConnectorError;

match client.chat(&request).await {
    Ok(response) => println!("Response: {}", response.content),
    Err(ref e) if e.is_rate_limited() => eprintln!("Rate limited, retry later"),
    Err(ref e) if e.should_reduce_context() => eprintln!("Input too long, reduce context"),
    Err(ref e) if e.is_auth_error() => eprintln!("Auth failed: {}", e),
    Err(ref e) if e.is_retryable() => eprintln!("Transient error, can retry: {}", e),
    Err(e) => eprintln!("Fatal error: {}", e),
}
```

## Model Discovery

```rust
let client = LlmClient::openai("sk-...")?;
let models = client.models().await?;
println!("Available models: {:?}", models);
```

## Examples

```bash
# Basic examples
cargo run --example openai_basic
cargo run --example anthropic_streaming --features streaming
cargo run --example ollama_basic
cargo run --example xiaomi_basic

# Advanced examples
cargo run --example multimodal_basic
cargo run --example zhipu_tools
cargo run --example volcengine_streaming --features streaming
```

See `examples/` directory for all available examples.

## Documentation

| Document | Description |
|----------|-------------|
| [PROVIDERS.md](docs/PROVIDERS.md) | Detailed provider configuration |
| [STREAMING.md](docs/STREAMING.md) | Streaming guide |
| [TOOLS.md](docs/TOOLS.md) | Function calling / tools |
| [MULTIMODAL.md](docs/MULTIMODAL.md) | Multi-modal content |
| [REASONING_MODELS_SUPPORT.md](docs/REASONING_MODELS_SUPPORT.md) | Reasoning models |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Technical architecture |
| [CHANGELOG.md](CHANGELOG.md) | Version history |

## Contributing

Contributions welcome! See [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

## License

MIT
