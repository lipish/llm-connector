# LongCat API 测试报告

## 📋 测试信息

- **测试日期**: 2025-10-18
- **API Key**: `ak_11o3bI6O03mx2yS8jb2hD61q7DJ4d`
- **测试端点**:
  - OpenAI 格式: `https://api.longcat.chat/openai`
  - Anthropic 格式: `https://api.longcat.chat/anthropic`

## 🎯 测试结果总结

| 测试项 | OpenAI 格式 | Anthropic 格式 |
|--------|------------|---------------|
| 非流式响应 | ✅ 成功 | ⚠️ 认证问题 |
| 流式响应 | ✅ 成功 | ⚠️ 认证问题 |
| llm-connector 兼容性 | ✅ 完全兼容 | ❌ 需要适配 |

## ✅ OpenAI 格式测试

### 测试 1: 非流式响应

**测试命令**:
```bash
cargo run --example test_longcat_openai
```

**结果**: ✅ 成功

**响应示例**:
```json
{
  "model": "longcat-flash-chatai-api",
  "content": "你好，我是一个乐于助人的AI助手，随时为你解答问题、提供帮助！ 😊",
  "usage": {
    "prompt_tokens": 18,
    "completion_tokens": 19,
    "total_tokens": 37
  },
  "choices": [
    {
      "finish_reason": "stop",
      "message": {
        "role": "assistant",
        "content": "..."
      }
    }
  ]
}
```

**验证点**:
- ✅ 请求成功
- ✅ 返回正确的内容
- ✅ 包含 usage 信息
- ✅ choices 数组不为空
- ✅ finish_reason 正确

### 测试 2: 流式响应

**测试命令**:
```bash
cargo run --example test_longcat_openai --features streaming
```

**结果**: ✅ 成功

**统计**:
- 总流式块数: 29
- 包含内容的块数: 27
- 完整内容长度: 207 字符

**验证点**:
- ✅ 流式请求成功
- ✅ 正确接收所有 chunks
- ✅ 内容完整
- ✅ finish_reason 正确
- ✅ 包含 usage 信息

**流式输出示例**:
```
北京是中国的首都，拥有三千多年建城史和八百多年建都史，是政治、文化、国际交往和科技创新中心，荟萃了故宫、长城等世界文化遗产与现代都市风貌。
```

## ⚠️ Anthropic 格式测试

### 问题描述

**错误信息**:
```
Authentication failed: Anthropic: missing_api_key
```

### 根本原因

LongCat 的 Anthropic 端点使用 **OpenAI 风格的认证**：
```
Authorization: Bearer YOUR_APP_KEY
```

而标准的 Anthropic API 使用：
```
x-api-key: YOUR_API_KEY
```

llm-connector 的 `AnthropicProtocol` 实现了标准 Anthropic 认证方式，因此无法直接用于 LongCat 的 Anthropic 端点。

### 原始 API 测试

**测试命令**:
```bash
./tests/test_longcat_anthropic_raw.sh
```

**结果**: ✅ 成功（使用正确的认证头）

**响应示例**:
```json
{
  "id": "7a0c3b2fbfe043b49fafe006580c6fe4",
  "type": "message",
  "role": "assistant",
  "model": "longcat-flash-chatai-api",
  "content": [
    {
      "type": "text",
      "text": "你好！😊 有什么我可以帮你的吗？✨"
    }
  ],
  "stop_reason": "end_turn",
  "usage": {
    "input_tokens": 12,
    "output_tokens": 12
  }
}
```

**验证点**:
- ✅ API 本身工作正常
- ✅ 返回标准 Anthropic 格式响应
- ✅ 包含 usage 信息
- ✅ 内容正确

## 📊 LongCat API 特点

### 认证方式

**OpenAI 格式**:
```bash
curl -X POST https://api.longcat.chat/openai/v1/chat/completions \
  -H "Authorization: Bearer YOUR_APP_KEY" \
  -H "Content-Type: application/json"
```

**Anthropic 格式**:
```bash
curl -X POST https://api.longcat.chat/anthropic/v1/messages \
  -H "Authorization: Bearer YOUR_APP_KEY" \  # ⚠️ 注意：使用 Bearer 而不是 x-api-key
  -H "Content-Type: application/json" \
  -H "anthropic-version: 2023-06-01"
```

### 支持的模型

- `LongCat-Flash-Chat` - 高性能通用对话模型
- `LongCat-Flash-Thinking` - 深度思考模型

### 限流规则

- 单次请求输出限制: 最大 8K Tokens
- 每日免费额度: 500,000 Tokens
- 可申请提升至: 5,000,000 Tokens/天

## 💡 使用建议

### 推荐方式：使用 OpenAI 格式

由于 LongCat 的两种格式都使用相同的认证方式（`Authorization: Bearer`），建议使用 OpenAI 格式，因为：

1. ✅ llm-connector 完全兼容
2. ✅ 流式和非流式都正常工作
3. ✅ 无需额外适配

**示例代码**:
```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 OpenAI 兼容模式
    let client = LlmClient::openai_compatible(
        "ak_11o3bI6O03mx2yS8jb2hD61q7DJ4d",
        "https://api.longcat.chat/openai",
        "longcat"
    )?;
    
    let request = ChatRequest {
        model: "LongCat-Flash-Chat".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "你好".to_string(),
            ..Default::default()
        }],
        max_tokens: Some(1000),
        ..Default::default()
    };
    
    let response = client.chat(&request).await?;
    println!("Response: {}", response.content);
    
    Ok(())
}
```

### 流式响应示例

```rust
#[cfg(feature = "streaming")]
{
    use futures_util::StreamExt;
    
    let mut stream = client.chat_stream(&request).await?;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(content) = chunk.get_content() {
            print!("{}", content);
        }
    }
}
```

## 🔧 Anthropic 格式适配方案

如果需要使用 LongCat 的 Anthropic 端点，有以下几种方案：

### 方案 1: 创建自定义 Provider（推荐）

创建一个专门的 LongCat Anthropic Provider，使用 `Authorization: Bearer` 认证：

```rust
// 未来可能的实现
let client = LlmClient::longcat_anthropic("ak_...")?;
```

### 方案 2: 使用 OpenAI 格式（当前推荐）

直接使用 OpenAI 格式端点，功能完全相同：

```rust
let client = LlmClient::openai_compatible(
    "ak_...",
    "https://api.longcat.chat/openai",
    "longcat"
)?;
```

### 方案 3: 扩展 AnthropicProtocol

为 `AnthropicProtocol` 添加可配置的认证方式：

```rust
// 可能的未来实现
let protocol = AnthropicProtocol::new_with_auth_type(
    "ak_...",
    AuthType::Bearer  // 而不是默认的 XApiKey
);
```

## 📝 测试文件

### 新增测试文件

1. `examples/test_longcat_openai.rs` - OpenAI 格式测试（非流式 + 流式）
2. `examples/test_longcat_anthropic.rs` - Anthropic 格式测试（认证问题）
3. `tests/test_longcat_anthropic_raw.sh` - Anthropic 原始 API 测试

### 运行测试

```bash
# OpenAI 格式非流式
cargo run --example test_longcat_openai

# OpenAI 格式流式
cargo run --example test_longcat_openai --features streaming

# Anthropic 原始 API
./tests/test_longcat_anthropic_raw.sh
```

## 🎉 总结

### 成功的部分

- ✅ **OpenAI 格式完全可用** - 非流式和流式都正常工作
- ✅ **响应格式正确** - 包含所有必要字段（content, usage, choices）
- ✅ **流式响应稳定** - 正确接收所有 chunks
- ✅ **llm-connector 兼容性好** - 无需修改即可使用

### 需要改进的部分

- ⚠️ **Anthropic 格式认证不兼容** - LongCat 使用 Bearer 认证而不是 x-api-key
- 💡 **建议使用 OpenAI 格式** - 功能完全相同，兼容性更好

### 推荐使用方式

```rust
// ✅ 推荐：使用 OpenAI 格式
let client = LlmClient::openai_compatible(
    "ak_11o3bI6O03mx2yS8jb2hD61q7DJ4d",
    "https://api.longcat.chat/openai",
    "longcat"
)?;
```

---

**测试人**: AI Assistant  
**测试日期**: 2025-10-18  
**llm-connector 版本**: v0.4.17  
**结论**: ✅ LongCat OpenAI 格式完全可用，推荐使用

