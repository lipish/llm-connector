# Providers

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

## Google Gemini

Support for Google's Gemini models.

### Usage
```rust
use llm_connector::LlmClient;

let client = LlmClient::google("your-api-key")?;
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
