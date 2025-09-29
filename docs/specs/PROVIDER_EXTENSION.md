# Provider Extension Guide

如何为 llm-connector 添加新的 LLM 提供商支持。

## 设计原则

- **统一接口**：所有提供商实现相同的 Provider trait
- **协议转换**：将提供商 API 转换为 OpenAI 兼容格式
- **错误映射**：统一错误处理和重试策略
- **配置一致**：使用相同的配置模式

## Streaming 适配规范（阶段一/阶段二）

- 解析器选择：
  - SSE Provider：在 chat_stream 使用 `utils::streaming::sse_data_events`，严格按事件边界聚合多 data 行、CRLF 归一化，忽略 "[DONE]" 完成帧（阶段一）。
  - NDJSON Provider：在 chat_stream 使用 `utils::streaming::ndjson_events`（阶段一最小实现），逐行 JSON，忽略空行。
- 终止语义：
  - 阶段一：以“连接正常结束 = 完成”为主；如提供商提供 `finish_reason`，在最后帧进行标记；不启用流级重试与空闲超时。
  - 阶段二：引入 CompletionSignal 抽象（Marker/JsonField/ConnectionClosed），由 Orchestrator 在 feature flag 下管理早关闭重试与 idle 超时。
- 转换规则：
  - 将 Provider 原始帧转换为内部 StreamingResponse/StreamingChoice/Delta；delta.role 仅首帧、delta.content 累积追加、tool_calls.arguments 增量透传、usage 仅可能在最后帧。
- 测试要求：
  - SSE：覆盖多 data 行聚合、CRLF 归一化、半包/粘包、忽略 "[DONE]"。
  - NDJSON：覆盖空行忽略、长行处理。
  - 集成：至少 1 个真实或本地兼容 Provider 端到端用例（DeepSeek 优先）。

## Provider Trait

所有提供商必须实现 `Provider` trait：

```rust
use async_trait::async_trait;
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse};

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

#[async_trait]
pub trait Provider: Send + Sync {
    /// 获取提供商名称
    fn name(&self) -> &str;
    
    /// 获取支持的模型列表
    fn supported_models(&self) -> Vec<String>;
    
    /// 检查是否支持指定模型
    fn supports_model(&self, model: &str) -> bool {
        self.supported_models().iter().any(|m| m == model)
    }
    
    /// 发送聊天完成请求
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError>;
    
    /// 发送流式聊天完成请求
    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError>;
}
```

## 实现步骤

### 1. 创建提供商结构体

```rust
// src/providers/example.rs
use crate::config::ProviderConfig;
use crate::error::LlmConnectorError;

pub struct ExampleProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl ExampleProvider {
    pub fn new(config: ProviderConfig) -> Result<Self, LlmConnectorError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(
                config.effective_timeout_ms()
            ))
            .build()
            .map_err(|e| LlmConnectorError::ConfigError(
                format!("Failed to create HTTP client: {}", e)
            ))?;
        
        Ok(Self { config, client })
    }
}
```

### 2. 实现 Provider trait

```rust
use async_trait::async_trait;
use crate::providers::Provider;

#[async_trait]
impl Provider for ExampleProvider {
    fn name(&self) -> &str {
        "example"
    }
    
    fn supported_models(&self) -> Vec<String> {
        vec![
            "example-chat".to_string(),
            "example-instruct".to_string(),
        ]
    }
    
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        // 1. 验证请求
        self.validate_request(request)?;
        
        // 2. 转换请求格式
        let provider_request = self.convert_request(request)?;
        
        // 3. 发送请求
        let response = self.send_request(provider_request).await?;
        
        // 4. 转换响应格式
        let chat_response = self.convert_response(response)?;
        
        Ok(chat_response)
    }
    
    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        // 流式实现
        todo!("Implement streaming")
    }
}
```

### 3. 实现辅助方法

```rust
impl ExampleProvider {
    fn validate_request(&self, request: &ChatRequest) -> Result<(), LlmConnectorError> {
        if !self.supports_model(&request.model) {
            return Err(LlmConnectorError::UnsupportedModel(
                format!("Model '{}' not supported by {}", request.model, self.name())
            ));
        }
        
        // 其他验证逻辑
        Ok(())
    }
    
    fn convert_request(&self, request: &ChatRequest) -> Result<ExampleRequest, LlmConnectorError> {
        // 将 ChatRequest 转换为提供商特定的请求格式
        Ok(ExampleRequest {
            model: request.model.clone(),
            messages: request.messages.iter().map(|m| ExampleMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            }).collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            // ... 其他字段映射
        })
    }
    
    async fn send_request(&self, request: ExampleRequest) -> Result<ExampleResponse, LlmConnectorError> {
        let url = format!("{}/chat/completions", 
            self.config.effective_base_url("example").unwrap()
        );
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await);
        }
        
        let example_response: ExampleResponse = response
            .json()
            .await
            .map_err(|e| LlmConnectorError::ProviderError(
                format!("Failed to parse response: {}", e)
            ))?;
        
        Ok(example_response)
    }
    
    fn convert_response(&self, response: ExampleResponse) -> Result<ChatResponse, LlmConnectorError> {
        // 将提供商响应转换为标准 ChatResponse 格式
        Ok(ChatResponse {
            id: response.id,
            object: "chat.completion".to_string(),
            created: response.created,
            model: response.model,
            choices: response.choices.into_iter().map(|c| Choice {
                index: c.index,
                message: Message {
                    role: c.message.role,
                    content: c.message.content,
                    ..Default::default()
                },
                finish_reason: c.finish_reason,
                ..Default::default()
            }).collect(),
            usage: response.usage.map(|u| Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
                ..Default::default()
            }),
            ..Default::default()
        })
    }
    
    async fn handle_error_response(&self, response: reqwest::Response) -> LlmConnectorError {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        
        match status.as_u16() {
            401 => LlmConnectorError::AuthenticationError(
                "Invalid API key".to_string()
            ),
            429 => LlmConnectorError::RateLimitError(
                "Rate limit exceeded".to_string()
            ),
            500..=599 => LlmConnectorError::ProviderError(
                format!("Server error: {}", text)
            ),
            _ => LlmConnectorError::ProviderError(
                format!("HTTP {}: {}", status, text)
            ),
        }
    }
}
```

### 4. 定义提供商特定类型

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct ExampleRequest {
    model: String,
    messages: Vec<ExampleMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
struct ExampleMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ExampleResponse {
    id: String,
    created: u64,
    model: String,
    choices: Vec<ExampleChoice>,
    usage: Option<ExampleUsage>,
}

#[derive(Debug, Deserialize)]
struct ExampleChoice {
    index: u32,
    message: ExampleMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExampleUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}
```

### 5. 注册提供商

在 `src/providers/mod.rs` 中添加：

```rust
#[cfg(feature = "reqwest")]
pub mod example;
```

在 `src/client.rs` 的 `initialize_providers` 方法中添加：

```rust
// Initialize Example provider
if let Some(example_config) = &self.config.example {
    if let Ok(provider) = example::ExampleProvider::new(example_config.clone()) {
        self.providers.insert("example".to_string(), Arc::new(provider));
    }
}
```

在 `src/config.rs` 的 `Config` 结构体中添加：

```rust
/// Example configuration
#[serde(skip_serializing_if = "Option::is_none")]
pub example: Option<ProviderConfig>,
```

在 `Config::from_env()` 方法中添加：

```rust
// Example
if let Ok(api_key) = env::var("EXAMPLE_API_KEY") {
    config.example = Some(ProviderConfig {
        api_key,
        base_url: env::var("EXAMPLE_BASE_URL").ok(),
        timeout_ms: env::var("EXAMPLE_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse().ok()),
    });
}
```

### 6. 更新模型检测

在 `src/providers/base.rs` 的 `detect_provider_from_model` 函数中添加：

```rust
} else if model.starts_with("example") {
    Some("example")
```

在 `ProviderConfig::default_base_url` 方法中添加：

```rust
"example" => Some("https://api.example.com/v1".to_string()),
```

## 流式实现

对于支持流式响应的提供商：

```rust
#[cfg(feature = "streaming")]
async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
    use futures_util::stream::StreamExt;
    use tokio_stream::wrappers::LinesStream;
    
    let mut request = self.convert_request(request)?;
    request.stream = Some(true);
    
    let url = format!("{}/chat/completions", 
        self.config.effective_base_url("example").unwrap()
    );
    
    let response = self.client
        .post(&url)
        .header("Authorization", format!("Bearer {}", self.config.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(self.handle_error_response(response).await);
    }
    
    let stream = response
        .bytes_stream()
        .map(|chunk| {
            chunk.map_err(|e| LlmConnectorError::StreamingError(e.to_string()))
        })
        .and_then(|chunk| async move {
            // 解析 SSE 数据
            self.parse_sse_chunk(&chunk)
        });
    
    Ok(Box::pin(stream))
}

fn parse_sse_chunk(&self, chunk: &[u8]) -> Result<StreamingResponse, LlmConnectorError> {
    // 解析 Server-Sent Events 格式
    let text = std::str::from_utf8(chunk)
        .map_err(|e| LlmConnectorError::StreamingError(e.to_string()))?;
    
    // 处理 SSE 格式：data: {...}
    if let Some(data) = text.strip_prefix("data: ") {
        if data.trim() == "[DONE]" {
            // 流结束标记
            return Err(LlmConnectorError::StreamingError("Stream ended".to_string()));
        }
        
        let response: ExampleStreamResponse = serde_json::from_str(data)
            .map_err(|e| LlmConnectorError::StreamingError(e.to_string()))?;
        
        // 转换为标准格式
        Ok(self.convert_stream_response(response)?)
    } else {
        Err(LlmConnectorError::StreamingError("Invalid SSE format".to_string()))
    }
}
```

## 测试

为新提供商编写测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProviderConfig;
    
    fn create_test_provider() -> ExampleProvider {
        let config = ProviderConfig {
            api_key: "test-key".to_string(),
            base_url: Some("https://api.example.com/v1".to_string()),
            timeout_ms: Some(30000),
        };
        ExampleProvider::new(config).unwrap()
    }
    
    #[test]
    fn test_provider_name() {
        let provider = create_test_provider();
        assert_eq!(provider.name(), "example");
    }
    
    #[test]
    fn test_supported_models() {
        let provider = create_test_provider();
        let models = provider.supported_models();
        assert!(models.contains(&"example-chat".to_string()));
    }
    
    #[test]
    fn test_model_support() {
        let provider = create_test_provider();
        assert!(provider.supports_model("example-chat"));
        assert!(!provider.supports_model("unknown-model"));
    }
    
    #[tokio::test]
    async fn test_request_conversion() {
        let provider = create_test_provider();
        let request = ChatRequest {
            model: "example-chat".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        
        let converted = provider.convert_request(&request).unwrap();
        assert_eq!(converted.model, "example-chat");
        assert_eq!(converted.messages.len(), 1);
    }
}
```

## 最佳实践

### 1. 错误处理
- 正确映射 HTTP 状态码到 LlmConnectorError
- 保留原始错误信息用于调试
- 实现适当的重试策略

### 2. 配置管理
- 使用统一的 ProviderConfig 结构
- 支持环境变量配置
- 提供合理的默认值

### 3. 类型转换
- 仔细映射请求和响应字段
- 处理可选字段和默认值
- 保持与 OpenAI API 的兼容性

### 4. 性能优化
- 复用 HTTP 客户端
- 实现连接池
- 合理设置超时时间

### 5. 文档
- 提供清晰的使用示例
- 说明支持的功能和限制
- 记录配置要求
