# LongCat Support 🐱

## Overview

llm-connector now supports [LongCat](https://longcat.chat), a Chinese LLM API platform that provides OpenAI and Anthropic-compatible APIs.

## Features

- ✅ **OpenAI-compatible API**: Use existing OpenAI protocol
- ✅ **Free Daily Quota**: 500,000 tokens/day (can increase to 5,000,000)
- ✅ **Multiple Models**: LongCat-Flash-Chat, LongCat-Flash-Thinking
- ✅ **Streaming Support**: Real-time response streaming
- ✅ **Automatic Retry**: Built-in retry mechanism for rate limits

## Quick Start

### 1. Get API Key

1. Register at [LongCat Platform](https://longcat.chat/platform/)
2. Navigate to [API Keys](https://longcat.chat/platform/api-keys)
3. Create a new API key

### 2. Basic Usage

```rust
use llm_connector::{
    config::ProviderConfig,
    protocols::{
        core::{Provider, GenericProvider},
        openai::longcat,
    },
    types::{ChatRequest, Message},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = ProviderConfig::new("your-longcat-api-key")
        .with_timeout_ms(30000);

    // Create provider
    let provider = GenericProvider::new(config, longcat())?;

    // Create request
    let request = ChatRequest {
        model: "LongCat-Flash-Chat".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "你好！".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }
        ],
        max_tokens: Some(1000),
        temperature: Some(0.7),
        ..Default::default()
    };

    // Send request
    let response = provider.chat(&request).await?;
    
    // Print response
    if let Some(choice) = response.choices.first() {
        println!("{}", choice.message.content);
    }

    Ok(())
}
```

### 3. With Retry Mechanism

```rust
use llm_connector::{
    config::{ProviderConfig, RetryConfig},
    middleware::RetryMiddleware,
    protocols::{
        core::{Provider, GenericProvider},
        openai::longcat,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration with retry
    let config = ProviderConfig::new("your-api-key")
        .with_retry(RetryConfig::default());

    let provider = GenericProvider::new(config, longcat())?;

    // Create retry middleware
    let retry = RetryMiddleware::default();

    // Execute with automatic retry
    let response = retry.execute(|| async {
        provider.chat(&request).await
    }).await?;

    Ok(())
}
```

### 4. Using Factory Pattern

```rust
use llm_connector::{
    config::ProviderConfig,
    protocols::factory::ProtocolFactoryRegistry,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ProtocolFactoryRegistry::with_defaults();
    
    let config = ProviderConfig::new("your-api-key");
    
    // Create provider dynamically
    let adapter = registry.create_for_provider("longcat", &config)?;

    Ok(())
}
```

## Supported Models

### 1. LongCat-Flash-Chat

- **Type**: General conversation model
- **Performance**: High-speed responses
- **Use Cases**: 
  - General chat
  - Q&A
  - Content generation
  - Translation

**Example**:
```rust
let request = ChatRequest {
    model: "LongCat-Flash-Chat".to_string(),
    messages: vec![
        Message {
            role: "user".to_string(),
            content: "请帮我写一首关于春天的诗".to_string(),
            ..Default::default()
        }
    ],
    ..Default::default()
};
```

### 2. LongCat-Flash-Thinking

- **Type**: Deep thinking model
- **Performance**: More thorough reasoning
- **Use Cases**:
  - Complex problem solving
  - Code analysis
  - Mathematical reasoning
  - Logical deduction

**Example**:
```rust
let request = ChatRequest {
    model: "LongCat-Flash-Thinking".to_string(),
    messages: vec![
        Message {
            role: "user".to_string(),
            content: "请分析这段代码的时间复杂度...".to_string(),
            ..Default::default()
        }
    ],
    ..Default::default()
};
```

## Configuration

### Environment Variables

```bash
export LONGCAT_API_KEY="your-api-key-here"
```

### Configuration File (JSON)

```json
{
  "providers": {
    "longcat": {
      "protocol": "openai",
      "api_key": "your-api-key",
      "base_url": "https://api.longcat.chat/openai",
      "timeout_ms": 30000,
      "retry": {
        "max_retries": 3,
        "initial_backoff_ms": 1000,
        "backoff_multiplier": 2.0,
        "max_backoff_ms": 30000
      }
    }
  }
}
```

### Load from Configuration

```rust
use llm_connector::config::RegistryConfig;

let config = RegistryConfig::from_file("config.json")?;
let registry = ProviderRegistry::from_config(config)?;
```

## Rate Limiting

LongCat implements rate limiting. When you exceed the rate limit, you'll receive:

```json
{
  "error": {
    "code": "rate_limit_exceeded",
    "message": "请求频率超限，请稍后重试",
    "type": "rate_limit_error",
    "retry_after": 60
  }
}
```

### Handling Rate Limits

The built-in retry middleware automatically handles rate limits:

```rust
use llm_connector::middleware::RetryMiddleware;

let retry = RetryMiddleware::default();

// Automatically retries on rate limit errors
let response = retry.execute(|| {
    provider.chat(&request)
}).await?;
```

## API Endpoints

LongCat provides two API formats:

### OpenAI Format (Recommended)
```
https://api.longcat.chat/openai
```

### Anthropic Format
```
https://api.longcat.chat/anthropic
```

llm-connector uses the OpenAI format by default for better compatibility.

## Quota Management

### Free Tier
- **Daily Quota**: 500,000 tokens
- **Can Increase To**: 5,000,000 tokens/day
- **Reset**: Daily at midnight (Beijing Time)

### Check Usage

Visit [Usage Dashboard](https://longcat.chat/platform/usage) to monitor your token usage.

## Best Practices

### 1. Use Retry Middleware

```rust
let config = ProviderConfig::new("api-key")
    .with_retry(RetryConfig {
        max_retries: 3,
        initial_backoff_ms: 1000,
        backoff_multiplier: 2.0,
        max_backoff_ms: 30000,
    });
```

### 2. Set Appropriate Timeouts

```rust
let config = ProviderConfig::new("api-key")
    .with_timeout_ms(30000); // 30 seconds
```

### 3. Monitor Token Usage

```rust
use llm_connector::middleware::MetricsMiddleware;

let metrics = MetricsMiddleware::new();

let response = metrics.execute(|| {
    provider.chat(&request)
}).await?;

// Check token usage
let snapshot = metrics.snapshot();
println!("Tokens used: {}", snapshot.tokens_total);
```

### 4. Handle Errors Gracefully

```rust
match provider.chat(&request).await {
    Ok(response) => {
        // Process response
    }
    Err(LlmConnectorError::RateLimitError(msg)) => {
        println!("Rate limited: {}", msg);
        // Wait and retry
    }
    Err(e) => {
        println!("Error: {}", e);
    }
}
```

## Examples

Run the LongCat demo:

```bash
# Set API key
export LONGCAT_API_KEY="your-api-key"

# Run demo
cargo run --example longcat_demo
```

## Troubleshooting

### Issue: Rate Limit Errors

**Solution**: Use RetryMiddleware with exponential backoff

```rust
let retry = RetryMiddleware::default();
```

### Issue: Timeout Errors

**Solution**: Increase timeout duration

```rust
let config = ProviderConfig::new("api-key")
    .with_timeout_ms(60000); // 60 seconds
```

### Issue: Invalid API Key

**Solution**: Verify your API key at [API Keys page](https://longcat.chat/platform/api-keys)

## Links

- 🌐 **Platform**: https://longcat.chat/platform/
- 📚 **Documentation**: https://longcat.chat/platform/docs/zh/
- 🔑 **API Keys**: https://longcat.chat/platform/api-keys
- 📊 **Usage Dashboard**: https://longcat.chat/platform/usage

## Summary

✅ LongCat is now fully supported in llm-connector  
✅ Uses OpenAI-compatible protocol  
✅ Easy integration with existing code  
✅ Free daily quota available  
✅ Production-ready with retry support  

---

*Last updated: 2025-09-30*
