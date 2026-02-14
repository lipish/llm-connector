# Streaming Guide

llm-connector provides unified streaming support across all providers with the `streaming` feature.

## Enable Streaming

```toml
[dependencies]
llm-connector = { version = "0.5", features = ["streaming"] }
```

## Basic Streaming

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}
```

**All providers return the same `StreamingResponse` type**, making it easy to switch between providers without changing your code.

## Streaming with Reasoning Models

```rust
let mut stream = client.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;

    // Get content from the current chunk
    if let Some(content) = chunk.get_content() {
        print!("{}", content);
    }

    // Access reasoning content (for providers that support it)
    if let Some(reasoning) = &chunk.reasoning_content {
        println!("Reasoning: {}", reasoning);
    }
}
```

## Advanced Streaming Features

```rust
let mut stream = client.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;

    // Access structured data
    println!("Model: {}", chunk.model);
    println!("ID: {}", chunk.id);

    // Get content from first choice
    if let Some(content) = chunk.get_content() {
        print!("{}", content);
    }

    // Access all choices
    for choice in &chunk.choices {
        if let Some(content) = &choice.delta.content {
            print!("{}", content);
        }
    }

    // Check for completion
    if chunk.choices.iter().any(|c| c.finish_reason.is_some()) {
        println!("\nStream completed!");
        break;
    }
}
```

## Pure Ollama Format

For perfect compatibility with tools like Zed.dev:

```rust
use futures_util::StreamExt;

let mut stream = client.chat_stream_ollama(&request).await?;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if !chunk.message.content.is_empty() {
        print!("{}", chunk.message.content);
    }
    if chunk.done {
        println!("\nStreaming complete!");
        break;
    }
}
```

## Format Comparison

| Format | Output Example | Use Case |
|--------|----------------|----------|
| **JSON** | `{"content":"hello"}` | API responses |
| **SSE** | `data: {"content":"hello"}\n\n` | Web real-time streaming |
| **NDJSON** | `{"content":"hello"}\n` | Log processing |

## Provider-Specific Notes

### Anthropic
- Proper handling of `message_start`, `content_block_delta`, `message_delta`, `message_stop` events
- Real-time token usage statistics during streaming

### Tencent Hunyuan
- Native API v3 streaming support
- PascalCase SSE format parsing

### Google Gemini
- SSE via `streamGenerateContent` endpoint

## Examples

```bash
cargo run --example anthropic_streaming --features streaming
cargo run --example ollama_streaming --features streaming
cargo run --example volcengine_streaming --features streaming
cargo run --example google_streaming --features streaming
cargo run --example tencent_native_streaming --features streaming
```

