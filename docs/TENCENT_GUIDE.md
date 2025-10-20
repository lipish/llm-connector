# 腾讯云混元（Tencent Hunyuan）使用指南

## 📋 概述

腾讯云混元是腾讯公司推出的大语言模型服务，提供强大的对话、创作和理解能力。

- **官网**: https://cloud.tencent.com/product/hunyuan
- **控制台**: https://console.cloud.tencent.com/hunyuan
- **API 文档**: https://cloud.tencent.com/document/product/1729

## 🎯 API 特点

### 兼容性

腾讯云混元使用 **OpenAI 兼容的 API 格式**：
- 端点: `https://api.hunyuan.cloud.tencent.com/v1`
- 认证: `Authorization: Bearer YOUR_API_KEY`
- 格式: 与 OpenAI API 完全兼容

### 可用模型

- **hunyuan-lite**: 轻量级模型，速度快，成本低
- **hunyuan-standard**: 标准模型，平衡性能和成本
- **hunyuan-pro**: 专业模型，性能强大
- **hunyuan-turbo**: 高速模型，响应快

## 🔑 获取 API Key

### 1. 注册腾讯云账号

访问: https://cloud.tencent.com

### 2. 开通混元服务

1. 访问混元控制台: https://console.cloud.tencent.com/hunyuan
2. 点击"立即开通"
3. 同意服务协议

### 3. 创建 API Key

1. 在控制台点击"API 密钥"
2. 点击"新建密钥"
3. 复制生成的 API Key
4. 格式: `sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`

## 💻 使用 llm-connector

### 基础用法

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端
    let client = LlmClient::openai_compatible(
        "sk-YMiR2Q7LNWVKVWKivkfPn49geQXT27OZXumFkSS3Ef6FlQ50",  // API Key
        "https://api.hunyuan.cloud.tencent.com",  // 端点（不包含 /v1）
        "tencent"  // 服务名称
    )?;
    
    // 创建请求
    let request = ChatRequest {
        model: "hunyuan-lite".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "你好".to_string(),
            ..Default::default()
        }],
        max_tokens: Some(1000),
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
#[cfg(feature = "streaming")]
{
    use futures_util::StreamExt;
    
    let mut streaming_request = request.clone();
    streaming_request.stream = Some(true);
    
    let mut stream = client.chat_stream(&streaming_request).await?;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(content) = chunk.get_content() {
            print!("{}", content);
        }
    }
}
```

## 🧪 测试

### 测试非流式响应

```bash
# 设置环境变量
export TENCENT_API_KEY="your-api-key"

# 运行测试
cargo run --example test_tencent
```

### 测试流式响应

```bash
cargo run --example test_tencent --features streaming
```

### 测试原始 API

```bash
./tests/test_tencent_raw.sh
```

## ⚠️ 注意事项

### 1. Base URL 设置

**错误示例**:
```rust
// ❌ 错误：包含 /v1 会导致 404
let client = LlmClient::openai_compatible(
    api_key,
    "https://api.hunyuan.cloud.tencent.com/v1",  // 错误
    "tencent"
)?;
```

**正确示例**:
```rust
// ✅ 正确：不包含 /v1，OpenAI protocol 会自动添加
let client = LlmClient::openai_compatible(
    api_key,
    "https://api.hunyuan.cloud.tencent.com",  // 正确
    "tencent"
)?;
```

### 2. API Key 格式

腾讯云混元的 API Key 格式：
```
sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

与 OpenAI 的格式相同。

### 3. 模型选择

根据需求选择合适的模型：
- **快速响应**: hunyuan-turbo
- **成本优先**: hunyuan-lite
- **性能优先**: hunyuan-pro
- **平衡选择**: hunyuan-standard

### 4. 响应中的 note 字段

腾讯云混元的响应中包含一个 `note` 字段：
```json
{
  "note": "以上内容为AI生成，不代表开发者立场，请勿删除或修改本标记"
}
```

这是腾讯云的合规要求，不影响正常使用。

## 🔧 常见错误

### 错误 1: HTTP 404

**错误信息**:
```
API error: OpenAI HTTP 404:
```

**原因**:
- Base URL 设置错误，包含了 `/v1`

**解决**:
```rust
// 使用正确的 base_url（不包含 /v1）
let client = LlmClient::openai_compatible(
    api_key,
    "https://api.hunyuan.cloud.tencent.com",  // 正确
    "tencent"
)?;
```

### 错误 2: Unauthorized

**错误信息**:
```json
{
  "error": {
    "code": "Unauthorized",
    "message": "Invalid API key"
  }
}
```

**原因**:
- API Key 无效或过期

**解决**:
1. 检查 API Key 是否正确
2. 在控制台重新生成 API Key

### 错误 3: Model Not Found

**错误信息**:
```json
{
  "error": {
    "code": "model_not_found",
    "message": "The model does not exist"
  }
}
```

**原因**:
- 模型名称错误

**解决**:
使用正确的模型名称：
- `hunyuan-lite`
- `hunyuan-standard`
- `hunyuan-pro`
- `hunyuan-turbo`

## 📊 支持的功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 非流式响应 | ✅ | 完全支持 |
| 流式响应 | ✅ | 完全支持 |
| 函数调用 | ✅ | 支持（部分模型） |
| 视觉理解 | ✅ | 支持（部分模型） |
| 嵌入 | ✅ | 支持 |

## 🎯 最佳实践

### 1. 环境变量管理

```bash
# .env 文件
TENCENT_API_KEY=sk-xxxxxx
TENCENT_MODEL=hunyuan-lite
```

```rust
use std::env;

let api_key = env::var("TENCENT_API_KEY")?;
let model = env::var("TENCENT_MODEL").unwrap_or_else(|_| "hunyuan-lite".to_string());

let client = LlmClient::openai_compatible(
    &api_key,
    "https://api.hunyuan.cloud.tencent.com",
    "tencent"
)?;

let request = ChatRequest {
    model,
    // ...
};
```

### 2. 错误处理

```rust
match client.chat(&request).await {
    Ok(response) => {
        println!("Success: {}", response.content);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
        // 检查是否是模型错误
        if e.to_string().contains("model_not_found") {
            eprintln!("提示: 请检查模型名称是否正确");
        }
    }
}
```

### 3. 超时设置

```rust
use llm_connector::providers::openai_compatible_with_config;

let provider = openai_compatible_with_config(
    &api_key,
    "https://api.hunyuan.cloud.tencent.com",
    "tencent",
    Some(60),  // 60秒超时
    None
)?;

let client = LlmClient::from_provider(Arc::new(provider));
```

## 📚 参考资源

- **官方文档**: https://cloud.tencent.com/document/product/1729
- **控制台**: https://console.cloud.tencent.com/hunyuan
- **API 参考**: https://cloud.tencent.com/document/product/1729/111007
- **定价**: https://cloud.tencent.com/document/product/1729/97731

## 🎉 总结

腾讯云混元使用 OpenAI 兼容的 API 格式，可以通过 `LlmClient::openai_compatible()` 方法轻松接入。

**关键点**:
1. ✅ 使用 OpenAI 兼容格式
2. ✅ 端点: `https://api.hunyuan.cloud.tencent.com`（不包含 `/v1`）
3. ✅ 支持流式和非流式响应
4. ✅ 多种模型可选（lite, standard, pro, turbo）
5. ✅ 完全兼容 llm-connector

---

**文档版本**: v1.0  
**更新日期**: 2025-10-18  
**llm-connector 版本**: v0.4.19+

