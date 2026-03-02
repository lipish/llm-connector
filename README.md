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

## Library Focus

`llm-connector` is designed as a **minimalist driver-level layer** for AI providers. Its primary goal is to provide a single, ergonomic, and type-safe API for major LLM services while managing the differences between them at a lower layer.

- **Minimalist Scope**: We focus on connectivity, normalization, and performance.
- **Driver Philosophy**: We do the "SDK gymnastics" so you don't have to, providing a cumulative and stable foundation.
- **Clear Boundaries**: This library does NOT include high-level agentic frameworks, prompt management systems, or complex RAG pipelines. It is the engine that powers them.

## llm-connector vs rust-genai

Both libraries aim to simplify interacting with multiple LLM providers in Rust, but they have different core philosophies:

| Feature | `llm-connector` (This Library) | `rust-genai` |
| :--- | :--- | :--- |
| **Primary Focus** | **Minimalism & Gateway-friendly**. Highly optimized for multi-tenant SaaS and proxy services with dynamic overrides. | **Native Protocol Depth**. Aims to deeply map and standardize unique features across many long-tail providers. |
| **Gateway Support** | ✅ **Native** `Per-Request Overrides` (change API key, Base URL, headers *per request* without new clients). | ❌ Requires creating separate clients or complex routing for multi-tenant API key swapping. |
| **Function Calling** | ✅ **Standardized**. Strong, cross-provider tool calling support (including Aliyun Native) compatible with OpenAI format. | ⚠️ Limited/in-development function calling. |
| **Reasoning Models** | ✅ **Universal Extraction**. Automatically detects and extracts `reasoning_content`, `thought`, and `thinking` fields natively. | ✅ Supports `ReasoningEffort` controls for Anthropic and Gemini. |
| **Tokens & Usage** | ✅ Core total/prompt/completion tracking. **Now with prompt caching support** (Anthropic & OpenAI). | ✅ **Ultra-granular**. Deep mapping of cached tokens and provider-specific quirks (e.g., Gemini cumulative streams). |
| **Ecosystem Size** | Focused on the absolute major players (OpenAI, Anthropic, Gemini, DeepSeek, Qwen, Moonshot, Zhipu, etc.). | Massive ecosystem support (Groq, xAI, Cohere, Together, Fireworks, etc.). |

*Choose `llm-connector` if you are building an AI SaaS, a unified gateway, or need robust function calling and reasoning model extraction out-of-the-box. Choose `rust-genai` if you need broad coverage of alternative endpoints and deep native parameter tweaking (like Gemini Thinking levels).*

## Philosophy

- One API, many providers.
- Unified response types for chat + streaming.
- Minimal configuration: explicit `base_url`, `timeout`, `proxy`.

## Key Features

- Provider-agnostic client API (`LlmClient`)
- Universal streaming with a unified `StreamingResponse`
- Function calling / tools (OpenAI-compatible)
- Multi-modal messages (text + images + **PDF/Documents**)
- **Embedding API Support** (Unified Vector Embeddings)
- Reasoning model normalization
- Per-request overrides: API key, base URL, and headers for multi-tenant / gateway use

## Installation

**MSRV**: Rust 1.85+ (Rust 2024 edition)

```toml
[dependencies]
llm-connector = "0.7.1"
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

### Embedding

Generate vector embeddings across any provider:

```rust
use llm_connector::{LlmClient, types::EmbedRequest};

let client = LlmClient::openai("sk-...")?;
let request = EmbedRequest {
    model: "text-embedding-3-small".to_string(),
    input: vec!["Hello world".to_string(), "Rust is awesome".to_string()],
    ..Default::default()
};

let response = client.embed(&request).await?;
for data in response.data {
    println!("Embedding [{}]: {:?}", data.index, data.embedding);
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

## Documentation

- [Providers](https://llmconn.com/guide/providers)
- [Streaming](https://llmconn.com/guide/streaming)
- [Tools / Function Calling](https://llmconn.com/guide/tools)
- [Multi-modal](https://llmconn.com/guide/multimodal)
- [Architecture](https://llmconn.com/guide/architecture)
- [Changelog](CHANGELOG.md)

## Recent Changelogs

Highlights from the latest updates (see [CHANGELOG.md](CHANGELOG.md) for full history):

- **v0.7.1 (2026-03-02)**
    - `+` **Embedding API**: Unified `.embed()` support for OpenAI, Anthropic, Google, Ollama, Aliyun, and Zhipu.
    - `+` **Document Support**: Added `MessageBlock::Document` for multi-modal file uploads (PDFs, etc.).
    - `^` **Usage Enhancements**: Added support for `prompt_cache_hit_tokens` and detailed caching stats.
- **v0.7.0 (2026-02-23)**
    - `+` **Per-Request Overrides**: Support for `api_key`, `base_url`, and `extra_headers` per request for multi-tenant gateways.
    - `^` Better handling of custom headers for all `GenericProvider`-based models.
- **v0.6.1 (2026-02-20)**
    - `^` **Rust 2024 Edition**: MSRV is now Rust 1.85+.
    - `🔧` Switched to `rustls-tls` by default for better cross-compilation.
- **v0.6.0 (2026-02-15)**
    - `+` **Builder Pattern for LlmClient**: New `LlmClient::builder()` for fluent client construction.
    - `+` **Zhipu Multimodal**: Native support for image URLs and base64 images.
    - `!` **Streaming enabled by default**: `chat_stream` is now part of the default feature set.

## Contributing

Contributions welcome! See https://llmconn.com/guide/contributing for guidelines.

## License

MIT
