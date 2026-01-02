# Supported Providers

`llm-connector` supports a wide range of LLM providers with a unified interface.

## Provider Overview

| Provider | Service Name | Struct | API Format |
|----------|--------------|--------|------------|
| **OpenAI** | openai | `OpenAIProvider` | Native |
| **Aliyun DashScope** | aliyun | `AliyunProvider` | Custom |
| **Anthropic Claude** | anthropic | `AnthropicProvider` | Native |
| **Zhipu GLM** | zhipu | `ZhipuProvider` | Native/OpenAI |
| **Ollama** | ollama | `OllamaProvider` | Native |
| **Tencent Hunyuan** | tencent | `TencentProvider` | Native V3 |
| **Volcengine** | volcengine | `VolcengineProvider` | OpenAI Compatible |
| **LongCat** | longcat | `LongCatAnthropicProvider` | OpenAI/Anthropic |
| **DeepSeek** | deepseek | `DeepSeekProvider` | OpenAI Compatible |
| **Moonshot** | moonshot | `MoonshotProvider` | OpenAI Compatible |

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

## Aliyun DashScope (通义千问)

Support for Alibaba Cloud's Qwen models.

### Usage
```rust
use llm_connector::LlmClient;

let client = LlmClient::aliyun("sk-...")?;
```

### Tool Calling
Aliyun provider supports full tool calling (function calling) capabilities, compatible with OpenAI's format.

---

## Zhipu GLM (智谱AI)

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

## Tencent Hunyuan (腾讯混元)

Native support for Tencent Cloud API v3 (TC3-HMAC-SHA256).

### Usage
```rust
use llm_connector::LlmClient;

// Requires SecretId and SecretKey
let client = LlmClient::tencent("AKID...", "SecretKey...")?;
```

> [!NOTE]
> As of v0.5.8, this uses the native Tencent Protocol. Previous versions used an OpenAI-compatible wrapper.

---

## Volcengine (火山引擎)

Support for Doubao models via Volcengine Ark.

### Usage
```rust
use llm_connector::LlmClient;

// Uses API Key (UUID format)
let client = LlmClient::volcengine("your-api-key")?;
```

---

## DeepSeek (深度求索)

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

## Moonshot (月之暗面)

Support for Kimi models.

### Usage
```rust
use llm_connector::LlmClient;

let client = LlmClient::moonshot("sk-...")?;
```

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
