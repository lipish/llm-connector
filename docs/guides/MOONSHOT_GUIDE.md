# Moonshot（月之暗面）使用指南

## 📋 概述

Moonshot（月之暗面）是一家专注于大语言模型的 AI 公司，提供 OpenAI 兼容的 API 接口。

**官网**: https://www.moonshot.cn/  
**API 文档**: https://platform.moonshot.cn/docs

---

## 🚀 快速开始

### 安装

```toml
[dependencies]
llm-connector = "0.4.20"
tokio = { version = "1", features = ["full"] }

# 流式支持（可选）
llm-connector = { version = "0.4.20", features = ["streaming"] }
```

### 获取 API Key

1. 访问 https://platform.moonshot.cn/
2. 注册/登录账号
3. 在控制台创建 API Key
4. API Key 格式: `sk-...`

---

## 💡 基础用法

### 非流式响应

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端
    let client = LlmClient::moonshot("sk-...")?;
    
    // 创建请求
    let request = ChatRequest {
        model: "moonshot-v1-8k".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "你好，请介绍一下你自己".to_string(),
            ..Default::default()
        }],
        max_tokens: Some(100),
        ..Default::default()
    };
    
    // 发送请求
    let response = client.chat(&request).await?;
    println!("Response: {}", response.content);
    
    Ok(())
}
```

### 流式响应

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::moonshot("sk-...")?;
    
    let request = ChatRequest {
        model: "moonshot-v1-8k".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "介绍一下北京".to_string(),
            ..Default::default()
        }],
        stream: Some(true),
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
    
    Ok(())
}
```

---

## 🎯 支持的模型

| 模型 | 上下文长度 | 说明 |
|------|-----------|------|
| **moonshot-v1-8k** | 8,192 tokens | 标准模型 |
| **moonshot-v1-32k** | 32,768 tokens | 长上下文模型 |
| **moonshot-v1-128k** | 131,072 tokens | 超长上下文模型 |

### 选择模型

```rust
// 8k 上下文（标准）
let request = ChatRequest {
    model: "moonshot-v1-8k".to_string(),
    // ...
};

// 32k 上下文（长文本）
let request = ChatRequest {
    model: "moonshot-v1-32k".to_string(),
    // ...
};

// 128k 上下文（超长文本）
let request = ChatRequest {
    model: "moonshot-v1-128k".to_string(),
    // ...
};
```

---

## ⚙️ 高级配置

### 自定义配置

```rust
use llm_connector::LlmClient;

let client = LlmClient::moonshot_with_config(
    "sk-...",           // API key
    None,               // base_url (使用默认)
    Some(60),           // timeout (60秒)
    None                // proxy
)?;
```

### 使用代理

```rust
let client = LlmClient::moonshot_with_config(
    "sk-...",
    None,
    Some(60),
    Some("http://proxy.example.com:8080")  // 代理地址
)?;
```

### 自定义端点

```rust
let client = LlmClient::moonshot_with_config(
    "sk-...",
    Some("https://custom.api.moonshot.cn"),  // 自定义端点
    Some(60),
    None
)?;
```

---

## 📊 请求参数

### 常用参数

```rust
let request = ChatRequest {
    model: "moonshot-v1-8k".to_string(),
    messages: vec![/* ... */],
    
    // 可选参数
    temperature: Some(0.7),        // 温度 (0.0-1.0)
    top_p: Some(0.9),              // 核采样
    max_tokens: Some(1000),        // 最大生成 tokens
    stream: Some(true),            // 流式响应
    
    ..Default::default()
};
```

### 参数说明

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `model` | String | 必需 | 模型名称 |
| `messages` | Vec<Message> | 必需 | 对话消息列表 |
| `temperature` | f32 | 0.3 | 控制随机性 (0.0-1.0) |
| `top_p` | f32 | 1.0 | 核采样参数 |
| `max_tokens` | u32 | - | 最大生成 tokens |
| `stream` | bool | false | 是否流式响应 |

---

## 🎨 使用场景

### 1. 长文本处理

利用 Moonshot 的长上下文能力处理长文本：

```rust
let request = ChatRequest {
    model: "moonshot-v1-128k".to_string(),  // 使用 128k 模型
    messages: vec![
        Message {
            role: Role::User,
            content: format!("请总结以下文章：\n\n{}", long_article),
            ..Default::default()
        }
    ],
    max_tokens: Some(500),
    ..Default::default()
};

let response = client.chat(&request).await?;
println!("摘要: {}", response.content);
```

### 2. 多轮对话

```rust
let mut messages = vec![
    Message {
        role: Role::System,
        content: "你是一个有帮助的助手".to_string(),
        ..Default::default()
    }
];

// 第一轮
messages.push(Message {
    role: Role::User,
    content: "什么是 Rust?".to_string(),
    ..Default::default()
});

let response = client.chat(&ChatRequest {
    model: "moonshot-v1-8k".to_string(),
    messages: messages.clone(),
    ..Default::default()
}).await?;

messages.push(Message {
    role: Role::Assistant,
    content: response.content.clone(),
    ..Default::default()
});

// 第二轮
messages.push(Message {
    role: Role::User,
    content: "它有什么优势?".to_string(),
    ..Default::default()
});

let response = client.chat(&ChatRequest {
    model: "moonshot-v1-8k".to_string(),
    messages: messages.clone(),
    ..Default::default()
}).await?;
```

### 3. 实时流式输出

```rust
use futures_util::StreamExt;

let mut stream = client.chat_stream(&request).await?;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    
    // 实时打印
    if let Some(content) = chunk.get_content() {
        print!("{}", content);
        use std::io::{self, Write};
        io::stdout().flush()?;
    }
    
    // 检查结束
    if let Some(reason) = chunk.get_finish_reason() {
        println!("\n完成原因: {}", reason);
    }
}
```

---

## 🔍 错误处理

```rust
use llm_connector::error::LlmConnectorError;

match client.chat(&request).await {
    Ok(response) => {
        println!("成功: {}", response.content);
    }
    Err(LlmConnectorError::ApiError { status, message }) => {
        eprintln!("API 错误 {}: {}", status, message);
    }
    Err(LlmConnectorError::NetworkError(e)) => {
        eprintln!("网络错误: {}", e);
    }
    Err(e) => {
        eprintln!("其他错误: {}", e);
    }
}
```

---

## 📈 性能优化

### 1. 复用客户端

```rust
// ✅ 推荐：复用客户端
let client = LlmClient::moonshot("sk-...")?;

for _ in 0..10 {
    let response = client.chat(&request).await?;
    // 处理响应
}
```

### 2. 合理设置超时

```rust
// 长文本处理，增加超时时间
let client = LlmClient::moonshot_with_config(
    "sk-...",
    None,
    Some(120),  // 120秒超时
    None
)?;
```

### 3. 使用流式响应

```rust
// 对于长响应，使用流式可以更快看到结果
let request = ChatRequest {
    stream: Some(true),  // 启用流式
    // ...
};
```

---

## 🎉 总结

Moonshot Provider 的特点：

1. ✅ **OpenAI 兼容** - 使用标准 OpenAI API 格式
2. ✅ **长上下文** - 支持最高 128k tokens
3. ✅ **统一输出** - 与其他 providers 输出相同的 `StreamingResponse`
4. ✅ **配置驱动** - 使用 ConfigurableProtocol 架构
5. ✅ **易于使用** - 简洁的 API: `LlmClient::moonshot("sk-...")`

**推荐使用场景**：
- 长文本处理和总结
- 多轮对话
- 实时流式输出
- 需要大上下文窗口的应用

---

**相关链接**:
- Moonshot 官网: https://www.moonshot.cn/
- API 文档: https://platform.moonshot.cn/docs
- llm-connector 文档: https://docs.rs/llm-connector

