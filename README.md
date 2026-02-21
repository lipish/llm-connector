<div align="center">

<h1>llm-connector</h1>

[![crates.io](https://img.shields.io/crates/v/llm-connector.svg)](https://crates.io/crates/llm-connector)
[![build](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/lipish/llm-connector)
[![downloads](https://img.shields.io/crates/d/llm-connector.svg)](https://crates.io/crates/llm-connector)
[![msrv](https://img.shields.io/badge/MSRV-1.85%2B-blue.svg)](https://www.rust-lang.org)
[![docs.rs](https://img.shields.io/docsrs/llm-connector)](https://docs.rs/llm-connector)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

High-performance Rust library for unifying LLM providers behind one type-safe API.

[Installation](#installation) • [Usage](#usage) • [Documentation](#documentation) • [Contributing](https://llmconn.com/guide/contributing) • [Providers](https://llmconn.com/guide/providers)

</div>

## Philosophy

- One API, many providers.
- Unified response types for chat + streaming.
- Minimal configuration: explicit `base_url`, `timeout`, `proxy`.

## Key Features

- Provider-agnostic client API (`LlmClient`)
- Universal streaming with a unified `StreamingResponse`
- Function calling / tools (OpenAI-compatible)
- Multi-modal messages (text + images)
- Reasoning model normalization

## Installation

**MSRV**: Rust 1.85+ (Rust 2024 edition)

```toml
[dependencies]
llm-connector = "0.6.1"
tokio = { version = "1", features = ["full"] }
```

## Usage

### Chat

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

### Streaming

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

## Documentation

- [Providers](https://llmconn.com/guide/providers)
- [Streaming](https://llmconn.com/guide/streaming)
- [Tools / Function Calling](https://llmconn.com/guide/tools)
- [Multi-modal](https://llmconn.com/guide/multimodal)
- [Architecture](https://llmconn.com/guide/architecture)
- [Changelog](CHANGELOG.md)

## Contributing

Contributions welcome! See https://llmconn.com/guide/contributing for guidelines.

## License

MIT
