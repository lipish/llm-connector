# llm-connector（中文文档）

最小化的 Rust 库，用于抽象 LLM 协议。

支持 6 种协议：OpenAI、Anthropic、Aliyun、Zhipu、Ollama、Hunyuan。
无需复杂配置——选择协议即可开始聊天。

## 🚨 身份验证问题？

立即测试你的 API Key：
```bash
cargo run --example test_keys_yaml
```

这将告诉你 API Key 的具体问题！详见下文「调试与排障」。

## ✨ 关键特性

- 6 协议支持：OpenAI、Anthropic、Aliyun、Zhipu、Ollama、Hunyuan
- 无硬编码模型限制：可使用任意模型名称
- 在线模型发现：从 API 动态获取模型列表
- 增强流式支持：实时流式响应，并正确处理 Anthropic 事件
- Ollama 模型管理：本地模型完整增删改查
- 统一接口：跨协议一致的调用方式
- 类型安全：Rust 异步/等待全类型保障

## 快速开始

### 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
llm-connector = "0.3.6"
tokio = { version = "1", features = ["full"] }
```

可选功能：
```toml
# 流式响应支持
llm-connector = { version = "0.3.6", features = ["streaming"] }

# 腾讯云原生 API 支持
llm-connector = { version = "0.3.6", features = ["tencent-native"] }

# 同时启用流式响应和腾讯云原生 API
llm-connector = { version = "0.3.6", features = ["streaming", "tencent-native"] }
```

### 基本用法

```rust
use llm_connector::{LlmClient, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OpenAI（默认 base URL）
    let client = LlmClient::openai("sk-...", None);

    // Anthropic
    let client = LlmClient::anthropic("sk-ant-...");

    // Aliyun（DashScope）
    let client = LlmClient::aliyun("sk-...");

    // 腾讯混元
    let client = LlmClient::hunyuan("sk-...");

    // Ollama（本地，无需 API Key）
    let client = LlmClient::ollama(None);

    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message::user("Hello!")],
        ..Default::default()
    };

    let response = client.chat(&request).await?;
    println!("Response: {}", response.choices[0].message.content);
    Ok(())
}
```

## 协议支持

### 1. OpenAI 协议
标准 OpenAI API 格式。

```rust
// OpenAI（默认 base URL）
let client = LlmClient::openai("sk-...", None);

// OpenAI 兼容端点（自定义 base URL）
let client = LlmClient::openai("sk-...", Some("https://api.example.com/v1"));
```

特性：
- 无硬编码模型限制——可使用任意模型名称
- 通过 `fetch_models()` 在线发现模型
- 支持 OpenAI 兼容提供商（DeepSeek、Zhipu、Moonshot 等）

示例模型：gpt-4、gpt-4-turbo、gpt-3.5-turbo、o1-preview、o1-mini

### 2. Anthropic 协议
Claude Messages API，系统消息独立。

```rust
let client = LlmClient::anthropic("sk-ant-...");
```

模型：claude-3-5-sonnet-20241022、claude-3-opus、claude-3-haiku

### 3. Aliyun 协议（DashScope）
Qwen 模型专用协议。

```rust
let client = LlmClient::aliyun("sk-...");
```

模型：qwen-turbo、qwen-plus、qwen-max

### 4. 腾讯混元协议
腾讯混元模型提供两种实现方式：

#### 4.1 OpenAI 兼容接口
```rust
let client = LlmClient::hunyuan("sk-...");
```

特性：
- OpenAI 兼容 API 格式
- 支持流式响应
- 通过 `fetch_models()` 在线发现模型

#### 4.2 腾讯云原生 API（推荐）
```rust
// 需要启用 "tencent-native" 功能
let client = LlmClient::hunyuan_native("secret-id", "secret-key", Some("ap-beijing"));
```

特性：
- 腾讯云原生 API，使用 TC3-HMAC-SHA256 签名
- 完整访问腾讯云功能
- 更好的错误处理和调试
- 支持流式响应
- 支持地域指定

模型：hunyuan-lite、hunyuan-standard、hunyuan-pro

### 5. Ollama 协议（本地）
本地 LLM 服务，无需 API Key。

```rust
// 默认：localhost:11434
let client = LlmClient::ollama(None);

// 自定义 URL
let client = LlmClient::ollama(Some("http://192.168.1.100:11434"));
```

模型：llama3.2、llama3.1、mistral、mixtral、qwen2.5 等。

特性：
- 通过 `/api/tags` 列出模型
- 模型管理（拉取、推送、删除、详情）
- 支持本地服务与自定义 URL
- 针对 Ollama 操作增强错误处理

## Ollama 模型管理

提供完整的 Ollama 模型管理能力：

```rust
use llm_connector::ollama::OllamaModelOps;
let client = LlmClient::ollama();

// 列出本地安装的所有模型
let models = client.list_models().await?;
for model in models {
    println!("Available model: {}", model);
}

// 拉取模型
client.pull_model("llama3.2").await?;

// 获取模型详情
let details = client.show_model("llama3.2").await?;
println!("Model size: {} bytes", details.size.unwrap_or(0));

// 删除模型
client.delete_model("llama3.2").await?;
```

支持的 Ollama 操作：
- 列出模型：`list_models()`
- 拉取模型：`pull_model(name)`
- 推送模型：`push_model(name)`
- 删除模型：`delete_model(name)`
- 模型详情：`show_model(name)`

## 增强的流式支持

对 Anthropic 的流式支持进行了改进，包含完整事件状态管理：

```rust
use futures_util::StreamExt;

let client = LlmClient::anthropic("sk-ant-...");
let request = ChatRequest {
    model: "claude-3-5-sonnet-20241022".to_string(),
    messages: vec![Message::user("Hello!")],
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

增强点：
- 状态管理：正确处理 `message_start`、`content_block_delta`、`message_delta`、`message_stop`
- 事件处理：解析复杂的 Anthropic 流式响应
- 用量跟踪：流式过程中实时统计 token 用量
- 错误韧性：对流式中断的鲁棒处理

## 模型发现

从 API 获取最新可用模型：

```rust
let client = LlmClient::openai("sk-...");

// 从 API 在线获取模型
let models = client.fetch_models().await?;
println!("Available models: {:?}", models);
```

支持情况：
- OpenAI 协议（包含 OpenAI 兼容服务，如 DeepSeek、Zhipu、Moonshot 等）
- Anthropic 协议（有限支持——返回回退端点）
- Ollama 协议（通过 `/api/tags` 完整支持）
- Aliyun 协议（不支持）

示例结果：
- DeepSeek：`["deepseek-chat", "deepseek-reasoner"]`
- Zhipu：`["glm-4.5", "glm-4.5-air", "glm-4.6"]`
- Moonshot：`["moonshot-v1-32k", "kimi-latest", ...]`

推荐：
- 缓存 `fetch_models()` 结果，避免重复请求
- 对不支持模型列表的协议，可直接使用任意模型名

## 请求示例

### OpenAI / OpenAI 兼容

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

### Anthropic（需要 max_tokens）

```rust
let request = ChatRequest {
    model: "claude-3-5-sonnet-20241022".to_string(),
    messages: vec![Message::user("Hello!")],
    max_tokens: Some(200), // Anthropic 必填
    ..Default::default()
};
```

### Aliyun（DashScope）

```rust
let request = ChatRequest {
    model: "qwen-max".to_string(),
    messages: vec![Message::user("你好！")],
    ..Default::default()
};
```

### Ollama（本地）

```rust
let request = ChatRequest {
    model: "llama3.2".to_string(),
    messages: vec![Message::user("Hello!")],
    ..Default::default()
};
```

#### Ollama 流式（通过远程网关调用 GLM-4.6）

如果你对外提供 Ollama 兼容 API，但后端调用 Zhipu 的 `glm-4.6`（远程网关），无需本地安装模型。将客户端指向你的网关地址，并使用服务端定义的模型标识：

```rust
use futures_util::StreamExt;
use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 指向你的远程 Ollama 兼容网关（替换为你的实际地址）
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

运行示例（需启用 `streaming` 功能）：

```bash
cargo run --example ollama_streaming --features streaming
```

说明：该方案面向远程 Ollama 兼容网关。模型标识由后端定义（如 `glm-4.6`），无需本地安装。如果你的网关使用不同标识，请替换为实际值。

## 流式（可选功能）

在 `Cargo.toml` 启用流式：
```toml
llm-connector = { version = "0.3.6", features = ["streaming"] }
```

```rust
use futures_util::StreamExt;

let mut stream = client.chat_stream(&request).await?;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if let Some(content) = chunk.get_content() {
        print!("{}", content);
    }
}
```

## 错误处理

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

## 配置

### 简单 API Key（推荐）

```rust
let client = LlmClient::openai("your-api-key");
```

### 环境变量

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

## 协议信息

```rust
let client = LlmClient::openai("sk-...");

// 获取协议名称
println!("Protocol: {}", client.protocol_name());

// 在线获取模型（需要 API 请求）
let models = client.fetch_models().await?;
println!("Available models: {:?}", models);
```

## 推理内容别名（Reasoning Synonyms）

许多提供商会返回隐藏或私有的推理内容键。为简化跨协议使用，我们统一归一化四个常见键：

- `reasoning_content`、`reasoning`、`thought`、`thinking`

后处理会自动扫描原始 JSON，并填充消息（`Message`）与流式增量（`Delta`）上的这些可选字段。可通过便捷方法读取首个可用值：

```rust
// 非流式
let msg = &response.choices[0].message;
if let Some(reason) = msg.reasoning_any() {
    println!("Reasoning: {}", reason);
}

// 流式
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if let Some(reason) = chunk.choices[0].delta.reasoning_any() {
        println!("Reasoning (stream): {}", reason);
    }
}
```

说明：
- 如果提供商不返回任何推理键，这些字段保持 `None`
- 归一化与提供商无关，统一适用于 OpenAI、Anthropic、Aliyun（Qwen）、Zhipu（GLM）、DeepSeek 等（含流式）
- `StreamingResponse` 也会从首个包含推理的增量回填其顶层 `reasoning_content`

## 调试与排障

### 测试你的 API Key

快速验证 API Key 是否有效：

```bash
# 从 keys.yaml 测试所有密钥
cargo run --example test_keys_yaml

# 专门调试 DeepSeek
cargo run --example debug_deepseek -- sk-your-key
```

测试工具将：
- 验证 API Key 格式
- 测试与提供商的身份验证
- 精确显示问题原因
- 提供具体修复建议

### 排障文档

- `TROUBLESHOOTING.md` — 综合排障指南
- `HOW_TO_TEST_YOUR_KEYS.md` — 如何测试你的 API Key
- `TEST_YOUR_DEEPSEEK_KEY.md` — DeepSeek 快速开始

### 常见问题

**身份验证错误：**
```
❌ Authentication failed: Incorrect API key provided
```

**解决方案：**
1. 检查 API Key 是否正确（无多余空格）
2. 确认账户是否有额度/权限
3. 在提供商控制台生成新的 API Key
4. 运行 `cargo run --example test_keys_yaml` 进行诊断

## 最近变更

### v0.3.1（最新）

重大新特性：
- 完整的 Ollama 模型管理：本地模型增删改查
  - `list_models()`、`pull_model()`、`push_model()`、`delete_model()`、`show_model()`
- 增强的 Anthropic 流式：正确的事件状态管理
  - 处理 `message_start`、`content_block_delta`、`message_delta`、`message_stop`
  - 流式过程实时 token 用量跟踪
  - 改进的错误韧性与状态管理

改进：
- 扩展模型发现支持：
  - 增加 Ollama 通过 `/api/tags` 列出模型
  - Anthropic 模型发现有限支持
- 增强客户端接口：提供 Ollama 模型管理方法
- 更新示例：增加模型管理与流式示例

文档：
- 重写 Ollama 章节并加入模型管理示例
- 增强流式文档与代码示例
- 更新特性描述与支持操作

### v0.2.3

不兼容变更：
- 移除 `supported_models()`（使用 `fetch_models()`）
- 移除 `supports_model()`

新特性：
- 改进错误信息（去除对其他提供商混淆的 OpenAI 链接）
- 新调试工具：
  - `examples/test_keys_yaml.rs` — 测试所有 API Key
  - `examples/debug_deepseek.rs` — 调试 DeepSeek 身份验证
- 综合文档：
  - `TROUBLESHOOTING.md`、`HOW_TO_TEST_YOUR_KEYS.md`、`TEST_YOUR_DEEPSEEK_KEY.md`

从 v0.2.2 迁移：
```rust
// 旧（已不支持）
let models = client.supported_models();

// 新
let models = client.fetch_models().await?;
```

### v0.2.2

新特性：
- 增加 `fetch_models()` 在线模型发现
- OpenAI 协议支持从 `/v1/models` 动态拉取
- 适配 OpenAI 兼容提供商（DeepSeek、Zhipu、Moonshot 等）

## 设计哲学

最小化设计：
- 仅 4 种协议覆盖多数主流提供商
- 无硬编码模型限制——任意模型名可用
- 无复杂配置文件或注册中心
- 直接 API 调用，抽象清晰

协议优先：
- 按协议而非公司归类提供商
- OpenAI 兼容提供商共享一套实现
- 通过协议适配器可扩展

## 示例

查看 `examples/` 目录：

```bash
# 从 keys.yaml 测试你的 API Key
cargo run --example test_keys_yaml

# 调试 DeepSeek 身份验证
cargo run --example debug_deepseek -- sk-your-key

# 简单 fetch_models() 演示
cargo run --example fetch_models_simple

# Ollama 模型管理（NEW!）
cargo run --example ollama_model_management

# Anthropic 流式（NEW!，需启用 streaming）
cargo run --example anthropic_streaming --features streaming

# Ollama 流式（NEW!，需启用 streaming）
cargo run --example ollama_streaming --features streaming

# LongCat 演示（兼容 OpenAI/Anthropic）
cargo run --example longcat_dual
```

### 示例说明

`test_keys_yaml.rs`（新）：
- 测试 `keys.yaml` 中的所有 API Key
- 验证密钥格式与身份认证
- 为每种错误提供具体排障建议
- 如果你遇到认证问题，请先运行它！

`debug_deepseek.rs`（新）：
- DeepSeek API 交互式调试工具
- 验证 API Key 格式
- 测试模型拉取与聊天请求
- 提供详细排障指导

`fetch_models_simple.rs`：
- 简单展示 `fetch_models()`
- 展示如何从 OpenAI 兼容提供商获取模型
- 附带使用建议

`ollama_model_management.rs`（新）：
- 展示完整的 Ollama 模型管理功能
- 列出、拉取、删除、获取模型详情
- 包含错误处理与实用示例

`anthropic_streaming.rs`（新）：
- 展示增强的 Anthropic 流式事件处理
- 实时响应流与用量统计
- 同时包含常规与流式聊天示例

已移除冗余示例：
- `test_fetch_models.rs`、`test_with_keys.rs` 与其他示例重叠，已移除

## 贡献

欢迎贡献！欢迎提交 Pull Request。

## 许可协议

MIT