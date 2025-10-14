# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is `llm-connector`, a minimal Rust library for LLM protocol abstraction. The library provides a unified interface for working with multiple LLM providers through a clean two-tier architecture:

### Architecture v2.0 (Post-Refactoring)

**Two-Tier Design:**
- **Protocols**: Pure API specifications (OpenAI, Anthropic)
- **Providers**: Service-specific implementations (Aliyun, Zhipu, Ollama) with optional protocol compatibility

**Key Design Philosophy:**
- **Clear Separation**: True protocols vs provider-specific implementations
- **Protocol Purity**: OpenAI and Anthropic protocols implement official specifications only
- **Provider Flexibility**: Custom providers can implement unique APIs or adopt standard protocols
- **Backward Compatibility**: Existing client constructors continue to work with deprecation warnings

## Build and Development Commands

### Building and Testing
```bash
# Build the library
cargo build

# Build with release optimizations
cargo build --release

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Check code formatting
cargo fmt --check

# Format code
cargo fmt

# Run clippy lints
cargo clippy

# Run clippy with all features
cargo clippy --all-features
```

### Running Examples
```bash
# Test API keys from keys.yaml
cargo run --example test_keys_yaml

# Basic OpenAI chat example
cargo run --example openai_basic

# Anthropic streaming (requires streaming feature)
cargo run --example anthropic_streaming --features streaming

# Ollama model management
cargo run --example ollama_model_management

# Fetch models from providers
cargo run --example fetch_models_simple

# Debug DeepSeek authentication
cargo run --example debug_deepseek -- your-api-key
```

### Version Management
```bash
# Check version vs crates.io
make check-version

# Sync version with crates.io
make sync-version

# Bump to specific version
make bump-version VERSION=0.3.7

# Full release process
make release VERSION=0.3.7
```

## Architecture Overview

### Two-Tier Architecture (v2.0)

The library uses a clean separation between **protocols** (API specifications) and **providers** (service implementations):

#### Tier 1: Pure Protocols (`src/protocols/`)
- **OpenAI Protocol**: Implements official OpenAI API specification only
- **Anthropic Protocol**: Implements official Anthropic Claude API specification only

#### Tier 2: Provider Implementations (`src/providers/`)
- **Aliyun Provider**: Custom DashScope API with unique request/response format
- **Zhipu Provider**: Custom GLM API with OpenAI compatibility mode
- **Ollama Provider**: Local server with model management capabilities
- **OpenAI-Compatible Providers**: Generic wrapper for services using standard protocols

### Core Design Patterns

1. **Protocol Trait** (`src/core/protocol.rs`): Defines pure API specification interfaces
2. **Provider Trait** (`src/core/provider.rs`): Defines service implementation interfaces
3. **Compatibility Layer**: `OpenAICompatibleProvider<T>` for providers using standard protocols
4. **Error Mapping**: Provider-specific error handling with unified error types

### Module Structure (Post-Refactoring)

- **`src/client.rs`**: Main `LlmClient` with unified API for all providers/protocols
- **`src/core/`**: Core traits and abstractions
  - `protocol.rs`: Protocol trait definitions
  - `provider.rs`: Provider trait definitions
  - `http.rs`: HTTP transport and client management
  - `error.rs`: Error mapping and handling
- **`src/protocols/`**: Pure protocol implementations
  - `openai.rs`: Official OpenAI API specification
  - `anthropic.rs`: Official Anthropic API specification
- **`src/providers/`**: Provider-specific implementations
  - `aliyun.rs`: Aliyun DashScope provider
  - `zhipu.rs`: Zhipu GLM provider
  - `ollama.rs`: Ollama local server provider
  - `openai_compatible.rs`: Generic OpenAI-compatible wrapper
- **`src/types/`**: Shared data types for requests, responses, and streaming
- **`src/config.rs`**: Provider configuration management
- **`src/sse.rs`**: Server-Sent Events utilities for streaming

### Key Features

- **Streaming Support**: Optional streaming via the "streaming" feature flag
- **Model Discovery**: Dynamic model fetching via `fetch_models()` method
- **Ollama Model Management**: Full CRUD operations for local Ollama models
- **Reasoning Content**: Provider-agnostic extraction of reasoning/thinking content from responses
- **Error Handling**: Comprehensive error mapping with provider-specific details

## Development Guidelines

### Code Style (from docs/RUST_PROJECT_CORE_RULES.md)

- Follow rustfmt formatting and clippy lints
- Use `CamelCase` for structs/enums, `snake_case` for functions/modules
- Prefer minimal public API surface
- **NEW**: Use protocol vs provider separation - protocols are API specs, providers are services
- Implement proper error handling with dedicated error types
- Use `Arc` for efficient sharing of configuration and HTTP clients
- Maintain backward compatibility when adding new features

### Testing Strategy

- Unit tests for each protocol implementation in respective files
- Integration tests in `tests/` directory
- Examples serve as both documentation and functional tests
- Use `wiremock` for HTTP mocking in tests
- Test both success and error scenarios

### Feature Flags

- **`default = ["reqwest"]`**: HTTP client support
- **`streaming`**: Enable streaming responses (requires `pin-project-lite`)
- **`config`**: Configuration support
- **`yaml`**: YAML configuration file support

## Common Development Tasks

### Adding a New Protocol

For adding support for a new API specification (e.g., Google Gemini):

1. Create protocol file in `src/protocols/` (e.g., `gemini.rs`)
2. Implement `Protocol` trait with pure API specification
3. Define protocol-specific request/response types
4. Add protocol exports in `src/protocols/mod.rs`
5. Add tests and examples using the protocol directly

### Adding a New Provider

For adding a new service provider:

1. **Custom API Provider** (like Aliyun):
   - Create provider file in `src/providers/` (e.g., `new_provider.rs`)
   - Implement `Provider` trait with custom logic
   - Define provider-specific request/response transformations
   - Add constructor in `src/client.rs`

2. **OpenAI-Compatible Provider** (like DeepSeek):
   - Use `OpenAICompatibleProvider<OpenAIProtocol>` wrapper
   - Specify custom base URL and error handling
   - Add convenience constructor in `src/client.rs`

### Adding New Request Parameters

1. Update relevant types in `src/types/request.rs`
2. Update protocol implementations to handle new parameters
3. Update provider implementations as needed
4. Ensure `Default` implementation includes sensible defaults
5. Update examples and documentation

### Debugging API Issues

Use the test tools available:

```bash
# Test all API keys
cargo run --example test_keys_yaml

# Debug specific provider
cargo run --example debug_deepseek -- your-key

# Enable debug output
export LLM_DEBUG_REQUEST_RAW=1
export LLM_DEBUG_RESPONSE_RAW=1
export LLM_DEBUG_STREAM_RAW=1
```

### Working with Streaming

Streaming requires the "streaming" feature and uses `futures_util::StreamExt`:

```rust
#[cfg(feature = "streaming")]
let mut stream = client.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if let Some(content) = chunk.get_content() {
        print!("{}", content);
    }
}
```

## Migration Guide (v1.x to v2.0)

### Backward Compatibility

Existing code continues to work with minimal changes:

```rust
// v1.x - Still works in v2.0 with deprecation warnings
let client = LlmClient::aliyun("sk-...");
let client = LlmClient::zhipu("sk-...", None);
let client = LlmClient::ollama(None);

// v2.0 - Recommended new patterns
let client = LlmClient::provider_aliyun("sk-...");
let client = LlmClient::provider_zhipu("sk-...", None);
let client = LlmClient::provider_ollama(None);

// v2.0 - Protocol-first (for true protocol implementations)
let client = LlmClient::protocol_openai("sk-...", None);
let client = LlmClient::protocol_anthropic("sk-ant-...");
```

### Breaking Changes

1. **Module Reorganization**: `src/protocols/` now contains only pure protocols
2. **New Module**: `src/providers/` contains provider-specific implementations
3. **Trait Changes**: `ProviderAdapter` → `Protocol` trait for protocols, `Provider` trait for providers
4. **Constructor Names**: Provider constructors prefixed with `provider_` for clarity

### Migration Steps

1. Update import statements if using internal traits directly
2. Replace deprecated constructor calls with new ones
3. Update custom protocol implementations to use new `Protocol` trait
4. Test with existing examples - they should continue to work

## Project Standards

- **No hardcoded model names** - users can specify any model name
- **Protocol vs Provider Separation** - protocols are API specs, providers are services
- **Minimal configuration** - sensible defaults, no complex config files
- **Type safety** - leverage Rust's type system for compile-time guarantees
- **Error context** - rich error information for debugging
- **Zero-cost abstractions** - efficient abstractions with no runtime overhead
- **Backward compatibility** - maintain existing APIs during transitions

The codebase follows strict Rust design patterns as documented in `docs/RUST_PROJECT_GUIDELINES_EN.md` and `docs/ARCHITECTURE.md`.