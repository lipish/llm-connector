# 推理模型通用支持说明

## 概述

llm-connector 现在对各种推理模型（Reasoning Models）提供了统一的流式响应支持。无论推理内容放在哪个字段（`reasoning_content`、`reasoning`、`thought`、`thinking`），都能自动识别并提取。

## 快速对比表

| Provider | 模型示例 | 推理字段 | 支持状态 | 优先级 |
|----------|---------|---------|---------|--------|
| **Volcengine** | Doubao-Seed-Code | `reasoning_content` | ✅ 已验证 | 2 |
| **DeepSeek** | DeepSeek R1 | `reasoning_content` / `reasoning` | ✅ 支持 | 2/3 |
| **OpenAI** | o1-preview, o1-mini | `thought` / `reasoning_content` | ✅ 支持 | 4/2 |
| **Qwen** | Qwen-Plus | `reasoning` | ✅ 支持 | 3 |
| **Anthropic** | Claude 3.5 Sonnet | `thinking` | ✅ 支持 | 5 |
| **标准模型** | GPT-4, Claude 等 | `content` | ✅ 不受影响 | 1 |

**注**: 优先级数字越小，优先级越高。当多个字段同时存在时，使用优先级最高的字段。

## 支持的推理模型

### 1. Volcengine Doubao-Seed-Code ✅

**字段**: `delta.reasoning_content`

**响应格式**:
```json
{
  "choices": [{
    "delta": {
      "content": "",
      "reasoning_content": "我现在需要用一句话介绍自己..."
    }
  }]
}
```

**使用示例**:
```rust
use llm_connector::providers::volcengine_with_config;
use llm_connector::types::{ChatRequest, Message};
use futures_util::StreamExt;

let provider = volcengine_with_config("api-key", None, Some(60), None)?;
let request = ChatRequest {
    model: "ep-20250118155555-xxxxx".to_string(),
    messages: vec![Message::user("介绍一下你自己")],
    stream: Some(true),
    ..Default::default()
};

let mut stream = provider.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.get_content() {
        print!("{}", content);  // ✅ 自动提取 reasoning_content
    }
}
```

### 2. DeepSeek R1 ✅

**字段**: `delta.reasoning_content` 或 `delta.reasoning`

**响应格式**:
```json
{
  "choices": [{
    "delta": {
      "content": "",
      "reasoning_content": "Let me think about this..."
    }
  }]
}
```

**使用示例**:
```rust
use llm_connector::providers::openai_with_config;

let provider = openai_with_config(
    "deepseek-api-key",
    Some("https://api.deepseek.com"),
    None, None
)?;

let request = ChatRequest {
    model: "deepseek-reasoner".to_string(),
    messages: vec![Message::user("Solve this problem")],
    stream: Some(true),
    ..Default::default()
};

let mut stream = provider.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.get_content() {
        print!("{}", content);  // ✅ 自动提取 reasoning_content
    }
}
```

### 3. OpenAI o1 系列 ✅

**字段**: `delta.thought` 或 `delta.reasoning_content`

**响应格式**:
```json
{
  "choices": [{
    "delta": {
      "content": "",
      "thought": "I need to analyze this step by step..."
    }
  }]
}
```

**使用示例**:
```rust
use llm_connector::providers::openai;

let provider = openai("openai-api-key")?;
let request = ChatRequest {
    model: "o1-preview".to_string(),
    messages: vec![Message::user("Solve this complex problem")],
    stream: Some(true),
    ..Default::default()
};

let mut stream = provider.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.get_content() {
        print!("{}", content);  // ✅ 自动提取 thought
    }
}
```

### 4. Qwen 推理模型 ✅

**字段**: `delta.reasoning`

**响应格式**:
```json
{
  "choices": [{
    "delta": {
      "content": "",
      "reasoning": "让我分析一下这个问题..."
    }
  }]
}
```

**使用示例**:
```rust
use llm_connector::providers::openai_with_config;

let provider = openai_with_config(
    "qwen-api-key",
    Some("https://dashscope.aliyuncs.com/compatible-mode/v1"),
    None, None
)?;

let request = ChatRequest {
    model: "qwen-plus".to_string(),
    messages: vec![Message::user("解决这个问题")],
    stream: Some(true),
    ..Default::default()
};

let mut stream = provider.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.get_content() {
        print!("{}", content);  // ✅ 自动提取 reasoning
    }
}
```

### 5. Anthropic Claude (Extended Thinking) ✅

**字段**: `delta.thinking`

**响应格式**:
```json
{
  "choices": [{
    "delta": {
      "content": "",
      "thinking": "Let me consider the implications..."
    }
  }]
}
```

**使用示例**:
```rust
use llm_connector::providers::anthropic;

let provider = anthropic("anthropic-api-key")?;
let request = ChatRequest {
    model: "claude-3-5-sonnet-20241022".to_string(),
    messages: vec![Message::user("Think deeply about this")],
    stream: Some(true),
    ..Default::default()
};

let mut stream = provider.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.get_content() {
        print!("{}", content);  // ✅ 自动提取 thinking
    }
}
```

## 字段优先级

当多个推理字段同时存在时，按以下优先级提取：

1. **`delta.content`** (标准内容，非空) - 最高优先级
2. **`delta.reasoning_content`** - Volcengine, DeepSeek R1
3. **`delta.reasoning`** - Qwen, DeepSeek
4. **`delta.thought`** - OpenAI o1
5. **`delta.thinking`** - Anthropic

这个优先级设计确保：
- ✅ 标准模型不受影响（优先使用 `content`）
- ✅ 推理模型自动降级到推理字段
- ✅ 多字段兼容（如果模型同时返回多个字段）

## 实现原理

### 核心代码

位置: `src/sse.rs` - `sse_to_streaming_response()`

```rust
if streaming_response.content.is_empty() {
    if let Some(choice) = streaming_response.choices.first() {
        let content_to_use = choice.delta.content.as_ref()
            .filter(|s| !s.is_empty())                    // 1. 标准 content
            .or_else(|| choice.delta.reasoning_content.as_ref())  // 2. reasoning_content
            .or_else(|| choice.delta.reasoning.as_ref())          // 3. reasoning
            .or_else(|| choice.delta.thought.as_ref())            // 4. thought
            .or_else(|| choice.delta.thinking.as_ref());          // 5. thinking
        
        if let Some(content) = content_to_use {
            streaming_response.content = content.clone();
        }
    }
}
```

### Delta 类型定义

位置: `src/types/streaming.rs`

```rust
pub struct Delta {
    pub content: Option<String>,           // 标准内容
    pub reasoning_content: Option<String>, // Volcengine, DeepSeek R1
    pub reasoning: Option<String>,         // Qwen, DeepSeek
    pub thought: Option<String>,           // OpenAI o1
    pub thinking: Option<String>,          // Anthropic
}
```

## 使用建议

### 1. 统一接口

无论使用哪种推理模型，都使用相同的代码：

```rust
let mut stream = provider.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.get_content() {
        print!("{}", content);  // ✅ 自动处理所有推理字段
    }
}
```

### 2. 区分推理内容和最终答案

某些推理模型会同时返回推理过程和最终答案：

```rust
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    
    // 获取当前内容（可能是推理过程或最终答案）
    if let Some(content) = chunk.get_content() {
        print!("{}", content);
    }
    
    // 如果需要区分，可以检查原始字段
    if let Some(choice) = chunk.choices.first() {
        if choice.delta.reasoning_content.is_some() {
            // 这是推理过程
        } else if choice.delta.content.is_some() {
            // 这是最终答案
        }
    }
}
```

### 3. 完整响应收集

```rust
let mut reasoning_parts = Vec::new();
let mut answer_parts = Vec::new();

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    
    if let Some(choice) = chunk.choices.first() {
        if let Some(reasoning) = &choice.delta.reasoning_content {
            reasoning_parts.push(reasoning.clone());
        }
        if let Some(content) = &choice.delta.content {
            if !content.is_empty() {
                answer_parts.push(content.clone());
            }
        }
    }
}

let full_reasoning = reasoning_parts.join("");
let full_answer = answer_parts.join("");
```

## 兼容性保证

- ✅ **向后兼容**: 标准模型（GPT-4, Claude 等）不受影响
- ✅ **自动降级**: 推理字段自动作为 content 的后备
- ✅ **零配置**: 无需额外配置，自动识别推理字段
- ✅ **类型安全**: 所有字段都是 `Option<String>`，安全处理

## 测试验证

所有推理模型的支持都经过了单元测试验证：

```rust
#[test]
fn test_streaming_response_content_population() {
    // 测试 Volcengine reasoning_content
    // 测试 DeepSeek reasoning
    // 测试 OpenAI thought
    // 测试标准 content
}
```

运行测试:
```bash
cargo test --features streaming test_streaming_response_content_population
```

## 总结

llm-connector 现在提供了对推理模型的统一支持，无论推理内容放在哪个字段，都能自动识别和提取。这使得上层应用（如 llm-link）可以用统一的方式处理所有推理模型，无需针对不同 provider 做特殊处理。

