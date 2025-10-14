# llm-connector Architecture v2.0

## Overview

This document describes the two-tier architecture of llm-connector v2.0, which provides a clean separation between **protocols** (API specifications) and **providers** (service implementations).

## Design Philosophy

### Problem Statement

The original llm-connector v1.x attempted to organize providers by "protocols", but this created confusion because many services implement custom APIs that are not truly protocol-compatible:

- **Aliyun (DashScope)**: Completely custom request/response format
- **Zhipu (GLM)**: Custom API with optional OpenAI compatibility mode
- **Ollama**: Local server with unique protocol and model management
- **True OpenAI-compatible services**: DeepSeek, Moonshot, etc.

### Solution: Two-Tier Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Client API                           │
│                  (LlmClient interface)                      │
└─────────────────────┬───────────────────────────────────────┘
                      │
        ┌─────────────┼─────────────┐
        │             │             │
┌───────▼──────┐ ┌────▼────┐ ┌─────▼──────┐
│   Protocols   │ │Providers│ │Compatibility│
│              │ │         │ │   Layer     │
│ • OpenAI      │ │ • Aliyun│ │ • OpenAI-   │
│ • Anthropic   │ │ • Zhipu │ │   Compatible│
│ • Future...   │ │ • Ollama│ │ • Protocol  │
│              │ │         │ │   Adapters  │
└──────────────┘ └─────────┘ └─────────────┘
```

## Architecture Components

### Tier 1: Protocols (`src/protocols/`)

**Definition**: Pure implementations of official API specifications.

#### Characteristics:
- Implement official API documentation exactly
- No provider-specific customizations
- Reusable across multiple providers when compatible
- Clear, minimal interfaces

#### Current Implementations:
- **OpenAI Protocol**: Official OpenAI API specification
- **Anthropic Protocol**: Official Anthropic Claude API specification

#### Protocol Trait Definition:
```rust
pub trait Protocol: Send + Sync {
    type Request: Serialize + Send + Sync;
    type Response: DeserializeOwned + Send + Sync;
    #[cfg(feature = "streaming")]
    type StreamResponse: DeserializeOwned + Send + Sync;
    type Error: ProtocolError;

    fn name(&self) -> &str;
    fn endpoint(&self, base_url: &str) -> String;
    fn models_endpoint(&self, base_url: &str) -> Option<String>;

    fn build_request(&self, chat_req: &ChatRequest, stream: bool) -> Self::Request;
    fn parse_response(&self, resp: Self::Response) -> ChatResponse;

    #[cfg(feature = "streaming")]
    fn parse_stream_response(&self, resp: Self::StreamResponse) -> StreamingResponse;
}
```

### Tier 2: Providers (`src/providers/`)

**Definition**: Service-specific implementations that may use custom APIs or adopt standard protocols.

#### Categories:

1. **Custom API Providers**: Services with unique APIs
   - **Aliyun Provider**: DashScope API with nested structure
   - **Ollama Provider**: Local server with model management

2. **Protocol-Compatible Providers**: Services using standard protocols
   - **Zhipu Provider**: GLM API with OpenAI compatibility mode
   - **DeepSeek**: Pure OpenAI protocol compatibility

#### Provider Trait Definition:
```rust
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError>;
    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError>;

    async fn fetch_models(&self) -> Result<Vec<String>, LlmConnectorError>;

    fn as_any(&self) -> &dyn std::any::Any;
}
```

### Compatibility Layer (`src/providers/openai_compatible.rs`)

**Purpose**: Generic wrapper for services that use standard protocols with minimal customizations.

```rust
pub struct OpenAICompatibleProvider<P: Protocol> {
    protocol: P,
    client: HttpClient,
    config: ProviderConfig,
    error_mapper: Box<dyn ErrorMapper>,
}

// Usage examples:
// - DeepSeek: OpenAICompatibleProvider<OpenAIProtocol>
// - Moonshot: OpenAICompatibleProvider<OpenAIProtocol>
// - Zhipu (compatibility mode): OpenAICompatibleProvider<OpenAIProtocol>
```

## Module Organization

```
src/
├── client.rs              # Main LlmClient API
├── core/                  # Core traits and abstractions
│   ├── mod.rs
│   ├── protocol.rs        # Protocol trait definitions
│   ├── provider.rs        # Provider trait definitions
│   ├── http.rs           # HTTP transport layer
│   └── error.rs          # Error handling
├── protocols/             # Pure protocol implementations
│   ├── mod.rs
│   ├── openai.rs         # OpenAI API specification
│   └── anthropic.rs      # Anthropic API specification
├── providers/             # Provider-specific implementations
│   ├── mod.rs
│   ├── aliyun.rs         # Aliyun DashScope provider
│   ├── zhipu.rs          # Zhipu GLM provider
│   ├── ollama.rs         # Ollama local server provider
│   └── openai_compatible.rs # Generic OpenAI-compatible wrapper
├── types/                 # Shared data types
│   ├── mod.rs
│   ├── request.rs        # ChatRequest types
│   ├── response.rs       # ChatResponse types
│   └── streaming.rs      # Streaming types
├── config.rs              # Configuration management
├── error.rs               # Public error types
└── sse.rs                 # Server-Sent Events utilities
```

## Implementation Patterns

### Pattern 1: Pure Protocol Implementation

```rust
// src/protocols/openai.rs
pub struct OpenAIProtocol {
    name: Arc<str>,
}

impl Protocol for OpenAIProtocol {
    type Request = OpenAIRequest;
    type Response = OpenAIResponse;
    // ... pure OpenAI API implementation
}
```

### Pattern 2: Custom Provider Implementation

```rust
// src/providers/aliyun.rs
pub struct AliyunProvider {
    client: HttpClient,
    config: ProviderConfig,
}

impl Provider for AliyunProvider {
    // ... custom Aliyun API implementation
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        // Custom request transformation
        let aliyun_request = self.build_aliyun_request(request);

        // HTTP call
        let response = self.client.post(&self.endpoint, &aliyun_request).await?;

        // Custom response parsing
        self.parse_aliyun_response(response).await
    }
}
```

### Pattern 3: OpenAI-Compatible Provider

```rust
// src/providers/openai_compatible.rs
pub struct OpenAICompatibleProvider<P: Protocol> {
    protocol: P,
    // ... HTTP client and config
}

impl<P: Protocol> Provider for OpenAICompatibleProvider<P> {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        // Use protocol implementation
        let protocol_request = self.protocol.build_request(request, false);
        // ... HTTP call with custom error mapping
        let response = self.make_request(&protocol_request).await?;
        Ok(self.protocol.parse_response(response))
    }
}
```

## Client Interface Design

### Constructor Naming Convention

```rust
impl LlmClient {
    // Pure protocol constructors
    pub fn protocol_openai(api_key: &str, base_url: Option<&str>) -> Self;
    pub fn protocol_anthropic(api_key: &str) -> Self;

    // Provider-specific constructors
    pub fn provider_aliyun(api_key: &str) -> Self;
    pub fn provider_zhipu(api_key: &str, base_url: Option<&str>) -> Self;
    pub fn provider_ollama(base_url: Option<&str>) -> Self;

    // Backward compatibility (deprecated)
    #[deprecated(since = "2.0.0", note = "Use provider_aliyun() instead")]
    pub fn aliyun(api_key: &str) -> Self;
}
```

## Migration Strategy

### Phase 1: Core Architecture (Breaking Changes)
- Create new core traits (`Protocol`, `Provider`)
- Refactor existing protocol implementations
- Move custom protocols to provider implementations

### Phase 2: Client Interface Updates
- Update `LlmClient` constructors
- Maintain backward compatibility with deprecation warnings
- Add new protocol-first constructors

### Phase 3: Documentation and Examples
- Update all documentation
- Create migration examples
- Update existing examples to use new patterns

## Benefits of v2.0 Architecture

1. **Conceptual Clarity**: Clear separation between API specifications and service implementations
2. **Better Extensibility**: Easy to add new protocols and providers independently
3. **Reduced Complexity**: Protocol implementations remain pure and simple
4. **Provider Flexibility**: Custom providers can implement unique features without protocol pollution
5. **Future-Proof**: Easy to add new API specifications (Google Gemini, etc.)
6. **Backward Compatibility**: Existing code continues to work with minimal changes

## Usage Examples

### Using Pure Protocols
```rust
// Direct protocol usage
let openai_protocol = OpenAIProtocol::new();
let client = LlmClient::from_protocol(openai_protocol, "sk-...", None);
```

### Using Custom Providers
```rust
// Provider with custom API
let client = LlmClient::provider_aliyun("sk-...");
let response = client.chat(&request).await?;
```

### Using OpenAI-Compatible Providers
```rust
// Generic OpenAI-compatible service
let client = LlmClient::openai_compatible::<OpenAIProtocol>(
    "sk-deepseek-...",
    "https://api.deepseek.com/v1"
);
```

This architecture provides a clean, maintainable, and extensible foundation for LLM connector development.