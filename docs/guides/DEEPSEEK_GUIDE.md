# DeepSeek 使用指南

## 📋 概述

DeepSeek 是一家专注于大语言模型的 AI 公司，提供 OpenAI 兼容的 API 接口，特别支持推理模型（Reasoning Model）。

**官网**: https://www.deepseek.com/  
**API 文档**: https://api-docs.deepseek.com/zh-cn/

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

1. 访问 https://platform.deepseek.com/
2. 注册/登录账号
3. 在控制台创建 API Key
4. API Key 格式: `sk-...`

---

## 💡 基础用法

### 标准对话模型（deepseek-chat）

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端
    let client = LlmClient::deepseek("sk-...")?;
    
    // 创建请求
    let request = ChatRequest {
        model: "deepseek-chat".to_string(),
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

### 推理模型（deepseek-reasoner）

DeepSeek 的推理模型会展示思考过程（reasoning content）：

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::deepseek("sk-...")?;
    
    let request = ChatRequest {
        model: "deepseek-reasoner".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "9.11 和 9.9 哪个更大？".to_string(),
            ..Default::default()
        }],
        max_tokens: Some(500),
        ..Default::default()
    };
    
    let response = client.chat(&request).await?;
    
    // 推理过程（思考过程）
    if let Some(reasoning) = response.reasoning_content {
        println!("🧠 思考过程:\n{}", reasoning);
    }
    
    // 最终答案
    println!("\n💡 最终答案:\n{}", response.content);
    
    Ok(())
}
```

### 流式响应

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::deepseek("sk-...")?;
    
    let request = ChatRequest {
        model: "deepseek-chat".to_string(),
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

| 模型 | 类型 | 说明 |
|------|------|------|
| **deepseek-chat** | 标准对话 | 通用对话模型 |
| **deepseek-reasoner** | 推理模型 | 展示思考过程的推理模型 |

### 模型选择

```rust
// 标准对话模型
let request = ChatRequest {
    model: "deepseek-chat".to_string(),
    // ...
};

// 推理模型（会返回思考过程）
let request = ChatRequest {
    model: "deepseek-reasoner".to_string(),
    // ...
};
```

---

## 🧠 推理模型详解

### 什么是推理模型？

DeepSeek 的推理模型（deepseek-reasoner）会在生成答案前展示其思考过程，类似于 OpenAI 的 o1 模型。

### 推理内容提取

**非流式响应**:
```rust
let response = client.chat(&request).await?;

// 自动提取推理内容
if let Some(reasoning) = response.reasoning_content {
    println!("思考过程: {}", reasoning);
}

// 最终答案
println!("答案: {}", response.content);
```

**流式响应**:
```rust
let mut stream = client.chat_stream(&request).await?;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    
    // 推理内容（思考过程）
    if let Some(reasoning) = chunk.choices.first()
        .and_then(|c| c.delta.reasoning_content.as_ref()) {
        print!("🧠 {}", reasoning);
    }
    
    // 最终答案
    if let Some(content) = chunk.get_content() {
        print!("💡 {}", content);
    }
}
```

### 推理模型示例

```rust
let request = ChatRequest {
    model: "deepseek-reasoner".to_string(),
    messages: vec![Message {
        role: Role::User,
        content: "计算 15 * 23".to_string(),
        ..Default::default()
    }],
    max_tokens: Some(500),
    ..Default::default()
};

let response = client.chat(&request).await?;

// 输出示例：
// 思考过程: "我需要计算 15 乘以 23。让我分步骤来：
//            15 * 20 = 300
//            15 * 3 = 45
//            300 + 45 = 345"
// 答案: "15 * 23 = 345"
```

---

## ⚙️ 高级配置

### 自定义配置

```rust
use llm_connector::LlmClient;

let client = LlmClient::deepseek_with_config(
    "sk-...",           // API key
    None,               // base_url (使用默认)
    Some(60),           // timeout (60秒)
    None                // proxy
)?;
```

### 使用代理

```rust
let client = LlmClient::deepseek_with_config(
    "sk-...",
    None,
    Some(60),
    Some("http://proxy.example.com:8080")  // 代理地址
)?;
```

### 自定义端点

```rust
let client = LlmClient::deepseek_with_config(
    "sk-...",
    Some("https://custom.api.deepseek.com"),  // 自定义端点
    Some(60),
    None
)?;
```

---

## 📊 请求参数

### 常用参数

```rust
let request = ChatRequest {
    model: "deepseek-chat".to_string(),
    messages: vec![/* ... */],
    
    // 可选参数
    temperature: Some(0.7),        // 温度 (0.0-2.0)
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
| `temperature` | f32 | 1.0 | 控制随机性 (0.0-2.0) |
| `top_p` | f32 | 1.0 | 核采样参数 |
| `max_tokens` | u32 | - | 最大生成 tokens |
| `stream` | bool | false | 是否流式响应 |

---

## 🎨 使用场景

### 1. 数学推理

利用推理模型解决数学问题：

```rust
let request = ChatRequest {
    model: "deepseek-reasoner".to_string(),
    messages: vec![Message {
        role: Role::User,
        content: "如果一个数的平方是 144，这个数是多少？".to_string(),
        ..Default::default()
    }],
    max_tokens: Some(500),
    ..Default::default()
};

let response = client.chat(&request).await?;

// 会展示完整的推理过程
if let Some(reasoning) = response.reasoning_content {
    println!("推理过程: {}", reasoning);
}
println!("答案: {}", response.content);
```

### 2. 逻辑推理

```rust
let request = ChatRequest {
    model: "deepseek-reasoner".to_string(),
    messages: vec![Message {
        role: Role::User,
        content: "所有的猫都是动物。Fluffy 是一只猫。那么 Fluffy 是动物吗？".to_string(),
        ..Default::default()
    }],
    ..Default::default()
};
```

### 3. 标准对话

对于不需要推理过程的场景，使用标准模型：

```rust
let request = ChatRequest {
    model: "deepseek-chat".to_string(),
    messages: vec![Message {
        role: Role::User,
        content: "介绍一下 Rust 编程语言".to_string(),
        ..Default::default()
    }],
    ..Default::default()
};
```

### 4. 实时流式推理

```rust
use futures_util::StreamExt;

let request = ChatRequest {
    model: "deepseek-reasoner".to_string(),
    messages: vec![Message {
        role: Role::User,
        content: "解释为什么天空是蓝色的".to_string(),
        ..Default::default()
    }],
    stream: Some(true),
    ..Default::default()
};

let mut stream = client.chat_stream(&request).await?;

println!("🧠 思考过程:");
let mut in_reasoning = true;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    
    // 推理内容
    if let Some(reasoning) = chunk.choices.first()
        .and_then(|c| c.delta.reasoning_content.as_ref()) {
        print!("{}", reasoning);
    }
    
    // 最终答案
    if let Some(content) = chunk.get_content() {
        if in_reasoning {
            println!("\n\n💡 最终答案:");
            in_reasoning = false;
        }
        print!("{}", content);
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
let client = LlmClient::deepseek("sk-...")?;

for _ in 0..10 {
    let response = client.chat(&request).await?;
    // 处理响应
}
```

### 2. 合理设置超时

```rust
// 推理模型可能需要更长时间
let client = LlmClient::deepseek_with_config(
    "sk-...",
    None,
    Some(120),  // 120秒超时
    None
)?;
```

### 3. 使用流式响应

```rust
// 对于推理模型，使用流式可以实时看到思考过程
let request = ChatRequest {
    model: "deepseek-reasoner".to_string(),
    stream: Some(true),  // 启用流式
    // ...
};
```

---

## 🎉 总结

DeepSeek Provider 的特点：

1. ✅ **OpenAI 兼容** - 使用标准 OpenAI API 格式
2. ✅ **推理模型** - 支持展示思考过程的推理模型
3. ✅ **自动提取** - 自动提取 reasoning_content
4. ✅ **统一输出** - 与其他 providers 输出相同的 `StreamingResponse`
5. ✅ **配置驱动** - 使用 ConfigurableProtocol 架构
6. ✅ **易于使用** - 简洁的 API: `LlmClient::deepseek("sk-...")`

**推荐使用场景**：
- 数学推理和计算
- 逻辑推理
- 需要展示思考过程的场景
- 标准对话（使用 deepseek-chat）

**推理模型 vs 标准模型**：
- **deepseek-reasoner**: 适合需要推理的复杂问题，会展示思考过程
- **deepseek-chat**: 适合标准对话，响应更快

---

**相关链接**:
- DeepSeek 官网: https://www.deepseek.com/
- API 文档: https://api-docs.deepseek.com/zh-cn/
- llm-connector 文档: https://docs.rs/llm-connector

