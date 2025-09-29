# llm-connector

- English | [中文](./README.zh-CN.md)

A lightweight Rust library for **protocol adaptation** across multiple LLM providers. This library focuses solely on converting between different LLM provider APIs and providing a unified OpenAI-compatible interface.

## 🎯 What llm-connector Does

**Core Purpose**: Protocol adaptation and API normalization

- ✅ **Protocol Conversion**: Convert different LLM provider APIs to OpenAI-compatible format
- ✅ **Request/Response Standardization**: Unified data structures across providers
- ✅ **Streaming Adaptation**: Normalize SSE streams from different providers
- ✅ **Provider Abstraction**: Extensible trait system for adding new providers
- ✅ **Simple Configuration**: Basic API key and endpoint management

## 🚫 What llm-connector Does NOT Do

This library intentionally **does not** include:
- ❌ Load balancing (use a reverse proxy or service mesh)
- ❌ Health checking (use external monitoring)
- ❌ Circuit breaking (use infrastructure-level solutions)
- ❌ Complex routing strategies (keep it simple)
- ❌ Built-in metrics collection (use tracing/metrics crates)
- ❌ Request queuing or rate limiting

## Features

- **Provider-agnostic types**: Unified request/response structures
- **Simple provider trait**: Easy to implement new providers
- **Streaming support**: Unified SSE handling across providers
- **Minimal dependencies**: Focused on core functionality
- **OpenAI compatibility**: Drop-in replacement for OpenAI client

## Supported Providers

- **DeepSeek** ✅ - DeepSeek models (deepseek-chat, deepseek-reasoner)
- **OpenAI** 🚧 - GPT models (gpt-4, gpt-3.5-turbo, etc.) - *Coming soon*
- **Anthropic** 🚧 - Claude models (claude-3-5-sonnet, claude-3-haiku, etc.) - *Coming soon*
- **Zhipu GLM** ✅ - GLM models (glm-4, glm-4-plus, glm-4-flash, etc.)
- **Alibaba Qwen** 🚧 - Qwen models (qwen-turbo, qwen-plus, etc.) - *Coming soon*
- **Moonshot Kimi** 🚧 - Kimi models (moonshot-v1-8k, moonshot-v1-32k, etc.) - *Coming soon*

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
llm-connector = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use llm_connector::{Client, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client with environment variables
    let client = Client::from_env();

    // Create a chat request
    let request = ChatRequest {
        model: "deepseek/deepseek-chat".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Hello, how are you?".to_string(),
                ..Default::default()
            }
        ],
        ..Default::default()
    };

    // Send request
    let response = client.chat(request).await?;
    println!("Response: {}", response.choices[0].message.content);

    Ok(())
}
```

### Configuration

#### Environment Variables (Recommended)

Set environment variables for the providers you want to use:

```bash
# DeepSeek (✅ Available now)
export DEEPSEEK_API_KEY="your-deepseek-key"
export DEEPSEEK_BASE_URL="https://api.deepseek.com"  # optional

# Zhipu GLM (✅ Available now)
export ZHIPU_API_KEY="your-zhipu-key"
export ZHIPU_BASE_URL="https://open.bigmodel.cn/api/paas/v4"  # optional

# OpenAI (🚧 Coming soon)
# export OPENAI_API_KEY="your-openai-key"
# export OPENAI_BASE_URL="https://api.openai.com/v1"  # optional

# Anthropic (🚧 Coming soon)
# export ANTHROPIC_API_KEY="your-anthropic-key"
# export ANTHROPIC_BASE_URL="https://api.anthropic.com"  # optional

# Add other providers as they become available...
```

#### Explicit Configuration

```rust
use llm_connector::{Client, Config, ProviderConfig};

let config = Config {
    openai: Some(ProviderConfig {
        api_key: "your-openai-key".to_string(),
        base_url: Some("https://api.openai.com/v1".to_string()),
        timeout_ms: None,
    }),
    deepseek: Some(ProviderConfig {
        api_key: "your-deepseek-key".to_string(),
        base_url: Some("https://api.deepseek.com".to_string()),
        timeout_ms: None,
    }),
    zhipu: Some(ProviderConfig {
        api_key: "your-zhipu-key".to_string(),
        base_url: Some("https://open.bigmodel.cn/api/paas/v4".to_string()),
        timeout_ms: None,
    }),
    // ... other providers
    ..Default::default()
};

let client = Client::with_config(config);
```

### DeepSeek Specific Features

DeepSeek provides two main models:

```rust
use llm_connector::{Client, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env();

    // DeepSeek Chat - General conversation model
    let chat_request = ChatRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful AI assistant.".to_string(),
                ..Default::default()
            },
            Message {
                role: "user".to_string(),
                content: "Explain quantum computing in simple terms.".to_string(),
                ..Default::default()
            }
        ],
        temperature: Some(0.7),
        max_tokens: Some(200),
        ..Default::default()
    };

    let response = client.chat(chat_request).await?;
    println!("DeepSeek Chat: {}", response.choices[0].message.content);

    // DeepSeek Reasoner - Advanced reasoning model
    let reasoner_request = ChatRequest {
        model: "deepseek-reasoner".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Solve this step by step: If a train travels 120 km in 2 hours, what's its speed?".to_string(),
                ..Default::default()
            }
        ],
        temperature: Some(0.1), // Lower temperature for reasoning tasks
        max_tokens: Some(300),
        ..Default::default()
    };

    let reasoning_response = client.chat(reasoner_request).await?;
    println!("DeepSeek Reasoner: {}", reasoning_response.choices[0].message.content);

    Ok(())
}
```

### Zhipu GLM Specific Features

Zhipu GLM provides several models optimized for different use cases:

```rust
use llm_connector::{Client, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env();

    // GLM-4 - General purpose model
    let glm4_request = ChatRequest {
        model: "glm-4".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful AI assistant.".to_string(),
                ..Default::default()
            },
            Message {
                role: "user".to_string(),
                content: "Explain machine learning in simple terms.".to_string(),
                ..Default::default()
            }
        ],
        temperature: Some(0.7),
        max_tokens: Some(200),
        ..Default::default()
    };

    let response = client.chat(glm4_request).await?;
    println!("GLM-4: {}", response.choices[0].message.content);

    // GLM-4-Plus - Enhanced model with better performance
    let glm4_plus_request = ChatRequest {
        model: "glm-4-plus".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Write a creative story about a robot learning to paint.".to_string(),
                ..Default::default()
            }
        ],
        temperature: Some(0.8),
        max_tokens: Some(500),
        ..Default::default()
    };

    let plus_response = client.chat(glm4_plus_request).await?;
    println!("GLM-4-Plus: {}", plus_response.choices[0].message.content);

    // GLM-4-Flash - Fast model for quick responses
    let glm4_flash_request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "What's the capital of France?".to_string(),
                ..Default::default()
            }
        ],
        temperature: Some(0.1),
        max_tokens: Some(50),
        ..Default::default()
    };

    let flash_response = client.chat(glm4_flash_request).await?;
    println!("GLM-4-Flash: {}", flash_response.choices[0].message.content);

    Ok(())
}
```

### Streaming

```rust
use futures_util::StreamExt;
use llm_connector::{Client, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env();

    let request = ChatRequest {
        model: "openai/gpt-4".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Tell me a story".to_string(),
                ..Default::default()
            }
        ],
        stream: Some(true),
        ..Default::default()
    };

    let mut stream = client.chat_stream(request).await?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(choice) = chunk.choices.first() {
            if let Some(content) = &choice.delta.content {
                print!("{}", content);
            }
        }
    }

    Ok(())
}
```

## Model Naming Convention

Use the format `provider/model` for explicit provider selection:

```rust
// Explicit provider selection (recommended)
"openai/gpt-4"
"anthropic/claude-3-5-sonnet-20241022"
"deepseek/deepseek-chat"
"zhipu/glm-4"
"qwen/qwen-turbo"
"kimi/moonshot-v1-8k"

// Direct model names (auto-detected)
"gpt-4"           // -> openai/gpt-4
"claude-3-haiku"  // -> anthropic/claude-3-haiku
"deepseek-chat"   // -> deepseek/deepseek-chat
```

## Error Handling

The library provides structured error types:

```rust
use llm_connector::{Client, LlmConnectorError};

match client.chat(request).await {
    Ok(response) => println!("Success: {}", response.choices[0].message.content),
    Err(LlmConnectorError::AuthenticationError(msg)) => {
        eprintln!("Auth error: {}", msg);
    },
    Err(LlmConnectorError::RateLimitError(msg)) => {
        eprintln!("Rate limited: {}", msg);
    },
    Err(LlmConnectorError::NetworkError(msg)) => {
        eprintln!("Network error: {}", msg);
    },
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Extending with New Providers

Implement the `Provider` trait to add support for new LLM providers:

```rust
use llm_connector::{Provider, ProviderConfig, ChatRequest, ChatResponse};
use async_trait::async_trait;

struct MyCustomProvider {
    config: ProviderConfig,
}

#[async_trait]
impl Provider for MyCustomProvider {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        // Implement your provider's API call here
        todo!()
    }

    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        // Implement streaming support
        todo!()
    }
}
```

## Design Philosophy

**llm-connector** follows the Unix philosophy: "Do one thing and do it well."

- **Single Responsibility**: Only handles protocol adaptation between LLM providers
- **Minimal Dependencies**: Keeps the dependency tree small and focused
- **Composable**: Designed to be used as a building block in larger systems
- **No Magic**: Explicit configuration and clear error messages
- **Provider Agnostic**: Treats all providers equally, no special cases

## What's NOT Included (By Design)

If you need these features, consider these alternatives:

- **Load Balancing**: Use nginx, HAProxy, or a service mesh
- **Rate Limiting**: Use Redis-based rate limiters or API gateways
- **Caching**: Use Redis, Memcached, or HTTP caching proxies
- **Monitoring**: Use Prometheus, Grafana, or APM solutions
- **Circuit Breaking**: Use Hystrix-style libraries or service mesh features
- **Request Queuing**: Use message queues like RabbitMQ or Apache Kafka

## Contributing

We welcome contributions! Please focus on:

1. **Adding new providers** - Implement the `Provider` trait
2. **Improving protocol compatibility** - Better OpenAI API compliance
3. **Bug fixes** - Especially around streaming and error handling
4. **Documentation** - Examples and provider-specific notes

Please **avoid** adding features outside the core scope (load balancing, complex routing, etc.).

## License

MIT
