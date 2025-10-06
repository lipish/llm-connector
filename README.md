# llm-connector

Minimal Rust library for LLM protocol abstraction.

Supports 4 protocols: OpenAI, Anthropic, Aliyun, Ollama.
No complex configuration - just pick a protocol and start chatting.

## ✨ Key Features

- **4 Protocol Support**: OpenAI, Anthropic, Aliyun, Ollama
- **No Hardcoded Models**: Use any model name without restrictions
- **Online Model Discovery**: Fetch available models dynamically from API
- **Streaming Support**: Real-time streaming responses (optional feature)
- **Unified Interface**: Same API for all protocols
- **Type-Safe**: Full Rust type safety with async/await

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
llm-connector = "0.2.2"
tokio = { version = "1", features = ["full"] }
```

Optional features:
```toml
llm-connector = { version = "0.2.2", features = ["streaming"] }
```

### Basic Usage

```rust
use llm_connector::{LlmClient, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OpenAI
    let client = LlmClient::openai("sk-...");

    // Anthropic
    let client = LlmClient::anthropic("sk-ant-...");

    // Aliyun (DashScope)
    let client = LlmClient::aliyun("sk-...");

    // Ollama (local, no API key needed)
    let client = LlmClient::ollama();

    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message::user("Hello!")],
        ..Default::default()
    };

    let response = client.chat(&request).await?;
    println!("Response: {}", response.choices[0].message.content);
    Ok(())
}
```

## Supported Protocols

### 1. OpenAI Protocol
Standard OpenAI API format.

```rust
// OpenAI
let client = LlmClient::openai("sk-...");

// OpenAI-compatible endpoints (if needed)
let client = LlmClient::openai_compatible("sk-...", "https://api.example.com/v1");
```

**Features:**
- ✅ No hardcoded models - use any model name
- ✅ Online model discovery via `fetch_models()`
- ✅ Works with OpenAI-compatible providers (DeepSeek, Zhipu, Moonshot, etc.)

**Example Models**: gpt-4, gpt-4-turbo, gpt-3.5-turbo, o1-preview, o1-mini

### 2. Anthropic Protocol
Claude Messages API with separate system messages.

```rust
let client = LlmClient::anthropic("sk-ant-...");
```

**Models**: claude-3-5-sonnet-20241022, claude-3-opus, claude-3-haiku

### 3. Aliyun Protocol (DashScope)
Custom protocol for Qwen models.

```rust
let client = LlmClient::aliyun("sk-...");
```

**Models**: qwen-turbo, qwen-plus, qwen-max

### 4. Ollama Protocol (Local)
Local LLM server with no API key required.

```rust
// Default: localhost:11434
let client = LlmClient::ollama();

// Custom URL
let client = LlmClient::ollama_at("http://192.168.1.100:11434");
```

**Models**: llama3.2, llama3.1, mistral, mixtral, qwen2.5, etc.

## Model Discovery

Fetch the latest available models from the API:

```rust
let client = LlmClient::openai("sk-...");

// Fetch models online from the API
let models = client.fetch_models().await?;
println!("Available models: {:?}", models);
```

**Supported by:**
- ✅ OpenAI Protocol (including OpenAI-compatible providers like DeepSeek, Zhipu, Moonshot)
- ❌ Anthropic Protocol (not supported)
- ❌ Aliyun Protocol (not supported)
- ❌ Ollama Protocol (not supported)

**Example Results:**
- DeepSeek: `["deepseek-chat", "deepseek-reasoner"]`
- Zhipu: `["glm-4.5", "glm-4.5-air", "glm-4.6"]`
- Moonshot: `["moonshot-v1-32k", "kimi-latest", ...]`

**Recommendation:**
- Cache `fetch_models()` results to avoid repeated API calls
- For protocols that don't support model listing, you can use any model name directly in your requests

## Request Examples

### OpenAI / OpenAI-compatible

```rust
let request = ChatRequest {
    model: "gpt-4".to_string(),
    messages: vec![
        Message::system("You are a helpful assistant."),
        Message::user("Hello!"),
    ],
    temperature: Some(0.7),
    max_tokens: Some(100),
    ..Default::default()
};
```

### Anthropic (requires max_tokens)

```rust
let request = ChatRequest {
    model: "claude-3-5-sonnet-20241022".to_string(),
    messages: vec![Message::user("Hello!")],
    max_tokens: Some(200), // Required for Anthropic
    ..Default::default()
};
```

### Aliyun (DashScope)

```rust
let request = ChatRequest {
    model: "qwen-max".to_string(),
    messages: vec![Message::user("你好！")],
    ..Default::default()
};
```

### Ollama (Local)

```rust
let request = ChatRequest {
    model: "llama3.2".to_string(),
    messages: vec![Message::user("Hello!")],
    ..Default::default()
};
```

## Streaming (Optional Feature)

Enable streaming in your `Cargo.toml`:
```toml
llm-connector = { version = "0.2.2", features = ["streaming"] }
```

```rust
use futures_util::StreamExt;

let mut stream = client.chat_stream(&request).await?;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if let Some(choice) = chunk.choices.first() {
        if let Some(content) = &choice.delta.content {
            print!("{}", content);
        }
    }
}
```

## Error Handling

```rust
use llm_connector::error::LlmConnectorError;

match client.chat(&request).await {
    Ok(response) => {
        println!("Response: {}", response.choices[0].message.content);
    }
    Err(e) => {
        match e {
            LlmConnectorError::AuthenticationError(msg) => {
                eprintln!("Auth error: {}", msg);
            }
            LlmConnectorError::RateLimitError(msg) => {
                eprintln!("Rate limit: {}", msg);
            }
            LlmConnectorError::UnsupportedOperation(msg) => {
                eprintln!("Not supported: {}", msg);
            }
            _ => eprintln!("Error: {}", e),
        }
    }
}
```

## Configuration

### Simple API Key (Recommended)

```rust
let client = LlmClient::openai("your-api-key");
```

### Environment Variables

```bash
export OPENAI_API_KEY="sk-your-key"
export ANTHROPIC_API_KEY="sk-ant-your-key"
export ALIYUN_API_KEY="sk-your-key"
```

```rust
use std::env;

let api_key = env::var("OPENAI_API_KEY")?;
let client = LlmClient::openai(&api_key);
```

## Protocol Information

```rust
let client = LlmClient::openai("sk-...");

// Get protocol name
println!("Protocol: {}", client.protocol_name());

// Get static models (fast, no API call)
println!("Static models: {:?}", client.supported_models());

// Fetch models online (requires API call)
let models = client.fetch_models().await?;
println!("Available models: {:?}", models);
```

## Recent Changes

### v0.2.1 (Latest)

**✨ New Features:**
- Added `fetch_models()` for online model discovery
- OpenAI protocol now supports dynamic model fetching from `/v1/models` endpoint
- Works with OpenAI-compatible providers (DeepSeek, Zhipu, Moonshot, etc.)

**🔧 Improvements:**
- Removed hardcoded model lists from OpenAI protocol
- `supported_models()` now returns empty `[]` for OpenAI (use `fetch_models()` instead)
- Added `UnsupportedOperation` error for protocols without model listing
- Added HTTP GET support to transport layer

**📝 Documentation:**
- Updated README with model discovery examples
- Added comprehensive examples: `test_fetch_models.rs`, `fetch_models_simple.rs`
- Created `FETCH_MODELS_FEATURE.md` with detailed documentation

## Design Philosophy

**Minimal by Design**:
- Only 4 protocols to cover all major LLM providers
- No hardcoded model restrictions - use any model name
- No complex configuration files or registries
- Direct API usage with clear abstractions

**Protocol-first**:
- Group providers by API protocol, not by company
- OpenAI-compatible providers share one implementation
- Extensible through protocol adapters

## Examples

Check out the `examples/` directory:

```bash
# Test online model fetching with all providers
cargo run --example test_fetch_models

# Compare supported_models() vs fetch_models()
cargo run --example fetch_models_simple

# Test with your API keys from keys.yaml
cargo run --example test_with_keys
```

### Example Descriptions

**`test_fetch_models.rs`**
- Tests `fetch_models()` with all providers from `keys.yaml`
- Shows which providers support online model listing
- Displays available models for each provider

**`fetch_models_simple.rs`**
- Simple comparison between `supported_models()` and `fetch_models()`
- Demonstrates the difference between static and online model discovery
- Includes usage recommendations

**`test_with_keys.rs`**
- Comprehensive test with real API keys
- Tests chat completion for all providers
- Verifies API connectivity and responses

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT
