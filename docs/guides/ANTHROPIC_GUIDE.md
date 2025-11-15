# Anthropic Claude 使用指南

## 概述

Anthropic 提供 Claude 系列大模型服务。

- **官网**: https://www.anthropic.com
- **控制台**: https://console.anthropic.com
- **API 文档**: https://docs.anthropic.com

## 基础用法

```rust
use llm_connector::providers::anthropic;
use llm_connector::types::{ChatRequest, Message};

let provider = anthropic("your-api-key")?;

let request = ChatRequest {
    model: "claude-3-5-sonnet-20241022".to_string(),
    messages: vec![Message::user("你好")],
    max_tokens: Some(1024),  // Anthropic 要求必须设置 max_tokens
    ..Default::default()
};

let response = provider.chat(&request).await?;
println!("{}", response.content);
```

## 流式响应

Anthropic 使用特殊的流式格式（多事件流），llm-connector 会自动处理：

```rust
use futures_util::StreamExt;

let request = ChatRequest {
    model: "claude-3-5-sonnet-20241022".to_string(),
    messages: vec![Message::user("介绍一下你自己")],
    max_tokens: Some(1024),
    stream: Some(true),
    ..Default::default()
};

let mut stream = provider.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.get_content() {
        print!("{}", content);
    }
}
```

## 多模态支持

```rust
use llm_connector::types::MessageBlock;

let request = ChatRequest {
    model: "claude-3-5-sonnet-20241022".to_string(),
    messages: vec![Message {
        role: Role::User,
        content: vec![
            MessageBlock::text("这张图片里有什么？"),
            MessageBlock::image_url("https://example.com/image.jpg"),
        ],
        ..Default::default()
    }],
    max_tokens: Some(1024),
    ..Default::default()
};
```

## 思考模式 (Extended Thinking)

Claude 支持扩展思考模式，使用 `thinking` 字段：

```rust
let request = ChatRequest {
    model: "claude-3-5-sonnet-20241022".to_string(),
    messages: vec![Message::user("深入思考这个问题")],
    max_tokens: Some(2048),
    ..Default::default()
};

// llm-connector 会自动提取 thinking 内容
```

## 配置选项

```rust
use llm_connector::providers::anthropic_with_config;

let provider = anthropic_with_config(
    "your-api-key",
    None,      // base_url (使用默认)
    Some(60),  // timeout (秒)
    None       // proxy
)?;
```

## 注意事项

### 1. max_tokens 必须设置

Anthropic API 要求必须设置 `max_tokens`，否则会报错。

### 2. API Key 格式

Anthropic 的 API Key 以 `sk-ant-` 开头。

### 3. 流式格式

Anthropic 使用多事件流格式，与 OpenAI 不同：
- `message_start` - 消息开始
- `content_block_delta` - 内容增量
- `message_delta` - 消息增量（包含 usage）
- `message_stop` - 消息结束

llm-connector 会自动处理这些事件，转换为统一的 `StreamingResponse` 格式。

## 支持的功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 非流式响应 | ✅ | 完全支持 |
| 流式响应 | ✅ | 自动处理多事件流 |
| 思考模式 | ✅ | 支持 thinking 字段 |
| 多模态 | ✅ | 支持图片输入 |
| 函数调用 | ✅ | 支持 |

## 参考资源

- [官方文档](https://docs.anthropic.com)
- [流式响应文档](https://docs.anthropic.com/en/api/streaming)
- [推理模型支持](../REASONING_MODELS_SUPPORT.md)

