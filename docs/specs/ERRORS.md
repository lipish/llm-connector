# Error Handling Specification

llm-connector 错误处理规范，专注于简单清晰的错误分类和处理。

## 设计原则

- **简单分类**：最小化错误类型复杂度
- **清晰映射**：提供商错误到标准错误的明确映射
- **可重试性**：明确区分可重试和不可重试错误
- **调试友好**：提供足够的上下文信息

## 错误类型

### 核心错误枚举

```rust
#[derive(Debug, thiserror::Error)]
pub enum LlmConnectorError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Unsupported model: {0}")]
    UnsupportedModel(String),
    
    #[error("Provider error: {0}")]
    ProviderError(String),
    
    #[error("Streaming error: {0}")]
    StreamingError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
```

## 错误分类详解

### 1. AuthenticationError
**认证相关错误**
- API 密钥无效或缺失
- 认证令牌过期
- 权限不足

**示例**：
- `"Invalid API key"`
- `"Missing API key for provider: openai"`
- `"API key expired"`

### 2. RateLimitError
**速率限制错误**
- 请求频率超限
- 配额用尽
- 并发限制

**示例**：
- `"Rate limit exceeded, retry after 60 seconds"`
- `"Monthly quota exceeded"`
- `"Too many concurrent requests"`

### 3. NetworkError
**网络相关错误**
- 连接超时
- DNS 解析失败
- 网络不可达

**示例**：
- `"Connection timeout after 30 seconds"`
- `"DNS resolution failed for api.openai.com"`
- `"Network unreachable"`

### 4. InvalidRequest
**请求格式错误**
- 参数缺失或无效
- 请求体格式错误
- 不支持的参数组合

**示例**：
- `"Missing required parameter: model"`
- `"Invalid temperature value: must be between 0.0 and 2.0"`
- `"Messages cannot be empty"`

### 5. UnsupportedModel
**模型不支持错误**
- 模型名称不存在
- 提供商不支持该模型
- 模型已下线

**示例**：
- `"Model 'gpt-5' not found"`
- `"Provider 'openai' does not support model 'claude-3'"`
- `"Model 'text-davinci-003' has been deprecated"`

### 6. ProviderError
**提供商服务错误**
- 提供商服务器错误
- 提供商维护中
- 提供商返回未知错误

**示例**：
- `"OpenAI server error: 500 Internal Server Error"`
- `"Anthropic service temporarily unavailable"`
- `"Provider returned unexpected response"`

### 7. StreamingError
**流式处理错误**
- 流连接中断
- 流数据格式错误
- 流处理超时

**示例**：
- `"Stream connection lost"`
- `"Invalid SSE data format"`
- `"Stream timeout after 60 seconds"`

### 8. ConfigError
**配置错误**
- 配置文件格式错误
- 必需配置缺失
- 配置值无效

**示例**：
- `"Missing API key for provider: openai"`
- `"Invalid timeout value: must be > 0"`
- `"Invalid base URL format"`

## 错误映射

### HTTP 状态码映射

| 错误类型 | HTTP 状态码 | 说明 |
|----------|-------------|------|
| AuthenticationError | 401 | 认证失败 |
| RateLimitError | 429 | 速率限制 |
| NetworkError | 502 | 网络错误 |
| InvalidRequest | 400 | 请求无效 |
| UnsupportedModel | 404 | 模型不存在 |
| ProviderError | 502 | 提供商错误 |
| StreamingError | 500 | 流式错误 |
| ConfigError | 500 | 配置错误 |

## 重试策略

### 可重试错误
- `RateLimitError`：遵循 Retry-After 头
- `NetworkError`：指数退避重试
- `ProviderError`：5xx 错误可重试

### 不可重试错误
- `AuthenticationError`：认证问题需要修复
- `InvalidRequest`：请求格式问题
- `UnsupportedModel`：模型不存在
- `ConfigError`：配置问题

### is_retryable() 判定规则
- 返回 true 的情况：
  - NetworkError（网络瞬时错误）
  - RateLimitError（依据 Retry-After）
  - ProviderError（5xx）
- 返回 false 的情况：AuthenticationError / InvalidRequest / UnsupportedModel / ConfigError / StreamingError（阶段一不做流级自动重试）

### 阶段一策略说明
- 仅进行“请求级重试”（初始请求失败），严格上限与指数退避。
- 流式过程中（建立成功后）的中断、超时不自动重试；由阶段二的编排层在 feature flag 下统一管理（idle/早关闭重试/退避）。

## 最佳实践

### 1. 错误处理
```rust
match client.chat(request).await {
    Ok(response) => {
        // 处理成功响应
    },
    Err(LlmConnectorError::RateLimitError(msg)) => {
        // 处理速率限制，可以重试
        eprintln!("Rate limited: {}", msg);
    },
    Err(LlmConnectorError::AuthenticationError(msg)) => {
        // 处理认证错误，需要检查 API 密钥
        eprintln!("Auth failed: {}", msg);
    },
    Err(e) => {
        // 处理其他错误
        eprintln!("Request failed: {}", e);
    }
}
```

### 2. 重试逻辑
```rust
use tokio::time::{sleep, Duration};

async fn chat_with_retry(
    client: &Client,
    request: ChatRequest,
    max_retries: u32,
) -> Result<ChatResponse, LlmConnectorError> {
    let mut retries = 0;
    
    loop {
        match client.chat(request.clone()).await {
            Ok(response) => return Ok(response),
            Err(e) if e.is_retryable() && retries < max_retries => {
                retries += 1;
                let delay = Duration::from_millis(1000 * 2_u64.pow(retries));
                sleep(delay).await;
            },
            Err(e) => return Err(e),
        }
    }
}
```
