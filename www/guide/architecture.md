# Architecture

`llm-connector` is designed to provide a unified, type-safe interface for various LLM providers while handling the differences in their underlying protocols.

## Core Design Principles

1. **Unified Interface**: A single `LlmClient` struct that can be instantiated with any backend provider.
2. **Protocol Abstraction**: Separation between `Provider` (high-level logic) and `Protocol` (request/response formatting).
3. **Unified Output**: All providers flow into a `StreamingResponse` unified format, simplifying downstream consumption.

## Unified Output Format

Regardless of the provider (OpenAI, Anthropic, Tencent, etc.), all responses are normalized.

```rust
pub struct StreamingResponse {
    pub content: Option<String>,
    pub reasoning_content: Option<String>, // For reasoning models (o1, R1)
    pub tool_calls: Option<Vec<ToolCall>>,
    pub usage: Option<Usage>,
    pub finish_reason: Option<FinishReason>,
    // ...
}
```

This ensures that switching providers does not require changing your response handling logic.

## Multi-Modal Support

The library supports multi-modal inputs (Text + Image) natively via the `MessageBlock` enum.

```rust
pub enum MessageBlock {
    Text { text: String },
    Image { source: ImageSource },     // Base64
    ImageUrl { image_url: ImageUrl },  // URL
}
```

- **OpenAI**: Converted to `content: [{type: "text"}, {type: "image_url"}]`
- **Anthropic**: Converted to `content: [{type: "text"}, {type: "image"}]`
- **Text-Only Providers**: Images are automatically stripped or result in an error (depending on implementation).

## Streaming Architecture

Streaming is handled via `reqwest` and `eventsource-stream`.

- **Parsing**: Each provider has a `parse_stream_response` method.
- **Normalization**: Provider-specific SSE events are mapped to the unified `StreamingResponse`.
- **Reasoning**: For reasoning models (DeepSeek R1, Volcengine), `reasoning_content` is extracted from specific fields (e.g., `delta.reasoning_content`) and exposed separately.

## Configuration System

The `ConfigurableProtocol` allows creating new providers purely through configuration, without writing new code, for any OpenAI-compatible service.

```rust
ProtocolConfig {
    endpoints: EndpointConfig {
        chat_template: "{base_url}/v1/chat/completions".to_string(),
        ..
    },
    auth: AuthConfig::Bearer,
}
```
