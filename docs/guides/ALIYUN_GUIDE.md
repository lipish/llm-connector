# 阿里云 DashScope 使用指南

## 概述

阿里云 DashScope 提供通义千问系列大模型服务。

- **官网**: https://dashscope.aliyun.com
- **控制台**: https://dashscope.console.aliyun.com
- **API 文档**: https://help.aliyun.com/zh/dashscope

## 基础用法

```rust
use llm_connector::providers::aliyun;
use llm_connector::types::{ChatRequest, Message};

let provider = aliyun("your-api-key")?;

let request = ChatRequest {
    model: "qwen-plus".to_string(),
    messages: vec![Message::user("你好")],
    ..Default::default()
};

let response = provider.chat(&request).await?;
println!("{}", response.content);
```

## 流式响应

```rust
use futures_util::StreamExt;

let request = ChatRequest {
    model: "qwen-plus".to_string(),
    messages: vec![Message::user("介绍一下你自己")],
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

## 推理模式 (Thinking)

阿里云支持两种推理模式：

### 1. 混合推理模式

需要设置 `enable_thinking: true` 参数：

```rust
let request = ChatRequest {
    model: "qwen-plus".to_string(),
    messages: vec![Message::user("解决这个问题")],
    enable_thinking: Some(true),  // 启用推理模式
    ..Default::default()
};
```

**支持的模型**:
- qwen-plus, qwen-plus-latest
- qwen-flash
- qwen-turbo, qwen-turbo-latest
- qwen3 系列
- deepseek-v3.2-exp, deepseek-v3.1

### 2. 纯推理模式

以下模型默认启用推理，无需额外参数：

- qwen3-next-80b-a3b-thinking
- qwen3-235b-a22b-thinking-2507
- qwq-plus, qwq-plus-latest
- deepseek-r1, deepseek-r1-0528

## 多模态支持

```rust
use llm_connector::types::MessageBlock;

let request = ChatRequest {
    model: "qwen-vl-plus".to_string(),
    messages: vec![Message {
        role: Role::User,
        content: vec![
            MessageBlock::text("这张图片里有什么？"),
            MessageBlock::image_url("https://example.com/image.jpg"),
        ],
        ..Default::default()
    }],
    ..Default::default()
};
```

## 配置选项

```rust
use llm_connector::providers::aliyun_with_config;

let provider = aliyun_with_config(
    "your-api-key",
    None,      // base_url (使用默认)
    Some(60),  // timeout (秒)
    None       // proxy
)?;
```

## 常见问题

### 1. 推理内容为空

**问题**: 使用 qwen-plus 等模型时，没有返回推理内容

**解决**: 设置 `enable_thinking: Some(true)`

### 2. 流式响应中断

**问题**: 流式响应提前结束

**解决**: 检查 `incremental_output` 参数是否正确设置（库会自动处理）

### 3. API Key 错误

**问题**: `InvalidApiKey` 错误

**解决**: 
1. 检查 API Key 格式（以 `sk-` 开头）
2. 在控制台确认 API Key 状态
3. 检查 API Key 权限

## 支持的功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 非流式响应 | ✅ | 完全支持 |
| 流式响应 | ✅ | 完全支持 |
| 推理模式 | ✅ | 支持 enable_thinking |
| 多模态 | ✅ | 支持图片输入 |
| 函数调用 | ✅ | 支持 |

## 参考资源

- [官方文档](https://help.aliyun.com/zh/dashscope)
- [推理模式文档](https://www.alibabacloud.com/help/en/model-studio/deep-thinking)
- [推理模型支持](../REASONING_MODELS_SUPPORT.md)

