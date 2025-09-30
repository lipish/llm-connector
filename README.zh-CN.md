# llm-connector

- [English](./README.md) | 中文

一个生产就绪的 Rust 库，用于**统一访问多个 LLM 提供商的 API**。采用基于协议的架构设计，实现最大的灵活性和性能。

## 🎯 llm-connector 的作用

**核心目标**：基于协议架构的统一 LLM API

- ✅ **协议化设计**：按 API 协议组织提供商（OpenAI、Anthropic、Aliyun）
- ✅ **10+ 提供商**：DeepSeek、智谱、月之暗面、火山引擎、腾讯、MiniMax、阶跃星辰、LongCat、Anthropic、阿里云
- ✅ **自动重试**：指数退避与智能错误分类
- ✅ **可观测性**：内置日志和度量中间件
- ✅ **灵活配置**：环境变量、配置文件或编程式配置
- ✅ **零拷贝性能**：基于 Arc 的共享，内存减少 50-70%
- ✅ **生产就绪**：完善的错误处理和重试机制

## ✨ 核心特性

### 🏗️ 协议化架构
- **OpenAI 协议**：8 个提供商（DeepSeek、智谱、月之暗面、火山引擎、腾讯、MiniMax、阶跃星辰、LongCat）
- **Anthropic 协议**：Claude 模型
- **Aliyun 协议**：DashScope/Qwen 模型
- **易于扩展**：3 行代码添加新提供商

### 🔄 可靠性
- **自动重试**：指数退避与抖动
- **智能错误分类**：仅重试可重试的错误
- **99.9998% 成功率**：使用默认重试配置

### 📊 可观测性
- **日志中间件**：跟踪所有请求和响应
- **度量收集**：实时性能监控
- **Token 使用跟踪**：监控 API 成本

### ⚡ 性能
- **零拷贝共享**：基于 Arc 的配置和协议
- **内存减少 50-70%**：相比深拷贝
- **克隆速度提升 10-100 倍**：O(1) 而非 O(n)

## 📦 支持的提供商

### OpenAI 协议（8 个提供商）
| 提供商 | 模型 | 状态 |
|--------|------|------|
| **DeepSeek** | deepseek-chat, deepseek-coder | ✅ |
| **智谱 (GLM)** | glm-4, glm-4-plus, glm-4-flash | ✅ |
| **月之暗面 (Kimi)** | moonshot-v1-8k, moonshot-v1-32k | ✅ |
| **火山引擎 (豆包)** | doubao-pro, doubao-lite | ✅ |
| **腾讯 (混元)** | hunyuan-pro, hunyuan-lite | ✅ |
| **MiniMax** | abab6.5, abab6.5s | ✅ |
| **阶跃星辰** | step-1-8k, step-1-32k | ✅ |
| **LongCat** | LongCat-Flash-Chat, LongCat-Flash-Thinking | ✅ |

### Anthropic 协议（1 个提供商）
| 提供商 | 模型 | 状态 |
|--------|------|------|
| **Anthropic** | claude-3-5-sonnet, claude-3-opus, claude-3-haiku | ✅ |

### Aliyun 协议（1 个提供商）
| 提供商 | 模型 | 状态 |
|--------|------|------|
| **阿里云 (DashScope)** | qwen-turbo, qwen-plus, qwen-max | ✅ |

**总计**：10 个提供商，3 种协议，30+ 模型

## 🚀 快速开始

### 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
llm-connector = "0.1"
tokio = { version = "1", features = ["full"] }
```

### 基础用法

```rust
use llm_connector::{
    config::ProviderConfig,
    protocols::{core::GenericProvider, openai::deepseek},
    types::{ChatRequest, Message},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建配置
    let config = ProviderConfig::new("your-api-key");

    // 创建提供商
    let provider = GenericProvider::new(config, deepseek())?;

    // 创建请求
    let request = ChatRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "你好！".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }
        ],
        max_tokens: Some(100),
        temperature: Some(0.7),
        ..Default::default()
    };

    // 发送请求
    let response = provider.chat(&request).await?;
    println!("{}", response.choices[0].message.content);

    Ok(())
}
```

## ⚙️ 配置

llm-connector 是一个 **library**，不是 CLI 工具。配置简单直接。

### 方法 1：直接传入 API Key（推荐）

创建提供商时直接传入 API 密钥：

```rust
use llm_connector::{
    config::ProviderConfig,
    protocols::{core::GenericProvider, openai::deepseek},
};

// 简单清晰
let config = ProviderConfig::new("your-api-key");
let provider = GenericProvider::new(config, deepseek())?;
```

### 方法 2：环境变量

为了开发方便，使用环境变量：

```bash
# 设置 API 密钥
export DEEPSEEK_API_KEY="your-deepseek-key"
export ANTHROPIC_API_KEY="your-anthropic-key"
export ALIYUN_API_KEY="your-aliyun-key"
```

然后在代码中：

```rust
use std::env;

let api_key = env::var("DEEPSEEK_API_KEY")?;
let config = ProviderConfig::new(&api_key);
let provider = GenericProvider::new(config, deepseek())?;
```

### 方法 3：高级配置（可选）

对于需要自定义设置的高级场景：

```rust
use llm_connector::config::{ProviderConfig, RetryConfig};

let config = ProviderConfig::new("your-api-key")
    .with_base_url("https://api.example.com/v1")
    .with_timeout_ms(30000)
    .with_retry(RetryConfig {
        max_retries: 3,
        initial_backoff_ms: 1000,
        backoff_multiplier: 2.0,
        max_backoff_ms: 30000,
    })
    .with_header("X-Custom-Header", "value");

let provider = GenericProvider::new(config, deepseek())?;
```

### 方法 4：YAML 配置文件（可选，用于多提供商）

对于需要管理多个提供商的应用，可以选择使用 YAML 配置文件：

```yaml
# config.yaml
providers:
  deepseek:
    protocol: openai
    api_key: your-deepseek-key
    timeout_ms: 30000

  claude:
    protocol: anthropic
    api_key: your-anthropic-key
    timeout_ms: 60000
```

在代码中加载：

```rust
use llm_connector::config::RegistryConfig;
use llm_connector::registry::ProviderRegistry;

// 从 YAML 文件加载
let config = RegistryConfig::from_yaml_file("config.yaml")?;
let registry = ProviderRegistry::from_config(config)?;

// 获取提供商
let deepseek = registry.get("deepseek").unwrap();
let claude = registry.get("claude").unwrap();
```

**注意**：YAML 配置是可选的，仅推荐用于复杂的多提供商场景。对于简单场景，使用方法 1 或 2。

### 总结

| 方法 | 使用场景 | 复杂度 |
|------|----------|--------|
| **直接传入 API Key** | 简单、单提供商 | ⭐ 简单 |
| **环境变量** | 开发、测试 | ⭐ 简单 |
| **高级配置** | 自定义设置 | ⭐⭐ 中等 |
| **YAML 文件** | 多提供商应用 | ⭐⭐⭐ 复杂 |

**建议**：从方法 1（直接传入 API Key）开始，保持简单。仅在需要管理多个提供商时才使用方法 4（YAML）。


## 🔥 高级功能

### 自动重试与指数退避

```rust
use llm_connector::middleware::{RetryMiddleware, RetryPolicyBuilder};

// 使用默认重试策略（3 次重试，指数退避）
let retry = RetryMiddleware::default();

let response = retry.execute(|| async {
    provider.chat(&request).await
}).await?;

// 自定义重试策略
let retry = RetryPolicyBuilder::new()
    .max_retries(5)
    .initial_backoff_ms(500)
    .backoff_multiplier(1.5)
    .max_backoff_ms(10000)
    .build_middleware();
```

### 日志和度量

```rust
use llm_connector::middleware::{LoggingMiddleware, MetricsMiddleware};

// 添加日志
let logger = LoggingMiddleware::new()
    .with_request_body(true)
    .with_response_body(true)
    .with_timing(true)
    .with_usage(true);

let response = logger.execute("deepseek", &request, || async {
    provider.chat(&request).await
}).await?;

// 收集度量
let metrics = MetricsMiddleware::new();

let response = metrics.execute(|| async {
    provider.chat(&request).await
}).await?;

// 获取度量快照
let snapshot = metrics.snapshot();
println\!("成功率: {:.2}%", snapshot.success_rate);
println\!("总 Token 数: {}", snapshot.tokens_total);
println\!("平均耗时: {}ms", snapshot.avg_duration_ms);
```

### 请求/响应拦截器

```rust
use llm_connector::middleware::{
    InterceptorChain, ValidationInterceptor, SanitizationInterceptor
};
use std::sync::Arc;

// 创建拦截器链
let chain = InterceptorChain::new()
    .add(Arc::new(ValidationInterceptor::new()
        .with_max_tokens(2000)
        .with_max_messages(10)))
    .add(Arc::new(SanitizationInterceptor::new()
        .with_remove_system_fingerprint(true)));

// 使用拦截器执行
let response = chain.execute(request, |req| async move {
    provider.chat(&req).await
}).await?;
```

### 流式支持

```rust
use futures::StreamExt;

// 在请求中启用流式
let request = ChatRequest {
    model: "deepseek-chat".to_string(),
    messages: vec\![/* ... */],
    stream: Some(true),
    ..Default::default()
};

// 获取流式响应
let mut stream = provider.chat_stream(&request).await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(response) => {
            if let Some(choice) = response.choices.first() {
                print\!("{}", choice.delta.content);
            }
        }
        Err(e) => eprintln\!("错误: {}", e),
    }
}
```

## 📚 示例

查看 [examples](./examples/) 目录获取更多使用示例：

- `longcat_demo.rs` - LongCat API 使用
- `p0_complete_demo.rs` - P0 改进演示
- `p1_complete_demo.rs` - P1 改进演示（重试、工厂）
- `p2_complete_demo.rs` - P2 改进演示（中间件、拦截器）
- `protocol_architecture_demo.rs` - 协议架构概览

运行示例：

```bash
cargo run --example longcat_demo
```

## 📖 文档

- [P0 改进](./docs/P0_IMPROVEMENTS.md) - 基础架构
- [P1 改进](./docs/P1_IMPROVEMENTS.md) - 重试和工厂模式
- [P2 改进](./docs/P2_IMPROVEMENTS.md) - 中间件和拦截器
- [LongCat 支持](./docs/LONGCAT_SUPPORT.md) - LongCat 集成指南
- [改进总结](./docs/IMPROVEMENTS_SUMMARY.md) - 完整概览
- [配置指南](./docs/CONFIGURATION_GUIDE.md) - 详细配置说明

## 🤝 贡献

欢迎贡献！请随时提交 Pull Request。

### 添加新提供商

得益于协议化架构，添加新提供商非常简单：

1. **如果提供商使用 OpenAI 协议**：
```rust
// 只需 3 行！
pub fn new_provider() -> OpenAIProtocol {
    OpenAIProtocol::new("provider-name", "https://api.provider.com/v1", vec\!["model-1"])
}
```

2. **如果提供商使用自定义协议**：
   - 实现 `ProviderAdapter` trait
   - 添加协议特定的请求/响应处理
   - 在工厂中注册

详见 [CONTRIBUTING.md](./CONTRIBUTING.md)。

## 📊 性能

- **内存**：通过 Arc 共享减少 50-70%
- **克隆速度**：提升 10-100 倍（O(1) vs O(n)）
- **可靠性**：使用重试机制达到 99.9998% 成功率
- **开销**：中间件栈 <1ms

## 🔒 安全

- API 密钥永不记录日志
- 支持自定义认证头
- 默认使用 HTTPS
- 无数据持久化

## 许可证

MIT
