# Supported Providers

`llm-connector` supports 12+ LLM providers with a unified interface.

## Provider Overview

| Provider | Service Name | Struct | API Format |
|----------|--------------|--------|------------|
| **OpenAI** | openai | `OpenAIProvider` | Native |
| **Anthropic Claude** | anthropic | `AnthropicProvider` | Native |
| **Google Gemini** | google | `GoogleProvider` | Native |
| **Aliyun DashScope** | aliyun | `AliyunProvider` | Custom |
| **Zhipu GLM** | zhipu | `ZhipuProvider` | Native/OpenAI |
| **Tencent Hunyuan** | tencent | `TencentProvider` | Native V3 |
| **Volcengine** | volcengine | `VolcengineProvider` | OpenAI Compatible |
| **DeepSeek** | deepseek | `DeepSeekProvider` | OpenAI Compatible |
| **Moonshot** | moonshot | `MoonshotProvider` | OpenAI Compatible |
| **Xiaomi MiMo** | xiaomi | `XiaomiProvider` | OpenAI Compatible |
| **Ollama** | ollama | `OllamaProvider` | Native |
| **LongCat** | longcat | `LongCatAnthropicProvider` | OpenAI/Anthropic |

---

## OpenAI

Standard OpenAI API support.

### Usage
```rust
use llm_connector::LlmClient;

let client = LlmClient::openai("sk-...")?;
// Or with custom base URL
let client = LlmClient::openai_with_base_url("sk-...", "https://api.openai.com")?;
```

---

## Anthropic Claude

Native support for Anthropic's Claude API.

### Usage
```rust
use llm_connector::LlmClient;

let client = LlmClient::anthropic("sk-ant-...")?;
```

### AWS Bedrock / Google Vertex AI
```rust
// AWS Bedrock
let client = LlmClient::anthropic_bedrock("us-east-1", "access_key", "secret_key")?;

// Google Vertex AI
let client = LlmClient::anthropic_vertex("project-id", "us-central1", "access-token")?;
```

---

## Aliyun DashScope (Qwen)

Support for Alibaba Cloud's Qwen models.

### Usage
```rust
use llm_connector::LlmClient;

let client = LlmClient::aliyun("sk-...")?;
```

### Tool Calling
Aliyun provider supports full tool calling (function calling) capabilities, compatible with OpenAI's format.

---

## Zhipu GLM (Zhipu AI)

Support for ChatGLM models.

### Usage
```rust
use llm_connector::LlmClient;

// Native SDK style
let client = LlmClient::zhipu("your-api-key")?;

// OpenAI Compatible Mode (Recommended for some models)
let client = LlmClient::zhipu_openai_compatible("your-api-key")?;
```

---

## Tencent Hunyuan (Hunyuan)

Native support for Tencent Cloud API v3 (TC3-HMAC-SHA256), including Streaming support.

### Usage
```rust
use llm_connector::LlmClient;

// Requires SecretId and SecretKey
let client = LlmClient::tencent("AKID...", "SecretKey...")?;
```

> [!NOTE]
> As of v0.5.8, this uses the native Tencent Protocol. Previous versions used an OpenAI-compatible wrapper.

---

## Volcengine

Support for Doubao models via Volcengine Ark.

### Usage
```rust
use llm_connector::LlmClient;

// Uses API Key (UUID format)
let client = LlmClient::volcengine("your-api-key")?;
```

---

## DeepSeek

Support for DeepSeek-V3 and R1 reasoning models.

### Usage
```rust
use llm_connector::LlmClient;

let client = LlmClient::deepseek("sk-...")?;
```

### Reasoning Models
DeepSeek R1 models output reasoning content. `llm-connector` automatically handles this:
- Non-streaming: Content is in `response.content` (reasoning usually stripped or separated depending on config)
- Streaming: `reasoning_content` is extracted from the stream.

---

## Moonshot

Support for Kimi models.

### Usage
```rust
use llm_connector::LlmClient;

let client = LlmClient::moonshot("sk-...")?;
```

---

## Xiaomi MiMo

Support for Xiaomi's MiMo LLM platform.

### Usage
```rust
use llm_connector::LlmClient;

let client = LlmClient::xiaomi("sk-...")?;

// With custom config
let client = LlmClient::xiaomi_with_config(
    "sk-...",
    None,        // Use default: https://api.xiaomimimo.com/v1
    Some(60),    // 60 second timeout
    None         // No proxy
)?;
```

### Models
- `mimo-v2-flash` - Fast model for general use

---

## Ollama

Support for local models via Ollama.

### Usage
```rust
use llm_connector::LlmClient;

// Default (http://localhost:11434)
let client = LlmClient::ollama()?;

// Custom URL
let client = LlmClient::ollama_with_base_url("http://192.168.1.100:11434")?;
```

### Model Management

Ollama provider supports full CRUD operations for local models:

```rust
use llm_connector::providers::ollama;

let client = ollama::ollama()?;

// List all local models
let models = client.list_models().await?;
for model in &models {
    println!("{}: {} bytes", model.name, model.size);
}

// Pull a new model
let status = client.pull_model("llama3:8b").await?;
println!("Pull status: {}", status.status);

// Delete a model
client.delete_model("llama3:8b").await?;

// Copy/rename a model
client.copy_model("llama3:8b", "my-llama").await?;

// Show model details
let info = client.show_model("llama3:8b").await?;
println!("License: {}", info.license.unwrap_or_default());
```

---

## Google Gemini

Support for Google's Gemini models.

### Usage
```rust
use llm_connector::LlmClient;

let client = LlmClient::google("your-api-key")?;

// With custom config
let client = LlmClient::google_with_config(
    "your-api-key",
    Some("https://generativelanguage.googleapis.com"),
    Some(120),  // 120 second timeout
    None        // No proxy
)?;
```

### Models
- `gemini-2.0-flash` - Fast and capable
- `gemini-1.5-pro` - Most capable
- `gemini-1.5-flash` - Fast responses

---

## Generic/Custom Providers

For any other OpenAI-compatible provider:

```rust
use llm_connector::LlmClient;

let client = LlmClient::openai_compatible(
    "api-key",
    "https://api.example.com",
    "provider-name"
)?;
```

---

## Environment Variables

| Provider | Environment Variable |
|----------|---------------------|
| OpenAI | `OPENAI_API_KEY` |
| Anthropic | `ANTHROPIC_API_KEY` |
| Aliyun | `DASHSCOPE_API_KEY` |
| Zhipu | `ZHIPU_API_KEY` |
| DeepSeek | `DEEPSEEK_API_KEY` |
| Moonshot | `MOONSHOT_API_KEY` |
| Xiaomi | `XIAOMI_API_KEY` |
| Google | `GOOGLE_API_KEY` |
| Tencent | `TENCENT_SECRET_ID`, `TENCENT_SECRET_KEY` |
| Volcengine | `VOLCENGINE_API_KEY` |
