# Configuration Guide

This guide explains all the ways to configure llm-connector and manage API keys.

## Table of Contents

- [Environment Variables](#environment-variables)
- [Configuration Files](#configuration-files)
- [Programmatic Configuration](#programmatic-configuration)
- [Factory Pattern](#factory-pattern)
- [Best Practices](#best-practices)

---

## Environment Variables

The simplest way to configure llm-connector is through environment variables.

### Supported Variables

```bash
# DeepSeek
export DEEPSEEK_API_KEY="sk-..."
export DEEPSEEK_BASE_URL="https://api.deepseek.com/v1"  # Optional
export DEEPSEEK_TIMEOUT_MS="30000"  # Optional

# Zhipu (GLM)
export ZHIPU_API_KEY="..."
export GLM_API_KEY="..."  # Alternative name
export ZHIPU_BASE_URL="https://open.bigmodel.cn/api/paas/v4"  # Optional

# Aliyun (DashScope)
export ALIYUN_API_KEY="sk-..."
export DASHSCOPE_API_KEY="sk-..."  # Alternative name
export ALIYUN_BASE_URL="..."  # Optional

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."
export ANTHROPIC_BASE_URL="https://api.anthropic.com"  # Optional

# LongCat
export LONGCAT_API_KEY="..."
export LONGCAT_BASE_URL="https://api.longcat.chat/openai"  # Optional

# Moonshot (Kimi)
export MOONSHOT_API_KEY="sk-..."

# VolcEngine (Doubao)
export VOLCENGINE_API_KEY="..."

# Tencent (Hunyuan)
export TENCENT_API_KEY="..."

# MiniMax
export MINIMAX_API_KEY="..."

# StepFun
export STEPFUN_API_KEY="..."
```

### Usage

```rust
use llm_connector::config::Config;

// Load from environment variables
let config = Config::from_env();

// Check which providers are configured
let providers = config.list_providers();
println!("Configured providers: {:?}", providers);
```

---

## Configuration Files

For more complex setups, use configuration files.

### JSON Format

Create `config.json`:

```json
{
  "providers": {
    "deepseek": {
      "protocol": "openai",
      "api_key": "sk-...",
      "base_url": "https://api.deepseek.com/v1",
      "timeout_ms": 30000,
      "retry": {
        "max_retries": 3,
        "initial_backoff_ms": 1000,
        "backoff_multiplier": 2.0,
        "max_backoff_ms": 30000
      },
      "headers": {
        "X-Request-Source": "my-app"
      },
      "max_concurrent_requests": 10
    },
    "claude": {
      "protocol": "anthropic",
      "api_key": "sk-ant-...",
      "timeout_ms": 60000,
      "retry": {
        "max_retries": 5,
        "initial_backoff_ms": 2000,
        "backoff_multiplier": 2.0,
        "max_backoff_ms": 60000
      }
    },
    "qwen": {
      "protocol": "aliyun",
      "api_key": "sk-...",
      "timeout_ms": 30000
    }
  }
}
```

### TOML Format (requires `toml` feature)

Create `config.toml`:

```toml
[providers.deepseek]
protocol = "openai"
api_key = "sk-..."
base_url = "https://api.deepseek.com/v1"
timeout_ms = 30000

[providers.deepseek.retry]
max_retries = 3
initial_backoff_ms = 1000
backoff_multiplier = 2.0
max_backoff_ms = 30000

[providers.deepseek.headers]
X-Request-Source = "my-app"

[providers.claude]
protocol = "anthropic"
api_key = "sk-ant-..."
timeout_ms = 60000
```

### YAML Format (requires `yaml` feature)

Create `config.yaml`:

```yaml
providers:
  deepseek:
    protocol: openai
    api_key: sk-...
    base_url: https://api.deepseek.com/v1
    timeout_ms: 30000
    retry:
      max_retries: 3
      initial_backoff_ms: 1000
      backoff_multiplier: 2.0
      max_backoff_ms: 30000
    headers:
      X-Request-Source: my-app
  
  claude:
    protocol: anthropic
    api_key: sk-ant-...
    timeout_ms: 60000
```

### Loading Configuration Files

llm-connector provides multiple ways to load configuration files:

#### Option 1: Load from Specific Path

```rust
use llm_connector::config::RegistryConfig;
use llm_connector::registry::ProviderRegistry;

// Load from any path (auto-detects format by extension)
let config = RegistryConfig::from_file("./config.json")?;
let config = RegistryConfig::from_file("/etc/llm-connector/config.yaml")?;
let config = RegistryConfig::from_file("~/my-config.toml")?;  // ~ expansion supported

// Or specify format explicitly
let config = RegistryConfig::from_json_file("config.json")?;
let config = RegistryConfig::from_toml_file("config.toml")?;
let config = RegistryConfig::from_yaml_file("config.yaml")?;

// Create registry from config
let registry = ProviderRegistry::from_config(config)?;
```

#### Option 2: Load from Environment Variable

Set the `LLM_CONNECTOR_CONFIG` environment variable:

```bash
export LLM_CONNECTOR_CONFIG="./config.json"
```

Then in your code:

```rust
// Automatically uses LLM_CONNECTOR_CONFIG if set
if let Some(config) = RegistryConfig::from_env_or_default() {
    let registry = ProviderRegistry::from_config(config)?;
}
```

#### Option 3: Auto-Discovery from Default Paths

llm-connector automatically searches for config files in these locations (in order):

1. `LLM_CONNECTOR_CONFIG` environment variable
2. `./llm-connector.config.json`
3. `./llm-connector.config.yaml`
4. `./llm-connector.config.toml`
5. `~/.config/llm-connector/config.json`
6. `~/.config/llm-connector/config.yaml`
7. `~/.config/llm-connector/config.toml`

```rust
// Automatically finds config from default paths
if let Some(config) = RegistryConfig::from_env_or_default() {
    let registry = ProviderRegistry::from_config(config)?;
} else {
    // Fall back to programmatic configuration
    let config = RegistryConfig::new()
        .add_provider("deepseek", "openai", ProviderConfig::new("key"));
}
```

#### Option 4: Hybrid Approach (Recommended)

Try a custom path first, then fall back to defaults:

```rust
// Try custom path first, then fall back to defaults
let config = RegistryConfig::from_path_or_default("./my-config.json");

// Or with command-line argument
let config_path = std::env::args().nth(1);
let config = if let Some(path) = config_path {
    RegistryConfig::from_path_or_default(path)
} else {
    RegistryConfig::from_env_or_default()
};
```

---

## Programmatic Configuration

For maximum flexibility, configure providers programmatically.

### Basic Configuration

```rust
use llm_connector::config::ProviderConfig;

let config = ProviderConfig::new("your-api-key");
```

### Builder Pattern

```rust
use llm_connector::config::{ProviderConfig, RetryConfig};
use std::collections::HashMap;

let mut headers = HashMap::new();
headers.insert("X-Custom-Header".to_string(), "value".to_string());

let config = ProviderConfig::new("your-api-key")
    .with_base_url("https://api.example.com/v1")
    .with_timeout_ms(30000)
    .with_proxy("http://proxy.example.com:8080")
    .with_retry(RetryConfig {
        max_retries: 3,
        initial_backoff_ms: 1000,
        backoff_multiplier: 2.0,
        max_backoff_ms: 30000,
    })
    .with_headers(headers)
    .with_max_concurrent_requests(10);
```

### Creating Providers

```rust
use llm_connector::protocols::{
    core::GenericProvider,
    openai::deepseek,
};

let provider = GenericProvider::new(config, deepseek())?;
```

---

## Factory Pattern

Use the factory pattern for dynamic provider creation.

### Basic Usage

```rust
use llm_connector::{
    config::ProviderConfig,
    protocols::factory::ProtocolFactoryRegistry,
};

// Create factory registry with default factories
let registry = ProtocolFactoryRegistry::with_defaults();

// List all supported providers
let providers = registry.list_providers();
println!("Supported: {:?}", providers);

// Create provider dynamically
let config = ProviderConfig::new("your-api-key");
let adapter = registry.create_for_provider("deepseek", &config)?;
```

### Custom Factory

```rust
use llm_connector::protocols::factory::{ProtocolFactory, ProtocolFactoryRegistry};
use std::sync::Arc;

// Create custom factory
struct MyFactory;

impl ProtocolFactory for MyFactory {
    fn protocol_name(&self) -> &str {
        "my-protocol"
    }
    
    fn supported_providers(&self) -> Vec<&str> {
        vec!["my-provider"]
    }
    
    fn create_adapter(&self, provider_name: &str, config: &ProviderConfig) 
        -> Result<Box<dyn std::any::Any + Send>, LlmConnectorError> 
    {
        // Create your custom adapter
        Ok(Box::new(MyAdapter::new(config)))
    }
}

// Register custom factory
let registry = ProtocolFactoryRegistry::new();
registry.register(Arc::new(MyFactory));
```

---

## Best Practices

### 1. Use Environment Variables for Development

```bash
# .env file (use with dotenv crate)
DEEPSEEK_API_KEY=sk-...
ZHIPU_API_KEY=...
ALIYUN_API_KEY=sk-...
```

```rust
// Load .env file
dotenv::dotenv().ok();

// Use environment variables
let config = Config::from_env();
```

### 2. Use Configuration Files for Production

```rust
// Load from file
let config = RegistryConfig::from_file("config.json")?;
let registry = ProviderRegistry::from_config(config)?;
```

### 3. Never Commit API Keys

Add to `.gitignore`:

```gitignore
# API keys
.env
config.json
config.toml
config.yaml
**/api_keys.txt
```

### 4. Use Retry Configuration

```rust
let config = ProviderConfig::new("api-key")
    .with_retry(RetryConfig {
        max_retries: 3,
        initial_backoff_ms: 1000,
        backoff_multiplier: 2.0,
        max_backoff_ms: 30000,
    });
```

### 5. Set Appropriate Timeouts

```rust
// For fast models
let config = ProviderConfig::new("api-key")
    .with_timeout_ms(10000);  // 10 seconds

// For slow models or long responses
let config = ProviderConfig::new("api-key")
    .with_timeout_ms(60000);  // 60 seconds
```

### 6. Use Custom Headers for Tracking

```rust
let config = ProviderConfig::new("api-key")
    .with_header("X-Request-ID", uuid::Uuid::new_v4().to_string())
    .with_header("X-App-Version", "1.0.0");
```

### 7. Monitor with Metrics

```rust
use llm_connector::middleware::MetricsMiddleware;

let metrics = MetricsMiddleware::new();

// Use metrics middleware
let response = metrics.execute(|| async {
    provider.chat(&request).await
}).await?;

// Check metrics periodically
let snapshot = metrics.snapshot();
if snapshot.success_rate < 95.0 {
    log::warn!("Low success rate: {:.2}%", snapshot.success_rate);
}
```

---

## Security Considerations

### 1. Protect API Keys

- Never commit API keys to version control
- Use environment variables or secure vaults
- Rotate keys regularly

### 2. Use HTTPS

All providers use HTTPS by default. Never override with HTTP.

### 3. Validate Configuration

```rust
// Validate before use
if config.api_key.is_empty() {
    return Err("API key is required");
}

if let Some(timeout) = config.timeout_ms {
    if timeout < 1000 {
        log::warn!("Timeout is very short: {}ms", timeout);
    }
}
```

### 4. Limit Concurrent Requests

```rust
let config = ProviderConfig::new("api-key")
    .with_max_concurrent_requests(10);  // Prevent overwhelming the API
```

---

## Troubleshooting

### Issue: API Key Not Found

**Solution**: Check environment variables or configuration file

```rust
// Debug: Print configured providers
let config = Config::from_env();
println!("Providers: {:?}", config.list_providers());
```

### Issue: Connection Timeout

**Solution**: Increase timeout

```rust
let config = ProviderConfig::new("api-key")
    .with_timeout_ms(60000);  // 60 seconds
```

### Issue: Rate Limiting

**Solution**: Use retry middleware

```rust
use llm_connector::middleware::RetryMiddleware;

let retry = RetryMiddleware::default();
let response = retry.execute(|| provider.chat(&request)).await?;
```

---

## Summary

llm-connector provides flexible configuration options:

✅ **Environment Variables** - Simple, good for development  
✅ **Configuration Files** - Structured, good for production  
✅ **Programmatic** - Flexible, good for dynamic scenarios  
✅ **Factory Pattern** - Extensible, good for plugins  

Choose the method that best fits your use case!

---

*Last updated: 2025-09-30*
