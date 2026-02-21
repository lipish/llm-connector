# Getting Started

## Requirements

- Rust 1.85+ (Rust 2024 edition)

## Install

```toml
[dependencies]
llm-connector = "0.6.1"
tokio = { version = "1", features = ["full"] }
```

## First Request

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

let client = LlmClient::openai("sk-...")?;

let request = ChatRequest {
    model: "gpt-4".to_string(),
    messages: vec![Message::text(Role::User, "Hello!")],
    ..Default::default()
};

let response = client.chat(&request).await?;
println!("{}", response.content);
```

## Next

- [Providers](/guide/providers)
- [Streaming](/guide/streaming)
- [Tools](/guide/tools)
- [Multi-modal](/guide/multimodal)
