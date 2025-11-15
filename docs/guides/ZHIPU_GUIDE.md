# 智谱 GLM 使用指南

## 概述

智谱 AI 提供 GLM 系列大模型服务。

- **官网**: https://www.zhipuai.cn
- **控制台**: https://open.bigmodel.cn
- **API 文档**: https://open.bigmodel.cn/dev/api

## 基础用法

```rust
use llm_connector::providers::zhipu;
use llm_connector::types::{ChatRequest, Message};

let provider = zhipu("your-api-key", None)?;

let request = ChatRequest {
    model: "glm-4-plus".to_string(),
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
    model: "glm-4-plus".to_string(),
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

## 推理模式

智谱的推理模型会在 `reasoning` 字段返回推理内容：

```rust
let request = ChatRequest {
    model: "glm-4-plus".to_string(),  // 或其他推理模型
    messages: vec![Message::user("解决这个问题")],
    ..Default::default()
};

// llm-connector 会自动提取 reasoning 内容
```

## 多模态支持

```rust
use llm_connector::types::MessageBlock;

let request = ChatRequest {
    model: "glm-4v-plus".to_string(),
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

## 工具调用 (Function Calling)

```rust
use serde_json::json;

let request = ChatRequest {
    model: "glm-4-plus".to_string(),
    messages: vec![Message::user("今天天气怎么样？")],
    tools: Some(vec![json!({
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "获取天气信息",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "城市名称"
                    }
                },
                "required": ["location"]
            }
        }
    })]),
    ..Default::default()
};

let response = provider.chat(&request).await?;

// 检查是否有工具调用
if let Some(choice) = response.choices.first() {
    if let Some(tool_calls) = &choice.message.tool_calls {
        for tool_call in tool_calls {
            println!("调用工具: {}", tool_call.function.name);
            println!("参数: {}", tool_call.function.arguments);
        }
    }
}
```

## 配置选项

```rust
use llm_connector::providers::zhipu_with_config;

let provider = zhipu_with_config(
    "your-api-key",
    None,      // base_url (使用默认)
    Some(60),  // timeout (秒)
    None       // proxy
)?;
```

## 注意事项

### 1. API Key 格式

智谱的 API Key 格式较长，通常包含多个部分。

### 2. 模型名称

常用模型：
- `glm-4-plus` - 最新旗舰模型
- `glm-4-flash` - 快速模型
- `glm-4v-plus` - 多模态模型

### 3. 流式响应中的工具调用

智谱支持在流式响应中返回工具调用，llm-connector 会自动处理。

## 支持的功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 非流式响应 | ✅ | 完全支持 |
| 流式响应 | ✅ | 完全支持 |
| 推理模式 | ✅ | 支持 reasoning 字段 |
| 多模态 | ✅ | 支持图片输入 |
| 函数调用 | ✅ | 完全支持，包括流式 |

## 参考资源

- [官方文档](https://open.bigmodel.cn/dev/api)
- [工具调用文档](https://open.bigmodel.cn/dev/api#glm-4)
- [推理模型支持](../REASONING_MODELS_SUPPORT.md)

