# llm-connector

Minimal Rust library for LLM protocol abstraction.

Supports 5 protocols: OpenAI, Anthropic, Zhipu, Aliyun, Ollama.
No complex configuration - just pick a protocol and start chatting.

## 🚨 Having Authentication Issues?

**Test your API keys right now:**
```bash
cargo run --example test_keys_yaml
```

This will tell you exactly what's wrong with your API keys! See [Debugging & Troubleshooting](#debugging--troubleshooting) for more details.

## ✨ Key Features

- **5 Protocol Support**: OpenAI, Anthropic, Zhipu, Aliyun, Ollama
- **No Hardcoded Models**: Use any model name without restrictions
- **Online Model Discovery**: Fetch available models dynamically from API
- **Enhanced Streaming Support**: Real-time streaming responses with proper Anthropic event handling
- **Ollama Model Management**: Full CRUD operations for local models
- **Unified Interface**: Same API for all protocols
- **Type-Safe**: Full Rust type safety with async/await

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
llm-connector = "0.3.6"
tokio = { version = "1", features = ["full"] }
```

Optional features:
```toml
llm-connector = { version = "0.3.6", features = ["streaming"] }
```

### Basic Usage

```rust
use llm_connector::{LlmClient, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OpenAI (default base URL)
    let client = LlmClient::openai("sk-...", None);

    // Anthropic
    let client = LlmClient::anthropic("sk-ant-...");

    // Aliyun (DashScope)
    let client = LlmClient::aliyun("sk-...");

    // Ollama (local, no API key needed)
    let client = LlmClient::ollama(None);

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
// OpenAI (default base URL)
let client = LlmClient::openai("sk-...", None);

// OpenAI-compatible endpoints (custom base URL)
let client = LlmClient::openai("sk-...", Some("https://api.example.com/v1"));
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

### 3. Zhipu Protocol (ChatGLM)
OpenAI-compatible format with Zhipu-specific error handling.

```rust
let client = LlmClient::zhipu("sk-...");
```

**Models**: glm-4, glm-4-flash, glm-4-air, glm-4-plus, glm-4x

### 4. Aliyun Protocol (DashScope)
Custom protocol for Qwen models.

```rust
let client = LlmClient::aliyun("sk-...");
```

**Models**: qwen-turbo, qwen-plus, qwen-max

### 4. Ollama Protocol (Local)
Local LLM server with no API key required.

```rust
// Default: localhost:11434
let client = LlmClient::ollama(None);

// Custom URL
let client = LlmClient::ollama(Some("http://192.168.1.100:11434"));
```

**Models**: llama3.2, llama3.1, mistral, mixtral, qwen2.5, etc.

**Features**:
- ✅ Model listing via `/api/tags`
- ✅ Model management (pull, push, delete, show details)
- ✅ Local server support with custom URLs
- ✅ Enhanced error handling for Ollama-specific operations

## Ollama Model Management

The library now provides comprehensive Ollama model management capabilities:

```rust
use llm_connector::ollama::OllamaModelOps;
let client = LlmClient::ollama();

// List all installed models
let models = client.list_models().await?;
for model in models {
    println!("Available model: {}", model);
}

// Pull a new model
client.pull_model("llama3.2").await?;

// Get detailed model information
let details = client.show_model("llama3.2").await?;
println!("Model size: {} bytes", details.size.unwrap_or(0));

// Delete a model
client.delete_model("llama3.2").await?;
```

### Supported Ollama Operations
- **List Models**: `list_models()` - Get all locally installed models
- **Pull Models**: `pull_model(name)` - Download models from registry
- **Push Models**: `push_model(name)` - Upload models to registry
- **Delete Models**: `delete_model(name)` - Remove local models
- **Show Details**: `show_model(name)` - Get comprehensive model information

## Enhanced Streaming Support

The library now includes improved streaming support for Anthropic with proper event state management:

```rust
use futures_util::StreamExt;

let client = LlmClient::anthropic("sk-ant-...");
let request = ChatRequest {
    model: "claude-3-5-sonnet-20241022".to_string(),
    messages: vec![Message::user("Hello!")],
    max_tokens: Some(200),
    ..Default::default()
};

let mut stream = client.chat_stream(&request).await?;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if let Some(content) = chunk.get_content() {
        print!("{}", content);
    }
}
```

### Enhanced Anthropic Streaming Features
- **State Management**: Proper handling of `message_start`, `content_block_delta`, `message_delta`, `message_stop` events
- **Event Processing**: Correct parsing of complex Anthropic streaming responses
- **Usage Tracking**: Real-time token usage statistics during streaming
- **Error Resilience**: Robust error handling for streaming interruptions

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
- ✅ Anthropic Protocol (limited support - returns fallback endpoint)
- ✅ Ollama Protocol (full support via `/api/tags`)
- ❌ Aliyun Protocol (not supported)

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

#### Ollama Streaming (GLM-4.6 via Remote Gateway)

If you expose an Ollama-compatible API while the backend actually calls Zhipu `glm-4.6` (remote gateway), you do NOT need any local model installation. Just point the client to your gateway and use the model id defined by your service:

```rust
use futures_util::StreamExt;
use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Point to your remote Ollama-compatible gateway (replace with your actual URL)
    let client = LlmClient::ollama(Some("https://your-ollama-gateway.example.com"));

    let request = ChatRequest {
        model: "glm-4.6".to_string(),
        messages: vec![Message::user("Briefly explain the benefits of streaming.")],
        max_tokens: Some(128),
        ..Default::default()
    };

    let mut stream = client.chat_stream(&request).await?;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(content) = chunk.get_content() {
            print!("{}", content);
        }
    }

    Ok(())
}
```

Run example (requires `streaming` feature):

```bash
cargo run --example ollama_streaming --features streaming
```

Note: This setup targets a remote Ollama-compatible gateway. The model id is defined by your backend (e.g. `glm-4.6`); no local installation is required. If your gateway uses a different identifier, replace it accordingly.

## Streaming (Optional Feature)

Enable streaming in your `Cargo.toml`:
```toml
llm-connector = { version = "0.3.6", features = ["streaming"] }
```

```rust
use futures_util::StreamExt;

let mut stream = client.chat_stream(&request).await?;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if let Some(content) = chunk.get_content() {
        print!("{}", content);
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
let client = LlmClient::openai(&api_key, None);
```

## Protocol Information

```rust
let client = LlmClient::openai("sk-...");

// Get protocol name
println!("Protocol: {}", client.protocol_name());

// Fetch models online (requires API call)
let models = client.fetch_models().await?;
println!("Available models: {:?}", models);
```

## Reasoning Synonyms

Many providers return hidden or provider-specific keys for model reasoning content (chain-of-thought). To simplify usage across providers, we normalize four common keys:

- `reasoning_content`, `reasoning`, `thought`, `thinking`

Post-processing automatically scans raw JSON and fills these optional fields on both regular messages (`Message`) and streaming deltas (`Delta`). You can read the first available value via a convenience method:

```rust
// Non-streaming
let msg = &response.choices[0].message;
if let Some(reason) = msg.reasoning_any() {
    println!("Reasoning: {}", reason);
}

// Streaming
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if let Some(reason) = chunk.choices[0].delta.reasoning_any() {
        println!("Reasoning (stream): {}", reason);
    }
}
```

Notes:
- Fields remain `None` if the provider does not return any reasoning keys.
- The normalization is provider-agnostic and applied uniformly to OpenAI, Anthropic, Aliyun (Qwen), Zhipu (GLM), and DeepSeek flows (including streaming).
- `StreamingResponse` also backfills its top-level `reasoning_content` from the first delta that contains reasoning.

## Debugging & Troubleshooting

### Test Your API Keys

Quickly test if your API keys are valid:

```bash
# Test all keys from keys.yaml
cargo run --example test_keys_yaml

# Debug DeepSeek specifically
cargo run --example debug_deepseek -- sk-your-key
```

The test tool will:
- ✅ Validate API key format
- ✅ Test authentication with the provider
- ✅ Show exactly what's wrong if a key fails
- ✅ Provide specific fix instructions

### Troubleshooting Guides

- **`TROUBLESHOOTING.md`** - Comprehensive troubleshooting guide
- **`HOW_TO_TEST_YOUR_KEYS.md`** - How to test your API keys
- **`TEST_YOUR_DEEPSEEK_KEY.md`** - Quick start for DeepSeek users

### Common Issues

**Authentication Error:**
```
❌ Authentication failed: Incorrect API key provided
```

**Solutions:**
1. Verify your API key is correct (no extra spaces)
2. Check if your account has credits
3. Generate a new API key from your provider's dashboard
4. Run `cargo run --example test_keys_yaml` to diagnose

## Recent Changes

### v0.3.1 (Latest)

**🚀 Major New Features:**
- **Complete Ollama Model Management**: Full CRUD operations for local models
  - `list_models()` - List all installed models
  - `pull_model()` - Download models from registry
  - `push_model()` - Upload models to registry
  - `delete_model()` - Remove local models
  - `show_model()` - Get detailed model information
- **Enhanced Anthropic Streaming**: Proper event state management
  - Correct handling of `message_start`, `content_block_delta`, `message_delta`, `message_stop` events
  - Real-time token usage tracking during streaming
  - Improved error resilience and state management

**🔧 Improvements:**
- **Expanded Model Discovery Support**:
  - Added Ollama model listing via `/api/tags` endpoint
  - Limited Anthropic model discovery support
- **Enhanced Client Interface**: New methods for Ollama model management
- **Updated Examples**: Added comprehensive model management and streaming examples

**📚 Documentation:**
- Complete rewrite of Ollama section with model management examples
- Enhanced streaming documentation with code examples
- Updated feature descriptions and supported operations

### v0.2.3

**🔧 Breaking Changes:**
- **Removed `supported_models()` method** - Use `fetch_models()` instead
- **Removed `supports_model()` method** - No longer needed

**✨ New Features:**
- **Improved error messages** - Removed confusing OpenAI URLs for other providers
- **New debugging tools:**
  - `examples/test_keys_yaml.rs` - Test all API keys
  - `examples/debug_deepseek.rs` - Debug DeepSeek authentication
- **Comprehensive documentation:**
  - `TROUBLESHOOTING.md` - Troubleshooting guide
  - `HOW_TO_TEST_YOUR_KEYS.md` - Testing instructions
  - `TEST_YOUR_DEEPSEEK_KEY.md` - Quick start guide

**Migration from v0.2.2:**
```rust
// ❌ Old (no longer works)
let models = client.supported_models();

// ✅ New
let models = client.fetch_models().await?;
```

### v0.2.2

**✨ New Features:**
- Added `fetch_models()` for online model discovery
- OpenAI protocol supports dynamic model fetching from `/v1/models` endpoint
- Works with OpenAI-compatible providers (DeepSeek, Zhipu, Moonshot, etc.)

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
# Test your API keys from keys.yaml
cargo run --example test_keys_yaml

# Debug DeepSeek authentication
cargo run --example debug_deepseek -- sk-your-key

# Simple fetch_models() demo
cargo run --example fetch_models_simple

# Ollama model management (NEW!)
cargo run --example ollama_model_management

# Anthropic streaming (NEW! - requires streaming feature)
cargo run --example anthropic_streaming --features streaming

# Ollama streaming (NEW! - requires streaming feature)
cargo run --example ollama_streaming --features streaming

# LongCat demo (OpenAI/Anthropic compatible)
cargo run --example longcat_dual
```

### Example Descriptions

**`test_keys_yaml.rs`** ⭐ New!
- Tests all API keys from your `keys.yaml` file
- Validates API key format and authentication
- Provides specific troubleshooting for each error
- **Run this first if you have authentication issues!**

**`debug_deepseek.rs`** ⭐ New!
- Interactive debugging tool for DeepSeek API
- Validates API key format
- Tests model fetching and chat requests
- Provides detailed troubleshooting guidance

**`fetch_models_simple.rs`**
- Simple demonstration of `fetch_models()`
- Shows how to fetch models from OpenAI-compatible providers
- Includes usage recommendations

**`ollama_model_management.rs`** ⭐ New!
- Demonstrates complete Ollama model management functionality
- Shows how to list, pull, delete, and get model details
- Includes error handling and practical usage examples

**`anthropic_streaming.rs`** ⭐ New!
- Shows enhanced Anthropic streaming with proper event handling
- Demonstrates real-time response streaming and usage tracking
- Includes both regular and streaming chat examples

**Removed redundant examples**
- `test_fetch_models.rs` and `test_with_keys.rs` were overlapping with other examples and have been removed.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT
