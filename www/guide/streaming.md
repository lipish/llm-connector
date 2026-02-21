# Streaming

llm-connector provides unified streaming support across all providers.

## Basic Streaming

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

## Streaming with Reasoning Models

```rust
let mut stream = client.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;

    if let Some(content) = chunk.get_content() {
        print!("{}", content);
    }

    if let Some(reasoning) = &chunk.reasoning_content {
        println!("Reasoning: {}", reasoning);
    }
}
```

## Pure Ollama Format

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

## Examples

```bash
cargo run --example anthropic_streaming
cargo run --example ollama_streaming
cargo run --example volcengine_streaming
cargo run --example google_streaming
cargo run --example tencent_native_streaming
```
