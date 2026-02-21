---
layout: home

hero:
  name: "llm-connector"
  text: "Rust LLM Connector"
  tagline: "Provider abstraction for OpenAI/Anthropic/Gemini and more — streaming, tools, and multimodal."
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/lipish/llm-connector
    - theme: alt
      text: Crates.io
      link: https://crates.io/crates/llm-connector

features:
  - title: Unified Streaming
    details: One streaming model across providers with a consistent response type.
  - title: Tools / Function Calling
    details: OpenAI-compatible tools with streaming support.
  - title: Multi-modal
    details: Text + images in a single message block.
  - title: Provider Matrix
    details: 12+ providers with clean Protocol/Provider separation.
---

# Install

```bash
cargo add llm-connector
```

# Quick Example

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
