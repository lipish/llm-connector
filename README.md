<div align="center">

<h1>llm-connector</h1>

[![crates.io](https://img.shields.io/crates/v/llm-connector.svg)](https://crates.io/crates/llm-connector)
[![build](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/lipish/llm-connector)
[![downloads](https://img.shields.io/crates/d/llm-connector.svg)](https://crates.io/crates/llm-connector)
[![msrv](https://img.shields.io/badge/MSRV-1.85%2B-blue.svg)](https://www.rust-lang.org)
[![docs.rs](https://img.shields.io/docsrs/llm-connector)](https://docs.rs/llm-connector)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**The Blind Protocol Engine** - A pure, URL-agnostic adapter for LLM services.

[Installation](#installation) • [Usage](#usage) • [Philosophy](#philosophy) • [Documentation](#documentation) • [Synergy](#synergy-with-llm-providers)

</div>

---

## 🚀 The Pure Protocol Engine

`llm-connector` is a **minimalist, standalone driver layer** for AI providers. Unlike other libraries that maintain lists of provider endpoints, `llm-connector` adopts a **Pure Gateway Architecture**: it handles protocol adaptation, streaming, and token normalization, but remains completely "blind" to API endpoints.

- **Zero Hardcoded URLs**: No default endpoints. You provide the `base_url`, we provide the protocol.
- **Pure Positioning**: We don't care where the model is hosted (SaaS, Local, Private Cloud). We only care about the *dialect* (Protocol) it speaks.
- **Standalone & Light**: `llm-connector` is self-contained. It does **not** depend on any endpoint management projects or external databases.

## 🤝 Ecosystem Decoupling

While `llm-connector` is the **engine (Protocol)**, it is designed to work seamlessly with (but not depend on) configuration managers like [llm-providers](https://github.com/lipish/llm-providers). 

- **llm-connector**: Handles *how* to talk (Request/Response logic).
- **llm-providers**: (External) Handles *where* to talk (URL/Region discovery).

This decoupling ensures that `llm-connector` remains a stable, logic-only library while the rapidly changing landscape of AI endpoints is managed elsewhere.

## 📊 Comparison

| Feature | `llm-connector` (Protocol Engine) | Traditional SDKs / Libraries |
| :--- | :--- | :--- |
| **Endpoint Logic** | ❌ None. Mandatory `base_url`. | ✅ Hardcoded/Default URLs. |
| **Maintenance** | 🛠️ Low. Logic-only updates. | ⚠️ High. Constant URL/Region list syncing. |
| **Self-Hosted** | ✅ Native. Any URL works. | ⚠️ Often requires hacky overrides. |
| **SaaS Gateway** | ✅ Optimized. Perfect for proxies. | ❌ Often inflexible. |

## 🛠️ Installation

**MSRV**: Rust 1.85+ (Rust 2024 edition)

```toml
[dependencies]
llm-connector = "0.8.0"
tokio = { version = "1", features = ["full"] }
```

## 📖 Usage

### Unified Chat
All client constructions now **mandatorily require** a `base_url`.

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

// Positioned as a blind protocol engine: You must provide the base_url
let client = LlmClient::openai("sk-...", "https://api.openai.com/v1")?;

let request = ChatRequest::new("gpt-4o")
    .add_message(Message::user("What is the speed of light?"));

let response = client.chat(&request).await?;
println!("{}", response.content);
```

### Universal Streaming

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
use futures_util::StreamExt;

let client = LlmClient::anthropic("sk-ant-...", "https://api.anthropic.com")?;
let request = ChatRequest::new("claude-3-5-sonnet-20240620").stream(true);

let mut stream = client.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.get_content() {
        print!("{}", content);
    }
}
```

### Fluent Builder

```rust
let client = LlmClient::builder()
    .openai("sk-...")
    .base_url("https://api.deepseek.com") // Mandatory
    .timeout(60)
    .build()?;
```

### Per-Request Overrides (Gateway Pattern)
Ideal for building unified AI gateways or multi-tenant proxies.

```rust
let request = ChatRequest::new("gpt-4o")
    .with_api_key("dynamic-tenant-key")
    .with_base_url("https://my-proxy.internal/v1");

let response = client.chat(&request).await?;
```

## 📂 Recent Changelogs

- **v0.8.0 (2026-03-02)**
    - `!` **BREAKING**: Rebranded as a **Pure Protocol Engine**.
    - `!` **BREAKING**: Removed all default/hardcoded URLs. `base_url` is now **mandatory** for all client constructors.
    - `!` **BREAKING**: Removed redundant provider files (`deepseek`, `moonshot`, `volcengine`, etc.) in favor of generic Protocol adapters.
    - `+` **Endpoints Module**: Added `llm_connector::endpoints` constants for common API addresses (optional reference only).
- **v0.7.1**
    - `+` **Embedding API**: Unified `.embed()` support for major providers.
    - `+` **Document Support**: Added `MessageBlock::Document` for PDFs and files.
    - `^` **Usage**: Added `prompt_cache_hit_tokens` support.

## 📜 License

MIT
