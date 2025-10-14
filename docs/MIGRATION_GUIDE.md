# llm-connector v1.x to v2.0 Migration Guide

## Overview

This guide helps you migrate from llm-connector v1.x to v2.0, which introduces a cleaner two-tier architecture separating protocols (API specifications) from providers (service implementations).

## Breaking Changes Summary

### 1. Module Reorganization

```
v1.x Structure:
src/protocols/
├── openai.rs      # OpenAI protocol
├── anthropic.rs   # Anthropic protocol
├── aliyun.rs      # Aliyun "protocol" (actually custom provider)
├── zhipu.rs       # Zhipu "protocol" (actually custom provider)
└── ollama.rs      # Ollama "protocol" (actually custom provider)

v2.0 Structure:
src/core/                  # Core traits and abstractions
├── protocol.rs           # Protocol trait definitions
├── provider.rs           # Provider trait definitions
└── http.rs               # HTTP transport layer

src/protocols/             # Pure protocol implementations only
├── openai.rs             # Official OpenAI API specification
└── anthropic.rs          # Official Anthropic API specification

src/providers/             # Provider-specific implementations
├── aliyun.rs             # Aliyun DashScope provider
├── zhipu.rs              # Zhipu GLM provider
├── ollama.rs             # Ollama local server provider
└── openai_compatible.rs  # Generic OpenAI-compatible wrapper
```

### 2. API Changes

All existing v1.x constructors continue to work but show deprecation warnings:

```rust
// v1.x - Still works in v2.0 (deprecated)
let client = LlmClient::openai("sk-...", None);
let client = LlmClient::anthropic("sk-ant-...");
let client = LlmClient::aliyun("sk-...");
let client = LlmClient::zhipu("sk-...", None);
let client = LlmClient::ollama(None);

// v2.0 - Recommended new patterns
let client = LlmClient::protocol_openai("sk-...", None);
let client = LlmClient::protocol_anthropic("sk-ant-...");
let client = LlmClient::provider_aliyun("sk-...");
let client = LlmClient::provider_zhipu("sk-...", None);
let client = LlmClient::provider_ollama(None);
```

### 3. Trait Changes

If you were using internal traits directly:

```rust
// v1.x - No longer available
use llm_connector::protocols::ProviderAdapter;

// v2.0 - New traits
use llm_connector::core::Protocol;
use llm_connector::core::Provider;
```

## Migration Steps

### Step 1: Update Dependencies

Ensure you're using the latest version:

```toml
[dependencies]
llm-connector = "2.0.0"  # Update to v2.0
```

### Step 2: Update Constructor Calls (Recommended)

Update your code to use the new constructor naming:

```rust
// Before (v1.x)
let aliyun_client = LlmClient::aliyun("sk-...");
let zhipu_client = LlmClient::zhipu("sk-...", None);
let ollama_client = LlmClient::ollama(None);

// After (v2.0 - recommended)
let aliyun_client = LlmClient::provider_aliyun("sk-...");
let zhipu_client = LlmClient::provider_zhipu("sk-...", None);
let ollama_client = LlmClient::provider_ollama(None);
```

### Step 3: Update Custom Protocol Implementations

If you implemented custom protocols:

```rust
// v1.x
use llm_connector::protocols::ProviderAdapter;

impl ProviderAdapter for MyProtocol {
    type RequestType = MyRequest;
    type ResponseType = MyResponse;
    // ...
}

// v2.0
use llm_connector::core::Protocol;

impl Protocol for MyProtocol {
    type Request = MyRequest;
    type Response = MyResponse;
    // ...
}
```

### Step 4: Update Provider Implementations

If you were implementing custom providers:

```rust
// v2.0 - New Provider trait
use llm_connector::core::Provider;

impl Provider for MyProvider {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        // Your implementation
    }

    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        // Your streaming implementation
    }

    async fn fetch_models(&self) -> Result<Vec<String>, LlmConnectorError> {
        // Model fetching implementation
    }
}
```

## Compatibility Options

### Option 1: Gradual Migration (Recommended)

Keep using v1.x APIs with deprecation warnings initially, then migrate gradually:

```rust
// This will work but show warnings
let client = LlmClient::aliyun("sk-...");

// Plan to migrate to this when ready
let client = LlmClient::provider_aliyun("sk-...");
```

### Option 2: Immediate Migration

Update all constructor calls immediately to avoid warnings:

```bash
# Run clippy to see all deprecation warnings
cargo clippy -- -W deprecated

# Update all at once
sed -i 's/LlmClient::aliyun(/LlmClient::provider_aliyun(/g' src/**/*.rs
sed -i 's/LlmClient::zhipu(/LlmClient::provider_zhipu(/g' src/**/*.rs
sed -i 's/LlmClient::ollama(/LlmClient::provider_ollama(/g' src/**/*.rs
```

### Option 3: Conditional Migration

Use feature flags to control migration:

```rust
#[cfg(feature = "v2-api")]
let client = LlmClient::provider_aliyun("sk-...");

#[cfg(not(feature = "v2-api"))]
let client = LlmClient::aliyun("sk-...");
```

## Testing Your Migration

### 1. Run Existing Tests

Your existing tests should continue to pass without changes:

```bash
cargo test
```

### 2. Check for Deprecation Warnings

```bash
cargo test 2>&1 | grep -i deprecated
```

### 3. Update Example Usage

Update your examples to use new APIs:

```rust
// examples/migration_example.rs
use llm_connector::{LlmClient, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // v2.0 API usage
    let aliyun_client = LlmClient::provider_aliyun("sk-...");
    let zhipu_client = LlmClient::provider_zhipu("sk-...", None);
    let ollama_client = LlmClient::provider_ollama(None);

    // Protocol usage
    let openai_client = LlmClient::protocol_openai("sk-...", None);
    let anthropic_client = LlmClient::protocol_anthropic("sk-ant-...");

    // Rest of your code remains the same
    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message::user("Hello!")],
        ..Default::default()
    };

    let response = openai_client.chat(&request).await?;
    println!("Response: {}", response.choices[0].message.content);

    Ok(())
}
```

## Common Migration Issues

### Issue 1: Missing Imports

**Problem**: Compile errors due to changed trait names.

**Solution**: Update imports:

```rust
// Old
use llm_connector::protocols::ProviderAdapter;

// New
use llm_connector::core::Protocol;
use llm_connector::core::Provider;
```

### Issue 2: Custom Implementations

**Problem**: Custom protocol implementations no longer compile.

**Solution**: Update trait implementations:

```rust
// Old - ProviderAdapter
impl ProviderAdapter for MyProtocol {
    type RequestType = MyRequest;
    type ResponseType = MyResponse;

    fn build_request_data(&self, request: &ChatRequest, stream: bool) -> Self::RequestType {
        // ...
    }
}

// New - Protocol
impl Protocol for MyProtocol {
    type Request = MyRequest;
    type Response = MyResponse;

    fn build_request(&self, chat_req: &ChatRequest, stream: bool) -> Self::Request {
        // ...
    }
}
```

### Issue 3: Feature Flags

**Problem**: Streaming functionality not working.

**Solution**: Ensure feature flags are enabled:

```toml
[dependencies]
llm-connector = { version = "2.0.0", features = ["streaming"] }
```

## Benefits of Migrating

1. **Clearer Code**: Protocol vs Provider separation makes intent clearer
2. **Better Extensibility**: Easy to add new protocols and providers
3. **Reduced Complexity**: Protocol implementations remain pure and simple
4. **Future-Proof**: Architecture supports new API specifications
5. **Maintainability**: Cleaner separation of concerns

## Getting Help

- **Architecture Details**: See `docs/ARCHITECTURE.md`
- **Development Guidelines**: See `docs/RUST_PROJECT_CORE_RULES.md`
- **Examples**: Check the `examples/` directory for updated usage patterns
- **Issues**: Report migration problems on GitHub issues

## Timeline

- **Phase 1**: v2.0 release with backward compatibility (current)
- **Phase 2**: 6 months deprecation period for v1.x APIs
- **Phase 3**: v2.1 removes deprecated APIs (planned)

---

*This migration guide ensures a smooth transition to the improved v2.0 architecture while maintaining backward compatibility.*