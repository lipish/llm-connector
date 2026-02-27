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
- Per-request overrides: API key, base URL, and headers for multi-tenant / gateway use

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

### Per-Request Overrides (Multi-Tenant / Gateway)

Override API key, base URL, or headers per request without creating a new client:

```rust
let request = ChatRequest::new("gpt-4")
    .add_message(Message::user("Hello"))
    .with_api_key("tenant-key")
    .with_base_url("https://proxy.example.com/v1")
    .with_header("X-Trace-Id", "trace-123");

let response = client.chat(&request).await?;
```

## Advanced Features

### Reasoning & Thinking

Support for reasoning models like OpenAI o1/o3 and Claude 3.7 Sonnet.

```rust
use llm_connector::types::ReasoningEffort;

let request = ChatRequest::new("claude-3-7-sonnet-20250219")
    .add_message(Message::user("Solve this logic puzzle..."))
    .with_thinking_budget(16000) // Enable thinking with 16k token budget
    .with_max_tokens(20000);     // Ensure max_tokens > thinking_budget

let response = client.chat(&request).await?;
```

### Dynamic Service Resolution

Resolve API keys and endpoints dynamically based on model name.

```rust
use llm_connector::core::{EnvVarResolver, ServiceResolver};

let resolver = EnvVarResolver::new()
    .with_mapping("gpt", "OPENAI_API_KEY")
    .with_mapping("claude", "ANTHROPIC_API_KEY");

let target = resolver.resolve("claude-3-opus").await?;
// Use target.api_key and target.endpoint to configure your request
```

### Request Overrides (Gateway Mode)

For gateway scenarios, you can override the API Key and Base URL per request without creating a new client.

```rust
let request = ChatRequest::new("gpt-4")
    .with_api_key("sk-new-key") // Override API Key
    .with_base_url("https://my-gateway/v1"); // Override Endpoint

let response = client.chat(&request).await?;
```

### File & Image Upload

Easily upload local files (Images, PDFs) with automatic Base64 encoding and MIME type detection.

```rust
use llm_connector::types::MessageBlock;

let request = ChatRequest::new("claude-3-5-sonnet")
    .add_message(Message::user("Analyze this document"))
    .add_message_block(MessageBlock::from_file_path("report.pdf")?);

let response = client.chat(&request).await?;
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
