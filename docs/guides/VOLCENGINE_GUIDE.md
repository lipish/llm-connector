# 火山引擎（Volcengine）使用指南

## 📋 概述

火山引擎（Volcengine）是字节跳动旗下的云服务平台，提供大模型服务（火山方舟）。

- **官网**: https://www.volcengine.com
- **控制台**: https://console.volcengine.com/ark
- **API 文档**: https://www.volcengine.com/docs/82379

## 🎯 API 特点

### 兼容性

火山引擎使用 **OpenAI 兼容的 API 格式**：
- 端点: `https://ark.cn-beijing.volces.com/api/v3`
- 认证: `Authorization: Bearer YOUR_API_KEY`
- 格式: 与 OpenAI API 完全兼容

### 特殊性

**模型名称使用端点 ID**：
- 不是使用模型名称（如 `gpt-4`）
- 而是使用端点 ID（如 `ep-20250118155555-xxxxx`）
- 端点 ID 在火山引擎控制台创建和获取

## 🔑 获取 API Key 和端点 ID

### 1. 获取 API Key

1. 访问火山引擎控制台: https://console.volcengine.com/ark
2. 进入"API 密钥"页面
3. 创建或复制 API Key
4. 格式: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`（UUID 格式）

### 2. 创建端点

1. 访问端点管理页面: https://console.volcengine.com/ark/region:ark+cn-beijing/endpoint/
2. 点击"创建推理接入点"
3. 选择模型（如 DeepSeek、Doubao 等）
4. 配置参数并创建
5. 获取端点 ID（格式: `ep-xxxxxx`）

### 3. 端点 ID 示例

```
ep-20250118155555-xxxxx
ep-20250119123456-yyyyy
```

## 💻 使用 llm-connector

### 基础用法

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端
    let client = LlmClient::openai_compatible(
        "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",  // API Key
        "https://ark.cn-beijing.volces.com/api/v3",  // 端点
        "volcengine"  // 服务名称
    )?;
    
    // 创建请求
    let request = ChatRequest {
        model: "ep-20250118155555-xxxxx".to_string(),  // 端点 ID
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

### 推理模型支持 (Doubao-Seed-Code)

Volcengine 的 Doubao-Seed-Code 是推理模型，它将推理过程输出到 `reasoning_content` 字段。llm-connector 会自动处理这种情况。

**使用示例**:
```rust
use llm_connector::providers::volcengine_with_config;
use llm_connector::types::{ChatRequest, Message};
use futures_util::StreamExt;

let provider = volcengine_with_config("api-key", None, Some(60), None)?;

let request = ChatRequest {
    model: "ep-20250118155555-xxxxx".to_string(),  // Doubao-Seed-Code 端点
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

**关键点**:
- ✅ 自动识别推理内容字段 (`reasoning_content`)
- ✅ 无需额外配置
- ✅ 与标准模型使用相同的代码
- ✅ 详见 [推理模型支持文档](../REASONING_MODELS_SUPPORT.md)

## 🧪 测试

### 测试非流式响应

```bash
# 设置环境变量
export VOLCENGINE_API_KEY="your-api-key"

# 运行测试（需要修改示例中的端点 ID）
cargo run --example test_volcengine
```

### 测试流式响应

```bash
cargo run --example test_volcengine --features streaming
```

### 测试原始 API

```bash
./tests/test_volcengine_raw.sh
```

## ⚠️ 注意事项

### 1. 端点 ID 必须正确

**错误示例**:
```rust
model: "gpt-4".to_string(),  // ❌ 错误：火山引擎不使用模型名称
```

**正确示例**:
```rust
model: "ep-20250118155555-xxxxx".to_string(),  // ✅ 正确：使用端点 ID
```

### 2. API Key 格式

火山引擎的 API Key 是 UUID 格式：
```
xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

不是 OpenAI 的 `sk-` 格式。

### 3. 端点区域

不同区域有不同的端点：
- 北京: `https://ark.cn-beijing.volces.com/api/v3`
- 其他区域: 查看火山引擎文档

### 4. 权限检查

确保 API Key 有权访问指定的端点 ID：
- 在控制台检查端点状态
- 确认 API Key 有相应权限

## 🔧 常见错误

### 错误 1: InvalidEndpointOrModel.NotFound

**错误信息**:
```json
{
  "error": {
    "code": "InvalidEndpointOrModel.NotFound",
    "message": "The model or endpoint ep-xxx does not exist or you do not have access to it."
  }
}
```

**原因**:
- 端点 ID 不存在
- API Key 无权访问该端点
- 端点 ID 格式错误

**解决**:
1. 在控制台检查端点 ID 是否正确
2. 确认 API Key 有权限
3. 检查端点状态是否正常

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

### 错误 3: Rate Limit Exceeded

**错误信息**:
```json
{
  "error": {
    "code": "RateLimitExceeded",
    "message": "Rate limit exceeded"
  }
}
```

**原因**:
- 超过了 API 调用频率限制

**解决**:
1. 降低请求频率
2. 联系火山引擎提升限额

## 📊 支持的功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 非流式响应 | ✅ | 完全支持 |
| 流式响应 | ✅ | 完全支持 |
| 函数调用 | ✅ | 支持（取决于模型） |
| 视觉理解 | ✅ | 支持（取决于模型） |
| 嵌入 | ✅ | 支持 |

## 🎯 最佳实践

### 1. 环境变量管理

```bash
# .env 文件
VOLCENGINE_API_KEY=your-api-key
VOLCENGINE_ENDPOINT_ID=ep-xxxxxx
```

```rust
use std::env;

let api_key = env::var("VOLCENGINE_API_KEY")?;
let endpoint_id = env::var("VOLCENGINE_ENDPOINT_ID")?;

let client = LlmClient::openai_compatible(
    &api_key,
    "https://ark.cn-beijing.volces.com/api/v3",
    "volcengine"
)?;

let request = ChatRequest {
    model: endpoint_id,
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
        // 检查是否是端点 ID 错误
        if e.to_string().contains("NotFound") {
            eprintln!("提示: 请检查端点 ID 是否正确");
        }
    }
}
```

### 3. 超时设置

```rust
use llm_connector::providers::openai_compatible_with_config;

let provider = openai_compatible_with_config(
    &api_key,
    "https://ark.cn-beijing.volces.com/api/v3",
    "volcengine",
    Some(60),  // 60秒超时
    None
)?;

let client = LlmClient::from_provider(Arc::new(provider));
```

## 📚 参考资源

- **官方文档**: https://www.volcengine.com/docs/82379
- **控制台**: https://console.volcengine.com/ark
- **API 参考**: https://www.volcengine.com/docs/82379/1494384
- **快速入门**: https://www.volcengine.com/docs/82379/1399008

## 🎉 总结

火山引擎使用 OpenAI 兼容的 API 格式，可以通过 `LlmClient::openai_compatible()` 方法轻松接入。

**关键点**:
1. ✅ 使用 OpenAI 兼容格式
2. ✅ 端点: `https://ark.cn-beijing.volces.com/api/v3`
3. ⚠️ 模型名称使用端点 ID（`ep-xxxxxx`）
4. ✅ 支持流式和非流式响应
5. ✅ 完全兼容 llm-connector

---

**文档版本**: v1.0  
**更新日期**: 2025-10-18  
**llm-connector 版本**: v0.4.18+

