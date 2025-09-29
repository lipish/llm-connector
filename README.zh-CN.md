# llm-connector

- [English](./README.md) | 中文

一个专注于**协议适配**的轻量级 Rust 库，用于在多个 LLM 提供商之间进行协议转换，提供统一的 OpenAI 兼容接口。

## 🎯 llm-connector 的作用

**核心目标**：协议适配和 API 标准化

- ✅ **协议转换**：将不同 LLM 提供商的 API 转换为 OpenAI 兼容格式
- ✅ **请求/响应标准化**：跨提供商的统一数据结构
- ✅ **流式适配**：标准化不同提供商的 SSE 流
- ✅ **提供商抽象**：可扩展的 trait 系统，便于添加新提供商
- ✅ **简单配置**：基础的 API 密钥和端点管理

## 🚫 llm-connector 不做什么

本库有意**不包含**以下功能：
- ❌ 负载均衡（请使用反向代理或服务网格）
- ❌ 健康检查（请使用外部监控）
- ❌ 熔断器（请使用基础设施级解决方案）
- ❌ 复杂路由策略（保持简单）
- ❌ 内置指标收集（请使用 tracing/metrics crate）
- ❌ 请求队列或限流

## 特性

- **提供商无关类型**：统一的请求/响应结构
- **简单的提供商 trait**：易于实现新提供商
- **流式支持**：跨提供商的统一 SSE 处理
- **最小依赖**：专注于核心功能
- **OpenAI 兼容**：可直接替换 OpenAI 客户端

## 支持的提供商

- **DeepSeek** ✅ - DeepSeek 模型（deepseek-chat, deepseek-reasoner）
- **OpenAI** 🚧 - GPT 模型（gpt-4, gpt-3.5-turbo 等）- *即将推出*
- **Anthropic** 🚧 - Claude 模型（claude-3-5-sonnet, claude-3-haiku 等）- *即将推出*
- **智谱 GLM** 🚧 - GLM 模型（glm-4, glm-3-turbo 等）- *即将推出*
- **阿里 Qwen** 🚧 - Qwen 模型（qwen-turbo, qwen-plus 等）- *即将推出*
- **月之暗面 Kimi** 🚧 - Kimi 模型（moonshot-v1-8k, moonshot-v1-32k 等）- *即将推出*

## 快速开始

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
llm-connector = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### 基础用法

```rust
use llm_connector::{Client, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用环境变量初始化客户端
    let client = Client::from_env();

    // 创建聊天请求
    let request = ChatRequest {
        model: "deepseek/deepseek-chat".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "你好，你怎么样？".to_string(),
                ..Default::default()
            }
        ],
        ..Default::default()
    };

    // 发送请求
    let response = client.chat(request).await?;
    println!("回复: {}", response.choices[0].message.content);

    Ok(())
}
```

### 配置

#### 环境变量（推荐）

为要使用的提供商设置环境变量：

```bash
# OpenAI
export OPENAI_API_KEY="your-openai-key"
export OPENAI_BASE_URL="https://api.openai.com/v1"  # 可选

# DeepSeek
export DEEPSEEK_API_KEY="your-deepseek-key"
export DEEPSEEK_BASE_URL="https://api.deepseek.com/v1"  # 可选

# Anthropic
export ANTHROPIC_API_KEY="your-anthropic-key"
export ANTHROPIC_BASE_URL="https://api.anthropic.com"  # 可选

# 根据需要添加其他提供商...
```

#### 显式配置

```rust
use llm_connector::{Client, Config, ProviderConfig};

let config = Config {
    openai: Some(ProviderConfig {
        api_key: "your-openai-key".to_string(),
        base_url: Some("https://api.openai.com/v1".to_string()),
    }),
    deepseek: Some(ProviderConfig {
        api_key: "your-deepseek-key".to_string(),
        base_url: Some("https://api.deepseek.com/v1".to_string()),
    }),
    // ... 其他提供商
    ..Default::default()
};

let client = Client::with_config(config);
```

## 端到端流程（从最小示例到可用）

本节在“快速上手”基础上，给出从配置到调用的完整流程，并涵盖“在线发现模型 + 缓存（TTL）”机制。

1) 准备配置文件（两种方式，二选一）
- 推荐放置路径（按顺序优先使用第一个存在的文件）：
  - 项目根目录：./llm-gateway.config.yaml
  - 用户目录：~/.config/llm-gateway/config.yaml
- 方式 A：在线发现模型 + 结果缓存（TTL）
  ```yaml
  # Library 模式下，是否发现与缓存 TTL 仅通过配置文件控制
  discover_models: true
  discover_models_ttl_secs: 600  # 10 分钟；当为 0 时使用内置默认 300 秒
  deepseek:
    base_url: https://api.deepseek.com/v1
  qwen:
    base_url: https://dashscope.aliyun.com/compatible-mode/v1
  glm:
    base_url: https://open.bigmodel.cn/api/paas/v4
  kimi:
    base_url: https://api.moonshot.cn/v1
  ```
- 方式 B：通过 YAML 导入模型列表（不在线发现、不缓存）
  ```yaml
  discover_models: false
  deepseek:
    base_url: https://api.deepseek.com/v1
    models:
      - deepseek-chat
      - deepseek-reasoner
  ```
提示：避免将 API Key 提交到仓库。仅在本机配置文件中保存，或在运行时通过环境变量注入。

2) 在代码中加载配置并创建客户端
```rust
use llm_gateway::Client;

let client = if let Some(c) = Client::from_yaml_file_auto() {
    c
} else {
    // 回退到环境变量驱动的默认（仅覆盖 provider 的 api_key/base_url）；
    // 是否发现与 TTL 在 Library 模式仅由配置文件控制
    Client::from_env()
};
```
3) 列出可用模型
- 仅本地/默认（不触发在线发现）：
```rust
let local = client.list_models();
println!("local: {:?}", local);
```
- 根据配置自动聚合（discover_models=true 时触发在线发现与缓存）：
```rust
let auto = client.list_models_auto().await?;
println!("auto: {:?}", auto);
```
缓存说明：
- discover_models=true 时，在线发现结果会按 discover_models_ttl_secs 进行缓存；
- 为 0 则使用内置默认 300s；
- 缓存命中直接返回，无需再次请求远端；TTL 过期会自动刷新；
- 发现失败会回退到本地 models 或内置默认列表。

4) 发起对话
```rust
use llm_gateway::types::ChatRequest;

let resp = client.chat(ChatRequest {
    model: "glm/glm-4".into(), // 建议使用 provider/model 形式
    messages: vec![("user".into(), "来一首关于 Rust 的两行短诗".into())],
}).await?;
println!("{}", resp.text);
```

## 环境变量

由 `from_env()` 识别的环境变量：
- DEEPSEEK_API_KEY, DEEPSEEK_BASE_URL (默认 https://api.deepseek.com/v1)\n- GLM_API_KEY or ZHIPU_API_KEY, GLM_BASE_URL or ZHIPU_BASE_URL (默认 https://open.bigmodel.cn/api/paas/v4)\n- QWEN_API_KEY or ALIBABA_QWEN_API_KEY, QWEN_BASE_URL or ALIBABA_QWEN_BASE_URL (默认 https://dashscope.aliyun.com/compatible-mode/v1)\n- KIMI_API_KEY or MOONSHOT_API_KEY, KIMI_BASE_URL or MOONSHOT_BASE_URL (默认 https://api.moonshot.cn/v1)

## 流式输出

```rust
use futures_util::StreamExt;
use llm_gateway::{Client};
use llm_gateway::types::ChatRequest;

# async fn run() -> anyhow::Result<()> {
let client = Client::from_env();
let mut stream = client.chat_stream(ChatRequest {\n    model: \"qwen/qwen2-7b-instruct\".into(),\n    messages: vec![(\"user\".into(), \"流式测试\".into())],\n}).await?;\n\nwhile let Some(chunk) = stream.next().await {\n    let text = chunk?; // 每个项是一个文本增量\n    print!(\"{}\", text);\n}\n# Ok(())\n# }
```

注意: 实现使用 OpenAI 兼容的 SSE 解析（以 `data: ` 开头的行）。如果提供商不同，我们可以添加特定于提供商的解析器。

5) 验证缓存行为（可选）
- 首次 list_models_auto() 触发远程发现并写入缓存；\n- TTL 内后续调用应为快速缓存命中；\n- TTL 过期后，再次调用会刷新远程列表；\n- 网络不稳定时，库会回退到缓存或本地列表以确保鲁棒性。

故障排除
- 配置未找到：确保它位于上述两个路径之一，或实现自己的加载器；\n- 模型名称解析：优先使用 provider/model 形式（例如，glm/glm-4, qwen/qwen2-7b-instruct）；\n- 来自提供商的 401/403：验证 API 密钥注入（通过配置或运行时）；\n- 基础 URL 不匹配：检查提供商的 OpenAI 兼容端点。

另见: ../../docs/config.sample.yaml

## 许可证

MIT