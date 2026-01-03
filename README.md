# llm-connector

Next-generation Rust library for LLM protocol abstraction with **native multi-modal support**.

Supports 11+ providers: OpenAI, Anthropic, Aliyun, Zhipu, Ollama, Tencent, Volcengine, LongCat, Moonshot, DeepSeek, and more.
Clean architecture with unified output format, multi-modal content support, and configuration-driven design for maximum flexibility.

## Key Features

- **Multi-modal Content Support**: Native support for text + images in a single message (v0.5.0+)
- **Function Calling / Tools**: Full support for OpenAI-compatible function calling with streaming
- **Reasoning Models Support**: Universal support for reasoning models (Volcengine Doubao-Seed-Code, DeepSeek R1, OpenAI o1, etc.)
- **11+ Provider Support**: OpenAI, Anthropic, Aliyun, Zhipu, Ollama, Tencent, Volcengine, LongCat, Moonshot, DeepSeek, and more
- **Unified Output Format**: All providers return the same `StreamingResponse` type
- **Configuration-Driven Architecture**: Clean Protocol/Provider separation with flexible configuration
- **Extreme Performance**: 7,000x+ faster client creation (7µs vs 53ms)
- **Memory Efficient**: Only 16 bytes per client instance
- **Type-Safe**: Full Rust type safety with Result-based error handling
- **No Hardcoded Models**: Use any model name without restrictions
- **Online Model Discovery**: Fetch available models dynamically from API
- **Universal Streaming**: Real-time streaming with format abstraction
- **Ollama Model Management**: Full CRUD operations for local models

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
llm-connector = "0.5.13"
tokio = { version = "1", features = ["full"] }
```

Optional features:
```toml
# Streaming support
llm-connector = { version = "0.5.13", features = ["streaming"] }
```

### Basic Usage

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OpenAI
    let client = LlmClient::openai("sk-...")?;

    // Anthropic Claude
    let client = LlmClient::anthropic("sk-ant-...")?;

    // Aliyun DashScope
    let client = LlmClient::aliyun("sk-...")?;

    // Zhipu GLM
    let client = LlmClient::zhipu("your-api-key")?;

    // Ollama (local, no API key needed)
    let client = LlmClient::ollama()?;

    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message::text(Role::User, "Hello!")],
        ..Default::default()
    };

    let response = client.chat(&request).await?;
    println!("Response: {}", response.content);
    Ok(())
}
```

### Multi-modal Content (v0.5.0+)

Send text and images in a single message:

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, MessageBlock, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::openai("sk-...")?;

    // Text + Image URL
    let request = ChatRequest {
        model: "gpt-4o".to_string(),
        messages: vec![
            Message::new(
                Role::User,
                vec![
                    MessageBlock::text("What's in this image?"),
                    MessageBlock::image_url("https://example.com/image.jpg"),
                ],
            ),
        ],
        ..Default::default()
    };

    // Text + Base64 Image
    let request = ChatRequest {
        model: "gpt-4o".to_string(),
        messages: vec![
            Message::new(
                Role::User,
                vec![
                    MessageBlock::text("Analyze this image"),
                    MessageBlock::image_base64("image/jpeg", base64_data),
                ],
            ),
        ],
        ..Default::default()
    };

    let response = client.chat(&request).await?;
    println!("Response: {}", response.content);
    Ok(())
}
```

**Supported Content Types**:
- `MessageBlock::text(text)` - Text content
- `MessageBlock::image_url(url)` - Image from URL (OpenAI format)
- `MessageBlock::image_base64(media_type, data)` - Base64 encoded image
- `MessageBlock::image_url_anthropic(url)` - Image from URL (Anthropic format)

**Provider Support**:
- OpenAI - Full support (text + images)
- Anthropic - Full support (text + images)
- Other providers - Text only (images converted to text description)

See `examples/multimodal_basic.rs` for more examples.

### List Supported Providers

Get a list of all supported provider names:

```rust
use llm_connector::LlmClient;

fn main() {
    let providers = LlmClient::supported_providers();
    for provider in providers {
        println!("{}", provider);
    }
}
```

Output:
```
openai
aliyun
anthropic
zhipu
ollama
tencent
volcengine
longcat_anthropic
azure_openai
openai_compatible
```

See `examples/list_providers.rs` for a complete example.

## Supported Providers

llm-connector supports 11+ LLM providers with a unified interface:

| Provider | Quick Start | Features |
|----------|-------------|----------|
| **OpenAI** | `LlmClient::openai("sk-...")` | Chat, Streaming, Tools, Multi-modal, Reasoning (o1) |
| **Anthropic** | `LlmClient::anthropic("sk-ant-...")` | Chat, Streaming, Multi-modal |
| **Aliyun** | `LlmClient::aliyun("sk-...")` | Chat, Streaming, Qwen models |
| **Zhipu** | `LlmClient::zhipu("key")` | Chat, Streaming, Tools, GLM models |
| **Ollama** | `LlmClient::ollama()` | Chat, Streaming, Local models, Model management |
| **Tencent** | `LlmClient::tencent("id", "key")` | Chat, Streaming, Hunyuan models (Native V3) |
| **Volcengine** | `LlmClient::volcengine("key")` | Chat, Streaming, Reasoning (Doubao-Seed-Code) |
| **DeepSeek** | `LlmClient::deepseek("sk-...")` | Chat, Streaming, Reasoning (R1) |
| **Moonshot** | `LlmClient::moonshot("sk-...")` | Chat, Streaming, Long context |
| **Google** | `LlmClient::google("key")` | Chat, Gemini models |
| **LongCat** | `LlmClient::longcat_openai("ak-...")` | Chat, Streaming |

For detailed provider documentation and advanced configuration, see:
- [Detailed Protocol Information](#supported-protocols) below
- [Provider Guides](docs/PROVIDERS.md) for provider-specific features

## Function Calling / Tools

llm-connector supports OpenAI-compatible function calling (tools) with both streaming and non-streaming modes.

### Basic Usage

```rust
use llm_connector::{LlmClient, types::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::zhipu("your-api-key")?;

    // Define tools
    let tools = vec![Tool {
        tool_type: "function".to_string(),
        function: Function {
            name: "get_weather".to_string(),
            description: Some("Get weather information for a city".to_string()),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "City name, e.g. Beijing, Shanghai"
                    },
                    "unit": {
                        "type": "string",
                        "enum": ["celsius", "fahrenheit"],
                        "description": "Temperature unit"
                    }
                },
                "required": ["location"]
            }),
        },
    }];

    let request = ChatRequest {
        model: "glm-4-plus".to_string(),
        messages: vec![Message::text(Role::User, "What's the weather in Beijing?")],
        tools: Some(tools),
        tool_choice: Some(ToolChoice::auto()),
        ..Default::default()
    };

    let response = client.chat(&request).await?;

    // Check if tools were called
    if let Some(choice) = response.choices.first() {
        if let Some(tool_calls) = &choice.message.tool_calls {
            for call in tool_calls {
                println!("Tool: {}", call.function.name);
                println!("Arguments: {}", call.function.arguments);
            }
        }
    }

    Ok(())
}
```

### Streaming Tool Calls

```rust
use futures_util::StreamExt;

let request = ChatRequest {
    model: "glm-4-plus".to_string(),
    messages: vec![Message::text(Role::User, "What's the weather?")],
    tools: Some(tools),
    stream: Some(true),
    ..Default::default()
};

let mut stream = client.chat_stream(&request).await?;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;

    if let Some(choice) = chunk.choices.first() {
        if let Some(tool_calls) = &choice.delta.tool_calls {
            // Process tool calls incrementally
            for call in tool_calls {
                println!("Tool: {}", call.function.name);
            }
        }
    }
}
```

**Key Features**:
- Automatic deduplication of streaming tool_calls
- Incremental accumulation support
- Compatible with OpenAI streaming format
- Works with Zhipu, OpenAI, and other compatible providers

For complete examples, see:
- `examples/zhipu_tools.rs` - Basic tool calling
- `examples/zhipu_multiround_tools.rs` - Multi-round conversations with tools
- `examples/test_aliyun_streaming_tools.rs` - Streaming tool calls

**Technical Details**: See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for implementation details.

## Streaming

llm-connector provides unified streaming support across all providers with the `streaming` feature.

### Enable Streaming

```toml
[dependencies]
llm-connector = { version = "0.5.12", features = ["streaming"] }
```

### Basic Streaming

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}
```

**All providers return the same `StreamingResponse` type**, making it easy to switch between providers without changing your code.

For examples, see:
- `examples/anthropic_streaming.rs`
- `examples/ollama_streaming.rs`
- `examples/volcengine_streaming.rs`

## Supported Protocols

### 1. OpenAI Protocol
Standard OpenAI API format with multiple deployment options.

```rust
// OpenAI (default)
let client = LlmClient::openai("sk-...")?;

// Custom base URL
let client = LlmClient::openai_with_base_url("sk-...", "https://api.deepseek.com")?;

// Azure OpenAI
let client = LlmClient::azure_openai(
    "your-key",
    "https://your-resource.openai.azure.com",
    "2024-02-15-preview"
)?;

// OpenAI-compatible services
let client = LlmClient::openai_compatible("sk-...", "https://api.deepseek.com", "deepseek")?;
```

**Features:**
- No hardcoded models - use any model name
- Online model discovery via `models()`
- Azure OpenAI support
- Works with OpenAI-compatible providers (DeepSeek, Moonshot, etc.)

**Example Models**: gpt-4, gpt-4-turbo, gpt-3.5-turbo, o1-preview, o1-mini

### 2. Anthropic Protocol
Claude Messages API with multiple deployment options.

```rust
// Standard Anthropic API
let client = LlmClient::anthropic("sk-ant-...")?;

// Google Vertex AI
let client = LlmClient::anthropic_vertex("project-id", "us-central1", "access-token")?;

// Amazon Bedrock
let client = LlmClient::anthropic_bedrock("us-east-1", "access-key", "secret-key")?;
```

**Models**: claude-3-5-sonnet-20241022, claude-3-opus, claude-3-haiku

### 3. Zhipu Protocol (ChatGLM)
Supports both native and OpenAI-compatible formats.

```rust
// Native format
let client = LlmClient::zhipu("your-api-key")?;

// OpenAI-compatible format (recommended)
let client = LlmClient::zhipu_openai_compatible("your-api-key")?;
```

**Models**: glm-4, glm-4-flash, glm-4-air, glm-4-plus, glm-4x

### 4. Aliyun Protocol (DashScope)
Custom protocol for Qwen models with regional support.

```rust
// Default (China)
let client = LlmClient::aliyun("sk-...")?;

// International
let client = LlmClient::aliyun_international("sk-...")?;

// Private cloud
let client = LlmClient::aliyun_private("sk-...", "https://your-endpoint.com")?;
```

**Models**: qwen-turbo, qwen-plus, qwen-max

### 5. Ollama Protocol (Local)
Local LLM server with comprehensive model management.

```rust
// Default: localhost:11434
let client = LlmClient::ollama()?;

// Custom URL
let client = LlmClient::ollama_with_base_url("http://192.168.1.100:11434")?;

// With custom configuration
let client = LlmClient::ollama_with_config(
    "http://localhost:11434",
    Some(120), // timeout in seconds
    None       // proxy
)?;
```

**Models**: llama3.2, llama3.1, mistral, mixtral, qwen2.5, etc.

**Features**:
- Model listing and management
- Pull, delete, and inspect models
- Local server support with custom URLs
- Enhanced error handling for Ollama-specific operations
- Direct access to Ollama-specific features

### 6. Tencent Hunyuan
OpenAI-compatible API for Tencent Cloud.

```rust
// Default (Native API v3)
let client = LlmClient::tencent("secret-id", "secret-key")?;

// With custom configuration
let client = LlmClient::tencent_with_config(
    "secret-id",
    "secret-key",
    None,      // base_url (uses default)
    Some(60),  // timeout in seconds
    None       // proxy
)?;

// Streaming example
#[cfg(feature = "streaming")]
{
    use futures_util::StreamExt;

    let request = ChatRequest {
        model: "hunyuan-standard".to_string(),
        messages: vec![Message::user("Write a short poem about the ocean.")],
        stream: Some(true),
        ..Default::default()
    };

    let mut stream = client.chat_stream(&request).await?;
    while let Some(chunk) = stream.next().await {
        if let Some(content) = chunk?.get_content() {
            print!("{}", content);
        }
    }
}
```

**Models**: hunyuan-lite, hunyuan-standard, hunyuan-pro, hunyuan-turbo

See `examples/tencent_native_streaming.rs` for a runnable end-to-end streaming example.

### 7. Volcengine
OpenAI-compatible API with custom endpoint paths. Supports both standard chat models and reasoning models (Doubao-Seed-Code).

```rust
// Default
let client = LlmClient::volcengine("api-key")?;

// With custom configuration
let client = LlmClient::volcengine_with_config(
    "api-key",
    None,      // base_url (uses default: https://ark.cn-beijing.volces.com)
    Some(120), // timeout in seconds
    None       // proxy
)?;

// Streaming example (works with both standard and reasoning models)
#[cfg(feature = "streaming")]
{
    use futures_util::StreamExt;

    let request = ChatRequest {
        model: "ep-20250118155555-xxxxx".to_string(), // Use endpoint ID as model
        messages: vec![Message::user("Introduce yourself")],
        stream: Some(true),
        ..Default::default()
    };

    let mut stream = client.chat_stream(&request).await?;
    while let Some(chunk) = stream.next().await {
        if let Some(content) = chunk?.get_content() {
            print!("{}", content);
        }
    }
}
```

**Endpoint**: Uses `/api/v3/chat/completions` instead of `/v1/chat/completions`

**Models**:
- Standard models: Use endpoint ID (e.g., `ep-...`)
- Reasoning models: Doubao-Seed-Code (outputs via `reasoning_content` field, automatically handled)

**Streaming Support**: Full support for both standard and reasoning models. The library automatically extracts content from the appropriate field (`content` or `reasoning_content`).

### 8. LongCat API
Supports both OpenAI and Anthropic formats.

```rust
// OpenAI format
let client = LlmClient::longcat_openai("ak-...")?;

// Anthropic format (with Bearer auth)
let client = LlmClient::longcat_anthropic("ak-...")?;
```

**Models**: LongCat-Flash-Chat and other LongCat models

**Note**: LongCat's Anthropic format uses `Authorization: Bearer` instead of `x-api-key`

### 9. Moonshot
OpenAI-compatible API for Moonshot AI.

```rust
// Default
let client = LlmClient::moonshot("sk-...")?;

// With custom configuration
let client = LlmClient::moonshot_with_config(
    "sk-...",
    None,      // base_url (uses default)
    Some(60),  // timeout in seconds
    None       // proxy
)?;
```

**Models**: moonshot-v1-8k, moonshot-v1-32k, moonshot-v1-128k

**Features**:
- OpenAI-compatible API format
- Long context support (up to 128k tokens)
- Streaming support
- Unified output format

### 10. DeepSeek
OpenAI-compatible API with reasoning models support.

```rust
// Default
let client = LlmClient::deepseek("sk-...")?;

// With custom configuration
let client = LlmClient::deepseek_with_config(
    "sk-...",
    None,      // base_url (uses default)
    Some(60),  // timeout in seconds
    None       // proxy
)?;
```

**Models**:
- `deepseek-chat` - Standard chat model
- `deepseek-reasoner` - Reasoning model with thinking process

**Features**:
- OpenAI-compatible API format
- Reasoning content support (thinking process)
- Streaming support
- Unified output format
- Automatic extraction of reasoning content

**Reasoning Model Example**:
```rust
let request = ChatRequest {
    model: "deepseek-reasoner".to_string(),
    messages: vec![Message {
        role: Role::User,
        content: "Which is larger, 9.11 or 9.9?".to_string(),
        ..Default::default()
    }],
    ..Default::default()
};

let response = client.chat(&request).await?;

// Get reasoning process (thinking)
if let Some(reasoning) = response.reasoning_content {
    println!("Thinking: {}", reasoning);
}

// Get final answer
println!("Answer: {}", response.content);
```

### 11. Google Gemini
Native Google Gemini API support.

```rust
// Environment variable (recommended)
// export GEMINI_API_KEY="your-api-key"

// Default
let client = LlmClient::google("api-key")?;

// With custom configuration
let client = LlmClient::google_with_config(
    "api-key",
    None,      // base_url (uses default)
    Some(60),  // timeout in seconds
    None       // proxy
)?;
```

**Models**: gemini-2.0-flash, gemini-2.0-flash-lite-preview-02-05, gemini-1.5-flash, gemini-1.5-pro

**Features**:
- Native Google API implementation
- Chat completion support
- Streaming chat completion support
- Token usage tracking

**Notes**:
- See `examples/google_basic.rs` for a runnable end-to-end example.
- Streaming example: `examples/google_streaming.rs`

**Streaming**:

Run the streaming example:

```bash
export GEMINI_API_KEY="your-api-key"
export GEMINI_MODEL="gemini-2.5-flash"
cargo run --example google_streaming --features streaming
```

Supported environment variables in `google_streaming` example:
- `GEMINI_API_KEY`: Google AI Studio API key
- `GEMINI_MODEL`: model name (e.g. `gemini-2.5-flash`)
- `GEMINI_PROMPT`: override the prompt text
- `GEMINI_MAX_TOKENS`: override `max_tokens` (e.g. `4096`)
- `GEMINI_TIMEOUT_SECS`: per-chunk timeout used by the example

## Ollama Model Management

Access Ollama-specific features through the special interface:

```rust
let client = LlmClient::ollama()?;

// Access Ollama-specific features
if let Some(ollama) = client.as_ollama() {
    // List all installed models
    let models = ollama.models().await?;
    for model in models {
        println!("Available model: {}", model);
    }

    // Pull a new model
    ollama.pull_model("llama3.2").await?;

    // Get detailed model information
    let details = ollama.show_model("llama3.2").await?;
    println!("Model format: {}", details.details.format);

    // Check if model exists
    let exists = ollama.model_exists("llama3.2").await?;
    println!("Model exists: {}", exists);

    // Delete a model
    ollama.delete_model("llama3.2").await?;
}
```

### Supported Ollama Operations
- **List Models**: `models()` - Get all locally installed models
- **Pull Models**: `pull_model(name)` - Download models from registry
- **Delete Models**: `delete_model(name)` - Remove local models
- **Show Details**: `show_model(name)` - Get comprehensive model information
- **Check Existence**: `model_exists(name)` - Verify if model is installed

## Universal Streaming Format Support

The library provides comprehensive streaming support with universal format abstraction for maximum flexibility:

### Standard OpenAI Format (Default)

```rust
use futures_util::StreamExt;
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

let client = LlmClient::anthropic("sk-ant-...")?;
let request = ChatRequest {
    model: "claude-3-5-sonnet-20241022".to_string(),
    messages: vec![Message {
        role: Role::User,
        content: "Hello!".to_string(),
        ..Default::default()
    }],
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

### Pure Ollama Format for Tool Integration

For perfect compatibility with tools like Zed.dev, use the pure Ollama streaming format:

```rust
use futures_util::StreamExt;

// Use pure Ollama format (perfect for Zed.dev)
let mut stream = client.chat_stream_ollama(&request).await?;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    // chunk is now a pure OllamaStreamChunk
    if !chunk.message.content.is_empty() {
        print!("{}", chunk.message.content);
    }

    // Check for final chunk
    if chunk.done {
        println!("\nStreaming complete!");
        break;
    }
}
```

### Legacy Ollama Format (Embedded)

For backward compatibility, the embedded format is still available:

```rust
use futures_util::StreamExt;

// Use embedded Ollama format (legacy)
let mut stream = client.chat_stream_ollama_embedded(&request).await?;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    // chunk.content contains Ollama-formatted JSON string
    if let Ok(ollama_chunk) = serde_json::from_str::<serde_json::Value>(&chunk.content) {
        if let Some(content) = ollama_chunk
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
        {
            print!("{}", content);
        }
    }
}
```

### Streaming Chat Completions

For real-time streaming responses, use the streaming interface:

```rust
use llm_connector::types::{ChatRequest, Message};
use futures_util::StreamExt;

let request = ChatRequest {
    model: "gpt-4".to_string(),
    messages: vec![Message::user("Tell me a story")],
    stream: Some(true),
    ..Default::default()
};

let mut stream = client.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;

    // Get content from the current chunk
    if let Some(content) = chunk.get_content() {
        print!("{}", content);
    }

    // Access reasoning content (for providers that support it)
    if let Some(reasoning) = &chunk.reasoning_content {
        println!("Reasoning: {}", reasoning);
    }
}
```

### Advanced Streaming Features

The streaming response provides rich information and convenience methods:

```rust
let mut stream = client.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;

    // Access structured data
    println!("Model: {}", chunk.model);
    println!("ID: {}", chunk.id);

    // Get content from first choice
    if let Some(content) = chunk.get_content() {
        print!("{}", content);
    }

    // Access all choices
    for choice in &chunk.choices {
        if let Some(content) = &choice.delta.content {
            print!("{}", content);
        }
    }

    // Check for completion
    if chunk.choices.iter().any(|c| c.finish_reason.is_some()) {
        println!("\nStream completed!");
        break;
    }
}
```

### Format Comparison

| Format | Output Example | Use Case |
|--------|----------------|----------|
| **JSON** | `{"content":"hello"}` | API responses, standard JSON |
| **SSE** | `data: {"content":"hello"}\n\n` | Web real-time streaming |
| **NDJSON** | `{"content":"hello"}\n` | Log processing, data pipelines |

### Enhanced Anthropic Streaming Features
- **State Management**: Proper handling of `message_start`, `content_block_delta`, `message_delta`, `message_stop` events
- **Event Processing**: Correct parsing of complex Anthropic streaming responses
- **Usage Tracking**: Real-time token usage statistics during streaming
- **Error Resilience**: Robust error handling for streaming interruptions

## Model Discovery

Fetch the latest available models from the API:

```rust
let client = LlmClient::openai("sk-...")?;

// Fetch models online from the API
let models = client.models().await?;
println!("Available models: {:?}", models);
```

**Supported by:**
- OpenAI Protocol (including OpenAI-compatible providers like DeepSeek, Zhipu, Moonshot)
- Anthropic Protocol (limited support - returns fallback endpoint)
- Ollama Protocol (full support via `/api/tags`)
- Aliyun Protocol (not supported)

**Example Results:**
- DeepSeek: `["deepseek-chat", "deepseek-reasoner"]`
- Zhipu: `["glm-4.5", "glm-4.5-air", "glm-4.6"]`
- Moonshot: `["moonshot-v1-32k", "kimi-latest", ...]`

**Recommendation:**
- Cache `models()` results to avoid repeated API calls
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
    messages: vec![Message::user("Hello!")],
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

## Reasoning Models Support

llm-connector provides **universal support for reasoning models** across different providers. No matter which field the reasoning content is in (`reasoning_content`, `reasoning`, `thought`, `thinking`), it's automatically extracted and available via `get_content()`.

### Supported Reasoning Models

| Provider | Model | Reasoning Field | Status |
|----------|-------|----------------|--------|
| **Volcengine** | Doubao-Seed-Code | `reasoning_content` | Verified |
| **DeepSeek** | DeepSeek R1 | `reasoning_content` / `reasoning` | Supported |
| **OpenAI** | o1-preview, o1-mini | `thought` / `reasoning_content` | Supported |
| **Qwen** | Qwen-Plus | `reasoning` | Supported |
| **Anthropic** | Claude 3.5 Sonnet | `thinking` | Supported |

### Usage Example

**The same code works for all reasoning models:**

```rust
use futures_util::StreamExt;

// Works with Volcengine Doubao-Seed-Code
let provider = volcengine_with_config("api-key", None, Some(60), None)?;

// Works with DeepSeek R1
// let provider = openai_with_config("deepseek-key", Some("https://api.deepseek.com"), None, None)?;

// Works with OpenAI o1
// let provider = openai("openai-key")?;

let request = ChatRequest {
    model: "ep-20250118155555-xxxxx".to_string(), // or "deepseek-reasoner", "o1-preview", etc.
    messages: vec![Message::user("Solve this problem")],
    stream: Some(true),
    ..Default::default()
};

let mut stream = provider.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.get_content() {
        print!("{}", content);  // Automatically extracts reasoning content
    }
}
```

**Key Benefits:**
- **Zero Configuration**: Automatic field detection
- **Unified Interface**: Same code for all reasoning models
- **Backward Compatible**: Standard models (GPT-4, Claude) work as before
- **Priority-Based**: Standard `content` field takes precedence when available

See [Reasoning Models Support Guide](docs/REASONING_MODELS_SUPPORT.md) for detailed documentation.

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
let client = LlmClient::openai("sk-...")?;

// Get provider name
println!("Provider: {}", client.provider_name());

// Fetch models online (requires API call)
let models = client.models().await?;
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

## Unified Output Format

**All providers output the same unified `StreamingResponse` format**, regardless of their native API format.

```
Different Input Formats → Protocol Conversion → Unified StreamingResponse
```

### Why This Matters

- **Consistent API** - Same code works with all providers
- **Easy Switching** - Change providers without changing business logic
- **Type Safety** - Compile-time guarantees across all providers
- **Lower Learning Curve** - Learn once, use everywhere

### Example

```rust
// Same code works with ANY provider
let mut stream = client.chat_stream(&request).await?;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;  // Always StreamingResponse

    // Unified access methods
    if let Some(content) = chunk.get_content() {
        print!("{}", content);
    }

    if let Some(reason) = chunk.get_finish_reason() {
        println!("\nfinish_reason: {}", reason);
    }

    if let Some(usage) = chunk.usage {
        println!("usage: {:?}", usage);
    }
}
```

### How It Works

| Provider | Native Format | Conversion | Complexity |
|----------|--------------|------------|------------|
| OpenAI | OpenAI standard | Direct mapping | Simple |
| Tencent | OpenAI compatible | Direct mapping | Simple |
| Volcengine | OpenAI compatible | Direct mapping | Simple |
| Anthropic | Multi-event stream | Custom parser | Complex |
| Aliyun | DashScope format | Custom parser | Medium |
| Zhipu | GLM format | Custom parser | Medium |

**All conversions happen transparently in the Protocol layer** - you just get consistent `StreamingResponse` objects!

## Debugging & Troubleshooting

### Common Issues

**Authentication Error:**
```
Authentication failed: Incorrect API key provided
```

**Solutions:**
1. Verify your API key is correct (no extra spaces)
2. Check if your account has credits
3. Generate a new API key from your provider's dashboard
4. Test with a simple chat request to verify the key works

**Timeout Error:**
```
Request timeout
```

**Solutions:**
1. Check your network connection
2. Increase timeout settings using `*_with_timeout()` methods
3. Verify the provider's API endpoint is accessible

**Model Not Found:**
```
Model not found
```

**Solutions:**
1. Use `fetch_models()` to get available models
2. Check the model name spelling
3. Verify your account has access to the model

## Recent Changes

### v0.5.8 (Latest)

**Tencent Native API v3**
- Replaced OpenAI-compatible wrapper with native Tencent Cloud API v3 (`TC3-HMAC-SHA256`)
- Improved security with proper signature authentication
- Breaking Change: `tencent()` now accepts `(secret_id, secret_key)`

### v0.5.4

**Streaming Tool Calls Fix**
- Fixed: Incremental accumulation and deduplication logic for streaming tool_calls
- Improved: Support for OpenAI streaming API's incremental tool_calls format
- Guaranteed: Each tool_call is sent only once, preventing duplicate execution
- Compatible: Fully backward compatible, no impact on existing code

### v0.5.3

**Universal Reasoning Models Support**
- Support for all major reasoning models (Volcengine Doubao-Seed-Code, DeepSeek R1, OpenAI o1, etc.)
- Zero-configuration automatic field detection
- Unified interface, same code works for all reasoning models

### v0.4.8

**Simplified Configuration Architecture**
- Unified `chat_stream()` method
- 3000x+ performance improvement
- Support for reasoning content and usage statistics

---

**For complete changelog, see [CHANGELOG.md](CHANGELOG.md)**

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

Check out the `examples/` directory for various usage examples:

```bash
# Basic usage examples
cargo run --example openai_basic
cargo run --example anthropic_streaming --features streaming
cargo run --example aliyun_basic
cargo run --example zhipu_basic
cargo run --example ollama_basic
cargo run --example tencent_basic

# Multi-modal support
cargo run --example multimodal_basic

# Ollama model management
cargo run --example ollama_model_management
cargo run --example ollama_streaming --features streaming

# Function calling / Tools
cargo run --example zhipu_tools
cargo run --example zhipu_multiround_tools

# Streaming examples
cargo run --example volcengine_streaming --features streaming
cargo run --example test_longcat_anthropic_streaming --features streaming

# List all available providers
cargo run --example list_providers
```

### Example Descriptions

**Basic Examples:**
- `openai_basic.rs` - Simple OpenAI chat example
- `anthropic_streaming.rs` - Anthropic streaming with proper event handling
- `aliyun_basic.rs` - Aliyun DashScope basic usage
- `zhipu_basic.rs` - Zhipu GLM basic usage
- `ollama_basic.rs` - Ollama local model usage
- `tencent_basic.rs` - Tencent Hunyuan basic usage

**Advanced Examples:**
- `multimodal_basic.rs` - Multi-modal content (text + images)
- `ollama_model_management.rs` - Complete Ollama model CRUD operations
- `zhipu_tools.rs` - Function calling with Zhipu
- `zhipu_multiround_tools.rs` - Multi-round conversation with tools
- `volcengine_streaming.rs` - Volcengine streaming with reasoning models

**Utility Examples:**
- `list_providers.rs` - List all available providers and their configurations

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
Please check [RULES.md](RULES.md) for project rules and [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) for development guidelines.

## License

MIT
